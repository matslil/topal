//! Whole semantic values over the shared owner/version protocol. Occupancy and
//! cached scalar facts do not establish actual ordinary graph/ABI evidence.
use crate::{
    SystemsModelError, SystemsProcessIdentityTransition, SystemsRetainedIdentityStore,
    SystemsSourceLibraryInterface as Interface, SystemsSourceLibraryRequirement,
    SystemsStructuredIdentityFacts as Facts, SystemsStructuredIdentityTransition as Transition,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsStructuredIdentityStore {
    protocol: SystemsRetainedIdentityStore,
    values: [Option<Facts>; 2],
    pending: Option<Transition>,
}
fn invalid() -> SystemsModelError {
    SystemsModelError::new(
        "E-SYSTEMS-RETAINED-STATE",
        "invalid structured value owner, preparation or version",
    )
}
impl SystemsStructuredIdentityStore {
    pub fn new(
        requirements: &[SystemsSourceLibraryRequirement],
        root: &str,
    ) -> Result<Self, SystemsModelError> {
        Ok(Self {
            protocol: SystemsRetainedIdentityStore::new_for_profile(
                requirements,
                root,
                Interface::StructuredProcessIdentityV1,
            )?,
            values: [None, None],
            pending: None,
        })
    }
    pub fn lend(&mut self, root: &str, process: &str) -> Result<(), SystemsModelError> {
        if !self.is_reclaimed() {
            return Err(invalid());
        }
        self.protocol.lend(root, process)
    }
    pub fn published_facts(&self, process: &str) -> Result<Facts, SystemsModelError> {
        let compatibility = self.protocol.published_facts(process)?;
        let value = match self.values[self.protocol.published_bank()] {
            Some(value) => {
                value.validate_published()?;
                value
            }
            None => Facts::INITIAL,
        };
        if compatibility.next != value.next {
            return Err(invalid());
        }
        Ok(value)
    }
    pub fn begin(&mut self, process: &str) -> Result<Facts, SystemsModelError> {
        let value = self.published_facts(process)?;
        // Exhausted history is a published value, but cannot start another call.
        value.validate_invocable()?;
        if self.pending.is_some() || self.values[1 - self.protocol.published_bank()].is_some() {
            return Err(invalid());
        }
        self.protocol.begin(process)?;
        Ok(value)
    }
    pub fn prepare(
        &mut self,
        process: &str,
        proposal: Transition,
    ) -> Result<(), SystemsModelError> {
        proposal.state.validate_published()?;
        if self.pending.is_some() || self.values[1 - self.protocol.published_bank()].is_some() {
            return Err(invalid());
        }
        self.protocol.prepare(
            process,
            SystemsProcessIdentityTransition {
                allocated: proposal.allocated,
                next: proposal.state.next,
            },
        )?;
        self.values[1 - self.protocol.published_bank()] = Some(proposal.state);
        self.pending = Some(proposal);
        Ok(())
    }
    pub fn finish_invocation(&mut self, process: &str) -> Result<(), SystemsModelError> {
        self.protocol.finish_invocation(process)
    }
    pub fn commit(&mut self, process: &str) -> Result<Transition, SystemsModelError> {
        self.published_facts(process)?;
        let proposal = self.pending.ok_or_else(invalid)?;
        if self.values[1 - self.protocol.published_bank()] != Some(proposal.state) {
            return Err(invalid());
        }
        // Validate the staged protocol before changing either whole value bank.
        let mut staged = self.protocol.clone();
        let scalar = staged.commit(process)?;
        if scalar.allocated != proposal.allocated || scalar.next != proposal.state.next {
            return Err(invalid());
        }
        self.protocol = staged;
        self.pending = None;
        Ok(proposal)
    }
    pub fn retire_previous(&mut self, process: &str) -> Result<(), SystemsModelError> {
        self.protocol.retire_previous(process)?;
        self.values[1 - self.protocol.published_bank()] = None;
        Ok(())
    }
    pub fn retain_across_exec(&self, process: &str) -> Result<(), SystemsModelError> {
        self.published_facts(process)?;
        if self.pending.is_some() || self.values[1 - self.protocol.published_bank()].is_some() {
            return Err(invalid());
        }
        self.protocol.retain_across_exec(process)
    }
    pub fn begin_other(
        &mut self,
        process: &str,
        policy: Interface,
    ) -> Result<(), SystemsModelError> {
        self.retain_across_exec(process)?;
        self.protocol.begin_other(process, policy)
    }
    pub fn finish_other(
        &mut self,
        process: &str,
        policy: Interface,
    ) -> Result<(), SystemsModelError> {
        self.protocol.finish_other(process, policy)
    }
    pub fn return_to_root(&mut self, process: &str) -> Result<(), SystemsModelError> {
        self.protocol.return_to_root(process)?;
        self.values = [None, None];
        self.pending = None;
        Ok(())
    }
    #[must_use]
    pub fn is_reclaimed(&self) -> bool {
        self.protocol.is_reclaimed() && self.values == [None, None] && self.pending.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controlled_wide_history_remains_a_value_and_exhaustion_cannot_reenter_or_resurrect_it() {
        // TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001
        let mut store = store();
        let first = store.begin("process").unwrap().transition().unwrap();
        store.prepare("process", first).unwrap();
        store.finish_invocation("process").unwrap();
        store.commit("process").unwrap();
        store.retire_previous("process").unwrap();
        // Controlled valid-domain value seed, not a claim of naturally reached history.
        let bank = store.protocol.published_bank();
        store.values[bank] = Some(Facts {
            next: first.state.next,
            creations: u64::MAX - 1,
            wrapped: true,
        });
        let seeded = store.clone();
        assert!(store.begin("other").is_err());
        assert_eq!(store, seeded);
        let proposal = store.begin("process").unwrap().transition().unwrap();
        assert_eq!(proposal.state.creations, u64::MAX);
        assert!(proposal.state.wrapped);
        store.prepare("process", proposal).unwrap();
        store.finish_invocation("process").unwrap();
        store.commit("process").unwrap();
        store.retire_previous("process").unwrap();
        store.retain_across_exec("process").unwrap();
        let exhausted = store.clone();
        assert!(store.begin("process").is_err());
        assert_eq!(store, exhausted);
        store.return_to_root("process").unwrap();
        assert!(store.is_reclaimed());
        let mut store = self::store();
        store.values[1] = Some(first.state);
        let dirty = store.clone();
        assert!(store.begin("process").is_err());
        assert_eq!(store, dirty);
        store.return_to_root("process").unwrap();
        assert!(store.is_reclaimed());
    }
    fn store() -> SystemsStructuredIdentityStore {
        let requirement = |interface| SystemsSourceLibraryRequirement {
            source: "policy/structured-process-identity.t".into(),
            export: "process-identity".into(),
            interface,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: if interface == Interface::StructuredProcessIdentityV1 {
                Some(4096)
            } else {
                None
            },
        };
        let mut store = SystemsStructuredIdentityStore::new(
            &[
                requirement(Interface::StructuredProcessIdentityV1),
                requirement(Interface::StaticLinuxDispatchV1),
            ],
            "root",
        )
        .unwrap();
        store.lend("root", "process").unwrap();
        store
    }
    #[test]
    fn whole_history_survives_preparation_clean_publication_retirement_and_other_policies() {
        // TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001; TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001
        let mut store = store();
        for count in 0..100 {
            let previous = store.published_facts("process").unwrap();
            assert_eq!(previous.creations, count);
            let value = store.begin("process").unwrap();
            assert_eq!(value, previous);
            let proposal = value.transition().unwrap();
            store.prepare("process", proposal).unwrap();
            assert_eq!(store.published_facts("process").unwrap(), previous);
            let prepared = store.clone();
            assert!(store.commit("process").is_err());
            assert_eq!(store, prepared);
            store.finish_invocation("process").unwrap();
            assert_eq!(store.published_facts("process").unwrap(), previous);
            assert_eq!(store.commit("process").unwrap(), proposal);
            assert_eq!(store.published_facts("process").unwrap(), proposal.state);
            let unretired = store.clone();
            assert!(store.begin("process").is_err());
            assert!(store.retain_across_exec("process").is_err());
            assert_eq!(store, unretired);
            store.retire_previous("process").unwrap();
            store.retain_across_exec("process").unwrap();
            store
                .begin_other("process", Interface::StaticLinuxDispatchV1)
                .unwrap();
            let other = store.clone();
            assert!(store.begin("process").is_err());
            assert!(store.return_to_root("process").is_err());
            assert_eq!(store, other);
            store
                .finish_other("process", Interface::StaticLinuxDispatchV1)
                .unwrap();
        }
        store.return_to_root("process").unwrap();
        assert!(store.is_reclaimed());
    }
    #[test]
    fn invalid_owner_shape_overlap_stale_commit_and_all_termination_phases_preserve_the_protocol() {
        // TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001
        for phase in 0..5 {
            let mut store = store();
            let initial = store.clone();
            assert!(store.begin("other").is_err());
            assert!(store.commit("process").is_err());
            assert!(store.return_to_root("other").is_err());
            assert_eq!(store, initial);
            if phase > 0 {
                let proposal = store.begin("process").unwrap().transition().unwrap();
                let active = store.clone();
                assert!(store.begin("process").is_err());
                let mut malformed = proposal;
                malformed.state.creations = 0;
                assert!(store.prepare("process", malformed).is_err());
                assert_eq!(store, active);
                if phase > 1 {
                    store.prepare("process", proposal).unwrap();
                    let prepared = store.clone();
                    assert!(store.prepare("process", proposal).is_err());
                    assert!(store.commit("process").is_err());
                    assert_eq!(store, prepared);
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
            let returned = store.clone();
            assert!(store.return_to_root("process").is_err());
            assert_eq!(store, returned);
        }
        let mut store = store();
        let proposal = store.begin("process").unwrap().transition().unwrap();
        store.prepare("process", proposal).unwrap();
        store.finish_invocation("process").unwrap();
        store.commit("process").unwrap();
        store.retire_previous("process").unwrap();
        let complete = store.clone();
        assert!(store.commit("process").is_err());
        assert!(store.retire_previous("process").is_err());
        assert_eq!(store, complete);
    }
}
