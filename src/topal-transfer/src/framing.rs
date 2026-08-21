//! Message-to-sequence framing.

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FramingFailure {
    Oversized,
    Malformed,
    Exhausted,
}

#[derive(Debug)]
pub struct LengthDecoder {
    limit: usize,
    buffered: Vec<u8>,
    messages: VecDeque<Vec<u8>>,
    queue_limit: usize,
}

impl LengthDecoder {
    #[must_use]
    pub const fn new(limit: usize, queue_limit: usize) -> Self {
        Self {
            limit,
            buffered: Vec::new(),
            messages: VecDeque::new(),
            queue_limit,
        }
    }
    /// Consumes arbitrary sequence chunks and preserves encoded message boundaries.
    /// # Errors
    /// Returns `Oversized`, `Malformed`, or `Exhausted` without emitting a partial message.
    pub fn push(&mut self, chunk: &[u8]) -> Result<(), FramingFailure> {
        self.buffered.extend_from_slice(chunk);
        loop {
            if self.buffered.len() < 4 {
                return Ok(());
            }
            let length = u32::from_be_bytes(
                self.buffered[..4]
                    .try_into()
                    .map_err(|_| FramingFailure::Malformed)?,
            ) as usize;
            if length > self.limit {
                return Err(FramingFailure::Oversized);
            }
            let total = 4usize
                .checked_add(length)
                .ok_or(FramingFailure::Oversized)?;
            if self.buffered.len() < total {
                return Ok(());
            }
            if self.messages.len() >= self.queue_limit {
                return Err(FramingFailure::Exhausted);
            }
            self.messages.push_back(self.buffered[4..total].to_vec());
            self.buffered.drain(..total);
        }
    }
    pub fn pop(&mut self) -> Option<Vec<u8>> {
        self.messages.pop_front()
    }
}

/// Encodes one application message for a sequence transport.
/// # Errors
/// Returns `Oversized` if the configured or representational bound is exceeded.
pub fn encode(message: &[u8], limit: usize) -> Result<Vec<u8>, FramingFailure> {
    if message.len() > limit {
        return Err(FramingFailure::Oversized);
    }
    let length = u32::try_from(message.len()).map_err(|_| FramingFailure::Oversized)?;
    let mut encoded = Vec::with_capacity(4 + message.len());
    encoded.extend_from_slice(&length.to_be_bytes());
    encoded.extend_from_slice(message);
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arbitrary_chunking_preserves_messages() {
        let bytes = [encode(b"one", 8).unwrap(), encode(b"two", 8).unwrap()].concat();
        for split in 0..=bytes.len() {
            let mut decoder = LengthDecoder::new(8, 2);
            decoder.push(&bytes[..split]).unwrap();
            decoder.push(&bytes[split..]).unwrap();
            assert_eq!(decoder.pop().unwrap(), b"one");
            assert_eq!(decoder.pop().unwrap(), b"two");
        }
    }
}
