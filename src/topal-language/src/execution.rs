use std::collections::BTreeMap;
use std::rc::Rc;

use crate::{TraceEvent, TraceSink, Value};
use topal_source::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ExecutionSnapshot<'a> {
    pub bindings: &'a BTreeMap<String, Value>,
    pub value: Option<&'a Value>,
    pub span: Option<Span>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionState {
    pub bindings: BTreeMap<String, Value>,
    pub value: Option<Value>,
    pub source_range: Option<SourceRange>,
    pub source_name: Option<Rc<str>>,
    pub source: Option<Rc<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Checkpoint {
    cursor: usize,
    binding_changes: Vec<(String, Option<Rc<Value>>)>,
    value: Option<Value>,
    source_range: Option<SourceRange>,
    source_name: Option<Rc<str>>,
    source: Option<Rc<str>>,
}

/// One owned semantic transition in deterministic execution order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionTransition {
    pub sequence: usize,
    pub event: &'static str,
    pub rule: &'static str,
    pub detail: String,
    pub transaction: Option<u64>,
    pub sender: Option<u64>,
    pub receiver: Option<u64>,
}

/// A cursor-addressable record of semantic execution transitions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionHistory {
    transitions: Vec<ExecutionTransition>,
    checkpoints: Vec<Checkpoint>,
    current_bindings: BTreeMap<String, Rc<Value>>,
    cursor: usize,
    source_stack: Vec<(Rc<str>, Rc<str>)>,
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

    pub fn seek(&mut self, cursor: usize) -> bool {
        if cursor > self.transitions.len() {
            return false;
        }
        self.cursor = cursor;
        true
    }

    pub fn finish(&mut self) -> Option<ExecutionState> {
        self.cursor = self.transitions.len();
        self.state()
    }

    pub fn reverse_finish(&mut self) -> Option<ExecutionState> {
        self.cursor = 0;
        self.state()
    }

    #[must_use]
    pub fn state(&self) -> Option<ExecutionState> {
        let (bindings, latest) = self.shared_state()?;
        Some(ExecutionState {
            bindings: bindings
                .into_iter()
                .map(|(name, value)| (name, (*value).clone()))
                .collect(),
            value: latest.value.clone(),
            source_range: latest.source_range,
            source_name: latest.source_name.clone(),
            source: latest.source.clone(),
        })
    }

    fn shared_state(&self) -> Option<(BTreeMap<String, Rc<Value>>, &Checkpoint)> {
        let mut bindings = BTreeMap::new();
        let mut latest = None;
        for checkpoint in self
            .checkpoints
            .iter()
            .take_while(|checkpoint| checkpoint.cursor <= self.cursor)
        {
            for (name, value) in &checkpoint.binding_changes {
                if let Some(value) = value {
                    bindings.insert(name.clone(), Rc::clone(value));
                } else {
                    bindings.remove(name);
                }
            }
            latest = Some(checkpoint);
        }
        latest.map(|checkpoint| (bindings, checkpoint))
    }

    pub fn step_source_forward(&mut self) -> Option<ExecutionState> {
        let cursor = self
            .checkpoints
            .iter()
            .find(|checkpoint| {
                checkpoint.cursor > self.cursor && checkpoint.source_range.is_some()
            })?
            .cursor;
        self.cursor = cursor;
        self.state()
    }

    pub fn step_source_backward(&mut self) -> Option<ExecutionState> {
        let cursor = self
            .checkpoints
            .iter()
            .rev()
            .find(|checkpoint| {
                checkpoint.cursor < self.cursor && checkpoint.source_range.is_some()
            })?
            .cursor;
        self.cursor = cursor;
        self.state()
    }

    pub fn continue_source_forward(
        &mut self,
        predicate: impl Fn(&ExecutionState) -> bool,
    ) -> Option<ExecutionState> {
        while let Some(state) = self.step_source_forward() {
            if predicate(&state) {
                return Some(state);
            }
        }
        None
    }

    pub fn continue_source_backward(
        &mut self,
        predicate: impl Fn(&ExecutionState) -> bool,
    ) -> Option<ExecutionState> {
        while let Some(state) = self.step_source_backward() {
            if predicate(&state) {
                return Some(state);
            }
        }
        None
    }

    pub fn step_forward(&mut self) -> Option<&ExecutionTransition> {
        if self.cursor == self.transitions.len() {
            return None;
        }
        self.cursor += 1;
        if self
            .current()
            .is_some_and(|transition| transition.event == "message.sent")
            && let Some(transaction) = self.current().and_then(|transition| transition.transaction)
            && let Some(received) = self.transitions.iter().find(|transition| {
                transition.event == "message.received"
                    && transition.transaction == Some(transaction)
            })
        {
            self.cursor = received.sequence + 1;
        }
        self.current()
    }

    pub fn step_backward(&mut self) -> Option<&ExecutionTransition> {
        if self.cursor == 0 {
            return None;
        }
        if self
            .current()
            .is_some_and(|transition| transition.event == "message.received")
            && let Some(transaction) = self.current().and_then(|transition| transition.transaction)
            && let Some(sent) = self.transitions.iter().find(|transition| {
                transition.event == "message.sent" && transition.transaction == Some(transaction)
            })
        {
            self.cursor = sent.sequence + 1;
            return self.current();
        }
        self.cursor -= 1;
        self.current()
    }

    pub const fn rewind(&mut self) {
        self.cursor = 0;
    }

    /// Record one complete message transfer using debugger-comparable events.
    pub fn record_message_transfer(
        &mut self,
        transaction: u64,
        sender: u64,
        receiver: u64,
        detail: impl Into<String>,
    ) {
        let detail = detail.into();
        for event in ["message.sent", "message.received"] {
            let sequence = self.transitions.len();
            self.transitions.push(ExecutionTransition {
                sequence,
                event,
                rule: "TOPAL-CONC-ORDER-001",
                detail: detail.clone(),
                transaction: Some(transaction),
                sender: Some(sender),
                receiver: Some(receiver),
            });
        }
        self.cursor = self.transitions.len();
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
            transaction: None,
            sender: None,
            receiver: None,
        });
        self.cursor = self.transitions.len();
    }

    fn checkpoint(&mut self, snapshot: ExecutionSnapshot<'_>) {
        if self
            .checkpoints
            .last()
            .is_some_and(|checkpoint| checkpoint.cursor == self.cursor)
        {
            self.checkpoints.pop();
            self.current_bindings = self
                .shared_state()
                .map_or_else(BTreeMap::new, |(bindings, _)| bindings);
        }
        let mut names = self
            .current_bindings
            .keys()
            .chain(snapshot.bindings.keys())
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        let binding_changes = names
            .into_iter()
            .filter_map(|name| {
                let previous = self.current_bindings.get(&name).map(Rc::as_ref);
                let current = snapshot.bindings.get(&name);
                (previous != current).then(|| (name, current.cloned().map(Rc::new)))
            })
            .collect::<Vec<_>>();
        for (name, value) in &binding_changes {
            if let Some(value) = value {
                self.current_bindings.insert(name.clone(), Rc::clone(value));
            } else {
                self.current_bindings.remove(name);
            }
        }
        self.checkpoints.push(Checkpoint {
            cursor: self.cursor,
            binding_changes,
            value: snapshot.value.cloned(),
            source_range: snapshot.span.map(|span| SourceRange {
                start: span.start,
                end: span.end,
            }),
            source_name: self.source_stack.last().map(|(name, _)| Rc::clone(name)),
            source: self
                .source_stack
                .last()
                .map(|(_, source)| Rc::clone(source)),
        });
    }

    fn push_source(&mut self, source_name: &str, source: &str) {
        self.source_stack
            .push((Rc::from(source_name), Rc::from(source)));
    }

    fn pop_source(&mut self) {
        self.source_stack.pop();
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

        history.rewind();
        let binding_state = loop {
            let state = history.step_source_forward().unwrap();
            if state.bindings.contains_key("answer") {
                break state;
            }
        };
        assert_eq!(binding_state.bindings["answer"].to_string(), "40");
        let result_state = loop {
            let state = history.step_source_forward().unwrap();
            if state
                .value
                .as_ref()
                .is_some_and(|value| value.to_string() == "42")
            {
                break state;
            }
        };
        assert_eq!(result_state.value.as_ref().unwrap().to_string(), "42");
        let result_cursor = history.cursor();
        history.step_source_backward().unwrap();
        assert!(history.cursor() < result_cursor);
    }

    #[test]
    fn stepping_follows_and_reverses_message_transactions() {
        let mut history = ExecutionHistory::new();
        history.record(TraceEvent {
            event: "before",
            rule: "TEST",
            detail: "sender",
        });
        history.record_message_transfer(7, 1, 2, "request query");
        history.rewind();
        assert_eq!(history.step_forward().unwrap().event, "before");
        let received = history.step_forward().unwrap();
        assert_eq!(received.event, "message.received");
        assert_eq!(received.transaction, Some(7));
        assert_eq!((received.sender, received.receiver), (Some(1), Some(2)));
        assert_eq!(history.step_backward().unwrap().event, "message.sent");
    }
}
