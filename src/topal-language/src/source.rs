use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;
use num_rational::BigRational;
use regex::Regex;
use topal_semantics::{LanguageVersion, ObjectKind};
use topal_serialization::{
    Event as SerializedEvent, Header as SerializationHeader, Limits as SerializationLimits,
    SerializedValue, Stream as SerializationStream, StreamByteOrder, TypeDefinition,
    deserialize as deserialize_native, serialize as serialize_native,
};
use topal_source::{
    Diagnostic, SourceText, Span, canonically_equal, case_fold, character_at, character_count,
    characters, lowercase, normalize_nfc, normalize_nfd, uppercase,
};
use topal_syntax::{
    CallableKind, DecisionMatcher, Expression, FunctionParameter, Statement, extract_documentation,
    lex, parse,
};

use crate::{ExecutionSnapshot, TraceEvent, TraceSink};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Type(String),
    Effects(Vec<String>),
    Boolean(bool),
    Version(LanguageVersion),
    NativeSerializer(LanguageVersion),
    SerializationStream(Vec<u8>),
    ObjectDescription {
        identity: String,
        kind: String,
        value: Box<Value>,
    },
    TaskType(Box<TaskTypeValue>),
    TaskDefinition(Box<TaskDefinitionValue>),
    TaskInstance(Box<RefCell<TaskInstanceValue>>),
    SizeBits(BigInt),
    AddressRangeType(Vec<(String, Value)>),
    AddressRange {
        attributes: Vec<(String, Value)>,
        lower: BigInt,
        upper: BigInt,
    },
    AddressOffsetType(Vec<(String, Value)>),
    AddressOffset {
        attributes: Vec<(String, Value)>,
        offset: BigInt,
    },
    LayoutType(Box<LayoutValue>),
    LayoutFactory(Vec<(String, Value)>),
    LayoutBacked {
        layout: Box<LayoutValue>,
        value: Box<Value>,
    },
    LocationType(Box<LayoutValue>),
    Location {
        layout: Box<LayoutValue>,
        offset: Box<Value>,
        storage: Box<RefCell<Option<Value>>>,
    },
    Int(BigInt),
    Rational(BigRational),
    IntRange {
        lower: BigInt,
        upper: BigInt,
        lower_inclusive: bool,
        upper_inclusive: bool,
    },
    RationalRange {
        lower: BigRational,
        upper: BigRational,
        lower_inclusive: bool,
        upper_inclusive: bool,
    },
    Optional {
        payload_classifier: String,
        payload: Option<Box<Self>>,
    },
    List {
        element_classifier: String,
        entries: Vec<Self>,
    },
    Callable(CallableKind),
    NamedFunction(Rc<NamedFunction>),
    Namespace(Rc<NamespaceValue>),
    AnonymousFunction(Rc<AnonymousFunction>),
    Array {
        element_classifier: String,
        entries: Vec<Self>,
    },
    Set {
        element_classifier: String,
        entries: Vec<Self>,
    },
    Bag {
        element_classifier: String,
        entries: Vec<(Self, usize)>,
    },
    Map {
        key_classifier: String,
        value_classifier: String,
        entries: Vec<(Self, Self)>,
    },
    CharacterGenerator {
        generated: Vec<String>,
        origin: String,
    },
    CharacterReturningGenerator {
        generated: Vec<String>,
        returned: String,
        origin: String,
    },
    IterateGenerator {
        current: Box<Self>,
        next: Box<Self>,
        take_while: Option<Box<Self>>,
        classifier: String,
    },
    UnfoldGenerator {
        seed: Box<Self>,
        step: Box<Self>,
    },
    SuspendedGenerator {
        source: Box<SourceText>,
        body: Box<Vec<Statement>>,
        cursor: usize,
        bindings: Box<BTreeMap<String, Self>>,
        scope_state: Box<GeneratorScopeState>,
        pending_yield: Option<Box<Self>>,
        resume_binding: Option<String>,
        returned: Option<Box<Self>>,
        yield_classifier: String,
        return_classifier: String,
        origin: String,
        task_state: Option<BTreeMap<String, Self>>,
        task_owner: Option<String>,
    },
    String(String),
    Tuple(Vec<Self>),
    Record(Vec<(String, Self)>),
    Enum {
        type_name: String,
        alternative: String,
    },
    Union(Box<UnionValue>),
    Constraint(Box<ConstraintValue>),
    Capability(Vec<BTreeSet<String>>),
    Interface(Box<InterfaceValue>),
    Introspection(Box<IntrospectionValue>),
    Refined {
        constraint: String,
        base_classifier: String,
        value: Box<Self>,
    },
    ModularType(Box<ModularType>),
    Modular {
        type_name: String,
        lower: BigInt,
        upper: BigInt,
        value: BigInt,
    },
    ErrorDomain(String),
    Error {
        domain: String,
        code: String,
        line: usize,
        column: usize,
    },
    Continue(Box<Self>),
    Finish(Box<Self>),
    Completed,
    Unit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntrospectionValue {
    Identity {
        kind: ObjectKind,
        canonical: String,
    },
    TypeView {
        form: String,
        identity: String,
    },
    FunctionView {
        identity: String,
        inputs: Vec<String>,
        output: String,
        is_static: bool,
        effects: Vec<String>,
    },
    ScopeView {
        identity: String,
        members: Vec<String>,
    },
    ConstraintView {
        identity: String,
        base: String,
    },
    EffectView {
        identities: Vec<String>,
    },
    ProtocolView {
        identity: String,
        operations: Vec<String>,
    },
    DeclarationView {
        name: Option<String>,
        canonical_path: Option<String>,
        documentation: Option<String>,
        language_version: LanguageVersion,
    },
    LanguageContext {
        language: String,
        version: LanguageVersion,
        features: Vec<String>,
    },
}

impl Value {
    /// Return the shared semantic kind without erasing this value's identity.
    #[must_use]
    pub const fn object_kind(&self) -> ObjectKind {
        match self {
            Self::Type(_) | Self::ModularType(_) => ObjectKind::Type,
            Self::Effects(_) => ObjectKind::Effect,
            Self::Callable(_) | Self::NamedFunction(_) | Self::AnonymousFunction(_) => {
                ObjectKind::Function
            }
            Self::Namespace(_) => ObjectKind::Scope,
            Self::Constraint(_) => ObjectKind::Constraint,
            Self::Capability(_) => ObjectKind::Capability,
            Self::Interface(_) => ObjectKind::Interface,
            _ => ObjectKind::Value,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[allow(clippy::box_collection)] // Keep recursive evaluator state below the tested stack-frame ceiling.
pub struct GeneratorScopeState {
    functions: BTreeMap<String, Vec<UserFunction>>,
    declared_names: BTreeSet<String>,
    local_function_names: BTreeSet<String>,
    enum_types: BTreeMap<String, BTreeSet<String>>,
    union_types: Box<BTreeMap<String, BTreeMap<String, Option<String>>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousFunction {
    source: SourceText,
    parameters: Vec<String>,
    body: Box<Expression>,
    bindings: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedFunction {
    name: String,
    candidates: Vec<UserFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceValue {
    name: String,
    bindings: BTreeMap<String, Value>,
    functions: BTreeMap<String, Vec<UserFunction>>,
    generators: BTreeMap<String, Vec<UserGenerator>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnionValue {
    type_name: String,
    alternative: String,
    payload_classifier: Option<String>,
    payload: Option<Box<Value>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstraintValue {
    name: Option<String>,
    base_classifier: String,
    predicate: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceValue {
    name: String,
    functions: BTreeMap<String, (Vec<String>, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModularType {
    name: Option<String>,
    signed: bool,
    lower: BigInt,
    upper: BigInt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutValue {
    semantic: String,
    attributes: Vec<(String, Value)>,
}

impl fmt::Display for Value {
    #[allow(clippy::too_many_lines)] // Every runtime value keeps an explicit stable source representation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(name) => formatter.write_str(name),
            Self::Interface(interface) => write!(formatter, "<Interface {}>", interface.name),
            Self::Introspection(value) => write!(formatter, "{value}"),
            Self::Capability(alternatives) => {
                let text = alternatives
                    .iter()
                    .map(|conjunction| {
                        conjunction
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" and ")
                    })
                    .collect::<Vec<_>>()
                    .join(" or ");
                formatter.write_str(&text)
            }
            Self::Effects(effects) => write!(formatter, "Effects ({})", effects.join(", ")),
            Self::Boolean(value) => value.fmt(formatter),
            Self::ObjectDescription {
                identity,
                kind,
                value,
            } => {
                write!(formatter, "ObjectDescription ({identity}, {kind}, {value})")
            }
            Self::SizeBits(bits) => write!(formatter, "{bits}[b]"),
            Self::AddressRangeType(_) => formatter.write_str("AddressRange <subtype>"),
            Self::AddressRange { lower, upper, .. } => {
                write!(formatter, "{lower} .. {upper}")
            }
            Self::IntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            } => {
                write!(
                    formatter,
                    "{lower} {} {upper}",
                    range_symbol(*lower_inclusive, *upper_inclusive)
                )
            }
            Self::AddressOffsetType(_) => formatter.write_str("AddressOffset <subtype>"),
            Self::AddressOffset { offset, .. } => offset.fmt(formatter),
            Self::LayoutType(layout) => write!(formatter, "Layout {}", layout.semantic),
            Self::LayoutFactory(_) => formatter.write_str("Layout <subtype constructor>"),
            Self::LayoutBacked { value, .. } => value.fmt(formatter),
            Self::LocationType(layout) => {
                write!(formatter, "Location (Layout {})", layout.semantic)
            }
            Self::Location { layout, offset, .. } => {
                write!(formatter, "Location {} {offset}", layout.semantic)
            }
            Self::Version(value) => value.fmt(formatter),
            Self::NativeSerializer(version) => write!(formatter, "<lang serialize {version}>"),
            Self::SerializationStream(bytes) => {
                write!(formatter, "SerializationStream ( {} bytes )", bytes.len())
            }
            Self::TaskType(task) => write!(
                formatter,
                "Task {}",
                task.name.as_deref().unwrap_or("<specialized>")
            ),
            Self::TaskDefinition(task) => write!(formatter, "<TaskDefinition {}>", task.name),
            Self::TaskInstance(task) => {
                let task = task.borrow();
                write!(
                    formatter,
                    "<Task {} #{}>",
                    task.definition.name, task.identity
                )
            }
            Self::Int(value) => value.fmt(formatter),
            Self::Rational(value) => {
                write!(
                    formatter,
                    "Rational ( {}, {} )",
                    value.numer(),
                    value.denom()
                )
            }
            Self::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            } => write!(
                formatter,
                "Rational ( {}, {} ) {} Rational ( {}, {} )",
                lower.numer(),
                lower.denom(),
                range_symbol(*lower_inclusive, *upper_inclusive),
                upper.numer(),
                upper.denom()
            ),
            Self::Optional {
                payload: Some(value),
                ..
            } => write!(formatter, "Some {value}"),
            Self::Optional { payload: None, .. } => formatter.write_str("None"),
            Self::List { entries, .. } => {
                for entry in entries {
                    write!(formatter, "Entry ( {entry}, ")?;
                }
                formatter.write_str("Empty")?;
                for _ in entries {
                    formatter.write_str(" )")?;
                }
                Ok(())
            }
            Self::Callable(kind) => formatter.write_str(callable_name(*kind)),
            Self::NamedFunction(function) => write!(formatter, "<fn {}>", function.name),
            Self::Namespace(namespace) => write!(formatter, "<namespace {}>", namespace.name),
            Self::AnonymousFunction(function) => {
                write!(formatter, "<anonymous fn/{}>", function.parameters.len())
            }
            Self::Array { entries, .. } => display_collection(formatter, "Array", entries),
            Self::Set { entries, .. } => display_collection(formatter, "Set", entries),
            Self::Bag { entries, .. } => {
                formatter.write_str("Bag (")?;
                for (index, (value, count)) in entries.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "({value}, {count})")?;
                }
                formatter.write_str(")")
            }
            Self::Map { entries, .. } => {
                formatter.write_str("Map (")?;
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "({key}, {value})")?;
                }
                formatter.write_str(")")
            }
            Self::CharacterGenerator { .. } => {
                formatter.write_str("<Generator Character Unit Unit>")
            }
            Self::CharacterReturningGenerator { .. } => {
                formatter.write_str("<Generator Character Unit Character>")
            }
            Self::IterateGenerator { classifier, .. } => {
                write!(formatter, "<Generator {classifier} Unit Unit>")
            }
            Self::UnfoldGenerator { .. } => formatter.write_str("<Generator Value Unit Unit>"),
            Self::SuspendedGenerator {
                yield_classifier,
                return_classifier,
                ..
            } => write!(
                formatter,
                "<Generator {yield_classifier} Unit {return_classifier}>"
            ),
            Self::String(value) => formatter.write_str(&display_string(value)),
            Self::Tuple(items) => {
                formatter.write_str("(")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    item.fmt(formatter)?;
                }
                if items.len() == 1 {
                    formatter.write_str(",")?;
                }
                formatter.write_str(")")
            }
            Self::Record(fields) => {
                formatter.write_str("(")?;
                for (index, (label, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{label} is {value}")?;
                }
                formatter.write_str(")")
            }
            Self::Enum { alternative, .. } => formatter.write_str(alternative),
            Self::Union(union) if union.payload.is_some() => write!(
                formatter,
                "{} {}",
                union.alternative,
                union.payload.as_deref().expect("present payload")
            ),
            Self::Union(union) => formatter.write_str(&union.alternative),
            Self::Constraint(constraint) => write!(
                formatter,
                "<Constraint {}>",
                constraint
                    .name
                    .as_deref()
                    .unwrap_or(&constraint.base_classifier)
            ),
            Self::Refined { value, .. } => write!(formatter, "{value}"),
            Self::ModularType(kind) => write!(
                formatter,
                "<{} {} .. {}>",
                if kind.signed { "ModInt" } else { "ModNat" },
                kind.lower,
                kind.upper
            ),
            Self::Modular {
                type_name, value, ..
            } => write!(formatter, "{type_name} {value}"),
            Self::ErrorDomain(domain) => formatter.write_str(domain),
            Self::Error { domain, code, .. } => {
                write!(formatter, "Error ( domain is {domain}, code is {code} )")
            }
            Self::Continue(value) => write!(formatter, "Continue {value}"),
            Self::Finish(value) => write!(formatter, "Finish {value}"),
            Self::Completed => formatter.write_str("Completed"),
            Self::Unit => formatter.write_str("()"),
        }
    }
}

const fn range_symbol(lower_inclusive: bool, upper_inclusive: bool) -> &'static str {
    match (lower_inclusive, upper_inclusive) {
        (true, false) => "..",
        (false, false) => "<..",
        (true, true) => "..=",
        (false, true) => "<..=",
    }
}

fn bound_contains<T: Ord>(
    value: &T,
    lower: &T,
    upper: &T,
    lower_inclusive: bool,
    upper_inclusive: bool,
) -> bool {
    (if lower_inclusive {
        value >= lower
    } else {
        value > lower
    }) && (if upper_inclusive {
        value <= upper
    } else {
        value < upper
    })
}

fn stricter_lower<T: Ord>(
    left: T,
    left_inclusive: bool,
    right: T,
    right_inclusive: bool,
) -> (T, bool) {
    match left.cmp(&right) {
        Ordering::Greater => (left, left_inclusive),
        Ordering::Less => (right, right_inclusive),
        Ordering::Equal => (left, left_inclusive && right_inclusive),
    }
}

fn stricter_upper<T: Ord>(
    left: T,
    left_inclusive: bool,
    right: T,
    right_inclusive: bool,
) -> (T, bool) {
    match left.cmp(&right) {
        Ordering::Less => (left, left_inclusive),
        Ordering::Greater => (right, right_inclusive),
        Ordering::Equal => (left, left_inclusive && right_inclusive),
    }
}

impl fmt::Display for IntrospectionValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identity { kind, canonical } => {
                write!(
                    formatter,
                    "lang Identity ( kind is {kind:?}, path is {canonical} )"
                )
            }
            Self::TypeView { form, identity } => {
                write!(formatter, "lang {form} ( identity is {identity} )")
            }
            Self::FunctionView {
                identity,
                inputs,
                output,
                is_static,
                effects,
            } => write!(
                formatter,
                "lang FunctionView ( identity is {identity}, inputs is {inputs:?}, output is {output}, static is {is_static}, effects is {effects:?} )"
            ),
            Self::ScopeView { identity, members } => write!(
                formatter,
                "lang ScopeView ( identity is {identity}, members is {members:?} )"
            ),
            Self::ConstraintView { identity, base } => write!(
                formatter,
                "lang ConstraintView ( identity is {identity}, base is {base} )"
            ),
            Self::EffectView { identities } => {
                write!(
                    formatter,
                    "lang EffectView ( identities is {identities:?} )"
                )
            }
            Self::ProtocolView {
                identity,
                operations,
            } => write!(
                formatter,
                "lang ProtocolView ( identity is {identity}, operations is {operations:?} )"
            ),
            Self::DeclarationView {
                name,
                canonical_path,
                documentation,
                language_version,
            } => write!(
                formatter,
                "lang DeclarationView ( name is {name:?}, path is {canonical_path:?}, documentation is {documentation:?}, version is {language_version} )"
            ),
            Self::LanguageContext {
                language,
                version,
                features,
            } => write!(
                formatter,
                "lang LanguageContext ( language is {language}, version is {version}, features is {features:?} )"
            ),
        }
    }
}

fn display_collection(
    formatter: &mut fmt::Formatter<'_>,
    kind: &str,
    entries: &[Value],
) -> fmt::Result {
    write!(formatter, "{kind} (")?;
    for (index, entry) in entries.iter().enumerate() {
        if index != 0 {
            formatter.write_str(", ")?;
        }
        write!(formatter, "{entry}")?;
    }
    formatter.write_str(")")
}

#[derive(Clone, Default)]
#[allow(clippy::box_collection)] // Keep recursive evaluator state below the tested stack-frame ceiling.
pub struct Session {
    bindings: BTreeMap<String, Value>,
    functions: Box<BTreeMap<String, Vec<UserFunction>>>,
    generators: Box<BTreeMap<String, Vec<UserGenerator>>>,
    declared_names: BTreeSet<String>,
    published_names: BTreeSet<String>,
    documentation: Box<BTreeMap<String, String>>,
    language_version: LanguageVersion,
    language_features: BTreeSet<String>,
    declared_libraries: BTreeSet<String>,
    consumed_names: BTreeSet<String>,
    local_function_names: BTreeSet<String>,
    enum_types: BTreeMap<String, BTreeSet<String>>,
    union_types: Box<BTreeMap<String, BTreeMap<String, Option<String>>>>,
    generic_types: BTreeMap<String, String>,
    call_stack: Vec<ActiveCall>,
    static_context: bool,
    task_state: Option<BTreeMap<String, Value>>,
    next_task_identity: Cell<u64>,
    next_transaction_identity: Cell<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserFunction {
    source: SourceText,
    is_static: bool,
    parameters: Vec<(String, String)>,
    parameter_packages: BTreeMap<usize, Vec<UserParameterField>>,
    result: String,
    generic_names: BTreeSet<String>,
    effect_bound: Option<String>,
    body: Vec<Statement>,
    bindings: BTreeMap<String, Value>,
    termination_rule: Option<&'static str>,
    recursion_target: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserParameterField {
    name: String,
    classifier: String,
    default: Option<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskTypeValue {
    name: Option<String>,
    options: Vec<(String, Value)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDefinitionValue {
    name: String,
    task_type: TaskTypeValue,
    source: SourceText,
    state_fields: Vec<(String, String)>,
    handlers: BTreeMap<String, Vec<UserFunction>>,
    streams: BTreeMap<String, Vec<UserGenerator>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskInstanceValue {
    identity: u64,
    definition: TaskDefinitionValue,
    state: BTreeMap<String, Value>,
    terminated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserGenerator {
    source: SourceText,
    parameters: Vec<(String, String)>,
    yielded: String,
    result: String,
    body: Vec<Statement>,
    bindings: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveCall {
    name: String,
    signature: String,
    termination_rule: Option<&'static str>,
    recursion_target: Option<String>,
}

pub struct Execution {
    source: SourceText,
    statements: Vec<Statement>,
    cursor: usize,
    return_classifier: Option<String>,
}

#[derive(Clone, Copy)]
struct FunctionDeclaration<'a> {
    name: Span,
    is_static: bool,
    parameters: &'a [FunctionParameter],
    result: Span,
    effect_bound: Option<Span>,
    body: &'a [Statement],
    span: Span,
}

#[derive(Clone, Copy)]
struct GeneratorDeclaration<'a> {
    name: Span,
    parameters: &'a [FunctionParameter],
    yielded: Span,
    resumed: Span,
    result: Span,
    body: &'a [Statement],
    span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStep {
    Advanced { value: Value, span: Span },
    Complete(Value),
    Returned { value: Value, span: Span },
}

enum BindingOutcome {
    Bound(Value, Span),
    Returned(Value, Span),
}

impl Session {
    fn layout_attributes(
        &self,
        source: &SourceText,
        expression: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Vec<(String, Value)>, Diagnostic> {
        let Value::Record(attributes) = self.evaluate_expression(source, expression, trace)? else {
            return Err(diagnostic(
                source,
                "E-LAYOUT-ATTRIBUTES",
                expression.span(),
                "layout attributes require a labeled record",
            ));
        };
        Ok(attributes)
    }

    #[allow(clippy::too_many_lines)] // Closed layout forms and their diagnostics remain auditable together.
    fn evaluate_layout_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Option<Result<Value, Diagnostic>> {
        let identifier = |expression: &Expression| match expression {
            Expression::Identifier(name) => Some(source.slice(*name)),
            _ => None,
        };
        if let [operation, location] = items
            && identifier(operation) == Some("read")
            && let Some(name) = identifier(location)
            && let Some(Value::Location {
                layout, storage, ..
            }) = self.bindings.get(name)
        {
            if matches!(layout_access(layout), "WriteOnly" | "Reserved") {
                return Some(Err(diagnostic(
                    source,
                    "E-LAYOUT-NOT-READABLE",
                    span,
                    "location layout does not permit reads",
                )));
            }
            let result = storage.borrow().clone().ok_or_else(|| {
                diagnostic(
                    source,
                    "E-LAYOUT-UNINITIALIZED",
                    span,
                    "location has no stored layout value",
                )
            });
            if result.is_ok() {
                trace.record(TraceEvent {
                    event: "location.read",
                    rule: "TOPAL-LOCATION-READ-001",
                    detail: name,
                });
            }
            return Some(result);
        }
        if let [location, operation, value] = items
            && identifier(operation) == Some("write")
            && let Some(name) = identifier(location)
            && let Some(Value::Location {
                layout, storage, ..
            }) = self.bindings.get(name)
        {
            if matches!(layout_access(layout), "ReadOnly" | "Reserved") {
                return Some(Err(diagnostic(
                    source,
                    "E-LAYOUT-NOT-WRITABLE",
                    span,
                    "location layout does not permit writes",
                )));
            }
            let value_span = value.span();
            let result = self
                .evaluate_expression(source, value, trace)
                .and_then(|value| {
                    let stored = coerce_layout_value(source, value_span, layout, value)?;
                    *storage.borrow_mut() = Some(stored);
                    trace.record(TraceEvent {
                        event: "location.written",
                        rule: "TOPAL-LOCATION-WRITE-001",
                        detail: name,
                    });
                    Ok(Value::Unit)
                });
            return Some(result);
        }
        if let [attributes, constructor, semantic @ ..] = items
            && identifier(constructor) == Some("Layout")
            && !semantic.is_empty()
        {
            let result = self
                .layout_attributes(source, attributes, trace)
                .and_then(|attributes| {
                    let semantic_span = Span::new(
                        semantic.first().expect("checked nonempty").span().start,
                        semantic.last().expect("checked nonempty").span().end,
                    );
                    let semantic = source.slice(semantic_span).trim();
                    validate_layout_attributes(source, span, semantic, &attributes)?;
                    trace.record(TraceEvent {
                        event: "layout.constructed",
                        rule: "TOPAL-LAYOUT-CONSTRUCT-001",
                        detail: semantic,
                    });
                    Ok(Value::LayoutType(Box::new(LayoutValue {
                        semantic: semantic.into(),
                        attributes,
                    })))
                });
            return Some(result);
        }
        if let [constructor, semantic] = items
            && identifier(constructor) == Some("Layout")
        {
            if matches!(semantic, Expression::Product { .. }) {
                return Some(
                    self.layout_attributes(source, semantic, trace)
                        .map(Value::LayoutFactory),
                );
            }
            let semantic = source.slice(semantic.span()).trim();
            let attributes = if semantic == "Unit" {
                vec![
                    ("storage-size".into(), Value::SizeBits(BigInt::from(0))),
                    (
                        "encoding".into(),
                        Value::Enum {
                            type_name: "LayoutEncoding".into(),
                            alternative: "Empty".into(),
                        },
                    ),
                ]
            } else {
                return Some(Err(diagnostic(
                    source,
                    "E-LAYOUT-ATTRIBUTES",
                    span,
                    "this semantic type requires explicit layout attributes",
                )));
            };
            return Some(Ok(Value::LayoutType(Box::new(LayoutValue {
                semantic: semantic.into(),
                attributes,
            }))));
        }
        if let [name, semantic @ ..] = items
            && !semantic.is_empty()
            && let Some(name) = identifier(name)
            && let Some(Value::LayoutFactory(attributes)) = self.bindings.get(name)
        {
            let semantic_span = Span::new(
                semantic.first().unwrap().span().start,
                semantic.last().unwrap().span().end,
            );
            let semantic = source.slice(semantic_span).trim();
            return Some(
                validate_layout_attributes(source, span, semantic, attributes).map(|()| {
                    Value::LayoutType(Box::new(LayoutValue {
                        semantic: semantic.into(),
                        attributes: attributes.clone(),
                    }))
                }),
            );
        }
        if let [constructor, attributes] = items
            && matches!(
                identifier(constructor),
                Some("AddressRange" | "AddressOffset")
            )
        {
            let kind = identifier(constructor).unwrap();
            return Some(
                self.layout_attributes(source, attributes, trace)
                    .map(|attributes| {
                        if kind == "AddressRange" {
                            Value::AddressRangeType(attributes)
                        } else {
                            Value::AddressOffsetType(attributes)
                        }
                    }),
            );
        }
        if let [attributes, constructor, argument] = items
            && matches!(
                identifier(constructor),
                Some("AddressRange" | "AddressOffset")
            )
        {
            let kind = identifier(constructor).unwrap();
            return Some(self.layout_attributes(source, attributes, trace).and_then(
                |attributes| {
                    let value = self.evaluate_expression(source, argument, trace)?;
                    if kind == "AddressRange" {
                        match value {
                            Value::IntRange { lower, upper, .. } if lower >= BigInt::from(0) => {
                                Ok(Value::AddressRange {
                                    attributes,
                                    lower,
                                    upper,
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-RANGE",
                                argument.span(),
                                "AddressRange requires a nonnegative Nat range",
                            )),
                        }
                    } else {
                        match value {
                            Value::Int(offset) if offset >= BigInt::from(0) => {
                                validate_address_offset(
                                    source,
                                    argument.span(),
                                    &attributes,
                                    &offset,
                                )?;
                                Ok(Value::AddressOffset { attributes, offset })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-OFFSET",
                                argument.span(),
                                "AddressOffset requires a Nat byte offset",
                            )),
                        }
                    }
                },
            ));
        }
        if let [constructor, layout] = items
            && identifier(constructor) == Some("Location")
        {
            return Some(self.evaluate_expression(source, layout, trace).and_then(
                |value| match value {
                    Value::LayoutType(layout) => Ok(Value::LocationType(layout)),
                    _ => Err(diagnostic(
                        source,
                        "E-LOCATION-LAYOUT",
                        layout.span(),
                        "Location requires an explicit Layout value",
                    )),
                },
            ));
        }
        if let [name, argument] = items
            && let Some(name) = identifier(name)
            && let Some(constructor) = self.bindings.get(name)
        {
            let result = match constructor {
                Value::AddressRangeType(attributes) => Some(
                    self.evaluate_expression(source, argument, trace).and_then(
                        |value| match value {
                            Value::IntRange { lower, upper, .. } if lower >= BigInt::from(0) => {
                                Ok(Value::AddressRange {
                                    attributes: attributes.clone(),
                                    lower,
                                    upper,
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-RANGE",
                                argument.span(),
                                "AddressRange requires a nonnegative Nat range",
                            )),
                        },
                    ),
                ),
                Value::AddressOffsetType(attributes) => Some(
                    self.evaluate_expression(source, argument, trace).and_then(
                        |value| match value {
                            Value::Int(offset) if offset >= BigInt::from(0) => {
                                validate_address_offset(
                                    source,
                                    argument.span(),
                                    attributes,
                                    &offset,
                                )?;
                                Ok(Value::AddressOffset {
                                    attributes: attributes.clone(),
                                    offset,
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-OFFSET",
                                argument.span(),
                                "AddressOffset requires a Nat byte offset",
                            )),
                        },
                    ),
                ),
                Value::LocationType(layout) => Some(
                    self.evaluate_expression(source, argument, trace).and_then(
                        |offset| match offset {
                            Value::AddressOffset { attributes, offset } => {
                                validate_location_fit(
                                    source,
                                    argument.span(),
                                    layout,
                                    &attributes,
                                    &offset,
                                )?;
                                Ok(Value::Location {
                                    layout: layout.clone(),
                                    offset: Box::new(Value::AddressOffset { attributes, offset }),
                                    storage: Box::new(RefCell::new(None)),
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-LOCATION-OFFSET",
                                argument.span(),
                                "a location requires an AddressOffset value",
                            )),
                        },
                    ),
                ),
                Value::LayoutType(layout) => Some(
                    self.evaluate_expression(source, argument, trace)
                        .and_then(|value| {
                            coerce_layout_value(source, argument.span(), layout, value)
                        }),
                ),
                _ => None,
            };
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn construct_task_instance(
        &self,
        source: &SourceText,
        definition_name: Span,
        argument: &Expression,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Some(Value::TaskDefinition(definition)) =
            self.bindings.get(source.slice(definition_name)).cloned()
        else {
            unreachable!("task definition was checked before construction");
        };
        let start = definition
            .handlers
            .get("start")
            .and_then(|handlers| handlers.first())
            .cloned()
            .expect("task definitions require start");
        let argument = self.evaluate_expression(source, argument, trace)?;
        if !function_accepts(&start.parameters, &argument) {
            return Err(diagnostic(
                source,
                "E-TASK-START-ARGUMENT",
                span,
                "task construction arguments do not match its start handler",
            ));
        }
        let state = definition
            .state_fields
            .iter()
            .map(|(name, _)| (name.clone(), Value::Unit))
            .collect();
        let (start_result, state) =
            self.invoke_task_handler(&definition, &start, argument, state, trace)?;
        if matches!(start_result, Value::Error { .. }) {
            trace.record(TraceEvent {
                event: "task.start.failed",
                rule: "TOPAL-TASK-LIFECYCLE-001",
                detail: &definition.name,
            });
            return Ok(start_result);
        }
        for (name, classifier) in &definition.state_fields {
            if !state
                .get(name)
                .is_some_and(|value| value_has_classifier(value, classifier))
            {
                return Err(diagnostic(
                    source,
                    "E-TASK-STATE-INITIALIZATION",
                    span,
                    format!("start did not initialize `{name}` as `{classifier}`"),
                ));
            }
        }
        let identity = self.next_task_identity.get();
        self.next_task_identity.set(identity + 1);
        let value = Value::TaskInstance(Box::new(RefCell::new(TaskInstanceValue {
            identity,
            definition: *definition,
            state,
            terminated: false,
        })));
        trace.record(TraceEvent {
            event: "task.started",
            rule: "TOPAL-TASK-LIFECYCLE-001",
            detail: &identity.to_string(),
        });
        Ok(value)
    }

    #[allow(clippy::too_many_lines)] // One message transaction keeps its stable trace and state transition together.
    fn evaluate_task_message(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(instance_name),
            Expression::Identifier(operation),
            payload,
        ] = items
        else {
            return Err(diagnostic(
                source,
                "E-TASK-MESSAGE-SHAPE",
                span,
                "task messages require `task operation payload`",
            ));
        };
        let instance_key = source.slice(*instance_name).to_owned();
        let Some(Value::TaskInstance(instance)) = self.bindings.get(&instance_key) else {
            unreachable!("task instance was checked before message execution");
        };
        let instance_snapshot = instance.borrow().clone();
        let operation_name = source.slice(*operation);
        if operation_name == "start" {
            return Err(diagnostic(
                source,
                "E-TASK-START-PRIVATE",
                *operation,
                "start is a lifecycle handler and cannot receive a message",
            ));
        }
        if instance_snapshot
            .definition
            .streams
            .contains_key(operation_name)
        {
            return self.construct_task_stream(
                source,
                &instance_key,
                operation_name,
                payload,
                span,
                trace,
            );
        }
        let handler = instance_snapshot
            .definition
            .handlers
            .get(operation_name)
            .and_then(|handlers| handlers.first())
            .cloned()
            .ok_or_else(|| {
                diagnostic(
                    source,
                    "E-TASK-HANDLER",
                    *operation,
                    "task capability exposes no such message handler",
                )
            })?;
        if instance_snapshot.terminated {
            if handler.result == "Unit" {
                trace.record(TraceEvent {
                    event: "message.discarded.terminated",
                    rule: "TOPAL-TASK-LIFECYCLE-001",
                    detail: operation_name,
                });
                return Ok(Value::Unit);
            }
            let position = source.position(operation.start);
            return Ok(Value::Error {
                domain: "lang task".into(),
                code: "task-terminated".into(),
                line: position.line,
                column: position.column,
            });
        }
        let payload = self.evaluate_expression(source, payload, trace)?;
        if operation_name == "terminate" {
            if !function_accepts(&handler.parameters, &payload) {
                return Err(diagnostic(
                    source,
                    "E-TASK-TERMINATE-ARGUMENT",
                    span,
                    "termination reason does not match the lifecycle handler",
                ));
            }
            let (result, state) = self.invoke_task_handler(
                &instance_snapshot.definition,
                &handler,
                payload,
                instance_snapshot.state.clone(),
                trace,
            )?;
            let mut updated = instance_snapshot;
            updated.state = state;
            updated.terminated = true;
            *instance.borrow_mut() = updated;
            trace.record(TraceEvent {
                event: "task.terminated",
                rule: "TOPAL-TASK-LIFECYCLE-001",
                detail: &instance_key,
            });
            return Ok(result);
        }
        let context = Value::Record(vec![
            (
                "session-id".into(),
                Value::Int(BigInt::from(self.next_transaction_identity.get())),
            ),
            ("sender".into(), Value::String("root".into())),
        ]);
        let argument = match handler.parameters.len() {
            1 => context,
            2 => Value::Tuple(vec![context, payload]),
            _ => {
                return Err(diagnostic(
                    source,
                    "E-TASK-HANDLER-SHAPE",
                    *operation,
                    "message handler requires MessageContext plus zero or one ordinary operand",
                ));
            }
        };
        let transaction = self.next_transaction_identity.get();
        self.next_transaction_identity.set(transaction + 1);
        let detail = format!(
            "transaction={transaction};task={};operation={operation_name}",
            instance_snapshot.identity
        );
        trace.record(TraceEvent {
            event: "message.sent",
            rule: "TOPAL-CONC-INTERACT-001",
            detail: &detail,
        });
        trace.record(TraceEvent {
            event: "message.received",
            rule: "TOPAL-DEBUG-MESSAGE-001",
            detail: &detail,
        });
        let (result, state) = self.invoke_task_handler(
            &instance_snapshot.definition,
            &handler,
            argument,
            instance_snapshot.state.clone(),
            trace,
        )?;
        for (name, classifier) in &instance_snapshot.definition.state_fields {
            if !state
                .get(name)
                .is_some_and(|value| value_has_classifier(value, classifier))
            {
                return Err(diagnostic(
                    source,
                    "E-TASK-STATE-REPLACEMENT",
                    span,
                    format!("message left `{name}` outside `{classifier}`"),
                ));
            }
        }
        let mut updated = instance_snapshot;
        updated.state = state;
        *instance.borrow_mut() = updated;
        trace.record(TraceEvent {
            event: "message.completed",
            rule: "TOPAL-CONC-ORDER-001",
            detail: &detail,
        });
        Ok(result)
    }

    #[allow(clippy::too_many_lines)] // Initial delivery, suspension state, and transaction evidence remain together.
    fn construct_task_stream(
        &self,
        source: &SourceText,
        instance_name: &str,
        operation: &str,
        payload: &Expression,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Some(Value::TaskInstance(instance)) = self.bindings.get(instance_name) else {
            unreachable!("task stream owner was resolved before dispatch")
        };
        let snapshot = instance.borrow().clone();
        if snapshot.terminated {
            let position = source.position(span.start);
            return Ok(Value::Error {
                domain: "lang task".into(),
                code: "task-terminated".into(),
                line: position.line,
                column: position.column,
            });
        }
        let payload = self.evaluate_expression(source, payload, trace)?;
        let context = Value::Record(vec![
            (
                "session-id".into(),
                Value::Int(BigInt::from(self.next_transaction_identity.get())),
            ),
            ("sender".into(), Value::String("root".into())),
        ]);
        let candidates = &snapshot.definition.streams[operation];
        let argument = if candidates
            .iter()
            .any(|candidate| candidate.parameters.len() == 2)
        {
            Value::Tuple(vec![context, payload])
        } else {
            context
        };
        let generator = candidates
            .iter()
            .find(|candidate| function_accepts(&candidate.parameters, &argument))
            .cloned()
            .ok_or_else(|| {
                diagnostic(
                    source,
                    "E-TASK-STREAM-ARGUMENT",
                    span,
                    "no stream handler accepts this payload",
                )
            })?;
        let transaction = self.next_transaction_identity.get();
        self.next_transaction_identity.set(transaction + 1);
        let detail = format!(
            "transaction={transaction};task={};operation={operation}",
            snapshot.identity
        );
        trace.record(TraceEvent {
            event: "message.sent",
            rule: "TOPAL-CONC-INTERACT-001",
            detail: &detail,
        });
        trace.record(TraceEvent {
            event: "message.received",
            rule: "TOPAL-DEBUG-MESSAGE-001",
            detail: &detail,
        });
        let mut scope = Self {
            bindings: generator.bindings.clone(),
            functions: Box::new(snapshot.definition.handlers.clone()),
            generators: Box::new(snapshot.definition.streams.clone()),
            declared_names: BTreeSet::new(),
            published_names: BTreeSet::new(),
            documentation: self.documentation.clone(),
            language_version: self.language_version,
            language_features: self.language_features.clone(),
            declared_libraries: self.declared_libraries.clone(),
            consumed_names: BTreeSet::new(),
            local_function_names: BTreeSet::new(),
            enum_types: self.enum_types.clone(),
            union_types: self.union_types.clone(),
            generic_types: self.generic_types.clone(),
            call_stack: Vec::new(),
            static_context: false,
            task_state: Some(snapshot.state),
            next_task_identity: Cell::new(self.next_task_identity.get()),
            next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
        };
        bind_generator_arguments(&mut scope, &generator.parameters, argument, trace);
        let mut cursor = 0;
        let mut pending_yield = None;
        let mut resume_binding = None;
        let mut returned = None;
        advance_custom_generator(
            &generator.source,
            &generator.body,
            &mut cursor,
            &mut scope,
            &mut pending_yield,
            &mut resume_binding,
            &mut returned,
            &generator.yielded,
            &generator.result,
            operation,
            trace,
        )?;
        sync_stream_task_state(self, Some(instance_name), scope.task_state.as_ref());
        trace.record(TraceEvent {
            event: "message.stream.started",
            rule: "TOPAL-TASK-MESSAGE-001",
            detail: &detail,
        });
        Ok(Value::SuspendedGenerator {
            source: Box::new(generator.source),
            body: Box::new(generator.body),
            cursor,
            bindings: Box::new(scope.bindings),
            scope_state: Box::new(GeneratorScopeState {
                functions: *scope.functions,
                declared_names: scope.declared_names,
                local_function_names: scope.local_function_names,
                enum_types: scope.enum_types,
                union_types: scope.union_types,
            }),
            pending_yield,
            resume_binding,
            returned: returned.map(Box::new),
            yield_classifier: generator.yielded,
            return_classifier: generator.result,
            origin: format!("task.{operation}.transaction-{transaction}"),
            task_state: scope.task_state,
            task_owner: Some(instance_name.into()),
        })
    }

    fn invoke_task_handler(
        &self,
        definition: &TaskDefinitionValue,
        function: &UserFunction,
        argument: Value,
        state: BTreeMap<String, Value>,
        trace: &mut impl TraceSink,
    ) -> Result<(Value, BTreeMap<String, Value>), Diagnostic> {
        let mut scope = Self {
            bindings: function.bindings.clone(),
            functions: Box::new(definition.handlers.clone()),
            generators: self.generators.clone(),
            declared_names: BTreeSet::new(),
            published_names: BTreeSet::new(),
            documentation: self.documentation.clone(),
            language_version: self.language_version,
            language_features: self.language_features.clone(),
            declared_libraries: self.declared_libraries.clone(),
            consumed_names: BTreeSet::new(),
            local_function_names: BTreeSet::new(),
            enum_types: self.enum_types.clone(),
            union_types: self.union_types.clone(),
            generic_types: self.generic_types.clone(),
            call_stack: vec![ActiveCall {
                name: definition.name.clone(),
                signature: function_signature(&definition.name, function),
                termination_rule: None,
                recursion_target: None,
            }],
            static_context: false,
            task_state: Some(state),
            next_task_identity: Cell::new(self.next_task_identity.get()),
            next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
        };
        bind_function_arguments(
            &mut scope,
            function,
            argument,
            trace,
            "TOPAL-FUNCTION-ORDINARY-001",
        )?;
        let mut execution = Execution {
            source: definition.source.clone(),
            statements: function.body.clone(),
            cursor: 0,
            return_classifier: Some(function.result.clone()),
        };
        let value = loop {
            match execution.step(&mut scope, trace)? {
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Complete(value) | ExecutionStep::Returned { value, .. } => {
                    break value;
                }
            }
        };
        if !value_has_classifier(&value, &function.result) {
            return Err(diagnostic(
                &definition.source,
                "E-TASK-HANDLER-RESULT",
                statement_span(function.body.last().expect("handler body is nonempty")),
                format!(
                    "task handler returned a value outside `{}`",
                    function.result
                ),
            ));
        }
        Ok((value, scope.task_state.unwrap_or_default()))
    }

    fn is_lang_operation(source: &SourceText, expression: &Expression, expected: &str) -> bool {
        matches!(
            expression,
            Expression::Application { items, .. }
                if matches!(items.as_slice(),
                    [Expression::Identifier(lang), Expression::Identifier(operation)]
                        if source.slice(*lang) == "lang" && source.slice(*operation) == expected)
        )
    }

    fn is_native_serialization(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items,
            [Expression::Identifier(lang), Expression::Identifier(operation), _]
                if source.slice(*lang) == "lang" && source.slice(*operation) == "deserialize")
            || matches!(items,
                [Expression::Identifier(lang), Expression::Identifier(version), operation]
                    if source.slice(*lang) == "lang" && source.slice(*version) == "version"
                        && Self::is_lang_operation(source, operation, "serialize"))
            || matches!(items, [_, operation, _] if Self::is_lang_operation(source, operation, "serialize"))
            || matches!(items,
                [Expression::Identifier(name), _]
                    if matches!(self.bindings.get(source.slice(*name)), Some(Value::NativeSerializer(_))))
    }

    #[allow(clippy::too_many_lines)] // Protocol boundary diagnostics stay beside each source form.
    fn evaluate_native_serialization(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [
            Expression::Identifier(lang),
            Expression::Identifier(operation),
            stream,
        ] = items
            && source.slice(*lang) == "lang"
            && source.slice(*operation) == "deserialize"
        {
            let Value::SerializationStream(bytes) =
                self.evaluate_expression(source, stream, trace)?
            else {
                return Err(diagnostic(
                    source,
                    "E-DESERIALIZE-OPERAND",
                    stream.span(),
                    "lang deserialize requires a native SerializationStream",
                ));
            };
            let decoded =
                deserialize_native(&bytes, SerializationLimits::default()).map_err(|error| {
                    diagnostic(
                        source,
                        "E-DESERIALIZATION",
                        span,
                        format!(
                            "native stream rejected at {} byte {}: {}",
                            error.stage, error.offset, error.message
                        ),
                    )
                })?;
            let event = decoded.events.first().ok_or_else(|| {
                diagnostic(
                    source,
                    "E-DESERIALIZATION",
                    span,
                    "native stream contains no value event",
                )
            })?;
            let value = value_from_serialized(event, &decoded.types).ok_or_else(|| diagnostic(source, "E-DESERIALIZATION-OBJECT", span, "native value is understood but cannot be reconstructed by this interpreter revision"))?;
            trace.record(TraceEvent {
                event: "serialization.deserialized",
                rule: "TOPAL-SER-DESER-001",
                detail: "validated native event",
            });
            return Ok(value);
        }

        let (version, subject) = match items {
            [
                Expression::Identifier(lang),
                Expression::Identifier(version),
                operation,
            ] if source.slice(*lang) == "lang"
                && source.slice(*version) == "version"
                && Self::is_lang_operation(source, operation, "serialize") =>
            {
                return Ok(Value::NativeSerializer(self.language_version));
            }
            [version, operation, subject]
                if Self::is_lang_operation(source, operation, "serialize") =>
            {
                let Value::Version(version) = self.evaluate_expression(source, version, trace)?
                else {
                    return Err(diagnostic(
                        source,
                        "E-SERIALIZATION-VERSION",
                        version.span(),
                        "the left operand of lang serialize must be a Version",
                    ));
                };
                (version, subject)
            }
            [Expression::Identifier(name), subject] => {
                let Some(Value::NativeSerializer(version)) = self.bindings.get(source.slice(*name))
                else {
                    return Err(diagnostic(
                        source,
                        "E-SERIALIZATION-OPERATION",
                        span,
                        "expected a native serialization operation",
                    ));
                };
                (*version, subject)
            }
            _ => {
                return Err(diagnostic(
                    source,
                    "E-SERIALIZATION-OPERATION",
                    span,
                    "expected `version (lang serialize) value`",
                ));
            }
        };
        let value = self.evaluate_expression(source, subject, trace)?;
        let stream = stream_for_value(version, &value).map_err(|message| {
            diagnostic(source, "E-SERIALIZATION-VALUE", subject.span(), message)
        })?;
        let bytes = serialize_native(&stream)
            .map_err(|error| diagnostic(source, "E-SERIALIZATION", span, error.message))?;
        trace.record(TraceEvent {
            event: "serialization.serialized",
            rule: "TOPAL-SER-CANON-001",
            detail: &format!("{} bytes", bytes.len()),
        });
        Ok(Value::SerializationStream(bytes))
    }

    fn is_lang_introspection(source: &SourceText, items: &[Expression]) -> bool {
        let qualified_prefix = matches!(
            (items.first(), items.get(1)),
            (Some(Expression::Identifier(lang)), Some(Expression::Identifier(operation)))
                if source.slice(*lang) == "lang"
                    && matches!(source.slice(*operation),
                        "context" | "version" | "lint" | "identity" | "view" | "declaration" | "public-members")
        );
        let qualified_infix = matches!(
            (items.get(1), items.get(2)),
            (Some(Expression::Identifier(lang)), Some(Expression::Identifier(operation)))
                if source.slice(*lang) == "lang"
                    && matches!(source.slice(*operation),
                        "same-object" | "equivalent-type" | "compatible-with" | "same-layout")
        );
        qualified_prefix || qualified_infix
    }

    #[allow(clippy::too_many_lines)] // Qualified static operations remain explicit and auditable.
    fn evaluate_lang_introspection(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [
            Expression::Identifier(lang),
            Expression::Identifier(operation),
        ] = items
            && source.slice(*lang) == "lang"
        {
            return match source.slice(*operation) {
                "context" => {
                    trace.record(TraceEvent {
                        event: "introspection.context.viewed",
                        rule: "TOPAL-SYN-CONTEXT-001",
                        detail: &self.language_version.to_string(),
                    });
                    Ok(Value::Introspection(Box::new(
                        IntrospectionValue::LanguageContext {
                            language: "topal".into(),
                            version: self.language_version,
                            features: self.language_features.iter().cloned().collect(),
                        },
                    )))
                }
                "version" => Ok(Value::Version(self.language_version)),
                "lint" => {
                    if !self.language_features.contains("lint") {
                        return Err(diagnostic(
                            source,
                            "E-LINT-VARIANT",
                            *operation,
                            "the `lang lint` namespace requires the `lint` language feature",
                        ));
                    }
                    trace.record(TraceEvent {
                        event: "lint.context.viewed",
                        rule: "TOPAL-SYN-CONTEXT-001",
                        detail: "lang lint",
                    });
                    Ok(Value::Namespace(Rc::new(NamespaceValue {
                        name: "lang lint".into(),
                        bindings: BTreeMap::new(),
                        functions: BTreeMap::new(),
                        generators: BTreeMap::new(),
                    })))
                }
                _ => Err(diagnostic(
                    source,
                    "E-INTROSPECTION-OPERATION",
                    *operation,
                    "unknown qualified static introspection operation",
                )),
            };
        }
        if let [
            Expression::Identifier(lang),
            Expression::Identifier(operation),
            subject,
        ] = items
            && source.slice(*lang) == "lang"
        {
            let subject_span = subject.span();
            let value = self.evaluate_expression(source, subject, trace)?;
            let result = match source.slice(*operation) {
                "identity" => Value::Introspection(Box::new(IntrospectionValue::Identity {
                    kind: value.object_kind(),
                    canonical: introspection_identity(&value).ok_or_else(|| {
                        diagnostic(
                            source,
                            "E-STATIC-INTROSPECTION-SUBJECT",
                            subject_span,
                            "lang identity requires a statically known language object",
                        )
                    })?,
                })),
                "view" => introspection_view(source, value, subject_span)?,
                "declaration" => {
                    let name = match subject {
                        Expression::Identifier(name) => Some(source.slice(*name).to_owned()),
                        _ => None,
                    };
                    Value::Introspection(Box::new(IntrospectionValue::DeclarationView {
                        canonical_path: name.as_ref().map(|name| format!("root.{name}")),
                        documentation: name
                            .as_ref()
                            .and_then(|name| self.documentation.get(name).cloned()),
                        name,
                        language_version: self.language_version,
                    }))
                }
                "public-members" => {
                    let Value::Namespace(namespace) = value else {
                        return Err(diagnostic(
                            source,
                            "E-INTROSPECTION-KIND",
                            subject_span,
                            "lang public-members requires a visible Scope",
                        ));
                    };
                    let mut members = if namespace.name == "root" {
                        self.published_names.iter().cloned().collect::<Vec<_>>()
                    } else {
                        namespace
                            .bindings
                            .keys()
                            .chain(namespace.functions.keys())
                            .chain(namespace.generators.keys())
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    members.sort();
                    members.dedup();
                    Value::Introspection(Box::new(IntrospectionValue::ScopeView {
                        identity: namespace.name.clone(),
                        members,
                    }))
                }
                _ => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-OPERATION",
                        *operation,
                        "unknown qualified static introspection operation",
                    ));
                }
            };
            trace.record(TraceEvent {
                event: "introspection.object.viewed",
                rule: "TOPAL-TYPE-KIND-001",
                detail: source.slice(*operation),
            });
            return Ok(result);
        }
        if let [
            left,
            Expression::Identifier(lang),
            Expression::Identifier(operation),
            right,
        ] = items
            && source.slice(*lang) == "lang"
        {
            let left = self.evaluate_expression(source, left, trace)?;
            let right = self.evaluate_expression(source, right, trace)?;
            let relation = source.slice(*operation);
            let result = match relation {
                "same-object" => {
                    let left = introspection_identity(&left);
                    let right = introspection_identity(&right);
                    match (left, right) {
                        (Some(left), Some(right)) => left == right,
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-STATIC-INTROSPECTION-SUBJECT",
                                span,
                                "lang same-object requires statically known language objects",
                            ));
                        }
                    }
                }
                "equivalent-type" | "compatible-with"
                    if matches!(left, Value::Type(_) | Value::ModularType(_))
                        && matches!(right, Value::Type(_) | Value::ModularType(_)) =>
                {
                    introspection_identity(&left) == introspection_identity(&right)
                }
                "equivalent-type" | "compatible-with" => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-KIND",
                        span,
                        "this relation requires two statically known Type objects",
                    ));
                }
                "same-layout" => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-KIND",
                        span,
                        "lang same-layout requires two explicit Layout values",
                    ));
                }
                _ => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-RELATION",
                        *operation,
                        "unknown qualified introspection relation",
                    ));
                }
            };
            trace.record(TraceEvent {
                event: "introspection.relation.compared",
                rule: "TOPAL-TYPE-ID-001",
                detail: relation,
            });
            return Ok(Value::Boolean(result));
        }
        Err(diagnostic(
            source,
            "E-INTROSPECTION-SYNTAX",
            span,
            "qualified introspection requires `lang operation subject`",
        ))
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an interactive evaluation context for a supported language version.
    ///
    /// # Errors
    ///
    /// Returns an error when this tool does not implement `language_version`.
    pub fn for_language_version(language_version: LanguageVersion) -> Result<Self, &'static str> {
        if language_version != LanguageVersion::DESIGN_0 {
            return Err("the highest language version supported by this tool is v0.1");
        }
        Ok(Self {
            language_version,
            ..Self::default()
        })
    }

    /// The highest source language version implemented by this evaluator.
    #[must_use]
    pub const fn highest_supported_language_version() -> LanguageVersion {
        LanguageVersion::DESIGN_0
    }

    /// Evaluate one source file as an isolated module and bind its published
    /// interface under `name` in this session.
    ///
    /// # Errors
    ///
    /// Returns a source, semantic, or duplicate-module diagnostic.
    ///
    /// # Panics
    ///
    /// Panics only if an already accepted Rust string cannot be represented by
    /// the shared source layer while rendering a duplicate-name diagnostic.
    pub fn load_module(
        &mut self,
        name: &str,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if self.declared_names.contains(name) {
            let source = SourceText::new(input).expect("module input was accepted as UTF-8");
            return Err(diagnostic(
                &source,
                "E-DUPLICATE-MODULE",
                Span::new(0, 0),
                format!("module `{name}` is already declared"),
            ));
        }
        let mut module = Self::new();
        module.evaluate_source_file(input, trace)?;
        self.attach_module(name, module, trace)
    }

    /// Attach an already evaluated child scope under one canonical component.
    ///
    /// # Errors
    ///
    /// Returns a duplicate-module diagnostic when `name` is already declared.
    pub fn attach_module(
        &mut self,
        name: &str,
        module: Self,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if self.declared_names.contains(name) {
            return Err(Diagnostic::error(
                "E-DUPLICATE-MODULE",
                1,
                1,
                format!("module `{name}` is already declared"),
            ));
        }
        let namespace = module.into_published_namespace(name);
        self.bindings.insert(name.to_owned(), namespace.clone());
        self.declared_names.insert(name.to_owned());
        self.published_names.insert(name.to_owned());
        trace.record(TraceEvent {
            event: "module.loaded",
            rule: "TOPAL-NAMESPACE-USE-001",
            detail: name,
        });
        Ok(namespace)
    }

    fn into_published_namespace(self, name: &str) -> Value {
        let bindings = self
            .bindings
            .into_iter()
            .filter(|(member, _)| self.published_names.contains(member))
            .collect();
        let functions = self
            .functions
            .into_iter()
            .filter(|(member, _)| self.published_names.contains(member))
            .collect();
        let generators = self
            .generators
            .into_iter()
            .filter(|(member, _)| self.published_names.contains(member))
            .collect();
        Value::Namespace(Rc::new(NamespaceValue {
            name: name.to_owned(),
            bindings,
            functions,
            generators,
        }))
    }

    /// Report whether a complete block statement should await a dedented line
    /// before an interactive session submits it.
    #[must_use]
    pub fn awaits_dedent(input: &str) -> bool {
        let Ok(source) = SourceText::new(input) else {
            return false;
        };
        let parsed = parse(&source, &lex(&source));
        parsed.diagnostics.is_empty()
            && matches!(
                parsed.statements.as_slice(),
                [Statement::Function { .. }
                    | Statement::Generator { .. }
                    | Statement::Union { .. }
                    | Statement::Interface { .. }
                    | Statement::Foreach { .. }]
            )
    }

    /// Evaluate one source unit and return its final value.
    ///
    /// # Errors
    ///
    /// Returns a source, syntax, name-resolution, or evaluation diagnostic.
    pub fn evaluate(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut execution = self.prepare(input, trace)?;
        loop {
            match execution.step(self, trace)? {
                ExecutionStep::Complete(value) => return Ok(value),
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Returned { .. } => {
                    unreachable!("top-level return is rejected before completing a step")
                }
            }
        }
    }

    /// Evaluate a complete source file, including its mandatory language header.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the header is absent or the source is invalid.
    pub fn evaluate_source_file(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut execution = self.prepare_source_file(input, trace)?;
        loop {
            match execution.step(self, trace)? {
                ExecutionStep::Complete(value) => return Ok(value),
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Returned { .. } => unreachable!("top-level return is rejected"),
            }
        }
    }

    /// Prepare a source unit for resumable execution.
    ///
    /// # Errors
    ///
    /// Returns a source or syntax diagnostic before any statement executes.
    pub fn prepare(
        &self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Execution, Diagnostic> {
        self.checkpoint(trace, None, None);
        trace.record(TraceEvent {
            event: "context.selected",
            rule: "TOPAL-SYN-UNICODE-001",
            detail: "design-0;Unicode=17.0.0",
        });
        let source = accepted_source(input, trace)?;
        let lexed = lex(&source);
        let parsed = parse(&source, &lexed);
        if let Some(error) = parsed.diagnostics.first() {
            return Err(diagnostic(&source, error.code, error.span, &error.message));
        }
        if parsed.statements.is_empty() {
            return Err(expected_statement(input));
        }
        Ok(Execution {
            source,
            statements: parsed.statements,
            cursor: 0,
            return_classifier: None,
        })
    }

    /// Prepare a complete source file and require its initial language selection.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the header is absent or the source is invalid.
    pub fn prepare_source_file(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Execution, Diagnostic> {
        let mut execution = self.prepare(input, trace)?;
        let Some(Statement::LanguageSelection {
            version, features, ..
        }) = execution.statements.first()
        else {
            return Err(diagnostic(
                &execution.source,
                "E-MISSING-LANGUAGE-VERSION",
                Span::new(0, 0),
                "source files begin with a language-version selection",
            )
            .with_help("add `use language (\n  version is v0.1\n)` at the start of the file"));
        };
        let features = features.clone();
        let requested = execution
            .source
            .slice(*version)
            .parse::<LanguageVersion>()
            .map_err(|message| {
                diagnostic(&execution.source, "E-LANGUAGE-VERSION", *version, message)
            })?;
        if requested != Self::highest_supported_language_version() {
            return Err(diagnostic(
                &execution.source,
                "E-UNSUPPORTED-LANGUAGE-VERSION",
                *version,
                format!(
                    "language version `{requested}` is not supported; highest supported version is `{}`",
                    Self::highest_supported_language_version()
                ),
            ));
        }
        self.language_version = requested;
        self.declared_libraries.clear();
        let mut libraries = BTreeSet::new();
        let mut declarations_closed = false;
        for statement in execution.statements.iter().skip(1) {
            match statement {
                Statement::LibrarySelection { name, span, .. } if !declarations_closed => {
                    let identity = execution.source.slice(*name);
                    if !libraries.insert(identity.to_owned()) {
                        return Err(diagnostic(
                            &execution.source,
                            "E-DUPLICATE-LIBRARY",
                            *span,
                            format!("library `{identity}` is declared more than once"),
                        ));
                    }
                }
                Statement::LibrarySelection { span, .. } => {
                    return Err(diagnostic(
                        &execution.source,
                        "E-LIBRARY-DECLARATION-ORDER",
                        *span,
                        "library dependencies immediately follow the initial language selection",
                    ));
                }
                _ => declarations_closed = true,
            }
        }
        let documentation_lexed = lex(&execution.source);
        let documentation_parsed = parse(&execution.source, &documentation_lexed);
        for declaration in extract_documentation(
            &execution.source,
            &documentation_lexed,
            &documentation_parsed,
        ) {
            if let Some(documentation) = declaration.documentation {
                self.documentation.insert(declaration.name, documentation);
            }
        }
        self.language_features = features
            .iter()
            .map(|feature| execution.source.slice(*feature).to_owned())
            .collect();
        execution.statements.remove(0);
        if execution.statements.is_empty() {
            return Err(expected_statement(input));
        }
        let feature_names = self
            .language_features
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(",");
        let detail = if feature_names.is_empty() {
            requested.to_string()
        } else {
            format!("{requested};features={feature_names}")
        };
        trace.record(TraceEvent {
            event: "language.context.selected",
            rule: "TOPAL-SYN-CONTEXT-001",
            detail: &detail,
        });
        Ok(execution)
    }

    /// Evaluate one expression against an immutable binding snapshot.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the input is not exactly one expression or
    /// when that expression cannot be evaluated.
    pub fn inspect(
        bindings: &BTreeMap<String, Value>,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut session = Self {
            bindings: bindings.clone(),
            functions: Box::new(BTreeMap::new()),
            generators: Box::new(BTreeMap::new()),
            declared_names: bindings.keys().cloned().collect(),
            published_names: BTreeSet::new(),
            documentation: Box::new(BTreeMap::new()),
            language_version: LanguageVersion::DESIGN_0,
            language_features: BTreeSet::new(),
            declared_libraries: BTreeSet::new(),
            consumed_names: BTreeSet::new(),
            local_function_names: BTreeSet::new(),
            enum_types: BTreeMap::new(),
            union_types: Box::new(BTreeMap::new()),
            generic_types: BTreeMap::new(),
            call_stack: Vec::new(),
            static_context: false,
            task_state: None,
            next_task_identity: Cell::new(0),
            next_transaction_identity: Cell::new(0),
        };
        let mut execution = session.prepare(input, trace)?;
        if !matches!(execution.statements.as_slice(), [Statement::Expression(_)]) {
            let span = execution
                .statements
                .first()
                .map_or_else(|| Span::new(0, 0), statement_span);
            return Err(diagnostic(
                &execution.source,
                "D-EXPECTED-EXPRESSION",
                span,
                "debugger inspection requires exactly one expression",
            ));
        }
        match execution.step(&mut session, trace)? {
            ExecutionStep::Complete(value) => Ok(value),
            ExecutionStep::Advanced { .. } => unreachable!("one expression completes execution"),
            ExecutionStep::Returned { .. } => unreachable!("inspection rejects return statements"),
        }
    }

    fn checkpoint(&self, trace: &mut impl TraceSink, value: Option<&Value>, span: Option<Span>) {
        trace.checkpoint(ExecutionSnapshot {
            bindings: &self.bindings,
            value,
            span,
        });
    }

    #[allow(clippy::too_many_lines)] // Keep recursive expression cases together and auditable.
    fn evaluate_expression(
        &self,
        source: &SourceText,
        expression: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let value = match expression {
            Expression::Block { statements, .. } => self.evaluate_block(source, statements, trace),
            Expression::Boolean(span) => Ok(evaluate_boolean_literal(source, *span, trace)),
            Expression::Measured { value, unit, span } => {
                let amount = parse_integer(source.slice(*value)).ok_or_else(|| {
                    diagnostic(
                        source,
                        "E-SIZE-LITERAL",
                        *span,
                        "size must use a Nat literal",
                    )
                })?;
                if amount < BigInt::from(0) || !matches!(source.slice(*unit), "b" | "B") {
                    return Err(diagnostic(
                        source,
                        "E-SIZE-LITERAL",
                        *span,
                        "size must be nonnegative and use `[b]` or `[B]`",
                    ));
                }
                let bits = if source.slice(*unit) == "B" {
                    amount * 8
                } else {
                    amount
                };
                trace.record(TraceEvent {
                    event: "layout.size.constructed",
                    rule: "TOPAL-LAYOUT-SIZE-001",
                    detail: source.slice(*span),
                });
                Ok(Value::SizeBits(bits))
            }
            Expression::Unit(_) => {
                trace.record(TraceEvent {
                    event: "product.unit",
                    rule: "TOPAL-TYPE-PRODUCT-001",
                    detail: "Tuple()",
                });
                Ok(Value::Unit)
            }
            Expression::Product { fields, span } => {
                self.evaluate_product(source, fields, *span, trace)
            }
            Expression::DecisionTable {
                subject,
                rules,
                span,
            } => {
                let subject_span = subject.span();
                let subject = self.evaluate_expression(source, subject, trace)?;
                let enum_matchers = rules
                    .iter()
                    .filter_map(|rule| match rule.matcher {
                        DecisionMatcher::Identifier(span) => Some(source.slice(span).to_owned()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                let has_result_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Result { .. } | DecisionMatcher::ErrorCode { .. }
                    )
                });
                let has_optional_matchers = rules
                    .iter()
                    .any(|rule| matches!(rule.matcher, DecisionMatcher::Optional { .. }));
                let has_list_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::ListEmpty(_) | DecisionMatcher::ListEntry { .. }
                    )
                });
                let has_union_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Union { .. } | DecisionMatcher::Variant { .. }
                    )
                });
                if !enum_matchers.is_empty()
                    && !has_union_matchers
                    && !rules
                        .iter()
                        .any(|rule| matches!(rule.matcher, DecisionMatcher::Otherwise(_)))
                {
                    let Value::Enum { type_name, .. } = &subject else {
                        return Err(diagnostic(
                            source,
                            "E-DECISION-SUBJECT-TYPE",
                            subject_span,
                            "enum alternative matchers require an Enum subject",
                        ));
                    };
                    if known_enum_alternatives(self, type_name).as_ref() != Some(&enum_matchers) {
                        return Err(diagnostic(
                            source,
                            "E-INCOMPLETE-DECISION",
                            *span,
                            format!("decision does not cover every `{type_name}` alternative"),
                        ));
                    }
                }
                let decision_rule = if has_union_matchers {
                    "TOPAL-DECISION-UNION-001"
                } else if has_list_matchers {
                    "TOPAL-DECISION-LIST-001"
                } else if has_optional_matchers {
                    "TOPAL-DECISION-OPTIONAL-001"
                } else if has_result_matchers {
                    "TOPAL-DECISION-RESULT-001"
                } else if !enum_matchers.is_empty() {
                    "TOPAL-DECISION-ENUM-001"
                } else if rules
                    .iter()
                    .any(|rule| matches!(&rule.matcher, DecisionMatcher::Comparison { .. }))
                {
                    "TOPAL-DECISION-COMPARISON-001"
                } else {
                    "TOPAL-DECISION-BOOLEAN-001"
                };
                let mut selected = None;
                for (index, rule) in rules.iter().enumerate() {
                    let matches = match &rule.matcher {
                        DecisionMatcher::Boolean { value, .. } => {
                            let Value::Boolean(subject) = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "Boolean literal matchers require a Boolean subject",
                                ));
                            };
                            *value == *subject
                        }
                        DecisionMatcher::Identifier(matcher) => {
                            let name = source.slice(*matcher);
                            if let Value::Enum {
                                type_name,
                                alternative,
                            } = &subject
                                && type_name == "Comparison"
                                && matches!(name, "Less" | "Equal" | "Greater")
                            {
                                alternative == name
                            } else {
                                let Some(candidate) = self.bindings.get(name).cloned() else {
                                    return Err(diagnostic(
                                        source,
                                        "E-UNBOUND-NAME",
                                        *matcher,
                                        format!("enum matcher `{name}` is not declared"),
                                    ));
                                };
                                values_equal(subject.clone(), candidate, trace).unwrap_or(false)
                            }
                        }
                        DecisionMatcher::Union { alternative, .. } => {
                            matches!(
                                &subject,
                                Value::Union(union)
                                    if union.payload.is_some()
                                        && union.alternative == source.slice(*alternative)
                            )
                        }
                        DecisionMatcher::Variant {
                            type_name, index, ..
                        } => {
                            let alternative = format!("at {}", source.slice(*index));
                            matches!(
                                &subject,
                                Value::Union(union)
                                    if union.payload.is_some()
                                        && union.type_name == source.slice(*type_name)
                                        && union.alternative == alternative
                            )
                        }
                        DecisionMatcher::Result { error, .. } => {
                            *error == matches!(subject, Value::Error { .. })
                        }
                        DecisionMatcher::Optional { some, .. } => {
                            let Value::Optional { payload, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "Optional matchers require an Optional subject",
                                ));
                            };
                            *some == payload.is_some()
                        }
                        DecisionMatcher::ListEmpty(_) => {
                            let Value::List { entries, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "list matchers require a List subject",
                                ));
                            };
                            entries.is_empty()
                        }
                        DecisionMatcher::ListEntry { .. } => {
                            let Value::List { entries, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "list matchers require a List subject",
                                ));
                            };
                            !entries.is_empty()
                        }
                        DecisionMatcher::ErrorCode {
                            namespace,
                            vocabulary,
                            code,
                            ..
                        } => {
                            let namespace = source.slice(*namespace);
                            let vocabulary = source.slice(*vocabulary);
                            let code_span = *code;
                            let code = source.slice(code_span);
                            let known = namespace == "lang"
                                && ((vocabulary == "arithmetic" && is_arithmetic_error_code(code))
                                    || (vocabulary == "generator" && code == "generator-closed"));
                            if !known {
                                return Err(diagnostic(
                                    source,
                                    "E-UNKNOWN-ERROR-CODE",
                                    code_span,
                                    "the error-code pattern requires a code published by the qualified language namespace",
                                ));
                            }
                            let matched = matches!(&subject, Value::Error { code: subject_code, .. } if subject_code == code);
                            if vocabulary == "generator" && matched {
                                trace.record(TraceEvent {
                                    event: "generator.error.code.matched",
                                    rule: "TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001",
                                    detail: code,
                                });
                            }
                            matched
                        }
                        DecisionMatcher::Comparison {
                            kind,
                            operand,
                            span: matcher_span,
                        } => {
                            let right_span = operand.span();
                            let right = self.evaluate_expression(source, operand, trace)?;
                            matches!(
                                apply_binary(
                                    source,
                                    *kind,
                                    subject.clone(),
                                    right,
                                    (*matcher_span, subject_span, right_span),
                                    trace,
                                )?,
                                Value::Boolean(true)
                            )
                        }
                        DecisionMatcher::Otherwise(_) => true,
                    };
                    let detail = format!("rule={index};matched={matches}");
                    trace.record(TraceEvent {
                        event: "decision.rule.considered",
                        rule: decision_rule,
                        detail: &detail,
                    });
                    if matches {
                        selected = Some((index, rule));
                        break;
                    }
                }
                let Some((index, selected_rule)) = selected else {
                    return Err(diagnostic(
                        source,
                        "E-INCOMPLETE-DECISION",
                        *span,
                        "no decision rule matched the subject",
                    ));
                };
                let detail = format!("rule={index}");
                trace.record(TraceEvent {
                    event: "decision.rule.selected",
                    rule: decision_rule,
                    detail: &detail,
                });
                if let DecisionMatcher::ErrorCode { code, .. } = selected_rule.matcher {
                    trace.record(TraceEvent {
                        event: "error.code.matched",
                        rule: "TOPAL-DECISION-ERROR-CODE-001",
                        detail: source.slice(code),
                    });
                }
                if let DecisionMatcher::Result { binding, .. } = selected_rule.matcher {
                    let name = source.slice(binding);
                    let mut branch = Self {
                        bindings: self.bindings.clone(),
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
                        declared_names: self.declared_names.clone(),
                        published_names: self.published_names.clone(),
                        documentation: self.documentation.clone(),
                        language_version: self.language_version,
                        language_features: self.language_features.clone(),
                        declared_libraries: self.declared_libraries.clone(),
                        consumed_names: self.consumed_names.clone(),
                        local_function_names: self.local_function_names.clone(),
                        enum_types: self.enum_types.clone(),
                        union_types: self.union_types.clone(),
                        generic_types: self.generic_types.clone(),
                        call_stack: self.call_stack.clone(),
                        static_context: self.static_context,
                        task_state: self.task_state.clone(),
                        next_task_identity: Cell::new(self.next_task_identity.get()),
                        next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
                    };
                    branch.bindings.insert(name.to_owned(), subject);
                    trace.record(TraceEvent {
                        event: "result.payload.bound",
                        rule: "TOPAL-DECISION-RESULT-001",
                        detail: name,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::Optional {
                    binding: Some(binding),
                    ..
                } = selected_rule.matcher
                {
                    let Value::Optional {
                        payload: Some(payload),
                        ..
                    } = subject
                    else {
                        unreachable!("Some matcher selected only for a present payload")
                    };
                    let name = source.slice(binding);
                    let mut branch = self.clone();
                    branch.bindings.insert(name.to_owned(), *payload);
                    trace.record(TraceEvent {
                        event: "optional.payload.bound",
                        rule: "TOPAL-DECISION-OPTIONAL-001",
                        detail: name,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::ListEntry { first, rest, .. } = selected_rule.matcher
                {
                    let Value::List {
                        element_classifier,
                        mut entries,
                    } = subject
                    else {
                        unreachable!("Entry matcher selected only for a nonempty List")
                    };
                    let first_value = entries.remove(0);
                    let first = source.slice(first);
                    let rest = source.slice(rest);
                    let mut branch = self.clone();
                    branch.bindings.insert(first.to_owned(), first_value);
                    branch.bindings.insert(
                        rest.to_owned(),
                        Value::List {
                            element_classifier,
                            entries,
                        },
                    );
                    let detail = format!("first={first};rest={rest}");
                    trace.record(TraceEvent {
                        event: "list.entry.decomposed",
                        rule: "TOPAL-DECISION-LIST-001",
                        detail: &detail,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::Union { binding, .. } = selected_rule.matcher {
                    self.evaluate_union_decision_action(
                        source,
                        subject,
                        binding,
                        &selected_rule.action,
                        trace,
                    )
                } else if let DecisionMatcher::Variant { binding, .. } = selected_rule.matcher {
                    self.evaluate_union_decision_action(
                        source,
                        subject,
                        binding,
                        &selected_rule.action,
                        trace,
                    )
                } else {
                    self.evaluate_expression(source, &selected_rule.action, trace)
                }
            }
            Expression::Integer(span) => evaluate_integer_literal(source, *span, trace),
            Expression::Rational(span) => evaluate_rational_literal(source, *span, trace),
            Expression::String(span) => evaluate_string_literal(source, *span, trace),
            Expression::Identifier(span) => self.resolve_identifier(source, *span, trace),
            Expression::ContextIdentifier(span) => {
                if self.call_stack.is_empty() && self.task_state.is_none() {
                    return Err(diagnostic(
                        source,
                        "E-CONTEXT-SELECTION",
                        *span,
                        "`@` selects the defining context from inside a function",
                    ));
                }
                let value = self
                    .task_state
                    .as_ref()
                    .and_then(|state| state.get(source.slice(*span)))
                    .cloned()
                    .map_or_else(|| self.resolve_identifier(source, *span, trace), Ok)?;
                trace.record(TraceEvent {
                    event: "context.member.selected",
                    rule: "TOPAL-CONTEXT-SELECT-001",
                    detail: source.slice(*span),
                });
                Ok(value)
            }
            Expression::Discard(span) => Err(diagnostic(
                source,
                "E-DISCARD-VALUE",
                *span,
                "discard is valid only in a declaration or pattern",
            )),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: _,
            } => Ok(self.capture_anonymous_function(source, parameters, body, trace)),
            Expression::Callable { kind, .. } => {
                trace.record(TraceEvent {
                    event: "function.callable.captured",
                    rule: "TOPAL-FUNCTION-CALLABLE-VALUE-001",
                    detail: callable_name(*kind),
                });
                Ok(Value::Callable(*kind))
            }
            Expression::Application { items, span } => {
                if let Some(value) = self.evaluate_layout_application(source, items, *span, trace) {
                    return value;
                }
                if let [Expression::Identifier(empty), element] = items.as_slice()
                    && source.slice(*empty) == "Empty"
                {
                    let element = self.evaluate_expression(source, element, trace)?;
                    let Value::Type(element_classifier) = element else {
                        return Err(diagnostic(
                            source,
                            "E-LIST-EMPTY-CLASSIFIER",
                            items[1].span(),
                            "Empty requires an element type",
                        ));
                    };
                    return Ok(construct_empty_list(element_classifier, trace));
                }
                if let [Expression::Identifier(task), options] = items.as_slice()
                    && source.slice(*task) == "Task"
                {
                    let Expression::Product { fields, .. } = options else {
                        return Err(diagnostic(
                            source,
                            "E-TASK-OPTIONS",
                            options.span(),
                            "Task requires one labeled option record",
                        ));
                    };
                    let mut resolved_options = Vec::with_capacity(fields.len());
                    for field in fields {
                        let Some(label) = field.label else {
                            return Err(diagnostic(
                                source,
                                "E-TASK-OPTIONS",
                                field.value.span(),
                                "Task options must be labeled",
                            ));
                        };
                        let value = if source.slice(label) == "identity"
                            && let Expression::Identifier(identity) = field.value
                        {
                            Value::String(source.slice(identity).to_owned())
                        } else {
                            self.evaluate_expression(source, &field.value, trace)?
                        };
                        resolved_options.push((source.slice(label).to_owned(), value));
                    }
                    trace.record(TraceEvent {
                        event: "task.type.specialized",
                        rule: "TOPAL-TASK-DEFINITION-001",
                        detail: "Task",
                    });
                    return Ok(Value::TaskType(Box::new(TaskTypeValue {
                        name: None,
                        options: resolved_options,
                    })));
                }
                if self.is_native_serialization(source, items) {
                    return self.evaluate_native_serialization(source, items, *span, trace);
                }
                if let [Expression::Identifier(definition), argument] = items.as_slice()
                    && matches!(
                        self.bindings.get(source.slice(*definition)),
                        Some(Value::TaskDefinition(_))
                    )
                {
                    return self.construct_task_instance(
                        source,
                        *definition,
                        argument,
                        *span,
                        trace,
                    );
                }
                if matches!(items.first(), Some(Expression::Identifier(instance))
                    if matches!(self.bindings.get(source.slice(*instance)), Some(Value::TaskInstance(_))))
                {
                    return self.evaluate_task_message(source, items, *span, trace);
                }
                if Self::is_lang_introspection(source, items) {
                    return self.evaluate_lang_introspection(source, items, *span, trace);
                }
                if Self::is_empty_effects(source, items) {
                    trace.record(TraceEvent {
                        event: "effects.empty.constructed",
                        rule: "TOPAL-EFFECT-EMPTY-001",
                        detail: "Effects ()",
                    });
                    return Ok(Value::Effects(Vec::new()));
                }
                if Self::is_use_application(source, items) {
                    return self.evaluate_use_application(source, items, *span, trace);
                }
                if self.is_bound_namespace_application(source, items) {
                    return self.evaluate_bound_namespace_application(source, items, *span, trace);
                }
                if Self::is_root_qualified_application(source, items) {
                    return self.evaluate_root_qualified_application(source, items, *span, trace);
                }
                if Self::is_unfold_construction(source, items) {
                    return self.construct_unfold_generator(source, items, *span, trace);
                }
                if Self::is_iterate_take_while_construction(source, items) {
                    return self.construct_iterate_take_while(source, items, *span, trace);
                }
                if Self::is_iterate_construction(source, items) {
                    return self.construct_iterate_generator(source, items, *span, trace);
                }
                if Self::is_generator_take_while_application(source, items) {
                    return self.apply_generator_take_while(source, items, *span, trace);
                }
                if self.is_bound_named_function_call(source, items) {
                    return self
                        .evaluate_bound_named_function_call(source, expression, items, trace);
                }
                if self.is_bound_callable_call(source, items) {
                    return self.evaluate_bound_callable_call(source, items, *span, trace);
                }
                if Self::is_traversal_control_constructor(source, items) {
                    return self.construct_traversal_control(source, items, *span, trace);
                }
                if self.is_bound_anonymous_call(source, items) {
                    return self.evaluate_bound_anonymous_call(source, items, *span, trace);
                }
                if Self::is_record_reconstruction(source, items) {
                    return self.evaluate_record_reconstruction(source, items, *span, trace);
                }
                if self.is_bound_list_higher_order_application(source, items) {
                    return self.evaluate_list_higher_order(source, items, *span, trace);
                }
                if Self::is_range_selection(source, items) {
                    return self.evaluate_range_selection(source, items, *span, trace);
                }
                if self.is_explicit_modulo(source, items) {
                    return self.apply_explicit_modulo(source, items, trace);
                }
                if Self::is_modular_type_definition(source, items) {
                    return self.construct_modular_type(source, items, trace);
                }
                if self.is_modular_construction(source, items) {
                    return self.construct_modular_value(source, items, *span, trace);
                }
                if Self::is_constraint_definition(source, items) {
                    return self.construct_constraint(source, items, trace);
                }
                if self.is_constraint_application(source, items) {
                    return self.apply_constraint(source, items, *span, trace);
                }
                if self.application_is_union_constructor(source, items) {
                    return self.construct_union_application(source, items, *span, trace);
                }
                if matches!(
                    items.as_slice(),
                    [Expression::Identifier(operation), ..]
                        if matches!(source.slice(*operation), "unzip" | "collect" | "collect-set" | "collect-bag" | "collect-map")
                ) || matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), ..]
                        if matches!(source.slice(*operation), "zip-longest" | "collect")
                ) {
                    return self.evaluate_list_materialization(source, items, *span, trace);
                }
                if matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
                        if matches!(source.slice(*operation), "map" | "select" | "remove-indexes" | "remove-values")
                ) || matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), _, Expression::AnonymousFunction { .. }]
                        if source.slice(*operation) == "fold"
                ) {
                    return self.evaluate_list_higher_order(source, items, *span, trace);
                }
                if Self::is_characters_application(source, items) {
                    return self.evaluate_characters_application(source, items, *span, trace);
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "canonically-equals"
                {
                    let left_span = left.span();
                    let right_span = right.span();
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    let (Value::String(left), Value::String(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-CANONICAL-EQUALITY-OPERANDS",
                            cover(left_span, right_span),
                            "canonically-equals requires two String operands",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.canonically-equals(String,String)",
                    });
                    let value = Value::Boolean(canonically_equal(&left, &right));
                    trace.record(TraceEvent {
                        event: "string.canonical-equality.compared",
                        rule: "TOPAL-STRING-CANONICAL-EQUALITY-001",
                        detail: if matches!(value, Value::Boolean(true)) {
                            "equal"
                        } else {
                            "unequal"
                        },
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if let Some(value) = evaluate_arithmetic_error_code(source, items, trace) {
                    return Ok(value);
                }
                if let Some(value) = evaluate_generator_error_code(source, items, trace) {
                    return Ok(value);
                }
                if let [Expression::Identifier(constructor), payload] = items.as_slice()
                    && source.slice(*constructor) == "Some"
                {
                    let value = self.evaluate_expression(source, payload, trace)?;
                    let payload_classifier = structural_value_classifier(&value);
                    trace.record(TraceEvent {
                        event: "optional.some.constructed",
                        rule: "TOPAL-TYPE-OPTIONAL-CONSTRUCT-001",
                        detail: &payload_classifier,
                    });
                    return Ok(Value::Optional {
                        payload_classifier,
                        payload: Some(Box::new(value)),
                    });
                }
                if let [Expression::Identifier(constructor), domain] = items.as_slice()
                    && source.slice(*constructor) == "None"
                    && let Some(mut payload_classifier) = classifier_expression(source, domain)
                {
                    payload_classifier =
                        substitute_classifier(&payload_classifier, &self.generic_types);
                    trace.record(TraceEvent {
                        event: "optional.none.constructed",
                        rule: "TOPAL-TYPE-OPTIONAL-CONSTRUCT-001",
                        detail: &payload_classifier,
                    });
                    return Ok(Value::Optional {
                        payload_classifier,
                        payload: None,
                    });
                }
                if let [Expression::Identifier(constructor), character] = items.as_slice()
                    && source.slice(*constructor) == "String"
                {
                    let value = self.evaluate_expression(source, character, trace)?;
                    let Value::String(text) = value else {
                        return Err(diagnostic(
                            source,
                            "E-STRING-CONSTRUCTOR-CHARACTER",
                            character.span(),
                            "String construction requires a Character value",
                        ));
                    };
                    let count = character_count(&text);
                    if count != 1 {
                        return Err(diagnostic(
                            source,
                            "E-STRING-CONSTRUCTOR-CHARACTER",
                            character.span(),
                            format!(
                                "String construction requires one Character, but the operand contains {count}"
                            ),
                        ));
                    }
                    trace.record(TraceEvent {
                        event: "string.from-character",
                        rule: "TOPAL-STRING-FROM-CHARACTER-001",
                        detail: "preserved",
                    });
                    return Ok(Value::String(text));
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Int"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_int(source, operand, value, trace);
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Nat"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_nat(source, operand, value, trace);
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Rational"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_rational(source, operand, value, trace);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "absolute"
                {
                    let operand_span = operand.span();
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let (value, selection, classifier) = match value {
                        Value::Int(value) => (
                            Value::Int(if value < BigInt::from(0) {
                                -value
                            } else {
                                value
                            }),
                            "root.absolute(Int)",
                            "Int",
                        ),
                        Value::Rational(value) => (
                            Value::Rational(
                                if value < BigRational::from_integer(BigInt::from(0)) {
                                    -value
                                } else {
                                    value
                                },
                            ),
                            "root.absolute(Rational)",
                            "Rational",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                operand_span,
                                "absolute requires an exact numeric operand",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.absolute",
                        rule: "TOPAL-NUM-ABS-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [
                    Expression::Identifier(callable),
                    Expression::Identifier(domain),
                ] = items.as_slice()
                    && source.slice(*callable) == "zero"
                {
                    let (value, selection, classifier) = match source.slice(*domain) {
                        "Int" => (Value::Int(BigInt::from(0)), "root.zero(Int)", "Int"),
                        "Nat" => (Value::Int(BigInt::from(0)), "root.zero(Nat)", "Nat"),
                        "Rational" => (
                            Value::Rational(BigRational::from_integer(BigInt::from(0))),
                            "root.zero(Rational)",
                            "Rational",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                *domain,
                                "zero requires a supported numeric type",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "numeric.zero.constructed",
                        rule: "TOPAL-NUM-ZERO-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [
                    Expression::Identifier(callable),
                    Expression::Identifier(domain),
                ] = items.as_slice()
                    && source.slice(*callable) == "one"
                    && matches!(source.slice(*domain), "Int" | "Nat" | "Rational")
                {
                    let (value, selection, classifier) = match source.slice(*domain) {
                        "Int" => (Value::Int(BigInt::from(1)), "root.one(Int)", "Int"),
                        "Nat" => (Value::Int(BigInt::from(1)), "root.one(Nat)", "Nat"),
                        "Rational" => (
                            Value::Rational(BigRational::from_integer(BigInt::from(1))),
                            "root.one(Rational)",
                            "Rational",
                        ),
                        _ => unreachable!("guarded numeric one domain"),
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "numeric.one.constructed",
                        rule: "TOPAL-NUM-ONE-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if is_singleton_list_construction(source, items) {
                    return evaluate_singleton_list(source, self, items, trace);
                }
                if is_explicit_empty_list_construction(source, items) {
                    return evaluate_empty_list(source, items, trace);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "negate"
                {
                    let operand_span = operand.span();
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let (value, selection, classifier, rule) = match value {
                        Value::Int(value) => (
                            Value::Int(-value),
                            "root.negate(Int)",
                            "Int",
                            "TOPAL-NUM-NEG-001",
                        ),
                        Value::Rational(value) => (
                            Value::Rational(-value),
                            "root.negate(Rational)",
                            "Rational",
                            "TOPAL-NUM-RAT-NEG-001",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                operand_span,
                                "negate requires an exact numeric operand",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.negate",
                        rule,
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "not"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let Value::Boolean(value) = value else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-NOT-OPERAND",
                            operand.span(),
                            "not requires a Boolean operand",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.not(Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "not",
                    });
                    return Ok(Value::Boolean(!value));
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && matches!(source.slice(*callable), "in" | "contains")
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    return apply_range_membership(
                        source,
                        source.slice(*callable),
                        left,
                        right,
                        *span,
                        trace,
                    );
                }
                if let [text, Expression::Identifier(callable), index] = items.as_slice()
                    && source.slice(*callable) == "character-at"
                {
                    let text_span = text.span();
                    let index_span = index.span();
                    let text = self.evaluate_expression(source, text, trace)?;
                    let index = self.evaluate_expression(source, index, trace)?;
                    let (Value::String(text), Value::Int(index)) = (text, index) else {
                        return Err(diagnostic(
                            source,
                            "E-CHARACTER-AT-OPERANDS",
                            cover(text_span, index_span),
                            "character-at requires a String and an Int index",
                        ));
                    };
                    let payload = usize::try_from(index)
                        .ok()
                        .and_then(|index| character_at(&text, index))
                        .map(|character| Box::new(Value::String(character.to_owned())));
                    trace.record(TraceEvent {
                        event: "string.character-at",
                        rule: "TOPAL-STRING-CHARACTER-AT-001",
                        detail: if payload.is_some() { "Some" } else { "None" },
                    });
                    return Ok(Value::Optional {
                        payload_classifier: "Character".to_owned(),
                        payload,
                    });
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "and"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    return apply_and(source, left, right, *span, trace);
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "or"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    if let (Value::Capability(mut left), Value::Capability(right)) =
                        (left.clone(), right.clone())
                    {
                        left.extend(right);
                        trace.record(TraceEvent {
                            event: "capability.composed",
                            rule: "TOPAL-CAPABILITY-EVIDENCE-001",
                            detail: "or",
                        });
                        return Ok(Value::Capability(left));
                    }
                    let (Value::Boolean(left), Value::Boolean(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-OR-OPERANDS",
                            *span,
                            "or requires two Boolean operands; range union is a Predicate",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.or(Boolean,Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "or:eager",
                    });
                    return Ok(Value::Boolean(left || right));
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "xor"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    let (Value::Boolean(left), Value::Boolean(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-XOR-OPERANDS",
                            *span,
                            "xor requires two Boolean operands",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.xor(Boolean,Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "xor:eager",
                    });
                    return Ok(Value::Boolean(left != right));
                }
                if items.len() == 3
                    && let Expression::Identifier(name_span) = items[1]
                    && !self.bindings.contains_key(source.slice(name_span))
                    && self.functions.contains_key(source.slice(name_span))
                {
                    let argument_span = cover(items[0].span(), items[2].span());
                    let call = Expression::Application {
                        items: vec![
                            Expression::Identifier(name_span),
                            Expression::Product {
                                fields: vec![
                                    topal_syntax::ProductField {
                                        label: None,
                                        value: items[0].clone(),
                                    },
                                    topal_syntax::ProductField {
                                        label: None,
                                        value: items[2].clone(),
                                    },
                                ],
                                span: argument_span,
                            },
                        ],
                        span: *span,
                    };
                    return self.evaluate_expression(source, &call, trace);
                }
                if items.len() == 2
                    && let Expression::Identifier(name_span) = &items[0]
                    && !self.bindings.contains_key(source.slice(*name_span))
                    && let Some(candidates) = self.generators.get(source.slice(*name_span)).cloned()
                {
                    let name = source.slice(*name_span);
                    let argument_span = items[1].span();
                    let argument = self.evaluate_expression(source, &items[1], trace)?;
                    let Some(generator) = candidates
                        .iter()
                        .find(|candidate| function_accepts(&candidate.parameters, &argument))
                        .cloned()
                    else {
                        return Err(no_applicable_generator(
                            source,
                            name,
                            argument_span,
                            &argument,
                            &candidates,
                        ));
                    };
                    let mut generator_scope = Self {
                        bindings: generator.bindings,
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
                        declared_names: BTreeSet::new(),
                        published_names: BTreeSet::new(),
                        documentation: self.documentation.clone(),
                        language_version: self.language_version,
                        language_features: self.language_features.clone(),
                        declared_libraries: self.declared_libraries.clone(),
                        consumed_names: BTreeSet::new(),
                        local_function_names: BTreeSet::new(),
                        enum_types: self.enum_types.clone(),
                        union_types: self.union_types.clone(),
                        generic_types: self.generic_types.clone(),
                        call_stack: self.call_stack.clone(),
                        static_context: false,
                        task_state: None,
                        next_task_identity: Cell::new(self.next_task_identity.get()),
                        next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
                    };
                    bind_generator_arguments(
                        &mut generator_scope,
                        &generator.parameters,
                        argument,
                        trace,
                    );
                    let signature = generator
                        .parameters
                        .iter()
                        .map(|(_, classifier)| classifier.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    trace.record(TraceEvent {
                        event: "generator.selected",
                        rule: "TOPAL-GENERATOR-OVERLOAD-001",
                        detail: &signature,
                    });
                    trace.record(TraceEvent {
                        event: "generator.started",
                        rule: "TOPAL-GENERATOR-DECLARATION-001",
                        detail: name,
                    });
                    let mut cursor = 0;
                    let mut pending_yield = None;
                    let mut resume_binding = None;
                    let mut returned = None;
                    advance_custom_generator(
                        &generator.source,
                        &generator.body,
                        &mut cursor,
                        &mut generator_scope,
                        &mut pending_yield,
                        &mut resume_binding,
                        &mut returned,
                        &generator.yielded,
                        &generator.result,
                        name,
                        trace,
                    )?;
                    let origin = format!("root.{name}");
                    let value = Value::SuspendedGenerator {
                        source: Box::new(generator.source),
                        body: Box::new(generator.body),
                        cursor,
                        bindings: Box::new(generator_scope.bindings),
                        scope_state: Box::new(GeneratorScopeState {
                            functions: *generator_scope.functions,
                            declared_names: generator_scope.declared_names,
                            local_function_names: generator_scope.local_function_names,
                            enum_types: generator_scope.enum_types,
                            union_types: generator_scope.union_types,
                        }),
                        pending_yield,
                        resume_binding,
                        returned: returned.map(Box::new),
                        yield_classifier: generator.yielded,
                        return_classifier: generator.result,
                        origin,
                        task_state: None,
                        task_owner: None,
                    };
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "list-permutations"
                            | "list-combinations"
                            | "list-subsets"
                            | "list-cartesian-product"
                    )
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_combinatorial_construction(
                        source,
                        operation,
                        operand,
                        operand_span,
                        trace,
                    )?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "graph-bfs"
                            | "graph-dfs"
                            | "graph-shortest-path"
                            | "graph-topological-sort"
                            | "graph-weak-components"
                            | "graph-weighted-shortest-path"
                    )
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value =
                        apply_graph_algorithm(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name_span) = &items[0]
                    && !self.bindings.contains_key(source.slice(*name_span))
                    && let Some(candidates) = self.functions.get(source.slice(*name_span)).cloned()
                {
                    let name = source.slice(*name_span);
                    let argument_span = items[1].span();
                    let argument = self.evaluate_expression(source, &items[1], trace)?;
                    let function = candidates
                        .iter()
                        .find(|function| {
                            (!self.static_context || function.is_static)
                                && user_function_accepts(function, &argument)
                        })
                        .cloned();
                    let Some(function) = function else {
                        if self.static_context
                            && candidates.iter().all(|function| !function.is_static)
                        {
                            return Err(diagnostic(
                                source,
                                "E-STATIC-CALLS-RUNTIME-FUNCTION",
                                *name_span,
                                format!("static execution cannot call ordinary function `{name}`"),
                            ));
                        }
                        return Err(no_applicable_overload(
                            source,
                            name,
                            argument_span,
                            &argument,
                            &candidates,
                            self.static_context,
                        ));
                    };
                    if matches!(
                        argument,
                        Value::CharacterGenerator { .. }
                            | Value::CharacterReturningGenerator { .. }
                            | Value::SuspendedGenerator { .. }
                    ) {
                        let classifier = structural_value_classifier(&argument);
                        trace.record(TraceEvent {
                            event: "generator.parameter.transferred",
                            rule: if matches!(argument, Value::SuspendedGenerator { .. }) {
                                "TOPAL-GENERATOR-FUNCTION-PARAMETER-001"
                            } else {
                                "TOPAL-STRING-CHARACTERS-PARAMETER-001"
                            },
                            detail: &classifier,
                        });
                    }
                    let signature = function_signature(name, &function);
                    let recursion_rule =
                        recursion_rule_for_call(&self.call_stack, name, &signature, &function);
                    if self
                        .call_stack
                        .iter()
                        .any(|active| active.signature == signature)
                        && recursion_rule.is_none()
                    {
                        return Err(diagnostic(
                            source,
                            "E-UNPROVEN-RECURSION",
                            *name_span,
                            format!(
                                "recursive cycle returning to `{name}` requires termination proof on every call edge"
                            ),
                        ));
                    }
                    let rule = function_rule(function.is_static, function.parameters.len());
                    if let Some(recursion_rule) = recursion_rule {
                        if is_mutual_recursion_rule(recursion_rule) {
                            trace.record(TraceEvent {
                                event: "function.recursion.cycle.proven",
                                rule: recursion_rule,
                                detail: name,
                            });
                        }
                        trace.record(TraceEvent {
                            event: "function.recursion.descended",
                            rule: recursion_rule,
                            detail: name,
                        });
                    }
                    if candidates.len() > 1 {
                        trace.record(TraceEvent {
                            event: "function.overload.selected",
                            rule: "TOPAL-FUNCTION-OVERLOAD-001",
                            detail: &signature,
                        });
                    }
                    let mut function_scope = Self {
                        bindings: function.bindings.clone(),
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
                        declared_names: BTreeSet::new(),
                        published_names: BTreeSet::new(),
                        documentation: self.documentation.clone(),
                        language_version: self.language_version,
                        language_features: self.language_features.clone(),
                        declared_libraries: self.declared_libraries.clone(),
                        consumed_names: BTreeSet::new(),
                        local_function_names: BTreeSet::new(),
                        enum_types: self.enum_types.clone(),
                        union_types: self.union_types.clone(),
                        generic_types: BTreeMap::new(),
                        call_stack: self.call_stack.clone(),
                        static_context: function.is_static,
                        task_state: None,
                        next_task_identity: Cell::new(self.next_task_identity.get()),
                        next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
                    };
                    function_scope.call_stack.push(ActiveCall {
                        name: name.to_owned(),
                        signature: signature.clone(),
                        termination_rule: function.termination_rule,
                        recursion_target: function.recursion_target.clone(),
                    });
                    bind_function_arguments(&mut function_scope, &function, argument, trace, rule)?;
                    let mut invocation_generics = BTreeMap::new();
                    populate_function_generics(
                        &function,
                        &function_scope,
                        &mut invocation_generics,
                    );
                    function_scope.generic_types = invocation_generics;
                    trace.record(TraceEvent {
                        event: "function.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: &signature,
                    });
                    trace.record(TraceEvent {
                        event: "function.entry",
                        rule,
                        detail: name,
                    });
                    let mut body_execution = Execution {
                        source: function.source.clone(),
                        statements: function.body.clone(),
                        cursor: 0,
                        return_classifier: Some(function.result.clone()),
                    };
                    let (value, result_span) = loop {
                        match body_execution.step(&mut function_scope, trace)? {
                            ExecutionStep::Advanced { .. } => {}
                            ExecutionStep::Complete(value) => {
                                break (
                                    value,
                                    statement_span(
                                        function.body.last().expect("function body is nonempty"),
                                    ),
                                );
                            }
                            ExecutionStep::Returned { value, span } => break (value, span),
                        }
                    };
                    if !function.result.starts_with("Generator ") {
                        close_remaining_character_generators(&mut function_scope, trace)?;
                    }
                    if !generic_result_accepts(&function, &function_scope, &value) {
                        return Err(diagnostic(
                            &function.source,
                            "E-FUNCTION-RESULT-TYPE",
                            result_span,
                            format!(
                                "function `{name}` returned a value outside `{}`",
                                function.result
                            ),
                        ));
                    }
                    if let Value::Error { domain, code, .. } = &value
                        && result_success_classifier(&function.result).is_some()
                    {
                        let detail = format!("domain={domain};code={code}");
                        trace.record(TraceEvent {
                            event: "result.error.propagated",
                            rule: "TOPAL-TYPE-RESULT-001",
                            detail: &detail,
                        });
                    }
                    if matches!(
                        value,
                        Value::CharacterGenerator { .. }
                            | Value::CharacterReturningGenerator { .. }
                            | Value::SuspendedGenerator { .. }
                    ) {
                        let classifier = structural_value_classifier(&value);
                        trace.record(TraceEvent {
                            event: "generator.result.transferred",
                            rule: if matches!(value, Value::SuspendedGenerator { .. }) {
                                "TOPAL-GENERATOR-FUNCTION-RESULT-001"
                            } else {
                                "TOPAL-STRING-CHARACTERS-RESULT-001"
                            },
                            detail: &classifier,
                        });
                    }
                    trace.record(TraceEvent {
                        event: "function.exit",
                        rule,
                        detail: name,
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "empty?")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_empty_predicate(source, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "array-at?" | "map-lookup" | "set-contains?" | "bag-multiplicity"
                    )
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value =
                        apply_collection_query(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "range-lower"
                            | "range-upper"
                            | "range-lower-inclusive?"
                            | "range-upper-inclusive?"
                    )
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_range_bound(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "string-starts-with"
                            | "string-ends-with"
                            | "string-contains"
                            | "string-trim"
                            | "string-replace-all"
                            | "string-repeat"
                            | "string-count-exact"
                            | "string-find-all"
                            | "string-split-exact"
                            | "string-glob-matches"
                            | "string-contains-any"
                            | "string-lines"
                            | "string-words"
                            | "string-join"
                            | "string-regex-contains"
                    )
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value =
                        apply_string_utility(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if is_list_uncons(source, items) {
                    return evaluate_list_uncons(source, self, items, *span, trace);
                }
                if is_list_projection(source, items) {
                    return evaluate_list_projection(source, self, items, *span, trace);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(source.slice(*name), "character-count" | "entry-count")
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_count(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "upper")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "upper requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.upper(String)",
                    });
                    let value = Value::String(uppercase(&text));
                    trace.record(TraceEvent {
                        event: "string.uppercased",
                        rule: "TOPAL-STRING-UPPER-001",
                        detail: "unicode-default",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "lower")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "lower requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.lower(String)",
                    });
                    let value = Value::String(lowercase(&text));
                    trace.record(TraceEvent {
                        event: "string.lowercased",
                        rule: "TOPAL-STRING-LOWER-001",
                        detail: "unicode-default",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "case-fold")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "case-fold requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.case-fold(String)",
                    });
                    let value = Value::String(case_fold(&text));
                    trace.record(TraceEvent {
                        event: "string.case-folded",
                        rule: "TOPAL-STRING-CASE-FOLD-001",
                        detail: "unicode-default-full",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "empty")
                    && matches!(&items[1], Expression::Identifier(name) if source.slice(*name) == "String")
                {
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.empty(String)",
                    });
                    trace.record(TraceEvent {
                        event: "string.empty",
                        rule: "TOPAL-STRING-EMPTY-001",
                        detail: "String",
                    });
                    let value = Value::String(String::new());
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                let (mut result, mut index) = if matches!(
                    items.first(),
                    Some(Expression::Callable {
                        kind: CallableKind::Minus,
                        ..
                    })
                ) {
                    let Some(operand) = items.get(1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            *span,
                            "expected an operand after prefix -",
                        ));
                    };
                    let operand = self.evaluate_expression(source, operand, trace)?;
                    (apply_negate(source, operand, *span, trace)?, 2)
                } else {
                    (self.evaluate_expression(source, &items[0], trace)?, 1)
                };
                let mut composing_literals = matches!(items.first(), Some(Expression::String(_)));
                while index < items.len() {
                    if composing_literals
                        && let Expression::String(right_span) = &items[index]
                        && let Value::String(left) = &result
                    {
                        let Value::String(right) =
                            self.evaluate_expression(source, &items[index], trace)?
                        else {
                            unreachable!("string literal constructs String");
                        };
                        trace.record(TraceEvent {
                            event: "string.literals.composed",
                            rule: "TOPAL-STRING-LITERAL-COMPOSE-001",
                            detail: "String",
                        });
                        result = Value::String(format!("{left}{right}"));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), *right_span)),
                        );
                        index += 1;
                        continue;
                    }
                    composing_literals = false;
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "normalize"
                        && let Value::String(text) = &result
                    {
                        let Some(form) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected a normalization form after normalize",
                            ));
                        };
                        let form_span = form.span();
                        let Expression::Identifier(form_name) = form else {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                form_span,
                                "the implemented String normalize operation requires NFC or NFD",
                            ));
                        };
                        let form_name = source.slice(*form_name);
                        let (normalized, selection, rule) = match form_name {
                            "NFC" => (
                                normalize_nfc(text),
                                "root.normalize(String,NFC)",
                                "TOPAL-STRING-NORMALIZE-NFC-001",
                            ),
                            "NFD" => (
                                normalize_nfd(text),
                                "root.normalize(String,NFD)",
                                "TOPAL-STRING-NORMALIZE-NFD-001",
                            ),
                            _ => {
                                return Err(diagnostic(
                                    source,
                                    "E-NO-APPLICABLE-OVERLOAD",
                                    form_span,
                                    "the implemented String normalize operation requires NFC or NFD",
                                ));
                            }
                        };
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: selection,
                        });
                        let changed = normalized != *text;
                        trace.record(TraceEvent {
                            event: "string.normalized",
                            rule,
                            detail: if changed {
                                "changed=true"
                            } else {
                                "changed=false"
                            },
                        });
                        result = Value::String(normalized);
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), form_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "byte-count"
                        && let Value::String(text) = &result
                    {
                        let Some(encoding) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected an Encoding after byte-count",
                            ));
                        };
                        let encoding_span = encoding.span();
                        if !matches!(encoding, Expression::Identifier(name) if source.slice(*name) == "Utf8")
                        {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                encoding_span,
                                "the implemented String byte-count operation requires Utf8",
                            ));
                        }
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: "root.byte-count(String,Utf8)",
                        });
                        let byte_count = text.len();
                        let detail = format!("bytes={byte_count}");
                        trace.record(TraceEvent {
                            event: "string.utf8-byte-count",
                            rule: "TOPAL-STRING-UTF8-BYTE-COUNT-001",
                            detail: &detail,
                        });
                        result = Value::Int(BigInt::from(byte_count));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), encoding_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "reverse"
                        && matches!(result, Value::List { .. })
                    {
                        apply_list_reverse(&mut result, trace);
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && matches!(
                            source.slice(*callable_span),
                            "list-enumerate" | "list-group-runs"
                        )
                        && matches!(result, Value::List { .. })
                    {
                        result = apply_list_sequence_unary(
                            source,
                            source.slice(*callable_span),
                            result,
                            *callable_span,
                            trace,
                        )?;
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && matches!(
                            source.slice(*callable_span),
                            "stable-sort" | "stable-sort-descending"
                        )
                        && matches!(result, Value::List { .. })
                    {
                        let descending = source.slice(*callable_span) == "stable-sort-descending";
                        apply_list_stable_sort(
                            source,
                            &mut result,
                            descending,
                            *callable_span,
                            trace,
                        )?;
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "entries"
                        && matches!(result, Value::List { .. })
                    {
                        result = apply_list_entries_view(result, trace);
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "insert-at"
                        && let Value::List { .. } = result
                    {
                        result = self.evaluate_list_insert_at(
                            source,
                            result,
                            items.get(index + 1),
                            items.get(index + 2),
                            *callable_span,
                            trace,
                        )?;
                        index += 3;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && matches!(
                            source.slice(*callable_span),
                            "prepend"
                                | "append"
                                | "concat"
                                | "contains-entry"
                                | "contains-sequence"
                                | "contains-subsequence"
                                | "split-at"
                                | "take"
                                | "drop"
                                | "remove"
                                | "remove-indexes"
                                | "zip-exact"
                                | "zip-shortest"
                                | "list-zip-shortest"
                                | "list-index-of"
                                | "list-last-index-of"
                                | "list-rotate-left"
                                | "list-rotate-right"
                                | "list-chunks"
                                | "list-windows"
                                | "ordered-binary-search"
                                | "ordered-merge"
                                | "ordered-nth"
                                | "ordered-smallest"
                                | "remove-first"
                                | "remove-all"
                        )
                        && matches!(result, Value::List { .. })
                    {
                        let operation = source.slice(*callable_span);
                        let Some(right) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                format!("expected an operand after {operation}"),
                            ));
                        };
                        let right_span = right.span();
                        let right_is_closed = expression_is_closed(right);
                        let right = self.evaluate_expression(source, right, trace)?;
                        result = apply_list_operation(
                            source,
                            operation,
                            result,
                            right,
                            right_span,
                            right_is_closed,
                            trace,
                        )?;
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), right_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "concat"
                        && let Value::String(left) = &result
                    {
                        let Some(right) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected a String after concat",
                            ));
                        };
                        let right_span = right.span();
                        let right = self.evaluate_expression(source, right, trace)?;
                        let Value::String(right) = right else {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                right_span,
                                "concat requires two String operands",
                            ));
                        };
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: "root.concat(String,String)",
                        });
                        trace.record(TraceEvent {
                            event: "evaluation.concat",
                            rule: "TOPAL-STRING-CONCAT-001",
                            detail: "String",
                        });
                        result = Value::String(format!("{left}{right}"));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), right_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(label_span) = &items[index]
                        && let Value::Error {
                            domain,
                            code,
                            line,
                            column,
                        } = &result
                    {
                        let label = source.slice(*label_span);
                        let selected = match label {
                            "code" => Value::Enum {
                                type_name: "lang arithmetic ArithmeticErrorCode".into(),
                                alternative: code.clone(),
                            },
                            "domain" => Value::ErrorDomain(domain.clone()),
                            "detail" => Value::Optional {
                                payload_classifier: "String".into(),
                                payload: None,
                            },
                            "cause" => Value::Optional {
                                payload_classifier: "Error".into(),
                                payload: None,
                            },
                            "source" => Value::Optional {
                                payload_classifier: "SourceLocation".into(),
                                payload: Some(Box::new(Value::Record(vec![
                                    ("line".into(), Value::Int(BigInt::from(*line))),
                                    ("column".into(), Value::Int(BigInt::from(*column))),
                                ]))),
                            },
                            _ => {
                                return Err(diagnostic(
                                    source,
                                    "E-NO-SUCH-ERROR-FIELD",
                                    *label_span,
                                    format!("Error has no implemented field named `{label}`"),
                                ));
                            }
                        };
                        trace.record(TraceEvent {
                            event: "error.field.selected",
                            rule: "TOPAL-ERROR-FIELD-001",
                            detail: label,
                        });
                        result = selected;
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(label_span) = &items[index]
                        && let Value::Record(fields) = &result
                    {
                        let label = source.slice(*label_span);
                        let selected = fields
                            .iter()
                            .find(|(field, _)| field == label)
                            .map(|(_, value)| value.clone())
                            .ok_or_else(|| {
                                diagnostic(
                                    source,
                                    "E-NO-SUCH-RECORD-FIELD",
                                    *label_span,
                                    format!("record has no field named `{label}`"),
                                )
                            })?;
                        trace.record(TraceEvent {
                            event: "record.field.selected",
                            rule: "TOPAL-TYPE-PRODUCT-001",
                            detail: label,
                        });
                        result = selected;
                        index += 1;
                        continue;
                    }
                    let Expression::Callable {
                        kind,
                        span: operator_span,
                    } = &items[index]
                    else {
                        let mut error = diagnostic(
                            source,
                            "E-UNSUPPORTED-APPLICATION",
                            items[index].span(),
                            "the implemented subset requires a symbolic callable",
                        );
                        if let Expression::Identifier(name_span) = &items[index]
                            && let Some(candidate) =
                                closest_root_operation(source.slice(*name_span))
                        {
                            error = error.with_help(format!("did you mean `{candidate}`?"));
                        }
                        return Err(error);
                    };
                    let Some(right) = items.get(index + 1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            Span::new(operator_span.end, operator_span.end),
                            "expected an operand after callable",
                        ));
                    };
                    let right_span = right.span();
                    let right = self.evaluate_expression(source, right, trace)?;
                    result = apply_binary(
                        source,
                        *kind,
                        result,
                        right,
                        (*span, items[0].span(), right_span),
                        trace,
                    )?;
                    self.checkpoint(
                        trace,
                        Some(&result),
                        Some(cover(items[0].span(), right_span)),
                    );
                    index += 2;
                }
                Ok(result)
            }
        }?;
        self.checkpoint(trace, Some(&value), Some(expression.span()));
        Ok(value)
    }

    #[inline(never)]
    fn evaluate_block(
        &self,
        source: &SourceText,
        statements: &[Statement],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if statements.is_empty() {
            trace.record(TraceEvent {
                event: "block.empty.evaluated",
                rule: "TOPAL-SYN-GRAMMAR-001",
                detail: "Unit",
            });
            return Ok(Value::Unit);
        }
        let mut branch = self.clone();
        let mut execution = Execution {
            source: source.clone(),
            statements: statements.to_vec(),
            cursor: 0,
            return_classifier: None,
        };
        loop {
            match execution.step(&mut branch, trace)? {
                ExecutionStep::Complete(value) => {
                    trace.record(TraceEvent {
                        event: "block.evaluated",
                        rule: "TOPAL-SYN-GRAMMAR-001",
                        detail: &structural_value_classifier(&value),
                    });
                    return Ok(value);
                }
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Returned { .. } => {
                    unreachable!("a standalone block rejects return without a function context")
                }
            }
        }
    }

    fn evaluate_record(
        &self,
        source: &SourceText,
        fields: &[topal_syntax::ProductField],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut values = Vec::with_capacity(fields.len());
        for field in fields {
            let label_span = field.label.expect("record fields are labeled");
            let label = source.slice(label_span);
            if values.iter().any(|(existing, _)| existing == label) {
                return Err(diagnostic(
                    source,
                    "E-DUPLICATE-RECORD-FIELD",
                    label_span,
                    "record field label occurs more than once",
                ));
            }
            let value = self.evaluate_expression(source, &field.value, trace)?;
            values.push((label.to_owned(), value));
        }
        let detail = format!("fields={}", values.len());
        trace.record(TraceEvent {
            event: "product.record",
            rule: "TOPAL-TYPE-PRODUCT-001",
            detail: &detail,
        });
        Ok(Value::Record(values))
    }

    fn evaluate_product(
        &self,
        source: &SourceText,
        fields: &[topal_syntax::ProductField],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let labeled = fields.iter().filter(|field| field.label.is_some()).count();
        if labeled != 0 && labeled != fields.len() {
            return Err(diagnostic(
                source,
                "E-MIXED-PRODUCT-FIELDS",
                span,
                "a product cannot mix positional and labeled fields",
            ));
        }
        if labeled == 0 {
            let values = fields
                .iter()
                .map(|field| self.evaluate_expression(source, &field.value, trace))
                .collect::<Result<Vec<_>, _>>()?;
            let detail = format!("fields={}", values.len());
            trace.record(TraceEvent {
                event: "product.tuple",
                rule: "TOPAL-TYPE-PRODUCT-001",
                detail: &detail,
            });
            Ok(Value::Tuple(values))
        } else {
            self.evaluate_record(source, fields, trace)
        }
    }

    #[allow(clippy::too_many_lines)] // Built-in static policy identities stay explicit and auditable.
    fn resolve_identifier(
        &self,
        source: &SourceText,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let name = source.slice(span);
        if let Ok(version) = name.parse::<LanguageVersion>() {
            return Ok(Value::Version(version));
        }
        if let Some(classifier) = self.generic_types.get(name) {
            trace.record(TraceEvent {
                event: "type.resolved",
                rule: "TOPAL-FUNCTION-GENERIC-HEADER-001",
                detail: classifier,
            });
            return Ok(Value::Type(classifier.clone()));
        }
        if matches!(name, "Little" | "Big") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ENDIAN-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Endian".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "ReadWrite" | "ReadOnly" | "WriteOnly" | "Reserved") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ACCESS-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Access".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "MostSignificantFirst" | "LeastSignificantFirst") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-BIT-ORDER-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "BitOrder".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "Natural" | "Packed") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-PACKING-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Packing".into(),
                alternative: name.into(),
            });
        }
        if name == "Declared" {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-FIELD-ORDER-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "FieldOrder".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "AfterTag" | "Overlay") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "PayloadPlacement".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "NoLength" | "NoTerminator") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ABSENCE-POLICY-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "LayoutPolicy".into(),
                alternative: name.into(),
            });
        }
        if matches!(
            name,
            "Empty"
                | "BooleanBits"
                | "RawBits"
                | "UnsignedBinary"
                | "TwosComplement"
                | "OnesComplement"
                | "SignMagnitude"
                | "BiasedBinary"
                | "Ratio"
                | "Utf8"
                | "Utf16"
                | "Utf32"
                | "Ascii"
                | "Tagged"
                | "NoPadding"
                | "Cached"
                | "Uncached"
                | "Memory"
                | "MMIO"
        ) {
            return Ok(Value::Enum {
                type_name: "LayoutEncoding".into(),
                alternative: name.into(),
            });
        }
        if matches!(
            name,
            "Boolean"
                | "Completed"
                | "Int"
                | "MessageContext"
                | "Nat"
                | "Rational"
                | "Scope"
                | "String"
                | "Unit"
        ) && name != "Completed"
        {
            trace.record(TraceEvent {
                event: "type.resolved",
                rule: "TOPAL-ABSTRACTION-TYPE-VALUE-001",
                detail: name,
            });
            return Ok(Value::Type(name.into()));
        }
        if matches!(
            name,
            "Equality" | "Ordering" | "Foldable" | "Membership" | "Indexed" | "Keyed"
        ) {
            trace.record(TraceEvent {
                event: "capability.resolved",
                rule: "TOPAL-CAPABILITY-EVIDENCE-001",
                detail: name,
            });
            return Ok(Value::Capability(vec![BTreeSet::from([name.to_owned()])]));
        }
        if name == "std" && !self.declared_libraries.contains("std") {
            return Err(diagnostic(
                source,
                "E-UNDECLARED-LIBRARY",
                span,
                "the `std` namespace requires `use library std ( version is v0.1 )`",
            ));
        }
        if name == "root" {
            trace.record(TraceEvent {
                event: "namespace.resolved",
                rule: "TOPAL-NAMESPACE-ROOT-001",
                detail: "root",
            });
            return Ok(Value::Namespace(Rc::new(NamespaceValue {
                name: "root".into(),
                bindings: self.bindings.clone(),
                functions: (*self.functions).clone(),
                generators: (*self.generators).clone(),
            })));
        }
        if name == "Completed" {
            trace.record(TraceEvent {
                event: "completion.evidence",
                rule: "TOPAL-EXEC-COMPLETED-001",
                detail: "Completed",
            });
            return Ok(Value::Completed);
        }
        if self.consumed_names.contains(name) {
            return Err(consumed_generator_diagnostic(source, span, name));
        }
        let value = if let Some(value) = self.bindings.get(name) {
            value.clone()
        } else if let Some(candidates) = self.functions.get(name) {
            Value::NamedFunction(Rc::new(NamedFunction {
                name: name.to_owned(),
                candidates: candidates.clone(),
            }))
        } else {
            let error = diagnostic(source, "E-UNBOUND-NAME", span, "name is not bound");
            return Err(closest_name(name, self.bindings.keys())
                .or_else(|| closest_name(name, self.functions.keys()))
                .or_else(|| closest_root_operation(name))
                .map_or(error.clone(), |candidate| {
                    error.with_help(format!("did you mean `{candidate}`?"))
                }));
        };
        trace.record(TraceEvent {
            event: "binding.resolved",
            rule: "TOPAL-SYN-BIND-001",
            detail: name,
        });
        Ok(value)
    }

    fn evaluate_union_decision_action(
        &self,
        source: &SourceText,
        subject: Value,
        binding: Span,
        action: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::Union(mut union) = subject else {
            unreachable!("payload Union matcher selected only for a payload alternative")
        };
        let payload = union
            .payload
            .take()
            .expect("payload matcher selected a present payload");
        let name = source.slice(binding);
        let mut branch = self.clone();
        branch.bindings.insert(name.to_owned(), *payload);
        trace.record(TraceEvent {
            event: "union.payload.bound",
            rule: "TOPAL-DECISION-UNION-001",
            detail: name,
        });
        branch.evaluate_expression(source, action, trace)
    }

    fn union_constructor(&self, name: &str) -> Option<(&str, &str)> {
        self.union_types
            .iter()
            .find_map(|(type_name, alternatives)| {
                alternatives
                    .get(name)
                    .and_then(|classifier| classifier.as_deref())
                    .map(|classifier| (type_name.as_str(), classifier))
            })
    }

    fn application_is_union_constructor(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(constructor), _] if self.union_constructor(source.slice(*constructor)).is_some())
            || matches!(
                items,
                [Expression::Identifier(type_name), Expression::Identifier(at), Expression::Integer(_), _]
                    if source.slice(*at) == "at" && self.union_types.contains_key(source.slice(*type_name))
            )
    }

    fn is_constraint_definition(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [Expression::Identifier(_), Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
                if source.slice(*operation) == "constraint"
        )
    }

    fn is_modular_type_definition(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(kind), _] if matches!(source.slice(*kind), "ModNat" | "ModInt"))
    }

    fn is_explicit_modulo(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), Expression::Identifier(type_name)]
                if source.slice(*operation) == "modulo"
                    && matches!(self.bindings.get(source.slice(*type_name)), Some(Value::ModularType(_)))
        )
    }

    fn is_range_selection(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), selector]
                if matches!(source.slice(*operation), "select" | "select-index")
                    && !matches!(selector, Expression::AnonymousFunction { .. })
        )
    }

    fn is_bound_list_higher_order_application(
        &self,
        source: &SourceText,
        items: &[Expression],
    ) -> bool {
        let bound_function = |expression: &Expression| {
            matches!(expression, Expression::Identifier(name)
                if matches!(self.bindings.get(source.slice(*name)), Some(Value::AnonymousFunction(_))))
        };
        matches!(
            items,
            [_, Expression::Identifier(operation), function]
                if matches!(source.slice(*operation), "map" | "select" | "remove-indexes" | "remove-values")
                    && bound_function(function)
        ) || matches!(
            items,
            [_, Expression::Identifier(operation), _, function]
                if source.slice(*operation) == "fold" && bound_function(function)
        )
    }

    fn is_bound_anonymous_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::AnonymousFunction(_))))
    }

    fn is_bound_callable_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::Callable(_))))
    }

    fn is_bound_named_function_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::NamedFunction(_))))
    }

    fn is_root_qualified_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(root), Expression::Identifier(_), ..]
            if source.slice(*root) == "root")
    }

    fn is_empty_effects(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), Expression::Unit(_)] if source.slice(*name) == "Effects")
    }

    fn is_use_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(keyword), _]
            if source.slice(*keyword) == "use")
    }

    fn evaluate_use_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [_, selected] = items else {
            unreachable!("preselected use application")
        };
        let value = self.evaluate_expression(source, selected, trace)?;
        if !matches!(value, Value::Namespace(_)) {
            return Err(diagnostic(
                source,
                "E-USE-NON-NAMESPACE",
                selected.span(),
                "use requires a published namespace path",
            ));
        }
        trace.record(TraceEvent {
            event: "namespace.made-available",
            rule: "TOPAL-NAMESPACE-USE-001",
            detail: &value.to_string(),
        });
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_bound_namespace_application(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(alias), Expression::Identifier(_), ..]
            if matches!(self.bindings.get(source.slice(*alias)), Some(Value::Namespace(_))))
    }

    fn evaluate_bound_namespace_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(alias),
            Expression::Identifier(member),
            remainder @ ..,
        ] = items
        else {
            unreachable!("preselected namespace alias application")
        };
        let Some(Value::Namespace(namespace)) = self.bindings.get(source.slice(*alias)) else {
            unreachable!("preselected namespace alias")
        };
        if source.slice(*alias) == "std" && !self.declared_libraries.contains("std") {
            return Err(diagnostic(
                source,
                "E-UNDECLARED-LIBRARY",
                *alias,
                "the `std` namespace requires `use library std ( version is v0.1 )`",
            ));
        }
        let member_name = source.slice(*member);
        if !namespace.bindings.contains_key(member_name)
            && !namespace.functions.contains_key(member_name)
            && !namespace.generators.contains_key(member_name)
        {
            let names = namespace
                .bindings
                .keys()
                .chain(namespace.functions.keys())
                .chain(namespace.generators.keys());
            let error = diagnostic(
                source,
                "E-NAMESPACE-MEMBER-NOT-FOUND",
                *member,
                format!(
                    "namespace `{}` has no member `{member_name}`",
                    namespace.name
                ),
            );
            return Err(
                closest_name(member_name, names).map_or(error.clone(), |candidate| {
                    error.with_help(format!("did you mean `{candidate}`?"))
                }),
            );
        }
        trace.record(TraceEvent {
            event: "namespace.alias.member.resolved",
            rule: "TOPAL-NAMESPACE-ALIAS-001",
            detail: member_name,
        });
        let mut qualified = self.clone();
        qualified.bindings = namespace.bindings.clone();
        *qualified.functions = namespace.functions.clone();
        *qualified.generators = namespace.generators.clone();
        if remainder.is_empty() {
            return qualified.resolve_identifier(source, *member, trace);
        }
        let expression = Expression::Application {
            items: std::iter::once(Expression::Identifier(*member))
                .chain(remainder.iter().cloned())
                .collect(),
            span,
        };
        qualified.evaluate_expression(source, &expression, trace)
    }

    fn evaluate_root_qualified_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(_),
            Expression::Identifier(member),
            remainder @ ..,
        ] = items
        else {
            unreachable!("preselected root-qualified application")
        };
        let member_name = source.slice(*member);
        trace.record(TraceEvent {
            event: "namespace.member.resolved",
            rule: "TOPAL-NAMESPACE-ROOT-001",
            detail: member_name,
        });
        if remainder.is_empty() {
            return self.resolve_identifier(source, *member, trace);
        }
        let expression = Expression::Application {
            items: std::iter::once(Expression::Identifier(*member))
                .chain(remainder.iter().cloned())
                .collect(),
            span,
        };
        self.evaluate_expression(source, &expression, trace)
    }

    fn evaluate_bound_named_function_call(
        &self,
        source: &SourceText,
        expression: &Expression,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(alias), _] = items else {
            unreachable!("preselected bound named function call")
        };
        let alias = source.slice(*alias);
        let Some(Value::NamedFunction(function)) = self.bindings.get(alias) else {
            unreachable!("preselected named function binding")
        };
        let mut invocation = self.clone();
        invocation.bindings.remove(alias);
        invocation
            .functions
            .insert(alias.to_owned(), function.candidates.clone());
        trace.record(TraceEvent {
            event: "function.value.called",
            rule: "TOPAL-FUNCTION-VALUE-001",
            detail: &function.name,
        });
        invocation.evaluate_expression(source, expression, trace)
    }

    fn evaluate_bound_callable_call(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), argument] = items else {
            unreachable!("preselected bound callable call")
        };
        let Value::Callable(kind) = self.resolve_identifier(source, *name, trace)? else {
            unreachable!("preselected callable binding")
        };
        let argument_span = argument.span();
        let argument = self.evaluate_expression(source, argument, trace)?;
        trace.record(TraceEvent {
            event: "function.callable.called",
            rule: "TOPAL-FUNCTION-CALLABLE-VALUE-001",
            detail: callable_name(kind),
        });
        match argument {
            Value::Tuple(mut operands) if operands.len() == 2 => {
                let right = operands.pop().expect("two operands");
                let left = operands.pop().expect("two operands");
                apply_binary(
                    source,
                    kind,
                    left,
                    right,
                    (span, argument_span, argument_span),
                    trace,
                )
            }
            operand if kind == CallableKind::Minus => apply_negate(source, operand, span, trace),
            value => Err(diagnostic(
                source,
                "E-CALLABLE-ARGUMENT-PACKAGE",
                argument_span,
                format!(
                    "callable `{}` requires a two-field positional product, found `{}`",
                    callable_name(kind),
                    structural_value_classifier(&value)
                ),
            )),
        }
    }

    fn is_traversal_control_constructor(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(source.slice(*name), "Continue" | "Finish"))
    }

    fn is_iterate_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
            if source.slice(*operation) == "iterate")
    }

    fn is_unfold_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
            if source.slice(*operation) == "unfold")
    }

    fn construct_unfold_generator(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [seed, _, step_expression] = items else {
            unreachable!("preselected unfold construction")
        };
        let seed = self.evaluate_expression(source, seed, trace)?;
        let step = self.evaluate_expression(source, step_expression, trace)?;
        if !matches!(&step, Value::AnonymousFunction(function) if function.parameters.len() == 1) {
            return Err(diagnostic(
                source,
                "E-UNFOLD-FUNCTION-ARITY",
                step_expression.span(),
                "unfold step function requires exactly one seed parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.unfold.constructed",
            rule: "TOPAL-GENERATOR-UNFOLD-001",
            detail: &structural_value_classifier(&seed),
        });
        let value = Value::UnfoldGenerator {
            seed: Box::new(seed),
            step: Box::new(step),
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_iterate_take_while_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items,
            [_, Expression::Identifier(iterate), Expression::AnonymousFunction { .. }, Expression::Identifier(take_while), Expression::AnonymousFunction { .. }]
                if source.slice(*iterate) == "iterate" && source.slice(*take_while) == "take-while")
    }

    fn construct_iterate_take_while(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [initial, _, next_expression, _, predicate_expression] = items else {
            unreachable!("preselected iterate take-while construction")
        };
        let current = self.evaluate_expression(source, initial, trace)?;
        let classifier = structural_value_classifier(&current);
        let next = self.evaluate_expression(source, next_expression, trace)?;
        let predicate = self.evaluate_expression(source, predicate_expression, trace)?;
        for (value, expression, role) in [
            (&next, next_expression, "next"),
            (&predicate, predicate_expression, "predicate"),
        ] {
            if !matches!(value, Value::AnonymousFunction(function) if function.parameters.len() == 1)
            {
                return Err(diagnostic(
                    source,
                    "E-GENERATED-TRAVERSAL-FUNCTION-ARITY",
                    expression.span(),
                    format!("iterate {role} function requires exactly one parameter"),
                ));
            }
        }
        trace.record(TraceEvent {
            event: "generator.take-while.constructed",
            rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current: Box::new(current),
            next: Box::new(next),
            take_while: Some(Box::new(predicate)),
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_generator_take_while_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items,
            [_, Expression::Identifier(take_while), Expression::AnonymousFunction { .. }]
                if source.slice(*take_while) == "take-while")
    }

    fn apply_generator_take_while(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [generator, _, predicate] = items else {
            unreachable!("preselected generator take-while application")
        };
        let generator_span = generator.span();
        let generator = self.evaluate_expression(source, generator, trace)?;
        let Value::IterateGenerator {
            current,
            next,
            classifier,
            ..
        } = generator
        else {
            return Err(diagnostic(
                source,
                "E-TAKE-WHILE-SOURCE",
                generator_span,
                "take-while requires a lazy iterate generator",
            ));
        };
        let predicate_value = self.evaluate_expression(source, predicate, trace)?;
        if !matches!(&predicate_value, Value::AnonymousFunction(function) if function.parameters.len() == 1)
        {
            return Err(diagnostic(
                source,
                "E-GENERATED-TRAVERSAL-FUNCTION-ARITY",
                predicate.span(),
                "take-while predicate requires exactly one parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.take-while.constructed",
            rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current,
            next,
            take_while: Some(Box::new(predicate_value)),
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_iterate_generator(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [initial, _, next_expression] = items else {
            unreachable!("preselected iterate construction")
        };
        let current = self.evaluate_expression(source, initial, trace)?;
        let classifier = structural_value_classifier(&current);
        let next = self.evaluate_expression(source, next_expression, trace)?;
        let Value::AnonymousFunction(function) = &next else {
            unreachable!("iterate syntax requires an anonymous function")
        };
        if function.parameters.len() != 1 {
            return Err(diagnostic(
                source,
                "E-ITERATE-FUNCTION-ARITY",
                next_expression.span(),
                "iterate next function requires exactly one parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.iterate.constructed",
            rule: "TOPAL-GENERATOR-ITERATE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current: Box::new(current),
            next: Box::new(next),
            take_while: None,
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_traversal_control(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), payload] = items else {
            unreachable!("preselected traversal-control constructor")
        };
        let payload = self.evaluate_expression(source, payload, trace)?;
        let constructor = source.slice(*name);
        trace.record(TraceEvent {
            event: "traversal.control.constructed",
            rule: "TOPAL-EXEC-TRAVERSAL-CONTROL-001",
            detail: constructor,
        });
        let value = if constructor == "Continue" {
            Value::Continue(Box::new(payload))
        } else {
            Value::Finish(Box::new(payload))
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn evaluate_bound_anonymous_call(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), argument_expression] = items else {
            unreachable!("preselected bound anonymous call")
        };
        let function = self.resolve_identifier(source, *name, trace)?;
        let arity = match &function {
            Value::AnonymousFunction(function) => function.parameters.len(),
            _ => unreachable!("preselected anonymous binding"),
        };
        let argument = self.evaluate_expression(source, argument_expression, trace)?;
        let arguments = match (arity, argument) {
            (1, value) => vec![value],
            (_, Value::Tuple(values)) => values,
            (_, value) => {
                return Err(diagnostic(
                    source,
                    "E-ANONYMOUS-ARGUMENT-PACKAGE",
                    argument_expression.span(),
                    format!(
                        "anonymous function expects {arity} arguments packaged as a tuple, found `{}`",
                        structural_value_classifier(&value)
                    ),
                ));
            }
        };
        self.invoke_anonymous_function(&function, arguments, span, trace)
    }

    fn is_record_reconstruction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), Expression::Product { fields, .. }]
                if source.slice(*operation) == "with"
                    && !fields.is_empty()
                    && fields.iter().all(|field| field.label.is_some())
        )
    }

    fn evaluate_record_reconstruction(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            base,
            _,
            Expression::Product {
                fields: replacements,
                ..
            },
        ] = items
        else {
            unreachable!("preselected record reconstruction")
        };
        let Value::Record(mut fields) = self.evaluate_expression(source, base, trace)? else {
            return Err(diagnostic(
                source,
                "E-RECONSTRUCT-NON-RECORD",
                base.span(),
                "`with` reconstruction requires a labeled product",
            ));
        };
        let mut replaced = BTreeSet::new();
        for replacement in replacements {
            let label_span = replacement.label.expect("preselected labeled replacement");
            let label = source.slice(label_span);
            if !replaced.insert(label) {
                return Err(diagnostic(
                    source,
                    "E-DUPLICATE-RECONSTRUCTION-FIELD",
                    label_span,
                    format!("field `{label}` is replaced more than once"),
                ));
            }
            let Some((_, value)) = fields.iter_mut().find(|(name, _)| name == label) else {
                return Err(diagnostic(
                    source,
                    "E-NO-SUCH-RECORD-FIELD",
                    label_span,
                    format!("record has no field named `{label}`"),
                ));
            };
            *value = self.evaluate_expression(source, &replacement.value, trace)?;
            trace.record(TraceEvent {
                event: "record.field.replaced",
                rule: "TOPAL-TYPE-RECONSTRUCT-001",
                detail: label,
            });
        }
        let result = Value::Record(fields);
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn evaluate_range_selection(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [collection, Expression::Identifier(operation), selector] = items else {
            unreachable!("preselected range selection")
        };
        let collection_value = self.evaluate_expression(source, collection, trace)?;
        let selector_value = self.evaluate_expression(source, selector, trace)?;
        let Value::IntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } = selector_value
        else {
            return Err(diagnostic(
                source,
                "E-SELECTION-RANGE",
                selector.span(),
                "range selection requires Range Int",
            ));
        };
        let operation = source.slice(*operation);
        let result = match collection_value {
            Value::List {
                element_classifier,
                entries,
            } if operation == "select-index" => {
                let entries = entries
                    .into_iter()
                    .enumerate()
                    .filter(|(index, _)| {
                        let index = BigInt::from(*index);
                        bound_contains(&index, &lower, &upper, lower_inclusive, upper_inclusive)
                    })
                    .map(|(_, value)| value)
                    .collect();
                Value::List {
                    element_classifier,
                    entries,
                }
            }
            Value::List {
                element_classifier,
                entries,
            } if operation == "select" => {
                let entries = entries
                    .into_iter()
                    .filter(|value| {
                        matches!(value, Value::Int(candidate) if bound_contains(candidate, &lower, &upper, lower_inclusive, upper_inclusive))
                    })
                    .collect();
                Value::List {
                    element_classifier,
                    entries,
                }
            }
            Value::String(text) if operation == "select-index" => {
                let selected = characters(&text)
                    .enumerate()
                    .filter(|(index, _)| {
                        let index = BigInt::from(*index);
                        bound_contains(&index, &lower, &upper, lower_inclusive, upper_inclusive)
                    })
                    .map(|(_, character)| character)
                    .collect::<String>();
                Value::String(selected)
            }
            value => {
                return Err(diagnostic(
                    source,
                    "E-SELECTION-SOURCE",
                    collection.span(),
                    format!(
                        "{operation} range has no overload for `{}`",
                        structural_value_classifier(&value)
                    ),
                ));
            }
        };
        trace.record(TraceEvent {
            event: "collection.range.selected",
            rule: if operation == "select-index" {
                "TOPAL-RANGE-INDEX-SELECTION-001"
            } else {
                "TOPAL-RANGE-VALUE-SELECTION-001"
            },
            detail: operation,
        });
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn apply_explicit_modulo(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [operand, _, Expression::Identifier(type_name)] = items else {
            unreachable!("preselected explicit modular reduction")
        };
        let operand_value = self.evaluate_expression(source, operand, trace)?;
        let Value::Int(value) = operand_value else {
            return Err(diagnostic(
                source,
                "E-MODULO-OPERAND",
                operand.span(),
                "explicit modulo construction requires Int",
            ));
        };
        let name = source.slice(*type_name);
        let Value::ModularType(kind) = self.bindings.get(name).expect("known modular type") else {
            unreachable!("preselected modular type")
        };
        let value = reduce_modular(value, &kind.lower, &kind.upper);
        trace.record(TraceEvent {
            event: "numeric.modular.reduced",
            rule: "TOPAL-NUM-MODULAR-REDUCE-001",
            detail: name,
        });
        Ok(Value::Modular {
            type_name: name.into(),
            lower: kind.lower.clone(),
            upper: kind.upper.clone(),
            value,
        })
    }

    fn construct_modular_type(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(kind), range] = items else {
            unreachable!("preselected modular type definition")
        };
        let signed = source.slice(*kind) == "ModInt";
        let range_value = self.evaluate_expression(source, range, trace)?;
        let Value::IntRange {
            lower,
            upper,
            lower_inclusive: true,
            upper_inclusive: true,
        } = range_value
        else {
            return Err(diagnostic(
                source,
                "E-MODULAR-RANGE",
                range.span(),
                "ModNat and ModInt require a finite Int range",
            ));
        };
        if lower > BigInt::from(0)
            || upper < BigInt::from(0)
            || (!signed && lower != BigInt::from(0))
        {
            return Err(diagnostic(
                source,
                "E-MODULAR-RANGE",
                range.span(),
                "modular range must contain zero and ModNat must begin at zero",
            ));
        }
        trace.record(TraceEvent {
            event: "numeric.modular.type.constructed",
            rule: "TOPAL-NUM-MODULAR-TYPE-001",
            detail: source.slice(*kind),
        });
        Ok(Value::ModularType(Box::new(ModularType {
            name: None,
            signed,
            lower,
            upper,
        })))
    }

    fn is_modular_construction(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _] if matches!(self.bindings.get(source.slice(*name)), Some(Value::ModularType(_))))
    }

    fn construct_modular_value(
        &self,
        source: &SourceText,
        items: &[Expression],
        _span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name_span), operand] = items else {
            unreachable!("preselected modular construction")
        };
        let name = source.slice(*name_span);
        let Value::ModularType(kind) = self.bindings.get(name).expect("known modular type") else {
            unreachable!("preselected modular type")
        };
        let operand_value = self.evaluate_expression(source, operand, trace)?;
        let Value::Int(value) = operand_value else {
            return Err(diagnostic(
                source,
                "E-MODULAR-CONSTRUCTION-OPERAND",
                operand.span(),
                "modular construction requires Int",
            ));
        };
        if value < kind.lower || value > kind.upper {
            if expression_is_closed(operand) {
                return Err(diagnostic(
                    source,
                    "E-MODULAR-OUT-OF-RANGE",
                    operand.span(),
                    format!("value is outside `{name}` canonical range"),
                ));
            }
            let position = source.position(operand.span().start);
            return Ok(Value::Error {
                domain: format!("root.{name}(Int)"),
                code: "out-of-range".into(),
                line: position.line,
                column: position.column,
            });
        }
        trace.record(TraceEvent {
            event: "numeric.modular.constructed",
            rule: "TOPAL-NUM-MODULAR-CONSTRUCT-001",
            detail: name,
        });
        Ok(Value::Modular {
            type_name: name.into(),
            lower: kind.lower.clone(),
            upper: kind.upper.clone(),
            value,
        })
    }

    fn construct_constraint(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(base), _, predicate] = items else {
            unreachable!("preselected constraint definition")
        };
        let base_classifier = source.slice(*base);
        if !matches!(
            base_classifier,
            "Boolean" | "Int" | "Nat" | "Rational" | "String"
        ) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-BASE",
                *base,
                "constraint base must be a supported value classifier",
            ));
        }
        let predicate = self.evaluate_expression(source, predicate, trace)?;
        trace.record(TraceEvent {
            event: "constraint.constructed",
            rule: "TOPAL-TYPE-CONSTRAINT-001",
            detail: base_classifier,
        });
        Ok(Value::Constraint(Box::new(ConstraintValue {
            name: None,
            base_classifier: base_classifier.into(),
            predicate,
        })))
    }

    fn is_constraint_application(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _] if matches!(self.bindings.get(source.slice(*name)), Some(Value::Constraint(_))))
    }

    fn apply_constraint(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name_span), operand] = items else {
            unreachable!("preselected constraint application")
        };
        let name = source.slice(*name_span);
        let Value::Constraint(constraint) = self.bindings.get(name).expect("known constraint")
        else {
            unreachable!("preselected constraint value")
        };
        let value = self.evaluate_expression(source, operand, trace)?;
        if !value_has_classifier(&value, &constraint.base_classifier) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-OPERAND",
                operand.span(),
                format!(
                    "constraint `{name}` requires `{}`",
                    constraint.base_classifier
                ),
            ));
        }
        let decision = self.invoke_anonymous_function(
            &constraint.predicate,
            vec![value.clone()],
            span,
            trace,
        )?;
        let Value::Boolean(accepted) = decision else {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-PREDICATE-RESULT",
                operand.span(),
                "constraint predicate must return Boolean",
            ));
        };
        trace.record(TraceEvent {
            event: "constraint.validated",
            rule: "TOPAL-TYPE-CONSTRAINT-VALIDATE-001",
            detail: if accepted { "accepted" } else { "rejected" },
        });
        if accepted {
            return Ok(Value::Refined {
                constraint: name.into(),
                base_classifier: constraint.base_classifier.clone(),
                value: Box::new(value),
            });
        }
        if expression_is_closed(operand) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-REJECTED",
                operand.span(),
                format!("value does not satisfy constraint `{name}`"),
            ));
        }
        let position = source.position(operand.span().start);
        Ok(Value::Error {
            domain: format!("root.{name}({})", constraint.base_classifier),
            code: "out-of-range".into(),
            line: position.line,
            column: position.column,
        })
    }

    fn is_characters_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items.first(), Some(Expression::Identifier(operation)) if source.slice(*operation) == "characters")
    }

    fn evaluate_characters_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let text = items.get(1).expect("characters application has text");
        let text_span = text.span();
        let text = self.evaluate_expression(source, text, trace)?;
        let Value::String(text) = text else {
            return Err(diagnostic(
                source,
                "E-CHARACTERS-OPERAND",
                text_span,
                "characters requires a String operand",
            ));
        };
        let value = if items.len() == 4 {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.characters(String)",
            });
            let mut collected = String::new();
            for character in characters(&text) {
                trace.record(TraceEvent {
                    event: "generator.yielded",
                    rule: "TOPAL-STRING-CHARACTERS-COLLECT-001",
                    detail: character,
                });
                collected.push_str(character);
            }
            trace.record(TraceEvent {
                event: "string.characters.collected",
                rule: "TOPAL-STRING-CHARACTERS-COLLECT-001",
                detail: "String",
            });
            Value::String(collected)
        } else {
            trace.record(TraceEvent {
                event: "generator.started",
                rule: "TOPAL-STRING-CHARACTERS-GENERATOR-001",
                detail: "Generator Character Unit Unit",
            });
            Value::CharacterGenerator {
                generated: characters(&text).map(str::to_owned).collect(),
                origin: "root.characters".to_owned(),
            }
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_union_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [Expression::Identifier(constructor), payload] = items {
            return self.construct_union(source, *constructor, payload, span, trace);
        }
        let [
            Expression::Identifier(type_name),
            _,
            Expression::Integer(index),
            payload,
        ] = items
        else {
            unreachable!("preselected positional Variant constructor application")
        };
        let index_text = source.slice(*index);
        let key = format!("at {index_text}");
        let type_text = source.slice(*type_name);
        let classifier = self
            .union_types
            .get(type_text)
            .and_then(|alternatives| alternatives.get(&key))
            .and_then(Option::as_deref)
            .ok_or_else(|| {
                diagnostic(
                    source,
                    "E-VARIANT-INDEX",
                    *index,
                    "Variant alternative index is outside its declared bounds",
                )
            })?;
        let value = self.evaluate_expression(source, payload, trace)?;
        if !value_has_classifier(&value, classifier) {
            return Err(diagnostic(
                source,
                "E-VARIANT-PAYLOAD-CLASSIFIER",
                payload.span(),
                format!("Variant alternative {index_text} requires `{classifier}`"),
            ));
        }
        trace.record(TraceEvent {
            event: "variant.constructed",
            rule: "TOPAL-TYPE-VARIANT-001",
            detail: index_text,
        });
        Ok(Value::Union(Box::new(UnionValue {
            type_name: type_text.into(),
            alternative: key,
            payload_classifier: Some(classifier.into()),
            payload: Some(Box::new(value)),
        })))
    }

    fn construct_union(
        &self,
        source: &SourceText,
        constructor: Span,
        payload: &Expression,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let name = source.slice(constructor);
        let (type_name, classifier) = self
            .union_constructor(name)
            .expect("preselected payload Union constructor");
        let value = self.evaluate_expression(source, payload, trace)?;
        if !value_has_classifier(&value, classifier) {
            return Err(diagnostic(
                source,
                "E-UNION-PAYLOAD-CLASSIFIER",
                payload.span(),
                format!(
                    "Union constructor `{name}` requires `{classifier}`, found `{}`",
                    structural_value_classifier(&value)
                ),
            ));
        }
        trace.record(TraceEvent {
            event: "union.constructed",
            rule: "TOPAL-TYPE-UNION-001",
            detail: name,
        });
        let result = Value::Union(Box::new(UnionValue {
            type_name: type_name.to_owned(),
            alternative: name.to_owned(),
            payload_classifier: Some(classifier.to_owned()),
            payload: Some(Box::new(value)),
        }));
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn invoke_anonymous_function(
        &self,
        function: &Value,
        arguments: Vec<Value>,
        call_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::AnonymousFunction(function) = function else {
            unreachable!("anonymous invocation is dispatched only for an anonymous function")
        };
        let AnonymousFunction {
            source,
            parameters,
            body,
            bindings,
        } = function.as_ref();
        if parameters.len() != arguments.len() {
            return Err(diagnostic(
                source,
                "E-ANONYMOUS-FUNCTION-ARITY",
                call_span,
                format!(
                    "anonymous function expects {} arguments, found {}",
                    parameters.len(),
                    arguments.len()
                ),
            ));
        }
        let mut invocation = self.clone();
        invocation.bindings = bindings.clone();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            invocation.bindings.insert(parameter.clone(), argument);
        }
        let detail = format!("arguments={}", parameters.len());
        trace.record(TraceEvent {
            event: "function.anonymous.called",
            rule: "TOPAL-FUNCTION-ANONYMOUS-001",
            detail: &detail,
        });
        invocation.evaluate_expression(source, body, trace)
    }

    fn capture_anonymous_function(
        &self,
        source: &SourceText,
        parameters: &[Span],
        body: &Expression,
        trace: &mut impl TraceSink,
    ) -> Value {
        let parameters = parameters
            .iter()
            .map(|parameter| source.slice(*parameter).to_owned())
            .collect::<Vec<_>>();
        let detail = format!("parameters={}", parameters.len());
        trace.record(TraceEvent {
            event: "function.anonymous.captured",
            rule: "TOPAL-FUNCTION-ANONYMOUS-001",
            detail: &detail,
        });
        Value::AnonymousFunction(Rc::new(AnonymousFunction {
            source: source.clone(),
            parameters,
            body: Box::new(body.clone()),
            bindings: self.bindings.clone(),
        }))
    }

    fn evaluate_list_insert_at(
        &self,
        source: &SourceText,
        list: Value,
        boundary: Option<&Expression>,
        inserted: Option<&Expression>,
        operation_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Some(boundary) = boundary else {
            return Err(diagnostic(
                source,
                "E-EXPECTED-OPERAND",
                operation_span,
                "expected a boundary after insert-at",
            ));
        };
        let Some(inserted) = inserted else {
            return Err(diagnostic(
                source,
                "E-EXPECTED-OPERAND",
                boundary.span(),
                "expected a value or List after the insertion boundary",
            ));
        };
        let boundary_value = self.evaluate_expression(source, boundary, trace)?;
        let inserted_value = self.evaluate_expression(source, inserted, trace)?;
        apply_list_insert_at(
            source,
            list,
            boundary_value,
            boundary.span(),
            inserted_value,
            inserted.span(),
            trace,
        )
    }

    #[allow(clippy::too_many_lines)] // Collection laws remain explicit in one isolated frame.
    fn evaluate_list_higher_order(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [collection, Expression::Identifier(operation_span), function] = items
            && matches!(
                source.slice(*operation_span),
                "map" | "select" | "remove-indexes" | "remove-values"
            )
        {
            return (|| {
                let collection_span = collection.span();
                let collection = self.evaluate_expression(source, collection, trace)?;
                let Value::List {
                    element_classifier,
                    entries,
                } = collection
                else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECTION-OPERATION-SOURCE",
                        collection_span,
                        format!(
                            "{} requires a homogeneous collection",
                            source.slice(*operation_span)
                        ),
                    ));
                };
                let function_span = function.span();
                let function = self.evaluate_expression(source, function, trace)?;
                let operation = source.slice(*operation_span);
                let mut output = Vec::new();
                for (index, entry) in entries.into_iter().enumerate() {
                    let input = entry.clone();
                    let argument = if operation == "remove-indexes" {
                        Value::Int(BigInt::from(index))
                    } else {
                        entry
                    };
                    let transformed =
                        self.invoke_anonymous_function(&function, vec![argument], span, trace)?;
                    if matches!(operation, "select" | "remove-indexes" | "remove-values") {
                        let Value::Boolean(retain) = transformed else {
                            return Err(diagnostic(
                                source,
                                "E-SELECT-PREDICATE-RESULT",
                                function_span,
                                format!("{operation} predicate must return Boolean"),
                            ));
                        };
                        if retain == (operation == "select") {
                            output.push(input);
                        }
                    } else {
                        output.push(transformed);
                    }
                }
                let output_classifier = if operation != "map" || output.is_empty() {
                    element_classifier
                } else {
                    let classifier = structural_value_classifier(&output[0]);
                    if output
                        .iter()
                        .any(|value| structural_value_classifier(value) != classifier)
                    {
                        return Err(diagnostic(
                            source,
                            "E-MAP-RESULT-CLASSIFIER",
                            function_span,
                            "map transformation returned values with different classifiers",
                        ));
                    }
                    classifier
                };
                let selection = format!("root.{operation}(List {output_classifier})");
                trace.record(TraceEvent {
                    event: "operator.selected",
                    rule: "TOPAL-TYPE-CALL-001",
                    detail: &selection,
                });
                trace.record(TraceEvent {
                    event: match operation {
                        "map" => "list.mapped",
                        "select" => "list.selected",
                        _ => "list.entries.removed",
                    },
                    rule: match operation {
                        "map" => "TOPAL-COLLECTION-MAP-001",
                        "select" => "TOPAL-COLLECTION-SELECT-001",
                        "remove-indexes" => "TOPAL-LIST-REMOVE-INDEXES-001",
                        "remove-values" => "TOPAL-LIST-REMOVE-VALUES-001",
                        _ => unreachable!("known higher-order List operation"),
                    },
                    detail: &output_classifier,
                });
                let result = Value::List {
                    element_classifier: output_classifier,
                    entries: output,
                };
                self.checkpoint(trace, Some(&result), Some(span));
                Ok(result)
            })();
        }
        if let [
            collection,
            Expression::Identifier(operation),
            initial,
            function,
        ] = items
            && source.slice(*operation) == "fold"
        {
            return (|| {
                let collection_span = collection.span();
                let collection = self.evaluate_expression(source, collection, trace)?;
                let Value::List { entries, .. } = collection else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECTION-OPERATION-SOURCE",
                        collection_span,
                        "fold requires an ordered homogeneous collection",
                    ));
                };
                let mut state = self.evaluate_expression(source, initial, trace)?;
                let expected = structural_value_classifier(&state);
                let function_span = function.span();
                let function = self.evaluate_expression(source, function, trace)?;
                for entry in entries {
                    let transformed = self.invoke_anonymous_function(
                        &function,
                        vec![state.clone(), entry],
                        span,
                        trace,
                    )?;
                    state = match transformed {
                        Value::Continue(next) => *next,
                        Value::Finish(result) => {
                            let result = *result;
                            if !value_has_classifier(&result, &expected) {
                                return Err(diagnostic(
                                    source,
                                    "E-FOLD-FINISH-CLASSIFIER",
                                    function_span,
                                    format!("Finish result must satisfy `{expected}`"),
                                ));
                            }
                            trace.record(TraceEvent {
                                event: "traversal.finished",
                                rule: "TOPAL-EXEC-TRAVERSAL-CONTROL-001",
                                detail: "fold",
                            });
                            return Ok(result);
                        }
                        value => value,
                    };
                    if !value_has_classifier(&state, &expected) {
                        return Err(diagnostic(
                            source,
                            "E-FOLD-STATE-CLASSIFIER",
                            function_span,
                            format!("fold step must preserve state classifier `{expected}`"),
                        ));
                    }
                }
                trace.record(TraceEvent {
                    event: "list.folded",
                    rule: "TOPAL-COLLECTION-FOLD-001",
                    detail: &expected,
                });
                self.checkpoint(trace, Some(&state), Some(span));
                Ok(state)
            })();
        }
        unreachable!("higher-order List operation is preselected by its application shape")
    }

    #[allow(clippy::too_many_lines)] // Collector spellings and their distinct laws remain auditable together.
    fn evaluate_list_materialization(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [Expression::Identifier(operation), pairs] = items
            && source.slice(*operation) == "unzip"
        {
            let pairs_span = pairs.span();
            let pairs = self.evaluate_expression(source, pairs, trace)?;
            return apply_list_unzip(source, pairs, pairs_span, trace);
        }
        if let [Expression::Identifier(operation), collection] = items
            && source.slice(*operation) == "collect"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            if let Value::IterateGenerator { .. } = value {
                return self.collect_iterate_generator(
                    source,
                    value,
                    collection.span(),
                    span,
                    trace,
                );
            }
            if let Value::UnfoldGenerator { .. } = value {
                return self.collect_unfold_generator(
                    source,
                    value,
                    collection.span(),
                    span,
                    trace,
                );
            }
            if matches!(value, Value::List { .. }) {
                trace.record(TraceEvent {
                    event: "list.collected",
                    rule: "TOPAL-COLLECTION-COLLECT-LIST-001",
                    detail: "List",
                });
                return Ok(value);
            }
            return Err(diagnostic(
                source,
                "E-COLLECT-SOURCE",
                collection.span(),
                "unary collect requires a finite homogeneous traversal",
            ));
        }
        if let [
            collection,
            Expression::Identifier(operation),
            Expression::Identifier(target),
        ] = items
            && source.slice(*operation) == "collect"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            if source.slice(*target) == "Array" {
                let Value::List {
                    element_classifier,
                    entries,
                } = value
                else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECT-ARRAY-SOURCE",
                        collection.span(),
                        "Array collection requires a finite List",
                    ));
                };
                trace.record(TraceEvent {
                    event: "array.collected",
                    rule: "TOPAL-ARRAY-COLLECT-001",
                    detail: &format!("count={}", entries.len()),
                });
                return Ok(Value::Array {
                    element_classifier,
                    entries,
                });
            }
            if source.slice(*target) != "String" {
                return Err(diagnostic(
                    source,
                    "E-COLLECT-TARGET",
                    *target,
                    "implemented collectors are Array and String",
                ));
            }
            let Value::List { entries, .. } = value else {
                return Err(diagnostic(
                    source,
                    "E-COLLECT-SOURCE",
                    collection.span(),
                    "String collection requires a finite List of Character or String entries",
                ));
            };
            let mut text = String::new();
            for entry in entries {
                let Value::String(fragment) = entry else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECT-STRING-ENTRY",
                        collection.span(),
                        "String collection requires Character or String entries",
                    ));
                };
                text.push_str(&fragment);
            }
            trace.record(TraceEvent {
                event: "string.collected",
                rule: "TOPAL-COLLECTION-COLLECT-STRING-001",
                detail: "String",
            });
            return Ok(Value::String(text));
        }
        if let [Expression::Identifier(operation), collection] = items
            && matches!(source.slice(*operation), "collect-set" | "collect-bag")
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            return collect_unordered(
                source,
                source.slice(*operation),
                value,
                collection.span(),
                trace,
            );
        }
        if let [
            Expression::Identifier(operation),
            collection,
            Expression::Identifier(resolving),
            Expression::Identifier(policy),
        ] = items
            && source.slice(*operation) == "collect-map"
            && source.slice(*resolving) == "resolving"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            return collect_map(
                source,
                value,
                source.slice(*policy),
                collection.span(),
                trace,
            );
        }
        if let [
            left_with_default,
            Expression::Identifier(operation),
            right_with_default,
        ] = items
            && source.slice(*operation) == "zip-longest"
        {
            let left = self.evaluate_expression(source, left_with_default, trace)?;
            let right = self.evaluate_expression(source, right_with_default, trace)?;
            return apply_list_zip_longest(source, left, right, span, trace);
        }
        Err(diagnostic(
            source,
            "E-COLLECTION-APPLICATION",
            span,
            "collection materialization does not match a declared operation form",
        ))
    }

    fn collect_iterate_generator(
        &self,
        source: &SourceText,
        generator: Value,
        source_span: Span,
        result_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::IterateGenerator {
            mut current,
            next,
            take_while,
            classifier,
        } = generator
        else {
            unreachable!("generated collection requires iterate generator")
        };
        let Some(predicate) = take_while else {
            return Err(diagnostic(
                source,
                "E-UNBOUNDED-GENERATOR-COLLECT",
                source_span,
                "collect requires a statically finite generated traversal",
            ));
        };
        let mut entries = Vec::new();
        loop {
            let accepted = self.invoke_anonymous_function(
                &predicate,
                vec![(*current).clone()],
                result_span,
                trace,
            )?;
            let Value::Boolean(accepted) = accepted else {
                return Err(diagnostic(
                    source,
                    "E-TAKE-WHILE-PREDICATE-RESULT",
                    source_span,
                    "take-while predicate must return Boolean",
                ));
            };
            if !accepted {
                break;
            }
            entries.push((*current).clone());
            let next_value =
                self.invoke_anonymous_function(&next, vec![*current], result_span, trace)?;
            if !value_has_classifier(&next_value, &classifier) {
                return Err(diagnostic(
                    source,
                    "E-ITERATE-NEXT-CLASSIFIER",
                    source_span,
                    format!("iterate next function must return `{classifier}`"),
                ));
            }
            *current = next_value;
        }
        trace.record(TraceEvent {
            event: "generator.collected",
            rule: "TOPAL-GENERATOR-COLLECT-001",
            detail: &classifier,
        });
        let value = Value::List {
            element_classifier: classifier,
            entries,
        };
        self.checkpoint(trace, Some(&value), Some(result_span));
        Ok(value)
    }

    fn collect_unfold_generator(
        &self,
        source: &SourceText,
        generator: Value,
        source_span: Span,
        result_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::UnfoldGenerator { mut seed, step } = generator else {
            unreachable!("unfold collection requires unfold generator")
        };
        let seed_classifier = structural_value_classifier(&seed);
        let mut element_classifier = None;
        let mut entries = Vec::new();
        loop {
            let result = self.invoke_anonymous_function(&step, vec![*seed], result_span, trace)?;
            let Value::Optional { payload, .. } = result else {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold step must return Optional (Yield, Seed)",
                ));
            };
            let Some(payload) = payload else {
                break;
            };
            let Value::Tuple(mut pair) = *payload else {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold Some payload must be a two-field positional product",
                ));
            };
            if pair.len() != 2 {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold Some payload must contain yielded value and next seed",
                ));
            }
            let next_seed = pair.pop().expect("two-field unfold payload");
            let yielded = pair.pop().expect("two-field unfold payload");
            if !value_has_classifier(&next_seed, &seed_classifier) {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-SEED-CLASSIFIER",
                    source_span,
                    format!("unfold next seed must satisfy `{seed_classifier}`"),
                ));
            }
            let yielded_classifier = structural_value_classifier(&yielded);
            if element_classifier
                .as_ref()
                .is_some_and(|expected| expected != &yielded_classifier)
            {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-YIELD-CLASSIFIER",
                    source_span,
                    "unfold step yielded inconsistent value classifiers",
                ));
            }
            element_classifier.get_or_insert(yielded_classifier);
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: "TOPAL-GENERATOR-UNFOLD-COLLECT-001",
                detail: &yielded.to_string(),
            });
            entries.push(yielded);
            *seed = next_seed;
        }
        let element_classifier = element_classifier.unwrap_or_else(|| "Value".into());
        trace.record(TraceEvent {
            event: "generator.collected",
            rule: "TOPAL-GENERATOR-UNFOLD-COLLECT-001",
            detail: &element_classifier,
        });
        let value = Value::List {
            element_classifier,
            entries,
        };
        self.checkpoint(trace, Some(&value), Some(result_span));
        Ok(value)
    }
}

fn known_enum_alternatives(session: &Session, type_name: &str) -> Option<BTreeSet<String>> {
    if type_name == "Comparison" {
        return Some(
            ["Less", "Equal", "Greater"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        );
    }
    session.enum_types.get(type_name).cloned()
}

impl Execution {
    #[allow(clippy::too_many_lines)] // Traversal keeps consumption, action, resume, and return auditable together.
    fn execute_foreach(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        source: &Expression,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let (generated, origin, returned, returned_classifier) = match source {
            Expression::Identifier(name) => {
                let name_text = self.source.slice(*name);
                let value = session.bindings.remove(name_text).ok_or_else(|| {
                    if session.consumed_names.contains(name_text) {
                        consumed_generator_diagnostic(&self.source, *name, name_text)
                    } else {
                        diagnostic(&self.source, "E-UNBOUND-NAME", *name, "name is not bound")
                    }
                })?;
                if let Value::SuspendedGenerator { .. } = value {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-GENERATOR-DECLARATION-001",
                        detail: name_text,
                    });
                    return self
                        .execute_suspended_foreach(session, trace, value, binding, body, span);
                }
                if let Value::IterateGenerator { .. } = value {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                        detail: name_text,
                    });
                    return self
                        .execute_iterate_foreach(session, trace, value, binding, body, span);
                }
                let (generated, origin, returned, returned_classifier) = match value {
                    Value::CharacterGenerator { generated, origin } => (
                        generated.into_iter().map(Value::String).collect(),
                        origin,
                        Value::Unit,
                        "Unit",
                    ),
                    Value::CharacterReturningGenerator {
                        generated,
                        returned,
                        origin,
                    } => (
                        generated.into_iter().map(Value::String).collect(),
                        origin,
                        Value::String(returned),
                        "Character",
                    ),
                    Value::List {
                        element_classifier,
                        entries,
                    } => {
                        session.bindings.insert(
                            name_text.to_owned(),
                            Value::List {
                                element_classifier,
                                entries: entries.clone(),
                            },
                        );
                        session.declared_names.insert(name_text.to_owned());
                        (entries, "root.List".into(), Value::Unit, "Unit")
                    }
                    _ => return Err(foreach_source_diagnostic(&self.source, source.span())),
                };
                if origin != "root.List" {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-STRING-CHARACTERS-GENERATOR-001",
                        detail: name_text,
                    });
                }
                (generated, origin, returned, returned_classifier)
            }
            Expression::Application { items, .. } => {
                let [Expression::Identifier(operation), text] = items.as_slice() else {
                    return Err(foreach_source_diagnostic(&self.source, source.span()));
                };
                if self.source.slice(*operation) != "characters" {
                    return Err(foreach_source_diagnostic(&self.source, source.span()));
                }
                let text_value = session.evaluate_expression(&self.source, text, trace)?;
                let Value::String(text_value) = text_value else {
                    return Err(diagnostic(
                        &self.source,
                        "E-CHARACTERS-OPERAND",
                        text.span(),
                        "characters requires a String operand",
                    ));
                };
                (
                    characters(&text_value)
                        .map(|character| Value::String(character.to_owned()))
                        .collect(),
                    "root.characters".to_owned(),
                    Value::Unit,
                    "Unit",
                )
            }
            _ => return Err(foreach_source_diagnostic(&self.source, source.span())),
        };
        let traversal_rule = if origin == "root.characters" {
            "TOPAL-STRING-CHARACTERS-FOREACH-001"
        } else if origin == "root.List" {
            "TOPAL-COLLECTION-FOREACH-001"
        } else {
            "TOPAL-GENERATOR-FOREACH-001"
        };
        let binding_name = self.source.slice(binding).to_owned();
        for entry in &generated {
            let mut iteration = session.clone();
            iteration
                .bindings
                .insert(binding_name.clone(), entry.clone());
            iteration.declared_names.insert(binding_name.clone());
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: traversal_rule,
                detail: &entry.to_string(),
            });
            let mut body_execution = Self {
                source: self.source.clone(),
                statements: body.to_vec(),
                cursor: 0,
                return_classifier: None,
            };
            loop {
                match body_execution.step(&mut iteration, trace)? {
                    ExecutionStep::Advanced { .. } => {}
                    ExecutionStep::Complete(Value::Unit) => break,
                    ExecutionStep::Complete(_) => {
                        return Err(diagnostic(
                            &self.source,
                            "E-FOREACH-ACTION-RESULT",
                            statement_span(body.last().expect("foreach body is nonempty")),
                            "foreach action must return Unit",
                        ));
                    }
                    ExecutionStep::Returned { .. } => {
                        unreachable!("foreach body has no function return context")
                    }
                }
            }
            trace.record(TraceEvent {
                event: "generator.resumed",
                rule: traversal_rule,
                detail: "Unit",
            });
        }
        trace.record(TraceEvent {
            event: "generator.returned",
            rule: generator_return_rule(
                &origin,
                generated.is_empty(),
                returned_classifier,
                traversal_rule,
            ),
            detail: returned_classifier,
        });
        Ok((returned, span))
    }

    fn execute_iterate_foreach(
        &self,
        session: &Session,
        trace: &mut impl TraceSink,
        generator: Value,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::IterateGenerator {
            mut current,
            next,
            take_while,
            classifier,
        } = generator
        else {
            unreachable!("iterate traversal requires iterate generator")
        };
        let Some(predicate) = take_while else {
            return Err(diagnostic(
                &self.source,
                "E-UNBOUNDED-GENERATOR-TRAVERSAL",
                span,
                "complete foreach traversal of unbounded iterate requires a stopping transformation",
            ));
        };
        let binding_name = self.source.slice(binding).to_owned();
        loop {
            let accepted = session.invoke_anonymous_function(
                &predicate,
                vec![(*current).clone()],
                span,
                trace,
            )?;
            let Value::Boolean(accepted) = accepted else {
                return Err(diagnostic(
                    &self.source,
                    "E-TAKE-WHILE-PREDICATE-RESULT",
                    span,
                    "take-while predicate must return Boolean",
                ));
            };
            if !accepted {
                trace.record(TraceEvent {
                    event: "generator.returned",
                    rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
                    detail: "Unit",
                });
                return Ok((Value::Unit, span));
            }
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                detail: &current.to_string(),
            });
            let mut iteration = session.clone();
            iteration
                .bindings
                .insert(binding_name.clone(), (*current).clone());
            iteration.declared_names.insert(binding_name.clone());
            let mut body_execution = Self {
                source: self.source.clone(),
                statements: body.to_vec(),
                cursor: 0,
                return_classifier: None,
            };
            loop {
                match body_execution.step(&mut iteration, trace)? {
                    ExecutionStep::Advanced { .. } => {}
                    ExecutionStep::Complete(Value::Unit) => break,
                    ExecutionStep::Complete(_) => {
                        return Err(diagnostic(
                            &self.source,
                            "E-FOREACH-ACTION-RESULT",
                            statement_span(body.last().expect("foreach body is nonempty")),
                            "foreach action must return Unit",
                        ));
                    }
                    ExecutionStep::Returned { .. } => {
                        unreachable!("foreach body has no function return context")
                    }
                }
            }
            let next_value =
                session.invoke_anonymous_function(&next, vec![*current], span, trace)?;
            if !value_has_classifier(&next_value, &classifier) {
                return Err(diagnostic(
                    &self.source,
                    "E-ITERATE-NEXT-CLASSIFIER",
                    span,
                    format!("iterate next function must return `{classifier}`"),
                ));
            }
            *current = next_value;
            trace.record(TraceEvent {
                event: "generator.resumed",
                rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                detail: "Unit",
            });
        }
    }

    #[allow(clippy::too_many_lines)] // State restoration and suspension order remain explicit and auditable.
    fn execute_suspended_foreach(
        &self,
        session: &Session,
        trace: &mut impl TraceSink,
        mut generator: Value,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::SuspendedGenerator {
            source,
            body: generator_body,
            ref mut cursor,
            ref mut bindings,
            ref mut scope_state,
            ref mut pending_yield,
            ref mut resume_binding,
            ref mut returned,
            yield_classifier,
            return_classifier,
            origin,
            ref mut task_state,
            ref task_owner,
        } = generator
        else {
            unreachable!("caller selects a suspended generator")
        };
        let binding_name = self.source.slice(binding).to_owned();
        let mut yielded_any = pending_yield.is_some();
        loop {
            if let Some(yielded) = pending_yield.take() {
                yielded_any = true;
                let detail = yielded.to_string();
                trace.record(TraceEvent {
                    event: "generator.yielded",
                    rule: "TOPAL-GENERATOR-FOREACH-001",
                    detail: &detail,
                });
                let mut iteration = session.clone();
                iteration.bindings.insert(binding_name.clone(), *yielded);
                iteration.declared_names.insert(binding_name.clone());
                let mut action = Self {
                    source: self.source.clone(),
                    statements: body.to_vec(),
                    cursor: 0,
                    return_classifier: None,
                };
                loop {
                    match action.step(&mut iteration, trace)? {
                        ExecutionStep::Advanced { .. } => {}
                        ExecutionStep::Complete(Value::Unit) => break,
                        ExecutionStep::Complete(_) => {
                            return Err(diagnostic(
                                &self.source,
                                "E-FOREACH-ACTION-RESULT",
                                statement_span(body.last().expect("foreach body is nonempty")),
                                "foreach action must return Unit",
                            ));
                        }
                        ExecutionStep::Returned { .. } => unreachable!("foreach cannot return"),
                    }
                }
                trace.record(TraceEvent {
                    event: "generator.resumed",
                    rule: "TOPAL-GENERATOR-FOREACH-001",
                    detail: "Unit",
                });
                let mut scope = session.clone();
                scope.task_state = task_state.take();
                scope.bindings = std::mem::take(bindings);
                scope.functions = Box::new(std::mem::take(&mut scope_state.functions));
                scope.declared_names = std::mem::take(&mut scope_state.declared_names);
                scope.local_function_names = std::mem::take(&mut scope_state.local_function_names);
                scope.enum_types = std::mem::take(&mut scope_state.enum_types);
                if let Some(name) = resume_binding.take() {
                    scope.bindings.insert(name.clone(), Value::Unit);
                    scope.declared_names.insert(name.clone());
                    trace.record(TraceEvent {
                        event: "generator.resume.bound",
                        rule: "TOPAL-GENERATOR-RESUME-BINDING-001",
                        detail: &name,
                    });
                }
                let mut next_returned = returned.take().map(|value| *value);
                advance_custom_generator(
                    &source,
                    &generator_body,
                    cursor,
                    &mut scope,
                    pending_yield,
                    resume_binding,
                    &mut next_returned,
                    &yield_classifier,
                    &return_classifier,
                    origin.rsplit('.').next().unwrap_or(&origin),
                    trace,
                )?;
                **bindings = scope.bindings;
                scope_state.functions = *scope.functions;
                scope_state.declared_names = scope.declared_names;
                scope_state.local_function_names = scope.local_function_names;
                scope_state.enum_types = scope.enum_types;
                *task_state = scope.task_state;
                sync_stream_task_state(session, task_owner.as_deref(), task_state.as_ref());
                *returned = next_returned.map(Box::new);
                continue;
            }
            let value = returned.take().map_or(Value::Unit, |value| *value);
            trace.record(TraceEvent {
                event: "generator.returned",
                rule: generator_return_rule(
                    &origin,
                    !yielded_any,
                    &return_classifier,
                    "TOPAL-GENERATOR-FOREACH-001",
                ),
                detail: &return_classifier,
            });
            sync_stream_task_state(session, task_owner.as_deref(), task_state.as_ref());
            return Ok((value, span));
        }
    }

    fn execute_discard(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        span: Span,
        value: &Expression,
    ) -> Result<(Value, Span), Diagnostic> {
        session.evaluate_expression(&self.source, value, trace)?;
        trace.record(TraceEvent {
            event: "binding.discarded",
            rule: "TOPAL-SYN-BIND-001",
            detail: "_",
        });
        Ok((Value::Unit, cover(span, value.span())))
    }

    #[allow(clippy::too_many_lines)] // Declaration validation and trace setup stay auditable together.
    fn declare_task_implementation(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        name: Span,
        classifier: &Expression,
        declarations: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::TaskType(task_type) =
            session.evaluate_expression(&self.source, classifier, trace)?
        else {
            return Err(diagnostic(
                &self.source,
                "E-TASK-IMPLEMENTATION-TYPE",
                classifier.span(),
                "an indented implementation requires a specialized Task type",
            ));
        };
        let mut state_fields = Vec::new();
        let mut handler_session = session.clone();
        for declaration in declarations {
            match declaration {
                Statement::StateField {
                    name: field,
                    classifier,
                } => state_fields.push((
                    self.source.slice(*field).to_owned(),
                    self.source.slice(*classifier).to_owned(),
                )),
                Statement::Function {
                    name,
                    is_static,
                    parameters,
                    result,
                    effect_bound,
                    body,
                    span,
                } => {
                    self.declare_function(
                        &mut handler_session,
                        trace,
                        FunctionDeclaration {
                            name: *name,
                            is_static: *is_static,
                            parameters,
                            result: *result,
                            effect_bound: *effect_bound,
                            body,
                            span: *span,
                        },
                    )?;
                }
                Statement::Generator {
                    name,
                    parameters,
                    yielded,
                    resumed,
                    result,
                    body,
                    span,
                } => {
                    self.declare_generator(
                        &mut handler_session,
                        trace,
                        GeneratorDeclaration {
                            name: *name,
                            parameters,
                            yielded: *yielded,
                            resumed: *resumed,
                            result: *result,
                            body,
                            span: *span,
                        },
                    )?;
                }
                _ => {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-IMPLEMENTATION-MEMBER",
                        statement_span(declaration),
                        "task implementations initially contain state fields and handlers",
                    ));
                }
            }
        }
        let handlers: BTreeMap<String, Vec<UserFunction>> = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Statement::Function { name, .. } => {
                    let name = self.source.slice(*name).to_owned();
                    Some((
                        name.clone(),
                        handler_session.functions.get(&name).cloned().unwrap(),
                    ))
                }
                _ => None,
            })
            .collect();
        let streams: BTreeMap<String, Vec<UserGenerator>> = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Statement::Generator { name, .. } => {
                    let name = self.source.slice(*name).to_owned();
                    Some((
                        name.clone(),
                        handler_session.generators.get(&name).cloned().unwrap(),
                    ))
                }
                _ => None,
            })
            .collect();
        if !handlers.contains_key("start") {
            return Err(diagnostic(
                &self.source,
                "E-TASK-START-REQUIRED",
                name,
                "every task implementation requires a start handler",
            ));
        }
        for (handler_name, candidates) in &handlers {
            for handler in candidates {
                if matches!(handler_name.as_str(), "start" | "terminate") {
                    if result_success_classifier(&handler.result) == Some("Unit") {
                        return Err(diagnostic(
                            &self.source,
                            "E-TASK-START-RESULT",
                            name,
                            "start cannot return Result with Unit success",
                        ));
                    }
                    continue;
                }
                if handler.parameters.is_empty()
                    || handler.parameters.len() > 2
                    || handler.parameters[0].1 != "MessageContext"
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-HANDLER-SHAPE",
                        name,
                        format!(
                            "message handler `{handler_name}` requires MessageContext plus zero or one ordinary operand"
                        ),
                    ));
                }
                if handler.result != "Unit"
                    && result_success_classifier(&handler.result)
                        .is_none_or(|success| success == "Unit")
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-HANDLER-RESULT",
                        name,
                        format!(
                            "message handler `{handler_name}` must return Unit or Result with a non-Unit success value"
                        ),
                    ));
                }
            }
        }
        for (stream_name, candidates) in &streams {
            for stream in candidates {
                if stream.parameters.is_empty()
                    || stream.parameters.len() > 2
                    || stream.parameters[0].1 != "MessageContext"
                    || result_success_classifier(&stream.result).is_none()
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-STREAM-SHAPE",
                        name,
                        format!(
                            "stream handler `{stream_name}` requires MessageContext, at most one payload, and a Result final return"
                        ),
                    ));
                }
            }
        }
        let task_type = TaskTypeValue {
            name: classifier_name(&self.source, classifier),
            ..*task_type
        };
        let value = Value::TaskDefinition(Box::new(TaskDefinitionValue {
            name: self.source.slice(name).to_owned(),
            task_type,
            source: self.source.clone(),
            state_fields,
            handlers,
            streams,
        }));
        session
            .bindings
            .insert(self.source.slice(name).to_owned(), value.clone());
        session
            .declared_names
            .insert(self.source.slice(name).to_owned());
        trace.record(TraceEvent {
            event: "task.definition.declared",
            rule: "TOPAL-TASK-DEFINITION-001",
            detail: self.source.slice(name),
        });
        Ok((value, span))
    }

    #[allow(clippy::too_many_lines)] // Declaration validation and trace setup stay auditable together.
    fn declare_function(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: FunctionDeclaration<'_>,
    ) -> Result<(Value, Span), Diagnostic> {
        let FunctionDeclaration {
            name,
            is_static,
            parameters,
            result,
            effect_bound,
            body,
            span,
        } = declaration;
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text)
            && !session.local_function_names.contains(name_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        let result_text = self.source.slice(result);
        let effect_bound_text = effect_bound.map(|bound| self.source.slice(bound).to_owned());
        let mut generic_names = BTreeSet::new();
        for parameter in parameters {
            collect_generic_names(
                self.source.slice(parameter.classifier),
                &session.enum_types,
                &mut generic_names,
            );
            for field in &parameter.fields {
                collect_generic_names(
                    self.source.slice(field.classifier),
                    &session.enum_types,
                    &mut generic_names,
                );
            }
        }
        if !supported_generic_classifier(result_text, &generic_names, &session.enum_types)
            && !session.union_types.contains_key(result_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-RESULT-CLASSIFIER",
                result,
                "the result classifier is not supported by this interpreter subset",
            ));
        }
        validate_parameter_names(&self.source, parameters)?;
        let mut parameter_packages = BTreeMap::new();
        let parameters = parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                if !parameter.fields.is_empty() {
                    let fields = parameter
                        .fields
                        .iter()
                        .map(|field| {
                            let classifier = self.source.slice(field.classifier);
                            if !supported_value_classifier(classifier, &session.enum_types)
                                && !supported_generic_classifier(
                                    classifier,
                                    &generic_names,
                                    &session.enum_types,
                                )
                                && !session.union_types.contains_key(classifier)
                            {
                                return Err(diagnostic(
                                    &self.source,
                                    "E-UNSUPPORTED-PARAMETER-CLASSIFIER",
                                    field.classifier,
                                    "the packaged parameter classifier is not supported by this interpreter subset",
                                ));
                            }
                            Ok(UserParameterField {
                                name: self.source.slice(field.name).to_owned(),
                                classifier: classifier.to_owned(),
                                default: field.default.clone(),
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let classifier = format!(
                        "({})",
                        fields
                            .iter()
                            .map(|field| format!("{} is {}", field.name, field.classifier))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    parameter_packages.insert(index, fields);
                    return Ok((format!("$package{index}"), classifier));
                }
                let classifier = self.source.slice(parameter.classifier);
                if !supported_value_classifier(classifier, &session.enum_types)
                    && generic_capability_classifier(classifier).is_none()
                    && !generic_names.contains(classifier)
                    && !supported_generic_classifier(
                        classifier,
                        &generic_names,
                        &session.enum_types,
                    )
                    && !session.union_types.contains_key(classifier)
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-PARAMETER-CLASSIFIER",
                        parameter.classifier,
                        "the parameter classifier is not supported by this interpreter subset",
                    ));
                }
                Ok((
                    self.source.slice(parameter.name).to_owned(),
                    classifier.to_owned(),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if session.local_function_names.contains(name_text)
            && session.functions[name_text].iter().any(|function| {
                function.is_static == is_static
                    && function
                        .parameters
                        .iter()
                        .map(|(_, classifier)| classifier)
                        .eq(parameters.iter().map(|(_, classifier)| classifier))
            })
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-FUNCTION-OVERLOAD",
                name,
                "an overload with the same input classifiers and staticness already exists",
            ));
        }
        let direct_termination_rule = prove_euclidean_recursion(
            &self.source,
            name_text,
            &parameters,
            effect_bound_text.as_deref(),
            body,
        )
        .or_else(|| prove_int_recursion(&self.source, name_text, &parameters, body));
        let mutual_edge = direct_termination_rule
            .is_none()
            .then(|| prove_mutual_int_recursion_edge(&self.source, name_text, &parameters, body))
            .flatten();
        let recursion_target = mutual_edge.as_ref().map(|(target, _)| target.clone());
        let termination_rule =
            direct_termination_rule.or_else(|| mutual_edge.as_ref().map(|(_, rule)| *rule));
        let rule = function_rule(is_static, parameters.len());
        let function = UserFunction {
            source: self.source.clone(),
            is_static,
            parameters,
            parameter_packages,
            result: result_text.to_owned(),
            generic_names,
            effect_bound: effect_bound_text.clone(),
            body: body.to_vec(),
            bindings: session.bindings.clone(),
            termination_rule,
            recursion_target: recursion_target.clone(),
        };
        if session.local_function_names.contains(name_text) {
            session.functions.get_mut(name_text).unwrap().push(function);
        } else {
            session
                .functions
                .insert(name_text.to_owned(), vec![function]);
        }
        session.bindings.remove(name_text);
        session.declared_names.insert(name_text.to_owned());
        session.local_function_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "function.declared",
            rule,
            detail: name_text,
        });
        if let Some(effect_bound) = &effect_bound_text {
            trace.record(TraceEvent {
                event: "function.effect-bound.declared",
                rule: "TOPAL-FUNCTION-EFFECT-BOUND-001",
                detail: effect_bound,
            });
        }
        if result_success_classifier(result_text).is_some() {
            trace.record(TraceEvent {
                event: "function.result.contract",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: result_text,
            });
        }
        if let Some(termination_rule) = direct_termination_rule {
            trace.record(TraceEvent {
                event: "function.recursion.proven",
                rule: termination_rule,
                detail: name_text,
            });
        } else if let Some(target) = recursion_target {
            let detail = format!("{name_text}->{target}");
            trace.record(TraceEvent {
                event: "function.recursion.edge.candidate",
                rule: termination_rule.expect("a mutual edge has a termination rule"),
                detail: &detail,
            });
        }
        Ok((Value::Unit, span))
    }

    fn declare_generator(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: GeneratorDeclaration<'_>,
    ) -> Result<(Value, Span), Diagnostic> {
        let GeneratorDeclaration {
            name,
            parameters,
            yielded,
            resumed,
            result,
            body,
            span,
        } = declaration;
        if !session.call_stack.is_empty() {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-SCOPE",
                name,
                "the implemented generator subset requires a root-namespace declaration",
            ));
        }
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text) && !session.generators.contains_key(name_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        validate_parameter_names(&self.source, parameters)?;
        let parameters = parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    self.source.slice(parameter.classifier).to_owned(),
                )
            })
            .collect::<Vec<_>>();
        let yield_classifier = self.source.slice(yielded);
        let result_classifier = self.source.slice(result);
        if !parameters.iter().all(|(_, classifier)| {
            supported_generator_value_classifier(classifier, &session.enum_types)
        }) || !supported_generator_value_classifier(yield_classifier, &session.enum_types)
            || self.source.slice(resumed) != "Unit"
            || !supported_generator_value_classifier(result_classifier, &session.enum_types)
        {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-SIGNATURE",
                span,
                "the implemented generator subset requires supported scalar, Optional, Range, or declared enum input/yield/return classifiers and Unit resume",
            ));
        }
        if !supported_generator_body(&self.source, body) {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-BODY",
                span,
                "the implemented generator subset requires bindings, discarded computations, or yield statements followed by a final expression",
            ));
        }
        let overloads = session.generators.entry(name_text.to_owned()).or_default();
        if overloads.iter().any(|candidate| {
            candidate.parameters.len() == parameters.len()
                && candidate
                    .parameters
                    .iter()
                    .zip(&parameters)
                    .all(|((_, left), (_, right))| left == right)
        }) {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-GENERATOR-OVERLOAD",
                name,
                format!("generator overload `{name_text}` has the same input classifiers"),
            ));
        }
        overloads.push(UserGenerator {
            source: self.source.clone(),
            parameters,
            yielded: yield_classifier.to_owned(),
            result: result_classifier.to_owned(),
            body: body.to_vec(),
            bindings: session.bindings.clone(),
        });
        session.declared_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "generator.declared",
            rule: "TOPAL-GENERATOR-DECLARATION-001",
            detail: name_text,
        });
        let classifier = format!("Generator {yield_classifier} Unit {result_classifier}");
        trace.record(TraceEvent {
            event: "generator.classified",
            rule: "TOPAL-GENERATOR-DECLARATION-001",
            detail: &classifier,
        });
        Ok((Value::Unit, span))
    }

    /// Execute one source statement.
    ///
    /// # Errors
    ///
    /// Returns a name-resolution or evaluation diagnostic at the failing step.
    #[allow(clippy::too_many_lines)] // Statement dispatch remains explicit and exhaustively typed.
    pub fn step(
        &mut self,
        session: &mut Session,
        trace: &mut impl TraceSink,
    ) -> Result<ExecutionStep, Diagnostic> {
        let statement = &self.statements[self.cursor];
        let (value, span) = match statement {
            Statement::LanguageSelection {
                version,
                features,
                span,
            } => {
                let requested = self
                    .source
                    .slice(*version)
                    .parse::<LanguageVersion>()
                    .map_err(|message| {
                        diagnostic(&self.source, "E-LANGUAGE-VERSION", *version, message)
                    })?;
                if requested != LanguageVersion::DESIGN_0 {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-LANGUAGE-VERSION",
                        *version,
                        format!(
                            "language version `{requested}` is not supported; highest supported version is `{}`",
                            LanguageVersion::DESIGN_0
                        ),
                    ));
                }
                session.language_version = requested;
                session.language_features = features
                    .iter()
                    .map(|feature| self.source.slice(*feature).to_owned())
                    .collect();
                let feature_names = features
                    .iter()
                    .map(|feature| self.source.slice(*feature))
                    .collect::<Vec<_>>()
                    .join(",");
                let detail = if feature_names.is_empty() {
                    requested.to_string()
                } else {
                    format!("{requested};features={feature_names}")
                };
                trace.record(TraceEvent {
                    event: "language.context.selected",
                    rule: "TOPAL-SYN-GRAMMAR-001",
                    detail: &detail,
                });
                (Value::Unit, *span)
            }
            Statement::LibrarySelection {
                name,
                version,
                span,
            } => {
                let library_name = self.source.slice(*name);
                let requested = self.source.slice(*version);
                if library_name != "std" {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-LIBRARY",
                        *name,
                        format!("library `{library_name}` is not available"),
                    ));
                }
                if requested != "v0.1" {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-LIBRARY-VERSION",
                        *version,
                        format!(
                            "standard-library version `{requested}` is not supported; available version is `v0.1`"
                        ),
                    ));
                }
                session.declared_libraries.insert(library_name.to_owned());
                trace.record(TraceEvent {
                    event: "library.dependency.selected",
                    rule: "TOPAL-SYN-LIBRARY-001",
                    detail: "std@v0.1",
                });
                (Value::Unit, *span)
            }
            Statement::Published {
                declaration, span, ..
            } => {
                let published_name = declaration_name(&self.source, declaration);
                trace.record(TraceEvent {
                    event: "namespace.member.published",
                    rule: "TOPAL-NAMESPACE-ROOT-001",
                    detail: self.source.slice(*span),
                });
                let outcome = self.execute_published(session, trace, declaration)?;
                if let Some(name) = published_name {
                    session.published_names.insert(name.to_owned());
                }
                match outcome {
                    ExecutionStep::Complete(value) | ExecutionStep::Advanced { value, .. } => {
                        (value, *span)
                    }
                    ExecutionStep::Returned { value, span } => {
                        return Ok(ExecutionStep::Returned { value, span });
                    }
                }
            }
            Statement::DiagnosticControl { span, .. } => (Value::Unit, *span),
            Statement::Binding {
                name,
                classifier,
                value,
            } => match self.execute_binding(session, trace, *name, *classifier, value)? {
                BindingOutcome::Bound(value, span) => (value, span),
                BindingOutcome::Returned(value, span) => {
                    return Ok(ExecutionStep::Returned { value, span });
                }
            },
            Statement::Implementation {
                name,
                classifier,
                declarations,
                span,
            } => self.declare_task_implementation(
                session,
                trace,
                *name,
                classifier,
                declarations,
                *span,
            )?,
            Statement::StateField { name, .. } => {
                return Err(diagnostic(
                    &self.source,
                    "E-STATE-FIELD-CONTEXT",
                    *name,
                    "a task state field is valid only inside a task implementation",
                ));
            }
            Statement::ContextAssignment { name, value, span } => {
                let value = session.evaluate_expression(&self.source, value, trace)?;
                let Some(state) = session.task_state.as_mut() else {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-STATE-CONTEXT",
                        *name,
                        "task state replacement requires an executing task handler",
                    ));
                };
                if !state.contains_key(self.source.slice(*name)) {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNKNOWN-TASK-STATE",
                        *name,
                        "task implementation declares no such state field",
                    ));
                }
                state.insert(self.source.slice(*name).to_owned(), value);
                trace.record(TraceEvent {
                    event: "task.state.replaced",
                    rule: "TOPAL-TASK-STATE-001",
                    detail: self.source.slice(*name),
                });
                (Value::Unit, *span)
            }
            Statement::Function {
                name,
                is_static,
                parameters,
                result,
                effect_bound,
                body,
                span,
            } => self.declare_function(
                session,
                trace,
                FunctionDeclaration {
                    name: *name,
                    is_static: *is_static,
                    parameters,
                    result: *result,
                    effect_bound: *effect_bound,
                    body,
                    span: *span,
                },
            )?,
            Statement::Generator {
                name,
                parameters,
                yielded,
                resumed,
                result,
                body,
                span,
            } => self.declare_generator(
                session,
                trace,
                GeneratorDeclaration {
                    name: *name,
                    parameters,
                    yielded: *yielded,
                    resumed: *resumed,
                    result: *result,
                    body,
                    span: *span,
                },
            )?,
            Statement::Union {
                name,
                alternatives,
                span,
            } => declare_union(&self.source, session, *name, alternatives, *span, trace)?,
            Statement::Interface {
                name,
                functions,
                span,
            } => {
                let name_text = self.source.slice(*name);
                if session.declared_names.contains(name_text) {
                    return Err(diagnostic(
                        &self.source,
                        "E-DUPLICATE-DECLARATION",
                        *name,
                        format!("`{name_text}` is already declared"),
                    ));
                }
                let mut operations = BTreeMap::new();
                for function in functions {
                    let operation = self.source.slice(function.name).to_owned();
                    if operations
                        .insert(
                            operation.clone(),
                            (
                                function
                                    .parameters
                                    .iter()
                                    .map(|parameter| {
                                        self.source.slice(parameter.classifier).to_owned()
                                    })
                                    .collect(),
                                self.source.slice(function.result).to_owned(),
                            ),
                        )
                        .is_some()
                    {
                        return Err(diagnostic(
                            &self.source,
                            "E-DUPLICATE-INTERFACE-OPERATION",
                            function.name,
                            format!("interface operation `{operation}` is declared twice"),
                        ));
                    }
                }
                let value = Value::Interface(Box::new(InterfaceValue {
                    name: name_text.to_owned(),
                    functions: operations,
                }));
                session.bindings.insert(name_text.to_owned(), value.clone());
                session.declared_names.insert(name_text.to_owned());
                trace.record(TraceEvent {
                    event: "interface.declared",
                    rule: "TOPAL-INTERFACE-SHAPE-001",
                    detail: name_text,
                });
                (value, *span)
            }
            Statement::InterfaceImplementation {
                interface,
                declarations,
                span,
            } => {
                let interface_name = self.source.slice(*interface);
                let Some(Value::Interface(shape)) = session.bindings.get(interface_name) else {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNKNOWN-INTERFACE",
                        *interface,
                        format!("`{interface_name}` is not a declared interface"),
                    ));
                };
                let supplied = declarations
                    .iter()
                    .map(|declaration| match declaration {
                        Statement::Function {
                            name,
                            parameters,
                            result,
                            ..
                        } => Some((
                            self.source.slice(*name).to_owned(),
                            (
                                parameters
                                    .iter()
                                    .map(|parameter| {
                                        self.source.slice(parameter.classifier).to_owned()
                                    })
                                    .collect::<Vec<_>>(),
                                self.source.slice(*result).to_owned(),
                            ),
                        )),
                        _ => None,
                    })
                    .collect::<Option<BTreeMap<_, _>>>()
                    .ok_or_else(|| {
                        diagnostic(
                            &self.source,
                            "E-INTERFACE-IMPLEMENTATION",
                            *span,
                            "an interface implementation contains function declarations only",
                        )
                    })?;
                if supplied != shape.functions {
                    return Err(diagnostic(
                        &self.source,
                        "E-INTERFACE-IMPLEMENTATION",
                        *span,
                        "implementation operations must exactly match the interface shapes",
                    ));
                }
                for declaration in declarations {
                    let mut nested = Execution {
                        source: self.source.clone(),
                        statements: vec![declaration.clone()],
                        cursor: 0,
                        return_classifier: None,
                    };
                    let _ = nested.step(session, trace)?;
                }
                trace.record(TraceEvent {
                    event: "interface.implemented",
                    rule: "TOPAL-INTERFACE-IMPLEMENTATION-001",
                    detail: interface_name,
                });
                (Value::Unit, *span)
            }
            Statement::Foreach {
                result,
                source,
                binding,
                body,
                span,
            } => {
                let (value, span) =
                    self.execute_foreach(session, trace, source, *binding, body, *span)?;
                if let Some((result, classifier)) = result {
                    let name = self.source.slice(*result);
                    if session.declared_names.contains(name) {
                        return Err(diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            *result,
                            "name is already bound in this scope",
                        ));
                    }
                    if let Some(classifier) = classifier {
                        let expected = self.source.slice(*classifier);
                        if !value_has_classifier(&value, expected) {
                            let found = structural_value_classifier(&value);
                            return Err(diagnostic(
                                &self.source,
                                "E-FOREACH-RESULT-CLASSIFIER",
                                *classifier,
                                format!(
                                    "foreach returned `{found}`, but binding `{name}` requires `{expected}`"
                                ),
                            )
                            .with_help(format!(
                                "use classifier `{found}` here or traverse a generator returning `{expected}`"
                            )));
                        }
                    }
                    session.bindings.insert(name.to_owned(), value.clone());
                    session.declared_names.insert(name.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.foreach.result.bound",
                        rule: "TOPAL-GENERATOR-FOREACH-RESULT-001",
                        detail: name,
                    });
                }
                (value, span)
            }
            Statement::Discard { span, value } => {
                self.execute_discard(session, trace, *span, value)?
            }
            Statement::Return { keyword, value } => {
                if self.return_classifier.is_none() {
                    return Err(diagnostic(
                        &self.source,
                        "E-RETURN-OUTSIDE-FUNCTION",
                        *keyword,
                        "`return` is valid only inside a function body",
                    ));
                }
                let span = cover(*keyword, value.span());
                let value = evaluate_expression_with_optional_context(
                    &self.source,
                    session,
                    value,
                    self.return_classifier.as_deref(),
                    trace,
                )?;
                let classifier = structural_value_classifier(&value);
                trace.record(TraceEvent {
                    event: "function.return.explicit",
                    rule: "TOPAL-FUNCTION-RETURN-001",
                    detail: &classifier,
                });
                session.checkpoint(trace, Some(&value), Some(span));
                self.cursor = self.statements.len();
                return Ok(ExecutionStep::Returned { value, span });
            }
            Statement::Expression(expression) => {
                let value = evaluate_expression_with_optional_context(
                    &self.source,
                    session,
                    expression,
                    self.return_classifier.as_deref(),
                    trace,
                )?;
                if self.cursor + 1 != self.statements.len() && value != Value::Unit {
                    return Err(diagnostic(
                        &self.source,
                        "E-DISCARDED-VALUE",
                        expression.span(),
                        "a non-final expression value cannot be discarded",
                    ));
                }
                consume_generator_argument(&self.source, session, expression);
                (value, expression.span())
            }
        };
        self.cursor += 1;
        if self.cursor == self.statements.len() {
            record_result(trace, &value);
            session.checkpoint(trace, Some(&value), Some(span));
            Ok(ExecutionStep::Complete(value))
        } else {
            session.checkpoint(trace, Some(&value), Some(span));
            Ok(ExecutionStep::Advanced { value, span })
        }
    }

    #[inline(never)]
    fn execute_published(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: &Statement,
    ) -> Result<ExecutionStep, Diagnostic> {
        let mut published = Self {
            source: self.source.clone(),
            statements: vec![declaration.clone()],
            cursor: 0,
            return_classifier: self.return_classifier.clone(),
        };
        published.step(session, trace)
    }

    #[allow(clippy::too_many_lines)] // Declaration specializations remain ordered before ordinary projection.
    fn execute_binding(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        name: Span,
        classifier: Option<Span>,
        initializer: &Expression,
    ) -> Result<BindingOutcome, Diagnostic> {
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text) {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        if let Some((value, span)) = declare_enum(&self.source, name, initializer, session, trace)?
        {
            return Ok(BindingOutcome::Bound(value, span));
        }
        if let Some((value, span)) =
            declare_variant(&self.source, name, initializer, session, trace)
        {
            return Ok(BindingOutcome::Bound(value, span));
        }
        let mut evaluated =
            evaluate_binding_initializer(&self.source, session, initializer, classifier, trace)?;
        consume_generator_argument(&self.source, session, initializer);
        if let Some(classifier) = classifier {
            let classifier_text = self.source.slice(classifier);
            evaluated = narrow_rational_to_int(
                &self.source,
                initializer,
                evaluated,
                classifier_text,
                self.return_classifier.as_deref(),
                trace,
            )?;
            if matches!(evaluated, Value::Error { .. }) {
                let Some(return_classifier) = &self.return_classifier else {
                    return Err(diagnostic(
                        &self.source,
                        "E-RESULT-PROJECTION-OUTSIDE-FUNCTION",
                        initializer.span(),
                        "a failed Result cannot propagate from top-level execution",
                    ));
                };
                if result_success_classifier(return_classifier).is_none() {
                    return Err(diagnostic(
                        &self.source,
                        "E-RESULT-PROJECTION-INFALLIBLE",
                        initializer.span(),
                        format!(
                            "cannot propagate a failed Result from a function returning `{return_classifier}`"
                        ),
                    ));
                }
                trace.record(TraceEvent {
                    event: "result.error.projected",
                    rule: "TOPAL-TYPE-RESULT-PROJECT-001",
                    detail: name_text,
                });
                return Ok(BindingOutcome::Returned(evaluated, initializer.span()));
            }
            if !value_has_classifier(&evaluated, classifier_text) {
                if classifier_text == "Character"
                    && let Value::String(text) = &evaluated
                {
                    let count = character_count(text);
                    return Err(diagnostic(
                        &self.source,
                        "E-CHARACTER-CLASSIFIER",
                        initializer.span(),
                        format!(
                            "Character requires exactly one user-perceived character, but this String contains {count}"
                        ),
                    ));
                }
                return Err(diagnostic(
                    &self.source,
                    "E-BINDING-CLASSIFIER",
                    initializer.span(),
                    format!("initializer does not satisfy `{classifier_text}`"),
                ));
            }
            trace.record(TraceEvent {
                event: "result.success.projected",
                rule: "TOPAL-TYPE-RESULT-PROJECT-001",
                detail: name_text,
            });
        }
        if let Value::Constraint(constraint) = &mut evaluated {
            constraint.name = Some(name_text.to_owned());
        }
        if let Value::ModularType(kind) = &mut evaluated {
            kind.name = Some(name_text.to_owned());
        }
        session.bindings.insert(name_text.to_owned(), evaluated);
        session.functions.remove(name_text);
        session.declared_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "binding.bind",
            rule: "TOPAL-SYN-BIND-001",
            detail: name_text,
        });
        Ok(BindingOutcome::Bound(
            Value::Unit,
            cover(name, initializer.span()),
        ))
    }
}

fn evaluate_binding_initializer(
    source: &SourceText,
    session: &mut Session,
    initializer: &Expression,
    classifier: Option<Span>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    evaluate_expression_with_optional_context(
        source,
        session,
        initializer,
        classifier.map(|classifier| source.slice(classifier)),
        trace,
    )
}

fn evaluate_expression_with_optional_context(
    source: &SourceText,
    session: &mut Session,
    expression: &Expression,
    expected_classifier: Option<&str>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let Some(element_classifier) = expected_classifier.and_then(list_element_classifier)
        && let Some(list) =
            evaluate_list_expression(source, session, expression, element_classifier, trace)?
    {
        return Ok(list);
    }
    let contextual_none = expected_classifier
        .and_then(optional_payload_classifier)
        .filter(
            |_| matches!(expression, Expression::Identifier(span) if source.slice(*span) == "None"),
        );
    let Some(payload_classifier) = contextual_none else {
        return session.evaluate_expression(source, expression, trace);
    };
    let payload_classifier = session
        .generic_types
        .get(payload_classifier)
        .map_or(payload_classifier, String::as_str);
    trace.record(TraceEvent {
        event: "optional.none.constructed",
        rule: "TOPAL-TYPE-OPTIONAL-CONTEXT-001",
        detail: payload_classifier,
    });
    Ok(Value::Optional {
        payload_classifier: payload_classifier.to_owned(),
        payload: None,
    })
}

fn evaluate_list_expression(
    source: &SourceText,
    session: &mut Session,
    expression: &Expression,
    element_classifier: &str,
    trace: &mut impl TraceSink,
) -> Result<Option<Value>, Diagnostic> {
    if matches!(expression, Expression::Identifier(span) if source.slice(*span) == "Empty") {
        trace.record(TraceEvent {
            event: "list.empty.constructed",
            rule: "TOPAL-TYPE-LIST-CONSTRUCT-001",
            detail: element_classifier,
        });
        return Ok(Some(Value::List {
            element_classifier: element_classifier.to_owned(),
            entries: Vec::new(),
        }));
    }
    let Expression::Application { items, span } = expression else {
        return Ok(None);
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Ok(None);
    };
    if source.slice(*constructor) != "Entry" {
        return Ok(None);
    }
    if fields.len() != 2 || fields.iter().any(|field| field.label.is_some()) {
        return Err(diagnostic(
            source,
            "E-LIST-ENTRY-SHAPE",
            *span,
            "Entry requires exactly `(value, remaining-list)`",
        )
        .with_help("write `Entry ( value, remaining-list )`"));
    }
    let entry = session.evaluate_expression(source, &fields[0].value, trace)?;
    if !value_has_classifier(&entry, element_classifier) {
        let found = structural_value_classifier(&entry);
        return Err(diagnostic(
            source,
            "E-LIST-ENTRY-CLASSIFIER",
            fields[0].value.span(),
            format!(
                "list entry has classifier `{found}`, but this list requires `{element_classifier}`"
            ),
        )
        .with_help(format!("use a `{element_classifier}` value for this entry")));
    }
    let Some(Value::List { mut entries, .. }) =
        evaluate_list_expression(source, session, &fields[1].value, element_classifier, trace)?
    else {
        return Err(diagnostic(
            source,
            "E-LIST-REMAINDER",
            fields[1].value.span(),
            "Entry requires another List as its remaining value",
        )
        .with_help("end the constructor chain with `Empty`"));
    };
    entries.insert(0, entry);
    trace.record(TraceEvent {
        event: "list.entry.constructed",
        rule: "TOPAL-TYPE-LIST-CONSTRUCT-001",
        detail: element_classifier,
    });
    Ok(Some(Value::List {
        element_classifier: element_classifier.to_owned(),
        entries,
    }))
}

fn expression_is_closed(expression: &Expression) -> bool {
    match expression {
        Expression::Block { statements, .. } => {
            statements.iter().all(|statement| match statement {
                Statement::Binding { value, .. }
                | Statement::Discard { value, .. }
                | Statement::Return { value, .. }
                | Statement::Expression(value) => expression_is_closed(value),
                _ => false,
            })
        }
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Callable { .. } => true,
        Expression::Product { fields, .. } => fields
            .iter()
            .all(|field| expression_is_closed(&field.value)),
        Expression::Application { items, .. } => items.iter().all(expression_is_closed),
        Expression::AnonymousFunction { body, .. } => expression_is_closed(body),
        Expression::DecisionTable { .. }
        | Expression::Identifier(_)
        | Expression::ContextIdentifier(_)
        | Expression::Discard(_) => false,
    }
}

fn narrow_rational_to_int(
    source: &SourceText,
    initializer: &Expression,
    value: Value,
    classifier: &str,
    return_classifier: Option<&str>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if classifier != "Int" {
        return Ok(value);
    }
    let Value::Rational(value) = value else {
        return Ok(value);
    };
    if value.denom() != &BigInt::from(1) {
        if !expression_is_closed(initializer)
            && return_classifier.and_then(result_success_classifier) == Some("Int")
        {
            let position = source.position(initializer.span().start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-NUM-RATIONAL-INT-VALIDATE-001",
                detail: "root.Int(Rational);not-representable",
            });
            return Ok(Value::Error {
                domain: "root.Int(Rational)".to_owned(),
                code: "not-representable".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-RATIONAL-NOT-EXACT-INT",
            initializer.span(),
            format!(
                "exact Rational result has denominator {}, so it cannot satisfy Int",
                value.denom()
            ),
        ));
    }
    trace.record(TraceEvent {
        event: "conversion.applied",
        rule: if expression_is_closed(initializer) {
            "TOPAL-NUM-RATIONAL-INT-EXACT-001"
        } else {
            "TOPAL-NUM-RATIONAL-INT-VALIDATE-001"
        },
        detail: if expression_is_closed(initializer) {
            "Rational->Int:exact"
        } else {
            "Rational->Int:validated"
        },
    });
    Ok(Value::Int(value.numer().clone()))
}

fn construct_int(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Rational(value) = value else {
        if matches!(value, Value::Int(_)) {
            trace.record(TraceEvent {
                event: "numeric.int.constructed",
                rule: "TOPAL-NUM-INT-CONSTRUCT-001",
                detail: "Int->Int:identity",
            });
            return Ok(value);
        }
        return Err(diagnostic(
            source,
            "E-INT-CONSTRUCTOR-OPERAND",
            operand.span(),
            "Int construction requires an exact numeric operand",
        ));
    };
    if value.denom() == &BigInt::from(1) {
        trace.record(TraceEvent {
            event: "numeric.int.constructed",
            rule: "TOPAL-NUM-INT-CONSTRUCT-001",
            detail: "Rational->Int:exact",
        });
        return Ok(Value::Int(value.numer().clone()));
    }
    if expression_is_closed(operand) {
        return Err(diagnostic(
            source,
            "E-RATIONAL-NOT-EXACT-INT",
            operand.span(),
            format!(
                "exact Rational operand has denominator {}, so Int cannot represent it",
                value.denom()
            ),
        ));
    }
    let position = source.position(operand.span().start);
    trace.record(TraceEvent {
        event: "result.error.constructed",
        rule: "TOPAL-NUM-INT-CONSTRUCT-001",
        detail: "root.Int(Rational);not-representable",
    });
    Ok(Value::Error {
        domain: "root.Int(Rational)".to_owned(),
        code: "not-representable".to_owned(),
        line: position.line,
        column: position.column,
    })
}

fn construct_nat(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Int(value) = value else {
        return Err(diagnostic(
            source,
            "E-NAT-CONSTRUCTOR-OPERAND",
            operand.span(),
            "Nat construction requires an Int operand",
        ));
    };
    if value >= BigInt::from(0) {
        trace.record(TraceEvent {
            event: "numeric.nat.constructed",
            rule: "TOPAL-NUM-NAT-CONSTRUCT-001",
            detail: "Int->Nat:nonnegative",
        });
        return Ok(Value::Int(value));
    }
    if expression_is_closed(operand) {
        return Err(diagnostic(
            source,
            "E-NAT-OUT-OF-RANGE",
            operand.span(),
            "a negative Int is outside the Nat constraint",
        ));
    }
    let position = source.position(operand.span().start);
    trace.record(TraceEvent {
        event: "result.error.constructed",
        rule: "TOPAL-NUM-NAT-CONSTRUCT-001",
        detail: "root.Nat(Int);out-of-range",
    });
    Ok(Value::Error {
        domain: "root.Nat(Int)".to_owned(),
        code: "out-of-range".to_owned(),
        line: position.line,
        column: position.column,
    })
}

fn construct_rational(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let Value::Int(value) = value {
        trace.record(TraceEvent {
            event: "numeric.rational.constructed",
            rule: "TOPAL-NUM-INT-RATIONAL-CONVERT-001",
            detail: "Int->Rational:explicit",
        });
        return Ok(Value::Rational(BigRational::from_integer(value)));
    }
    let Value::Tuple(values) = value else {
        return Err(diagnostic(
            source,
            "E-RATIONAL-CONSTRUCTOR-PRODUCT",
            operand.span(),
            "Rational construction requires a positional (numerator, denominator) product",
        ));
    };
    let [Value::Int(numerator), Value::Int(denominator)] = values.as_slice() else {
        return Err(diagnostic(
            source,
            "E-RATIONAL-CONSTRUCTOR-COMPONENTS",
            operand.span(),
            "Rational numerator and denominator must both be Int values",
        ));
    };
    if denominator == &BigInt::from(0) {
        let code = if numerator == &BigInt::from(0) {
            "indeterminate"
        } else {
            "division-by-zero"
        };
        if expression_is_closed(operand) {
            let (diagnostic_code, message) = if code == "indeterminate" {
                (
                    "E-INDETERMINATE-RATIONAL",
                    "Rational (0, 0) does not determine one numeric value",
                )
            } else {
                (
                    "E-DIVISION-BY-ZERO",
                    "a finite Rational constructor requires a nonzero denominator",
                )
            };
            return Err(diagnostic(source, diagnostic_code, operand.span(), message));
        }
        let position = source.position(operand.span().start);
        trace.record(TraceEvent {
            event: "result.error.constructed",
            rule: "TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001",
            detail: if code == "indeterminate" {
                "root.Rational(Int,Int);indeterminate"
            } else {
                "root.Rational(Int,Int);division-by-zero"
            },
        });
        return Ok(Value::Error {
            domain: "root.Rational(Int,Int)".to_owned(),
            code: code.to_owned(),
            line: position.line,
            column: position.column,
        });
    }
    let value = BigRational::new(numerator.clone(), denominator.clone());
    trace.record(TraceEvent {
        event: "numeric.rational.constructed",
        rule: if expression_is_closed(operand) {
            "TOPAL-NUM-RATIONAL-CONSTRUCT-001"
        } else {
            "TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"
        },
        detail: if expression_is_closed(operand) {
            "canonical"
        } else {
            "canonical:validated"
        },
    });
    Ok(Value::Rational(value))
}

const fn cover(first: Span, second: Span) -> Span {
    Span {
        start: first.start,
        end: second.end,
    }
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::Binding { name, value, .. } => cover(*name, value.span()),
        Statement::LanguageSelection { span, .. }
        | Statement::LibrarySelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. } => *span,
        Statement::StateField { name, classifier } => cover(*name, *classifier),
        Statement::Discard { span, value } => cover(*span, value.span()),
        Statement::Return { keyword, value } => cover(*keyword, value.span()),
        Statement::Expression(expression) => expression.span(),
    }
}

fn declaration_name<'a>(source: &'a SourceText, statement: &Statement) -> Option<&'a str> {
    let name = match statement {
        Statement::Binding { name, .. }
        | Statement::Implementation { name, .. }
        | Statement::Function { name, .. }
        | Statement::Generator { name, .. }
        | Statement::Union { name, .. }
        | Statement::Interface { name, .. } => *name,
        Statement::Published { declaration, .. } => return declaration_name(source, declaration),
        _ => return None,
    };
    Some(source.slice(name))
}

fn supported_generator_body(source: &SourceText, body: &[Statement]) -> bool {
    if !matches!(
        body.last(),
        Some(Statement::Expression(_) | Statement::Return { .. })
    ) {
        return false;
    }
    for statement in &body[..body.len().saturating_sub(1)] {
        if yielded_statement(source, statement).is_none()
            && !matches!(
                statement,
                Statement::Published { .. }
                    | Statement::DiagnosticControl { .. }
                    | Statement::Binding { .. }
                    | Statement::ContextAssignment { .. }
                    | Statement::Discard { .. }
                    | Statement::Function { .. }
                    | Statement::Return { .. }
            )
        {
            return false;
        }
    }
    true
}

fn discarded_yield_expression<'a>(
    source: &SourceText,
    statement: &'a Statement,
) -> Option<&'a Expression> {
    let Statement::Discard {
        value: Expression::Application { items, .. },
        ..
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(keyword), yielded] = items.as_slice() else {
        return None;
    };
    (source.slice(*keyword) == "yield").then_some(yielded)
}

fn yielded_statement<'a>(
    source: &SourceText,
    statement: &'a Statement,
) -> Option<(Option<Span>, &'a Expression)> {
    if let Some(expression) = discarded_yield_expression(source, statement) {
        return Some((None, expression));
    }
    if let Statement::Expression(Expression::Application { items, .. }) = statement
        && let [Expression::Identifier(keyword), yielded] = items.as_slice()
        && source.slice(*keyword) == "yield"
    {
        return Some((None, yielded));
    }
    let Statement::Binding {
        name,
        value: Expression::Application { items, .. },
        ..
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(keyword), yielded] = items.as_slice() else {
        return None;
    };
    (source.slice(*keyword) == "yield").then_some((Some(*name), yielded))
}

#[allow(clippy::too_many_arguments)]
fn advance_custom_generator(
    source: &SourceText,
    body: &[Statement],
    cursor: &mut usize,
    scope: &mut Session,
    pending_yield: &mut Option<Box<Value>>,
    resume_binding: &mut Option<String>,
    returned: &mut Option<Value>,
    yield_classifier: &str,
    return_classifier: &str,
    name: &str,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    while *cursor < body.len() {
        let statement = &body[*cursor];
        *cursor += 1;
        if let Some((binding, expression)) = yielded_statement(source, statement) {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, yield_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-YIELD-TYPE",
                    expression.span(),
                    name,
                    "yielded",
                    yield_classifier,
                    &value,
                ));
            }
            *pending_yield = Some(Box::new(value));
            *resume_binding = binding.map(|span| source.slice(span).to_owned());
            trace.record(TraceEvent {
                event: "generator.suspended",
                rule: "TOPAL-GENERATOR-SUSPEND-001",
                detail: name,
            });
            return Ok(());
        }
        if let Statement::Expression(expression) = statement {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, return_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-RETURN-TYPE",
                    expression.span(),
                    name,
                    "returned",
                    return_classifier,
                    &value,
                ));
            }
            *returned = Some(value);
            return Ok(());
        }
        if let Statement::Return {
            value: expression, ..
        } = statement
        {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, return_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-RETURN-TYPE",
                    expression.span(),
                    name,
                    "returned",
                    return_classifier,
                    &value,
                ));
            }
            trace.record(TraceEvent {
                event: "generator.return.explicit",
                rule: "TOPAL-GENERATOR-EXPLICIT-RETURN-001",
                detail: return_classifier,
            });
            *returned = Some(value);
            *cursor = body.len();
            return Ok(());
        }
        let mut execution = Execution {
            source: source.clone(),
            statements: vec![statement.clone()],
            cursor: 0,
            return_classifier: None,
        };
        match execution.step(scope, trace)? {
            ExecutionStep::Advanced { .. } | ExecutionStep::Complete(_) => {}
            ExecutionStep::Returned { .. } => unreachable!("generator bindings cannot return"),
        }
    }
    Ok(())
}

fn generator_return_rule(
    origin: &str,
    empty: bool,
    returned: &str,
    traversal_rule: &'static str,
) -> &'static str {
    if origin != "root.characters" && returned != "Unit" {
        "TOPAL-GENERATOR-FINAL-RETURN-001"
    } else if origin != "root.characters" && empty {
        "TOPAL-GENERATOR-EARLY-RETURN-001"
    } else {
        traversal_rule
    }
}

fn foreach_source_diagnostic(source: &SourceText, span: Span) -> Diagnostic {
    diagnostic(
        source,
        "E-FOREACH-SOURCE",
        span,
        "the implemented foreach subset requires `characters text`",
    )
}

fn consumed_generator_diagnostic(source: &SourceText, span: Span, name: &str) -> Diagnostic {
    diagnostic(
        source,
        "E-GENERATOR-CONSUMED",
        span,
        format!("generator `{name}` was already consumed"),
    )
    .with_help("construct a fresh generator before traversing it again")
}

fn consume_generator_argument(source: &SourceText, session: &mut Session, expression: &Expression) {
    let Expression::Application { items, .. } = expression else {
        return;
    };
    let [
        Expression::Identifier(function),
        Expression::Identifier(argument),
    ] = items.as_slice()
    else {
        return;
    };
    let function_name = source.slice(*function);
    let argument_name = source.slice(*argument);
    let accepts_generator = session
        .functions
        .get(function_name)
        .is_some_and(|candidates| {
            candidates.iter().any(|candidate| {
                matches!(
                    candidate.parameters.as_slice(),
                    [(_, classifier)] if classifier.starts_with("Generator ")
                )
            })
        });
    if accepts_generator
        && matches!(
            session.bindings.get(argument_name),
            Some(
                Value::CharacterGenerator { .. }
                    | Value::CharacterReturningGenerator { .. }
                    | Value::SuspendedGenerator { .. }
            )
        )
    {
        session.bindings.remove(argument_name);
        session.declared_names.remove(argument_name);
        session.consumed_names.insert(argument_name.to_owned());
    }
}

#[allow(clippy::too_many_lines)] // Close delivery, handler execution, and trace order stay auditable together.
fn close_remaining_character_generators(
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    let generators = session
        .bindings
        .iter()
        .filter(|(_, value)| {
            matches!(
                value,
                Value::CharacterGenerator { .. }
                    | Value::CharacterReturningGenerator { .. }
                    | Value::SuspendedGenerator { .. }
            )
        })
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    for name in generators {
        let value = session
            .bindings
            .remove(&name)
            .expect("collected binding exists");
        session.declared_names.remove(&name);
        session.consumed_names.insert(name.clone());
        let origin = match &value {
            Value::CharacterGenerator { origin, .. }
            | Value::CharacterReturningGenerator { origin, .. }
            | Value::SuspendedGenerator { origin, .. } => origin.clone(),
            _ => unreachable!("only generators were collected"),
        };
        let detail = format!("domain=root;code=generator-closed;generator={origin}");
        trace.record(TraceEvent {
            event: "generator.close.signaled",
            rule: "TOPAL-GENERATOR-ERROR-CODE-001",
            detail: &detail,
        });
        if let Value::SuspendedGenerator {
            source,
            body,
            mut cursor,
            bindings,
            scope_state,
            pending_yield: _,
            resume_binding,
            returned,
            return_classifier,
            yield_classifier,
            ..
        } = value
            && let Some(resume_binding) = resume_binding
        {
            let mut pending_yield = None;
            let yield_span = body
                .get(cursor.saturating_sub(1))
                .map_or(Span::new(0, 0), statement_span);
            let position = source.position(yield_span.start);
            let mut scope = session.clone();
            scope.bindings = *bindings;
            *scope.functions = scope_state.functions;
            scope.declared_names = scope_state.declared_names;
            scope.local_function_names = scope_state.local_function_names;
            scope.enum_types = scope_state.enum_types;
            scope.bindings.insert(
                resume_binding.clone(),
                Value::Error {
                    domain: "root".into(),
                    code: "generator-closed".into(),
                    line: position.line,
                    column: position.column,
                },
            );
            scope.declared_names.insert(resume_binding.clone());
            trace.record(TraceEvent {
                event: "generator.close.bound",
                rule: "TOPAL-GENERATOR-CLOSE-HANDLER-001",
                detail: &resume_binding,
            });
            let mut handled_return = returned.map(|value| *value);
            let mut next_resume_binding = None;
            advance_custom_generator(
                &source,
                &body,
                &mut cursor,
                &mut scope,
                &mut pending_yield,
                &mut next_resume_binding,
                &mut handled_return,
                &yield_classifier,
                &return_classifier,
                origin.rsplit('.').next().unwrap_or(&origin),
                trace,
            )?;
            if pending_yield.is_some() {
                return Err(diagnostic(
                    &source,
                    "E-GENERATOR-YIELD-AFTER-CLOSE",
                    body.get(cursor.saturating_sub(1))
                        .map_or(yield_span, statement_span),
                    "a generator cannot yield again after observing `generator-closed`",
                ));
            }
        }
        trace.record(TraceEvent {
            event: "generator.closed",
            rule: if origin == "root.characters" {
                "TOPAL-STRING-CHARACTERS-CLOSE-001"
            } else {
                "TOPAL-GENERATOR-CLOSE-001"
            },
            detail: &origin,
        });
    }
    Ok(())
}

fn enum_alternatives(source: &SourceText, expression: &Expression) -> Option<Vec<(String, Span)>> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Enum" {
        return None;
    }
    fields
        .iter()
        .map(|field| {
            let Expression::Identifier(alternative) = &field.value else {
                return None;
            };
            field
                .label
                .is_none()
                .then(|| (source.slice(*alternative).to_owned(), *alternative))
        })
        .collect()
}

fn variant_alternatives(source: &SourceText, expression: &Expression) -> Option<Vec<String>> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Variant" {
        return None;
    }
    fields
        .iter()
        .map(|field| {
            field
                .label
                .is_none()
                .then(|| classifier_expression(source, &field.value))
                .flatten()
        })
        .collect()
}

fn evaluate_arithmetic_error_code(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Option<Value> {
    let [
        Expression::Identifier(lang),
        Expression::Identifier(arithmetic),
        Expression::Identifier(code),
    ] = items
    else {
        return None;
    };
    if source.slice(*lang) != "lang" || source.slice(*arithmetic) != "arithmetic" {
        return None;
    }
    let code = source.slice(*code);
    if !matches!(
        code,
        "out-of-range" | "not-representable" | "division-by-zero" | "indeterminate"
    ) {
        return None;
    }
    trace.record(TraceEvent {
        event: "namespace.member.selected",
        rule: "TOPAL-NUM-ARITHMETIC-ERROR-001",
        detail: code,
    });
    Some(Value::Enum {
        type_name: "lang arithmetic ArithmeticErrorCode".to_owned(),
        alternative: code.to_owned(),
    })
}

fn evaluate_generator_error_code(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Option<Value> {
    let [
        Expression::Identifier(lang),
        Expression::Identifier(generator),
        Expression::Identifier(code),
    ] = items
    else {
        return None;
    };
    if source.slice(*lang) != "lang"
        || source.slice(*generator) != "generator"
        || source.slice(*code) != "generator-closed"
    {
        return None;
    }
    trace.record(TraceEvent {
        event: "namespace.member.selected",
        rule: "TOPAL-GENERATOR-ERROR-CODE-001",
        detail: "generator-closed",
    });
    Some(Value::Enum {
        type_name: "lang generator GeneratorErrorCode".to_owned(),
        alternative: "generator-closed".to_owned(),
    })
}

fn declare_enum(
    source: &SourceText,
    name: Span,
    expression: &Expression,
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Result<Option<(Value, Span)>, Diagnostic> {
    let Some(alternatives) = enum_alternatives(source, expression) else {
        return Ok(None);
    };
    let name_text = source.slice(name);
    let mut seen = BTreeSet::new();
    for (alternative, span) in &alternatives {
        if !seen.insert(alternative.as_str())
            || session.declared_names.contains(alternative)
            || alternative == name_text
        {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-ENUM-ALTERNATIVE",
                *span,
                format!("enum alternative `{alternative}` is already declared in this scope"),
            ));
        }
    }
    session.declared_names.insert(name_text.to_owned());
    session.enum_types.insert(
        name_text.to_owned(),
        alternatives
            .iter()
            .map(|(alternative, _)| alternative.clone())
            .collect(),
    );
    for (alternative, _) in alternatives {
        session.bindings.insert(
            alternative.clone(),
            Value::Enum {
                type_name: name_text.to_owned(),
                alternative: alternative.clone(),
            },
        );
        session.declared_names.insert(alternative);
    }
    trace.record(TraceEvent {
        event: "enum.declared",
        rule: "TOPAL-TYPE-ENUM-001",
        detail: name_text,
    });
    Ok(Some((Value::Unit, cover(name, expression.span()))))
}

fn declare_union(
    source: &SourceText,
    session: &mut Session,
    name: Span,
    alternatives: &[topal_syntax::UnionAlternative],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<(Value, Span), Diagnostic> {
    let type_name = source.slice(name);
    if session.declared_names.contains(type_name) {
        return Err(diagnostic(
            source,
            "E-DUPLICATE-UNION",
            name,
            "name is already declared",
        ));
    }
    let mut declared = BTreeMap::new();
    for alternative in alternatives {
        let alternative_name = source.slice(alternative.name);
        if declared.contains_key(alternative_name) {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-UNION-ALTERNATIVE",
                alternative.name,
                "Union alternative occurs more than once",
            ));
        }
        let classifier = alternative
            .classifier
            .map(|classifier| source.slice(classifier).to_owned());
        declared.insert(alternative_name.to_owned(), classifier.clone());
        if classifier.is_none() {
            session.bindings.insert(
                alternative_name.to_owned(),
                Value::Union(Box::new(UnionValue {
                    type_name: type_name.to_owned(),
                    alternative: alternative_name.to_owned(),
                    payload_classifier: None,
                    payload: None,
                })),
            );
        }
        session.declared_names.insert(alternative_name.to_owned());
    }
    session.union_types.insert(type_name.to_owned(), declared);
    session.declared_names.insert(type_name.to_owned());
    trace.record(TraceEvent {
        event: "union.declared",
        rule: "TOPAL-TYPE-UNION-001",
        detail: type_name,
    });
    Ok((Value::Unit, span))
}

fn declare_variant(
    source: &SourceText,
    name: Span,
    expression: &Expression,
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Option<(Value, Span)> {
    let alternatives = variant_alternatives(source, expression)?;
    let type_name = source.slice(name);
    let declared = alternatives
        .into_iter()
        .enumerate()
        .map(|(index, classifier)| (format!("at {index}"), Some(classifier)))
        .collect();
    session.union_types.insert(type_name.to_owned(), declared);
    session.declared_names.insert(type_name.to_owned());
    trace.record(TraceEvent {
        event: "variant.declared",
        rule: "TOPAL-TYPE-VARIANT-001",
        detail: type_name,
    });
    Some((Value::Unit, expression.span()))
}

fn value_has_classifier(value: &Value, classifier: &str) -> bool {
    if classifier == "MessageContext"
        && matches!(value, Value::Record(fields)
            if fields.iter().any(|(name, _)| name == "session-id")
                && fields.iter().any(|(name, _)| name == "sender"))
    {
        return true;
    }
    if let Value::LayoutBacked { layout, value } = value {
        return classifier == layout.semantic || value_has_classifier(value, classifier);
    }
    if let Value::Refined {
        constraint,
        base_classifier,
        value,
    } = value
    {
        return classifier == constraint
            || (classifier == base_classifier && value_has_classifier(value, base_classifier));
    }
    if let Value::SuspendedGenerator {
        yield_classifier,
        return_classifier,
        ..
    } = value
    {
        return classifier == format!("Generator {yield_classifier} Unit {return_classifier}");
    }
    if let Value::IterateGenerator {
        classifier: yielded,
        ..
    } = value
    {
        return classifier == format!("Generator {yielded} Unit Unit");
    }
    if matches!(value, Value::UnfoldGenerator { .. }) {
        return classifier == "Generator Value Unit Unit";
    }
    if let Value::Optional {
        payload_classifier, ..
    } = value
        && let Some(expected) = optional_payload_classifier(classifier)
    {
        return payload_classifier == expected;
    }
    if let Value::List {
        element_classifier,
        entries,
    } = value
        && let Some(expected) = list_element_classifier(classifier)
    {
        return element_classifier == expected
            && entries
                .iter()
                .all(|entry| value_has_classifier(entry, expected));
    }
    if let Some(success) = result_success_classifier(classifier) {
        return matches!(value, Value::Error { code, .. } if is_arithmetic_error_code(code))
            || value_has_classifier(value, success);
    }
    if let (Value::Tuple(values), Some(classifiers)) = (value, tuple_classifiers(classifier)) {
        return values.len() == classifiers.len()
            && values
                .iter()
                .zip(classifiers)
                .all(|(value, classifier)| value_has_classifier(value, classifier));
    }
    let requested_kind = match classifier {
        "Type" => Some(ObjectKind::Type),
        "Function" => Some(ObjectKind::Function),
        "Constraint" => Some(ObjectKind::Constraint),
        "Capability" => Some(ObjectKind::Capability),
        "Effect" => Some(ObjectKind::Effect),
        "Scope" => Some(ObjectKind::Scope),
        _ => None,
    };
    if let Some(requested_kind) = requested_kind {
        return value.object_kind().satisfies(requested_kind);
    }
    match (value, classifier) {
        (Value::Boolean(_), "Boolean")
        | (Value::Int(_), "Int")
        | (Value::Rational(_), "Rational")
        | (Value::IntRange { .. }, "Range Int")
        | (Value::RationalRange { .. }, "Range Rational")
        | (Value::CharacterGenerator { .. }, "Generator Character Unit Unit")
        | (Value::CharacterReturningGenerator { .. }, "Generator Character Unit Character")
        | (Value::String(_), "String")
        | (Value::Continue(_) | Value::Finish(_), "TraversalControl")
        | (Value::Completed, "Completed")
        | (Value::Unit, "Unit") => true,
        (Value::String(value), "Character") => character_count(value) == 1,
        (Value::Int(value), "Nat") => value >= &BigInt::from(0),
        (Value::Enum { type_name, .. } | Value::Modular { type_name, .. }, classifier) => {
            type_name == classifier
        }
        (Value::Union(union), classifier) => union.type_name == classifier,
        _ => false,
    }
}

fn supported_generator_value_classifier(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    matches!(
        classifier,
        "Unit"
            | "Boolean"
            | "Character"
            | "Comparison"
            | "Constraint"
            | "Effect"
            | "Error"
            | "Int"
            | "MessageContext"
            | "Nat"
            | "Rational"
            | "Scope"
            | "String"
            | "Range Int"
            | "Range Rational"
    ) || enum_types.contains_key(classifier)
        || optional_payload_classifier(classifier)
            .is_some_and(|payload| supported_generator_value_classifier(payload, enum_types))
        || list_element_classifier(classifier)
            .is_some_and(|element| supported_generator_value_classifier(element, enum_types))
        || tuple_classifiers(classifier).is_some_and(|items| {
            items
                .into_iter()
                .all(|item| supported_generator_value_classifier(item, enum_types))
        })
        || result_success_classifier(classifier)
            .is_some_and(|success| supported_generator_value_classifier(success, enum_types))
}

fn is_arithmetic_error_code(code: &str) -> bool {
    matches!(
        code,
        "out-of-range" | "not-representable" | "division-by-zero" | "indeterminate"
    )
}

fn error_code_classifier(code: &str) -> &'static str {
    if is_arithmetic_error_code(code) {
        "lang arithmetic ArithmeticErrorCode"
    } else {
        "lang generator GeneratorErrorCode"
    }
}

fn result_classifier_parts(classifier: &str) -> Option<(&str, &str)> {
    let contents = classifier
        .trim()
        .strip_prefix("Result")?
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let comma = top_level_comma(contents)?;
    Some((contents[..comma].trim(), contents[comma + 1..].trim()))
}

fn result_success_classifier(classifier: &str) -> Option<&str> {
    let (success, errors) = result_classifier_parts(classifier)?;
    let errors = errors.split_whitespace().collect::<Vec<_>>().join(" ");
    matches!(
        errors.as_str(),
        "lang arithmetic ArithmeticErrorCode" | "()"
    )
    .then_some(success)
}

fn optional_payload_classifier(classifier: &str) -> Option<&str> {
    classifier.trim().strip_prefix("Optional ").map(str::trim)
}

fn list_element_classifier(classifier: &str) -> Option<&str> {
    classifier.trim().strip_prefix("List ").map(str::trim)
}

fn tuple_classifiers(classifier: &str) -> Option<Vec<&str>> {
    let contents = classifier.trim().strip_prefix('(')?.strip_suffix(')')?;
    let mut classifiers = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0_usize;
    for (offset, character) in contents.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                classifiers.push(contents[start..offset].trim());
                start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    classifiers.push(contents[start..].trim());
    (classifiers.len() > 1 && classifiers.iter().all(|item| !item.is_empty()))
        .then_some(classifiers)
}

fn top_level_comma(text: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => return Some(offset),
            _ => {}
        }
    }
    None
}

const fn function_rule(is_static: bool, parameter_count: usize) -> &'static str {
    if !is_static {
        return "TOPAL-FUNCTION-ORDINARY-001";
    }
    match parameter_count {
        0 => "TOPAL-FUNCTION-STATIC-NULLARY-001",
        1 => "TOPAL-FUNCTION-STATIC-UNARY-001",
        _ => "TOPAL-FUNCTION-STATIC-BINARY-001",
    }
}

fn function_accepts(parameters: &[(String, String)], argument: &Value) -> bool {
    match parameters {
        [] => matches!(argument, Value::Unit),
        [(_, classifier)] => value_has_classifier(argument, classifier),
        parameters => {
            let Value::Tuple(arguments) = argument else {
                return false;
            };
            arguments.len() == parameters.len()
                && parameters
                    .iter()
                    .zip(arguments)
                    .all(|((_, classifier), argument)| value_has_classifier(argument, classifier))
        }
    }
}

fn user_function_accepts(function: &UserFunction, argument: &Value) -> bool {
    let arguments = match function.parameters.as_slice() {
        [] => return matches!(argument, Value::Unit),
        [_] => std::slice::from_ref(argument),
        parameters => {
            let Value::Tuple(arguments) = argument else {
                return false;
            };
            if arguments.len() != parameters.len() {
                return false;
            }
            arguments
        }
    };
    let mut generic_types = BTreeMap::new();
    function.parameters.iter().enumerate().zip(arguments).all(
        |((index, (_, classifier)), argument)| {
            if let Some(fields) = function.parameter_packages.get(&index) {
                package_generic_accepts(fields, argument, &mut generic_types)
            } else {
                generic_parameter_accepts(argument, classifier, &mut generic_types)
            }
        },
    )
}

fn generic_capability_classifier(classifier: &str) -> Option<(&str, &str)> {
    let contents = classifier.trim().strip_prefix('(')?.strip_suffix(')')?;
    let (name, capability) = contents.split_once(':')?;
    let name = name.trim();
    let capability = capability.trim();
    (!name.is_empty() && !capability.is_empty()).then_some((name, capability))
}

fn generic_parameter_accepts(
    argument: &Value,
    classifier: &str,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if let Some((name, capability)) = generic_capability_classifier(classifier) {
        if !value_has_capability(argument, capability) {
            return false;
        }
        let actual = structural_value_classifier(argument);
        return generic_types
            .insert(name.to_owned(), actual.clone())
            .is_none_or(|existing| existing == actual);
    }
    if let Some(expected) = generic_types.get(classifier) {
        return structural_value_classifier(argument) == *expected;
    }
    if let Some(payload_classifier) = applied_classifier(classifier, "Optional") {
        let Value::Optional {
            payload_classifier: actual,
            ..
        } = argument
        else {
            return false;
        };
        return generic_classifier_accepts_name(actual, payload_classifier, generic_types);
    }
    if let Some(element) = applied_classifier(classifier, "List") {
        let Value::List {
            element_classifier, ..
        } = argument
        else {
            return false;
        };
        return generic_classifier_accepts_name(element_classifier, element, generic_types);
    }
    if let Some(endpoint) = applied_classifier(classifier, "Range") {
        let actual = match argument {
            Value::IntRange { .. } => "Int",
            Value::RationalRange { .. } => "Rational",
            _ => return false,
        };
        return generic_classifier_accepts_name(actual, endpoint, generic_types);
    }
    if let Some((success, codes)) = result_classifier_parts(classifier) {
        if let Value::Error { code, .. } = argument {
            return generic_classifier_accepts_name(
                error_code_classifier(code),
                codes,
                generic_types,
            );
        }
        return generic_classifier_accepts_name(
            &structural_value_classifier(argument),
            success,
            generic_types,
        );
    }
    if function_classifier_parts(classifier).is_some() {
        return value_classifier(argument) == "Function";
    }
    value_has_classifier(argument, classifier)
}

fn generic_classifier_accepts_name(
    actual: &str,
    expected: &str,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if let Some((name, _)) = generic_capability_classifier(expected) {
        return generic_types
            .insert(name.to_owned(), actual.to_owned())
            .is_none_or(|existing| existing == actual);
    }
    if let Some(bound) = generic_types.get(expected) {
        return bound == actual;
    }
    if let (Some(actual_payload), Some(expected_payload)) = (
        applied_classifier(actual, "Optional"),
        applied_classifier(expected, "Optional"),
    ) {
        return generic_classifier_accepts_name(actual_payload, expected_payload, generic_types);
    }
    if let (Some(actual_element), Some(expected_element)) = (
        applied_classifier(actual, "List"),
        applied_classifier(expected, "List"),
    ) {
        return generic_classifier_accepts_name(actual_element, expected_element, generic_types);
    }
    if let (Some(actual_endpoint), Some(expected_endpoint)) = (
        applied_classifier(actual, "Range"),
        applied_classifier(expected, "Range"),
    ) {
        return generic_classifier_accepts_name(actual_endpoint, expected_endpoint, generic_types);
    }
    if let (Some(actual_payload), Some(expected_payload)) = (
        result_classifier_parts(actual),
        result_classifier_parts(expected),
    ) {
        return generic_classifier_accepts_name(
            actual_payload.0,
            expected_payload.0,
            generic_types,
        ) && generic_classifier_accepts_name(
            actual_payload.1,
            expected_payload.1,
            generic_types,
        );
    }
    if let (Some(actual_items), Some(expected_items)) =
        (tuple_classifiers(actual), tuple_classifiers(expected))
    {
        return actual_items.len() == expected_items.len()
            && actual_items
                .into_iter()
                .zip(expected_items)
                .all(|(actual, expected)| {
                    generic_classifier_accepts_name(actual, expected, generic_types)
                });
    }
    actual == substitute_classifier(expected, generic_types)
}

fn substitute_classifier(classifier: &str, generic_types: &BTreeMap<String, String>) -> String {
    if let Some(concrete) = generic_types.get(classifier) {
        return concrete.clone();
    }
    if let Some(items) = tuple_classifiers(classifier) {
        return format!(
            "({})",
            items
                .into_iter()
                .map(|item| substitute_classifier(item, generic_types))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    classifier.to_owned()
}

fn value_has_capability(value: &Value, capability: &str) -> bool {
    match capability {
        "Type" => true,
        "Equality" => values_equal(value.clone(), value.clone(), &mut Vec::new()).is_some(),
        "PartialOrder" | "TotalOrder" | "Ordering" => {
            values_compare(value.clone(), value.clone(), &mut Vec::new()).is_some()
        }
        _ => false,
    }
}

fn generic_result_accepts(function: &UserFunction, scope: &Session, value: &Value) -> bool {
    let mut generic_types = BTreeMap::new();
    populate_function_generics(function, scope, &mut generic_types);
    value_matches_substituted_classifier(
        value,
        &function.result,
        &function.generic_names,
        &mut generic_types,
    )
}

fn populate_function_generics(
    function: &UserFunction,
    scope: &Session,
    generic_types: &mut BTreeMap<String, String>,
) {
    for (index, (parameter, classifier)) in function.parameters.iter().enumerate() {
        if let Some(fields) = function.parameter_packages.get(&index) {
            for field in fields {
                if let Some(argument) = scope.bindings.get(&field.name) {
                    let _ = generic_parameter_accepts(argument, &field.classifier, generic_types);
                }
            }
        } else if let Some(argument) = scope.bindings.get(parameter) {
            let _ = generic_parameter_accepts(argument, classifier, generic_types);
            if let (Some((_, expected_result)), Value::NamedFunction(named_function)) =
                (function_classifier_parts(classifier), argument)
                && named_function.candidates.len() == 1
            {
                let _ = bind_named_classifier_generics(
                    &named_function.candidates[0].result,
                    expected_result,
                    &function.generic_names,
                    generic_types,
                );
            }
        }
    }
}

fn value_matches_substituted_classifier(
    value: &Value,
    classifier: &str,
    generic_names: &BTreeSet<String>,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if let Some(expected) = generic_types.get(classifier) {
        return structural_value_classifier(value) == *expected;
    }
    if generic_names.contains(classifier) {
        generic_types.insert(classifier.to_owned(), structural_value_classifier(value));
        return true;
    }
    if let Some(expected_payload) = applied_classifier(classifier, "Optional") {
        let Value::Optional {
            payload_classifier, ..
        } = value
        else {
            return false;
        };
        if generic_names.contains(expected_payload) && !generic_types.contains_key(expected_payload)
        {
            generic_types.insert(expected_payload.to_owned(), payload_classifier.clone());
            return true;
        }
        return generic_classifier_accepts_name(
            payload_classifier,
            expected_payload,
            generic_types,
        );
    }
    if let Some(element) = applied_classifier(classifier, "List") {
        let Value::List {
            element_classifier, ..
        } = value
        else {
            return false;
        };
        return generic_classifier_accepts_name(element_classifier, element, generic_types);
    }
    if let Some(endpoint) = applied_classifier(classifier, "Range") {
        let actual = match value {
            Value::IntRange { .. } => "Int",
            Value::RationalRange { .. } => "Rational",
            _ => return false,
        };
        return generic_classifier_accepts_name(actual, endpoint, generic_types);
    }
    if let Some((success, codes)) = result_classifier_parts(classifier) {
        if let Value::Error { code, .. } = value {
            if generic_names.contains(codes) && !generic_types.contains_key(codes) {
                generic_types.insert(codes.to_owned(), error_code_classifier(code).to_owned());
                return true;
            }
            return generic_classifier_accepts_name(
                error_code_classifier(code),
                codes,
                generic_types,
            );
        }
        return value_matches_substituted_classifier(value, success, generic_names, generic_types);
    }
    if let (Value::Tuple(values), Some(classifiers)) = (value, tuple_classifiers(classifier)) {
        return values.len() == classifiers.len()
            && values.iter().zip(classifiers).all(|(value, classifier)| {
                value_matches_substituted_classifier(
                    value,
                    classifier,
                    generic_names,
                    generic_types,
                )
            });
    }
    value_has_classifier(value, classifier)
}

fn supported_generic_classifier(
    classifier: &str,
    generic_names: &BTreeSet<String>,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    supported_value_classifier(classifier, enum_types)
        || generic_names.contains(classifier)
        || applied_classifier(classifier, "Optional").is_some_and(|payload| {
            supported_generic_classifier(payload, generic_names, enum_types)
                || generic_capability_classifier(payload).is_some()
        })
        || applied_classifier(classifier, "List").is_some_and(|element| {
            supported_generic_classifier(element, generic_names, enum_types)
                || generic_capability_classifier(element).is_some()
        })
        || applied_classifier(classifier, "Range").is_some_and(|endpoint| {
            supported_generic_classifier(endpoint, generic_names, enum_types)
                || generic_capability_classifier(endpoint).is_some()
        })
        || result_classifier_parts(classifier).is_some_and(|(success, codes)| {
            (supported_generic_classifier(success, generic_names, enum_types)
                || generic_capability_classifier(success).is_some())
                && (supported_generic_classifier(codes, generic_names, enum_types)
                    || generic_capability_classifier(codes).is_some())
        })
        || function_classifier_parts(classifier).is_some_and(|(input, result)| {
            supported_generic_classifier(input, generic_names, enum_types)
                && supported_generic_classifier(result, generic_names, enum_types)
        })
        || tuple_classifiers(classifier).is_some_and(|items| {
            items
                .into_iter()
                .all(|item| supported_generic_classifier(item, generic_names, enum_types))
        })
}

fn applied_classifier<'a>(classifier: &'a str, constructor: &str) -> Option<&'a str> {
    let payload = classifier
        .trim()
        .strip_prefix(constructor)?
        .strip_prefix(' ')
        .map(str::trim)
        .filter(|payload| !payload.is_empty())?;
    Some(strip_classifier_group(payload))
}

fn strip_classifier_group(classifier: &str) -> &str {
    let Some(inner) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    else {
        return classifier;
    };
    let mut depth = 0_i32;
    for character in inner.chars() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' | ':' if depth == 0 => return classifier,
            _ => {}
        }
    }
    if depth == 0 { inner.trim() } else { classifier }
}

fn function_classifier_parts(classifier: &str) -> Option<(&str, &str)> {
    let classifier = classifier
        .trim()
        .strip_prefix("static ")
        .unwrap_or(classifier.trim());
    let signature = classifier.strip_prefix("fn (")?;
    let boundary = signature.rfind(") -> ")?;
    Some((
        signature[..boundary].trim(),
        signature[boundary + 5..].trim(),
    ))
}

fn collect_generic_names(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
    names: &mut BTreeSet<String>,
) {
    if let Some((name, _)) = generic_capability_classifier(classifier) {
        names.insert(name.to_owned());
        return;
    }
    if let Some(payload) = applied_classifier(classifier, "Optional") {
        collect_generic_names(payload, enum_types, names);
        return;
    }
    if let Some(element) = applied_classifier(classifier, "List") {
        collect_generic_names(element, enum_types, names);
        return;
    }
    if let Some(endpoint) = applied_classifier(classifier, "Range") {
        collect_generic_names(endpoint, enum_types, names);
        return;
    }
    if let Some((success, codes)) = result_classifier_parts(classifier) {
        collect_generic_names(success, enum_types, names);
        collect_generic_names(codes, enum_types, names);
        return;
    }
    if let Some((input, result)) = function_classifier_parts(classifier) {
        collect_generic_names(input, enum_types, names);
        collect_function_result_generic_names(result, enum_types, names);
        return;
    }
    if let Some(items) = tuple_classifiers(classifier) {
        for item in items {
            collect_generic_names(item, enum_types, names);
        }
    }
}

fn collect_function_result_generic_names(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
    names: &mut BTreeSet<String>,
) {
    if let Some(payload) = applied_classifier(classifier, "Optional") {
        collect_function_result_generic_names(payload, enum_types, names);
    } else if let Some(element) = applied_classifier(classifier, "List") {
        collect_function_result_generic_names(element, enum_types, names);
    } else if let Some(endpoint) = applied_classifier(classifier, "Range") {
        collect_function_result_generic_names(endpoint, enum_types, names);
    } else if let Some((success, codes)) = result_classifier_parts(classifier) {
        collect_function_result_generic_names(success, enum_types, names);
        collect_function_result_generic_names(codes, enum_types, names);
    } else if let Some(items) = tuple_classifiers(classifier) {
        for item in items {
            collect_function_result_generic_names(item, enum_types, names);
        }
    } else if !supported_value_classifier(classifier, enum_types)
        && classifier.chars().all(char::is_alphanumeric)
    {
        names.insert(classifier.to_owned());
    }
}

fn package_generic_accepts(
    fields: &[UserParameterField],
    argument: &Value,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    match argument {
        Value::Record(values) => {
            values
                .iter()
                .all(|(label, _)| fields.iter().any(|field| field.name == *label))
                && fields.iter().all(|field| {
                    values
                        .iter()
                        .find(|(label, _)| *label == field.name)
                        .map_or(field.default.is_some(), |(_, value)| {
                            generic_parameter_accepts(value, &field.classifier, generic_types)
                        })
                })
        }
        Value::Tuple(values) => {
            values.len() == fields.len()
                && fields.iter().zip(values).all(|(field, value)| {
                    generic_parameter_accepts(value, &field.classifier, generic_types)
                })
        }
        _ => false,
    }
}

fn bind_function_arguments(
    scope: &mut Session,
    function: &UserFunction,
    argument: Value,
    trace: &mut impl TraceSink,
    rule: &'static str,
) -> Result<(), Diagnostic> {
    let arguments = match (function.parameters.as_slice(), argument) {
        ([], Value::Unit) => return Ok(()),
        ([_], argument) => vec![argument],
        (_, Value::Tuple(arguments)) => arguments,
        _ => unreachable!("selected overload has already validated its argument"),
    };
    for (index, ((parameter, _), argument)) in function.parameters.iter().zip(arguments).enumerate()
    {
        if let Some(fields) = function.parameter_packages.get(&index) {
            bind_package_fields(scope, &function.source, fields, argument, trace, rule)?;
            continue;
        }
        if parameter == "_" {
            trace.record(TraceEvent {
                event: "function.argument.discarded",
                rule: "TOPAL-TYPE-MATCH-001",
                detail: "_",
            });
            continue;
        }
        scope.bindings.insert(parameter.clone(), argument);
        scope.declared_names.insert(parameter.clone());
        trace.record(TraceEvent {
            event: "function.argument.bound",
            rule,
            detail: parameter,
        });
    }
    Ok(())
}

fn bind_named_classifier_generics(
    actual: &str,
    expected: &str,
    generic_names: &BTreeSet<String>,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if generic_names.contains(expected) {
        return generic_types
            .insert(expected.to_owned(), actual.to_owned())
            .is_none_or(|existing| existing == actual);
    }
    for constructor in ["Optional", "List", "Range"] {
        if let (Some(actual_inner), Some(expected_inner)) = (
            applied_classifier(actual, constructor),
            applied_classifier(expected, constructor),
        ) {
            return bind_named_classifier_generics(
                actual_inner,
                expected_inner,
                generic_names,
                generic_types,
            );
        }
    }
    generic_classifier_accepts_name(actual, expected, generic_types)
}

fn bind_package_fields(
    scope: &mut Session,
    source: &SourceText,
    fields: &[UserParameterField],
    argument: Value,
    trace: &mut impl TraceSink,
    rule: &'static str,
) -> Result<(), Diagnostic> {
    let supplied = match argument {
        Value::Record(values) => values.into_iter().collect::<BTreeMap<_, _>>(),
        Value::Tuple(values) => fields
            .iter()
            .map(|field| field.name.clone())
            .zip(values)
            .collect(),
        _ => unreachable!("selected package has validated its argument"),
    };
    for field in fields {
        let value = if let Some(value) = supplied.get(&field.name) {
            value.clone()
        } else {
            let default = field
                .default
                .as_ref()
                .expect("selected package requires a supplied field or default");
            let value = scope.evaluate_expression(source, default, trace)?;
            trace.record(TraceEvent {
                event: "function.argument.defaulted",
                rule: "TOPAL-FUNCTION-PACKAGED-OPERAND-001",
                detail: &field.name,
            });
            value
        };
        if field.name == "_" {
            continue;
        }
        scope.bindings.insert(field.name.clone(), value);
        scope.declared_names.insert(field.name.clone());
        trace.record(TraceEvent {
            event: "function.argument.bound",
            rule,
            detail: &field.name,
        });
    }
    Ok(())
}

fn bind_generator_arguments(
    scope: &mut Session,
    parameters: &[(String, String)],
    argument: Value,
    trace: &mut impl TraceSink,
) {
    let arguments = match (parameters, argument) {
        ([_], argument) => vec![argument],
        (_, Value::Tuple(arguments)) => arguments,
        _ => unreachable!("selected generator overload has validated its argument"),
    };
    for ((parameter, _), argument) in parameters.iter().zip(arguments) {
        if parameter == "_" {
            trace.record(TraceEvent {
                event: "generator.argument.discarded",
                rule: "TOPAL-TYPE-MATCH-001",
                detail: "_",
            });
            continue;
        }
        scope.bindings.insert(parameter.clone(), argument);
        scope.declared_names.insert(parameter.clone());
        trace.record(TraceEvent {
            event: "generator.argument.bound",
            rule: "TOPAL-GENERATOR-OVERLOAD-001",
            detail: parameter,
        });
    }
}

fn no_applicable_generator(
    source: &SourceText,
    name: &str,
    argument_span: Span,
    argument: &Value,
    candidates: &[UserGenerator],
) -> Diagnostic {
    let found = structural_value_classifier(argument);
    let expected = candidates
        .iter()
        .map(|candidate| {
            candidate
                .parameters
                .iter()
                .map(|(_, classifier)| classifier.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect::<Vec<_>>()
        .join(" or ");
    diagnostic(
        source,
        "E-NO-APPLICABLE-GENERATOR",
        argument_span,
        format!("no `{name}` generator overload accepts `{found}`"),
    )
    .with_help(format!("available input classifiers: {expected}"))
}

fn no_applicable_overload(
    source: &SourceText,
    name: &str,
    argument_span: Span,
    argument: &Value,
    candidates: &[UserFunction],
    static_context: bool,
) -> Diagnostic {
    let eligible = candidates
        .iter()
        .filter(|function| !static_context || function.is_static)
        .collect::<Vec<_>>();
    if let [function] = eligible.as_slice() {
        match function.parameters.as_slice() {
            [] => {
                return diagnostic(
                    source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    argument_span,
                    format!("nullary function `{name}` requires ()"),
                );
            }
            [(parameter, classifier)] => {
                return diagnostic(
                    source,
                    "E-FUNCTION-ARGUMENT-TYPE",
                    argument_span,
                    format!("argument for `{parameter}` is outside `{classifier}`"),
                );
            }
            parameters => {
                let Value::Tuple(arguments) = argument else {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-SHAPE",
                        argument_span,
                        format!(
                            "function `{name}` requires a positional product with {} fields",
                            parameters.len()
                        ),
                    );
                };
                if arguments.len() != parameters.len() {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-ARITY",
                        argument_span,
                        format!(
                            "function `{name}` requires {} arguments but received {}",
                            parameters.len(),
                            arguments.len()
                        ),
                    );
                }
                if let Some(((parameter, classifier), _)) = parameters
                    .iter()
                    .zip(arguments)
                    .find(|((_, classifier), argument)| !value_has_classifier(argument, classifier))
                {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-TYPE",
                        argument_span,
                        format!("argument for `{parameter}` is outside `{classifier}`"),
                    );
                }
            }
        }
    }
    let signatures = eligible
        .iter()
        .map(|function| {
            let inputs = function
                .parameters
                .iter()
                .map(|(_, classifier)| classifier.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name} ({inputs})")
        })
        .collect::<Vec<_>>()
        .join(", ");
    diagnostic(
        source,
        "E-NO-APPLICABLE-OVERLOAD",
        argument_span,
        format!(
            "no overload of `{name}` accepts {} in this context",
            value_classifier(argument)
        ),
    )
    .with_help(format!("available overloads: {signatures}"))
}

fn function_signature(name: &str, function: &UserFunction) -> String {
    let inputs = function
        .parameters
        .iter()
        .map(|(_, classifier)| classifier.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let staticness = if function.is_static { " static" } else { "" };
    format!("{name}{staticness} ({inputs})")
}

fn validate_parameter_names(
    source: &SourceText,
    parameters: &[FunctionParameter],
) -> Result<(), Diagnostic> {
    let flattened = parameters
        .iter()
        .flat_map(|parameter| {
            if parameter.fields.is_empty() {
                std::slice::from_ref(parameter)
            } else {
                parameter.fields.as_slice()
            }
        })
        .collect::<Vec<_>>();
    for (index, parameter) in flattened.iter().enumerate() {
        let name = source.slice(parameter.name);
        if name == "_" {
            continue;
        }
        if flattened[..index]
            .iter()
            .any(|earlier| source.slice(earlier.name) == name)
        {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-FUNCTION-PARAMETER",
                parameter.name,
                format!("parameter `{name}` is already declared in this function"),
            ));
        }
    }
    Ok(())
}

fn prove_int_recursion(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    body: &[Statement],
) -> Option<&'static str> {
    let [(parameter, classifier)] = parameters else {
        return None;
    };
    if classifier != "Int" && classifier != "Nat" {
        return None;
    }
    let [Statement::Expression(Expression::DecisionTable { subject, rules, .. })] = body else {
        return None;
    };
    if !matches!(subject.as_ref(), Expression::Identifier(span) if source.slice(*span) == parameter)
    {
        return None;
    }
    let [base, recursive] = rules.as_slice() else {
        return None;
    };
    let (step, proof_rule) = match (&**classifier, &base.matcher) {
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(bound),
                ..
            },
        ) if parse_integer(source.slice(*bound)).is_some_and(|value| value >= BigInt::from(0)) => {
            (CallableKind::Minus, "TOPAL-FUNCTION-RECURSION-NAT-001")
        }
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (CallableKind::Minus, "TOPAL-FUNCTION-RECURSION-INT-001"),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-INT-INCREASING-001",
        ),
        _ => return None,
    };
    if !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_))
        || contains_self_call(source, function_name, &base.action)
    {
        return None;
    }
    let (found, valid) =
        bounded_self_calls(source, function_name, parameter, step, &recursive.action);
    let nat_step_limit = nat_decrement_step_limit(source, &base.matcher);
    let preserves_nat = classifier != "Nat"
        || step == CallableKind::Plus
        || nat_step_limit.is_some_and(|limit| {
            recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                &recursive.action,
                &limit,
            )
        });
    (found && valid && preserves_nat).then_some(proof_rule)
}

fn prove_euclidean_recursion(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    effect_bound: Option<&str>,
    body: &[Statement],
) -> Option<&'static str> {
    let [(left, left_classifier), (right, right_classifier)] = parameters else {
        return None;
    };
    if left_classifier != "Int"
        || right_classifier != "Int"
        || !effect_bound.is_some_and(|bound| bound.trim_start().starts_with("Decreases"))
    {
        return None;
    }
    let [Statement::Expression(Expression::DecisionTable { subject, rules, .. })] = body else {
        return None;
    };
    if !matches!(subject.as_ref(), Expression::Identifier(span) if source.slice(*span) == right) {
        return None;
    }
    let [base, recursive] = rules.as_slice() else {
        return None;
    };
    if !matches!(&base.matcher, DecisionMatcher::Comparison {
        kind: CallableKind::Equal,
        operand: Expression::Integer(zero),
        ..
    } if parse_integer(source.slice(*zero)).is_some_and(|value| value == BigInt::from(0)))
        || !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_))
        || contains_self_call(source, function_name, &base.action)
    {
        return None;
    }
    let Expression::Application { items, .. } = &recursive.action else {
        return None;
    };
    let [
        Expression::Identifier(callee),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    let [first, second] = fields.as_slice() else {
        return None;
    };
    let modulo_is_measure_reducing = matches!(
        &second.value,
        Expression::Application { items, .. }
            if matches!(items.as_slice(), [
                Expression::Identifier(dividend),
                Expression::Callable { kind: CallableKind::Modulo, .. },
                Expression::Identifier(divisor)
            ] if source.slice(*dividend) == left && source.slice(*divisor) == right)
    );
    (source.slice(*callee) == function_name
        && matches!(&first.value, Expression::Identifier(span) if source.slice(*span) == right)
        && modulo_is_measure_reducing)
        .then_some("TOPAL-FUNCTION-RECURSION-EUCLIDEAN-001")
}

fn nat_decrement_step_limit(source: &SourceText, matcher: &DecisionMatcher) -> Option<BigInt> {
    let DecisionMatcher::Comparison {
        kind: CallableKind::LessEqual,
        operand: Expression::Integer(bound),
        ..
    } = matcher
    else {
        return None;
    };
    parse_integer(source.slice(*bound)).map(|bound| bound + BigInt::from(1))
}

fn recursive_calls_fit_nat_bound(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    expression: &Expression,
    maximum_step: &BigInt,
) -> bool {
    match expression {
        Expression::Application { items, .. } if matches!(items.first(), Some(Expression::Identifier(span)) if source.slice(*span) == function_name) =>
        {
            matches!(items.as_slice(), [_, Expression::Application { items, .. }]
                if matches!(items.as_slice(), [Expression::Identifier(name), Expression::Callable { kind: CallableKind::Minus, .. }, Expression::Integer(amount)]
                    if source.slice(*name) == parameter && parse_integer(source.slice(*amount)).is_some_and(|step| step > BigInt::from(0) && step <= *maximum_step)))
        }
        Expression::Application { items, .. } => items.iter().all(|item| {
            recursive_calls_fit_nat_bound(source, function_name, parameter, item, maximum_step)
        }),
        Expression::Product { fields, .. } => fields.iter().all(|field| {
            recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                &field.value,
                maximum_step,
            )
        }),
        Expression::DecisionTable { subject, rules, .. } => {
            recursive_calls_fit_nat_bound(source, function_name, parameter, subject, maximum_step)
                && rules.iter().all(|rule| {
                    recursive_calls_fit_nat_bound(
                        source,
                        function_name,
                        parameter,
                        &rule.action,
                        maximum_step,
                    )
                })
        }
        _ => true,
    }
}

fn prove_mutual_int_recursion_edge(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    body: &[Statement],
) -> Option<(String, &'static str)> {
    let [(parameter, classifier)] = parameters else {
        return None;
    };
    if classifier != "Int" && classifier != "Nat" {
        return None;
    }
    let [Statement::Expression(Expression::DecisionTable { subject, rules, .. })] = body else {
        return None;
    };
    if !matches!(subject.as_ref(), Expression::Identifier(span) if source.slice(*span) == parameter)
    {
        return None;
    }
    let [base, recursive] = rules.as_slice() else {
        return None;
    };
    let (step, proof_rule) = match (&**classifier, &base.matcher) {
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(bound),
                ..
            },
        ) if parse_integer(source.slice(*bound)).is_some_and(|value| value >= BigInt::from(0)) => (
            CallableKind::Minus,
            "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001",
        ),
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Minus,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001",
        ),
        _ => return None,
    };
    if !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_)) {
        return None;
    }
    let (target, valid) =
        mutual_call_target(source, function_name, parameter, step, &recursive.action);
    let target = target?;
    if !valid
        || contains_self_call(source, &target, &base.action)
        || (classifier == "Nat"
            && step == CallableKind::Minus
            && !nat_decrement_step_limit(source, &base.matcher).is_some_and(|limit| {
                recursive_calls_fit_nat_bound(source, &target, parameter, &recursive.action, &limit)
            }))
    {
        return None;
    }
    Some((target, proof_rule))
}

fn mutual_call_target(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> (Option<String>, bool) {
    match expression {
        Expression::Application { items, .. }
            if matches!(items.as_slice(), [Expression::Identifier(_), _]) =>
        {
            let [Expression::Identifier(target), argument] = items.as_slice() else {
                unreachable!("guard established a unary named application");
            };
            let target = source.slice(*target);
            (
                Some(target.to_owned()),
                target != function_name
                    && is_positive_literal_step(source, parameter, step, argument),
            )
        }
        Expression::Application { items, .. } => combine_mutual_call_targets(
            items
                .iter()
                .map(|item| mutual_call_target(source, function_name, parameter, step, item)),
        ),
        Expression::Product { fields, .. } => {
            combine_mutual_call_targets(fields.iter().map(|field| {
                mutual_call_target(source, function_name, parameter, step, &field.value)
            }))
        }
        Expression::DecisionTable { subject, rules, .. } => combine_mutual_call_targets(
            std::iter::once(mutual_call_target(
                source,
                function_name,
                parameter,
                step,
                subject,
            ))
            .chain(rules.iter().map(|rule| {
                mutual_call_target(source, function_name, parameter, step, &rule.action)
            })),
        ),
        _ => (None, true),
    }
}

fn combine_mutual_call_targets(
    checks: impl Iterator<Item = (Option<String>, bool)>,
) -> (Option<String>, bool) {
    checks.fold(
        (None, true),
        |(target, valid), (next_target, next_valid)| match (target, next_target) {
            (Some(target), Some(next)) => {
                let same = target == next;
                (Some(target), valid && next_valid && same)
            }
            (Some(target), None) | (None, Some(target)) => (Some(target), valid && next_valid),
            (None, None) => (None, valid && next_valid),
        },
    )
}

const MUTUAL_INT_RECURSION_RULE: &str = "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001";
const MUTUAL_INCREASING_INT_RECURSION_RULE: &str =
    "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001";
const MUTUAL_NAT_RECURSION_RULE: &str = "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001";
const MUTUAL_INCREASING_NAT_RECURSION_RULE: &str =
    "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001";

fn is_mutual_recursion_rule(rule: &str) -> bool {
    matches!(
        rule,
        MUTUAL_INT_RECURSION_RULE
            | MUTUAL_INCREASING_INT_RECURSION_RULE
            | MUTUAL_NAT_RECURSION_RULE
            | MUTUAL_INCREASING_NAT_RECURSION_RULE
    )
}

fn recursion_rule_for_call(
    call_stack: &[ActiveCall],
    target: &str,
    target_signature: &str,
    function: &UserFunction,
) -> Option<&'static str> {
    let cycle_start = call_stack
        .iter()
        .position(|active| active.signature == target_signature)?;
    let cycle = &call_stack[cycle_start..];
    if function.recursion_target.is_none() {
        return function.termination_rule;
    }
    let cycle_rule = function.termination_rule?;
    if !is_mutual_recursion_rule(cycle_rule)
        || cycle
            .iter()
            .any(|active| active.termination_rule != Some(cycle_rule))
    {
        return None;
    }
    let internal_edges_match = cycle
        .windows(2)
        .all(|pair| pair[0].recursion_target.as_deref() == Some(pair[1].name.as_str()));
    let closes_cycle = cycle
        .last()
        .and_then(|active| active.recursion_target.as_deref())
        == Some(target);
    (internal_edges_match && closes_cycle).then_some(cycle_rule)
}

fn contains_self_call(source: &SourceText, function_name: &str, expression: &Expression) -> bool {
    let (found, _) = bounded_self_calls(source, function_name, "", CallableKind::Minus, expression);
    found
}

fn bounded_self_calls(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> (bool, bool) {
    match expression {
        Expression::Application { items, .. } if matches!(items.first(), Some(Expression::Identifier(span)) if source.slice(*span) == function_name) =>
        {
            let valid = matches!(items.as_slice(), [_, argument] if is_positive_literal_step(source, parameter, step, argument));
            (true, valid)
        }
        Expression::Application { items, .. } => combine_call_checks(
            items
                .iter()
                .map(|item| bounded_self_calls(source, function_name, parameter, step, item)),
        ),
        Expression::Product { fields, .. } => {
            combine_call_checks(fields.iter().map(|field| {
                bounded_self_calls(source, function_name, parameter, step, &field.value)
            }))
        }
        Expression::DecisionTable { subject, rules, .. } => combine_call_checks(
            std::iter::once(bounded_self_calls(
                source,
                function_name,
                parameter,
                step,
                subject,
            ))
            .chain(rules.iter().map(|rule| {
                bounded_self_calls(source, function_name, parameter, step, &rule.action)
            })),
        ),
        _ => (false, true),
    }
}

fn combine_call_checks(checks: impl Iterator<Item = (bool, bool)>) -> (bool, bool) {
    checks.fold((false, true), |(found, valid), (next_found, next_valid)| {
        (found || next_found, valid && next_valid)
    })
}

fn is_positive_literal_step(
    source: &SourceText,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> bool {
    matches!(
        expression,
        Expression::Application { items, .. }
            if matches!(
                items.as_slice(),
                [
                    Expression::Identifier(name),
                    Expression::Callable { kind, .. },
                    Expression::Integer(amount)
                ] if source.slice(*name) == parameter
                    && *kind == step
                    && parse_integer(source.slice(*amount)).is_some_and(|value| value > BigInt::from(0_u8))
            )
    )
}

fn supported_value_classifier(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    matches!(
        classifier,
        "Boolean"
            | "Character"
            | "Completed"
            | "Comparison"
            | "Constraint"
            | "Effect"
            | "Error"
            | "Generator Character Unit Unit"
            | "Generator Character Unit Character"
            | "Generator String Unit Unit"
            | "Generator String Unit Character"
            | "Generator String Unit String"
            | "Generator Character Unit String"
            | "Function"
            | "Int"
            | "MessageContext"
            | "Nat"
            | "Range Int"
            | "Range Rational"
            | "Rational"
            | "Scope"
            | "String"
            | "Type"
            | "Unit"
    ) || enum_types.contains_key(classifier)
        || generator_classifiers(classifier).is_some_and(|(yielded, resumed, returned)| {
            resumed == "Unit"
                && supported_generator_value_classifier(yielded, enum_types)
                && supported_generator_value_classifier(returned, enum_types)
        })
        || optional_payload_classifier(classifier)
            .is_some_and(|payload| supported_value_classifier(payload, enum_types))
        || list_element_classifier(classifier)
            .is_some_and(|element| supported_value_classifier(element, enum_types))
        || tuple_classifiers(classifier).is_some_and(|items| {
            items
                .into_iter()
                .all(|item| supported_value_classifier(item, enum_types))
        })
        || result_success_classifier(classifier)
            .is_some_and(|success| supported_value_classifier(success, enum_types))
}

fn generator_classifiers(classifier: &str) -> Option<(&str, &str, &str)> {
    let contents = classifier.trim().strip_prefix("Generator")?;
    let (yielded, contents) = take_classifier(contents)?;
    let (resumed, contents) = take_classifier(contents)?;
    let (returned, remainder) = take_classifier(contents)?;
    remainder
        .trim()
        .is_empty()
        .then_some((yielded, resumed, returned))
}

fn take_classifier(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    if text.starts_with('(') {
        let end = parenthesized_end(text)?;
        return Some((&text[..end], &text[end..]));
    }
    let head_end = text.find(char::is_whitespace).unwrap_or(text.len());
    let head = &text[..head_end];
    if head.is_empty() {
        return None;
    }
    if head == "Result" {
        let tail = text[head_end..].trim_start();
        let result_end = parenthesized_end(tail)?;
        let end = text.len() - tail.len() + result_end;
        return Some((&text[..end], &text[end..]));
    }
    let arity = match head {
        "Optional" | "Range" | "List" => 1,
        "Generator" => 3,
        _ => 0,
    };
    let mut remainder = &text[head_end..];
    for _ in 0..arity {
        let (_, next) = take_classifier(remainder)?;
        remainder = next;
    }
    let end = text.len() - remainder.len();
    Some((text[..end].trim_end(), remainder))
}

fn parenthesized_end(text: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(offset + character.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

fn accepted_source(input: &str, trace: &mut impl TraceSink) -> Result<SourceText, Diagnostic> {
    let source = SourceText::new(input).map_err(|error| {
        let (line, column) = raw_position(input, error.span.start);
        let mut diagnostic = Diagnostic::error(error.code, line, column, error.message)
            .with_source_excerpt(
                raw_source_line(input, line),
                marker_width(input, error.span),
            );
        if let Some(help) = diagnostic_help(error.code) {
            diagnostic = diagnostic.with_help(help);
        }
        diagnostic
    })?;
    trace.record(TraceEvent {
        event: "source.accepted",
        rule: "TOPAL-SYN-SOURCE-001",
        detail: "unicode source normalized",
    });
    Ok(source)
}

fn expected_statement(input: &str) -> Diagnostic {
    Diagnostic::error("E-EXPECTED-EXPRESSION", 1, 1, "expected a statement")
        .with_source_excerpt(raw_source_line(input, 1), 1)
        .with_help(
            diagnostic_help("E-EXPECTED-EXPRESSION")
                .expect("expected-expression diagnostic has stable help"),
        )
}

fn record_result(trace: &mut impl TraceSink, value: &Value) {
    let classifier = structural_value_classifier(value);
    trace.record(TraceEvent {
        event: "evaluation.result",
        rule: "TOPAL-SYN-GRAMMAR-001",
        detail: &classifier,
    });
}

fn validate_layout_attributes(
    source: &SourceText,
    span: Span,
    semantic: &str,
    attributes: &[(String, Value)],
) -> Result<(), Diagnostic> {
    const COMMON: &[&str] = &["storage-size", "encoding", "endian", "access", "alignment"];
    const EXTRA: &[&str] = &[
        "bit-order",
        "unit-size",
        "canonical",
        "length",
        "false-pattern",
        "true-pattern",
        "bias",
        "numerator-layout",
        "denominator-layout",
        "integer-layout",
        "quantum",
        "exponent-bits",
        "fraction-bits",
        "exponent-bias",
        "subnormal",
        "infinity",
        "signed-zero",
        "nan",
        "termination",
        "padding",
        "packing",
        "field-order",
        "tag-layout",
        "tags",
        "payload-placement",
        "element-layout",
        "stride",
        "entry-layout",
        "ordering",
        "measurement-unit",
    ];
    for (name, _) in attributes {
        if !COMMON.contains(&name.as_str()) && !EXTRA.contains(&name.as_str()) {
            return Err(diagnostic(
                source,
                "E-LAYOUT-UNKNOWN-FIELD",
                span,
                format!("`{name}` is not a layout attribute"),
            ));
        }
    }
    let field = |name: &str| {
        attributes
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value)
    };
    if let Some(size) = field("storage-size")
        && !matches!(size, Value::SizeBits(bits) if bits >= &BigInt::from(0))
    {
        return Err(diagnostic(
            source,
            "E-LAYOUT-STORAGE-SIZE",
            span,
            "storage-size requires a nonnegative bit or byte size",
        ));
    }
    if semantic == "Unit" {
        if field("storage-size").is_some_and(|value| value != &Value::SizeBits(BigInt::from(0))) {
            return Err(diagnostic(
                source,
                "E-LAYOUT-UNIT-SIZE",
                span,
                "Layout Unit has exactly 0[b] storage",
            ));
        }
    } else if matches!(
        semantic,
        "Boolean" | "Nat" | "Int" | "Rational" | "String" | "Character"
    ) && field("encoding").is_none()
    {
        return Err(diagnostic(
            source,
            "E-LAYOUT-ENCODING",
            span,
            format!("Layout {semantic} requires an encoding"),
        ));
    }
    Ok(())
}

fn sync_stream_task_state(
    session: &Session,
    owner: Option<&str>,
    state: Option<&BTreeMap<String, Value>>,
) {
    let (Some(_), Some(state), Some(Value::TaskInstance(instance))) = (
        owner,
        state,
        owner.and_then(|name| session.bindings.get(name)),
    ) else {
        return;
    };
    instance.borrow_mut().state = state.clone();
}

fn layout_access(layout: &LayoutValue) -> &str {
    layout
        .attributes
        .iter()
        .find_map(|(name, value)| {
            (name == "access")
                .then_some(value)
                .and_then(|value| match value {
                    Value::Enum { alternative, .. } => Some(alternative.as_str()),
                    _ => None,
                })
        })
        .unwrap_or("ReadWrite")
}

fn validate_address_offset(
    source: &SourceText,
    span: Span,
    attributes: &[(String, Value)],
    offset: &BigInt,
) -> Result<(), Diagnostic> {
    if let Some(Value::Int(alignment)) = attributes
        .iter()
        .find_map(|(name, value)| (name == "alignment").then_some(value))
        && (alignment <= &BigInt::from(0) || offset % alignment != BigInt::from(0))
    {
        return Err(diagnostic(
            source,
            "E-ADDRESS-OFFSET-ALIGNMENT",
            span,
            "address offset does not satisfy its byte alignment",
        ));
    }
    if let Some(Value::AddressRange { lower, upper, .. }) = attributes
        .iter()
        .find_map(|(name, value)| (name == "range").then_some(value))
        && offset > &(upper - lower)
    {
        return Err(diagnostic(
            source,
            "E-ADDRESS-OFFSET-RANGE",
            span,
            "address offset lies outside its associated range",
        ));
    }
    Ok(())
}

fn validate_location_fit(
    source: &SourceText,
    span: Span,
    layout: &LayoutValue,
    offset_attributes: &[(String, Value)],
    offset: &BigInt,
) -> Result<(), Diagnostic> {
    let size_bits = layout.attributes.iter().find_map(|(name, value)| {
        (name == "storage-size")
            .then_some(value)
            .and_then(|value| match value {
                Value::SizeBits(bits) => Some(bits),
                _ => None,
            })
    });
    if let (Some(bits), Some(Value::AddressRange { lower, upper, .. })) = (
        size_bits,
        offset_attributes
            .iter()
            .find_map(|(name, value)| (name == "range").then_some(value)),
    ) {
        let bytes = (bits + BigInt::from(7)) / BigInt::from(8);
        if offset + bytes > upper - lower + BigInt::from(1) {
            return Err(diagnostic(
                source,
                "E-LOCATION-RANGE",
                span,
                "layout does not fit in the associated address range",
            ));
        }
    }
    Ok(())
}

fn coerce_layout_value(
    source: &SourceText,
    span: Span,
    layout: &LayoutValue,
    value: Value,
) -> Result<Value, Diagnostic> {
    if let Value::LayoutBacked {
        layout: existing, ..
    } = &value
        && existing.as_ref() == layout
    {
        return Ok(value);
    }
    if !value_has_classifier(&value, &layout.semantic) {
        return Err(diagnostic(
            source,
            "E-LAYOUT-SEMANTIC-VALUE",
            span,
            format!(
                "Layout {} requires a {} value",
                layout.semantic, layout.semantic
            ),
        ));
    }
    if let Value::Int(integer) = &value
        && let Some(Value::SizeBits(bits)) = layout
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "storage-size").then_some(value))
    {
        let unsigned = layout.attributes.iter().any(|(name, value)| {
            name == "encoding"
                && matches!(value, Value::Enum { alternative, .. } if alternative == "UnsignedBinary")
        });
        let limit = usize::try_from(bits)
            .ok()
            .map(|width| BigInt::from(1_u8) << width);
        if unsigned
            && (integer < &BigInt::from(0) || limit.as_ref().is_some_and(|limit| integer >= limit))
        {
            return Err(diagnostic(
                source,
                "E-LAYOUT-NOT-REPRESENTABLE",
                span,
                "integer is not representable by the selected layout",
            ));
        }
    }
    Ok(Value::LayoutBacked {
        layout: Box::new(layout.clone()),
        value: Box::new(value),
    })
}

fn value_classifier(value: &Value) -> &'static str {
    match value {
        Value::Boolean(_) => "Boolean",
        Value::Version(_) => "Version",
        Value::SerializationStream(_) => "SerializationStream",
        Value::ObjectDescription { .. } => "ObjectDescription",
        Value::TaskType(_)
        | Value::AddressRangeType(_)
        | Value::AddressOffsetType(_)
        | Value::LayoutType(_)
        | Value::LayoutFactory(_)
        | Value::LocationType(_)
        | Value::Type(_)
        | Value::ModularType(_) => "Type",
        Value::TaskDefinition(_) => "TaskDefinition",
        Value::TaskInstance(_) => "Task",
        Value::SizeBits(_) => "Size",
        Value::AddressRange { .. } => "AddressRange",
        Value::AddressOffset { .. } => "AddressOffset",
        Value::Location { .. } => "Location",
        Value::LayoutBacked { .. } => "Layout",
        Value::Effects(_) => "Effect",
        Value::Int(_) => "Int",
        Value::Rational(_) => "Rational",
        Value::IntRange { .. } | Value::RationalRange { .. } => "Range",
        Value::Optional { .. } => "Optional",
        Value::List { .. } => "List",
        Value::Callable(_)
        | Value::NamedFunction(_)
        | Value::AnonymousFunction(_)
        | Value::NativeSerializer(_) => "Function",
        Value::Namespace(_) => "Scope",
        Value::Array { .. } => "Array",
        Value::Set { .. } => "Set",
        Value::Bag { .. } => "Bag",
        Value::Map { .. } => "Map",
        Value::CharacterReturningGenerator { .. } => "Generator Character Unit Character",
        Value::IterateGenerator { .. } | Value::UnfoldGenerator { .. } => "Generator",
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } if yield_classifier == "String" && return_classifier == "String" => {
            "Generator String Unit String"
        }
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } if yield_classifier == "String" && return_classifier == "Character" => {
            "Generator String Unit Character"
        }
        Value::SuspendedGenerator {
            yield_classifier, ..
        } if yield_classifier == "String" => "Generator String Unit Unit",
        Value::SuspendedGenerator {
            return_classifier, ..
        } if return_classifier == "String" => "Generator Character Unit String",
        Value::SuspendedGenerator {
            return_classifier, ..
        } if return_classifier == "Character" => "Generator Character Unit Character",
        Value::CharacterGenerator { .. } | Value::SuspendedGenerator { .. } => {
            "Generator Character Unit Unit"
        }
        Value::String(_) => "String",
        Value::Tuple(_) => "Tuple",
        Value::Record(_) => "Record",
        Value::Enum { .. } => "Enum",
        Value::Union(_) => "Union",
        Value::Constraint(_) => "Constraint",
        Value::Capability(_) => "Capability",
        Value::Interface(_) => "Interface",
        Value::Introspection(value) => match value.as_ref() {
            IntrospectionValue::Identity { .. } => "lang Identity",
            IntrospectionValue::TypeView { .. } => "lang TypeView",
            IntrospectionValue::FunctionView { .. } => "lang FunctionView",
            IntrospectionValue::ScopeView { .. } => "lang ScopeView",
            IntrospectionValue::ConstraintView { .. } => "lang ConstraintView",
            IntrospectionValue::EffectView { .. } => "lang EffectView",
            IntrospectionValue::ProtocolView { .. } => "lang ProtocolView",
            IntrospectionValue::DeclarationView { .. } => "lang DeclarationView",
            IntrospectionValue::LanguageContext { .. } => "lang LanguageContext",
        },
        Value::Refined { .. } => "Refined",
        Value::Modular { .. } => "Modular",
        Value::ErrorDomain(_) => "ErrorDomain",
        Value::Error { .. } => "Error",
        Value::Continue(_) | Value::Finish(_) => "TraversalControl",
        Value::Completed => "Completed",
        Value::Unit => "Unit",
    }
}

fn structural_value_classifier(value: &Value) -> String {
    match value {
        Value::IntRange { .. } => "Range Int".into(),
        Value::RationalRange { .. } => "Range Rational".into(),
        Value::Tuple(values) => format!(
            "({})",
            values
                .iter()
                .map(structural_value_classifier)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Optional {
            payload_classifier, ..
        } => format!("Optional {payload_classifier}"),
        Value::List {
            element_classifier, ..
        } => format!("List {element_classifier}"),
        Value::Array {
            element_classifier,
            entries,
        } => format!("Array {} {element_classifier}", entries.len()),
        Value::Set {
            element_classifier, ..
        } => format!("Set {element_classifier}"),
        Value::Bag {
            element_classifier, ..
        } => format!("Bag {element_classifier}"),
        Value::Map {
            key_classifier,
            value_classifier,
            ..
        } => format!("Map ({key_classifier}, {value_classifier})"),
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } => format!("Generator {yield_classifier} Unit {return_classifier}"),
        Value::IterateGenerator { classifier, .. } => {
            format!("Generator {classifier} Unit Unit")
        }
        Value::UnfoldGenerator { .. } => "Generator Value Unit Unit".into(),
        Value::Enum { type_name, .. } | Value::Modular { type_name, .. } => type_name.clone(),
        Value::Union(union) => union.type_name.clone(),
        Value::Constraint(constraint) => format!("Constraint {}", constraint.base_classifier),
        Value::Refined { constraint, .. } => constraint.clone(),
        Value::Type(name) => name.clone(),
        Value::Effects(_) => "Effect".into(),
        Value::ModularType(kind) => kind.name.clone().unwrap_or_else(|| "Type".into()),
        Value::LayoutBacked { layout, .. } => layout.semantic.clone(),
        _ => value_classifier(value).to_owned(),
    }
}

fn classifier_name(source: &SourceText, expression: &Expression) -> Option<String> {
    match expression {
        Expression::Identifier(span) => Some(source.slice(*span).to_owned()),
        _ => None,
    }
}

fn introspection_identity(value: &Value) -> Option<String> {
    match value {
        Value::Type(name) => Some(format!("type:{name}")),
        Value::ModularType(kind) => Some(format!(
            "type:{}:{}..{}",
            if kind.signed { "ModInt" } else { "ModNat" },
            kind.lower,
            kind.upper
        )),
        Value::NamedFunction(function) => Some(format!("function:root.{}", function.name)),
        Value::Callable(kind) => Some(format!("function:root.{}", callable_name(*kind))),
        Value::Namespace(namespace) => Some(format!("scope:{}", namespace.name)),
        Value::Constraint(constraint) => constraint
            .name
            .as_ref()
            .map(|name| format!("constraint:root.{name}")),
        Value::Capability(alternatives) => Some(format!("capability:{alternatives:?}")),
        Value::Effects(effects) => Some(format!("effect-set:{effects:?}")),
        Value::Interface(interface) => Some(format!("interface:root.{}", interface.name)),
        Value::Introspection(value) => match value.as_ref() {
            IntrospectionValue::Identity { canonical, .. } => Some(canonical.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn introspection_view(source: &SourceText, value: Value, span: Span) -> Result<Value, Diagnostic> {
    let view = match value {
        Value::Type(identity) => IntrospectionValue::TypeView {
            form: "PrimitiveType".into(),
            identity,
        },
        Value::ModularType(kind) => IntrospectionValue::TypeView {
            form: "RefinedType".into(),
            identity: kind.name.clone().unwrap_or_else(|| {
                format!(
                    "{} {}..{}",
                    if kind.signed { "ModInt" } else { "ModNat" },
                    kind.lower,
                    kind.upper
                )
            }),
        },
        Value::NamedFunction(function) => {
            let Some(first) = function.candidates.first() else {
                return Err(diagnostic(
                    source,
                    "E-INTROSPECTION-EMPTY-FUNCTION",
                    span,
                    "function view has no declared overload",
                ));
            };
            IntrospectionValue::FunctionView {
                identity: format!("root.{}", function.name),
                inputs: first
                    .parameters
                    .iter()
                    .map(|(_, classifier)| classifier.clone())
                    .collect(),
                output: first.result.clone(),
                is_static: first.is_static,
                effects: first.effect_bound.iter().cloned().collect(),
            }
        }
        Value::Namespace(namespace) => {
            let mut members = namespace
                .bindings
                .keys()
                .chain(namespace.functions.keys())
                .chain(namespace.generators.keys())
                .cloned()
                .collect::<Vec<_>>();
            members.sort();
            members.dedup();
            IntrospectionValue::ScopeView {
                identity: namespace.name.clone(),
                members,
            }
        }
        Value::Constraint(constraint) => IntrospectionValue::ConstraintView {
            identity: constraint
                .name
                .clone()
                .unwrap_or_else(|| constraint.base_classifier.clone()),
            base: constraint.base_classifier,
        },
        Value::Effects(identities) => IntrospectionValue::EffectView { identities },
        Value::Interface(interface) => IntrospectionValue::ProtocolView {
            identity: format!("root.{}", interface.name),
            operations: interface.functions.keys().cloned().collect(),
        },
        _ => {
            return Err(diagnostic(
                source,
                "E-STATIC-INTROSPECTION-SUBJECT",
                span,
                "lang view requires a statically known Type, Function, Scope, Constraint, Effect, or Protocol",
            ));
        }
    };
    Ok(Value::Introspection(Box::new(view)))
}

fn stream_for_value(
    version: LanguageVersion,
    value: &Value,
) -> Result<SerializationStream, &'static str> {
    let mut types = Vec::new();
    let mut identities = BTreeMap::new();
    let (type_id, value) = serialize_language_value(value, &mut types, &mut identities)?;
    Ok(SerializationStream {
        header: SerializationHeader {
            language_identity: "topal".into(),
            language_version: version,
            byte_order: if cfg!(target_endian = "little") {
                StreamByteOrder::Little
            } else {
                StreamByteOrder::Big
            },
            streaming: false,
        },
        types,
        events: vec![SerializedEvent { type_id, value }],
    })
}

#[allow(clippy::too_many_lines)] // Each supported semantic schema remains explicit at the authority-free boundary.
fn serialize_language_value(
    value: &Value,
    types: &mut Vec<TypeDefinition>,
    identities: &mut BTreeMap<String, usize>,
) -> Result<(usize, SerializedValue), &'static str> {
    let (identity, definition, serialized) = match value {
        Value::Unit => (
            "Unit".to_owned(),
            TypeDefinition::Unit {
                identity: "Unit".into(),
            },
            SerializedValue::Unit,
        ),
        Value::Boolean(value) => (
            "Boolean".to_owned(),
            TypeDefinition::Boolean {
                identity: "Boolean".into(),
            },
            SerializedValue::Boolean(*value),
        ),
        Value::Int(value) => (
            "Int".to_owned(),
            TypeDefinition::Int {
                identity: "Int".into(),
                signed: true,
                width_bits: 0,
            },
            SerializedValue::ArbitraryInt(value.clone()),
        ),
        Value::Rational(value) => {
            let (numerator_id, numerator) =
                serialize_language_value(&Value::Int(value.numer().clone()), types, identities)?;
            let (denominator_id, denominator) =
                serialize_language_value(&Value::Int(value.denom().clone()), types, identities)?;
            let mut schema_payload = Vec::new();
            put_protocol_uvarint(numerator_id, &mut schema_payload);
            put_protocol_uvarint(denominator_id, &mut schema_payload);
            (
                "Rational".into(),
                TypeDefinition::ObjectDescription {
                    identity: "Rational".into(),
                    kind: 3,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![numerator, denominator]),
            )
        }
        Value::String(value) => (
            "String".to_owned(),
            TypeDefinition::Text {
                identity: "String".into(),
            },
            SerializedValue::Text(value.clone()),
        ),
        Value::Tuple(values) => {
            let mut components = Vec::with_capacity(values.len());
            let mut encoded = Vec::with_capacity(values.len());
            for value in values {
                let (id, value) = serialize_language_value(value, types, identities)?;
                components.push(id);
                encoded.push(value);
            }
            let identity = structural_value_classifier(value);
            (
                identity.clone(),
                TypeDefinition::Tuple {
                    identity,
                    components,
                },
                SerializedValue::Product(encoded),
            )
        }
        Value::Record(fields) => {
            let mut definitions = Vec::with_capacity(fields.len());
            let mut encoded = Vec::with_capacity(fields.len());
            for (label, value) in fields {
                let (id, value) = serialize_language_value(value, types, identities)?;
                definitions.push((label.clone(), id));
                encoded.push(value);
            }
            let identity = structural_value_classifier(value);
            (
                identity.clone(),
                TypeDefinition::Record {
                    identity,
                    fields: definitions,
                },
                SerializedValue::Product(encoded),
            )
        }
        Value::List {
            element_classifier,
            entries,
        }
        | Value::Array {
            element_classifier,
            entries,
        } => {
            let (element, encoded) = serialize_homogeneous_values(entries, types, identities)?;
            let identity = match value {
                Value::List { .. } => format!("List {element_classifier}"),
                Value::Array { .. } => format!("Array {} {element_classifier}", entries.len()),
                _ => unreachable!(),
            };
            (
                identity.clone(),
                TypeDefinition::Sequence { identity, element },
                SerializedValue::Sequence(encoded),
            )
        }
        Value::Bag {
            element_classifier,
            entries,
        } => {
            let values = entries
                .iter()
                .flat_map(|(value, count)| std::iter::repeat_n(value.clone(), *count))
                .collect::<Vec<_>>();
            let (element, encoded) = serialize_homogeneous_values(&values, types, identities)?;
            let identity = format!("Bag {element_classifier}");
            (
                identity.clone(),
                TypeDefinition::Sequence { identity, element },
                SerializedValue::Sequence(encoded),
            )
        }
        Value::Set {
            element_classifier,
            entries,
        } => {
            let values = entries.clone();
            let (element, mut encoded) = serialize_homogeneous_values(&values, types, identities)?;
            encoded.sort();
            let mut schema_payload = Vec::new();
            put_protocol_uvarint(element, &mut schema_payload);
            put_protocol_text("semantic-total-order", &mut schema_payload);
            let identity = format!("Set {element_classifier}");
            (
                identity.clone(),
                TypeDefinition::ObjectDescription {
                    identity,
                    kind: 11,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![SerializedValue::Sequence(encoded)]),
            )
        }
        Value::Map {
            key_classifier,
            value_classifier,
            entries,
        } => {
            let mut key_type = None;
            let mut value_type = None;
            let mut encoded = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                let (key_id, key) = serialize_language_value(key, types, identities)?;
                let (value_id, value) = serialize_language_value(value, types, identities)?;
                if key_type.replace(key_id).is_some_and(|id| id != key_id)
                    || value_type
                        .replace(value_id)
                        .is_some_and(|id| id != value_id)
                {
                    return Err("map entries do not share one key and value serialization schema");
                }
                encoded.push(SerializedValue::Product(vec![key, value]));
            }
            let (key_type, value_type) = (
                key_type.ok_or("an empty Map needs explicit serializable schemas")?,
                value_type.unwrap(),
            );
            encoded.sort_by(|left, right| match (left, right) {
                (SerializedValue::Product(left), SerializedValue::Product(right)) => {
                    left[0].cmp(&right[0])
                }
                _ => Ordering::Equal,
            });
            let mut schema_payload = Vec::new();
            put_protocol_uvarint(key_type, &mut schema_payload);
            put_protocol_uvarint(value_type, &mut schema_payload);
            put_protocol_text("semantic-total-order", &mut schema_payload);
            let identity = format!("Map ({key_classifier}, {value_classifier})");
            (
                identity.clone(),
                TypeDefinition::ObjectDescription {
                    identity,
                    kind: 12,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![SerializedValue::Sequence(encoded)]),
            )
        }
        _ => {
            let (schema, description) =
                serialize_language_value(&Value::String(value.to_string()), types, identities)?;
            let identity = format!("description:{}", structural_value_classifier(value));
            let kind = format!("{:?}", value.object_kind());
            let mut schema_payload = Vec::new();
            put_protocol_text(&kind, &mut schema_payload);
            put_protocol_uvarint(schema, &mut schema_payload);
            (
                identity.clone(),
                TypeDefinition::ObjectDescription {
                    identity,
                    kind: 15,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![description]),
            )
        }
    };
    if let Some(id) = identities.get(&identity) {
        return Ok((*id, serialized));
    }
    let id = types.len();
    types.push(definition);
    identities.insert(identity, id);
    Ok((id, serialized))
}

fn serialize_homogeneous_values(
    values: &[Value],
    types: &mut Vec<TypeDefinition>,
    identities: &mut BTreeMap<String, usize>,
) -> Result<(usize, Vec<SerializedValue>), &'static str> {
    let mut element = None;
    let mut encoded = Vec::with_capacity(values.len());
    for value in values {
        let (id, value) = serialize_language_value(value, types, identities)?;
        if element.replace(id).is_some_and(|existing| existing != id) {
            return Err("collection entries do not share one native serialization type");
        }
        encoded.push(value);
    }
    let element =
        element.ok_or("an empty collection needs an explicit serializable element schema")?;
    Ok((element, encoded))
}

fn put_protocol_uvarint(mut value: usize, output: &mut Vec<u8>) {
    loop {
        let mut byte = u8::try_from(value & 0x7f).expect("seven-bit payload fits in u8");
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn put_protocol_text(value: &str, output: &mut Vec<u8>) {
    put_protocol_uvarint(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}

fn value_from_serialized(event: &SerializedEvent, types: &[TypeDefinition]) -> Option<Value> {
    deserialize_language_value(event.type_id, &event.value, types)
}

#[allow(clippy::too_many_lines)] // Schema reconstruction mirrors the explicit serializer cases.
fn deserialize_language_value(
    type_id: usize,
    value: &SerializedValue,
    types: &[TypeDefinition],
) -> Option<Value> {
    match (types.get(type_id)?, value) {
        (TypeDefinition::Unit { .. }, SerializedValue::Unit) => Some(Value::Unit),
        (TypeDefinition::Boolean { .. }, SerializedValue::Boolean(value)) => {
            Some(Value::Boolean(*value))
        }
        (TypeDefinition::Int { .. }, SerializedValue::Int(value)) => {
            Some(Value::Int(BigInt::from(*value)))
        }
        (TypeDefinition::Int { .. }, SerializedValue::ArbitraryInt(value)) => {
            Some(Value::Int(value.clone()))
        }
        (TypeDefinition::Text { .. }, SerializedValue::Text(value)) => {
            Some(Value::String(value.clone()))
        }
        (TypeDefinition::Tuple { components, .. }, SerializedValue::Product(values)) => {
            Some(Value::Tuple(
                components
                    .iter()
                    .zip(values)
                    .map(|(id, value)| deserialize_language_value(*id, value, types))
                    .collect::<Option<Vec<_>>>()?,
            ))
        }
        (TypeDefinition::Record { fields, .. }, SerializedValue::Product(values)) => {
            Some(Value::Record(
                fields
                    .iter()
                    .zip(values)
                    .map(|((label, id), value)| {
                        Some((
                            label.clone(),
                            deserialize_language_value(*id, value, types)?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?,
            ))
        }
        (
            TypeDefinition::ObjectDescription {
                identity, kind: 3, ..
            },
            SerializedValue::ObjectDescription(values),
        ) if identity == "Rational" => {
            let [
                SerializedValue::ArbitraryInt(numerator),
                SerializedValue::ArbitraryInt(denominator),
            ] = values.as_slice()
            else {
                return None;
            };
            Some(Value::Rational(BigRational::new(
                numerator.clone(),
                denominator.clone(),
            )))
        }
        (TypeDefinition::Sequence { identity, element }, SerializedValue::Sequence(values)) => {
            let entries = values
                .iter()
                .map(|value| deserialize_language_value(*element, value, types))
                .collect::<Option<Vec<_>>>()?;
            if let Some(classifier) = identity.strip_prefix("List ") {
                Some(Value::List {
                    element_classifier: classifier.into(),
                    entries,
                })
            } else if let Some(rest) = identity.strip_prefix("Array ") {
                let (_, classifier) = rest.split_once(' ')?;
                Some(Value::Array {
                    element_classifier: classifier.into(),
                    entries,
                })
            } else {
                identity.strip_prefix("Bag ").map(|classifier| Value::Bag {
                    element_classifier: classifier.into(),
                    entries: entries.into_iter().map(|value| (value, 1)).collect(),
                })
            }
        }
        (
            TypeDefinition::ObjectDescription {
                identity,
                kind: 11,
                schema_payload,
            },
            SerializedValue::ObjectDescription(values),
        ) => {
            let [SerializedValue::Sequence(values)] = values.as_slice() else {
                return None;
            };
            let (element, _) = read_protocol_uvarint(schema_payload)?;
            let entries = values
                .iter()
                .map(|value| deserialize_language_value(element, value, types))
                .collect::<Option<Vec<_>>>()?;
            Some(Value::Set {
                element_classifier: identity.strip_prefix("Set ")?.into(),
                entries,
            })
        }
        (
            TypeDefinition::ObjectDescription {
                identity,
                kind: 12,
                schema_payload,
            },
            SerializedValue::ObjectDescription(values),
        ) => {
            let [SerializedValue::Sequence(values)] = values.as_slice() else {
                return None;
            };
            let (key_type, used) = read_protocol_uvarint(schema_payload)?;
            let (value_type, _) = read_protocol_uvarint(&schema_payload[used..])?;
            let entries = values
                .iter()
                .map(|entry| {
                    let SerializedValue::Product(pair) = entry else {
                        return None;
                    };
                    let [key, value] = pair.as_slice() else {
                        return None;
                    };
                    Some((
                        deserialize_language_value(key_type, key, types)?,
                        deserialize_language_value(value_type, value, types)?,
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            let classifiers = identity.strip_prefix("Map (")?.strip_suffix(')')?;
            let (key_classifier, value_classifier) = classifiers.split_once(", ")?;
            Some(Value::Map {
                key_classifier: key_classifier.into(),
                value_classifier: value_classifier.into(),
                entries,
            })
        }
        (
            TypeDefinition::ObjectDescription {
                identity,
                kind: 15,
                schema_payload,
            },
            SerializedValue::ObjectDescription(values),
        ) => {
            let (kind, used) = read_protocol_text(schema_payload)?;
            let (schema, _) = read_protocol_uvarint(&schema_payload[used..])?;
            let [description] = values.as_slice() else {
                return None;
            };
            Some(Value::ObjectDescription {
                identity: identity.clone(),
                kind,
                value: Box::new(deserialize_language_value(schema, description, types)?),
            })
        }
        _ => None,
    }
}

fn read_protocol_uvarint(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0_usize;
    for (index, byte) in bytes.iter().copied().enumerate() {
        let shift = index.checked_mul(7)?;
        value |= usize::from(byte & 0x7f).checked_shl(u32::try_from(shift).ok()?)?;
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
    }
    None
}

fn read_protocol_text(bytes: &[u8]) -> Option<(String, usize)> {
    let (length, used) = read_protocol_uvarint(bytes)?;
    let end = used.checked_add(length)?;
    Some((std::str::from_utf8(bytes.get(used..end)?).ok()?.into(), end))
}

fn generator_classifier_diagnostic(
    source: &SourceText,
    code: &'static str,
    span: Span,
    name: &str,
    action: &str,
    expected: &str,
    value: &Value,
) -> Diagnostic {
    let found = structural_value_classifier(value);
    diagnostic(
        source,
        code,
        span,
        format!("generator `{name}` {action} `{found}`, but its declaration requires `{expected}`"),
    )
    .with_help(format!(
        "produce `{expected}` here or change the generator's declared classifier from `{expected}`"
    ))
}

fn classifier_expression(source: &SourceText, expression: &Expression) -> Option<String> {
    match expression {
        Expression::Identifier(span) => Some(source.slice(*span).to_owned()),
        Expression::Product { fields, .. }
            if fields.len() > 1 && fields.iter().all(|field| field.label.is_none()) =>
        {
            Some(format!(
                "({})",
                fields
                    .iter()
                    .map(|field| classifier_expression(source, &field.value))
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            ))
        }
        _ => None,
    }
}

fn evaluate_boolean_literal(source: &SourceText, span: Span, trace: &mut impl TraceSink) -> Value {
    let lexeme = source.slice(span);
    trace.record(TraceEvent {
        event: "token.boolean",
        rule: "TOPAL-TYPE-BOOLEAN-001",
        detail: lexeme,
    });
    Value::Boolean(lexeme == "true")
}

fn evaluate_integer_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let text = source.slice(span);
    let value = parse_integer(text)
        .ok_or_else(|| diagnostic(source, "E-NUMERIC-LITERAL", span, "invalid integer literal"))?;
    trace.record(TraceEvent {
        event: "token.integer",
        rule: "TOPAL-NUM-LITERAL-001",
        detail: text,
    });
    Ok(Value::Int(value))
}

fn evaluate_rational_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let text = source.slice(span);
    let value = parse_rational(text).ok_or_else(|| {
        diagnostic(
            source,
            "E-NUMERIC-LITERAL",
            span,
            "invalid rational literal",
        )
    })?;
    trace.record(TraceEvent {
        event: "token.rational",
        rule: "TOPAL-NUM-RATIONAL-LITERAL-001",
        detail: text,
    });
    Ok(Value::Rational(value))
}

fn evaluate_string_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let lexeme = source.slice(span);
    let value = parse_string(lexeme).ok_or_else(|| {
        diagnostic(
            source,
            "E-STRING-LITERAL",
            span,
            "invalid string literal delimiter",
        )
    })?;
    trace.record(TraceEvent {
        event: "token.string",
        rule: "TOPAL-SYN-STRING-001",
        detail: lexeme,
    });
    Ok(Value::String(value.to_owned()))
}

fn parse_string(lexeme: &str) -> Option<&str> {
    let opening = lexeme.find('"')?;
    let closing_length = opening + 1;
    (lexeme.len() >= opening + 1 + closing_length)
        .then(|| &lexeme[opening + 1..lexeme.len() - closing_length])
}

fn display_string(value: &str) -> String {
    if !value.contains('"') {
        return format!("\"{value}\"");
    }
    let mut tag = "text".to_owned();
    while value.contains(&format!("\"{tag}")) {
        tag.push('_');
    }
    format!("{tag}\"{value}\"{tag}")
}

#[allow(clippy::too_many_lines)] // Numeric domains keep explicit, non-coercing dispatch arms.
fn apply_binary(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    spans: (Span, Span, Span),
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (span, left_span, right_span) = spans;
    let left = forget_refinement(left, trace, "constraint->base:left");
    let right = forget_refinement(right, trace, "constraint->base:right");
    if matches!(kind, CallableKind::Equal | CallableKind::NotEqual) {
        return apply_equality(source, kind, left, right, span, trace);
    }
    if kind == CallableKind::Compare {
        let Some(ordering) = values_compare(left, right, trace) else {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "the operand types do not share an applicable TotalOrder",
            ));
        };
        let alternative = match ordering {
            Ordering::Less => "Less",
            Ordering::Equal => "Equal",
            Ordering::Greater => "Greater",
        };
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.<=>(TotalOrder,TotalOrder)",
        });
        trace.record(TraceEvent {
            event: "comparison.result",
            rule: "TOPAL-NUM-THREE-WAY-COMPARE-001",
            detail: alternative,
        });
        return Ok(Value::Enum {
            type_name: "Comparison".to_owned(),
            alternative: alternative.to_owned(),
        });
    }
    if matches!(
        kind,
        CallableKind::Range
            | CallableKind::RangeOpen
            | CallableKind::RangeInclusive
            | CallableKind::RangeOpenInclusive
    ) {
        return apply_range(source, kind, left, right, span, trace);
    }
    if matches!(
        kind,
        CallableKind::Less
            | CallableKind::Greater
            | CallableKind::LessEqual
            | CallableKind::GreaterEqual
    ) {
        return apply_comparison(source, kind, left, right, span, trace);
    }
    match (left, right) {
        (
            Value::Modular {
                type_name: left_type,
                lower,
                upper,
                value: left,
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => {
            let raw = match kind {
                CallableKind::Plus => left + right,
                CallableKind::Minus => left - right,
                CallableKind::Multiply => left * right,
                _ => {
                    return Err(diagnostic(
                        source,
                        "E-NO-APPLICABLE-OVERLOAD",
                        span,
                        "modular values support settled wrapping +, -, and * operations",
                    ));
                }
            };
            let value = reduce_modular(raw, &lower, &upper);
            trace.record(TraceEvent {
                event: "numeric.modular.wrapped",
                rule: "TOPAL-NUM-MODULAR-ARITHMETIC-001",
                detail: &left_type,
            });
            Ok(Value::Modular {
                type_name: left_type,
                lower,
                upper,
                value,
            })
        }
        (Value::Int(left), Value::Int(right)) => {
            apply_int_binary(source, kind, left, right, right_span, trace)
        }
        (Value::Rational(left), Value::Rational(right)) => {
            apply_rational_binary(source, kind, left, right, span, right_span, trace)
        }
        (Value::Rational(left), Value::Int(right)) if kind == CallableKind::Power => {
            apply_rational_power(source, left, right, left_span, right_span, trace)
        }
        (Value::Int(left), Value::Rational(right)) if kind != CallableKind::Power => {
            trace_conversion(trace, "Int->Rational:left");
            apply_rational_binary(
                source,
                kind,
                BigRational::from_integer(left),
                right,
                span,
                right_span,
                trace,
            )
        }
        (Value::Rational(left), Value::Int(right)) if kind != CallableKind::Power => {
            trace_conversion(trace, "Int->Rational:right");
            apply_rational_binary(
                source,
                kind,
                left,
                BigRational::from_integer(right),
                span,
                right_span,
                trace,
            )
        }
        _ => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "the implemented subset requires operands from one exact numeric domain",
        )),
    }
}

fn reduce_modular(value: BigInt, lower: &BigInt, upper: &BigInt) -> BigInt {
    let modulus = upper - lower + BigInt::from(1);
    let offset = value - lower;
    let reduced = ((offset % &modulus) + &modulus) % &modulus;
    reduced + lower
}

fn forget_refinement(value: Value, trace: &mut impl TraceSink, detail: &'static str) -> Value {
    if let Value::Refined { value, .. } = value {
        trace_conversion(trace, detail);
        *value
    } else {
        value
    }
}

fn apply_range(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (lower_inclusive, upper_inclusive) = match kind {
        CallableKind::Range => (true, false),
        CallableKind::RangeOpen => (false, false),
        CallableKind::RangeInclusive => (true, true),
        CallableKind::RangeOpenInclusive => (false, true),
        _ => unreachable!("range operator dispatched with range kind"),
    };
    let (range, nonempty) = match (left, right) {
        (Value::Int(lower), Value::Int(upper)) => {
            let nonempty = lower < upper || (lower == upper && lower_inclusive && upper_inclusive);
            (
                Value::IntRange {
                    lower,
                    upper,
                    lower_inclusive,
                    upper_inclusive,
                },
                nonempty,
            )
        }
        (Value::Rational(lower), Value::Rational(upper)) => {
            let nonempty = lower < upper || (lower == upper && lower_inclusive && upper_inclusive);
            (
                Value::RationalRange {
                    lower,
                    upper,
                    lower_inclusive,
                    upper_inclusive,
                },
                nonempty,
            )
        }
        (Value::Int(lower), Value::Rational(upper)) => {
            trace_conversion(trace, "Int->Rational:left");
            let lower = BigRational::from_integer(lower);
            let nonempty = lower < upper || (lower == upper && lower_inclusive && upper_inclusive);
            (
                Value::RationalRange {
                    lower,
                    upper,
                    lower_inclusive,
                    upper_inclusive,
                },
                nonempty,
            )
        }
        (Value::Rational(lower), Value::Int(upper)) => {
            trace_conversion(trace, "Int->Rational:right");
            let upper = BigRational::from_integer(upper);
            let nonempty = lower < upper || (lower == upper && lower_inclusive && upper_inclusive);
            (
                Value::RationalRange {
                    lower,
                    upper,
                    lower_inclusive,
                    upper_inclusive,
                },
                nonempty,
            )
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-ENDPOINTS",
                span,
                "range endpoints require finite Int or Rational values",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.constructed",
        rule: "TOPAL-RANGE-BOUNDS-001",
        detail: if nonempty { "nonempty" } else { "empty" },
    });
    Ok(range)
}

fn apply_range_bound(
    source: &SourceText,
    operation: &str,
    range: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let value = match (operation, range) {
        ("range-lower", Value::IntRange { lower, .. }) => Value::Int(lower),
        ("range-upper", Value::IntRange { upper, .. }) => Value::Int(upper),
        ("range-lower", Value::RationalRange { lower, .. }) => Value::Rational(lower),
        ("range-upper", Value::RationalRange { upper, .. }) => Value::Rational(upper),
        (
            "range-lower-inclusive?",
            Value::IntRange {
                lower_inclusive, ..
            },
        )
        | (
            "range-lower-inclusive?",
            Value::RationalRange {
                lower_inclusive, ..
            },
        ) => Value::Boolean(lower_inclusive),
        (
            "range-upper-inclusive?",
            Value::IntRange {
                upper_inclusive, ..
            },
        )
        | (
            "range-upper-inclusive?",
            Value::RationalRange {
                upper_inclusive, ..
            },
        ) => Value::Boolean(upper_inclusive),
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-BOUND-OPERAND",
                span,
                format!("{operation} requires a bounded exact Range operand"),
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.bound.observed",
        rule: "TOPAL-RANGE-BOUND-001",
        detail: operation,
    });
    Ok(value)
}

fn apply_range_membership(
    source: &SourceText,
    callable: &str,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let operands = match (callable, left, right) {
        (
            "in",
            Value::Int(value),
            Value::IntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
        )
        | (
            "contains",
            Value::IntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
            Value::Int(value),
        ) => Some((
            BigRational::from_integer(value),
            BigRational::from_integer(lower),
            BigRational::from_integer(upper),
            lower_inclusive,
            upper_inclusive,
        )),
        (
            "in",
            Value::Rational(value),
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
        )
        | (
            "contains",
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
            Value::Rational(value),
        ) => Some((value, lower, upper, lower_inclusive, upper_inclusive)),
        (
            "in",
            Value::Int(value),
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
        )
        | (
            "contains",
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
            Value::Int(value),
        ) => {
            trace_conversion(trace, "Int->Rational:membership");
            Some((
                BigRational::from_integer(value),
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            ))
        }
        _ => None,
    };
    let Some((value, lower, upper, lower_inclusive, upper_inclusive)) = operands else {
        return Err(diagnostic(
            source,
            "E-RANGE-MEMBERSHIP-OPERANDS",
            span,
            "range membership requires compatible exact numeric operands",
        ));
    };
    let accepted = bound_contains(&value, &lower, &upper, lower_inclusive, upper_inclusive);
    trace.record(TraceEvent {
        event: "range.membership.tested",
        rule: "TOPAL-RANGE-MEMBERSHIP-001",
        detail: if accepted { "accepted" } else { "rejected" },
    });
    Ok(Value::Boolean(accepted))
}

fn apply_and(
    source: &SourceText,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let (Value::Boolean(left), Value::Boolean(right)) = (&left, &right) {
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.and(Boolean,Boolean)",
        });
        trace.record(TraceEvent {
            event: "evaluation.logical",
            rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
            detail: "and:eager",
        });
        return Ok(Value::Boolean(*left && *right));
    }
    if let (Value::Capability(left), Value::Capability(right)) = (&left, &right) {
        let alternatives = left
            .iter()
            .flat_map(|left| {
                right.iter().map(move |right| {
                    let mut combined = left.clone();
                    combined.extend(right.iter().cloned());
                    combined
                })
            })
            .collect();
        trace.record(TraceEvent {
            event: "capability.composed",
            rule: "TOPAL-CAPABILITY-EVIDENCE-001",
            detail: "and",
        });
        return Ok(Value::Capability(alternatives));
    }
    let result = match (left, right) {
        (
            Value::IntRange {
                lower: left_lower,
                upper: left_upper,
                lower_inclusive: left_lower_inclusive,
                upper_inclusive: left_upper_inclusive,
            },
            Value::IntRange {
                lower: right_lower,
                upper: right_upper,
                lower_inclusive: right_lower_inclusive,
                upper_inclusive: right_upper_inclusive,
            },
        ) => {
            let (lower, lower_inclusive) = stricter_lower(
                left_lower,
                left_lower_inclusive,
                right_lower,
                right_lower_inclusive,
            );
            let (upper, upper_inclusive) = stricter_upper(
                left_upper,
                left_upper_inclusive,
                right_upper,
                right_upper_inclusive,
            );
            Value::IntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            }
        }
        (
            Value::RationalRange {
                lower: left_lower,
                upper: left_upper,
                lower_inclusive: left_lower_inclusive,
                upper_inclusive: left_upper_inclusive,
            },
            Value::RationalRange {
                lower: right_lower,
                upper: right_upper,
                lower_inclusive: right_lower_inclusive,
                upper_inclusive: right_upper_inclusive,
            },
        ) => {
            let (lower, lower_inclusive) = stricter_lower(
                left_lower,
                left_lower_inclusive,
                right_lower,
                right_lower_inclusive,
            );
            let (upper, upper_inclusive) = stricter_upper(
                left_upper,
                left_upper_inclusive,
                right_upper,
                right_upper_inclusive,
            );
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            }
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-INTERSECTION-OPERANDS",
                span,
                "and requires two Booleans or ranges from the same endpoint domain",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.intersection.constructed",
        rule: "TOPAL-RANGE-INTERSECTION-001",
        detail: "conjunction",
    });
    Ok(result)
}

fn apply_comparison(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let tuple = matches!((&left, &right), (Value::Tuple(_), Value::Tuple(_)));
    let Some(ordering) = values_compare(left, right, trace) else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "ordering requires operands with shared TotalOrder evidence",
        ));
    };
    let (callable, result) = match kind {
        CallableKind::Less => ("root.<(TotalOrder,TotalOrder)", ordering == Ordering::Less),
        CallableKind::Greater => (
            "root.>(TotalOrder,TotalOrder)",
            ordering == Ordering::Greater,
        ),
        CallableKind::LessEqual => (
            "root.<=(TotalOrder,TotalOrder)",
            ordering != Ordering::Greater,
        ),
        CallableKind::GreaterEqual => {
            ("root.>=(TotalOrder,TotalOrder)", ordering != Ordering::Less)
        }
        _ => unreachable!("comparison dispatch accepts only ordering predicates"),
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: callable,
    });
    trace.record(TraceEvent {
        event: "comparison.result",
        rule: if tuple {
            "TOPAL-TYPE-ORDERING-001"
        } else {
            "TOPAL-NUM-COMPARE-001"
        },
        detail: match ordering {
            Ordering::Less => "Less",
            Ordering::Equal => "Equal",
            Ordering::Greater => "Greater",
        },
    });
    Ok(Value::Boolean(result))
}

fn values_compare(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<Ordering> {
    match (left, right) {
        (Value::Refined { value, .. }, right) => values_compare(*value, right, trace),
        (left, Value::Refined { value, .. }) => values_compare(left, *value, trace),
        (
            Value::Modular {
                type_name: left_type,
                value: left,
                ..
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => Some(left.cmp(&right)),
        (Value::Int(left), Value::Int(right)) => Some(left.cmp(&right)),
        (Value::Rational(left), Value::Rational(right)) => Some(left.cmp(&right)),
        (Value::Int(left), Value::Rational(right)) => {
            trace_conversion(trace, "Int->Rational:left");
            Some(BigRational::from_integer(left).cmp(&right))
        }
        (Value::Rational(left), Value::Int(right)) => {
            trace_conversion(trace, "Int->Rational:right");
            Some(left.cmp(&BigRational::from_integer(right)))
        }
        (Value::Tuple(left), Value::Tuple(right)) if left.len() == right.len() => {
            for (left, right) in left.into_iter().zip(right) {
                let ordering = values_compare(left, right, trace)?;
                if ordering != Ordering::Equal {
                    return Some(ordering);
                }
            }
            Some(Ordering::Equal)
        }
        _ => None,
    }
}

fn apply_equality(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Some(equal) = values_equal(left, right, trace) else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "the operand types do not share an applicable Equality operation",
        ));
    };
    let equal = if kind == CallableKind::NotEqual {
        !equal
    } else {
        equal
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: if kind == CallableKind::NotEqual {
            "root.!=(Equality,Equality)"
        } else {
            "root.=(Equality,Equality)"
        },
    });
    trace.record(TraceEvent {
        event: "evaluation.equal",
        rule: "TOPAL-TYPE-EQUALITY-001",
        detail: if equal { "true" } else { "false" },
    });
    Ok(Value::Boolean(equal))
}

fn is_singleton_list_construction(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(callable), _] if source.slice(*callable) == "one")
}

fn evaluate_singleton_list(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [_, entry] = items else {
        unreachable!("singleton construction shape checked")
    };
    let entry = session.evaluate_expression(source, entry, trace)?;
    Ok(construct_singleton_list(entry, trace))
}

fn is_explicit_empty_list_construction(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(empty), Expression::Identifier(list), _]
        if source.slice(*empty) == "empty" && source.slice(*list) == "List")
}

fn evaluate_empty_list(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [
        Expression::Identifier(empty),
        Expression::Identifier(list),
        element,
    ] = items
    else {
        unreachable!("empty List construction shape checked")
    };
    debug_assert_eq!(source.slice(*empty), "empty");
    debug_assert_eq!(source.slice(*list), "List");
    let Some(element_classifier) = classifier_expression(source, element) else {
        return Err(diagnostic(
            source,
            "E-LIST-ELEMENT-CLASSIFIER",
            element.span(),
            "empty List requires a supported element classifier",
        ));
    };
    Ok(construct_empty_list(element_classifier, trace))
}

fn construct_singleton_list(entry: Value, trace: &mut impl TraceSink) -> Value {
    let element_classifier = structural_value_classifier(&entry);
    let selection = format!("root.one({element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.singleton.constructed",
        rule: "TOPAL-LIST-ONE-001",
        detail: &element_classifier,
    });
    Value::List {
        element_classifier,
        entries: vec![entry],
    }
}

fn construct_empty_list(element_classifier: String, trace: &mut impl TraceSink) -> Value {
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.empty({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.empty.constructed",
        rule: "TOPAL-LIST-EMPTY-001",
        detail: &element_classifier,
    });
    Value::List {
        element_classifier,
        entries: Vec::new(),
    }
}

fn apply_empty_predicate(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (is_empty, classifier, event, rule) = match operand {
        Value::String(text) => (
            text.is_empty(),
            "String".to_owned(),
            "string.empty.tested",
            "TOPAL-STRING-EMPTY-PREDICATE-001",
        ),
        Value::List {
            element_classifier,
            entries,
        } => (
            entries.is_empty(),
            format!("List {element_classifier}"),
            "list.empty.tested",
            "TOPAL-LIST-EMPTY-PREDICATE-001",
        ),
        Value::Array { entries, .. } | Value::Set { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::Bag { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::Map { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::IntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => (
            lower > upper || (lower == upper && !(lower_inclusive && upper_inclusive)),
            "Range Int".into(),
            "range.empty.tested",
            "TOPAL-RANGE-EMPTY-001",
        ),
        Value::RationalRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => (
            lower > upper || (lower == upper && !(lower_inclusive && upper_inclusive)),
            "Range Rational".into(),
            "range.empty.tested",
            "TOPAL-RANGE-EMPTY-001",
        ),
        value => {
            let found = structural_value_classifier(&value);
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("empty? requires a collection, text, or Range operand, found `{found}`"),
            ));
        }
    };
    let selection = format!("root.empty?({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event,
        rule,
        detail: if is_empty { "true" } else { "false" },
    });
    Ok(Value::Boolean(is_empty))
}

fn is_list_uncons(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(name), _] if source.slice(*name) == "uncons")
}

fn evaluate_list_uncons(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [_, operand] = items else {
        unreachable!("uncons expression shape checked")
    };
    let operand_span = operand.span();
    let operand = session.evaluate_expression(source, operand, trace)?;
    let value = apply_list_uncons(source, operand, operand_span, trace)?;
    session.checkpoint(trace, Some(&value), Some(span));
    Ok(value)
}

fn apply_list_uncons(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = operand
    else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "uncons requires a List operand",
        ));
    };
    let payload_classifier = format!("({element_classifier}, List {element_classifier})");
    let payload = if entries.is_empty() {
        None
    } else {
        let first = entries.remove(0);
        Some(Box::new(Value::Tuple(vec![
            first,
            Value::List {
                element_classifier: element_classifier.clone(),
                entries,
            },
        ])))
    };
    let selection = format!("root.uncons(List {element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.uncons",
        rule: "TOPAL-LIST-UNCONS-001",
        detail: if payload.is_some() { "Some" } else { "None" },
    });
    Ok(Value::Optional {
        payload_classifier,
        payload,
    })
}

fn is_list_projection(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(name), _]
        if matches!(source.slice(*name), "first" | "rest"))
}

fn evaluate_list_projection(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [Expression::Identifier(operation), operand] = items else {
        unreachable!("List projection expression shape checked")
    };
    let operation = source.slice(*operation);
    let operand_span = operand.span();
    let operand = session.evaluate_expression(source, operand, trace)?;
    let Value::List {
        element_classifier,
        mut entries,
    } = operand
    else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            operand_span,
            format!("{operation} requires a List operand"),
        ));
    };
    let (payload_classifier, payload) = if operation == "first" {
        (
            element_classifier.clone(),
            (!entries.is_empty()).then(|| Box::new(entries.remove(0))),
        )
    } else {
        let payload_classifier = format!("List {element_classifier}");
        let payload = if entries.is_empty() {
            None
        } else {
            entries.remove(0);
            Some(Box::new(Value::List {
                element_classifier: element_classifier.clone(),
                entries,
            }))
        };
        (payload_classifier, payload)
    };
    let selection = format!("root.{operation}(List {element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: if operation == "first" {
            "list.first"
        } else {
            "list.rest"
        },
        rule: if operation == "first" {
            "TOPAL-LIST-FIRST-001"
        } else {
            "TOPAL-LIST-REST-001"
        },
        detail: if payload.is_some() { "Some" } else { "None" },
    });
    let value = Value::Optional {
        payload_classifier,
        payload,
    };
    session.checkpoint(trace, Some(&value), Some(span));
    Ok(value)
}

fn apply_string_utility(
    source: &SourceText,
    operation: &str,
    argument: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let result = match (operation, argument) {
        ("string-trim", Value::String(text)) => {
            Value::String(text.trim_matches(is_unicode_white_space).to_owned())
        }
        ("string-starts-with" | "string-ends-with" | "string-contains", Value::Tuple(values))
            if values.len() == 2 =>
        {
            let [Value::String(text), Value::String(pattern)] = values.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    format!("{operation} requires two String operands"),
                ));
            };
            Value::Boolean(match operation {
                "string-starts-with" => text.starts_with(pattern),
                "string-ends-with" => text.ends_with(pattern),
                _ => text.contains(pattern),
            })
        }
        ("string-replace-all", Value::Tuple(values)) if values.len() == 3 => {
            let [
                Value::String(text),
                Value::String(pattern),
                Value::String(replacement),
            ] = values.as_slice()
            else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-replace-all requires three String operands",
                ));
            };
            Value::String(text.replace(pattern, replacement))
        }
        ("string-repeat", Value::Tuple(values)) if values.len() == 2 => {
            let [Value::String(text), Value::Int(count)] = values.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-repeat requires String and Nat operands",
                ));
            };
            let count = usize::try_from(count).map_err(|_| {
                diagnostic(
                    source,
                    "E-STRING-REPEAT-COUNT",
                    span,
                    "string repetition count is outside the executable platform limit",
                )
            })?;
            Value::String(text.repeat(count))
        }
        ("string-count-exact" | "string-find-all", Value::Tuple(values)) if values.len() == 2 => {
            let [Value::String(text), Value::String(pattern)] = values.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    format!("{operation} requires two String operands"),
                ));
            };
            let text = characters(text).collect::<Vec<_>>();
            let pattern = characters(pattern).collect::<Vec<_>>();
            if pattern.is_empty() {
                return Err(diagnostic(
                    source,
                    "E-STRING-EMPTY-PATTERN",
                    span,
                    format!("{operation} requires a nonempty pattern"),
                ));
            }
            let indexes = text
                .windows(pattern.len())
                .enumerate()
                .filter_map(|(index, candidate)| (candidate == pattern.as_slice()).then_some(index))
                .collect::<Vec<_>>();
            if operation == "string-count-exact" {
                Value::Int(BigInt::from(indexes.len()))
            } else {
                Value::List {
                    element_classifier: "Nat".into(),
                    entries: indexes
                        .into_iter()
                        .map(|index| Value::Int(BigInt::from(index)))
                        .collect(),
                }
            }
        }
        ("string-split-exact", Value::Tuple(values)) if values.len() == 2 => {
            let [Value::String(text), Value::String(pattern)] = values.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-split-exact requires two String operands",
                ));
            };
            if pattern.is_empty() {
                return Err(diagnostic(
                    source,
                    "E-STRING-EMPTY-PATTERN",
                    span,
                    "string-split-exact requires a nonempty pattern",
                ));
            }
            Value::List {
                element_classifier: "String".into(),
                entries: text
                    .split(pattern)
                    .map(|part| Value::String(part.to_owned()))
                    .collect(),
            }
        }
        ("string-glob-matches", Value::Tuple(values)) if values.len() == 2 => {
            let [Value::String(text), Value::String(pattern)] = values.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-glob-matches requires two String operands",
                ));
            };
            Value::Boolean(glob_matches(text, pattern))
        }
        ("string-regex-contains", Value::Tuple(values)) if values.len() == 2 => {
            let [Value::String(text), Value::String(pattern)] = values.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-regex-contains requires two String operands",
                ));
            };
            let expression = Regex::new(pattern).map_err(|error| {
                diagnostic(
                    source,
                    "E-REGEX-PATTERN",
                    span,
                    format!("invalid regular expression: {error}"),
                )
            })?;
            Value::Boolean(expression.is_match(text))
        }
        ("string-contains-any", Value::Tuple(values)) if values.len() == 2 => {
            let [
                Value::String(text),
                Value::List {
                    element_classifier,
                    entries,
                },
            ] = values.as_slice()
            else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-contains-any requires String and List String operands",
                ));
            };
            if element_classifier != "String" {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-contains-any requires List String patterns",
                ));
            }
            Value::Boolean(
                entries
                    .iter()
                    .any(|entry| matches!(entry, Value::String(pattern) if text.contains(pattern))),
            )
        }
        ("string-lines", Value::String(text)) => Value::List {
            element_classifier: "String".into(),
            entries: text
                .lines()
                .map(|line| Value::String(line.to_owned()))
                .collect(),
        },
        ("string-words", Value::String(text)) => Value::List {
            element_classifier: "String".into(),
            entries: text
                .split_whitespace()
                .map(|word| Value::String(word.to_owned()))
                .collect(),
        },
        ("string-join", Value::Tuple(values)) if values.len() == 2 => {
            let [
                Value::List {
                    element_classifier,
                    entries,
                },
                Value::String(separator),
            ] = values.as_slice()
            else {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-join requires List String and String operands",
                ));
            };
            if element_classifier != "String" {
                return Err(diagnostic(
                    source,
                    "E-STRING-UTILITY-OPERANDS",
                    span,
                    "string-join requires List String entries",
                ));
            }
            let parts = entries
                .iter()
                .map(|entry| match entry {
                    Value::String(part) => part.as_str(),
                    _ => unreachable!("List String contains String values"),
                })
                .collect::<Vec<_>>();
            Value::String(parts.join(separator))
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-STRING-UTILITY-OPERANDS",
                span,
                format!("{operation} received unsupported operands"),
            ));
        }
    };
    trace.record(TraceEvent {
        event: "string.utility.applied",
        rule: "TOPAL-STRING-UTILITY-001",
        detail: operation,
    });
    Ok(result)
}

fn is_unicode_white_space(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

fn glob_matches(text: &str, pattern: &str) -> bool {
    let text = characters(text).collect::<Vec<_>>();
    let pattern = characters(pattern).collect::<Vec<_>>();
    let mut matched = vec![vec![false; text.len() + 1]; pattern.len() + 1];
    matched[0][0] = true;
    for pattern_index in 1..=pattern.len() {
        if pattern[pattern_index - 1] == "*" {
            matched[pattern_index][0] = matched[pattern_index - 1][0];
        }
        for text_index in 1..=text.len() {
            matched[pattern_index][text_index] = if pattern[pattern_index - 1] == "*" {
                matched[pattern_index - 1][text_index] || matched[pattern_index][text_index - 1]
            } else {
                (pattern[pattern_index - 1] == "?"
                    || pattern[pattern_index - 1] == text[text_index - 1])
                    && matched[pattern_index - 1][text_index - 1]
            };
        }
    }
    matched[pattern.len()][text.len()]
}

fn string_list(value: &Value) -> Option<Vec<String>> {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        return None;
    };
    if element_classifier != "String" {
        return None;
    }
    entries
        .iter()
        .map(|entry| match entry {
            Value::String(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

fn graph_edges(value: &Value) -> Option<Vec<(String, String)>> {
    let Value::List { entries, .. } = value else {
        return None;
    };
    entries
        .iter()
        .map(|entry| match entry {
            Value::Tuple(fields) if fields.len() == 2 => match fields.as_slice() {
                [Value::String(source), Value::String(destination)] => {
                    Some((source.clone(), destination.clone()))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn weighted_graph_edges(value: &Value) -> Option<Vec<(String, String, BigRational)>> {
    let Value::List { entries, .. } = value else {
        return None;
    };
    entries
        .iter()
        .map(|entry| match entry {
            Value::Tuple(fields) if fields.len() == 3 => match fields.as_slice() {
                [
                    Value::String(source),
                    Value::String(destination),
                    Value::Rational(weight),
                ] => Some((source.clone(), destination.clone(), weight.clone())),
                [
                    Value::String(source),
                    Value::String(destination),
                    Value::Int(weight),
                ] => Some((
                    source.clone(),
                    destination.clone(),
                    BigRational::from_integer(weight.clone()),
                )),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn adjacency(
    nodes: &[String],
    edges: &[(String, String)],
    undirected: bool,
) -> BTreeMap<String, Vec<String>> {
    let mut result = nodes
        .iter()
        .cloned()
        .map(|node| (node, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for (source, destination) in edges {
        result
            .entry(source.clone())
            .or_default()
            .push(destination.clone());
        result.entry(destination.clone()).or_default();
        if undirected {
            result
                .entry(destination.clone())
                .or_default()
                .push(source.clone());
        }
    }
    result
}

fn string_list_value(entries: Vec<String>) -> Value {
    Value::List {
        element_classifier: "String".into(),
        entries: entries.into_iter().map(Value::String).collect(),
    }
}

fn reconstruct_path(
    start: &str,
    destination: &str,
    previous: &BTreeMap<String, String>,
) -> Option<Vec<String>> {
    let mut path = vec![destination.to_owned()];
    while path.last().is_some_and(|node| node != start) {
        path.push(previous.get(path.last()?)?.clone());
    }
    path.reverse();
    Some(path)
}

fn apply_graph_algorithm(
    source: &SourceText,
    operation: &str,
    argument: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Tuple(fields) = argument else {
        return Err(diagnostic(
            source,
            "E-GRAPH-OPERANDS",
            span,
            format!("{operation} requires a packaged graph"),
        ));
    };
    let result = match operation {
        "graph-bfs" | "graph-dfs" => {
            let [Value::String(start), edges, nodes] = fields.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    format!("{operation} requires (String, edges, nodes)"),
                ));
            };
            let edges = graph_edges(edges).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph edges require List (String, String)",
                )
            })?;
            let nodes = string_list(nodes).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph nodes require List String",
                )
            })?;
            let adjacent = adjacency(&nodes, &edges, false);
            let mut visited = BTreeSet::new();
            let mut order = Vec::new();
            let mut frontier = VecDeque::from([start.clone()]);
            while let Some(node) = if operation == "graph-bfs" {
                frontier.pop_front()
            } else {
                frontier.pop_back()
            } {
                if !visited.insert(node.clone()) {
                    continue;
                }
                order.push(node.clone());
                let neighbors = adjacent.get(&node).cloned().unwrap_or_default();
                if operation == "graph-bfs" {
                    frontier.extend(neighbors);
                } else {
                    frontier.extend(neighbors.into_iter().rev());
                }
            }
            string_list_value(order)
        }
        "graph-shortest-path" => {
            let [
                Value::String(start),
                Value::String(destination),
                edges,
                nodes,
            ] = fields.as_slice()
            else {
                return Err(diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph-shortest-path requires (start, destination, edges, nodes)",
                ));
            };
            let edges = graph_edges(edges).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph edges require List (String, String)",
                )
            })?;
            let nodes = string_list(nodes).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph nodes require List String",
                )
            })?;
            let adjacent = adjacency(&nodes, &edges, false);
            let mut visited = BTreeSet::from([start.clone()]);
            let mut previous = BTreeMap::new();
            let mut frontier = VecDeque::from([start.clone()]);
            while let Some(node) = frontier.pop_front() {
                if &node == destination {
                    break;
                }
                for neighbor in adjacent.get(&node).into_iter().flatten() {
                    if visited.insert(neighbor.clone()) {
                        previous.insert(neighbor.clone(), node.clone());
                        frontier.push_back(neighbor.clone());
                    }
                }
            }
            let path = visited
                .contains(destination)
                .then(|| reconstruct_path(start, destination, &previous))
                .flatten();
            Value::Optional {
                payload_classifier: "List String".into(),
                payload: path.map(|path| Box::new(string_list_value(path))),
            }
        }
        "graph-topological-sort" => {
            let [edges, nodes] = fields.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph-topological-sort requires (edges, nodes)",
                ));
            };
            let edges = graph_edges(edges).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph edges require List (String, String)",
                )
            })?;
            let nodes = string_list(nodes).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph nodes require List String",
                )
            })?;
            let adjacent = adjacency(&nodes, &edges, false);
            let mut incoming = nodes
                .iter()
                .cloned()
                .map(|node| (node, 0usize))
                .collect::<BTreeMap<_, _>>();
            for (_, destination) in &edges {
                *incoming.entry(destination.clone()).or_default() += 1;
            }
            let mut ready = VecDeque::from(
                nodes
                    .iter()
                    .filter(|node| incoming.get(*node) == Some(&0))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            let mut order = Vec::new();
            while let Some(node) = ready.pop_front() {
                order.push(node.clone());
                for destination in adjacent.get(&node).into_iter().flatten() {
                    let count = incoming
                        .get_mut(destination)
                        .expect("destination is registered");
                    *count -= 1;
                    if *count == 0 {
                        ready.push_back(destination.clone());
                    }
                }
            }
            Value::Optional {
                payload_classifier: "List String".into(),
                payload: (order.len() == incoming.len())
                    .then(|| Box::new(string_list_value(order))),
            }
        }
        "graph-weak-components" => {
            let [edges, nodes] = fields.as_slice() else {
                return Err(diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph-weak-components requires (edges, nodes)",
                ));
            };
            let edges = graph_edges(edges).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph edges require List (String, String)",
                )
            })?;
            let nodes = string_list(nodes).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph nodes require List String",
                )
            })?;
            let adjacent = adjacency(&nodes, &edges, true);
            let mut visited = BTreeSet::new();
            let mut components = Vec::new();
            for start in nodes {
                if visited.contains(&start) {
                    continue;
                }
                let mut component = Vec::new();
                let mut frontier = VecDeque::from([start]);
                while let Some(node) = frontier.pop_front() {
                    if !visited.insert(node.clone()) {
                        continue;
                    }
                    component.push(node.clone());
                    frontier.extend(adjacent.get(&node).into_iter().flatten().cloned());
                }
                components.push(string_list_value(component));
            }
            Value::List {
                element_classifier: "List String".into(),
                entries: components,
            }
        }
        "graph-weighted-shortest-path" => {
            let [
                Value::String(start),
                Value::String(destination),
                edges,
                nodes,
            ] = fields.as_slice()
            else {
                return Err(diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph-weighted-shortest-path requires (start, destination, weighted-edges, nodes)",
                ));
            };
            let edges = weighted_graph_edges(edges).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "weighted edges require List (String, String, Rational)",
                )
            })?;
            if edges
                .iter()
                .any(|(_, _, weight)| weight < &BigRational::from_integer(BigInt::from(0)))
            {
                return Err(diagnostic(
                    source,
                    "E-GRAPH-NEGATIVE-WEIGHT",
                    span,
                    "weighted shortest path requires nonnegative weights",
                ));
            }
            let nodes = string_list(nodes).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-GRAPH-OPERANDS",
                    span,
                    "graph nodes require List String",
                )
            })?;
            let mut distance =
                BTreeMap::from([(start.clone(), BigRational::from_integer(BigInt::from(0)))]);
            let mut previous = BTreeMap::new();
            let mut unvisited = nodes.into_iter().collect::<BTreeSet<_>>();
            while !unvisited.is_empty() {
                let current = unvisited
                    .iter()
                    .filter_map(|node| {
                        distance
                            .get(node)
                            .map(|value| (node.clone(), value.clone()))
                    })
                    .min_by(|left, right| left.1.cmp(&right.1));
                let Some((node, node_distance)) = current else {
                    break;
                };
                unvisited.remove(&node);
                if &node == destination {
                    break;
                }
                for (source_node, next, weight) in edges
                    .iter()
                    .filter(|(source_node, _, _)| source_node == &node)
                {
                    let _ = source_node;
                    let candidate = node_distance.clone() + weight;
                    if distance
                        .get(next)
                        .is_none_or(|existing| candidate < *existing)
                    {
                        distance.insert(next.clone(), candidate);
                        previous.insert(next.clone(), node.clone());
                    }
                }
            }
            let payload = distance
                .get(destination)
                .and_then(|total| {
                    reconstruct_path(start, destination, &previous).map(|path| {
                        Value::Tuple(vec![
                            string_list_value(path),
                            Value::Rational(total.clone()),
                        ])
                    })
                })
                .map(Box::new);
            Value::Optional {
                payload_classifier: "(List String, Rational)".into(),
                payload,
            }
        }
        _ => unreachable!("known graph algorithm"),
    };
    trace.record(TraceEvent {
        event: "graph.algorithm.applied",
        rule: "TOPAL-LIB-GRAPH-ADVANCED-001",
        detail: operation,
    });
    Ok(result)
}

fn list_of_lists(element_classifier: &str, entries: Vec<Vec<Value>>) -> Value {
    Value::List {
        element_classifier: format!("List {element_classifier}"),
        entries: entries
            .into_iter()
            .map(|entries| Value::List {
                element_classifier: element_classifier.to_owned(),
                entries,
            })
            .collect(),
    }
}

fn permutations(entries: &[Value]) -> Vec<Vec<Value>> {
    if entries.is_empty() {
        return vec![Vec::new()];
    }
    let mut result = Vec::new();
    for index in 0..entries.len() {
        let mut remainder = entries.to_vec();
        let selected = remainder.remove(index);
        for mut suffix in permutations(&remainder) {
            let mut permutation = vec![selected.clone()];
            permutation.append(&mut suffix);
            result.push(permutation);
        }
    }
    result
}

fn combinations(entries: &[Value], count: usize) -> Vec<Vec<Value>> {
    if count == 0 {
        return vec![Vec::new()];
    }
    if count > entries.len() {
        return Vec::new();
    }
    let mut result = Vec::new();
    for index in 0..=entries.len() - count {
        for mut suffix in combinations(&entries[index + 1..], count - 1) {
            let mut combination = vec![entries[index].clone()];
            combination.append(&mut suffix);
            result.push(combination);
        }
    }
    result
}

fn apply_combinatorial_construction(
    source: &SourceText,
    operation: &str,
    argument: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let result = match (operation, argument) {
        (
            "list-permutations",
            Value::List {
                element_classifier,
                entries,
            },
        ) => list_of_lists(&element_classifier, permutations(&entries)),
        ("list-combinations", Value::Tuple(mut fields)) if fields.len() == 2 => {
            let count = fields.pop().expect("length checked");
            let values = fields.pop().expect("length checked");
            let Value::Int(count) = count else {
                return Err(diagnostic(
                    source,
                    "E-COMBINATORICS-COUNT",
                    span,
                    "combinations requires a Nat count",
                ));
            };
            let count = usize::try_from(count).map_err(|_| {
                diagnostic(
                    source,
                    "E-COMBINATORICS-COUNT",
                    span,
                    "combinations count exceeds this platform's addressable List size",
                )
            })?;
            let Value::List {
                element_classifier,
                entries,
            } = values
            else {
                return Err(diagnostic(
                    source,
                    "E-COMBINATORICS-OPERAND",
                    span,
                    "combinations requires a finite List",
                ));
            };
            list_of_lists(&element_classifier, combinations(&entries, count))
        }
        (
            "list-subsets",
            Value::List {
                element_classifier,
                entries,
            },
        ) => {
            let mut subsets = vec![Vec::new()];
            for entry in entries {
                let additions = subsets
                    .iter()
                    .map(|subset| {
                        let mut addition = subset.clone();
                        addition.push(entry.clone());
                        addition
                    })
                    .collect::<Vec<_>>();
                subsets.extend(additions);
            }
            list_of_lists(&element_classifier, subsets)
        }
        ("list-cartesian-product", Value::Tuple(mut fields)) if fields.len() == 2 => {
            let right = fields.pop().expect("length checked");
            let left = fields.pop().expect("length checked");
            let Value::List {
                element_classifier: right_classifier,
                entries: right_entries,
            } = right
            else {
                return Err(diagnostic(
                    source,
                    "E-COMBINATORICS-OPERAND",
                    span,
                    "Cartesian product requires two finite Lists",
                ));
            };
            let Value::List {
                element_classifier: left_classifier,
                entries: left_entries,
            } = left
            else {
                return Err(diagnostic(
                    source,
                    "E-COMBINATORICS-OPERAND",
                    span,
                    "Cartesian product requires two finite Lists",
                ));
            };
            Value::List {
                element_classifier: format!("({left_classifier}, {right_classifier})"),
                entries: left_entries
                    .iter()
                    .flat_map(|left| {
                        right_entries
                            .iter()
                            .map(move |right| Value::Tuple(vec![left.clone(), right.clone()]))
                    })
                    .collect(),
            }
        }
        (_, value) => {
            return Err(diagnostic(
                source,
                "E-COMBINATORICS-OPERAND",
                span,
                format!(
                    "{operation} does not accept {}",
                    structural_value_classifier(&value)
                ),
            ));
        }
    };
    trace.record(TraceEvent {
        event: "combinatorics.construction.applied",
        rule: "TOPAL-LIB-COMBINATORICS-ADVANCED-001",
        detail: operation,
    });
    Ok(result)
}

fn apply_count(
    source: &SourceText,
    operation: &str,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (count, classifier, event, rule) = match operand {
        Value::String(text) => (
            character_count(&text),
            "String".to_owned(),
            if operation == "entry-count" {
                "string.entry-count"
            } else {
                "string.character-count"
            },
            if operation == "entry-count" {
                "TOPAL-STRING-ENTRY-COUNT-001"
            } else {
                "TOPAL-STRING-CHARACTER-COUNT-001"
            },
        ),
        Value::List {
            element_classifier,
            entries,
        } if operation == "entry-count" => (
            entries.len(),
            format!("List {element_classifier}"),
            "list.entry-count",
            "TOPAL-LIST-ENTRY-COUNT-001",
        ),
        Value::Array { entries, .. } | Value::Set { entries, .. } if operation == "entry-count" => {
            (
                entries.len(),
                "Collection".into(),
                "collection.entry-count",
                "TOPAL-COLLECTION-ENTRY-COUNT-001",
            )
        }
        Value::Bag { entries, .. } if operation == "entry-count" => (
            entries.iter().map(|(_, count)| count).sum(),
            "Bag".into(),
            "collection.entry-count",
            "TOPAL-COLLECTION-ENTRY-COUNT-001",
        ),
        Value::Map { entries, .. } if operation == "entry-count" => (
            entries.len(),
            "Map".into(),
            "collection.entry-count",
            "TOPAL-COLLECTION-ENTRY-COUNT-001",
        ),
        value => {
            let found = structural_value_classifier(&value);
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("{operation} has no overload accepting `{found}`"),
            ));
        }
    };
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    let detail = if event.starts_with("string.") {
        format!("characters={count}")
    } else {
        format!("entries={count}")
    };
    trace.record(TraceEvent {
        event,
        rule,
        detail: &detail,
    });
    Ok(Value::Int(BigInt::from(count)))
}

#[allow(clippy::too_many_lines)] // Keep the collection-query operations together and auditable.
fn apply_collection_query(
    source: &SourceText,
    operation: &str,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Tuple(mut arguments) = operand else {
        return Err(diagnostic(
            source,
            "E-COLLECTION-QUERY-ARGUMENT",
            span,
            format!("{operation} requires one two-field product"),
        ));
    };
    if arguments.len() != 2 {
        return Err(diagnostic(
            source,
            "E-COLLECTION-QUERY-ARGUMENT",
            span,
            format!("{operation} requires one two-field product"),
        ));
    }
    let query = arguments.pop().unwrap();
    let collection = arguments.pop().unwrap();
    let (value, rule, detail) = match (operation, collection) {
        (
            "array-at?",
            Value::Array {
                element_classifier,
                entries,
            },
        ) => {
            let Value::Int(index) = query else {
                return Err(diagnostic(
                    source,
                    "E-ARRAY-INDEX-CLASSIFIER",
                    span,
                    "array-at? requires a Nat index",
                ));
            };
            let payload = usize::try_from(index)
                .ok()
                .and_then(|index| entries.get(index).cloned())
                .map(Box::new);
            (
                Value::Optional {
                    payload_classifier: element_classifier,
                    payload,
                },
                "TOPAL-ARRAY-GET-CHECKED-001",
                "array.checked-access",
            )
        }
        (
            "map-lookup",
            Value::Map {
                key_classifier,
                value_classifier,
                entries,
            },
        ) => {
            if !value_has_classifier(&query, &key_classifier) {
                return Err(diagnostic(
                    source,
                    "E-MAP-KEY-CLASSIFIER",
                    span,
                    format!("map-lookup requires a `{key_classifier}` key"),
                ));
            }
            let payload = entries
                .into_iter()
                .find_map(|(key, value)| {
                    values_equal(key, query.clone(), &mut Vec::new())?.then_some(value)
                })
                .map(Box::new);
            (
                Value::Optional {
                    payload_classifier: value_classifier,
                    payload,
                },
                "TOPAL-MAP-LOOKUP-001",
                "map.lookup",
            )
        }
        (
            "set-contains?",
            Value::Set {
                element_classifier,
                entries,
            },
        ) => {
            if !value_has_classifier(&query, &element_classifier) {
                return Err(diagnostic(
                    source,
                    "E-SET-ELEMENT-CLASSIFIER",
                    span,
                    format!("set-contains? requires a `{element_classifier}` value"),
                ));
            }
            let present = entries
                .into_iter()
                .any(|entry| values_equal(entry, query.clone(), &mut Vec::new()).unwrap_or(false));
            (
                Value::Boolean(present),
                "TOPAL-SET-CONTAINS-001",
                "set.membership",
            )
        }
        (
            "bag-multiplicity",
            Value::Bag {
                element_classifier,
                entries,
            },
        ) => {
            if !value_has_classifier(&query, &element_classifier) {
                return Err(diagnostic(
                    source,
                    "E-BAG-ELEMENT-CLASSIFIER",
                    span,
                    format!("bag-multiplicity requires a `{element_classifier}` value"),
                ));
            }
            let count = entries
                .into_iter()
                .find_map(|(entry, count)| {
                    values_equal(entry, query.clone(), &mut Vec::new())?.then_some(count)
                })
                .unwrap_or(0);
            (
                Value::Int(BigInt::from(count)),
                "TOPAL-BAG-MULTIPLICITY-001",
                "bag.multiplicity",
            )
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("{operation} received the wrong collection kind"),
            ));
        }
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail,
    });
    trace.record(TraceEvent {
        event: detail,
        rule,
        detail: operation,
    });
    Ok(value)
}

fn apply_list_reverse(value: &mut Value, trace: &mut impl TraceSink) {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        unreachable!("List reverse dispatched only for a List")
    };
    entries.reverse();
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.reverse(List)",
    });
    trace.record(TraceEvent {
        event: "list.reversed",
        rule: "TOPAL-LIST-REVERSE-001",
        detail: &classifier,
    });
}

fn apply_list_stable_sort(
    source: &SourceText,
    value: &mut Value,
    descending: bool,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        unreachable!("stable sort dispatched only for a List")
    };
    if !matches!(element_classifier.as_str(), "Int" | "Rational") {
        return Err(diagnostic(
            source,
            "E-LIST-SORT-CLASSIFIER",
            span,
            "stable sorting currently requires List Int or List Rational",
        ));
    }
    entries.sort_by(|left, right| {
        let ordering = values_compare(left.clone(), right.clone(), trace)
            .expect("validated exact numeric entries are totally ordered");
        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: if descending {
            "root.stable-sort-descending(List)"
        } else {
            "root.stable-sort(List)"
        },
    });
    trace.record(TraceEvent {
        event: "list.stably-sorted",
        rule: "TOPAL-LIST-STABLE-SORT-001",
        detail: &classifier,
    });
    Ok(())
}

fn apply_list_sequence_unary(
    source: &SourceText,
    operation: &str,
    value: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        unreachable!("sequence operation dispatched only for a List")
    };
    let result = match operation {
        "list-enumerate" => Value::List {
            element_classifier: format!("(Nat, {element_classifier})"),
            entries: entries
                .into_iter()
                .enumerate()
                .map(|(index, entry)| Value::Tuple(vec![Value::Int(BigInt::from(index)), entry]))
                .collect(),
        },
        "list-group-runs" => {
            let mut groups: Vec<Vec<Value>> = Vec::new();
            for entry in entries {
                if groups.is_empty() {
                    groups.push(vec![entry]);
                    continue;
                }
                let same_run = groups
                    .last()
                    .and_then(|group| group.last())
                    .and_then(|previous| values_equal(previous.clone(), entry.clone(), trace));
                let Some(same_run) = same_run else {
                    return Err(diagnostic(
                        source,
                        "E-LIST-GROUP-CLASSIFIER",
                        span,
                        "group-runs requires entries with Equality",
                    ));
                };
                if same_run {
                    groups.last_mut().expect("a current run exists").push(entry);
                } else {
                    groups.push(vec![entry]);
                }
            }
            Value::List {
                element_classifier: format!("List {element_classifier}"),
                entries: groups
                    .into_iter()
                    .map(|entries| Value::List {
                        element_classifier: element_classifier.clone(),
                        entries,
                    })
                    .collect(),
            }
        }
        _ => unreachable!("known unary sequence operation"),
    };
    trace.record(TraceEvent {
        event: "list.sequence.transformed",
        rule: "TOPAL-LIST-SEQUENCE-ALGORITHMS-001",
        detail: operation,
    });
    Ok(result)
}

#[allow(clippy::too_many_lines)] // Keep ordered List operation dispatch together.
fn apply_list_operation(
    source: &SourceText,
    operation: &str,
    left: Value,
    right: Value,
    right_span: Span,
    right_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = left
    else {
        unreachable!("List operation is dispatched only for a List left operand")
    };
    if operation.starts_with("contains-") {
        return apply_list_containment(
            source,
            operation,
            &element_classifier,
            &entries,
            right,
            right_span,
            trace,
        );
    }
    if matches!(operation, "remove-first" | "remove-all") {
        return apply_list_value_removal(
            source,
            operation,
            element_classifier,
            entries,
            &right,
            right_span,
            trace,
        );
    }
    if matches!(
        operation,
        "split-at" | "take" | "drop" | "remove" | "remove-indexes"
    ) {
        return apply_list_index_operation(
            source,
            operation,
            element_classifier,
            entries,
            right,
            right_span,
            right_is_closed,
            trace,
        );
    }
    if matches!(
        operation,
        "zip-exact" | "zip-shortest" | "list-zip-shortest"
    ) {
        return apply_list_zip(
            source,
            operation,
            &element_classifier,
            entries,
            right,
            right_span,
            trace,
        );
    }
    if matches!(operation, "list-index-of" | "list-last-index-of") {
        if !value_has_classifier(&right, &element_classifier) {
            return Err(diagnostic(
                source,
                "E-LIST-SEARCH-CLASSIFIER",
                right_span,
                format!("{operation} requires an `{element_classifier}` value"),
            ));
        }
        let indexes = entries.iter().enumerate().filter_map(|(index, entry)| {
            values_equal(entry.clone(), right.clone(), trace)
                .and_then(|equal| equal.then_some(index))
        });
        let index = if operation == "list-index-of" {
            indexes.into_iter().next()
        } else {
            indexes.into_iter().last()
        };
        trace.record(TraceEvent {
            event: "list.index.searched",
            rule: "TOPAL-LIST-SEQUENCE-ALGORITHMS-001",
            detail: operation,
        });
        return Ok(Value::Optional {
            payload_classifier: "Nat".into(),
            payload: index.map(|index| Box::new(Value::Int(BigInt::from(index)))),
        });
    }
    if matches!(
        operation,
        "ordered-binary-search" | "ordered-nth" | "ordered-smallest"
    ) {
        if !matches!(element_classifier.as_str(), "Int" | "Rational") {
            return Err(diagnostic(
                source,
                "E-LIST-ORDERED-CLASSIFIER",
                right_span,
                "ordered selection currently requires List Int or List Rational",
            ));
        }
        if operation == "ordered-binary-search" {
            if !value_has_classifier(&right, &element_classifier) {
                return Err(diagnostic(
                    source,
                    "E-LIST-SEARCH-CLASSIFIER",
                    right_span,
                    format!("binary search requires an `{element_classifier}` value"),
                ));
            }
            let index = entries
                .binary_search_by(|entry| {
                    values_compare(entry.clone(), right.clone(), trace)
                        .expect("validated exact numeric entries are totally ordered")
                })
                .ok();
            trace.record(TraceEvent {
                event: "list.binary.searched",
                rule: "TOPAL-LIST-ORDERED-ALGORITHMS-001",
                detail: operation,
            });
            return Ok(Value::Optional {
                payload_classifier: "Nat".into(),
                payload: index.map(|index| Box::new(Value::Int(BigInt::from(index)))),
            });
        }
        let Value::Int(count) = right else {
            return Err(diagnostic(
                source,
                "E-LIST-ORDERED-INDEX",
                right_span,
                format!("{operation} requires a Nat index or count"),
            ));
        };
        let Ok(count) = usize::try_from(count) else {
            return Err(diagnostic(
                source,
                "E-LIST-ORDERED-INDEX",
                right_span,
                format!("{operation} requires a representable Nat"),
            ));
        };
        entries.sort_by(|left, right| {
            values_compare(left.clone(), right.clone(), trace)
                .expect("validated exact numeric entries are totally ordered")
        });
        if operation == "ordered-nth" {
            let payload = entries.get(count).cloned().map(Box::new);
            trace.record(TraceEvent {
                event: "list.order.selected",
                rule: "TOPAL-LIST-ORDERED-ALGORITHMS-001",
                detail: operation,
            });
            return Ok(Value::Optional {
                payload_classifier: element_classifier,
                payload,
            });
        }
        entries.truncate(count);
        trace.record(TraceEvent {
            event: "list.order.selected",
            rule: "TOPAL-LIST-ORDERED-ALGORITHMS-001",
            detail: operation,
        });
        return Ok(Value::List {
            element_classifier,
            entries,
        });
    }
    if operation == "ordered-merge" {
        let Value::List {
            element_classifier: right_classifier,
            entries: right_entries,
        } = right
        else {
            return Err(diagnostic(
                source,
                "E-LIST-ORDERED-MERGE",
                right_span,
                "ordered merge requires another List",
            ));
        };
        if right_classifier != element_classifier
            || !matches!(element_classifier.as_str(), "Int" | "Rational")
        {
            return Err(diagnostic(
                source,
                "E-LIST-ORDERED-MERGE",
                right_span,
                "ordered merge requires exact matching Int or Rational Lists",
            ));
        }
        entries.extend(right_entries);
        entries.sort_by(|left, right| {
            values_compare(left.clone(), right.clone(), trace)
                .expect("validated exact numeric entries are totally ordered")
        });
        trace.record(TraceEvent {
            event: "list.ordered.merged",
            rule: "TOPAL-LIST-ORDERED-ALGORITHMS-001",
            detail: operation,
        });
        return Ok(Value::List {
            element_classifier,
            entries,
        });
    }
    if matches!(
        operation,
        "list-rotate-left" | "list-rotate-right" | "list-chunks" | "list-windows"
    ) {
        let Value::Int(amount) = right else {
            return Err(diagnostic(
                source,
                "E-LIST-SEQUENCE-COUNT",
                right_span,
                format!("{operation} requires a Nat count"),
            ));
        };
        let Ok(amount) = usize::try_from(amount) else {
            return Err(diagnostic(
                source,
                "E-LIST-SEQUENCE-COUNT",
                right_span,
                format!("{operation} requires a representable Nat count"),
            ));
        };
        let value = match operation {
            "list-rotate-left" | "list-rotate-right" => {
                if !entries.is_empty() {
                    let shift = amount % entries.len();
                    if operation == "list-rotate-left" {
                        entries.rotate_left(shift);
                    } else {
                        entries.rotate_right(shift);
                    }
                }
                Value::List {
                    element_classifier,
                    entries,
                }
            }
            "list-chunks" => {
                if amount == 0 {
                    return Err(diagnostic(
                        source,
                        "E-LIST-SEQUENCE-COUNT",
                        right_span,
                        "chunks requires a positive count",
                    ));
                }
                Value::List {
                    element_classifier: format!("List {element_classifier}"),
                    entries: entries
                        .chunks(amount)
                        .map(|chunk| Value::List {
                            element_classifier: element_classifier.clone(),
                            entries: chunk.to_vec(),
                        })
                        .collect(),
                }
            }
            "list-windows" => {
                if amount == 0 {
                    return Err(diagnostic(
                        source,
                        "E-LIST-SEQUENCE-COUNT",
                        right_span,
                        "windows requires a positive count",
                    ));
                }
                Value::List {
                    element_classifier: format!("List {element_classifier}"),
                    entries: entries
                        .windows(amount)
                        .map(|window| Value::List {
                            element_classifier: element_classifier.clone(),
                            entries: window.to_vec(),
                        })
                        .collect(),
                }
            }
            _ => unreachable!(),
        };
        trace.record(TraceEvent {
            event: "list.sequence.transformed",
            rule: "TOPAL-LIST-SEQUENCE-ALGORITHMS-001",
            detail: operation,
        });
        return Ok(value);
    }
    match operation {
        "prepend" | "append" => {
            if !value_has_classifier(&right, &element_classifier) {
                let found = structural_value_classifier(&right);
                return Err(diagnostic(
                    source,
                    "E-LIST-ENTRY-CLASSIFIER",
                    right_span,
                    format!(
                        "{operation} received `{found}`, but this list requires `{element_classifier}`"
                    ),
                )
                .with_help(format!("use a `{element_classifier}` value here")));
            }
            if operation == "prepend" {
                entries.insert(0, right);
            } else {
                entries.push(right);
            }
        }
        "concat" => {
            let Value::List {
                element_classifier: right_classifier,
                entries: right_entries,
            } = right
            else {
                return Err(diagnostic(
                    source,
                    "E-LIST-CONCAT-OPERAND",
                    right_span,
                    "List concat requires another List",
                ));
            };
            if right_classifier != element_classifier {
                return Err(diagnostic(
                    source,
                    "E-LIST-CONCAT-CLASSIFIER",
                    right_span,
                    format!(
                        "cannot concatenate `List {right_classifier}` with `List {element_classifier}`"
                    ),
                )
                .with_help("use Lists with the same element classifier"));
            }
            entries.extend(right_entries);
        }
        _ => unreachable!("known List operation"),
    }
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: match operation {
            "prepend" => "list.prepended",
            "append" => "list.appended",
            "concat" => "list.concatenated",
            _ => unreachable!("known List operation"),
        },
        rule: match operation {
            "prepend" => "TOPAL-LIST-PREPEND-001",
            "append" => "TOPAL-LIST-APPEND-001",
            "concat" => "TOPAL-LIST-CONCAT-001",
            _ => unreachable!("known List operation"),
        },
        detail: &classifier,
    });
    Ok(Value::List {
        element_classifier,
        entries,
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Bounds, source evidence, and trace context remain explicit.
fn apply_list_index_operation(
    source: &SourceText,
    operation: &str,
    element_classifier: String,
    mut entries: Vec<Value>,
    operand: Value,
    operand_span: Span,
    operand_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if operation == "remove-indexes"
        && let Value::IntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } = operand
    {
        let Ok(lower) = usize::try_from(lower) else {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        };
        let Ok(upper) = usize::try_from(upper) else {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        };
        let start = lower + usize::from(!lower_inclusive);
        let end = upper + usize::from(upper_inclusive);
        if start > end || end > entries.len() {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        }
        entries.drain(start..end);
        trace.record(TraceEvent {
            event: "list.entries.removed",
            rule: "TOPAL-LIST-REMOVE-INDEXES-001",
            detail: &format!("start={start};end={end}"),
        });
        return Ok(Value::List {
            element_classifier,
            entries,
        });
    }
    let Value::Int(index) = operand else {
        return Err(diagnostic(
            source,
            "E-LIST-INDEX-CLASSIFIER",
            operand_span,
            format!("{operation} requires a Nat operand"),
        ));
    };
    let Ok(index) = usize::try_from(index) else {
        return list_boundary_failure(source, operation, operand_span, operand_is_closed, trace);
    };
    let valid = if operation == "remove" {
        index < entries.len()
    } else {
        index <= entries.len()
    };
    if !valid {
        return list_boundary_failure(source, operation, operand_span, operand_is_closed, trace);
    }
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &format!("root.{operation}({classifier},Nat)"),
    });
    let value = match operation {
        "split-at" => {
            let suffix = entries.split_off(index);
            Value::Tuple(vec![
                Value::List {
                    element_classifier: element_classifier.clone(),
                    entries,
                },
                Value::List {
                    element_classifier,
                    entries: suffix,
                },
            ])
        }
        "take" => {
            entries.truncate(index);
            Value::List {
                element_classifier,
                entries,
            }
        }
        "drop" => Value::List {
            element_classifier,
            entries: entries.split_off(index),
        },
        "remove" => {
            entries.remove(index);
            Value::List {
                element_classifier,
                entries,
            }
        }
        _ => unreachable!("known indexed List operation"),
    };
    trace.record(TraceEvent {
        event: "list.region.selected",
        rule: match operation {
            "split-at" => "TOPAL-LIST-SPLIT-AT-001",
            "take" => "TOPAL-LIST-TAKE-001",
            "drop" => "TOPAL-LIST-DROP-001",
            "remove" => "TOPAL-LIST-REMOVE-INDEX-001",
            _ => unreachable!("known indexed List operation"),
        },
        detail: &format!("index={index}"),
    });
    Ok(value)
}

fn apply_list_insert_at(
    source: &SourceText,
    list: Value,
    boundary: Value,
    boundary_span: Span,
    inserted: Value,
    inserted_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = list
    else {
        unreachable!("insert-at is dispatched only for List")
    };
    let Value::Int(boundary) = boundary else {
        return Err(diagnostic(
            source,
            "E-LIST-BOUNDARY-CLASSIFIER",
            boundary_span,
            "insert-at boundary must be Nat",
        ));
    };
    let Ok(boundary) = usize::try_from(boundary) else {
        return Ok(list_boundary_error(
            source,
            "insert-at",
            boundary_span,
            trace,
        ));
    };
    if boundary > entries.len() {
        return Ok(list_boundary_error(
            source,
            "insert-at",
            boundary_span,
            trace,
        ));
    }
    let inserted_entries = match inserted {
        Value::List {
            element_classifier: classifier,
            entries,
        } => {
            if classifier != element_classifier {
                return Err(diagnostic(
                    source,
                    "E-LIST-INSERT-CLASSIFIER",
                    inserted_span,
                    format!("cannot insert `List {classifier}` into `List {element_classifier}`"),
                ));
            }
            entries
        }
        value if value_has_classifier(&value, &element_classifier) => vec![value],
        value => {
            return Err(diagnostic(
                source,
                "E-LIST-INSERT-CLASSIFIER",
                inserted_span,
                format!(
                    "insert-at requires `{element_classifier}` or `List {element_classifier}`, found `{}`",
                    structural_value_classifier(&value)
                ),
            ));
        }
    };
    let inserted_count = inserted_entries.len();
    entries.splice(boundary..boundary, inserted_entries);
    trace.record(TraceEvent {
        event: "list.inserted",
        rule: "TOPAL-LIST-INSERT-AT-001",
        detail: &format!("boundary={boundary};count={inserted_count}"),
    });
    Ok(Value::List {
        element_classifier,
        entries,
    })
}

fn apply_list_entries_view(list: Value, trace: &mut impl TraceSink) -> Value {
    let Value::List {
        element_classifier,
        entries,
    } = list
    else {
        unreachable!("entries view is dispatched only for List")
    };
    let entry_classifier = format!("IndexedEntry {element_classifier}");
    let entries = entries
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            Value::Record(vec![
                ("index".into(), Value::Int(BigInt::from(index))),
                ("value".into(), value),
            ])
        })
        .collect();
    trace.record(TraceEvent {
        event: "list.entries.viewed",
        rule: "TOPAL-COLLECTION-ENTRIES-001",
        detail: &entry_classifier,
    });
    Value::List {
        element_classifier: entry_classifier,
        entries,
    }
}

fn list_boundary_error(
    source: &SourceText,
    operation: &str,
    span: Span,
    trace: &mut impl TraceSink,
) -> Value {
    let position = source.position(span.start);
    trace.record(TraceEvent {
        event: "list.boundary.rejected",
        rule: "TOPAL-LIST-BOUNDARY-CHECK-001",
        detail: operation,
    });
    Value::Error {
        domain: if operation.starts_with("zip-") {
            format!("root.{operation}(List,List)")
        } else {
            format!("root.{operation}(List,Nat)")
        },
        code: "out-of-range".into(),
        line: position.line,
        column: position.column,
    }
}

fn list_boundary_failure(
    source: &SourceText,
    operation: &str,
    span: Span,
    operand_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if operand_is_closed {
        return Err(diagnostic(
            source,
            "E-LIST-BOUNDARY-OUT-OF-RANGE",
            span,
            format!("{operation} operand is outside the List's valid bounds"),
        )
        .with_help(
            "use a boundary no greater than the entry count, or an existing index for remove",
        ));
    }
    Ok(list_boundary_error(source, operation, span, trace))
}

fn apply_list_zip(
    source: &SourceText,
    operation: &str,
    left_classifier: &str,
    left: Vec<Value>,
    right: Value,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier: right_classifier,
        entries: right,
    } = right
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-OPERAND",
            right_span,
            format!("{operation} requires another List"),
        ));
    };
    if operation == "zip-exact" && left.len() != right.len() {
        return Ok(list_boundary_error(source, operation, right_span, trace));
    }
    let entries = left
        .into_iter()
        .zip(right)
        .map(|(left, right)| Value::Tuple(vec![left, right]))
        .collect();
    let pair_classifier = format!("({left_classifier}, {right_classifier})");
    trace.record(TraceEvent {
        event: "list.zipped",
        rule: if operation == "zip-exact" {
            "TOPAL-LIST-ZIP-EXACT-001"
        } else {
            "TOPAL-LIST-ZIP-SHORTEST-001"
        },
        detail: operation,
    });
    Ok(Value::List {
        element_classifier: pair_classifier,
        entries,
    })
}

fn apply_list_unzip(
    source: &SourceText,
    pairs: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List { entries, .. } = pairs else {
        return Err(diagnostic(
            source,
            "E-LIST-UNZIP-SOURCE",
            span,
            "unzip requires a List of two-field products",
        ));
    };
    let mut left = Vec::with_capacity(entries.len());
    let mut right = Vec::with_capacity(entries.len());
    for entry in entries {
        let Value::Tuple(mut fields) = entry else {
            return Err(diagnostic(
                source,
                "E-LIST-UNZIP-ENTRY",
                span,
                "unzip requires every List entry to be a two-field product",
            ));
        };
        if fields.len() != 2 {
            return Err(diagnostic(
                source,
                "E-LIST-UNZIP-ENTRY",
                span,
                "unzip requires every List entry to contain exactly two fields",
            ));
        }
        right.push(fields.pop().expect("two fields"));
        left.push(fields.pop().expect("two fields"));
    }
    let left_classifier = left
        .first()
        .map_or_else(|| "Object".into(), structural_value_classifier);
    let right_classifier = right
        .first()
        .map_or_else(|| "Object".into(), structural_value_classifier);
    trace.record(TraceEvent {
        event: "list.unzipped",
        rule: "TOPAL-LIST-UNZIP-001",
        detail: &format!("count={}", left.len()),
    });
    Ok(Value::Tuple(vec![
        Value::List {
            element_classifier: left_classifier,
            entries: left,
        },
        Value::List {
            element_classifier: right_classifier,
            entries: right,
        },
    ]))
}

fn apply_list_zip_longest(
    source: &SourceText,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Tuple(mut left_fields) = left else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-LEFT",
            span,
            "zip-longest left operand must be `(List, default)`",
        ));
    };
    let Value::Tuple(mut right_fields) = right else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-RIGHT",
            span,
            "zip-longest right operand must be `(List, default)`",
        ));
    };
    if left_fields.len() != 2 || right_fields.len() != 2 {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-OPERAND",
            span,
            "zip-longest operands must each contain a List and its default",
        ));
    }
    let left_default = left_fields.pop().expect("two fields");
    let right_default = right_fields.pop().expect("two fields");
    let Value::List {
        element_classifier: left_classifier,
        entries: left_entries,
    } = left_fields.pop().expect("two fields")
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-LEFT",
            span,
            "first left field must be a List",
        ));
    };
    let Value::List {
        element_classifier: right_classifier,
        entries: right_entries,
    } = right_fields.pop().expect("two fields")
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-RIGHT",
            span,
            "first right field must be a List",
        ));
    };
    if !value_has_classifier(&left_default, &left_classifier)
        || !value_has_classifier(&right_default, &right_classifier)
    {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-DEFAULT",
            span,
            "each zip-longest default must match its List element classifier",
        ));
    }
    let count = left_entries.len().max(right_entries.len());
    let entries = (0..count)
        .map(|index| {
            Value::Tuple(vec![
                left_entries
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| left_default.clone()),
                right_entries
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| right_default.clone()),
            ])
        })
        .collect();
    trace.record(TraceEvent {
        event: "list.zipped",
        rule: "TOPAL-LIST-ZIP-LONGEST-001",
        detail: &format!("count={count}"),
    });
    Ok(Value::List {
        element_classifier: format!("({left_classifier}, {right_classifier})"),
        entries,
    })
}

fn collect_unordered(
    source: &SourceText,
    operation: &str,
    value: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        return Err(diagnostic(
            source,
            "E-UNORDERED-COLLECT-SOURCE",
            span,
            format!("{operation} requires a finite List"),
        ));
    };
    let mut distinct: Vec<(Value, usize)> = Vec::new();
    for entry in entries {
        let mut found = None;
        for (index, (candidate, _)) in distinct.iter().enumerate() {
            if values_equal(candidate.clone(), entry.clone(), trace).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-UNORDERED-COLLECT-EQUALITY",
                    span,
                    format!("`{element_classifier}` must provide equality for {operation}"),
                )
            })? {
                found = Some(index);
                break;
            }
        }
        if let Some(index) = found {
            distinct[index].1 += 1;
        } else {
            distinct.push((entry, 1));
        }
    }
    let count = distinct.len();
    trace.record(TraceEvent {
        event: if operation == "collect-set" {
            "set.collected"
        } else {
            "bag.collected"
        },
        rule: if operation == "collect-set" {
            "TOPAL-SET-COLLECT-001"
        } else {
            "TOPAL-BAG-COLLECT-001"
        },
        detail: &format!("distinct={count}"),
    });
    if operation == "collect-set" {
        Ok(Value::Set {
            element_classifier,
            entries: distinct.into_iter().map(|(value, _)| value).collect(),
        })
    } else {
        Ok(Value::Bag {
            element_classifier,
            entries: distinct,
        })
    }
}

fn collect_map(
    source: &SourceText,
    value: Value,
    policy: &str,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if !matches!(policy, "reject" | "keep-first" | "keep-last") {
        return Err(diagnostic(
            source,
            "E-MAP-COLLISION-POLICY",
            span,
            "collect-map policy must be reject, keep-first, or keep-last",
        ));
    }
    let Value::List { entries, .. } = value else {
        return Err(diagnostic(
            source,
            "E-MAP-COLLECT-SOURCE",
            span,
            "collect-map requires a List of key/value products",
        ));
    };
    let mut mapping: Vec<(Value, Value)> = Vec::new();
    for entry in entries {
        let Value::Tuple(mut pair) = entry else {
            return Err(diagnostic(
                source,
                "E-MAP-COLLECT-ENTRY",
                span,
                "collect-map entries must be two-field products",
            ));
        };
        if pair.len() != 2 {
            return Err(diagnostic(
                source,
                "E-MAP-COLLECT-ENTRY",
                span,
                "collect-map entries must have exactly two fields",
            ));
        }
        let value = pair.pop().expect("two fields");
        let key = pair.pop().expect("two fields");
        let mut collision = None;
        for (index, (candidate, _)) in mapping.iter().enumerate() {
            if values_equal(candidate.clone(), key.clone(), trace).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-MAP-KEY-EQUALITY",
                    span,
                    "map keys must provide equality",
                )
            })? {
                collision = Some(index);
                break;
            }
        }
        match (collision, policy) {
            (Some(_), "reject") => {
                return Err(diagnostic(
                    source,
                    "E-MAP-KEY-COLLISION",
                    span,
                    "collect-map encountered a duplicate key under reject policy",
                ));
            }
            (Some(_), "keep-first") => {}
            (Some(index), "keep-last") => mapping[index].1 = value,
            (None, _) => mapping.push((key, value)),
            _ => unreachable!("validated collision policy"),
        }
    }
    let key_classifier = mapping.first().map_or_else(
        || "Object".into(),
        |(key, _)| structural_value_classifier(key),
    );
    let value_classifier = mapping.first().map_or_else(
        || "Object".into(),
        |(_, value)| structural_value_classifier(value),
    );
    trace.record(TraceEvent {
        event: "map.collected",
        rule: "TOPAL-MAP-COLLECT-001",
        detail: policy,
    });
    Ok(Value::Map {
        key_classifier,
        value_classifier,
        entries: mapping,
    })
}

fn apply_list_value_removal(
    source: &SourceText,
    operation: &str,
    element_classifier: String,
    entries: Vec<Value>,
    target: &Value,
    target_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if !value_has_classifier(target, &element_classifier) {
        let found = structural_value_classifier(target);
        return Err(diagnostic(
            source,
            "E-LIST-REMOVAL-CLASSIFIER",
            target_span,
            format!("{operation} requires `{element_classifier}`, found `{found}`"),
        ));
    }
    let mut removed = false;
    let mut retained = Vec::with_capacity(entries.len());
    for entry in entries {
        let equal = values_equal(entry.clone(), target.clone(), trace).ok_or_else(|| {
            diagnostic(
                source,
                "E-LIST-REMOVAL-EQUALITY",
                target_span,
                format!("`{element_classifier}` does not provide equality required by {operation}"),
            )
        })?;
        if equal && (operation == "remove-all" || !removed) {
            removed = true;
        } else {
            retained.push(entry);
        }
    }
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: if operation == "remove-first" {
            "list.first.removed"
        } else {
            "list.all.removed"
        },
        rule: if operation == "remove-first" {
            "TOPAL-LIST-REMOVE-FIRST-001"
        } else {
            "TOPAL-LIST-REMOVE-ALL-001"
        },
        detail: if removed {
            "removed=true"
        } else {
            "removed=false"
        },
    });
    Ok(Value::List {
        element_classifier,
        entries: retained,
    })
}

fn apply_list_containment(
    source: &SourceText,
    operation: &str,
    element_classifier: &str,
    entries: &[Value],
    right: Value,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let contained = if operation == "contains-entry" {
        if !value_has_classifier(&right, element_classifier) {
            let found = structural_value_classifier(&right);
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-CLASSIFIER",
                right_span,
                format!("contains-entry requires `{element_classifier}`, found `{found}`"),
            ));
        }
        entries.iter().try_fold(false, |found, entry| {
            values_equal(entry.clone(), right.clone(), trace).map(|equal| found || equal)
        })
    } else {
        let Value::List {
            element_classifier: right_classifier,
            entries: pattern,
        } = right
        else {
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-OPERAND",
                right_span,
                format!("{operation} requires another List"),
            ));
        };
        if right_classifier != element_classifier {
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-CLASSIFIER",
                right_span,
                format!("{operation} requires `List {element_classifier}`, found `List {right_classifier}`"),
            ));
        }
        if operation == "contains-sequence" {
            contains_consecutive(entries, &pattern, trace)
        } else {
            contains_ordered_subsequence(entries, &pattern, trace)
        }
    }
    .ok_or_else(|| {
        diagnostic(
            source,
            "E-LIST-CONTAINMENT-EQUALITY",
            right_span,
            format!("`{element_classifier}` does not provide equality required by {operation}"),
        )
    })?;
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    let rule = match operation {
        "contains-entry" => "TOPAL-LIST-CONTAINS-ENTRY-001",
        "contains-sequence" => "TOPAL-LIST-CONTAINS-SEQUENCE-001",
        "contains-subsequence" => "TOPAL-LIST-CONTAINS-SUBSEQUENCE-001",
        _ => unreachable!("known List containment operation"),
    };
    trace.record(TraceEvent {
        event: "list.containment.tested",
        rule,
        detail: if contained { "true" } else { "false" },
    });
    Ok(Value::Boolean(contained))
}

fn contains_consecutive(
    entries: &[Value],
    pattern: &[Value],
    trace: &mut impl TraceSink,
) -> Option<bool> {
    if pattern.is_empty() {
        return Some(true);
    }
    entries
        .windows(pattern.len())
        .try_fold(false, |found, window| {
            window
                .iter()
                .zip(pattern)
                .try_fold(true, |equal, (left, right)| {
                    values_equal(left.clone(), right.clone(), trace).map(|item| equal && item)
                })
                .map(|equal| found || equal)
        })
}

fn contains_ordered_subsequence(
    entries: &[Value],
    pattern: &[Value],
    trace: &mut impl TraceSink,
) -> Option<bool> {
    let mut matched = 0;
    for entry in entries {
        if let Some(expected) = pattern.get(matched)
            && values_equal(entry.clone(), expected.clone(), trace)?
        {
            matched += 1;
        }
    }
    Some(matched == pattern.len())
}

#[allow(clippy::too_many_lines)] // Every recursively derived equality remains explicit.
fn values_equal(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<bool> {
    match (left, right) {
        (
            Value::Refined {
                constraint: left_constraint,
                value: left,
                ..
            },
            Value::Refined {
                constraint: right_constraint,
                value: right,
                ..
            },
        ) if left_constraint == right_constraint => values_equal(*left, *right, trace),
        (Value::Refined { value, .. }, right) => values_equal(*value, right, trace),
        (left, Value::Refined { value, .. }) => values_equal(left, *value, trace),
        (Value::Type(left), Value::Type(right)) | (Value::String(left), Value::String(right)) => {
            Some(left == right)
        }
        (Value::Effects(left), Value::Effects(right)) => Some(left == right),
        (Value::Boolean(left), Value::Boolean(right)) => Some(left == right),
        (Value::Int(left), Value::Int(right)) => Some(left == right),
        (Value::Rational(left), Value::Rational(right)) => Some(left == right),
        (Value::Int(left), Value::Rational(right)) => {
            trace_conversion(trace, "Int->Rational:left");
            Some(BigRational::from_integer(left) == right)
        }
        (Value::Rational(left), Value::Int(right)) => {
            trace_conversion(trace, "Int->Rational:right");
            Some(left == BigRational::from_integer(right))
        }
        (
            Value::Modular {
                type_name: left_type,
                value: left,
                ..
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => Some(left == right),
        (
            Value::List {
                element_classifier: left_classifier,
                entries: left,
            },
            Value::List {
                element_classifier: right_classifier,
                entries: right,
            },
        ) if left_classifier == right_classifier && left.len() == right.len() => {
            trace.record(TraceEvent {
                event: "equality.list",
                rule: "TOPAL-TYPE-LIST-EQUALITY-001",
                detail: &left_classifier,
            });
            left.into_iter()
                .zip(right)
                .try_fold(true, |equal, (left, right)| {
                    values_equal(left, right, trace).map(|entry_equal| equal && entry_equal)
                })
        }
        (
            Value::Enum {
                type_name: left_type,
                alternative: left,
            },
            Value::Enum {
                type_name: right_type,
                alternative: right,
            },
        ) if left_type == right_type => Some(left == right),
        (Value::Union(left), Value::Union(right))
            if left.type_name == right.type_name && left.alternative == right.alternative =>
        {
            match (left.payload, right.payload) {
                (None, None) => Some(true),
                (Some(left), Some(right)) => values_equal(*left, *right, trace),
                _ => Some(false),
            }
        }
        (
            Value::Optional {
                payload_classifier: left_classifier,
                payload: left,
            },
            Value::Optional {
                payload_classifier: right_classifier,
                payload: right,
            },
        ) if left_classifier == right_classifier => {
            trace.record(TraceEvent {
                event: "equality.optional",
                rule: "TOPAL-TYPE-OPTIONAL-EQUALITY-001",
                detail: &left_classifier,
            });
            match (left, right) {
                (None, None) => Some(true),
                (Some(left), Some(right)) => values_equal(*left, *right, trace),
                _ => Some(false),
            }
        }
        (Value::Unit, Value::Unit) => Some(true),
        (Value::Tuple(left), Value::Tuple(right)) if left.len() == right.len() => left
            .into_iter()
            .zip(right)
            .try_fold(true, |equal, (left, right)| {
                values_equal(left, right, trace).map(|field_equal| equal && field_equal)
            }),
        (Value::Record(left), Value::Record(right)) if left.len() == right.len() => {
            left.into_iter().try_fold(true, |equal, (label, left)| {
                let right = right
                    .iter()
                    .find(|(right_label, _)| right_label == &label)
                    .map(|(_, value)| value.clone())?;
                values_equal(left, right, trace).map(|field_equal| equal && field_equal)
            })
        }
        _ => None,
    }
}

fn trace_conversion(trace: &mut impl TraceSink, detail: &'static str) {
    trace.record(TraceEvent {
        event: "conversion.applied",
        rule: "TOPAL-TYPE-CONVERT-001",
        detail,
    });
}

fn apply_int_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Compare
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Range
        | CallableKind::RangeOpen
        | CallableKind::RangeInclusive
        | CallableKind::RangeOpenInclusive => {
            unreachable!("range is dispatched before numeric operations")
        }
        CallableKind::Plus => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.+(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.add",
                rule: "TOPAL-NUM-ADD-001",
                detail: "Int",
            });
            Ok(Value::Int(left + right))
        }
        CallableKind::Minus => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.subtract",
                rule: "TOPAL-NUM-SUB-001",
                detail: "Int",
            });
            Ok(Value::Int(left - right))
        }
        CallableKind::Multiply => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.*(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.multiply",
                rule: "TOPAL-NUM-MUL-001",
                detail: "Int",
            });
            Ok(Value::Int(left * right))
        }
        CallableKind::Divide => apply_divide(source, left, right, right_span, trace),
        CallableKind::QuotientModulo => {
            apply_quotient_modulo(source, left, right, right_span, trace)
        }
        CallableKind::Modulo => apply_modulo(source, left, &right, right_span, trace),
        CallableKind::Power => apply_power(source, left, right, right_span, trace),
    }
}

fn apply_modulo(
    source: &SourceText,
    left: BigInt,
    right: &BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == &BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        if parse_integer(source.slice(right_span)).is_none() {
            let position = source.position(right_span.start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: "root.%(Int,Int);division-by-zero",
            });
            return Ok(Value::Error {
                domain: "root.%(Int,Int)".to_owned(),
                code: "division-by-zero".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident modulo by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.%(Int,Int)",
    });
    let remainder = euclidean_remainder(left, right);
    trace.record(TraceEvent {
        event: "evaluation.modulo",
        rule: "TOPAL-NUM-INT-MODULO-001",
        detail: "Euclidean",
    });
    Ok(Value::Int(remainder))
}

fn apply_quotient_modulo(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        if parse_integer(source.slice(right_span)).is_none() {
            let position = source.position(right_span.start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: "root./%(Int,Int);division-by-zero",
            });
            return Ok(Value::Error {
                domain: "root./%(Int,Int)".to_owned(),
                code: "division-by-zero".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident quotient/modulo by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root./%(Int,Int)",
    });
    let remainder = euclidean_remainder(left.clone(), &right);
    let quotient = (left - &remainder) / right;
    trace.record(TraceEvent {
        event: "evaluation.quotient-modulo",
        rule: "TOPAL-NUM-INT-QUOTIENT-MODULO-001",
        detail: "Euclidean",
    });
    Ok(Value::Tuple(vec![
        Value::Int(quotient),
        Value::Int(remainder),
    ]))
}

fn euclidean_remainder(left: BigInt, right: &BigInt) -> BigInt {
    let mut remainder = left % right;
    if remainder < BigInt::from(0) {
        remainder += if right < &BigInt::from(0) {
            -right
        } else {
            right.clone()
        };
    }
    remainder
}

fn apply_rational_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigRational,
    right: BigRational,
    span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if matches!(kind, CallableKind::Modulo | CallableKind::QuotientModulo) {
        return Err(discrete_operand_diagnostic(source, span));
    }
    let (callable, event, rule, result) = match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Compare
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Range
        | CallableKind::RangeOpen
        | CallableKind::RangeInclusive
        | CallableKind::RangeOpenInclusive => {
            unreachable!("range is dispatched before numeric operations")
        }
        CallableKind::Plus => (
            "root.+(Rational,Rational)",
            "evaluation.add",
            "TOPAL-NUM-RAT-ADD-001",
            left + right,
        ),
        CallableKind::Minus => (
            "root.-(Rational,Rational)",
            "evaluation.subtract",
            "TOPAL-NUM-RAT-SUB-001",
            left - right,
        ),
        CallableKind::Multiply => (
            "root.*(Rational,Rational)",
            "evaluation.multiply",
            "TOPAL-NUM-RAT-MUL-001",
            left * right,
        ),
        CallableKind::Modulo | CallableKind::QuotientModulo => {
            unreachable!("discrete operations are rejected before Rational dispatch")
        }
        CallableKind::Divide => {
            if right.numer() == &BigInt::from(0) {
                trace.record(TraceEvent {
                    event: "obligation.refuted",
                    rule: "TOPAL-NUM-DIVZERO-001",
                    detail: "divisor.nonzero",
                });
                if parse_rational(source.slice(right_span)).is_none()
                    && parse_integer(source.slice(right_span)).is_none()
                {
                    let position = source.position(right_span.start);
                    trace.record(TraceEvent {
                        event: "result.error.constructed",
                        rule: "TOPAL-TYPE-RESULT-001",
                        detail: "root./(Rational,Rational);division-by-zero",
                    });
                    return Ok(Value::Error {
                        domain: "root./(Rational,Rational)".to_owned(),
                        code: "division-by-zero".to_owned(),
                        line: position.line,
                        column: position.column,
                    });
                }
                return Err(diagnostic(
                    source,
                    "E-DIVISION-BY-ZERO",
                    right_span,
                    "statically evident division by zero",
                ));
            }
            trace.record(TraceEvent {
                event: "obligation.proved",
                rule: "TOPAL-NUM-DIVZERO-001",
                detail: "divisor.nonzero",
            });
            (
                "root./(Rational,Rational)",
                "evaluation.divide",
                "TOPAL-NUM-RAT-DIV-001",
                left / right,
            )
        }
        CallableKind::Power => {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "Rational exponentiation is not in the implemented subset",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: callable,
    });
    trace.record(TraceEvent {
        event,
        rule,
        detail: "Rational",
    });
    Ok(Value::Rational(result))
}

fn discrete_operand_diagnostic(source: &SourceText, span: Span) -> Diagnostic {
    diagnostic(
        source,
        "E-NO-APPLICABLE-OVERLOAD",
        span,
        "Euclidean modulo requires discrete Int operands",
    )
}

fn apply_divide(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident division by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root./(Int,Int)",
    });
    trace.record(TraceEvent {
        event: "evaluation.divide",
        rule: "TOPAL-NUM-DIV-001",
        detail: "Rational",
    });
    Ok(Value::Rational(BigRational::new(left, right)))
}

fn apply_power(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right < BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-POW-001",
            detail: "exponent.finite-nat",
        });
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            right_span,
            "Int exponentiation requires a finite Nat exponent",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-POW-001",
        detail: "exponent.finite-nat",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.^(Int,Nat)",
    });
    trace.record(TraceEvent {
        event: "evaluation.power",
        rule: "TOPAL-NUM-POW-001",
        detail: "Int",
    });
    Ok(Value::Int(pow_int(left, right)))
}

fn apply_rational_power(
    source: &SourceText,
    left: BigRational,
    right: BigInt,
    left_span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right < BigInt::from(0) {
        if left.numer() == &BigInt::from(0) {
            trace.record(TraceEvent {
                event: "obligation.refuted",
                rule: "TOPAL-NUM-RAT-NEG-POW-001",
                detail: "base.nonzero",
            });
            if parse_rational(source.slice(left_span)).is_none()
                && parse_integer(source.slice(left_span)).is_none()
            {
                let position = source.position(left_span.start);
                trace.record(TraceEvent {
                    event: "result.error.constructed",
                    rule: "TOPAL-TYPE-RESULT-001",
                    detail: "root.^(Rational,Int);division-by-zero",
                });
                return Ok(Value::Error {
                    domain: "root.^(Rational,Int)".to_owned(),
                    code: "division-by-zero".to_owned(),
                    line: position.line,
                    column: position.column,
                });
            }
            return Err(diagnostic(
                source,
                "E-DIVISION-BY-ZERO",
                right_span,
                "a zero Rational base cannot be raised to a negative exponent",
            ));
        }
        trace.record(TraceEvent {
            event: "obligation.proved",
            rule: "TOPAL-NUM-RAT-NEG-POW-001",
            detail: "base.nonzero",
        });
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.^(Rational,Int)",
        });
        trace.record(TraceEvent {
            event: "evaluation.power",
            rule: "TOPAL-NUM-RAT-NEG-POW-001",
            detail: "Rational",
        });
        let power = pow_rational(left, -right);
        return Ok(Value::Rational(BigRational::new(
            power.denom().clone(),
            power.numer().clone(),
        )));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-RAT-POW-001",
        detail: "exponent.finite-nat",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.^(Rational,Nat)",
    });
    trace.record(TraceEvent {
        event: "evaluation.power",
        rule: "TOPAL-NUM-RAT-POW-001",
        detail: "Rational",
    });
    Ok(Value::Rational(pow_rational(left, right)))
}

fn pow_int(mut base: BigInt, mut exponent: BigInt) -> BigInt {
    let zero = BigInt::from(0);
    let one = BigInt::from(1);
    let two = BigInt::from(2);
    let mut result = one.clone();
    while exponent > zero {
        if &exponent % &two == one {
            result *= &base;
        }
        exponent /= &two;
        if exponent > zero {
            base = &base * &base;
        }
    }
    result
}

fn pow_rational(mut base: BigRational, mut exponent: BigInt) -> BigRational {
    let zero = BigInt::from(0);
    let one = BigInt::from(1);
    let two = BigInt::from(2);
    let mut result = BigRational::from_integer(one.clone());
    while exponent > zero {
        if &exponent % &two == one {
            result *= &base;
        }
        exponent /= &two;
        if exponent > zero {
            base = &base * &base;
        }
    }
    result
}

#[allow(clippy::too_many_lines)] // Rejection remains exhaustive across every nonnumeric value kind.
fn apply_negate(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match operand {
        Value::Int(operand) => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.negate",
                rule: "TOPAL-NUM-NEG-001",
                detail: "Int",
            });
            Ok(Value::Int(-operand))
        }
        Value::Rational(operand) => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Rational)",
            });
            trace.record(TraceEvent {
                event: "evaluation.negate",
                rule: "TOPAL-NUM-RAT-NEG-001",
                detail: "Rational",
            });
            Ok(Value::Rational(-operand))
        }
        Value::SizeBits(_) => Err(diagnostic(
            source,
            "E-NEGATE-OPERAND",
            span,
            "a storage size cannot be negated",
        )),
        Value::Modular {
            type_name,
            lower,
            upper,
            value,
        } => {
            let value = reduce_modular(-value, &lower, &upper);
            trace.record(TraceEvent {
                event: "numeric.modular.wrapped",
                rule: "TOPAL-NUM-MODULAR-ARITHMETIC-001",
                detail: &type_name,
            });
            Ok(Value::Modular {
                type_name,
                lower,
                upper,
                value,
            })
        }
        Value::Boolean(_)
        | Value::Version(_)
        | Value::NativeSerializer(_)
        | Value::SerializationStream(_)
        | Value::TaskType(_)
        | Value::TaskDefinition(_)
        | Value::TaskInstance(_)
        | Value::Type(_)
        | Value::Effects(_)
        | Value::IntRange { .. }
        | Value::RationalRange { .. }
        | Value::Optional { .. }
        | Value::List { .. }
        | Value::Callable(_)
        | Value::NamedFunction(_)
        | Value::Namespace(_)
        | Value::AnonymousFunction(_)
        | Value::Array { .. }
        | Value::Set { .. }
        | Value::Bag { .. }
        | Value::Map { .. }
        | Value::CharacterGenerator { .. }
        | Value::CharacterReturningGenerator { .. }
        | Value::IterateGenerator { .. }
        | Value::UnfoldGenerator { .. }
        | Value::SuspendedGenerator { .. }
        | Value::String(_)
        | Value::Tuple(_)
        | Value::Record(_)
        | Value::Enum { .. }
        | Value::Union(_)
        | Value::Constraint(_)
        | Value::Capability(_)
        | Value::Interface(_)
        | Value::Introspection(_)
        | Value::ObjectDescription { .. }
        | Value::Refined { .. }
        | Value::ModularType(_)
        | Value::AddressRangeType(_)
        | Value::AddressRange { .. }
        | Value::AddressOffsetType(_)
        | Value::AddressOffset { .. }
        | Value::LayoutType(_)
        | Value::LayoutFactory(_)
        | Value::LayoutBacked { .. }
        | Value::LocationType(_)
        | Value::Location { .. }
        | Value::ErrorDomain(_)
        | Value::Error { .. }
        | Value::Continue(_)
        | Value::Finish(_)
        | Value::Completed
        | Value::Unit => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "prefix - requires an exact numeric operand",
        )),
    }
}

const fn callable_name(kind: CallableKind) -> &'static str {
    match kind {
        CallableKind::Equal => "=",
        CallableKind::NotEqual => "/=",
        CallableKind::Less => "<",
        CallableKind::Greater => ">",
        CallableKind::LessEqual => "<=",
        CallableKind::Compare => "<=>",
        CallableKind::Range => "..",
        CallableKind::RangeOpen => "<..",
        CallableKind::RangeInclusive => "..=",
        CallableKind::RangeOpenInclusive => "<..=",
        CallableKind::GreaterEqual => ">=",
        CallableKind::Plus => "+",
        CallableKind::Minus => "-",
        CallableKind::Multiply => "*",
        CallableKind::Divide => "/",
        CallableKind::QuotientModulo => "/%",
        CallableKind::Modulo => "%",
        CallableKind::Power => "^",
    }
}

fn diagnostic(
    source: &SourceText,
    code: &'static str,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let position = source.position(span.start);
    let mut diagnostic = Diagnostic::error(code, position.line, position.column, message)
        .with_source_excerpt(
            source
                .as_str()
                .lines()
                .nth(position.line - 1)
                .map(str::to_owned),
            marker_width(source.as_str(), span),
        );
    if let Some(help) = diagnostic_help(code) {
        diagnostic = diagnostic.with_help(help);
    }
    diagnostic
}

fn closest_name<'a>(name: &str, candidates: impl Iterator<Item = &'a String>) -> Option<&'a str> {
    let maximum = 2.max(name.chars().count() / 3);
    candidates
        .map(|candidate| (edit_distance(name, candidate), candidate.as_str()))
        .filter(|(distance, _)| *distance <= maximum)
        .min()
        .map(|(_, candidate)| candidate)
}

const ROOT_OPERATIONS: [&str; 65] = [
    "absolute",
    "byte-count",
    "case-fold",
    "canonically-equals",
    "characters",
    "character-count",
    "concat",
    "collect",
    "empty",
    "list-enumerate",
    "list-permutations",
    "list-combinations",
    "list-subsets",
    "list-cartesian-product",
    "entry-count",
    "first",
    "graph-bfs",
    "graph-dfs",
    "graph-shortest-path",
    "graph-topological-sort",
    "graph-weak-components",
    "graph-weighted-shortest-path",
    "list-group-runs",
    "list-index-of",
    "lower",
    "list-last-index-of",
    "normalize",
    "range-lower",
    "range-lower-inclusive?",
    "range-upper",
    "range-upper-inclusive?",
    "upper",
    "uncons",
    "not",
    "negate",
    "one",
    "ordered-binary-search",
    "ordered-merge",
    "ordered-nth",
    "ordered-smallest",
    "rest",
    "reverse",
    "list-rotate-left",
    "list-rotate-right",
    "list-chunks",
    "list-windows",
    "list-zip-shortest",
    "stable-sort",
    "stable-sort-descending",
    "string-contains",
    "string-contains-any",
    "string-count-exact",
    "string-ends-with",
    "string-find-all",
    "string-glob-matches",
    "string-join",
    "string-lines",
    "string-repeat",
    "string-regex-contains",
    "string-replace-all",
    "string-starts-with",
    "string-split-exact",
    "string-trim",
    "string-words",
    "zero",
];

fn closest_root_operation(name: &str) -> Option<&'static str> {
    if name == "concatenate" {
        return Some("concat");
    }
    let maximum = 2.max(name.chars().count() / 3);
    ROOT_OPERATIONS
        .into_iter()
        .map(|candidate| (edit_distance(name, candidate), candidate))
        .filter(|(distance, _)| *distance <= maximum)
        .min()
        .map(|(_, candidate)| candidate)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_character) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            current.push(
                (current[right_index] + 1)
                    .min(previous[right_index + 1] + 1)
                    .min(previous[right_index] + usize::from(left_character != *right_character)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

fn diagnostic_help(code: &str) -> Option<&'static str> {
    match code {
        "E-UNKNOWN-TOKEN" => Some("remove this character or use a symbol declared by design-0"),
        "E-UNBOUND-NAME" => Some("declare this name earlier in the same source session"),
        "E-EXPECTED-RPAREN" => Some("add a closing `)` for this parenthesized expression"),
        "E-UNTERMINATED-STRING" => Some("add the literal's matching closing quote and tag"),
        "E-DIVISION-BY-ZERO" => Some("use a divisor that is provably nonzero"),
        "E-NO-APPLICABLE-OVERLOAD" => {
            Some("use operands supported by one overload or apply an explicit conversion")
        }
        "E-RESERVED-BOOLEAN-LITERAL" => {
            Some("choose an identifier other than the reserved literals `true` and `false`")
        }
        "E-MIXED-PRODUCT-FIELDS" => {
            Some("nest a tuple in a labeled field, or place a record inside a tuple")
        }
        "E-RESULT-PROJECTION-INFALLIBLE" => {
            Some("change the function result to `Result (T, Codes)`, or match the Error explicitly")
        }
        "E-RESULT-PROJECTION-OUTSIDE-FUNCTION" => Some("match the Result explicitly at top level"),
        "E-INCOMPLETE-ERROR-CODE-DECISION" => {
            Some("add each missing qualified code pattern, or add an `Error problem` fallback")
        }
        "E-DUPLICATE-ERROR-CODE-PATTERN" => {
            Some("remove the later duplicate pattern or replace it with a missing alternative")
        }
        "E-UNREACHABLE-ERROR-CODE-PATTERN" => {
            Some("move qualified code patterns before the generic `Error problem` fallback")
        }
        "E-UNREACHABLE-DECISION-RULE" => Some("move `otherwise` after every specific matcher"),
        "E-CHARACTER-CLASSIFIER" => {
            Some("use a String containing exactly one Unicode grapheme cluster")
        }
        "E-STRING-CONSTRUCTOR-CHARACTER" => {
            Some("classify a one-character String as Character before construction")
        }
        "E-RATIONAL-NOT-EXACT-INT" => {
            Some("use an exactly divisible expression or keep the result classified as Rational")
        }
        "E-NAT-OUT-OF-RANGE" => Some("use a provably nonnegative Int or handle dynamic validation"),
        "E-INDETERMINATE-RATIONAL" => {
            Some("use a nonzero denominator or handle dynamic Rational construction")
        }
        _ => None,
    }
}

fn raw_source_line(source: &str, line: usize) -> Option<String> {
    source.lines().nth(line - 1).map(str::to_owned)
}

fn marker_width(source: &str, span: Span) -> usize {
    source
        .get(span.start..span.end)
        .unwrap_or("")
        .split(['\r', '\n'])
        .next()
        .unwrap_or("")
        .chars()
        .count()
        .max(1)
}

fn raw_position(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit_once('\n')
        .map_or(before, |(_, tail)| tail)
        .chars()
        .count()
        + 1;
    (line, column)
}

fn parse_integer(token: &str) -> Option<BigInt> {
    if let Some(unsigned) = token.strip_prefix('-') {
        return parse_unsigned_integer(unsigned).map(std::ops::Neg::neg);
    }
    parse_unsigned_integer(token)
}

fn parse_rational(token: &str) -> Option<BigRational> {
    if let Some(unsigned) = token.strip_prefix('-') {
        return parse_unsigned_rational(unsigned).map(std::ops::Neg::neg);
    }
    parse_unsigned_rational(token)
}

fn parse_unsigned_rational(token: &str) -> Option<BigRational> {
    let (mantissa, exponent) = if let Some(offset) = token.find(['e', 'E']) {
        (
            &token[..offset],
            parse_signed_decimal_integer(&token[offset + 1..])?,
        )
    } else {
        (token, BigInt::from(0))
    };
    let (integer, fractional) = mantissa
        .split_once('.')
        .map_or((mantissa, ""), |parts| parts);
    if !valid_decimal_integer(integer) || (!fractional.is_empty() && !valid_fractional(fractional))
    {
        return None;
    }
    if fractional.is_empty() && !token.contains(['e', 'E']) {
        return None;
    }
    let integer_digits = integer.replace('_', "");
    let fractional_digits = fractional.replace('_', "");
    let numerator = format!("{integer_digits}{fractional_digits}")
        .parse::<BigInt>()
        .ok()?;
    let scale = BigInt::from(fractional_digits.len()) - exponent;
    if scale >= BigInt::from(0) {
        Some(BigRational::new(
            numerator,
            pow_int(BigInt::from(10), scale),
        ))
    } else {
        Some(BigRational::from_integer(
            numerator * pow_int(BigInt::from(10), -scale),
        ))
    }
}

fn parse_signed_decimal_integer(token: &str) -> Option<BigInt> {
    let (negative, unsigned) = token
        .strip_prefix('-')
        .map_or((false, token), |value| (true, value));
    let unsigned = unsigned.strip_prefix('+').unwrap_or(unsigned);
    if !valid_decimal_integer(unsigned) {
        return None;
    }
    let value = unsigned.replace('_', "").parse::<BigInt>().ok()?;
    Some(if negative { -value } else { value })
}

fn valid_fractional(token: &str) -> bool {
    if !token.contains('_') {
        return token.bytes().all(|byte| byte.is_ascii_digit());
    }
    let groups = token.split('_').collect::<Vec<_>>();
    groups
        .first()
        .is_some_and(|group| group.len() == 3 && group.bytes().all(|byte| byte.is_ascii_digit()))
        && groups.iter().skip(1).enumerate().all(|(index, group)| {
            let final_group = index + 2 == groups.len();
            (group.len() == 3 || (final_group && (1..=2).contains(&group.len())))
                && group.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn parse_unsigned_integer(token: &str) -> Option<BigInt> {
    if valid_decimal_integer(token) {
        return token.replace('_', "").parse().ok();
    }
    let (radix, digits) = if let Some(digits) = token.strip_prefix("0b") {
        (2, digits)
    } else if let Some(digits) = token.strip_prefix("0o") {
        (8, digits)
    } else {
        (16, token.strip_prefix("0x")?)
    };
    valid_based_digits(digits, radix)
        .then(|| BigInt::parse_bytes(digits.replace('_', "").as_bytes(), radix))
        .flatten()
}

fn valid_decimal_integer(token: &str) -> bool {
    if token == "0" {
        return true;
    }
    if token.starts_with('0')
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'_')
    {
        return false;
    }
    if !token.contains('_') {
        return true;
    }
    let mut groups = token.split('_');
    let first = groups.next().unwrap_or_default();
    (1..=3).contains(&first.len())
        && first.bytes().all(|byte| byte.is_ascii_digit())
        && groups.all(|group| group.len() == 3 && group.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_based_digits(digits: &str, radix: u32) -> bool {
    if digits.is_empty() {
        return false;
    }
    let valid_group =
        |group: &str| !group.is_empty() && group.chars().all(|character| character.is_digit(radix));
    if !digits.contains('_') {
        return valid_group(digits);
    }
    let mut groups = digits.split('_');
    let first = groups.next().unwrap_or_default();
    (1..=4).contains(&first.len())
        && valid_group(first)
        && groups.all(|group| group.len() == 4 && valid_group(group))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(source: &str) -> Result<Value, Diagnostic> {
        Session::new().evaluate(source, &mut std::io::sink())
    }

    #[test]
    fn evaluates_qualified_static_introspection() {
        assert!(matches!(
            evaluate("lang context\n").unwrap(),
            Value::Introspection(_)
        ));
        assert_eq!(
            evaluate("lang version\n").unwrap(),
            Value::Version(LanguageVersion::DESIGN_0)
        );

        let identity = evaluate("lang identity Int\n").unwrap();
        assert!(matches!(
            identity,
            Value::Introspection(value)
                if matches!(&*value, IntrospectionValue::Identity { canonical, .. } if canonical == "type:Int")
        ));

        let view = evaluate("lang view Int\n").unwrap();
        assert!(matches!(
            view,
            Value::Introspection(value)
                if matches!(&*value, IntrospectionValue::TypeView { form, identity } if form == "PrimitiveType" && identity == "Int")
        ));

        let effect = evaluate("lang view (Effects ())\n").unwrap();
        assert!(matches!(
            effect,
            Value::Introspection(value)
                if matches!(&*value, IntrospectionValue::EffectView { identities } if identities.is_empty())
        ));
    }

    #[test]
    fn preserves_constructed_language_variant_features() {
        let value = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1, features is ( debug, lint ) )\nlang context\n",
                &mut std::io::sink(),
            )
            .unwrap();
        assert!(matches!(
            value,
            Value::Introspection(context)
                if matches!(&*context, IntrospectionValue::LanguageContext { features, .. }
                    if features == &["debug", "lint"])
        ));
    }

    #[test]
    fn declaration_view_exposes_attached_documentation() {
        let value = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1 )\n### The documented answer.\npub answer is 42\nlang declaration answer\n",
                &mut std::io::sink(),
            )
            .unwrap();
        assert!(matches!(
            value,
            Value::Introspection(view)
                if matches!(&*view, IntrospectionValue::DeclarationView { documentation, .. }
                    if documentation.as_deref() == Some("The documented answer."))
        ));
    }

    #[test]
    fn exposes_lint_namespace_only_in_the_lint_variant() {
        let value = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1, features is ( lint ) )\nlang lint\n",
                &mut std::io::sink(),
            )
            .unwrap();
        assert!(matches!(value, Value::Namespace(namespace) if namespace.name == "lang lint"));

        let error = Session::new()
            .evaluate_source_file(
                "use language ( version is v0.1 )\nlang lint\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
        assert_eq!(error.code, "E-LINT-VARIANT");
    }

    #[test]
    fn evaluates_static_object_relations_without_runtime_reflection() {
        assert_eq!(
            evaluate("Int lang same-object Int\n").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            evaluate("Int lang equivalent-type Rational\n").unwrap(),
            Value::Boolean(false)
        );
        let error = evaluate("lang view 42\n").unwrap_err();
        assert_eq!(error.code, "E-STATIC-INTROSPECTION-SUBJECT");
        let error = evaluate("1 lang same-object 1\n").unwrap_err();
        assert_eq!(error.code, "E-STATIC-INTROSPECTION-SUBJECT");
    }

    #[test]
    fn serializes_with_an_explicit_version_and_deserializes_validated_streams() {
        let direct = evaluate("v0.1 (lang serialize) (answer is 42, ok is true)\n").unwrap();
        assert!(matches!(direct, Value::SerializationStream(_)));

        let round_trip = evaluate(
            "serialize is lang version (lang serialize)\nstream is serialize (answer is 42, ok is true)\nlang deserialize stream\n",
        )
        .unwrap();
        assert_eq!(round_trip.to_string(), "(answer is 42, ok is true)");

        let huge = "1606938044258990275541962092341162602522202993782792835301376";
        let round_trip = evaluate(&format!(
            "stream is v0.1 (lang serialize) {huge}\nlang deserialize stream\n"
        ))
        .unwrap();
        assert_eq!(round_trip.to_string(), huge);
    }

    #[test]
    fn native_serialization_round_trips_numeric_and_collection_schemas() {
        let values = [
            Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(3))),
            Value::List {
                element_classifier: "Int".into(),
                entries: vec![Value::Int(BigInt::from(1)), Value::Int(BigInt::from(2))],
            },
            Value::Set {
                element_classifier: "Int".into(),
                entries: vec![Value::Int(BigInt::from(1)), Value::Int(BigInt::from(2))],
            },
            Value::Map {
                key_classifier: "Int".into(),
                value_classifier: "String".into(),
                entries: vec![
                    (Value::Int(BigInt::from(1)), Value::String("one".into())),
                    (Value::Int(BigInt::from(2)), Value::String("two".into())),
                ],
            },
        ];
        for value in values {
            let stream = stream_for_value(LanguageVersion::DESIGN_0, &value).unwrap();
            let bytes = serialize_native(&stream).unwrap();
            let decoded = deserialize_native(&bytes, SerializationLimits::default()).unwrap();
            assert_eq!(
                value_from_serialized(&decoded.events[0], &decoded.types),
                Some(value)
            );
        }

        let described = Value::Type("Int".into());
        let stream = stream_for_value(LanguageVersion::DESIGN_0, &described).unwrap();
        let bytes = serialize_native(&stream).unwrap();
        let decoded = deserialize_native(&bytes, SerializationLimits::default()).unwrap();
        assert!(matches!(
            value_from_serialized(&decoded.events[0], &decoded.types),
            Some(Value::ObjectDescription { kind, .. }) if kind == "Type"
        ));
    }

    #[test]
    fn retains_explicit_function_effect_upper_bounds_for_static_views() {
        let value = evaluate(
            "identity is fn ( value : Int ) -> Int\n  : Effects ()\n  value\nlang view identity\n",
        )
        .unwrap();
        assert!(matches!(
            value,
            Value::Introspection(view)
                if matches!(&*view, IntrospectionValue::FunctionView { effects, .. } if effects == &["Effects ()"])
        ));
    }

    #[test]
    fn binds_packaged_function_operands_and_fills_field_defaults() {
        let value = evaluate(
            "sum is fn ( ( value : Int, fallback : Int default 2 ) ) -> Int\n  value + fallback\nsum (value is 40)\n",
        )
        .unwrap();
        assert_eq!(value, Value::Int(BigInt::from(42)));

        let value = evaluate(
            "sum is fn ( ( value : Int, fallback : Int default 2 ) ) -> Int\n  value + fallback\nsum (value is 40, fallback is 3)\n",
        )
        .unwrap();
        assert_eq!(value, Value::Int(BigInt::from(43)));
    }

    #[test]
    fn constructs_tasks_and_routes_stateful_message_transactions() {
        let source = "Counter is Task (queue-size is 10, identity is counter)\
\ncounter-service is Counter\
\n  count : Nat\
\n  start is fn ( initial : Nat ) -> Completed\
\n    @ count is initial\
\n    Completed\
\n  increment is fn ( _ : MessageContext, amount : Nat ) -> Unit\
\n    @ count is @ count + amount\
\n  current is fn ( _ : MessageContext, _ : Unit ) -> Result ( Nat, () )\
\n    @ count\
\ncounter is counter-service 40\
\ncounter increment 2\
\ncounter current ()\n";
        let mut trace = Vec::new();
        let value = Session::new().evaluate(source, &mut trace).unwrap();
        assert_eq!(value, Value::Int(BigInt::from(42)));
        assert!(trace.iter().any(|event| event.contains("message.sent")));
        assert!(trace.iter().any(|event| event.contains("message.received")));
        assert!(
            trace
                .iter()
                .any(|event| event.contains("task.state.replaced"))
        );
    }

    #[test]
    fn task_termination_discards_events_and_fails_requests_in_task_domain() {
        let source = "Counter is Task (identity is counter)\
\nservice is Counter\
\n  count : Nat\
\n  start is fn (initial : Nat) -> Completed\
\n    @ count is initial\
\n    Completed\
\n  ping is fn (_ : MessageContext, _ : Unit) -> Unit\
\n    ()\
\n  current is fn (_ : MessageContext, _ : Unit) -> Result (Nat, ())\
\n    @ count\
\n  terminate is fn (_ : String) -> Unit\
\n    ()\
\ninstance is service 1\
\ninstance terminate \"done\"\
\ninstance ping ()\
\ninstance current ()\n";
        assert!(matches!(
            evaluate(source).unwrap(),
            Value::Error { domain, code, .. }
                if domain == "lang task" && code == "task-terminated"
        ));
    }

    #[test]
    fn task_streams_follow_transactions_and_reacquire_current_state() {
        let source = "Counter is Task (identity is counter)\
\nservice is Counter\
\n  count : Nat\
\n  start is fn (initial : Nat) -> Completed\
\n    @ count is initial\
\n    Completed\
\n  values is generator (_ : MessageContext, _ : Unit)\
\n    yields Nat\
\n    resumes Unit\
\n    -> Result (Unit, ())\
\n    yield @ count\
\n    @ count is @ count + 1\
\n    yield @ count\
\n    ()\
\n  current is fn (_ : MessageContext, _ : Unit) -> Result (Nat, ())\
\n    @ count\
\ninstance is service 1\
\nstream is instance values ()\
\nstream foreach { value }\
\n  ()\
\ninstance current ()\n";
        let mut trace = Vec::new();
        let value = Session::new().evaluate(source, &mut trace).unwrap();
        assert_eq!(value, Value::Int(BigInt::from(2)));
        assert!(
            trace
                .iter()
                .any(|event| event.contains("message.stream.started"))
        );
    }

    #[test]
    fn constructs_external_layouts_ranges_offsets_and_locations() {
        let source = "UInt32LE is (storage-size is 32[b], encoding is UnsignedBinary, endian is Little) Layout Nat\
\nDeviceAddresses is AddressRange (caching is Uncached, minimum-access-size is 32[b], medium is MMIO)\
\ndevice is DeviceAddresses (0x40000000 .. 0x4000ffff)\
\nDeviceOffset is AddressOffset (range is device, alignment is 4)\
\ncontrol-offset is DeviceOffset 32\
\nControlLocation is Location UInt32LE\
\ncontrol is ControlLocation control-offset\
\nstored is UInt32LE 42\
\ncontrol write stored\
\nread control\n";
        let value = evaluate(source).unwrap();
        assert!(
            matches!(value, Value::LayoutBacked { value, .. } if *value == Value::Int(BigInt::from(42)))
        );
    }

    #[test]
    fn advances_a_prepared_execution_one_statement_at_a_time() {
        let mut session = Session::new();
        let mut trace = Vec::new();
        let mut execution = session
            .prepare("answer is 40\nanswer + 2\n", &mut trace)
            .unwrap();

        let first = execution.step(&mut session, &mut trace).unwrap();
        assert!(matches!(
            first,
            ExecutionStep::Advanced {
                value: Value::Unit,
                ..
            }
        ));
        assert!(
            !trace
                .iter()
                .any(|event| event.contains("evaluation.result"))
        );

        let second = execution.step(&mut session, &mut trace).unwrap();
        assert!(matches!(second, ExecutionStep::Complete(Value::Int(_))));
        assert!(trace.last().unwrap().contains("evaluation.result"));
    }

    #[test]
    fn evaluates_discard_without_introducing_a_binding() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("_ is 20 + 22\n7\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "7");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("binding.discarded"))
        );
        assert!(
            Session::new()
                .evaluate("_\n", &mut std::io::sink())
                .is_err()
        );
    }

    #[test]
    fn evaluates_labeled_record_products_in_field_order() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("(name is \"Ada\", active is true)\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "(name is \"Ada\", active is true)");
        assert!(trace.iter().any(|event| event.contains("product.record")));

        let duplicate = Session::new()
            .evaluate("(name is 1, name is 2)\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(duplicate.code, "E-DUPLICATE-RECORD-FIELD");

        let mixed = Session::new()
            .evaluate("(1, name is 2)\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(mixed.code, "E-MIXED-PRODUCT-FIELDS");
    }

    #[test]
    fn selects_record_fields_without_resolving_the_label_as_a_name() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate(
                "person is (name is \"Ada\", active is true)\nperson name\n",
                &mut trace,
            )
            .unwrap();
        assert_eq!(value.to_string(), "\"Ada\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("record.field.selected"))
        );

        let error = Session::new()
            .evaluate("(name is \"Ada\") age\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-SUCH-RECORD-FIELD");
    }

    #[test]
    fn derives_equality_for_records_with_the_same_shape() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate(
                "(name is \"Ada\", score is 1) = (score is 1.0, name is \"Ada\")\n",
                &mut trace,
            )
            .unwrap();
        assert_eq!(value.to_string(), "true");
        assert!(trace.iter().any(|event| event.contains("Int->Rational")));

        let error = Session::new()
            .evaluate(
                "(name is \"Ada\") = (name is \"Ada\", active is true)\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn concatenates_plain_strings_without_normalizing_the_join() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"e\" concat \"\u{301}\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"e\u{301}\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-CONCAT-001"))
        );

        let error = Session::new()
            .evaluate("\"value\" concat 1\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn composes_only_adjacent_string_literals_implicitly() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"Hello, \" \"Topal\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"Hello, Topal\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-LITERAL-COMPOSE-001"))
        );

        let value = Session::new()
            .evaluate(
                "left is \"Hello, \"\nright is \"Topal\"\nleft concat right\n",
                &mut std::io::sink(),
            )
            .unwrap();
        assert_eq!(value.to_string(), "\"Hello, Topal\"");

        let error = Session::new()
            .evaluate(
                "left is \"Hello, \"\nright is \"Topal\"\nleft right\n",
                &mut std::io::sink(),
            )
            .unwrap_err();
        assert_eq!(error.code, "E-UNSUPPORTED-APPLICATION");
    }

    #[test]
    fn constructs_the_unique_empty_plain_string() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("empty String\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"\"");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-EMPTY-001"))
        );
    }

    #[test]
    fn tests_plain_string_emptiness() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("empty? (empty String)\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "true");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-EMPTY-PREDICATE-001"))
        );
        assert_eq!(
            Session::new()
                .evaluate("empty? \"Topal\"\n", &mut std::io::sink())
                .unwrap()
                .to_string(),
            "false"
        );
    }

    #[test]
    fn counts_unicode_user_perceived_characters() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("character-count \"a\u{301}👩‍🔬🇸🇪\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "3");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-CHARACTER-COUNT-001") && event.contains("characters=3")
        }));

        let error = Session::new()
            .evaluate("character-count 1\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn string_entry_count_agrees_with_character_count() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("entry-count \"a\u{301}👩‍🔬🇸🇪\"\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "3");
        assert!(
            trace
                .iter()
                .any(|event| event.contains("TOPAL-STRING-ENTRY-COUNT-001"))
        );
    }

    #[test]
    fn counts_prospective_utf8_bytes_without_normalizing() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"e\u{301}👩‍🔬\" byte-count Utf8\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "14");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-UTF8-BYTE-COUNT-001") && event.contains("bytes=14")
        }));

        let error = Session::new()
            .evaluate("\"text\" byte-count Utf16\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn normalizes_plain_strings_to_nfc_explicitly() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"e\u{301}\" normalize NFC\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"é\"");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-NORMALIZE-NFC-001") && event.contains("changed=true")
        }));
    }

    #[test]
    fn normalizes_plain_strings_to_nfd_explicitly() {
        let mut trace = Vec::new();
        let value = Session::new()
            .evaluate("\"é\" normalize NFD\n", &mut trace)
            .unwrap();
        assert_eq!(value.to_string(), "\"e\u{301}\"");
        assert!(trace.iter().any(|event| {
            event.contains("TOPAL-STRING-NORMALIZE-NFD-001") && event.contains("changed=true")
        }));

        let error = Session::new()
            .evaluate("\"text\" normalize NFKD\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn adds_signed_arbitrary_precision_integers() {
        assert_eq!(
            evaluate("-1 + 123456789012345678901234567890")
                .unwrap()
                .to_string(),
            "123456789012345678901234567889"
        );
    }

    #[test]
    fn follows_left_association() {
        assert_eq!(evaluate("1 + 2 + 3").unwrap().to_string(), "6");
    }

    #[test]
    fn negates_and_subtracts_exact_integers() {
        assert_eq!(evaluate("- 42").unwrap().to_string(), "-42");
        assert_eq!(evaluate("10 - 3 - 2").unwrap().to_string(), "5");
        assert_eq!(evaluate("10 - -2").unwrap().to_string(), "12");
    }

    #[test]
    fn multiplies_without_hidden_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap().to_string(), "20");
        assert_eq!(evaluate("2 + (3 * 4)").unwrap().to_string(), "14");
        assert_eq!(
            evaluate("99999999999999999999 * 99999999999999999999")
                .unwrap()
                .to_string(),
            "9999999999999999999800000000000000000001"
        );
    }

    #[test]
    fn divides_to_canonical_rational() {
        assert_eq!(evaluate("6 / 8").unwrap().to_string(), "Rational ( 3, 4 )");
        assert_eq!(
            evaluate("6 / -8").unwrap().to_string(),
            "Rational ( -3, 4 )"
        );
        assert_eq!(evaluate("6 / 3").unwrap().to_string(), "Rational ( 2, 1 )");
    }

    #[test]
    fn rejects_statically_evident_zero_divisor() {
        assert_eq!(evaluate("1 / 0").unwrap_err().code, "E-DIVISION-BY-ZERO");
    }

    #[test]
    fn renders_unicode_aligned_actionable_diagnostics() {
        let error = evaluate("α is 1\nα + missing").unwrap_err();
        assert_eq!(
            error.render("example.t"),
            "error[E-UNBOUND-NAME]: name is not bound\n --> example.t:2:5\n  |\n2 | α + missing\n  |     ^^^^^^^\n  |\n  = help: declare this name earlier in the same source session"
        );
    }

    #[test]
    fn edit_distance_counts_unicode_scalars() {
        assert_eq!(edit_distance("räknare", "räknaren"), 1);
        assert_eq!(edit_distance("αβ", "βα"), 2);
    }

    #[test]
    fn diagnostics_suggest_root_operations_and_the_concat_migration() {
        let error = Session::new()
            .evaluate("charcter-count \"Topal\"\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-UNBOUND-NAME");
        assert_eq!(
            error.help.as_deref(),
            Some("did you mean `character-count`?")
        );

        let error = Session::new()
            .evaluate("\"a\" concatenate \"b\"\n", &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-UNSUPPORTED-APPLICATION");
        assert_eq!(error.help.as_deref(), Some("did you mean `concat`?"));
    }

    #[test]
    fn raises_integer_to_natural_power_exactly() {
        assert_eq!(
            evaluate("2 ^ 100").unwrap().to_string(),
            "1267650600228229401496703205376"
        );
        assert_eq!(evaluate("0 ^ 0").unwrap().to_string(), "1");
        assert_eq!(evaluate("-2 ^ 3").unwrap().to_string(), "-8");
    }

    #[test]
    fn exponentiation_uses_ordinary_left_association() {
        assert_eq!(evaluate("2 + 3 ^ 2").unwrap().to_string(), "25");
        assert_eq!(evaluate("2 + (3 ^ 2)").unwrap().to_string(), "11");
    }

    #[test]
    fn raises_rationals_to_natural_powers_exactly() {
        assert_eq!(
            evaluate("1.5 ^ 3").unwrap().to_string(),
            "Rational ( 27, 8 )"
        );
        assert_eq!(
            evaluate("0.0 ^ 0").unwrap().to_string(),
            "Rational ( 1, 1 )"
        );
        assert_eq!(
            evaluate("1.5 ^ -2").unwrap().to_string(),
            "Rational ( 4, 9 )"
        );
        assert_eq!(
            evaluate("1.5 ^ 2.0").unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
        assert_eq!(evaluate("0.0 ^ -1").unwrap_err().code, "E-DIVISION-BY-ZERO");
    }

    #[test]
    fn rejects_negative_integer_exponent() {
        assert_eq!(
            evaluate("2 ^ -1").unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn constructs_exact_rational_literals() {
        assert_eq!(evaluate("0.1").unwrap().to_string(), "Rational ( 1, 10 )");
        assert_eq!(
            evaluate("1.25e3").unwrap().to_string(),
            "Rational ( 1250, 1 )"
        );
        assert_eq!(
            evaluate("-6.022e-24").unwrap().to_string(),
            "Rational ( -3011, 500000000000000000000000000 )"
        );
        assert_eq!(
            evaluate("1_000.000_125").unwrap().to_string(),
            "Rational ( 8000001, 8000 )"
        );
    }

    #[test]
    fn rejects_malformed_rational_literal() {
        assert_eq!(evaluate("1.2e").unwrap_err().code, "E-NUMERIC-LITERAL");
    }

    #[test]
    fn evaluates_exact_rational_arithmetic() {
        assert_eq!(
            evaluate("0.5 + 0.25").unwrap().to_string(),
            "Rational ( 3, 4 )"
        );
        assert_eq!(
            evaluate("- 1.5 - 0.25").unwrap().to_string(),
            "Rational ( -7, 4 )"
        );
        assert_eq!(
            evaluate("1.5 * 0.5").unwrap().to_string(),
            "Rational ( 3, 4 )"
        );
        assert_eq!(
            evaluate("1.5 / 0.25").unwrap().to_string(),
            "Rational ( 6, 1 )"
        );
    }

    #[test]
    fn converts_int_for_mixed_exact_arithmetic() {
        assert_eq!(
            evaluate("1 + 0.5").unwrap().to_string(),
            "Rational ( 3, 2 )"
        );
        assert_eq!(
            evaluate("0.5 * 2").unwrap().to_string(),
            "Rational ( 1, 1 )"
        );
        assert_eq!(
            evaluate("1 / 0.5").unwrap().to_string(),
            "Rational ( 2, 1 )"
        );
    }

    #[test]
    fn rejects_rational_zero_divisor() {
        assert_eq!(
            evaluate("1.0 / 0.0").unwrap_err().code,
            "E-DIVISION-BY-ZERO"
        );
    }

    #[test]
    fn preserves_ordinary_and_tagged_string_contents() {
        assert_eq!(
            evaluate(r#""plain\n{value}""#).unwrap().to_string(),
            r#""plain\n{value}""#
        );
        assert_eq!(
            evaluate(r#"text"He said "hello"."text"#)
                .unwrap()
                .to_string(),
            r#"text"He said "hello"."text"#
        );
        assert_eq!(
            evaluate("\"first\nsecond\"").unwrap().to_string(),
            "\"first\nsecond\""
        );
    }

    #[test]
    fn display_extends_colliding_string_tag() {
        assert_eq!(
            evaluate(r#"tag"contains "text closing"tag"#)
                .unwrap()
                .to_string(),
            r#"text_"contains "text closing"text_"#
        );
    }

    #[test]
    fn parentheses_group_addition() {
        assert_eq!(evaluate("1 + (2 + 3)").unwrap().to_string(), "6");
    }

    #[test]
    fn evaluates_binding_and_lookup() {
        assert_eq!(
            evaluate("answer is 40 + 2\nanswer").unwrap().to_string(),
            "42"
        );
    }

    #[test]
    fn rejects_incomplete_grouping() {
        assert_eq!(evaluate("12_34").unwrap_err().code, "E-NUMERIC-LITERAL");
    }
}
#[test]
fn declares_and_calls_static_nullary_functions() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  40 + 2\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let declared = trace
        .iter()
        .position(|event| event.contains("function.declared"))
        .unwrap();
    let entered = trace
        .iter()
        .position(|event| event.contains("function.entry"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("function.exit"))
        .unwrap();
    assert!(declared < entered && entered < returned);
}

#[test]
fn static_function_body_uses_declaration_order_lexical_bindings() {
    let value = Session::new()
        .evaluate(
            "base is 40\nanswer is fn static () -> Int\n  base + 2\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  later + 2\nlater is 40\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn static_unary_function_binds_a_typed_local_parameter() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "increment is fn static (input : Int) -> Int\n  input + 1\nincrement 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("function.argument.bound") && event.contains("input") })
    );

    let error = Session::new()
        .evaluate(
            "increment is fn static (input : Int) -> Int\n  input + 1\ninput\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn static_product_function_binds_typed_parameters_in_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "subtract is fn static (left : Int, right : Int) -> Int\n  left - right\n50 subtract 8\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let bindings = trace
        .iter()
        .filter(|event| event.contains("function.argument.bound"))
        .collect::<Vec<_>>();
    assert!(bindings[0].contains("left"));
    assert!(bindings[1].contains("right"));

    let error = Session::new()
        .evaluate(
            "bad is fn static (value : Int, value : Int) -> Int\n  value\nbad (1, 2)\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DUPLICATE-FUNCTION-PARAMETER");
}

#[test]
fn function_block_bindings_are_local_to_each_invocation() {
    let mut trace = Vec::new();
    let mut session = Session::new();
    let value = session
        .evaluate(
            "answer is fn static () -> Int\n  local is 40 + 2\n  local\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let created = trace
        .iter()
        .position(|event| event.contains("binding.bind") && event.contains("local"))
        .unwrap();
    let resolved = trace
        .iter()
        .position(|event| event.contains("binding.resolved") && event.contains("local"))
        .unwrap();
    assert!(created < resolved);

    let error = session
        .evaluate("local\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");

    let error = session
        .evaluate(
            "invalid is fn static () -> Int\n  1\n  2\ninvalid ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DISCARDED-VALUE");
}

#[test]
fn explicit_return_skips_later_function_statements() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  return 40 + 2\n  missing\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.return.explicit"))
    );
    assert!(!trace.iter().any(|event| event.contains("missing")));

    let error = Session::new()
        .evaluate("return 42\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn ordinary_runtime_function_uses_ordinary_trace_rule() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "subtract is fn (left : Int, right : Int) -> Int\n  left - right\n50 subtract 8\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function."))
            .all(|event| event.contains("TOPAL-FUNCTION-ORDINARY-001")
                || event.contains("TOPAL-TYPE-CALL-001"))
    );
}

#[test]
fn nat_classifiers_accept_only_nonnegative_int_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "identity is fn (value : Nat) -> Nat\n  value\nidentity 42\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(trace.iter().any(|event| event.contains("identity (Nat)")));

    let argument_error = Session::new()
        .evaluate(
            "identity is fn (value : Nat) -> Nat\n  value\nidentity -1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(argument_error.code, "E-FUNCTION-ARGUMENT-TYPE");

    let result_error = Session::new()
        .evaluate(
            "negative is fn () -> Nat\n  -1\nnegative ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(result_error.code, "E-FUNCTION-RESULT-TYPE");
}

#[test]
fn proves_unit_step_nat_recursion_without_overshoot() {
    let source = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 0 then 0\n    otherwise count-down (value - 1)\ncount-down 3\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "0"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-001"))
    );

    let error = Session::new()
        .evaluate(
            &source.replace("value - 1", "value - 2"),
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn proves_nat_decrement_when_the_bound_prevents_overshoot() {
    let safe = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 2 then value\n    otherwise count-down (value - 3)\ncount-down 8\n";
    assert_eq!(
        Session::new()
            .evaluate(safe, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "2"
    );
    let error = Session::new()
        .evaluate(
            &safe.replace("value - 3", "value - 4"),
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn proves_increasing_nat_recursion_with_positive_steps() {
    let source = "advance is fn (value : Nat) -> Nat\n  value\n    >= 5 then value\n    otherwise advance (value + 2)\nadvance 0\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "6"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"))
    );
}

#[test]
fn proves_closed_mutual_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"))
    );
}

#[test]
fn proves_closed_mutual_increasing_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    >= 6 then true\n    otherwise odd (value + 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    >= 6 then false\n    otherwise even (value + 1)\n(even 0, odd 0)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"))
    );
}

#[test]
fn declares_nominal_payload_free_enum_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "Color is Enum (Red, Green, Blue)\n(Red, Green, Red = Red, Red = Green)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Red, Green, true, false)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-TYPE-ENUM-001"))
    );
}

#[test]
fn validates_enum_function_parameters_and_results() {
    let source = "Color is Enum (Red, Green)\nidentity is fn (value : Color) -> Color\n  value\n(identity Red, identity Green)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(Red, Green)"
    );
    assert!(trace.iter().any(|event| event.contains("identity (Color)")));
}

#[test]
fn executes_only_complete_enum_decisions() {
    let source = "Color is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"red\"\n    Green then \"green\"\nname Green\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "\"green\""
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-ENUM-001"))
    );
}

#[test]
fn resolves_namespaced_arithmetic_error_codes_without_a_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(lang arithmetic division-by-zero) = (lang arithmetic division-by-zero)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(true));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-NUM-ARITHMETIC-ERROR-001"))
    );
}

#[test]
fn matches_both_result_paths_exhaustively() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error problem then \"error\"\n(describe 2.0, describe 0.0)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(\"ok\", \"error\")"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-RESULT-001"))
    );
}

#[test]
fn nested_function_calls_preserve_staticness_and_detect_cycles() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  increment 41\nincrement is fn (input : Int) -> Int\n  input + 1\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let outer_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("answer"))
        .unwrap();
    let inner_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("increment"))
        .unwrap();
    let inner_return = trace
        .iter()
        .position(|event| event.contains("function.exit") && event.contains("increment"))
        .unwrap();
    let outer_return = trace
        .iter()
        .position(|event| event.contains("function.exit") && event.contains("answer"))
        .unwrap();
    assert!(outer_entry < inner_entry && inner_entry < inner_return && inner_return < outer_return);

    let recursion = Session::new()
        .evaluate(
            "again is fn () -> Int\n  again ()\nagain ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(recursion.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn function_local_binding_shadows_capture_without_leaking() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "value is 40\nanswer is fn () -> Int\n  value is 42\n  value\n(answer (), value)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 40)");

    let duplicate = Session::new()
        .evaluate(
            "bad is fn (value : Int) -> Int\n  value is 42\n  value\nbad 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-BINDING");
}

#[test]
fn overload_selection_uses_input_classifier_and_rejects_duplicate_signature() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  value\n(describe 42, describe \"Topal\")\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"integer\", \"Topal\")");
    assert!(trace.iter().any(|event| event.contains("describe (Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("describe (String)"))
    );

    let duplicate = Session::new()
        .evaluate(
            "same is fn (first : Int) -> Int\n  first\nsame is fn (second : Int) -> String\n  \"duplicate\"\nsame 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-FUNCTION-OVERLOAD");
}

#[test]
fn boolean_decision_evaluates_only_selected_action() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise missing\nchoose true\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
    assert!(!trace.iter().any(|event| event.contains("missing")));
}

#[test]
fn exhaustive_boolean_decision_selects_both_literal_cases() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    false then 0\n(choose true, choose false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 0)");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=1") })
    );
}

#[test]
fn earlier_function_body_calls_later_declaration() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  second value\nsecond is fn (value : Int) -> Int\n  value + 1\nfirst 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let first = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("first"))
        .unwrap();
    let second = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("second"))
        .unwrap();
    assert!(first < second);
}

#[test]
fn mutual_int_recursion_executes_only_when_every_cycle_edge_decreases() {
    let source = "even is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.recursion.cycle.proven"))
    );

    let three_member = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise third (value - 1)\nthird is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 1)\nfirst 3\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(three_member.to_string(), "true");

    let invalid = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first value\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(invalid.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn mutual_increasing_int_recursion_requires_one_direction_for_the_complete_cycle() {
    let source = "even-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then true\n    otherwise odd-up (value + 1)\nodd-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then false\n    otherwise even-up (value + 1)\n(even-up (-6), odd-up (-6))\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    assert!(trace.iter().any(|event| {
        event.contains("function.recursion.cycle.proven")
            && event.contains("TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001")
    }));

    let mixed = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    >= 10 then false\n    otherwise first (value + 1)\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(mixed.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn same_named_distinct_overloads_are_not_recursive() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  (describe 42) concat \":\" concat value\ndescribe \"Topal\"\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "\"integer:Topal\"");
    let string = trace
        .iter()
        .position(|event| event.contains("describe (String)"))
        .unwrap();
    let integer = trace
        .iter()
        .position(|event| event.contains("describe (Int)"))
        .unwrap();
    assert!(string < integer);
}

#[test]
fn bounded_int_recursion_accepts_only_positive_literal_progress() {
    let value = Session::new()
        .evaluate(
            "down is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise 1 + (down (value - 3))\nup is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (up (value + 2))\n(down 7, up (-5))\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(3, 3)");

    let mutual = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 2)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 3)\nfirst 7\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(mutual.to_string(), "false");

    for invalid_step in ["0", "-1"] {
        let source = format!(
            "stuck is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise stuck (value - {invalid_step})\nstuck 1\n"
        );
        let error = Session::new()
            .evaluate(&source, &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-UNPROVEN-RECURSION");
    }
}

#[test]
fn every_recursive_call_in_one_action_must_progress() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "branch-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (branch-count (value - 1)) + (branch-count (value - 2))\nbranch-count 3\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "5");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count()
            > 2
    );

    let error = Session::new()
        .evaluate(
            "unsafe-branch is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (unsafe-branch (value - 1)) + (unsafe-branch value)\nunsafe-branch 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn every_call_on_one_mutual_edge_must_share_target_and_progress() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "first-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second-count (value - 1)) + (second-count (value - 2))\nsecond-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first-count (value - 1)\nfirst-count 3\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "3");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count()
            > 1
    );

    let error = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second (value - 1)) + (second value)\nsecond is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first (value - 1)\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");

    let different_target = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second (value - 1)) + (third (value - 1))\nsecond is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first (value - 1)\nthird is fn (value : Int) -> Int\n  value\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(different_target.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn comparison_decision_uses_subject_as_left_operand() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "minimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise missing\n42 minimum 50\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(trace.iter().any(|event| {
        event.contains("decision.rule.selected") && event.contains("TOPAL-DECISION-COMPARISON-001")
    }));
    assert!(!trace.iter().any(|event| event.contains("missing")));
}

#[test]
fn decreasing_int_recursion_executes_only_after_structural_proof() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "sum-down is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise value + (sum-down (value - 1))\nsum-down 5\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "15");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.recursion.proven"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count(),
        5
    );

    let unproven = Session::new()
        .evaluate(
            "wrong is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise wrong (value + 1)\nwrong 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(unproven.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn increasing_int_recursion_executes_only_after_structural_proof() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "distance-up is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (distance-up (value + 1))\ndistance-up (-5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "5");
    assert!(trace.iter().any(|event| {
        event.contains("function.recursion.proven")
            && event.contains("TOPAL-FUNCTION-RECURSION-INT-INCREASING-001")
    }));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count(),
        5
    );
}

#[test]
fn comparison_matcher_evaluates_complete_operand_expression() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "within is fn (value : Int, limit : Int) -> Boolean\n  value\n    < limit + 1 then true\n    otherwise false\n(5 within 5, 6 within 5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    let addition = trace
        .iter()
        .position(|event| event.contains("root.+(Int,Int)"))
        .unwrap();
    let comparison = trace
        .iter()
        .position(|event| event.contains("root.<(TotalOrder,TotalOrder)"))
        .unwrap();
    assert!(addition < comparison);
}

#[test]
fn nested_function_captures_outer_parameter_without_leaking() {
    let mut session = Session::new();
    let mut trace = Vec::new();
    let value = session
        .evaluate(
            "answer is fn (input : Int) -> Int\n  add-input is fn (value : Int) -> Int\n    value + input\n  add-input 2\nanswer 40\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let outer_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("answer"))
        .unwrap();
    let nested_declaration = trace
        .iter()
        .position(|event| event.contains("function.declared") && event.contains("add-input"))
        .unwrap();
    let nested_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("add-input"))
        .unwrap();
    assert!(outer_entry < nested_declaration && nested_declaration < nested_entry);

    let error = session
        .evaluate("add-input 2\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn structured_error_fields_retain_code_type_and_domain_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproblem is 1.0 divide 0.0\n(problem code, problem domain)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value,
        Value::Tuple(vec![
            Value::Enum {
                type_name: "lang arithmetic ArithmeticErrorCode".into(),
                alternative: "division-by-zero".into(),
            },
            Value::ErrorDomain("root./(Rational,Rational)".into()),
        ])
    );
    assert_eq!(
        value.to_string(),
        "(division-by-zero, root./(Rational,Rational))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("error.field.selected"))
            .count(),
        2
    );
}

#[test]
fn qualified_error_code_pattern_selects_without_using_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n    Error problem then \"other\"\ndescribe 0.0\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("zero".into()));
    assert!(trace.iter().any(|event| {
        event.contains("error.code.matched") && event.contains("TOPAL-DECISION-ERROR-CODE-001")
    }));
}

#[test]
fn classified_binding_projects_success_and_propagates_error() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproject is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  quotient : Rational is 1.0 divide denominator\n  quotient + 1.0\n(project 2.0, project 0.0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 3, 2 ), Error ( domain is root./(Rational,Rational), code is division-by-zero ))"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("result.success.projected"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("result.error.projected"))
    );
}

#[test]
fn classified_binding_rejects_error_propagation_from_infallible_function() {
    let error = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nbad is fn (denominator : Rational) -> Rational\n  quotient : Rational is 1.0 divide denominator\n  quotient\nbad 0.0\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RESULT-PROJECTION-INFALLIBLE");
    assert!(error.message.contains("returning `Rational`"));
    assert!(
        error
            .help
            .is_some_and(|help| help.contains("match the Error"))
    );
}

#[test]
fn character_classifier_uses_pinned_grapheme_segmentation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "identity is fn (value : Character) -> Character\n  value\ncomposed : Character is \"a\u{301}\"\n(String (identity \"🙂\"), String composed)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"🙂\", \"a\u{301}\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-FROM-CHARACTER-001"))
            .count(),
        2
    );

    let error = Session::new()
        .evaluate("invalid : Character is \"ab\"\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-CHARACTER-CLASSIFIER");
    assert!(error.message.contains("contains 2"));
}

#[test]
fn int_modulo_is_euclidean_and_dynamic_zero_returns_error() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "modulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  left % right\nquotient-modulo is fn (left : Int, right : Int) -> Result ((Int, Int), lang arithmetic ArithmeticErrorCode)\n  left /% right\n(17 % 5, -17 % 5, 17 % -5, -17 /% 5, 17 /% -5, 17 modulo 0, 17 quotient-modulo 0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(2, 3, 2, (-4, 3), (-3, 2), Error ( domain is root.%(Int,Int), code is division-by-zero ), Error ( domain is root./%(Int,Int), code is division-by-zero ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-MODULO-001"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-QUOTIENT-MODULO-001"))
            .count(),
        2
    );
}

#[test]
fn exact_numeric_absolute_retains_operand_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(absolute -42, absolute 42, absolute -1.25, absolute 1.25)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(42, 42, Rational ( 5, 4 ), Rational ( 5, 4 ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-ABS-001"))
            .count(),
        4
    );
}

#[test]
fn named_numeric_negate_matches_exact_additive_inverse() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(negate 42, negate -42, negate 1.25, negate -1.25)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(-42, 42, Rational ( -5, 4 ), Rational ( 5, 4 ))"
    );
    assert!(trace.iter().any(|event| event.contains("root.negate(Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.negate(Rational)"))
    );
}

#[test]
fn exact_numeric_zero_uses_explicit_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(zero Int, zero Nat, zero Rational, one Int, one Nat, one Rational)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(0, 0, Rational ( 0, 1 ), 1, 1, Rational ( 1, 1 ))"
    );
    assert!(trace.iter().any(|event| event.contains("root.zero(Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.zero(Rational)"))
    );
    assert!(trace.iter().any(|event| event.contains("root.one(Int)")));
    assert!(trace.iter().any(|event| event.contains("root.zero(Nat)")));
    assert!(trace.iter().any(|event| event.contains("root.one(Nat)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.one(Rational)"))
    );
}

#[test]
fn exact_three_way_comparison_returns_nominal_alternatives() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("describe is fn (value : Comparison) -> String\n  value\n    Less then \"less\"\n    Equal then \"equal\"\n    Greater then \"greater\"\n(1 <=> 2, 2 <=> 2, 3 <=> 2, 1 <=> 1.5, describe (1 <=> 2), describe (2 <=> 2), describe (3 <=> 2))\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Less, Equal, Greater, Less, \"less\", \"equal\", \"greater\")"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-THREE-WAY-COMPARE-001"))
            .count(),
        7
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("decision.rule.selected"))
            .filter(|event| event.contains("TOPAL-DECISION-ENUM-001"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:left"))
    );
}

#[test]
fn closed_exact_rational_narrows_to_int_without_rounding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "fifty : Int is 100 / 2\nnegative-three : Int is -9 / 3\n(fifty, negative-three)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(50, -3)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-INT-EXACT-001"))
            .count(),
        2
    );

    let error = Session::new()
        .evaluate("half : Int is 1 / 2\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RATIONAL-NOT-EXACT-INT");
    assert!(error.message.contains("denominator 2"));
}

#[test]
fn dynamic_rational_to_int_validation_returns_typed_result() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "halve is fn (value : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  half : Int is value / 2\n  half\n(halve 100, halve 3)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(50, Error ( domain is root.Int(Rational), code is not-representable ))"
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001")
            && event.contains("Rational->Int:validated")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001")
            && event.contains("root.Int(Rational);not-representable")
    }));
}

#[test]
fn checked_int_construction_is_exact_and_fallible() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "as-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  Int value\n(Int 7, as-int 6.0, as-int 1.5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(7, 6, Error ( domain is root.Int(Rational), code is not-representable ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-CONSTRUCT-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Int 1.5\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RATIONAL-NOT-EXACT-INT");
}

#[test]
fn checked_nat_construction_validates_the_nonnegative_constraint() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "as-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  Nat value\n(Nat 7, as-nat 6, as-nat -1)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(7, 6, Error ( domain is root.Nat(Int), code is out-of-range ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-NAT-CONSTRUCT-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Nat -1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NAT-OUT-OF-RANGE");
}

#[test]
fn closed_rational_construction_canonicalizes_components() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(Rational 7, Rational (2, 4), Rational (2, -4), Rational (0, 5))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 7, 1 ), Rational ( 1, 2 ), Rational ( -1, 2 ), Rational ( 0, 1 ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-001"))
            .count(),
        3
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-INT-RATIONAL-CONVERT-001")
            && event.contains("Int->Rational:explicit")
    }));

    let error = Session::new()
        .evaluate("Rational (1, 0)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-DIVISION-BY-ZERO");
}

#[test]
fn dynamic_rational_construction_distinguishes_zero_failures() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "ratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  Rational (numerator, denominator)\n(1 ratio 2, 1 ratio 0, 0 ratio 0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 1, 2 ), Error ( domain is root.Rational(Int,Int), code is division-by-zero ), Error ( domain is root.Rational(Int,Int), code is indeterminate ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Rational (0, 0)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-INDETERMINATE-RATIONAL");
}

#[test]
fn int_ranges_preserve_all_endpoint_forms_and_allow_empty_ranges() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("half-open is 0 .. 10\nopen is 0 <.. 10\nclosed is 0 ..= 10\nlower-open is 0 <..= 10\nempty-interval is 10 .. 10\n(half-open, 0 in half-open, 10 in half-open, 0 in open, 10 in open, 0 in closed, 10 in closed, 0 in lower-open, 10 in lower-open, empty? empty-interval, range-lower-inclusive? lower-open, range-upper-inclusive? lower-open)\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(0 .. 10, true, false, false, false, true, true, false, true, true, false, true)"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-RANGE-BOUNDS-001"))
            .count(),
        5
    );
    assert!(trace.iter().any(|event| event.contains("empty")));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-RANGE-MEMBERSHIP-001"))
            .count(),
        8
    );
}

#[test]
fn boolean_not_is_eager_and_type_checked() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("(not true, not false, not (not true))\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, true)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-BOOLEAN-LOGIC-001"))
            .count(),
        4
    );
    let error = Session::new()
        .evaluate("not 1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-BOOLEAN-NOT-OPERAND");
}

#[test]
fn boolean_and_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true and true, true and false, false and true, false and false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false, false, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("and:eager"))
            .count(),
        4
    );
}

#[test]
fn boolean_or_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true or true, true or false, false or true, false or false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, true, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("or:eager"))
            .count(),
        4
    );
}

#[test]
fn boolean_xor_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true xor true, true xor false, false xor true, false xor false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("xor:eager"))
            .count(),
        4
    );
}

#[test]
fn explicit_optional_constructors_preserve_payload_classifiers() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(Some 42, Some \"present\", None Int, None String)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some 42, Some \"present\", None, None)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-CONSTRUCT-001"))
            .count(),
        4
    );
}

#[test]
fn contextual_none_uses_the_binding_classifier() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("missing : Optional Int is None\nmissing\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "None");
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001") && event.contains("Int")
    }));

    let error = Session::new()
        .evaluate("None\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn optional_values_cross_matching_function_boundaries() {
    let value = Session::new()
        .evaluate(
            "preserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\n(preserve (Some 7), preserve (None Int))\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some 7, None)");

    let error = Session::new()
        .evaluate(
            "preserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\npreserve (None String)\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FUNCTION-ARGUMENT-TYPE");
}

#[test]
fn contextual_none_uses_function_result_classifiers() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "implicit is fn () -> Optional Int\n  None\nexplicit is fn () -> Optional String\n  return None\n(implicit (), explicit ())\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(None, None)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001"))
            .count(),
        2
    );
}

#[test]
fn optional_decisions_bind_only_present_payloads() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (candidate : Optional Int) -> String\n  candidate\n    Some payload then \"present\"\n    None then \"absent\"\n(describe (Some 7), describe (None Int))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"present\", \"absent\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("decision.rule.selected"))
            .filter(|event| event.contains("TOPAL-DECISION-OPTIONAL-001"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("optional.payload.bound"))
            .count(),
        1
    );
}

#[test]
fn optional_equality_uses_nominal_payload_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "((None Int) = (None Int), (Some 7) = (Some 7), (Some 7) = (None Int), (Some 7) != (Some 8))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, true, false, true)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-EQUALITY-001"))
            .count(),
        4
    );

    let error = Session::new()
        .evaluate("(None Int) = (None String)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn string_character_at_returns_optional_grapheme_clusters() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "text is \"a\u{301}👩‍🔬🇸🇪\"\n(text character-at 0, text character-at 1, text character-at 2, text character-at -1, text character-at 3)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some \"a\u{301}\", Some \"👩‍🔬\", Some \"🇸🇪\", None, None)"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-CHARACTER-AT-001"))
            .count(),
        5
    );
}

#[test]
fn optional_decisions_consume_indexed_characters() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (candidate : Optional Character) -> String\n  candidate\n    Some character then String character\n    None then \"missing\"\n(describe (\"👩‍🔬\" character-at 0), describe (\"👩‍🔬\" character-at 1))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"👩‍🔬\", \"missing\")");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-OPTIONAL-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-FROM-CHARACTER-001"))
    );
}

#[test]
fn upper_uses_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("upper \"Straße σς\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"STRASSE ΣΣ\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-UPPER-001"))
    );
}

#[test]
fn lower_uses_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("lower \"İΣ\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"i\u{307}ς\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-LOWER-001"))
    );
}

#[test]
fn case_fold_uses_full_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("case-fold \"Straße Σς\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"strasse σσ\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CASE-FOLD-001"))
    );
}

#[test]
fn canonical_string_equality_preserves_exact_equality_distinction() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "composed is \"é\"\ndecomposed is \"e\u{301}\"\n(composed = decomposed, composed canonically-equals decomposed, composed canonically-equals \"e\")\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-CANONICAL-EQUALITY-001"))
            .count(),
        2
    );
}

#[test]
fn character_traversal_collects_the_exact_preserved_string() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("characters \"a\u{301}👩‍🔬🇸🇪\" collect String\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"a\u{301}👩‍🔬🇸🇪\"");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CHARACTERS-COLLECT-001"))
    );
}

#[test]
fn foreach_consumes_character_generator_with_unit() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "characters \"a\u{301}👩‍🔬🇸🇪\" foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.resumed"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.returned") && event.contains("Unit"))
    );
}

#[test]
fn foreach_rejects_non_unit_action_result() {
    let error = Session::new()
        .evaluate(
            "characters \"Topal\" foreach { character }\n  String character\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FOREACH-ACTION-RESULT");
}

#[test]
fn named_character_generator_is_consumed_linearly() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "generated is characters \"a\u{301}👩‍🔬🇸🇪\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.started"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.consumed"))
    );
}

#[test]
fn character_generator_accepts_its_explicit_classifier() {
    let value = Session::new()
        .evaluate(
            "generated : Generator Character Unit Unit is characters \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
}

#[test]
fn function_returns_fresh_character_generator() {
    let value = Session::new()
        .evaluate(
            "generate is fn (text : String) -> Generator Character Unit Unit\n  characters text\ngenerated is generate \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
}

#[test]
fn reused_character_generator_reports_consumption() {
    let error = Session::new()
        .evaluate(
            "generated is characters \"Topal\"\ngenerated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-CONSUMED");
    assert_eq!(
        error.help.as_deref(),
        Some("construct a fresh generator before traversing it again")
    );
}

#[test]
fn generator_parameter_transfers_linear_continuation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "consume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is characters \"Topal\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.parameter.transferred"))
    );
}

#[test]
fn abandoned_generator_parameter_closes_at_function_boundary() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "ignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\ngenerated is characters \"Topal\"\nignore generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CHARACTERS-CLOSE-001"))
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-GENERATOR-ERROR-CODE-001")
            && event.contains("domain=root")
            && event.contains("generator=root.characters")
    }));
}

#[test]
fn generator_error_code_has_qualified_nominal_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "code is lang generator generator-closed\n(code, code = (lang generator generator-closed))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(generator-closed, true)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-ERROR-CODE-001"))
    );
}

#[test]
fn named_single_yield_generator_is_traversable() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is yield initial\n  ()\ngenerated is once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.declared"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.started"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        2
    );
}

#[test]
fn named_generator_yield_reads_local_binding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "copy-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  copy : Character is initial\n  _ is yield copy\n  ()\ngenerated is copy-once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(trace.iter().any(|event| event.contains("binding.bind")));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        1
    );
}

#[test]
fn named_generator_can_return_before_first_yield() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "nothing is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  ()\ngenerated is nothing \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("generator.yielded"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.returned"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-EARLY-RETURN-001"))
    );
}

#[test]
fn named_generator_returns_character_after_yields() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-then-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\ngenerated is yield-then-return \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_defers_post_yield_binding_until_resume() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-twice is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  copy : Character is initial\n  _ is yield copy\n  ()\ngenerated is pause-twice \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let local = trace
        .iter()
        .position(|event| event.contains("binding.bind") && event.contains("copy"))
        .unwrap();
    let second_suspend = trace
        .iter()
        .rposition(|event| event.contains("generator.suspended"))
        .unwrap();
    assert!(resumed < local && local < second_suspend);
}

#[test]
fn custom_generator_binds_unit_resume_after_yield() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "bind-resume is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resumed is yield initial\n  resumed\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let bound = trace
        .iter()
        .position(|event| event.contains("generator.resume.bound"))
        .unwrap();
    let resolved = trace
        .iter()
        .rposition(|event| event.contains("binding.resolved") && event.contains("resumed"))
        .unwrap();
    assert!(resumed < bound && bound < resolved);
}

#[test]
fn abandoned_custom_generator_keeps_domain_separate_from_provenance() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is pause-once initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert!(trace.iter().any(|event| {
        event.contains("domain=root;code=generator-closed;generator=root.pause-once")
    }));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-CLOSE-001"))
    );
}

#[test]
fn abandoned_custom_generator_handles_close_result() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "handle-close is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resume-result is yield initial\n  resume-result\n    Error problem then ()\n    Ok resumed then ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is handle-close initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.close.bound"))
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
}

#[test]
fn custom_generator_matches_qualified_close_code() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "handle-code is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  result is yield initial\n  result\n    Error ( code is lang generator generator-closed ) then ()\n    Error problem then ()\n    Ok resumed then ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is handle-code initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
}

#[test]
fn function_transfers_custom_generator_result_to_caller() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nmake is fn ( initial : Character ) -> Generator Character Unit Unit\n  pause-once initial\ngenerated is make \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"))
    );
    assert!(
        !trace.iter().any(|event| {
            event.contains("generator.closed") && event.contains("root.pause-once")
        })
    );
}

#[test]
fn function_parameter_receives_custom_generator_ownership() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nconsume is fn ( generated : Generator Character Unit Unit ) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is pause-once \"T\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        1
    );
}

#[test]
fn function_closes_unconsumed_custom_generator_parameter() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nignore is fn ( generated : Generator Character Unit Unit ) -> Unit\n  ()\ngenerated is pause-once \"T\"\nignore generated\n",
            &mut trace,
        )
        .unwrap();
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-GENERATOR-CLOSE-001") && event.contains("root.pause-once")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("domain=root;code=generator-closed;generator=root.pause-once")
    }));
}

#[test]
fn function_parameter_preserves_generator_final_character() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\nconsume is fn ( generated : Generator Character Unit Character ) -> Character\n  generated foreach { character }\n    _ is String character\ngenerated is yield-return \"Y\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn function_result_preserves_generator_final_character() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\nmake is fn ( initial : Character ) -> Generator Character Unit Character\n  yield-return initial\ngenerated is make \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_accepts_string_initial_input() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "from-text is generator ( initial : String )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  initial-is-empty : Boolean is empty? initial\n  _ is yield \"T\"\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.empty?(String)"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.suspended"))
    );
}

#[test]
fn custom_generator_yields_strings() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "texts is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is yield \"\"\n  ()\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        2
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator String Unit Unit"))
    );
}

#[test]
fn custom_generator_returns_distinct_string() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "text-result is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator String Unit String"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_returns_explicitly_before_yielding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "done is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  return \"done\"\ngenerated is done \"unused\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-EXPLICIT-RETURN-001"))
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("generator.yielded"))
    );
}

#[test]
fn custom_generator_returns_explicitly_after_resuming() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "finish is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  return \"done\"\ngenerated is finish \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("generator.return.explicit"))
        .unwrap();
    assert!(resumed < returned);
}

#[test]
fn custom_generator_transfers_boolean_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "invert is generator ( initial : Boolean )\n  yields Boolean\n  resumes Unit\n  -> Boolean\n\n  _ is yield initial\n  not initial\ngenerated is invert true\ngenerated foreach { value }\n  _ is not value\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(false));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Boolean Unit Boolean"))
    );
}

#[test]
fn custom_generator_preserves_int_values() {
    let value = Session::new().evaluate("next is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> Int\n\n  _ is yield initial\n  initial + 1\ngenerated is next 999999999999999999999999999999\ngenerated foreach { value }\n  _ is value + 1\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "1000000000000000000000000000000");
}

#[test]
fn custom_generator_preserves_rational_values() {
    let value = Session::new().evaluate("next is generator ( initial : Rational )\n  yields Rational\n  resumes Unit\n  -> Rational\n\n  _ is yield initial\n  initial + (Rational (1, 3))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "Rational ( 2, 3 )");
}

#[test]
fn custom_generator_transfers_unit_values() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("pulse is generator ( initial : Unit )\n  yields Unit\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\ngenerated is pulse ()\ngenerated foreach { signal }\n  signal\n", &mut trace).unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Unit Unit Unit"))
    );
}

#[test]
fn custom_generator_preserves_optional_values() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("optional is generator ( initial : Optional Int )\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n\n  _ is yield initial\n  None Int\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n", &mut trace).unwrap();
    assert_eq!(value.to_string(), "None");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Optional Int Unit Optional Int"))
    );
}

#[test]
fn custom_generator_preserves_range_values() {
    let value = Session::new().evaluate("narrow is generator ( initial : Range Int )\n  yields Range Int\n  resumes Unit\n  -> Range Int\n\n  _ is yield initial\n  initial and (5 ..= 15)\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "5 ..= 10");
}

#[test]
fn custom_generator_preserves_nat_constraint() {
    let value = Session::new().evaluate("next is generator ( initial : Nat )\n  yields Nat\n  resumes Unit\n  -> Nat\n\n  _ is yield initial\n  initial + 1\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "8");
}

#[test]
fn custom_generator_preserves_enum_identity() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("Choice is Enum ( First, Second )\nchoose is generator ( initial : Choice )\n  yields Choice\n  resumes Unit\n  -> Choice\n\n  _ is yield initial\n  Second\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n", &mut trace).unwrap();
    assert_eq!(value.to_string(), "Second");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Choice Unit Choice"))
    );
}

#[test]
fn custom_generator_preserves_product_values() {
    let value = Session::new().evaluate("pair is generator ( initial : (Int, String) )\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn custom_generator_returns_structured_result_error() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-result-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert!(matches!(value, Value::Error { ref code, .. } if code == "division-by-zero"));
}

#[test]
fn custom_generator_preserves_comparison_identity() {
    let value = Session::new().evaluate("order is generator ( initial : Comparison )\n  yields Comparison\n  resumes Unit\n  -> Comparison\n\n  _ is yield initial\n  3 <=> 2\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "Greater");
}

#[test]
fn custom_generator_preserves_nested_optional_product() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-nested-optional-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "Some (8, \"done\")");
}

#[test]
fn custom_generator_preserves_nested_result_product() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-nested-result-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn custom_generator_preserves_nested_absent_optional() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-nested-none-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert!(
        matches!(value, Value::Optional { ref payload_classifier, payload: None } if payload_classifier == "(Int, String)")
    );
}

#[test]
fn custom_generators_preserve_recursive_nominal_classifiers() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-recursive-nominal-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some Second, Second)");
}

#[test]
fn custom_generator_selects_final_decision_after_resuming() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-final-decision.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("accepted".into()));
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let selected = trace
        .iter()
        .position(|event| event.contains("decision.rule.selected"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("generator.returned"))
        .unwrap();
    assert!(resumed < selected && selected < returned);
}

#[test]
fn generator_return_mismatch_reports_expected_and_found_classifiers() {
    let error = Session::new()
        .evaluate(
            "invalid is generator ( initial : Boolean )\n  yields Boolean\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  42\ngenerated is invalid true\ngenerated foreach { value }\n  _ is not value\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-RETURN-TYPE");
    assert!(error.message.contains("returned `Int`"));
    assert!(error.message.contains("requires `String`"));
    assert!(error.help.as_deref().unwrap().contains("produce `String`"));
}

#[test]
fn custom_generator_retains_local_function_across_resumption() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-local-function.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("accepted".into()));
    let declared_enum = trace
        .iter()
        .position(|event| event.contains("enum.declared") && event.contains("Choice"))
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let called = trace
        .iter()
        .rposition(|event| event.contains("function.entry"))
        .unwrap();
    assert!(declared_enum < resumed && resumed < called);
}

#[test]
fn custom_generator_restores_local_declarations_during_close() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-local-close-handler.t"),
            &mut trace,
        )
        .unwrap();
    let close_bound = trace
        .iter()
        .position(|event| event.contains("generator.close.bound"))
        .unwrap();
    let entered = trace
        .iter()
        .rposition(|event| event.contains("function.entry"))
        .unwrap();
    let closed = trace
        .iter()
        .position(|event| event.contains("generator.closed"))
        .unwrap();
    assert!(close_bound < entered && entered < closed);
}

#[test]
fn custom_generator_selects_unary_and_binary_overloads() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-overloads.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"unary\", \"binary\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.selected"))
            .count(),
        2
    );
    assert!(trace.iter().any(|event| event.contains("Int, String")));
}

#[test]
fn duplicate_generator_input_signature_is_rejected() {
    let error = Session::new()
        .evaluate(
            "same is generator ( value : Int )\n  yields Int\n  resumes Unit\n  -> Unit\n\n  _ is yield value\n  ()\nsame is generator ( other : Int )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield \"duplicate\"\n  \"duplicate\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DUPLICATE-GENERATOR-OVERLOAD");
}

#[test]
fn generator_overload_error_lists_available_inputs() {
    let error = Session::new()
        .evaluate(
            "select is generator ( value : Int )\n  yields Int\n  resumes Unit\n  -> Unit\n\n  _ is yield value\n  ()\ngenerated is select true\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-GENERATOR");
    assert!(error.message.contains("Boolean"));
    assert!(error.help.as_deref().unwrap().contains("Int"));
}

#[test]
fn foreach_result_binding_is_available_to_later_statements() {
    let value = Session::new()
        .evaluate(
            "once is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is once 7\nresult is generated foreach { value }\n  _ is value + 1\nempty? result\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(false));
}

#[test]
fn classified_foreach_result_reports_mismatch() {
    let error = Session::new()
        .evaluate(
            "once is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is once 7\nresult : Int is generated foreach { value }\n  _ is value + 1\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FOREACH-RESULT-CLASSIFIER");
    assert!(error.message.contains("returned `String`"));
    assert!(error.message.contains("requires `Int`"));
}

#[test]
fn custom_generator_crosses_generic_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../examples/language/custom-generator-generic-function-boundaries.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.result.transferred"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.parameter.transferred"))
    );
}

#[test]
fn compound_generator_crosses_function_boundaries() {
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../examples/language/custom-generator-compound-function-boundaries.t"
            ),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn nested_generator_crosses_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../examples/language/custom-generator-nested-function-boundaries.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
    let classifier = "Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)";
    assert!(trace.iter().any(|event| {
        event.contains("generator.result.transferred") && event.contains(classifier)
    }));
    assert!(trace.iter().any(|event| {
        event.contains("generator.parameter.transferred") && event.contains(classifier)
    }));
}

#[test]
fn list_generator_crosses_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/custom-generator-list-values.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "Entry ( 7, Entry ( 9, Empty ) )");
    assert!(trace.iter().any(|event| {
        event.contains("generator.result.transferred")
            && event.contains("Generator List Int Unit List Int")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("generator.parameter.transferred")
            && event.contains("Generator List Int Unit List Int")
    }));
}

#[test]
fn custom_generator_executes_discard_after_resume() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "inspect-between is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is empty? initial\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let tested = trace
        .iter()
        .enumerate()
        .skip(resumed + 1)
        .find_map(|(index, event)| event.contains("string.empty.tested").then_some(index))
        .unwrap();
    let suspended = trace
        .iter()
        .rposition(|event| event.contains("generator.suspended"))
        .unwrap();
    assert!(resumed < tested && tested < suspended);
}

#[test]
fn custom_generator_cannot_yield_after_close_result() {
    let error = Session::new()
        .evaluate(
            "invalid-close is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resume-result is yield initial\n  _ is yield initial\n  ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is invalid-close initial\n  ()\nabandon \"T\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-YIELD-AFTER-CLOSE");
    assert!(error.message.contains("cannot yield again"));
}

#[test]
fn rational_ranges_use_exact_canonical_conversion() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("interval is 0 .. 2.5\n(interval, 1.5 in interval, interval contains 2, 3 in interval)\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 0, 1 ) .. Rational ( 5, 2 ), true, true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:left"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:membership"))
    );
}

#[test]
fn lists_construct_compare_and_decompose() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/lists.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some 6, Some 6, Some Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) ), None, None, 5, false, true, true, Some (6, Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) )), Some 10, true)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("list.entry.constructed"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("list.entry.decomposed"))
    );
    assert!(trace.iter().any(|event| event.contains("equality.list")));
    for event in [
        "list.prepended",
        "list.appended",
        "list.concatenated",
        "list.entry-count",
        "list.empty.tested",
        "list.empty.constructed",
        "list.singleton.constructed",
        "list.uncons",
        "list.first",
        "list.rest",
        "list.reversed",
    ] {
        assert!(trace.iter().any(|record| record.contains(event)), "{event}");
    }
}

#[test]
fn first_and_rest_reject_non_lists() {
    for operation in ["first", "rest"] {
        let error = Session::new()
            .evaluate(&format!("{operation} 7\n"), &mut Vec::new())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
        assert!(error.message.contains("requires a List"));
    }
}

#[test]
fn recursive_list_classifiers_cross_function_boundaries() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/nested-lists.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some Entry ( (7, \"seven\"), Empty ), 1, true)"
    );
}

#[test]
fn list_containment_distinguishes_entry_sequence_and_subsequence() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/list-containment.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false, true, true, false, false)");
    for rule in [
        "TOPAL-LIST-CONTAINS-ENTRY-001",
        "TOPAL-LIST-CONTAINS-SEQUENCE-001",
        "TOPAL-LIST-CONTAINS-SUBSEQUENCE-001",
    ] {
        assert!(trace.iter().any(|event| event.contains(rule)), "{rule}");
    }
}

#[test]
fn list_containment_requires_compatible_classifiers() {
    let error = Session::new()
        .evaluate(
            "numbers : List Int is one 1\ntexts : List String is one \"one\"\nnumbers contains-sequence texts\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-CONTAINMENT-CLASSIFIER");
    assert!(error.message.contains("List String"));
}

#[test]
fn list_value_removal_preserves_retained_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../examples/language/list-removal.t"),
            &mut trace,
        )
        .unwrap();
    assert!(
        value
            .to_string()
            .contains("Entry ( 1, Entry ( 3, Entry ( 2")
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-LIST-REMOVE-FIRST-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-LIST-REMOVE-ALL-001"))
    );
}

#[test]
fn list_value_removal_rejects_wrong_classifier() {
    let error = Session::new()
        .evaluate(
            "values : List Int is one 1\nvalues remove-first \"1\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-REMOVAL-CLASSIFIER");
}

#[test]
fn uncons_is_total_for_empty_lists_and_rejects_other_values() {
    let value = Session::new()
        .evaluate("uncons (empty List Int)\n", &mut Vec::new())
        .unwrap();
    assert_eq!(value.to_string(), "None");

    let error = Session::new()
        .evaluate("uncons 7\n", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    assert!(error.message.contains("requires a List"));
}

#[test]
fn explicit_empty_and_singleton_lists_preserve_numeric_one() {
    let value = Session::new()
        .evaluate(
            "empty-values is empty List String\nsingleton is one \"Topal\"\n(empty-values, singleton, one Int)\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Empty, Entry ( \"Topal\", Empty ), 1)");
}

#[test]
fn list_operations_reject_incompatible_classifiers() {
    let entry = Session::new()
        .evaluate(
            "values : List Int is Empty\nvalues append \"bad\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(entry.code, "E-LIST-ENTRY-CLASSIFIER");
    assert!(entry.message.contains("requires `Int`"));

    let concat = Session::new()
        .evaluate(
            "numbers : List Int is Empty\ntexts : List String is Empty\nnumbers concat texts\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(concat.code, "E-LIST-CONCAT-CLASSIFIER");
    assert!(concat.message.contains("List String"));
}

#[test]
fn list_entry_classifier_mismatch_is_precise() {
    let error = Session::new()
        .evaluate(
            "values : List Int is Entry ( \"bad\", Empty )\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-ENTRY-CLASSIFIER");
    assert!(error.message.contains("requires `Int`"));
    assert!(error.help.unwrap().contains("use a `Int` value"));
}

#[test]
fn list_remainder_must_be_a_list() {
    let error = Session::new()
        .evaluate("values : List Int is Entry ( 7, 8 )\n", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-REMAINDER");
    assert!(error.help.unwrap().contains("Empty"));
}

#[test]
fn loaded_modules_expose_only_published_members() {
    let mut session = Session::new();
    session
        .load_module(
            "math",
            "use language (\n  version is v0.1\n)\nprivate-value is 40\npub answer is private-value + 2\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(
        session.evaluate("math answer", &mut Vec::new()).unwrap(),
        Value::Int(BigInt::from(42))
    );
    let error = session
        .evaluate("math private-value", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-NAMESPACE-MEMBER-NOT-FOUND");
}

#[test]
fn interface_implementations_require_exact_shapes() {
    let source = "Parser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  other is fn (source : String) -> Boolean\n    true\n()";
    let error = Session::new()
        .evaluate(source, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-INTERFACE-IMPLEMENTATION");
}
