//! TK-DEC-269: reference semantics only, not native or source qualification.
use super::*;
use crate::{BootstrapStorageDescriptor, BootstrapStoragePlacement, BootstrapStorageRequest};

fn binding(views: usize) -> RegionLibraryBinding {
    RegionLibraryBinding::new(
        "selected-source-and-code".into(),
        "entry".into(),
        BTreeMap::from([
            ("entry".into(), BTreeSet::from(["helper".into()])),
            ("helper".into(), BTreeSet::from(["leaf".into()])),
            ("leaf".into(), BTreeSet::new()),
        ]),
        3,
        views,
    )
    .unwrap()
}

fn storage(identity: &str) -> (BootstrapStorageState, BootstrapRegion) {
    let mut storage = BootstrapStorageState::new(
        BootstrapStorageDescriptor {
            capacity_bytes: 4096,
            alignment_bytes: 8,
        },
        identity,
    )
    .unwrap();
    let region = storage
        .allocate(BootstrapStorageRequest {
            byte_count: 16,
            alignment_bytes: 8,
            placement: BootstrapStoragePlacement::BootstrapReclaimable,
        })
        .unwrap();
    (storage, region)
}

fn unchanged<T: std::fmt::Debug>(
    invocation: &mut RegionLibraryInvocation<'_>,
    error: RegionLibraryError,
    operation: impl FnOnce(&mut RegionLibraryInvocation<'_>) -> Result<T, RegionLibraryError>,
) {
    // Includes complete underlying storage contents, counters and observations.
    let before = format!("{invocation:?}");
    assert_eq!(operation(invocation).unwrap_err(), error);
    assert_eq!(format!("{invocation:?}"), before);
}

#[test]
fn binding_requires_closed_nonrecursive_code_and_complete_limits() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-BINDING-001, -CALL-001.
    let valid = binding(8);
    for (frames, views) in [(0, 8), (65, 8), (2, 8), (3, 0), (3, 257)] {
        assert_eq!(
            RegionLibraryBinding::new(
                valid.identity.clone(),
                valid.entry.clone(),
                valid.calls.clone(),
                frames,
                views,
            ),
            Err(RegionLibraryError::Binding)
        );
    }
    let mut unknown = valid.calls.clone();
    unknown.get_mut("leaf").unwrap().insert("absent".into());
    let mut cycle = valid.calls.clone();
    cycle.get_mut("leaf").unwrap().insert("entry".into());
    let mut unreachable = valid.calls.clone();
    unreachable.insert("unused".into(), BTreeSet::new());
    for graph in [unknown, cycle, unreachable] {
        assert!(RegionLibraryBinding::new("code".into(), "entry".into(), graph, 64, 8).is_err());
    }
    assert!(
        RegionLibraryBinding::new("code".into(), "absent".into(), valid.calls.clone(), 3, 8)
            .is_err()
    );
    assert!(RegionLibraryBinding::new("".into(), valid.entry, valid.calls, 3, 8).is_err());
    let mut graph = BTreeMap::from([("entry".into(), BTreeSet::new())]);
    for index in 0..63 {
        let name = format!("leaf{index}");
        graph.get_mut("entry").unwrap().insert(name.clone());
        graph.insert(name, BTreeSet::new());
    }
    assert!(RegionLibraryBinding::new("code".into(), "entry".into(), graph.clone(), 2, 1).is_ok());
    graph.get_mut("entry").unwrap().insert("overflow".into());
    graph.insert("overflow".into(), BTreeSet::new());
    assert!(RegionLibraryBinding::new("code".into(), "entry".into(), graph, 2, 1).is_err());
}

