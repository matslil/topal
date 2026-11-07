use super::*;

const ADMITTED: SystemsFileUserAdmission = SystemsFileUserAdmission {
    numerical_range: true,
    complete_prefix: true,
};

fn request() -> SystemsNamedFileOpen {
    SystemsNamedFileOpen {
        rights: SystemsFileRights::ReadWrite,
        create: true,
        exclusive: false,
        truncate: false,
        append: false,
        mode: 0o660,
        umask: SystemsFileUmask::default(),
    }
}
fn pool() -> SystemsVolatileFilePool {
    SystemsVolatileFilePool::new("owner").unwrap()
}
fn create(p: &mut SystemsVolatileFilePool, name: &[u8], bytes: &[u8]) -> SystemsVolatileFile {
    let file = p.open_named("owner", name, request(), 100).unwrap();
    p.write("owner", &file, bytes, ADMITTED, 101).unwrap();
    file
}
fn anonymous(p: &mut SystemsVolatileFilePool) -> SystemsVolatileFile {
    p.create(
        "owner",
        SystemsFileRights::ReadWrite,
        0o600,
        SystemsFileUmask::default(),
        false,
        90,
    )
    .unwrap()
}
fn bytes(p: &mut SystemsVolatileFilePool, file: &SystemsVolatileFile) -> Vec<u8> {
    p.seek("owner", file, 0, SystemsFileSeek::Set).unwrap();
    p.read("owner", file, 256, ADMITTED).unwrap()
}
fn names(p: &SystemsVolatileFilePool) -> BTreeMap<Vec<u8>, u64> {
    p.directory_entries("owner", 0, 512, true)
        .unwrap()
        .0
        .into_iter()
        .filter(|entry| !entry.directory)
        .map(|entry| (entry.name, entry.identity))
        .collect()
}
fn failed<T: std::fmt::Debug>(result: Result<T, SystemsModelError>, errno: &str) {
    assert!(result.unwrap_err().to_string().contains(errno));
}

#[test]
fn hard_links_share_inode_bytes_but_keep_independent_description_positions_and_rights() {
    // TOPAL-SYSTEMS-NAMESPACE-OWNERSHIP-001.
    let mut p = pool();
    let a = create(&mut p, b"a", b"shared");
    let before = p.metadata("owner", &a).unwrap();
    p.hard_link("owner", b"a", b"b", 120).unwrap();
    let after = p.metadata("owner", &a).unwrap();
    assert_eq!(
        after,
        SystemsVolatileFileMetadata {
            links: 2,
            changed_seconds: 120,
            ..before.clone()
        }
    );
    let b = p
        .open_named(
            "owner",
            b"b",
            SystemsNamedFileOpen {
                create: false,
                rights: SystemsFileRights::Read,
                ..request()
            },
            121,
        )
        .unwrap();
    assert_eq!(p.metadata("owner", &b).unwrap().identity, before.identity);
    assert_eq!(p.metadata("owner", &b).unwrap().position, 0);
    assert_eq!(p.metadata("owner", &a).unwrap().position, 6);
    p.chmod_by_name("owner", b"b", 0o2751, 122).unwrap();
    assert_eq!(p.metadata_by_name("owner", b"a").unwrap().mode, 0o2751);
    p.write("owner", &a, b"!", ADMITTED, 123).unwrap();
    assert_eq!(bytes(&mut p, &b), b"shared!");
    let state = p.state.clone();
    failed(p.write("owner", &b, b"no", ADMITTED, 124), "EBADF");
    assert_eq!(p.state, state);
    assert_eq!(p.directory_times("owner").unwrap(), [120, 120]);
}

