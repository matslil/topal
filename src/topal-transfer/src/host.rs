//! Versioned semantic host boundary and deterministic virtual backend.

use std::collections::{BTreeMap, VecDeque};

pub const HOST_ABI_REVISION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HostCapability(u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostOperation {
    Read { offset: usize, length: usize },
    Message(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostObservation {
    Bytes(Vec<u8>),
    MessageAccepted,
    Denied,
    OutOfBounds,
}

pub trait HostBackend {
    fn submit(&mut self, capability: HostCapability, operation: HostOperation) -> HostObservation;
}

#[derive(Debug, Default)]
pub struct VirtualHost {
    next: u64,
    regions: BTreeMap<HostCapability, Vec<u8>>,
    trace: Vec<HostObservation>,
}

impl VirtualHost {
    #[must_use]
    pub fn inject_region(&mut self, bytes: Vec<u8>) -> HostCapability {
        self.next += 1;
        let capability = HostCapability(self.next);
        self.regions.insert(capability, bytes);
        capability
    }
    #[must_use]
    pub fn trace(&self) -> &[HostObservation] {
        &self.trace
    }
}

impl HostBackend for VirtualHost {
    fn submit(&mut self, capability: HostCapability, operation: HostOperation) -> HostObservation {
        let observation = match (self.regions.get(&capability), operation) {
            (None, _) => HostObservation::Denied,
            (Some(bytes), HostOperation::Read { offset, length }) => offset
                .checked_add(length)
                .and_then(|end| bytes.get(offset..end))
                .map_or(HostObservation::OutOfBounds, |span| {
                    HostObservation::Bytes(span.to_vec())
                }),
            (Some(_), HostOperation::Message(_)) => HostObservation::MessageAccepted,
        };
        self.trace.push(observation.clone());
        observation
    }
}

#[derive(Debug)]
pub struct ReplayHost {
    observations: VecDeque<HostObservation>,
}

impl ReplayHost {
    #[must_use]
    pub fn new(observations: impl Into<VecDeque<HostObservation>>) -> Self {
        Self {
            observations: observations.into(),
        }
    }
}

impl HostBackend for ReplayHost {
    fn submit(&mut self, _: HostCapability, _: HostOperation) -> HostObservation {
        self.observations
            .pop_front()
            .unwrap_or(HostObservation::Denied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authority_is_injected_and_replay_repeats_no_effect() {
        let mut host = VirtualHost::default();
        let capability = host.inject_region(vec![1, 2, 3]);
        assert_eq!(
            host.submit(
                capability,
                HostOperation::Read {
                    offset: 1,
                    length: 2
                }
            ),
            HostObservation::Bytes(vec![2, 3])
        );
        assert_eq!(
            host.submit(HostCapability(999), HostOperation::Message(vec![1])),
            HostObservation::Denied
        );
        let mut replay = ReplayHost::new(host.trace().to_vec());
        assert_eq!(
            replay.submit(capability, HostOperation::Message(vec![9])),
            HostObservation::Bytes(vec![2, 3])
        );
    }
}
