//! Typed semantic identities shared by trace producers, observers, and adapters.

use crate::{QualifiedName, TypeIdentity};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TraceProfile {
    Debugging,
    Testing,
}

impl TraceProfile {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Debugging => "debugging",
            Self::Testing => "testing",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueEventKind {
    Create,
    Destroy,
    Access,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionEventKind {
    Entry,
    Exit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueEvent {
    pub kind: ValueEventKind,
    pub identity: u64,
    pub type_identity: TypeIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionEvent {
    pub kind: FunctionEventKind,
    pub invocation: u64,
    pub function: QualifiedName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FundamentalEvent {
    Value(ValueEvent),
    Function(FunctionEvent),
}

/// Deterministic task-like translation from observed events to a typed event group.
///
/// Implementations may keep private recognition state. `None` emits no derived
/// event; `Some` emits exactly the returned value. The execution tool owns
/// delivery and prevents observers from acquiring application authority.
pub trait TraceObserver {
    type Event;

    fn observe(&mut self, event: &FundamentalEvent) -> Option<Self::Event>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operators_share_function_events() {
        let event = FunctionEvent {
            kind: FunctionEventKind::Entry,
            invocation: 7,
            function: QualifiedName(vec!["lang".into(), "arithmetic".into(), "+".into()]),
        };
        assert_eq!(event.kind, FunctionEventKind::Entry);
        assert_eq!(event.function.0.last().unwrap(), "+");
    }

    #[test]
    fn profiles_have_canonical_names() {
        assert_eq!(TraceProfile::Debugging.name(), "debugging");
        assert_eq!(TraceProfile::Testing.name(), "testing");
    }

    #[test]
    fn observers_may_recognize_stateful_lifecycles() {
        #[derive(Default)]
        struct Lifetime {
            active: bool,
        }
        impl TraceObserver for Lifetime {
            type Event = &'static str;

            fn observe(&mut self, event: &FundamentalEvent) -> Option<Self::Event> {
                let FundamentalEvent::Value(event) = event else {
                    return None;
                };
                match event.kind {
                    ValueEventKind::Create => {
                        self.active = true;
                        Some("started")
                    }
                    ValueEventKind::Destroy if self.active => {
                        self.active = false;
                        Some("cancelled")
                    }
                    ValueEventKind::Destroy | ValueEventKind::Access => None,
                }
            }
        }

        let mut observer = Lifetime::default();
        let value = |kind| {
            FundamentalEvent::Value(ValueEvent {
                kind,
                identity: 1,
                type_identity: TypeIdentity::Fundamental("StartedTransaction"),
            })
        };
        assert_eq!(
            observer.observe(&value(ValueEventKind::Create)),
            Some("started")
        );
        assert_eq!(observer.observe(&value(ValueEventKind::Access)), None);
        assert_eq!(
            observer.observe(&value(ValueEventKind::Destroy)),
            Some("cancelled")
        );
    }
}
