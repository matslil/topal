//! Shared static concurrency evidence used by the interpreter, debugger, and
//! future compiler rather than embedding scheduler policy in any one tool.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractionForm {
    Event,
    Request,
    Stream,
    Direct,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admission {
    BoundedWait,
    BoundedReject,
    DiagnosticLoss,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Interaction {
    pub form: InteractionForm,
    pub admission: Admission,
    pub returns_unit: bool,
}

impl Interaction {
    /// Validate the portable interaction and backpressure contract.
    ///
    /// # Errors
    /// Returns the violated stable concurrency rule.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.form == InteractionForm::Event
            && self.returns_unit
            && self.admission == Admission::DiagnosticLoss
        {
            return Err("TOPAL-CONC-BACKPRESSURE-001: an ordinary Unit event cannot be lost");
        }
        if self.form != InteractionForm::Event && self.returns_unit {
            return Err("TOPAL-CONC-INTERACT-001: requests and streams require a response Result");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolTransition {
    pub from: String,
    pub message: String,
    pub to: String,
    pub obligations: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Protocol {
    pub initial: String,
    pub terminal: BTreeSet<String>,
    pub transitions: Vec<ProtocolTransition>,
}

impl Protocol {
    /// # Errors
    /// Returns a protocol-fidelity error for ambiguous or invalid terminals.
    pub fn validate(&self) -> Result<(), &'static str> {
        let mut labels = BTreeSet::new();
        for transition in &self.transitions {
            if !labels.insert((&transition.from, &transition.message)) {
                return Err("TOPAL-CONC-PROTOCOL-001: duplicate transition label");
            }
        }
        if self
            .terminal
            .iter()
            .any(|state| self.transitions.iter().any(|edge| &edge.from == state))
        {
            return Err(
                "TOPAL-CONC-PROTOCOL-001: terminal protocol state has an outgoing transition",
            );
        }
        Ok(())
    }

    #[must_use]
    pub fn advance(&self, state: &str, message: &str) -> Option<(&str, &BTreeSet<String>)> {
        self.transitions
            .iter()
            .find(|edge| edge.from == state && edge.message == message)
            .map(|edge| (edge.to.as_str(), &edge.obligations))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyKind {
    Internal,
    External,
    Runnable,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DependencyGraph {
    nodes: BTreeMap<String, DependencyKind>,
    edges: BTreeSet<(String, String)>,
}

impl DependencyGraph {
    pub fn add_node(&mut self, name: impl Into<String>, kind: DependencyKind) {
        self.nodes.insert(name.into(), kind);
    }

    pub fn add_dependency(&mut self, before: impl Into<String>, after: impl Into<String>) {
        self.edges.insert((before.into(), after.into()));
    }

    #[must_use]
    pub fn happens_before(&self, before: &str, after: &str) -> bool {
        let mut frontier = vec![before];
        let mut visited = BTreeSet::new();
        while let Some(node) = frontier.pop() {
            if !visited.insert(node) {
                continue;
            }
            for (_, next) in self.edges.iter().filter(|(from, _)| from == node) {
                if next == after {
                    return true;
                }
                frontier.push(next);
            }
        }
        false
    }

    /// Reject a closed wait cycle made only of internal dependencies.
    ///
    /// # Errors
    /// Returns the deadlock rule when a closed internal cycle is reachable.
    pub fn validate_progress(&self) -> Result<(), &'static str> {
        for start in self.nodes.keys() {
            if self.nodes[start] != DependencyKind::Internal {
                continue;
            }
            if self.edges.iter().any(|(_, to)| to == start)
                && self.happens_before(start, start)
                && !self.nodes.iter().any(|(name, kind)| {
                    *kind != DependencyKind::Internal && self.happens_before(name, start)
                })
            {
                return Err("TOPAL-CONC-DEADLOCK-001: closed internal wait cycle");
            }
        }
        Ok(())
    }

    /// # Errors
    /// Returns the race-freedom rule when a conflict is unordered.
    pub fn validate_conflicts(&self, conflicts: &[(String, String)]) -> Result<(), &'static str> {
        if conflicts.iter().any(|(left, right)| {
            !self.happens_before(left, right) && !self.happens_before(right, left)
        }) {
            Err("TOPAL-CONC-RACE-001: conflicting events are unordered")
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaskScope {
    obligations: BTreeSet<String>,
    cancelled: BTreeSet<String>,
}

impl TaskScope {
    pub fn spawn(&mut self, identity: impl Into<String>) {
        self.obligations.insert(identity.into());
    }
    pub fn complete(&mut self, identity: &str) {
        self.obligations.remove(identity);
    }
    pub fn cancel(&mut self, identity: &str) {
        if self.obligations.contains(identity) {
            self.cancelled.insert(identity.into());
        }
    }
    pub fn acknowledge_cancellation(&mut self, identity: &str) {
        self.cancelled.remove(identity);
        self.obligations.remove(identity);
    }
    /// # Errors
    /// Returns the outstanding structured-lifetime or cancellation obligation.
    pub fn close(&self) -> Result<(), &'static str> {
        if self.obligations.is_empty() {
            Ok(())
        } else if self.cancelled.is_empty() {
            Err("TOPAL-CONC-SCOPE-001: child obligations remain")
        } else {
            Err("TOPAL-CONC-CANCEL-001: cancellation is not acknowledged")
        }
    }
}

/// # Errors
/// Returns the determinism rule when permitted schedules have unequal results.
pub fn validate_schedule_equivalence(results: &[&str]) -> Result<(), &'static str> {
    if results.windows(2).all(|pair| pair[0] == pair[1]) {
        Ok(())
    } else {
        Err("TOPAL-CONC-DETERMINISM-001: permitted schedules differ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_protocol_dependency_scope_and_determinism_evidence() {
        let protocol = Protocol {
            initial: "ready".into(),
            terminal: BTreeSet::from(["done".into()]),
            transitions: vec![ProtocolTransition {
                from: "ready".into(),
                message: "request".into(),
                to: "done".into(),
                obligations: BTreeSet::from(["reply".into()]),
            }],
        };
        assert!(protocol.validate().is_ok());
        assert_eq!(protocol.advance("ready", "request").unwrap().0, "done");

        let mut graph = DependencyGraph::default();
        graph.add_node("send", DependencyKind::Runnable);
        graph.add_node("receive", DependencyKind::Internal);
        graph.add_dependency("send", "receive");
        assert!(graph.validate_progress().is_ok());
        assert!(
            graph
                .validate_conflicts(&[("send".into(), "receive".into())])
                .is_ok()
        );

        let mut scope = TaskScope::default();
        scope.spawn("child");
        scope.cancel("child");
        assert!(scope.close().unwrap_err().contains("TOPAL-CONC-CANCEL-001"));
        scope.acknowledge_cancellation("child");
        assert!(scope.close().is_ok());
        assert!(validate_schedule_equivalence(&["same", "same"]).is_ok());
    }

    #[test]
    fn rejects_loss_cycles_races_and_schedule_divergence() {
        assert!(
            Interaction {
                form: InteractionForm::Event,
                admission: Admission::DiagnosticLoss,
                returns_unit: true
            }
            .validate()
            .is_err()
        );
        let mut graph = DependencyGraph::default();
        graph.add_node("a", DependencyKind::Internal);
        graph.add_node("b", DependencyKind::Internal);
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "a");
        assert!(graph.validate_progress().is_err());
        assert!(
            DependencyGraph::default()
                .validate_conflicts(&[("a".into(), "b".into())])
                .is_err()
        );
        assert!(validate_schedule_equivalence(&["left", "right"]).is_err());
    }
}
