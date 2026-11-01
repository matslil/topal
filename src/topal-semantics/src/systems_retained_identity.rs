//! Owned immutable versions. Occupancy markers model reclamation, not a native
//! Record/pointer layout; actual graph, ABI and frame evidence is separate.
use crate::{
    SystemsModelError, SystemsNumericRuntimeArena, SystemsProcessIdentityFacts,
    SystemsProcessIdentityTransition, SystemsSourceLibraryInterface as Interface,
    SystemsSourceLibraryRequirement,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsRetainedIdentityStore {
    interface: Interface,
    graph_bytes: usize,
    root: String,
    process: Option<String>,
    temporary: SystemsNumericRuntimeArena,
    published_bank: usize,
    epoch: u64,
    values: [Option<u64>; 2],
    active: bool,
    prepared: Option<SystemsProcessIdentityTransition>,
    retired: Option<usize>,
    storage: Vec<u8>,
}
fn invalid() -> SystemsModelError {
    SystemsModelError::new(
        "E-SYSTEMS-RETAINED-STATE",
        "invalid store owner, version, preparation, publication or reclamation state",
    )
}
impl SystemsRetainedIdentityStore {
    pub fn new(
        requirements: &[SystemsSourceLibraryRequirement],
        root: &str,
    ) -> Result<Self, SystemsModelError> {
        Self::new_for_profile(requirements, root, Interface::RetainedProcessIdentityV1)
    }
    /// Shared owner/version protocol. The structured wrapper separately retains
    /// its whole semantic value; this layer also checks its compatibility cursor.
    pub(crate) fn new_for_profile(
        requirements: &[SystemsSourceLibraryRequirement],
        root: &str,
        interface: Interface,
    ) -> Result<Self, SystemsModelError> {
        let graph_bytes = match interface {
            Interface::RetainedProcessIdentityV1 => 40,
            Interface::StructuredProcessIdentityV1 => 88,
            _ => return Err(invalid()),
        };
        if requirements
            .iter()
            .filter(|r| r.interface == interface)
            .count()
            != 1
        {
            return Err(invalid());
        }
        Ok(Self {
            interface,
            graph_bytes,
            root: root.into(),
            process: None,
            temporary: SystemsNumericRuntimeArena::new_with_policies(requirements, root)?,
            published_bank: 0,
            epoch: 0,
            values: [None, None],
            active: false,
            prepared: None,
            retired: None,
            storage: vec![0; 4096],
        })
    }
    fn owner(&self, process: &str) -> Result<(), SystemsModelError> {
        if self.process.as_deref() != Some(process) {
            return Err(invalid());
        }
        Ok(())
    }
    fn bank_range(bank: usize) -> std::ops::Range<usize> {
        let start = 64 + bank * 2016;
        start..start + 2016
    }
    pub(crate) fn published_bank(&self) -> usize {
        self.published_bank
    }
    fn inactive_clean(&self) -> bool {
        let bank = 1 - self.published_bank;
        self.values[bank].is_none() && self.storage[Self::bank_range(bank)].iter().all(|b| *b == 0)
    }
    pub fn lend(&mut self, root: &str, process: &str) -> Result<(), SystemsModelError> {
        if root != self.root || !self.is_reclaimed() {
            return Err(invalid());
        }
        self.temporary.lend(root, process)?;
        self.process = Some(process.into());
        self.storage[0] = 1;
        Ok(())
    }
    /// A model fact carries no pointer/borrow authority. Native qualification
    /// must observe the actual published ordinary graph at the source boundary.
    pub fn published_facts(
        &self,
        process: &str,
    ) -> Result<SystemsProcessIdentityFacts, SystemsModelError> {
        self.owner(process)?;
        Ok(SystemsProcessIdentityFacts {
            next: self.values[self.published_bank].unwrap_or(0),
        })
    }
    pub fn begin(
        &mut self,
        process: &str,
    ) -> Result<SystemsProcessIdentityFacts, SystemsModelError> {
        self.owner(process)?;
        if self.active
            || self.prepared.is_some()
            || self.retired.is_some()
            || !self.inactive_clean()
        {
            return Err(invalid());
        }
        let facts = self.published_facts(process)?;
        self.temporary.begin_policy(process, self.interface)?;
        self.active = true;
        Ok(facts)
    }
    /// This is storage/protocol validation. The independent source policy law
    /// is SystemsProcessIdentityFacts::transition, not duplicated owner policy.
    pub fn prepare(
        &mut self,
        process: &str,
        proposal: SystemsProcessIdentityTransition,
    ) -> Result<(), SystemsModelError> {
        self.owner(process)?;
        if !self.active
            || self.prepared.is_some()
            || self.retired.is_some()
            || !self.inactive_clean()
            || !(2..=i32::MAX as u64).contains(&proposal.allocated)
            || !(2..=i32::MAX as u64).contains(&proposal.next)
        {
            return Err(invalid());
        }
        let bank = 1 - self.published_bank;
        self.values[bank] = Some(proposal.next);
        // An abstract graph occupancy mark, deliberately not a native encoding.
        let range = Self::bank_range(bank);
        self.storage[range.start..range.start + self.graph_bytes].fill(0x5a);
        self.prepared = Some(proposal);
        Ok(())
    }
    pub fn finish_invocation(&mut self, process: &str) -> Result<(), SystemsModelError> {
        self.owner(process)?;
        if !self.active || self.prepared.is_none() {
            return Err(invalid());
        }
        self.temporary.finish_policy(process, self.interface)?;
        self.active = false;
        Ok(())
    }
    pub fn commit(
        &mut self,
        process: &str,
    ) -> Result<SystemsProcessIdentityTransition, SystemsModelError> {
        self.owner(process)?;
        let proposal = self.prepared.ok_or_else(invalid)?;
        let epoch = self.epoch.checked_add(1).ok_or_else(invalid)?;
        if self.active
            || self.retired.is_some()
            || self.values[1 - self.published_bank] != Some(proposal.next)
        {
            return Err(invalid());
        }
        self.temporary.retain_across_exec(process)?;
        let old = self.published_bank;
        self.published_bank = 1 - old;
        self.epoch = epoch;
        self.prepared = None;
        self.retired = Some(old);
        Ok(proposal)
    }
    pub fn retire_previous(&mut self, process: &str) -> Result<(), SystemsModelError> {
        self.owner(process)?;
        let old = self.retired.ok_or_else(invalid)?;
        if self.active || self.prepared.is_some() || old == self.published_bank {
            return Err(invalid());
        }
        self.temporary.retain_across_exec(process)?;
        self.values[old] = None;
        self.storage[Self::bank_range(old)].fill(0);
        self.retired = None;
        Ok(())
    }
    pub fn retain_across_exec(&self, process: &str) -> Result<(), SystemsModelError> {
        self.owner(process)?;
        if self.active
            || self.prepared.is_some()
            || self.retired.is_some()
            || !self.inactive_clean()
        {
            return Err(invalid());
        }
        self.temporary.retain_across_exec(process)
    }
    pub fn begin_other(
        &mut self,
        process: &str,
        policy: Interface,
    ) -> Result<(), SystemsModelError> {
        self.retain_across_exec(process)?;
        if policy == self.interface {
            return Err(invalid());
        }
        self.temporary.begin_policy(process, policy)
    }
    pub fn finish_other(
        &mut self,
        process: &str,
        policy: Interface,
    ) -> Result<(), SystemsModelError> {
        self.owner(process)?;
        if policy == self.interface {
            return Err(invalid());
        }
        self.temporary.finish_policy(process, policy)
    }
    pub fn return_to_root(&mut self, process: &str) -> Result<(), SystemsModelError> {
        self.owner(process)?;
        // Validate/clean a staged owner so a rejected return never mutates it.
        let mut returned = self.clone();
        if returned.active {
            returned.temporary.finish_policy(process, self.interface)?;
            returned.active = false;
        }
        returned.temporary.return_to_root(process)?;
        returned.values = [None, None];
        returned.storage.fill(0);
        returned.process = None;
        returned.prepared = None;
        returned.retired = None;
        returned.published_bank = 0;
        returned.epoch = 0;
        *self = returned;
        Ok(())
    }
    #[must_use]
    pub fn is_reclaimed(&self) -> bool {
        self.process.is_none()
            && !self.active
            && self.prepared.is_none()
            && self.retired.is_none()
            && self.values == [None, None]
            && self.epoch == 0
            && self.published_bank == 0
            && self.storage.iter().all(|b| *b == 0)
            && self.temporary.is_reclaimed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn requirement(interface: Interface) -> SystemsSourceLibraryRequirement {
        SystemsSourceLibraryRequirement {
            source: "policy/process-identity.t".into(),
            export: "process-identity".into(),
            interface,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: if interface == Interface::RetainedProcessIdentityV1 {
                Some(4096)
            } else {
                None
            },
        }
    }
    fn store() -> SystemsRetainedIdentityStore {
        let mut store = SystemsRetainedIdentityStore::new(
            &[
                requirement(Interface::RetainedProcessIdentityV1),
                requirement(Interface::StaticLinuxDispatchV1),
            ],
            "root",
        )
        .unwrap();
        store.lend("root", "process").unwrap();
        store
    }
    #[test]
    fn versions_keep_old_state_until_clean_commit_and_retire_after_last_borrow() {
        // TOPAL-SYSTEMS-RETAINED-NUMERIC-STATE-001; TOPAL-SYSTEMS-RETAINED-PROCESS-IDENTITY-001
        let mut store = store();
        for expected in 2..102 {
            let old = store.published_facts("process").unwrap();
            let facts = store.begin("process").unwrap();
            assert_eq!(facts, old);
            let proposal = facts.transition().unwrap();
            assert_eq!(proposal.allocated, expected);
            store.prepare("process", proposal).unwrap();
            assert_eq!(store.published_facts("process").unwrap(), old);
            let before = store.clone();
            assert!(store.commit("process").is_err());
            assert_eq!(before, store);
            assert!(store.retain_across_exec("process").is_err());
            store.finish_invocation("process").unwrap();
            assert_eq!(store.published_facts("process").unwrap(), old);
            assert_eq!(store.commit("process").unwrap(), proposal);
            assert_eq!(store.published_facts("process").unwrap().next, expected + 1);
            assert!(store.begin("process").is_err());
            assert!(store.retain_across_exec("process").is_err());
            store.retire_previous("process").unwrap();
            assert_eq!(store.epoch, expected - 1);
            store.retain_across_exec("process").unwrap();
            store
                .begin_other("process", Interface::StaticLinuxDispatchV1)
                .unwrap();
            let before = store.clone();
            assert!(store.begin("process").is_err());
            assert_eq!(before, store);
            store
                .finish_other("process", Interface::StaticLinuxDispatchV1)
                .unwrap();
        }
        store.return_to_root("process").unwrap();
        assert!(store.is_reclaimed());
    }
    #[test]
    fn wrong_owner_overlap_stale_publication_and_invalid_returns_preserve_state() {
        // TOPAL-SYSTEMS-RETAINED-NUMERIC-STATE-001
        let mut store = store();
        let before = store.clone();
        assert!(store.begin("other").is_err());
        assert!(store.commit("process").is_err());
        assert!(store.retire_previous("process").is_err());
        assert!(store.return_to_root("other").is_err());
        assert_eq!(store, before);
        let facts = store.begin("process").unwrap();
        let proposal = facts.transition().unwrap();
        let active = store.clone();
        assert!(store.begin("process").is_err());
        assert!(store.finish_invocation("process").is_err());
        assert!(store.prepare("other", proposal).is_err());
        assert!(
            store
                .prepare(
                    "process",
                    SystemsProcessIdentityTransition {
                        allocated: 1,
                        next: 3
                    }
                )
                .is_err()
        );
        assert_eq!(store, active);
        store.prepare("process", proposal).unwrap();
        let prepared = store.clone();
        assert!(store.prepare("process", proposal).is_err());
        assert!(store.commit("process").is_err());
        assert!(
            store
                .begin_other("process", Interface::StaticLinuxDispatchV1)
                .is_err()
        );
        assert_eq!(store, prepared);
        store.finish_invocation("process").unwrap();
        store.commit("process").unwrap();
        store.retire_previous("process").unwrap();
        let published = store.clone();
        assert!(store.commit("process").is_err());
        assert!(store.retire_previous("process").is_err());
        assert_eq!(store, published);
        assert_eq!(store.begin("process").unwrap().next, 3);
        store.return_to_root("process").unwrap();
        assert!(store.is_reclaimed());
        let returned = store.clone();
        assert!(store.return_to_root("process").is_err());
        assert_eq!(store, returned);
    }
    #[test]
    fn termination_reclaims_active_prepared_and_published_versions_once() {
        // TOPAL-SYSTEMS-RETAINED-PROCESS-IDENTITY-001
        for phase in 0..5 {
            let mut store = store();
            if phase > 0 {
                let proposal = store.begin("process").unwrap().transition().unwrap();
                if phase > 1 {
                    store.prepare("process", proposal).unwrap();
                }
                if phase > 2 {
                    store.finish_invocation("process").unwrap();
                }
                if phase > 3 {
                    store.commit("process").unwrap();
                }
            }
            store.return_to_root("process").unwrap();
            assert!(store.is_reclaimed());
        }
    }
    #[test]
    fn other_policy_activity_and_epoch_exhaustion_reject_without_partial_commit() {
        // TOPAL-SYSTEMS-RETAINED-NUMERIC-STATE-001
        let mut store = store();
        store
            .begin_other("process", Interface::StaticLinuxDispatchV1)
            .unwrap();
        let active_other = store.clone();
        assert!(store.return_to_root("process").is_err());
        assert_eq!(store, active_other);
        store
            .finish_other("process", Interface::StaticLinuxDispatchV1)
            .unwrap();
        let proposal = store.begin("process").unwrap().transition().unwrap();
        store.prepare("process", proposal).unwrap();
        store.finish_invocation("process").unwrap();
        store.epoch = u64::MAX;
        let prepared = store.clone();
        assert!(store.commit("process").is_err());
        assert_eq!(store, prepared);
        store.return_to_root("process").unwrap();
        assert!(store.is_reclaimed());
    }
    #[test]
    fn malformed_store_requirements_and_unclean_inactive_storage_are_rejected() {
        // TOPAL-SYSTEMS-RETAINED-PROCESS-IDENTITY-001
        let valid = requirement(Interface::RetainedProcessIdentityV1);
        for limit in [None, Some(0), Some(4095), Some(4097)] {
            let mut changed = valid.clone();
            changed.state_limit_bytes = limit;
            assert!(SystemsRetainedIdentityStore::new(&[changed], "root").is_err());
        }
        assert!(SystemsRetainedIdentityStore::new(&[], "root").is_err());
        assert!(
            SystemsRetainedIdentityStore::new(
                &[
                    valid.clone(),
                    requirement(Interface::ProcessIdentityTransitionV1)
                ],
                "root"
            )
            .is_err()
        );
        assert!(SystemsRetainedIdentityStore::new(&[valid], "").is_err());
        let mut store = store();
        store.storage[2080 + 100] = 1;
        let corrupt = store.clone();
        assert!(store.begin("process").is_err());
        assert_eq!(store, corrupt);
        store.return_to_root("process").unwrap();
        assert!(store.is_reclaimed());
    }
}