#[test]
fn each_name_owns_a_link_and_last_unlink_waits_for_all_descriptions() {
    let mut p = pool();
    let a = create(&mut p, b"a", b"old");
    let identity = p.metadata("owner", &a).unwrap().identity;
    p.hard_link("owner", b"a", b"b", 110).unwrap();
    p.close("owner", &a).unwrap();
    p.unlink("owner", b"a", 111).unwrap();
    assert_eq!(p.metadata_by_name("owner", b"b").unwrap().links, 1);
    let retained = p
        .open_named(
            "owner",
            b"b",
            SystemsNamedFileOpen {
                create: false,
                ..request()
            },
            112,
        )
        .unwrap();
    let inherited = p.alias("owner", &retained).unwrap();
    p.unlink("owner", b"b", 113).unwrap();
    assert_eq!(p.metadata("owner", &retained).unwrap().links, 0);
    let new = create(&mut p, b"b", b"new");
    assert!(p.metadata("owner", &new).unwrap().identity > identity);
    p.close("owner", &retained).unwrap();
    assert_eq!(bytes(&mut p, &inherited), b"old");
    p.close("owner", &inherited).unwrap();
    assert!(
        p.state
            .page_records()
            .all(|page| page.owner != Some(identity))
    );
    assert_eq!(bytes(&mut p, &new), b"new");
}

#[test]
fn absent_destination_rename_needs_no_free_entry_and_changes_only_name_and_times() {
    // TOPAL-SYSTEMS-NAMESPACE-TRANSACTION-001.
    let mut p = pool();
    let a = create(&mut p, b"a", b"contents");
    for name in [b"b", b"c", b"d"] {
        create(&mut p, name, b"");
    }
    let mut expected = p.state.clone();
    expected.update_name(0, |entry| entry.name = b"moved".to_vec());
    expected.update_object(0, |object| object.inode.changed(200));
    expected.update_directory_inode(|inode| inode.modified(200));
    p.rename("owner", b"a", b"moved", SystemsFileRename::Replace, 200)
        .unwrap();
    assert_eq!(p.state, expected);
    failed(p.metadata_by_name("owner", b"a"), "ENOENT");
    assert_eq!(p.metadata("owner", &a).unwrap().position, 8);
    assert_eq!(bytes(&mut p, &a), b"contents");
}

#[test]
fn replacement_keeps_victim_alive_through_other_links_and_open_descriptions() {
    let mut p = pool();
    let source = create(&mut p, b"source", b"source data");
    let victim = create(&mut p, b"target", b"victim data");
    let source_before = p.metadata("owner", &source).unwrap();
    let victim_before = p.metadata("owner", &victim).unwrap();
    p.hard_link("owner", b"target", b"victim-alias", 110)
        .unwrap();
    p.rename(
        "owner",
        b"source",
        b"target",
        SystemsFileRename::Replace,
        120,
    )
    .unwrap();
    assert_eq!(
        p.metadata_by_name("owner", b"target").unwrap().identity,
        source_before.identity
    );
    assert_eq!(
        p.metadata_by_name("owner", b"victim-alias")
            .unwrap()
            .identity,
        victim_before.identity
    );
    assert_eq!(
        p.metadata("owner", &source).unwrap(),
        SystemsVolatileFileMetadata {
            changed_seconds: 120,
            ..source_before
        }
    );
    assert_eq!(
        p.metadata("owner", &victim).unwrap(),
        SystemsVolatileFileMetadata {
            changed_seconds: 120,
            ..victim_before.clone()
        }
    );
    p.unlink("owner", b"victim-alias", 121).unwrap();
    assert_eq!(p.metadata("owner", &victim).unwrap().links, 0);
    assert_eq!(bytes(&mut p, &victim), b"victim data");
    p.close("owner", &victim).unwrap();
    assert!(
        p.state
            .page_records()
            .all(|page| page.owner != Some(victim_before.identity))
    );
    assert_eq!(bytes(&mut p, &source), b"source data");
}

#[test]
fn replacing_a_closed_last_link_reclaims_victim_pages_without_reusing_its_identity() {
    let mut p = pool();
    let source = create(&mut p, b"a", b"source");
    let victim = create(&mut p, b"b", b"victim");
    let old = p.metadata("owner", &victim).unwrap().identity;
    p.close("owner", &victim).unwrap();
    p.rename("owner", b"a", b"b", SystemsFileRename::Replace, 130)
        .unwrap();
    assert!(p.state.page_records().all(|page| page.owner != Some(old)));
    assert_eq!(p.state.objects().iter().flatten().count(), 1);
    let fresh = create(&mut p, b"c", b"");
    assert!(p.metadata("owner", &fresh).unwrap().identity > old);
    assert_eq!(bytes(&mut p, &source), b"source");
}

