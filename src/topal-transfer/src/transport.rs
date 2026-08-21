//! Transport-independent service bindings.

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    Local,
    UdpV4,
    UdpV6,
    TcpV4,
    TcpV6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceState {
    Open,
    ReadClosed,
    WriteClosed,
    Closed,
    Reset,
}

#[derive(Debug)]
pub struct VirtualSequence {
    state: SequenceState,
    bytes: VecDeque<u8>,
    capacity: usize,
}

impl VirtualSequence {
    #[must_use]
    pub const fn new(capacity: usize) -> Self {
        Self {
            state: SequenceState::Open,
            bytes: VecDeque::new(),
            capacity,
        }
    }
    #[must_use]
    pub fn write(&mut self, input: &[u8]) -> usize {
        if !matches!(self.state, SequenceState::Open | SequenceState::ReadClosed) {
            return 0;
        }
        let count = input
            .len()
            .min(self.capacity.saturating_sub(self.bytes.len()));
        self.bytes.extend(&input[..count]);
        count
    }
    #[must_use]
    pub fn read(&mut self, output: &mut [u8]) -> usize {
        let count = output.len().min(self.bytes.len());
        for target in &mut output[..count] {
            if let Some(byte) = self.bytes.pop_front() {
                *target = byte;
            }
        }
        count
    }
    pub fn close_write(&mut self) {
        self.state = match self.state {
            SequenceState::ReadClosed => SequenceState::Closed,
            _ => SequenceState::WriteClosed,
        };
    }
    #[must_use]
    pub const fn state(&self) -> SequenceState {
        self.state
    }
}

pub trait ServiceBinding<Request> {
    type Reply;
    type Failure;
    fn transport(&self) -> TransportKind;
    /// Applies the service contract through this binding.
    /// # Errors
    /// Returns the service's typed failure.
    fn call(&mut self, request: Request) -> Result<Self::Reply, Self::Failure>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequence_reports_partial_progress_and_half_close() {
        let mut sequence = VirtualSequence::new(2);
        assert_eq!(sequence.write(b"abc"), 2);
        let mut output = [0; 1];
        assert_eq!(sequence.read(&mut output), 1);
        assert_eq!(output, [b'a']);
        sequence.close_write();
        assert_eq!(sequence.state(), SequenceState::WriteClosed);
        assert_eq!(sequence.write(b"x"), 0);
    }
}