#[test]
fn maximum_call_depth_retains_every_suspended_owner_view() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-CALL-001, -LIFETIME-001.
    let graph = (0..64)
        .map(|index| {
            (
                format!("f{index}"),
                if index == 63 {
                    BTreeSet::new()
                } else {
                    BTreeSet::from([format!("f{}", index + 1)])
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert!(RegionLibraryBinding::new("code".into(), "f0".into(), graph.clone(), 63, 64).is_err());
    let code = RegionLibraryBinding::new("code".into(), "f0".into(), graph, 64, 64).unwrap();
    let (mut storage, region) = storage("depth");
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    let mut current = root.clone();
    for index in 1..64 {
        current = i.enter(&format!("f{index}"), &[current]).unwrap().remove(0);
    }
    assert_eq!(i.frames.len(), 64);
    assert_eq!(i.windows.len(), 64);
    i.store_byte(&current, 0, 99).unwrap();
    for _ in 1..64 {
        i.return_from_call().unwrap();
    }
    assert_eq!(i.windows.len(), 1);
    assert_eq!(i.load_byte(&root, 0), Ok(99));
    i.complete().unwrap();
}

#[test]
fn nested_aliases_change_the_existing_owner_bytes() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-CALL-001, -ACCESS-001, -LIFETIME-001.
    let (mut storage, region) = storage("nested");
    let code = binding(16);
    let (mut invocation, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    let slice = invocation
        .derive(&root, 4, 4, RegionLibraryAccess::ReadWrite)
        .unwrap();
    let args = invocation
        .enter("helper", &[slice.clone(), root.clone()])
        .unwrap();
    invocation.store_byte(&args[0], 1, 91).unwrap();
    assert_eq!(invocation.load_byte(&args[1], 5), Ok(91));
    assert_eq!(args[0].owner, args[1].owner);
    unchanged(&mut invocation, RegionLibraryError::InactiveFrame, |i| {
        i.load_byte(&root, 5)
    });
    let leaf = invocation.enter("leaf", &[args[0].clone()]).unwrap();
    invocation.store_byte(&leaf[0], 2, 255).unwrap();
    let local = invocation
        .derive(&leaf[0], 1, 2, RegionLibraryAccess::ReadOnly)
        .unwrap();
    assert_eq!(invocation.load_byte(&local, 1), Ok(255));
    invocation.return_from_call().unwrap();
    unchanged(&mut invocation, RegionLibraryError::InactiveFrame, |i| {
        i.load_byte(&local, 0)
    });
    assert_eq!(invocation.load_byte(&args[1], 6), Ok(255));
    invocation.return_from_call().unwrap();
    assert_eq!(invocation.load_byte(&slice, 1), Ok(91));
    unchanged(&mut invocation, RegionLibraryError::InactiveFrame, |i| {
        i.load_byte(&args[0], 0)
    });
    invocation.complete().unwrap();
    assert!(invocation.windows.is_empty() && invocation.frames.is_empty());
    drop(invocation);
    assert_eq!(storage.load_byte(&region, 5).unwrap(), 91);
    assert_eq!(storage.load_byte(&region, 6).unwrap(), 255);
    storage.release(region).unwrap();
    storage.complete_bootstrap().unwrap();
}

#[test]
fn access_refusals_do_not_reach_storage() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-ACCESS-001.
    let (mut storage, region) = storage("range");
    let code = binding(8);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    i.store_byte(&root, 15, 255).unwrap();
    for offset in [16, u64::MAX] {
        unchanged(&mut i, RegionLibraryError::Bounds, |i| {
            i.load_byte(&root, offset)
        });
        unchanged(&mut i, RegionLibraryError::Bounds, |i| {
            i.store_byte(&root, offset, 1)
        });
    }
    for value in [256, u64::MAX] {
        unchanged(&mut i, RegionLibraryError::ByteValue, |i| {
            i.store_byte(&root, 0, value)
        });
    }
    for (offset, count) in [(17, 0), (16, 1), (u64::MAX, 2), (1, u64::MAX)] {
        unchanged(&mut i, RegionLibraryError::Bounds, |i| {
            i.derive(&root, offset, count, RegionLibraryAccess::ReadWrite)
        });
    }
    let empty = i
        .derive(&root, 16, 0, RegionLibraryAccess::ReadOnly)
        .unwrap();
    unchanged(&mut i, RegionLibraryError::Bounds, |i| {
        i.load_byte(&empty, 0)
    });
    assert_eq!(i.load_byte(&root, 15), Ok(255));
}

#[test]
fn narrowed_rights_and_windows_cannot_be_widened() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-ACCESS-001, -CALL-001.
    let (mut storage, region) = storage("rights");
    let code = binding(8);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    let read = i
        .derive(&root, 4, 3, RegionLibraryAccess::ReadOnly)
        .unwrap();
    unchanged(&mut i, RegionLibraryError::ReadOnly, |i| {
        i.store_byte(&read, 0, 1)
    });
    unchanged(&mut i, RegionLibraryError::ReadOnly, |i| {
        i.derive(&read, 0, 3, RegionLibraryAccess::ReadWrite)
    });
    unchanged(&mut i, RegionLibraryError::Bounds, |i| {
        i.derive(&read, 0, 4, RegionLibraryAccess::ReadOnly)
    });
    let args = i.enter("helper", &[read]).unwrap();
    unchanged(&mut i, RegionLibraryError::ReadOnly, |i| {
        i.store_byte(&args[0], 0, 1)
    });
    assert_eq!(i.load_byte(&args[0], 0), Ok(0));
}

#[test]
fn readonly_initial_binding_stays_readonly() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-BINDING-001, -ACCESS-001.
    let (mut storage, region) = storage("readonly");
    let code = binding(8);
    let (mut i, root) =
        RegionLibraryInvocation::begin(&mut storage, &region, &code, RegionLibraryAccess::ReadOnly)
            .unwrap();
    unchanged(&mut i, RegionLibraryError::ReadOnly, |i| {
        i.store_byte(&root, 0, 1)
    });
    unchanged(&mut i, RegionLibraryError::ReadOnly, |i| {
        i.derive(&root, 0, 1, RegionLibraryAccess::ReadWrite)
    });
}

