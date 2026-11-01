//! Reference lifetime/erasure model. Labels and offsets grant no machine authority.
use crate::{SystemsModelError, SystemsSourceLibraryInterface, SystemsSourceLibraryRequirement};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsNumericRuntimeArena {
    root_owner: String,
    process: Option<String>,
    active: bool,
    policies: BTreeSet<SystemsSourceLibraryInterface>,
    active_policy: Option<SystemsSourceLibraryInterface>,
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
        Self::new_with_policies(std::slice::from_ref(requirement), root_owner)
    }
    pub fn new_with_policies(
        requirements: &[SystemsSourceLibraryRequirement],
        root_owner: &str,
    ) -> Result<Self, SystemsModelError> {
        let mut policies = BTreeSet::new();
        let mut identity_selected = false;
        let mut dispatch_selected = false;
        for requirement in requirements {
            requirement.validate()?;
            if matches!(
                requirement.interface,
                SystemsSourceLibraryInterface::StaticLinuxDispatchV1
                    | SystemsSourceLibraryInterface::StaticLinuxDispatchV2
            ) && std::mem::replace(&mut dispatch_selected, true)
            {
                return Err(invalid());
            }
            if matches!(
                requirement.interface,
                SystemsSourceLibraryInterface::ProcessIdentityTransitionV1
                    | SystemsSourceLibraryInterface::RetainedProcessIdentityV1
                    | SystemsSourceLibraryInterface::StructuredProcessIdentityV1
            ) && std::mem::replace(&mut identity_selected, true)
            {
                return Err(invalid());
            }
            if !matches!(
                requirement.interface,
                SystemsSourceLibraryInterface::RelativeSleepDeadlineV1
                    | SystemsSourceLibraryInterface::StaticElf64HeaderV1
                    | SystemsSourceLibraryInterface::StaticLinuxDispatchV1
                    | SystemsSourceLibraryInterface::StaticLinuxDispatchV2
                    | SystemsSourceLibraryInterface::ProcessIdentityTransitionV1
                    | SystemsSourceLibraryInterface::RetainedProcessIdentityV1
                    | SystemsSourceLibraryInterface::StructuredProcessIdentityV1
            ) || !policies.insert(requirement.interface)
            {
                return Err(invalid());
            }
        }
        if policies.is_empty() || root_owner.is_empty() {
            return Err(invalid());
        }
        Ok(Self {
            root_owner: root_owner.into(),
            process: None,
            active: false,
            policies,
            active_policy: None,
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
            || self.active_policy.is_some()
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
            || self.active_policy.is_some()
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
        if self.policies.len() != 1 {
            return Err(invalid());
        }
        self.begin_policy(process, *self.policies.first().unwrap())
    }
    pub fn begin_policy(
        &mut self,
        process: &str,
        policy: SystemsSourceLibraryInterface,
    ) -> Result<(), SystemsModelError> {
        if !self.process_matches(process)
            || !self.policies.contains(&policy)
            || self.active_policy.is_some()
            || self.active
            || self.cursor != 0
            || self.allocations != 0
            || !self.clean()
            || self.binding_erased
        {
            return Err(invalid());
        }
        self.active = true;
        self.active_policy = Some(policy);
        self.cursor = 32;
        self.allocations = 0;
        self.storage[..32].fill(0xa5);
        self.stack.fill(0xa5);
        Ok(())
    }
    /// Return a modeled byte offset, never an addressed-storage capability.
    pub fn allocate(&mut self, process: &str, length: u64) -> Result<u64, SystemsModelError> {
        if !self.process_matches(process)
            || !self.active
            || length == 0
            || self
                .active_policy
                .is_none_or(|policy| !self.policies.contains(&policy))
        {
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
        self.finish_policy(process, self.active_policy.ok_or_else(invalid)?)
    }
    pub fn finish_policy(
        &mut self,
        process: &str,
        policy: SystemsSourceLibraryInterface,
    ) -> Result<(), SystemsModelError> {
        if !self.process_matches(process) || !self.active || self.active_policy != Some(policy) {
            return Err(invalid());
        }
        self.storage.fill(0);
        self.stack.fill(0);
        if !self.clean() {
            return Err(invalid());
        }
        self.active = false;
        self.active_policy = None;
        self.cursor = 0;
        self.allocations = 0;
        Ok(())
    }
    pub fn return_to_root(&mut self, process: &str) -> Result<(), SystemsModelError> {
        if !self.process_matches(process)
            || self.active
            || self.active_policy.is_some()
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
            && self.active_policy.is_none()
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
    fn interface_versions_share_their_semantic_policy_slots() {
        // TOPAL-SYSTEMS-RETAINED-PROCESS-IDENTITY-001
        use SystemsSourceLibraryInterface::*;
        let policies = [
            RelativeSleepDeadlineV1,
            StaticElf64HeaderV1,
            StaticLinuxDispatchV1,
            ProcessIdentityTransitionV1,
            RetainedProcessIdentityV1,
            StructuredProcessIdentityV1,
            StaticLinuxDispatchV2,
        ];
        let requirement = |interface| SystemsSourceLibraryRequirement {
            source: "policy/identity.t".into(),
            export: "process-identity".into(),
            interface,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: if matches!(
                interface,
                RetainedProcessIdentityV1 | StructuredProcessIdentityV1
            ) {
                Some(4096)
            } else {
                None
            },
        };
        let mut accepted = 0;
        for mask in 1u32..128 {
            let selected = policies
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, interface)| requirement(*interface))
                .collect::<Vec<_>>();
            let result = SystemsNumericRuntimeArena::new_with_policies(&selected, "root");
            if (mask & 56).count_ones() > 1 || (mask & 68).count_ones() > 1 {
                assert!(result.is_err());
                continue;
            }
            accepted += 1;
            let mut arena = result.unwrap();
            arena.lend("root", "process").unwrap();
            for policy in &selected {
                arena.begin_policy("process", policy.interface).unwrap();
                arena.allocate("process", 24).unwrap();
                arena.finish_policy("process", policy.interface).unwrap();
            }
            arena.return_to_root("process").unwrap();
            assert!(arena.is_reclaimed());
        }
        assert_eq!(accepted, 47);
    }
    #[test]
    fn distinct_numeric_policies_share_only_inactive_clean_storage() {
        // TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001
        let requirement = |interface| SystemsSourceLibraryRequirement {
            source: "policy/unit.t".into(),
            export: "policy".into(),
            interface,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: None,
        };
        let time = SystemsSourceLibraryInterface::RelativeSleepDeadlineV1;
        let header = SystemsSourceLibraryInterface::StaticElf64HeaderV1;
        let dispatch = SystemsSourceLibraryInterface::StaticLinuxDispatchV1;
        let policies = [
            time,
            header,
            dispatch,
            SystemsSourceLibraryInterface::ProcessIdentityTransitionV1,
        ];
        for mask in 1..16 {
            let selected = policies
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, interface)| requirement(*interface))
                .collect::<Vec<_>>();
            let mut arena =
                SystemsNumericRuntimeArena::new_with_policies(&selected, "root").unwrap();
            arena.lend("root", "process").unwrap();
            assert!(arena.begin_policy("other", header).is_err());
            assert!(
                arena
                    .begin_policy("process", SystemsSourceLibraryInterface::BootOpenDecisionV1)
                    .is_err()
            );
            for policy in selected.iter().cycle().take(4).map(|r| r.interface) {
                arena.begin_policy("process", policy).unwrap();
                let before = arena.clone();
                let other = if policy == header { time } else { header };
                assert!(arena.begin_policy("process", other).is_err());
                assert!(arena.finish_policy("process", other).is_err());
                assert_eq!(arena, before);
                arena.allocate("process", 24).unwrap();
                arena.finish_policy("process", policy).unwrap();
                arena.retain_across_exec("process").unwrap();
            }
            arena.return_to_root("process").unwrap();
            assert!(arena.is_reclaimed());
        }
        assert!(SystemsNumericRuntimeArena::new_with_policies(&[], "root").is_err());
        assert!(
            SystemsNumericRuntimeArena::new_with_policies(
                &[requirement(time), requirement(time)],
                "root"
            )
            .is_err()
        );
    }
    #[test]
    fn numeric_storage_lending_is_exclusive_and_completely_erased() {
        // TOPAL-SYSTEMS-NUMERIC-RUNTIME-001
        let requirement = SystemsSourceLibraryRequirement {
            source: "policy/time.t".into(),
            export: "deadline".into(),
            interface: SystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: None,
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
