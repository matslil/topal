use std::collections::BTreeMap;

use crate::{TraceEvent, TraceSink, Value};

#[derive(Clone, Copy, Debug)]
pub struct ExecutionSnapshot<'a> {
    pub bindings: &'a BTreeMap<String, Value>,
    pub value: Option<&'a Value>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionState {
    pub bindings: BTreeMap<String, Value>,
    pub value: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Checkpoint {
    cursor: usize,
    state: ExecutionState,
}

/// One owned semantic transition in deterministic execution order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionTransition {
    pub sequence: usize,
    pub event: &'static str,
    pub rule: &'static str,
    pub detail: String,
}

/// A cursor-addressable record of semantic execution transitions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionHistory {
    transitions: Vec<ExecutionTransition>,
    checkpoints: Vec<Checkpoint>,
    cursor: usize,
}

impl ExecutionHistory {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn transitions(&self) -> &[ExecutionTransition] {
        &self.transitions
    }

    #[must_use]
    pub fn current(&self) -> Option<&ExecutionTransition> {
        self.cursor
            .checked_sub(1)
            .and_then(|index| self.transitions.get(index))
    }

    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    #[must_use]
    pub fn state(&self) -> Option<&ExecutionState> {
        self.checkpoints
            .iter()
            .rev()
            .find(|checkpoint| checkpoint.cursor <= self.cursor)
            .map(|checkpoint| &checkpoint.state)
    }

    pub fn step_forward(&mut self) -> Option<&ExecutionTransition> {
        if self.cursor == self.transitions.len() {
            return None;
        }
        self.cursor += 1;
        self.current()
    }

    pub fn step_backward(&mut self) -> Option<&ExecutionTransition> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        self.current()
    }

    pub const fn rewind(&mut self) {
        self.cursor = 0;
    }
}

impl TraceSink for ExecutionHistory {
    fn record(&mut self, event: TraceEvent<'_>) {
        let sequence = self.transitions.len();
        self.transitions.push(ExecutionTransition {
            sequence,
            event: event.event,
            rule: event.rule,
            detail: event.detail.to_owned(),
        });
        self.cursor = self.transitions.len();
    }

    fn checkpoint(&mut self, snapshot: ExecutionSnapshot<'_>) {
        let state = ExecutionState {
            bindings: snapshot.bindings.clone(),
            value: snapshot.value.cloned(),
        };
        if let Some(checkpoint) = self
            .checkpoints
            .iter_mut()
            .find(|checkpoint| checkpoint.cursor == self.cursor)
        {
            checkpoint.state = state;
        } else {
            self.checkpoints.push(Checkpoint {
                cursor: self.cursor,
                state,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    #[test]
    fn records_and_navigates_deterministic_semantic_history() {
        let mut history = ExecutionHistory::new();
        let value = Session::new()
            .evaluate(
                include_str!("../../../examples/debugger/basic-history.t"),
                &mut history,
            )
            .expect("example should evaluate");

        assert_eq!(value.to_string(), "42");
        assert!(!history.transitions().is_empty());
        assert_eq!(history.cursor(), history.transitions().len());
        assert_eq!(history.current().unwrap().event, "evaluation.result");

        let last_sequence = history.current().unwrap().sequence;
        history.step_backward();
        assert_eq!(history.cursor(), last_sequence);
        history.rewind();
        assert_eq!(history.cursor(), 0);
        assert!(history.current().is_none());
        assert_eq!(history.step_forward().unwrap().sequence, 0);
        assert!(history.state().unwrap().bindings.is_empty());

        while history.step_forward().is_some() {}
        let state = history.state().unwrap();
        assert_eq!(state.bindings["answer"].to_string(), "40");
        assert_eq!(state.value.as_ref().unwrap().to_string(), "42");
    }
}
