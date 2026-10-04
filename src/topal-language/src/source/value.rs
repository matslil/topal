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
    Infinity {
        negative: bool,
        classifier: String,
    },
    Rational(BigRational),
    IntRange {
        lower: BigInt,
        upper: BigInt,
        lower_inclusive: bool,
        upper_inclusive: bool,
    },
    InfiniteIntRange {
        lower: ExtendedInt,
        upper: ExtendedInt,
        lower_inclusive: bool,
        upper_inclusive: bool,
    },
    RationalRange {
        lower: BigRational,
        upper: BigRational,
        lower_inclusive: bool,
        upper_inclusive: bool,
    },
    InfiniteRationalRange {
        lower: ExtendedRational,
        upper: ExtendedRational,
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
        yield_classifier: String,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ExtendedInt {
    NegativeInfinity,
    Finite(BigInt),
    PositiveInfinity,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ExtendedRational {
    NegativeInfinity,
    Finite(BigRational),
    PositiveInfinity,
}

impl fmt::Display for ExtendedRational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeInfinity => formatter.write_str("-Infinity"),
            Self::Finite(value) => write!(
                formatter,
                "Rational ( {}, {} )",
                value.numer(),
                value.denom()
            ),
            Self::PositiveInfinity => formatter.write_str("+Infinity"),
        }
    }
}