#[test]
fn alias_effects_follow_the_owner_across_windows_calls_and_invocations() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-CALL-001.
    let (mut storage, region) = storage("effects");
    let code = binding(8);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    let owner = i.effect_resource(&root).unwrap();
    let left = i
        .derive(&root, 0, 1, RegionLibraryAccess::ReadOnly)
        .unwrap();
    let right = i
        .derive(&root, 15, 1, RegionLibraryAccess::ReadWrite)
        .unwrap();
    assert_eq!(i.effect_resource(&left), Ok(owner.clone()));
    assert_eq!(i.effect_resource(&right), Ok(owner.clone()));
    let args = i.enter("helper", &[left, right]).unwrap();
    assert_eq!(i.effect_resource(&args[0]), Ok(owner.clone()));
    assert_eq!(i.effect_resource(&args[1]), Ok(owner.clone()));
    assert_eq!(
        i.effect_resource(&root),
        Err(RegionLibraryError::InactiveFrame)
    );
    i.cancel().unwrap();
    assert_eq!(i.effect_resource(&args[0]), Err(RegionLibraryError::Ended));
    drop(i);
    let (mut i, root) =
        RegionLibraryInvocation::begin(&mut storage, &region, &code, RegionLibraryAccess::ReadOnly)
            .unwrap();
    assert_eq!(i.effect_resource(&root), Ok(owner));
    i.complete().unwrap();
}

#[test]
fn all_call_arguments_and_selected_edges_are_admitted_before_entry() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-CALL-001.
    let (mut storage, region) = storage("arguments");
    let code = binding(8);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    let mut wrong = root.clone();
    wrong.generation += 1;
    unchanged(&mut i, RegionLibraryError::View, |i| {
        i.enter("helper", &[root.clone(), wrong])
    });
    unchanged(&mut i, RegionLibraryError::Binding, |i| {
        i.enter("leaf", std::slice::from_ref(&root))
    });
    unchanged(&mut i, RegionLibraryError::Binding, |i| {
        i.enter("missing", &[])
    });
    unchanged(&mut i, RegionLibraryError::NotRoot, |i| {
        i.return_from_call()
    });
}

#[test]
fn view_capacity_includes_suspended_frames_and_alias_arguments() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-CALL-001.
    let (mut storage, region) = storage("capacity");
    let code = binding(2);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    unchanged(&mut i, RegionLibraryError::Capacity, |i| {
        i.enter("helper", &[root.clone(), root.clone()])
    });
    let args = i.enter("helper", std::slice::from_ref(&root)).unwrap();
    unchanged(&mut i, RegionLibraryError::Capacity, |i| {
        i.derive(&args[0], 0, 1, RegionLibraryAccess::ReadOnly)
    });
    i.return_from_call().unwrap();
    let next = i.enter("helper", std::slice::from_ref(&root)).unwrap();
    assert_ne!(args[0].frame, next[0].frame);
    assert_ne!(args[0].identity, next[0].identity);
    unchanged(&mut i, RegionLibraryError::InactiveFrame, |i| {
        i.load_byte(&args[0], 0)
    });
    assert_eq!(i.load_byte(&next[0], 0), Ok(0));
}

#[test]
fn full_view_budget_is_checked_without_reusing_identifiers() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-CALL-001.
    let (mut storage, region) = storage("maximum-views");
    let code = binding(256);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    for _ in 1..256 {
        i.derive(&root, 0, 16, RegionLibraryAccess::ReadOnly)
            .unwrap();
    }
    assert_eq!(i.windows.len(), 256);
    unchanged(&mut i, RegionLibraryError::Capacity, |i| {
        i.derive(&root, 0, 1, RegionLibraryAccess::ReadOnly)
    });
}