#[test]
fn exchange_changes_associations_and_ctimes_without_moving_data_positions_or_link_counts() {
    let mut p = pool();
    let a = create(&mut p, b"a", b"alpha");
    let b = create(&mut p, b"b", b"beta");
    p.hard_link("owner", b"a", b"a2", 110).unwrap();
    let old_a = p.metadata("owner", &a).unwrap();
    let old_b = p.metadata("owner", &b).unwrap();
    let pages = p.state.page_snapshot();
    let descriptions = p.state.descriptions.clone();
    p.rename("owner", b"a", b"b", SystemsFileRename::Exchange, 140)
        .unwrap();
    assert_eq!(
        names(&p),
        BTreeMap::from([
            (b"a".to_vec(), old_b.identity),
            (b"b".to_vec(), old_a.identity),
            (b"a2".to_vec(), old_a.identity),
        ])
    );
    assert_eq!(
        p.metadata("owner", &a).unwrap(),
        SystemsVolatileFileMetadata {
            changed_seconds: 140,
            ..old_a
        }
    );
    assert_eq!(
        p.metadata("owner", &b).unwrap(),
        SystemsVolatileFileMetadata {
            changed_seconds: 140,
            ..old_b
        }
    );
    assert_eq!(p.state.page_snapshot(), pages);
    assert_eq!(p.state.descriptions, descriptions);
    assert_eq!(p.directory_times("owner").unwrap(), [140, 140]);
}

#[test]
fn same_inode_rename_and_exchange_are_complete_noops_but_noreplace_is_eexist() {
    let mut p = pool();
    create(&mut p, b"a", b"data");
    p.hard_link("owner", b"a", b"b", 110).unwrap();
    let before = p.state.clone();
    for destination in [b"a", b"b"] {
        for operation in [SystemsFileRename::Replace, SystemsFileRename::Exchange] {
            p.rename("owner", b"a", destination, operation, u64::MAX)
                .unwrap();
            assert_eq!(p.state, before);
        }
        failed(
            p.rename(
                "owner",
                b"a",
                destination,
                SystemsFileRename::NoReplace,
                u64::MAX,
            ),
            "EEXIST",
        );
        assert_eq!(p.state, before);
    }
}

#[test]
fn entry_and_inode_capacity_are_independent_and_refusals_preserve_all_state() {
    let mut p = pool();
    let first = create(&mut p, b"a", b"data");
    for name in [b"b", b"c", b"d"] {
        p.hard_link("owner", b"a", name, 110).unwrap();
    }
    assert_eq!(p.metadata("owner", &first).unwrap().links, 4);
    assert_eq!(p.state.objects().iter().flatten().count(), 1);
    let before = p.state.clone();
    failed(p.hard_link("owner", b"a", b"fifth", 120), "ENOSPC");
    assert_eq!(p.state, before);
    failed(p.open_named("owner", b"fifth", request(), 120), "ENOSPC");
    assert_eq!(p.state, before);
    let temporary = anonymous(&mut p);
    assert_eq!(p.metadata("owner", &temporary).unwrap().links, 0);
    let mut q = pool();
    let files: Vec<_> = (0..4).map(|_| anonymous(&mut q)).collect();
    assert!(q.state.names().iter().all(Option::is_none));
    q.hard_link_by_file("owner", &files[2], b"first-name", 130)
        .unwrap();
    let before = q.state.clone();
    failed(
        q.open_named("owner", b"new-inode", request(), 140),
        "ENOSPC",
    );
    assert_eq!(q.state, before);
    q.rename(
        "owner",
        b"first-name",
        b"renamed",
        SystemsFileRename::NoReplace,
        150,
    )
    .unwrap();
    assert_eq!(
        q.metadata_by_name("owner", b"renamed").unwrap().identity,
        q.metadata("owner", &files[2]).unwrap().identity
    );
}

