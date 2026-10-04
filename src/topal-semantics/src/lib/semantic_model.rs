/// Source-visible immutable language revision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LanguageVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub build: u64,
}

impl LanguageVersion {
    pub const DESIGN_0: Self = Self {
        major: 0,
        minor: 1,
        patch: 0,
        build: 0,
    };

    /// Portable design-pattern assurance revision.
    pub const DESIGN_1: Self = Self {
        major: 0,
        minor: 2,
        patch: 0,
        build: 0,
    };

    #[must_use]
    pub const fn is_supported_core(self) -> bool {
        matches!(self, Self::DESIGN_0 | Self::DESIGN_1)
    }
}

impl Default for LanguageVersion {
    fn default() -> Self {
        Self::DESIGN_0
    }
}

impl fmt::Display for LanguageVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.patch == 0 && self.build == 0 {
            write!(formatter, "v{}.{}", self.major, self.minor)
        } else if self.build == 0 {
            write!(formatter, "v{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            write!(
                formatter,
                "v{}.{}.{}-{}",
                self.major, self.minor, self.patch, self.build
            )
        }
    }
}

impl FromStr for LanguageVersion {
    type Err = &'static str;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let body = text
            .strip_prefix('v')
            .ok_or("a language version begins with `v`")?;
        let (release, build) = body.split_once('-').unwrap_or((body, "0"));
        let components = release.split('.').collect::<Vec<_>>();
        if !(2..=3).contains(&components.len()) {
            return Err("a language version requires major and minor components");
        }
        Ok(Self {
            major: components[0].parse().map_err(|_| "invalid major version")?,
            minor: components[1].parse().map_err(|_| "invalid minor version")?,
            patch: components.get(2).map_or(Ok(0), |value| {
                value.parse().map_err(|_| "invalid patch version")
            })?,
            build: build.parse().map_err(|_| "invalid build version")?,
        })
    }
}

/// The closed object-kind vocabulary from `TOPAL-TYPE-KIND-001`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ObjectKind {
    Value,
    Type,
    Function,
    Predicate,
    Constraint,
    Capability,
    Interface,
    Pattern,
    Effect,
    Protocol,
    Scope,
    Module,
}

impl ObjectKind {
    /// Whether an object of `self` satisfies the requested kind classifier.
    #[must_use]
    pub fn satisfies(self, requested: Self) -> bool {
        self == requested || matches!((self, requested), (Self::Predicate, Self::Function))
    }
}

/// Canonical identity of a Topal type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeIdentity {
    Fundamental(&'static str),
    Structural(StructuralType),
    Nominal {
        declaration: DeclarationIdentity,
        parameters: Vec<Self>,
    },
}

/// Recursively exact structural type identity from `TOPAL-TYPE-ID-001`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StructuralType {
    Tuple(Vec<TypeIdentity>),
    Record(Vec<(String, TypeIdentity)>),
    Variant(Vec<TypeIdentity>),
    Union(Vec<(String, TypeIdentity)>),
}

/// Stable source declaration identity; aliases retain this identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeclarationIdentity {
    pub module: String,
    pub name: String,
    pub ordinal: usize,
}

/// A canonical qualified semantic name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QualifiedName(pub Vec<String>);

/// A generic type pattern whose parameters are replaced simultaneously.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypePattern {
    Parameter(String),
    Concrete(TypeIdentity),
    Tuple(Vec<Self>),
    Record(Vec<(String, Self)>),
}

impl TypePattern {
    /// Instantiate this pattern without discarding the substitution evidence.
    #[must_use]
    pub fn instantiate(&self, arguments: &BTreeMap<String, TypeIdentity>) -> Option<TypeIdentity> {
        match self {
            Self::Parameter(name) => arguments.get(name).cloned(),
            Self::Concrete(identity) => Some(identity.clone()),
            Self::Tuple(fields) => Some(TypeIdentity::Structural(StructuralType::Tuple(
                fields
                    .iter()
                    .map(|field| field.instantiate(arguments))
                    .collect::<Option<Vec<_>>>()?,
            ))),
            Self::Record(fields) => Some(TypeIdentity::Structural(StructuralType::Record(
                fields
                    .iter()
                    .map(|(label, field)| Some((label.clone(), field.instantiate(arguments)?)))
                    .collect::<Option<Vec<_>>>()?,
            ))),
        }
    }
}

/// Canonical evidence retained by one successful generic instantiation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstantiationEvidence {
    pub declaration: DeclarationIdentity,
    pub arguments: BTreeMap<String, TypeIdentity>,
    pub result: TypeIdentity,
}

