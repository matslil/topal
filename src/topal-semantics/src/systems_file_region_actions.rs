//! Field loans on the same persistent bytes used by independent VFS laws.
use super::*;
use crate::{RegionLibraryBinding, RegionLibraryError, RegionLibraryInvocation, RegionLibraryView};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsPositionTransfer {
    Read,
    Write,
}

impl SystemsVolatileFilePool {
    /// Record supplied admitted transfer facts; this does not execute the copy
    /// or a source body, or turn model cleanup flags into physical evidence.
    /// The returned invocation exclusively borrows this entire controller.
    pub fn borrow_position_after_admitted_transfer<'a>(
        &'a mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        operation: SystemsPositionTransfer,
        base: u64,
        count: u64,
        binding: &'a RegionLibraryBinding,
    ) -> Result<(RegionLibraryInvocation<'a>, RegionLibraryView), RegionLibraryError> {
        let identity = self
            .description(owner, file)
            .map_err(|_| RegionLibraryError::Owner)?;
        let description = &self.state.descriptions[&identity];
        let object = self.state.objects[description.object]
            .as_ref()
            .ok_or(RegionLibraryError::Owner)?;
        let allowed = match operation {
            SystemsPositionTransfer::Read => description.rights != SystemsFileRights::Write,
            SystemsPositionTransfer::Write => description.rights != SystemsFileRights::Read,
        };
        let expected_base = if operation == SystemsPositionTransfer::Write && description.append {
            object.size
        } else {
            description.position()
        };
        let end = base.checked_add(count).ok_or(RegionLibraryError::Bounds)?;
        if !allowed
            || count == 0
            || count > QUANTUM as u64
            || end > MAX_OFFSET
            || base != expected_base
            || count > PAGE as u64 - base % PAGE as u64
            || (operation == SystemsPositionTransfer::Read && end > object.size)
        {
            return Err(RegionLibraryError::Bounds);
        }
        let field = &mut self.state.descriptions.get_mut(&identity).unwrap().position;
        RegionLibraryInvocation::begin_position(
            field,
            &mut self.state.position_actions,
            &self.owner,
            self.identity.clone(),
            binding,
            base,
            count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RegionActionReclamation, RegionLibraryAccess};
    use std::collections::{BTreeMap, BTreeSet};
    fn binding() -> RegionLibraryBinding {
        RegionLibraryBinding::new(
            "model-action".into(),
            "advance".into(),
            BTreeMap::from([
                ("advance".into(), BTreeSet::from(["store".into()])),
                ("store".into(), BTreeSet::new()),
            ]),
            2,
            8,
        )
        .unwrap()
    }
    fn proof() -> RegionActionReclamation {
        RegionActionReclamation {
            arena_erased: true,
            control_erased: true,
            retired_stack_erased: true,
            bindings_revoked: true,
            caller_restored: true,
        }
    }
    fn prepared(append: bool) -> (SystemsVolatileFilePool, SystemsVolatileFile) {
        let mut pool = SystemsVolatileFilePool::new("controller").unwrap();
        let file = pool
            .create(
                "controller",
                SystemsFileRights::ReadWrite,
                0o600,
                SystemsFileUmask::default(),
                append,
                100,
            )
            .unwrap();
        pool.write(
            "controller",
            &file,
            b"abcdef",
            SystemsFileUserAdmission {
                numerical_range: true,
                complete_prefix: true,
            },
            100,
        )
        .unwrap();
        pool.seek("controller", &file, 0, SystemsFileSeek::Set)
            .unwrap();
        (pool, file)
    }

    #[test]
    fn persistent_position_loan_changes_actual_shared_bytes_under_one_controller() {
        // TOPAL-SYSTEMS-REGION-ACTION-OWNER-001, -RETURN-001, -POSITION-001.
        // This models checked byte operations, not compiled helper execution.
        let binding = binding();
        let (mut pool, file) = prepared(true);
        let alias = pool.alias("controller", &file).unwrap();
        let (mut action, view) = pool
            .borrow_position_after_admitted_transfer(
                "controller",
                &alias,
                SystemsPositionTransfer::Write,
                6,
                2,
                &binding,
            )
            .unwrap();
        assert_eq!(action.position_action_inputs().unwrap(), (6, 2));
        let resource = action.effect_resource(&view).unwrap();
        let readonly = action
            .derive(&view, 0, 8, RegionLibraryAccess::ReadOnly)
            .unwrap();
        assert_eq!(
            action.store_byte(&readonly, 0, 8),
            Err(RegionLibraryError::ReadOnly)
        );
        assert_eq!(
            action.store_byte(&view, 8, 8),
            Err(RegionLibraryError::Bounds)
        );
        assert_eq!(
            action.store_byte(&view, 0, 256),
            Err(RegionLibraryError::ByteValue)
        );
        let nested = action
            .enter("store", std::slice::from_ref(&view))
            .unwrap()
            .remove(0);
        assert_eq!(
            action.load_byte(&view, 0),
            Err(RegionLibraryError::InactiveFrame)
        );
        for (offset, byte) in 8_u64.to_le_bytes().into_iter().enumerate() {
            action
                .store_byte(&nested, offset as u64, byte as u64)
                .unwrap();
        }
        action.return_from_call().unwrap();
        assert!(action.load_byte(&nested, 0).is_err());
        assert!(action.complete().is_err());
        action.complete_position_action(true, proof()).unwrap();
        assert!(action.load_byte(&view, 0).is_err());
        drop(action);
        assert_eq!(pool.metadata("controller", &file).unwrap().position, 8);
        assert_eq!(pool.metadata("controller", &alias).unwrap().position, 8);
        assert_eq!(pool.metadata("controller", &file).unwrap().size, 6);
        pool.seek("controller", &file, 0, SystemsFileSeek::Set)
            .unwrap();
        let (mut next, current) = pool
            .borrow_position_after_admitted_transfer(
                "controller",
                &file,
                SystemsPositionTransfer::Read,
                0,
                1,
                &binding,
            )
            .unwrap();
        assert_eq!(next.effect_resource(&current).unwrap(), resource);
        assert!(next.load_byte(&view, 0).is_err());
        next.store_byte(&current, 0, 1).unwrap();
        next.complete_position_action(true, proof()).unwrap();
        drop(next);
        assert_eq!(pool.state.position_actions.generation, 2);
        pool.close("controller", &alias).unwrap();
        pool.close("controller", &file).unwrap();
        // The controller, rather than one description, owns the scalar history.
        assert_eq!(pool.state.position_actions.generation, 2);
        pool.terminate("controller").unwrap();
        assert_eq!(pool.state.position_actions, Default::default());
    }

    #[test]
    fn failed_action_or_missing_reclamation_never_resumes_controller_or_rolls_back() {
        // TOPAL-SYSTEMS-REGION-ACTION-RETURN-001, -POSITION-001.
        for case in 0..8 {
            let binding = binding();
            let (mut pool, file) = prepared(false);
            let (mut action, view) = pool
                .borrow_position_after_admitted_transfer(
                    "controller",
                    &file,
                    SystemsPositionTransfer::Read,
                    0,
                    1,
                    &binding,
                )
                .unwrap();
            let byte = if case == 6 { 9 } else { 1 };
            action.store_byte(&view, 0, byte).unwrap();
            let mut reclaimed = proof();
            match case {
                0 => reclaimed.arena_erased = false,
                1 => reclaimed.control_erased = false,
                2 => reclaimed.retired_stack_erased = false,
                3 => reclaimed.bindings_revoked = false,
                4 => reclaimed.caller_restored = false,
                _ => (),
            }
            if case != 7 {
                assert!(
                    action
                        .complete_position_action(case != 5, reclaimed)
                        .is_err()
                );
            }
            drop(action);
            assert!(pool.metadata("controller", &file).is_err());
            assert!(
                pool.seek("controller", &file, 0, SystemsFileSeek::Set)
                    .is_err()
            );
            assert_eq!(
                pool.state.descriptions.values().next().unwrap().position(),
                byte
            );
            assert!(pool.state.position_actions.poisoned);
            pool.terminate("controller").unwrap();
            assert!(pool.state.descriptions.is_empty());
            assert!(
                pool.state
                    .pages
                    .iter()
                    .all(|p| p.owner.is_none() && p.bytes.iter().all(|b| *b == 0))
            );
            assert_eq!(pool.state.position_actions, Default::default());
        }
    }

    #[test]
    fn position_loan_validates_facts_generation_and_actual_pool_identity() {
        // TOPAL-SYSTEMS-REGION-ACTION-OWNER-001, -POSITION-001.
        let binding = binding();
        let (mut pool, file) = prepared(true);
        for (base, count) in [(0, 2), (6, 0), (6, 257), (u64::MAX, 1)] {
            let before = pool.state.clone();
            assert!(
                pool.borrow_position_after_admitted_transfer(
                    "controller",
                    &file,
                    SystemsPositionTransfer::Write,
                    base,
                    count,
                    &binding
                )
                .is_err()
            );
            assert_eq!(pool.state, before);
        }
        pool.state.position_actions.generation = u64::MAX;
        let before = pool.state.clone();
        assert!(
            pool.borrow_position_after_admitted_transfer(
                "controller",
                &file,
                SystemsPositionTransfer::Write,
                6,
                1,
                &binding
            )
            .is_err()
        );
        assert_eq!(pool.state, before);
        let (mut first_pool, first_file) = prepared(false);
        let (first, foreign_view) = first_pool
            .borrow_position_after_admitted_transfer(
                "controller",
                &first_file,
                SystemsPositionTransfer::Read,
                0,
                1,
                &binding,
            )
            .unwrap();
        let (mut second_pool, second_file) = prepared(false);
        let (mut second, view) = second_pool
            .borrow_position_after_admitted_transfer(
                "controller",
                &second_file,
                SystemsPositionTransfer::Read,
                0,
                1,
                &binding,
            )
            .unwrap();
        assert_eq!(
            second.load_byte(&foreign_view, 0),
            Err(RegionLibraryError::View)
        );
        second.store_byte(&view, 0, 1).unwrap();
        second.complete_position_action(true, proof()).unwrap();
        drop(first); // uncompleted first owner remains poisoned, not reusable.
    }
}
