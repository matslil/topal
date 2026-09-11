use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::rc::Rc;

use crate::{ProgressClass, QualifiedName};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderingPolicy {
    Ordered,
    Unordered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionPolicyV2 {
    Block,
    Reject,
    Drop { policy: QualifiedName },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionPolicy {
    pub capacity: usize,
    pub ordering: OrderingPolicy,
    pub admission: AdmissionPolicyV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointTopology {
    pub producers: usize,
    pub consumers: usize,
    pub fixed_layout: bool,
    pub dynamically_resized: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelImplementation {
    SerializedQueue,
    SpscRing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelSelection {
    pub implementation: ChannelImplementation,
    pub progress: ProgressClass,
}

/// Select a reference or certified SPSC implementation from derived topology.
///
/// # Errors
///
/// A hard progress class fails when the topology and backend certificate do not
/// provide it.
pub fn select_channel(
    topology: &EndpointTopology,
    hard_progress: Option<&ProgressClass>,
    backend_has_certified_spsc: bool,
) -> Result<ChannelSelection, &'static str> {
    let spsc = topology.producers == 1
        && topology.consumers == 1
        && topology.fixed_layout
        && !topology.dynamically_resized
        && backend_has_certified_spsc;
    let selection = if spsc {
        ChannelSelection {
            implementation: ChannelImplementation::SpscRing,
            progress: ProgressClass::LockFree,
        }
    } else {
        ChannelSelection {
            implementation: ChannelImplementation::SerializedQueue,
            progress: ProgressClass::MayBlock,
        }
    };
    if hard_progress.is_some_and(|required| !selection.progress.satisfies(required)) {
        Err("selected channel cannot prove the hard progress requirement")
    } else {
        Ok(selection)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPressure {
    Block,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPublishError {
    WouldBlock,
    Capacity,
}

#[derive(Clone, Debug)]
struct SnapshotVersion<T> {
    version: u64,
    value: Rc<T>,
}

#[derive(Clone, Debug)]
pub struct SnapshotView<T> {
    pub version: u64,
    value: Rc<T>,
}

impl<T> SnapshotView<T> {
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct PublishedSnapshot<T> {
    retained_versions: usize,
    pressure: SnapshotPressure,
    versions: VecDeque<SnapshotVersion<T>>,
    next_version: u64,
}

impl<T> PublishedSnapshot<T> {
    /// Construct a versioned snapshot resource.
    ///
    /// # Errors
    ///
    /// At least one version must be retainable.
    pub fn new(
        initial: T,
        retained_versions: usize,
        pressure: SnapshotPressure,
    ) -> Result<Self, &'static str> {
        if retained_versions == 0 {
            return Err("a published snapshot must retain at least one version");
        }
        Ok(Self {
            retained_versions,
            pressure,
            versions: VecDeque::from([SnapshotVersion {
                version: 0,
                value: Rc::new(initial),
            }]),
            next_version: 1,
        })
    }

    /// Observe the currently published complete version.
    ///
    /// # Panics
    ///
    /// Panics only if the private invariant established by `new` and preserved
    /// by `publish` is broken and no version remains.
    #[must_use]
    pub fn observe(&self) -> SnapshotView<T> {
        let current = self.versions.back().expect("snapshot always has a version");
        SnapshotView {
            version: current.version,
            value: Rc::clone(&current.value),
        }
    }

    /// Publish a complete new version at one reference-model protocol point.
    ///
    /// # Errors
    ///
    /// Retention pressure follows the declared blocking or rejection policy.
    ///
    /// # Panics
    ///
    /// Panics only if the private nonempty-version invariant is broken.
    pub fn publish(&mut self, value: T) -> Result<u64, SnapshotPublishError> {
        while self.versions.len() >= self.retained_versions {
            let oldest = self
                .versions
                .front()
                .expect("snapshot always has a version");
            if Rc::strong_count(&oldest.value) == 1 {
                self.versions.pop_front();
            } else {
                return Err(match self.pressure {
                    SnapshotPressure::Block => SnapshotPublishError::WouldBlock,
                    SnapshotPressure::Reject => SnapshotPublishError::Capacity,
                });
            }
        }
        let version = self.next_version;
        self.next_version = self.next_version.saturating_add(1);
        self.versions.push_back(SnapshotVersion {
            version,
            value: Rc::new(value),
        });
        Ok(version)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionView<S> {
    pub version: u64,
    state: Rc<S>,
}

impl<S> TransactionView<S> {
    #[must_use]
    pub fn state(&self) -> &S {
        &self.state
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionDecision<V, S, E> {
    Commit { value: V, state: S },
    Abort(E),
    RetryWhenChanged(BTreeSet<QualifiedName>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionOutcome<V, E> {
    Committed { value: V, version: u64 },
    Conflict { observed_version: u64 },
    Aborted(E),
    RetryWhenChanged(BTreeSet<QualifiedName>),
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionDomain<S> {
    state: Rc<S>,
    version: u64,
}

impl<S: Clone> TransactionDomain<S> {
    #[must_use]
    pub fn new(initial_state: S) -> Self {
        Self {
            state: Rc::new(initial_state),
            version: 0,
        }
    }

    #[must_use]
    pub fn view(&self) -> TransactionView<S> {
        TransactionView {
            version: self.version,
            state: Rc::clone(&self.state),
        }
    }

    /// Apply a decision derived from one immutable view.
    pub fn commit<V, E>(
        &mut self,
        view: &TransactionView<S>,
        decision: TransactionDecision<V, S, E>,
        cancellation_requested: bool,
    ) -> TransactionOutcome<V, E> {
        if cancellation_requested {
            return TransactionOutcome::Cancelled;
        }
        match decision {
            TransactionDecision::Abort(error) => TransactionOutcome::Aborted(error),
            TransactionDecision::RetryWhenChanged(observed) => {
                TransactionOutcome::RetryWhenChanged(observed)
            }
            TransactionDecision::Commit { .. } if view.version != self.version => {
                TransactionOutcome::Conflict {
                    observed_version: self.version,
                }
            }
            TransactionDecision::Commit { value, state } => {
                self.state = Rc::new(state);
                self.version = self.version.saturating_add(1);
                TransactionOutcome::Committed {
                    value,
                    version: self.version,
                }
            }
        }
    }

    pub fn transact<V, E>(
        &mut self,
        decide: impl FnOnce(&TransactionView<S>) -> TransactionDecision<V, S, E>,
    ) -> TransactionOutcome<V, E> {
        let view = self.view();
        let decision = decide(&view);
        self.commit(&view, decision, false)
    }
}

#[must_use]
pub fn transaction_or_else<V, S, E>(
    left: TransactionDecision<V, S, E>,
    right: impl FnOnce() -> TransactionDecision<V, S, E>,
) -> TransactionDecision<V, S, E> {
    if matches!(left, TransactionDecision::RetryWhenChanged(_)) {
        right()
    } else {
        left
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClockIdentity(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Instant {
    pub clock: ClockIdentity,
    pub ticks: u128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Duration(pub u128);

impl Instant {
    /// Compare instants from the same clock.
    ///
    /// # Errors
    ///
    /// Cross-clock comparison has no meaning.
    pub fn compare(&self, other: &Self) -> Result<std::cmp::Ordering, &'static str> {
        if self.clock != other.clock {
            return Err("instants from different clocks cannot be compared");
        }
        Ok(self.ticks.cmp(&other.ticks))
    }

    /// Add a duration without changing clock identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the instant range overflows.
    pub fn checked_add(self, duration: Duration) -> Result<Self, &'static str> {
        Ok(Self {
            clock: self.clock,
            ticks: self
                .ticks
                .checked_add(duration.0)
                .ok_or("instant range overflow")?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Deadline {
    pub instant: Instant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonotonicClock {
    identity: ClockIdentity,
    last: Option<u128>,
}

impl MonotonicClock {
    #[must_use]
    pub const fn new(identity: ClockIdentity) -> Self {
        Self {
            identity,
            last: None,
        }
    }

    /// Accept one provider observation.
    ///
    /// # Errors
    ///
    /// A decreasing observation violates monotonicity.
    pub fn observe(&mut self, ticks: u128) -> Result<Instant, &'static str> {
        if self.last.is_some_and(|last| ticks < last) {
            return Err("monotonic clock observation decreased");
        }
        self.last = Some(ticks);
        Ok(Instant {
            clock: self.identity,
            ticks,
        })
    }

    /// Construct one absolute deadline from a monotonic observation.
    ///
    /// # Errors
    ///
    /// Returns an error for a decreasing observation or instant overflow.
    pub fn deadline_after(
        &mut self,
        now: u128,
        duration: Duration,
    ) -> Result<Deadline, &'static str> {
        Ok(Deadline {
            instant: self.observe(now)?.checked_add(duration)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LatePolicy {
    CatchUp,
    SkipLate,
    Coalesce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tick {
    pub sequence: u64,
    pub scheduled: Instant,
    pub observed: Instant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Periodic {
    clock: ClockIdentity,
    period: u128,
    phase: u128,
    policy: LatePolicy,
    next_sequence: u64,
}

impl Periodic {
    /// Construct a periodic source.
    ///
    /// # Errors
    ///
    /// Zero period cannot define a productive sequence.
    pub const fn new(
        clock: ClockIdentity,
        period: u128,
        phase: u128,
        policy: LatePolicy,
    ) -> Result<Self, &'static str> {
        if period == 0 {
            return Err("period must be positive");
        }
        Ok(Self {
            clock,
            period,
            phase,
            policy,
            next_sequence: 0,
        })
    }

    /// Produce the ticks admitted by one observation.
    ///
    /// # Errors
    ///
    /// Schedule arithmetic is exact and rejects overflow.
    pub fn release(&mut self, observed_ticks: u128) -> Result<Vec<Tick>, &'static str> {
        let due = |sequence: u64| {
            self.period
                .checked_mul(u128::from(sequence))
                .and_then(|offset| self.phase.checked_add(offset))
                .ok_or("periodic schedule overflow")
        };
        let mut scheduled = Vec::new();
        while due(self.next_sequence)? <= observed_ticks {
            scheduled.push((self.next_sequence, due(self.next_sequence)?));
            self.next_sequence = self.next_sequence.saturating_add(1);
        }
        let selected: Vec<_> = match self.policy {
            LatePolicy::CatchUp => scheduled,
            LatePolicy::SkipLate => scheduled
                .into_iter()
                .filter(|(_, instant)| *instant == observed_ticks)
                .collect(),
            LatePolicy::Coalesce => scheduled.into_iter().next_back().into_iter().collect(),
        };
        Ok(selected
            .into_iter()
            .map(|(sequence, scheduled)| Tick {
                sequence,
                scheduled: Instant {
                    clock: self.clock,
                    ticks: scheduled,
                },
                observed: Instant {
                    clock: self.clock,
                    ticks: observed_ticks,
                },
            })
            .collect())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowEdge {
    pub source: usize,
    pub target: usize,
    pub produces: u64,
    pub consumes: u64,
    pub initial_delay: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowSchedule {
    pub repetitions: Vec<u64>,
    pub firings: Vec<usize>,
    pub capacities: Vec<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fraction {
    numerator: u128,
    denominator: u128,
}

impl Fraction {
    fn normalized(numerator: u128, denominator: u128) -> Result<Self, &'static str> {
        if denominator == 0 {
            return Err("flow rate denominator is zero");
        }
        let divisor = gcd(numerator, denominator);
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    fn multiply(self, numerator: u64, denominator: u64) -> Result<Self, &'static str> {
        Self::normalized(
            self.numerator
                .checked_mul(u128::from(numerator))
                .ok_or("flow balance arithmetic overflow")?,
            self.denominator
                .checked_mul(u128::from(denominator))
                .ok_or("flow balance arithmetic overflow")?,
        )
    }
}

const fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn lcm(left: u128, right: u128) -> Option<u128> {
    left.checked_div(gcd(left, right))?.checked_mul(right)
}

/// Derive a balanced deterministic sequential schedule and exact capacities.
///
/// # Errors
///
/// Zero rates, inconsistent balance, zero-delay cycles, dead schedules, and
/// arithmetic overflow are rejected.
#[allow(clippy::too_many_lines)] // Balance solving and scheduling form one checked derivation.
pub fn derive_flow_schedule(
    actor_count: usize,
    edges: &[FlowEdge],
) -> Result<FlowSchedule, &'static str> {
    if actor_count == 0 {
        return Err("a flow graph requires an actor");
    }
    if edges.iter().any(|edge| {
        edge.source >= actor_count
            || edge.target >= actor_count
            || edge.produces == 0
            || edge.consumes == 0
    }) {
        return Err("flow edge has an invalid actor or zero rate");
    }
    if has_zero_delay_cycle(actor_count, edges) {
        return Err("zero-delay causal cycle");
    }
    let mut ratios = vec![None; actor_count];
    for root in 0..actor_count {
        if ratios[root].is_some() {
            continue;
        }
        ratios[root] = Some(Fraction::normalized(1, 1)?);
        let mut work = VecDeque::from([root]);
        while let Some(actor) = work.pop_front() {
            let ratio = ratios[actor].ok_or("flow actor has no derived balance ratio")?;
            for edge in edges {
                let candidate = if edge.source == actor {
                    Some((edge.target, ratio.multiply(edge.produces, edge.consumes)?))
                } else if edge.target == actor {
                    Some((edge.source, ratio.multiply(edge.consumes, edge.produces)?))
                } else {
                    None
                };
                if let Some((other, candidate)) = candidate {
                    if let Some(existing) = ratios[other] {
                        if existing != candidate {
                            return Err("flow graph has inconsistent balance equations");
                        }
                    } else {
                        ratios[other] = Some(candidate);
                        work.push_back(other);
                    }
                }
            }
        }
    }
    let denominator = ratios.iter().flatten().try_fold(1_u128, |value, ratio| {
        lcm(value, ratio.denominator).ok_or("flow repetition arithmetic overflow")
    })?;
    let raw = ratios
        .iter()
        .map(|ratio| {
            let ratio = ratio.ok_or("flow actor has no derived balance ratio")?;
            ratio
                .numerator
                .checked_mul(denominator / ratio.denominator)
                .ok_or("flow repetition arithmetic overflow")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let divisor = raw.iter().copied().reduce(gcd).unwrap_or(1);
    let repetitions = raw
        .into_iter()
        .map(|value| u64::try_from(value / divisor).map_err(|_| "flow repetition is too large"))
        .collect::<Result<Vec<_>, _>>()?;

    let mut remaining = repetitions.clone();
    let mut tokens = edges
        .iter()
        .map(|edge| edge.initial_delay)
        .collect::<Vec<_>>();
    let mut capacities = tokens.clone();
    let mut firings = Vec::new();
    while remaining.iter().any(|count| *count > 0) {
        let actor = (0..actor_count)
            .filter(|actor| {
                remaining[*actor] > 0
                    && edges.iter().enumerate().all(|(index, edge)| {
                        edge.target != *actor || tokens[index] >= edge.consumes
                    })
            })
            .max_by_key(|actor| {
                (
                    edges
                        .iter()
                        .filter(|edge| edge.target == *actor)
                        .map(|edge| edge.consumes)
                        .sum::<u64>(),
                    std::cmp::Reverse(*actor),
                )
            });
        let Some(actor) = actor else {
            return Err("flow graph has no live bounded sequential schedule");
        };
        for (index, edge) in edges.iter().enumerate() {
            if edge.target == actor {
                tokens[index] -= edge.consumes;
            }
            if edge.source == actor {
                tokens[index] = tokens[index]
                    .checked_add(edge.produces)
                    .ok_or("flow capacity arithmetic overflow")?;
                capacities[index] = capacities[index].max(tokens[index]);
            }
        }
        remaining[actor] -= 1;
        firings.push(actor);
    }
    Ok(FlowSchedule {
        repetitions,
        firings,
        capacities,
    })
}

fn has_zero_delay_cycle(actor_count: usize, edges: &[FlowEdge]) -> bool {
    fn visit(
        actor: usize,
        edges: &[FlowEdge],
        visiting: &mut [bool],
        visited: &mut [bool],
    ) -> bool {
        if visiting[actor] {
            return true;
        }
        if visited[actor] {
            return false;
        }
        visiting[actor] = true;
        for edge in edges
            .iter()
            .filter(|edge| edge.source == actor && edge.initial_delay == 0)
        {
            if visit(edge.target, edges, visiting, visited) {
                return true;
            }
        }
        visiting[actor] = false;
        visited[actor] = true;
        false
    }
    let mut visiting = vec![false; actor_count];
    let mut visited = vec![false; actor_count];
    (0..actor_count).any(|actor| visit(actor, edges, &mut visiting, &mut visited))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumptionMode {
    OneShot,
    Multiple,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectOperation {
    pub input: String,
    pub result: String,
    pub mode: ResumptionMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectProtocol {
    pub operations: BTreeMap<String, EffectOperation>,
}

impl EffectProtocol {
    /// Validate exact handler completeness.
    ///
    /// # Errors
    ///
    /// Returns an error when handlers are missing or add undeclared operations.
    pub fn validate_handler<'a>(
        &self,
        handlers: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), &'static str> {
        let handlers = handlers
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<BTreeSet<_>>();
        if handlers == self.operations.keys().cloned().collect() {
            Ok(())
        } else {
            Err("handler operations do not exactly match the effect protocol")
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumptionState {
    Live,
    Resumed,
    Abandoned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resumption {
    mode: ResumptionMode,
    state: ResumptionState,
    resumes: u64,
    multishot_verified: bool,
    cleanup_performed: bool,
}

impl Resumption {
    #[must_use]
    pub const fn new(mode: ResumptionMode, multishot_verified: bool) -> Self {
        Self {
            mode,
            state: ResumptionState::Live,
            resumes: 0,
            multishot_verified,
            cleanup_performed: false,
        }
    }

    /// Resume the continuation once, or repeatedly only with verified
    /// multi-shot evidence.
    ///
    /// # Errors
    ///
    /// Returns an error after abandon, on a repeated one-shot resume, or when
    /// multi-shot evidence is unavailable.
    pub fn resume(&mut self) -> Result<(), &'static str> {
        match (self.mode, self.resumes, self.multishot_verified) {
            (_, _, _) if self.state == ResumptionState::Abandoned => {
                Err("an abandoned resumption cannot be resumed")
            }
            (ResumptionMode::OneShot, 0, _) => {
                self.resumes = 1;
                self.state = ResumptionState::Resumed;
                Ok(())
            }
            (ResumptionMode::OneShot, _, _) => Err("one-shot resumption was already used"),
            (ResumptionMode::Multiple, _, true) => {
                self.resumes = self.resumes.saturating_add(1);
                self.state = ResumptionState::Resumed;
                Ok(())
            }
            (ResumptionMode::Multiple, _, false) => {
                Err("multiple resumption requires verified MultiShot evidence")
            }
        }
    }

    /// Abandon a live continuation and perform its cleanup transition once.
    ///
    /// # Errors
    ///
    /// Returns an error when the continuation was already resumed or abandoned.
    pub fn abandon(&mut self) -> Result<(), &'static str> {
        if self.state != ResumptionState::Live {
            return Err("only a live resumption can be abandoned");
        }
        self.state = ResumptionState::Abandoned;
        self.cleanup_performed = true;
        Ok(())
    }

    #[must_use]
    pub const fn cleanup_performed(&self) -> bool {
        self.cleanup_performed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bound;

    #[test]
    fn channel_progress_requires_derived_spsc_and_certificate() {
        let topology = EndpointTopology {
            producers: 1,
            consumers: 1,
            fixed_layout: true,
            dynamically_resized: false,
        };
        assert!(select_channel(&topology, Some(&ProgressClass::LockFree), false).is_err());
        assert_eq!(
            select_channel(&topology, Some(&ProgressClass::LockFree), true)
                .unwrap()
                .implementation,
            ChannelImplementation::SpscRing
        );
    }

    #[test]
    fn snapshots_never_reclaim_an_observed_version() {
        let mut snapshot = PublishedSnapshot::new(1, 1, SnapshotPressure::Reject).unwrap();
        let old = snapshot.observe();
        assert_eq!(snapshot.publish(2), Err(SnapshotPublishError::Capacity));
        assert_eq!(*old.value(), 1);
        drop(old);
        assert_eq!(snapshot.publish(2), Ok(1));
        assert_eq!(*snapshot.observe().value(), 2);
    }

    #[test]
    fn transaction_conflict_and_cancellation_publish_nothing() {
        let mut domain = TransactionDomain::new(1_u64);
        let stale = domain.view();
        assert!(matches!(
            domain.transact(|view| TransactionDecision::<(), _, ()>::Commit {
                value: (),
                state: view.state() + 1,
            }),
            TransactionOutcome::Committed { version: 1, .. }
        ));
        assert!(matches!(
            domain.commit(
                &stale,
                TransactionDecision::<(), _, ()>::Commit {
                    value: (),
                    state: 99
                },
                false,
            ),
            TransactionOutcome::Conflict {
                observed_version: 1
            }
        ));
        let current = domain.view();
        assert!(matches!(
            domain.commit(
                &current,
                TransactionDecision::<(), _, ()>::Commit {
                    value: (),
                    state: 99
                },
                true,
            ),
            TransactionOutcome::Cancelled
        ));
        assert_eq!(*domain.view().state(), 2);
    }

    #[test]
    fn time_is_clock_typed_and_periodic_release_does_not_drift() {
        let mut clock = MonotonicClock::new(ClockIdentity(1));
        let first = clock.observe(10).unwrap();
        assert!(
            first
                .compare(&Instant {
                    clock: ClockIdentity(2),
                    ticks: 10
                })
                .is_err()
        );
        assert!(clock.observe(9).is_err());

        let mut periodic = Periodic::new(ClockIdentity(1), 10, 5, LatePolicy::CatchUp).unwrap();
        let ticks = periodic.release(27).unwrap();
        assert_eq!(
            ticks
                .iter()
                .map(|tick| tick.scheduled.ticks)
                .collect::<Vec<_>>(),
            vec![5, 15, 25]
        );
    }

    #[test]
    fn flow_derives_repetitions_schedule_and_capacity() {
        let schedule = derive_flow_schedule(
            2,
            &[FlowEdge {
                source: 0,
                target: 1,
                produces: 2,
                consumes: 3,
                initial_delay: 0,
            }],
        )
        .unwrap();
        assert_eq!(schedule.repetitions, vec![3, 2]);
        assert_eq!(schedule.firings, vec![0, 0, 1, 0, 1]);
        assert_eq!(schedule.capacities, vec![4]);
        assert!(
            derive_flow_schedule(
                2,
                &[
                    FlowEdge {
                        source: 0,
                        target: 1,
                        produces: 1,
                        consumes: 1,
                        initial_delay: 0
                    },
                    FlowEdge {
                        source: 1,
                        target: 0,
                        produces: 1,
                        consumes: 1,
                        initial_delay: 0
                    },
                ],
            )
            .is_err()
        );
    }

    #[test]
    fn resumptions_are_affine_unless_multishot_is_verified() {
        let mut one = Resumption::new(ResumptionMode::OneShot, false);
        assert!(one.resume().is_ok());
        assert!(one.resume().is_err());
        let mut multiple = Resumption::new(ResumptionMode::Multiple, false);
        assert!(multiple.resume().is_err());
        let mut abandoned = Resumption::new(ResumptionMode::OneShot, false);
        abandoned.abandon().unwrap();
        assert!(abandoned.cleanup_performed());
        assert!(abandoned.resume().is_err());
    }

    #[test]
    fn wait_free_bound_is_not_implied_by_lock_free() {
        assert!(
            !ProgressClass::LockFree.satisfies(&ProgressClass::WaitFree {
                maximum_own_steps: Bound::Exact(10),
            })
        );
    }
}
