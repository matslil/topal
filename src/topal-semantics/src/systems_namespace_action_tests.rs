//! TOPAL-SYSTEMS-NAMESPACE-{OWNERSHIP,TRANSACTION,ACTION}-001.
//! Byte effects on the actual owner; not execution of an opaque source body.
use super::*;
use crate::{RegionLibraryAccess, SystemsPositionTransfer, SystemsRamPageAction};
use std::collections::{BTreeMap, BTreeSet};

const COPY: SystemsFileUserAdmission = SystemsFileUserAdmission {
    numerical_range: true,
    complete_prefix: true,
};
fn binding() -> RegionLibraryBinding {
    RegionLibraryBinding::new(
        "shared-model-binding".into(),
        "apply".into(),
        BTreeMap::from([
            ("apply".into(), BTreeSet::from(["store".into()])),
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
fn request() -> SystemsNamedFileOpen {
    SystemsNamedFileOpen {
        rights: SystemsFileRights::ReadWrite,
        create: true,
        exclusive: true,
        truncate: false,
        append: false,
        mode: 0o660,
        umask: SystemsFileUmask::default(),
    }
}
fn create(pool: &mut SystemsVolatileFilePool, name: &[u8], bytes: &[u8]) -> SystemsVolatileFile {
    let file = pool.open_named("fs", name, request(), 100).unwrap();
    if !bytes.is_empty() {
        pool.write("fs", &file, bytes, COPY, 101).unwrap();
    }
    file
}
fn fixture() -> (
    SystemsVolatileFilePool,
    SystemsVolatileFile,
    SystemsVolatileFile,
) {
    let mut pool = SystemsVolatileFilePool::new("fs").unwrap();
    let a = create(&mut pool, b"a", b"source bytes");
    let b = create(&mut pool, b"b", b"victim bytes");
    (pool, a, b)
}
fn load(call: &mut RegionLibraryInvocation<'_>, view: &RegionLibraryView, offset: usize) -> u64 {
    let mut raw = [0; 8];
    for (index, byte) in raw.iter_mut().enumerate() {
        *byte = call.load_byte(view, (offset + index) as u64).unwrap();
    }
    u64::from_le_bytes(raw)
}
fn store(
    call: &mut RegionLibraryInvocation<'_>,
    view: &RegionLibraryView,
    offset: usize,
    value: u64,
) {
    for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
        call.store_byte(view, (offset + index) as u64, u64::from(byte))
            .unwrap();
    }
}
fn bytes(call: &mut RegionLibraryInvocation<'_>, view: &RegionLibraryView) -> [u8; PAGE] {
    std::array::from_fn(|offset| call.load_byte(view, offset as u64).unwrap())
}
/// Supplied test byte operations, independently compared with the portable
/// direct transition. This function is not the compiled Topal implementation.
fn mutate(call: &mut RegionLibraryInvocation<'_>, view: &RegionLibraryView) {
    let (operation, seconds) = call.namespace_action_inputs().unwrap();
    let source = load(call, view, 3072) as usize;
    let old = load(call, view, 3080) as usize;
    let target = load(call, view, 3088) as usize;
    let next = load(call, view, 3096) as usize;
    let length = load(call, view, 3104);
    let identity = load(call, view, 3112);
    let victim = load(call, view, 3120);
    if operation <= 1 {
        let entry = if operation == 0 { next } else { old };
        for index in 0..256 {
            let byte = call.load_byte(view, (3200 + index) as u64).unwrap();
            call.store_byte(view, (1536 + entry * 256 + index) as u64, u64::from(byte))
                .unwrap();
        }
        store(call, view, 72 + entry * 16, length);
        if operation == 0 {
            let count = load(call, view, 256 + source * 128 + 56);
            store(call, view, 64 + entry * 16, identity);
            store(call, view, 256 + source * 128 + 56, count + 1);
            store(call, view, 256 + source * 128 + 64, 0);
        }
    } else if operation == 2 {
        let count = load(call, view, 256 + target * 128 + 56);
        store(call, view, 64 + next * 16, identity);
        store(call, view, 64 + old * 16, 0);
        store(call, view, 72 + old * 16, 0);
        for index in 0..256 {
            call.store_byte(view, (1536 + old * 256 + index) as u64, 0)
                .unwrap();
        }
        store(call, view, 256 + target * 128 + 56, count - 1);
        store(call, view, 256 + target * 128 + 48, seconds);
    } else {
        assert_eq!(operation, 3);
        store(call, view, 64 + old * 16, victim);
        store(call, view, 64 + next * 16, identity);
        store(call, view, 256 + target * 128 + 48, seconds);
    }
    store(call, view, 256 + source * 128 + 48, seconds);
    store(call, view, 48, seconds);
    store(call, view, 56, seconds);
}

#[test]
fn canonical_page_geometry_is_independent_of_decoded_observation_helpers() {
    let mut pool = SystemsVolatileFilePool::new("fs").unwrap();
    create(&mut pool, b"alpha", b"");
    let mut expected = [0; PAGE];
    for (offset, value) in [
        (0, 8),
        (32, 0o41777),
        (48, 100),
        (56, 100),
        (64, 7),
        (72, 5),
        (256, 1),
        (264, 7),
        (280, 0o100640),
        (288, 100),
        (296, 100),
        (304, 100),
        (312, 1),
    ] {
        expected[offset..offset + 8].copy_from_slice(&u64::to_le_bytes(value));
    }
    expected[1536..1541].copy_from_slice(b"alpha");
    assert_eq!(pool.state.metadata, expected);
    pool.terminate("fs").unwrap();
    assert_eq!(pool.state.metadata, [0; PAGE]);
}

#[test]
fn actual_namespace_loans_match_portable_transitions_and_all_alias_observations() {
    let graph = binding();
    for operation in 0..4 {
        for length in [1, 255] {
            let (mut pool, a, b) = fixture();
            let (mut oracle, oa, ob) = fixture();
            let name = vec![0xff; length];
            for p in [&mut pool, &mut oracle] {
                p.hard_link("fs", b"a", b"a2", 102).unwrap();
                if operation == 1 || operation == 2 {
                    p.hard_link("fs", b"b", b"b2", 103).unwrap();
                }
            }
            let choose = |operation| {
                if operation == 3 {
                    SystemsFileRename::Exchange
                } else {
                    SystemsFileRename::Replace
                }
            };
            if operation == 0 {
                oracle.hard_link("fs", b"a", &name, u64::MAX).unwrap();
            } else {
                oracle
                    .rename(
                        "fs",
                        b"a",
                        if operation == 1 { &name } else { b"b" },
                        choose(operation),
                        u64::MAX,
                    )
                    .unwrap();
            }
            let selected = if operation == 0 {
                SystemsNamespaceRequest::Link {
                    source: b"a",
                    destination: &name,
                }
            } else {
                SystemsNamespaceRequest::Rename {
                    source: b"a",
                    destination: if operation == 1 { &name } else { b"b" },
                    operation: choose(operation),
                }
            };
            let (mut call, view) = pool
                .borrow_namespace_action_after_admission("fs", selected, u64::MAX, &graph)
                .unwrap()
                .unwrap();
            assert_eq!(
                call.namespace_action_inputs().unwrap(),
                (operation, u64::MAX)
            );
            let before = bytes(&mut call, &view);
            let readonly = call
                .derive(&view, 0, PAGE as u64, RegionLibraryAccess::ReadOnly)
                .unwrap();
            assert_eq!(
                call.store_byte(&readonly, 0, 1),
                Err(RegionLibraryError::ReadOnly)
            );
            assert_eq!(
                call.store_byte(&view, PAGE as u64, 1),
                Err(RegionLibraryError::Bounds)
            );
            assert_eq!(
                call.store_byte(&view, 0, 256),
                Err(RegionLibraryError::ByteValue)
            );
            assert_eq!(bytes(&mut call, &view), before);
            let nested = call
                .enter("store", std::slice::from_ref(&view))
                .unwrap()
                .remove(0);
            assert_eq!(
                call.load_byte(&view, 0),
                Err(RegionLibraryError::InactiveFrame)
            );
            mutate(&mut call, &nested);
            assert_eq!(
                call.complete_namespace_action(true, proof()),
                Err(RegionLibraryError::NotRoot)
            );
            call.return_from_call().unwrap();
            assert!(call.load_byte(&nested, 0).is_err());
            assert_eq!(call.complete(), Err(RegionLibraryError::Invariant));
            assert_eq!(call.cancel(), Err(RegionLibraryError::Invariant));
            assert_eq!(
                call.complete_page_action(true, proof()),
                Err(RegionLibraryError::Binding)
            );
            assert_eq!(
                call.complete_inode_mode_action(true, proof()),
                Err(RegionLibraryError::Binding)
            );
            assert_eq!(
                call.complete_position_action(true, proof()),
                Err(RegionLibraryError::Binding)
            );
            call.complete_namespace_action(true, proof()).unwrap();
            assert!(call.load_byte(&view, 0).is_err());
            drop(call);
            oracle.state.namespace_actions.generation = 1;
            assert_eq!(pool.state, oracle.state);
            assert_eq!(
                pool.metadata("fs", &a).unwrap(),
                oracle.metadata("fs", &oa).unwrap()
            );
            assert_eq!(
                pool.metadata("fs", &b).unwrap(),
                oracle.metadata("fs", &ob).unwrap()
            );
            assert_eq!(
                pool.read_at("fs", &a, 64, 0, COPY).unwrap(),
                b"source bytes"
            );
            assert_eq!(
                pool.read_at("fs", &b, 64, 0, COPY).unwrap(),
                b"victim bytes"
            );
            assert!(
                pool.state.metadata[3072..3584]
                    .iter()
                    .all(|&byte| byte == 0)
            );
        }
    }
}

#[test]
fn replacement_reclaims_only_after_loan_retirement_and_last_description() {
    let graph = binding();
    for retained in [false, true] {
        let (mut pool, a, b) = fixture();
        let victim = pool.metadata("fs", &b).unwrap().identity;
        if !retained {
            pool.close("fs", &b).unwrap();
        }
        let (mut call, view) = pool
            .borrow_namespace_action_after_admission(
                "fs",
                SystemsNamespaceRequest::Rename {
                    source: b"a",
                    destination: b"b",
                    operation: SystemsFileRename::Replace,
                },
                150,
                &graph,
            )
            .unwrap()
            .unwrap();
        mutate(&mut call, &view);
        assert_eq!(load(&mut call, &view, 384), 1);
        assert_eq!(load(&mut call, &view, 384 + 8), victim);
        assert_eq!(load(&mut call, &view, 384 + 56), 0);
        assert_eq!(load(&mut call, &view, 1024 + 32), victim);
        call.complete_namespace_action(true, proof()).unwrap();
        drop(call);
        if retained {
            assert_eq!(
                pool.read_at("fs", &b, 64, 0, COPY).unwrap(),
                b"victim bytes"
            );
            pool.close("fs", &b).unwrap();
        }
        assert_eq!(&pool.state.metadata[384..512], &[0; 128]);
        assert_eq!(&pool.state.metadata[1056..1088], &[0; 32]);
        assert!(pool.state.pages[1].bytes.iter().all(|&byte| byte == 0));
        assert_eq!(
            pool.metadata_by_name("fs", b"b").unwrap().identity,
            pool.metadata("fs", &a).unwrap().identity
        );
        let fresh = create(&mut pool, b"c", b"fresh");
        assert!(pool.metadata("fs", &fresh).unwrap().identity > victim);
        assert_eq!(pool.state.namespace_actions.generation, 1);
    }
}

#[test]
fn initial_temporary_and_readonly_descriptor_links_use_actual_reference_authority() {
    let graph = binding();
    let mut pool = SystemsVolatileFilePool::new("fs").unwrap();
    let file = pool
        .create(
            "fs",
            SystemsFileRights::ReadWrite,
            0o600,
            SystemsFileUmask::default(),
            false,
            100,
        )
        .unwrap();
    let (mut call, view) = pool
        .borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::LinkByFile {
                source: &file,
                destination: b"first",
            },
            110,
            &graph,
        )
        .unwrap()
        .unwrap();
    assert_eq!(load(&mut call, &view, 3080), 4);
    mutate(&mut call, &view);
    call.complete_namespace_action(true, proof()).unwrap();
    drop(call);
    let reader = pool
        .open_named(
            "fs",
            b"first",
            SystemsNamedFileOpen {
                rights: SystemsFileRights::Read,
                create: false,
                exclusive: false,
                ..request()
            },
            111,
        )
        .unwrap();
    let rights = pool.state.descriptions.clone();
    let (mut call, view) = pool
        .borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::LinkByFile {
                source: &reader,
                destination: b"second",
            },
            112,
            &graph,
        )
        .unwrap()
        .unwrap();
    mutate(&mut call, &view);
    call.complete_namespace_action(true, proof()).unwrap();
    drop(call);
    assert_eq!(pool.state.descriptions, rights);
    pool.unlink("fs", b"first", 113).unwrap();
    pool.unlink("fs", b"second", 114).unwrap();
    let before = pool.state.clone();
    assert!(
        pool.borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::LinkByFile {
                source: &reader,
                destination: b"resurrect"
            },
            115,
            &graph
        )
        .is_err()
    );
    assert_eq!(pool.state, before);
    let excluded = pool
        .create_unlinkable(
            "fs",
            SystemsFileRights::ReadWrite,
            0o600,
            SystemsFileUmask::default(),
            false,
            120,
        )
        .unwrap();
    let before = pool.state.clone();
    assert!(
        pool.borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::LinkByFile {
                source: &excluded,
                destination: b"excluded"
            },
            121,
            &graph
        )
        .is_err()
    );
    assert_eq!(pool.state, before);
}