#[test]
fn initial_tmpfile_linkability_is_consumed_and_unlinked_inodes_cannot_resurrect() {
    let mut p = pool();
    let temporary = anonymous(&mut p);
    p.write("owner", &temporary, b"anonymous", ADMITTED, 100)
        .unwrap();
    p.hard_link_by_file("owner", &temporary, b"published", 110)
        .unwrap();
    p.hard_link_by_file("owner", &temporary, b"alias", 111)
        .unwrap();
    assert_eq!(p.metadata("owner", &temporary).unwrap().links, 2);
    p.unlink("owner", b"published", 112).unwrap();
    p.unlink("owner", b"alias", 113).unwrap();
    let before = p.state.clone();
    failed(
        p.hard_link_by_file("owner", &temporary, b"resurrect", 114),
        "ENOENT",
    );
    assert_eq!(p.state, before);
    assert_eq!(bytes(&mut p, &temporary), b"anonymous");
    let exclusive = p
        .create_unlinkable(
            "owner",
            SystemsFileRights::ReadWrite,
            0o600,
            SystemsFileUmask::default(),
            false,
            120,
        )
        .unwrap();
    let before = p.state.clone();
    failed(
        p.hard_link_by_file("owner", &exclusive, b"forbidden", 121),
        "ENOENT",
    );
    assert_eq!(p.state, before);
}

#[test]
fn wrong_owner_foreign_stale_names_and_failed_transactions_preserve_complete_store() {
    let mut p = pool();
    let a = create(&mut p, b"a", b"data");
    let mut foreign = pool();
    let foreign_a = create(&mut foreign, b"a", b"foreign");
    let stale = p.alias("owner", &a).unwrap();
    p.close("owner", &stale).unwrap();
    let before = p.state.clone();
    assert!(p.hard_link_by_file("owner", &foreign_a, b"x", 999).is_err());
    assert!(p.hard_link_by_file("owner", &stale, b"x", 999).is_err());
    assert!(p.hard_link("other", b"a", b"x", 999).is_err());
    assert!(
        p.rename("other", b"a", b"x", SystemsFileRename::Replace, 999)
            .is_err()
    );
    assert_eq!(p.state, before);
    for bad in [b"".as_slice(), b".", b"..", b"a/b", b"a\0b", &[b'x'; 256]] {
        assert!(p.hard_link("owner", b"a", bad, 999).is_err());
        assert!(
            p.rename("owner", b"a", bad, SystemsFileRename::Replace, 999)
                .is_err()
        );
        assert_eq!(p.state, before);
    }
    failed(p.hard_link("owner", b"absent", b"x", 999), "ENOENT");
    failed(p.hard_link("owner", b"a", b"a", 999), "EEXIST");
    failed(
        p.rename("owner", b"a", b"absent", SystemsFileRename::Exchange, 999),
        "ENOENT",
    );
    assert_eq!(p.state, before);
    for domain in 0..3 {
        let mut poisoned = before.clone();
        match domain {
            0 => poisoned.position_actions.poisoned = true,
            1 => poisoned.page_actions.poisoned = true,
            _ => poisoned.inode_actions.poisoned = true,
        }
        p.state = poisoned.clone();
        assert!(p.hard_link("owner", b"a", b"x", 999).is_err());
        assert!(
            p.rename("owner", b"a", b"x", SystemsFileRename::Replace, 999)
                .is_err()
        );
        assert_eq!(p.state, poisoned);
    }
}

#[test]
fn entry_cookies_do_not_depend_on_object_slots_and_long_raw_names_remain_exact() {
    let mut p = pool();
    let _anonymous = anonymous(&mut p);
    let a = create(&mut p, b"a", b"bytes");
    let identity = p.metadata("owner", &a).unwrap().identity;
    let long = vec![0xff; 255];
    p.hard_link("owner", b"a", &long, 110).unwrap();
    let entries = p.directory_entries("owner", 0, 512, true).unwrap().0;
    assert_eq!((entries[2].cookie, entries[3].cookie), (3, 4));
    assert_eq!(
        (entries[2].identity, entries[3].identity),
        (identity, identity)
    );
    assert_eq!(entries[3].name, long);
    assert_eq!(entries[3].native_x86_64_bytes, 280);
    let renamed = vec![0x80; 255];
    p.rename("owner", &long, &renamed, SystemsFileRename::NoReplace, 120)
        .unwrap();
    assert_eq!(
        p.metadata_by_name("owner", &renamed).unwrap().identity,
        identity
    );
    let before = p.state.clone();
    failed(p.directory_entries("owner", 3, 512, false), "EFAULT");
    assert_eq!(p.state, before);
}

