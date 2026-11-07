//! TOPAL-SYSTEMS-INODE-MODE-ACTION-001; model effects, not executed source.
use super::*;
use crate::{
    RegionActionReclamation, RegionLibraryAccess, RegionLibraryByteOrder,
    RegionLibraryUnsignedFormat,
};
use num_bigint::BigUint;
use std::collections::{BTreeMap, BTreeSet};

fn binding() -> RegionLibraryBinding {
    RegionLibraryBinding::new(
        "inode-action".into(),
        "apply".into(),
        BTreeMap::from([
            ("apply".into(), BTreeSet::from(["store".into()])),
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

fn create_file(pool: &mut SystemsVolatileFilePool) -> SystemsVolatileFile {
    pool.create(
        "owner",
        SystemsFileRights::ReadWrite,
        0o600,
        SystemsFileUmask::default(),
        false,
        100,
    )
    .unwrap()
}

#[test]
fn real_inode_loan_publishes_to_aliases_after_nested_effects_and_complete_reclamation() {
    let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
    let file = create_file(&mut pool);
    let alias = pool.alias("owner", &file).unwrap();
    let graph = binding();
    let mut expected = pool.metadata("owner", &file).unwrap();
    let (mut action, view) = pool
        .borrow_inode_mode_after_admission("owner", &alias, 0o7751, u64::MAX, &graph)
        .unwrap();
    assert_eq!(
        action.inode_mode_action_inputs().unwrap(),
        (0o7751, u64::MAX)
    );
    let resource = action.effect_resource(&view).unwrap();
    let readonly = action
        .derive(&view, 0, 32, RegionLibraryAccess::ReadOnly)
        .unwrap();
    assert_eq!(
        action.store_byte(&readonly, 0, 0),
        Err(RegionLibraryError::ReadOnly)
    );
    let nested = action
        .enter("store", std::slice::from_ref(&view))
        .unwrap()
        .remove(0);
    assert_eq!(
        action.load_byte(&view, 0),
        Err(RegionLibraryError::InactiveFrame)
    );
    assert_eq!(action.effect_resource(&nested).unwrap(), resource);
    store(&mut action, &nested, 0, 0o107751);
    store(&mut action, &nested, 24, u64::MAX);
    assert_eq!(
        action.complete_inode_mode_action(true, proof()),
        Err(RegionLibraryError::NotRoot)
    );
    action.return_from_call().unwrap();
    assert!(action.load_byte(&nested, 0).is_err());
    assert_eq!(action.complete(), Err(RegionLibraryError::Invariant));
    assert_eq!(
        action.complete_position_action(true, proof()),
        Err(RegionLibraryError::Binding)
    );
    assert_eq!(
        action.complete_page_action(true, proof()),
        Err(RegionLibraryError::Binding)
    );
    action.complete_inode_mode_action(true, proof()).unwrap();
    assert_eq!(action.load_byte(&view, 0), Err(RegionLibraryError::Ended));
    drop(action);
    expected.mode = 0o7751;
    expected.changed_seconds = u64::MAX;
    assert_eq!(pool.metadata("owner", &file).unwrap(), expected);
    assert_eq!(pool.metadata("owner", &alias).unwrap(), expected);
    assert_eq!(pool.state.inode_actions.generation, 1);
    assert!(!pool.state.inode_actions.active);
}

#[test]
fn directory_and_closed_linked_file_lend_their_real_blocks_without_creating_descriptions() {
    let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
    let file = pool
        .open_named(
            "owner",
            b"linked",
            SystemsNamedFileOpen {
                rights: SystemsFileRights::Read,
                create: true,
                exclusive: false,
                truncate: false,
                append: false,
                mode: 0o644,
                umask: SystemsFileUmask::default(),
            },
            100,
        )
        .unwrap();
    let graph = binding();
    // A read-only description can lend its owned inode for an admitted chmod.
    let (mut action, view) = pool
        .borrow_inode_mode_after_admission("owner", &file, 0o4000, 101, &graph)
        .unwrap();
    store(&mut action, &view, 0, 0o104000);
    store(&mut action, &view, 24, 101);
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    pool.close("owner", &file).unwrap();
    let descriptions = pool.state.descriptions.clone();
    let (mut action, view) = pool
        .borrow_named_inode_mode_after_admission("owner", b"linked", 0o3456, 102, &graph)
        .unwrap();
    store(&mut action, &view, 0, 0o103456);
    store(&mut action, &view, 24, 102);
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    assert_eq!(
        pool.metadata_by_name("owner", b"linked").unwrap().mode,
        0o3456
    );
    assert_eq!(pool.state.descriptions, descriptions);
    let before = pool.directory_metadata("owner").unwrap();
    let (mut action, view) = pool
        .borrow_directory_mode_after_admission("owner", 0o2750, 103, &graph)
        .unwrap();
    store(&mut action, &view, 0, 0o42750);
    store(&mut action, &view, 24, 103);
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    let after = pool.directory_metadata("owner").unwrap();
    assert_eq!(after.mode(), 0o42750);
    assert_eq!(after.accessed_seconds(), before.accessed_seconds());
    assert_eq!(after.modified_seconds(), before.modified_seconds());
    assert_eq!(after.changed_seconds(), 103);
    assert_eq!(pool.state.inode_actions.generation, 3);
}

#[test]
fn false_wrong_bytes_or_incomplete_reclamation_poison_without_undoing_successful_stores() {
    for failure in 0..9 {
        let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
        let file = create_file(&mut pool);
        let graph = binding();
        let (mut action, view) = pool
            .borrow_inode_mode_after_admission("owner", &file, 0o755, 101, &graph)
            .unwrap();
        store(&mut action, &view, 0, 0o100755);
        if failure != 7 {
            store(&mut action, &view, 24, 101);
        }
        if failure == 8 {
            store(&mut action, &view, 8, 999);
        }
        let mut cleanup = proof();
        match failure {
            0 => cleanup.arena_erased = false,
            1 => cleanup.control_erased = false,
            2 => cleanup.retired_stack_erased = false,
            3 => cleanup.bindings_revoked = false,
            4 => cleanup.caller_restored = false,
            _ => {}
        }
        if failure == 6 {
            drop(action); // Abandonment is not ordinary cancellation/rollback.
        } else {
            assert_eq!(
                action.complete_inode_mode_action(failure != 5, cleanup),
                Err(RegionLibraryError::Invariant),
                "failure {failure}"
            );
            assert!(action.load_byte(&view, 0).is_err());
            drop(action);
        }
        let inode = &pool.state.object(0).unwrap().inode;
        assert_eq!(inode.mode(), 0o100755);
        assert_eq!(
            inode.changed_seconds(),
            if failure == 7 { 100 } else { 101 }
        );
        assert!(pool.state.inode_actions.poisoned);
        assert!(pool.metadata("owner", &file).is_err());
        assert!(pool.chmod("owner", &file, 0, 999).is_err());
        assert!(pool.chmod_directory("owner", 0, 999).is_err());
        assert!(
            pool.borrow_inode_mode_after_admission("owner", &file, 0, 999, &graph)
                .is_err()
        );
        pool.terminate("owner").unwrap();
        assert!(pool.state.objects().iter().all(Option::is_none));
        assert_eq!(pool.state.inode_actions, Default::default());
    }
}

#[test]
fn invalid_facts_foreign_references_mode_and_exhaustion_reject_before_publishing_history() {
    let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
    let file = create_file(&mut pool);
    let mut foreign_pool = SystemsVolatileFilePool::new("owner").unwrap();
    let foreign = create_file(&mut foreign_pool);
    let graph = binding();
    let before = pool.state.clone();
    for permissions in [4096, 65535, u64::MAX] {
        assert!(
            pool.borrow_inode_mode_after_admission("owner", &file, permissions, 999, &graph)
                .is_err()
        );
        assert!(
            pool.borrow_directory_mode_after_admission("owner", permissions, 999, &graph)
                .is_err()
        );
    }
    assert!(
        pool.borrow_inode_mode_after_admission("owner", &foreign, 0, 999, &graph)
            .is_err()
    );
    assert!(
        pool.borrow_inode_mode_after_admission("other", &file, 0, 999, &graph)
            .is_err()
    );
    assert!(
        pool.borrow_named_inode_mode_after_admission("owner", b"absent", 0, 999, &graph)
            .is_err()
    );
    assert_eq!(pool.state, before);
    pool.state.inode_actions.generation = u64::MAX;
    let before = pool.state.clone();
    assert!(
        pool.borrow_inode_mode_after_admission("owner", &file, 0, 999, &graph)
            .is_err()
    );
    assert_eq!(pool.state, before);
    pool.state.inode_actions.generation = 0;
    pool.state.object_inode_bytes_mut(0)[7] = 1;
    let before = pool.state.clone();
    assert!(
        pool.borrow_inode_mode_after_admission("owner", &file, 0, 999, &graph)
            .is_err()
    );
    assert_eq!(pool.state, before);
}

#[test]
fn object_reuse_does_not_reset_invocation_history_or_reauthorize_a_retired_view() {
    let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
    let first = create_file(&mut pool);
    let graph = binding();
    let (mut action, old) = pool
        .borrow_inode_mode_after_admission("owner", &first, 0o755, 101, &graph)
        .unwrap();
    store(&mut action, &old, 0, 0o100755);
    store(&mut action, &old, 24, 101);
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    pool.close("owner", &first).unwrap();
    let second = create_file(&mut pool);
    let (mut action, view) = pool
        .borrow_inode_mode_after_admission("owner", &second, 0o640, 102, &graph)
        .unwrap();
    assert_eq!(action.load_byte(&old, 0), Err(RegionLibraryError::View));
    store(&mut action, &view, 0, 0o100640);
    store(&mut action, &view, 24, 102);
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    assert_eq!(pool.state.inode_actions.generation, 2);
    assert_eq!(pool.metadata("owner", &second).unwrap().mode, 0o640);
}

#[test]
fn identical_labels_and_generations_do_not_merge_inode_position_or_page_domains() {
    // Same actual filesystem, graph label and epoch: the resource class must
    // still reject a retired inode view at each independently admitted caller.
    let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
    let file = create_file(&mut pool);
    let graph = binding();
    let (mut action, old) = pool
        .borrow_inode_mode_after_admission("owner", &file, 0o600, 100, &graph)
        .unwrap();
    store(&mut action, &old, 0, 0o100600);
    store(&mut action, &old, 24, 100);
    let inode_effect = action.effect_resource(&old).unwrap();
    action.complete_inode_mode_action(true, proof()).unwrap();
    drop(action);
    let (mut position, position_view) = pool
        .borrow_position_after_admitted_transfer(
            "owner",
            &file,
            SystemsPositionTransfer::Write,
            0,
            1,
            &graph,
        )
        .unwrap();
    assert_eq!(
        position.effect_resource(&position_view).unwrap(),
        inode_effect
    );
    assert_eq!(position.load_byte(&old, 0), Err(RegionLibraryError::View));
    assert_eq!(
        position.complete_inode_mode_action(true, proof()),
        Err(RegionLibraryError::Binding)
    );
    store(&mut position, &position_view, 0, 1);
    position.complete_position_action(true, proof()).unwrap();
    drop(position);
    let (mut page, page_view) = pool
        .borrow_page_action_after_admission(
            "owner",
            &file,
            SystemsRamPageAction::Allocate { logical_page: 0 },
            &graph,
        )
        .unwrap();
    assert_eq!(page.effect_resource(&page_view).unwrap(), inode_effect);
    assert_eq!(page.load_byte(&old, 0), Err(RegionLibraryError::View));
    page.store_byte(&page_view, 0, 1).unwrap();
    page.complete_page_action(true, proof()).unwrap();
    drop(page);
    assert_eq!(pool.state.inode_actions.generation, 1);
    assert_eq!(pool.state.position_actions.generation, 1);
    assert_eq!(pool.state.page_actions.generation, 1);
}
