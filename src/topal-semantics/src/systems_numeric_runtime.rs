//! Reference lifetime/erasure model. Labels and offsets grant no machine authority.
use crate::{SystemsModelError, SystemsSourceLibraryInterface, SystemsSourceLibraryRequirement};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsNumericRuntimeArena {
    root_owner: String,
    process: Option<String>,
    active: bool,
    cursor: u64,
    allocations: u64,
    storage: Vec<u8>,
    stack: Vec<u8>,
    binding_erased: bool,
}
fn invalid() -> SystemsModelError {
    SystemsModelError::new(
        "E-SYSTEMS-NUMERIC-RUNTIME",
        "invalid owner, activity, allocation or erasure state",
    )
}
impl SystemsNumericRuntimeArena {
    pub fn new(
        requirement: &SystemsSourceLibraryRequirement,
        root_owner: &str,
    ) -> Result<Self, SystemsModelError> {
        requirement.validate()?;
        if requirement.interface != SystemsSourceLibraryInterface::RelativeSleepDeadlineV1
            || root_owner.is_empty()
        {
            return Err(invalid());
        }
        Ok(Self {
            root_owner: root_owner.into(),
            process: None,
            active: false,
            cursor: 0,
            allocations: 0,
            storage: vec![0; 65536],
            stack: vec![0; 4096],
            binding_erased: true,
        })
    }
    fn clean(&self) -> bool {
        self.storage.iter().chain(&self.stack).all(|b| *b == 0)
    }
    fn process_matches(&self, process: &str) -> bool {
        self.process.as_deref() == Some(process)
    }
    pub fn lend(&mut self, root_owner: &str, process: &str) -> Result<(), SystemsModelError> {
        if root_owner != self.root_owner
            || process.is_empty()
            || self.process.is_some()
            || self.active
            || self.cursor != 0
            || self.allocations != 0
            || !self.binding_erased
            || !self.clean()
        {
            return Err(invalid());
        }
        self.process = Some(process.into());
        self.binding_erased = false;
        Ok(())
    }
    pub fn retain_across_exec(&self, process: &str) -> Result<(), SystemsModelError> {
        if !self.process_matches(process)
            || self.active
            || self.cursor != 0
            || self.allocations != 0
            || !self.clean()
            || self.binding_erased
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn begin(&mut self, process: &str) -> Result<(), SystemsModelError> {
        if !self.process_matches(process)
            || self.active
            || self.cursor != 0
            || self.allocations != 0
            || !self.clean()
            || self.binding_erased
        {
            return Err(invalid());
        }
        self.active = true;
        self.cursor = 32;
        self.allocations = 0;
        self.storage[..32].fill(0xa5);
        self.stack.fill(0xa5);
        Ok(())
    }
    /// Return a modeled byte offset, never an addressed-storage capability.
    pub fn allocate(&mut self, process: &str, length: u64) -> Result<u64, SystemsModelError> {
        if !self.process_matches(process) || !self.active || length == 0 {
            return Err(invalid());
        }
        let aligned = length.checked_add(7).ok_or_else(invalid)? & !7;
        let next = self
            .cursor
            .checked_add(aligned)
            .filter(|end| *end <= 65536)
            .ok_or_else(invalid)?;
        let previous = self.cursor;
        self.storage[previous as usize..next as usize].fill(0xcc);
        self.cursor = next;
        self.allocations += 1;
        Ok(previous)
    }
    pub fn finish(&mut self, process: &str) -> Result<(), SystemsModelError> {
        if !self.process_matches(process) || !self.active {
            return Err(invalid());
        }
        self.storage.fill(0);
        self.stack.fill(0);
        if !self.clean() {
            return Err(invalid());
        }
        self.active = false;
        self.cursor = 0;
        self.allocations = 0;
        Ok(())
    }
    pub fn return_to_root(&mut self, process: &str) -> Result<(), SystemsModelError> {
        if !self.process_matches(process)
            || self.active
            || !self.clean()
            || self.cursor != 0
            || self.allocations != 0
        {
            return Err(invalid());
        }
        self.process = None;
        self.binding_erased = true;
        Ok(())
    }
    #[must_use]
    pub fn is_reclaimed(&self) -> bool {
        self.process.is_none()
            && !self.active
            && self.cursor == 0
            && self.allocations == 0
            && self.clean()
            && self.binding_erased
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_storage_lending_is_exclusive_and_completely_erased() {
        // TOPAL-SYSTEMS-NUMERIC-RUNTIME-001
        let requirement = SystemsSourceLibraryRequirement {
            source: "policy/time.t".into(),
            export: "deadline".into(),
            interface: SystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
        };
        let mut arena = SystemsNumericRuntimeArena::new(&requirement, "root").unwrap();
        assert!(arena.is_reclaimed());
        assert!(arena.lend("other", "process").is_err());
        arena.lend("root", "process").unwrap();
        assert!(arena.lend("root", "second").is_err());
        assert!(arena.begin("other").is_err());
        for _ in 0..3 {
            arena.retain_across_exec("process").unwrap();
            arena.begin("process").unwrap();
            assert!(arena.begin("process").is_err());
            assert!(arena.return_to_root("process").is_err());
            assert_eq!(arena.allocate("process", 1).unwrap(), 32);
            assert_eq!(arena.allocate("process", 24).unwrap(), 40);
            assert!(arena.allocate("other", 8).is_err());
            assert!(arena.allocate("process", 0).is_err());
            assert!(arena.allocate("process", u64::MAX).is_err());
            let before = arena.clone();
            assert!(arena.allocate("process", 65536).is_err());
            assert_eq!(arena, before);
            arena.finish("process").unwrap();
            assert!(arena.clean());
        }
        arena.storage[1] = 7;
        assert!(arena.return_to_root("process").is_err());
        arena.storage[1] = 0;
        arena.return_to_root("process").unwrap();
        assert!(arena.is_reclaimed());
        assert!(arena.begin("process").is_err());
        assert!(arena.finish("process").is_err());
    }
}