#[test]
fn noops_refusals_capacity_and_generation_exhaustion_preserve_complete_owner() {
    let graph = binding();
    let (mut pool, a, _) = fixture();
    pool.hard_link("fs", b"a", b"a2", 110).unwrap();
    let stale = pool.alias("fs", &a).unwrap();
    pool.close("fs", &stale).unwrap();
    let before = pool.state.clone();
    assert!(
        pool.borrow_namespace_action_after_admission(
            "wrong",
            SystemsNamespaceRequest::Link {
                source: b"a",
                destination: b"new"
            },
            120,
            &graph
        )
        .is_err()
    );
    assert!(
        pool.borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::LinkByFile {
                source: &stale,
                destination: b"new"
            },
            120,
            &graph
        )
        .is_err()
    );
    let long = [b'x'; 256];
    assert!(
        pool.borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::Link {
                source: b"a",
                destination: &long
            },
            120,
            &graph
        )
        .is_err()
    );
    assert_eq!(pool.state, before);
    for operation in [SystemsFileRename::Replace, SystemsFileRename::Exchange] {
        let before = pool.state.clone();
        assert!(
            pool.borrow_namespace_action_after_admission(
                "fs",
                SystemsNamespaceRequest::Rename {
                    source: b"a",
                    destination: b"a2",
                    operation
                },
                120,
                &graph
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(pool.state, before);
    }
    for request in [
        SystemsNamespaceRequest::Rename {
            source: b"a",
            destination: b"a2",
            operation: SystemsFileRename::NoReplace,
        },
        SystemsNamespaceRequest::Rename {
            source: b"missing",
            destination: b"new",
            operation: SystemsFileRename::Replace,
        },
        SystemsNamespaceRequest::Rename {
            source: b"a",
            destination: b"missing",
            operation: SystemsFileRename::Exchange,
        },
        SystemsNamespaceRequest::Link {
            source: b"a",
            destination: b"b",
        },
        SystemsNamespaceRequest::Link {
            source: b"a",
            destination: b"..",
        },
        SystemsNamespaceRequest::Link {
            source: b"a",
            destination: b"bad/name",
        },
    ] {
        let before = pool.state.clone();
        assert!(
            pool.borrow_namespace_action_after_admission("fs", request, 120, &graph)
                .is_err()
        );
        assert_eq!(pool.state, before);
    }
    pool.hard_link("fs", b"b", b"b2", 120).unwrap();
    let before = pool.state.clone();
    assert!(matches!(
        pool.borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::LinkByFile {
                source: &a,
                destination: b"full"
            },
            121,
            &graph
        ),
        Err(RegionLibraryError::Capacity)
    ));
    assert_eq!(pool.state, before);
    pool.unlink("fs", b"b2", 122).unwrap();
    pool.state.namespace_actions.generation = u64::MAX;
    let before = pool.state.clone();
    assert!(matches!(
        pool.borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::Link {
                source: b"a",
                destination: b"new"
            },
            123,
            &graph
        ),
        Err(RegionLibraryError::IdentityExhausted)
    ));
    assert_eq!(pool.state, before);
}

