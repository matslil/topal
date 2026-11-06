#[test]
fn page_source_selection_derives_instance_and_count_and_rejects_other_contexts() {
    // TOPAL-SYSTEMS-REGION-PAGE-OWNER-001.
    let root = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
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
    let source = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
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
