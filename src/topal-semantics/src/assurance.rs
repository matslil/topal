use std::collections::{BTreeMap, BTreeSet};

use crate::{LanguageVersion, QualifiedName};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceKind {
    Semantic,
    Implementation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceStatus {
    Verified,
    TrustedUnverified,
    ExternallyAssumed,
    Refuted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceProducer {
    SourceAuthor,
    SourceGenerator,
    Checker,
    Compiler,
    InterpreterRuntime,
    CheckedProvider,
    AnalysisTool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSubject {
    pub identity: QualifiedName,
    pub static_parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    pub kind: EvidenceKind,
    pub property: QualifiedName,
    pub subject: EvidenceSubject,
    pub status: EvidenceStatus,
    pub producer: QualifiedName,
    pub assumptions: BTreeSet<QualifiedName>,
    pub language_revision: LanguageVersion,
    pub architecture_model: Option<QualifiedName>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceObligation {
    OrdinarySemanticLaw,
    ProtectedSafety,
    ImplementationRequirement,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvidencePolicy {
    pub admitted_external_assumptions: BTreeSet<QualifiedName>,
    pub allow_trusted_ordinary_laws: bool,
}

impl EvidenceRecord {
    /// Validate the producer boundary before accepting retained evidence.
    ///
    /// # Errors
    ///
    /// Source, generators, and analysis tools cannot mint evidence. Verified
    /// implementation evidence must come from an implementation-owning actor.
    pub fn validate_producer(&self, actor: EvidenceProducer) -> Result<(), &'static str> {
        if matches!(
            actor,
            EvidenceProducer::SourceAuthor
                | EvidenceProducer::SourceGenerator
                | EvidenceProducer::AnalysisTool
        ) {
            return Err("ordinary source and analysis tools cannot mint evidence");
        }
        if self.kind == EvidenceKind::Implementation
            && self.status == EvidenceStatus::TrustedUnverified
        {
            return Err("implementation evidence cannot be programmer-trusted");
        }
        if self.kind == EvidenceKind::Implementation
            && self.status == EvidenceStatus::Verified
            && !matches!(
                actor,
                EvidenceProducer::Compiler
                    | EvidenceProducer::InterpreterRuntime
                    | EvidenceProducer::CheckedProvider
            )
        {
            return Err("verified implementation evidence requires an implementation producer");
        }
        if self.architecture_model.is_some()
            && !matches!(
                actor,
                EvidenceProducer::Compiler | EvidenceProducer::CheckedProvider
            )
        {
            return Err("architecture evidence requires a checked provider boundary");
        }
        if self.producer.0.is_empty() {
            return Err("evidence requires a producer identity");
        }
        Ok(())
    }

    /// Determine whether this exact record discharges an obligation.
    ///
    /// # Errors
    ///
    /// Mismatched, refuted, unadmitted, or insufficiently trusted records fail.
    pub fn discharge(
        &self,
        property: &QualifiedName,
        subject: &EvidenceSubject,
        obligation: EvidenceObligation,
        policy: &EvidencePolicy,
    ) -> Result<(), &'static str> {
        if &self.property != property || &self.subject != subject {
            return Err("evidence property or subject does not match the obligation");
        }
        match self.status {
            EvidenceStatus::Verified => Ok(()),
            EvidenceStatus::TrustedUnverified
                if obligation == EvidenceObligation::OrdinarySemanticLaw
                    && policy.allow_trusted_ordinary_laws =>
            {
                Ok(())
            }
            EvidenceStatus::ExternallyAssumed
                if obligation == EvidenceObligation::OrdinarySemanticLaw
                    && self
                        .assumptions
                        .is_subset(&policy.admitted_external_assumptions) =>
            {
                Ok(())
            }
            EvidenceStatus::Refuted => Err("refuted evidence cannot discharge an obligation"),
            EvidenceStatus::ExternallyAssumed => {
                Err("external evidence has assumptions not admitted by the application")
            }
            EvidenceStatus::TrustedUnverified => {
                Err("unverified evidence cannot discharge this obligation")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractProof {
    Verified,
    Unknown,
    Refuted,
    TrustedUnverified,
}

/// Enforce a caller, return, or invariant proof without introducing a hidden
/// runtime failure.
///
/// # Errors
///
/// Anything other than a verified proof fails before the protected transition.
pub const fn require_contract_proof(proof: ContractProof) -> Result<(), &'static str> {
    match proof {
        ContractProof::Verified => Ok(()),
        ContractProof::Unknown => Err("contract obligation is not proven"),
        ContractProof::Refuted => Err("contract obligation is refuted"),
        ContractProof::TrustedUnverified => {
            Err("unverified trust cannot discharge a contract obligation")
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceDimension {
    Work,
    Span,
    AllocationTotal,
    PeakLive,
    Retained,
    Stack,
    QueueEntries,
    TransferCount,
    CodeSize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Bound {
    Exact(u128),
    Symbolic(String),
    Unknown,
}

impl Bound {
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Exact(left), Self::Exact(right)) => {
                left.checked_add(*right).map_or(Self::Unknown, Self::Exact)
            }
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Symbolic(left), Self::Symbolic(right)) => {
                Self::Symbolic(format!("({left}) + ({right})"))
            }
            (Self::Symbolic(expression), Self::Exact(0))
            | (Self::Exact(0), Self::Symbolic(expression)) => Self::Symbolic(expression.clone()),
            (Self::Symbolic(expression), Self::Exact(value))
            | (Self::Exact(value), Self::Symbolic(expression)) => {
                Self::Symbolic(format!("({expression}) + {value}"))
            }
        }
    }

    #[must_use]
    pub fn maximum(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Exact(left), Self::Exact(right)) => Self::Exact((*left).max(*right)),
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Symbolic(left), Self::Symbolic(right)) if left == right => self.clone(),
            (Self::Symbolic(left), Self::Symbolic(right)) => {
                Self::Symbolic(format!("max ({left}) ({right})"))
            }
            (Self::Symbolic(expression), Self::Exact(value))
            | (Self::Exact(value), Self::Symbolic(expression)) => {
                Self::Symbolic(format!("max ({expression}) {value}"))
            }
        }
    }

    #[must_use]
    pub const fn satisfies(&self, required: u128) -> bool {
        matches!(self, Self::Exact(value) if *value <= required)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceBound {
    pub dimension: ResourceDimension,
    pub scope: QualifiedName,
    pub bound: Bound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Composition {
    Sequential,
    Independent,
    Alternative,
}

impl ResourceBound {
    /// Conservatively compose two facts in the same dimension and scope.
    ///
    /// # Errors
    ///
    /// Facts from different dimensions or scopes cannot be silently combined.
    pub fn compose(&self, other: &Self, mode: Composition) -> Result<Self, &'static str> {
        if self.dimension != other.dimension || self.scope != other.scope {
            return Err("resource bounds require the same dimension and scope");
        }
        let additive = match mode {
            Composition::Alternative => false,
            Composition::Independent => matches!(
                self.dimension,
                ResourceDimension::Work
                    | ResourceDimension::AllocationTotal
                    | ResourceDimension::TransferCount
                    | ResourceDimension::CodeSize
            ),
            Composition::Sequential => !matches!(
                self.dimension,
                ResourceDimension::PeakLive | ResourceDimension::Retained
            ),
        };
        Ok(Self {
            dimension: self.dimension,
            scope: self.scope.clone(),
            bound: if additive {
                self.bound.add(&other.bound)
            } else {
                self.bound.maximum(&other.bound)
            },
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgressClass {
    MayBlock,
    ObstructionFree,
    LockFree,
    WaitFree { maximum_own_steps: Bound },
}

impl ProgressClass {
    #[must_use]
    pub fn satisfies(&self, required: &Self) -> bool {
        match (self, required) {
            (_, Self::MayBlock)
            | (
                Self::ObstructionFree | Self::LockFree | Self::WaitFree { .. },
                Self::ObstructionFree,
            )
            | (Self::LockFree | Self::WaitFree { .. }, Self::LockFree) => true,
            (
                Self::WaitFree {
                    maximum_own_steps: actual,
                },
                Self::WaitFree {
                    maximum_own_steps: Bound::Exact(required),
                },
            ) => actual.satisfies(*required),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StorageIdentity(pub u64);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExclusivityTracker {
    aliases: BTreeMap<StorageIdentity, BTreeSet<String>>,
    consumed: BTreeSet<StorageIdentity>,
}

impl ExclusivityTracker {
    pub fn bind(&mut self, storage: StorageIdentity, binding: impl Into<String>) {
        self.aliases
            .entry(storage)
            .or_default()
            .insert(binding.into());
    }

    pub fn end_use(&mut self, storage: StorageIdentity, binding: &str) {
        if let Some(aliases) = self.aliases.get_mut(&storage) {
            aliases.remove(binding);
        }
    }

    #[must_use]
    pub fn is_exclusive(&self, storage: StorageIdentity, binding: &str) -> bool {
        !self.consumed.contains(&storage)
            && self
                .aliases
                .get(&storage)
                .is_some_and(|aliases| aliases.len() == 1 && aliases.contains(binding))
    }

    /// Consume the sole live binding.
    ///
    /// # Errors
    ///
    /// Consumption rejects aliases and repeated consumption.
    pub fn consume(&mut self, storage: StorageIdentity, binding: &str) -> Result<(), &'static str> {
        if !self.is_exclusive(storage, binding) {
            return Err("consumption requires the sole live binding");
        }
        self.aliases.remove(&storage);
        self.consumed.insert(storage);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegionIdentity(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionEscape {
    MoveRegion,
    Promote,
    Copy,
    StorageIndependent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegionState {
    capacity: u128,
    allocated: u128,
    dependencies: BTreeSet<StorageIdentity>,
    open: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RegionTracker {
    regions: BTreeMap<RegionIdentity, RegionState>,
}

impl RegionTracker {
    /// Open one named region.
    ///
    /// # Errors
    ///
    /// A live identity cannot be reopened.
    pub fn open(&mut self, region: RegionIdentity, capacity: u128) -> Result<(), &'static str> {
        if self.regions.get(&region).is_some_and(|state| state.open) {
            return Err("allocation region is already live");
        }
        self.regions.insert(
            region,
            RegionState {
                capacity,
                allocated: 0,
                dependencies: BTreeSet::new(),
                open: true,
            },
        );
        Ok(())
    }

    /// Record allocation and its hidden storage dependency.
    ///
    /// # Errors
    ///
    /// Closed, unknown, overflowing, or exhausted regions reject allocation.
    pub fn allocate(
        &mut self,
        region: RegionIdentity,
        storage: StorageIdentity,
        amount: u128,
    ) -> Result<(), &'static str> {
        let state = self
            .regions
            .get_mut(&region)
            .ok_or("unknown allocation region")?;
        if !state.open {
            return Err("allocation region is closed");
        }
        let next = state
            .allocated
            .checked_add(amount)
            .ok_or("allocation amount overflows")?;
        if next > state.capacity {
            return Err("allocation region capacity is exhausted");
        }
        state.allocated = next;
        state.dependencies.insert(storage);
        Ok(())
    }

    /// Validate an attempted escape.
    ///
    /// # Errors
    ///
    /// A dependent value needs one of the four checked escape forms.
    pub fn escape(
        &mut self,
        region: RegionIdentity,
        storage: StorageIdentity,
        evidence: Option<RegionEscape>,
    ) -> Result<(), &'static str> {
        let state = self
            .regions
            .get_mut(&region)
            .ok_or("unknown allocation region")?;
        if !state.dependencies.contains(&storage) {
            return Ok(());
        }
        match evidence {
            Some(RegionEscape::MoveRegion) => {
                state.open = false;
                Ok(())
            }
            Some(RegionEscape::Promote | RegionEscape::Copy | RegionEscape::StorageIndependent) => {
                state.dependencies.remove(&storage);
                Ok(())
            }
            None => Err("region-dependent value cannot escape without evidence"),
        }
    }

    /// Close and bulk-release a region.
    ///
    /// # Errors
    ///
    /// Live dependent values prevent cleanup.
    pub fn close(&mut self, region: RegionIdentity) -> Result<(), &'static str> {
        let state = self
            .regions
            .get_mut(&region)
            .ok_or("unknown allocation region")?;
        if !state.dependencies.is_empty() {
            return Err("allocation region still has dependent values");
        }
        if !state.open {
            return Err("allocation region is already closed or moved");
        }
        state.open = false;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlanDecision {
    Fuse {
        producer: String,
        consumer: String,
    },
    Materialize {
        value: String,
    },
    Reuse {
        storage: String,
    },
    Channel {
        endpoint: String,
        implementation: String,
    },
    Layout {
        value: String,
        layout: String,
    },
    Transfer {
        boundary: String,
    },
    Specialize {
        input: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplementationPlan {
    pub language_revision: LanguageVersion,
    pub source_artifact: String,
    pub cost_model: String,
    pub architecture_model: Option<String>,
    decisions: BTreeSet<PlanDecision>,
}

impl ImplementationPlan {
    /// Construct a compiler-owned plan.
    ///
    /// # Errors
    ///
    /// No other actor can create or mutate the plan.
    pub fn new(
        actor: EvidenceProducer,
        language_revision: LanguageVersion,
        source_artifact: impl Into<String>,
        cost_model: impl Into<String>,
        architecture_model: Option<String>,
    ) -> Result<Self, &'static str> {
        if actor != EvidenceProducer::Compiler {
            return Err("only the compiler may create an implementation plan");
        }
        Ok(Self {
            language_revision,
            source_artifact: source_artifact.into(),
            cost_model: cost_model.into(),
            architecture_model,
            decisions: BTreeSet::new(),
        })
    }

    /// Add one typed plan decision.
    ///
    /// # Errors
    ///
    /// Plan mutation remains compiler-only.
    pub fn insert(
        &mut self,
        actor: EvidenceProducer,
        decision: PlanDecision,
    ) -> Result<(), &'static str> {
        if actor != EvidenceProducer::Compiler {
            return Err("only the compiler may mutate an implementation plan");
        }
        self.decisions.insert(decision);
        Ok(())
    }

    #[must_use]
    pub fn diagnostic_projection(&self) -> Vec<String> {
        self.decisions
            .iter()
            .map(|decision| format!("{decision:?}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(parts: &[&str]) -> QualifiedName {
        QualifiedName(parts.iter().map(|part| (*part).to_owned()).collect())
    }

    #[test]
    fn evidence_authority_and_assumptions_fail_closed() {
        let subject = EvidenceSubject {
            identity: name(&["service", "commit"]),
            static_parameters: BTreeMap::new(),
        };
        let assumption = name(&["deployment", "stable-storage"]);
        let record = EvidenceRecord {
            kind: EvidenceKind::Semantic,
            property: name(&["Durable"]),
            subject: subject.clone(),
            status: EvidenceStatus::ExternallyAssumed,
            producer: name(&["store", "provider"]),
            assumptions: BTreeSet::from([assumption.clone()]),
            language_revision: LanguageVersion::DESIGN_1,
            architecture_model: None,
        };
        assert!(
            record
                .validate_producer(EvidenceProducer::SourceAuthor)
                .is_err()
        );
        assert!(
            record
                .validate_producer(EvidenceProducer::CheckedProvider)
                .is_ok()
        );
        let policy = EvidencePolicy::default();
        assert!(
            record
                .discharge(
                    &name(&["Durable"]),
                    &subject,
                    EvidenceObligation::OrdinarySemanticLaw,
                    &policy,
                )
                .is_err()
        );
        let admitted = EvidencePolicy {
            admitted_external_assumptions: BTreeSet::from([assumption]),
            ..EvidencePolicy::default()
        };
        assert!(
            record
                .discharge(
                    &name(&["Durable"]),
                    &subject,
                    EvidenceObligation::ProtectedSafety,
                    &admitted,
                )
                .is_err()
        );
        assert!(
            record
                .discharge(
                    &name(&["Durable"]),
                    &subject,
                    EvidenceObligation::OrdinarySemanticLaw,
                    &admitted,
                )
                .is_ok()
        );
    }

    #[test]
    fn resource_composition_is_dimension_specific_and_unknown_fails() {
        let work = ResourceBound {
            dimension: ResourceDimension::Work,
            scope: name(&["invocation"]),
            bound: Bound::Exact(5),
        };
        let combined = work.compose(&work, Composition::Sequential).unwrap();
        assert_eq!(combined.bound, Bound::Exact(10));
        let live = ResourceBound {
            dimension: ResourceDimension::PeakLive,
            scope: name(&["invocation"]),
            bound: Bound::Exact(5),
        };
        assert_eq!(
            live.compose(&live, Composition::Sequential).unwrap().bound,
            Bound::Exact(5)
        );
        assert!(!Bound::Unknown.satisfies(u128::MAX));
    }

    #[test]
    fn exclusivity_and_regions_reject_aliases_and_escapes() {
        let storage = StorageIdentity(1);
        let mut exclusive = ExclusivityTracker::default();
        exclusive.bind(storage, "left");
        exclusive.bind(storage, "right");
        assert!(exclusive.consume(storage, "left").is_err());
        exclusive.end_use(storage, "right");
        assert!(exclusive.consume(storage, "left").is_ok());

        let mut regions = RegionTracker::default();
        regions.open(RegionIdentity(1), 64).unwrap();
        regions.allocate(RegionIdentity(1), storage, 32).unwrap();
        assert!(regions.close(RegionIdentity(1)).is_err());
        assert!(regions.escape(RegionIdentity(1), storage, None).is_err());
        regions
            .escape(RegionIdentity(1), storage, Some(RegionEscape::Promote))
            .unwrap();
        assert!(regions.close(RegionIdentity(1)).is_ok());
    }

    #[test]
    fn implementation_plan_is_compiler_owned_and_canonical() {
        assert!(
            ImplementationPlan::new(
                EvidenceProducer::AnalysisTool,
                LanguageVersion::DESIGN_1,
                "source",
                "cost",
                None,
            )
            .is_err()
        );
        let mut plan = ImplementationPlan::new(
            EvidenceProducer::Compiler,
            LanguageVersion::DESIGN_1,
            "source",
            "cost",
            None,
        )
        .unwrap();
        plan.insert(
            EvidenceProducer::Compiler,
            PlanDecision::Specialize { input: "T".into() },
        )
        .unwrap();
        assert_eq!(plan.diagnostic_projection().len(), 1);
        assert!(
            plan.insert(
                EvidenceProducer::AnalysisTool,
                PlanDecision::Materialize { value: "x".into() },
            )
            .is_err()
        );
    }
}