#[test]
fn admitted_failure_or_abandonment_never_rolls_back_or_resumes_owner() {
    let graph = binding();
    for fault in 0..13 {
        let (mut pool, a, _) = fixture();
        let payloads = pool.state.pages.clone();
        let (mut call, view) = pool
            .borrow_namespace_action_after_admission(
                "fs",
                SystemsNamespaceRequest::Link {
                    source: b"a",
                    destination: b"new",
                },
                130,
                &graph,
            )
            .unwrap()
            .unwrap();
        if fault != 12 {
            mutate(&mut call, &view);
        }
        let mut cleanup = proof();
        match fault {
            0 => cleanup.arena_erased = false,
            1 => cleanup.control_erased = false,
            2 => cleanup.retired_stack_erased = false,
            3 => cleanup.bindings_revoked = false,
            4 => cleanup.caller_restored = false,
            6 => store(&mut call, &view, 64 + 2 * 16, 999),
            7 => call.store_byte(&view, 1536 + 2 * 256, b'x' as u64).unwrap(),
            8 => store(&mut call, &view, 256 + 48, 999),
            9 => call.store_byte(&view, 900, 1).unwrap(),
            10 => call.store_byte(&view, 3072, 1).unwrap(),
            _ => (),
        }
        let mut actual = bytes(&mut call, &view);
        if fault != 11 {
            assert_eq!(
                call.complete_namespace_action(fault != 5, cleanup),
                Err(RegionLibraryError::Invariant)
            );
            assert!(call.load_byte(&view, 0).is_err());
        }
        drop(call);
        actual[3072..3584].fill(0);
        assert_eq!(pool.state.metadata, actual);
        assert_eq!(pool.state.pages, payloads);
        assert!(pool.state.namespace_actions.poisoned);
        assert!(pool.metadata("fs", &a).is_err());
        assert!(pool.unlink("fs", b"a", 140).is_err());
        assert!(
            pool.borrow_inode_mode_after_admission("fs", &a, 0o600, 140, &graph)
                .is_err()
        );
        pool.terminate("fs").unwrap();
        assert_eq!(pool.state.metadata, [0; PAGE]);
        assert!(
            pool.state
                .pages
                .iter()
                .all(|page| page.bytes.iter().all(|&byte| byte == 0))
        );
        assert!(pool.state.references.is_empty() && pool.state.descriptions.is_empty());
        assert_eq!(pool.state.namespace_actions, Default::default());
    }
}

