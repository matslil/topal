//! Deterministic operation submission, cancellation, and completion.

use std::collections::{BTreeMap, VecDeque};

/// Stable identity correlating submission with exactly one completion.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperationId(u64);

/// Terminal semantic outcome of an operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome<Value, Failure> {
    Success(Value),
    Failed(Failure),
    Cancelled,
    EndpointLost,
    TimedOutUncertain,
}

/// One terminal completion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Completion<Value, Failure> {
    pub operation: OperationId,
    pub outcome: Outcome<Value, Failure>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingState {
    Submitted,
    CancellationRequested,
}

/// Deterministic reference scheduler with bounded outstanding work.
#[derive(Debug)]
pub struct Scheduler<Value, Failure> {
    next: u64,
    limit: usize,
    pending: BTreeMap<OperationId, PendingState>,
    completions: VecDeque<Completion<Value, Failure>>,
}

impl<Value, Failure> Scheduler<Value, Failure> {
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self {
            next: 1,
            limit,
            pending: BTreeMap::new(),
            completions: VecDeque::new(),
        }
    }

    /// Submits a new semantic operation.
    ///
    /// # Errors
    /// Returns `Exhausted` when the outstanding-operation bound is reached.
    pub fn submit(&mut self) -> Result<OperationId, SubmitFailure> {
        if self.pending.len() >= self.limit {
            return Err(SubmitFailure::Exhausted);
        }
        let identity = OperationId(self.next);
        self.next = self.next.checked_add(1).ok_or(SubmitFailure::Exhausted)?;
        self.pending.insert(identity, PendingState::Submitted);
        Ok(identity)
    }

    /// Requests cancellation without claiming that cancellation won the race.
    #[must_use]
    pub fn request_cancel(&mut self, operation: OperationId) -> bool {
        self.pending.get_mut(&operation).is_some_and(|state| {
            *state = PendingState::CancellationRequested;
            true
        })
    }

    /// Records the single observed terminal native or virtual outcome.
    #[must_use]
    pub fn complete(&mut self, operation: OperationId, outcome: Outcome<Value, Failure>) -> bool {
        if self.pending.remove(&operation).is_none() {
            return false;
        }
        self.completions
            .push_back(Completion { operation, outcome });
        true
    }

    /// Returns the next completion in declared completion order.
    pub fn poll(&mut self) -> Option<Completion<Value, Failure>> {
        self.completions.pop_front()
    }

    #[must_use]
    pub fn outstanding(&self) -> usize {
        self.pending.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmitFailure {
    Exhausted,
    UnsafeRetry,
}

/// Evidence required before automatically retrying an effectful operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryEvidence {
    Idempotent,
    Deduplicated,
    Transactional,
    None,
}

/// Checks whether retry evidence prevents duplicated effects.
///
/// # Errors
/// Returns `UnsafeRetry` when no qualifying evidence is present.
pub const fn admit_retry(evidence: RetryEvidence) -> Result<(), SubmitFailure> {
    match evidence {
        RetryEvidence::Idempotent | RetryEvidence::Deduplicated | RetryEvidence::Transactional => {
            Ok(())
        }
        RetryEvidence::None => Err(SubmitFailure::UnsafeRetry),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_and_completion_have_one_terminal_observation() {
        for cancel_first in [false, true] {
            let mut scheduler = Scheduler::<u8, u8>::new(1);
            let operation = scheduler.submit().unwrap();
            if cancel_first {
                assert!(scheduler.request_cancel(operation));
            }
            assert!(scheduler.complete(operation, Outcome::Success(7)));
            assert!(!scheduler.complete(operation, Outcome::Cancelled));
            assert!(!scheduler.request_cancel(operation));
            assert_eq!(scheduler.poll().unwrap().outcome, Outcome::Success(7));
        }
    }

    #[test]
    fn queue_bound_and_retry_admission_are_explicit() {
        let mut scheduler = Scheduler::<(), ()>::new(1);
        scheduler.submit().unwrap();
        assert_eq!(scheduler.submit(), Err(SubmitFailure::Exhausted));
        assert_eq!(
            admit_retry(RetryEvidence::None),
            Err(SubmitFailure::UnsafeRetry)
        );
        assert_eq!(admit_retry(RetryEvidence::Transactional), Ok(()));
    }

    #[test]
    fn completion_order_is_observation_order() {
        let mut scheduler = Scheduler::<u8, ()>::new(2);
        let first = scheduler.submit().unwrap();
        let second = scheduler.submit().unwrap();
        assert!(scheduler.complete(second, Outcome::Success(2)));
        assert!(scheduler.complete(first, Outcome::Success(1)));
        assert_eq!(scheduler.poll().unwrap().operation, second);
        assert_eq!(scheduler.poll().unwrap().operation, first);
    }
}
