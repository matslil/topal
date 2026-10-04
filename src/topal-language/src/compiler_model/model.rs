const COMPILER_SYMBOLIC_CALLABLES: &[(CallableKind, &str)] = &[
    (CallableKind::Plus, "+"),
    (CallableKind::Minus, "-"),
    (CallableKind::Compare, "<=>"),
    (CallableKind::Equal, "="),
    (CallableKind::NotEqual, "/="),
    (CallableKind::Less, "<"),
    (CallableKind::Greater, ">"),
    (CallableKind::LessEqual, "<="),
    (CallableKind::GreaterEqual, ">="),
    (CallableKind::Multiply, "*"),
    (CallableKind::Divide, "/"),
    (CallableKind::QuotientModulo, "/%"),
    (CallableKind::Modulo, "%"),
    (CallableKind::Power, "^"),
    (CallableKind::Range, ".."),
    (CallableKind::RangeOpen, "<.."),
    (CallableKind::RangeInclusive, "..="),
    (CallableKind::RangeOpenInclusive, "<..="),
];

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerTaskHandlerKind {
    Start,
    Event,
    Request,
    Stream,
    Terminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerTaskHandler {
    pub name: String,
    pub payload_type: CompilerType,
    /// Successful response payload. Events and lifecycle handlers use Unit.
    pub response_type: CompilerType,
    /// Complete yield, resume, and final-result directions for stream handlers.
    pub stream_type: Option<CompilerGeneratorType>,
    pub kind: CompilerTaskHandlerKind,
    pub message_context_observable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerTaskScheduler {
    DeterministicImmediateFifo,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerTaskMessage {
    pub operation: String,
    pub transaction_identity: u64,
}

/// Target-independent task metadata retained by the checked compiler model.
///
/// This describes language identity and behavior, not the private LLVM layout.
/// A compiled-library format can therefore carry it without freezing the first
/// Linux x86-64 representation or calling convention.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerTaskType {
    pub classifier: String,
    pub definition: String,
    pub identity: String,
    pub queue_size: Option<u64>,
    pub state_name: String,
    pub state_type: Box<CompilerType>,
    pub handlers: Vec<CompilerTaskHandler>,
    pub scheduler: CompilerTaskScheduler,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerExternalLayoutFamily {
    UnsignedNat,
    Utf8Text,
    Product {
        fields: Vec<(String, String)>,
        packing: String,
    },
    TaggedOptionalNat {
        tag_layout: String,
        tags: Vec<(String, u64)>,
        payload_placement: String,
    },
    NatArray {
        count: usize,
        element_layout: String,
        stride_bits: u64,
    },
}

/// Target-independent evidence for one closed external representation.
///
/// These fields describe Topal semantics, never an LLVM type or native object
/// layout. A future compiled-library format can therefore serialize them
/// without adopting the first backend's private representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerExternalLayout {
    pub identity: String,
    pub semantic: String,
    pub storage_size_bits: u64,
    pub encoding: Option<String>,
    pub endian: Option<String>,
    pub access: String,
    pub alignment_bytes: u64,
    pub family: CompilerExternalLayoutFamily,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerAddressRangeType {
    pub identity: String,
    pub caching: String,
    pub minimum_access_size_bits: u64,
    pub medium: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerAddressRange {
    pub identity: String,
    pub range_type: CompilerAddressRangeType,
    pub lower: BigInt,
    pub upper: BigInt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerAddressOffsetType {
    pub identity: String,
    pub range: CompilerAddressRange,
    pub alignment_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerAddressOffset {
    pub identity: String,
    pub offset_type: CompilerAddressOffsetType,
    pub offset: BigInt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerLocationType {
    pub identity: String,
    pub layout: CompilerExternalLayout,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerLocation {
    pub location_type: CompilerLocationType,
    pub offset: CompilerAddressOffset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerExternalMetadata {
    Layout(CompilerExternalLayout),
    AddressRangeType(CompilerAddressRangeType),
    AddressRange(CompilerAddressRange),
    AddressOffsetType(CompilerAddressOffsetType),
    AddressOffset(CompilerAddressOffset),
    LocationType(CompilerLocationType),
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
    NativeSerializer(LanguageVersion),
    ExternalMetadata,
    SerializationStream(Box<Self>),
    Constraint,
    Boolean,
    Version,
    Int,
    Nat,
    InfiniteInt,
    InfiniteNat,
    InfiniteRational,
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
    TaskResponse(Box<Self>),
    Optional(Box<Self>),
    List(Box<Self>),
    Array { count: usize, element: Box<Self> },
    Set(Box<Self>),
    Bag(Box<Self>),
    Map { key: Box<Self>, value: Box<Self> },
    TraversalControl(Box<Self>),
    Generator(CompilerGeneratorType),
    Task(Box<CompilerTaskType>),
    ExternalLocation(Box<CompilerLocationType>),
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
                | Self::SerializationStream(_)
                | Self::Constraint
                | Self::Boolean
                | Self::Version
                | Self::Int
                | Self::Nat
                | Self::InfiniteInt
                | Self::InfiniteNat
                | Self::InfiniteRational
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
                | Self::TaskResponse(_)
                | Self::Optional(_)
                | Self::List(_)
                | Self::Array { .. }
                | Self::Set(_)
                | Self::Bag(_)
                | Self::Map { .. }
                | Self::TraversalControl(_)
                | Self::Generator(_)
                | Self::Task(_)
                | Self::ExternalLocation(_)
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
            Self::NativeSerializer(_) => "lang NativeSerializer".into(),
            Self::ExternalMetadata => "ExternalStorageMetadata".into(),
            Self::SerializationStream(_) => "SerializationStream".into(),
            Self::Constraint => "Constraint".into(),
            Self::Boolean => "Boolean".into(),
            Self::Version => "Version".into(),
            Self::Int | Self::InfiniteInt => "Int".into(),
            Self::Nat | Self::InfiniteNat => "Nat".into(),
            Self::Rational | Self::InfiniteRational => "Rational".into(),
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
            Self::TaskResponse(success) => format!("Result ({}, ())", success.name()),
            Self::Optional(payload) => format!("Optional {}", payload.name()),
            Self::List(element) => format!("List {}", element.name()),
            Self::Array { count, element } => format!("Array {count} {}", element.name()),
            Self::Set(element) => format!("Set {}", element.name()),
            Self::Bag(element) => format!("Bag {}", element.name()),
            Self::Map { key, value } => format!("Map ({}, {})", key.name(), value.name()),
            Self::TraversalControl(payload) => format!("TraversalControl {}", payload.name()),
            Self::Generator(generator) => format!(
                "Generator {} {} {}",
                generator.yield_type.name(),
                generator.resume_type.name(),
                generator.result_type.name()
            ),
            Self::Task(task) => task.classifier.clone(),
            Self::ExternalLocation(location) => location.identity.clone(),
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
    InfinityMultiply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerValidation {
    RationalToInt,
    IntToNat,
    Constraint(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerListIndexOperation {
    Split,
    Take,
    Drop,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerListZipOperation {
    Exact,
    Shortest,
    Longest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerContainerKind {
    Array,
    Set,
    Bag,
    Map,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerMapCollisionPolicy {
    Reject,
    KeepFirst,
    KeepLast,
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
    LintNamespace,
    FunctionValue(u32),
    Identity(CompilerIdentity),
    TypeView(CompilerTypeView),
    FunctionView(CompilerFunctionView),
    LanguageContext(CompilerLanguageContext),
    Capability(CompilerCapability),
    NativeSerializer(LanguageVersion),
    ExternalMetadata(CompilerExternalMetadata),
    ExternalLayoutCoerce {
        layout: CompilerExternalLayout,
        value: Box<CompilerExpression>,
    },
    ExternalLocationConstruct(CompilerLocation),
    ExternalLocationWrite {
        location: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
        metadata: CompilerLocation,
    },
    ExternalLocationRead {
        location: Box<CompilerExpression>,
        metadata: CompilerLocation,
    },
    Serialize {
        bytes: Vec<u8>,
        value: Box<CompilerExpression>,
    },
    Deserialize(Box<CompilerExpression>),
    TaskConstruct {
        task: CompilerTaskType,
        initial: Box<CompilerExpression>,
    },
    TaskStateLoad {
        task: Box<CompilerExpression>,
        message: CompilerTaskMessage,
    },
    TaskStateReplace {
        task: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
        message: CompilerTaskMessage,
    },
    ConstraintValue(u32),
    Boolean(bool),
    Version(LanguageVersion),
    Int(BigInt),
    Infinity {
        negative: bool,
    },
    Rational(BigRational),
    String(String),
    StringEmpty,
    StringConcat {
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
    },
    StringRangeSelect {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
        range: Box<CompilerExpression>,
    },
    StringEmptyPredicate(Box<CompilerExpression>),
    StringUtf8ByteCount(Box<CompilerExpression>),
    StringCharacterCount(Box<CompilerExpression>),
    StringUnicodeScalarValue(Box<CompilerExpression>),
    CharacterAsciiDecimalDigit(Box<CompilerExpression>),
    CharacterUnicodeWhitespace(Box<CompilerExpression>),
    StringCharactersGenerator {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
    },
    StringRangeCharactersGenerator {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
        range: Box<CompilerExpression>,
    },
    StringRangeCharactersCollect {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
        range: Box<CompilerExpression>,
    },
    StringProvenanceCharactersGenerator {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
    },
    StringProvenanceCharactersCollect {
        text: Box<CompilerExpression>,
        characters: Vec<String>,
    },
    StringDynamicScalarCharactersCollect(Box<CompilerExpression>),
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
    CustomCharacterGenerator {
        declaration: String,
        declaration_namespace: String,
        declaration_span: Span,
        initial_parameter: Box<CompilerParameter>,
        initial: Box<CompilerExpression>,
        prefix: Box<CompilerBlock>,
        characters: Vec<String>,
        locals: Vec<CompilerGeneratorLocal>,
        close_handler: Option<CompilerGeneratorCloseHandler>,
        result: Box<CompilerExpression>,
    },
    CustomValueGenerator {
        declaration: String,
        declaration_namespace: String,
        declaration_span: Span,
        initial_parameter: Box<CompilerParameter>,
        initial: Box<CompilerExpression>,
        additional_initial_parameters: Vec<CompilerParameter>,
        additional_initials: Vec<CompilerExpression>,
        prefix: Box<CompilerBlock>,
        yields: Vec<CompilerGeneratorYield>,
        continuations: Vec<CompilerGeneratorContinuation>,
        explicit_return: Option<Span>,
        result: Box<CompilerExpression>,
    },
    CustomCharacterClose {
        generator: Box<CompilerExpression>,
        provenance: Box<CompilerExpression>,
        close_domain: String,
    },
    CustomCharacterHandledClose {
        generator: Box<CompilerExpression>,
        handler: CompilerGeneratorCloseHandler,
    },
    CustomCharacterForeach {
        source: Box<CompilerExpression>,
        declaration_span: Span,
        characters: Vec<String>,
        locals: Vec<CompilerGeneratorLocal>,
        parameter: CompilerParameter,
        body: Box<CompilerBlock>,
        result: Box<CompilerExpression>,
    },
    CustomValueForeach {
        source: Box<CompilerExpression>,
        transferred_initial: Option<Box<CompilerExpression>>,
        declaration_span: Span,
        initial_parameter: Box<CompilerParameter>,
        additional_initial_parameters: Vec<CompilerParameter>,
        prefix: Box<CompilerBlock>,
        yields: Vec<CompilerGeneratorYield>,
        continuations: Vec<CompilerGeneratorContinuation>,
        explicit_return: Option<Span>,
        parameter: CompilerParameter,
        body: Box<CompilerBlock>,
        result: Box<CompilerExpression>,
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
    TupleField {
        tuple: Box<CompilerExpression>,
        index: usize,
    },
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
    ExitSequence {
        preceding: Vec<CompilerExpression>,
        result: Box<CompilerExpression>,
    },
    PrivateBinding {
        storage_name: String,
        value: Box<CompilerExpression>,
        body: Box<CompilerExpression>,
    },
    Local(String),
    InfinityLocal {
        storage_name: String,
        negative: bool,
    },
    Negate(Box<CompilerExpression>),
    Absolute(Box<CompilerExpression>),
    IntToRational(Box<CompilerExpression>),
    RationalConstruct {
        numerator: Box<CompilerExpression>,
        denominator: Box<CompilerExpression>,
    },
    RationalToInt(Box<CompilerExpression>),
    IntToNat(Box<CompilerExpression>),
    IntToNatBoundary(Box<CompilerExpression>),
    ResultSuccess(Box<CompilerExpression>),
    ResultError(Box<CompilerExpression>),
    ResultProject(Box<CompilerExpression>),
    ResultProjectBoundary(Box<CompilerExpression>),
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
    ListForeach {
        list: Box<CompilerExpression>,
        parameter: CompilerParameter,
        body: Box<CompilerBlock>,
    },
    ListInsertAt {
        list: Box<CompilerExpression>,
        boundary: usize,
        inserted: Box<CompilerExpression>,
        inserts_list: bool,
    },
    ListInsertEverywhere {
        list: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    ListCartesianStringInt {
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
    },
    ListIndexOperation {
        list: Box<CompilerExpression>,
        index: usize,
        operation: CompilerListIndexOperation,
    },
    ListRemoveIndexRange {
        list: Box<CompilerExpression>,
        start: usize,
        end: usize,
    },
    ListReject {
        list: Box<CompilerExpression>,
        parameters: Vec<CompilerParameter>,
        predicate: Box<CompilerBlock>,
        indexes: bool,
    },
    ListZip {
        left: Box<CompilerExpression>,
        right: Box<CompilerExpression>,
        operation: CompilerListZipOperation,
        left_default: Option<Box<CompilerExpression>>,
        right_default: Option<Box<CompilerExpression>>,
    },
    ListUnzip(Box<CompilerExpression>),
    ListEntries(Box<CompilerExpression>),
    ListCollectString(Box<CompilerExpression>),
    ContainerCollect {
        source: Box<CompilerExpression>,
        kind: CompilerContainerKind,
        map_policy: Option<CompilerMapCollisionPolicy>,
        map_keys: Option<Vec<String>>,
    },
    ContainerEntryCount(Box<CompilerExpression>),
    ContainerEmpty(Box<CompilerExpression>),
    ArrayAt {
        array: Box<CompilerExpression>,
        index: usize,
    },
    SetContains {
        set: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    BagMultiplicity {
        bag: Box<CompilerExpression>,
        value: Box<CompilerExpression>,
    },
    MapLookup {
        mapping: Box<CompilerExpression>,
        key: Box<CompilerExpression>,
        exact_key: Option<String>,
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
    pub source_visible: bool,
    pub value_type: CompilerType,
    pub int_range: Option<IntRange>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerPatternIdentity {
    pub first_parameter: usize,
    pub repeated_parameter: usize,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerGeneratorLocal {
    pub parameter: CompilerParameter,
    pub activation_after_resumptions: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerGeneratorYield {
    Initial(Span),
    Value(Box<CompilerExpression>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerGeneratorContinuation {
    pub after_resumptions: usize,
    pub body: CompilerBlock,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerGeneratorCloseHandler {
    pub result_binding: String,
    pub result_binding_span: Span,
    pub error_codes: Vec<CompilerErrorCodeRule>,
    pub error_binding: String,
    pub error_binding_span: Span,
    pub error_action: Box<CompilerExpression>,
    pub ok_binding: String,
    pub ok_binding_span: Span,
    pub ok_action: Box<CompilerExpression>,
    pub error_code_type: CompilerEnumType,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerFunction {
    pub source_name: String,
    pub module_identity: Option<Vec<String>>,
    pub symbol: String,
    pub parameters: Vec<CompilerParameter>,
    pub pattern_identities: Vec<CompilerPatternIdentity>,
    pub result_type: CompilerType,
    pub result_captures: Vec<CompilerFunctionResultCapture>,
    pub body: CompilerBlock,
    pub span: Span,
    pub is_static: bool,
    pub declared_effects: Option<CompilerEffectRow>,
    pub foreign: Option<CompilerCFunction>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerCValue {
    Void,
    SignedInt32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerCFunction {
    pub library_identity: String,
    pub external_symbol: String,
    pub parameters: Vec<CompilerCValue>,
    pub result: CompilerCValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerFunctionResultCapture {
    pub name: String,
    pub path: Vec<CompilerAggregatePathElement>,
    pub value_type: CompilerType,
    pub value: CompilerExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompilerAggregatePathElement {
    Tuple(usize),
    Record(String),
    ListEntry(usize),
    ArrayEntry(usize),
    MapValue(String),
    OptionalPayload,
    SumPayload(String),
    ResultSuccess,
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
    pub primary_source_end: usize,
    pub dependencies: Vec<CompilerDependency>,
    pub language_version: LanguageVersion,
    pub language_features: Vec<String>,
    pub main: CompilerBlock,
    pub function_value_names: Vec<String>,
    pub constraints: Vec<CompilerConstraint>,
    pub interfaces: Vec<CompilerInterface>,
    pub interface_implementations: Vec<CompilerInterfaceImplementation>,
    pub tasks: Vec<CompilerTaskType>,
    /// Instances are in callee-before-caller order.
    pub functions: Vec<CompilerFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerDependency {
    pub identity: String,
    pub source_name: String,
    pub source_text: String,
    pub source_span: Span,
}

/// Compatibility name for the shared source-module boundary.
pub type CompilerSourceModule = SourceModule;

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
    published: bool,
    module_identity: Option<Vec<String>>,
}

#[derive(Clone)]
struct GeneratorSource {
    name: Span,
    span: Span,
    initial_parameter: CompilerParameter,
    additional_initial_parameters: Vec<CompilerParameter>,
    yield_parameter: usize,
    prefix: CompilerBlock,
    literal_characters: Option<Vec<String>>,
    value_yields: Option<Vec<CompilerGeneratorYield>>,
    value_continuations: Vec<CompilerGeneratorContinuation>,
    explicit_return: Option<Span>,
    yield_count: usize,
    local: Option<CompilerGeneratorLocal>,
    local_function: Option<CompilerFunction>,
    close_handler: Option<CompilerGeneratorCloseHandler>,
    result: CompilerExpression,
}

#[derive(Clone)]
struct TaskTypeSource {
    classifier: String,
    identity: String,
    queue_size: Option<u64>,
    span: Span,
}

#[derive(Clone)]
struct TaskHandlerSource {
    name: String,
    parameters: Vec<FunctionParameter>,
    body: Vec<Statement>,
    span: Span,
}

#[derive(Clone)]
struct TaskDefinitionSource {
    task: CompilerTaskType,
    handlers: BTreeMap<String, TaskHandlerSource>,
    span: Span,
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
type TaskTypes = BTreeMap<String, TaskTypeSource>;
type TaskDefinitions = BTreeMap<String, TaskDefinitionSource>;
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
    origin: usize,
    runtime_bound: bool,
    value_type: CompilerType,
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
    infinity_negative: Option<bool>,
    string_value: Option<String>,
    string_characters: Option<Vec<String>>,
    closed_int_range: Option<ClosedIntRange>,
    list_count: Option<usize>,
    list_string_keys: Option<Vec<String>>,
    list_string_characters: Option<Vec<String>>,
    list_entries: Option<Vec<StaticValueFacts>>,
    array_entries: Option<Vec<StaticValueFacts>>,
    map_entries: Option<Vec<(String, StaticValueFacts)>>,
    tuple_fields: Vec<StaticValueFacts>,
    record_fields: BTreeMap<String, StaticValueFacts>,
    optional: Option<CompilerOptionalFacts>,
    sum: Option<CompilerSumFacts>,
    result: Option<CompilerResultFacts>,
    namespace: Option<CompilerNamespaceFacts>,
    callable: Option<CompilerCallableFacts>,
    static_capability: Option<CompilerCapability>,
}

#[derive(Clone)]
struct CompilerNamespaceFacts {
    name: String,
    functions: BTreeMap<String, Vec<FunctionSource>>,
    generators: BTreeMap<String, Vec<GeneratorSource>>,
    bindings: BTreeMap<String, CompilerDataMemberFacts>,
}

#[derive(Clone)]
struct CompilerDataMemberFacts {
    storage_name: String,
    value_type: CompilerType,
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
    infinity_negative: Option<bool>,
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
    aggregate_arguments: Vec<StaticValueFacts>,
    callable_captures: Vec<CompilerContextCapture>,
    aggregate_captures: Vec<CompilerContextCapture>,
    scope_arguments: Vec<Option<CompilerNamespaceFacts>>,
    scope_captures: Vec<CompilerContextCapture>,
    lexical_captures: Vec<CompilerContextCapture>,
    context_captures: Vec<CompilerContextCapture>,
}

struct CompilerArgumentBinding {
    storage_name: String,
    value: CompilerExpression,
}

#[derive(Clone)]
struct CompilerFunctionCallReference {
    declarations: Vec<FunctionSource>,
    span: Span,
    arguments: Vec<Expression>,
    is_ordinary_unqualified: bool,
    is_retained_value: bool,
}

#[derive(Clone)]
struct CompilerNamedCallTarget {
    declarations: Vec<FunctionSource>,
}

type NormalizedPackagedCall = (
    FunctionSource,
    Vec<CompilerExpression>,
    Vec<CompilerArgumentBinding>,
    BTreeMap<String, CompilerType>,
);

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
            infinity_negative: facts.infinity_negative,
            declaration_end,
        }
    }
}

#[derive(Clone, Default)]
struct StaticValueFacts {
    int_range: Option<IntRange>,
    rational_value: Option<BigRational>,
    string_value: Option<String>,
    string_characters: Option<Vec<String>>,
    callable: Option<CompilerCallableFacts>,
    tuple_fields: Vec<Self>,
    record_fields: BTreeMap<String, Self>,
    list_entries: Option<Vec<Self>>,
    list_string_characters: Option<Vec<String>>,
    array_entries: Option<Vec<Self>>,
    map_entries: Option<Vec<(String, Self)>>,
    optional: Option<CompilerOptionalFacts>,
    sum: Option<CompilerSumFacts>,
    result: Option<CompilerResultFacts>,
}

#[derive(Clone)]
enum CompilerOptionalFacts {
    None,
    Some(Box<StaticValueFacts>),
}

#[derive(Clone)]
struct CompilerSumFacts {
    alternative: u32,
    alternative_name: String,
    payload: Option<Box<StaticValueFacts>>,
}

#[derive(Clone)]
struct CompilerResultFacts {
    success: Box<StaticValueFacts>,
}

fn retain_static_value_facts(binding: &mut BindingFacts, value: &StaticValueFacts) {
    binding.int_range.clone_from(&value.int_range);
    binding.rational_value.clone_from(&value.rational_value);
    binding.string_value.clone_from(&value.string_value);
    binding
        .string_characters
        .clone_from(&value.string_characters);
    binding.callable.clone_from(&value.callable);
    binding.tuple_fields.clone_from(&value.tuple_fields);
    binding.record_fields.clone_from(&value.record_fields);
    binding.list_entries.clone_from(&value.list_entries);
    binding
        .list_string_characters
        .clone_from(&value.list_string_characters);
    binding.array_entries.clone_from(&value.array_entries);
    binding.map_entries.clone_from(&value.map_entries);
    binding.optional.clone_from(&value.optional);
    binding.sum.clone_from(&value.sum);
    binding.result.clone_from(&value.result);
}

fn present_optional_payload(facts: StaticValueFacts) -> Option<StaticValueFacts> {
    match facts.optional {
        Some(CompilerOptionalFacts::Some(payload)) => Some(*payload),
        Some(CompilerOptionalFacts::None) | None => None,
    }
}

fn merge_string_characters(
    left: Option<Vec<String>>,
    right: Option<Vec<String>>,
) -> Option<Vec<String>> {
    let mut merged = left?;
    for character in right? {
        if !merged.contains(&character) {
            merged.push(character);
        }
    }
    Some(merged)
}
