//! Reference conformance, TOPAL-SYSTEMS-TAIL-SESSION-001. No compiled-source claim.
use super::*;
use crate::{
    BootstrapStorageDescriptor, BootstrapStoragePlacement, BootstrapStorageRequest,
    BootstrapStorageTransition,
};
use num_bigint::{BigInt, BigUint};

fn n(value: u64) -> TailScalar {
    TailScalar::Nat(value.into())
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
fn storage(count: u64) -> (BootstrapStorageState, BootstrapRegion) {
    let mut storage = BootstrapStorageState::new(
        BootstrapStorageDescriptor {
            capacity_bytes: count,
            alignment_bytes: 8,
        },
        "tail-owner",
    )
    .unwrap();
    let region = storage
        .allocate(BootstrapStorageRequest {
            byte_count: count,
            alignment_bytes: 8,
            placement: BootstrapStoragePlacement::BootstrapReclaimable,
        })
        .unwrap();
    (storage, region)
}
fn binding() -> TailSessionBinding {
    TailSessionBinding::new(
        RegionLibraryBinding::new(
            "source/code test labels".into(),
            "entry".into(),
            BTreeMap::from([
                ("entry".into(), BTreeSet::from(["next".into()])),
                ("next".into(), BTreeSet::from(["store".into()])),
                ("store".into(), BTreeSet::new()),
            ]),
            3,
            3,
        )
        .unwrap(),
        BTreeSet::from(["entry".into(), "next".into(), "store".into()]),
        vec![
            TailScalarBound::Nat {
                maximum: u64::MAX.into(),
            },
            TailScalarBound::Nat {
                maximum: BigUint::from(u64::MAX) * 2u8,
            },
            TailScalarBound::Nat {
                maximum: u64::MAX.into(),
            },
        ],
        0,
        TailScalarKind::Boolean,
        vec![TailScalar::Boolean(true)],
        TailSessionLimits {
            base_bound: 0u8.into(),
            decrements: BTreeSet::from([1u8.into()]),
            frames_per_step: 3,
            loans_per_step: 3,
            work_bytes: 1024,
        },
    )
    .unwrap()
}
fn transfer(session: &mut TailSession<'_>, values: &[TailScalar]) {
    for (i, v) in values.iter().enumerate() {
        let r = session.allocate(v).unwrap();
        session.stage(i, &r).unwrap();
    }
    session.returned(true).unwrap();
    session.checkpoint().unwrap();
}
fn finish(session: &mut TailSession<'_>) -> TailScalar {
    let r = session.constant(0).unwrap();
    session.stage_result(Some(&r)).unwrap();
    session.returned(false).unwrap();
    session.checkpoint().unwrap();
    session.complete(proof()).unwrap()
}

#[test]
fn actual_owner_bytes_and_history_survive_seventy_thousand_retirements() {
    let b = binding();
    let (mut storage, region) = storage(70032);
    for i in 0..70032 {
        storage.store_byte(&region, i, 0x7b).unwrap();
    }
    let initial = vec![n(70000), n(11), n(1)];
    let saved = initial.clone();
    let mut s = TailSession::begin(
        &mut storage,
        &region,
        &b,
        RegionLibraryAccess::ReadWrite,
        &initial,
    )
    .unwrap();
    let mut first = None;
    for step in 0..70000 {
        s.start_step().unwrap();
        let root = s.root_view().unwrap();
        if first.is_none() {
            first = Some(s.effect_resource(&root).unwrap());
        }
        let next = s.enter_helper("next", Some(&root)).unwrap().unwrap();
        let store = s.enter_helper("store", Some(&next)).unwrap().unwrap();
        assert_eq!(s.effect_resource(&store).unwrap(), *first.as_ref().unwrap());
        s.store_byte(&store, 11 + step, 0).unwrap();
        s.return_helper().unwrap();
        s.return_helper().unwrap();
        transfer(&mut s, &[n(69999 - step), n(12 + step), n(1)]);
        assert!(s.work.iter().all(|b| *b == 0));
        assert!(s.banks[1 - s.bank].iter().all(|b| *b == 0));
        assert_eq!(
            s.identity_counters(),
            (2 + 3 * (step + 1), 2 + 3 * (step + 1))
        );
    }
    s.start_step().unwrap();
    assert_eq!(finish(&mut s), TailScalar::Boolean(true));
    assert!(s.private_buffers_reclaimed());
    drop(s);
    assert_eq!(initial, saved);
    for i in 0..70032 {
        assert_eq!(
            storage.load_byte(&region, i).unwrap(),
            if (11..70011).contains(&i) { 0 } else { 0x7b }
        );
    }
    assert_eq!(
        storage.tail_identity_counters(&region).unwrap(),
        (210002, 210002)
    );
}

#[test]
fn complete_multilimb_alias_boolean_and_unit_transfers_use_disjoint_banks() {
    for negative in [false, true] {
        let base = binding();
        let wide = BigInt::from(1u8) << 159usize;
        let value = if negative { -&wide } else { wide.clone() };
        let roots = vec![
            TailScalarBound::Nat {
                maximum: u64::MAX.into(),
            },
            TailScalarBound::Int {
                minimum: -&wide,
                maximum: wide.clone(),
            },
            TailScalarBound::Int {
                minimum: -&wide,
                maximum: wide.clone(),
            },
            TailScalarBound::Boolean,
        ];
        let b = TailSessionBinding::new(
            base.library,
            base.region_functions,
            roots,
            0,
            TailScalarKind::Int,
            vec![TailScalar::Int(value.clone())],
            base.limits,
        )
        .unwrap();
        let (mut storage, region) = storage(8);
        let mut s = TailSession::begin(
            &mut storage,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[
                n(3),
                TailScalar::Int((-1).into()),
                TailScalar::Int(0.into()),
                TailScalar::Boolean(true),
            ],
        )
        .unwrap();
        for measure in (1..=3).rev() {
            s.start_step().unwrap();
            let number = s.constant(0).unwrap();
            let boolean = s.input(3).unwrap();
            let next = s.allocate(&n(measure - 1)).unwrap();
            s.stage(0, &next).unwrap();
            s.stage(1, &number).unwrap();
            s.stage(2, &number.clone()).unwrap();
            s.stage(3, &boolean).unwrap();
            s.returned(true).unwrap();
            s.checkpoint().unwrap();
            assert_eq!(
                s.bank_value(s.bank, 1).unwrap(),
                TailScalar::Int(value.clone())
            );
            assert_eq!(
                s.bank_value(s.bank, 2).unwrap(),
                TailScalar::Int(value.clone())
            );
            assert_eq!(s.bank_value(s.bank, 3).unwrap(), TailScalar::Boolean(true));
        }
        s.start_step().unwrap();
        let result = s.input(1).unwrap();
        s.stage_result(Some(&result)).unwrap();
        s.returned(false).unwrap();
        s.checkpoint().unwrap();
        assert_eq!(s.complete(proof()).unwrap(), TailScalar::Int(value));
        assert!(s.private_buffers_reclaimed());
    }
    let mut b = binding();
    b.result = TailScalarKind::Unit;
    let (mut storage, region) = storage(8);
    let mut s = TailSession::begin(
        &mut storage,
        &region,
        &b,
        RegionLibraryAccess::ReadWrite,
        &[n(0), n(0), n(1)],
    )
    .unwrap();
    s.start_step().unwrap();
    s.stage_result(None).unwrap();
    s.returned(false).unwrap();
    s.checkpoint().unwrap();
    assert_eq!(s.complete(proof()).unwrap(), TailScalar::Unit);
    assert!(s.private_buffers_reclaimed());
}

#[test]
fn retired_bank_work_constant_and_view_tokens_cannot_revive() {
    for origin in 0..4 {
        let b = binding();
        let (mut storage, region) = storage(8);
        let mut s = TailSession::begin(
            &mut storage,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[n(2), n(0), n(1)],
        )
        .unwrap();
        s.start_step().unwrap();
        let view = s.root_view().unwrap();
        s.store_byte(&view, 0, 0x6a).unwrap();
        let old = match origin {
            0 => s.input(0).unwrap(),
            1 => s.allocate(&n(2)).unwrap(),
            _ => s.constant(0).unwrap(),
        };
        transfer(&mut s, &[n(1), n(1), n(1)]);
        s.start_step().unwrap();
        if origin == 3 {
            assert_eq!(
                s.load_byte(&view, 0),
                Err(RegionLibraryError::InactiveFrame)
            );
        } else {
            assert_eq!(s.read(&old), Err(RegionLibraryError::View));
        }
        assert_eq!(s.phase(), TailSessionPhase::Fatal);
        assert!(s.private_buffers_reclaimed());
        drop(s);
        assert_eq!(
            storage.known_byte(&region, 0).unwrap_err().code,
            "E-SYSTEMS-TAIL-FATAL"
        );
        assert!(storage.transitions().iter().any(|t| matches!(
            t,
            BootstrapStorageTransition::StoredByte { value: 0x6a, .. }
        )));
        storage.release(region).unwrap();
        storage.complete_bootstrap().unwrap();
    }
}

#[test]
fn helpers_revoke_views_but_returned_immutable_numbers_remain_live_until_checkpoint() {
    let b = binding();
    let (mut storage, region) = storage(8);
    let mut s = TailSession::begin(
        &mut storage,
        &region,
        &b,
        RegionLibraryAccess::ReadWrite,
        &[n(1), n(0), n(1)],
    )
    .unwrap();
    s.start_step().unwrap();
    let root = s.root_view().unwrap();
    let helper = s.enter_helper("next", Some(&root)).unwrap().unwrap();
    let value = s.allocate(&n(0)).unwrap();
    s.return_helper().unwrap();
    assert_eq!(s.read(&value).unwrap(), n(0));
    s.stage(0, &value).unwrap();
    let one = s.input(2).unwrap();
    s.stage(1, &one).unwrap();
    s.stage(2, &one).unwrap();
    s.returned(true).unwrap();
    s.checkpoint().unwrap();
    s.start_step().unwrap();
    assert_eq!(
        s.load_byte(&helper, 0),
        Err(RegionLibraryError::InactiveFrame)
    );
    assert!(s.private_buffers_reclaimed());
}

#[test]
fn admission_refusals_preserve_every_storage_fact_and_observation() {
    let b = binding();
    for values in [
        vec![n(1)],
        vec![n(u64::MAX), n(0), n(1)],
        vec![n(1), TailScalar::Nat(BigUint::from(1u8) << 64), n(1)],
        vec![n(1), TailScalar::Int((-1).into()), n(1)],
    ] {
        let (mut storage, region) = storage(8);
        let before = format!("{storage:?}");
        assert!(
            TailSession::begin(
                &mut storage,
                &region,
                &b,
                RegionLibraryAccess::ReadWrite,
                &values
            )
            .is_err()
        );
        assert_eq!(format!("{storage:?}"), before);
    }
    let (_other, region) = storage(16);
    let (mut target, _) = storage(8);
    // Equal display labels do not repair an actual allocation-layout mismatch.
    let before = format!("{target:?}");
    assert_eq!(
        TailSession::begin(
            &mut target,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[n(1), n(0), n(1)]
        )
        .unwrap_err(),
        RegionLibraryError::Owner
    );
    assert_eq!(format!("{target:?}"), before);
}

#[test]
fn new_sessions_continue_actual_allocation_history_and_never_revive_foreign_roots() {
    let b = binding();
    let (mut storage, region) = storage(8);
    let mut foreign = None;
    for invocation in 0..2 {
        let mut s = TailSession::begin(
            &mut storage,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[n(1), n(0), n(1)],
        )
        .unwrap();
        s.start_step().unwrap();
        let old = s.input(0).unwrap();
        if invocation == 0 {
            foreign = Some(old);
        }
        transfer(&mut s, &[n(0), n(1), n(1)]);
        s.start_step().unwrap();
        finish(&mut s);
        drop(s);
        assert_eq!(
            storage.tail_identity_counters(&region).unwrap(),
            (3 + invocation, 3 + invocation)
        );
    }
    let mut s = TailSession::begin(
        &mut storage,
        &region,
        &b,
        RegionLibraryAccess::ReadWrite,
        &[n(0), n(0), n(1)],
    )
    .unwrap();
    s.start_step().unwrap();
    assert_eq!(s.read(&foreign.unwrap()), Err(RegionLibraryError::View));
    assert!(s.private_buffers_reclaimed());
}

#[test]
fn invalid_progress_incomplete_roots_scope_and_rights_take_fatal_cleanup() {
    for bad in 0..8 {
        let b = binding();
        let (mut storage, region) = storage(8);
        let mut s = TailSession::begin(
            &mut storage,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[n(2), n(0), n(1)],
        )
        .unwrap();
        s.start_step().unwrap();
        let root = s.root_view().unwrap();
        s.store_byte(&root, 0, 7).unwrap();
        let error = match bad {
            0 => s.returned(true),
            1 => s.return_helper(),
            2 => s.enter_helper("entry", Some(&root)).map(|_| ()),
            3 => {
                let h = s.enter_helper("next", Some(&root)).unwrap().unwrap();
                let _ = h;
                s.store_byte(&root, 1, 8)
            }
            4 => {
                let r = s.input(0).unwrap();
                s.stage(0, &r).unwrap();
                s.stage(0, &r)
            }
            5 => {
                let r = s.constant(0).unwrap();
                s.stage(0, &r)
            }
            6 => {
                for (i, v) in [n(2), n(1), n(1)].iter().enumerate() {
                    let r = s.allocate(v).unwrap();
                    s.stage(i, &r).unwrap();
                }
                s.returned(true).unwrap();
                s.checkpoint()
            }
            _ => s.start_step(),
        };
        assert!(error.is_err());
        assert_eq!(s.phase(), TailSessionPhase::Fatal);
        assert!(s.private_buffers_reclaimed());
    }
    let b = binding();
    let (mut storage, region) = storage(8);
    let mut s = TailSession::begin(
        &mut storage,
        &region,
        &b,
        RegionLibraryAccess::ReadOnly,
        &[n(0), n(0), n(1)],
    )
    .unwrap();
    s.start_step().unwrap();
    let view = s.root_view().unwrap();
    assert_eq!(s.store_byte(&view, 0, 1), Err(RegionLibraryError::ReadOnly));
    assert_eq!(s.phase(), TailSessionPhase::Running);
    s.cancel();
    assert!(s.private_buffers_reclaimed());
}

#[test]
fn cancellation_abandonment_and_incomplete_completion_never_resume_the_owner() {
    for phase in 0..6 {
        let b = binding();
        let (mut storage, region) = storage(8);
        {
            let mut s = TailSession::begin(
                &mut storage,
                &region,
                &b,
                RegionLibraryAccess::ReadWrite,
                &[n(1), n(0), n(1)],
            )
            .unwrap();
            if phase > 0 {
                s.start_step().unwrap();
                let root = s.root_view().unwrap();
                s.store_byte(&root, 0, 0x42).unwrap();
            }
            if phase == 2 {
                let r = s.allocate(&n(0)).unwrap();
                s.stage(0, &r).unwrap();
            }
            if phase == 3 {
                transfer(&mut s, &[n(0), n(1), n(1)]);
            }
            if phase == 4 {
                let r = s.constant(0).unwrap();
                s.stage_result(Some(&r)).unwrap();
                s.returned(false).unwrap();
            }
            if phase != 5 {
                s.cancel();
                assert!(s.private_buffers_reclaimed());
                assert_eq!(s.start_step(), Err(RegionLibraryError::Ended));
            }
        }
        assert_eq!(
            storage.load_byte(&region, 0).unwrap_err().code,
            "E-SYSTEMS-TAIL-FATAL"
        );
    }
    for missing in 0..5 {
        let b = binding();
        let (mut storage, region) = storage(8);
        let mut s = TailSession::begin(
            &mut storage,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[n(0), n(0), n(1)],
        )
        .unwrap();
        s.start_step().unwrap();
        let r = s.constant(0).unwrap();
        s.stage_result(Some(&r)).unwrap();
        s.returned(false).unwrap();
        s.checkpoint().unwrap();
        let mut p = proof();
        match missing {
            0 => p.arena_erased = false,
            1 => p.control_erased = false,
            2 => p.retired_stack_erased = false,
            3 => p.bindings_revoked = false,
            _ => p.caller_restored = false,
        };
        assert_eq!(s.complete(p), Err(RegionLibraryError::Invariant));
        assert!(s.private_buffers_reclaimed());
    }
}

#[test]
fn nonunit_progress_and_sixty_five_bit_carried_values_keep_exact_meaning() {
    let mut b = binding();
    b.limits.base_bound = 2u8.into();
    b.limits.decrements = BTreeSet::from([3u8.into()]);
    let (mut storage, region) = storage(8);
    let mut s = TailSession::begin(
        &mut storage,
        &region,
        &b,
        RegionLibraryAccess::ReadWrite,
        &[n(5), n(u64::MAX), n(1)],
    )
    .unwrap();
    s.start_step().unwrap();
    let offset = TailScalar::Nat(BigUint::from(u64::MAX) + 1u8);
    transfer(&mut s, &[n(2), offset.clone(), n(1)]);
    s.start_step().unwrap();
    let root = s.input(1).unwrap();
    assert_eq!(s.read(&root).unwrap(), offset);
    finish(&mut s);
    for (initial, next) in [(2, 0), (5, 3)] {
        let (mut storage, region) = super::tests::storage(8);
        let mut s = TailSession::begin(
            &mut storage,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[n(initial), n(0), n(1)],
        )
        .unwrap();
        s.start_step().unwrap();
        for (i, v) in [n(next), n(0), n(1)].iter().enumerate() {
            let r = s.allocate(v).unwrap();
            s.stage(i, &r).unwrap();
        }
        s.returned(true).unwrap();
        assert_eq!(s.checkpoint(), Err(RegionLibraryError::Invariant));
        assert!(s.private_buffers_reclaimed());
    }
}

#[test]
fn malformed_values_and_allocation_exhaustion_reclaim_the_entire_session() {
    for fault in 0..3 {
        let mut b = binding();
        if fault == 0 {
            b.limits.work_bytes = 16;
        }
        let (mut storage, region) = storage(8);
        let mut s = TailSession::begin(
            &mut storage,
            &region,
            &b,
            RegionLibraryAccess::ReadWrite,
            &[n(1), n(0), n(1)],
        )
        .unwrap();
        s.start_step().unwrap();
        let view = s.root_view().unwrap();
        s.store_byte(&view, 0, 0x77).unwrap();
        if fault == 0 {
            assert_eq!(s.allocate(&n(1)).unwrap_err(), RegionLibraryError::Capacity);
        } else {
            let value = s.allocate(&n(0)).unwrap();
            if fault == 1 {
                s.work[8..16].copy_from_slice(&9u64.to_le_bytes());
            } else {
                s.work[..8].copy_from_slice(&1u64.to_le_bytes());
            }
            assert_eq!(s.read(&value), Err(RegionLibraryError::Invariant));
        }
        assert!(s.private_buffers_reclaimed());
    }
}

#[test]
fn ordinary_byte_refusals_preserve_state_and_never_truncate_large_naturals() {
    let b = binding();
    for access in [
        RegionLibraryAccess::ReadOnly,
        RegionLibraryAccess::ReadWrite,
    ] {
        let (mut storage, region) = storage(8);
        let mut s =
            TailSession::begin(&mut storage, &region, &b, access, &[n(1), n(0), n(1)]).unwrap();
        s.start_step().unwrap();
        let root = s.root_view().unwrap();
        let huge = BigUint::from(1u8) << 100usize;
        for (index, value) in [
            (BigUint::from(8u8), 0u8.into()),
            (huge.clone(), 0u8.into()),
            (0u8.into(), 256u16.into()),
            (0u8.into(), huge.clone()),
        ] {
            let before = format!("{s:?}");
            assert!(!s.byte_store(&root, &index, &value).unwrap());
            assert_eq!(format!("{s:?}"), before);
        }
        for index in [BigUint::from(8u8), huge] {
            let before = format!("{s:?}");
            assert_eq!(s.byte_load(&root, &index).unwrap(), None);
            assert_eq!(format!("{s:?}"), before);
        }
        if access == RegionLibraryAccess::ReadOnly {
            let before = format!("{s:?}");
            assert!(!s.byte_store(&root, &0u8.into(), &7u8.into()).unwrap());
            assert_eq!(format!("{s:?}"), before);
        } else {
            assert!(s.byte_store(&root, &0u8.into(), &7u8.into()).unwrap());
            assert_eq!(s.byte_load(&root, &0u8.into()).unwrap(), Some(7));
        }
        // Invalid scoped authority has precedence over an ordinary wide-index refusal.
        let helper = s.enter_helper("next", Some(&root)).unwrap().unwrap();
        let _ = helper;
        assert_eq!(
            s.byte_load(&root, &(BigUint::from(1u8) << 100usize)),
            Err(RegionLibraryError::InactiveFrame)
        );
        assert!(s.private_buffers_reclaimed());
    }
}
