//! Shared, deterministic semantic identities for every Topal source tool.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

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
}

impl ResourceTracker {
    /// Add one fresh ownership obligation.
    ///
    /// # Errors
    ///
    /// Returns an error when the resource identity already has an obligation.
    pub fn declare(
        &mut self,
        identity: ResourceIdentity,
        binding: impl Into<String>,
        lifetime: u64,
    ) -> Result<(), &'static str> {
        if self.resources.contains_key(&identity) {
            return Err("resource identity already has an ownership obligation");
        }
        self.declaration_order.push(identity.clone());
        self.resources.insert(
            identity,
            ResourceState::Owned {
                binding: binding.into(),
                lifetime,
            },
        );
        Ok(())
    }

    /// Move ownership to a new binding without duplicating the obligation.
    ///
    /// # Errors
    ///
    /// Returns an error after destruction or from a binding that is not the owner.
    pub fn move_to(
        &mut self,
        identity: &ResourceIdentity,
        from: &str,
        to: impl Into<String>,
    ) -> Result<(), &'static str> {
        let Some(ResourceState::Owned { binding, .. }) = self.resources.get_mut(identity) else {
            return Err("resource is not live");
        };
        if binding != from {
            return Err("move source does not own the resource");
        }
        *binding = to.into();
        Ok(())
    }

    /// Destroy live resources in reverse declaration order.
    #[must_use]
    pub fn destroy_all(&mut self) -> Vec<ResourceIdentity> {
        let mut destroyed = Vec::new();
        for identity in self.declaration_order.iter().rev() {
            if matches!(
                self.resources.get(identity),
                Some(ResourceState::Owned { .. })
            ) {
                self.resources
                    .insert(identity.clone(), ResourceState::Destroyed);
                destroyed.push(identity.clone());
            }
        }
        destroyed
    }

    #[must_use]
    pub fn state(&self, identity: &ResourceIdentity) -> Option<&ResourceState> {
        self.resources.get(identity)
    }
}

impl EffectRow {
    #[must_use]
    pub fn compose(&self, other: &Self) -> Option<Self> {
        let tail = match (&self.tail, &other.tail) {
            (Some(left), Some(right)) if left != right => return None,
            (Some(tail), _) | (_, Some(tail)) => Some(tail.clone()),
            (None, None) => None,
        };
        Some(Self {
            known: self.known.union(&other.known),
            tail,
        })
    }

    #[must_use]
    pub fn is_contained_by(&self, allowed: &Self) -> bool {
        allowed.known.contains_all(&self.known)
            && (self.tail.is_none() || self.tail == allowed.tail)
    }
}

/// Stable relationship evidence retained by a typing derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relationship {
    pub name: QualifiedName,
    pub subjects: Vec<SemanticIdentity>,
}

/// Identity of an exact semantic object participating in relationships.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticIdentity {
    Declaration(DeclarationIdentity),
    Type(TypeIdentity),
    Value(String),
}

/// The reusable result of semantic analysis under `TOPAL-TYPE-JUDGE-001`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedObject {
    pub kind: ObjectKind,
    pub classifier: Option<TypeIdentity>,
    pub effects: EffectSet,
    pub relationships: Vec<Relationship>,
}

/// One declaration candidate in deterministic source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration<T> {
    pub identity: DeclarationIdentity,
    pub value: T,
}

/// Lexical declarations grouped by name while retaining source order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticScope<T> {
    declarations: BTreeMap<String, Vec<Declaration<T>>>,
}

impl<T> SemanticScope<T> {
    pub fn declare(&mut self, name: impl Into<String>, declaration: Declaration<T>) {
        self.declarations
            .entry(name.into())
            .or_default()
            .push(declaration);
    }