#[test]
fn malformed_canonical_bytes_and_overlapping_domains_refuse_before_staging() {
    let graph = binding();
    let (mut pool, a, _) = fixture();
    let good = pool.state.clone();
    for offset in [
        0,
        8,
        128,
        900,
        1400,
        2700,
        3072,
        4095,
        256 + 7,
        256 + 31,
        256 + 56,
        256 + 64,
        256 + 72,
        512 + 16,
        1024 + 7 * 32 + 8,
        25,
        72,
        1537,
        1600,
    ] {
        pool.state = good.clone();
        if offset == 0 {
            pool.state.metadata[..8].fill(0);
        } else {
            pool.state.metadata[offset] ^= 2;
        }
        let before = pool.state.clone();
        assert!(
            pool.borrow_namespace_action_after_admission(
                "fs",
                SystemsNamespaceRequest::Link {
                    source: b"a",
                    destination: b"new"
                },
                130,
                &graph
            )
            .is_err(),
            "{offset}"
        );
        assert_eq!(pool.state, before);
    }
    for domain in 0..4 {
        for poisoned in [false, true] {
            pool.state = good.clone();
            let state = match domain {
                0 => &mut pool.state.position_actions,
                1 => &mut pool.state.page_actions,
                2 => &mut pool.state.inode_actions,
                _ => &mut pool.state.namespace_actions,
            };
            state.active = !poisoned;
            state.poisoned = poisoned;
            let before = pool.state.clone();
            assert!(
                pool.borrow_namespace_action_after_admission(
                    "fs",
                    SystemsNamespaceRequest::Link {
                        source: b"a",
                        destination: b"new"
                    },
                    130,
                    &graph
                )
                .is_err()
            );
            assert!(
                pool.borrow_page_action_after_admission(
                    "fs",
                    &a,
                    SystemsRamPageAction::Allocate { logical_page: 4 },
                    &graph
                )
                .is_err()
            );
            assert!(
                pool.borrow_position_after_admitted_transfer(
                    "fs",
                    &a,
                    SystemsPositionTransfer::Write,
                    12,
                    1,
                    &graph
                )
                .is_err()
            );
            assert!(
                pool.borrow_inode_mode_after_admission("fs", &a, 0o640, 130, &graph)
                    .is_err()
            );
            assert_eq!(pool.state, before);
        }
    }
    for fault in 0..4 {
        pool.state = good.clone();
        let description = pool.state.references[&a.ticket];
        match fault {
            0 => {
                pool.state.references.insert(a.ticket, u64::MAX);
            }
            1 => {
                pool.state
                    .descriptions
                    .get_mut(&description)
                    .unwrap()
                    .object = OBJECTS
            }
            2 => {
                pool.state
                    .descriptions
                    .get_mut(&description)
                    .unwrap()
                    .references += 1
            }
            _ => pool
                .state
                .descriptions
                .get_mut(&description)
                .unwrap()
                .set_position(u64::MAX),
        }
        let before = pool.state.clone();
        assert!(
            pool.borrow_namespace_action_after_admission(
                "fs",
                SystemsNamespaceRequest::LinkByFile {
                    source: &a,
                    destination: b"new"
                },
                130,
                &graph
            )
            .is_err()
        );
        assert_eq!(pool.state, before);
    }
}