/// Canonical proof that one subject provides an atomic capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityEvidence {
    pub capability: QualifiedName,
    pub subject: TypeIdentity,
    pub roles: BTreeMap<String, DeclarationIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceTrust {
    Verified,
    TrustedUnverified,
    Refuted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyObligation {
    OrdinaryLaw,
    MemorySafety,
    Totality,
    RaceFreedom,
    DeadlockFreedom,
}

/// Validate whether capability evidence may discharge an obligation.
///
/// # Errors
///
/// Refuted evidence never applies; safety-critical obligations require proof.
pub const fn admit_evidence(
    trust: EvidenceTrust,
    obligation: SafetyObligation,
) -> Result<(), &'static str> {
    match (trust, obligation) {
        (EvidenceTrust::Refuted, _) => Err("refuted capability evidence cannot be admitted"),
        (EvidenceTrust::Verified, _)
        | (EvidenceTrust::TrustedUnverified, SafetyObligation::OrdinaryLaw) => Ok(()),
        (EvidenceTrust::TrustedUnverified, _) => {
            Err("safety-critical capability evidence must be verified")
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExistentialPackage<W, V> {
    witness: W,
    value: V,
}

impl<W, V> ExistentialPackage<W, V> {
    #[must_use]
    pub const fn pack(witness: W, value: V) -> Self {
        Self { witness, value }
    }

    /// Eliminate the package without exposing an independently owned witness.
    pub fn eliminate<R>(&self, use_package: impl FnOnce(&W, &V) -> R) -> R {
        use_package(&self.witness, &self.value)
    }
}

/// A coherence-checked collection of capability proofs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilitySet(BTreeMap<(QualifiedName, TypeIdentity), CapabilityEvidence>);

impl CapabilitySet {
    /// Insert evidence, accepting an identical proof and rejecting ambiguity.
    ///
    /// # Errors
    ///
    /// Returns an error when the same capability and subject have different roles.
    pub fn insert(&mut self, evidence: CapabilityEvidence) -> Result<(), &'static str> {
        let key = (evidence.capability.clone(), evidence.subject.clone());
        if let Some(existing) = self.0.get(&key) {
            return if existing == &evidence {
                Ok(())
            } else {
                Err("conflicting canonical capability evidence")
            };
        }
        self.0.insert(key, evidence);
        Ok(())
    }

    #[must_use]
    pub fn select(
        &self,
        capability: &QualifiedName,
        subject: &TypeIdentity,
    ) -> Option<&CapabilityEvidence> {
        self.0.get(&(capability.clone(), subject.clone()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InterfaceOperation {
    Function {
        inputs: Vec<TypeIdentity>,
        result: TypeIdentity,
    },
    Generator {
        inputs: Vec<TypeIdentity>,
        yielded: TypeIdentity,
        resumed: TypeIdentity,
        result: TypeIdentity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceShape {
    pub identity: DeclarationIdentity,
    pub operations: BTreeMap<String, InterfaceOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceImplementation {
    pub interface: InterfaceShape,
    pub operations: BTreeMap<String, DeclarationIdentity>,
}

impl InterfaceImplementation {
    /// Validate that an implementation supplies exactly the interface roles.
    ///
    /// # Errors
    ///
    /// Returns an error for a missing or additional operation role.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.interface.operations.keys().eq(self.operations.keys()) {
            Ok(())
        } else {
            Err("interface implementation roles do not match the interface")
        }
    }
}

/// Conservative, canonically ordered effect row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectSet(BTreeSet<QualifiedName>);

impl EffectSet {
    #[must_use]
    pub fn from_effects(effects: impl IntoIterator<Item = QualifiedName>) -> Self {
        Self(effects.into_iter().collect())
    }

    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0).cloned().collect())
    }

    pub fn iter(&self) -> impl Iterator<Item = &QualifiedName> {
        self.0.iter()
    }

    #[must_use]
    pub fn contains_all(&self, required: &Self) -> bool {
        required.0.is_subset(&self.0)
    }
}

/// A canonical effect row with an optional polymorphic tail variable.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectRow {
    pub known: EffectSet,
    pub tail: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceIdentity(pub QualifiedName);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceState {
    Owned { binding: String, lifetime: u64 },
    Destroyed,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResourceTracker {
    resources: BTreeMap<ResourceIdentity, ResourceState>,
    declaration_order: Vec<ResourceIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessRights {
    pub read: bool,
    pub write: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location {
    pub resource: ResourceIdentity,
    pub base: u128,
    pub size: u128,
    pub layout_size: u128,
    pub alignment: u128,
    pub rights: AccessRights,
    pub lifetime: u64,
    pub access_widths: BTreeSet<u128>,
}

impl Location {
    /// Validate the complete semantic location tuple.
    ///
    /// # Errors
    ///
    /// Returns an error for overflow, range, alignment, rights, or access defects.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.base.checked_add(self.size).is_none() {
            return Err("location range overflows its address domain");
        }
        if self.layout_size > self.size {
            return Err("layout exceeds the location range");
        }
        if self.alignment == 0 || !self.base.is_multiple_of(self.alignment) {
            return Err("location base does not satisfy alignment");
        }
        if !self.rights.read && !self.rights.write {
            return Err("location grants no access rights");
        }
        if self.access_widths.is_empty()
            || self
                .access_widths
                .iter()
                .any(|width| *width == 0 || *width > self.size)
        {
            return Err("location has an invalid access width");
        }
        Ok(())
    }

    #[must_use]
    pub fn aliases(&self, other: &Self) -> bool {
        self.resource == other.resource
            && self.base < other.base.saturating_add(other.size)
            && other.base < self.base.saturating_add(self.size)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemoryEvent {
    Read {
        location: Location,
        width: u128,
    },
    Write {
        location: Location,
        width: u128,
        value_identity: String,
    },
}

impl MemoryEvent {
    /// Check authorization and declared width before an event occurs.
    ///
    /// # Errors
    ///
    /// Returns an error when the event is not permitted by its location.
    pub fn validate(&self) -> Result<(), &'static str> {
        let (location, width, permitted) = match self {
            Self::Read { location, width } => (location, width, location.rights.read),
            Self::Write {
                location, width, ..
            } => (location, width, location.rights.write),
        };
        location.validate()?;
        if !permitted {
            return Err("memory event is not authorized");
        }
        if !location.access_widths.contains(width) || !location.base.is_multiple_of(*width) {
            return Err("memory event width or alignment is invalid");
        }
        Ok(())
    }

    fn location(&self) -> &Location {
        match self {
            Self::Read { location, .. } | Self::Write { location, .. } => location,
        }
    }

    fn writes(&self) -> bool {
        matches!(self, Self::Write { .. })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MemoryExecution {
    events: Vec<MemoryEvent>,
    order: BTreeSet<(usize, usize)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerSynchronization {
    Atomic,
    Lock,
    Transaction,
    MessageQueue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SynchronizationRefinement {
    pub strategy: CompilerSynchronization,
    pub source_visible: bool,
    pub preserves_happens_before: bool,
    pub preserves_coherence: bool,
}

impl SynchronizationRefinement {
    /// Validate compiler-introduced synchronization against design-0.
    ///
    /// # Errors
    ///
    /// Rejects source-visible or memory-order-weakening synchronization.
    pub const fn validate(&self) -> Result<(), &'static str> {
        if self.source_visible {
            return Err("compiler synchronization cannot be a design-0 source value");
        }
        if !self.preserves_happens_before || !self.preserves_coherence {
            return Err("compiler synchronization does not refine the memory model");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HardwareAccessPolicy {
    pub volatile: bool,
    pub widths: BTreeSet<u128>,
    pub ordering: QualifiedName,
}

impl HardwareAccessPolicy {
    /// Validate an explicit hardware access capability.
    ///
    /// # Errors
    ///
    /// Rejects capabilities without an access width or named ordering.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.widths.is_empty() || self.widths.contains(&0) {
            return Err("hardware access capability has no valid width");
        }
        if self.ordering.0.is_empty() {
            return Err("hardware access capability has no ordering identity");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservableMemoryOutcome {
    Value(String),
    Error(String),
    ProtocolTransition(String),
    Effect(String),
}

/// Check the observable proof obligation for a compiler transformation.
///
/// # Errors
///
/// Rejects any introduced, removed, or reordered observation.
pub fn validate_optimization(
    source: &[ObservableMemoryOutcome],
    transformed: &[ObservableMemoryOutcome],
) -> Result<(), &'static str> {
    if source == transformed {
        Ok(())
    } else {
        Err("optimization changes an observable memory outcome")
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaskIdentity(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskState {
    Constructed,
    Runnable,
    Waiting { external: bool },
    Closing,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TaskRecord {
    parent: Option<TaskIdentity>,
    children: BTreeSet<TaskIdentity>,
    state: TaskState,
    cancellation_requested: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaskScheduler {
    next_identity: u64,
    tasks: BTreeMap<TaskIdentity, TaskRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteractionKind {
    Event,
    Request,
    Stream,
    DirectCall,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionPolicy {
    Wait,
    Reject,
    ContainedDiagnosticLoss,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionState {
    Enqueued,
    Received,
    Replied { result_identity: String },
    Streaming { values: Vec<String> },
    Closed,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageTransaction {
    pub identity: u64,
    pub sender: TaskIdentity,
    pub receiver: TaskIdentity,
    pub endpoint: QualifiedName,
    pub kind: InteractionKind,
    pub payload_identity: String,
    pub state: TransactionState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageSend {
    pub sender: TaskIdentity,
    pub receiver: TaskIdentity,
    pub endpoint: QualifiedName,
    pub kind: InteractionKind,
    pub payload_identity: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MessageLedger {
    next_identity: u64,
    transactions: BTreeMap<u64, MessageTransaction>,
    admitted: BTreeMap<QualifiedName, usize>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DependencyNode {
    Task(TaskIdentity),
    Transaction(u64),
    Resource(ResourceIdentity),
    External(QualifiedName),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DependencyGraph {
    nodes: BTreeSet<DependencyNode>,
    edges: BTreeSet<(DependencyNode, DependencyNode)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteOrder {
    Little,
    Big,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Packing {
    Natural,
    Packed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadPlacement {
    AfterTag,
    Overlay,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Layout {
    Scalar {
        bits: u64,
        signed: bool,
        byte_order: ByteOrder,
    },
    Product {
        fields: Vec<(String, Self)>,
        packing: Packing,
    },
    Sum {
        tag: Box<Self>,
        alternatives: Vec<(String, Self)>,
        placement: PayloadPlacement,
    },
    Sequence {
        element: Box<Self>,
        count: Option<u64>,
    },
    Text {
        code_unit_bits: u64,
        byte_order: ByteOrder,
        length_prefix: bool,
        terminator: Option<u32>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutValue {
    Integer(i128),
    Product(Vec<Self>),
    Sum {
        alternative: usize,
        value: Box<Self>,
    },
    Sequence(Vec<Self>),
    Text(String),
}

impl Layout {
    /// Validate recursive external-representation policy.
    ///
    /// # Errors
    ///
    /// Returns an error for zero/non-octet widths, duplicate fields or
    /// alternatives, unbounded sequences, or text without a boundary policy.
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::Scalar { bits, .. } => validate_octet_width(*bits),
            Self::Product { fields, .. } => {
                validate_unique_names(fields.iter().map(|(name, _)| name))?;
                for (_, layout) in fields {
                    layout.validate()?;
                }
                Ok(())
            }
            Self::Sum {
                tag, alternatives, ..
            } => {
                tag.validate()?;
                validate_unique_names(alternatives.iter().map(|(name, _)| name))?;
                if alternatives.is_empty() {
                    return Err("sum layout requires an alternative");
                }
                for (_, layout) in alternatives {
                    layout.validate()?;
                }
                Ok(())
            }
            Self::Sequence { element, count } => {
                element.validate()?;
                if count.is_none() {
                    return Err("sequence layout requires a finite count policy");
                }
                Ok(())
            }
            Self::Text {
                code_unit_bits,
                length_prefix,
                terminator,
                ..
            } => {
                validate_octet_width(*code_unit_bits)?;
                if !length_prefix && terminator.is_none() {
                    return Err("text layout requires a length or terminator policy");
                }
                Ok(())
            }
        }
    }

    /// Encode one value after checking write authority and layout conformance.
    ///
    /// # Errors
    ///
    /// Returns an error for absent write rights, a mismatched value, overflow,
    /// invalid text, or an invalid layout.
    pub fn write(
        &self,
        value: &LayoutValue,
        rights: AccessRights,
    ) -> Result<Vec<u8>, &'static str> {
        self.validate()?;
        if !rights.write {
            return Err("layout write is not authorized");
        }
        self.encode_value(value)
    }

    /// Decode one complete value after checking read authority.
    ///
    /// # Errors
    ///
    /// Returns an error for absent read rights, malformed bytes, trailing data,
    /// or an invalid layout.
    pub fn read(&self, bytes: &[u8], rights: AccessRights) -> Result<LayoutValue, &'static str> {
        self.validate()?;
        if !rights.read {
            return Err("layout read is not authorized");
        }
        let (value, consumed) = self.decode_value(bytes)?;
        if consumed != bytes.len() {
            return Err("layout value has trailing bytes");
        }
        Ok(value)
    }

    fn encode_value(&self, value: &LayoutValue) -> Result<Vec<u8>, &'static str> {
        match (self, value) {
            (
                Self::Scalar {
                    bits,
                    signed,
                    byte_order,
                },
                LayoutValue::Integer(value),
            ) => encode_fixed_integer(*value, *bits, *signed, *byte_order),
            (Self::Product { fields, packing }, LayoutValue::Product(values)) => {
                if fields.len() != values.len() {
                    return Err("product value has the wrong field count");
                }
                let mut bytes = Vec::new();
                for ((_, layout), value) in fields.iter().zip(values) {
                    if *packing == Packing::Natural {
                        pad_to_alignment(&mut bytes, layout.byte_alignment()?);
                    }
                    bytes.extend(layout.encode_value(value)?);
                }
                Ok(bytes)
            }
            (
                Self::Sum {
                    tag, alternatives, ..
                },
                LayoutValue::Sum { alternative, value },
            ) => {
                let Some((_, payload)) = alternatives.get(*alternative) else {
                    return Err("sum value has an invalid alternative");
                };
                let mut bytes = tag.encode_value(&LayoutValue::Integer(
                    i128::try_from(*alternative).map_err(|_| "sum tag does not fit")?,
                ))?;
                bytes.extend(payload.encode_value(value)?);
                Ok(bytes)
            }
            (Self::Sequence { element, count }, LayoutValue::Sequence(values)) => {
                if usize::try_from(count.unwrap_or_default()).ok() != Some(values.len()) {
                    return Err("sequence value has the wrong entry count");
                }
                let mut bytes = Vec::new();
                for value in values {
                    bytes.extend(element.encode_value(value)?);
                }
                Ok(bytes)
            }
            (
                Self::Text {
                    code_unit_bits: 8,
                    length_prefix,
                    terminator,
                    ..
                },
                LayoutValue::Text(text),
            ) => {
                if text.contains('\0') {
                    return Err("text layout cannot encode U+0000");
                }
                let mut bytes = Vec::new();
                if *length_prefix {
                    encode_uvarint(text.len() as u64, &mut bytes);
                }
                bytes.extend_from_slice(text.as_bytes());
                if let Some(terminator) = terminator {
                    bytes.push(
                        u8::try_from(*terminator).map_err(|_| "terminator is not one octet")?,
                    );
                }
                Ok(bytes)
            }
            (Self::Text { .. }, LayoutValue::Text(_)) => {
                Err("only UTF-8 text layouts are implemented by the shared codec")
            }
            _ => Err("value does not conform to layout"),
        }
    }

    #[allow(clippy::too_many_lines)] // Keep recursive layout cases symmetric and auditable.
    fn decode_value(&self, bytes: &[u8]) -> Result<(LayoutValue, usize), &'static str> {
        match self {
            Self::Scalar {
                bits,
                signed,
                byte_order,
            } => {
                let width = usize::try_from(bits / 8).map_err(|_| "scalar width is too large")?;
                let input = bytes.get(..width).ok_or("premature end of scalar")?;
                Ok((
                    LayoutValue::Integer(decode_fixed_integer(input, *signed, *byte_order)?),
                    width,
                ))
            }
            Self::Product { fields, packing } => {
                let mut consumed = 0;
                let mut values = Vec::new();
                for (_, layout) in fields {
                    if *packing == Packing::Natural {
                        consumed = aligned_offset(consumed, layout.byte_alignment()?);
                        if consumed > bytes.len() {
                            return Err("premature end of product padding");
                        }
                    }
                    let (value, length) = layout.decode_value(&bytes[consumed..])?;
                    consumed += length;
                    values.push(value);
                }
                Ok((LayoutValue::Product(values), consumed))
            }
            Self::Sum {
                tag, alternatives, ..
            } => {
                let (LayoutValue::Integer(tag_value), tag_length) = tag.decode_value(bytes)? else {
                    return Err("sum tag layout is not scalar");
                };
                let alternative = usize::try_from(tag_value).map_err(|_| "invalid sum tag")?;
                let Some((_, payload)) = alternatives.get(alternative) else {
                    return Err("invalid sum tag");
                };
                let (value, payload_length) = payload.decode_value(&bytes[tag_length..])?;
                Ok((
                    LayoutValue::Sum {
                        alternative,
                        value: Box::new(value),
                    },
                    tag_length + payload_length,
                ))
            }
            Self::Sequence { element, count } => {
                let mut consumed = 0;
                let mut values = Vec::new();
                for _ in 0..count.unwrap_or_default() {
                    let (value, length) = element.decode_value(&bytes[consumed..])?;
                    consumed += length;
                    values.push(value);
                }
                Ok((LayoutValue::Sequence(values), consumed))
            }
            Self::Text {
                code_unit_bits: 8,
                length_prefix,
                terminator,
                ..
            } => {
                let (start, length) = if *length_prefix {
                    let (length, consumed) = decode_uvarint(bytes)?;
                    (
                        consumed,
                        usize::try_from(length).map_err(|_| "text length is too large")?,
                    )
                } else {
                    let terminator = u8::try_from(terminator.ok_or("text has no boundary")?)
                        .map_err(|_| "terminator is not one octet")?;
                    (
                        0,
                        bytes
                            .iter()
                            .position(|byte| *byte == terminator)
                            .ok_or("text terminator is absent")?,
                    )
                };
                let text_bytes = bytes
                    .get(start..start + length)
                    .ok_or("premature end of text")?;
                let text = std::str::from_utf8(text_bytes).map_err(|_| "text is not UTF-8")?;
                let terminator_length = usize::from(terminator.is_some());
                if terminator_length == 1
                    && bytes.get(start + length)
                        != terminator
                            .and_then(|value| u8::try_from(value).ok())
                            .as_ref()
                {
                    return Err("text terminator is absent");
                }
                Ok((
                    LayoutValue::Text(text.to_owned()),
                    start + length + terminator_length,
                ))
            }
            Self::Text { .. } => Err("only UTF-8 text layouts are implemented by the shared codec"),
        }
    }

    fn byte_alignment(&self) -> Result<usize, &'static str> {
        match self {
            Self::Scalar { bits, .. } => {
                usize::try_from(bits / 8).map_err(|_| "layout alignment is too large")
            }
            Self::Product { fields, packing } => {
                if *packing == Packing::Packed {
                    Ok(1)
                } else {
                    fields
                        .iter()
                        .map(|(_, layout)| layout.byte_alignment())
                        .try_fold(1, |maximum, alignment| Ok(maximum.max(alignment?)))
                }
            }
            Self::Sum {
                tag, alternatives, ..
            } => alternatives
                .iter()
                .map(|(_, layout)| layout.byte_alignment())
                .try_fold(tag.byte_alignment()?, |maximum, alignment| {
                    Ok(maximum.max(alignment?))
                }),
            Self::Sequence { element, .. } => element.byte_alignment(),
            Self::Text { code_unit_bits, .. } => {
                usize::try_from(code_unit_bits / 8).map_err(|_| "text alignment is too large")
            }
        }
    }
}

fn aligned_offset(offset: usize, alignment: usize) -> usize {
    offset.saturating_add(alignment - 1) / alignment * alignment
}

fn pad_to_alignment(bytes: &mut Vec<u8>, alignment: usize) {
    bytes.resize(aligned_offset(bytes.len(), alignment), 0);
}

fn encode_fixed_integer(
    value: i128,
    bits: u64,
    signed: bool,
    order: ByteOrder,
) -> Result<Vec<u8>, &'static str> {
    let width = usize::try_from(bits / 8).map_err(|_| "scalar width is too large")?;
    if width > 16 {
        return Err("scalar width exceeds the shared integer codec");
    }
    if bits < 128 {
        let (minimum, maximum) = if signed {
            (-(1_i128 << (bits - 1)), (1_i128 << (bits - 1)) - 1)
        } else {
            (0, (1_i128 << bits) - 1)
        };
        if !(minimum..=maximum).contains(&value) {
            return Err("integer does not fit scalar layout");
        }
    } else if !signed && value < 0 {
        return Err("negative integer does not fit unsigned layout");
    }
    let encoded = match order {
        ByteOrder::Little => value.to_le_bytes(),
        ByteOrder::Big => value.to_be_bytes(),
    };
    Ok(match order {
        ByteOrder::Little => encoded[..width].to_vec(),
        ByteOrder::Big => encoded[16 - width..].to_vec(),
    })
}

fn decode_fixed_integer(
    bytes: &[u8],
    signed: bool,
    order: ByteOrder,
) -> Result<i128, &'static str> {
    if bytes.len() > 16 {
        return Err("scalar width exceeds the shared integer codec");
    }
    let negative = signed
        && match order {
            ByteOrder::Little => bytes.last(),
            ByteOrder::Big => bytes.first(),
        }
        .is_some_and(|byte| byte & 0x80 != 0);
    let mut encoded = [if negative { 0xff } else { 0 }; 16];
    match order {
        ByteOrder::Little => encoded[..bytes.len()].copy_from_slice(bytes),
        ByteOrder::Big => encoded[16 - bytes.len()..].copy_from_slice(bytes),
    }
    Ok(match order {
        ByteOrder::Little => i128::from_le_bytes(encoded),
        ByteOrder::Big => i128::from_be_bytes(encoded),
    })
}

fn encode_uvarint(mut value: u64, bytes: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn decode_uvarint(bytes: &[u8]) -> Result<(u64, usize), &'static str> {
    let mut value = 0_u64;
    for (index, byte) in bytes.iter().copied().take(10).enumerate() {
        let shift = index * 7;
        value |= u64::from(byte & 0x7f)
            .checked_shl(u32::try_from(shift).map_err(|_| "invalid varint")?)
            .ok_or("varint overflow")?;
        if byte & 0x80 == 0 {
            if index > 0 && byte == 0 {
                return Err("nonminimal varint");
            }
            return Ok((value, index + 1));
        }
    }
    Err("unterminated or overflowing varint")
}

fn validate_octet_width(bits: u64) -> Result<(), &'static str> {
    if bits == 0 || !bits.is_multiple_of(8) {
        Err("layout width must be a positive multiple of eight")
    } else {
        Ok(())
    }
}

fn validate_unique_names<'a>(
    names: impl IntoIterator<Item = &'a String>,
) -> Result<(), &'static str> {
    let mut unique = BTreeSet::new();
    if names.into_iter().all(|name| unique.insert(name)) {
        Ok(())
    } else {
        Err("layout names must be unique")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleEvidence {
    pub order: Vec<DependencyNode>,
}

impl DependencyGraph {
    pub fn add_node(&mut self, node: DependencyNode) {
        self.nodes.insert(node);
    }

    /// Add one completion dependency.
    ///
    /// # Errors
    ///
    /// Returns an error unless both nodes have been declared.
    pub fn depends_on(
        &mut self,
        prerequisite: &DependencyNode,
        dependent: &DependencyNode,
    ) -> Result<(), &'static str> {
        if !self.nodes.contains(prerequisite) || !self.nodes.contains(dependent) {
            return Err("dependency edge names an unknown node");
        }
        self.edges.insert((prerequisite.clone(), dependent.clone()));
        Ok(())
    }

    /// Produce the canonical source-independent topological schedule.
    ///
    /// # Errors
    ///
    /// Returns an error for a closed internal dependency cycle. A cycle which
    /// contains an explicitly external node is suspension and is retained.
    pub fn schedule(&self) -> Result<ScheduleEvidence, &'static str> {
        let mut remaining = self.nodes.clone();
        let mut edges = self.edges.clone();
        let mut order = Vec::new();
        loop {
            let ready = remaining
                .iter()
                .filter(|node| !edges.iter().any(|(_, dependent)| dependent == *node))
                .cloned()
                .collect::<Vec<_>>();
            if ready.is_empty() {
                break;
            }
            for node in ready {
                remaining.remove(&node);
                edges.retain(|(prerequisite, _)| prerequisite != &node);
                order.push(node);
            }
        }
        if remaining.is_empty() {
            return Ok(ScheduleEvidence { order });
        }
        let suspended_only = remaining.iter().all(|start| {
            let mut pending = vec![start.clone()];
            let mut visited = BTreeSet::new();
            while let Some(current) = pending.pop() {
                if !visited.insert(current.clone()) {
                    continue;
                }
                if matches!(current, DependencyNode::External(_)) {
                    return true;
                }
                for (left, right) in &edges {
                    if left == &current && remaining.contains(right) {
                        pending.push(right.clone());
                    }
                    if right == &current && remaining.contains(left) {
                        pending.push(left.clone());
                    }
                }
            }
            false
        });
        if suspended_only {
            order.extend(remaining);
            return Ok(ScheduleEvidence { order });
        }
        Err("closed internal dependency cycle would deadlock")
    }
}

impl MessageLedger {
    /// Atomically admit one interaction without partially transferring payload.
    ///
    /// # Errors
    ///
    /// Returns an error when bounded rejection applies or contained loss is
    /// requested for a non-event interaction.
    pub fn send(
        &mut self,
        send: MessageSend,
        capacity: usize,
        policy: &AdmissionPolicy,
    ) -> Result<u64, &'static str> {
        let occupied = self.admitted.get(&send.endpoint).copied().unwrap_or(0);
        if occupied >= capacity {
            return match policy {
                AdmissionPolicy::Wait => Err("interaction is waiting for endpoint capacity"),
                AdmissionPolicy::Reject => Err("interaction was rejected before transfer"),
                AdmissionPolicy::ContainedDiagnosticLoss if send.kind == InteractionKind::Event => {
                    Err("contained diagnostic event was not admitted")
                }
                AdmissionPolicy::ContainedDiagnosticLoss => {
                    Err("contained loss is valid only for diagnostic events")
                }
            };
        }
        let identity = self.next_identity;
        self.next_identity += 1;
        *self.admitted.entry(send.endpoint.clone()).or_default() += 1;
        self.transactions.insert(
            identity,
            MessageTransaction {
                identity,
                sender: send.sender,
                receiver: send.receiver,
                endpoint: send.endpoint,
                kind: send.kind,
                payload_identity: send.payload_identity,
                state: TransactionState::Enqueued,
            },
        );
        Ok(identity)
    }

    /// Transfer an admitted interaction to its receiver.
    ///
    /// # Errors
    ///
    /// Returns an error unless the transaction is enqueued.
    pub fn receive(&mut self, identity: u64) -> Result<(), &'static str> {
        let transaction = self
            .transactions
            .get_mut(&identity)
            .ok_or("message transaction does not exist")?;
        if transaction.state != TransactionState::Enqueued {
            return Err("message transaction cannot be received twice");
        }
        transaction.state = match transaction.kind {
            InteractionKind::Stream => TransactionState::Streaming { values: Vec::new() },
            _ => TransactionState::Received,
        };
        Ok(())
    }

    /// Complete exactly one request reply.
    ///
    /// # Errors
    ///
    /// Returns an error for non-requests, duplicate replies, or unknown transactions.
    pub fn reply(
        &mut self,
        identity: u64,
        result_identity: impl Into<String>,
    ) -> Result<(), &'static str> {
        let transaction = self
            .transactions
            .get_mut(&identity)
            .ok_or("message transaction does not exist")?;
        if transaction.kind != InteractionKind::Request
            || transaction.state != TransactionState::Received
        {
            return Err("request is not awaiting exactly one reply");
        }
        transaction.state = TransactionState::Replied {
            result_identity: result_identity.into(),
        };
        Self::release_capacity(&mut self.admitted, &transaction.endpoint);
        Ok(())
    }

    /// Append one ordered stream result.
    ///
    /// # Errors
    ///
    /// Returns an error unless the transaction is an open stream.
    pub fn yield_stream(
        &mut self,
        identity: u64,
        value_identity: impl Into<String>,
    ) -> Result<(), &'static str> {
        let transaction = self
            .transactions
            .get_mut(&identity)
            .ok_or("message transaction does not exist")?;
        let TransactionState::Streaming { values } = &mut transaction.state else {
            return Err("message transaction is not an open stream");
        };
        values.push(value_identity.into());
        Ok(())
    }

    /// Close an event, direct call, or stream and release capacity.
    ///
    /// # Errors
    ///
    /// Returns an error when the transaction cannot close from its current state.
    pub fn close(&mut self, identity: u64) -> Result<(), &'static str> {
        let transaction = self
            .transactions
            .get_mut(&identity)
            .ok_or("message transaction does not exist")?;
        if !matches!(
            transaction.state,
            TransactionState::Received | TransactionState::Streaming { .. }
        ) {
            return Err("message transaction is not ready to close");
        }
        transaction.state = TransactionState::Closed;
        Self::release_capacity(&mut self.admitted, &transaction.endpoint);
        Ok(())
    }

    #[must_use]
    pub fn transaction(&self, identity: u64) -> Option<&MessageTransaction> {
        self.transactions.get(&identity)
    }

    fn release_capacity(admitted: &mut BTreeMap<QualifiedName, usize>, endpoint: &QualifiedName) {
        if let Some(occupied) = admitted.get_mut(endpoint) {
            *occupied = occupied.saturating_sub(1);
        }
    }
}

impl TaskScheduler {
    /// Construct a task in one structured parent scope.
    ///
    /// # Errors
    ///
    /// Returns an error when the parent does not exist or is already closing.
    pub fn construct(
        &mut self,
        parent: Option<TaskIdentity>,
    ) -> Result<TaskIdentity, &'static str> {
        if let Some(parent) = parent {
            let Some(record) = self.tasks.get(&parent) else {
                return Err("task parent does not exist");
            };
            if matches!(record.state, TaskState::Closing | TaskState::Completed) {
                return Err("cannot add a child to a closing task scope");
            }
        }
        let identity = TaskIdentity(self.next_identity);
        self.next_identity += 1;
        self.tasks.insert(
            identity,
            TaskRecord {
                parent,
                children: BTreeSet::new(),
                state: TaskState::Constructed,
                cancellation_requested: false,
            },
        );
        if let Some(parent) = parent {
            let Some(parent_record) = self.tasks.get_mut(&parent) else {
                return Err("task parent disappeared during construction");
            };
            parent_record.children.insert(identity);
        }
        Ok(identity)
    }

    /// Start a newly constructed task.
    ///
    /// # Errors
    ///
    /// Returns an error unless the task is constructed.
    pub fn start(&mut self, task: TaskIdentity) -> Result<(), &'static str> {
        let record = self.tasks.get_mut(&task).ok_or("task does not exist")?;
        if record.state != TaskState::Constructed {
            return Err("only a constructed task may start");
        }
        record.state = TaskState::Runnable;
        Ok(())
    }

    /// Begin structured cancellation for a task and every child.
    ///
    /// # Errors
    ///
    /// Returns an error when the task does not exist.
    pub fn cancel(&mut self, task: TaskIdentity) -> Result<(), &'static str> {
        let children = self
            .tasks
            .get(&task)
            .ok_or("task does not exist")?
            .children
            .iter()
            .copied()
            .collect::<Vec<_>>();
        for child in children {
            self.cancel(child)?;
        }
        let Some(record) = self.tasks.get_mut(&task) else {
            return Err("task disappeared during cancellation");
        };
        record.cancellation_requested = true;
        record.state = TaskState::Closing;
        Ok(())
    }

    /// Begin ordinary scope closure; children retain their own completion path.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown or already completed task.
    pub fn begin_close(&mut self, task: TaskIdentity) -> Result<(), &'static str> {
        let record = self.tasks.get_mut(&task).ok_or("task does not exist")?;
        if record.state == TaskState::Completed {
            return Err("completed task cannot close again");
        }
        record.state = TaskState::Closing;
        Ok(())
    }

    /// Acknowledge closure after every child has completed.
    ///
    /// # Errors
    ///
    /// Returns an error while a child remains incomplete or before closure.
    pub fn acknowledge_closed(&mut self, task: TaskIdentity) -> Result<(), &'static str> {
        let record = self.tasks.get(&task).ok_or("task does not exist")?;
        if record.state != TaskState::Closing {
            return Err("task has not begun closing");
        }
        if record.children.iter().any(|child| {
            !matches!(
                self.tasks.get(child).map(|record| &record.state),
                Some(TaskState::Completed)
            )
        }) {
            return Err("task scope still has a live child");
        }
        let Some(record) = self.tasks.get_mut(&task) else {
            return Err("task disappeared during closure");
        };
        record.state = TaskState::Completed;
        Ok(())
    }

    #[must_use]
    pub fn state(&self, task: TaskIdentity) -> Option<&TaskState> {
        self.tasks.get(&task).map(|record| &record.state)
    }
}

impl MemoryExecution {
    /// Append one validated event and return its stable execution index.
    ///
    /// # Errors
    ///
    /// Returns an error when the event is invalid.
    pub fn record(&mut self, event: MemoryEvent) -> Result<usize, &'static str> {
        event.validate()?;
        let index = self.events.len();
        self.events.push(event);
        Ok(index)
    }

    /// Add a happens-before edge while preserving acyclicity.
    ///
    /// # Errors
    ///
    /// Returns an error for unknown events or an edge that creates a cycle.
    pub fn order_before(&mut self, before: usize, after: usize) -> Result<(), &'static str> {
        if before >= self.events.len() || after >= self.events.len() {
            return Err("memory ordering edge names an unknown event");
        }
        if before == after || self.happens_before(after, before) {
            return Err("memory ordering edge creates a cycle");
        }
        self.order.insert((before, after));
        Ok(())
    }

    #[must_use]
    pub fn happens_before(&self, before: usize, after: usize) -> bool {
        let mut pending = vec![before];
        let mut visited = BTreeSet::new();
        while let Some(current) = pending.pop() {
            if !visited.insert(current) {
                continue;
            }
            for &(_, next) in self.order.iter().filter(|(source, _)| *source == current) {
                if next == after {
                    return true;
                }
                pending.push(next);
            }
        }
        false
    }

    /// Reject unordered conflicting events.
    ///
    /// # Errors
    ///
    /// Returns an error when two aliasing events conflict without ordering.
    pub fn validate_race_free(&self) -> Result<(), &'static str> {
        for left in 0..self.events.len() {
            for right in left + 1..self.events.len() {
                let left_event = &self.events[left];
                let right_event = &self.events[right];
                if left_event.location().aliases(right_event.location())
                    && (left_event.writes() || right_event.writes())
                    && !self.happens_before(left, right)
                    && !self.happens_before(right, left)
                {
                    return Err("unordered conflicting memory events form a data race");
                }
            }
        }
        Ok(())
    }

    /// Select a deterministic coherence order consistent with happens-before.
    ///
    /// # Errors
    ///
    /// Rejects executions containing a race before selecting an order.
    pub fn coherence_order(&self) -> Result<Vec<usize>, &'static str> {
        self.validate_race_free()?;
        let mut remaining = (0..self.events.len()).collect::<BTreeSet<_>>();
        let mut result = Vec::with_capacity(remaining.len());
        while !remaining.is_empty() {
            let next = remaining
                .iter()
                .copied()
                .find(|candidate| {
                    !remaining
                        .iter()
                        .any(|prior| prior != candidate && self.happens_before(*prior, *candidate))
                })
                .ok_or("happens-before relation is cyclic")?;
            remaining.remove(&next);
            result.push(next);
        }
        Ok(result)
    }
}
