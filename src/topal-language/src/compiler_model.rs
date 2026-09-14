//! Shared checked model consumed by the native compiler.
//!
//! This first model intentionally covers a bounded, explicit language slice.
//! Extending it is how later compiler increments acquire semantics; the LLVM
//! backend never reinterprets the source syntax itself.

use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;
use num_rational::BigRational;
use topal_semantics::{LanguageVersion, ObjectKind};
use topal_source::{
    Diagnostic, SourceText, Span, canonically_equal, case_fold, character_at, character_count,
    characters, lowercase, normalize_nfc, normalize_nfd, uppercase,
};
use topal_syntax::{
    AnonymousPattern, CallableKind, DecisionMatcher, Expression, FunctionClauses,
    FunctionParameter, InterfaceFunction, ProductField, Statement, lex, parse,
};

use crate::source::{
    explicit_single_measure, expression_mentions_name, parse_integer, parse_rational, parse_string,
    prove_explicit_parameter_recursion, prove_int_recursion, prove_mutual_bounded_recursion_edge,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerEnumType {
    pub name: String,
    pub alternatives: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerSumAlternative {
    pub name: String,
    pub payload: Option<CompilerType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerSumType {
    pub name: String,
    pub positional: bool,
    pub alternatives: Vec<CompilerSumAlternative>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerModularType {
    pub name: String,
    pub signed: bool,
    pub lower: BigInt,
    pub upper: BigInt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerEffectRow {
    pub identities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerGeneratorType {
    pub yield_type: Box<CompilerType>,
    pub resume_type: Box<CompilerType>,
    pub result_type: Box<CompilerType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerIdentity {
    pub kind: ObjectKind,
    pub canonical: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerTypeViewForm {
    Primitive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerTypeView {
    pub form: CompilerTypeViewForm,
    pub identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerLanguageContext {
    pub language: String,
    pub version: LanguageVersion,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerCapability {
    pub alternatives: Vec<Vec<String>>,
}

impl CompilerCapability {
    fn atomic(identity: &str) -> Self {
        Self {
            alternatives: vec![vec![identity.to_owned()]],
        }
    }

    fn and(&self, other: &Self) -> Self {
        Self::canonical(self.alternatives.iter().flat_map(|left| {
            other.alternatives.iter().map(move |right| {
                let mut conjunction = left.clone();
                conjunction.extend(right.iter().cloned());
                conjunction
            })
        }))
    }

    fn or(&self, other: &Self) -> Self {
        Self::canonical(self.alternatives.iter().chain(&other.alternatives).cloned())
    }

    fn canonical(alternatives: impl IntoIterator<Item = Vec<String>>) -> Self {
        let alternatives = alternatives
            .into_iter()
            .map(|mut conjunction| {
                conjunction.sort();
                conjunction.dedup();
                conjunction
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self { alternatives }
    }

    #[must_use]
    pub fn display(&self) -> String {
        self.alternatives
            .iter()
            .map(|conjunction| conjunction.join(" and "))
            .collect::<Vec<_>>()
            .join(" or ")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerFunctionView {
    pub identity: String,
    pub inputs: Vec<String>,
    pub output: String,
    pub is_static: bool,
    pub declared_effects: Option<CompilerEffectRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerInterfaceOperation {
    pub name: String,
    pub parameters: Vec<CompilerType>,
    pub result: CompilerType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerInterface {
    pub identity: String,
    pub operations: Vec<CompilerInterfaceOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerInterfaceOperationEvidence {
    pub role: String,
    pub declaration_identity: String,
    pub declared_effects: Option<CompilerEffectRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerInterfaceImplementation {
    pub interface_identity: String,
    pub operations: Vec<CompilerInterfaceOperationEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerType {
    Unit,
    Completed,
    Effect,
    Type,
    Scope,
    Function,
    Identity,
    TypeView,
    FunctionView,
    LanguageContext,
    Capability,
    Constraint,
    Boolean,
    Version,
    Int,
    Nat,
    Rational,
    Comparison,
    Error,
    ErrorCode,
    ErrorDomain,
    SourceLocation,
    Modular(CompilerModularType),
    Enum(CompilerEnumType),
    Sum(CompilerSumType),
    Range(Box<Self>),
    Result(Box<Self>),
    Optional(Box<Self>),
    List(Box<Self>),
    TraversalControl(Box<Self>),
    Generator(CompilerGeneratorType),
    Refined { constraint: String, base: Box<Self> },
    Character,
    String,
    Tuple(Vec<Self>),
    Record(Vec<(String, Self)>),
}

impl CompilerType {
    #[must_use]
    pub const fn machine_scalar(&self) -> bool {
        matches!(
            self,
            Self::Unit
                | Self::Completed
                | Self::Effect
                | Self::Type
                | Self::Scope
                | Self::Function
                | Self::Constraint
                | Self::Boolean
                | Self::Version
                | Self::Int
                | Self::Nat
                | Self::Rational
                | Self::Comparison
                | Self::Error
                | Self::ErrorCode
                | Self::ErrorDomain
                | Self::SourceLocation
                | Self::Modular(_)
                | Self::Enum(_)
                | Self::Range(_)
                | Self::Result(_)
                | Self::Optional(_)
                | Self::List(_)
                | Self::TraversalControl(_)
                | Self::Generator(_)
                | Self::Character
                | Self::String
        ) || matches!(self, Self::Refined { base, .. } if base.machine_scalar())
    }

    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Unit => "Unit".into(),
            Self::Completed => "Completed".into(),
            Self::Effect => "Effect".into(),
            Self::Type => "Type".into(),
            Self::Scope => "Scope".into(),
            Self::Function => "Function".into(),
            Self::Identity => "lang Identity".into(),
            Self::TypeView => "lang TypeView".into(),
            Self::FunctionView => "lang FunctionView".into(),
            Self::LanguageContext => "lang LanguageContext".into(),
            Self::Capability => "Capability".into(),
            Self::Constraint => "Constraint".into(),
            Self::Boolean => "Boolean".into(),
            Self::Version => "Version".into(),
            Self::Int => "Int".into(),
            Self::Nat => "Nat".into(),
            Self::Rational => "Rational".into(),
            Self::Comparison => "Comparison".into(),
            Self::Error => "Error".into(),
            Self::ErrorCode => "lang arithmetic ArithmeticErrorCode".into(),
            Self::ErrorDomain => "ErrorDomain".into(),
            Self::SourceLocation => "SourceLocation".into(),
            Self::Modular(modular) => modular.name.clone(),
            Self::Enum(enumeration) => enumeration.name.clone(),
            Self::Sum(sum) => sum.name.clone(),
            Self::Range(endpoint) => format!("Range {}", endpoint.name()),
            Self::Result(success) => format!(
                "Result ({}, lang arithmetic ArithmeticErrorCode)",
                success.name()
            ),
            Self::Optional(payload) => format!("Optional {}", payload.name()),
            Self::List(element) => format!("List {}", element.name()),
            Self::TraversalControl(payload) => format!("TraversalControl {}", payload.name()),
            Self::Generator(generator) => format!(
                "Generator {} {} {}",
                generator.yield_type.name(),
                generator.resume_type.name(),
                generator.result_type.name()
            ),
            Self::Refined { constraint, .. } => constraint.clone(),
            Self::Character => "Character".into(),
            Self::String => "String".into(),
            Self::Tuple(fields) => format!(
                "({})",
                fields.iter().map(Self::name).collect::<Vec<_>>().join(", ")
            ),
            Self::Record(fields) => format!(
                "({})",
                fields
                    .iter()
                    .map(|(name, value_type)| format!("{name} : {}", value_type.name()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntRange {
    pub lower: BigInt,
    pub upper: BigInt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClosedIntRange {
    lower: BigInt,
    upper: BigInt,
    lower_inclusive: bool,
    upper_inclusive: bool,
}

impl ClosedIntRange {
    fn contains(&self, value: &BigInt) -> bool {
        let above_lower = if self.lower_inclusive {
            value >= &self.lower
        } else {
            value > &self.lower
        };
        let below_upper = if self.upper_inclusive {
            value <= &self.upper
        } else {
            value < &self.upper
        };
        above_lower && below_upper
    }
}

impl IntRange {
    fn exact(value: BigInt) -> Self {
        Self {
            lower: value.clone(),
            upper: value,
        }
    }

    fn union(left: &Self, right: &Self) -> Self {
        Self {
            lower: left.lower.clone().min(right.lower.clone()),
            upper: left.upper.clone().max(right.upper.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerBinary {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    QuotientModulo,
    Power,
    Compare,
    Range,
    RangeOpen,
    RangeInclusive,
    RangeOpenInclusive,
    In,
    Contains,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Xor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerExpression {
    pub kind: CompilerExpressionKind,
    pub value_type: CompilerType,
    pub int_range: Option<IntRange>,
    pub rational_value: Option<BigRational>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerComparisonRule {
    pub operation: CompilerBinary,
    pub operand: CompilerExpression,
    pub action: CompilerExpression,
    pub subject_to_rational: bool,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerFallible {
    RationalConstruct,
    RationalDivide,
    RationalPower,
    IntModulo,
    IntQuotientModulo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerValidation {
    RationalToInt,
    IntToNat,
    Constraint(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerErrorField {
    Code,
    Domain,
    Detail,
    Cause,
    Source,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerErrorCodeRule {
    pub code: u32,
    pub action: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerEnumRule {
    pub value: u32,
    pub action: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerSumRule {
    pub value: u32,
    pub binding: Option<(String, Span)>,
    pub action: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerExpressionKind {
    Unit,
    Completed,
    Effect,
    TypeValue(u32),
    Root,
    FunctionValue(u32),
    Identity(CompilerIdentity),
    TypeView(CompilerTypeView),
    FunctionView(CompilerFunctionView),
    LanguageContext(CompilerLanguageContext),
    Capability(CompilerCapability),
    ConstraintValue(u32),
    Boolean(bool),
    Version(LanguageVersion),
    Int(BigInt),
    Rational(BigRational),
    String(String),
    StringEmpty,
    StringConcat {
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
    },
    StringEmptyPredicate(Box<CompilerExpression>),
    StringUtf8ByteCount(Box<CompilerExpression>),
    StringCharactersGenerator {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
    },
    StringCharactersCollect {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
    },
    StringCharactersClose(Box<CompilerExpression>),
    StringCharactersForeach {
        source: Box<CompilerExpression>,
        characters: Vec<String>,
        parameter: CompilerParameter,
        body: Box<CompilerBlock>,
    },
    ErrorCode(u32),
    IntToModular {
        value: Box<CompilerExpression>,
        modular: CompilerModularType,
    },
    ModularReduce {
        value: Box<CompilerExpression>,
        modular: CompilerModularType,
    },
    ModularValidate {
        value: Box<CompilerExpression>,
        modular: CompilerModularType,
        error_span: Span,
    },
    Enum(u32),
    Sum {
        value: u32,
        payload: Option<Box<CompilerExpression>>,
    },
    Tuple(Vec<CompilerExpression>),
    Record(Vec<(String, CompilerExpression)>),
    RecordReconstruct {
        base: Box<CompilerExpression>,
        replacements: Vec<(String, CompilerExpression)>,
    },
    RecordField {
        record: Box<CompilerExpression>,
        label: String,
    },
    Block(Box<CompilerBlock>),
    Local(String),
    Negate(Box<CompilerExpression>),
    Absolute(Box<CompilerExpression>),
    IntToRational(Box<CompilerExpression>),
    RationalConstruct {
        numerator: Box<CompilerExpression>,
        denominator: Box<CompilerExpression>,
    },
    RationalToInt(Box<CompilerExpression>),
    IntToNat(Box<CompilerExpression>),
    ResultSuccess(Box<CompilerExpression>),
    ResultProject(Box<CompilerExpression>),
    OptionalSome(Box<CompilerExpression>),
    OptionalNone,
    ListEmpty,
    ListEntry {
        value: Box<CompilerExpression>,
        remaining: Box<CompilerExpression>,
    },
    ListContainsEntry {
        list: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    ListContainsSequence {
        list: Box<CompilerExpression>,
        pattern: Box<CompilerExpression>,
    },
    ListContainsSubsequence {
        list: Box<CompilerExpression>,
        pattern: Box<CompilerExpression>,
    },
    ListRemoveFirst {
        list: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    ListRemoveAll {
        list: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    ListPrepend {
        list: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    ListAppend {
        list: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    ListConcat {
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
    },
    ListReverse(Box<CompilerExpression>),
    ListEntryCount(Box<CompilerExpression>),
    ListEmptyPredicate(Box<CompilerExpression>),
    ListFirst(Box<CompilerExpression>),
    ListRest(Box<CompilerExpression>),
    ListUncons(Box<CompilerExpression>),
    ListMap {
        list: Box<CompilerExpression>,
        parameters: Vec<CompilerParameter>,
        body: Box<CompilerBlock>,
    },
    ListSelect {
        list: Box<CompilerExpression>,
        parameters: Vec<CompilerParameter>,
        body: Box<CompilerBlock>,
    },
    ListRangeSelect {
        list: Box<CompilerExpression>,
        range: Box<CompilerExpression>,
        indexes: bool,
    },
    TraversalControl {
        finish: bool,
        value: Box<CompilerExpression>,
    },
    ListFold {
        list: Box<CompilerExpression>,
        initial: Box<CompilerExpression>,
        parameters: Vec<CompilerParameter>,
        body: Box<CompilerBlock>,
    },
    IterateGenerator {
        initial: Box<CompilerExpression>,
        parameters: Vec<CompilerParameter>,
        next: Box<CompilerBlock>,
    },
    GeneratorTakeWhile {
        generator: Box<CompilerExpression>,
        parameters: Vec<CompilerParameter>,
        predicate: Box<CompilerBlock>,
    },
    UnfoldGenerator {
        seed: Box<CompilerExpression>,
        parameters: Vec<CompilerParameter>,
        step: Box<CompilerBlock>,
    },
    IterateGeneratorForeach {
        generator: Box<CompilerExpression>,
        parameter: CompilerParameter,
        body: Box<CompilerBlock>,
    },
    GeneratorCollect(Box<CompilerExpression>),
    ErrorField {
        error: Box<CompilerExpression>,
        field: CompilerErrorField,
    },
    Validate {
        operation: CompilerValidation,
        value: Box<CompilerExpression>,
        error_span: Span,
    },
    Fallible {
        operation: CompilerFallible,
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
        error_span: Span,
    },
    RangeLower(Box<CompilerExpression>),
    RangeUpper(Box<CompilerExpression>),
    RangeLowerInclusive(Box<CompilerExpression>),
    RangeUpperInclusive(Box<CompilerExpression>),
    RangeEmpty(Box<CompilerExpression>),
    Not(Box<CompilerExpression>),
    Binary {
        operation: CompilerBinary,
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
    },
    Call {
        symbol: String,
        arguments: Vec<CompilerExpression>,
    },
    BooleanDecision {
        subject: Box<CompilerExpression>,
        when_true: Box<CompilerExpression>,
        when_false: Box<CompilerExpression>,
    },
    OrderedComparisonDecision {
        subject: Box<CompilerExpression>,
        rules: Vec<CompilerComparisonRule>,
        otherwise: Box<CompilerExpression>,
    },
    ComparisonValueDecision {
        subject: Box<CompilerExpression>,
        when_less: Box<CompilerExpression>,
        when_equal: Box<CompilerExpression>,
        when_greater: Box<CompilerExpression>,
    },
    EnumDecision {
        subject: Box<CompilerExpression>,
        rules: Vec<CompilerEnumRule>,
        otherwise: Option<Box<CompilerExpression>>,
    },
    SumDecision {
        subject: Box<CompilerExpression>,
        rules: Vec<CompilerSumRule>,
        otherwise: Option<Box<CompilerExpression>>,
    },
    ResultDecision {
        subject: Box<CompilerExpression>,
        ok_binding: String,
        ok_binding_span: Span,
        ok_action: Box<CompilerExpression>,
        error_codes: Vec<CompilerErrorCodeRule>,
        error_fallback: Option<(String, Span, Box<CompilerExpression>)>,
    },
    OptionalDecision {
        subject: Box<CompilerExpression>,
        some_binding: Option<(String, Span)>,
        some_action: Box<CompilerExpression>,
        none_action: Box<CompilerExpression>,
    },
    ListDecision {
        subject: Box<CompilerExpression>,
        entry_bindings: Option<((String, Span), (String, Span))>,
        entry_action: Box<CompilerExpression>,
        empty_action: Box<CompilerExpression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerBinding {
    pub name: String,
    pub storage_name: String,
    pub value: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerBlock {
    pub statements: Vec<CompilerStatement>,
    pub result: CompilerExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerStatement {
    Binding(CompilerBinding),
    Discard(CompilerExpression),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerParameter {
    pub name: String,
    pub discarded: bool,
    pub value_type: CompilerType,
    pub int_range: Option<IntRange>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerFunction {
    pub source_name: String,
    pub symbol: String,
    pub parameters: Vec<CompilerParameter>,
    pub result_type: CompilerType,
    pub body: CompilerBlock,
    pub span: Span,
    pub is_static: bool,
    pub declared_effects: Option<CompilerEffectRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerConstraint {
    pub name: String,
    pub base_type: CompilerType,
    pub parameter: String,
    pub parameter_storage: String,
    pub predicate: CompilerExpression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerProgram {
    pub source: SourceText,
    pub language_version: LanguageVersion,
    pub main: CompilerBlock,
    pub function_value_names: Vec<String>,
    pub constraints: Vec<CompilerConstraint>,
    pub interfaces: Vec<CompilerInterface>,
    pub interface_implementations: Vec<CompilerInterfaceImplementation>,
    /// Instances are in callee-before-caller order.
    pub functions: Vec<CompilerFunction>,
}

#[derive(Clone)]
struct FunctionSource {
    name: Span,
    parameters: Vec<FunctionParameter>,
    result: Span,
    effect_bound: Option<Span>,
    declared_effects: Option<CompilerEffectRow>,
    body: Vec<Statement>,
    span: Span,
    is_static: bool,
}

#[derive(Clone)]
struct CompilerRecursionProof {
    rule: &'static str,
    nat_step_parameters: BTreeSet<usize>,
    mutual_target: Option<String>,
}

#[derive(Clone)]
struct ActiveRecursiveFunction {
    source_name: String,
    symbol: String,
    result_type: CompilerType,
    proof: CompilerRecursionProof,
}

type EnumTypes = BTreeMap<String, (CompilerEnumType, Span)>;
type EnumAlternativeBindings = BTreeMap<String, (CompilerEnumType, u32, Span)>;
type SumTypes = BTreeMap<String, (CompilerSumType, Span)>;
type SumAlternativeBindings = BTreeMap<String, (CompilerSumType, u32, Span)>;
type ModularTypes = BTreeMap<String, (CompilerModularType, Span)>;
type InterfaceTypes = BTreeMap<String, (CompilerInterface, Span)>;

struct EnumSource {
    name: Span,
    alternatives: Vec<(String, Span)>,
    span: Span,
}

struct SumSource {
    name: Span,
    positional: bool,
    alternatives: Vec<(String, Option<Span>, Span)>,
    span: Span,
}

struct ModularSource {
    name: Span,
    signed: bool,
    range: Expression,
    span: Span,
}

struct InterfaceSource {
    name: Span,
    functions: Vec<InterfaceFunction>,
    span: Span,
}

#[derive(Clone)]
struct BindingFacts {
    storage_name: String,
    runtime_bound: bool,
    value_type: CompilerType,
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
    string_value: Option<String>,
    closed_int_range: Option<ClosedIntRange>,
    record_fields: BTreeMap<String, StaticValueFacts>,
    namespace: Option<CompilerNamespaceFacts>,
    callable: Option<CompilerCallableFacts>,
    static_capability: Option<CompilerCapability>,
}

#[derive(Clone)]
struct CompilerNamespaceFacts {
    name: String,
    functions: BTreeMap<String, Vec<FunctionSource>>,
    bindings: BTreeMap<String, CompilerDataMemberFacts>,
}

#[derive(Clone)]
struct CompilerDataMemberFacts {
    storage_name: String,
    value_type: CompilerType,
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
    declaration_end: usize,
}

#[derive(Clone)]
struct CompilerContextCapture {
    parameter_name: String,
    value_type: CompilerType,
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
    argument: CompilerExpression,
    span: Span,
}

struct CompilerCallMetadata {
    callable_arguments: Vec<Option<CompilerCallableFacts>>,
    scope_arguments: Vec<Option<CompilerNamespaceFacts>>,
    scope_captures: Vec<CompilerContextCapture>,
    lexical_captures: Vec<CompilerContextCapture>,
    context_captures: Vec<CompilerContextCapture>,
}

#[derive(Clone)]
enum CompilerCallableFacts {
    Named {
        name: String,
        declarations: Vec<FunctionSource>,
        captures: Vec<CompilerContextCapture>,
    },
    Symbolic(CallableKind),
    Anonymous {
        parameters: Vec<AnonymousPattern>,
        body: Expression,
        captures: BTreeMap<String, BindingFacts>,
        static_context: bool,
        span: Span,
    },
}

impl CompilerDataMemberFacts {
    fn from_binding(facts: &BindingFacts, declaration_end: usize) -> Self {
        Self {
            storage_name: facts.storage_name.clone(),
            value_type: facts.value_type.clone(),
            int_range: facts.int_range.clone(),
            rational_value: facts.rational_value.clone(),
            declaration_end,
        }
    }
}

#[derive(Clone, Default)]
struct StaticValueFacts {
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
    string_value: Option<String>,
    record_fields: BTreeMap<String, Self>,
}

struct Analyzer {
    source: SourceText,
    language_version: LanguageVersion,
    language_features: Vec<String>,
    enums: EnumTypes,
    enum_alternatives: EnumAlternativeBindings,
    sums: SumTypes,
    sum_alternatives: SumAlternativeBindings,
    modulars: ModularTypes,
    interfaces: InterfaceTypes,
    interface_implementations: Vec<CompilerInterfaceImplementation>,
    functions: BTreeMap<String, Vec<FunctionSource>>,
    instances: Vec<CompilerFunction>,
    active_calls: Vec<String>,
    active_recursive_functions: BTreeMap<String, ActiveRecursiveFunction>,
    root_bindings: BTreeMap<String, CompilerDataMemberFacts>,
    anonymous_callables: BTreeMap<u32, CompilerCallableFacts>,
    anonymous_function_value_names: Vec<String>,
    constraints: Vec<CompilerConstraint>,
    constraint_bindings: BTreeMap<String, u32>,
    in_function: bool,
    function_values_used: bool,
    consumed_generators: BTreeSet<String>,
    generator_values: BTreeMap<String, CompilerExpression>,
    static_context: bool,
    next_instance: usize,
}

impl Analyzer {
    fn new(
        source: SourceText,
        language_version: LanguageVersion,
        enums: EnumTypes,
        enum_alternatives: EnumAlternativeBindings,
        sums: SumTypes,
        sum_alternatives: SumAlternativeBindings,
    ) -> Self {
        Self {
            source,
            language_version,
            language_features: Vec::new(),
            enums,
            enum_alternatives,
            sums,
            sum_alternatives,
            modulars: BTreeMap::new(),
            interfaces: BTreeMap::new(),
            interface_implementations: Vec::new(),
            functions: BTreeMap::new(),
            instances: Vec::new(),
            active_calls: Vec::new(),
            active_recursive_functions: BTreeMap::new(),
            root_bindings: BTreeMap::new(),
            anonymous_callables: BTreeMap::new(),
            anonymous_function_value_names: Vec::new(),
            constraints: Vec::new(),
            constraint_bindings: BTreeMap::new(),
            in_function: false,
            function_values_used: false,
            consumed_generators: BTreeSet::new(),
            generator_values: BTreeMap::new(),
            static_context: false,
            next_instance: 0,
        }
    }
}

/// Analyze the currently implemented native-compiler subset.
///
/// # Errors
///
/// Returns a shared source diagnostic for invalid or not-yet-supported input.
pub fn analyze_for_compiler(text: &str) -> Result<CompilerProgram, Diagnostic> {
    let source = SourceText::new(text).map_err(|error| {
        Diagnostic::error(error.code, 1, 1, error.message).with_source_span(error.span)
    })?;
    let parsed = parse(&source, &lex(&source));
    if let Some(error) = parsed.diagnostics.first() {
        return Err(source_diagnostic(
            &source,
            error.code,
            error.span,
            error.message.clone(),
        ));
    }
    let language_version = compiler_language_version(&source, &parsed.statements)?;
    reject_later_language_selections(&source, &parsed.statements)?;

    let (enums, enum_alternatives) = collect_enums(&source, &parsed.statements)?;
    let (sums, sum_alternatives) =
        collect_sums(&source, &parsed.statements, &enums, &enum_alternatives)?;
    let modular_sources = collect_modular_sources(
        &source,
        &parsed.statements,
        &enums,
        &enum_alternatives,
        &sums,
        &sum_alternatives,
    )?;
    let interface_sources = collect_interface_sources(
        &source,
        &parsed.statements,
        &enums,
        &enum_alternatives,
        &sums,
        &sum_alternatives,
        &modular_sources,
    )?;
    let reserved_names = compiler_reserved_names(
        &source,
        &enums,
        &enum_alternatives,
        &sums,
        &sum_alternatives,
        &modular_sources,
        &interface_sources,
    );
    let mut analyzer = Analyzer::new(
        source.clone(),
        language_version,
        enums,
        enum_alternatives,
        sums,
        sum_alternatives,
    );
    analyzer.install_modular_types(&modular_sources)?;
    analyzer.install_interfaces(&interface_sources)?;
    analyzer.validate_interface_implementations(&parsed.statements)?;
    collect_functions(
        &source,
        &parsed.statements,
        &reserved_names,
        &mut analyzer.functions,
    )?;
    let mut environment = BTreeMap::new();
    let main = analyzer.analyze_block(
        &parsed.statements,
        &mut environment,
        BlockKind::TopLevel,
        None,
    )?;
    require_runtime_main_result(&source, &main)?;
    let function_value_names = compiler_function_value_names(&mut analyzer);
    let interfaces = analyzer
        .interfaces
        .values()
        .map(|(interface, _)| interface.clone())
        .collect();
    Ok(CompilerProgram {
        source,
        language_version,
        main,
        function_value_names,
        constraints: analyzer.constraints,
        interfaces,
        interface_implementations: analyzer.interface_implementations,
        functions: analyzer.instances,
    })
}

fn compiler_language_version(
    source: &SourceText,
    statements: &[Statement],
) -> Result<LanguageVersion, Diagnostic> {
    let Some(Statement::LanguageSelection {
        version, features, ..
    }) = statements.first()
    else {
        return Err(source_diagnostic(
            source,
            "E-LANGUAGE-CONTEXT",
            Span::new(0, 0),
            "a source file begins with `use language ( version is v0.1 )`",
        ));
    };
    let language_version = source
        .slice(*version)
        .parse()
        .map_err(|message| source_diagnostic(source, "E-LANGUAGE-VERSION", *version, message))?;
    if language_version != LanguageVersion::DESIGN_0 || !features.is_empty() {
        return Err(source_diagnostic(
            source,
            "E-COMPILER-UNSUPPORTED",
            *version,
            "the native compiler increment supports language version v0.1 without optional features",
        ));
    }
    Ok(language_version)
}

fn reject_later_language_selections(
    source: &SourceText,
    statements: &[Statement],
) -> Result<(), Diagnostic> {
    let Some(Statement::LanguageSelection { span, .. }) = statements
        .iter()
        .skip(1)
        .find(|statement| matches!(statement, Statement::LanguageSelection { .. }))
    else {
        return Ok(());
    };
    Err(unsupported(
        source,
        *span,
        "language-context change after the bootstrap selection",
    ))
}

fn require_runtime_main_result(
    source: &SourceText,
    main: &CompilerBlock,
) -> Result<(), Diagnostic> {
    if compiler_type_contains_static_only(&main.result.value_type)
        && !matches!(main.result.kind, CompilerExpressionKind::Capability(_))
    {
        Err(unsupported(
            source,
            main.result.span,
            "runtime observation of a static compiler value",
        ))
    } else {
        Ok(())
    }
}

fn compiler_function_value_names(analyzer: &mut Analyzer) -> Vec<String> {
    if !analyzer.function_values_used {
        return Vec::new();
    }
    analyzer
        .functions
        .keys()
        .map(|name| format!("<fn {name}>"))
        .chain(["+".into(), "-".into(), "<=>".into()])
        .chain(std::mem::take(&mut analyzer.anonymous_function_value_names))
        .collect()
}

fn compiler_reserved_names(
    source: &SourceText,
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
    sums: &SumTypes,
    sum_alternatives: &SumAlternativeBindings,
    modulars: &[ModularSource],
    interfaces: &BTreeMap<String, InterfaceSource>,
) -> BTreeSet<String> {
    enums
        .keys()
        .chain(enum_alternatives.keys())
        .chain(sums.keys())
        .chain(sum_alternatives.keys())
        .cloned()
        .chain(
            modulars
                .iter()
                .map(|declaration| source.slice(declaration.name).to_owned()),
        )
        .chain(interfaces.keys().cloned())
        .chain(["root".to_owned()])
        .collect()
}

fn collect_enums(
    source: &SourceText,
    statements: &[Statement],
) -> Result<(EnumTypes, EnumAlternativeBindings), Diagnostic> {
    let mut enums = BTreeMap::new();
    let mut alternatives = BTreeMap::new();
    for statement in statements {
        let Some(declaration) = enum_declaration(source, statement) else {
            continue;
        };
        let EnumSource {
            name: name_span,
            alternatives: declarations,
            span: declaration_span,
        } = declaration;
        let name = source.slice(name_span).to_owned();
        if name == "root" || enums.contains_key(&name) || alternatives.contains_key(&name) {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                name_span,
                format!("`{name}` is already declared in this scope"),
            ));
        }
        let mut local = BTreeSet::new();
        for (label, alternative_span) in &declarations {
            if label == "root"
                || label == &name
                || !local.insert(label.clone())
                || enums.contains_key(label)
                || alternatives.contains_key(label)
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-ENUM-ALTERNATIVE",
                    *alternative_span,
                    format!("enum alternative `{label}` is already declared in this scope"),
                ));
            }
        }
        let enumeration = CompilerEnumType {
            name: name.clone(),
            alternatives: declarations.into_iter().map(|(label, _)| label).collect(),
        };
        enums.insert(name, (enumeration.clone(), declaration_span));
        for (index, label) in enumeration.alternatives.iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| {
                source_diagnostic(
                    source,
                    "E-COMPILER-UNSUPPORTED",
                    declaration_span,
                    "an Enum has more alternatives than the native tag can represent",
                )
            })?;
            alternatives.insert(
                label.clone(),
                (enumeration.clone(), index, declaration_span),
            );
        }
    }
    Ok((enums, alternatives))
}

fn enum_declaration(source: &SourceText, statement: &Statement) -> Option<EnumSource> {
    let Statement::Binding {
        name,
        classifier: None,
        value,
    } = (match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    })
    else {
        return None;
    };
    let Expression::Application { items, span } = value else {
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
    let alternatives = fields
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
        .collect::<Option<Vec<_>>>()?;
    Some(EnumSource {
        name: *name,
        alternatives,
        span: Span::new(name.start, span.end),
    })
}

fn sum_declaration(source: &SourceText, statement: &Statement) -> Option<SumSource> {
    let statement = match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    };
    if let Statement::Union {
        name,
        alternatives,
        span,
    } = statement
    {
        return Some(SumSource {
            name: *name,
            positional: false,
            alternatives: alternatives
                .iter()
                .map(|alternative| {
                    (
                        source.slice(alternative.name).to_owned(),
                        alternative.classifier,
                        alternative.name,
                    )
                })
                .collect(),
            span: *span,
        });
    }
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Variant" || fields.iter().any(|field| field.label.is_some()) {
        return None;
    }
    Some(SumSource {
        name: *name,
        positional: true,
        alternatives: fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                (
                    format!("at {index}"),
                    Some(field.value.span()),
                    field.value.span(),
                )
            })
            .collect(),
        span: Span::new(name.start, span.end),
    })
}

fn modular_declaration(source: &SourceText, statement: &Statement) -> Option<ModularSource> {
    let statement = match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    };
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(kind), range] = items.as_slice() else {
        return None;
    };
    let signed = match source.slice(*kind) {
        "ModNat" => false,
        "ModInt" => true,
        _ => return None,
    };
    Some(ModularSource {
        name: *name,
        signed,
        range: range.clone(),
        span: Span::new(name.start, span.end),
    })
}

fn interface_declaration(statement: &Statement) -> Option<InterfaceSource> {
    let statement = match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    };
    let Statement::Interface {
        name,
        functions,
        span,
    } = statement
    else {
        return None;
    };
    Some(InterfaceSource {
        name: *name,
        functions: functions.clone(),
        span: *span,
    })
}

#[allow(clippy::too_many_arguments)] // Every existing nominal root namespace is a collision boundary.
fn collect_interface_sources(
    source: &SourceText,
    statements: &[Statement],
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
    sums: &SumTypes,
    sum_alternatives: &SumAlternativeBindings,
    modulars: &[ModularSource],
) -> Result<BTreeMap<String, InterfaceSource>, Diagnostic> {
    let modular_names = modulars
        .iter()
        .map(|declaration| source.slice(declaration.name))
        .collect::<BTreeSet<_>>();
    let mut interfaces = BTreeMap::new();
    for statement in statements {
        let Some(declaration) = interface_declaration(statement) else {
            continue;
        };
        let name = source.slice(declaration.name).to_owned();
        if name == "root"
            || enums.contains_key(&name)
            || enum_alternatives.contains_key(&name)
            || sums.contains_key(&name)
            || sum_alternatives.contains_key(&name)
            || modular_names.contains(name.as_str())
            || interfaces.contains_key(&name)
        {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-DECLARATION",
                declaration.name,
                format!("`{name}` is already declared"),
            ));
        }
        let mut operations = BTreeSet::new();
        for function in &declaration.functions {
            let operation = source.slice(function.name);
            if !operations.insert(operation) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-INTERFACE-OPERATION",
                    function.name,
                    format!("interface operation `{operation}` is declared twice"),
                ));
            }
        }
        interfaces.insert(name, declaration);
    }
    Ok(interfaces)
}

fn collect_modular_sources(
    source: &SourceText,
    statements: &[Statement],
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
    sums: &SumTypes,
    sum_alternatives: &SumAlternativeBindings,
) -> Result<Vec<ModularSource>, Diagnostic> {
    let mut declarations = Vec::new();
    let mut names = BTreeSet::new();
    for statement in statements {
        let Some(mut declaration) = modular_declaration(source, statement) else {
            continue;
        };
        let name = source.slice(declaration.name);
        if name == "root"
            || enums.contains_key(name)
            || enum_alternatives.contains_key(name)
            || sums.contains_key(name)
            || sum_alternatives.contains_key(name)
            || !names.insert(name.to_owned())
        {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                declaration.name,
                format!("`{name}` is already declared in this scope"),
            ));
        }
        declaration.range = resolve_modular_range_source(
            source,
            statements,
            &declaration.range,
            declaration.name.start,
        );
        declarations.push(declaration);
    }
    Ok(declarations)
}

fn resolve_modular_range_source(
    source: &SourceText,
    statements: &[Statement],
    range: &Expression,
    mut before: usize,
) -> Expression {
    let mut range = range.clone();
    let mut seen = BTreeSet::new();
    while let Expression::Identifier(name) = &range {
        let name = source.slice(*name);
        if !seen.insert(name.to_owned()) {
            break;
        }
        let Some((value, declaration_start)) = statements.iter().rev().find_map(|statement| {
            let statement = match statement {
                Statement::Published { declaration, .. } => declaration.as_ref(),
                statement => statement,
            };
            let Statement::Binding {
                name: candidate,
                value,
                ..
            } = statement
            else {
                return None;
            };
            (candidate.start < before && source.slice(*candidate) == name)
                .then_some((value.clone(), candidate.start))
        }) else {
            break;
        };
        range = value;
        before = declaration_start;
    }
    range
}

fn collect_sums(
    source: &SourceText,
    statements: &[Statement],
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
) -> Result<(SumTypes, SumAlternativeBindings), Diagnostic> {
    let mut sums: SumTypes = BTreeMap::new();
    let mut alternatives: SumAlternativeBindings = BTreeMap::new();
    for statement in statements {
        let Some(declaration) = sum_declaration(source, statement) else {
            continue;
        };
        let name = source.slice(declaration.name).to_owned();
        if name == "root"
            || enums.contains_key(&name)
            || enum_alternatives.contains_key(&name)
            || sums.contains_key(&name)
            || alternatives.contains_key(&name)
        {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-UNION",
                declaration.name,
                format!("`{name}` is already declared in this scope"),
            ));
        }
        let mut local = BTreeSet::new();
        let mut lowered = Vec::with_capacity(declaration.alternatives.len());
        for (label, classifier, alternative_span) in &declaration.alternatives {
            if !local.insert(label.clone()) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-UNION-ALTERNATIVE",
                    *alternative_span,
                    format!("sum alternative `{label}` occurs more than once"),
                ));
            }
            if !declaration.positional
                && (label == "root"
                    || label == &name
                    || enums.contains_key(label)
                    || enum_alternatives.contains_key(label)
                    || sums.contains_key(label)
                    || alternatives.contains_key(label))
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-UNION-ALTERNATIVE",
                    *alternative_span,
                    format!("sum alternative `{label}` is already declared in this scope"),
                ));
            }
            let payload = classifier
                .map(|classifier| {
                    let classifier_text = compact_classifier(source.slice(classifier));
                    enums
                        .get(&classifier_text)
                        .filter(|(_, declaration)| declaration.end <= classifier.start)
                        .map(|(enumeration, _)| CompilerType::Enum(enumeration.clone()))
                        .or_else(|| {
                            sums.get(&classifier_text)
                                .filter(|(_, declaration)| declaration.end <= classifier.start)
                                .map(|(sum, _)| CompilerType::Sum(sum.clone()))
                        })
                        .or_else(|| parse_compact_classifier(&classifier_text))
                        .ok_or_else(|| unsupported(source, classifier, "sum payload classifier"))
                })
                .transpose()?;
            if payload
                .as_ref()
                .is_some_and(|payload| !compiler_function_result_supported(payload))
            {
                return Err(unsupported(
                    source,
                    classifier.expect("unsupported payload has a classifier"),
                    "sum payload without an admitted private representation",
                ));
            }
            lowered.push(CompilerSumAlternative {
                name: label.clone(),
                payload,
            });
        }
        let sum = CompilerSumType {
            name: name.clone(),
            positional: declaration.positional,
            alternatives: lowered,
        };
        if !sum.positional {
            for (index, alternative) in sum.alternatives.iter().enumerate() {
                let index = u32::try_from(index).map_err(|_| {
                    unsupported(source, declaration.span, "native Union alternative tag")
                })?;
                alternatives.insert(
                    alternative.name.clone(),
                    (sum.clone(), index, declaration.span),
                );
            }
        }
        sums.insert(name, (sum, declaration.span));
    }
    Ok((sums, alternatives))
}

fn constraint_definition<'a>(
    source: &SourceText,
    expression: &'a Expression,
) -> Option<(Span, &'a [AnonymousPattern], &'a Expression, Span)> {
    let Expression::Application { items, span } = expression else {
        return None;
    };
    let [
        Expression::Identifier(base),
        Expression::Identifier(operation),
        Expression::AnonymousFunction {
            parameters, body, ..
        },
    ] = items.as_slice()
    else {
        return None;
    };
    (source.slice(*operation) == "constraint").then_some((
        *base,
        parameters.as_slice(),
        body.as_ref(),
        *span,
    ))
}

fn collect_functions(
    source: &SourceText,
    statements: &[Statement],
    reserved_names: &BTreeSet<String>,
    functions: &mut BTreeMap<String, Vec<FunctionSource>>,
) -> Result<(), Diagnostic> {
    for statement in statements {
        if let Statement::InterfaceImplementation { declarations, .. } = statement {
            collect_functions(source, declarations, reserved_names, functions)?;
            continue;
        }
        let declaration = match statement {
            Statement::Function {
                name,
                parameters,
                result,
                body,
                span,
                is_static,
                effect_bound,
                clauses,
            } if **clauses == FunctionClauses::default() => Some(FunctionSource {
                name: *name,
                parameters: parameters.clone(),
                result: *result,
                effect_bound: *effect_bound,
                declared_effects: None,
                body: body.clone(),
                span: *span,
                is_static: *is_static,
            }),
            Statement::Published { declaration, .. } => {
                if let Statement::Function {
                    name,
                    parameters,
                    result,
                    body,
                    span,
                    is_static,
                    effect_bound,
                    clauses,
                } = declaration.as_ref()
                    && **clauses == FunctionClauses::default()
                {
                    Some(FunctionSource {
                        name: *name,
                        parameters: parameters.clone(),
                        result: *result,
                        effect_bound: *effect_bound,
                        declared_effects: None,
                        body: body.clone(),
                        span: *span,
                        is_static: *is_static,
                    })
                } else if interface_declaration(statement).is_some()
                    || enum_declaration(source, statement).is_some()
                    || sum_declaration(source, statement).is_some()
                    || matches!(declaration.as_ref(), Statement::Binding { .. })
                {
                    None
                } else {
                    return Err(unsupported(
                        source,
                        statement_span(statement),
                        "published declaration",
                    ));
                }
            }
            Statement::Function { .. } => {
                return Err(unsupported(
                    source,
                    statement_span(statement),
                    "constrained or effectful function",
                ));
            }
            _ => None,
        };
        if let Some(mut function) = declaration {
            function.declared_effects =
                compiler_declared_effect_row(source, function.effect_bound)?;
            let name = source.slice(function.name).to_owned();
            if reserved_names.contains(&name) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-BINDING",
                    function.name,
                    format!("`{name}` is already declared in this scope"),
                ));
            }
            let overloads = functions.entry(name.clone()).or_default();
            if overloads
                .iter()
                .any(|candidate| same_function_input_header(source, candidate, &function))
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-FUNCTION-OVERLOAD",
                    statement_span(statement),
                    format!("function `{name}` repeats an input signature and staticness"),
                ));
            }
            overloads.push(function);
        }
    }
    Ok(())
}

fn compiler_declared_effect_row(
    source: &SourceText,
    effect_bound: Option<Span>,
) -> Result<Option<CompilerEffectRow>, Diagnostic> {
    let Some(effect_bound) = effect_bound else {
        return Ok(None);
    };
    let text = source.slice(effect_bound);
    if compact_classifier(text) == "Effects()" {
        return Ok(Some(CompilerEffectRow {
            identities: Vec::new(),
        }));
    }
    if explicit_single_measure(text).is_some() {
        return Ok(None);
    }
    Err(unsupported(
        source,
        effect_bound,
        "nonempty or polymorphic function effect bound",
    ))
}

fn same_function_input_header(
    source: &SourceText,
    left: &FunctionSource,
    right: &FunctionSource,
) -> bool {
    left.is_static == right.is_static
        && left.parameters.len() == right.parameters.len()
        && left
            .parameters
            .iter()
            .zip(&right.parameters)
            .all(|(left, right)| {
                compact_classifier(source.slice(left.classifier))
                    == compact_classifier(source.slice(right.classifier))
            })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BlockKind {
    TopLevel,
    Function,
    Lexical,
}

impl Analyzer {
    fn install_modular_types(&mut self, declarations: &[ModularSource]) -> Result<(), Diagnostic> {
        let environment = BTreeMap::new();
        for declaration in declarations {
            let range = self.analyze_expression(&declaration.range, &environment)?;
            let CompilerExpressionKind::Binary {
                operation: CompilerBinary::RangeInclusive,
                left,
                right,
            } = &range.kind
            else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-MODULAR-RANGE",
                    declaration.range.span(),
                    "ModNat and ModInt require a finite inclusive Int range",
                ));
            };
            if left.value_type != CompilerType::Int || right.value_type != CompilerType::Int {
                return Err(source_diagnostic(
                    &self.source,
                    "E-MODULAR-RANGE",
                    declaration.range.span(),
                    "ModNat and ModInt require a finite inclusive Int range",
                ));
            }
            let Some(lower) = exact_int(left) else {
                return Err(unsupported(
                    &self.source,
                    left.span,
                    "dynamic modular lower bound",
                ));
            };
            let Some(upper) = exact_int(right) else {
                return Err(unsupported(
                    &self.source,
                    right.span,
                    "dynamic modular upper bound",
                ));
            };
            if lower > BigInt::from(0)
                || upper < BigInt::from(0)
                || (!declaration.signed && lower != BigInt::from(0))
            {
                return Err(source_diagnostic(
                    &self.source,
                    "E-MODULAR-RANGE",
                    declaration.range.span(),
                    "modular range must contain zero and ModNat must begin at zero",
                ));
            }
            let name = self.source.slice(declaration.name).to_owned();
            self.modulars.insert(
                name.clone(),
                (
                    CompilerModularType {
                        name,
                        signed: declaration.signed,
                        lower,
                        upper,
                    },
                    declaration.span,
                ),
            );
        }
        Ok(())
    }

    fn interface_operation(
        &self,
        name: Span,
        parameters: &[FunctionParameter],
        result: Span,
        clauses: &FunctionClauses,
        span: Span,
    ) -> Result<CompilerInterfaceOperation, Diagnostic> {
        if clauses != &FunctionClauses::default() {
            return Err(unsupported(
                &self.source,
                span,
                "v0.2 interface contract clauses in a v0.1 compilation",
            ));
        }
        let parameters = parameters
            .iter()
            .map(|parameter| {
                if parameter.qualifier.is_some()
                    || parameter.default.is_some()
                    || !parameter.fields.is_empty()
                {
                    return Err(unsupported(
                        &self.source,
                        parameter.name,
                        "packaged, defaulted, or qualified interface parameter",
                    ));
                }
                let parameter_type = self.parse_classifier(parameter.classifier)?;
                if !compiler_function_parameter_supported(&parameter_type)
                    || compiler_type_contains_static_only(&parameter_type)
                {
                    return Err(unsupported(
                        &self.source,
                        parameter.classifier,
                        "interface parameter classifier without an admitted native function ABI",
                    ));
                }
                Ok(parameter_type)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result_type = self.parse_classifier(result)?;
        if !compiler_function_result_supported(&result_type)
            || compiler_type_contains_static_only(&result_type)
        {
            return Err(unsupported(
                &self.source,
                result,
                "interface result classifier without an admitted native function ABI",
            ));
        }
        Ok(CompilerInterfaceOperation {
            name: self.source.slice(name).to_owned(),
            parameters,
            result: result_type,
        })
    }

    fn install_interfaces(
        &mut self,
        declarations: &BTreeMap<String, InterfaceSource>,
    ) -> Result<(), Diagnostic> {
        for (name, declaration) in declarations {
            let mut operations = declaration
                .functions
                .iter()
                .map(|function| {
                    self.interface_operation(
                        function.name,
                        &function.parameters,
                        function.result,
                        &function.clauses,
                        function.span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            operations.sort_by(|left, right| left.name.cmp(&right.name));
            self.interfaces.insert(
                name.clone(),
                (
                    CompilerInterface {
                        identity: format!("root.{name}"),
                        operations,
                    },
                    declaration.span,
                ),
            );
        }
        Ok(())
    }

    fn validate_interface_implementations(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), Diagnostic> {
        for statement in statements {
            let Statement::InterfaceImplementation {
                interface,
                declarations,
                span,
            } = statement
            else {
                continue;
            };
            let implementation =
                self.validate_interface_implementation(*interface, declarations, *span)?;
            self.interface_implementations.push(implementation);
        }
        Ok(())
    }

    fn validate_interface_implementation(
        &self,
        interface: Span,
        declarations: &[Statement],
        span: Span,
    ) -> Result<CompilerInterfaceImplementation, Diagnostic> {
        let interface_name = self.source.slice(interface);
        let (shape, _) = self
            .interfaces
            .get(interface_name)
            .filter(|(_, declaration_span)| declaration_span.end <= interface.start)
            .ok_or_else(|| {
                source_diagnostic(
                    &self.source,
                    "E-UNKNOWN-INTERFACE",
                    interface,
                    format!("`{interface_name}` is not a declared interface"),
                )
            })?;
        let mut supplied = BTreeMap::new();
        for declaration in declarations {
            let Statement::Function {
                name,
                is_static,
                parameters,
                result,
                effect_bound,
                clauses,
                span: function_span,
                ..
            } = declaration
            else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-INTERFACE-IMPLEMENTATION",
                    span,
                    "an interface implementation contains function declarations only",
                ));
            };
            if *is_static {
                return Err(unsupported(
                    &self.source,
                    *function_span,
                    "static interface implementation function",
                ));
            }
            let declared_effects = compiler_declared_effect_row(&self.source, *effect_bound)?;
            let actual =
                self.interface_operation(*name, parameters, *result, clauses, *function_span)?;
            let operation_name = actual.name.clone();
            if supplied.contains_key(&operation_name) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-INTERFACE-IMPLEMENTATION",
                    *name,
                    format!("interface operation `{operation_name}` is implemented more than once"),
                ));
            }
            let inputs = parameters
                .iter()
                .map(|parameter| compact_classifier(self.source.slice(parameter.classifier)))
                .collect::<Vec<_>>()
                .join(",");
            supplied.insert(
                operation_name,
                (
                    actual,
                    CompilerInterfaceOperationEvidence {
                        role: self.source.slice(*name).to_owned(),
                        declaration_identity: format!(
                            "root.{}:ordinary({inputs})",
                            self.source.slice(*name)
                        ),
                        declared_effects,
                    },
                ),
            );
        }
        if supplied.len() != shape.operations.len()
            || shape.operations.iter().any(|expected| {
                supplied
                    .get(&expected.name)
                    .is_none_or(|(actual, _)| actual != expected)
            })
        {
            return Err(source_diagnostic(
                &self.source,
                "E-INTERFACE-IMPLEMENTATION",
                span,
                "implementation operations must exactly match the interface shapes",
            ));
        }
        Ok(CompilerInterfaceImplementation {
            interface_identity: shape.identity.clone(),
            operations: supplied
                .into_values()
                .map(|(_, evidence)| evidence)
                .collect(),
        })
    }

    fn analyze_expression_with_expected(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Identifier(name) = expression
            && self.source.slice(*name) == "None"
            && let Some(CompilerType::Optional(payload)) = expected
        {
            return self.finish_optional_none(payload.as_ref().clone(), expression.span());
        }
        if let Some(CompilerType::List(element)) = expected {
            if let Expression::Identifier(name) = expression
                && self.source.slice(*name) == "Empty"
            {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::ListEmpty,
                    value_type: CompilerType::List(element.clone()),
                    int_range: None,
                    rational_value: None,
                    span: expression.span(),
                });
            }
            if let Expression::Application { items, span } = expression
                && let [
                    Expression::Identifier(constructor),
                    Expression::Product { fields, .. },
                ] = items.as_slice()
                && self.source.slice(*constructor) == "Entry"
                && let [value, remaining] = fields.as_slice()
                && value.label.is_none()
                && remaining.label.is_none()
            {
                let value = self.analyze_expression_with_expected(
                    &value.value,
                    environment,
                    Some(element),
                )?;
                require_same_type(&self.source, value.span, element, &value.value_type)?;
                let list_type = CompilerType::List(element.clone());
                let remaining = self.analyze_expression_with_expected(
                    &remaining.value,
                    environment,
                    Some(&list_type),
                )?;
                require_same_type(
                    &self.source,
                    remaining.span,
                    &list_type,
                    &remaining.value_type,
                )?;
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::ListEntry {
                        value: Box::new(value),
                        remaining: Box::new(remaining),
                    },
                    value_type: list_type,
                    int_range: None,
                    rational_value: None,
                    span: *span,
                });
            }
        }
        let value = self.analyze_expression(expression, environment)?;
        match expected {
            Some(CompilerType::Character) if value.value_type == CompilerType::String => {
                self.finish_character_conversion(value, expression.span())
            }
            Some(CompilerType::String) if value.value_type == CompilerType::Character => {
                Ok(forget_character_evidence(value))
            }
            _ => Ok(value),
        }
    }

    fn parse_classifier(&self, span: Span) -> Result<CompilerType, Diagnostic> {
        let classifier = compact_classifier(self.source.slice(span));
        if let Some((enumeration, declaration)) = self.enums.get(&classifier)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Enum(enumeration.clone()));
        }
        if let Some((sum, declaration)) = self.sums.get(&classifier)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Sum(sum.clone()));
        }
        if let Some((modular, declaration)) = self.modulars.get(&classifier)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Modular(modular.clone()));
        }
        if let Some((success, codes)) = classifier
            .strip_prefix("Result(")
            .and_then(|value| value.strip_suffix(')'))
            .and_then(split_classifier_once)
            && codes == "langarithmeticArithmeticErrorCode"
            && let Some((modular, declaration)) = self.modulars.get(success)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Result(Box::new(CompilerType::Modular(
                modular.clone(),
            ))));
        }
        if let Some(tag) = self.constraint_bindings.get(&classifier)
            && let Some(constraint) = self
                .constraints
                .get(usize::try_from(*tag).expect("u32 tag fits usize"))
            && constraint.span.end <= span.start
        {
            return Ok(CompilerType::Refined {
                constraint: classifier,
                base: Box::new(constraint.base_type.clone()),
            });
        }
        parse_compact_classifier(&classifier)
            .ok_or_else(|| unsupported(&self.source, span, "classifier"))
    }

    fn is_declaration(&self, statement: &Statement) -> bool {
        if matches!(
            statement,
            Statement::LanguageSelection { .. }
                | Statement::Function { .. }
                | Statement::Interface { .. }
                | Statement::InterfaceImplementation { .. }
        ) || matches!(statement, Statement::Published { declaration, .. } if matches!(declaration.as_ref(), Statement::Function { .. } | Statement::Interface { .. }))
        {
            return true;
        }
        if let Some(declaration) = enum_declaration(&self.source, statement) {
            let name = self.source.slice(declaration.name);
            return self
                .enums
                .get(name)
                .is_some_and(|(_, span)| *span == declaration.span);
        }
        if let Some(declaration) = sum_declaration(&self.source, statement) {
            let name = self.source.slice(declaration.name);
            return self
                .sums
                .get(name)
                .is_some_and(|(_, span)| *span == declaration.span);
        }
        if let Some(declaration) = modular_declaration(&self.source, statement) {
            let name = self.source.slice(declaration.name);
            return self
                .modulars
                .get(name)
                .is_some_and(|(_, span)| *span == declaration.span);
        }
        false
    }

    #[allow(clippy::too_many_lines)] // Capture validation keeps every nested-function boundary explicit.
    fn bind_nested_function(
        &self,
        statement: &Statement,
        environment: &mut BTreeMap<String, BindingFacts>,
        declared: &mut BTreeSet<String>,
    ) -> Result<(), Diagnostic> {
        let Statement::Function {
            name,
            is_static,
            parameters,
            result,
            effect_bound,
            clauses,
            body,
            span,
        } = statement
        else {
            return Err(unsupported(
                &self.source,
                statement_span(statement),
                "published nested function declaration",
            ));
        };
        if self.static_context
            || *is_static
            || effect_bound.is_some()
            || **clauses != FunctionClauses::default()
        {
            return Err(unsupported(
                &self.source,
                *span,
                "static, measured, constrained, or effectful nested function",
            ));
        }
        let name_text = self.source.slice(*name).to_owned();
        if declared.contains(&name_text)
            || self.functions.contains_key(&name_text)
            || self.active_calls.iter().any(|identity| {
                identity
                    .split_once(':')
                    .is_some_and(|(name, _)| name == name_text)
            })
        {
            return Err(source_diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                *name,
                format!("`{name_text}` is already declared in this invocation scope"),
            ));
        }
        let parameter_names = parameters
            .iter()
            .map(|parameter| self.source.slice(parameter.name))
            .collect::<BTreeSet<_>>();
        let captures = environment
            .iter()
            .filter(|(candidate, facts)| {
                facts.runtime_bound
                    && !candidate.starts_with("@ ")
                    && !parameter_names.contains(candidate.as_str())
                    && compiler_function_result_supported(&facts.value_type)
            })
            .map(|(candidate, facts)| CompilerContextCapture {
                parameter_name: candidate.clone(),
                value_type: facts.value_type.clone(),
                int_range: facts.int_range.clone(),
                rational_value: facts.rational_value.clone(),
                argument: CompilerExpression {
                    kind: CompilerExpressionKind::Local(facts.storage_name.clone()),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    span: *name,
                },
                span: *name,
            })
            .collect();
        let declaration = FunctionSource {
            name: *name,
            parameters: parameters.clone(),
            result: *result,
            effect_bound: None,
            declared_effects: None,
            body: body.clone(),
            span: *span,
            is_static: false,
        };
        environment.insert(
            name_text.clone(),
            BindingFacts {
                storage_name: format!("topal.nested.function.{}.{}", name.start, name_text),
                runtime_bound: false,
                value_type: CompilerType::Function,
                int_range: None,
                rational_value: None,
                string_value: None,
                closed_int_range: None,
                record_fields: BTreeMap::new(),
                namespace: None,
                callable: Some(CompilerCallableFacts::Named {
                    name: name_text.clone(),
                    declarations: vec![declaration],
                    captures,
                }),
                static_capability: None,
            },
        );
        declared.insert(name_text);
        Ok(())
    }

    #[allow(clippy::too_many_lines)] // Exhaustive statement admission keeps the subset boundary visible.
    fn analyze_block(
        &mut self,
        statements: &[Statement],
        environment: &mut BTreeMap<String, BindingFacts>,
        kind: BlockKind,
        enclosing_result: Option<&CompilerType>,
    ) -> Result<CompilerBlock, Diagnostic> {
        let mut lowered = Vec::new();
        let mut generator_bindings = Vec::new();
        let mut result = None;
        let mut declared = if kind == BlockKind::Lexical {
            BTreeSet::new()
        } else {
            environment.keys().cloned().collect()
        };
        if kind != BlockKind::TopLevel
            && let Some(statement) = statements.iter().find(|statement| {
                interface_declaration(statement).is_some()
                    || matches!(statement, Statement::InterfaceImplementation { .. })
            })
        {
            return Err(unsupported(
                &self.source,
                statement_span(statement),
                "non-root Interface declaration or implementation",
            ));
        }
        if kind == BlockKind::Lexical
            && let Some(declaration) = statements
                .iter()
                .find(|statement| self.is_declaration(statement))
        {
            return Err(unsupported(
                &self.source,
                statement_span(declaration),
                "nested declaration",
            ));
        }
        let executable = statements
            .iter()
            .filter(|statement| {
                !self.is_declaration(statement)
                    || (kind == BlockKind::Function
                        && (matches!(statement, Statement::Function { .. })
                            || matches!(
                                statement,
                                Statement::Published { declaration, .. }
                                    if matches!(declaration.as_ref(), Statement::Function { .. })
                            )))
            })
            .collect::<Vec<_>>();

        for (index, statement) in executable.iter().enumerate() {
            let last = index + 1 == executable.len();
            let statement = match *statement {
                Statement::Published { declaration, .. }
                    if matches!(declaration.as_ref(), Statement::Binding { .. }) =>
                {
                    declaration.as_ref()
                }
                statement => statement,
            };
            match statement {
                Statement::DiagnosticControl { .. } => {
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Function { .. } if kind == BlockKind::Function => {
                    self.bind_nested_function(statement, environment, &mut declared)?;
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Published { declaration, .. }
                    if kind == BlockKind::Function
                        && matches!(declaration.as_ref(), Statement::Function { .. }) =>
                {
                    self.bind_nested_function(statement, environment, &mut declared)?;
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Binding {
                    name,
                    classifier,
                    value: initializer,
                } => {
                    let name_text = self.source.slice(*name).to_owned();
                    if declared.contains(&name_text)
                        || (kind == BlockKind::TopLevel
                            && (name_text == "root"
                                || self.functions.contains_key(&name_text)
                                || self.enums.contains_key(&name_text)
                                || self.enum_alternatives.contains_key(&name_text)
                                || self.sums.contains_key(&name_text)
                                || self.sum_alternatives.contains_key(&name_text)
                                || self.modulars.contains_key(&name_text)
                                || self.interfaces.contains_key(&name_text)))
                    {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            *name,
                            format!("`{name_text}` is already declared in this scope"),
                        ));
                    }
                    let expected = classifier
                        .map(|classifier| self.parse_classifier(classifier))
                        .transpose()?;
                    let mut value = if constraint_definition(&self.source, initializer).is_some() {
                        self.analyze_constraint_definition(&name_text, initializer, environment)?
                    } else {
                        self.analyze_expression_with_expected(
                            initializer,
                            environment,
                            expected.as_ref(),
                        )?
                    };
                    if let (Some(classifier), Some(expected)) = (classifier, expected) {
                        if expected == CompilerType::Int
                            && value.value_type == CompilerType::Rational
                        {
                            let span = value.span;
                            value = self.finish_int_conversion(value, span, span)?;
                        } else if expected == CompilerType::Nat
                            && value.value_type == CompilerType::Int
                        {
                            let span = value.span;
                            value = self.finish_nat_conversion(value, span, span)?;
                        }
                        if let CompilerType::Result(success) = &value.value_type
                            && success.as_ref() == &expected
                        {
                            if !matches!(enclosing_result, Some(CompilerType::Result(_))) {
                                return Err(source_diagnostic(
                                    &self.source,
                                    "E-RESULT-PROJECTION-CONTEXT",
                                    value.span,
                                    "Result success projection requires an enclosing compatible Result function",
                                ));
                            }
                            let span = value.span;
                            value = CompilerExpression {
                                kind: CompilerExpressionKind::ResultProject(Box::new(value)),
                                value_type: expected.clone(),
                                int_range: None,
                                rational_value: None,
                                span,
                            };
                        }
                        require_same_type(&self.source, *classifier, &expected, &value.value_type)?;
                    }
                    reject_static_value_containment(&self.source, &value)?;
                    let constraint_tag = if value.value_type == CompilerType::Constraint {
                        if kind != BlockKind::TopLevel {
                            return Err(unsupported(
                                &self.source,
                                *name,
                                "non-root Constraint binding",
                            ));
                        }
                        let tag = if constraint_definition(&self.source, initializer).is_some() {
                            let CompilerExpressionKind::ConstraintValue(tag) = &value.kind else {
                                unreachable!("checked constraint construction has an identity tag")
                            };
                            *tag
                        } else {
                            let Expression::Identifier(source_name_span) = initializer else {
                                return Err(unsupported(
                                    &self.source,
                                    initializer.span(),
                                    "Constraint value expression",
                                ));
                            };
                            let source_name = self.source.slice(*source_name_span);
                            let source_tag =
                                *self.constraint_bindings.get(source_name).ok_or_else(|| {
                                    unsupported(
                                        &self.source,
                                        *source_name_span,
                                        "Constraint value without retained static metadata",
                                    )
                                })?;
                            let mut constraint = self.constraints
                                [usize::try_from(source_tag).expect("u32 tag fits usize")]
                            .clone();
                            constraint.name.clone_from(&name_text);
                            constraint.span = Span::new(name.start, initializer.span().end);
                            let tag = u32::try_from(self.constraints.len()).map_err(|_| {
                                unsupported(&self.source, *name, "native Constraint value tag")
                            })?;
                            self.constraints.push(constraint);
                            value.kind = CompilerExpressionKind::ConstraintValue(tag);
                            tag
                        };
                        Some(tag)
                    } else {
                        None
                    };
                    let string_value = Self::known_string_value(&value, environment);
                    let closed_int_range = Self::known_closed_int_range(&value, environment);
                    let record_fields = Self::known_record_fields(&value, environment);
                    let namespace =
                        self.known_namespace(&value, environment, initializer.span().start, kind)?;
                    let callable =
                        self.known_callable(&value, environment, initializer.span().start)?;
                    let static_capability = match &value.kind {
                        CompilerExpressionKind::Capability(capability) => Some(capability.clone()),
                        _ => None,
                    };
                    if static_capability.is_some() && kind != BlockKind::TopLevel {
                        return Err(unsupported(
                            &self.source,
                            *name,
                            "non-root Capability binding",
                        ));
                    }
                    let storage_name = if kind == BlockKind::TopLevel {
                        format!("topal.root.{}.{}", name.start, name_text)
                    } else if matches!(value.value_type, CompilerType::Generator(_)) {
                        format!("topal.generator.{}.{}", name.start, name_text)
                    } else {
                        name_text.clone()
                    };
                    let generator_value = if matches!(value.value_type, CompilerType::Generator(_))
                    {
                        match &value.kind {
                            CompilerExpressionKind::Local(storage_name) => {
                                self.generator_values.get(storage_name).cloned()
                            }
                            CompilerExpressionKind::IterateGenerator { .. }
                            | CompilerExpressionKind::GeneratorTakeWhile { .. }
                            | CompilerExpressionKind::UnfoldGenerator { .. }
                            | CompilerExpressionKind::StringCharactersGenerator { .. } => {
                                Some(value.clone())
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let facts = BindingFacts {
                        storage_name: storage_name.clone(),
                        runtime_bound: !compiler_type_contains_static_only(&value.value_type),
                        value_type: value.value_type.clone(),
                        int_range: value.int_range.clone(),
                        rational_value: value.rational_value.clone(),
                        string_value,
                        closed_int_range,
                        record_fields,
                        namespace,
                        callable,
                        static_capability,
                    };
                    if matches!(facts.value_type, CompilerType::Generator(_)) {
                        if let Some(generator_value) = generator_value {
                            self.generator_values
                                .insert(storage_name.clone(), generator_value);
                        }
                        generator_bindings.push((storage_name.clone(), name_text.clone(), *name));
                    }
                    environment.insert(name_text.clone(), facts.clone());
                    if let Some(tag) = constraint_tag {
                        self.constraint_bindings.insert(name_text.clone(), tag);
                    }
                    if kind == BlockKind::TopLevel && facts.runtime_bound {
                        self.root_bindings.insert(
                            name_text.clone(),
                            CompilerDataMemberFacts::from_binding(
                                &facts,
                                statement_span(statement).end,
                            ),
                        );
                    }
                    declared.insert(name_text.clone());
                    lowered.push(CompilerStatement::Binding(CompilerBinding {
                        name: name_text,
                        storage_name,
                        span: *name,
                        value,
                    }));
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Foreach {
                    result: foreach_result,
                    source,
                    binding,
                    body,
                    span,
                } if kind == BlockKind::TopLevel
                    || (kind == BlockKind::Function
                        && foreach_result.is_none()
                        && matches!(source, Expression::Identifier(name)
                            if environment.get(self.source.slice(*name)).is_some_and(|facts|
                                facts.storage_name == self.source.slice(*name)
                                    && is_character_unit_generator_type(&facts.value_type)))) =>
                {
                    let value =
                        self.analyze_root_foreach(source, *binding, body, *span, environment)?;
                    if let Some((name, classifier)) = foreach_result {
                        let name_text = self.source.slice(*name).to_owned();
                        if declared.contains(&name_text)
                            || name_text == "root"
                            || self.functions.contains_key(&name_text)
                            || self.enums.contains_key(&name_text)
                            || self.enum_alternatives.contains_key(&name_text)
                            || self.sums.contains_key(&name_text)
                            || self.sum_alternatives.contains_key(&name_text)
                            || self.modulars.contains_key(&name_text)
                            || self.interfaces.contains_key(&name_text)
                        {
                            return Err(source_diagnostic(
                                &self.source,
                                "E-DUPLICATE-BINDING",
                                *name,
                                format!("`{name_text}` is already declared in this scope"),
                            ));
                        }
                        if let Some(classifier) = classifier {
                            let expected = self.parse_classifier(*classifier)?;
                            require_same_type(
                                &self.source,
                                *classifier,
                                &expected,
                                &CompilerType::Unit,
                            )?;
                        }
                        let storage_name = format!("topal.root.{}.{}", name.start, name_text);
                        let facts = BindingFacts {
                            storage_name: storage_name.clone(),
                            runtime_bound: true,
                            value_type: CompilerType::Unit,
                            int_range: None,
                            rational_value: None,
                            string_value: None,
                            closed_int_range: None,
                            record_fields: BTreeMap::new(),
                            namespace: None,
                            callable: None,
                            static_capability: None,
                        };
                        environment.insert(name_text.clone(), facts.clone());
                        self.root_bindings.insert(
                            name_text.clone(),
                            CompilerDataMemberFacts::from_binding(&facts, span.end),
                        );
                        declared.insert(name_text.clone());
                        lowered.push(CompilerStatement::Binding(CompilerBinding {
                            name: name_text,
                            storage_name,
                            value,
                            span: *name,
                        }));
                    } else {
                        lowered.push(CompilerStatement::Discard(value));
                    }
                    if last {
                        result = Some(unit_expression(*span));
                    }
                }
                Statement::Discard { value, .. } => {
                    let value = self.analyze_expression(value, environment)?;
                    if matches!(value.value_type, CompilerType::Generator(_)) {
                        return Err(unsupported(
                            &self.source,
                            value.span,
                            "generator abandonment and close delivery",
                        ));
                    }
                    reject_static_value_containment(&self.source, &value)?;
                    lowered.push(CompilerStatement::Discard(value));
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Expression(expression) if last => {
                    result = Some(self.analyze_expression_with_expected(
                        expression,
                        environment,
                        enclosing_result,
                    )?);
                }
                Statement::Expression(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-NONFINAL-VALUE",
                        statement_span(statement),
                        "a non-final value expression must be explicitly discarded or bound",
                    ));
                }
                Statement::Return { value, .. } if kind == BlockKind::Function => {
                    result = Some(self.analyze_expression_with_expected(
                        value,
                        environment,
                        enclosing_result,
                    )?);
                    break;
                }
                Statement::Return { .. } if kind == BlockKind::TopLevel => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-RETURN-OUTSIDE-FUNCTION",
                        statement_span(statement),
                        "`return` is available only inside a function",
                    ));
                }
                Statement::Return { .. } if kind == BlockKind::Lexical => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "return through a nested lexical block",
                    ));
                }
                Statement::LibrarySelection { .. } => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "source library dependency",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "statement form",
                    ));
                }
            }
        }
        if let Some((_, name, span)) = generator_bindings
            .iter()
            .find(|(storage, _, _)| !self.consumed_generators.contains(storage))
        {
            let feature = format!("unconsumed generator `{name}` and close delivery");
            return Err(unsupported(&self.source, *span, &feature));
        }
        Ok(CompilerBlock {
            statements: lowered,
            result: result.unwrap_or_else(|| unit_expression(Span::new(0, 0))),
        })
    }

    fn analyze_root_foreach(
        &mut self,
        source: &Expression,
        binding: Span,
        statements: &[Statement],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Application { items, .. } = source
            && let [Expression::Identifier(operation), text] = items.as_slice()
            && self.source.slice(*operation) == "characters"
        {
            let generator =
                self.analyze_closed_string_characters_generator(text, source.span(), environment)?;
            let CompilerExpressionKind::StringCharactersGenerator { characters, .. } =
                &generator.kind
            else {
                unreachable!("closed characters construction retains its generator")
            };
            return self.finish_string_characters_foreach(
                generator.clone(),
                characters.clone(),
                binding,
                statements,
                span,
            );
        }
        let retained_generator = if let Expression::Identifier(name) = source {
            environment
                .get(self.source.slice(*name))
                .and_then(|facts| self.generator_values.get(&facts.storage_name))
                .cloned()
        } else {
            None
        };
        if let Some(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersGenerator { characters, .. },
            ..
        }) = retained_generator
        {
            let source_value = self.analyze_expression(source, environment)?;
            require_type(
                &self.source,
                source_value.span,
                &character_unit_generator_type(),
                &source_value.value_type,
            )?;
            return self.finish_string_characters_foreach(
                source_value,
                characters,
                binding,
                statements,
                span,
            );
        }
        self.analyze_bounded_int_iterate_foreach(source, binding, statements, span, environment)
    }

    fn analyze_closed_string_characters_generator(
        &mut self,
        text: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let text_value = self.analyze_expression(text, environment)?;
        require_type(
            &self.source,
            text_value.span,
            &CompilerType::String,
            &text_value.value_type,
        )?;
        let text = Self::known_string_value(&text_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                text.span(),
                "dynamic String Character generator",
            )
        })?;
        let characters = characters(&text).map(str::to_owned).collect();
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersGenerator {
                text: Box::new(text_value),
                characters,
            },
            value_type: character_unit_generator_type(),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn finish_string_characters_foreach(
        &mut self,
        source: CompilerExpression,
        characters: Vec<String>,
        binding: Span,
        statements: &[Statement],
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        let (parameter, body) =
            self.analyze_unit_foreach_body(binding, statements, CompilerType::Character)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersForeach {
                source: Box::new(source),
                characters,
                parameter,
                body: Box::new(body),
            },
            value_type: CompilerType::Unit,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_bounded_int_iterate_foreach(
        &mut self,
        source: &Expression,
        binding: Span,
        statements: &[Statement],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let retained_generator = if let Expression::Identifier(name) = source {
            environment
                .get(self.source.slice(*name))
                .and_then(|facts| self.generator_values.get(&facts.storage_name))
                .cloned()
        } else {
            None
        };
        let source_value = self.analyze_expression(source, environment)?;
        require_type(
            &self.source,
            source_value.span,
            &int_unit_generator_type(),
            &source_value.value_type,
        )?;
        let generator = retained_generator.as_ref().unwrap_or(&source_value);
        let CompilerExpressionKind::GeneratorTakeWhile {
            generator: iterate,
            parameters: predicate_parameters,
            predicate,
        } = &generator.kind
        else {
            return Err(source_diagnostic(
                &self.source,
                "E-UNBOUNDED-GENERATOR-TRAVERSAL",
                source.span(),
                "foreach requires a statically finite generated traversal",
            ));
        };
        let CompilerExpressionKind::IterateGenerator {
            initial,
            parameters: next_parameters,
            next,
        } = &iterate.kind
        else {
            return Err(unsupported(
                &self.source,
                source.span(),
                "foreach through this Generator construction",
            ));
        };
        if !matches!(initial.kind, CompilerExpressionKind::Int(_))
            || !compiler_block_is_closed_over_parameters(next_parameters, next)
            || !compiler_block_is_closed_over_parameters(predicate_parameters, predicate)
        {
            return Err(unsupported(
                &self.source,
                source.span(),
                "captured or dynamically initialized iterate foreach",
            ));
        }

        let (parameter, body) =
            self.analyze_unit_foreach_body(binding, statements, CompilerType::Int)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::IterateGeneratorForeach {
                generator: Box::new(generator.clone()),
                parameter,
                body: Box::new(body),
            },
            value_type: CompilerType::Unit,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_unit_foreach_body(
        &mut self,
        binding: Span,
        statements: &[Statement],
        value_type: CompilerType,
    ) -> Result<(CompilerParameter, CompilerBlock), Diagnostic> {
        let parameter_name = self.source.slice(binding).to_owned();
        let parameter = CompilerParameter {
            name: parameter_name.clone(),
            discarded: parameter_name == "_",
            value_type: value_type.clone(),
            int_range: None,
            span: binding,
        };
        let mut body_environment = BTreeMap::new();
        if !parameter.discarded {
            body_environment.insert(
                parameter_name,
                BindingFacts {
                    storage_name: parameter.name.clone(),
                    runtime_bound: true,
                    value_type,
                    int_range: None,
                    rational_value: None,
                    string_value: None,
                    closed_int_range: None,
                    record_fields: BTreeMap::new(),
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
        }
        let body = self.analyze_block(
            statements,
            &mut body_environment,
            BlockKind::Lexical,
            Some(&CompilerType::Unit),
        )?;
        require_type(
            &self.source,
            body.result.span,
            &CompilerType::Unit,
            &body.result.value_type,
        )?;
        Ok((parameter, body))
    }

    #[allow(clippy::too_many_lines)] // Exhaustive expression admission keeps the subset boundary visible.
    fn analyze_expression(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let span = expression.span();
        match expression {
            Expression::Block { statements, .. } => {
                let mut nested = environment.clone();
                let mut block =
                    self.analyze_block(statements, &mut nested, BlockKind::Lexical, None)?;
                if statements.is_empty() {
                    block.result = unit_expression(span);
                }
                Ok(CompilerExpression {
                    value_type: block.result.value_type.clone(),
                    int_range: block.result.int_range.clone(),
                    rational_value: block.result.rational_value.clone(),
                    kind: CompilerExpressionKind::Block(Box::new(block)),
                    span,
                })
            }
            Expression::Unit(_) => Ok(unit_expression(span)),
            Expression::Identifier(name) if self.source.slice(*name) == "Completed" => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Completed,
                    value_type: CompilerType::Completed,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name) if self.source.slice(*name) == "root" => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Root,
                    value_type: CompilerType::Scope,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::ContextIdentifier(member) => {
                if !self.in_function {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-CONTEXT-SELECTION",
                        *member,
                        "defining-context selection is available only inside a function body",
                    ));
                }
                let member_name = self.source.slice(*member);
                let parameter_name = format!("@ {member_name}");
                let facts = environment.get(&parameter_name).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-COMPILER-UNSUPPORTED",
                        *member,
                        format!(
                            "compiler increment cannot capture defining-context member `{member_name}`"
                        ),
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Local(facts.storage_name.clone()),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    span,
                })
            }
            Expression::AnonymousFunction {
                parameters,
                body,
                span,
            } => {
                self.function_values_used = true;
                let parameter_names = parameters
                    .iter()
                    .flat_map(|parameter| match parameter {
                        AnonymousPattern::Binding(binding) => vec![self.source.slice(*binding)],
                        AnonymousPattern::Product { bindings, .. } => bindings
                            .iter()
                            .map(|binding| self.source.slice(*binding))
                            .collect(),
                    })
                    .collect::<BTreeSet<_>>();
                let captures = environment
                    .iter()
                    .filter(|(name, _)| {
                        !parameter_names.contains(name.as_str())
                            && expression_mentions_name(&self.source, body, name)
                    })
                    .map(|(name, facts)| (name.clone(), facts.clone()))
                    .collect();
                let tag_index = self
                    .functions
                    .len()
                    .checked_add(3)
                    .and_then(|value| value.checked_add(self.anonymous_function_value_names.len()))
                    .ok_or_else(|| unsupported(&self.source, *span, "native Function value tag"))?;
                let tag = u32::try_from(tag_index)
                    .map_err(|_| unsupported(&self.source, *span, "native Function value tag"))?;
                let display = format!("<anonymous fn/{}>", parameters.len());
                self.anonymous_function_value_names.push(display);
                self.anonymous_callables.insert(
                    tag,
                    CompilerCallableFacts::Anonymous {
                        parameters: parameters.clone(),
                        body: body.as_ref().clone(),
                        captures,
                        static_context: self.static_context,
                        span: *span,
                    },
                );
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::FunctionValue(tag),
                    value_type: CompilerType::Function,
                    int_range: None,
                    rational_value: None,
                    span: *span,
                })
            }
            Expression::Callable { kind, span }
                if matches!(
                    kind,
                    CallableKind::Plus | CallableKind::Minus | CallableKind::Compare
                ) =>
            {
                self.function_values_used = true;
                let offset = match kind {
                    CallableKind::Plus => 0,
                    CallableKind::Minus => 1,
                    CallableKind::Compare => 2,
                    _ => unreachable!("guard selected the symbolic Function subset"),
                };
                let value = u32::try_from(self.functions.len() + offset)
                    .map_err(|_| unsupported(&self.source, *span, "native Function value tag"))?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::FunctionValue(value),
                    value_type: CompilerType::Function,
                    int_range: None,
                    rational_value: None,
                    span: *span,
                })
            }
            Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name))
                    && self.functions.contains_key(self.source.slice(*name)) =>
            {
                self.function_values_used = true;
                let function_name = self.source.slice(*name);
                let declarations = self
                    .functions
                    .get(function_name)
                    .expect("guard established a function declaration");
                if !declarations
                    .iter()
                    .any(|declaration| declaration.span.end <= name.start)
                {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNBOUND-NAME",
                        *name,
                        format!("function `{function_name}` is not yet a value"),
                    ));
                }
                let value = self
                    .functions
                    .keys()
                    .position(|candidate| candidate == function_name)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or_else(|| unsupported(&self.source, *name, "native Function value tag"))?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::FunctionValue(value),
                    value_type: CompilerType::Function,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if environment
                    .get(self.source.slice(*name))
                    .and_then(|facts| facts.static_capability.as_ref())
                    .is_some() =>
            {
                let capability = environment
                    .get(self.source.slice(*name))
                    .and_then(|facts| facts.static_capability.clone())
                    .expect("guard established retained Capability metadata");
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Capability(capability),
                    value_type: CompilerType::Capability,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name) if fundamental_capability(self.source.slice(*name)) => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Capability(CompilerCapability::atomic(
                        self.source.slice(*name),
                    )),
                    value_type: CompilerType::Capability,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if fundamental_type_value(self.source.slice(*name)).is_some() =>
            {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::TypeValue(
                        fundamental_type_value(self.source.slice(*name))
                            .expect("guard established a fundamental Type value"),
                    ),
                    value_type: CompilerType::Type,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if compiler_layout_policy(self.source.slice(*name)).is_some() =>
            {
                let (enumeration, value) = compiler_layout_policy(self.source.slice(*name))
                    .expect("guard established a fundamental layout policy");
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Enum(value),
                    value_type: CompilerType::Enum(enumeration),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Boolean(value) => Ok(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(self.source.slice(*value) == "true"),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            }),
            Expression::Integer(value) => {
                let integer = parse_integer(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NUMERIC-LITERAL",
                        *value,
                        "invalid integer literal",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Int(integer.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(integer)),
                    rational_value: None,
                    span,
                })
            }
            Expression::Rational(value) => {
                let rational = parse_rational(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NUMERIC-LITERAL",
                        *value,
                        "invalid rational literal",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Rational(rational.clone()),
                    value_type: CompilerType::Rational,
                    int_range: None,
                    rational_value: Some(rational),
                    span,
                })
            }
            Expression::String(value) => {
                let value = parse_string(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-STRING-LITERAL",
                        *value,
                        "invalid string literal delimiter",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::String(value.to_owned()),
                    value_type: CompilerType::String,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name))
                    && self
                        .enum_alternatives
                        .get(self.source.slice(*name))
                        .is_some_and(|(_, _, declaration)| declaration.end <= name.start) =>
            {
                let (enumeration, value, _) = self
                    .enum_alternatives
                    .get(self.source.slice(*name))
                    .expect("checked enum alternative exists")
                    .clone();
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Enum(value),
                    value_type: CompilerType::Enum(enumeration),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name))
                    && self
                        .sum_alternatives
                        .get(self.source.slice(*name))
                        .is_some_and(|(sum, value, declaration)| {
                            declaration.end <= name.start
                                && sum.alternatives
                                    [usize::try_from(*value).expect("u32 sum tag fits usize")]
                                .payload
                                .is_none()
                        }) =>
            {
                let (sum, value, _) = self
                    .sum_alternatives
                    .get(self.source.slice(*name))
                    .expect("checked payload-free Union alternative exists")
                    .clone();
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Sum {
                        value,
                        payload: None,
                    },
                    value_type: CompilerType::Sum(sum),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Product { fields, .. } => {
                if !fields.is_empty() && fields.iter().all(|field| field.label.is_some()) {
                    let mut values = Vec::with_capacity(fields.len());
                    let mut value_types = Vec::with_capacity(fields.len());
                    for field in fields {
                        let label_span = field.label.expect("record fields are labeled");
                        let label = self.source.slice(label_span).to_owned();
                        if values
                            .iter()
                            .any(|(existing, _): &(String, CompilerExpression)| existing == &label)
                        {
                            return Err(source_diagnostic(
                                &self.source,
                                "E-DUPLICATE-RECORD-FIELD",
                                label_span,
                                "record field label occurs more than once",
                            ));
                        }
                        let value = self.analyze_expression(&field.value, environment)?;
                        value_types.push((label.clone(), value.value_type.clone()));
                        values.push((label, value));
                    }
                    if value_types
                        .iter()
                        .any(|(_, value_type)| compiler_type_contains_generator(value_type))
                    {
                        return Err(unsupported(
                            &self.source,
                            span,
                            "generator containment in a product",
                        ));
                    }
                    value_types.sort_by(|left, right| left.0.cmp(&right.0));
                    return Ok(CompilerExpression {
                        value_type: CompilerType::Record(value_types),
                        kind: CompilerExpressionKind::Record(values),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                let values = fields
                    .iter()
                    .map(|field| self.analyze_expression(&field.value, environment))
                    .collect::<Result<Vec<_>, _>>()?;
                if values
                    .iter()
                    .any(|value| compiler_type_contains_generator(&value.value_type))
                {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "generator containment in a product",
                    ));
                }
                Ok(CompilerExpression {
                    value_type: CompilerType::Tuple(
                        values
                            .iter()
                            .map(|value| value.value_type.clone())
                            .collect(),
                    ),
                    kind: CompilerExpressionKind::Tuple(values),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name) => {
                let name_text = self.source.slice(*name);
                let facts = environment.get(name_text).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-UNBOUND-NAME",
                        *name,
                        format!("name `{name_text}` is not bound"),
                    )
                })?;
                if !facts.runtime_bound {
                    let feature = if compiler_type_contains_static_only(&facts.value_type) {
                        "runtime use of a static compiler value"
                    } else {
                        "nested Function value outside direct application"
                    };
                    return Err(unsupported(&self.source, *name, feature));
                }
                if matches!(facts.value_type, CompilerType::Generator(_))
                    && !self.consumed_generators.insert(facts.storage_name.clone())
                {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-GENERATOR-CONSUMED",
                        *name,
                        format!("generator `{name_text}` was already consumed"),
                    ));
                }
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Local(facts.storage_name.clone()),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    span,
                })
            }
            Expression::Application { items, .. } => {
                self.analyze_application(items, span, environment)
            }
            Expression::DecisionTable { subject, rules, .. } => {
                self.analyze_decision(subject, rules, span, environment)
            }
            _ => Err(unsupported(&self.source, span, "expression form")),
        }
    }

    fn analyze_constraint_definition(
        &mut self,
        name: &str,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let (base, parameters, predicate, span) =
            constraint_definition(&self.source, expression).expect("preselected constraint");
        let base_type = self.parse_classifier(base)?;
        if !matches!(
            base_type,
            CompilerType::Boolean
                | CompilerType::Int
                | CompilerType::Nat
                | CompilerType::Rational
                | CompilerType::String
        ) {
            return Err(unsupported(
                &self.source,
                base,
                "native constraint base classifier",
            ));
        }
        let [AnonymousPattern::Binding(parameter)] = parameters else {
            return Err(unsupported(
                &self.source,
                span,
                "native constraint predicate pattern",
            ));
        };
        let parameter_name = self.source.slice(*parameter).to_owned();
        if let Some(capture) = environment.keys().find(|candidate| {
            candidate.as_str() != parameter_name
                && expression_mentions_name(&self.source, predicate, candidate)
        }) {
            return Err(unsupported(
                &self.source,
                predicate.span(),
                &format!("captured constraint predicate value `{capture}`"),
            ));
        }
        let storage_name = format!("topal.constraint.{}.{}", span.start, parameter_name);
        let mut predicate_environment = BTreeMap::new();
        predicate_environment.insert(
            parameter_name.clone(),
            BindingFacts {
                storage_name: storage_name.clone(),
                runtime_bound: true,
                value_type: base_type.clone(),
                int_range: None,
                rational_value: None,
                string_value: None,
                closed_int_range: None,
                record_fields: BTreeMap::new(),
                namespace: None,
                callable: None,
                static_capability: None,
            },
        );
        let previous_in_function = self.in_function;
        self.in_function = true;
        let predicate_result = self.analyze_expression(predicate, &predicate_environment);
        self.in_function = previous_in_function;
        let predicate = predicate_result?;
        require_type(
            &self.source,
            predicate.span,
            &CompilerType::Boolean,
            &predicate.value_type,
        )?;
        let tag = u32::try_from(self.constraints.len())
            .map_err(|_| unsupported(&self.source, span, "native Constraint value tag"))?;
        self.constraints.push(CompilerConstraint {
            name: name.to_owned(),
            base_type,
            parameter: parameter_name,
            parameter_storage: storage_name,
            predicate,
            span,
        });
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ConstraintValue(tag),
            value_type: CompilerType::Constraint,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_introspection(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let Some(value) = self.analyze_static_introspection_prefix(items, span)? {
            return Ok(Some(value));
        }
        self.analyze_static_introspection_relation(items, span)
    }

    fn analyze_static_introspection_prefix(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
        ] = items
            && self.source.slice(*namespace) == "lang"
        {
            let value = match self.source.slice(*operation) {
                "context" => CompilerExpression {
                    kind: CompilerExpressionKind::LanguageContext(CompilerLanguageContext {
                        language: "topal".into(),
                        version: self.language_version,
                        features: self.language_features.clone(),
                    }),
                    value_type: CompilerType::LanguageContext,
                    int_range: None,
                    rational_value: None,
                    span,
                },
                "version" => CompilerExpression {
                    kind: CompilerExpressionKind::Version(self.language_version),
                    value_type: CompilerType::Version,
                    int_range: None,
                    rational_value: None,
                    span,
                },
                _ => return Ok(None),
            };
            return Ok(Some(value));
        }

        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
            subject,
        ] = items
            && self.source.slice(*namespace) == "lang"
            && matches!(self.source.slice(*operation), "identity" | "view")
        {
            let Expression::Identifier(subject_span) = subject else {
                return Err(unsupported(
                    &self.source,
                    subject.span(),
                    "static introspection of a runtime value",
                ));
            };
            let identity = self.source.slice(*subject_span);
            if fundamental_type_value(identity).is_none() {
                if self.source.slice(*operation) == "view" {
                    return Ok(None);
                }
                return Err(unsupported(
                    &self.source,
                    *subject_span,
                    "lang identity for this static object",
                ));
            }
            let (kind, value_type) = if self.source.slice(*operation) == "identity" {
                (
                    CompilerExpressionKind::Identity(CompilerIdentity {
                        kind: ObjectKind::Type,
                        canonical: format!("type:{identity}"),
                    }),
                    CompilerType::Identity,
                )
            } else {
                (
                    CompilerExpressionKind::TypeView(CompilerTypeView {
                        form: CompilerTypeViewForm::Primitive,
                        identity: identity.into(),
                    }),
                    CompilerType::TypeView,
                )
            };
            return Ok(Some(CompilerExpression {
                kind,
                value_type,
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        Ok(None)
    }

    fn analyze_static_introspection_relation(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let [
            left,
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
            right,
        ] = items
            && self.source.slice(*namespace) == "lang"
            && matches!(
                self.source.slice(*operation),
                "same-object" | "equivalent-type"
            )
        {
            let (Expression::Identifier(left), Expression::Identifier(right)) = (left, right)
            else {
                return Err(unsupported(
                    &self.source,
                    span,
                    "static introspection relation over runtime values",
                ));
            };
            let left = self.source.slice(*left);
            let right = self.source.slice(*right);
            if fundamental_type_value(left).is_none() || fundamental_type_value(right).is_none() {
                return Err(unsupported(
                    &self.source,
                    span,
                    "static introspection relation for these object kinds",
                ));
            }
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(left == right),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        Ok(None)
    }

    fn analyze_function_view(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let [
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
            subject,
        ] = items
        else {
            return Ok(None);
        };
        if self.source.slice(*namespace) != "lang" || self.source.slice(*operation) != "view" {
            return Ok(None);
        }
        let Expression::Identifier(name_span) = subject else {
            return Err(unsupported(
                &self.source,
                subject.span(),
                "lang view for this static object",
            ));
        };
        let name = self.source.slice(*name_span);
        let Some(declarations) = self.functions.get(name) else {
            return Err(unsupported(
                &self.source,
                *name_span,
                "lang view for this static object",
            ));
        };
        let visible = declarations
            .iter()
            .filter(|declaration| declaration.span.end <= name_span.start)
            .collect::<Vec<_>>();
        let [declaration] = visible.as_slice() else {
            return Err(unsupported(
                &self.source,
                *name_span,
                "zero- or multi-overload Function view",
            ));
        };
        if declaration.declared_effects.is_none() {
            return Err(unsupported(
                &self.source,
                declaration.effect_bound.unwrap_or(declaration.span),
                "Function view without an admitted explicit empty effect bound",
            ));
        }
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::FunctionView(CompilerFunctionView {
                identity: format!("root.{name}"),
                inputs: declaration
                    .parameters
                    .iter()
                    .map(|parameter| compact_classifier(self.source.slice(parameter.classifier)))
                    .collect(),
                output: compact_classifier(self.source.slice(declaration.result)),
                is_static: declaration.is_static,
                declared_effects: declaration.declared_effects.clone(),
            }),
            value_type: CompilerType::FunctionView,
            int_range: None,
            rational_value: None,
            span,
        }))
    }

    fn analyze_constraint_application(
        &mut self,
        tag: u32,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let constraint =
            self.constraints[usize::try_from(tag).expect("u32 tag fits usize")].clone();
        if constraint.base_type != CompilerType::Int {
            return Err(unsupported(
                &self.source,
                operand.span(),
                "native constraint application base classifier",
            ));
        }
        let mut value = self.analyze_expression(operand, environment)?;
        if matches!(value.value_type, CompilerType::Refined { .. }) {
            value = forget_refined_evidence(value);
        }
        require_same_type(
            &self.source,
            operand.span(),
            &constraint.base_type,
            &value.value_type,
        )?;
        if compiler_expression_is_closed(&value) {
            let accepted = known_constraint_predicate(
                &constraint.predicate,
                &constraint.parameter_storage,
                &value,
            )
            .ok_or_else(|| {
                unsupported(
                    &self.source,
                    constraint.predicate.span,
                    "closed constraint predicate evaluation",
                )
            })?;
            if !accepted {
                return Err(source_diagnostic(
                    &self.source,
                    "E-CONSTRAINT-REJECTED",
                    operand.span(),
                    format!("value does not satisfy constraint `{}`", constraint.name),
                ));
            }
            value.value_type = CompilerType::Refined {
                constraint: constraint.name,
                base: Box::new(constraint.base_type),
            };
            value.span = span;
            return Ok(value);
        }
        Ok(Self::finish_validation(
            CompilerValidation::Constraint(tag),
            value,
            constraint.base_type,
            span,
            operand.span(),
        ))
    }

    fn analyze_sum_construction(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let selected = if let [Expression::Identifier(constructor), payload] = items {
            let Some((sum, value, declaration)) = self
                .sum_alternatives
                .get(self.source.slice(*constructor))
                .cloned()
            else {
                return Ok(None);
            };
            if declaration.end > constructor.start {
                return Ok(None);
            }
            let expected = sum.alternatives
                [usize::try_from(value).expect("u32 sum tag fits usize")]
            .payload
            .clone();
            let Some(expected) = expected else {
                return Ok(None);
            };
            (sum, value, expected, payload)
        } else if let [
            Expression::Identifier(type_name),
            Expression::Identifier(at),
            Expression::Integer(index),
            payload,
        ] = items
            && self.source.slice(*at) == "at"
            && let Some((sum, declaration)) = self.sums.get(self.source.slice(*type_name)).cloned()
            && sum.positional
            && declaration.end <= type_name.start
        {
            let value = parse_integer(self.source.slice(*index))
                .and_then(|value| value.to_string().parse::<usize>().ok())
                .filter(|value| *value < sum.alternatives.len())
                .ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-VARIANT-INDEX",
                        *index,
                        "Variant alternative index is outside its declared bounds",
                    )
                })?;
            let expected = sum.alternatives[value]
                .payload
                .clone()
                .expect("positional Variant alternatives carry payloads");
            (
                sum,
                u32::try_from(value).expect("validated native sum tag"),
                expected,
                payload,
            )
        } else {
            return Ok(None);
        };
        let (sum, value, expected, payload) = selected;
        let payload_value =
            self.analyze_expression_with_expected(payload, environment, Some(&expected))?;
        let payload_value = adapt_call_argument(&expected, &payload_value).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                if sum.positional {
                    "E-VARIANT-PAYLOAD-CLASSIFIER"
                } else {
                    "E-UNION-PAYLOAD-CLASSIFIER"
                },
                payload.span(),
                format!(
                    "sum alternative `{}` requires {}, found {}",
                    sum.alternatives[usize::try_from(value).expect("u32 sum tag fits usize")].name,
                    expected.name(),
                    payload_value.value_type.name()
                ),
            )
        })?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::Sum {
                value,
                payload: Some(Box::new(payload_value)),
            },
            value_type: CompilerType::Sum(sum),
            int_range: None,
            rational_value: None,
            span,
        }))
    }

    fn analyze_modular_construction(
        &mut self,
        modular: CompilerModularType,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand.span,
            &CompilerType::Int,
            &operand.value_type,
        )?;
        if operand
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= modular.lower && range.upper <= modular.upper)
        {
            let int_range = operand.int_range.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToModular {
                    value: Box::new(operand),
                    modular: modular.clone(),
                },
                value_type: CompilerType::Modular(modular),
                int_range,
                rational_value: None,
                span,
            });
        }
        if operand
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < modular.lower || range.lower > modular.upper)
            && compiler_expression_is_closed(&operand)
        {
            return Err(source_diagnostic(
                &self.source,
                "E-MODULAR-OUT-OF-RANGE",
                operand.span,
                format!("value is outside `{}` canonical range", modular.name),
            ));
        }
        let error_span = operand.span;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ModularValidate {
                value: Box::new(operand),
                modular: modular.clone(),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(CompilerType::Modular(modular))),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_modular_reduction(
        &mut self,
        modular: CompilerModularType,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand.span,
            &CompilerType::Int,
            &operand.value_type,
        )?;
        let int_range = exact_int(&operand)
            .map(|value| IntRange::exact(reduce_modular(value, &modular)))
            .or_else(|| Some(modular_range(&modular)));
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ModularReduce {
                value: Box::new(operand),
                modular: modular.clone(),
            },
            value_type: CompilerType::Modular(modular),
            int_range,
            rational_value: None,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Root operations are admitted explicitly and in source-selection order.
    fn analyze_application(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Some(value) = self.analyze_static_introspection(items, span)? {
            return Ok(value);
        }
        if let Some(view) = self.analyze_function_view(items, span)? {
            return Ok(view);
        }
        if let Some(sum) = self.analyze_sum_construction(items, span, environment)? {
            return Ok(sum);
        }
        if let [Expression::Identifier(operation), text] = items
            && self.source.slice(*operation) == "characters"
        {
            return self.analyze_closed_string_characters_generator(text, span, environment);
        }
        if let [
            Expression::Identifier(characters),
            text,
            Expression::Identifier(operation),
            Expression::Identifier(target),
        ] = items
            && self.source.slice(*characters) == "characters"
            && self.source.slice(*operation) == "collect"
            && self.source.slice(*target) == "String"
        {
            let generator = self.analyze_closed_string_characters_generator(
                text,
                Span::new(characters.start, text.span().end),
                environment,
            )?;
            let CompilerExpressionKind::StringCharactersGenerator { text, characters } =
                generator.kind
            else {
                unreachable!("closed characters construction retains its generator")
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersCollect { text, characters },
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(name), operand] = items
            && let Some((modular, declaration)) = self.modulars.get(self.source.slice(*name))
            && declaration.end <= name.start
        {
            return self.analyze_modular_construction(modular.clone(), operand, span, environment);
        }
        if let [
            operand,
            Expression::Identifier(operation),
            Expression::Identifier(name),
        ] = items
            && self.source.slice(*operation) == "modulo"
            && let Some((modular, declaration)) = self.modulars.get(self.source.slice(*name))
            && declaration.end <= name.start
        {
            return self.analyze_modular_reduction(modular.clone(), operand, span, environment);
        }
        if let [Expression::Identifier(keyword), selected] = items
            && self.source.slice(*keyword) == "use"
        {
            if self.in_function {
                return Err(unsupported(
                    &self.source,
                    span,
                    "function-body namespace use",
                ));
            }
            let selected = self.analyze_expression(selected, environment)?;
            if selected.value_type != CompilerType::Scope {
                return Err(source_diagnostic(
                    &self.source,
                    "E-USE-NON-NAMESPACE",
                    selected.span,
                    "use requires a published namespace path",
                ));
            }
            return Ok(selected);
        }
        if let Some((Expression::Identifier(namespace), remaining)) = items.split_first()
            && self.source.slice(*namespace) == "root"
            && let Some(Expression::Identifier(member)) = remaining.first()
        {
            let member_name = self.source.slice(*member).to_owned();
            if self.functions.contains_key(&member_name) {
                return self.analyze_resolved_call(remaining, span, environment, 0, &member_name);
            }
            if remaining.len() == 1
                && !self.in_function
                && let Some(facts) = self.root_bindings.get(&member_name)
            {
                if compiler_type_contains_generator(&facts.value_type) {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "qualified generator access",
                    ));
                }
                return Ok(data_member_expression(facts, span));
            }
            return Err(unsupported(&self.source, *member, "qualified root member"));
        }
        if let Some((Expression::Identifier(alias), remaining)) = items.split_first()
            && let Some(namespace) = environment
                .get(self.source.slice(*alias))
                .and_then(|facts| facts.namespace.as_ref())
            && let Some(Expression::Identifier(member)) = remaining.first()
        {
            let member_name = self.source.slice(*member).to_owned();
            if let Some(declarations) = namespace.functions.get(&member_name) {
                return self.analyze_resolved_call_from(
                    remaining,
                    span,
                    environment,
                    0,
                    &member_name,
                    declarations,
                    &[],
                );
            }
            if remaining.len() == 1
                && let Some(facts) = namespace.bindings.get(&member_name)
            {
                if compiler_type_contains_generator(&facts.value_type) {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "qualified generator access",
                    ));
                }
                return Ok(data_member_expression(facts, span));
            }
            return Err(source_diagnostic(
                &self.source,
                "E-COMPILER-UNSUPPORTED",
                *member,
                format!(
                    "compiler increment cannot resolve member `{member_name}` from namespace `{}`",
                    namespace.name
                ),
            ));
        }
        if let Some(Expression::Identifier(alias)) = items.first()
            && let Some(callable) = environment
                .get(self.source.slice(*alias))
                .and_then(|facts| facts.callable.as_ref())
        {
            return match callable {
                CompilerCallableFacts::Named {
                    name,
                    declarations,
                    captures,
                } => self.analyze_resolved_call_from(
                    items,
                    span,
                    environment,
                    0,
                    name,
                    declarations,
                    captures,
                ),
                CompilerCallableFacts::Symbolic(kind) => {
                    self.analyze_bound_symbolic_callable(*kind, items, span, environment)
                }
                CompilerCallableFacts::Anonymous {
                    parameters,
                    body,
                    captures,
                    static_context,
                    span: declaration_span,
                } => self.analyze_bound_anonymous_function(
                    parameters,
                    body,
                    captures,
                    *static_context,
                    *declaration_span,
                    items,
                    span,
                    environment,
                ),
            };
        }
        if let [Expression::Identifier(name), operand] = items
            && let Some(tag) = self
                .constraint_bindings
                .get(self.source.slice(*name))
                .copied()
            && self.constraints[usize::try_from(tag).expect("u32 tag fits usize")]
                .span
                .end
                <= name.start
        {
            return self.analyze_constraint_application(tag, operand, span, environment);
        }
        if items.len() > 1
            && items
                .iter()
                .all(|item| matches!(item, Expression::String(_)))
        {
            let mut value = String::new();
            for item in items {
                let Expression::String(literal) = item else {
                    unreachable!("checked adjacent String literals")
                };
                value.push_str(parse_string(self.source.slice(*literal)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-STRING-LITERAL",
                        *literal,
                        "invalid string literal delimiter",
                    )
                })?);
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(value),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(operation),
            Expression::Identifier(domain),
        ] = items
            && self.source.slice(*operation) == "empty"
            && self.source.slice(*domain) == "String"
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringEmpty,
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), Expression::Unit(_)] = items
            && self.source.slice(*constructor) == "Effects"
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Effect,
                value_type: CompilerType::Effect,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), value] = items
            && matches!(self.source.slice(*constructor), "Continue" | "Finish")
        {
            let finish = self.source.slice(*constructor) == "Finish";
            let value = self.analyze_expression(value, environment)?;
            require_type(
                &self.source,
                value.span,
                &CompilerType::Int,
                &value.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::TraversalControl {
                    finish,
                    value: Box::new(value),
                },
                value_type: CompilerType::TraversalControl(Box::new(CompilerType::Int)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if items.len() >= 3
            && items.len() % 2 == 1
            && items.iter().skip(1).step_by(2).all(
                |item| matches!(item, Expression::Identifier(operation) if self.source.slice(*operation) == "concat"),
            )
        {
            let mut result = self.analyze_expression(&items[0], environment)?;
            if result.value_type == CompilerType::String {
                for operand in items.iter().skip(2).step_by(2) {
                    let right = self.analyze_expression(operand, environment)?;
                    require_type(
                        &self.source,
                        right.span,
                        &CompilerType::String,
                        &right.value_type,
                    )?;
                    let expression_span = Span::new(result.span.start, right.span.end);
                    result = CompilerExpression {
                        kind: CompilerExpressionKind::StringConcat {
                            left: Box::new(result),
                            right: Box::new(right),
                        },
                        value_type: CompilerType::String,
                        int_range: None,
                        rational_value: None,
                        span: expression_span,
                    };
                }
                result.span = span;
                return Ok(result);
            }
        }
        if let [
            Expression::Identifier(empty),
            Expression::Identifier(list),
            element,
        ] = items
            && self.source.slice(*empty) == "empty"
            && self.source.slice(*list) == "List"
        {
            let element_type = self.parse_classifier(element.span())?;
            if element_type != CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    element.span(),
                    "explicit empty List element classifier",
                ));
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEmpty,
                value_type: CompilerType::List(Box::new(element_type)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), value] = items
            && self.source.slice(*constructor) == "one"
            && !matches!(value, Expression::Identifier(domain) if matches!(self.source.slice(*domain), "Int" | "Nat" | "Rational"))
        {
            let value = self.analyze_expression(value, environment)?;
            require_type(
                &self.source,
                value.span,
                &CompilerType::Int,
                &value.value_type,
            )?;
            let list_type = CompilerType::List(Box::new(CompilerType::Int));
            let empty = CompilerExpression {
                kind: CompilerExpressionKind::ListEmpty,
                value_type: list_type.clone(),
                int_range: None,
                rational_value: None,
                span,
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEntry {
                    value: Box::new(value),
                    remaining: Box::new(empty),
                },
                value_type: list_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            operand,
            Expression::Identifier(reverse),
            Expression::Callable { kind, .. },
            right,
        ] = items
            && self.source.slice(*reverse) == "reverse"
            && matches!(kind, CallableKind::Equal | CallableKind::NotEqual)
        {
            let operand = self.analyze_expression(operand, environment)?;
            let CompilerType::List(element) = &operand.value_type else {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this value type",
                ));
            };
            if element.as_ref() != &CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this List element type",
                ));
            }
            let list_type = operand.value_type.clone();
            let reversed = CompilerExpression {
                kind: CompilerExpressionKind::ListReverse(Box::new(operand)),
                value_type: list_type.clone(),
                int_range: None,
                rational_value: None,
                span: Span::new(items[0].span().start, items[1].span().end),
            };
            let right = self.analyze_expression(right, environment)?;
            require_same_type(&self.source, right.span, &list_type, &right.value_type)?;
            return Ok(Self::finish_binary(
                if kind == &CallableKind::Equal {
                    CompilerBinary::Equal
                } else {
                    CompilerBinary::NotEqual
                },
                reversed,
                right,
                CompilerType::Boolean,
                span,
            ));
        }
        if let [operand, Expression::Identifier(operation)] = items
            && self.source.slice(*operation) == "reverse"
        {
            let operand = self.analyze_expression(operand, environment)?;
            let CompilerType::List(element) = &operand.value_type else {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this value type",
                ));
            };
            if element.as_ref() != &CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this List element type",
                ));
            }
            let value_type = operand.value_type.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListReverse(Box::new(operand)),
                value_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(self.source.slice(*operation), "first" | "rest" | "uncons")
        {
            let operation = self.source.slice(*operation).to_owned();
            let operand = self.analyze_expression(operand, environment)?;
            let CompilerType::List(element) = &operand.value_type else {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "List projection operand",
                ));
            };
            if element.as_ref() != &CompilerType::Int
                && !(operation == "first"
                    && compiler_nested_int_string_list_element(element.as_ref()))
            {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "projection for this List element type",
                ));
            }
            let element_type = element.as_ref().clone();
            let list_type = operand.value_type.clone();
            let (kind, payload_type) = match operation.as_str() {
                "first" => (
                    CompilerExpressionKind::ListFirst(Box::new(operand)),
                    element_type,
                ),
                "rest" => (
                    CompilerExpressionKind::ListRest(Box::new(operand)),
                    list_type.clone(),
                ),
                "uncons" => (
                    CompilerExpressionKind::ListUncons(Box::new(operand)),
                    CompilerType::Tuple(vec![CompilerType::Int, list_type]),
                ),
                _ => unreachable!(),
            };
            return Ok(CompilerExpression {
                kind,
                value_type: CompilerType::Optional(Box::new(payload_type)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [list, Expression::Identifier(operation), right] = items
            && matches!(
                self.source.slice(*operation),
                "prepend" | "append" | "concat"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            if let CompilerType::List(element) = &list.value_type {
                if element.as_ref() != &CompilerType::Int {
                    return Err(unsupported(
                        &self.source,
                        list.span,
                        "operation for this List element type",
                    ));
                }
                let list_type = list.value_type.clone();
                let right = self.analyze_expression(right, environment)?;
                let kind = match operation.as_str() {
                    "prepend" => {
                        require_same_type(&self.source, right.span, element, &right.value_type)?;
                        CompilerExpressionKind::ListPrepend {
                            list: Box::new(list),
                            value: Box::new(right),
                        }
                    }
                    "append" => {
                        require_same_type(&self.source, right.span, element, &right.value_type)?;
                        CompilerExpressionKind::ListAppend {
                            list: Box::new(list),
                            value: Box::new(right),
                        }
                    }
                    "concat" => {
                        require_same_type(&self.source, right.span, &list_type, &right.value_type)?;
                        CompilerExpressionKind::ListConcat {
                            left: Box::new(list),
                            right: Box::new(right),
                        }
                    }
                    _ => unreachable!(),
                };
                return Ok(CompilerExpression {
                    kind,
                    value_type: list_type,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if operation != "concat" {
                return Err(unsupported(
                    &self.source,
                    list.span,
                    "List insertion subject",
                ));
            }
        }
        if let [
            list,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && matches!(self.source.slice(*operation), "map" | "select")
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            let parameter_type = if operation == "map" {
                require_int_or_int_pair_list(&self.source, &list, "map subject")?
            } else {
                require_int_list(&self.source, &list, "select subject")?;
                CompilerType::Int
            };
            let (parameters, body) = self.analyze_collection_function(
                parameters,
                body,
                &[parameter_type],
                environment,
                self.static_context,
                *function_span,
            )?;
            let expected = if operation == "map" {
                CompilerType::Int
            } else {
                CompilerType::Boolean
            };
            require_type(
                &self.source,
                body.result.span,
                &expected,
                &body.result.value_type,
            )?;
            let kind = if operation == "map" {
                CompilerExpressionKind::ListMap {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            } else {
                CompilerExpressionKind::ListSelect {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            };
            return Ok(CompilerExpression {
                kind,
                value_type: CompilerType::List(Box::new(CompilerType::Int)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), generator] = items
            && self.source.slice(*operation) == "collect"
        {
            let retained_generator = if let Expression::Identifier(name) = generator {
                environment
                    .get(self.source.slice(*name))
                    .and_then(|facts| self.generator_values.get(&facts.storage_name))
                    .cloned()
            } else {
                None
            };
            let direct_bounded = direct_bounded_iterate_functions(&self.source, generator);
            if let Some((next, predicate)) = direct_bounded {
                for (parameters, body, function_span, role) in [
                    (next.0, next.1, next.2, "iterate operation"),
                    (
                        predicate.0,
                        predicate.1,
                        predicate.2,
                        "take-while predicate",
                    ),
                ] {
                    if let Some(capture) =
                        anonymous_body_capture(&self.source, parameters, body, environment)
                    {
                        return Err(unsupported(
                            &self.source,
                            function_span,
                            &format!("captured {role} value `{capture}`"),
                        ));
                    }
                }
            }
            let generator = self.analyze_expression(generator, environment)?;
            require_type(
                &self.source,
                generator.span,
                &int_unit_generator_type(),
                &generator.value_type,
            )?;
            let collection_generator = retained_generator.as_ref().unwrap_or(&generator);
            match &collection_generator.kind {
                CompilerExpressionKind::GeneratorTakeWhile {
                    generator: source, ..
                } if direct_bounded.is_some()
                    && matches!(source.kind, CompilerExpressionKind::IterateGenerator { .. }) =>
                {
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::GeneratorCollect(Box::new(generator)),
                        value_type: CompilerType::List(Box::new(CompilerType::Int)),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerExpressionKind::UnfoldGenerator { .. }
                    if directly_collectable_list_uncons_unfold(
                        collection_generator,
                        environment,
                    ) =>
                {
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::GeneratorCollect(Box::new(
                            collection_generator.clone(),
                        )),
                        value_type: CompilerType::List(Box::new(CompilerType::Int)),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerExpressionKind::IterateGenerator { .. } => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNBOUNDED-GENERATOR-COLLECT",
                        generator.span,
                        "collect requires a statically finite generated traversal",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        generator.span,
                        "collection through a non-direct Generator value",
                    ));
                }
            }
        }
        if let [
            initial,
            Expression::Identifier(iterate),
            Expression::AnonymousFunction {
                parameters: next_parameters,
                body: next_body,
                span: next_span,
            },
            Expression::Identifier(take_while),
            Expression::AnonymousFunction {
                parameters: predicate_parameters,
                body: predicate_body,
                span: predicate_span,
            },
        ] = items
            && self.source.slice(*iterate) == "iterate"
            && self.source.slice(*take_while) == "take-while"
        {
            let generator = self.analyze_int_iterate_generator(
                initial,
                next_parameters,
                next_body,
                *next_span,
                span,
                environment,
            )?;
            return self.analyze_int_generator_take_while(
                generator,
                predicate_parameters,
                predicate_body,
                *predicate_span,
                span,
                environment,
            );
        }
        if let [
            initial,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "iterate"
        {
            return self.analyze_int_iterate_generator(
                initial,
                parameters,
                body,
                *function_span,
                span,
                environment,
            );
        }
        if let [
            generator,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "take-while"
        {
            let generator = self.analyze_expression(generator, environment)?;
            return self.analyze_int_generator_take_while(
                generator,
                parameters,
                body,
                *function_span,
                span,
                environment,
            );
        }
        if let [
            seed,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "unfold"
        {
            return self.analyze_int_list_unfold_generator(
                seed,
                parameters,
                body,
                *function_span,
                span,
                environment,
            );
        }
        if let [
            list,
            Expression::Identifier(operation),
            initial,
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "fold"
        {
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "fold subject")?;
            let initial = self.analyze_expression(initial, environment)?;
            require_type(
                &self.source,
                initial.span,
                &CompilerType::Int,
                &initial.value_type,
            )?;
            let (parameters, body) = self.analyze_collection_function(
                parameters,
                body,
                &[CompilerType::Int, CompilerType::Int],
                environment,
                self.static_context,
                *function_span,
            )?;
            require_int_fold_result(&self.source, &body.result)?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListFold {
                    list: Box::new(list),
                    initial: Box::new(initial),
                    parameters,
                    body: Box::new(body),
                },
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            list,
            Expression::Identifier(operation),
            Expression::Identifier(function),
        ] = items
            && matches!(self.source.slice(*operation), "map" | "select")
            && let Some(CompilerCallableFacts::Anonymous {
                parameters,
                body,
                captures,
                static_context,
                span: function_span,
            }) = environment
                .get(self.source.slice(*function))
                .and_then(|facts| facts.callable.as_ref())
                .cloned()
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            let parameter_type = if operation == "map" {
                require_int_or_int_pair_list(&self.source, &list, "map subject")?
            } else {
                require_int_list(&self.source, &list, "select subject")?;
                CompilerType::Int
            };
            let (parameters, body) = self.analyze_collection_function(
                &parameters,
                &body,
                &[parameter_type],
                &captures,
                static_context,
                function_span,
            )?;
            let expected = if operation == "map" {
                CompilerType::Int
            } else {
                CompilerType::Boolean
            };
            require_type(
                &self.source,
                body.result.span,
                &expected,
                &body.result.value_type,
            )?;
            let kind = if operation == "map" {
                CompilerExpressionKind::ListMap {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            } else {
                CompilerExpressionKind::ListSelect {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            };
            return Ok(CompilerExpression {
                kind,
                value_type: CompilerType::List(Box::new(CompilerType::Int)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            list,
            Expression::Identifier(operation),
            initial,
            Expression::Identifier(function),
        ] = items
            && self.source.slice(*operation) == "fold"
            && let Some(CompilerCallableFacts::Anonymous {
                parameters,
                body,
                captures,
                static_context,
                span: function_span,
            }) = environment
                .get(self.source.slice(*function))
                .and_then(|facts| facts.callable.as_ref())
                .cloned()
        {
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "fold subject")?;
            let initial = self.analyze_expression(initial, environment)?;
            require_type(
                &self.source,
                initial.span,
                &CompilerType::Int,
                &initial.value_type,
            )?;
            let (parameters, body) = self.analyze_collection_function(
                &parameters,
                &body,
                &[CompilerType::Int, CompilerType::Int],
                &captures,
                static_context,
                function_span,
            )?;
            require_int_fold_result(&self.source, &body.result)?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListFold {
                    list: Box::new(list),
                    initial: Box::new(initial),
                    parameters,
                    body: Box::new(body),
                },
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [collection, Expression::Identifier(operation), selector] = items
            && matches!(self.source.slice(*operation), "select" | "select-index")
        {
            let operation = self.source.slice(*operation).to_owned();
            let collection = self.analyze_expression(collection, environment)?;
            let selector = self.analyze_expression(selector, environment)?;
            require_type(
                &self.source,
                selector.span,
                &CompilerType::Range(Box::new(CompilerType::Int)),
                &selector.value_type,
            )?;
            match collection.value_type.clone() {
                CompilerType::List(element) if element.as_ref() == &CompilerType::Int => {
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::ListRangeSelect {
                            list: Box::new(collection),
                            range: Box::new(selector),
                            indexes: operation == "select-index",
                        },
                        value_type: CompilerType::List(Box::new(CompilerType::Int)),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerType::String if operation == "select-index" => {
                    let text =
                        Self::known_string_value(&collection, environment).ok_or_else(|| {
                            unsupported(
                                &self.source,
                                collection.span,
                                "dynamic String range selection",
                            )
                        })?;
                    let range =
                        Self::known_closed_int_range(&selector, environment).ok_or_else(|| {
                            unsupported(
                                &self.source,
                                selector.span,
                                "dynamic String range selector",
                            )
                        })?;
                    let selected = characters(&text)
                        .enumerate()
                        .filter_map(|(index, character)| {
                            range.contains(&BigInt::from(index)).then_some(character)
                        })
                        .collect();
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::String(selected),
                        value_type: CompilerType::String,
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerType::List(_) => {
                    return Err(unsupported(
                        &self.source,
                        collection.span,
                        "range selection for this List element type",
                    ));
                }
                CompilerType::String => {
                    return Err(unsupported(
                        &self.source,
                        collection.span,
                        "Range Int value selection for String",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        collection.span,
                        "range selection source",
                    ));
                }
            }
        }
        if let [list, Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "contains-entry" | "contains-sequence" | "contains-subsequence"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            let CompilerType::List(element) = &list.value_type else {
                return Err(unsupported(
                    &self.source,
                    list.span,
                    "List containment subject",
                ));
            };
            let element = element.as_ref().clone();
            let list_type = list.value_type.clone();
            if element != CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "containment for this List element type",
                ));
            }
            let operand = self.analyze_expression(operand, environment)?;
            let kind = match operation.as_str() {
                "contains-entry" => {
                    require_same_type(&self.source, operand.span, &element, &operand.value_type)?;
                    CompilerExpressionKind::ListContainsEntry {
                        list: Box::new(list),
                        value: Box::new(operand),
                    }
                }
                "contains-sequence" => {
                    require_same_type(&self.source, operand.span, &list_type, &operand.value_type)?;
                    CompilerExpressionKind::ListContainsSequence {
                        list: Box::new(list),
                        pattern: Box::new(operand),
                    }
                }
                "contains-subsequence" => {
                    require_same_type(&self.source, operand.span, &list_type, &operand.value_type)?;
                    CompilerExpressionKind::ListContainsSubsequence {
                        list: Box::new(list),
                        pattern: Box::new(operand),
                    }
                }
                _ => unreachable!(),
            };
            return Ok(CompilerExpression {
                kind,
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [list, Expression::Identifier(operation), value] = items
            && matches!(self.source.slice(*operation), "remove-first" | "remove-all")
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            let CompilerType::List(element) = &list.value_type else {
                return Err(unsupported(&self.source, list.span, "List removal subject"));
            };
            let element = element.as_ref().clone();
            let list_type = list.value_type.clone();
            if element != CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "removal for this List element type",
                ));
            }
            let value = self.analyze_expression(value, environment)?;
            require_same_type(&self.source, value.span, &element, &value.value_type)?;
            let kind = if operation == "remove-first" {
                CompilerExpressionKind::ListRemoveFirst {
                    list: Box::new(list),
                    value: Box::new(value),
                }
            } else {
                CompilerExpressionKind::ListRemoveAll {
                    list: Box::new(list),
                    value: Box::new(value),
                }
            };
            return Ok(CompilerExpression {
                kind,
                value_type: list_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), value] = items
            && self.source.slice(*constructor) == "Some"
        {
            let value = self.analyze_expression(value, environment)?;
            require_optional_payload(&self.source, value.span, &value.value_type)?;
            return Ok(CompilerExpression {
                value_type: CompilerType::Optional(Box::new(value.value_type.clone())),
                kind: CompilerExpressionKind::OptionalSome(Box::new(value)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "character-count" | "entry-count"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_static_character_count(&operation, operand, span, environment);
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "upper" | "lower" | "case-fold"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_static_unicode_transform(&operation, operand, span, environment);
        }
        if let [text, Expression::Identifier(operation), index] = items
            && self.source.slice(*operation) == "character-at"
        {
            return self.analyze_static_character_at(text, index, span, environment);
        }
        if let [
            base,
            Expression::Identifier(operation),
            Expression::Product { fields, .. },
        ] = items
            && self.source.slice(*operation) == "with"
            && !fields.is_empty()
            && fields.iter().all(|field| field.label.is_some())
        {
            return self.analyze_record_reconstruction(base, fields, span, environment);
        }
        if let [
            text,
            Expression::Identifier(operation),
            Expression::Identifier(form),
        ] = items
            && self.source.slice(*operation) == "normalize"
        {
            let form = self.source.slice(*form).to_owned();
            return self.analyze_static_normalization(text, &form, span, environment);
        }
        if let [left, Expression::Identifier(operation), right] = items
            && self.source.slice(*operation) == "canonically-equals"
        {
            return self.analyze_static_canonical_equality(left, right, span, environment);
        }
        if let [
            Expression::Identifier(constructor),
            Expression::Identifier(payload),
        ] = items
            && self.source.slice(*constructor) == "None"
        {
            let payload = self.parse_classifier(*payload)?;
            return self.finish_optional_none(payload, span);
        }
        if let [
            value,
            Expression::Identifier(operation),
            Expression::Identifier(encoding),
        ] = items
            && self.source.slice(*operation) == "byte-count"
        {
            if self.source.slice(*encoding) != "Utf8" {
                return Err(source_diagnostic(
                    &self.source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    *encoding,
                    "the compiler String byte-count operation requires Utf8",
                ));
            }
            let value = self.analyze_expression(value, environment)?;
            require_type(
                &self.source,
                value.span,
                &CompilerType::String,
                &value.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringUtf8ByteCount(Box::new(value)),
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(vocabulary),
            Expression::Identifier(code),
        ] = items
            && let Some(value) = arithmetic_error_code(
                self.source.slice(*namespace),
                self.source.slice(*vocabulary),
                self.source.slice(*code),
            )
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ErrorCode(value),
                value_type: CompilerType::ErrorCode,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(vocabulary),
            Expression::Identifier(code),
        ] = items
            && let Some((enumeration, value)) = generator_error_code(
                self.source.slice(*namespace),
                self.source.slice(*vocabulary),
                self.source.slice(*code),
            )
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Enum(value),
                value_type: CompilerType::Enum(enumeration),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(operation),
            Expression::Identifier(domain),
        ] = items
            && matches!(self.source.slice(*operation), "zero" | "one")
        {
            let one = self.source.slice(*operation) == "one";
            return match self.source.slice(*domain) {
                "Int" | "Nat" => {
                    let value = BigInt::from(u8::from(one));
                    let value_type = if self.source.slice(*domain) == "Nat" {
                        CompilerType::Nat
                    } else {
                        CompilerType::Int
                    };
                    Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Int(value.clone()),
                        value_type,
                        int_range: Some(IntRange::exact(value)),
                        rational_value: None,
                        span,
                    })
                }
                "Rational" => {
                    let value = BigRational::from_integer(BigInt::from(u8::from(one)));
                    Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Rational(value.clone()),
                        value_type: CompilerType::Rational,
                        int_range: None,
                        rational_value: Some(value),
                        span,
                    })
                }
                _ => Err(unsupported(
                    &self.source,
                    *domain,
                    "numeric identity domain",
                )),
            };
        }
        if let [Expression::Identifier(constructor), argument] = items {
            match self.source.slice(*constructor) {
                "Int" => return self.analyze_int_constructor(argument, span, environment),
                "Nat" => return self.analyze_nat_constructor(argument, span, environment),
                "Character" => {
                    let value = self.analyze_expression(argument, environment)?;
                    return self.finish_character_conversion(value, argument.span());
                }
                "String" => {
                    let value = self.analyze_expression(argument, environment)?;
                    let value = self.finish_character_conversion(value, argument.span())?;
                    return Ok(forget_character_evidence(value));
                }
                _ => {}
            }
        }
        if let [Expression::Identifier(constructor), argument] = items
            && self.source.slice(*constructor) == "Rational"
        {
            return self.analyze_rational_constructor(argument, span, environment);
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "empty?"
                    | "range-lower"
                    | "range-upper"
                    | "range-lower-inclusive?"
                    | "range-upper-inclusive?"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_range_observation(&operation, operand, span, environment);
        }
        if let [record, Expression::Identifier(field)] = items
            && self.record_selection_candidate(record, environment)
        {
            return self.analyze_record_field(record, *field, span, environment);
        }
        if let [error, Expression::Identifier(field)] = items
            && matches!(
                self.source.slice(*field),
                "code" | "domain" | "detail" | "cause" | "source"
            )
        {
            return self.analyze_error_field(error, *field, span, environment);
        }
        if let [
            Expression::Callable {
                kind: CallableKind::Minus,
                ..
            },
            operand,
        ] = items
        {
            let operand = self.analyze_expression(operand, environment)?;
            if let CompilerType::Modular(modular) = operand.value_type.clone() {
                let int_range = exact_int(&operand)
                    .map(|value| IntRange::exact(reduce_modular(-value, &modular)))
                    .or_else(|| Some(modular_range(&modular)));
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Negate(Box::new(operand)),
                    value_type: CompilerType::Modular(modular),
                    int_range,
                    rational_value: None,
                    span,
                });
            }
            require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
            let range = (operand.value_type == CompilerType::Int)
                .then_some(operand.int_range.as_ref())
                .flatten()
                .map(|range| IntRange {
                    lower: -range.upper.clone(),
                    upper: -range.lower.clone(),
                });
            let rational_value = operand.rational_value.as_ref().map(|value| -value.clone());
            let value_type = operand.value_type.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Negate(Box::new(operand)),
                value_type,
                int_range: range,
                rational_value,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(self.source.slice(*operation), "negate" | "absolute")
        {
            let negate = self.source.slice(*operation) == "negate";
            let operand = self.analyze_expression(operand, environment)?;
            if let CompilerType::Modular(modular) = operand.value_type.clone() {
                if !negate {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "absolute value over a modular value",
                    ));
                }
                let int_range = exact_int(&operand)
                    .map(|value| IntRange::exact(reduce_modular(-value, &modular)))
                    .or_else(|| Some(modular_range(&modular)));
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Negate(Box::new(operand)),
                    value_type: CompilerType::Modular(modular),
                    int_range,
                    rational_value: None,
                    span,
                });
            }
            require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
            let range = (operand.value_type == CompilerType::Int)
                .then_some(operand.int_range.as_ref())
                .flatten()
                .map(|range| {
                    if negate {
                        IntRange {
                            lower: -range.upper.clone(),
                            upper: -range.lower.clone(),
                        }
                    } else {
                        absolute_range(range)
                    }
                });
            let rational_value = operand.rational_value.as_ref().map(|value| {
                if negate {
                    -value.clone()
                } else {
                    rational_absolute(value)
                }
            });
            let value_type = operand.value_type.clone();
            return Ok(CompilerExpression {
                kind: if negate {
                    CompilerExpressionKind::Negate(Box::new(operand))
                } else {
                    CompilerExpressionKind::Absolute(Box::new(operand))
                },
                value_type,
                int_range: range,
                rational_value,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && self.source.slice(*operation) == "not"
        {
            let operand = self.analyze_expression(operand, environment)?;
            require_type(
                &self.source,
                operand.span,
                &CompilerType::Boolean,
                &operand.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Not(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if items.len() >= 5
            && items.len() % 2 == 1
            && items
                .iter()
                .skip(1)
                .step_by(2)
                .all(|item| matches!(item, Expression::Callable { .. }))
        {
            let mut grouped = Expression::Application {
                items: items[..3].to_vec(),
                span: Span::new(items[0].span().start, items[2].span().end),
            };
            for pair in items[3..].chunks_exact(2) {
                grouped = Expression::Application {
                    items: vec![grouped, pair[0].clone(), pair[1].clone()],
                    span: Span::new(items[0].span().start, pair[1].span().end),
                };
            }
            return self
                .analyze_expression(&grouped, environment)
                .map(|mut value| {
                    value.span = span;
                    value
                });
        }
        if let [
            record,
            Expression::Identifier(field),
            Expression::Callable { kind, .. },
            right,
        ] = items
            && self.record_selection_candidate(record, environment)
        {
            let left = Expression::Application {
                items: vec![record.clone(), Expression::Identifier(*field)],
                span: Span::new(record.span().start, field.end),
            };
            return self.analyze_symbolic_binary(*kind, &left, right, span, environment);
        }
        if let [left, Expression::Callable { kind, .. }, right] = items {
            return self.analyze_symbolic_binary(*kind, left, right, span, environment);
        }
        if let [left, Expression::Identifier(operation), right] = items
            && matches!(
                self.source.slice(*operation),
                "and" | "or" | "xor" | "in" | "contains"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_identifier_binary(&operation, left, right, span, environment);
        }
        self.analyze_call(items, span, environment)
    }

    fn record_selection_candidate(
        &self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> bool {
        match expression {
            Expression::Product { fields, .. } => {
                !fields.is_empty() && fields.iter().all(|field| field.label.is_some())
            }
            Expression::Identifier(name) => environment
                .get(self.source.slice(*name))
                .is_some_and(|facts| matches!(facts.value_type, CompilerType::Record(_))),
            _ => false,
        }
    }

    fn analyze_record_field(
        &mut self,
        record: &Expression,
        field: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let record = self.analyze_expression(record, environment)?;
        let CompilerType::Record(fields) = &record.value_type else {
            unreachable!("record selection candidate retains a Record type")
        };
        let label = self.source.slice(field).to_owned();
        let value_type = fields
            .iter()
            .find_map(|(name, value_type)| (name == &label).then(|| value_type.clone()))
            .ok_or_else(|| {
                source_diagnostic(
                    &self.source,
                    "E-NO-SUCH-RECORD-FIELD",
                    field,
                    format!("record has no field named `{label}`"),
                )
            })?;
        let facts =
            Self::known_record_field_facts(&record, &label, environment).unwrap_or_default();
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::RecordField {
                record: Box::new(record),
                label,
            },
            value_type,
            int_range: facts.int_range,
            rational_value: facts.rational_value,
            span,
        })
    }

    fn analyze_record_reconstruction(
        &mut self,
        base: &Expression,
        replacements: &[ProductField],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let base_value = self.analyze_expression(base, environment)?;
        let CompilerType::Record(fields) = &base_value.value_type else {
            return Err(source_diagnostic(
                &self.source,
                "E-RECONSTRUCT-NON-RECORD",
                base.span(),
                "`with` reconstruction requires a labeled product",
            ));
        };
        let record_type = base_value.value_type.clone();
        let mut replaced = BTreeSet::new();
        let mut values = Vec::with_capacity(replacements.len());
        for replacement in replacements {
            let label_span = replacement.label.expect("preselected labeled replacement");
            let label = self.source.slice(label_span).to_owned();
            if !replaced.insert(label.clone()) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-RECONSTRUCTION-FIELD",
                    label_span,
                    format!("field `{label}` is replaced more than once"),
                ));
            }
            let expected = fields
                .iter()
                .find_map(|(name, value_type)| (name == &label).then_some(value_type))
                .ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NO-SUCH-RECORD-FIELD",
                        label_span,
                        format!("record has no field named `{label}`"),
                    )
                })?;
            let value = self.analyze_expression(&replacement.value, environment)?;
            let value = adapt_call_argument(expected, &value).ok_or_else(|| {
                source_diagnostic(
                    &self.source,
                    "E-TYPE-MISMATCH",
                    value.span,
                    format!(
                        "record field `{label}` expects {}, found {}",
                        expected.name(),
                        value.value_type.name()
                    ),
                )
            })?;
            values.push((label, value));
        }
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::RecordReconstruct {
                base: Box::new(base_value),
                replacements: values,
            },
            value_type: record_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn finish_optional_none(
        &self,
        payload: CompilerType,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_optional_payload(&self.source, span, &payload)?;
        Ok(CompilerExpression {
            value_type: CompilerType::Optional(Box::new(payload)),
            kind: CompilerExpressionKind::OptionalNone,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_error_field(
        &mut self,
        error: &Expression,
        field: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let error = self.analyze_expression(error, environment)?;
        if !matches!(
            &error.value_type,
            CompilerType::Error | CompilerType::Result(_)
        ) {
            return Err(source_diagnostic(
                &self.source,
                "E-TYPE-MISMATCH",
                error.span,
                format!(
                    "expected Error or Result, found {}",
                    error.value_type.name()
                ),
            ));
        }
        let (field, value_type) = match self.source.slice(field) {
            "code" => (CompilerErrorField::Code, CompilerType::ErrorCode),
            "domain" => (CompilerErrorField::Domain, CompilerType::ErrorDomain),
            "detail" => (
                CompilerErrorField::Detail,
                CompilerType::Optional(Box::new(CompilerType::String)),
            ),
            "cause" => (
                CompilerErrorField::Cause,
                CompilerType::Optional(Box::new(CompilerType::Error)),
            ),
            "source" => (
                CompilerErrorField::Source,
                CompilerType::Optional(Box::new(CompilerType::SourceLocation)),
            ),
            _ => unreachable!("implemented Error field spelling selected above"),
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ErrorField {
                error: Box::new(error),
                field,
            },
            value_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_int_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let value = self.analyze_expression(argument, environment)?;
        self.finish_int_conversion(value, span, argument.span())
    }

    fn finish_int_conversion(
        &self,
        value: CompilerExpression,
        span: Span,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        if value.value_type == CompilerType::Int {
            return Ok(value);
        }
        require_type(
            &self.source,
            value.span,
            &CompilerType::Rational,
            &value.value_type,
        )?;
        if let Some(rational) = &value.rational_value {
            if rational.denom() != &BigInt::from(1) && compiler_expression_is_closed(&value) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-RATIONAL-NOT-EXACT-INT",
                    error_span,
                    format!(
                        "exact Rational operand has denominator {}, so Int cannot represent it",
                        rational.denom()
                    ),
                ));
            }
            if rational.denom() == &BigInt::from(1) {
                let numerator = rational.numer().clone();
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::RationalToInt(Box::new(value)),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(numerator)),
                    rational_value: None,
                    span,
                });
            }
        }
        Ok(Self::finish_validation(
            CompilerValidation::RationalToInt,
            value,
            CompilerType::Int,
            span,
            error_span,
        ))
    }

    fn analyze_nat_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let value = self.analyze_expression(argument, environment)?;
        self.finish_nat_conversion(value, span, argument.span())
    }

    fn finish_character_conversion(
        &self,
        mut value: CompilerExpression,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        if value.value_type == CompilerType::Character {
            return Ok(value);
        }
        require_type(
            &self.source,
            value.span,
            &CompilerType::String,
            &value.value_type,
        )?;
        let Some(text) = exact_string(&value) else {
            return Err(unsupported(
                &self.source,
                error_span,
                "dynamic Character constraint validation",
            ));
        };
        let count = character_count(&text);
        if count != 1 {
            return Err(source_diagnostic(
                &self.source,
                "E-CHARACTER-CLASSIFIER",
                error_span,
                format!(
                    "Character requires exactly one user-perceived character, but this String contains {count}"
                ),
            ));
        }
        value.value_type = CompilerType::Character;
        Ok(value)
    }

    fn analyze_static_character_count(
        &mut self,
        operation: &str,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand_value = self.analyze_expression(operand, environment)?;
        if operation == "entry-count"
            && let CompilerType::List(element) = &operand_value.value_type
        {
            if element.as_ref() != &CompilerType::Int
                && !compiler_nested_int_string_list_element(element.as_ref())
            {
                return Err(unsupported(
                    &self.source,
                    operand_value.span,
                    "entry-count for this List element type",
                ));
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEntryCount(Box::new(operand_value)),
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        require_type(
            &self.source,
            operand_value.span,
            &CompilerType::String,
            &operand_value.value_type,
        )?;
        let text = Self::known_string_value(&operand_value, environment).ok_or_else(|| {
            unsupported(&self.source, operand.span(), "dynamic Character counting")
        })?;
        let count = BigInt::from(character_count(&text));
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Int(count.clone()),
            value_type: CompilerType::Int,
            int_range: Some(IntRange::exact(count)),
            rational_value: None,
            span,
        })
    }

    fn analyze_collection_function(
        &mut self,
        parameters: &[AnonymousPattern],
        body: &Expression,
        parameter_types: &[CompilerType],
        outer_environment: &BTreeMap<String, BindingFacts>,
        static_context: bool,
        span: Span,
    ) -> Result<(Vec<CompilerParameter>, CompilerBlock), Diagnostic> {
        let flattened = if let (
            [
                AnonymousPattern::Product {
                    bindings,
                    span: pattern_span,
                },
            ],
            [CompilerType::Tuple(fields)],
        ) = (parameters, parameter_types)
        {
            if bindings.len() != fields.len() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-ANONYMOUS-FUNCTION-ARITY",
                    *pattern_span,
                    format!(
                        "collection product pattern expects {} fields, found {}",
                        fields.len(),
                        bindings.len()
                    ),
                ));
            }
            bindings
                .iter()
                .copied()
                .zip(fields.iter().cloned())
                .collect::<Vec<_>>()
        } else {
            if parameters.len() != parameter_types.len() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-ANONYMOUS-FUNCTION-ARITY",
                    span,
                    format!(
                        "collection function expects {} parameters, found {}",
                        parameter_types.len(),
                        parameters.len()
                    ),
                ));
            }
            parameters
                .iter()
                .zip(parameter_types)
                .map(|(parameter, value_type)| {
                    let AnonymousPattern::Binding(name_span) = parameter else {
                        return Err(unsupported(
                            &self.source,
                            span,
                            "collection anonymous product parameter pattern",
                        ));
                    };
                    Ok((*name_span, value_type.clone()))
                })
                .collect::<Result<Vec<_>, Diagnostic>>()?
        };
        let mut environment = outer_environment.clone();
        let mut lowered = Vec::with_capacity(flattened.len());
        let mut declared = BTreeSet::new();
        for (name_span, value_type) in flattened {
            let name = self.source.slice(name_span).to_owned();
            let discarded = name == "_";
            if !discarded && !declared.insert(name.clone()) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-BINDING",
                    name_span,
                    format!("`{name}` is already declared in this parameter pattern"),
                ));
            }
            if !discarded {
                environment = decision_binding_environment(&environment, &name, value_type.clone());
            }
            lowered.push(CompilerParameter {
                name,
                discarded,
                value_type,
                int_range: None,
                span: name_span,
            });
        }

        let previous_static_context = self.static_context;
        let previous_in_function = self.in_function;
        self.static_context = static_context;
        self.in_function = true;
        let analyzed = match body {
            Expression::Block { statements, .. } => {
                self.analyze_block(statements, &mut environment, BlockKind::Function, None)
            }
            expression => self
                .analyze_expression(expression, &environment)
                .map(|result| CompilerBlock {
                    statements: Vec::new(),
                    result,
                }),
        };
        self.static_context = previous_static_context;
        self.in_function = previous_in_function;
        Ok((lowered, analyzed?))
    }

    fn analyze_int_iterate_generator(
        &mut self,
        initial: &Expression,
        parameters: &[AnonymousPattern],
        next_body: &Expression,
        function_span: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let initial = self.analyze_expression(initial, environment)?;
        require_type(
            &self.source,
            initial.span,
            &CompilerType::Int,
            &initial.value_type,
        )?;
        let consumed_before_body = self.consumed_generators.clone();
        let (parameters, next) = self.analyze_collection_function(
            parameters,
            next_body,
            &[CompilerType::Int],
            environment,
            self.static_context,
            function_span,
        )?;
        if self.consumed_generators != consumed_before_body {
            return Err(unsupported(
                &self.source,
                function_span,
                "generator capture in an iterate operation",
            ));
        }
        require_type(
            &self.source,
            next.result.span,
            &CompilerType::Int,
            &next.result.value_type,
        )?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::IterateGenerator {
                initial: Box::new(initial),
                parameters,
                next: Box::new(next),
            },
            value_type: int_unit_generator_type(),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_int_generator_take_while(
        &mut self,
        generator: CompilerExpression,
        parameters: &[AnonymousPattern],
        predicate_body: &Expression,
        function_span: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            generator.span,
            &int_unit_generator_type(),
            &generator.value_type,
        )?;
        let consumed_before_body = self.consumed_generators.clone();
        let (parameters, predicate) = self.analyze_collection_function(
            parameters,
            predicate_body,
            &[CompilerType::Int],
            environment,
            self.static_context,
            function_span,
        )?;
        if self.consumed_generators != consumed_before_body {
            return Err(unsupported(
                &self.source,
                function_span,
                "generator capture in a take-while predicate",
            ));
        }
        require_type(
            &self.source,
            predicate.result.span,
            &CompilerType::Boolean,
            &predicate.result.value_type,
        )?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::GeneratorTakeWhile {
                generator: Box::new(generator),
                parameters,
                predicate: Box::new(predicate),
            },
            value_type: int_unit_generator_type(),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_int_list_unfold_generator(
        &mut self,
        seed: &Expression,
        parameters: &[AnonymousPattern],
        step_body: &Expression,
        function_span: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let seed = self.analyze_expression(seed, environment)?;
        let seed_type = CompilerType::List(Box::new(CompilerType::Int));
        require_type(&self.source, seed.span, &seed_type, &seed.value_type)?;
        let consumed_before_body = self.consumed_generators.clone();
        let (parameters, step) = self.analyze_collection_function(
            parameters,
            step_body,
            std::slice::from_ref(&seed_type),
            environment,
            self.static_context,
            function_span,
        )?;
        if self.consumed_generators != consumed_before_body {
            return Err(unsupported(
                &self.source,
                function_span,
                "generator capture in an unfold operation",
            ));
        }
        let expected = CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            seed_type,
        ])));
        require_type(
            &self.source,
            step.result.span,
            &expected,
            &step.result.value_type,
        )?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::UnfoldGenerator {
                seed: Box::new(seed),
                parameters,
                step: Box::new(step),
            },
            value_type: int_unit_generator_type(),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_character_at(
        &mut self,
        text: &Expression,
        index: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let text_value = self.analyze_expression(text, environment)?;
        require_type(
            &self.source,
            text_value.span,
            &CompilerType::String,
            &text_value.value_type,
        )?;
        let index_value = self.analyze_expression(index, environment)?;
        require_type(
            &self.source,
            index_value.span,
            &CompilerType::Int,
            &index_value.value_type,
        )?;
        let text = Self::known_string_value(&text_value, environment)
            .ok_or_else(|| unsupported(&self.source, text.span(), "dynamic Character indexing"))?;
        let exact_index = exact_int(&index_value)
            .ok_or_else(|| unsupported(&self.source, index.span(), "dynamic Character index"))?;
        let payload = usize::try_from(&exact_index)
            .ok()
            .and_then(|index| character_at(&text, index))
            .map(|character| CompilerExpression {
                kind: CompilerExpressionKind::String(character.to_owned()),
                value_type: CompilerType::Character,
                int_range: None,
                rational_value: None,
                span,
            });
        Ok(CompilerExpression {
            kind: payload.map_or(CompilerExpressionKind::OptionalNone, |character| {
                CompilerExpressionKind::OptionalSome(Box::new(character))
            }),
            value_type: CompilerType::Optional(Box::new(CompilerType::Character)),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_unicode_transform(
        &mut self,
        operation: &str,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand_value = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand_value.span,
            &CompilerType::String,
            &operand_value.value_type,
        )?;
        let text = Self::known_string_value(&operand_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                operand.span(),
                "dynamic Unicode transformation",
            )
        })?;
        let transformed = match operation {
            "upper" => uppercase(&text),
            "lower" => lowercase(&text),
            "case-fold" => case_fold(&text),
            _ => unreachable!("Unicode transform spelling selected above"),
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::String(transformed),
            value_type: CompilerType::String,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_normalization(
        &mut self,
        operand: &Expression,
        form: &str,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand_value = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand_value.span,
            &CompilerType::String,
            &operand_value.value_type,
        )?;
        let text = Self::known_string_value(&operand_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                operand.span(),
                "dynamic Unicode normalization",
            )
        })?;
        let normalized = match form {
            "NFC" => normalize_nfc(&text),
            "NFD" => normalize_nfd(&text),
            _ => {
                return Err(source_diagnostic(
                    &self.source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    span,
                    "the compiler normalization subset requires NFC or NFD",
                ));
            }
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::String(normalized),
            value_type: CompilerType::String,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_canonical_equality(
        &mut self,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let left_value = self.analyze_expression(left, environment)?;
        let right_value = self.analyze_expression(right, environment)?;
        require_type(
            &self.source,
            left_value.span,
            &CompilerType::String,
            &left_value.value_type,
        )?;
        require_type(
            &self.source,
            right_value.span,
            &CompilerType::String,
            &right_value.value_type,
        )?;
        let left_text = Self::known_string_value(&left_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                left.span(),
                "dynamic canonical String equality",
            )
        })?;
        let right_text = Self::known_string_value(&right_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                right.span(),
                "dynamic canonical String equality",
            )
        })?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Boolean(canonically_equal(&left_text, &right_text)),
            value_type: CompilerType::Boolean,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn known_string_value(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<String> {
        Self::known_string_expression(value, environment)
    }

    fn known_closed_int_range(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<ClosedIntRange> {
        match &value.kind {
            CompilerExpressionKind::Binary {
                operation,
                left,
                right,
            } if matches!(
                operation,
                CompilerBinary::Range
                    | CompilerBinary::RangeOpen
                    | CompilerBinary::RangeInclusive
                    | CompilerBinary::RangeOpenInclusive
            ) =>
            {
                let left = left
                    .int_range
                    .as_ref()
                    .filter(|range| range.lower == range.upper)?;
                let right = right
                    .int_range
                    .as_ref()
                    .filter(|range| range.lower == range.upper)?;
                let (lower_inclusive, upper_inclusive) = match operation {
                    CompilerBinary::Range => (true, false),
                    CompilerBinary::RangeOpen => (false, false),
                    CompilerBinary::RangeInclusive => (true, true),
                    CompilerBinary::RangeOpenInclusive => (false, true),
                    _ => unreachable!("guard selected a Range constructor"),
                };
                Some(ClosedIntRange {
                    lower: left.lower.clone(),
                    upper: right.lower.clone(),
                    lower_inclusive,
                    upper_inclusive,
                })
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.closed_int_range.clone()),
            _ => None,
        }
    }

    fn known_string_expression(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<String> {
        match &value.kind {
            CompilerExpressionKind::String(value) => Some(value.clone()),
            CompilerExpressionKind::StringEmpty => Some(String::new()),
            CompilerExpressionKind::StringConcat { left, right } => {
                let mut value = Self::known_string_expression(left, environment)?;
                value.push_str(&Self::known_string_expression(right, environment)?);
                Some(value)
            }
            CompilerExpressionKind::StringCharactersCollect { text, .. } => {
                Self::known_string_expression(text, environment)
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.string_value.clone()),
            CompilerExpressionKind::RecordField { record, label } => {
                Self::known_record_string(record, label, environment)
            }
            _ => None,
        }
    }

    fn known_record_string(
        value: &CompilerExpression,
        label: &str,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<String> {
        Self::known_record_field_facts(value, label, environment)?.string_value
    }

    fn known_record_field_facts(
        value: &CompilerExpression,
        label: &str,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<StaticValueFacts> {
        match &value.kind {
            CompilerExpressionKind::Record(fields) => fields
                .iter()
                .find_map(|(name, value)| (name == label).then_some(value))
                .map(|value| Self::known_value_facts(value, environment)),
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.record_fields.get(label).cloned()),
            CompilerExpressionKind::RecordField {
                record,
                label: outer_label,
            } => Self::known_record_field_facts(record, outer_label, environment)?
                .record_fields
                .get(label)
                .cloned(),
            _ => None,
        }
    }

    fn known_record_fields(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> BTreeMap<String, StaticValueFacts> {
        match &value.kind {
            CompilerExpressionKind::Record(fields) => fields
                .iter()
                .map(|(name, value)| (name.clone(), Self::known_value_facts(value, environment)))
                .collect(),
            CompilerExpressionKind::RecordReconstruct { base, replacements } => {
                let mut fields = Self::known_record_fields(base, environment);
                for (name, value) in replacements {
                    fields.insert(name.clone(), Self::known_value_facts(value, environment));
                }
                fields
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .map_or_else(BTreeMap::new, |facts| facts.record_fields.clone()),
            CompilerExpressionKind::RecordField { record, label } => {
                Self::known_record_field_facts(record, label, environment)
                    .map_or_else(BTreeMap::new, |facts| facts.record_fields)
            }
            _ => BTreeMap::new(),
        }
    }

    fn known_namespace(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        capture_position: usize,
        kind: BlockKind,
    ) -> Result<Option<CompilerNamespaceFacts>, Diagnostic> {
        if value.value_type != CompilerType::Scope {
            return Ok(None);
        }
        if kind != BlockKind::TopLevel {
            return Err(unsupported(
                &self.source,
                value.span,
                "non-root namespace alias binding",
            ));
        }
        self.resolve_namespace(value, environment, capture_position)
    }

    fn resolve_namespace(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        capture_position: usize,
    ) -> Result<Option<CompilerNamespaceFacts>, Diagnostic> {
        match &value.kind {
            CompilerExpressionKind::Root => Ok(Some(CompilerNamespaceFacts {
                name: "root".into(),
                bindings: self.root_bindings.clone(),
                functions: self
                    .functions
                    .iter()
                    .filter_map(|(name, declarations)| {
                        let visible = declarations
                            .iter()
                            .filter(|declaration| declaration.span.end <= capture_position)
                            .cloned()
                            .collect::<Vec<_>>();
                        (!visible.is_empty()).then(|| (name.clone(), visible))
                    })
                    .collect(),
            })),
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.namespace.clone())
                .map(Some)
                .ok_or_else(|| unsupported(&self.source, value.span, "opaque Scope alias")),
            _ => Err(unsupported(
                &self.source,
                value.span,
                "computed Scope alias",
            )),
        }
    }

    fn scope_parameter_arguments(
        &self,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<
        (
            Vec<Option<CompilerNamespaceFacts>>,
            Vec<CompilerContextCapture>,
        ),
        Diagnostic,
    > {
        let mut namespaces = Vec::with_capacity(declaration.parameters.len());
        let mut captures = Vec::new();
        for (parameter, argument) in declaration.parameters.iter().zip(arguments) {
            if self.parse_classifier(parameter.classifier)? != CompilerType::Scope {
                namespaces.push(None);
                continue;
            }
            if self.in_function && matches!(argument.kind, CompilerExpressionKind::Root) {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "function-body live root Scope argument",
                ));
            }
            let mut namespace = self
                .resolve_namespace(argument, environment, argument.span.start)?
                .ok_or_else(|| unsupported(&self.source, argument.span, "opaque Scope argument"))?;
            let parameter_name = self.source.slice(parameter.name);
            if parameter_name == "_" {
                namespaces.push(Some(namespace));
                continue;
            }
            let represented_members = namespace
                .bindings
                .iter()
                .map(|(member_name, facts)| (member_name.clone(), facts.clone()))
                .collect::<Vec<_>>();
            for (member_name, facts) in represented_members {
                if !compiler_function_result_supported(&facts.value_type) {
                    namespace.bindings.remove(&member_name);
                    continue;
                }
                let span = parameter.name;
                let hidden_name = format!("{parameter_name} {member_name}");
                namespace
                    .bindings
                    .get_mut(&member_name)
                    .expect("captured namespace data member exists")
                    .storage_name
                    .clone_from(&hidden_name);
                captures.push(CompilerContextCapture {
                    parameter_name: hidden_name,
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    argument: data_member_expression(&facts, span),
                    span,
                });
            }
            namespaces.push(Some(namespace));
        }
        Ok((namespaces, captures))
    }

    fn known_callable(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        capture_position: usize,
    ) -> Result<Option<CompilerCallableFacts>, Diagnostic> {
        if value.value_type != CompilerType::Function {
            return Ok(None);
        }
        match &value.kind {
            CompilerExpressionKind::FunctionValue(tag) => {
                let index = usize::try_from(*tag).expect("u32 tag fits usize");
                if index >= self.functions.len() {
                    return match index - self.functions.len() {
                        0 => Ok(Some(CompilerCallableFacts::Symbolic(CallableKind::Plus))),
                        1 => Ok(Some(CompilerCallableFacts::Symbolic(CallableKind::Minus))),
                        2 => Ok(Some(CompilerCallableFacts::Symbolic(CallableKind::Compare))),
                        _ => self
                            .anonymous_callables
                            .get(tag)
                            .cloned()
                            .map(Some)
                            .ok_or_else(|| {
                                unsupported(&self.source, value.span, "unknown Function value tag")
                            }),
                    };
                }
                let name = self
                    .functions
                    .keys()
                    .nth(index)
                    .expect("checked Function value tag names a declaration")
                    .clone();
                let declarations = self
                    .functions
                    .get(&name)
                    .expect("checked Function value retains its declarations")
                    .iter()
                    .filter(|declaration| declaration.span.end <= capture_position)
                    .cloned()
                    .collect::<Vec<_>>();
                Ok(Some(CompilerCallableFacts::Named {
                    name,
                    declarations,
                    captures: Vec::new(),
                }))
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.callable.clone())
                .map(Some)
                .ok_or_else(|| unsupported(&self.source, value.span, "opaque Function value")),
            _ => Err(unsupported(
                &self.source,
                value.span,
                "computed Function value",
            )),
        }
    }

    fn known_value_facts(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> StaticValueFacts {
        StaticValueFacts {
            int_range: value.int_range.clone(),
            rational_value: value.rational_value.clone(),
            string_value: Self::known_string_expression(value, environment),
            record_fields: Self::known_record_fields(value, environment),
        }
    }

    fn finish_nat_conversion(
        &self,
        value: CompilerExpression,
        span: Span,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            value.span,
            &CompilerType::Int,
            &value.value_type,
        )?;
        if value
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= BigInt::from(0))
        {
            let int_range = value.int_range.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(value)),
                value_type: CompilerType::Nat,
                int_range,
                rational_value: None,
                span,
            });
        }
        if value
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < BigInt::from(0))
            && compiler_expression_is_closed(&value)
        {
            return Err(source_diagnostic(
                &self.source,
                "E-NAT-OUT-OF-RANGE",
                error_span,
                "a negative Int is outside the Nat constraint",
            ));
        }
        Ok(Self::finish_validation(
            CompilerValidation::IntToNat,
            value,
            CompilerType::Nat,
            span,
            error_span,
        ))
    }

    fn analyze_rational_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Product { fields, .. } = argument
            && fields.len() == 2
            && fields.iter().all(|field| field.label.is_none())
        {
            let numerator = self.analyze_expression(&fields[0].value, environment)?;
            let denominator = self.analyze_expression(&fields[1].value, environment)?;
            require_type(
                &self.source,
                numerator.span,
                &CompilerType::Int,
                &numerator.value_type,
            )?;
            require_type(
                &self.source,
                denominator.span,
                &CompilerType::Int,
                &denominator.value_type,
            )?;
            if is_proven_zero_numeric(&denominator) {
                if compiler_expression_is_closed(&denominator) {
                    if is_proven_zero_numeric(&numerator) {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-INDETERMINATE-RATIONAL",
                            argument.span(),
                            "Rational (0, 0) does not determine one numeric value",
                        ));
                    }
                    return Err(division_by_zero(&self.source, argument.span()));
                }
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Fallible {
                        operation: CompilerFallible::RationalConstruct,
                        left: Box::new(numerator),
                        right: Box::new(denominator),
                        error_span: argument.span(),
                    },
                    value_type: CompilerType::Result(Box::new(CompilerType::Rational)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if !is_proven_nonzero_numeric(&denominator) {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Fallible {
                        operation: CompilerFallible::RationalConstruct,
                        left: Box::new(numerator),
                        right: Box::new(denominator),
                        error_span: argument.span(),
                    },
                    value_type: CompilerType::Result(Box::new(CompilerType::Rational)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            let rational_value = exact_int(&numerator)
                .zip(exact_int(&denominator))
                .map(|(numerator, denominator)| BigRational::new(numerator, denominator));
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::RationalConstruct {
                    numerator: Box::new(numerator),
                    denominator: Box::new(denominator),
                },
                value_type: CompilerType::Rational,
                int_range: None,
                rational_value,
                span,
            });
        }
        let value = self.analyze_expression(argument, environment)?;
        require_type(
            &self.source,
            value.span,
            &CompilerType::Int,
            &value.value_type,
        )?;
        let rational_value = exact_int(&value).map(BigRational::from_integer);
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::IntToRational(Box::new(value)),
            value_type: CompilerType::Rational,
            int_range: None,
            rational_value,
            span,
        })
    }

    fn analyze_range_observation(
        &mut self,
        operation: &str,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand = self.analyze_expression(operand, environment)?;
        if operation == "empty?" && operand.value_type == CompilerType::String {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringEmptyPredicate(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if operation == "empty?"
            && let CompilerType::List(element) = &operand.value_type
        {
            if element.as_ref() != &CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "empty? for this List element type",
                ));
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEmptyPredicate(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        let CompilerType::Range(endpoint) = &operand.value_type else {
            if operation == "empty?" {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "empty? for this value type",
                ));
            }
            return Err(source_diagnostic(
                &self.source,
                "E-TYPE-MISMATCH",
                operand.span,
                format!("{operation} requires a finite exact Range operand"),
            ));
        };
        let endpoint = endpoint.as_ref().clone();
        let (kind, value_type) = match operation {
            "range-lower" => (
                CompilerExpressionKind::RangeLower(Box::new(operand)),
                endpoint,
            ),
            "range-upper" => (
                CompilerExpressionKind::RangeUpper(Box::new(operand)),
                endpoint,
            ),
            "range-lower-inclusive?" => (
                CompilerExpressionKind::RangeLowerInclusive(Box::new(operand)),
                CompilerType::Boolean,
            ),
            "range-upper-inclusive?" => (
                CompilerExpressionKind::RangeUpperInclusive(Box::new(operand)),
                CompilerType::Boolean,
            ),
            "empty?" => (
                CompilerExpressionKind::RangeEmpty(Box::new(operand)),
                CompilerType::Boolean,
            ),
            _ => unreachable!("range observation spelling selected above"),
        };
        Ok(CompilerExpression {
            kind,
            value_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_identifier_binary(
        &mut self,
        operation: &str,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut left = self.analyze_expression(left, environment)?;
        let mut right = self.analyze_expression(right, environment)?;
        let binary = match operation {
            "and" => CompilerBinary::And,
            "or" => CompilerBinary::Or,
            "xor" => CompilerBinary::Xor,
            "in" => CompilerBinary::In,
            "contains" => CompilerBinary::Contains,
            _ => unreachable!("identifier binary spelling selected above"),
        };
        if matches!(binary, CompilerBinary::In | CompilerBinary::Contains) {
            let (range, value) = if binary == CompilerBinary::In {
                (&right.value_type, &mut left)
            } else {
                (&left.value_type, &mut right)
            };
            let CompilerType::Range(endpoint) = range else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-RANGE-MEMBERSHIP-OPERANDS",
                    span,
                    "range membership requires a finite exact Range operand",
                ));
            };
            require_exact_numeric(&self.source, value.span, &value.value_type)?;
            if endpoint.as_ref() == &CompilerType::Rational && value.value_type == CompilerType::Int
            {
                *value = into_rational(value.clone());
            }
            require_same_type(&self.source, value.span, endpoint, &value.value_type)?;
            return Ok(Self::finish_binary(
                binary,
                left,
                right,
                CompilerType::Boolean,
                span,
            ));
        }
        if binary == CompilerBinary::And && matches!(&left.value_type, CompilerType::Range(_)) {
            require_same_type(&self.source, span, &left.value_type, &right.value_type)?;
            let value_type = left.value_type.clone();
            return Ok(Self::finish_binary(binary, left, right, value_type, span));
        }
        if let (
            CompilerExpressionKind::Capability(left),
            CompilerExpressionKind::Capability(right),
        ) = (&left.kind, &right.kind)
        {
            let capability = match binary {
                CompilerBinary::And => left.and(right),
                CompilerBinary::Or => left.or(right),
                _ => {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Capability operator other than and/or",
                    ));
                }
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Capability(capability),
                value_type: CompilerType::Capability,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        require_type(
            &self.source,
            left.span,
            &CompilerType::Boolean,
            &left.value_type,
        )?;
        require_type(
            &self.source,
            right.span,
            &CompilerType::Boolean,
            &right.value_type,
        )?;
        Ok(Self::finish_binary(
            binary,
            left,
            right,
            CompilerType::Boolean,
            span,
        ))
    }

    fn analyze_bound_symbolic_callable(
        &mut self,
        kind: CallableKind,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let [alias, argument] = items else {
            return Err(source_diagnostic(
                &self.source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "a symbolic Function value accepts one direct or product operand",
            ));
        };
        if let Expression::Product { fields, .. } = argument
            && let [left, right] = fields.as_slice()
            && left.label.is_none()
            && right.label.is_none()
        {
            return self.analyze_symbolic_binary(
                kind,
                &left.value,
                &right.value,
                span,
                environment,
            );
        }
        if kind == CallableKind::Minus {
            let application = [
                Expression::Callable {
                    kind,
                    span: alias.span(),
                },
                argument.clone(),
            ];
            return self.analyze_application(&application, span, environment);
        }
        Err(source_diagnostic(
            &self.source,
            "E-NO-APPLICABLE-OVERLOAD",
            argument.span(),
            "a binary symbolic Function value requires a two-field positional product",
        ))
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Retained source identity and call-site evidence are intentionally explicit.
    fn analyze_bound_anonymous_function(
        &mut self,
        parameters: &[AnonymousPattern],
        body: &Expression,
        captures: &BTreeMap<String, BindingFacts>,
        static_context: bool,
        declaration_span: Span,
        items: &[Expression],
        call_span: Span,
        call_environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let [_, argument_source] = items else {
            return Err(source_diagnostic(
                &self.source,
                "E-ANONYMOUS-FUNCTION-ARITY",
                call_span,
                "an anonymous Function value accepts exactly one direct or product operand",
            ));
        };
        if !captures.is_empty() {
            return Err(unsupported(
                &self.source,
                declaration_span,
                "lexically capturing anonymous function",
            ));
        }
        if let Some(pattern_span) = parameters.iter().find_map(|parameter| match parameter {
            AnonymousPattern::Binding(_) => None,
            AnonymousPattern::Product { span, .. } => Some(*span),
        }) {
            return Err(unsupported(
                &self.source,
                pattern_span,
                "anonymous product parameter pattern",
            ));
        }

        let argument = self.analyze_expression(argument_source, call_environment)?;
        let arguments = if parameters.len() == 1 {
            vec![argument]
        } else if let CompilerExpressionKind::Tuple(values) = argument.kind {
            values
        } else if let CompilerType::Tuple(fields) = &argument.value_type {
            if fields.len() != parameters.len() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-ANONYMOUS-FUNCTION-ARITY",
                    argument_source.span(),
                    format!(
                        "anonymous function expects {} arguments, found {}",
                        parameters.len(),
                        fields.len()
                    ),
                ));
            }
            return Err(unsupported(
                &self.source,
                argument_source.span(),
                "opaque anonymous argument product decomposition",
            ));
        } else {
            return Err(source_diagnostic(
                &self.source,
                "E-ANONYMOUS-ARGUMENT-PACKAGE",
                argument_source.span(),
                format!(
                    "anonymous function expects {} arguments packaged as a tuple, found `{}`",
                    parameters.len(),
                    argument.value_type.name()
                ),
            ));
        };
        if arguments.len() != parameters.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-ANONYMOUS-FUNCTION-ARITY",
                argument_source.span(),
                format!(
                    "anonymous function expects {} arguments, found {}",
                    parameters.len(),
                    arguments.len()
                ),
            ));
        }

        let mut environment = BTreeMap::new();
        let mut lowered_parameters = Vec::with_capacity(parameters.len());
        let mut declared = BTreeSet::new();
        for (parameter, argument) in parameters.iter().zip(&arguments) {
            let AnonymousPattern::Binding(name_span) = parameter else {
                unreachable!("anonymous product patterns were rejected above")
            };
            let name = self.source.slice(*name_span).to_owned();
            let discarded = name == "_";
            if !discarded && !declared.insert(name.clone()) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-BINDING",
                    *name_span,
                    format!("`{name}` is already declared in this parameter pattern"),
                ));
            }
            if argument.value_type == CompilerType::Scope
                || !compiler_function_parameter_supported(&argument.value_type)
            {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "unsupported inferred anonymous-function parameter",
                ));
            }
            if !discarded {
                environment.insert(
                    name.clone(),
                    BindingFacts {
                        storage_name: name.clone(),
                        runtime_bound: true,
                        value_type: argument.value_type.clone(),
                        int_range: argument.int_range.clone(),
                        rational_value: argument.rational_value.clone(),
                        string_value: exact_string(argument),
                        closed_int_range: None,
                        record_fields: BTreeMap::new(),
                        namespace: None,
                        callable: self.known_callable(
                            argument,
                            call_environment,
                            argument.span.start,
                        )?,
                        static_capability: None,
                    },
                );
            }
            lowered_parameters.push(CompilerParameter {
                name,
                discarded,
                value_type: argument.value_type.clone(),
                int_range: argument.int_range.clone(),
                span: *name_span,
            });
        }

        let previous_static_context = self.static_context;
        let previous_in_function = self.in_function;
        self.static_context = static_context;
        self.in_function = true;
        let analyzed_body = match body {
            Expression::Block { statements, .. } => {
                self.analyze_block(statements, &mut environment, BlockKind::Function, None)
            }
            expression => self
                .analyze_expression(expression, &environment)
                .map(|result| CompilerBlock {
                    statements: Vec::new(),
                    result,
                }),
        };
        self.static_context = previous_static_context;
        self.in_function = previous_in_function;
        let analyzed_body = analyzed_body?;
        if !compiler_function_result_supported(&analyzed_body.result.value_type) {
            return Err(unsupported(
                &self.source,
                analyzed_body.result.span,
                "unsupported inferred anonymous-function result",
            ));
        }

        let source_name = format!("<anonymous fn/{}>", parameters.len());
        let symbol = self.reserve_function_symbol("anonymous");
        let result_type = analyzed_body.result.value_type.clone();
        let int_range = analyzed_body.result.int_range.clone();
        let rational_value = analyzed_body.result.rational_value.clone();
        self.instances.push(CompilerFunction {
            source_name,
            symbol: symbol.clone(),
            parameters: lowered_parameters,
            result_type: result_type.clone(),
            body: analyzed_body,
            span: declaration_span,
            is_static: static_context,
            declared_effects: None,
        });
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Call { symbol, arguments },
            value_type: result_type,
            int_range,
            rational_value,
            span: call_span,
        })
    }

    #[allow(clippy::too_many_lines)] // Numeric coercion and fail-closed obligations stay in one selection path.
    fn analyze_symbolic_binary(
        &mut self,
        kind: CallableKind,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operation = match kind {
            CallableKind::Plus => CompilerBinary::Add,
            CallableKind::Minus => CompilerBinary::Subtract,
            CallableKind::Multiply => CompilerBinary::Multiply,
            CallableKind::Divide => CompilerBinary::Divide,
            CallableKind::Modulo => CompilerBinary::Modulo,
            CallableKind::QuotientModulo => CompilerBinary::QuotientModulo,
            CallableKind::Power => CompilerBinary::Power,
            CallableKind::Compare => CompilerBinary::Compare,
            CallableKind::Range => CompilerBinary::Range,
            CallableKind::RangeOpen => CompilerBinary::RangeOpen,
            CallableKind::RangeInclusive => CompilerBinary::RangeInclusive,
            CallableKind::RangeOpenInclusive => CompilerBinary::RangeOpenInclusive,
            CallableKind::Equal => CompilerBinary::Equal,
            CallableKind::NotEqual => CompilerBinary::NotEqual,
            CallableKind::Less => CompilerBinary::Less,
            CallableKind::Greater => CompilerBinary::Greater,
            CallableKind::LessEqual => CompilerBinary::LessEqual,
            CallableKind::GreaterEqual => CompilerBinary::GreaterEqual,
        };
        let mut left_value = self.analyze_expression(left, environment)?;
        let mut right_value = self.analyze_expression(right, environment)?;
        if matches!(left_value.value_type, CompilerType::Refined { .. }) {
            left_value = forget_refined_evidence(left_value);
        }
        if matches!(right_value.value_type, CompilerType::Refined { .. }) {
            right_value = forget_refined_evidence(right_value);
        }

        if matches!(left_value.value_type, CompilerType::Modular(_))
            || matches!(right_value.value_type, CompilerType::Modular(_))
        {
            return self.finish_modular_binary(operation, left_value, right_value, span);
        }

        if is_range_construction(operation) {
            require_exact_numeric(&self.source, left_value.span, &left_value.value_type)?;
            require_exact_numeric(&self.source, right_value.span, &right_value.value_type)?;
            let endpoint = if left_value.value_type == CompilerType::Rational
                || right_value.value_type == CompilerType::Rational
            {
                left_value = into_rational(left_value);
                right_value = into_rational(right_value);
                CompilerType::Rational
            } else {
                CompilerType::Int
            };
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Range(Box::new(endpoint)),
                span,
            ));
        }

        if matches!(
            operation,
            CompilerBinary::Modulo | CompilerBinary::QuotientModulo
        ) {
            require_type(
                &self.source,
                left_value.span,
                &CompilerType::Int,
                &left_value.value_type,
            )?;
            require_type(
                &self.source,
                right_value.span,
                &CompilerType::Int,
                &right_value.value_type,
            )?;
            let result_type = if operation == CompilerBinary::Modulo {
                CompilerType::Int
            } else {
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int])
            };
            if is_proven_zero_numeric(&right_value) {
                if compiler_expression_is_closed(&right_value) {
                    return Err(division_by_zero(&self.source, right_value.span));
                }
                return Ok(Self::finish_fallible_binary(
                    if operation == CompilerBinary::Modulo {
                        CompilerFallible::IntModulo
                    } else {
                        CompilerFallible::IntQuotientModulo
                    },
                    left_value,
                    right_value,
                    result_type,
                    span,
                    right.span(),
                ));
            }
            if !is_proven_nonzero_numeric(&right_value) {
                return Ok(Self::finish_fallible_binary(
                    if operation == CompilerBinary::Modulo {
                        CompilerFallible::IntModulo
                    } else {
                        CompilerFallible::IntQuotientModulo
                    },
                    left_value,
                    right_value,
                    result_type,
                    span,
                    right.span(),
                ));
            }
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                result_type,
                span,
            ));
        }

        if operation == CompilerBinary::Power {
            return self.finish_power(left_value, right_value, span);
        }

        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual)
            && (matches!(
                left_value.value_type,
                CompilerType::Tuple(_) | CompilerType::Record(_)
            ) || matches!(
                right_value.value_type,
                CompilerType::Tuple(_) | CompilerType::Record(_)
            ))
        {
            let (left_value, right_value, value_type) =
                self.adapt_structural_equality(left_value, right_value, span)?;
            if !compiler_equality_supported(&value_type) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "equality for this structural value type",
                ));
            }
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Boolean,
                span,
            ));
        }

        if matches!(
            operation,
            CompilerBinary::Less
                | CompilerBinary::Greater
                | CompilerBinary::LessEqual
                | CompilerBinary::GreaterEqual
                | CompilerBinary::Compare
        ) && (matches!(left_value.value_type, CompilerType::Tuple(_))
            || matches!(right_value.value_type, CompilerType::Tuple(_)))
        {
            let (left_value, right_value, value_type) =
                self.adapt_tuple_ordering(left_value, right_value, span)?;
            if !compiler_ordering_supported(&value_type) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "ordering for this structural value type",
                ));
            }
            let result_type = if operation == CompilerBinary::Compare {
                CompilerType::Comparison
            } else {
                CompilerType::Boolean
            };
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                result_type,
                span,
            ));
        }

        let comparison = matches!(
            operation,
            CompilerBinary::Equal
                | CompilerBinary::NotEqual
                | CompilerBinary::Less
                | CompilerBinary::Greater
                | CompilerBinary::LessEqual
                | CompilerBinary::GreaterEqual
                | CompilerBinary::Compare
        );
        if comparison
            && is_exact_comparable(&left_value.value_type)
            && is_exact_comparable(&right_value.value_type)
        {
            left_value = forget_nat_evidence(left_value);
            right_value = forget_nat_evidence(right_value);
        }
        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual) {
            if left_value.value_type == CompilerType::Character
                && right_value.value_type == CompilerType::String
            {
                left_value = forget_character_evidence(left_value);
            } else if left_value.value_type == CompilerType::String
                && right_value.value_type == CompilerType::Character
            {
                right_value = forget_character_evidence(right_value);
            }
        }
        if self.active_nat_recursion()
            && matches!(operation, CompilerBinary::Add | CompilerBinary::Subtract)
        {
            left_value = forget_nat_evidence(left_value);
            right_value = forget_nat_evidence(right_value);
        }
        let numeric =
            is_exact_numeric(&left_value.value_type) && is_exact_numeric(&right_value.value_type);
        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual) && !numeric {
            require_same_type(
                &self.source,
                span,
                &left_value.value_type,
                &right_value.value_type,
            )?;
            if !compiler_equality_supported(&left_value.value_type) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "equality for this value type",
                ));
            }
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Boolean,
                span,
            ));
        }

        require_exact_numeric(&self.source, left_value.span, &left_value.value_type)?;
        require_exact_numeric(&self.source, right_value.span, &right_value.value_type)?;
        let both_int = left_value.value_type == CompilerType::Int
            && right_value.value_type == CompilerType::Int;
        let rational_result = operation == CompilerBinary::Divide
            || left_value.value_type == CompilerType::Rational
            || right_value.value_type == CompilerType::Rational;
        if rational_result {
            left_value = into_rational(left_value);
            right_value = into_rational(right_value);
        }
        if operation == CompilerBinary::Divide && is_proven_zero_numeric(&right_value) {
            if compiler_expression_is_closed(&right_value) {
                return Err(division_by_zero(&self.source, right_value.span));
            }
            if both_int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "dynamic Int division Result",
                ));
            }
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalDivide,
                left_value,
                right_value,
                CompilerType::Rational,
                span,
                right.span(),
            ));
        }
        if operation == CompilerBinary::Divide && !is_proven_nonzero_numeric(&right_value) {
            if both_int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "dynamic Int division Result",
                ));
            }
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalDivide,
                left_value,
                right_value,
                CompilerType::Rational,
                span,
                right.span(),
            ));
        }
        let result_type = match operation {
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply => {
                if rational_result {
                    CompilerType::Rational
                } else {
                    CompilerType::Int
                }
            }
            CompilerBinary::Divide => CompilerType::Rational,
            CompilerBinary::Compare => CompilerType::Comparison,
            CompilerBinary::Equal
            | CompilerBinary::NotEqual
            | CompilerBinary::Less
            | CompilerBinary::Greater
            | CompilerBinary::LessEqual
            | CompilerBinary::GreaterEqual => CompilerType::Boolean,
            _ => unreachable!("numeric operation handled above"),
        };
        debug_assert!(!both_int || !rational_result || operation == CompilerBinary::Divide);
        Ok(Self::finish_binary(
            operation,
            left_value,
            right_value,
            result_type,
            span,
        ))
    }

    fn adapt_structural_equality(
        &self,
        mut left: CompilerExpression,
        mut right: CompilerExpression,
        span: Span,
    ) -> Result<(CompilerExpression, CompilerExpression, CompilerType), Diagnostic> {
        if is_exact_comparable(&left.value_type) && is_exact_comparable(&right.value_type) {
            left = forget_nat_evidence(left);
            right = forget_nat_evidence(right);
            let value_type = if left.value_type == CompilerType::Rational
                || right.value_type == CompilerType::Rational
            {
                left = into_rational(left);
                right = into_rational(right);
                CompilerType::Rational
            } else {
                CompilerType::Int
            };
            return Ok((left, right, value_type));
        }
        if left.value_type == CompilerType::Character && right.value_type == CompilerType::String {
            left = forget_character_evidence(left);
        } else if left.value_type == CompilerType::String
            && right.value_type == CompilerType::Character
        {
            right = forget_character_evidence(right);
        }
        if left.value_type == right.value_type {
            if compiler_equality_supported(&left.value_type) {
                let value_type = left.value_type.clone();
                return Ok((left, right, value_type));
            }
            return Err(
                self.no_structural_comparison(span, "corresponding fields have no common equality")
            );
        }

        match (left.value_type.clone(), right.value_type.clone()) {
            (CompilerType::Tuple(left_types), CompilerType::Tuple(right_types))
                if left_types.len() == right_types.len() =>
            {
                let (
                    CompilerExpressionKind::Tuple(left_fields),
                    CompilerExpressionKind::Tuple(right_fields),
                ) = (&mut left.kind, &mut right.kind)
                else {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "structural equality conversion on an opaque tuple",
                    ));
                };
                let mut field_types = Vec::with_capacity(left_fields.len());
                for index in 0..left_fields.len() {
                    let (left_field, right_field, field_type) = self.adapt_structural_equality(
                        left_fields[index].clone(),
                        right_fields[index].clone(),
                        span,
                    )?;
                    left_fields[index] = left_field;
                    right_fields[index] = right_field;
                    field_types.push(field_type);
                }
                let value_type = CompilerType::Tuple(field_types);
                left.value_type = value_type.clone();
                right.value_type = value_type.clone();
                Ok((left, right, value_type))
            }
            (CompilerType::Record(left_types), CompilerType::Record(right_types)) => {
                self.adapt_record_equality(left, right, left_types, &right_types, span)
            }
            _ => {
                Err(self
                    .no_structural_comparison(span, "corresponding fields have no common equality"))
            }
        }
    }

    fn adapt_record_equality(
        &self,
        mut left: CompilerExpression,
        mut right: CompilerExpression,
        left_types: Vec<(String, CompilerType)>,
        right_types: &[(String, CompilerType)],
        span: Span,
    ) -> Result<(CompilerExpression, CompilerExpression, CompilerType), Diagnostic> {
        let left_labels = left_types
            .iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>();
        let right_labels = right_types
            .iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>();
        if left_labels != right_labels {
            return Err(self.no_structural_comparison(span, "record shapes differ"));
        }
        let (
            CompilerExpressionKind::Record(left_fields),
            CompilerExpressionKind::Record(right_fields),
        ) = (&mut left.kind, &mut right.kind)
        else {
            return Err(unsupported(
                &self.source,
                span,
                "structural equality conversion on an opaque record",
            ));
        };
        let mut field_types = Vec::with_capacity(left_types.len());
        for (label, _) in left_types {
            let left_index = left_fields
                .iter()
                .position(|(name, _)| name == &label)
                .expect("checked left Record retains its type labels");
            let right_index = right_fields
                .iter()
                .position(|(name, _)| name == &label)
                .expect("checked right Record retains its type labels");
            let (left_field, right_field, field_type) = self.adapt_structural_equality(
                left_fields[left_index].1.clone(),
                right_fields[right_index].1.clone(),
                span,
            )?;
            left_fields[left_index].1 = left_field;
            right_fields[right_index].1 = right_field;
            field_types.push((label, field_type));
        }
        let value_type = CompilerType::Record(field_types);
        left.value_type = value_type.clone();
        right.value_type = value_type.clone();
        Ok((left, right, value_type))
    }

    fn adapt_tuple_ordering(
        &self,
        mut left: CompilerExpression,
        mut right: CompilerExpression,
        span: Span,
    ) -> Result<(CompilerExpression, CompilerExpression, CompilerType), Diagnostic> {
        if is_exact_comparable(&left.value_type) && is_exact_comparable(&right.value_type) {
            left = forget_nat_evidence(left);
            right = forget_nat_evidence(right);
            let value_type = if left.value_type == CompilerType::Rational
                || right.value_type == CompilerType::Rational
            {
                left = into_rational(left);
                right = into_rational(right);
                CompilerType::Rational
            } else {
                CompilerType::Int
            };
            return Ok((left, right, value_type));
        }
        if left.value_type == right.value_type {
            if compiler_ordering_supported(&left.value_type) {
                let value_type = left.value_type.clone();
                return Ok((left, right, value_type));
            }
            return Err(self.no_structural_comparison(
                span,
                "corresponding fields have no common total order",
            ));
        }
        let (CompilerType::Tuple(left_types), CompilerType::Tuple(right_types)) =
            (left.value_type.clone(), right.value_type.clone())
        else {
            return Err(self.no_structural_comparison(
                span,
                "corresponding fields have no common total order",
            ));
        };
        if left_types.len() != right_types.len() {
            return Err(self.no_structural_comparison(span, "tuple arities differ"));
        }
        let (
            CompilerExpressionKind::Tuple(left_fields),
            CompilerExpressionKind::Tuple(right_fields),
        ) = (&mut left.kind, &mut right.kind)
        else {
            return Err(unsupported(
                &self.source,
                span,
                "structural ordering conversion on an opaque tuple",
            ));
        };
        let mut field_types = Vec::with_capacity(left_fields.len());
        for index in 0..left_fields.len() {
            let (left_field, right_field, field_type) = self.adapt_tuple_ordering(
                left_fields[index].clone(),
                right_fields[index].clone(),
                span,
            )?;
            left_fields[index] = left_field;
            right_fields[index] = right_field;
            field_types.push(field_type);
        }
        let value_type = CompilerType::Tuple(field_types);
        left.value_type = value_type.clone();
        right.value_type = value_type.clone();
        Ok((left, right, value_type))
    }

    fn no_structural_comparison(&self, span: Span, reason: &str) -> Diagnostic {
        source_diagnostic(
            &self.source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            format!("no structural comparison applies: {reason}"),
        )
    }

    fn finish_power(
        &self,
        left: CompilerExpression,
        right: CompilerExpression,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_exact_numeric(&self.source, left.span, &left.value_type)?;
        require_type(
            &self.source,
            right.span,
            &CompilerType::Int,
            &right.value_type,
        )?;
        let exponent = exact_int(&right);
        if left.value_type == CompilerType::Int {
            let Some(ref exponent) = exponent else {
                return Err(unsupported(
                    &self.source,
                    right.span,
                    "Int power with an exponent not proven to satisfy Nat",
                ));
            };
            if exponent < &BigInt::from(0) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-TYPE-MISMATCH",
                    right.span,
                    "an Int exponent must satisfy Nat",
                ));
            }
        }
        let can_fail = left.value_type == CompilerType::Rational
            && !is_proven_nonzero_numeric(&left)
            && exponent
                .as_ref()
                .is_none_or(|value| value < &BigInt::from(0));
        if can_fail && is_proven_zero_numeric(&left) && compiler_expression_is_closed(&left) {
            return Err(division_by_zero(&self.source, left.span));
        }
        if can_fail {
            let error_span = left.span;
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalPower,
                left,
                right,
                CompilerType::Rational,
                span,
                error_span,
            ));
        }
        let result_type = left.value_type.clone();
        Ok(Self::finish_binary(
            CompilerBinary::Power,
            left,
            right,
            result_type,
            span,
        ))
    }

    fn finish_modular_binary(
        &self,
        operation: CompilerBinary,
        left: CompilerExpression,
        right: CompilerExpression,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_same_type(&self.source, span, &left.value_type, &right.value_type)?;
        let CompilerType::Modular(modular) = &left.value_type else {
            unreachable!("same-type modular operation has two modular operands")
        };
        let (value_type, int_range) = match operation {
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply => {
                let exact = exact_int(&left)
                    .zip(exact_int(&right))
                    .map(|(left, right)| {
                        let value = match operation {
                            CompilerBinary::Add => left + right,
                            CompilerBinary::Subtract => left - right,
                            CompilerBinary::Multiply => left * right,
                            _ => unreachable!("selected modular arithmetic operation"),
                        };
                        IntRange::exact(reduce_modular(value, modular))
                    });
                (
                    left.value_type.clone(),
                    exact.or_else(|| Some(modular_range(modular))),
                )
            }
            CompilerBinary::Equal
            | CompilerBinary::NotEqual
            | CompilerBinary::Less
            | CompilerBinary::Greater
            | CompilerBinary::LessEqual
            | CompilerBinary::GreaterEqual => (CompilerType::Boolean, None),
            CompilerBinary::Compare => (CompilerType::Comparison, None),
            _ => {
                return Err(unsupported(
                    &self.source,
                    span,
                    "operation over modular values",
                ));
            }
        };
        let mut result = Self::finish_binary(operation, left, right, value_type, span);
        result.int_range = int_range;
        Ok(result)
    }

    fn finish_binary(
        operation: CompilerBinary,
        left: CompilerExpression,
        right: CompilerExpression,
        value_type: CompilerType,
        span: Span,
    ) -> CompilerExpression {
        let int_range = match (&value_type, operation) {
            (CompilerType::Int, CompilerBinary::Add) => {
                combine_ranges(&left, &right, |a, b| IntRange {
                    lower: &a.lower + &b.lower,
                    upper: &a.upper + &b.upper,
                })
            }
            (CompilerType::Int, CompilerBinary::Subtract) => {
                combine_ranges(&left, &right, |a, b| IntRange {
                    lower: &a.lower - &b.upper,
                    upper: &a.upper - &b.lower,
                })
            }
            (CompilerType::Int, CompilerBinary::Multiply) => {
                combine_ranges(&left, &right, multiply_range)
            }
            (CompilerType::Int, CompilerBinary::Modulo) => modulo_range(&left, &right),
            (CompilerType::Int, CompilerBinary::Power) => power_range(&left, &right),
            _ => None,
        };
        let rational_value = (value_type == CompilerType::Rational)
            .then(|| exact_rational_binary(operation, &left, &right))
            .flatten();
        CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation,
                left: Box::new(left),
                right: Box::new(right),
            },
            value_type,
            int_range,
            rational_value,
            span,
        }
    }

    fn finish_fallible_binary(
        operation: CompilerFallible,
        left: CompilerExpression,
        right: CompilerExpression,
        success_type: CompilerType,
        span: Span,
        error_span: Span,
    ) -> CompilerExpression {
        CompilerExpression {
            kind: CompilerExpressionKind::Fallible {
                operation,
                left: Box::new(left),
                right: Box::new(right),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(success_type)),
            int_range: None,
            rational_value: None,
            span,
        }
    }

    fn finish_validation(
        operation: CompilerValidation,
        value: CompilerExpression,
        success_type: CompilerType,
        span: Span,
        error_span: Span,
    ) -> CompilerExpression {
        CompilerExpression {
            kind: CompilerExpressionKind::Validate {
                operation,
                value: Box::new(value),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(success_type)),
            int_range: None,
            rational_value: None,
            span,
        }
    }

    fn analyze_call(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let function = items.iter().enumerate().find_map(|(index, item)| {
            let Expression::Identifier(name) = item else {
                return None;
            };
            let name = self.source.slice(*name);
            (!environment.contains_key(name) && self.functions.contains_key(name))
                .then_some((index, name.to_owned()))
        });
        let Some((function_index, function_name)) = function else {
            return Err(unsupported(&self.source, span, "application"));
        };
        self.analyze_resolved_call(items, span, environment, function_index, &function_name)
    }

    fn analyze_resolved_call(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        function_index: usize,
        function_name: &str,
    ) -> Result<CompilerExpression, Diagnostic> {
        let declarations = self
            .functions
            .get(function_name)
            .expect("selected overload set exists")
            .clone();
        self.analyze_resolved_call_from(
            items,
            span,
            environment,
            function_index,
            function_name,
            &declarations,
            &[],
        )
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Candidate-specific source packages and captures remain visibly fail-closed.
    fn analyze_resolved_call_from(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        function_index: usize,
        function_name: &str,
        declarations: &[FunctionSource],
        lexical_captures: &[CompilerContextCapture],
    ) -> Result<CompilerExpression, Diagnostic> {
        let argument_sources = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (index != function_index).then_some(item))
            .collect::<Vec<_>>();
        let arguments = argument_sources
            .iter()
            .map(|argument| self.analyze_expression(argument, environment))
            .collect::<Result<Vec<_>, _>>()?;
        let flattened_arguments = flattened_product_arguments(&argument_sources, &arguments);

        let mut selected = None;
        let static_context = self.static_context;
        for declaration in declarations
            .iter()
            .filter(|declaration| !static_context || declaration.is_static)
        {
            if declaration
                .parameters
                .iter()
                .any(|parameter| !parameter.fields.is_empty())
            {
                if let Some((normalized, adapted)) =
                    self.normalize_packaged_call(declaration, &arguments)?
                {
                    selected = Some((normalized, adapted));
                    break;
                }
                continue;
            }
            let candidate_arguments = if declaration.parameters.is_empty()
                && matches!(argument_sources.as_slice(), [Expression::Unit(_)])
            {
                &[][..]
            } else if declaration.parameters.len() > 1
                && let Some(flattened) = &flattened_arguments
            {
                flattened.as_slice()
            } else {
                arguments.as_slice()
            };
            if candidate_arguments.len() != declaration.parameters.len() {
                continue;
            }
            let mut adapted = Vec::with_capacity(candidate_arguments.len());
            for (parameter_index, (parameter, argument)) in declaration
                .parameters
                .iter()
                .zip(candidate_arguments)
                .enumerate()
            {
                if !parameter.fields.is_empty()
                    || parameter.default.is_some()
                    || parameter.qualifier.is_some()
                {
                    return Err(unsupported(
                        &self.source,
                        parameter.name,
                        "packaged, defaulted, or qualified parameter",
                    ));
                }
                let expected = self.parse_classifier(parameter.classifier)?;
                let Some(argument) = adapt_call_argument(&expected, argument).or_else(|| {
                    self.adapt_proven_recursive_nat_argument(
                        function_name,
                        declaration,
                        parameter_index,
                        &expected,
                        argument,
                    )
                }) else {
                    adapted.clear();
                    break;
                };
                adapted.push(argument);
            }
            if adapted.len() == declaration.parameters.len() {
                selected = Some((declaration.clone(), adapted));
                break;
            }
        }
        let Some((declaration, arguments)) = selected else {
            let actual = arguments
                .iter()
                .map(|argument| argument.value_type.name())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(source_diagnostic(
                &self.source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("no overload of `{function_name}` accepts ({actual}) in this context"),
            ));
        };
        let callable_arguments = arguments
            .iter()
            .map(|argument| self.known_callable(argument, environment, argument.span.start))
            .collect::<Result<Vec<_>, _>>()?;
        let (scope_arguments, scope_captures) =
            self.scope_parameter_arguments(&declaration, &arguments, environment)?;
        let context_captures = self.defining_context_captures(&declaration)?;
        if self.in_function && !context_captures.is_empty() {
            return Err(unsupported(
                &self.source,
                span,
                "cross-function defining-context capture forwarding",
            ));
        }
        let metadata = CompilerCallMetadata {
            callable_arguments,
            scope_arguments,
            scope_captures,
            lexical_captures: lexical_captures.to_vec(),
            context_captures,
        };
        let mut arguments = arguments;
        arguments.extend(
            metadata
                .scope_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        arguments.extend(
            metadata
                .lexical_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        arguments.extend(
            metadata
                .context_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        self.finish_selected_call(function_name, &declaration, arguments, &metadata, span)
    }

    #[allow(clippy::too_many_lines)] // Every admitted and deferred package shape is checked explicitly.
    fn normalize_packaged_call(
        &mut self,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
    ) -> Result<Option<(FunctionSource, Vec<CompilerExpression>)>, Diagnostic> {
        let [package] = declaration.parameters.as_slice() else {
            return Err(unsupported(
                &self.source,
                declaration.span,
                "multiple packaged function operands",
            ));
        };
        if package.fields.is_empty() {
            return Ok(None);
        }
        if package.qualifier.is_some() || package.default.is_some() {
            return Err(unsupported(
                &self.source,
                package.name,
                "qualified or defaulted outer package",
            ));
        }
        let mut declared_fields = BTreeSet::new();
        for field in &package.fields {
            let name = self.source.slice(field.name);
            if name != "_" && !declared_fields.insert(name) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-FUNCTION-PARAMETER",
                    field.name,
                    format!("parameter `{name}` is already declared in this function"),
                ));
            }
        }
        let [argument] = arguments else {
            return Ok(None);
        };

        let supplied = match &argument.kind {
            CompilerExpressionKind::Record(values) => {
                let declared_names = package
                    .fields
                    .iter()
                    .map(|field| self.source.slice(field.name))
                    .collect::<Vec<_>>();
                if values
                    .iter()
                    .any(|(label, _)| !declared_names.contains(&label.as_str()))
                    || package.fields.iter().any(|field| {
                        field.default.is_none()
                            && !values
                                .iter()
                                .any(|(label, _)| label == self.source.slice(field.name))
                    })
                {
                    return Ok(None);
                }
                if values.len() > package.fields.len()
                    || values
                        .iter()
                        .zip(&package.fields)
                        .any(|((label, _), field)| label != self.source.slice(field.name))
                {
                    return Err(unsupported(
                        &self.source,
                        argument.span,
                        "non-prefix or reordered labeled function package",
                    ));
                }
                values
                    .iter()
                    .map(|(_, value)| value.clone())
                    .collect::<Vec<_>>()
            }
            CompilerExpressionKind::Tuple(values) if values.len() == package.fields.len() => {
                values.clone()
            }
            _ if matches!(
                argument.value_type,
                CompilerType::Tuple(_) | CompilerType::Record(_)
            ) =>
            {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "opaque packaged function operand",
                ));
            }
            _ => return Ok(None),
        };

        let mut adapted = Vec::with_capacity(package.fields.len());
        for (index, field) in package.fields.iter().enumerate() {
            if field.qualifier.is_some() || !field.fields.is_empty() {
                return Err(unsupported(
                    &self.source,
                    field.name,
                    "nested or qualified packaged field",
                ));
            }
            let expected = self.parse_classifier(field.classifier)?;
            if expected == CompilerType::Scope
                || !expected.machine_scalar()
                || !compiler_function_parameter_supported(&expected)
            {
                return Err(unsupported(
                    &self.source,
                    field.classifier,
                    "non-scalar packaged field",
                ));
            }
            let value = if let Some(value) = supplied.get(index) {
                value.clone()
            } else {
                let Some(default) = &field.default else {
                    return Ok(None);
                };
                let value = match self.analyze_expression(default, &BTreeMap::new()) {
                    Ok(value) => value,
                    Err(error) if error.code == "E-UNBOUND-NAME" => {
                        return Err(unsupported(
                            &self.source,
                            default.span(),
                            "non-closed packaged field default",
                        ));
                    }
                    Err(error) => return Err(error),
                };
                if !compiler_expression_is_closed(&value) {
                    return Err(unsupported(
                        &self.source,
                        default.span(),
                        "non-closed packaged field default",
                    ));
                }
                value
            };
            let Some(value) = adapt_call_argument(&expected, &value) else {
                return Ok(None);
            };
            adapted.push(value);
        }

        let mut normalized = declaration.clone();
        normalized.parameters = package
            .fields
            .iter()
            .cloned()
            .map(|mut field| {
                field.default = None;
                field
            })
            .collect();
        Ok(Some((normalized, adapted)))
    }

    fn defining_context_captures(
        &self,
        declaration: &FunctionSource,
    ) -> Result<Vec<CompilerContextCapture>, Diagnostic> {
        let mut captures = self
            .root_bindings
            .iter()
            .filter_map(|(member_name, facts)| {
                (facts.declaration_end <= declaration.span.start)
                    .then(|| {
                        function_body_context_member_span(
                            &self.source,
                            &declaration.body,
                            member_name,
                        )
                    })
                    .flatten()
                    .map(|span| (member_name, facts, span))
            })
            .collect::<Vec<_>>();
        captures.sort_by_key(|(_, facts, _)| facts.declaration_end);
        captures
            .into_iter()
            .map(|(member_name, facts, span)| {
                if !facts.value_type.machine_scalar()
                    || !compiler_function_result_supported(&facts.value_type)
                {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "non-scalar defining-context capture",
                    ));
                }
                Ok(CompilerContextCapture {
                    parameter_name: format!("@ {member_name}"),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    argument: data_member_expression(facts, span),
                    span,
                })
            })
            .collect()
    }

    fn finish_selected_call(
        &mut self,
        function_name: &str,
        declaration: &FunctionSource,
        arguments: Vec<CompilerExpression>,
        metadata: &CompilerCallMetadata,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        let identity = function_overload_identity(&self.source, function_name, declaration);
        let recursion_proof = self.recursion_proof(function_name, declaration);
        if self.active_calls.contains(&identity) {
            if let Some(active) = self.active_recursive_functions.get(&identity).cloned()
                && ((self.active_calls.last() == Some(&identity)
                    && active.proof.mutual_target.is_none())
                    || self.closes_proven_mutual_bounded_cycle(&identity, function_name))
            {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Call {
                        symbol: active.symbol,
                        arguments,
                    },
                    value_type: active.result_type,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            return Err(unsupported(&self.source, span, "recursive function call"));
        }
        let reserved_symbol = if let Some(proof) = recursion_proof.clone() {
            let symbol = self.reserve_function_symbol(function_name);
            let result_type = self.parse_classifier(declaration.result)?;
            self.active_recursive_functions.insert(
                identity.clone(),
                ActiveRecursiveFunction {
                    source_name: function_name.to_owned(),
                    symbol: symbol.clone(),
                    result_type,
                    proof,
                },
            );
            Some(symbol)
        } else {
            None
        };
        self.active_calls.push(identity.clone());
        let result = self.instantiate_function(
            function_name,
            declaration,
            &arguments,
            metadata,
            reserved_symbol.as_deref(),
            recursion_proof.is_some(),
        );
        let popped = self.active_calls.pop();
        debug_assert_eq!(popped.as_deref(), Some(identity.as_str()));
        self.active_recursive_functions.remove(&identity);
        let (symbol, result_type, int_range, rational_value) = result?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Call { symbol, arguments },
            value_type: result_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Specialization keeps ABI and checked evidence decisions together.
    fn instantiate_function(
        &mut self,
        function_name: &str,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
        metadata: &CompilerCallMetadata,
        reserved_symbol: Option<&str>,
        generalize_parameters: bool,
    ) -> Result<(String, CompilerType, Option<IntRange>, Option<BigRational>), Diagnostic> {
        let CompilerCallMetadata {
            callable_arguments,
            scope_arguments,
            scope_captures,
            lexical_captures,
            context_captures,
        } = metadata;
        let mut environment = BTreeMap::new();
        let mut parameters = Vec::new();
        for (parameter_index, (parameter, argument)) in
            declaration.parameters.iter().zip(arguments).enumerate()
        {
            if !parameter.fields.is_empty()
                || parameter.default.is_some()
                || parameter.qualifier.is_some()
            {
                return Err(unsupported(
                    &self.source,
                    parameter.name,
                    "packaged, defaulted, or qualified parameter",
                ));
            }
            let expected = self.parse_classifier(parameter.classifier)?;
            require_same_type(
                &self.source,
                parameter.classifier,
                &expected,
                &argument.value_type,
            )?;
            if !compiler_function_parameter_supported(&expected) {
                return Err(unsupported(
                    &self.source,
                    parameter.classifier,
                    "unsupported function parameter",
                ));
            }
            let name = self.source.slice(parameter.name).to_owned();
            let discarded = name == "_";
            if !discarded {
                environment.insert(
                    name.clone(),
                    BindingFacts {
                        storage_name: name.clone(),
                        runtime_bound: true,
                        value_type: expected.clone(),
                        int_range: (!generalize_parameters)
                            .then(|| argument.int_range.clone())
                            .flatten(),
                        rational_value: (!generalize_parameters)
                            .then(|| argument.rational_value.clone())
                            .flatten(),
                        string_value: (!generalize_parameters)
                            .then(|| exact_string(argument))
                            .flatten(),
                        closed_int_range: None,
                        record_fields: BTreeMap::new(),
                        namespace: scope_arguments[parameter_index].clone(),
                        callable: callable_arguments[parameter_index].clone(),
                        static_capability: None,
                    },
                );
            }
            parameters.push(CompilerParameter {
                name,
                discarded,
                value_type: expected,
                int_range: (!generalize_parameters)
                    .then(|| argument.int_range.clone())
                    .flatten(),
                span: parameter.name,
            });
        }
        let scope_arguments_start = declaration.parameters.len();
        let lexical_arguments_start = scope_arguments_start + scope_captures.len();
        let context_arguments_start = lexical_arguments_start + lexical_captures.len();
        let captured_scope_arguments = &arguments[scope_arguments_start..lexical_arguments_start];
        debug_assert_eq!(captured_scope_arguments.len(), scope_captures.len());
        for (capture, argument) in scope_captures.iter().zip(captured_scope_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    string_value: exact_string(argument),
                    closed_int_range: None,
                    record_fields: BTreeMap::new(),
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let captured_lexical_arguments =
            &arguments[lexical_arguments_start..context_arguments_start];
        debug_assert_eq!(captured_lexical_arguments.len(), lexical_captures.len());
        for (capture, argument) in lexical_captures.iter().zip(captured_lexical_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    string_value: exact_string(argument),
                    closed_int_range: None,
                    record_fields: BTreeMap::new(),
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let context_arguments = &arguments[context_arguments_start..];
        debug_assert_eq!(context_arguments.len(), context_captures.len());
        for (capture, argument) in context_captures.iter().zip(context_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    string_value: exact_string(argument),
                    closed_int_range: None,
                    record_fields: BTreeMap::new(),
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let result_type = self.parse_classifier(declaration.result)?;
        if !compiler_function_result_supported(&result_type) {
            return Err(unsupported(
                &self.source,
                declaration.result,
                "non-scalar function result",
            ));
        }
        let generator_parameters = parameters
            .iter()
            .filter(|parameter| is_character_unit_generator_type(&parameter.value_type))
            .collect::<Vec<_>>();
        let character_generator_parameter = if generator_parameters.is_empty() {
            None
        } else {
            let parameter = generator_parameters[0];
            if generator_parameters.len() != 1
                || parameters.len() != 1
                || parameter.discarded
                || declaration.is_static
            {
                return Err(unsupported(
                    &self.source,
                    parameter.span,
                    "Generator parameter behavior beyond one ordinary owned specialization",
                ));
            }
            Some(parameter.clone())
        };
        let scoped_generator_value = if let Some(parameter) = &character_generator_parameter {
            if self.in_function {
                return Err(unsupported(
                    &self.source,
                    parameter.span,
                    "nested Character Generator parameter transfer",
                ));
            }
            let argument = &arguments[0];
            let CompilerExpressionKind::Local(storage_name) = &argument.kind else {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "non-local Character Generator parameter transfer",
                ));
            };
            let retained = self
                .generator_values
                .get(storage_name)
                .cloned()
                .ok_or_else(|| {
                    unsupported(
                        &self.source,
                        argument.span,
                        "Character Generator parameter without closed provenance",
                    )
                })?;
            let previous = self
                .generator_values
                .insert(parameter.name.clone(), retained);
            Some((parameter.name.clone(), previous))
        } else {
            None
        };
        let previous_static_context = self.static_context;
        let previous_in_function = self.in_function;
        let consumed_before_body = self.consumed_generators.clone();
        self.static_context = declaration.is_static;
        self.in_function = true;
        let body = self.analyze_block(
            &declaration.body,
            &mut environment,
            BlockKind::Function,
            Some(&result_type),
        );
        self.static_context = previous_static_context;
        self.in_function = previous_in_function;
        let character_generator_was_consumed = character_generator_parameter
            .as_ref()
            .is_some_and(|parameter| self.consumed_generators.contains(&parameter.name));
        if character_generator_parameter.is_some() {
            self.consumed_generators = consumed_before_body;
        }
        if let Some((name, previous)) = scoped_generator_value {
            if let Some(previous) = previous {
                self.generator_values.insert(name, previous);
            } else {
                self.generator_values.remove(&name);
            }
        }
        let mut body = body?;
        if let Some(parameter) = character_generator_parameter {
            let traverses_parameter = matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::StringCharactersForeach { source, .. },
                    ..
                })] if matches!(source.kind, CompilerExpressionKind::Local(ref name) if name == &parameter.name)
            ) && matches!(body.result.kind, CompilerExpressionKind::Unit);
            let closes_parameter = !character_generator_was_consumed
                && body.statements.is_empty()
                && matches!(body.result.kind, CompilerExpressionKind::Unit);
            if character_generator_was_consumed && traverses_parameter {
                // Exhaustion consumes the transferred continuation; no close is delivered.
            } else if closes_parameter {
                let close_span = body.result.span;
                body.statements
                    .push(CompilerStatement::Discard(CompilerExpression {
                        kind: CompilerExpressionKind::StringCharactersClose(Box::new(
                            CompilerExpression {
                                kind: CompilerExpressionKind::Local(parameter.name),
                                value_type: parameter.value_type,
                                int_range: None,
                                rational_value: None,
                                span: parameter.span,
                            },
                        )),
                        value_type: CompilerType::Unit,
                        int_range: None,
                        rational_value: None,
                        span: close_span,
                    }));
            } else {
                return Err(unsupported(
                    &self.source,
                    parameter.span,
                    "Generator parameter behavior beyond one traversal or implicit built-in close",
                ));
            }
        }
        if let CompilerType::Result(success_type) = &result_type
            && body.result.value_type == **success_type
        {
            let result = body.result;
            let span = result.span;
            body.result = CompilerExpression {
                kind: CompilerExpressionKind::ResultSuccess(Box::new(result)),
                value_type: result_type.clone(),
                int_range: None,
                rational_value: None,
                span,
            };
        } else {
            require_same_type(
                &self.source,
                declaration.result,
                &result_type,
                &body.result.value_type,
            )?;
        }
        let symbol = reserved_symbol.map_or_else(
            || self.reserve_function_symbol(function_name),
            str::to_owned,
        );
        let int_range = body.result.int_range.clone();
        let rational_value = body.result.rational_value.clone();
        self.instances.push(CompilerFunction {
            source_name: function_name.to_owned(),
            symbol: symbol.clone(),
            parameters,
            result_type: result_type.clone(),
            body,
            span: declaration.span,
            is_static: declaration.is_static,
            declared_effects: declaration.declared_effects.clone(),
        });
        Ok((symbol, result_type, int_range, rational_value))
    }

    fn reserve_function_symbol(&mut self, function_name: &str) -> String {
        let symbol = format!("topal.fn.{}.{}", mangle(function_name), self.next_instance);
        self.next_instance += 1;
        symbol
    }

    fn recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        self.explicit_measure_recursion_proof(function_name, declaration)
            .or_else(|| self.direct_bounded_recursion_proof(function_name, declaration))
            .or_else(|| self.mutual_bounded_recursion_proof(function_name, declaration))
    }

    fn direct_bounded_recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        let [parameter] = declaration.parameters.as_slice() else {
            return None;
        };
        let classifier = compact_classifier(self.source.slice(parameter.classifier));
        if !matches!(classifier.as_str(), "Int" | "Nat") {
            return None;
        }
        let parameters = vec![(self.source.slice(parameter.name).to_owned(), classifier)];
        let Some(
            rule @ ("TOPAL-FUNCTION-RECURSION-INT-001"
            | "TOPAL-FUNCTION-RECURSION-INT-INCREASING-001"
            | "TOPAL-FUNCTION-RECURSION-NAT-001"
            | "TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"),
        ) = prove_int_recursion(&self.source, function_name, &parameters, &declaration.body)
        else {
            return None;
        };
        let nat_step_parameters = (parameters[0].1 == "Nat")
            .then_some(0)
            .into_iter()
            .collect();
        Some(CompilerRecursionProof {
            rule,
            nat_step_parameters,
            mutual_target: None,
        })
    }

    fn explicit_measure_recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        let effect_bound = self.source.slice(declaration.effect_bound?);
        let parameters = declaration
            .parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    compact_classifier(self.source.slice(parameter.classifier)),
                )
            })
            .collect::<Vec<_>>();
        let rule = prove_explicit_parameter_recursion(
            &self.source,
            function_name,
            &parameters,
            Some(effect_bound),
            &declaration.body,
        )?;
        if rule != "TOPAL-FUNCTION-DECREASES-001" {
            return None;
        }
        let measure = explicit_single_measure(effect_bound)?;
        let measure_index = parameters.iter().position(|(name, _)| name == measure)?;
        let nat_step_parameters = (parameters[measure_index].1 == "Nat")
            .then_some(measure_index)
            .into_iter()
            .collect();
        Some(CompilerRecursionProof {
            rule,
            nat_step_parameters,
            mutual_target: None,
        })
    }

    fn mutual_bounded_recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        let parameters = declaration
            .parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    compact_classifier(self.source.slice(parameter.classifier)),
                )
            })
            .collect::<Vec<_>>();
        let (mutual_target, rule) = prove_mutual_bounded_recursion_edge(
            &self.source,
            function_name,
            &parameters,
            &declaration.body,
        )?;
        if !matches!(
            rule,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001"
                | "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001"
                | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
        ) {
            return None;
        }
        let nat_step_parameters = (parameters[0].1 == "Nat")
            .then_some(0)
            .into_iter()
            .collect();
        Some(CompilerRecursionProof {
            rule,
            nat_step_parameters,
            mutual_target: Some(mutual_target),
        })
    }

    fn closes_proven_mutual_bounded_cycle(&self, target_identity: &str, target_name: &str) -> bool {
        let Some(cycle_start) = self
            .active_calls
            .iter()
            .position(|identity| identity == target_identity)
        else {
            return false;
        };
        let cycle = &self.active_calls[cycle_start..];
        if cycle.len() < 2 {
            return false;
        }
        let Some(target) = self.active_recursive_functions.get(target_identity) else {
            return false;
        };
        let rule = target.proof.rule;
        if target.source_name != target_name
            || !matches!(
                rule,
                "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001"
                    | "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001"
                    | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                    | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
            )
            || !cycle.iter().all(|identity| {
                self.active_recursive_functions
                    .get(identity)
                    .is_some_and(|active| active.proof.rule == rule)
            })
        {
            return false;
        }
        let internal_edges_close = cycle.windows(2).all(|pair| {
            let current = self
                .active_recursive_functions
                .get(&pair[0])
                .expect("active cycle member retains proof metadata");
            let next = self
                .active_recursive_functions
                .get(&pair[1])
                .expect("active cycle member retains proof metadata");
            current.proof.mutual_target.as_deref() == Some(next.source_name.as_str())
        });
        let last = self
            .active_recursive_functions
            .get(cycle.last().expect("cycle has at least two members"))
            .expect("active cycle member retains proof metadata");
        internal_edges_close && last.proof.mutual_target.as_deref() == Some(target_name)
    }

    fn adapt_proven_recursive_nat_argument(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
        parameter_index: usize,
        expected: &CompilerType,
        argument: &CompilerExpression,
    ) -> Option<CompilerExpression> {
        if expected != &CompilerType::Nat || argument.value_type != CompilerType::Int {
            return None;
        }
        let target_identity = function_overload_identity(&self.source, function_name, declaration);
        let active_identity = self.active_calls.last()?;
        let active = self.active_recursive_functions.get(active_identity)?;
        let direct_edge =
            active_identity == &target_identity && active.proof.mutual_target.is_none();
        let mutual_nat_edge = active.proof.mutual_target.as_deref() == Some(function_name)
            && matches!(
                active.proof.rule,
                "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                    | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
            );
        if !(direct_edge || mutual_nat_edge)
            || !active.proof.nat_step_parameters.contains(&parameter_index)
        {
            return None;
        }
        Some(CompilerExpression {
            kind: CompilerExpressionKind::IntToNat(Box::new(argument.clone())),
            value_type: CompilerType::Nat,
            int_range: argument.int_range.clone(),
            rational_value: None,
            span: argument.span,
        })
    }

    fn active_nat_recursion(&self) -> bool {
        self.active_calls
            .last()
            .and_then(|identity| self.active_recursive_functions.get(identity))
            .is_some_and(|active| {
                !active.proof.nat_step_parameters.is_empty()
                    && matches!(
                        active.proof.rule,
                        "TOPAL-FUNCTION-RECURSION-NAT-001"
                            | "TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"
                            | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                            | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
                            | "TOPAL-FUNCTION-DECREASES-001"
                    )
            })
    }

    fn analyze_decision(
        &mut self,
        subject: &Expression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let subject = self.analyze_expression(subject, environment)?;
        match &subject.value_type {
            CompilerType::Boolean => {
                self.analyze_boolean_decision(subject, rules, span, environment)
            }
            CompilerType::Comparison => {
                self.analyze_comparison_value_decision(subject, rules, span, environment)
            }
            CompilerType::Enum(enumeration) => {
                let enumeration = enumeration.clone();
                self.analyze_enum_decision(subject, &enumeration, rules, span, environment)
            }
            CompilerType::Sum(sum) => {
                let sum = sum.clone();
                self.analyze_sum_decision(subject, &sum, rules, span, environment)
            }
            CompilerType::Result(success) => {
                let success = success.as_ref().clone();
                self.analyze_result_decision(subject, &success, rules, span, environment)
            }
            CompilerType::Optional(payload) => {
                let payload = payload.as_ref().clone();
                self.analyze_optional_decision(subject, &payload, rules, span, environment)
            }
            CompilerType::List(element) if element.as_ref() == &CompilerType::Int => {
                self.analyze_list_decision(subject, rules, span, environment)
            }
            CompilerType::List(_) => Err(unsupported(
                &self.source,
                subject.span,
                "decision for this List element type",
            )),
            CompilerType::Int | CompilerType::Rational => {
                self.analyze_ordered_comparison_decision(subject, rules, span, environment)
            }
            CompilerType::Nat if self.active_nat_recursion() => self
                .analyze_ordered_comparison_decision(
                    forget_nat_evidence(subject),
                    rules,
                    span,
                    environment,
                ),
            _ => Err(unsupported(
                &self.source,
                subject.span,
                "decision subject type",
            )),
        }
    }

    fn analyze_optional_decision(
        &mut self,
        subject: CompilerExpression,
        payload_type: &CompilerType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut some = None;
        let mut none = None;
        let mut otherwise = None;
        for rule in rules {
            if otherwise.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-UNREACHABLE-DECISION-RULE",
                    rule.span,
                    "an Optional rule cannot follow otherwise",
                ));
            }
            match rule.matcher {
                DecisionMatcher::Optional {
                    some: true,
                    binding: Some(binding),
                    ..
                } if some.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let branch =
                        decision_binding_environment(environment, &name, payload_type.clone());
                    some = Some((
                        name,
                        binding,
                        self.analyze_expression(&rule.action, &branch)?,
                    ));
                }
                DecisionMatcher::Optional {
                    some: false,
                    binding: None,
                    ..
                } if none.is_none() => {
                    none = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Otherwise(_) => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Optional { .. } => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-DUPLICATE-DECISION-RULE",
                        rule.span,
                        "an Optional alternative appears more than once",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Optional decision matcher",
                    ));
                }
            }
        }
        let (some_binding, some_action) = if let Some((name, binding, action)) = some {
            (Some((name, binding)), action)
        } else if let Some(action) = otherwise.clone() {
            (None, action)
        } else {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Optional decision does not cover Some",
            ));
        };
        let none_action = none.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Optional decision does not cover None",
            )
        })?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&some_action, &none_action], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::OptionalDecision {
                subject: Box::new(subject),
                some_binding,
                some_action: Box::new(some_action),
                none_action: Box::new(none_action),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_list_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut entry = None;
        let mut empty = None;
        let mut otherwise = None;
        for rule in rules {
            if otherwise.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-UNREACHABLE-DECISION-RULE",
                    rule.span,
                    "a List rule cannot follow otherwise",
                ));
            }
            match rule.matcher {
                DecisionMatcher::ListEmpty(_) if empty.is_none() => {
                    empty = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::ListEntry { first, rest, .. } if entry.is_none() => {
                    let first_name = self.source.slice(first).to_owned();
                    let rest_name = self.source.slice(rest).to_owned();
                    if first_name == rest_name {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            rest,
                            "the List entry and remaining List bindings must be distinct",
                        ));
                    }
                    let branch =
                        decision_binding_environment(environment, &first_name, CompilerType::Int);
                    let branch = decision_binding_environment(
                        &branch,
                        &rest_name,
                        CompilerType::List(Box::new(CompilerType::Int)),
                    );
                    entry = Some((
                        ((first_name, first), (rest_name, rest)),
                        self.analyze_expression(&rule.action, &branch)?,
                    ));
                }
                DecisionMatcher::Otherwise(_) => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::ListEmpty(_) | DecisionMatcher::ListEntry { .. } => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-DUPLICATE-DECISION-RULE",
                        rule.span,
                        "a List alternative appears more than once",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "List decision matcher",
                    ));
                }
            }
        }
        let (entry_bindings, entry_action) = if let Some((bindings, action)) = entry {
            (Some(bindings), action)
        } else if let Some(action) = otherwise.clone() {
            (None, action)
        } else {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "List decision does not cover Entry",
            ));
        };
        let empty_action = empty.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "List decision does not cover Empty",
            )
        })?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&entry_action, &empty_action], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ListDecision {
                subject: Box::new(subject),
                entry_bindings,
                entry_action: Box::new(entry_action),
                empty_action: Box::new(empty_action),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_enum_decision(
        &mut self,
        subject: CompilerExpression,
        enumeration: &CompilerEnumType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut lowered = Vec::new();
        let mut seen = BTreeSet::new();
        let mut otherwise = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Identifier(matcher) if otherwise.is_none() => {
                    let label = self.source.slice(matcher);
                    let Some(value) = enumeration
                        .alternatives
                        .iter()
                        .position(|alternative| alternative == label)
                    else {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNKNOWN-ENUM-ALTERNATIVE",
                            matcher,
                            format!("`{label}` is not an alternative of `{}`", enumeration.name),
                        ));
                    };
                    let value =
                        u32::try_from(value).expect("enum declaration already fits the native tag");
                    if !seen.insert(value) {
                        // The earlier source-ordered matcher always selects this
                        // alternative, so the repeated action is unreachable.
                        continue;
                    }
                    lowered.push(CompilerEnumRule {
                        value,
                        action: self.analyze_expression(&rule.action, environment)?,
                        span: rule.span,
                    });
                }
                DecisionMatcher::Otherwise(_) if otherwise.is_none() => {
                    otherwise = Some(Box::new(
                        self.analyze_expression(&rule.action, environment)?,
                    ));
                }
                _ if otherwise.is_some() => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "an Enum rule cannot follow otherwise",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Enum decision matcher",
                    ));
                }
            }
        }
        if otherwise.is_none() && seen.len() != enumeration.alternatives.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!(
                    "decision does not cover every `{}` alternative",
                    enumeration.name
                ),
            ));
        }
        let mut actions = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        if let Some(action) = &otherwise {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::EnumDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise,
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Nominal and positional matcher validation stays adjacent to completeness checks.
    fn analyze_sum_decision(
        &mut self,
        subject: CompilerExpression,
        sum: &CompilerSumType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut lowered = Vec::new();
        let mut seen = BTreeSet::new();
        let mut otherwise = None;
        for rule in rules {
            if otherwise.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-UNREACHABLE-DECISION-RULE",
                    rule.span,
                    "a sum rule cannot follow otherwise",
                ));
            }
            if matches!(rule.matcher, DecisionMatcher::Otherwise(_)) {
                otherwise = Some(Box::new(
                    self.analyze_expression(&rule.action, environment)?,
                ));
                continue;
            }
            let (value, binding) = match rule.matcher {
                DecisionMatcher::Identifier(matcher) if !sum.positional => {
                    let label = self.source.slice(matcher);
                    let value = sum
                        .alternatives
                        .iter()
                        .position(|alternative| alternative.name == label)
                        .ok_or_else(|| {
                            source_diagnostic(
                                &self.source,
                                "E-UNKNOWN-UNION-ALTERNATIVE",
                                matcher,
                                format!("`{label}` is not an alternative of `{}`", sum.name),
                            )
                        })?;
                    if sum.alternatives[value].payload.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNION-PAYLOAD-BINDING",
                            matcher,
                            format!("Union alternative `{label}` requires a payload binding"),
                        ));
                    }
                    (value, None)
                }
                DecisionMatcher::Union {
                    alternative,
                    binding,
                    ..
                } if !sum.positional => {
                    let label = self.source.slice(alternative);
                    let value = sum
                        .alternatives
                        .iter()
                        .position(|candidate| candidate.name == label)
                        .ok_or_else(|| {
                            source_diagnostic(
                                &self.source,
                                "E-UNKNOWN-UNION-ALTERNATIVE",
                                alternative,
                                format!("`{label}` is not an alternative of `{}`", sum.name),
                            )
                        })?;
                    if sum.alternatives[value].payload.is_none() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNION-PAYLOAD-BINDING",
                            binding,
                            format!("Union alternative `{label}` has no payload"),
                        ));
                    }
                    (value, Some(binding))
                }
                DecisionMatcher::Variant {
                    type_name,
                    index,
                    binding,
                    ..
                } if sum.positional => {
                    let matcher_type = self.source.slice(type_name);
                    if matcher_type != sum.name {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-VARIANT-TYPE",
                            type_name,
                            format!(
                                "Variant matcher `{matcher_type}` does not match `{}`",
                                sum.name
                            ),
                        ));
                    }
                    let value = parse_integer(self.source.slice(index))
                        .and_then(|value| value.to_string().parse::<usize>().ok())
                        .filter(|value| *value < sum.alternatives.len())
                        .ok_or_else(|| {
                            source_diagnostic(
                                &self.source,
                                "E-VARIANT-INDEX",
                                index,
                                "Variant alternative index is outside its declared bounds",
                            )
                        })?;
                    (value, Some(binding))
                }
                _ => {
                    return Err(unsupported(&self.source, rule.span, "sum decision matcher"));
                }
            };
            let value = u32::try_from(value).expect("sum alternative fits native tag");
            if !seen.insert(value) {
                continue;
            }
            let binding = binding.map(|binding| (self.source.slice(binding).to_owned(), binding));
            let branch = if let Some((name, _)) = &binding {
                decision_binding_environment(
                    environment,
                    name,
                    sum.alternatives[usize::try_from(value).expect("u32 sum tag fits usize")]
                        .payload
                        .clone()
                        .expect("payload matcher selected a payload alternative"),
                )
            } else {
                environment.clone()
            };
            lowered.push(CompilerSumRule {
                value,
                binding,
                action: self.analyze_expression(&rule.action, &branch)?,
                span: rule.span,
            });
        }
        if otherwise.is_none() && seen.len() != sum.alternatives.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!("decision does not cover every `{}` alternative", sum.name),
            ));
        }
        let mut actions = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        if let Some(action) = &otherwise {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::SumDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise,
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Result binding, reachability, and completeness checks stay adjacent.
    fn analyze_result_decision(
        &mut self,
        subject: CompilerExpression,
        success_type: &CompilerType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut ok = None;
        let mut error_codes = Vec::new();
        let mut seen_codes = BTreeSet::new();
        let mut error_fallback = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Result {
                    error: false,
                    binding,
                    ..
                } if ok.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let branch =
                        decision_binding_environment(environment, &name, success_type.clone());
                    ok = Some((
                        name,
                        binding,
                        self.analyze_expression(&rule.action, &branch)?,
                    ));
                }
                DecisionMatcher::Result {
                    error: true,
                    binding,
                    ..
                } if error_fallback.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let branch =
                        decision_binding_environment(environment, &name, CompilerType::Error);
                    error_fallback = Some((
                        name,
                        binding,
                        Box::new(self.analyze_expression(&rule.action, &branch)?),
                    ));
                }
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } => {
                    if error_fallback.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNREACHABLE-ERROR-CODE-PATTERN",
                            rule.span,
                            "qualified Error-code pattern is unreachable after Error fallback",
                        ));
                    }
                    let code_value = arithmetic_error_code(
                        self.source.slice(namespace),
                        self.source.slice(vocabulary),
                        self.source.slice(code),
                    )
                    .ok_or_else(|| {
                        source_diagnostic(
                            &self.source,
                            "E-UNKNOWN-ERROR-CODE",
                            code,
                            "the compiler subset requires a qualified arithmetic Error code",
                        )
                    })?;
                    if !seen_codes.insert(code_value) {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-ERROR-CODE-PATTERN",
                            rule.span,
                            "an arithmetic Error code is matched more than once",
                        ));
                    }
                    error_codes.push(CompilerErrorCodeRule {
                        code: code_value,
                        action: self.analyze_expression(&rule.action, environment)?,
                        span: rule.span,
                    });
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Result decision matcher",
                    ));
                }
            }
        }
        let (ok_binding, ok_binding_span, ok_action) = ok.ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Result decision does not cover Ok",
            )
        })?;
        if error_fallback.is_none() && seen_codes != BTreeSet::from([0, 1, 2, 3]) {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-ERROR-CODE-DECISION",
                span,
                "Result decision requires Error fallback or every arithmetic Error code",
            ));
        }
        let mut actions = vec![&ok_action];
        actions.extend(error_codes.iter().map(|rule| &rule.action));
        if let Some((_, _, action)) = &error_fallback {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ResultDecision {
                subject: Box::new(subject),
                ok_binding,
                ok_binding_span,
                ok_action: Box::new(ok_action),
                error_codes,
                error_fallback,
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_boolean_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            subject.span,
            &CompilerType::Boolean,
            &subject.value_type,
        )?;
        let mut when_true = None;
        let mut when_false = None;
        let mut otherwise = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Boolean { value: true, .. } if when_true.is_none() => {
                    when_true = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Boolean { value: false, .. } if when_false.is_none() => {
                    when_false = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Otherwise(_) if otherwise.is_none() => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            }
        }
        let when_true = when_true.or_else(|| otherwise.clone()).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `true`",
            )
        })?;
        let when_false = when_false.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `false`",
            )
        })?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_true, &when_false], span)?;
        Ok(CompilerExpression {
            value_type,
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_ordered_comparison_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_exact_numeric(&self.source, subject.span, &subject.value_type)?;
        let mut lowered = Vec::new();
        let mut otherwise = None;
        for (index, rule) in rules.iter().enumerate() {
            match &rule.matcher {
                DecisionMatcher::Comparison {
                    kind,
                    operand,
                    span: matcher_span,
                } => {
                    if otherwise.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNREACHABLE-DECISION-RULE",
                            rule.span,
                            "a comparison rule cannot follow otherwise",
                        ));
                    }
                    let Some(operation) = comparison_binary(*kind) else {
                        return Err(unsupported(
                            &self.source,
                            *matcher_span,
                            "comparison decision callable",
                        ));
                    };
                    let mut operand = self.analyze_expression(operand, environment)?;
                    require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
                    let subject_to_rational = subject.value_type == CompilerType::Int
                        && operand.value_type == CompilerType::Rational;
                    if subject.value_type == CompilerType::Rational
                        && operand.value_type == CompilerType::Int
                    {
                        operand = into_rational(operand);
                    }
                    if !subject_to_rational {
                        require_same_type(
                            &self.source,
                            operand.span,
                            &subject.value_type,
                            &operand.value_type,
                        )?;
                    }
                    lowered.push(CompilerComparisonRule {
                        operation,
                        operand,
                        action: self.analyze_expression(&rule.action, environment)?,
                        subject_to_rational,
                        span: rule.span,
                    });
                }
                DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::Otherwise(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "otherwise must be the final comparison rule",
                    ));
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            }
        }
        let otherwise = otherwise.ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "a comparison decision requires otherwise",
            )
        })?;
        let mut branches = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        branches.push(&otherwise);
        let (value_type, int_range, rational_value) = self.decision_facts(&branches, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::OrderedComparisonDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise: Box::new(otherwise),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_comparison_value_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut when_less = None;
        let mut when_equal = None;
        let mut when_greater = None;
        let mut otherwise = None;
        for (index, rule) in rules.iter().enumerate() {
            let destination = match &rule.matcher {
                DecisionMatcher::Identifier(name) => match self.source.slice(*name) {
                    "Less" => &mut when_less,
                    "Equal" => &mut when_equal,
                    "Greater" => &mut when_greater,
                    _ => return Err(unsupported(&self.source, *name, "Comparison alternative")),
                },
                DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => &mut otherwise,
                DecisionMatcher::Otherwise(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "otherwise must be the final Comparison rule",
                    ));
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            };
            if destination.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-DECISION-RULE",
                    rule.span,
                    "a Comparison alternative appears more than once",
                ));
            }
            *destination = Some(self.analyze_expression(&rule.action, environment)?);
        }
        let missing = |name: &str| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!("Comparison decision does not cover {name}"),
            )
        };
        let when_less = when_less
            .or_else(|| otherwise.clone())
            .ok_or_else(|| missing("Less"))?;
        let when_equal = when_equal
            .or_else(|| otherwise.clone())
            .ok_or_else(|| missing("Equal"))?;
        let when_greater = when_greater
            .or(otherwise)
            .ok_or_else(|| missing("Greater"))?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_less, &when_equal, &when_greater], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ComparisonValueDecision {
                subject: Box::new(subject),
                when_less: Box::new(when_less),
                when_equal: Box::new(when_equal),
                when_greater: Box::new(when_greater),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn decision_facts(
        &self,
        branches: &[&CompilerExpression],
        span: Span,
    ) -> Result<(CompilerType, Option<IntRange>, Option<BigRational>), Diagnostic> {
        let first = branches.first().expect("a complete decision has a branch");
        for branch in &branches[1..] {
            require_same_type(&self.source, span, &first.value_type, &branch.value_type)?;
        }
        if !compiler_function_result_supported(&first.value_type) {
            return Err(unsupported(
                &self.source,
                span,
                "decision actions with unsupported machine results",
            ));
        }
        let int_range = branches
            .iter()
            .try_fold(None, |range: Option<IntRange>, branch| {
                match (range, &branch.int_range) {
                    (None, Some(next)) => Some(Some(next.clone())),
                    (Some(current), Some(next)) => Some(Some(IntRange::union(&current, next))),
                    (_, None) => None,
                }
            })
            .flatten();
        let rational_value = first.rational_value.clone().filter(|value| {
            branches
                .iter()
                .all(|branch| branch.rational_value.as_ref() == Some(value))
        });
        Ok((first.value_type.clone(), int_range, rational_value))
    }
}

fn function_body_context_member_span(
    source: &SourceText,
    statements: &[Statement],
    member_name: &str,
) -> Option<Span> {
    statements
        .iter()
        .find_map(|statement| statement_context_member_span(source, statement, member_name))
}

fn statement_context_member_span(
    source: &SourceText,
    statement: &Statement,
    member_name: &str,
) -> Option<Span> {
    match statement {
        Statement::Published { declaration, .. } => {
            statement_context_member_span(source, declaration, member_name)
        }
        Statement::Binding { value, .. }
        | Statement::ContextAssignment { value, .. }
        | Statement::Discard { value, .. }
        | Statement::Return { value, .. }
        | Statement::Expression(value) => {
            expression_context_member_span(source, value, member_name)
        }
        _ => None,
    }
}

fn expression_context_member_span(
    source: &SourceText,
    expression: &Expression,
    member_name: &str,
) -> Option<Span> {
    match expression {
        Expression::ContextIdentifier(span) if source.slice(*span) == member_name => Some(*span),
        Expression::Block { statements, .. } => {
            function_body_context_member_span(source, statements, member_name)
        }
        Expression::Product { fields, .. } => fields
            .iter()
            .find_map(|field| expression_context_member_span(source, &field.value, member_name)),
        Expression::DecisionTable { subject, rules, .. } => {
            expression_context_member_span(source, subject, member_name).or_else(|| {
                rules.iter().find_map(|rule| {
                    let matcher = match &rule.matcher {
                        DecisionMatcher::Comparison { operand, .. } => {
                            expression_context_member_span(source, operand, member_name)
                        }
                        _ => None,
                    };
                    matcher.or_else(|| {
                        expression_context_member_span(source, &rule.action, member_name)
                    })
                })
            })
        }
        Expression::Application { items, .. } => items
            .iter()
            .find_map(|item| expression_context_member_span(source, item, member_name)),
        Expression::AnonymousFunction { .. }
        | Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Identifier(_)
        | Expression::ContextIdentifier(_)
        | Expression::Discard(_)
        | Expression::Callable { .. } => None,
    }
}

fn compact_classifier(classifier: &str) -> String {
    classifier
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn function_overload_identity(
    source: &SourceText,
    name: &str,
    declaration: &FunctionSource,
) -> String {
    let inputs = declaration
        .parameters
        .iter()
        .map(|parameter| compact_classifier(source.slice(parameter.classifier)))
        .collect::<Vec<_>>()
        .join(",");
    let staticness = if declaration.is_static {
        "static"
    } else {
        "ordinary"
    };
    format!("{name}:{staticness}({inputs})")
}

fn fundamental_type_value(name: &str) -> Option<u32> {
    match name {
        "Boolean" => Some(0),
        "Int" => Some(1),
        "Nat" => Some(2),
        "Rational" => Some(3),
        "String" => Some(4),
        "Unit" => Some(5),
        "Scope" => Some(6),
        _ => None,
    }
}

fn fundamental_capability(name: &str) -> bool {
    matches!(
        name,
        "Equality" | "Ordering" | "Foldable" | "Membership" | "Indexed" | "Keyed"
    )
}

fn compiler_layout_policy(name: &str) -> Option<(CompilerEnumType, u32)> {
    let (type_name, alternatives): (&str, &[&str]) = match name {
        "Little" | "Big" => ("Endian", &["Little", "Big"]),
        "ReadWrite" | "ReadOnly" | "WriteOnly" | "Reserved" => (
            "Access",
            &["ReadWrite", "ReadOnly", "WriteOnly", "Reserved"],
        ),
        "MostSignificantFirst" | "LeastSignificantFirst" => (
            "BitOrder",
            &["MostSignificantFirst", "LeastSignificantFirst"],
        ),
        "Natural" | "Packed" => ("Packing", &["Natural", "Packed"]),
        "Declared" => ("FieldOrder", &["Declared"]),
        "AfterTag" | "Overlay" => ("PayloadPlacement", &["AfterTag", "Overlay"]),
        "NoLength" | "NoTerminator" => ("LayoutPolicy", &["NoLength", "NoTerminator"]),
        _ => return None,
    };
    let value = alternatives
        .iter()
        .position(|candidate| *candidate == name)?;
    Some((
        CompilerEnumType {
            name: type_name.to_owned(),
            alternatives: alternatives
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
        },
        u32::try_from(value).expect("fundamental layout policy count fits u32"),
    ))
}

fn compiler_int_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Int, CompilerType::String]
    )
}

fn compiler_nested_int_string_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element) if compiler_int_string_pair(element)
    )
}

fn compiler_list_node_element_supported(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Effect | CompilerType::Int)
        || matches!(
            value_type,
            CompilerType::Tuple(fields)
                if fields.as_slice() == [CompilerType::Int, CompilerType::Int]
        )
        || compiler_int_string_pair(value_type)
        || compiler_nested_int_string_list_element(value_type)
}

fn parse_compact_classifier(classifier: &str) -> Option<CompilerType> {
    if let Some(element) = classifier.strip_prefix("List") {
        let element = parse_compact_classifier(element)?;
        if compiler_list_node_element_supported(&element) {
            return Some(CompilerType::List(Box::new(element)));
        }
        return None;
    }
    if let Some(payload) = classifier.strip_prefix("Optional") {
        return Some(CompilerType::Optional(Box::new(parse_compact_classifier(
            payload,
        )?)));
    }
    if let Some(success_and_codes) = classifier
        .strip_prefix("Result(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let (success, codes) = split_classifier_once(success_and_codes)?;
        if codes != "langarithmeticArithmeticErrorCode" {
            return None;
        }
        return Some(CompilerType::Result(Box::new(parse_compact_classifier(
            success,
        )?)));
    }
    if let Some(fields) = classifier
        .strip_prefix("Record(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let mut parsed = split_classifier_fields(fields)?
            .into_iter()
            .map(|field| {
                let (label, classifier) = split_record_classifier_field(field)?;
                Some((label.to_owned(), parse_compact_classifier(classifier)?))
            })
            .collect::<Option<Vec<_>>>()?;
        parsed.sort_by(|left, right| left.0.cmp(&right.0));
        return parsed
            .windows(2)
            .all(|fields| fields[0].0 != fields[1].0)
            .then_some(CompilerType::Record(parsed));
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        let fields = split_classifier_fields(fields)?;
        if fields.len() <= 1 {
            return None;
        }
        return Some(CompilerType::Tuple(
            fields
                .into_iter()
                .map(parse_compact_classifier)
                .collect::<Option<Vec<_>>>()?,
        ));
    }
    parse_compact_scalar_classifier(classifier)
}

fn parse_compact_scalar_classifier(classifier: &str) -> Option<CompilerType> {
    match classifier {
        "RangeInt" => Some(CompilerType::Range(Box::new(CompilerType::Int))),
        "RangeRational" => Some(CompilerType::Range(Box::new(CompilerType::Rational))),
        "Unit" => Some(CompilerType::Unit),
        "Completed" => Some(CompilerType::Completed),
        "Effect" => Some(CompilerType::Effect),
        "Type" => Some(CompilerType::Type),
        "Scope" => Some(CompilerType::Scope),
        "Function" => Some(CompilerType::Function),
        "Capability" => Some(CompilerType::Capability),
        "Constraint" => Some(CompilerType::Constraint),
        "Boolean" => Some(CompilerType::Boolean),
        "Int" => Some(CompilerType::Int),
        "Nat" => Some(CompilerType::Nat),
        "Rational" => Some(CompilerType::Rational),
        "Comparison" => Some(CompilerType::Comparison),
        "Error" => Some(CompilerType::Error),
        "ErrorCode" | "langarithmeticArithmeticErrorCode" => Some(CompilerType::ErrorCode),
        "ErrorDomain" => Some(CompilerType::ErrorDomain),
        "SourceLocation" => Some(CompilerType::SourceLocation),
        "Character" => Some(CompilerType::Character),
        "String" => Some(CompilerType::String),
        "GeneratorCharacterUnitUnit" => Some(character_unit_generator_type()),
        _ => None,
    }
}

fn split_record_classifier_field(field: &str) -> Option<(&str, &str)> {
    let mut depth = 0_u32;
    for (index, byte) in field.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b':' if depth == 0 => {
                let label = &field[..index];
                let classifier = &field[index + 1..];
                return (!label.is_empty() && !classifier.is_empty())
                    .then_some((label, classifier));
            }
            _ => {}
        }
    }
    None
}

fn split_classifier_once(classifier: &str) -> Option<(&str, &str)> {
    let mut depth = 0_u32;
    for (index, byte) in classifier.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => return Some((&classifier[..index], &classifier[index + 1..])),
            _ => {}
        }
    }
    None
}

fn split_classifier_fields(classifier: &str) -> Option<Vec<&str>> {
    let mut fields = Vec::new();
    let mut remaining = classifier;
    while let Some((field, rest)) = split_classifier_once(remaining) {
        fields.push(field);
        remaining = rest;
    }
    fields.push(remaining);
    fields
        .iter()
        .all(|field| !field.is_empty())
        .then_some(fields)
}

fn is_range_construction(operation: CompilerBinary) -> bool {
    matches!(
        operation,
        CompilerBinary::Range
            | CompilerBinary::RangeOpen
            | CompilerBinary::RangeInclusive
            | CompilerBinary::RangeOpenInclusive
    )
}

fn compiler_abi_type_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::TraversalControl(_) | CompilerType::Generator(_) => false,
        CompilerType::List(element) => {
            matches!(element.as_ref(), CompilerType::Effect | CompilerType::Int)
                || compiler_nested_int_string_list_element(element.as_ref())
        }
        CompilerType::Optional(payload) => {
            matches!(
                payload.as_ref(),
                CompilerType::Int
                    | CompilerType::Rational
                    | CompilerType::Character
                    | CompilerType::String
                    | CompilerType::Error
                    | CompilerType::SourceLocation
            )
        }
        CompilerType::Result(success) => {
            matches!(
                success.as_ref(),
                CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Rational
                    | CompilerType::String
                    | CompilerType::Modular(_)
            ) || matches!(
                success.as_ref(),
                CompilerType::Tuple(fields)
                    if matches!(fields.as_slice(), [CompilerType::Int, CompilerType::Int])
            )
        }
        _ => true,
    }
}

fn compiler_type_is_static_only(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Identity
            | CompilerType::TypeView
            | CompilerType::FunctionView
            | CompilerType::LanguageContext
            | CompilerType::Capability
    )
}

fn compiler_type_contains_static_only(value_type: &CompilerType) -> bool {
    compiler_type_is_static_only(value_type)
        || match value_type {
            CompilerType::Range(value)
            | CompilerType::Result(value)
            | CompilerType::Optional(value)
            | CompilerType::List(value)
            | CompilerType::TraversalControl(value)
            | CompilerType::Refined { base: value, .. } => {
                compiler_type_contains_static_only(value)
            }
            CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_static_only),
            CompilerType::Record(fields) => fields
                .iter()
                .any(|(_, field)| compiler_type_contains_static_only(field)),
            CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
                alternative
                    .payload
                    .as_ref()
                    .is_some_and(compiler_type_contains_static_only)
            }),
            _ => false,
        }
}

fn compiler_type_contains_generator(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Generator(_) => true,
        CompilerType::Range(value)
        | CompilerType::Result(value)
        | CompilerType::Optional(value)
        | CompilerType::List(value)
        | CompilerType::TraversalControl(value)
        | CompilerType::Refined { base: value, .. } => compiler_type_contains_generator(value),
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_generator),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, field)| compiler_type_contains_generator(field)),
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_contains_generator)
        }),
        _ => false,
    }
}

fn anonymous_body_capture(
    source: &SourceText,
    parameters: &[AnonymousPattern],
    body: &Expression,
    environment: &BTreeMap<String, BindingFacts>,
) -> Option<String> {
    let parameter_names = parameters
        .iter()
        .flat_map(|parameter| match parameter {
            AnonymousPattern::Binding(binding) => vec![source.slice(*binding)],
            AnonymousPattern::Product { bindings, .. } => bindings
                .iter()
                .map(|binding| source.slice(*binding))
                .collect(),
        })
        .collect::<BTreeSet<_>>();
    environment
        .keys()
        .find(|name| {
            !parameter_names.contains(name.as_str()) && expression_mentions_name(source, body, name)
        })
        .cloned()
}

fn directly_collectable_list_uncons_unfold(
    generator: &CompilerExpression,
    environment: &BTreeMap<String, BindingFacts>,
) -> bool {
    let CompilerExpressionKind::UnfoldGenerator {
        seed,
        parameters,
        step,
    } = &generator.kind
    else {
        return false;
    };
    let [parameter] = parameters.as_slice() else {
        return false;
    };
    let CompilerExpressionKind::ListUncons(argument) = &step.result.kind else {
        return false;
    };
    let CompilerExpressionKind::Local(seed) = &seed.kind else {
        return false;
    };
    step.statements.is_empty()
        && binding_facts_by_storage(environment, seed).is_some()
        && matches!(&argument.kind, CompilerExpressionKind::Local(name) if name == &parameter.name)
}

type AnonymousBodyRef<'a> = (&'a [AnonymousPattern], &'a Expression, Span);

fn direct_bounded_iterate_functions<'a>(
    source: &SourceText,
    expression: &'a Expression,
) -> Option<(AnonymousBodyRef<'a>, AnonymousBodyRef<'a>)> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    if let [
        _,
        Expression::Identifier(iterate),
        Expression::AnonymousFunction {
            parameters: next_parameters,
            body: next_body,
            span: next_span,
        },
        Expression::Identifier(take_while),
        Expression::AnonymousFunction {
            parameters: predicate_parameters,
            body: predicate_body,
            span: predicate_span,
        },
    ] = items.as_slice()
        && source.slice(*iterate) == "iterate"
        && source.slice(*take_while) == "take-while"
    {
        return Some((
            (next_parameters, next_body, *next_span),
            (predicate_parameters, predicate_body, *predicate_span),
        ));
    }
    let [iterate, Expression::Identifier(take_while), predicate] = items.as_slice() else {
        return None;
    };
    let Expression::AnonymousFunction {
        parameters: predicate_parameters,
        body: predicate_body,
        span: predicate_span,
    } = predicate
    else {
        return None;
    };
    let Expression::Application { items, .. } = iterate else {
        return None;
    };
    let [
        _,
        Expression::Identifier(iterate),
        Expression::AnonymousFunction {
            parameters: next_parameters,
            body: next_body,
            span: next_span,
        },
    ] = items.as_slice()
    else {
        return None;
    };
    (source.slice(*iterate) == "iterate" && source.slice(*take_while) == "take-while").then_some((
        (next_parameters, next_body, *next_span),
        (predicate_parameters, predicate_body, *predicate_span),
    ))
}

fn reject_static_value_containment(
    source: &SourceText,
    value: &CompilerExpression,
) -> Result<(), Diagnostic> {
    if compiler_type_contains_static_only(&value.value_type)
        && !compiler_type_is_static_only(&value.value_type)
    {
        Err(unsupported(
            source,
            value.span,
            "containment of a static compiler value",
        ))
    } else {
        Ok(())
    }
}

fn compiler_function_result_supported(value_type: &CompilerType) -> bool {
    if matches!(
        value_type,
        CompilerType::Scope
            | CompilerType::Function
            | CompilerType::Capability
            | CompilerType::Constraint
            | CompilerType::Version
            | CompilerType::Refined { .. }
    ) {
        return false;
    }
    if value_type.machine_scalar() {
        return compiler_abi_type_supported(value_type);
    }
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.iter().all(compiler_function_result_supported)
    ) || matches!(
        value_type,
        CompilerType::Record(fields)
            if fields
                .iter()
                .all(|(_, field)| compiler_function_result_supported(field))
    ) || matches!(
        value_type,
        CompilerType::Sum(sum)
            if sum.alternatives.iter().all(|alternative| alternative
                .payload
                .as_ref()
                .is_none_or(compiler_function_result_supported))
    )
}

fn compiler_function_parameter_supported(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Scope | CompilerType::Function)
        || is_character_unit_generator_type(value_type)
        || compiler_function_result_supported(value_type)
}

fn data_member_expression(facts: &CompilerDataMemberFacts, span: Span) -> CompilerExpression {
    CompilerExpression {
        kind: CompilerExpressionKind::Local(facts.storage_name.clone()),
        value_type: facts.value_type.clone(),
        int_range: facts.int_range.clone(),
        rational_value: facts.rational_value.clone(),
        span,
    }
}

fn binding_facts_by_storage<'a>(
    environment: &'a BTreeMap<String, BindingFacts>,
    storage_name: &str,
) -> Option<&'a BindingFacts> {
    environment
        .values()
        .find(|facts| facts.storage_name == storage_name)
}

fn decision_binding_environment(
    environment: &BTreeMap<String, BindingFacts>,
    name: &str,
    value_type: CompilerType,
) -> BTreeMap<String, BindingFacts> {
    let mut branch = environment.clone();
    branch.insert(
        name.to_owned(),
        BindingFacts {
            storage_name: name.to_owned(),
            runtime_bound: true,
            value_type,
            int_range: None,
            rational_value: None,
            string_value: None,
            closed_int_range: None,
            record_fields: BTreeMap::new(),
            namespace: None,
            callable: None,
            static_capability: None,
        },
    );
    branch
}

fn arithmetic_error_code(namespace: &str, vocabulary: &str, code: &str) -> Option<u32> {
    if namespace != "lang" || vocabulary != "arithmetic" {
        return None;
    }
    match code {
        "out-of-range" => Some(0),
        "not-representable" => Some(1),
        "division-by-zero" => Some(2),
        "indeterminate" => Some(3),
        _ => None,
    }
}

fn generator_error_code(
    namespace: &str,
    vocabulary: &str,
    code: &str,
) -> Option<(CompilerEnumType, u32)> {
    if namespace != "lang" || vocabulary != "generator" || code != "generator-closed" {
        return None;
    }
    Some((
        CompilerEnumType {
            name: "lang generator GeneratorErrorCode".to_owned(),
            alternatives: vec!["generator-closed".to_owned()],
        },
        0,
    ))
}

fn int_unit_generator_type() -> CompilerType {
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(CompilerType::Int),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(CompilerType::Unit),
    })
}

fn character_unit_generator_type() -> CompilerType {
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(CompilerType::Character),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(CompilerType::Unit),
    })
}

fn is_character_unit_generator_type(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Generator(CompilerGeneratorType {
            yield_type,
            resume_type,
            result_type,
        }) if yield_type.as_ref() == &CompilerType::Character
            && resume_type.as_ref() == &CompilerType::Unit
            && result_type.as_ref() == &CompilerType::Unit
    )
}

fn comparison_binary(kind: CallableKind) -> Option<CompilerBinary> {
    match kind {
        CallableKind::Equal => Some(CompilerBinary::Equal),
        CallableKind::NotEqual => Some(CompilerBinary::NotEqual),
        CallableKind::Less => Some(CompilerBinary::Less),
        CallableKind::Greater => Some(CompilerBinary::Greater),
        CallableKind::LessEqual => Some(CompilerBinary::LessEqual),
        CallableKind::GreaterEqual => Some(CompilerBinary::GreaterEqual),
        _ => None,
    }
}

fn is_exact_numeric(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Int | CompilerType::Rational)
}

fn forget_refined_evidence(mut expression: CompilerExpression) -> CompilerExpression {
    let CompilerType::Refined { base, .. } = expression.value_type else {
        return expression;
    };
    expression.value_type = *base;
    expression
}

fn known_constraint_predicate(
    predicate: &CompilerExpression,
    parameter_storage: &str,
    argument: &CompilerExpression,
) -> Option<bool> {
    match &predicate.kind {
        CompilerExpressionKind::Boolean(value) => Some(*value),
        CompilerExpressionKind::Not(value) => Some(!known_constraint_predicate(
            value,
            parameter_storage,
            argument,
        )?),
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::And,
            left,
            right,
        } => Some(
            known_constraint_predicate(left, parameter_storage, argument)?
                & known_constraint_predicate(right, parameter_storage, argument)?,
        ),
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Or,
            left,
            right,
        } => Some(
            known_constraint_predicate(left, parameter_storage, argument)?
                | known_constraint_predicate(right, parameter_storage, argument)?,
        ),
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Xor,
            left,
            right,
        } => Some(
            known_constraint_predicate(left, parameter_storage, argument)?
                ^ known_constraint_predicate(right, parameter_storage, argument)?,
        ),
        CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } if matches!(
            operation,
            CompilerBinary::Equal
                | CompilerBinary::NotEqual
                | CompilerBinary::Less
                | CompilerBinary::Greater
                | CompilerBinary::LessEqual
                | CompilerBinary::GreaterEqual
        ) =>
        {
            let left = known_constraint_numeric(left, parameter_storage, argument)?;
            let right = known_constraint_numeric(right, parameter_storage, argument)?;
            Some(match operation {
                CompilerBinary::Equal => left == right,
                CompilerBinary::NotEqual => left != right,
                CompilerBinary::Less => left < right,
                CompilerBinary::Greater => left > right,
                CompilerBinary::LessEqual => left <= right,
                CompilerBinary::GreaterEqual => left >= right,
                _ => unreachable!("guard selected a Boolean comparison"),
            })
        }
        _ => None,
    }
}

fn known_constraint_numeric(
    expression: &CompilerExpression,
    parameter_storage: &str,
    argument: &CompilerExpression,
) -> Option<BigRational> {
    match &expression.kind {
        CompilerExpressionKind::Local(name) if name == parameter_storage => argument
            .rational_value
            .clone()
            .or_else(|| exact_int(argument).map(BigRational::from_integer)),
        CompilerExpressionKind::Int(value) => Some(BigRational::from_integer(value.clone())),
        CompilerExpressionKind::Rational(value) => Some(value.clone()),
        CompilerExpressionKind::Negate(value) => Some(-known_constraint_numeric(
            value,
            parameter_storage,
            argument,
        )?),
        CompilerExpressionKind::Absolute(value) => Some(rational_absolute(
            &known_constraint_numeric(value, parameter_storage, argument)?,
        )),
        CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::RationalToInt(value)
        | CompilerExpressionKind::IntToNat(value) => {
            known_constraint_numeric(value, parameter_storage, argument)
        }
        CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } if matches!(
            operation,
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply
        ) =>
        {
            let left = known_constraint_numeric(left, parameter_storage, argument)?;
            let right = known_constraint_numeric(right, parameter_storage, argument)?;
            Some(match operation {
                CompilerBinary::Add => left + right,
                CompilerBinary::Subtract => left - right,
                CompilerBinary::Multiply => left * right,
                _ => unreachable!("guard selected exact arithmetic"),
            })
        }
        _ => None,
    }
}

fn is_exact_comparable(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Int | CompilerType::Nat | CompilerType::Rational
    )
}

fn forget_nat_evidence(mut expression: CompilerExpression) -> CompilerExpression {
    if expression.value_type == CompilerType::Nat {
        expression.value_type = CompilerType::Int;
    }
    expression
}

fn compiler_equality_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Unit
        | CompilerType::Completed
        | CompilerType::Effect
        | CompilerType::Type
        | CompilerType::Boolean
        | CompilerType::Int
        | CompilerType::Nat
        | CompilerType::Rational
        | CompilerType::Comparison
        | CompilerType::ErrorCode
        | CompilerType::Character
        | CompilerType::String
        | CompilerType::Modular(_)
        | CompilerType::Enum(_) => true,
        CompilerType::Optional(payload) => {
            matches!(
                payload.as_ref(),
                CompilerType::Int | CompilerType::Rational | CompilerType::String
            )
        }
        CompilerType::List(element) => {
            element.as_ref() == &CompilerType::Int
                || compiler_nested_int_string_list_element(element.as_ref())
        }
        CompilerType::Tuple(fields) => fields.iter().all(compiler_equality_supported),
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, value_type)| compiler_equality_supported(value_type)),
        CompilerType::Refined { base, .. } => compiler_equality_supported(base),
        CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::Constraint
        | CompilerType::Version
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::SourceLocation
        | CompilerType::Sum(_)
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::TraversalControl(_)
        | CompilerType::Generator(_) => false,
    }
}

fn compiler_ordering_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::Rational
        | CompilerType::Modular(_) => true,
        CompilerType::Tuple(fields) => fields.iter().all(compiler_ordering_supported),
        _ => false,
    }
}

fn require_optional_payload(
    source: &SourceText,
    span: Span,
    value_type: &CompilerType,
) -> Result<(), Diagnostic> {
    if matches!(
        value_type,
        CompilerType::Int
            | CompilerType::Rational
            | CompilerType::String
            | CompilerType::Error
            | CompilerType::SourceLocation
    ) {
        Ok(())
    } else {
        Err(unsupported(source, span, "Optional payload type"))
    }
}

fn require_exact_numeric(
    source: &SourceText,
    span: Span,
    value_type: &CompilerType,
) -> Result<(), Diagnostic> {
    if is_exact_numeric(value_type) {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            span,
            format!("expected Int or Rational, found {}", value_type.name()),
        ))
    }
}

fn exact_int(expression: &CompilerExpression) -> Option<BigInt> {
    let range = expression.int_range.as_ref()?;
    (range.lower == range.upper).then(|| range.lower.clone())
}

fn compiler_expression_is_closed(expression: &CompilerExpression) -> bool {
    compiler_expression_is_closed_with(expression, &BTreeSet::new())
}

#[allow(clippy::too_many_lines)] // Exhaustive closure classification keeps every checked form visible.
fn compiler_expression_is_closed_with(
    expression: &CompilerExpression,
    bound: &BTreeSet<String>,
) -> bool {
    match &expression.kind {
        CompilerExpressionKind::Unit
        | CompilerExpressionKind::Completed
        | CompilerExpressionKind::Effect
        | CompilerExpressionKind::TypeValue(_)
        | CompilerExpressionKind::Root
        | CompilerExpressionKind::FunctionValue(_)
        | CompilerExpressionKind::Identity(_)
        | CompilerExpressionKind::TypeView(_)
        | CompilerExpressionKind::FunctionView(_)
        | CompilerExpressionKind::LanguageContext(_)
        | CompilerExpressionKind::Capability(_)
        | CompilerExpressionKind::ConstraintValue(_)
        | CompilerExpressionKind::Boolean(_)
        | CompilerExpressionKind::Version(_)
        | CompilerExpressionKind::Int(_)
        | CompilerExpressionKind::Rational(_)
        | CompilerExpressionKind::String(_)
        | CompilerExpressionKind::StringEmpty
        | CompilerExpressionKind::ErrorCode(_)
        | CompilerExpressionKind::Enum(_)
        | CompilerExpressionKind::OptionalNone
        | CompilerExpressionKind::ListEmpty => true,
        CompilerExpressionKind::ListEntry { value, remaining } => {
            compiler_expression_is_closed_with(value, bound)
                && compiler_expression_is_closed_with(remaining, bound)
        }
        CompilerExpressionKind::Sum { payload, .. } => payload
            .as_deref()
            .is_none_or(|value| compiler_expression_is_closed_with(value, bound)),
        CompilerExpressionKind::Tuple(values) => values
            .iter()
            .all(|value| compiler_expression_is_closed_with(value, bound)),
        CompilerExpressionKind::Record(fields) => fields
            .iter()
            .all(|(_, value)| compiler_expression_is_closed_with(value, bound)),
        CompilerExpressionKind::RecordReconstruct { base, replacements } => {
            compiler_expression_is_closed_with(base, bound)
                && replacements
                    .iter()
                    .all(|(_, value)| compiler_expression_is_closed_with(value, bound))
        }
        CompilerExpressionKind::Block(block) => compiler_block_is_closed(block, bound),
        CompilerExpressionKind::Local(name) => bound.contains(name),
        CompilerExpressionKind::Call { .. }
        | CompilerExpressionKind::Fallible { .. }
        | CompilerExpressionKind::Validate { .. }
        | CompilerExpressionKind::ModularValidate { .. }
        | CompilerExpressionKind::SumDecision { .. }
        | CompilerExpressionKind::ResultDecision { .. }
        | CompilerExpressionKind::OptionalDecision { .. }
        | CompilerExpressionKind::ListDecision { .. }
        | CompilerExpressionKind::ListMap { .. }
        | CompilerExpressionKind::ListSelect { .. }
        | CompilerExpressionKind::ListFold { .. }
        | CompilerExpressionKind::IterateGenerator { .. }
        | CompilerExpressionKind::GeneratorTakeWhile { .. }
        | CompilerExpressionKind::UnfoldGenerator { .. }
        | CompilerExpressionKind::StringCharactersGenerator { .. }
        | CompilerExpressionKind::StringCharactersForeach { .. }
        | CompilerExpressionKind::IterateGeneratorForeach { .. }
        | CompilerExpressionKind::GeneratorCollect(_) => false,
        CompilerExpressionKind::IntToModular { value, .. }
        | CompilerExpressionKind::ModularReduce { value, .. }
        | CompilerExpressionKind::Negate(value)
        | CompilerExpressionKind::Absolute(value)
        | CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::RationalToInt(value)
        | CompilerExpressionKind::IntToNat(value)
        | CompilerExpressionKind::ResultSuccess(value)
        | CompilerExpressionKind::ResultProject(value)
        | CompilerExpressionKind::OptionalSome(value)
        | CompilerExpressionKind::ListReverse(value)
        | CompilerExpressionKind::ListEntryCount(value)
        | CompilerExpressionKind::ListEmptyPredicate(value)
        | CompilerExpressionKind::ListFirst(value)
        | CompilerExpressionKind::ListRest(value)
        | CompilerExpressionKind::ListUncons(value)
        | CompilerExpressionKind::TraversalControl { value, .. }
        | CompilerExpressionKind::StringEmptyPredicate(value)
        | CompilerExpressionKind::StringUtf8ByteCount(value)
        | CompilerExpressionKind::StringCharactersCollect { text: value, .. }
        | CompilerExpressionKind::StringCharactersClose(value)
        | CompilerExpressionKind::RecordField { record: value, .. }
        | CompilerExpressionKind::ErrorField { error: value, .. }
        | CompilerExpressionKind::RangeLower(value)
        | CompilerExpressionKind::RangeUpper(value)
        | CompilerExpressionKind::RangeLowerInclusive(value)
        | CompilerExpressionKind::RangeUpperInclusive(value)
        | CompilerExpressionKind::RangeEmpty(value)
        | CompilerExpressionKind::Not(value) => compiler_expression_is_closed_with(value, bound),
        CompilerExpressionKind::RationalConstruct {
            numerator,
            denominator,
        } => {
            compiler_expression_is_closed_with(numerator, bound)
                && compiler_expression_is_closed_with(denominator, bound)
        }
        CompilerExpressionKind::StringConcat { left, right }
        | CompilerExpressionKind::ListContainsEntry {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListContainsSequence {
            list: left,
            pattern: right,
        }
        | CompilerExpressionKind::ListContainsSubsequence {
            list: left,
            pattern: right,
        }
        | CompilerExpressionKind::ListRemoveFirst {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListRemoveAll {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListPrepend {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListAppend {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListRangeSelect {
            list: left,
            range: right,
            ..
        }
        | CompilerExpressionKind::ListConcat { left, right }
        | CompilerExpressionKind::Binary { left, right, .. } => {
            compiler_expression_is_closed_with(left, bound)
                && compiler_expression_is_closed_with(right, bound)
        }
        CompilerExpressionKind::BooleanDecision {
            subject,
            when_true,
            when_false,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && compiler_expression_is_closed_with(when_true, bound)
                && compiler_expression_is_closed_with(when_false, bound)
        }
        CompilerExpressionKind::OrderedComparisonDecision {
            subject,
            rules,
            otherwise,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && rules.iter().all(|rule| {
                    compiler_expression_is_closed_with(&rule.operand, bound)
                        && compiler_expression_is_closed_with(&rule.action, bound)
                })
                && compiler_expression_is_closed_with(otherwise, bound)
        }
        CompilerExpressionKind::ComparisonValueDecision {
            subject,
            when_less,
            when_equal,
            when_greater,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && compiler_expression_is_closed_with(when_less, bound)
                && compiler_expression_is_closed_with(when_equal, bound)
                && compiler_expression_is_closed_with(when_greater, bound)
        }
        CompilerExpressionKind::EnumDecision {
            subject,
            rules,
            otherwise,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && rules
                    .iter()
                    .all(|rule| compiler_expression_is_closed_with(&rule.action, bound))
                && otherwise
                    .as_deref()
                    .is_none_or(|value| compiler_expression_is_closed_with(value, bound))
        }
    }
}

fn compiler_block_is_closed(block: &CompilerBlock, outer: &BTreeSet<String>) -> bool {
    let mut bound = outer.clone();
    for statement in &block.statements {
        match statement {
            CompilerStatement::Binding(binding) => {
                if !compiler_expression_is_closed_with(&binding.value, &bound) {
                    return false;
                }
                bound.insert(binding.storage_name.clone());
            }
            CompilerStatement::Discard(expression) => {
                if !compiler_expression_is_closed_with(expression, &bound) {
                    return false;
                }
            }
        }
    }
    compiler_expression_is_closed_with(&block.result, &bound)
}

fn compiler_block_is_closed_over_parameters(
    parameters: &[CompilerParameter],
    block: &CompilerBlock,
) -> bool {
    let bound = parameters
        .iter()
        .filter(|parameter| !parameter.discarded)
        .map(|parameter| parameter.name.clone())
        .collect();
    compiler_block_is_closed(block, &bound)
}

fn into_rational(expression: CompilerExpression) -> CompilerExpression {
    if expression.value_type == CompilerType::Rational {
        return expression;
    }
    let span = expression.span;
    let rational_value = exact_int(&expression).map(BigRational::from_integer);
    CompilerExpression {
        kind: CompilerExpressionKind::IntToRational(Box::new(expression)),
        value_type: CompilerType::Rational,
        int_range: None,
        rational_value,
        span,
    }
}

fn forget_character_evidence(mut expression: CompilerExpression) -> CompilerExpression {
    debug_assert_eq!(expression.value_type, CompilerType::Character);
    expression.value_type = CompilerType::String;
    expression
}

fn retain_character_evidence(mut expression: CompilerExpression) -> Option<CompilerExpression> {
    if expression.value_type == CompilerType::Character {
        return Some(expression);
    }
    if expression.value_type != CompilerType::String
        || exact_string(&expression).is_none_or(|text| character_count(&text) != 1)
    {
        return None;
    }
    expression.value_type = CompilerType::Character;
    Some(expression)
}

fn exact_string(expression: &CompilerExpression) -> Option<String> {
    match &expression.kind {
        CompilerExpressionKind::String(value) => Some(value.clone()),
        CompilerExpressionKind::StringEmpty => Some(String::new()),
        CompilerExpressionKind::StringConcat { left, right } => {
            let mut value = exact_string(left)?;
            value.push_str(&exact_string(right)?);
            Some(value)
        }
        CompilerExpressionKind::StringCharactersCollect { text, .. } => exact_string(text),
        _ => None,
    }
}

fn adapt_call_argument(
    expected: &CompilerType,
    argument: &CompilerExpression,
) -> Option<CompilerExpression> {
    if expected == &argument.value_type {
        return Some(argument.clone());
    }
    match (expected, &argument.value_type) {
        (CompilerType::Character, CompilerType::String) => {
            retain_character_evidence(argument.clone())
        }
        (CompilerType::String, CompilerType::Character) => {
            Some(forget_character_evidence(argument.clone()))
        }
        (CompilerType::Int, CompilerType::Nat) => {
            let mut value = argument.clone();
            value.value_type = CompilerType::Int;
            Some(value)
        }
        (CompilerType::Rational, CompilerType::Int | CompilerType::Nat) => {
            Some(into_rational(argument.clone()))
        }
        (CompilerType::Int, CompilerType::Rational) => {
            let rational = argument.rational_value.as_ref()?;
            (rational.denom() == &BigInt::from(1)).then(|| CompilerExpression {
                kind: CompilerExpressionKind::RationalToInt(Box::new(argument.clone())),
                value_type: CompilerType::Int,
                int_range: Some(IntRange::exact(rational.numer().clone())),
                rational_value: None,
                span: argument.span,
            })
        }
        (CompilerType::Nat, CompilerType::Int) => argument
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= BigInt::from(0))
            .then(|| CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(argument.clone())),
                value_type: CompilerType::Nat,
                int_range: argument.int_range.clone(),
                rational_value: None,
                span: argument.span,
            }),
        (CompilerType::Nat, CompilerType::Rational) => {
            let rational = argument.rational_value.as_ref()?;
            (rational.denom() == &BigInt::from(1) && rational.numer() >= &BigInt::from(0)).then(
                || {
                    let integer = CompilerExpression {
                        kind: CompilerExpressionKind::RationalToInt(Box::new(argument.clone())),
                        value_type: CompilerType::Int,
                        int_range: Some(IntRange::exact(rational.numer().clone())),
                        rational_value: None,
                        span: argument.span,
                    };
                    CompilerExpression {
                        kind: CompilerExpressionKind::IntToNat(Box::new(integer)),
                        value_type: CompilerType::Nat,
                        int_range: Some(IntRange::exact(rational.numer().clone())),
                        rational_value: None,
                        span: argument.span,
                    }
                },
            )
        }
        _ => None,
    }
}

fn flattened_product_arguments(
    sources: &[&Expression],
    arguments: &[CompilerExpression],
) -> Option<Vec<CompilerExpression>> {
    let [Expression::Product { fields, .. }] = sources else {
        return None;
    };
    let [
        CompilerExpression {
            kind: CompilerExpressionKind::Tuple(values),
            ..
        },
    ] = arguments
    else {
        return None;
    };
    fields
        .iter()
        .all(|field| field.label.is_none())
        .then(|| values.clone())
}

fn rational_absolute(value: &BigRational) -> BigRational {
    if value.numer() < &BigInt::from(0) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn is_proven_zero_numeric(expression: &CompilerExpression) -> bool {
    match expression.value_type {
        CompilerType::Int => exact_int(expression).is_some_and(|value| value == BigInt::from(0)),
        CompilerType::Rational => expression
            .rational_value
            .as_ref()
            .is_some_and(|value| value.numer() == &BigInt::from(0)),
        _ => false,
    }
}

fn is_proven_nonzero_numeric(expression: &CompilerExpression) -> bool {
    match expression.value_type {
        CompilerType::Int => expression
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < BigInt::from(0) || range.lower > BigInt::from(0)),
        CompilerType::Rational => expression
            .rational_value
            .as_ref()
            .is_some_and(|value| value.numer() != &BigInt::from(0)),
        _ => false,
    }
}

fn division_by_zero(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-DIVISION-BY-ZERO",
        span,
        "exact division requires a nonzero divisor",
    )
}

fn modular_range(modular: &CompilerModularType) -> IntRange {
    IntRange {
        lower: modular.lower.clone(),
        upper: modular.upper.clone(),
    }
}

fn reduce_modular(value: BigInt, modular: &CompilerModularType) -> BigInt {
    let modulus = &modular.upper - &modular.lower + BigInt::from(1);
    let mut residue = (value - &modular.lower) % &modulus;
    if residue < BigInt::from(0) {
        residue += &modulus;
    }
    &modular.lower + residue
}

fn modulo_range(dividend: &CompilerExpression, divisor: &CompilerExpression) -> Option<IntRange> {
    if let (Some(dividend), Some(divisor)) = (exact_int(dividend), exact_int(divisor)) {
        let magnitude = if divisor < BigInt::from(0) {
            -divisor
        } else {
            divisor
        };
        let mut remainder = dividend % &magnitude;
        if remainder < BigInt::from(0) {
            remainder += magnitude;
        }
        return Some(IntRange::exact(remainder));
    }
    let range = divisor.int_range.as_ref()?;
    let magnitude = (-range.lower.clone()).max(range.upper.clone());
    Some(IntRange {
        lower: BigInt::from(0),
        upper: magnitude - 1,
    })
}

fn power_range(base: &CompilerExpression, exponent: &CompilerExpression) -> Option<IntRange> {
    let base = exact_int(base)?;
    let exponent = exact_int(exponent)?.to_string().parse::<u32>().ok()?;
    Some(IntRange::exact(base.pow(exponent)))
}

fn exact_rational_binary(
    operation: CompilerBinary,
    left: &CompilerExpression,
    right: &CompilerExpression,
) -> Option<BigRational> {
    let left = left.rational_value.as_ref()?;
    if operation == CompilerBinary::Power {
        let exponent = exact_int(right)?.to_string().parse::<i32>().ok()?;
        Some(left.pow(exponent))
    } else {
        let right = right.rational_value.as_ref()?;
        match operation {
            CompilerBinary::Add => Some(left + right),
            CompilerBinary::Subtract => Some(left - right),
            CompilerBinary::Multiply => Some(left * right),
            CompilerBinary::Divide => Some(left / right),
            _ => None,
        }
    }
}

fn combine_ranges(
    left: &CompilerExpression,
    right: &CompilerExpression,
    operation: impl FnOnce(&IntRange, &IntRange) -> IntRange,
) -> Option<IntRange> {
    Some(operation(
        left.int_range.as_ref()?,
        right.int_range.as_ref()?,
    ))
}

fn multiply_range(left: &IntRange, right: &IntRange) -> IntRange {
    let products = [
        &left.lower * &right.lower,
        &left.lower * &right.upper,
        &left.upper * &right.lower,
        &left.upper * &right.upper,
    ];
    IntRange {
        lower: products.iter().min().expect("four products").clone(),
        upper: products.iter().max().expect("four products").clone(),
    }
}

fn absolute_range(range: &IntRange) -> IntRange {
    let zero = BigInt::from(0);
    if range.lower >= zero {
        range.clone()
    } else if range.upper <= zero {
        IntRange {
            lower: -range.upper.clone(),
            upper: -range.lower.clone(),
        }
    } else {
        IntRange {
            lower: zero,
            upper: (-range.lower.clone()).max(range.upper.clone()),
        }
    }
}

fn require_type(
    source: &SourceText,
    span: Span,
    expected: &CompilerType,
    actual: &CompilerType,
) -> Result<(), Diagnostic> {
    if expected == &CompilerType::String && actual == &CompilerType::Character {
        return Ok(());
    }
    require_same_type(source, span, expected, actual)
}

fn require_int_list(
    source: &SourceText,
    value: &CompilerExpression,
    operation: &str,
) -> Result<(), Diagnostic> {
    let CompilerType::List(element) = &value.value_type else {
        return Err(unsupported(source, value.span, operation));
    };
    if element.as_ref() == &CompilerType::Int {
        Ok(())
    } else {
        Err(unsupported(
            source,
            value.span,
            &format!("{operation} for this List element type"),
        ))
    }
}

fn require_int_or_int_pair_list(
    source: &SourceText,
    value: &CompilerExpression,
    operation: &str,
) -> Result<CompilerType, Diagnostic> {
    let CompilerType::List(element) = &value.value_type else {
        return Err(unsupported(source, value.span, operation));
    };
    if element.as_ref() == &CompilerType::Int
        || matches!(
            element.as_ref(),
            CompilerType::Tuple(fields)
                if fields.as_slice() == [CompilerType::Int, CompilerType::Int]
        )
    {
        Ok(element.as_ref().clone())
    } else {
        Err(unsupported(
            source,
            value.span,
            &format!("{operation} for this List element type"),
        ))
    }
}

fn require_int_fold_result(
    source: &SourceText,
    result: &CompilerExpression,
) -> Result<(), Diagnostic> {
    if result.value_type == CompilerType::Int
        || result.value_type == CompilerType::TraversalControl(Box::new(CompilerType::Int))
    {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            result.span,
            format!(
                "expected Int or TraversalControl Int, found {}",
                result.value_type.name()
            ),
        ))
    }
}

fn require_same_type(
    source: &SourceText,
    span: Span,
    expected: &CompilerType,
    actual: &CompilerType,
) -> Result<(), Diagnostic> {
    if expected == actual {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            span,
            format!("expected {}, found {}", expected.name(), actual.name()),
        ))
    }
}

fn unit_expression(span: Span) -> CompilerExpression {
    CompilerExpression {
        kind: CompilerExpressionKind::Unit,
        value_type: CompilerType::Unit,
        int_range: None,
        rational_value: None,
        span,
    }
}

fn mangle(name: &str) -> String {
    name.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                char::from(byte).to_string()
            } else {
                format!("_{byte:02x}")
            }
        })
        .collect()
}

fn unsupported(source: &SourceText, span: Span, construct: &str) -> Diagnostic {
    source_diagnostic(
        source,
        "E-COMPILER-UNSUPPORTED",
        span,
        format!("the current native compiler increment does not yet support {construct}"),
    )
}

fn source_diagnostic(
    source: &SourceText,
    code: impl Into<String>,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let start = span.start.min(source.as_str().len());
    let end = span.end.min(source.as_str().len()).max(start);
    let position = source.position(start);
    let line = source
        .as_str()
        .lines()
        .nth(position.line.saturating_sub(1))
        .map(ToOwned::to_owned);
    let width = source.slice(Span::new(start, end)).chars().count().max(1);
    Diagnostic::error(code, position.line, position.column, message)
        .with_source_span(span)
        .with_source_excerpt(line, width)
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::LanguageSelection { span, .. }
        | Statement::LibrarySelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::StateField { name: span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. }
        | Statement::Discard { span, .. } => *span,
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_functions_and_boolean_decisions_for_the_native_backend() {
        let source = "use language (version is v0.1)\nchoose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise 0\n(choose true, choose false)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(program.functions.len(), 2);
        assert_eq!(program.main.result.value_type.name(), "(Int, Int)");
    }

    #[test]
    fn erases_valid_diagnostic_controls_after_shared_validation() {
        // TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001, TOPAL-SYN-DIAG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/diagnostic-controls.t"
        ))
        .unwrap();
        assert_eq!(program.main.statements.len(), 2);
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

        let invalid = "use language (version is v0.1)\nlang push-disable-warning unclosed\n()\n";
        assert_eq!(
            analyze_for_compiler(invalid).unwrap_err().code,
            "E-DIAGNOSTIC-CONTROL-UNCLOSED"
        );
    }

    #[test]
    fn preserves_arbitrary_integer_ranges_for_the_native_backend() {
        let source = "use language (version is v0.1)\n9223372036854775807 + 1\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.int_range,
            Some(IntRange::exact(BigInt::from(i64::MAX) + 1))
        );
    }

    #[test]
    fn models_finite_exact_conversion_division_and_power() {
        let source = "use language (version is v0.1)\n(1 + 0.5, 6 / 8, -17 % 5, 2 ^ 16, 1 <=> 2)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Rational, Rational, Int, Int, Comparison)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected a tuple")
        };
        assert_eq!(
            values[0].rational_value,
            Some(BigRational::new(BigInt::from(3), BigInt::from(2)))
        );
        assert_eq!(values[2].int_range, Some(IntRange::exact(BigInt::from(3))));
        assert_eq!(
            values[3].int_range,
            Some(IntRange::exact(BigInt::from(65_536)))
        );
    }

    #[test]
    fn models_nat_comparison_by_forgetting_constraint_evidence() {
        // TOPAL-NUM-NAT-001, TOPAL-TYPE-EQUALITY-001,
        // TOPAL-TYPE-ORDERING-001, TOPAL-NUM-THREE-WAY-COMPARE-001
        let source = include_str!("../../../examples/language/nat-equality-and-ordering.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Comparison, Boolean, Boolean)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected a comparison result tuple")
        };
        let CompilerExpressionKind::Binary { left, right, .. } = &values[0].kind else {
            panic!("expected mixed Nat/Int equality")
        };
        assert_eq!(left.value_type, CompilerType::Int);
        assert_eq!(right.value_type, CompilerType::Int);
        let CompilerExpressionKind::Binary { left, right, .. } = &values[1].kind else {
            panic!("expected mixed Nat/Rational equality")
        };
        assert_eq!(left.value_type, CompilerType::Rational);
        assert_eq!(right.value_type, CompilerType::Rational);
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "compare-nat")
            .unwrap();
        let CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } = &function.body.result.kind
        else {
            panic!("expected Nat three-way comparison")
        };
        assert_eq!(*operation, CompilerBinary::Compare);
        assert_eq!(left.value_type, CompilerType::Int);
        assert_eq!(right.value_type, CompilerType::Int);
        let CompilerExpressionKind::Binary { left, right, .. } = &values[8].kind else {
            panic!("expected derived product equality")
        };
        assert_eq!(left.value_type.name(), "(Nat, Nat)");
        assert_eq!(right.value_type.name(), "(Nat, Nat)");

        let arithmetic = "use language (version is v0.1)\none : Nat is 1\none + one\n";
        assert_eq!(
            analyze_for_compiler(arithmetic).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn rejects_statically_zero_exact_divisors() {
        let source = "use language (version is v0.1)\n1 / 0\n";
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-DIVISION-BY-ZERO"
        );
    }

    #[test]
    fn models_finite_exact_ranges_and_observations() {
        let source = "use language (version is v0.1)\ninterval is 0 ..= 2.5\npreserve is fn (value : Range Rational) -> Range Rational\n  value\nkept is preserve interval\n(1 in kept, kept contains 3, empty? (2 .. 2), range-lower kept, range-upper-inclusive? kept, kept and (1.0 <.. 4.0))\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Boolean, Boolean, Boolean, Rational, Boolean, Range Rational)"
        );
        assert!(
            program
                .functions
                .iter()
                .any(|function| function.result_type.name() == "Range Rational")
        );
    }

    #[test]
    fn models_ordered_and_comparison_value_decisions() {
        let source = "use language (version is v0.1)\nrank is fn (value : Comparison) -> Int\n  value\n    Less then -1\n    Equal then 0\n    Greater then 1\nlocate is fn (value : Int, pivot : Rational) -> Int\n  value\n    < pivot - 0.5 then -1\n    = pivot then 0\n    otherwise 1\n(rank (1 <=> 2), 0 locate 1.5)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::ComparisonValueDecision { .. }
        )));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::OrderedComparisonDecision { .. }
        )));
    }

    #[test]
    fn models_dynamic_arithmetic_results_without_reclassifying_parameter_errors_as_static() {
        // TOPAL-COMPILER-RESULT-001
        let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  Rational (numerator, denominator)\nmodulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  left % right\n(1.0 divide 2.0, 1.0 divide 0.0, 1 ratio 0, 17 modulo 0)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Int, lang arithmetic ArithmeticErrorCode))"
        );
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::Fallible { .. }
        )));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::ResultSuccess(_)
        )));
    }

    #[test]
    fn models_optional_construction_boundaries_decisions_and_equality() {
        // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-OPTIONAL-CONTEXT-001,
        // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-DECISION-OPTIONAL-001,
        // TOPAL-TYPE-OPTIONAL-EQUALITY-001
        let source = "use language (version is v0.1)\nmissing : Optional Int is None\npreserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\nabsent is fn () -> Optional Int\n  None\ndescribe is fn (candidate : Optional Int) -> String\n  candidate\n    Some payload then \"present\"\n    None then \"absent\"\n(Some 42, Some \"present\", None Int, None String, missing, preserve (Some 7), preserve (None Int), absent (), describe (Some 7), describe missing, (None Int) = (None Int), (Some 7) != (Some 8))\n";
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| {
            function.source_name == "preserve"
                && function.parameters[0].value_type
                    == CompilerType::Optional(Box::new(CompilerType::Int))
                && function.result_type == CompilerType::Optional(Box::new(CompilerType::Int))
        }));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::OptionalDecision { .. }
        )));
        assert!(matches!(
            program.main.result.kind,
            CompilerExpressionKind::Tuple(_)
        ));
    }

    #[test]
    fn models_optional_rational_values_and_equality() {
        // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-OPTIONAL-CONTEXT-001,
        // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-DECISION-OPTIONAL-001,
        // TOPAL-TYPE-OPTIONAL-EQUALITY-001
        let source = include_str!("../../../examples/language/optional-rational-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let optional_rational = CompilerType::Optional(Box::new(CompilerType::Rational));
        assert!(program.functions.iter().any(|function| {
            function.source_name == "preserve"
                && function.parameters[0].value_type == optional_rational
                && function.result_type == optional_rational
        }));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::OptionalDecision { .. }
        )));
        assert_eq!(
            program.main.result.value_type.name(),
            "(Optional Rational, Optional Rational, Boolean, Boolean, Boolean, Boolean, Boolean, String, String)"
        );
    }

    #[test]
    fn models_static_character_constraint_evidence() {
        // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
        // TOPAL-STRING-CHARACTER-CLASSIFIER-001,
        // TOPAL-STRING-FROM-CHARACTER-001, TOPAL-TYPE-EQUALITY-001
        let source = include_str!("../../../examples/language/character-classification.t");
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| {
            function.source_name == "identity"
                && function.parameters[0].value_type == CompilerType::Character
                && function.result_type == CompilerType::Character
        }));
        assert_eq!(
            program.main.result.value_type,
            CompilerType::Tuple(vec![
                CompilerType::String,
                CompilerType::String,
                CompilerType::Boolean,
                CompilerType::Boolean,
                CompilerType::Boolean,
            ])
        );

        let invalid =
            analyze_for_compiler("use language (version is v0.1)\ninvalid : Character is \"ab\"\n")
                .unwrap_err();
        assert_eq!(invalid.code, "E-CHARACTER-CLASSIFIER");
        assert!(invalid.message.contains("contains 2"));
        let dynamic = "use language (version is v0.1)\nretain is fn (value : String) -> Character\n  value\nretain \"a\"\n";
        assert_eq!(
            analyze_for_compiler(dynamic).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_closed_character_counting_and_indexing() {
        // TOPAL-STRING-CHARACTER-COUNT-001, TOPAL-STRING-ENTRY-COUNT-001,
        // TOPAL-STRING-CHARACTER-AT-001, TOPAL-TYPE-OPTIONAL-BOUNDARY-001,
        // TOPAL-DECISION-OPTIONAL-001
        let source = include_str!("../../../examples/language/string-character-at.t");
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| {
            function.source_name == "describe"
                && function.parameters[0].value_type
                    == CompilerType::Optional(Box::new(CompilerType::Character))
                && function.result_type == CompilerType::String
        }));
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected the observation tuple")
        };
        assert_eq!(values.len(), 9);
        assert!(matches!(values[0].kind, CompilerExpressionKind::Int(_)));
        assert!(matches!(values[1].kind, CompilerExpressionKind::Int(_)));
        assert!(matches!(
            values[2].kind,
            CompilerExpressionKind::OptionalSome(_)
        ));
        assert!(matches!(
            values[5].kind,
            CompilerExpressionKind::OptionalNone
        ));
    }

    #[test]
    fn models_closed_pinned_unicode_transformations() {
        // TOPAL-STRING-UPPER-001, TOPAL-STRING-LOWER-001,
        // TOPAL-STRING-CASE-FOLD-001, TOPAL-STRING-NORMALIZE-NFC-001,
        // TOPAL-STRING-NORMALIZE-NFD-001,
        // TOPAL-STRING-CANONICAL-EQUALITY-001
        for (source, expected) in [
            (
                include_str!("../../../examples/language/string-uppercase.t"),
                "STRASSE ΣΣ",
            ),
            (
                include_str!("../../../examples/language/string-lowercase.t"),
                "i\u{307}ς",
            ),
            (
                include_str!("../../../examples/language/string-case-fold.t"),
                "strasse σσ",
            ),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            assert_eq!(
                exact_string(&program.main.result).as_deref(),
                Some(expected)
            );
        }

        for source in [
            include_str!("../../../examples/language/string-normalization.t"),
            include_str!("../../../examples/language/string-normalization-nfd.t"),
            include_str!("../../../examples/language/string-canonical-equality.t"),
        ] {
            assert!(analyze_for_compiler(source).is_ok());
        }
        let canonical = analyze_for_compiler(include_str!(
            "../../../examples/language/string-canonical-equality.t"
        ))
        .unwrap();
        let CompilerExpressionKind::Tuple(values) = &canonical.main.result.kind else {
            panic!("expected the canonical-equality tuple")
        };
        assert!(matches!(
            values[1].kind,
            CompilerExpressionKind::Boolean(true)
        ));
        assert!(matches!(
            values[2].kind,
            CompilerExpressionKind::Boolean(false)
        ));

        let specialized = analyze_for_compiler(
            "use language (version is v0.1)\ntransform is fn (value : String) -> String\n  upper value\ntransform \"a\"\n",
        )
        .unwrap();
        assert_eq!(
            exact_string(&specialized.functions[0].body.result).as_deref(),
            Some("A")
        );

        for source in [
            "use language (version is v0.1)\nidentity is fn (value : String) -> String\n  value\nupper (identity \"a\")\n",
            "use language (version is v0.1)\nidentity is fn (value : String) -> String\n  value\n(identity \"a\") normalize NFC\n",
            "use language (version is v0.1)\nidentity is fn (value : String) -> String\n  value\n(identity \"a\") canonically-equals \"a\"\n",
        ] {
            assert_eq!(
                analyze_for_compiler(source).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_anonymous_record_construction_and_selection() {
        // TOPAL-TYPE-PRODUCT-001, TOPAL-COMPILER-RECORD-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/strings-and-products.t"
        ))
        .unwrap();
        let person = program
            .main
            .statements
            .iter()
            .find_map(|statement| match statement {
                CompilerStatement::Binding(binding) if binding.name == "person" => Some(binding),
                _ => None,
            })
            .expect("the shared regression binds person");
        let CompilerExpressionKind::Record(fields) = &person.value.kind else {
            panic!("expected a checked anonymous Record")
        };
        assert_eq!(
            fields
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["name", "active"]
        );
        let CompilerType::Record(field_types) = &person.value.value_type else {
            panic!("expected an inferred Record type")
        };
        assert_eq!(
            field_types
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["active", "name"]
        );
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::RecordField { label, .. },
                    value_type: CompilerType::String,
                    ..
                },
                ..
            }) if name == "person-name" && label == "name"
        )));

        let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\ntext is \"outer\"\nrecord is (name is text)\n{\n  text is \"inner\"\n  upper (record name)\n}\n",
        )
        .unwrap();
        let CompilerExpressionKind::Block(block) = &shadowed.main.result.kind else {
            panic!("expected a lexical block")
        };
        assert_eq!(exact_string(&block.result).as_deref(), Some("OUTER"));

        let code_field = analyze_for_compiler(
            "use language (version is v0.1)\nrecord is (code is 7)\nrecord code\n",
        )
        .unwrap();
        assert_eq!(exact_int(&code_field.main.result), Some(BigInt::from(7)));

        let duplicate = analyze_for_compiler(
            "use language (version is v0.1)\nvalue is (a is 1, a is 2)\nvalue\n",
        )
        .unwrap_err();
        assert_eq!(duplicate.code, "E-DUPLICATE-RECORD-FIELD");
        let absent =
            analyze_for_compiler("use language (version is v0.1)\nvalue is (a is 1)\nvalue b\n")
                .unwrap_err();
        assert_eq!(absent.code, "E-NO-SUCH-RECORD-FIELD");
    }

    #[test]
    fn models_immutable_record_reconstruction() {
        // TOPAL-TYPE-RECONSTRUCT-001, TOPAL-COMPILER-RECONSTRUCT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/record-reconstruction.t"
        ))
        .unwrap();
        let updated = program
            .main
            .statements
            .iter()
            .find_map(|statement| match statement {
                CompilerStatement::Binding(binding) if binding.name == "updated" => Some(binding),
                _ => None,
            })
            .expect("the shared regression binds updated");
        let CompilerExpressionKind::RecordReconstruct { base, replacements } = &updated.value.kind
        else {
            panic!("expected checked Record reconstruction")
        };
        assert_eq!(updated.value.value_type, base.value_type);
        assert_eq!(replacements.len(), 1);
        assert_eq!(replacements[0].0, "age");
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("expected the observation tuple")
        };
        assert_eq!(exact_int(&results[0]), Some(BigInt::from(36)));
        assert_eq!(exact_int(&results[2]), Some(BigInt::from(37)));

        let converted = analyze_for_compiler(
            "use language (version is v0.1)\nrecord is (score is Rational (1, 2))\nupdated is record with (score is 1)\nupdated score\n",
        )
        .unwrap();
        let CompilerStatement::Binding(converted) = &converted.main.statements[1] else {
            panic!("expected updated binding")
        };
        let CompilerExpressionKind::RecordReconstruct { replacements, .. } = &converted.value.kind
        else {
            panic!("expected converted reconstruction")
        };
        assert!(matches!(
            replacements[0].1.kind,
            CompilerExpressionKind::IntToRational(_)
        ));

        for (source, code) in [
            (
                "use language (version is v0.1)\n1 with (a is 2)\n",
                "E-RECONSTRUCT-NON-RECORD",
            ),
            (
                "use language (version is v0.1)\nrecord is (a is 1)\nrecord with (a is 2, a is 3)\n",
                "E-DUPLICATE-RECONSTRUCTION-FIELD",
            ),
            (
                "use language (version is v0.1)\nrecord is (a is 1)\nrecord with (b is 2)\n",
                "E-NO-SUCH-RECORD-FIELD",
            ),
            (
                "use language (version is v0.1)\nrecord is (a is 1)\nrecord with (a is \"wrong\")\n",
                "E-TYPE-MISMATCH",
            ),
        ] {
            assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
        }
    }

    #[test]
    fn models_recursive_structural_comparisons() {
        // TOPAL-TYPE-EQUALITY-001, TOPAL-TYPE-ORDERING-001,
        // TOPAL-NUM-INT-RATIONAL-CONVERT-001,
        // TOPAL-COMPILER-STRUCTURAL-COMPARISON-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/equality-and-ordering.t"
        ))
        .unwrap();
        let binding = |name: &str| {
            program
                .main
                .statements
                .iter()
                .find_map(|statement| match statement {
                    CompilerStatement::Binding(binding) if binding.name == name => Some(binding),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("the shared regression binds `{name}`"))
        };
        let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Less,
            left,
            right,
        } = &binding("ordered").value.kind
        else {
            panic!("expected derived tuple ordering")
        };
        let ordered_type = CompilerType::Tuple(vec![
            CompilerType::Rational,
            CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]),
        ]);
        assert_eq!(left.value_type, ordered_type);
        assert_eq!(right.value_type, ordered_type);

        let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            left,
            right,
        } = &binding("same-record").value.kind
        else {
            panic!("expected derived Record equality")
        };
        let record_type = CompilerType::Record(vec![
            ("name".into(), CompilerType::String),
            ("score".into(), CompilerType::Rational),
        ]);
        assert_eq!(left.value_type, record_type);
        assert_eq!(right.value_type, record_type);
        let CompilerExpressionKind::Record(left_fields) = &left.kind else {
            panic!("expected the left Record")
        };
        let CompilerExpressionKind::Record(right_fields) = &right.kind else {
            panic!("expected the right Record")
        };
        assert_eq!(left_fields[0].0, "name");
        assert_eq!(right_fields[0].0, "score");

        let shape_error =
            analyze_for_compiler("use language (version is v0.1)\n(a is 1) = (b is 1)\n")
                .unwrap_err();
        assert_eq!(shape_error.code, "E-NO-APPLICABLE-OVERLOAD");
        for source in [
            "use language (version is v0.1)\n(value is (1 .. 2)) = (value is (1 .. 2))\n",
            "use language (version is v0.1)\n(true, 1) < (false, 1)\n",
        ] {
            assert_eq!(
                analyze_for_compiler(source).unwrap_err().code,
                "E-NO-APPLICABLE-OVERLOAD"
            );
        }
    }

    #[test]
    fn models_prospective_utf8_string_byte_counts() {
        // TOPAL-TYPE-CALL-001, TOPAL-STRING-UTF8-BYTE-COUNT-001
        let source = include_str!("../../../examples/language/string-utf8-byte-count.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program
                .main
                .statements
                .iter()
                .filter(|statement| matches!(
                    statement,
                    CompilerStatement::Binding(CompilerBinding {
                        value: CompilerExpression {
                            kind: CompilerExpressionKind::StringUtf8ByteCount(_),
                            ..
                        },
                        ..
                    })
                ))
                .count(),
            3
        );
        assert_eq!(program.main.result.value_type.name(), "(Int, Int, Int)");
    }

    #[test]
    fn models_exact_string_and_derived_optional_string_equality() {
        // TOPAL-TYPE-EQUALITY-001, TOPAL-TYPE-OPTIONAL-EQUALITY-001
        let source = include_str!("../../../examples/language/string-exact-equality.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected equality result tuple")
        };
        assert!(values.iter().all(|value| matches!(
            value.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Equal | CompilerBinary::NotEqual,
                ..
            }
        )));

        let unsupported_field = "use language (version is v0.1)\nleft is (1 .. 2, true)\nright is (1 .. 2, true)\nleft = right\n";
        assert_eq!(
            analyze_for_compiler(unsupported_field).unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn models_string_construction_concatenation_and_emptiness() {
        // TOPAL-STRING-EMPTY-001, TOPAL-STRING-LITERAL-COMPOSE-001,
        // TOPAL-STRING-CONCAT-001, TOPAL-STRING-EMPTY-PREDICATE-001
        let source = include_str!("../../../examples/language/string-construction.t");
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::StringEmpty,
                    ..
                },
                ..
            })
        )));
        assert_eq!(
            program
                .main
                .statements
                .iter()
                .filter(|statement| matches!(
                    statement,
                    CompilerStatement::Binding(CompilerBinding {
                        value: CompilerExpression {
                            kind: CompilerExpressionKind::StringConcat { .. },
                            ..
                        },
                        ..
                    })
                ))
                .count(),
            4
        );
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::String(value),
                    ..
                },
                ..
            }) if value == "adjacent literals"
        )));
        assert_eq!(
            program.main.result.value_type.name(),
            "(String, Boolean, Boolean, Boolean, String, String, String, String, String)"
        );
    }

    #[test]
    fn models_recursive_positional_product_equality() {
        // TOPAL-TYPE-PRODUCT-001, TOPAL-TYPE-EQUALITY-001
        let source = include_str!("../../../examples/language/tuple-equality.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Boolean, Boolean, Boolean, Boolean, Boolean)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected equality result tuple")
        };
        assert!(values.iter().all(|value| matches!(
            value.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Equal | CompilerBinary::NotEqual,
                ..
            }
        )));
    }

    #[test]
    fn rejects_equality_between_distinct_optional_classifiers() {
        // TOPAL-TYPE-OPTIONAL-EQUALITY-001
        let source = "use language (version is v0.1)\n(None Int) = (None String)\n";
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn models_optional_otherwise_as_the_missing_alternative_without_a_payload_binding() {
        // TOPAL-DECISION-OPTIONAL-001
        let source = "use language (version is v0.1)\ndescribe is fn (candidate : Optional Int) -> Int\n  candidate\n    None then 0\n    otherwise 1\n(describe (Some 42), describe (None Int))\n";
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "describe")
            .unwrap();
        let CompilerExpressionKind::OptionalDecision { some_binding, .. } =
            &function.body.result.kind
        else {
            panic!("expected Optional decision")
        };
        assert!(some_binding.is_none());
    }

    #[test]
    fn models_exact_validation_and_contextual_result_projection() {
        // TOPAL-COMPILER-RESULT-001
        let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nincrement is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  quotient : Rational is 1.0 divide denominator\n  quotient + 1.0\nas-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  Int value\nas-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  Nat value\n(increment 0.0, as-int 1.5, as-nat -1)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert!(program.functions.iter().any(|function| {
            function.body.statements.iter().any(|statement| {
                matches!(
                    statement,
                    CompilerStatement::Binding(CompilerBinding {
                        value: CompilerExpression {
                            kind: CompilerExpressionKind::ResultProject(_),
                            ..
                        },
                        ..
                    })
                )
            })
        }));

        let repeated = "use language (version is v0.1)\nColor is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"first\"\n    Red then 42\n    Green then \"green\"\nname Red\n";
        let repeated = analyze_for_compiler(repeated).unwrap();
        assert!(repeated.functions.iter().any(|function| matches!(
            &function.body.result.kind,
            CompilerExpressionKind::EnumDecision { rules, .. } if rules.len() == 2
        )));
        assert!(program.functions.iter().any(|function| matches!(
            function.body.result.kind,
            CompilerExpressionKind::Validate { .. }
        )));
    }

    #[test]
    fn models_result_decisions_and_structured_error_observation() {
        // TOPAL-DECISION-RESULT-001, TOPAL-DECISION-ERROR-CODE-001,
        // TOPAL-ERROR-FIELD-001
        let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error problem then \"other\"\nexhaustive is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic out-of-range) then \"range\"\n    Error (code is lang arithmetic not-representable) then \"representation\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error (code is lang arithmetic indeterminate) then \"indeterminate\"\nproblem is 1.0 divide 0.0\n(describe 0.0, exhaustive 0.0, problem code, problem domain)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(String, String, lang arithmetic ArithmeticErrorCode, ErrorDomain)"
        );
        assert!(program.functions.iter().any(|function| matches!(
            &function.body.result.kind,
            CompilerExpressionKind::ResultDecision {
                error_codes,
                error_fallback: Some(_),
                ..
            } if error_codes.len() == 1
        )));
        assert!(program.functions.iter().any(|function| matches!(
            &function.body.result.kind,
            CompilerExpressionKind::ResultDecision {
                error_codes,
                error_fallback: None,
                ..
            } if error_codes.len() == 4
        )));
        let CompilerExpressionKind::Tuple(fields) = &program.main.result.kind else {
            panic!("expected top-level tuple")
        };
        assert!(matches!(
            fields[2].kind,
            CompilerExpressionKind::ErrorField {
                field: CompilerErrorField::Code,
                ..
            }
        ));
        assert!(matches!(
            fields[3].kind,
            CompilerExpressionKind::ErrorField {
                field: CompilerErrorField::Domain,
                ..
            }
        ));
    }

    #[test]
    fn models_optional_structured_error_fields() {
        // TOPAL-COMPILER-ERROR-OPTIONAL-FIELDS-001, TOPAL-ERROR-FIELD-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/optional-result-composition.t"
        ))
        .unwrap();
        let CompilerExpressionKind::Tuple(fields) = &program.main.result.kind else {
            panic!("expected top-level tuple")
        };
        for (index, field, payload) in [
            (4, CompilerErrorField::Detail, CompilerType::String),
            (5, CompilerErrorField::Cause, CompilerType::Error),
            (6, CompilerErrorField::Source, CompilerType::SourceLocation),
        ] {
            assert_eq!(
                fields[index].value_type,
                CompilerType::Optional(Box::new(payload))
            );
            assert!(matches!(
                fields[index].kind,
                CompilerExpressionKind::ErrorField { field: actual, .. } if actual == field
            ));
        }
    }

    #[test]
    fn models_qualified_arithmetic_error_code_values() {
        // TOPAL-NUM-ARITHMETIC-ERROR-001
        let source = "use language (version is v0.1)\nretain is fn (value : ErrorCode) -> ErrorCode\n  value\n(retain (lang arithmetic division-by-zero), lang arithmetic indeterminate, (lang arithmetic out-of-range) = (lang arithmetic out-of-range))\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(lang arithmetic ArithmeticErrorCode, lang arithmetic ArithmeticErrorCode, Boolean)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected a tuple")
        };
        assert!(matches!(
            values[1].kind,
            CompilerExpressionKind::ErrorCode(3)
        ));
    }

    #[test]
    fn models_qualified_generator_error_code_as_a_nominal_value() {
        // TOPAL-GENERATOR-ERROR-CODE-001,
        // TOPAL-COMPILER-GENERATOR-ERROR-CODE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/generator-error-codes.t"
        ))
        .unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(lang generator GeneratorErrorCode, Boolean)"
        );
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Enum(0),
                    value_type: CompilerType::Enum(enumeration),
                    ..
                },
                ..
            }) if enumeration.name == "lang generator GeneratorErrorCode"
                && enumeration.alternatives == ["generator-closed"]
        )));

        let unknown = "use language (version is v0.1)\nlang generator generator-reopened\n";
        assert_eq!(
            analyze_for_compiler(unknown).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
        let arithmetic_mismatch = "use language (version is v0.1)\n(lang generator generator-closed) = (lang arithmetic division-by-zero)\n";
        assert_eq!(
            analyze_for_compiler(arithmetic_mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn models_lazy_int_iterate_construction_without_invoking_captured_functions() {
        // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
        // TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001
        let unbounded = analyze_for_compiler(include_str!(
            "../../../examples/language/iterate-generator.t"
        ))
        .unwrap();
        assert_eq!(unbounded.main.result.value_type, int_unit_generator_type());
        assert!(unbounded.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::IterateGenerator {
                        initial,
                        parameters,
                        next,
                    },
                    ..
                },
                ..
            }) if matches!(initial.kind, CompilerExpressionKind::Int(ref value) if value == &BigInt::from(0))
                && parameters.len() == 1
                && matches!(next.result.kind, CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    ..
                })
        )));

        let bounded = analyze_for_compiler(include_str!(
            "../../../examples/language/iterate-take-while.t"
        ))
        .unwrap();
        assert_eq!(bounded.main.result.value_type, int_unit_generator_type());
        assert!(bounded.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::GeneratorTakeWhile {
                        generator,
                        parameters,
                        predicate,
                    },
                    ..
                },
                ..
            }) if matches!(generator.kind, CompilerExpressionKind::IterateGenerator { .. })
                && parameters.len() == 1
                && matches!(predicate.result.kind, CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Less,
                    ..
                })
        )));

        let moved = analyze_for_compiler(
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\nmoved is numbers\nmoved\n",
        )
        .unwrap();
        assert_eq!(moved.main.result.value_type, int_unit_generator_type());

        for (source, code) in [
            (
                "use language (version is v0.1)\n0 iterate ({ value } value < 1)\n",
                "E-TYPE-MISMATCH",
            ),
            (
                "use language (version is v0.1)\n0 iterate ({ value } value + 1) take-while ({ value } value + 1)\n",
                "E-TYPE-MISMATCH",
            ),
            (
                "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\n(numbers, numbers)\n",
                "E-GENERATOR-CONSUMED",
            ),
            (
                "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\n()\n",
                "E-COMPILER-UNSUPPORTED",
            ),
            (
                "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\n(numbers,)\n",
                "E-COMPILER-UNSUPPORTED",
            ),
            (
                "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\nroot numbers\n",
                "E-COMPILER-UNSUPPORTED",
            ),
            (
                "use language (version is v0.1)\n(0 iterate ({ value } value + 1)) = (1 iterate ({ value } value + 1))\n",
                "E-COMPILER-UNSUPPORTED",
            ),
        ] {
            assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
        }
    }

    #[test]
    fn models_direct_bounded_int_iterate_collection() {
        // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
        // TOPAL-GENERATOR-COLLECT-001,
        // TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/generated-collect.t"
        ))
        .unwrap();
        assert_eq!(
            program.main.result.value_type,
            CompilerType::List(Box::new(CompilerType::Int))
        );
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::GeneratorCollect(generator),
                    ..
                },
                ..
            }) if name == "digits"
                && matches!(generator.kind, CompilerExpressionKind::GeneratorTakeWhile {
                    generator: ref source,
                    ..
                } if matches!(source.kind, CompilerExpressionKind::IterateGenerator { .. }))
        )));
        analyze_for_compiler(
            "use language (version is v0.1)\ncollect ((0 iterate ({ value } value + 1)) take-while ({ value } value < 2))\n",
        )
        .unwrap();

        let unbounded = analyze_for_compiler(
            "use language (version is v0.1)\ncollect (0 iterate ({ value } value + 1))\n",
        )
        .unwrap_err();
        assert_eq!(unbounded.code, "E-UNBOUNDED-GENERATOR-COLLECT");

        let indirect = analyze_for_compiler(
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1) take-while ({ value } value < 5)\ncollect numbers\n",
        )
        .unwrap_err();
        assert_eq!(indirect.code, "E-COMPILER-UNSUPPORTED");

        for captured in [
            "use language (version is v0.1)\nstep is 1\ncollect (0 iterate ({ value } value + step) take-while ({ value } value < 5))\n",
            "use language (version is v0.1)\nlimit is 5\ncollect (0 iterate ({ value } value + 1) take-while ({ value } value < limit))\n",
        ] {
            assert_eq!(
                analyze_for_compiler(captured).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_capture_free_bounded_int_iterate_foreach() {
        // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
        // TOPAL-GENERATOR-ITERATE-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/generated-foreach.t"
        ))
        .unwrap();
        assert_eq!(program.main.result.value_type, CompilerType::Unit);
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::IterateGeneratorForeach {
                        generator,
                        parameter,
                        body,
                    },
                    ..
                },
                ..
            }) if name == "completed"
                && parameter.name == "digit"
                && body.result.value_type == CompilerType::Unit
                && matches!(generator.kind, CompilerExpressionKind::GeneratorTakeWhile { .. })
        )));

        for (source, code) in [
            (
                "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\ncompleted is numbers foreach { digit }\n  _ is digit\ncompleted\n",
                "E-UNBOUNDED-GENERATOR-TRAVERSAL",
            ),
            (
                "use language (version is v0.1)\nstart is 0\ndigits is start iterate ({ value } value + 1) take-while ({ value } value < 2)\ncompleted is digits foreach { digit }\n  _ is digit\ncompleted\n",
                "E-COMPILER-UNSUPPORTED",
            ),
            (
                "use language (version is v0.1)\nstep is 1\ndigits is 0 iterate ({ value } value + step) take-while ({ value } value < 2)\ncompleted is digits foreach { digit }\n  _ is digit\ncompleted\n",
                "E-COMPILER-UNSUPPORTED",
            ),
            (
                "use language (version is v0.1)\nlimit is 1\ndigits is 0 iterate ({ value } value + 1) take-while ({ value } value < 2)\ncompleted is digits foreach { digit }\n  _ is digit + limit\ncompleted\n",
                "E-UNBOUND-NAME",
            ),
        ] {
            assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
        }
    }

    #[test]
    fn models_closed_string_character_foreach() {
        // TOPAL-STRING-CHARACTERS-COLLECT-001,
        // TOPAL-STRING-CHARACTERS-FOREACH-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/string-character-foreach.t"
        ))
        .unwrap();
        assert_eq!(program.main.result.value_type, CompilerType::Unit);
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersForeach {
                    characters,
                    parameter,
                    body,
                    ..
                },
                ..
            }) if characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
                && parameter.name == "character"
                && parameter.value_type == CompilerType::Character
                && body.result.value_type == CompilerType::Unit
        )));

        for (source, code) in [
            (
                "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ncharacters (identity \"a\") foreach { character }\n  _ is String character\n",
                "E-COMPILER-UNSUPPORTED",
            ),
            (
                "use language (version is v0.1)\noutside is \"x\"\ncharacters \"a\" foreach { character }\n  _ is outside\n",
                "E-UNBOUND-NAME",
            ),
            (
                "use language (version is v0.1)\ncharacters \"a\" foreach { character }\n  String character\n",
                "E-TYPE-MISMATCH",
            ),
        ] {
            assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
        }

        let empty = analyze_for_compiler(
            "use language (version is v0.1)\ncharacters \"\" foreach { character }\n  _ is String character\n",
        )
        .unwrap();
        assert!(matches!(
            &empty.main.statements[0],
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersForeach { characters, .. },
                ..
            }) if characters.is_empty()
        ));
    }

    #[test]
    fn models_closed_string_character_collection() {
        // TOPAL-STRING-CHARACTERS-COLLECT-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/string-character-traversal.t"
        ))
        .unwrap();
        assert!(matches!(
            &program.main.result,
            CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersCollect { text, characters },
                value_type: CompilerType::String,
                ..
            } if matches!(text.kind, CompilerExpressionKind::String(_))
                && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
        ));
        assert_eq!(
            exact_string(&program.main.result).as_deref(),
            Some("a\u{301}👩‍🔬🇸🇪")
        );

        let empty = analyze_for_compiler(
            "use language (version is v0.1)\ncharacters \"\" collect String\n",
        )
        .unwrap();
        assert!(matches!(
            &empty.main.result.kind,
            CompilerExpressionKind::StringCharactersCollect { characters, .. }
                if characters.is_empty()
        ));

        let dynamic = "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ncharacters (identity \"a\") collect String\n";
        assert_eq!(
            analyze_for_compiler(dynamic).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_named_linear_string_character_generator() {
        // TOPAL-STRING-CHARACTERS-COLLECT-001,
        // TOPAL-STRING-CHARACTERS-FOREACH-001,
        // TOPAL-STRING-CHARACTERS-GENERATOR-001,
        // TOPAL-STRING-CHARACTERS-CLASSIFIER-001,
        // TOPAL-STRING-CHARACTERS-LINEAR-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-GENERATOR-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/string-named-character-generator.t"
        ))
        .unwrap();
        assert!(matches!(
            &program.main.statements[0],
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::StringCharactersGenerator {
                        characters,
                        ..
                    },
                    value_type: CompilerType::Generator(CompilerGeneratorType {
                        yield_type,
                        resume_type,
                        result_type,
                    }),
                    ..
                },
                ..
            }) if name == "generated"
                && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
                && yield_type.as_ref() == &CompilerType::Character
                && resume_type.as_ref() == &CompilerType::Unit
                && result_type.as_ref() == &CompilerType::Unit
        ));
        assert!(matches!(
            &program.main.statements[1],
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersForeach {
                    source,
                    characters,
                    ..
                },
                ..
            }) if matches!(source.kind, CompilerExpressionKind::Local(_))
                && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
        ));

        let consumed_twice = "use language (version is v0.1)\ngenerated : Generator Character Unit Unit is characters \"a\"\ngenerated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n";
        assert_eq!(
            analyze_for_compiler(consumed_twice).unwrap_err().code,
            "E-GENERATOR-CONSUMED"
        );
        let abandoned = "use language (version is v0.1)\ngenerated : Generator Character Unit Unit is characters \"a\"\n";
        assert_eq!(
            analyze_for_compiler(abandoned).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_owned_close_of_transferred_string_character_generator() {
        // TOPAL-STRING-CHARACTERS-GENERATOR-001,
        // TOPAL-STRING-CHARACTERS-PARAMETER-001,
        // TOPAL-STRING-CHARACTERS-CLOSE-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/string-character-generator-close.t"
        ))
        .unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "ignore")
            .expect("called close function is instantiated");
        assert!(matches!(
            function.parameters.as_slice(),
            [CompilerParameter {
                name,
                value_type,
                ..
            }] if name == "generated" && is_character_unit_generator_type(value_type)
        ));
        assert!(matches!(
            function.body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersClose(generator),
                value_type: CompilerType::Unit,
                ..
            })] if matches!(generator.kind, CompilerExpressionKind::Local(ref name) if name == "generated")
        ));
        assert!(matches!(
            &program.main.result,
            CompilerExpression {
                kind: CompilerExpressionKind::Call { arguments, .. },
                value_type: CompilerType::Unit,
                ..
            } if matches!(arguments.as_slice(), [CompilerExpression {
                kind: CompilerExpressionKind::Local(_),
                value_type,
                ..
            }] if is_character_unit_generator_type(value_type))
        ));

        for source in [
            "use language (version is v0.1)\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  _ is 1\ngenerated is characters \"a\"\nignore generated\n",
            "use language (version is v0.1)\nignore is fn (left : Generator Character Unit Unit, right : Generator Character Unit Unit) -> Unit\n  ()\nleft is characters \"a\"\nright is characters \"b\"\nignore (left, right)\n",
        ] {
            assert_eq!(
                analyze_for_compiler(source).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_specialized_string_character_generator_parameter_traversal() {
        // TOPAL-STRING-CHARACTERS-FOREACH-001,
        // TOPAL-STRING-CHARACTERS-GENERATOR-001,
        // TOPAL-STRING-CHARACTERS-PARAMETER-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/string-character-generator-parameter.t"
        ))
        .unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("called traversal function is instantiated");
        assert!(matches!(
            function.parameters.as_slice(),
            [CompilerParameter { value_type, .. }]
                if is_character_unit_generator_type(value_type)
        ));
        assert!(matches!(
            function.body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersForeach {
                    source,
                    characters,
                    parameter,
                    ..
                },
                ..
            })] if matches!(source.kind, CompilerExpressionKind::Local(ref name) if name == "generated")
                && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
                && parameter.value_type == CompilerType::Character
        ));
        assert!(!function.body.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersClose(_),
                ..
            })
        )));

        let distinct = analyze_for_compiler(
            "use language (version is v0.1)\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\nfirst is characters \"a\"\n_ is consume first\nsecond is characters \"🇸🇪\"\nconsume second\n",
        )
        .unwrap();
        let traversals = distinct
            .functions
            .iter()
            .filter(|function| function.source_name == "consume")
            .map(|function| match &function.body.statements[0] {
                CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::StringCharactersForeach { characters, .. },
                    ..
                }) => characters.clone(),
                statement => panic!("expected specialized traversal, found {statement:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            traversals,
            [vec![String::from("a")], vec![String::from("🇸🇪")]]
        );

        let extra_statement = "use language (version is v0.1)\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  _ is 1\n  generated foreach { character }\n    _ is String character\ngenerated is characters \"a\"\nconsume generated\n";
        assert_eq!(
            analyze_for_compiler(extra_statement).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
        let direct_function_traversal = "use language (version is v0.1)\nconsume is fn () -> Unit\n  characters \"a\" foreach { character }\n    _ is String character\nconsume ()\n";
        assert_eq!(
            analyze_for_compiler(direct_function_traversal)
                .unwrap_err()
                .code,
            "E-COMPILER-UNSUPPORTED"
        );
        let nested_transfer = "use language (version is v0.1)\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\nouter is fn () -> Unit\n  generated is characters \"a\"\n  consume generated\nouter ()\n";
        let nested_transfer = analyze_for_compiler(nested_transfer).unwrap_err();
        assert_eq!(nested_transfer.code, "E-COMPILER-UNSUPPORTED");
        assert!(
            nested_transfer
                .message
                .contains("nested Character Generator parameter transfer")
        );
    }

    #[test]
    fn models_lazy_list_int_unfold_with_distinct_seed_and_yield_types() {
        // TOPAL-GENERATOR-UNFOLD-001,
        // TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/unfold-generator.t"
        ))
        .unwrap();
        assert_eq!(program.main.result.value_type, int_unit_generator_type());
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::UnfoldGenerator {
                        seed,
                        parameters,
                        step,
                    },
                    ..
                },
                ..
            }) if name == "generated"
                && matches!(seed.kind, CompilerExpressionKind::Local(_))
                && parameters.len() == 1
                && matches!(step.result.kind, CompilerExpressionKind::ListUncons(_))
        )));

        for (invalid, code) in [
            (
                "use language (version is v0.1)\n0 unfold ({ value } value)\n",
                "E-TYPE-MISMATCH",
            ),
            (
                "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\nvalues unfold ({ remaining } remaining)\n",
                "E-TYPE-MISMATCH",
            ),
            (
                "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\ngenerated is values unfold ({ remaining } uncons remaining)\n(generated, generated)\n",
                "E-GENERATOR-CONSUMED",
            ),
            (
                "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\ngenerated is values unfold ({ remaining } uncons remaining)\n()\n",
                "E-COMPILER-UNSUPPORTED",
            ),
        ] {
            assert_eq!(analyze_for_compiler(invalid).unwrap_err().code, code);
        }
    }

    #[test]
    fn models_direct_finite_list_uncons_unfold_collection() {
        // TOPAL-GENERATOR-UNFOLD-001, TOPAL-GENERATOR-UNFOLD-COLLECT-001,
        // TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/unfold-collect.t"))
                .unwrap();
        assert_eq!(
            program.main.result.value_type,
            CompilerType::List(Box::new(CompilerType::Int))
        );
        assert!(matches!(
            program.main.result.kind,
            CompilerExpressionKind::GeneratorCollect(ref generator)
                if matches!(generator.kind, CompilerExpressionKind::UnfoldGenerator { .. })
        ));

        let moved = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Entry (2, Empty))\ngenerated is values unfold ({ remaining } uncons remaining)\nmoved is generated\ncollect moved\n",
        )
        .unwrap();
        assert_eq!(
            moved.main.result.value_type,
            CompilerType::List(Box::new(CompilerType::Int))
        );

        let non_finite = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\ncollect (values unfold ({ remaining } Some (1, remaining)))\n",
        )
        .unwrap_err();
        assert_eq!(non_finite.code, "E-COMPILER-UNSUPPORTED");
    }

    #[test]
    fn models_lexical_blocks_with_fresh_shadowing_scope() {
        // TOPAL-SYN-GRAMMAR-001, TOPAL-EXEC-BLOCK-001
        let source = "use language (version is v0.1)\nColor is Enum (Red, Green)\nidentity is fn (number : Int) -> Int\n  number\nempty is {}\nvalue is 40\nshadow is {\n  Red is value + 1\n  identity is Red + 1\n  (Red, identity)\n}\n(empty, shadow, Red, identity 43, value)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Unit, (Int, Int), Color, Int, Int)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected a tuple")
        };
        assert!(program.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Block(_),
                    ..
                },
                ..
            }) if name == "shadow"
        )));
        assert!(matches!(values[2].kind, CompilerExpressionKind::Enum(0)));
        assert!(matches!(
            values[3].kind,
            CompilerExpressionKind::Call { .. }
        ));
        assert_eq!(exact_int(&values[4]), Some(BigInt::from(40)));

        let closed_zero = "use language (version is v0.1)\n1 / {\n  zero is 0\n  zero\n}\n";
        assert_eq!(
            analyze_for_compiler(closed_zero).unwrap_err().code,
            "E-DIVISION-BY-ZERO"
        );
    }

    #[test]
    fn models_completed_as_distinct_zero_data_evidence() {
        // TOPAL-EXEC-COMPLETED-001
        let source = "use language (version is v0.1)\nfinish is fn () -> Completed\n  Completed\nretain is fn (value : Completed) -> Completed\n  value\n(finish (), retain Completed, Completed = Completed)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Completed, Completed, Boolean)"
        );
        assert!(program.functions.iter().all(|function| {
            function.result_type == CompilerType::Completed
                && function.body.result.value_type == CompilerType::Completed
        }));
        assert_ne!(CompilerType::Completed, CompilerType::Unit);
    }

    #[test]
    fn models_empty_effect_as_a_distinct_inert_scalar() {
        // TOPAL-EFFECT-EMPTY-001, TOPAL-EFFECT-CLASSIFIER-001,
        // TOPAL-EFFECT-BOUNDARY-001, TOPAL-EFFECT-PRODUCT-001
        let boundary = analyze_for_compiler(include_str!(
            "../../../examples/language/effect-function-boundary.t"
        ))
        .unwrap();
        assert_eq!(boundary.functions.len(), 1);
        let function = &boundary.functions[0];
        assert_eq!(function.parameters[0].value_type, CompilerType::Effect);
        assert_eq!(function.result_type, CompilerType::Effect);
        assert_eq!(function.body.result.value_type, CompilerType::Effect);
        assert!(matches!(
            boundary.main.result.kind,
            CompilerExpressionKind::Call { .. }
        ));

        let pair =
            analyze_for_compiler(include_str!("../../../examples/language/effect-products.t"))
                .unwrap();
        assert_eq!(
            pair.main.result.value_type,
            CompilerType::Tuple(vec![CompilerType::Effect, CompilerType::Effect])
        );
        let unit = analyze_for_compiler(include_str!(
            "../../../examples/language/unit-effect-value.t"
        ))
        .unwrap();
        assert_eq!(
            unit.main.result.value_type,
            CompilerType::Tuple(vec![CompilerType::Unit, CompilerType::Effect])
        );
        assert_ne!(CompilerType::Effect, CompilerType::Unit);
        assert_ne!(CompilerType::Effect, CompilerType::Completed);

        for source in [
            include_str!("../../../examples/language/empty-effects.t"),
            include_str!("../../../examples/language/effect-classifier.t"),
            include_str!("../../../examples/language/effect-identity.t"),
        ] {
            assert!(analyze_for_compiler(source).is_ok());
        }
    }

    #[test]
    fn models_contextual_effect_list_construction() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COMPILER-LIST-EFFECT-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/effect-list.t")).unwrap();
        let [CompilerStatement::Binding(rows)] = program.main.statements.as_slice() else {
            panic!("shared regression binds one List value")
        };
        assert_eq!(
            rows.value.value_type,
            CompilerType::List(Box::new(CompilerType::Effect))
        );
        let CompilerExpressionKind::ListEntry { value, remaining } = &rows.value.kind else {
            panic!("expected contextual Entry construction")
        };
        assert!(matches!(value.kind, CompilerExpressionKind::Effect));
        assert!(matches!(remaining.kind, CompilerExpressionKind::ListEmpty));
        assert!(matches!(
            program.main.result.kind,
            CompilerExpressionKind::Local(ref storage) if storage == &rows.storage_name
        ));

        for invalid in [
            "use language (version is v0.1)\nrows : List Effect is Entry (Completed, Empty)\nrows\n",
            "use language (version is v0.1)\nrows : List Effect is Entry (Effects (), Effects ())\nrows\n",
        ] {
            assert_eq!(
                analyze_for_compiler(invalid).unwrap_err().code,
                "E-TYPE-MISMATCH"
            );
        }
    }

    #[test]
    fn models_int_list_containment_without_erasing_list_identity() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-CONTAINS-ENTRY-001,
        // TOPAL-LIST-CONTAINS-SEQUENCE-001, TOPAL-LIST-CONTAINS-SUBSEQUENCE-001,
        // TOPAL-COMPILER-LIST-INT-CONTAINMENT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/list-containment.t"
        ))
        .unwrap();
        for statement in &program.main.statements {
            let CompilerStatement::Binding(binding) = statement else {
                continue;
            };
            assert_eq!(
                binding.value.value_type,
                CompilerType::List(Box::new(CompilerType::Int))
            );
        }
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared containment regression returns a Tuple")
        };
        assert_eq!(results.len(), 6);
        assert!(matches!(
            results[0].kind,
            CompilerExpressionKind::ListContainsEntry { .. }
        ));
        assert!(matches!(
            results[2].kind,
            CompilerExpressionKind::ListContainsSequence { .. }
        ));
        assert!(matches!(
            results[3].kind,
            CompilerExpressionKind::ListContainsSubsequence { .. }
        ));
        assert!(
            results
                .iter()
                .all(|result| result.value_type == CompilerType::Boolean)
        );

        let mismatch = "use language (version is v0.1)\nvalues : List Int is Empty\nvalues contains-entry \"no\"\n";
        assert_eq!(
            analyze_for_compiler(mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
        let unavailable = "use language (version is v0.1)\nvalues : List Effect is Empty\nvalues contains-entry Effects ()\n";
        assert_eq!(
            analyze_for_compiler(unavailable).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_immutable_int_list_removal() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-REMOVE-FIRST-001,
        // TOPAL-LIST-REMOVE-ALL-001, TOPAL-COMPILER-LIST-INT-REMOVAL-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/list-removal.t"))
                .unwrap();
        let [CompilerStatement::Binding(values)] = program.main.statements.as_slice() else {
            panic!("shared removal regression binds one List value")
        };
        let list_int = CompilerType::List(Box::new(CompilerType::Int));
        assert_eq!(values.value.value_type, list_int);
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared removal regression returns a Tuple")
        };
        assert_eq!(results.len(), 3);
        assert!(matches!(
            results[0].kind,
            CompilerExpressionKind::ListRemoveFirst { .. }
        ));
        assert!(
            results[1..]
                .iter()
                .all(|result| matches!(result.kind, CompilerExpressionKind::ListRemoveAll { .. }))
        );
        assert!(results.iter().all(|result| result.value_type == list_int));

        let mismatch = "use language (version is v0.1)\nvalues : List Int is Empty\nvalues remove-all \"no\"\n";
        assert_eq!(
            analyze_for_compiler(mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
        let unavailable = "use language (version is v0.1)\nvalues : List Effect is Empty\nvalues remove-first Effects ()\n";
        assert_eq!(
            analyze_for_compiler(unavailable).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_basic_int_list_operations_and_total_decomposition() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
        // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-PREPEND-001,
        // TOPAL-LIST-APPEND-001, TOPAL-LIST-CONCAT-001,
        // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
        // TOPAL-LIST-EMPTY-001, TOPAL-LIST-ONE-001, TOPAL-LIST-UNCONS-001,
        // TOPAL-LIST-FIRST-001, TOPAL-LIST-REST-001, TOPAL-LIST-REVERSE-001,
        // TOPAL-COMPILER-LIST-INT-CORE-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/lists.t")).unwrap();
        let list_int = CompilerType::List(Box::new(CompilerType::Int));
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].parameters[0].value_type, list_int);
        assert_eq!(
            program.functions[0].result_type,
            CompilerType::Optional(Box::new(CompilerType::Int))
        );
        assert!(matches!(
            program.functions[0].body.result.kind,
            CompilerExpressionKind::ListDecision { .. }
        ));
        assert_eq!(program.main.statements.len(), 8);
        assert!(program.main.statements.iter().all(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding { value, .. }) if value.value_type == list_int
        )));
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared List regression returns a Tuple")
        };
        assert_eq!(results.len(), 12);
        assert!(matches!(
            results[1].kind,
            CompilerExpressionKind::ListFirst(_)
        ));
        assert!(matches!(
            results[2].kind,
            CompilerExpressionKind::ListRest(_)
        ));
        assert!(matches!(
            results[5].kind,
            CompilerExpressionKind::ListEntryCount(_)
        ));
        assert!(matches!(
            results[6].kind,
            CompilerExpressionKind::ListEmptyPredicate(_)
        ));
        assert!(matches!(
            results[8].kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Equal,
                ..
            }
        ));
        assert!(matches!(
            results[9].kind,
            CompilerExpressionKind::ListUncons(_)
        ));

        let mismatch =
            "use language (version is v0.1)\nvalues : List Int is Empty\nvalues append \"no\"\n";
        assert_eq!(
            analyze_for_compiler(mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );

        let incomplete = "use language (version is v0.1)\ninspect is fn (values : List Int) -> Int\n  values\n    Empty then 0\ninspect Empty\n";
        assert_eq!(
            analyze_for_compiler(incomplete).unwrap_err().code,
            "E-UNSUPPORTED-INCOMPLETE-DECISION"
        );

        let duplicate_binding = "use language (version is v0.1)\ninspect is fn (values : List Int) -> Int\n  values\n    Empty then 0\n    Entry (value, value) then value\nvalues : List Int is Entry (1, Empty)\ninspect values\n";
        assert_eq!(
            analyze_for_compiler(duplicate_binding).unwrap_err().code,
            "E-DUPLICATE-BINDING"
        );
    }

    #[test]
    fn models_contextual_int_list_map_select_and_fold() {
        // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
        // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
        // TOPAL-COMPILER-LIST-INT-FUNCTIONS-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-list-functions.t"
        ))
        .unwrap();
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared anonymous List regression returns a Tuple")
        };
        assert_eq!(results.len(), 3);
        assert!(matches!(
            results[0].kind,
            CompilerExpressionKind::ListMap { .. }
        ));
        assert!(matches!(
            results[1].kind,
            CompilerExpressionKind::ListSelect { .. }
        ));
        assert!(matches!(
            results[2].kind,
            CompilerExpressionKind::ListFold { .. }
        ));
        assert_eq!(
            results[0].value_type,
            CompilerType::List(Box::new(CompilerType::Int))
        );
        assert_eq!(results[2].value_type, CompilerType::Int);

        let wrong_map = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues map { value } value > 0\n";
        assert_eq!(
            analyze_for_compiler(wrong_map).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
        let wrong_select = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues select { value } value + 1\n";
        assert_eq!(
            analyze_for_compiler(wrong_select).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
        let wrong_fold = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues fold 0 { value } value\n";
        assert_eq!(
            analyze_for_compiler(wrong_fold).unwrap_err().code,
            "E-ANONYMOUS-FUNCTION-ARITY"
        );
    }

    #[test]
    fn models_int_pair_list_product_map_pattern() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COLLECTION-MAP-001,
        // TOPAL-FUNCTION-ANONYMOUS-001,
        // TOPAL-COMPILER-LIST-INT-PAIR-MAP-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-product-pattern.t"
        ))
        .unwrap();
        let [CompilerStatement::Binding(pairs)] = program.main.statements.as_slice() else {
            panic!("shared product-pattern regression binds its pair List")
        };
        assert_eq!(
            pairs.value.value_type,
            CompilerType::List(Box::new(CompilerType::Tuple(vec![
                CompilerType::Int,
                CompilerType::Int,
            ])))
        );
        let CompilerExpressionKind::ListMap {
            parameters, body, ..
        } = &program.main.result.kind
        else {
            panic!("shared product-pattern regression maps the pair List")
        };
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name, "left");
        assert_eq!(parameters[1].name, "right");
        assert!(
            parameters
                .iter()
                .all(|parameter| parameter.value_type == CompilerType::Int)
        );
        assert!(matches!(
            body.result.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                ..
            }
        ));
        assert_eq!(
            program.main.result.value_type,
            CompilerType::List(Box::new(CompilerType::Int))
        );

        let wrong_arity = "use language (version is v0.1)\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map { (a, b, c) } a + b\n";
        assert_eq!(
            analyze_for_compiler(wrong_arity).unwrap_err().code,
            "E-ANONYMOUS-FUNCTION-ARITY"
        );
        let duplicate = "use language (version is v0.1)\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map { (same, same) } same\n";
        assert_eq!(
            analyze_for_compiler(duplicate).unwrap_err().code,
            "E-DUPLICATE-BINDING"
        );
        let bound = "use language (version is v0.1)\ncombine is { (left, right) } left + right\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map combine\n";
        assert!(analyze_for_compiler(bound).is_ok());
    }

    #[test]
    fn models_exact_recursive_int_string_lists() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-TYPE-LIST-EQUALITY-001,
        // TOPAL-TYPE-LIST-RECURSIVE-001, TOPAL-LIST-FIRST-001,
        // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-COMPILER-LIST-RECURSIVE-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/nested-lists.t"))
                .unwrap();
        let pair = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]);
        let inner = CompilerType::List(Box::new(pair));
        let nested = CompilerType::List(Box::new(inner.clone()));
        assert_eq!(program.functions[0].parameters[0].value_type, nested);
        assert_eq!(program.functions[0].result_type, nested);
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared recursive List regression returns a Tuple")
        };
        assert!(matches!(
            results[0].kind,
            CompilerExpressionKind::ListFirst(_)
        ));
        assert_eq!(
            results[0].value_type,
            CompilerType::Optional(Box::new(inner))
        );
        assert!(matches!(
            results[1].kind,
            CompilerExpressionKind::ListEntryCount(_)
        ));
        assert!(matches!(
            results[2].kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Equal,
                ..
            }
        ));

        let inner_boundary = "use language (version is v0.1)\npreserve is fn (values : List (Int, String)) -> List (Int, String)\n  values\nvalues : List (Int, String) is Entry ((1, \"one\"), Empty)\npreserve values\n";
        assert_eq!(
            analyze_for_compiler(inner_boundary).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
        let unsupported_rest = "use language (version is v0.1)\nvalues : List (Int, String) is Entry ((1, \"one\"), Empty)\nnested : List List (Int, String) is Entry (values, Empty)\nrest nested\n";
        assert_eq!(
            analyze_for_compiler(unsupported_rest).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
        let deeper = "use language (version is v0.1)\nvalues : List List List (Int, String) is Empty\nvalues\n";
        assert_eq!(
            analyze_for_compiler(deeper).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_bound_anonymous_int_list_functions() {
        // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
        // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
        // TOPAL-COMPILER-LIST-INT-BOUND-FUNCTIONS-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/bound-anonymous-functions.t"
        ))
        .unwrap();
        assert_eq!(
            program
                .main
                .statements
                .iter()
                .filter(|statement| matches!(
                    statement,
                    CompilerStatement::Binding(binding)
                        if binding.value.value_type == CompilerType::Function
                ))
                .count(),
            3
        );
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared bound anonymous List regression returns a Tuple")
        };
        assert!(matches!(
            results.as_slice(),
            [
                CompilerExpression {
                    kind: CompilerExpressionKind::ListMap { .. },
                    ..
                },
                CompilerExpression {
                    kind: CompilerExpressionKind::ListSelect { .. },
                    ..
                },
                CompilerExpression {
                    kind: CompilerExpressionKind::ListFold { .. },
                    ..
                }
            ]
        ));
    }

    #[test]
    fn models_short_circuiting_int_list_fold_control() {
        // TOPAL-EXEC-TRAVERSAL-CONTROL-001,
        // TOPAL-COMPILER-TRAVERSAL-CONTROL-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/traversal-control.t"
        ))
        .unwrap();
        let [_, CompilerStatement::Binding(controls), _] = program.main.statements.as_slice()
        else {
            panic!("shared traversal regression binds values, controls, and its action")
        };
        let CompilerExpressionKind::Tuple(control_values) = &controls.value.kind else {
            panic!("controls binding retains both constructor values")
        };
        assert!(matches!(
            control_values.as_slice(),
            [
                CompilerExpression {
                    kind: CompilerExpressionKind::TraversalControl { finish: false, .. },
                    value_type: CompilerType::TraversalControl(_),
                    ..
                },
                CompilerExpression {
                    kind: CompilerExpressionKind::TraversalControl { finish: true, .. },
                    value_type: CompilerType::TraversalControl(_),
                    ..
                }
            ]
        ));
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared traversal regression returns a Tuple")
        };
        let CompilerExpressionKind::ListFold { body, .. } = &results[0].kind else {
            panic!("first result is the controlled fold")
        };
        assert_eq!(
            body.result.value_type,
            CompilerType::TraversalControl(Box::new(CompilerType::Int))
        );
        assert_eq!(results[0].value_type, CompilerType::Int);

        let invalid_constructor = "use language (version is v0.1)\nContinue true\n";
        assert_eq!(
            analyze_for_compiler(invalid_constructor).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
        let invalid_fold = "use language (version is v0.1)\nvalues : List Int is one 1\nvalues fold 0 { state, value } true\n";
        assert_eq!(
            analyze_for_compiler(invalid_fold).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn models_range_selection_without_exposing_slice_storage() {
        // TOPAL-RANGE-VALUE-SELECTION-001, TOPAL-RANGE-INDEX-SELECTION-001,
        // TOPAL-COMPILER-RANGE-SELECTION-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/range-selection.t"))
                .unwrap();
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("shared range-selection regression returns a Tuple")
        };
        assert!(matches!(
            results.as_slice(),
            [
                CompilerExpression {
                    kind: CompilerExpressionKind::ListRangeSelect { indexes: false, .. },
                    ..
                },
                CompilerExpression {
                    kind: CompilerExpressionKind::ListRangeSelect { indexes: true, .. },
                    ..
                },
                CompilerExpression {
                    kind: CompilerExpressionKind::String(value),
                    ..
                }
            ] if value == "opa"
        ));

        let unicode = analyze_for_compiler(
            "use language (version is v0.1)\ntext : String is \"a\u{301}👩‍🔬🇸🇪x\"\ntext select-index (1 ..= 2)\n",
        )
        .unwrap();
        assert!(matches!(
            unicode.main.result.kind,
            CompilerExpressionKind::String(ref value) if value == "👩‍🔬🇸🇪"
        ));

        let wrong_selector = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\nvalues select 1\n",
        )
        .unwrap_err();
        assert_eq!(wrong_selector.code, "E-TYPE-MISMATCH");

        let unsupported_element = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List Effect is Entry (Effects (), Empty)\nvalues select-index (0 .. 1)\n",
        )
        .unwrap_err();
        assert_eq!(unsupported_element.code, "E-COMPILER-UNSUPPORTED");
    }

    #[test]
    fn models_private_tuple_function_results_recursively() {
        // TOPAL-EXEC-COMPLETION-EFFECT-VALUE-001,
        // TOPAL-COMPILER-TUPLE-RESULT-001
        let completion = analyze_for_compiler(include_str!(
            "../../../examples/language/completion-effect-value.t"
        ))
        .unwrap();
        assert_eq!(completion.functions.len(), 1);
        assert_eq!(
            completion.functions[0].result_type,
            CompilerType::Tuple(vec![CompilerType::Completed, CompilerType::Effect])
        );
        let [CompilerStatement::Binding(binding)] = completion.main.statements.as_slice() else {
            panic!("expected one result binding")
        };
        assert_eq!(binding.name, "result");
        assert!(matches!(
            binding.value.kind,
            CompilerExpressionKind::Call { .. }
        ));
        assert!(matches!(
            completion.main.result.kind,
            CompilerExpressionKind::Local(ref storage) if storage == &binding.storage_name
        ));

        let nested = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn static () -> ((Int, Boolean), String)\n  ((42, true), \"Topal\")\nmake ()\n",
        )
        .unwrap();
        assert_eq!(
            nested.functions[0].result_type,
            CompilerType::Tuple(vec![
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Boolean]),
                CompilerType::String,
            ])
        );
    }

    #[test]
    fn models_tuple_results_for_every_admitted_decision_family() {
        // TOPAL-COMPILER-TUPLE-DECISION-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/tuple-decision-results.t"
        ))
        .unwrap();
        for function in program
            .functions
            .iter()
            .filter(|function| function.source_name.starts_with("choose-"))
        {
            assert!(matches!(function.result_type, CompilerType::Tuple(_)));
        }
        for (name, expected) in [
            ("choose-boolean", "boolean"),
            ("choose-ordered", "ordered"),
            ("choose-comparison", "comparison"),
            ("choose-enum", "enum"),
            ("choose-optional", "optional"),
            ("choose-result", "result"),
        ] {
            let function = program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap();
            let actual = match function.body.result.kind {
                CompilerExpressionKind::BooleanDecision { .. } => "boolean",
                CompilerExpressionKind::OrderedComparisonDecision { .. } => "ordered",
                CompilerExpressionKind::ComparisonValueDecision { .. } => "comparison",
                CompilerExpressionKind::EnumDecision { .. } => "enum",
                CompilerExpressionKind::OptionalDecision { .. } => "optional",
                CompilerExpressionKind::ResultDecision { .. } => "result",
                _ => panic!("expected a checked decision result for {name}"),
            };
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn models_private_tuple_parameters_and_candidate_specific_product_calls() {
        // TOPAL-COMPILER-TUPLE-PARAMETER-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/tuple-function-parameters.t"
        ))
        .unwrap();

        let retain = program
            .functions
            .iter()
            .find(|function| function.source_name == "retain")
            .unwrap();
        assert_eq!(retain.parameters.len(), 1);
        assert_eq!(
            retain.parameters[0].value_type,
            CompilerType::Tuple(vec![
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Boolean]),
                CompilerType::String,
            ])
        );

        for function in program
            .functions
            .iter()
            .filter(|function| function.source_name == "choose")
        {
            assert!(matches!(
                function.parameters.as_slice(),
                [
                    CompilerParameter {
                        value_type: CompilerType::Tuple(_),
                        ..
                    },
                    CompilerParameter {
                        value_type: CompilerType::Boolean,
                        ..
                    }
                ]
            ));
        }

        let tuple_first = program
            .functions
            .iter()
            .find(|function| function.source_name == "tuple-first")
            .unwrap();
        assert!(matches!(
            tuple_first.parameters.as_slice(),
            [CompilerParameter {
                value_type: CompilerType::Tuple(_),
                ..
            }]
        ));
        let fields_first = program
            .functions
            .iter()
            .find(|function| function.source_name == "fields-first")
            .unwrap();
        assert!(matches!(
            fields_first.parameters.as_slice(),
            [
                CompilerParameter {
                    value_type: CompilerType::Int,
                    ..
                },
                CompilerParameter {
                    value_type: CompilerType::String,
                    ..
                }
            ]
        ));

        let discard = program
            .functions
            .iter()
            .find(|function| function.source_name == "discard-pair")
            .unwrap();
        assert!(discard.parameters[0].discarded);
        assert!(matches!(
            discard.parameters[0].value_type,
            CompilerType::Tuple(_)
        ));
    }

    #[test]
    fn models_order_preserving_private_record_boundaries() {
        // TOPAL-COMPILER-RECORD-BOUNDARY-001, TOPAL-TYPE-PRODUCT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/record-function-boundaries.t"
        ))
        .unwrap();
        let person_type = CompilerType::Record(vec![
            ("active".to_owned(), CompilerType::Boolean),
            ("name".to_owned(), CompilerType::String),
        ]);

        let function = |name: &str| {
            program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
        };
        let make = function("make-person");
        assert_eq!(make.result_type, person_type);
        let retain = function("retain-person");
        assert_eq!(retain.parameters[0].value_type, person_type);
        assert_eq!(retain.result_type, person_type);
        for (name, expected) in [
            ("choose-person", "boolean"),
            ("choose-ordered", "ordered"),
            ("choose-comparison", "comparison"),
            ("choose-enum", "enum"),
            ("choose-optional", "optional"),
            ("choose-result", "result"),
        ] {
            let actual = match function(name).body.result.kind {
                CompilerExpressionKind::BooleanDecision { .. } => "boolean",
                CompilerExpressionKind::OrderedComparisonDecision { .. } => "ordered",
                CompilerExpressionKind::ComparisonValueDecision { .. } => "comparison",
                CompilerExpressionKind::EnumDecision { .. } => "enum",
                CompilerExpressionKind::OptionalDecision { .. } => "optional",
                CompilerExpressionKind::ResultDecision { .. } => "result",
                _ => panic!("expected a checked Record decision result for {name}"),
            };
            assert_eq!(actual, expected);
        }

        let wrapper = function("retain-wrapper");
        assert_eq!(
            wrapper.result_type,
            CompilerType::Record(vec![
                ("person".to_owned(), person_type),
                ("score".to_owned(), CompilerType::Int),
            ])
        );
        assert_eq!(wrapper.parameters[0].value_type, wrapper.result_type);
        assert!(matches!(
            program.main.result.kind,
            CompilerExpressionKind::Tuple(_)
        ));
    }

    #[test]
    fn models_closed_fundamental_type_values_and_identity() {
        // TOPAL-ABSTRACTION-TYPE-VALUE-001,
        // TOPAL-ABSTRACTION-TYPE-IDENTITY-001,
        // TOPAL-ABSTRACTION-TYPE-BOUNDARY-001
        let values =
            analyze_for_compiler(include_str!("../../../examples/language/type-values.t")).unwrap();
        assert_eq!(
            values.main.result.value_type,
            CompilerType::Tuple(vec![CompilerType::Type; 7])
        );
        let CompilerExpressionKind::Tuple(fields) = &values.main.result.kind else {
            panic!("expected fundamental Type-value product")
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| match field.kind {
                    CompilerExpressionKind::TypeValue(value) => value,
                    _ => panic!("expected a fundamental Type value"),
                })
                .collect::<Vec<_>>(),
            [0, 1, 2, 3, 4, 5, 6]
        );

        let boundary = analyze_for_compiler(include_str!(
            "../../../examples/language/type-function-boundary.t"
        ))
        .unwrap();
        let function = &boundary.functions[0];
        assert_eq!(function.parameters[0].value_type, CompilerType::Type);
        assert_eq!(function.result_type, CompilerType::Type);
        assert!(matches!(
            boundary.main.result.kind,
            CompilerExpressionKind::Call { .. }
        ));
        assert!(
            analyze_for_compiler(include_str!("../../../examples/language/type-identity.t"))
                .is_ok()
        );
        assert!(
            analyze_for_compiler(include_str!("../../../examples/language/type-classifier.t"))
                .is_ok()
        );
        assert_ne!(CompilerType::Type, CompilerType::Int);
    }

    #[test]
    fn models_named_constraint_identity_with_a_checked_predicate() {
        // TOPAL-ABSTRACTION-CONSTRAINT-CLASSIFIER-001,
        // TOPAL-TYPE-CONSTRAINT-001, TOPAL-COMPILER-CONSTRAINT-VALUE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/constraint-classifier.t"
        ))
        .unwrap();
        assert_eq!(program.constraints.len(), 2);
        let constraint = &program.constraints[0];
        assert_eq!(constraint.name, "Positive");
        assert_eq!(constraint.base_type, CompilerType::Int);
        assert_eq!(constraint.parameter, "value");
        assert_eq!(constraint.predicate.value_type, CompilerType::Boolean);
        assert!(matches!(
            constraint.predicate.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Greater,
                ..
            }
        ));
        assert_eq!(program.constraints[1].name, "rule");
        assert_eq!(program.constraints[1].base_type, CompilerType::Int);
        assert_eq!(program.constraints[1].predicate, constraint.predicate);
        assert!(matches!(
            program.main.statements[1],
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::ConstraintValue(1),
                    value_type: CompilerType::Constraint,
                    ..
                },
                ..
            })
        ));
        assert!(matches!(
            program.main.statements[0],
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::ConstraintValue(0),
                    value_type: CompilerType::Constraint,
                    ..
                },
                ..
            })
        ));
        assert_eq!(program.main.result.value_type, CompilerType::Constraint);
        assert!(matches!(
            program.main.result.kind,
            CompilerExpressionKind::Local(_)
        ));

        let non_boolean = "use language (version is v0.1)\nBroken is Int constraint { value } value + 1\nBroken\n";
        assert_eq!(
            analyze_for_compiler(non_boolean).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
        let captured = "use language (version is v0.1)\nlimit is 0\nPositive is Int constraint { value } value > limit\nPositive\n";
        let error = analyze_for_compiler(captured).unwrap_err();
        assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
        assert!(
            error
                .message
                .contains("captured constraint predicate value")
        );
    }

    #[test]
    fn models_named_constraint_validation_and_refined_base_operations() {
        // TOPAL-TYPE-CONSTRAINT-001, TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
        // TOPAL-COMPILER-CONSTRAINT-VALIDATE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/constraints-and-derived-capabilities.t"
        ))
        .unwrap();
        let refined = CompilerType::Refined {
            constraint: "Positive".into(),
            base: Box::new(CompilerType::Int),
        };
        for statement in &program.main.statements[1..=2] {
            let CompilerStatement::Binding(binding) = statement else {
                panic!("expected a refined binding")
            };
            assert_eq!(binding.value.value_type, refined);
            assert!(matches!(binding.value.kind, CompilerExpressionKind::Int(_)));
        }
        let validate = program
            .functions
            .iter()
            .find(|function| function.source_name == "validate")
            .unwrap();
        assert_eq!(
            validate.result_type,
            CompilerType::Result(Box::new(CompilerType::Int))
        );
        assert!(matches!(
            validate.body.result.kind,
            CompilerExpressionKind::Validate {
                operation: CompilerValidation::Constraint(0),
                ..
            }
        ));
        let CompilerExpressionKind::Tuple(observations) = &program.main.result.kind else {
            panic!("expected constraint observations")
        };
        assert_eq!(observations[0].value_type, refined);
        assert_eq!(observations[1].value_type, CompilerType::Boolean);
        assert_eq!(observations[2].value_type, CompilerType::Boolean);
        assert_eq!(observations[3].value_type, CompilerType::Int);
        assert_eq!(
            observations[4].value_type,
            CompilerType::Result(Box::new(CompilerType::Int))
        );

        let rejected = "use language (version is v0.1)\nPositive is Int constraint { value } value > 0\nPositive 0\n";
        assert_eq!(
            analyze_for_compiler(rejected).unwrap_err().code,
            "E-CONSTRAINT-REJECTED"
        );
        let wrong_base = "use language (version is v0.1)\nPositive is Int constraint { value } value > 0\nPositive \"one\"\n";
        assert_eq!(
            analyze_for_compiler(wrong_base).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn models_the_executable_root_namespace_and_qualified_function() {
        // TOPAL-COMPILER-ROOT-NAMESPACE-001, TOPAL-NAMESPACE-ROOT-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/root-namespace.t"))
                .unwrap();
        assert_eq!(program.functions.len(), 1);
        let CompilerExpressionKind::Tuple(fields) = &program.main.result.kind else {
            panic!("expected the root namespace observation product")
        };
        assert!(matches!(fields[0].kind, CompilerExpressionKind::Root));
        assert_eq!(fields[0].value_type, CompilerType::Scope);
        assert!(matches!(
            fields[1].kind,
            CompilerExpressionKind::Call { .. }
        ));
        assert_eq!(exact_int(&fields[1]), Some(BigInt::from(42)));

        let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\n{\n  increment is 100\n  root increment 41\n}\n",
        )
        .unwrap();
        assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));

        for (source, expected) in [
            (
                "use language (version is v0.1)\nroot is 1\n",
                "E-DUPLICATE-BINDING",
            ),
            (
                "use language (version is v0.1)\nroot is fn () -> Int\n  1\n",
                "E-DUPLICATE-BINDING",
            ),
            (
                "use language (version is v0.1)\nKind is Enum (root)\nKind\n",
                "E-COMPILER-UNSUPPORTED",
            ),
        ] {
            assert_eq!(analyze_for_compiler(source).unwrap_err().code, expected);
        }
    }

    #[test]
    fn models_use_of_root_and_root_alias_namespaces() {
        // TOPAL-COMPILER-NAMESPACE-USE-001, TOPAL-NAMESPACE-USE-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/use-namespace.t"))
                .unwrap();
        assert!(matches!(
            program.main.statements.as_slice(),
            [CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Root,
                    value_type: CompilerType::Scope,
                    ..
                },
                ..
            })]
        ));
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

        let alias = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\napi is root\ncurrent is use api\ncurrent increment 41\n",
        )
        .unwrap();
        assert_eq!(exact_int(&alias.main.result), Some(BigInt::from(42)));

        let data = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\ncurrent is use root\ncurrent answer\n",
        )
        .unwrap();
        assert_eq!(exact_int(&data.main.result), Some(BigInt::from(42)));

        let stale = analyze_for_compiler(
            "use language (version is v0.1)\ncurrent is use root\nlater is 42\ncurrent later\n",
        )
        .unwrap_err();
        assert_eq!(stale.code, "E-COMPILER-UNSUPPORTED");

        let non_namespace =
            analyze_for_compiler("use language (version is v0.1)\nuse 42\n").unwrap_err();
        assert_eq!(non_namespace.code, "E-USE-NON-NAMESPACE");

        let function_body = analyze_for_compiler(
            "use language (version is v0.1)\nprobe is fn () -> Scope\n  use root\nprobe ()\n",
        )
        .unwrap_err();
        assert_eq!(function_body.code, "E-COMPILER-UNSUPPORTED");
    }

    #[test]
    fn models_function_namespace_aliases_chains_and_snapshots() {
        // TOPAL-COMPILER-NAMESPACE-FUNCTION-ALIAS-001,
        // TOPAL-NAMESPACE-ALIAS-001, TOPAL-NAMESPACE-SNAPSHOT-001,
        // TOPAL-NAMESPACE-OVERLOAD-001, TOPAL-NAMESPACE-CLASSIFIER-001,
        // TOPAL-NAMESPACE-ALIAS-CHAIN-001
        let alias =
            analyze_for_compiler(include_str!("../../../examples/language/namespace-alias.t"))
                .unwrap();
        assert!(matches!(
            alias.main.statements.as_slice(),
            [CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Root,
                    value_type: CompilerType::Scope,
                    ..
                },
                ..
            })]
        ));
        assert!(matches!(
            alias.main.result.kind,
            CompilerExpressionKind::Call { .. }
        ));

        let overloads = analyze_for_compiler(include_str!(
            "../../../examples/language/namespace-overloads.t"
        ))
        .unwrap();
        let CompilerExpressionKind::Tuple(values) = &overloads.main.result.kind else {
            panic!("expected qualified overload result product")
        };
        assert_eq!(values.len(), 2);
        assert!(
            values
                .iter()
                .all(|value| matches!(value.kind, CompilerExpressionKind::Call { .. }))
        );

        let chained = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nfirst is root\nsecond : Scope is first\nsecond increment 41\n",
        )
        .unwrap();
        assert_eq!(exact_int(&chained.main.result), Some(BigInt::from(42)));

        let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\napi is root\n{\n  increment is 100\n  api increment 41\n}\n",
        )
        .unwrap();
        assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));

        let snapshot = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\napi is root\nidentity is fn (value : String) -> String\n  value\n(api identity 42, root identity \"Topal\")\n",
        )
        .unwrap();
        let CompilerExpressionKind::Tuple(values) = &snapshot.main.result.kind else {
            panic!("expected snapshot and live-root result product")
        };
        assert_eq!(values[0].value_type, CompilerType::Int);
        assert_eq!(values[1].value_type, CompilerType::String);

        let rejected = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\napi is root\nidentity is fn (value : String) -> String\n  value\napi identity \"Topal\"\n",
        )
        .unwrap_err();
        assert_eq!(rejected.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn models_root_and_alias_data_member_snapshots() {
        // TOPAL-COMPILER-NAMESPACE-DATA-001, TOPAL-NAMESPACE-ROOT-001,
        // TOPAL-NAMESPACE-ALIAS-001, TOPAL-NAMESPACE-SNAPSHOT-001,
        // TOPAL-NAMESPACE-CLASSIFIER-001, TOPAL-NAMESPACE-ALIAS-CHAIN-001
        for source in [
            include_str!("../../../examples/language/namespace-alias-chain.t"),
            include_str!("../../../examples/language/namespace-snapshot.t"),
            include_str!("../../../examples/language/scope-classifier.t"),
            include_str!("../../../examples/language/published-root-member.t"),
        ] {
            analyze_for_compiler(source).unwrap();
        }

        let snapshot = analyze_for_compiler(include_str!(
            "../../../examples/language/namespace-snapshot.t"
        ))
        .unwrap();
        let CompilerExpressionKind::Tuple(values) = &snapshot.main.result.kind else {
            panic!("expected earlier-alias and live-root data product")
        };
        assert_eq!(exact_int(&values[0]), Some(BigInt::from(41)));
        assert_eq!(exact_int(&values[1]), Some(BigInt::from(42)));

        let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\napi is root\n{\n  answer is 0\n  (root answer, api answer, answer)\n}\n",
        )
        .unwrap();
        let CompilerExpressionKind::Block(block) = &shadowed.main.result.kind else {
            panic!("expected lexical shadow block")
        };
        let CompilerExpressionKind::Tuple(values) = &block.result.kind else {
            panic!("expected qualified and lexical result product")
        };
        assert_eq!(exact_int(&values[0]), Some(BigInt::from(42)));
        assert_eq!(exact_int(&values[1]), Some(BigInt::from(42)));
        assert_eq!(exact_int(&values[2]), Some(BigInt::from(0)));
        let CompilerExpressionKind::Local(root_storage) = &values[0].kind else {
            panic!("expected stable root storage reference")
        };
        let CompilerExpressionKind::Local(local_storage) = &values[2].kind else {
            panic!("expected lexical storage reference")
        };
        assert_ne!(root_storage, local_storage);

        let rejected = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\nread is fn () -> Int\n  root answer\nread ()\n",
        )
        .unwrap_err();
        assert_eq!(rejected.code, "E-COMPILER-UNSUPPORTED");
    }

    #[test]
    fn models_scope_parameters_as_specialized_private_environments() {
        // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
        // TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001
        let program = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\nincrement is fn (value : Int) -> Int\n  value + 1\nread-answer is fn (api : Scope) -> Int\n  api answer\napply is fn (api : Scope, value : Int) -> Int\n  api increment value\nforward is fn (api : Scope) -> Int\n  read-answer api\n(read-answer root, apply root 41, forward root)\n",
        )
        .unwrap();
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("expected Scope-boundary result product")
        };
        assert!(
            values
                .iter()
                .all(|value| exact_int(value) == Some(BigInt::from(42)))
        );
        let forward = program
            .functions
            .iter()
            .find(|function| function.source_name == "forward")
            .expect("forwarding Scope specialization exists");
        assert_eq!(forward.parameters.len(), 2);
        assert_eq!(forward.parameters[0].value_type, CompilerType::Scope);
        assert_eq!(forward.parameters[1].name, "api answer");
        assert_eq!(forward.parameters[1].value_type, CompilerType::Int);
        let CompilerExpressionKind::Call { arguments, .. } = &forward.body.result.kind else {
            panic!("forwarding body retains a direct private call")
        };
        assert_eq!(arguments.len(), 2);
        assert!(matches!(
            &arguments[1].kind,
            CompilerExpressionKind::Local(name) if name == "api answer"
        ));

        let stale = analyze_for_compiler(
            "use language (version is v0.1)\napi is root\nanswer is 42\nread-answer is fn (scope : Scope) -> Int\n  scope answer\nread-answer api\n",
        )
        .unwrap_err();
        assert_eq!(stale.code, "E-COMPILER-UNSUPPORTED");

        let unsupported_member = analyze_for_compiler(
            "use language (version is v0.1)\nnested is root\nread is fn (api : Scope) -> Int\n  api nested\nread root\n",
        )
        .unwrap_err();
        assert_eq!(unsupported_member.code, "E-COMPILER-UNSUPPORTED");

        let local_root = analyze_for_compiler(
            "use language (version is v0.1)\naccept is fn (_ : Scope) -> Int\n  1\nwrapper is fn () -> Int\n  accept root\nwrapper ()\n",
        )
        .unwrap_err();
        assert_eq!(local_root.code, "E-COMPILER-UNSUPPORTED");
    }

    #[test]
    fn models_private_scalar_defining_context_capture() {
        // TOPAL-COMPILER-CONTEXT-CAPTURE-001, TOPAL-CONTEXT-SELECT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/constructed-context.t"
        ))
        .unwrap();
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
        let function = &program.functions[0];
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.parameters[0].name, "value");
        assert_eq!(function.parameters[1].name, "@ offset");
        let CompilerExpressionKind::Call { arguments, .. } = &program.main.result.kind else {
            panic!("expected direct context-capturing call")
        };
        assert_eq!(arguments.len(), 2);
        assert_eq!(exact_int(&arguments[0]), Some(BigInt::from(2)));
        assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(40)));
        let [CompilerStatement::Binding(root_binding)] = program.main.statements.as_slice() else {
            panic!("expected one defining-context root binding")
        };
        assert!(matches!(
            &arguments[1].kind,
            CompilerExpressionKind::Local(storage) if storage == &root_binding.storage_name
        ));

        let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nadd-offset is fn (value : Int) -> Int\n  value + @ offset\n{\n  offset is 100\n  add-offset 2\n}\n",
        )
        .unwrap();
        assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));

        let later = analyze_for_compiler(
            "use language (version is v0.1)\nadd-offset is fn (value : Int) -> Int\n  value + @ offset\noffset is 40\nadd-offset 2\n",
        )
        .unwrap_err();
        assert_eq!(later.code, "E-COMPILER-UNSUPPORTED");

        let forwarded = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nadd-offset is fn (value : Int) -> Int\n  value + @ offset\nwrapper is fn (value : Int) -> Int\n  add-offset value\nwrapper 2\n",
        )
        .unwrap_err();
        assert_eq!(forwarded.code, "E-COMPILER-UNSUPPORTED");

        let outside =
            analyze_for_compiler("use language (version is v0.1)\noffset is 40\n@ offset\n")
                .unwrap_err();
        assert_eq!(outside.code, "E-CONTEXT-SELECTION");
    }

    #[test]
    fn models_explicit_empty_function_effect_bound_and_static_view() {
        // TOPAL-FUNCTION-EFFECT-BOUND-001, TOPAL-EFFECT-CONTAIN-001,
        // TOPAL-INTRO-STATIC-001, TOPAL-INTRO-VIEW-001,
        // TOPAL-COMPILER-FUNCTION-EMPTY-EFFECT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-effect-bound.t"
        ))
        .unwrap();
        let empty = CompilerEffectRow {
            identities: Vec::new(),
        };
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].declared_effects, Some(empty.clone()));
        let [CompilerStatement::Binding(signature)] = program.main.statements.as_slice() else {
            panic!("the static Function view is the only lowered root binding")
        };
        assert_eq!(signature.value.value_type, CompilerType::FunctionView);
        let CompilerExpressionKind::FunctionView(view) = &signature.value.kind else {
            panic!("the static Function view retains checked metadata")
        };
        assert_eq!(view.identity, "root.identity");
        assert_eq!(view.inputs, ["Int"]);
        assert_eq!(view.output, "Int");
        assert!(!view.is_static);
        assert_eq!(view.declared_effects, Some(empty));
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

        let nonempty = analyze_for_compiler(
            "use language (version is v0.1)\nread is fn (value : Int) -> Int : Read value\n  value\n1\n",
        )
        .unwrap_err();
        assert_eq!(nonempty.code, "E-COMPILER-UNSUPPORTED");

        let runtime_view = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int : Effects ()\n  value\nsignature is lang view identity\nsignature\n",
        )
        .unwrap_err();
        assert_eq!(runtime_view.code, "E-COMPILER-UNSUPPORTED");
    }

    #[test]
    fn models_closed_static_introspection_and_runtime_version() {
        // TOPAL-INTRO-QUALIFIED-001, TOPAL-INTRO-STATIC-001,
        // TOPAL-INTRO-VIEW-001, TOPAL-INTRO-CONTEXT-001,
        // TOPAL-INTRO-RELATION-001, TOPAL-COMPILER-STATIC-INTROSPECTION-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/static-introspection.t"
        ))
        .unwrap();
        let [
            CompilerStatement::Binding(identity_binding),
            CompilerStatement::Binding(view_binding),
            CompilerStatement::Binding(context_binding),
        ] = program.main.statements.as_slice()
        else {
            panic!("the three static values retain checked metadata")
        };
        assert_eq!(identity_binding.value.value_type, CompilerType::Identity);
        let CompilerExpressionKind::Identity(identity) = &identity_binding.value.kind else {
            panic!("lang identity retains a typed compiler value")
        };
        assert_eq!(identity.kind, ObjectKind::Type);
        assert_eq!(identity.canonical, "type:Int");

        assert_eq!(view_binding.value.value_type, CompilerType::TypeView);
        let CompilerExpressionKind::TypeView(view) = &view_binding.value.kind else {
            panic!("lang view retains a typed compiler value")
        };
        assert_eq!(view.form, CompilerTypeViewForm::Primitive);
        assert_eq!(view.identity, "Int");

        assert_eq!(
            context_binding.value.value_type,
            CompilerType::LanguageContext
        );
        let CompilerExpressionKind::LanguageContext(context) = &context_binding.value.kind else {
            panic!("lang context retains a typed compiler value")
        };
        assert_eq!(context.language, "topal");
        assert_eq!(context.version, LanguageVersion::DESIGN_0);
        assert!(context.features.is_empty());

        let CompilerExpressionKind::Tuple(result) = &program.main.result.kind else {
            panic!("the regression result is a typed Tuple")
        };
        assert!(matches!(
            result[0].kind,
            CompilerExpressionKind::Boolean(true)
        ));
        assert!(matches!(
            result[1].kind,
            CompilerExpressionKind::Boolean(false)
        ));
        assert!(matches!(
            result[2].kind,
            CompilerExpressionKind::Version(LanguageVersion::DESIGN_0)
        ));
        assert_eq!(
            program.main.result.value_type,
            CompilerType::Tuple(vec![
                CompilerType::Boolean,
                CompilerType::Boolean,
                CompilerType::Version,
            ])
        );

        for source in [
            "use language (version is v0.1)\nidentity is lang identity Int\nidentity\n",
            "use language (version is v0.1)\nview is lang view Int\nview\n",
            "use language (version is v0.1)\ncontext is lang context\ncontext\n",
            "use language (version is v0.1)\nlang identity 42\n",
            "use language (version is v0.1)\n1 lang same-object 1\n",
            "use language (version is v0.1)\nuse language (version is v0.1)\nlang version\n",
        ] {
            assert_eq!(
                analyze_for_compiler(source).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_closed_static_capability_composition() {
        // TOPAL-CAPABILITY-EVIDENCE-001, TOPAL-CAPABILITY-COHERENCE-001,
        // TOPAL-CAPABILITY-COMPOSE-001, TOPAL-COMPILER-CAPABILITY-COMPOSE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/capability-composition.t"
        ))
        .unwrap();
        let [
            CompilerStatement::Binding(comparable),
            CompilerStatement::Binding(searchable),
            CompilerStatement::Binding(alternatives),
        ] = program.main.statements.as_slice()
        else {
            panic!("the three root Capability bindings retain checked metadata")
        };
        for binding in [comparable, searchable, alternatives] {
            assert_eq!(binding.value.value_type, CompilerType::Capability);
        }
        let CompilerExpressionKind::Capability(comparable) = &comparable.value.kind else {
            panic!("Comparable is a checked Capability")
        };
        assert_eq!(
            comparable.alternatives,
            [vec!["Equality".to_owned(), "Ordering".to_owned()]]
        );
        let CompilerExpressionKind::Capability(searchable) = &searchable.value.kind else {
            panic!("Searchable is a checked Capability")
        };
        assert_eq!(
            searchable.alternatives,
            [vec!["Foldable".to_owned(), "Membership".to_owned()]]
        );
        let CompilerExpressionKind::Capability(result) = &program.main.result.kind else {
            panic!("the final result retains canonical Capability alternatives")
        };
        assert_eq!(
            result.alternatives,
            [
                vec!["Equality".to_owned(), "Ordering".to_owned()],
                vec!["Foldable".to_owned(), "Membership".to_owned()],
            ]
        );
        assert_eq!(
            result.display(),
            "Equality and Ordering or Foldable and Membership"
        );

        let canonical_source = "use language (version is v0.1)\nSame is Equality and Equality\nRepeated : Capability is Same or Equality\nReversed : Capability is Membership or Repeated\nReversed\n";
        let idempotent = analyze_for_compiler(canonical_source).unwrap();
        let CompilerExpressionKind::Capability(result) = &idempotent.main.result.kind else {
            panic!("the classified alias remains a Capability")
        };
        assert_eq!(
            result.alternatives,
            [vec!["Equality".to_owned()], vec!["Membership".to_owned()]]
        );
        let interpreted = crate::source::Session::new()
            .evaluate_source_file(canonical_source, &mut std::io::sink())
            .unwrap();
        assert_eq!(interpreted.to_string(), result.display());

        for source in [
            "use language (version is v0.1)\n(Equality, true)\n",
            "use language (version is v0.1)\nchoose is fn static () -> Capability\n  Equality\nchoose ()\n",
            "use language (version is v0.1)\nEquality xor Ordering\n",
        ] {
            assert_eq!(
                analyze_for_compiler(source).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED",
                "{source}"
            );
        }
        assert_eq!(
            analyze_for_compiler(
                "use language (version is v0.1)\nvalue : Boolean is Equality\nvalue\n"
            )
            .unwrap_err()
            .code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn retains_and_erases_exact_function_interface_evidence() {
        // TOPAL-INTERFACE-SHAPE-001, TOPAL-INTERFACE-IMPLEMENTATION-001,
        // TOPAL-COMPILER-FUNCTION-INTERFACE-001
        let source = include_str!("../../../examples/language/function-interface.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(program.interfaces.len(), 1);
        assert_eq!(program.interfaces[0].identity, "root.Parser");
        assert_eq!(
            program.interfaces[0].operations,
            [CompilerInterfaceOperation {
                name: "parse".into(),
                parameters: vec![CompilerType::String],
                result: CompilerType::Boolean,
            }]
        );
        assert_eq!(program.interface_implementations.len(), 1);
        assert_eq!(
            program.interface_implementations[0],
            CompilerInterfaceImplementation {
                interface_identity: "root.Parser".into(),
                operations: vec![CompilerInterfaceOperationEvidence {
                    role: "parse".into(),
                    declaration_identity: "root.parse:ordinary(String)".into(),
                    declared_effects: None,
                }],
            }
        );
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].source_name, "parse");
        assert_eq!(program.main.result.value_type, CompilerType::Boolean);
        let interpreted = crate::source::Session::new()
            .evaluate_source_file(source, &mut std::io::sink())
            .unwrap();
        assert_eq!(interpreted.to_string(), "true");

        let multi_source = "use language (version is v0.1)\nService is Interface\n  zed is fn (value : Int) -> Int\n  alpha is fn (value : String) -> Boolean\nService\n  zed is fn (value : Int) -> Int\n    value\n  alpha is fn (value : String) -> Boolean\n    : Effects ()\n    value = \"ok\"\nalpha \"ok\"\n";
        let multi = analyze_for_compiler(multi_source).unwrap();
        assert_eq!(
            multi.interfaces[0]
                .operations
                .iter()
                .map(|operation| operation.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zed"]
        );
        assert_eq!(
            multi.interface_implementations[0]
                .operations
                .iter()
                .map(|operation| operation.role.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zed"]
        );
        assert_eq!(
            multi.interface_implementations[0].operations[0].declared_effects,
            Some(CompilerEffectRow {
                identities: Vec::new(),
            })
        );
        assert_eq!(
            multi.interface_implementations[0].operations[1].declared_effects,
            None
        );

        for invalid in [
            "use language (version is v0.1)\nParser\n  parse is fn (source : String) -> Boolean\n    true\nparse \"ok\"\n",
            "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  other is fn (source : String) -> Boolean\n    true\nother \"ok\"\n",
            "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\n  other is fn (source : String) -> Boolean\nParser\n  parse is fn (source : String) -> Boolean\n    true\nparse \"ok\"\n",
            "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  parse is fn (source : String) -> Boolean\n    true\n  other is fn (source : String) -> Boolean\n    false\nparse \"ok\"\n",
            "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  parse is fn (source : Int) -> Boolean\n    true\nparse 1\n",
            "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  parse is fn (source : String) -> Boolean\n    true\n  parse is fn (source : String) -> Boolean\n    false\nparse \"ok\"\n",
        ] {
            assert!(
                matches!(
                    analyze_for_compiler(invalid).unwrap_err().code.as_str(),
                    "E-UNKNOWN-INTERFACE" | "E-INTERFACE-IMPLEMENTATION"
                ),
                "{invalid}"
            );
        }
        let nested = "use language (version is v0.1)\nconstruct is fn () -> Unit\n  Parser is Interface\n    parse is fn (source : String) -> Boolean\n  ()\nconstruct ()\n";
        assert_eq!(
            analyze_for_compiler(nested).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_named_function_values_without_restarting_lookup() {
        // TOPAL-COMPILER-NAMED-FUNCTION-VALUE-001, TOPAL-FUNCTION-VALUE-001,
        // TOPAL-FUNCTION-OVERLOAD-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/named-function-values.t"
        ))
        .unwrap();
        assert_eq!(
            program.function_value_names,
            ["<fn increment>", "+", "-", "<=>"]
        );
        assert!(matches!(
            program.main.statements.as_slice(),
            [CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::FunctionValue(0),
                    value_type: CompilerType::Function,
                    ..
                },
                ..
            })]
        ));
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

        let chained = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\noperation : Function is increment\nagain is operation\n{\n  increment is 100\n  again 41\n}\n",
        )
        .unwrap();
        assert_eq!(exact_int(&chained.main.result), Some(BigInt::from(42)));

        let snapshot = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\noperation is identity\nidentity is fn (value : String) -> String\n  value\n(operation 42, identity \"Topal\")\n",
        )
        .unwrap();
        let CompilerExpressionKind::Tuple(values) = &snapshot.main.result.kind else {
            panic!("expected captured and live function result product")
        };
        assert_eq!(values[0].value_type, CompilerType::Int);
        assert_eq!(values[1].value_type, CompilerType::String);

        let rejected = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\noperation is identity\nidentity is fn (value : String) -> String\n  value\noperation \"Topal\"\n",
        )
        .unwrap_err();
        assert_eq!(rejected.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn models_symbolic_callable_values_and_function_classification() {
        // TOPAL-COMPILER-SYMBOLIC-CALLABLE-VALUE-001,
        // TOPAL-FUNCTION-CALLABLE-VALUE-001
        let values =
            analyze_for_compiler(include_str!("../../../examples/language/callable-values.t"))
                .unwrap();
        assert_eq!(values.function_value_names, ["+", "-", "<=>"]);
        let CompilerExpressionKind::Tuple(results) = &values.main.result.kind else {
            panic!("expected callable result product")
        };
        assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
        assert_eq!(exact_int(&results[1]), Some(BigInt::from(-5)));
        assert_eq!(results[2].value_type, CompilerType::Comparison);

        let classified = analyze_for_compiler(include_str!(
            "../../../examples/language/function-classifier.t"
        ))
        .unwrap();
        assert_eq!(exact_int(&classified.main.result), Some(BigInt::from(42)));

        let chained = analyze_for_compiler(
            "use language (version is v0.1)\nadd : Function is +\noperation is add\n(operation (20, 22), operation (1, 2))\n",
        )
        .unwrap();
        let CompilerExpressionKind::Tuple(results) = &chained.main.result.kind else {
            panic!("expected chained symbolic results")
        };
        assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
        assert_eq!(exact_int(&results[1]), Some(BigInt::from(3)));

        let binary_minus = analyze_for_compiler(
            "use language (version is v0.1)\nsubtract is -\n(subtract (9, 4), subtract 5)\n",
        )
        .unwrap();
        let CompilerExpressionKind::Tuple(results) = &binary_minus.main.result.kind else {
            panic!("expected unary and binary minus results")
        };
        assert_eq!(exact_int(&results[0]), Some(BigInt::from(5)));
        assert_eq!(exact_int(&results[1]), Some(BigInt::from(-5)));

        let rejected =
            analyze_for_compiler("use language (version is v0.1)\nadd is +\nadd 1\n").unwrap_err();
        assert_eq!(rejected.code, "E-NO-APPLICABLE-OVERLOAD");
    }

    #[test]
    fn models_specialized_function_input_boundaries() {
        // TOPAL-COMPILER-FUNCTION-PARAMETER-001,
        // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-TYPE-CALL-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-value-boundary.t"
        ))
        .unwrap();
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
        assert_eq!(
            program.functions[0].parameters[0].value_type,
            CompilerType::Function
        );
        assert!(matches!(
            program.functions[0].body.result.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                ..
            }
        ));

        let aliased = analyze_for_compiler(
            "use language (version is v0.1)\napply-pair is fn (operation : Function) -> Int\n  operation (20, 22)\nadd is +\napply-pair add\n",
        )
        .unwrap();
        assert_eq!(exact_int(&aliased.main.result), Some(BigInt::from(42)));

        let named = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\napply is fn (operation : Function) -> Int\n  operation 41\napply increment\n",
        )
        .unwrap();
        assert_eq!(exact_int(&named.main.result), Some(BigInt::from(42)));
        assert!(
            named
                .functions
                .iter()
                .any(|function| function.source_name == "increment")
        );

        let two_specializations = analyze_for_compiler(
            "use language (version is v0.1)\napply is fn (operation : Function) -> Int\n  operation (9, 4)\nadd is +\nsubtract is -\n(apply add, apply subtract)\n",
        )
        .unwrap();
        let CompilerExpressionKind::Tuple(results) = &two_specializations.main.result.kind else {
            panic!("expected two Function specializations")
        };
        assert_eq!(exact_int(&results[0]), Some(BigInt::from(13)));
        assert_eq!(exact_int(&results[1]), Some(BigInt::from(5)));
        assert_eq!(
            two_specializations
                .functions
                .iter()
                .filter(|function| function.source_name == "apply")
                .count(),
            2
        );
    }

    #[test]
    fn models_direct_non_capturing_anonymous_functions() {
        // TOPAL-COMPILER-ANONYMOUS-DIRECT-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-SYN-GRAMMAR-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-function-application.t"
        ))
        .unwrap();
        assert_eq!(
            program.function_value_names,
            ["+", "-", "<=>", "<anonymous fn/1>", "<anonymous fn/2>"]
        );
        assert_eq!(program.functions.len(), 2);
        assert_eq!(program.functions[0].source_name, "<anonymous fn/1>");
        assert_eq!(program.functions[0].parameters.len(), 1);
        assert_eq!(program.functions[1].source_name, "<anonymous fn/2>");
        assert_eq!(program.functions[1].parameters.len(), 2);
        assert!(
            program
                .functions
                .iter()
                .flat_map(|function| &function.parameters)
                .all(|parameter| parameter.value_type == CompilerType::Int)
        );
        let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
            panic!("expected two direct anonymous calls")
        };
        assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
        assert_eq!(exact_int(&results[1]), Some(BigInt::from(42)));
        assert!(
            results
                .iter()
                .all(|result| matches!(result.kind, CompilerExpressionKind::Call { .. }))
        );

        let left_associative = analyze_for_compiler(
            "use language (version is v0.1)\ncalculate is { value } value + 3 * 4\ncalculate 2\n",
        )
        .unwrap();
        assert_eq!(
            exact_int(&left_associative.main.result),
            Some(BigInt::from(20))
        );

        let contextual = analyze_for_compiler(
            "use language (version is v0.1)\napply is fn (operation : Function) -> Int\n  operation 41\napply { value } value + 1\n",
        )
        .unwrap();
        assert_eq!(exact_int(&contextual.main.result), Some(BigInt::from(42)));
        assert!(
            contextual
                .functions
                .iter()
                .any(|function| function.source_name == "<anonymous fn/1>")
        );

        let capture = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 1\nincrement is { value } value + offset\nincrement 41\n",
        )
        .unwrap_err();
        assert_eq!(capture.code, "E-COMPILER-UNSUPPORTED");

        let arity = analyze_for_compiler(
            "use language (version is v0.1)\ncombine is { left, right } left + right\ncombine 42\n",
        )
        .unwrap_err();
        assert_eq!(arity.code, "E-ANONYMOUS-ARGUMENT-PACKAGE");
    }

    #[test]
    fn models_one_closed_scalar_packaged_function_operand() {
        // TOPAL-COMPILER-PACKAGED-OPERAND-001,
        // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/packaged-function-operand.t"
        ))
        .unwrap();
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
        let function = &program.functions[0];
        assert_eq!(function.source_name, "sum");
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.parameters[0].name, "value");
        assert_eq!(function.parameters[1].name, "fallback");
        let CompilerExpressionKind::Call { arguments, .. } = &program.main.result.kind else {
            panic!("expected a normalized packaged call")
        };
        assert_eq!(arguments.len(), 2);
        assert_eq!(exact_int(&arguments[0]), Some(BigInt::from(40)));
        assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(2)));

        for (call, expected) in [
            ("sum (value is 40, fallback is 5)", 45),
            ("sum (40, 2)", 42),
        ] {
            let source = format!(
                "use language (version is v0.1)\nsum is fn ((value : Int, fallback : Int default 2)) -> Int\n  value + fallback\n{call}\n"
            );
            let program = analyze_for_compiler(&source).unwrap();
            assert_eq!(
                exact_int(&program.main.result),
                Some(BigInt::from(expected))
            );
        }

        let missing = analyze_for_compiler(
            "use language (version is v0.1)\nsum is fn ((value : Int, fallback : Int default 2)) -> Int\n  value + fallback\nsum (fallback is 2)\n",
        )
        .unwrap_err();
        assert_eq!(missing.code, "E-NO-APPLICABLE-OVERLOAD");

        for rejected in [
            "use language (version is v0.1)\nsum is fn ((value : Int, fallback : Int default 2)) -> Int\n  value + fallback\nsum (fallback is 2, value is 40)\n",
            "use language (version is v0.1)\nsum is fn ((value : Int, fallback : Int default value)) -> Int\n  value + fallback\nsum (value is 40)\n",
        ] {
            assert_eq!(
                analyze_for_compiler(rejected).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_discarded_function_parameters_without_binding_them() {
        // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-PATTERN-001
        let source = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (0, 42)\n";
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "second")
            .unwrap();
        assert!(function.parameters[0].discarded);
        assert!(!function.parameters[1].discarded);
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Local(ref name) if name == "value"
        ));

        let mismatch = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (\"ignored\", 42)\n";
        assert_eq!(
            analyze_for_compiler(mismatch).unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn models_ordered_overloads_and_static_call_boundaries() {
        // TOPAL-FUNCTION-OVERLOAD-001, TOPAL-FUNCTION-STATIC-NULLARY-001,
        // TOPAL-FUNCTION-STATIC-UNARY-001, TOPAL-FUNCTION-STATIC-BINARY-001
        let source = "use language (version is v0.1)\ndescribe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  describe 42\nanswer is fn static () -> Int\n  42\nadd is fn static (left : Int, right : Int) -> Int\n  left + right\n(describe \"Topal\", answer (), 20 add 22)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program
                .functions
                .iter()
                .filter(|function| function.source_name == "describe")
                .count(),
            2
        );
        assert_eq!(
            program
                .functions
                .iter()
                .filter(|function| function.is_static)
                .count(),
            2
        );

        let duplicate = "use language (version is v0.1)\nsame is fn (first : Int) -> Int\n  first\nsame is fn (second : Int) -> String\n  \"duplicate\"\nsame 1\n";
        assert_eq!(
            analyze_for_compiler(duplicate).unwrap_err().code,
            "E-DUPLICATE-FUNCTION-OVERLOAD"
        );

        let invalid_static_call = "use language (version is v0.1)\nruntime is fn () -> Int\n  42\nanswer is fn static () -> Int\n  runtime ()\nanswer ()\n";
        assert_eq!(
            analyze_for_compiler(invalid_static_call).unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn models_same_named_cross_overload_call_as_acyclic() {
        // TOPAL-FUNCTION-RECURSION-OVERLOAD-IDENTITY-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/overload-recursion-identity.t"
        ))
        .unwrap();
        let integer = program
            .functions
            .iter()
            .find(|function| function.parameters[0].value_type == CompilerType::Int)
            .unwrap();
        let string = program
            .functions
            .iter()
            .find(|function| function.parameters[0].value_type == CompilerType::String)
            .unwrap();
        assert_ne!(integer.symbol, string.symbol);
        let CompilerExpressionKind::StringConcat { left, .. } = &string.body.result.kind else {
            panic!("expected outer String concatenation")
        };
        let CompilerExpressionKind::StringConcat { left, .. } = &left.kind else {
            panic!("expected inner String concatenation")
        };
        let CompilerExpressionKind::Call { symbol, arguments } = &left.kind else {
            panic!("expected the cross-overload call")
        };
        assert_eq!(symbol, &integer.symbol);
        assert_ne!(symbol, &string.symbol);
        assert_eq!(arguments[0].value_type, CompilerType::Int);
    }

    #[test]
    fn models_only_complete_uniform_mutual_int_cycles() {
        // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001,
        // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001
        let source = include_str!("../../../examples/language/mutual-int-recursion.t")
            .replace("(even 6, odd 6)", "even 6");
        let program = analyze_for_compiler(&source).unwrap();
        assert_eq!(program.functions.len(), 2);
        let even = program
            .functions
            .iter()
            .find(|function| function.source_name == "even")
            .unwrap();
        let odd = program
            .functions
            .iter()
            .find(|function| function.source_name == "odd")
            .unwrap();
        for (function, target) in [(even, odd), (odd, even)] {
            assert!(function.parameters[0].int_range.is_none());
            let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
                &function.body.result.kind
            else {
                panic!("expected a proven mutual recursion decision")
            };
            let CompilerExpressionKind::Call { symbol, .. } = &otherwise.kind else {
                panic!("expected the next mutual edge")
            };
            assert_eq!(symbol, &target.symbol);
        }
        assert!(
            analyze_for_compiler(include_str!(
                "../../../examples/language/mutual-increasing-int-recursion.t"
            ))
            .is_ok()
        );
        assert!(
            analyze_for_compiler(include_str!(
                "../../../examples/language/mutual-multiple-recursive-calls.t"
            ))
            .is_ok()
        );

        for invalid in [
            "use language (version is v0.1)\nfirst is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    >= 0 then false\n    otherwise first (value + 1)\nfirst 2\n",
            "use language (version is v0.1)\nfirst is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 0)\nfirst 2\n",
        ] {
            assert_eq!(
                analyze_for_compiler(invalid).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_only_range_preserving_mutual_nat_cycles() {
        // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001,
        // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001
        let source = include_str!("../../../examples/language/nat-mutual-recursion.t")
            .replace("(even 8, odd 8)", "even 8");
        let program = analyze_for_compiler(&source).unwrap();
        assert_eq!(program.functions.len(), 2);
        let even = program
            .functions
            .iter()
            .find(|function| function.source_name == "even")
            .unwrap();
        let odd = program
            .functions
            .iter()
            .find(|function| function.source_name == "odd")
            .unwrap();
        for (function, target) in [(even, odd), (odd, even)] {
            assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
            assert!(function.parameters[0].int_range.is_none());
            let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
                &function.body.result.kind
            else {
                panic!("expected a proven mutual Nat recursion decision")
            };
            let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
                panic!("expected the next mutual Nat edge")
            };
            assert_eq!(symbol, &target.symbol);
            let [argument] = arguments.as_slice() else {
                panic!("expected one recursive Nat argument")
            };
            assert!(matches!(argument.kind, CompilerExpressionKind::IntToNat(_)));
        }

        assert!(
            analyze_for_compiler(include_str!(
                "../../../examples/language/nat-mutual-increasing-recursion.t"
            ))
            .is_ok()
        );

        let unsafe_overshoot = include_str!("../../../examples/language/nat-mutual-recursion.t")
            .replace("<= 2", "<= 1");
        assert_eq!(
            analyze_for_compiler(&unsafe_overshoot).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_non_escaping_nested_functions_with_private_captures() {
        // TOPAL-COMPILER-NESTED-FUNCTION-001, TOPAL-FUNCTION-NESTED-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/nested-functions.t"
        ))
        .unwrap();
        assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
        let nested = program
            .functions
            .iter()
            .find(|function| function.source_name == "add-input")
            .expect("nested specialization is emitted");
        assert_eq!(nested.parameters.len(), 2);
        assert_eq!(nested.parameters[0].name, "value");
        assert_eq!(nested.parameters[1].name, "input");
        assert!(
            nested
                .parameters
                .iter()
                .all(|parameter| parameter.value_type == CompilerType::Int)
        );
        let outer = program
            .functions
            .iter()
            .find(|function| function.source_name == "answer")
            .expect("outer specialization is emitted");
        let CompilerExpressionKind::Call { arguments, .. } = &outer.body.result.kind else {
            panic!("outer body directly calls the nested specialization")
        };
        assert_eq!(arguments.len(), 2);
        assert!(matches!(
            &arguments[1].kind,
            CompilerExpressionKind::Local(name) if name == "input"
        ));

        let escaped = analyze_for_compiler(
            "use language (version is v0.1)\nconsume is fn (_ : Function) -> Int\n  1\nouter is fn (input : Int) -> Int\n  helper is fn (value : Int) -> Int\n    value + input\n  consume helper\nouter 1\n",
        )
        .unwrap_err();
        assert_eq!(escaped.code, "E-COMPILER-UNSUPPORTED");

        let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nouter is fn (value : Int) -> Int\n  helper is fn (value : Int) -> Int\n    value\n  helper 42\nouter 1\n",
        )
        .unwrap();
        let nested = shadowed
            .functions
            .iter()
            .find(|function| function.source_name == "helper")
            .unwrap();
        assert_eq!(nested.parameters.len(), 1);
        assert_eq!(nested.parameters[0].name, "value");
        assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));
    }

    #[test]
    fn models_complete_header_forward_function_calls() {
        // TOPAL-FUNCTION-FORWARD-DECLARATION-001, TOPAL-COMPILER-FUNCTION-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/forward-function-declarations.t"
        ))
        .unwrap();
        assert_eq!(
            program
                .functions
                .iter()
                .map(|function| function.source_name.as_str())
                .collect::<Vec<_>>(),
            ["decorate", "render"]
        );
        let CompilerExpressionKind::Call { symbol, .. } = &program.functions[1].body.result.kind
        else {
            panic!("expected the earlier function to call the later declaration")
        };
        assert_eq!(symbol, &program.functions[0].symbol);
    }

    #[test]
    fn models_only_structurally_proven_decreasing_int_recursion() {
        // TOPAL-FUNCTION-RECURSION-INT-001,
        // TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/decreasing-int-recursion.t"
        ))
        .unwrap();
        assert_eq!(program.functions.len(), 1);
        let function = &program.functions[0];
        assert_eq!(function.source_name, "sum-down");
        assert!(function.parameters[0].int_range.is_none());
        let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
            &function.body.result.kind
        else {
            panic!("expected the structurally proven recursion decision")
        };
        let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            right,
            ..
        } = &otherwise.kind
        else {
            panic!("expected the recursive action")
        };
        let CompilerExpressionKind::Call { symbol, .. } = &right.kind else {
            panic!("expected the direct recursive edge")
        };
        assert_eq!(symbol, &function.symbol);

        assert!(
            analyze_for_compiler(include_str!(
                "../../../examples/language/multiple-recursive-calls.t"
            ))
            .is_ok()
        );

        for source in [
            "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise loop (value - 0)\nloop 1\n",
            "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    <= 0 then loop (value - 1)\n    otherwise loop (value - 1)\nloop 1\n",
            "use language (version is v0.1)\nfirst is fn (value : Int) -> Int\n  second value\nsecond is fn (value : Int) -> Int\n  first value\nfirst 1\n",
        ] {
            assert_eq!(
                analyze_for_compiler(source).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_only_structurally_proven_increasing_int_recursion() {
        // TOPAL-FUNCTION-RECURSION-INT-INCREASING-001,
        // TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/increasing-int-recursion.t"
        ))
        .unwrap();
        assert_eq!(program.functions.len(), 1);
        let function = &program.functions[0];
        assert_eq!(function.source_name, "distance-up");
        assert!(function.parameters[0].int_range.is_none());
        let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
            &function.body.result.kind
        else {
            panic!("expected the structurally proven recursion decision")
        };
        let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            right,
            ..
        } = &otherwise.kind
        else {
            panic!("expected the recursive action")
        };
        let CompilerExpressionKind::Call { symbol, .. } = &right.kind else {
            panic!("expected the direct recursive edge")
        };
        assert_eq!(symbol, &function.symbol);

        assert!(
            analyze_for_compiler(include_str!(
                "../../../examples/language/positive-recursion-steps.t"
            ))
            .is_ok()
        );

        for source in [
            "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise loop (value + 0)\nloop (-1)\n",
            "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise loop (value - 1)\nloop (-1)\n",
        ] {
            assert_eq!(
                analyze_for_compiler(source).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
    }

    #[test]
    fn models_only_range_preserving_nat_recursion() {
        // TOPAL-FUNCTION-RECURSION-NAT-001,
        // TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001
        for (source, name, operation) in [
            (
                include_str!("../../../examples/language/nat-recursion.t"),
                "count-down",
                CompilerBinary::Subtract,
            ),
            (
                include_str!("../../../examples/language/nat-increasing-recursion.t"),
                "advance",
                CompilerBinary::Add,
            ),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            assert_eq!(program.functions.len(), 1);
            let function = &program.functions[0];
            assert_eq!(function.source_name, name);
            assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
            assert_eq!(function.result_type, CompilerType::Nat);
            assert!(function.parameters[0].int_range.is_none());
            let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
                &function.body.result.kind
            else {
                panic!("expected the structurally proven Nat decision")
            };
            let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
                panic!("expected the direct recursive edge")
            };
            assert_eq!(symbol, &function.symbol);
            let [argument] = arguments.as_slice() else {
                panic!("expected one recursive Nat argument")
            };
            let CompilerExpressionKind::IntToNat(argument) = &argument.kind else {
                panic!("expected proof-backed Nat evidence")
            };
            assert!(matches!(
                argument.kind,
                CompilerExpressionKind::Binary {
                    operation: actual,
                    ..
                } if actual == operation
            ));
        }

        let unsafe_overshoot = "use language (version is v0.1)\nloop is fn (value : Nat) -> Nat\n  value\n    <= 0 then value\n    otherwise loop (value - 2)\nloop 3\n";
        assert!(analyze_for_compiler(unsafe_overshoot).is_err());
    }

    #[test]
    fn models_only_structurally_verified_explicit_integer_measure() {
        // TOPAL-FUNCTION-DECREASES-001
        let source =
            include_str!("../../../examples/language/explicit-multi-parameter-decreases.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(program.functions.len(), 1);
        let function = &program.functions[0];
        assert_eq!(function.source_name, "repeat-add");
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
        assert_eq!(function.parameters[1].value_type, CompilerType::Int);
        assert!(
            function
                .parameters
                .iter()
                .all(|parameter| parameter.int_range.is_none())
        );
        let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
            &function.body.result.kind
        else {
            panic!("expected the measured recursion decision")
        };
        let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
            panic!("expected the measured recursive edge")
        };
        assert_eq!(symbol, &function.symbol);
        let [count, total] = arguments.as_slice() else {
            panic!("expected the complete measured recursive state")
        };
        assert!(matches!(count.kind, CompilerExpressionKind::IntToNat(_)));
        assert!(matches!(
            total.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                ..
            }
        ));

        for invalid in [
            source.replace("Decreases count", "Decreases total"),
            source.replace("count - 1", "count - 0"),
        ] {
            assert!(analyze_for_compiler(&invalid).is_err());
        }
    }

    #[test]
    fn models_explicit_early_return_and_skips_the_tail() {
        // TOPAL-FUNCTION-RETURN-001
        let source = "use language (version is v0.1)\nanswer is fn static () -> Int\n  return 40 + 2\n  0\nanswer ()\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(program.main.result.value_type, CompilerType::Int);
        assert!(program.functions.iter().any(|function| {
            function.source_name == "answer"
                && matches!(
                    function.body.result.kind,
                    CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Add,
                        ..
                    }
                )
        }));

        let outside = "use language (version is v0.1)\nreturn 42\n";
        assert_eq!(
            analyze_for_compiler(outside).unwrap_err().code,
            "E-RETURN-OUTSIDE-FUNCTION"
        );
    }

    #[test]
    fn models_fundamental_layout_policy_values_as_distinct_nominal_enums() {
        // TOPAL-LAYOUT-ENDIAN-001, TOPAL-LAYOUT-ACCESS-001,
        // TOPAL-LAYOUT-BIT-ORDER-001, TOPAL-LAYOUT-PACKING-001,
        // TOPAL-LAYOUT-FIELD-ORDER-001, TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001,
        // TOPAL-LAYOUT-ABSENCE-POLICY-001,
        // TOPAL-COMPILER-LAYOUT-POLICY-001
        let cases = [
            (
                include_str!("../../../examples/language/layout-endian.t"),
                "Endian",
                vec![0, 1],
            ),
            (
                include_str!("../../../examples/language/layout-access.t"),
                "Access",
                vec![0, 1, 2, 3],
            ),
            (
                include_str!("../../../examples/language/layout-bit-order.t"),
                "BitOrder",
                vec![0, 1],
            ),
            (
                include_str!("../../../examples/language/layout-packing.t"),
                "Packing",
                vec![0, 1],
            ),
            (
                include_str!("../../../examples/language/layout-field-order.t"),
                "FieldOrder",
                vec![0],
            ),
            (
                include_str!("../../../examples/language/layout-payload-placement.t"),
                "PayloadPlacement",
                vec![0, 1],
            ),
            (
                include_str!("../../../examples/language/layout-absence-policies.t"),
                "LayoutPolicy",
                vec![0, 1],
            ),
        ];
        for (source, expected_type, expected_values) in cases {
            let program = analyze_for_compiler(source).unwrap();
            let values = match &program.main.result.kind {
                CompilerExpressionKind::Enum(value) => vec![(&program.main.result, *value)],
                CompilerExpressionKind::Tuple(fields) => fields
                    .iter()
                    .map(|field| {
                        let CompilerExpressionKind::Enum(value) = field.kind else {
                            panic!("layout policy example contains only enum values")
                        };
                        (field, value)
                    })
                    .collect(),
                _ => panic!("layout policy example returns one enum or a Tuple"),
            };
            assert_eq!(
                values.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
                expected_values
            );
            assert!(values.iter().all(|(expression, _)| matches!(
                &expression.value_type,
                CompilerType::Enum(enumeration) if enumeration.name == expected_type
            )));
        }

        let mismatch = "use language (version is v0.1)\nLittle = Natural\n";
        assert_eq!(
            analyze_for_compiler(mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn models_nominal_enum_values_functions_and_exhaustive_decisions() {
        // TOPAL-TYPE-ENUM-001, TOPAL-DECISION-ENUM-001
        let source = "use language (version is v0.1)\nColor is Enum (Red, Green, Blue)\nnext is fn (value : Color) -> Color\n  value\n    Red then Green\n    Green then Blue\n    Blue then Red\nfavorite : Color is Red\n(next favorite, next Green, Red = Green)\n";
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Color, Color, Boolean)"
        );
        let enumeration = CompilerEnumType {
            name: "Color".into(),
            alternatives: vec!["Red".into(), "Green".into(), "Blue".into()],
        };
        assert!(program.functions.iter().any(|function| {
            function.parameters[0].value_type == CompilerType::Enum(enumeration.clone())
                && matches!(
                    &function.body.result.kind,
                    CompilerExpressionKind::EnumDecision {
                        rules,
                        otherwise: None,
                        ..
                    } if rules.len() == 3
                )
        }));
    }

    #[test]
    fn rejects_invalid_nominal_enum_declarations_and_decisions() {
        // TOPAL-TYPE-ENUM-001, TOPAL-DECISION-ENUM-001
        let duplicate = "use language (version is v0.1)\nColor is Enum (Red, Red)\nRed\n";
        assert_eq!(
            analyze_for_compiler(duplicate).unwrap_err().code,
            "E-DUPLICATE-ENUM-ALTERNATIVE"
        );

        let type_as_alternative =
            "use language (version is v0.1)\nColor is Enum (Color, Green)\nGreen\n";
        assert_eq!(
            analyze_for_compiler(type_as_alternative).unwrap_err().code,
            "E-DUPLICATE-ENUM-ALTERNATIVE"
        );

        let incomplete = "use language (version is v0.1)\nColor is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"red\"\nname Green\n";
        assert_eq!(
            analyze_for_compiler(incomplete).unwrap_err().code,
            "E-INCOMPLETE-DECISION"
        );

        let nominal_mismatch = "use language (version is v0.1)\nColor is Enum (Red, Green)\nSignal is Enum (Stop, Go)\n(Red = Stop)\n";
        assert_eq!(
            analyze_for_compiler(nominal_mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );

        let before_declaration =
            "use language (version is v0.1)\nvalue is Red\nColor is Enum (Red, Green)\nvalue\n";
        assert_eq!(
            analyze_for_compiler(before_declaration).unwrap_err().code,
            "E-UNBOUND-NAME"
        );

        let nested = "use language (version is v0.1)\nmake is fn () -> Unit\n  Local is Enum (First, Second)\n  ()\nmake ()\n";
        assert_eq!(
            analyze_for_compiler(nested).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    #[test]
    fn models_nominal_union_and_variant_payload_decisions() {
        // TOPAL-TYPE-UNION-001, TOPAL-TYPE-VARIANT-001,
        // TOPAL-DECISION-UNION-001
        let source = include_str!("../../../examples/language/unions-and-recursive-products.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "((Int, (Int, Int)), (Int, (Int, Int)), String, String)"
        );
        let describe = program
            .functions
            .iter()
            .find(|function| function.source_name == "describe")
            .expect("shared regression instantiates describe");
        let CompilerType::Sum(message) = &describe.parameters[0].value_type else {
            panic!("describe retains the nominal Message type")
        };
        assert!(!message.positional);
        assert_eq!(message.alternatives.len(), 2);
        assert!(message.alternatives[0].payload.is_none());
        let CompilerExpressionKind::SumDecision { rules, .. } = &describe.body.result.kind else {
            panic!("describe lowers a sum decision")
        };
        assert_eq!(rules.len(), 2);
        assert!(rules.iter().any(|rule| rule.binding.is_some()));

        let show = program
            .functions
            .iter()
            .find(|function| function.source_name == "show-scalar")
            .expect("shared regression instantiates show-scalar");
        let CompilerType::Sum(scalar) = &show.parameters[0].value_type else {
            panic!("show-scalar retains the nominal Scalar type")
        };
        assert!(scalar.positional);
        assert!(
            scalar
                .alternatives
                .iter()
                .all(|alternative| alternative.payload.is_some())
        );
    }

    #[test]
    fn rejects_invalid_nominal_sum_construction_and_matching() {
        // TOPAL-TYPE-UNION-001, TOPAL-TYPE-VARIANT-001,
        // TOPAL-DECISION-UNION-001
        let wrong_payload = "use language (version is v0.1)\nMessage is Union\n  Move : Int\n\nvalue is Move true\nvalue\n";
        assert_eq!(
            analyze_for_compiler(wrong_payload).unwrap_err().code,
            "E-UNION-PAYLOAD-CLASSIFIER"
        );
        let invalid_index =
            "use language (version is v0.1)\nScalar is Variant (String, Int)\nScalar at 2 42\n";
        assert_eq!(
            analyze_for_compiler(invalid_index).unwrap_err().code,
            "E-VARIANT-INDEX"
        );
        let incomplete = "use language (version is v0.1)\nMessage is Union\n  Stop\n  Move : Int\n\nread is fn (message : Message) -> Int\n  message\n    Stop then 0\nread Stop\n";
        assert_eq!(
            analyze_for_compiler(incomplete).unwrap_err().code,
            "E-INCOMPLETE-DECISION"
        );
        let foreign_variant = "use language (version is v0.1)\nScalar is Variant (String, Int)\nOther is Variant (String, Int)\nread is fn (scalar : Scalar) -> String\n  scalar\n    Other at 0 text then text\n    otherwise \"number\"\nread (Scalar at 0 \"text\")\n";
        assert_eq!(
            analyze_for_compiler(foreign_variant).unwrap_err().code,
            "E-VARIANT-TYPE"
        );
    }

    #[test]
    fn models_nominal_modular_construction_reduction_and_arithmetic() {
        // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
        // TOPAL-NUM-MODULAR-REDUCE-001, TOPAL-NUM-MODULAR-ARITHMETIC-001
        let source = include_str!("../../../examples/language/modular-numbers.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(ByteCounter, SignedByte, ByteCounter, SignedByte, Boolean, Boolean)"
        );
        let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
            panic!("modular regression retains its result product")
        };
        assert!(matches!(
            values[0].kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                ..
            }
        ));
        let CompilerExpressionKind::Binary { left, .. } = &values[0].kind else {
            unreachable!("checked above")
        };
        assert!(matches!(
            left.kind,
            CompilerExpressionKind::IntToModular { .. }
        ));
        assert!(matches!(
            values[2].kind,
            CompilerExpressionKind::ModularReduce { .. }
        ));
        assert_eq!(exact_int(&values[0]), Some(BigInt::from(0)));
        assert_eq!(exact_int(&values[1]), Some(BigInt::from(-128)));
        assert_eq!(exact_int(&values[2]), Some(BigInt::from(255)));
        assert_eq!(exact_int(&values[3]), Some(BigInt::from(-128)));
    }

    #[test]
    fn models_named_ranges_and_dynamic_checked_modular_construction() {
        // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001
        let source = include_str!("../../../examples/language/modular-checked-construction.t");
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            program.main.result.value_type.name(),
            "(Result (ByteCounter, lang arithmetic ArithmeticErrorCode), Result (ByteCounter, lang arithmetic ArithmeticErrorCode), lang arithmetic ArithmeticErrorCode, ErrorDomain, Optional SourceLocation)"
        );
        assert!(program.functions.iter().any(|function| {
            function.source_name == "construct"
                && matches!(
                    function.body.result.kind,
                    CompilerExpressionKind::ResultSuccess(_)
                )
        }));
        assert!(program.functions.iter().any(|function| {
            function.source_name == "construct"
                && matches!(
                    function.body.result.kind,
                    CompilerExpressionKind::ModularValidate { .. }
                )
        }));
    }

    #[test]
    fn rejects_invalid_modular_construction() {
        // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001
        let invalid_range = "use language (version is v0.1)\nDigit is ModNat (1 ..= 9)\nDigit 1\n";
        assert_eq!(
            analyze_for_compiler(invalid_range).unwrap_err().code,
            "E-MODULAR-RANGE"
        );

        let outside = "use language (version is v0.1)\nDigit is ModNat (0 ..= 9)\nDigit 10\n";
        assert_eq!(
            analyze_for_compiler(outside).unwrap_err().code,
            "E-MODULAR-OUT-OF-RANGE"
        );

        let nominal_mismatch = "use language (version is v0.1)\nDigit is ModNat (0 ..= 9)\nHour is ModNat (0 ..= 23)\n(Digit 1) + (Hour 1)\n";
        assert_eq!(
            analyze_for_compiler(nominal_mismatch).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }

    #[test]
    fn rejects_a_library_until_compiled_dependency_loading_exists() {
        let source = "use language (version is v0.1)\nuse library std (version is v0.1)\n()\n";
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}
