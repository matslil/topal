//! TOPAL-SYSTEMS-REGION-UNSIGNED-001;
//! TOPAL-SYSTEMS-REGION-UNSIGNED-QUALIFY-001. Portable model only.
use super::*;
use crate::{
    BootstrapStorageDescriptor, BootstrapStoragePlacement, BootstrapStorageRequest,
    BootstrapStorageTransition, RegionActionReclamation, SystemsFileRights, SystemsFileSeek,
    SystemsFileUmask, SystemsFileUserAdmission, SystemsPositionTransfer, SystemsRamPageAction,
    SystemsVolatileFile, SystemsVolatileFilePool,
};

fn binding() -> RegionLibraryBinding {
    RegionLibraryBinding::new(
        "field-code".into(),
        "entry".into(),
        BTreeMap::from([
            ("entry".into(), BTreeSet::from(["helper".into()])),
            ("helper".into(), BTreeSet::from(["leaf".into()])),
            ("leaf".into(), BTreeSet::new()),
        ]),
        3,
        32,
    )
    .unwrap()
}

fn storage(name: &str) -> (BootstrapStorageState, BootstrapRegion) {
    let mut storage = BootstrapStorageState::new(
        BootstrapStorageDescriptor {
            capacity_bytes: 4096,
            alignment_bytes: 8,
        },
        name,
    )
    .unwrap();
    let region = storage
        .allocate(BootstrapStorageRequest {
            byte_count: 16,
            alignment_bytes: 8,
            placement: BootstrapStoragePlacement::BootstrapReclaimable,
        })
        .unwrap();
    for offset in 0..16 {
        storage
            .store_byte(&region, offset, 160 + u8::try_from(offset).unwrap())
            .unwrap();
    }
    (storage, region)
}