#[test]
fn counter_exhaustion_does_not_publish_a_partial_view_or_call() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-BINDING-001, -CALL-001.
    let (mut storage, region) = storage("overflow");
    let code = binding(8);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    i.next_view = u64::MAX;
    unchanged(&mut i, RegionLibraryError::IdentityExhausted, |i| {
        i.derive(&root, 0, 1, RegionLibraryAccess::ReadOnly)
    });
    unchanged(&mut i, RegionLibraryError::IdentityExhausted, |i| {
        i.enter("helper", std::slice::from_ref(&root))
    });
    i.next_view = 2;
    i.next_frame = u64::MAX;
    unchanged(&mut i, RegionLibraryError::IdentityExhausted, |i| {
        i.enter("helper", std::slice::from_ref(&root))
    });
}

#[test]
fn invocation_generations_reject_retained_views_after_completion() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-LIFETIME-001.
    let (mut storage, region) = storage("generations");
    let code = binding(8);
    let (mut i, stale) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    i.store_byte(&stale, 0, 72).unwrap();
    i.complete().unwrap();
    unchanged(&mut i, RegionLibraryError::Ended, |i| {
        i.load_byte(&stale, 0)
    });
    unchanged(&mut i, RegionLibraryError::Ended, |i| i.complete());
    drop(i);
    let (mut i, current) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    assert_ne!(stale.generation, current.generation);
    unchanged(&mut i, RegionLibraryError::View, |i| i.load_byte(&stale, 0));
    assert_eq!(i.load_byte(&current, 0), Ok(72));
}

#[test]
fn cancelled_nested_calls_revoke_views_without_rolling_back_stores() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-LIFETIME-001.
    let (mut storage, region) = storage("cancel");
    let code = binding(8);
    let (mut i, root) = RegionLibraryInvocation::begin(
        &mut storage,
        &region,
        &code,
        RegionLibraryAccess::ReadWrite,
    )
    .unwrap();
    let args = i.enter("helper", std::slice::from_ref(&root)).unwrap();
    i.store_byte(&args[0], 7, 81).unwrap();
    unchanged(&mut i, RegionLibraryError::NotRoot, |i| i.complete());
    i.cancel().unwrap();
    assert!(i.windows.is_empty() && i.frames.is_empty());
    unchanged(&mut i, RegionLibraryError::Ended, |i| {
        i.load_byte(&args[0], 7)
    });
    unchanged(&mut i, RegionLibraryError::Ended, |i| i.load_byte(&root, 7));
    drop(i);
    assert_eq!(storage.load_byte(&region, 7), Ok(81));
}

#[test]
fn implicit_drop_cancels_without_rolling_back_owner_bytes() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-LIFETIME-001.
    let (mut storage, region) = storage("drop");
    let code = binding(8);
    {
        let (mut i, root) = RegionLibraryInvocation::begin(
            &mut storage,
            &region,
            &code,
            RegionLibraryAccess::ReadWrite,
        )
        .unwrap();
        let args = i.enter("helper", &[root]).unwrap();
        i.store_byte(&args[0], 2, 61).unwrap();
    }
    assert_eq!(storage.load_byte(&region, 2), Ok(61));
    storage.release(region).unwrap();
    storage.complete_bootstrap().unwrap();
}

#[test]
fn wrong_owner_and_binding_never_authorize_access() {
    // TOPAL-SYSTEMS-REGION-LIBRARY-BINDING-001, -ACCESS-001.
    let (mut first, region) = storage("first");
    let (mut second, _) = storage("second");
    let code = binding(8);
    let before = format!("{second:?}");
    assert_eq!(RegionLibraryInvocation::begin(
        &mut second, &region, &code, RegionLibraryAccess::ReadWrite,
    ).unwrap_err(), RegionLibraryError::Owner);
    assert_eq!(format!("{second:?}"), before);
    let (mut i, root) =
        RegionLibraryInvocation::begin(&mut first, &region, &code, RegionLibraryAccess::ReadWrite)
            .unwrap();
    let mut wrong_owner = root.clone();
    wrong_owner.owner.1 += 1;
    let mut wrong_code = root.clone();
    wrong_code.binding = "other-code".into();
    let mut wrong_window = root.clone();
    wrong_window.identity += 10;
    for wrong in [wrong_owner, wrong_code, wrong_window] {
        unchanged(&mut i, RegionLibraryError::View, |i| i.load_byte(&wrong, 0));
    }
}
