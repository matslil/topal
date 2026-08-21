//! Virtual device controller and explicit DMA obligations.
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaRequirements {
    pub alignment: usize,
    pub maximum_length: usize,
    pub coherent: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceFailure {
    Misaligned,
    Oversized,
    Removed,
    QueueExhausted,
}
#[derive(Debug)]
pub struct VirtualController {
    requirements: DmaRequirements,
    online: bool,
    queue: VecDeque<Vec<u8>>,
    limit: usize,
}
impl VirtualController {
    #[must_use]
    pub const fn new(requirements: DmaRequirements, limit: usize) -> Self {
        Self {
            requirements,
            online: true,
            queue: VecDeque::new(),
            limit,
        }
    }
    /// Submits an owned transfer buffer.
    /// # Errors
    /// Returns exact lifetime, alignment, size, or queue failures.
    pub fn submit(&mut self, address: usize, buffer: Vec<u8>) -> Result<(), DeviceFailure> {
        if !self.online {
            return Err(DeviceFailure::Removed);
        }
        if self.requirements.alignment == 0 || !address.is_multiple_of(self.requirements.alignment)
        {
            return Err(DeviceFailure::Misaligned);
        }
        if buffer.len() > self.requirements.maximum_length {
            return Err(DeviceFailure::Oversized);
        }
        if self.queue.len() >= self.limit {
            return Err(DeviceFailure::QueueExhausted);
        }
        self.queue.push_back(buffer);
        Ok(())
    }
    pub fn remove(&mut self) {
        self.online = false;
    }
    pub fn complete(&mut self) -> Option<Vec<u8>> {
        self.queue.pop_front()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_survives_removal() {
        let mut c = VirtualController::new(
            DmaRequirements {
                alignment: 4,
                maximum_length: 4,
                coherent: true,
            },
            1,
        );
        c.submit(4, vec![1]).unwrap();
        c.remove();
        assert_eq!(c.submit(4, vec![2]), Err(DeviceFailure::Removed));
        assert_eq!(c.complete(), Some(vec![1]));
    }
}