#[test]
fn malformed_tables_refuse_normal_access_but_terminal_cleanup_retires_all_state() {
    let mut p = pool();
    let a = create(&mut p, b"a", b"data");
    p.hard_link("owner", b"a", b"b", 110).unwrap();
    let good = p.state.clone();
    for fault in 0..5 {
        p.state = good.clone();
        match fault {
            0 => p.state.update_name(0, |entry| entry.identity = u64::MAX),
            1 => p.state.update_name(1, |entry| entry.name = b"a".to_vec()),
            2 => p.state.update_name(0, |entry| entry.name = b"..".to_vec()),
            3 => p
                .state
                .update_object(0, |object| object.may_link_unlinked = true),
            _ => p.state.set_object(1, p.state.object(0)),
        }
        let before = p.state.clone();
        assert!(p.metadata("owner", &a).is_err());
        assert!(p.hard_link("owner", b"a", b"x", 999).is_err());
        assert_eq!(p.state, before);
        p.terminate("owner").unwrap();
        assert!(p.state.names().iter().all(Option::is_none));
        assert!(p.state.objects().iter().all(Option::is_none));
        assert!(
            p.state
                .page_records()
                .all(|page| page.owner.is_none() && page.bytes.iter().all(|byte| *byte == 0))
        );
        assert!(p.state.references.is_empty() && p.state.descriptions.is_empty());
        assert_eq!(p.state.directory_inode().encoded(), &[0; 32]);
    }
}

#[test]
fn native_flag_adaptation_keeps_high_bits_separate_and_does_not_accept_whiteout() {
    // TOPAL-SYSTEMS-NAMESPACE-ABI-001; no complete syscall error-order claim.
    assert_eq!(
        SystemsFileRename::from_native(1 << 32).unwrap(),
        SystemsFileRename::Replace
    );
    assert_eq!(
        SystemsFileRename::from_native((1 << 32) | 1).unwrap(),
        SystemsFileRename::NoReplace
    );
    assert_eq!(
        SystemsFileRename::from_native((1 << 32) | 2).unwrap(),
        SystemsFileRename::Exchange
    );
    for flags in [3, 6, 7, 8, u64::MAX] {
        failed(SystemsFileRename::from_native(flags), "EINVAL");
    }
    for flags in [4, 5] {
        failed(
            SystemsFileRename::from_native(flags),
            "unqualified whiteout",
        );
    }
}

#[test]
fn readonly_descriptor_can_link_without_acquiring_write_rights_or_sharing_position() {
    let mut p = pool();
    let writable = create(&mut p, b"original", b"bytes");
    let readonly = p
        .open_named(
            "owner",
            b"original",
            SystemsNamedFileOpen {
                rights: SystemsFileRights::Read,
                create: false,
                ..request()
            },
            110,
        )
        .unwrap();
    let descriptions = p.state.descriptions.clone();
    let before = p.metadata("owner", &writable).unwrap();
    p.hard_link_by_file("owner", &readonly, b"second", 120)
        .unwrap();
    assert_eq!(p.state.descriptions, descriptions);
    assert_eq!(
        p.metadata("owner", &writable).unwrap(),
        SystemsVolatileFileMetadata {
            links: 2,
            changed_seconds: 120,
            ..before
        }
    );
    assert_eq!(p.read("owner", &readonly, 5, ADMITTED).unwrap(), b"bytes");
    assert_eq!(p.metadata("owner", &writable).unwrap().position, 5);
    let before = p.state.clone();
    failed(p.write("owner", &readonly, b"no", ADMITTED, 130), "EBADF");
    assert_eq!(p.state, before);
}