    #[must_use]
    pub fn candidates(&self, name: &str) -> &[Declaration<T>] {
        self.declarations.get(name).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(name: &str, ordinal: usize) -> DeclarationIdentity {
        DeclarationIdentity {
            module: "example".into(),
            name: name.into(),
            ordinal,
        }
    }

    #[test]
    fn predicate_is_the_only_subkind() {
        assert!(ObjectKind::Predicate.satisfies(ObjectKind::Function));
        assert!(!ObjectKind::Type.satisfies(ObjectKind::Value));
        assert!(!ObjectKind::Function.satisfies(ObjectKind::Predicate));
    }

    #[test]
    fn nominal_and_structural_identity_remain_distinct() {
        let shape =
            TypeIdentity::Structural(StructuralType::Tuple(vec![TypeIdentity::Fundamental(
                "Int",
            )]));
        let nominal = TypeIdentity::Nominal {
            declaration: declaration("Coordinate", 0),
            parameters: vec![shape.clone()],
        };
        assert_ne!(shape, nominal);
    }

    #[test]
    fn effect_union_is_canonical_and_idempotent() {
        let io = QualifiedName(vec!["lang".into(), "io".into()]);
        let clock = QualifiedName(vec!["lang".into(), "clock".into()]);
        let left = EffectSet::from_effects([io.clone(), clock.clone()]);
        let right = EffectSet::from_effects([io]);
        assert_eq!(left.union(&right), left);
        assert_eq!(left.iter().count(), 2);
    }

    #[test]
    fn declarations_retain_source_order() {
        let mut scope = SemanticScope::default();
        scope.declare(
            "choose",
            Declaration {
                identity: declaration("choose", 0),
                value: "first",
            },
        );
        scope.declare(
            "choose",
            Declaration {
                identity: declaration("choose", 1),
                value: "second",
            },
        );
        assert_eq!(scope.candidates("choose")[0].value, "first");
    }

    #[test]
    fn language_versions_expand_and_order_canonically() {
        assert_eq!("v0.1".parse(), Ok(LanguageVersion::DESIGN_0));
        assert_eq!(LanguageVersion::DESIGN_0.to_string(), "v0.1");
        assert!("v0.2".parse::<LanguageVersion>().unwrap() > LanguageVersion::DESIGN_0);
    }

    #[test]
    fn generic_patterns_preserve_exact_substitutions() {
        let pattern = TypePattern::Record(vec![
            ("first".into(), TypePattern::Parameter("T".into())),
            ("second".into(), TypePattern::Parameter("T".into())),
        ]);
        let nominal = TypeIdentity::Nominal {
            declaration: declaration("UserId", 2),
            parameters: Vec::new(),
        };
        let arguments = BTreeMap::from([("T".into(), nominal.clone())]);
        assert_eq!(
            pattern.instantiate(&arguments),
            Some(TypeIdentity::Structural(StructuralType::Record(vec![
                ("first".into(), nominal.clone()),
                ("second".into(), nominal),
            ])))
        );
        assert!(pattern.instantiate(&BTreeMap::new()).is_none());
    }

    #[test]
    fn capability_evidence_is_coherent_per_exact_subject() {
        let subject = TypeIdentity::Fundamental("Int");
        let capability = QualifiedName(vec!["lang".into(), "Equality".into()]);
        let evidence = CapabilityEvidence {
            capability: capability.clone(),
            subject: subject.clone(),
            roles: BTreeMap::from([("equal".into(), declaration("equal-int", 0))]),
        };
        let mut set = CapabilitySet::default();
        assert_eq!(set.insert(evidence.clone()), Ok(()));
        assert_eq!(set.insert(evidence.clone()), Ok(()));
        assert_eq!(set.select(&capability, &subject), Some(&evidence));

        let conflicting = CapabilityEvidence {
            roles: BTreeMap::from([("equal".into(), declaration("other-equal", 1))]),
            ..evidence
        };
        assert!(set.insert(conflicting).is_err());
    }

    #[test]
    fn interface_implementation_requires_exact_roles() {
        let shape = InterfaceShape {
            identity: declaration("Parser", 0),
            operations: BTreeMap::from([(
                "parse".into(),
                InterfaceOperation::Function {
                    inputs: vec![TypeIdentity::Fundamental("String")],
                    result: TypeIdentity::Fundamental("Boolean"),
                },
            )]),
        };
        let complete = InterfaceImplementation {
            interface: shape.clone(),
            operations: BTreeMap::from([("parse".into(), declaration("parse", 1))]),
        };
        assert_eq!(complete.validate(), Ok(()));
        assert!(
            InterfaceImplementation {
                interface: shape,
                operations: BTreeMap::new(),
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn effect_rows_compose_and_check_containment_canonically() {
        let read = QualifiedName(vec!["app".into(), "read".into()]);
        let write = QualifiedName(vec!["app".into(), "write".into()]);
        let left = EffectRow {
            known: EffectSet::from_effects([read.clone()]),
            tail: Some("E".into()),
        };
        let right = EffectRow {
            known: EffectSet::from_effects([write.clone()]),
            tail: Some("E".into()),
        };
        let composed = left.compose(&right).unwrap();
        assert_eq!(composed.known.iter().count(), 2);
        assert!(left.is_contained_by(&composed));
        assert!(right.is_contained_by(&composed));
        assert!(
            left.compose(&EffectRow {
                known: EffectSet::default(),
                tail: Some("Other".into()),
            })
            .is_none()
        );
    }

    #[test]
    fn resource_moves_retain_one_obligation_and_cleanup_once() {
        let first = ResourceIdentity(QualifiedName(vec!["resource".into(), "first".into()]));
        let second = ResourceIdentity(QualifiedName(vec!["resource".into(), "second".into()]));
        let mut tracker = ResourceTracker::default();
        tracker.declare(first.clone(), "input", 1).unwrap();
        tracker.declare(second.clone(), "other", 1).unwrap();
        tracker.move_to(&first, "input", "output").unwrap();
        assert!(tracker.move_to(&first, "input", "again").is_err());
        assert!(matches!(
            tracker.state(&first),
            Some(ResourceState::Owned { binding, .. }) if binding == "output"
        ));
        assert_eq!(tracker.destroy_all(), vec![second.clone(), first.clone()]);
        assert!(tracker.destroy_all().is_empty());
        assert_eq!(tracker.state(&first), Some(&ResourceState::Destroyed));
    }

    #[test]
    fn locations_separate_resources_and_validate_events() {
        let resource = ResourceIdentity(QualifiedName(vec!["memory".into(), "buffer".into()]));
        let location = Location {
            resource: resource.clone(),
            base: 16,
            size: 8,
            layout_size: 8,
            alignment: 8,
            rights: AccessRights {
                read: true,
                write: false,
            },
            lifetime: 1,
            access_widths: BTreeSet::from([1, 2, 4, 8]),
        };
        assert_eq!(location.validate(), Ok(()));
        assert_eq!(
            MemoryEvent::Read {
                location: location.clone(),
                width: 4,
            }
            .validate(),
            Ok(())
        );
        assert!(
            MemoryEvent::Write {
                location: location.clone(),
                width: 4,
                value_identity: "value".into(),
            }
            .validate()
            .is_err()
        );
        let overlapping_other_resource = Location {
            resource: ResourceIdentity(QualifiedName(vec!["memory".into(), "other".into()])),
            ..location.clone()
        };
        assert!(!location.aliases(&overlapping_other_resource));
        assert!(location.aliases(&Location {
            base: 20,
            size: 4,
            layout_size: 4,
            alignment: 4,
            ..location.clone()
        }));
    }

    #[test]
    fn memory_execution_requires_order_for_conflicts() {
        let location = Location {
            resource: ResourceIdentity(QualifiedName(vec!["memory".into(), "shared".into()])),
            base: 0,
            size: 4,
            layout_size: 4,
            alignment: 4,
            rights: AccessRights {
                read: true,
                write: true,
            },
            lifetime: 1,
            access_widths: BTreeSet::from([4]),
        };
        let mut execution = MemoryExecution::default();
        let write = execution
            .record(MemoryEvent::Write {
                location: location.clone(),
                width: 4,
                value_identity: "one".into(),
            })
            .unwrap();
        let read = execution
            .record(MemoryEvent::Read { location, width: 4 })
            .unwrap();
        assert!(execution.validate_race_free().is_err());
        execution.order_before(write, read).unwrap();
        assert_eq!(execution.validate_race_free(), Ok(()));
        assert!(execution.order_before(read, write).is_err());
    }

    #[test]
    fn task_cancellation_closes_children_before_parent() {
        let mut scheduler = TaskScheduler::default();
        let parent = scheduler.construct(None).unwrap();
        scheduler.start(parent).unwrap();
        let child = scheduler.construct(Some(parent)).unwrap();
        scheduler.start(child).unwrap();
        scheduler.cancel(parent).unwrap();
        assert_eq!(scheduler.state(child), Some(&TaskState::Closing));
        assert!(scheduler.acknowledge_closed(parent).is_err());
        scheduler.acknowledge_closed(child).unwrap();
        scheduler.acknowledge_closed(parent).unwrap();
        assert_eq!(scheduler.state(parent), Some(&TaskState::Completed));
    }

    #[test]
    fn requests_reply_once_and_streams_preserve_order() {
        let endpoint = QualifiedName(vec!["service".into(), "query".into()]);
        let mut ledger = MessageLedger::default();
        let request = ledger
            .send(
                MessageSend {
                    sender: TaskIdentity(1),
                    receiver: TaskIdentity(2),
                    endpoint: endpoint.clone(),
                    kind: InteractionKind::Request,
                    payload_identity: "question".into(),
                },
                1,
                &AdmissionPolicy::Reject,
            )
            .unwrap();
        assert!(
            ledger
                .send(
                    MessageSend {
                        sender: TaskIdentity(1),
                        receiver: TaskIdentity(2),
                        endpoint: endpoint.clone(),
                        kind: InteractionKind::Request,
                        payload_identity: "second".into(),
                    },
                    1,
                    &AdmissionPolicy::Reject,
                )
                .is_err()
        );
        ledger.receive(request).unwrap();
        ledger.reply(request, "answer").unwrap();
        assert!(ledger.reply(request, "duplicate").is_err());

        let stream = ledger
            .send(
                MessageSend {
                    sender: TaskIdentity(1),
                    receiver: TaskIdentity(2),
                    endpoint,
                    kind: InteractionKind::Stream,
                    payload_identity: "range".into(),
                },
                1,
                &AdmissionPolicy::Wait,
            )
            .unwrap();
        ledger.receive(stream).unwrap();
        ledger.yield_stream(stream, "one").unwrap();
        ledger.yield_stream(stream, "two").unwrap();
        assert!(matches!(
            &ledger.transaction(stream).unwrap().state,
            TransactionState::Streaming { values } if values == &["one", "two"]
        ));
        ledger.close(stream).unwrap();
    }
}
