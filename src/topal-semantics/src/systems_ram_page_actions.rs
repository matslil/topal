//! Real allocation-map loans; model facts and cleanup flags are not native proof.
use super::*;
use crate::{
    RegionActionReclamation, RegionLibraryBinding, RegionLibraryError, RegionLibraryInvocation,
    RegionLibraryView,
};

/// An internal controller request, not a Linux syscall or numeric memory grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemsRamPageAction {
    Allocate { logical_page: u64 },
    Release { slot: u64 },
}

#[derive(Debug)]
enum Transition {
    Allocate { object: u64, index: u64 },
    Release { slot: usize },
}

/// Holds the owner's real map and associated storage under one Rust borrow.
/// Its private construction validates live authority before release preparation.
#[derive(Debug)]
pub(crate) struct PageActionStorage<'a> {
    state: &'a mut State,
    controller: String,
    identity: Arc<()>,
    before: [u8; PAGES],
    transition: Transition,
    inputs: (u64, u64),
}

impl SystemsVolatileFilePool {
    /// Model an already admitted internal page operation on the actual owner.
    /// This does not execute source or prove a complete Linux request's admission.
    /// Native write/truncate/destruction must separately establish that admission.
    pub fn borrow_page_action_after_admission<'a>(
        &'a mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        request: SystemsRamPageAction,
        binding: &'a RegionLibraryBinding,
    ) -> Result<(RegionLibraryInvocation<'a>, RegionLibraryView), RegionLibraryError> {
        // Includes actual pool/file identity, inactive domains and canonical map.
        let description = self
            .description(owner, file)
            .map_err(|_| RegionLibraryError::Owner)?;
        let description = &self.state.descriptions[&description];
        let object = self
            .state
            .object(description.object)
            .ok_or(RegionLibraryError::Owner)?
            .identity;
        let (transition, inputs) = match request {
            SystemsRamPageAction::Allocate { logical_page } => {
                if description.rights == SystemsFileRights::Read {
                    return Err(RegionLibraryError::Owner);
                }
                if logical_page > MAX_OFFSET / PAGE as u64 {
                    return Err(RegionLibraryError::Bounds);
                }
                if self
                    .state
                    .page_records()
                    .any(|page| page.owner == Some(object) && page.index == logical_page)
                {
                    return Err(RegionLibraryError::Owner);
                }
                if !self.state.page_allocation().contains(&0) {
                    return Err(RegionLibraryError::Capacity);
                }
                (
                    Transition::Allocate {
                        object,
                        index: logical_page,
                    },
                    (0, 0),
                )
            }
            SystemsRamPageAction::Release { slot } => {
                let index = usize::try_from(slot).map_err(|_| RegionLibraryError::Bounds)?;
                if index >= PAGES {
                    return Err(RegionLibraryError::Bounds);
                }
                if self.state.page_allocation()[index] != 1
                    || self.state.page_owner(index) != Some(object)
                {
                    return Err(RegionLibraryError::Owner);
                }
                (Transition::Release { slot: index }, (1, slot))
            }
        };
        let generation = self
            .state
            .page_actions
            .generation
            .checked_add(1)
            .ok_or(RegionLibraryError::IdentityExhausted)?;
        // No fallible admission follows this point. In particular, exhausted
        // generations and foreign resources cannot erase a release payload.
        if let Transition::Release { slot } = transition {
            self.state.pages[slot].bytes.fill(0);
            debug_assert!(self.state.pages[slot].bytes.iter().all(|byte| *byte == 0));
        }
        let before = self.state.page_allocation();
        self.state.page_actions.generation = generation;
        self.state.page_actions.active = true;
        let storage = PageActionStorage {
            state: &mut self.state,
            controller: self.owner.clone(),
            identity: self.identity.clone(),
            before,
            transition,
            inputs,
        };
        Ok(RegionLibraryInvocation::begin_page(
            storage, binding, generation,
        ))
    }
}

impl PageActionStorage<'_> {
    pub(crate) fn owner(&self) -> (String, u64) {
        // Distinguishes page-map views from position views, even with the same
        // model graph label, pool identity and numeric invocation generation.
        (self.controller.clone(), 1)
    }
    pub(crate) fn identity(&self) -> Arc<()> {
        self.identity.clone()
    }
    pub(crate) fn controller(&self) -> &str {
        &self.controller
    }
    pub(crate) fn inputs(&self) -> (u64, u64) {
        self.inputs
    }
    pub(crate) fn load(&self, offset: u64) -> Result<u8, RegionLibraryError> {
        if !self.state.page_actions.active || self.state.page_actions.poisoned {
            return Err(RegionLibraryError::Owner);
        }
        let offset = usize::try_from(offset).map_err(|_| RegionLibraryError::Bounds)?;
        self.state
            .page_allocation()
            .get(offset)
            .copied()
            .ok_or(RegionLibraryError::Bounds)
    }
    pub(crate) fn store(&mut self, offset: u64, value: u8) -> Result<(), RegionLibraryError> {
        if !self.state.page_actions.active || self.state.page_actions.poisoned {
            return Err(RegionLibraryError::Owner);
        }
        let offset = usize::try_from(offset).map_err(|_| RegionLibraryError::Bounds)?;
        *self
            .state
            .page_allocation_mut()
            .get_mut(offset)
            .ok_or(RegionLibraryError::Bounds)? = value;
        Ok(())
    }
    pub(crate) fn finish(
        &mut self,
        result: bool,
        proof: RegionActionReclamation,
    ) -> Result<u64, RegionLibraryError> {
        let selected = match self.transition {
            Transition::Allocate { .. } => {
                let changed = self
                    .before
                    .iter()
                    .zip(self.state.page_allocation().iter())
                    .enumerate()
                    .filter(|(_, (before, after))| before != after)
                    .map(|(slot, _)| slot)
                    .collect::<Vec<_>>();
                match changed.as_slice() {
                    [slot]
                        if self.before[*slot] == 0 && self.state.page_allocation()[*slot] == 1 =>
                    {
                        Some(*slot)
                    }
                    _ => None,
                }
            }
            Transition::Release { slot } => {
                let mut expected = self.before;
                expected[slot] = 0;
                (self.state.page_allocation() == expected).then_some(slot)
            }
        };
        let valid_payload = selected.is_some_and(|slot| {
            self.state.pages[slot].bytes.iter().all(|byte| *byte == 0)
                && match self.transition {
                    Transition::Allocate { .. } => {
                        self.state.page_owner(slot).is_none() && self.state.page_index(slot) == 0
                    }
                    Transition::Release { .. } => self.state.page_owner(slot).is_some(),
                }
        });
        if !result || !proof.complete() || !valid_payload {
            self.state.page_actions.poisoned = true;
            return Err(RegionLibraryError::Invariant);
        }
        let slot = selected.unwrap();
        match self.transition {
            Transition::Allocate { object, index } => {
                self.state.associate_page(slot, object, index);
            }
            Transition::Release { .. } => self.state.erase_page(slot),
        }
        self.state.page_actions.active = false;
        Ok(slot as u64)
    }
    pub(crate) fn abandon(&mut self) {
        if self.state.page_actions.active {
            self.state.page_actions.poisoned = true;
        }
    }
}

#[cfg(test)]
#[path = "systems_ram_page_action_tests.rs"]
mod tests;
