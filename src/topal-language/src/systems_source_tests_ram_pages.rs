fn legacy_filesystem_test_root() -> String {
    // Absence and component controls retain V5. V6 dependencies are tested
    // separately, including the real current root, below.
    let mut source = include_str!("../../../linux-kernel/labs/qemu/x86_64/pre-frame-kernel-root.t")
        .replace("static-linux-dispatch-v6", "static-linux-dispatch-v5");
    if let Some(start) = source.find("  namespace-library is") {
        let end = start + source[start..].find("\n  ),\n").unwrap() + 6;
        source.replace_range(start..end, "");
    }
    source
}

#[test]
fn page_source_selection_derives_instance_and_count_and_rejects_other_contexts() {
    // TOPAL-SYSTEMS-REGION-PAGE-OWNER-001.
    let legacy = legacy_filesystem_test_root();
    let root = legacy.as_str();
    let declaration = "  ram-page-pool-library is lang systems source-library (\n    source is \"memory/ram-page-pool.t\",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
    let selected = if root.contains("  ram-page-pool-library is") {
        root.to_owned()
    } else {
        root.replace(
            "  region-library is",
            &(declaration.to_owned() + "  region-library is"),
        )
    };
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let valid = analyze_systems_for_compiler(&selected, &target).unwrap();
    assert_eq!(
        valid.ram_page_pool_library.as_ref().unwrap().interface,
        CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
    );
    let absent = selected.replace(declaration, "");
    assert!(
        analyze_systems_for_compiler(&absent, &target)
            .unwrap()
            .ram_page_pool_library
            .is_none()
    );
    let mut invalid = vec![
        declaration.repeat(2),
        declaration.replace("4096", "2048"),
        declaration.replace("65536", "32768"),
        declaration.replace("memory/ram-page-pool.t", "../outside.t"),
        declaration.replace(
            "    export is",
            "    instance is file-position,\n    export is",
        ),
        declaration.replace("    export is", "    scalar-count is 2,\n    export is"),
    ];
    for context in [
        "borrowed-byte-region-v1",
        "borrowed-byte-region-action-v1",
        "borrowed-byte-region-framed-action-v1",
        "relative-sleep-deadline-v1",
    ] {
        invalid.push(declaration.replace("borrowed-byte-region-instance-action-v1", context));
    }
    for changed in invalid {
        assert!(
            analyze_systems_for_compiler(&selected.replace(declaration, &changed), &target)
                .is_err(),
            "{changed}"
        );
    }
    let mut invalid_model = valid;
    invalid_model
        .ram_page_pool_library
        .as_mut()
        .unwrap()
        .interface = CompilerSystemsSourceLibraryInterface::BorrowedByteRegionFramedActionV1;
    assert!(topal_semantics::validate_systems_program(&invalid_model).is_err());
}

#[test]
fn inode_source_is_unique_qualified_and_required_by_v5() {
    // TOPAL-SYSTEMS-INODE-MODE-ACTION-001; TOPAL-SYSTEMS-LINUX-DISPATCH-V5-001.
    let legacy = legacy_filesystem_test_root();
    let source = legacy.as_str();
    let start = source.find("  inode-mode-library is").unwrap();
    let end = start + source[start..].find("\n  ),\n").unwrap() + 6;
    let declaration = &source[start..end];
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let program = analyze_systems_for_compiler(source, &target).unwrap();
    assert_eq!(
        program.inode_mode_library.as_ref().unwrap().interface,
        CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
    );
    for changed in [
        String::new(),
        declaration.repeat(2),
        declaration.replace("4096", "4095"),
        declaration.replace("65536", "65535"),
        declaration.replace("policy/inode-mode-action.t", "../outside.t"),
        declaration.replace(
            "    export is",
            "    instance is file-position,\n    export is",
        ),
        declaration.replace("    export is", "    scalar-count is 2,\n    export is"),
        declaration.replace(
            "borrowed-byte-region-instance-action-v1",
            "borrowed-byte-region-framed-action-v1",
        ),
    ] {
        assert!(
            analyze_systems_for_compiler(&source.replace(declaration, &changed), &target).is_err(),
            "{changed}"
        );
    }
    let mut missing = program;
    missing.inode_mode_library = None;
    assert!(topal_semantics::validate_systems_program(&missing).is_err());
    let frozen = source.replace("static-linux-dispatch-v5", "static-linux-dispatch-v4");
    assert!(
        analyze_systems_for_compiler(&frozen.replace(declaration, ""), &target)
            .unwrap()
            .inode_mode_library
            .is_none()
    );
}