impl fmt::Display for ExtendedInt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeInfinity => formatter.write_str("-Infinity"),
            Self::Finite(value) => value.fmt(formatter),
            Self::PositiveInfinity => formatter.write_str("+Infinity"),
        }
    }
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
    parameters: Vec<CapturedPattern>,
    body: Box<Expression>,
    bindings: BTreeMap<String, Value>,
    defining_context: Option<BTreeMap<String, Value>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CapturedPattern {
    Binding(String),
    Product(Vec<Self>),
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
    supports_equality: bool,
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
    functions: BTreeMap<String, InterfaceFunctionShape>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InterfaceFunctionShape {
    parameters: Vec<(String, Option<String>)>,
    result: String,
    clauses: InterfaceClauseShape,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct InterfaceClauseShape {
    requires: Option<String>,
    effects: Option<String>,
    guarantees: Option<String>,
    result_binding: Option<String>,
    ensures: Option<String>,
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
            Self::InfiniteIntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            } => write!(
                formatter,
                "{lower} {} {upper}",
                range_symbol(*lower_inclusive, *upper_inclusive)
            ),
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
            Self::Infinity { negative, .. } => {
                formatter.write_str(if *negative { "-Infinity" } else { "+Infinity" })
            }
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
            Self::InfiniteRationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            } => write!(
                formatter,
                "{lower} {} {upper}",
                range_symbol(*lower_inclusive, *upper_inclusive)
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
            Self::UnfoldGenerator {
                yield_classifier, ..
            } => write!(formatter, "<Generator {yield_classifier} Unit Unit>"),
            Self::SuspendedGenerator {
                yield_classifier,
                return_classifier,
                ..
            } => write!(
                formatter,
                "<Generator {yield_classifier} Unit {return_classifier}>"
            ),
            Self::String(value) => formatter.write_str(&display_string_literal(value)),
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
    root_namespace: Option<Rc<NamespaceValue>>,
    defining_context: Option<BTreeMap<String, Value>>,
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
#[allow(clippy::box_collection)] // Keep recursive evaluator state below the tested stack ceiling.
struct UserFunction {
    source: SourceText,
    is_static: bool,
    parameters: Vec<(String, String)>,
    parameter_packages: BTreeMap<usize, Vec<UserParameterField>>,
    result: String,
    generic_names: BTreeSet<String>,
    metadata: Box<UserFunctionMetadata>,
    body: Box<Vec<Statement>>,
    bindings: BTreeMap<String, Value>,
    context_bindings: BTreeMap<String, Value>,
    termination_rule: Option<&'static str>,
    recursion_target: Option<String>,
}

fn function_defining_context(function: &UserFunction) -> Option<BTreeMap<String, Value>> {
    (!function.context_bindings.is_empty()).then(|| function.context_bindings.clone())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserFunctionContracts {
    requires: Option<Expression>,
    guarantees: Option<Expression>,
    result_binding: Option<String>,
    ensures: Option<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserFunctionMetadata {
    effect_bound: Option<String>,
    contracts: Option<UserFunctionContracts>,
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
    result_classifier: Option<String>,
    return_classifier: Option<String>,
}

#[derive(Clone, Copy)]
struct FunctionDeclaration<'a> {
    name: Span,
    is_static: bool,
    parameters: &'a [FunctionParameter],
    result: Span,
    effect_bound: Option<Span>,
    clauses: &'a FunctionClauses,
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

pub(crate) fn expression_mentions_name(
    source: &SourceText,
    expression: &Expression,
    name: &str,
) -> bool {
    match expression {
        Expression::Identifier(span) | Expression::ContextIdentifier(span) => {
            source.slice(*span) == name
        }
        Expression::Block { statements, .. } => statements
            .iter()
            .any(|statement| statement_mentions_name(source, statement, name)),
        Expression::Product { fields, .. } => fields
            .iter()
            .any(|field| expression_mentions_name(source, &field.value, name)),
        Expression::DecisionTable { subject, rules, .. } => {
            expression_mentions_name(source, subject, name)
                || rules.iter().any(|rule| {
                    decision_matcher_mentions_name(source, &rule.matcher, name)
                        || expression_mentions_name(source, &rule.action, name)
                })
        }
        Expression::AnonymousFunction { body, .. } => expression_mentions_name(source, body, name),
        Expression::Application { items, .. } => items
            .iter()
            .any(|item| expression_mentions_name(source, item, name)),
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Infinity(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Discard(_)
        | Expression::Callable { .. } => false,
    }
}

fn decision_matcher_mentions_name(
    source: &SourceText,
    matcher: &DecisionMatcher,
    name: &str,
) -> bool {
    match matcher {
        DecisionMatcher::Identifier(span) => source.slice(*span) == name,
        DecisionMatcher::Union { alternative, .. } => source.slice(*alternative) == name,
        DecisionMatcher::Variant {
            type_name, index, ..
        } => source.slice(*type_name) == name || source.slice(*index) == name,
        DecisionMatcher::ErrorCode {
            namespace,
            vocabulary,
            code,
            ..
        } => {
            source.slice(*namespace) == name
                || source.slice(*vocabulary) == name
                || source.slice(*code) == name
        }
        DecisionMatcher::Comparison { operand, .. } => {
            expression_mentions_name(source, operand, name)
        }
        DecisionMatcher::Boolean { .. }
        | DecisionMatcher::Result { .. }
        | DecisionMatcher::Optional { .. }
        | DecisionMatcher::ListEmpty(_)
        | DecisionMatcher::ListEntry { .. }
        | DecisionMatcher::Otherwise(_) => false,
    }
}

fn statement_mentions_name(source: &SourceText, statement: &Statement, name: &str) -> bool {
    match statement {
        Statement::Published { declaration, .. } => {
            statement_mentions_name(source, declaration, name)
        }
        Statement::Binding { value, .. }
        | Statement::ContextAssignment { value, .. }
        | Statement::Discard { value, .. }
        | Statement::Return { value, .. }
        | Statement::Expression(value) => expression_mentions_name(source, value, name),
        Statement::Implementation {
            classifier,
            declarations,
            ..
        } => {
            expression_mentions_name(source, classifier, name)
                || declarations
                    .iter()
                    .any(|declaration| statement_mentions_name(source, declaration, name))
        }
        Statement::Function {
            parameters,
            clauses,
            body,
            ..
        } => {
            parameters.iter().any(|parameter| {
                parameter
                    .default
                    .as_ref()
                    .is_some_and(|default| expression_mentions_name(source, default, name))
            }) || [
                clauses.requires.as_deref(),
                clauses.effects.as_deref(),
                clauses.guarantees.as_deref(),
                clauses.ensures.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(|expression| expression_mentions_name(source, expression, name))
                || body
                    .iter()
                    .any(|statement| statement_mentions_name(source, statement, name))
        }
        Statement::Generator {
            parameters, body, ..
        } => {
            parameters.iter().any(|parameter| {
                parameter
                    .default
                    .as_ref()
                    .is_some_and(|default| expression_mentions_name(source, default, name))
            }) || body
                .iter()
                .any(|statement| statement_mentions_name(source, statement, name))
        }
        Statement::InterfaceImplementation { declarations, .. } => declarations
            .iter()
            .any(|declaration| statement_mentions_name(source, declaration, name)),
        Statement::Foreach {
            source: input,
            body,
            ..
        } => {
            expression_mentions_name(source, input, name)
                || body
                    .iter()
                    .any(|statement| statement_mentions_name(source, statement, name))
        }
        Statement::LanguageSelection { .. }
        | Statement::LibrarySelection { .. }
        | Statement::DiagnosticControl { .. }
        | Statement::StateField { .. }
        | Statement::Union { .. }
        | Statement::Interface { .. } => false,
    }
}

pub(crate) fn body_mentions_name(source: &SourceText, body: &[Statement], name: &str) -> bool {
    body.iter()
        .any(|statement| statement_mentions_name(source, statement, name))
}

fn body_context_names(source: &SourceText, body: &[Statement]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for statement in body {
        collect_statement_context_names(source, statement, &mut names);
    }
    names
}

fn collect_statement_context_names(
    source: &SourceText,
    statement: &Statement,
    names: &mut BTreeSet<String>,
) {
    match statement {
        Statement::Published { declaration, .. } => {
            collect_statement_context_names(source, declaration, names);
        }
        Statement::Binding { value, .. }
        | Statement::ContextAssignment { value, .. }
        | Statement::Discard { value, .. }
        | Statement::Return { value, .. }
        | Statement::Expression(value) => collect_expression_context_names(source, value, names),
        Statement::Implementation {
            classifier,
            declarations,
            ..
        } => {
            collect_expression_context_names(source, classifier, names);
            for declaration in declarations {
                collect_statement_context_names(source, declaration, names);
            }
        }
        Statement::Function {
            parameters,
            clauses,
            body,
            ..
        } => {
            for default in parameters
                .iter()
                .filter_map(|parameter| parameter.default.as_ref())
            {
                collect_expression_context_names(source, default, names);
            }
            for expression in [
                clauses.requires.as_deref(),
                clauses.effects.as_deref(),
                clauses.guarantees.as_deref(),
                clauses.ensures.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                collect_expression_context_names(source, expression, names);
            }
            for statement in body {
                collect_statement_context_names(source, statement, names);
            }
        }
        Statement::Generator {
            parameters, body, ..
        } => {
            for default in parameters
                .iter()
                .filter_map(|parameter| parameter.default.as_ref())
            {
                collect_expression_context_names(source, default, names);
            }
            for statement in body {
                collect_statement_context_names(source, statement, names);
            }
        }
        Statement::InterfaceImplementation { declarations, .. } => {
            for declaration in declarations {
                collect_statement_context_names(source, declaration, names);
            }
        }
        Statement::Foreach {
            source: input,
            body,
            ..
        } => {
            collect_expression_context_names(source, input, names);
            for statement in body {
                collect_statement_context_names(source, statement, names);
            }
        }
        Statement::LanguageSelection { .. }
        | Statement::LibrarySelection { .. }
        | Statement::DiagnosticControl { .. }
        | Statement::StateField { .. }
        | Statement::Union { .. }
        | Statement::Interface { .. } => {}
    }
}

fn collect_expression_context_names(
    source: &SourceText,
    expression: &Expression,
    names: &mut BTreeSet<String>,
) {
    match expression {
        Expression::ContextIdentifier(span) => {
            names.insert(source.slice(*span).to_owned());
        }
        Expression::Block { statements, .. } => {
            for statement in statements {
                collect_statement_context_names(source, statement, names);
            }
        }
        Expression::Product { fields, .. } => {
            for field in fields {
                collect_expression_context_names(source, &field.value, names);
            }
        }
        Expression::DecisionTable { subject, rules, .. } => {
            collect_expression_context_names(source, subject, names);
            for rule in rules {
                if let DecisionMatcher::Comparison { operand, .. } = &rule.matcher {
                    collect_expression_context_names(source, operand, names);
                }
                collect_expression_context_names(source, &rule.action, names);
            }
        }
        Expression::AnonymousFunction { body, .. } => {
            collect_expression_context_names(source, body, names);
        }
        Expression::Application { items, .. } => {
            for item in items {
                collect_expression_context_names(source, item, names);
            }
        }
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Infinity(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Identifier(_)
        | Expression::Discard(_)
        | Expression::Callable { .. } => {}
    }
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
