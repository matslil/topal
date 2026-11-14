use super::*;
use crate::RegionLibraryAccess;
use std::collections::{BTreeMap, BTreeSet};

const OK: SystemsFileUserAdmission = SystemsFileUserAdmission {
    numerical_range: true,
    complete_prefix: true,
};

fn binding() -> RegionLibraryBinding {
    RegionLibraryBinding::new(
        "same-model-label".into(),
        "entry".into(),
        BTreeMap::from([
            ("entry".into(), BTreeSet::from(["store".into()])),
            ("store".into(), BTreeSet::new()),
        ]),
        2,
        16,
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

fn fixture(mask: u8) -> (SystemsVolatileFilePool, SystemsVolatileFile) {
    let mut pool = SystemsVolatileFilePool::new("fs").unwrap();
    let file = pool
        .create(
            "fs",
            SystemsFileRights::ReadWrite,
            0o600,
            SystemsFileUmask::default(),
            false,
            10,
        )
        .unwrap();
    pool.truncate("fs", &file, i64::MAX, 10).unwrap();
    let identity = pool.metadata("fs", &file).unwrap().identity;
    // Independent canonical resource fixtures: actual map, live object token,
    // associations and distinctive occupied payloads agree for every mask.
    for slot in 0..PAGES {
        if mask & (1 << slot) != 0 {
            pool.state.page_allocation_mut()[slot] = 1;
            pool.state.associate_page(slot, identity, slot as u64);
            pool.state.pages[slot].bytes.fill(slot as u8 + 1);
        }
    }
    assert!(pool.allocation_consistent());
    (pool, file)
}

#[test]
fn every_occupancy_mask_accepts_actual_first_or_last_free_source_delta() {
    // TOPAL-SYSTEMS-REGION-PAGE-OWNER-001, -ALLOCATE-001.
    // These checked byte operations model opaque effects, not source execution.
    let binding = binding();
    for mask in 0..=u8::MAX {
        for reverse in [false, true] {
            let (mut pool, file) = fixture(mask);
            let before = pool.state.clone();
            if mask == u8::MAX {
                assert_eq!(
                    pool.borrow_page_action_after_admission(
                        "fs",
                        &file,
                        SystemsRamPageAction::Allocate { logical_page: 49 },
                        &binding
                    )
                    .unwrap_err(),
                    RegionLibraryError::Capacity
                );
                assert_eq!(pool.state, before);
                continue;
            }
            let selected = if reverse {
                (0..PAGES)
                    .rev()
                    .find(|slot| mask & (1 << slot) == 0)
                    .unwrap()
            } else {
                (0..PAGES).find(|slot| mask & (1 << slot) == 0).unwrap()
            };
            let (mut call, view) = pool
                .borrow_page_action_after_admission(
                    "fs",
                    &file,
                    SystemsRamPageAction::Allocate { logical_page: 49 },
                    &binding,
                )
                .unwrap();
            assert_eq!(call.page_action_inputs().unwrap(), (0, 0));
            assert!(call.complete().is_err());
            assert!(call.cancel().is_err());
            let readonly = call
                .derive(&view, 0, 8, RegionLibraryAccess::ReadOnly)
                .unwrap();
            assert_eq!(
                call.store_byte(&readonly, 0, 1),
                Err(RegionLibraryError::ReadOnly)
            );
            assert_eq!(
                call.store_byte(&view, 8, 1),
                Err(RegionLibraryError::Bounds)
            );
            assert_eq!(
                call.store_byte(&view, 0, 256),
                Err(RegionLibraryError::ByteValue)
            );
            for slot in 0..PAGES {
                assert_eq!(
                    call.load_byte(&view, slot as u64).unwrap(),
                    before.page_allocation()[slot]
                );
            }
            let nested = call
                .enter("store", std::slice::from_ref(&view))
                .unwrap()
                .remove(0);
            assert_eq!(
                call.complete_page_action(true, proof()),
                Err(RegionLibraryError::NotRoot)
            );
            assert!(call.load_byte(&view, 0).is_err());
            call.store_byte(&nested, selected as u64, 1).unwrap();
            call.return_from_call().unwrap();
            assert!(call.load_byte(&nested, 0).is_err());
            assert_eq!(
                call.complete_page_action(true, proof()).unwrap(),
                selected as u64
            );
            assert!(call.load_byte(&view, 0).is_err());
            drop(call);
            assert!(pool.allocation_consistent());
            let mut expected_metadata = before.metadata;
            expected_metadata[24 + selected] = 1;
            let identity = pool.metadata("fs", &file).unwrap().identity;
            let record = 1024 + selected * 32;
            for (offset, value) in [(record, identity), (record + 8, 49_u64)] {
                expected_metadata[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
            }
            assert_eq!(pool.state.metadata, expected_metadata);
            for slot in 0..PAGES {
                if slot != selected {
                    assert_eq!(pool.state.pages[slot], before.pages[slot]);
                    assert_eq!(
                        pool.state.page_allocation()[slot],
                        before.page_allocation()[slot]
                    );
                }
            }
            assert_eq!(pool.state.page_allocation()[selected], 1);
            assert_eq!(pool.state.page_index(selected), 49);
            assert_eq!(
                pool.read_at("fs", &file, 3, 49 * PAGE as i64, OK).unwrap(),
                [0, 0, 0]
            );
            assert_eq!(
                pool.metadata("fs", &file).unwrap().allocated_bytes,
                (mask.count_ones() as usize + 1) * PAGE
            );
        }
    }
}

#[test]
fn every_release_slot_erases_real_payload_and_reuses_it_with_a_fresh_loan() {
    // TOPAL-SYSTEMS-REGION-PAGE-RELEASE-001.
    let binding = binding();
    for slot in 0..PAGES {
        let (mut pool, file) = fixture(u8::MAX);
        let alias = pool.alias("fs", &file).unwrap();
        let before = pool.state.clone();
        let (mut release, old_view) = pool
            .borrow_page_action_after_admission(
                "fs",
                &alias,
                SystemsRamPageAction::Release { slot: slot as u64 },
                &binding,
            )
            .unwrap();
        assert_eq!(release.page_action_inputs().unwrap(), (1, slot as u64));
        assert_eq!(release.load_byte(&old_view, slot as u64).unwrap(), 1);
        release.store_byte(&old_view, slot as u64, 0).unwrap();
        assert_eq!(
            release.complete_page_action(true, proof()).unwrap(),
            slot as u64
        );
        drop(release);
        let mut expected_metadata = before.metadata;
        expected_metadata[24 + slot] = 0;
        expected_metadata[1024 + slot * 32..1056 + slot * 32].fill(0);
        assert_eq!(pool.state.metadata, expected_metadata);
        assert!(pool.state.page_owner(slot).is_none());
        assert!(pool.state.pages[slot].bytes.iter().all(|byte| *byte == 0));
        assert_eq!(pool.state.page_index(slot), 0);
        for other in 0..PAGES {
            if other != slot {
                assert_eq!(pool.state.pages[other], before.pages[other]);
            }
        }
        assert_eq!(
            pool.metadata("fs", &alias).unwrap().allocated_bytes,
            7 * PAGE
        );
        let (mut allocate, current) = pool
            .borrow_page_action_after_admission(
                "fs",
                &file,
                SystemsRamPageAction::Allocate { logical_page: 99 },
                &binding,
            )
            .unwrap();
        assert!(allocate.load_byte(&old_view, slot as u64).is_err());
        allocate.store_byte(&current, slot as u64, 1).unwrap();
        allocate.complete_page_action(true, proof()).unwrap();
        drop(allocate);
        assert_eq!(pool.state.page_actions.generation, 2);
        assert_eq!(
            pool.read_at("fs", &alias, 8, 99 * PAGE as i64, OK).unwrap(),
            [0; 8]
        );
        pool.write_at("fs", &file, b"fresh", 99 * PAGE as i64, OK, 11)
            .unwrap();
        assert_eq!(
            pool.read_at("fs", &alias, 5, 99 * PAGE as i64, OK).unwrap(),
            b"fresh"
        );
        assert!(pool.allocation_consistent());
        pool.close("fs", &alias).unwrap();
        pool.close("fs", &file).unwrap();
        assert_eq!(pool.state.page_allocation(), [0; PAGES]);
        assert_eq!(pool.state.page_actions.generation, 2);
        pool.terminate("fs").unwrap();
        assert_eq!(pool.state.page_actions, Default::default());
    }
}

#[test]
fn invalid_admission_or_generation_exhaustion_cannot_erase_release_payloads() {
    // TOPAL-SYSTEMS-REGION-PAGE-OWNER-001, -RELEASE-001.
    let binding = binding();
    let (mut pool, file) = fixture(1 << 3);
    let (_, foreign) = fixture(0);
    for request in [
        SystemsRamPageAction::Release { slot: 0 },
        SystemsRamPageAction::Release { slot: 8 },
        SystemsRamPageAction::Release { slot: u64::MAX },
        SystemsRamPageAction::Allocate {
            logical_page: u64::MAX,
        },
        SystemsRamPageAction::Allocate { logical_page: 3 },
    ] {
        let before = pool.state.clone();
        assert!(
            pool.borrow_page_action_after_admission("fs", &file, request, &binding)
                .is_err()
        );
        assert_eq!(pool.state, before);
    }
    for owner in ["wrong", "fs"] {
        let before = pool.state.clone();
        assert!(
            pool.borrow_page_action_after_admission(
                owner,
                &foreign,
                SystemsRamPageAction::Release { slot: 3 },
                &binding
            )
            .is_err()
        );
        assert_eq!(pool.state, before);
    }
    pool.state.page_actions.generation = u64::MAX;
    let before = pool.state.clone();
    assert_eq!(
        pool.borrow_page_action_after_admission(
            "fs",
            &file,
            SystemsRamPageAction::Release { slot: 3 },
            &binding
        )
        .unwrap_err(),
        RegionLibraryError::IdentityExhausted
    );
    assert_eq!(pool.state, before);
    assert!(pool.state.pages[3].bytes.iter().all(|byte| *byte == 4));
}

#[test]
fn failed_or_abandoned_action_preserves_source_stores_and_poisoned_owner_exclusion() {
    // TOPAL-SYSTEMS-REGION-PAGE-ALLOCATE-001, -RELEASE-001.
    let binding = binding();
    for case in 0..11 {
        let (mut pool, file) = fixture(1);
        let (mut call, view) = pool
            .borrow_page_action_after_admission(
                "fs",
                &file,
                SystemsRamPageAction::Allocate { logical_page: 99 },
                &binding,
            )
            .unwrap();
        call.store_byte(&view, 1, 1).unwrap();
        let mut reclaimed = proof();
        match case {
            0 => reclaimed.arena_erased = false,
            1 => reclaimed.control_erased = false,
            2 => reclaimed.retired_stack_erased = false,
            3 => reclaimed.bindings_revoked = false,
            4 => reclaimed.caller_restored = false,
            6 => call.store_byte(&view, 2, 1).unwrap(),
            7 => call.store_byte(&view, 0, 0).unwrap(),
            8 => call.store_byte(&view, 1, 2).unwrap(),
            9 => call.store_byte(&view, 1, 0).unwrap(),
            _ => (),
        }
        if case != 10 {
            assert_eq!(
                call.complete_page_action(case != 5, reclaimed),
                Err(RegionLibraryError::Invariant)
            );
        }
        drop(call);
        assert!(pool.state.page_actions.poisoned);
        assert_eq!(
            pool.state.page_allocation()[1],
            match case {
                8 => 2,
                9 => 0,
                _ => 1,
            }
        );
        assert!(pool.state.page_owner(1).is_none());
        assert!(pool.metadata("fs", &file).is_err());
        assert!(pool.write("fs", &file, b"blocked", OK, 11).is_err());
        pool.terminate("fs").unwrap();
        assert_eq!(pool.state.page_allocation(), [0; PAGES]);
        assert!(
            pool.state
                .page_records()
                .all(|page| page.owner.is_none() && page.bytes.iter().all(|b| *b == 0))
        );
        assert_eq!(pool.state.page_actions, Default::default());
    }
}

#[test]
fn identical_binding_labels_and_generations_do_not_merge_position_and_page_views() {
    // TOPAL-SYSTEMS-REGION-PAGE-OWNER-001.
    let binding = binding();
    let (mut pool, file) = fixture(0);
    let (mut position, position_view) = pool
        .borrow_position_after_admitted_transfer(
            "fs",
            &file,
            SystemsPositionTransfer::Read,
            0,
            1,
            &binding,
        )
        .unwrap();
    let effect = position.effect_resource(&position_view).unwrap();
    position.store_byte(&position_view, 0, 1).unwrap();
    position.complete_position_action(true, proof()).unwrap();
    drop(position);
    let (mut page, page_view) = pool
        .borrow_page_action_after_admission(
            "fs",
            &file,
            SystemsRamPageAction::Allocate { logical_page: 99 },
            &binding,
        )
        .unwrap();
    assert_eq!(page.effect_resource(&page_view).unwrap(), effect);
    assert!(page.load_byte(&position_view, 0).is_err());
    assert!(page.complete_position_action(true, proof()).is_err());
    page.store_byte(&page_view, 0, 1).unwrap();
    page.complete_page_action(true, proof()).unwrap();
    drop(page);
    assert_eq!(pool.state.page_actions.generation, 1);
    assert_eq!(pool.state.position_actions.generation, 1);
    assert_eq!(pool.metadata("fs", &file).unwrap().position, 1);

    let (mut other, other_file) = fixture(0);
    let (mut call, current) = other
        .borrow_page_action_after_admission(
            "fs",
            &other_file,
            SystemsRamPageAction::Allocate { logical_page: 99 },
            &binding,
        )
        .unwrap();
    assert!(call.store_byte(&page_view, 1, 1).is_err());
    call.store_byte(&current, 1, 1).unwrap();
    call.complete_page_action(true, proof()).unwrap();
}

#[test]
fn normal_filesystem_laws_share_the_same_bitmap_through_truncate_and_final_close() {
    // TOPAL-SYSTEMS-REGION-PAGE-RELEASE-001.
    let (mut pool, file) = fixture(0);
    let alias = pool.alias("fs", &file).unwrap();
    for index in 0..3 {
        pool.write_at("fs", &file, b"payload", index * PAGE as i64, OK, 11)
            .unwrap();
    }
    assert_eq!(&pool.state.page_allocation()[..4], &[1, 1, 1, 0]);
    pool.truncate("fs", &alias, PAGE as i64 + 2, 12).unwrap();
    assert_eq!(&pool.state.page_allocation()[..4], &[1, 1, 0, 0]);
    assert_eq!(
        pool.read_at("fs", &file, 4, PAGE as i64, OK).unwrap(),
        b"pa"
    );
    pool.truncate("fs", &file, (3 * PAGE) as i64, 13).unwrap();
    assert_eq!(
        pool.read_at("fs", &alias, 5, PAGE as i64 + 2, OK).unwrap(),
        [0; 5]
    );
    assert_eq!(&pool.state.page_allocation()[..4], &[1, 1, 0, 0]);
    pool.close("fs", &file).unwrap();
    assert_eq!(&pool.state.page_allocation()[..4], &[1, 1, 0, 0]);
    pool.close("fs", &alias).unwrap();
    assert_eq!(pool.state.page_allocation(), [0; PAGES]);
    assert!(pool.allocation_consistent());
}

#[test]
fn release_failure_cannot_resume_or_undo_admitted_payload_erasure() {
    // TOPAL-SYSTEMS-REGION-PAGE-RELEASE-001.
    let binding = binding();
    for case in 0..9 {
        let (mut pool, file) = fixture(u8::MAX);
        let (mut call, view) = pool
            .borrow_page_action_after_admission(
                "fs",
                &file,
                SystemsRamPageAction::Release { slot: 3 },
                &binding,
            )
            .unwrap();
        call.store_byte(&view, 3, 0).unwrap();
        let mut reclaimed = proof();
        match case {
            0 => reclaimed.arena_erased = false,
            1 => reclaimed.control_erased = false,
            2 => reclaimed.retired_stack_erased = false,
            3 => reclaimed.bindings_revoked = false,
            4 => reclaimed.caller_restored = false,
            6 => call.store_byte(&view, 4, 0).unwrap(),
            7 => call.store_byte(&view, 3, 1).unwrap(),
            _ => (),
        }
        if case != 8 {
            assert_eq!(
                call.complete_page_action(case != 5, reclaimed),
                Err(RegionLibraryError::Invariant)
            );
        }
        drop(call);
        assert!(pool.state.page_actions.poisoned);
        assert!(pool.state.page_owner(3).is_some());
        assert!(pool.state.pages[3].bytes.iter().all(|byte| *byte == 0));
        assert!(pool.metadata("fs", &file).is_err());
        pool.terminate("fs").unwrap();
        assert_eq!(pool.state.page_allocation(), [0; PAGES]);
    }
}

#[test]
fn malformed_map_or_association_refuses_before_any_loan_or_payload_change() {
    // TOPAL-SYSTEMS-REGION-PAGE-OWNER-001.
    let binding = binding();
    for case in 0..5 {
        let (mut pool, file) = fixture(1 << 3);
        match case {
            0 => pool.state.page_allocation_mut()[0] = 2,
            1 => pool.state.page_allocation_mut()[0] = 1,
            2 => pool.state.page_allocation_mut()[3] = 0,
            3 => pool.state.pages[0].bytes[0] = 9,
            4 => pool.state.put_word(1032 + 3 * 32, u64::MAX),
            _ => unreachable!(),
        }
        let before = pool.state.clone();
        assert_eq!(
            pool.borrow_page_action_after_admission(
                "fs",
                &file,
                SystemsRamPageAction::Release { slot: 3 },
                &binding
            )
            .unwrap_err(),
            RegionLibraryError::Owner
        );
        assert_eq!(pool.state, before);
        assert_eq!(pool.state.page_actions.generation, 0);
        assert!(pool.state.pages[3].bytes.iter().all(|byte| *byte == 4));
    }
}