#[test]
fn namespace_source_requires_a_unique_instance_and_all_filesystem_controllers() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001. This selects the component controller;
    // V6 handlers/entry consumers remain a separate integration obligation.
    let legacy = legacy_filesystem_test_root();
    let source = legacy.as_str();
    let declaration = "  namespace-library is lang systems source-library (\n    source is \"policy/namespace-action.t\",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
    let selected = source.replace(
        "  inode-mode-library is",
        &format!("{declaration}  inode-mode-library is"),
    );
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let program = analyze_systems_for_compiler(&selected, &target).unwrap();
    assert_eq!(
        program.namespace_library.as_ref().unwrap().interface,
        CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
    );
    for changed in [
        declaration.repeat(2),
        declaration.replace("65536", "65535"),
        declaration.replace("4096", "4095"),
        declaration.replace("policy/namespace-action.t", "../outside.t"),
        declaration.replace(
            "    export is",
            "    instance is inode-metadata,\n    export is",
        ),
        declaration.replace(
            "borrowed-byte-region-instance-action-v1",
            "borrowed-byte-region-framed-action-v1",
        ),
    ] {
        assert!(
            analyze_systems_for_compiler(&selected.replace(declaration, &changed), &target)
                .is_err()
        );
    }
    for missing in 0..4 {
        let mut wrong = program.clone();
        match missing {
            0 => wrong.inode_mode_library = None,
            1 => wrong.ram_page_pool_library = None,
            2 => wrong.file_position_library = None,
            _ => {
                wrong.file_position_library.as_mut().unwrap().interface =
                    CompilerSystemsSourceLibraryInterface::BorrowedByteRegionFramedActionV1
            }
        }
        assert!(topal_semantics::validate_systems_program(&wrong).is_err());
    }
    assert!(
        analyze_systems_for_compiler(source, &target)
            .unwrap()
            .namespace_library
            .is_none()
    );
}

#[test]
fn v6_dispatch_requires_namespace_and_named_open_policy_without_changing_older_roots() {
    // TOPAL-SYSTEMS-NAMESPACE-ABI-001; declaration validation is distinct from
    // the backend's later complete entry-table/handler qualification.
    let legacy = legacy_filesystem_test_root();
    let frozen = legacy.as_str();
    let selected = frozen.replace("static-linux-dispatch-v5", "static-linux-dispatch-v6");
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let current = analyze_systems_for_compiler(
        include_str!("../../../linux-kernel/labs/qemu/x86_64/pre-frame-kernel-root.t"),
        &target,
    )
    .unwrap();
    assert_eq!(
        current.dispatch_library.unwrap().interface,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6
    );
    assert!(current.namespace_library.is_some());
    assert_eq!(
        analyze_systems_for_compiler(&selected, &target)
            .unwrap_err()
            .code,
        "E-SYSTEMS-NAMESPACE"
    );
    let declaration = "  namespace-library is lang systems source-library (\n    source is \"policy/namespace-action.t\",\n    export is apply,\n    interface is borrowed-byte-region-instance-action-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
    let source = selected.replace(
        "  inode-mode-library is",
        &format!("{declaration}  inode-mode-library is"),
    );
    let program = analyze_systems_for_compiler(&source, &target).unwrap();
    assert_eq!(
        program.dispatch_library.as_ref().unwrap().interface,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6
    );
    assert!(
        analyze_systems_for_compiler(
            &source.replace(declaration, &declaration.repeat(2)),
            &target
        )
        .is_err()
    );
    let kernel_start = source.find("  kernel-library is").unwrap();
    let kernel_end =
        source[kernel_start..].find("\n  ),\n").unwrap() + kernel_start + "\n  ),\n".len();
    let mut without_kernel = source.clone();
    without_kernel.replace_range(kernel_start..kernel_end, "");
    let boot_only = source.replace(
        "interface is file-open-decision-v1",
        "interface is boot-open-decision-v1",
    );
    for wrong in [&without_kernel, &boot_only] {
        let error = analyze_systems_for_compiler(wrong, &target).unwrap_err();
        assert_eq!(error.code, "E-SYSTEMS-NAMESPACE");
        assert!(error.message.contains("file-open-decision-v1"));
    }
    for missing in 0..6 {
        let mut wrong = program.clone();
        match missing {
            0 => wrong.namespace_library = None,
            1 => wrong.inode_mode_library = None,
            2 => wrong.ram_page_pool_library = None,
            3 => wrong.file_position_library = None,
            4 => wrong.kernel_library = None,
            _ => {
                wrong.kernel_library.as_mut().unwrap().interface =
                    CompilerSystemsSourceLibraryInterface::BootOpenDecisionV1
            }
        }
        let error = topal_semantics::validate_systems_program(&wrong).unwrap_err();
        assert_eq!(error.code, "E-SYSTEMS-NAMESPACE");
    }
    let old = analyze_systems_for_compiler(frozen, &target).unwrap();
    assert_eq!(
        old.dispatch_library.unwrap().interface,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV5
    );
    assert!(old.namespace_library.is_none());
    // A standalone namespace-controller component retains the older root's
    // open-policy selection; only V6's mutable table requires the named plan.
    for current in [&without_kernel, &boot_only] {
        let frozen_dispatch =
            current.replace("static-linux-dispatch-v6", "static-linux-dispatch-v5");
        let old_component = analyze_systems_for_compiler(&frozen_dispatch, &target).unwrap();
        topal_semantics::validate_systems_program(&old_component).unwrap();
    }
}
