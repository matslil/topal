//! TOPAL-SYSTEMS-INODE-MODE-ACTION-001; pipe owners remain a separate domain.
use super::*;
use crate::{RegionActionReclamation, RegionLibraryByteOrder, RegionLibraryUnsignedFormat};
use num_bigint::BigUint;
use std::collections::BTreeSet;

fn binding() -> RegionLibraryBinding {
    RegionLibraryBinding::new(
        "inode-action".into(),
        "apply".into(),
        BTreeMap::from([("apply".into(), BTreeSet::new())]),
        1,
        2,
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
fn store(
    action: &mut RegionLibraryInvocation<'_>,
    view: &RegionLibraryView,
    offset: u64,
    value: u64,
) {
    action
        .store_unsigned(
            view,
            &BigUint::from(offset),
            RegionLibraryUnsignedFormat::new(8, RegionLibraryByteOrder::LittleEndian).unwrap(),
            &BigUint::from(value),
        )
        .unwrap();
}

#[test]
fn pipe_loan_updates_actual_shared_metadata_and_keeps_other_owner_views_separate() {
    let mut first = SystemsPipePool::new("owner").unwrap();
    let mut second = SystemsPipePool::new("owner").unwrap();
    let (read, write) = first.create("owner", false, true, 100).unwrap();
    let (foreign, _) = second.create("owner", false, true, 100).unwrap();
    let graph = binding();
    let before = first.metadata("owner", &write).unwrap();
    let (mut action, view) = first
        .borrow_inode_mode_after_admission("owner", &read, 0o6754, 101, &graph)
        .unwrap();
    let (mut other, foreign_view) = second
        .borrow_inode_mode_after_admission("owner", &foreign, 0o600, 100, &graph)
        .unwrap();
    assert_eq!(
        action.load_byte(&foreign_view, 0),
        Err(RegionLibraryError::View)
    );
    assert_eq!(other.load_byte(&view, 0), Err(RegionLibraryError::View));
    store(&mut other, &foreign_view, 0, 0o10600);
    store(&mut other, &foreign_view, 24, 100);
    other.complete_inode_mode_action(true, proof()).unwrap();
    drop(other);
    store(&mut action, &view, 0, 0o16754);
    store(&mut action, &view, 24, 101);
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    let mut expected = before;
    expected.mode = 0o6754;
    expected.changed_seconds = 101;
    assert_eq!(first.metadata("owner", &read).unwrap(), expected);
    assert_eq!(first.metadata("owner", &write).unwrap(), expected);
    first.close("owner", &read).unwrap();
    first.close("owner", &write).unwrap();
    let (fresh, _) = first.create("owner", false, true, 102).unwrap();
    let (mut action, fresh_view) = first
        .borrow_inode_mode_after_admission("owner", &fresh, 0o600, 102, &graph)
        .unwrap();
    assert_eq!(action.load_byte(&view, 0), Err(RegionLibraryError::View));
    store(&mut action, &fresh_view, 0, 0o10600);
    store(&mut action, &fresh_view, 24, 102);
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    assert_eq!(first.inode_actions.generation, 2);
}

#[test]
fn pipe_false_completion_or_abandonment_preserves_stores_and_allows_only_terminal_reclamation() {
    for abandon in [false, true] {
        let mut pool = SystemsPipePool::new("owner").unwrap();
        let (read, write) = pool.create("owner", false, true, 100).unwrap();
        let graph = binding();
        let (mut action, view) = pool
            .borrow_inode_mode_after_admission("owner", &write, 0o755, 101, &graph)
            .unwrap();
        store(&mut action, &view, 0, 0o10755);
        if !abandon {
            assert_eq!(
                action.complete_inode_mode_action(false, proof()),
                Err(RegionLibraryError::Invariant)
            );
        }
        drop(action);
        assert_eq!(pool.objects[0].as_ref().unwrap().inode.mode(), 0o10755);
        assert!(pool.inode_actions.poisoned);
        assert!(pool.metadata("owner", &read).is_err());
        assert!(pool.read("owner", &read, 0, true, 999).is_err());
        assert!(pool.write("owner", &write, b"", true, false, 999).is_err());
        assert!(pool.chmod("owner", &read, 0, 999).is_err());
        assert!(pool.terminate("other").is_err());
        pool.terminate("owner").unwrap();
        assert!(pool.objects.iter().all(Option::is_none));
        assert!(pool.endpoints.is_empty() && pool.waits.is_empty());
        assert_eq!(pool.inode_actions, Default::default());
    }
}

#[test]
fn pipe_invalid_facts_stale_identity_and_generation_exhaustion_leave_bytes_and_history_unchanged() {
    let mut pool = SystemsPipePool::new("owner").unwrap();
    let (read, write) = pool.create("owner", false, true, 100).unwrap();
    let graph = binding();
    let before = pool.objects.clone();
    for permissions in [4096, u64::MAX] {
        assert!(
            pool.borrow_inode_mode_after_admission("owner", &read, permissions, 999, &graph)
                .is_err()
        );
    }
    assert!(
        pool.borrow_inode_mode_after_admission("other", &write, 0, 999, &graph)
            .is_err()
    );
    assert_eq!(pool.objects, before);
    assert_eq!(pool.inode_actions, Default::default());
    pool.inode_actions.generation = u64::MAX;
    let domain = pool.inode_actions.clone();
    assert!(
        pool.borrow_inode_mode_after_admission("owner", &read, 0, 999, &graph)
            .is_err()
    );
    assert_eq!(pool.objects, before);
    assert_eq!(pool.inode_actions, domain);
    pool.inode_actions.generation = 0;
    pool.close("owner", &read).unwrap();
    pool.close("owner", &write).unwrap();
    let before = pool.objects.clone();
    assert!(
        pool.borrow_inode_mode_after_admission("owner", &read, 0, 999, &graph)
            .is_err()
    );
    assert_eq!(pool.objects, before);
    assert_eq!(pool.inode_actions, Default::default());
}