fn format(width: u8, order: RegionLibraryByteOrder) -> RegionLibraryUnsignedFormat {
    RegionLibraryUnsignedFormat::new(width, order).unwrap()
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

fn unchanged<T: std::fmt::Debug>(
    call: &mut RegionLibraryInvocation<'_>,
    error: RegionLibraryError,
    operation: impl FnOnce(&mut RegionLibraryInvocation<'_>) -> Result<T, RegionLibraryError>,
) {
    // Includes payload, storage observations, frames, views and generation.
    let before = format!("{call:?}");
    assert_eq!(operation(call).unwrap_err(), error);
    assert_eq!(format!("{call:?}"), before);
}

fn observations<'a>(call: &'a RegionLibraryInvocation<'_>) -> &'a [BootstrapStorageTransition] {
    let Storage::Bootstrap { storage, .. } = &call.storage else {
        unreachable!()
    };
    storage.transitions()
}

#[test]
fn formats_admit_every_byte_width_without_architecture_or_authority_data() {
    for width in 0..=u8::MAX {
        for order in [
            RegionLibraryByteOrder::LittleEndian,
            RegionLibraryByteOrder::BigEndian,
        ] {
            let value = RegionLibraryUnsignedFormat::new(width, order);
            if (1..=8).contains(&width) {
                let value = value.unwrap();
                assert_eq!(value.byte_count(), width);
                assert_eq!(value.byte_order(), order);
            } else {
                assert_eq!(value, Err(RegionLibraryError::Binding));
            }
        }
    }
}

#[test]
fn every_width_order_offset_and_bit_matches_independent_standard_byte_encodings() {
    let code = binding();
    for width in 1..=8_u8 {
        let maximum = u64::MAX >> ((8 - width) * 8);
        let mut values =
            BTreeSet::from([0, 1, maximum, maximum / 2, 0xc127_0893_adef_5516 & maximum]);
        values.extend((0..width * 8).map(|bit| 1_u64 << bit));
        if width == 1 {
            values.extend(0..=255);
        }
        for order in [
            RegionLibraryByteOrder::LittleEndian,
            RegionLibraryByteOrder::BigEndian,
        ] {
            let encoding = format(width, order);
            for start in 0..=16 - u64::from(width) {
                let (mut owner, region) = storage("bytes");
                let (mut call, root) = RegionLibraryInvocation::begin(
                    &mut owner,
                    &region,
                    &code,
                    RegionLibraryAccess::ReadWrite,
                )
                .unwrap();
                let field = call
                    .derive(
                        &root,
                        start,
                        u64::from(width),
                        RegionLibraryAccess::ReadWrite,
                    )
                    .unwrap();
                for value in &values {
                    let number = BigUint::from(*value);
                    let before_store = observations(&call).len();
                    call.store_unsigned(&field, &BigUint::from(0_u8), encoding, &number)
                        .unwrap();
                    // This oracle uses the standard integer encoding routines,
                    // independently of the model's shift/merge algorithms.
                    let expected = match order {
                        RegionLibraryByteOrder::LittleEndian => {
                            value.to_le_bytes()[..usize::from(width)].to_vec()
                        }
                        RegionLibraryByteOrder::BigEndian => {
                            value.to_be_bytes()[8 - usize::from(width)..].to_vec()
                        }
                    };
                    let writes = &observations(&call)[before_store..];
                    assert_eq!(writes.len(), usize::from(width));
                    let actual = writes
                        .iter()
                        .map(|event| match event {
                            BootstrapStorageTransition::StoredByte {
                                offset_bytes,
                                value,
                                ..
                            } => (*offset_bytes, *value),
                            other => panic!(
                                "store performed an unexpected payload observation: {other:?}"
                            ),
                        })
                        .collect::<BTreeSet<_>>();
                    let expected_events = expected
                        .iter()
                        .enumerate()
                        .map(|(index, byte)| (start + index as u64, *byte))
                        .collect::<BTreeSet<_>>();
                    assert_eq!(actual, expected_events);
                    for offset in 0..16 {
                        let byte = if (start..start + u64::from(width)).contains(&offset) {
                            expected[(offset - start) as usize]
                        } else {
                            160 + offset as u8
                        };
                        assert_eq!(call.load_byte(&root, offset), Ok(byte));
                    }
                    let before_load = observations(&call).len();
                    assert_eq!(
                        call.load_unsigned(&field, &BigUint::from(0_u8), encoding),
                        Ok(number)
                    );
                    let reads = &observations(&call)[before_load..];
                    assert_eq!(reads.len(), usize::from(width));
                    let actual = reads
                        .iter()
                        .map(|event| match event {
                            BootstrapStorageTransition::LoadedByte {
                                offset_bytes,
                                value,
                                ..
                            } => (*offset_bytes, *value),
                            other => panic!(
                                "load performed an unexpected payload observation: {other:?}"
                            ),
                        })
                        .collect::<BTreeSet<_>>();
                    assert_eq!(actual, expected_events);
                }
                call.complete().unwrap();
            }
        }
    }
}

#[test]
fn complete_preflight_rejects_large_facts_narrow_windows_and_readonly_without_observations() {
    let code = binding();
    let (mut owner, region) = storage("preflight");
    let (mut call, root) =
        RegionLibraryInvocation::begin(&mut owner, &region, &code, RegionLibraryAccess::ReadWrite)
            .unwrap();
    let readonly = call
        .derive(&root, 0, 16, RegionLibraryAccess::ReadOnly)
        .unwrap();
    for width in 1..=8 {
        for order in [
            RegionLibraryByteOrder::LittleEndian,
            RegionLibraryByteOrder::BigEndian,
        ] {
            let encoding = format(width, order);
            let narrow = call
                .derive(
                    &root,
                    1,
                    u64::from(width) - 1,
                    RegionLibraryAccess::ReadWrite,
                )
                .unwrap();
            let zero = BigUint::from(0_u8);
            let value = BigUint::from(1_u8);
            for offset in [
                BigUint::from(17 - u64::from(width)),
                BigUint::from(u64::MAX),
                BigUint::from(1_u8) << 256,
            ] {
                unchanged(&mut call, RegionLibraryError::Bounds, |i| {
                    i.load_unsigned(&root, &offset, encoding)
                });
                unchanged(&mut call, RegionLibraryError::Bounds, |i| {
                    i.store_unsigned(&root, &offset, encoding, &value)
                });
            }
            unchanged(&mut call, RegionLibraryError::Bounds, |i| {
                i.load_unsigned(&narrow, &zero, encoding)
            });
            unchanged(&mut call, RegionLibraryError::Bounds, |i| {
                i.store_unsigned(&narrow, &zero, encoding, &value)
            });
            unchanged(&mut call, RegionLibraryError::ReadOnly, |i| {
                i.store_unsigned(&readonly, &zero, encoding, &value)
            });
            for value in [
                BigUint::from(1_u8) << (u32::from(width) * 8),
                BigUint::from(1_u8) << 512,
            ] {
                unchanged(&mut call, RegionLibraryError::UnsignedValue, |i| {
                    i.store_unsigned(&root, &zero, encoding, &value)
                });
            }
        }
    }
    call.complete().unwrap();
}

#[test]
fn nested_aliases_retain_owner_effects_narrowed_rights_and_frame_lifetimes() {
    let code = binding();
    let (mut owner, region) = storage("nested-fields");
    let (mut call, root) =
        RegionLibraryInvocation::begin(&mut owner, &region, &code, RegionLibraryAccess::ReadWrite)
            .unwrap();
    let resource = call.effect_resource(&root).unwrap();
    let view = call
        .derive(&root, 3, 8, RegionLibraryAccess::ReadWrite)
        .unwrap();
    let helper = call
        .enter("helper", std::slice::from_ref(&view))
        .unwrap()
        .remove(0);
    let word = format(4, RegionLibraryByteOrder::LittleEndian);
    let zero = BigUint::from(0_u8);
    call.store_unsigned(&helper, &zero, word, &BigUint::from(0x1020_3040_u32))
        .unwrap();
    unchanged(&mut call, RegionLibraryError::InactiveFrame, |i| {
        i.load_unsigned(&root, &zero, word)
    });
    let readonly = call
        .derive(&helper, 1, 2, RegionLibraryAccess::ReadOnly)
        .unwrap();
    let leaf = call
        .enter("leaf", std::slice::from_ref(&readonly))
        .unwrap()
        .remove(0);
    assert_eq!(call.effect_resource(&leaf).unwrap(), resource);
    let pair = format(2, RegionLibraryByteOrder::BigEndian);
    assert_eq!(
        call.load_unsigned(&leaf, &zero, pair),
        Ok(BigUint::from(0x3020_u16))
    );
    unchanged(&mut call, RegionLibraryError::ReadOnly, |i| {
        i.store_unsigned(&leaf, &zero, pair, &BigUint::from(7_u8))
    });
    call.return_from_call().unwrap();
    unchanged(&mut call, RegionLibraryError::InactiveFrame, |i| {
        i.load_unsigned(&leaf, &zero, pair)
    });
    call.return_from_call().unwrap();
    assert_eq!(call.effect_resource(&view).unwrap(), resource);
    assert_eq!(
        call.load_unsigned(&view, &zero, word),
        Ok(BigUint::from(0x1020_3040_u32))
    );
    call.complete().unwrap();
    unchanged(&mut call, RegionLibraryError::Ended, |i| {
        i.load_unsigned(&root, &zero, word)
    });
}

#[test]
fn foreign_binding_owner_and_retired_generation_never_reach_owner_bytes() {
    let code = binding();
    let encoding = format(8, RegionLibraryByteOrder::LittleEndian);
    let zero = BigUint::from(0_u8);
    let (mut first, first_region) = storage("first");
    let (mut second, second_region) = storage("second");
    let (mut a, foreign) = RegionLibraryInvocation::begin(
        &mut first,
        &first_region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    let (mut b, view) = RegionLibraryInvocation::begin(
        &mut second,
        &second_region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    unchanged(&mut b, RegionLibraryError::View, |i| {
        i.load_unsigned(&foreign, &zero, encoding)
    });
    unchanged(&mut b, RegionLibraryError::View, |i| {
        i.store_unsigned(&foreign, &zero, encoding, &zero)
    });
    let mut wrong_binding = view.clone();
    wrong_binding.binding = "other-code".into();
    unchanged(&mut b, RegionLibraryError::View, |i| {
        i.load_unsigned(&wrong_binding, &zero, encoding)
    });
    a.complete().unwrap();
    b.complete().unwrap();
    drop(a);
    drop(b);
    let (mut next, current) = RegionLibraryInvocation::begin(
        &mut first,
        &first_region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    unchanged(&mut next, RegionLibraryError::View, |i| {
        i.store_unsigned(&foreign, &zero, encoding, &zero)
    });
    assert_eq!(
        next.load_unsigned(&current, &zero, encoding)
            .unwrap()
            .bits(),
        64
    );
    next.complete().unwrap();
}

fn file_pool() -> (SystemsVolatileFilePool, SystemsVolatileFile) {
    let mut pool = SystemsVolatileFilePool::new("same-owner-label").unwrap();
    let file = pool
        .create(
            "same-owner-label",
            SystemsFileRights::ReadWrite,
            0o600,
            SystemsFileUmask::default(),
            false,
            10,
        )
        .unwrap();
    (pool, file)
}

#[test]
fn compound_stores_update_the_actual_shared_persistent_position() {
    let code = binding();
    let (mut pool, file) = file_pool();
    pool.write(
        "same-owner-label",
        &file,
        b"abcdef",
        SystemsFileUserAdmission {
            numerical_range: true,
            complete_prefix: true,
        },
        10,
    )
    .unwrap();
    pool.seek("same-owner-label", &file, 0, SystemsFileSeek::Set)
        .unwrap();
    let alias = pool.alias("same-owner-label", &file).unwrap();
    let little = format(8, RegionLibraryByteOrder::LittleEndian);
    let big = format(8, RegionLibraryByteOrder::BigEndian);
    let zero = BigUint::from(0_u8);
    for (base, count) in [(0, 1), (1, 2)] {
        let (mut action, view) = pool
            .borrow_position_after_admitted_transfer(
                "same-owner-label",
                &alias,
                SystemsPositionTransfer::Read,
                base,
                count,
                &code,
            )
            .unwrap();
        assert_eq!(
            action.load_unsigned(&view, &zero, little),
            Ok(BigUint::from(base))
        );
        action
            .store_unsigned(&view, &zero, little, &BigUint::from(base + count))
            .unwrap();
        assert_eq!(
            action.load_unsigned(&view, &zero, big),
            Ok(BigUint::from((base + count).swap_bytes()))
        );
        action.complete_position_action(true, proof()).unwrap();
        drop(action);
        assert_eq!(
            pool.metadata("same-owner-label", &file).unwrap().position,
            base + count
        );
        assert_eq!(
            pool.metadata("same-owner-label", &alias).unwrap().position,
            base + count
        );
    }
}

#[test]
fn identical_controller_labels_do_not_merge_real_page_loans() {
    let code = binding();
    let (mut first, a_file) = file_pool();
    let (mut second, b_file) = file_pool();
    let (mut a, a_view) = first
        .borrow_page_action_after_admission(
            "same-owner-label",
            &a_file,
            SystemsRamPageAction::Allocate { logical_page: 0 },
            &code,
        )
        .unwrap();
    let (mut b, b_view) = second
        .borrow_page_action_after_admission(
            "same-owner-label",
            &b_file,
            SystemsRamPageAction::Allocate { logical_page: 0 },
            &code,
        )
        .unwrap();
    let format = format(8, RegionLibraryByteOrder::LittleEndian);
    let zero = BigUint::from(0_u8);
    unchanged(&mut b, RegionLibraryError::View, |i| {
        i.store_unsigned(&a_view, &zero, format, &BigUint::from(1_u8))
    });
    unchanged(&mut a, RegionLibraryError::View, |i| {
        i.load_unsigned(&b_view, &zero, format)
    });
    a.store_unsigned(&a_view, &zero, format, &BigUint::from(1_u8))
        .unwrap();
    b.store_unsigned(
        &b_view,
        &zero,
        RegionLibraryUnsignedFormat::new(8, RegionLibraryByteOrder::BigEndian).unwrap(),
        &BigUint::from(1_u8),
    )
    .unwrap();
    assert_eq!(a.complete_page_action(true, proof()), Ok(0));
    assert_eq!(b.complete_page_action(true, proof()), Ok(7));
    drop(a);
    drop(b);
    assert!(first.metadata("same-owner-label", &a_file).is_ok());
    assert!(second.metadata("same-owner-label", &b_file).is_ok());
}

#[test]
fn encoding_success_cannot_replace_the_owning_controllers_completion_guards() {
    let code = binding();
    let encoding = format(8, RegionLibraryByteOrder::LittleEndian);
    for case in 0..3 {
        let (mut pool, file) = file_pool();
        pool.write(
            "same-owner-label",
            &file,
            b"x",
            SystemsFileUserAdmission {
                numerical_range: true,
                complete_prefix: true,
            },
            10,
        )
        .unwrap();
        pool.seek("same-owner-label", &file, 0, SystemsFileSeek::Set)
            .unwrap();
        let (mut action, view) = pool
            .borrow_position_after_admitted_transfer(
                "same-owner-label",
                &file,
                SystemsPositionTransfer::Read,
                0,
                1,
                &code,
            )
            .unwrap();
        // This encoding is valid even when the application's required value is
        // different; the controller remains responsible for that postcondition.
        let value = if case == 0 { u64::MAX } else { 1 };
        action
            .store_unsigned(&view, &BigUint::from(0_u8), encoding, &BigUint::from(value))
            .unwrap();
        let mut reclamation = proof();
        if case == 2 {
            reclamation.arena_erased = false;
        }
        assert_eq!(
            action.complete_position_action(case != 1, reclamation),
            Err(RegionLibraryError::Invariant)
        );
        // Inspect actual model backing independently after view revocation.
        let Storage::Position { field, .. } = &action.storage else {
            unreachable!()
        };
        assert_eq!(**field, value.to_le_bytes());
        assert_eq!(
            action.load_unsigned(&view, &BigUint::from(0_u8), encoding),
            Err(RegionLibraryError::Ended)
        );
        drop(action);
        assert!(pool.metadata("same-owner-label", &file).is_err());
    }
}

#[test]
fn cancellation_revokes_views_without_rolling_back_successful_compound_stores() {
    let code = binding();
    let (mut owner, region) = storage("cancel-fields");
    let encoding = format(8, RegionLibraryByteOrder::BigEndian);
    let (mut call, view) =
        RegionLibraryInvocation::begin(&mut owner, &region, &code, RegionLibraryAccess::ReadWrite)
            .unwrap();
    call.store_unsigned(
        &view,
        &BigUint::from(1_u8),
        encoding,
        &BigUint::from(u64::MAX),
    )
    .unwrap();
    call.cancel().unwrap();
    unchanged(&mut call, RegionLibraryError::Ended, |i| {
        i.load_unsigned(&view, &BigUint::from(1_u8), encoding)
    });
    drop(call);
    for offset in 0..16 {
        assert_eq!(
            owner.load_byte(&region, offset).unwrap(),
            if (1..9).contains(&offset) {
                255
            } else {
                160 + offset as u8
            }
        );
    }
}