#[test]
fn namespace_views_distinguish_pool_identity_other_domains_and_later_generations() {
    let graph = binding();
    let (mut pool, a, _) = fixture();
    let (mut inode, inode_view) = pool
        .borrow_inode_mode_after_admission("fs", &a, 0o640, 101, &graph)
        .unwrap();
    store(&mut inode, &inode_view, 0, 0o100640);
    store(&mut inode, &inode_view, 24, 101);
    inode.complete_inode_mode_action(true, proof()).unwrap();
    drop(inode);
    let (mut page, page_view) = pool
        .borrow_page_action_after_admission(
            "fs",
            &a,
            SystemsRamPageAction::Allocate { logical_page: 4 },
            &graph,
        )
        .unwrap();
    page.store_byte(&page_view, 2, 1).unwrap();
    page.complete_page_action(true, proof()).unwrap();
    drop(page);
    pool.seek("fs", &a, 0, SystemsFileSeek::Set).unwrap();
    let (mut position, position_view) = pool
        .borrow_position_after_admitted_transfer(
            "fs",
            &a,
            SystemsPositionTransfer::Read,
            0,
            1,
            &graph,
        )
        .unwrap();
    store(&mut position, &position_view, 0, 1);
    position.complete_position_action(true, proof()).unwrap();
    drop(position);
    let (mut other, foreign, _) = fixture();
    let before = other.state.clone();
    assert!(
        other
            .borrow_namespace_action_after_admission(
                "fs",
                SystemsNamespaceRequest::LinkByFile {
                    source: &a,
                    destination: b"new"
                },
                130,
                &graph
            )
            .is_err()
    );
    assert_eq!(other.state, before);
    let (mut outside, foreign_view) = other
        .borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::LinkByFile {
                source: &foreign,
                destination: b"new",
            },
            130,
            &graph,
        )
        .unwrap()
        .unwrap();
    let (mut call, view) = pool
        .borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::Link {
                source: b"a",
                destination: b"new",
            },
            130,
            &graph,
        )
        .unwrap()
        .unwrap();
    for stale in [&inode_view, &page_view, &position_view, &foreign_view] {
        assert_eq!(call.load_byte(stale, 0), Err(RegionLibraryError::View));
    }
    mutate(&mut call, &view);
    call.complete_namespace_action(true, proof()).unwrap();
    drop(call);
    mutate(&mut outside, &foreign_view);
    outside.complete_namespace_action(true, proof()).unwrap();
    drop(outside);
    pool.unlink("fs", b"new", 131).unwrap();
    let (mut next, current) = pool
        .borrow_namespace_action_after_admission(
            "fs",
            SystemsNamespaceRequest::Link {
                source: b"a",
                destination: b"later",
            },
            132,
            &graph,
        )
        .unwrap()
        .unwrap();
    assert_eq!(next.load_byte(&view, 0), Err(RegionLibraryError::View));
    mutate(&mut next, &current);
    next.complete_namespace_action(true, proof()).unwrap();
    drop(next);
    assert_eq!(pool.state.namespace_actions.generation, 2);
}
