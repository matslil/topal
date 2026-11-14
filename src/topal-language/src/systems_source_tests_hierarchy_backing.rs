// TOPAL-SYSTEMS-STATIC-ELF-PROVIDER-BACKING-001.
fn provider_backing_source() -> String {
    let start = SOURCE.find("frame-count is 5,").unwrap();
    let last = "alignment-frames is 1";
    let end = start + SOURCE[start..].find(last).unwrap() + last.len();
    SOURCE.replacen(&SOURCE[start..end], "backing is provider-selected", 1)
}

#[test]
fn provider_selected_hierarchy_preserves_purpose_and_complete_lifecycle() {
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let legacy = analyze_systems_for_compiler(SOURCE, &target).unwrap();
    let selected = analyze_systems_for_compiler(&provider_backing_source(), &target).unwrap();
    let mut expected = legacy.bootstrap.handler.operations.clone();
    let position = expected.iter().position(|op| matches!(op,
        CompilerSystemsOperation::AllocatePhysicalFrames { request, .. }
        if request.frame_count == 5)).unwrap();
    expected[position] = CompilerSystemsOperation::ReserveStaticImageHierarchy {
        failure_message: "static ELF hierarchy frame reservation failed".into(),
    };
    assert_eq!(selected.bootstrap.handler.operations, expected);
    assert_eq!(legacy.bootstrap.handler.disposition, selected.bootstrap.handler.disposition);
    let transitions = model_systems_transitions(&selected).unwrap();
    assert_eq!(transitions.iter().filter(|t| matches!(t,
        CompilerSystemsTransition::ReserveStaticImageHierarchy)).count(), 1);
    assert!(!transitions.iter().any(|t| matches!(t,
        CompilerSystemsTransition::AllocatePhysicalFrames { request }
        if request.frame_count == 5)));
    for identity in [
        "topal.systems.static-image.construction.reserve-hierarchy/1",
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE,
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY,
        SYSTEMS_FRAMES_RELEASE,
    ] {
        assert!(transitions.iter().any(|t| t.semantic_identity() == identity), "{identity}");
    }
}

#[test]
fn provider_selected_hierarchy_rejects_mixed_duplicate_and_native_parameters() {
    let source = provider_backing_source();
    for bad in [
        "backing is provider-selected, frame-count is 5",
        "backing is provider-selected, alignment-frames is 1",
        "backing is provider-selected, frame-count is 8, alignment-frames is 1",
        "backing is provider-selected, backing is provider-selected",
        "backing is provider-selected, owner is sealed-static-elf-construction",
        "backing is bootstrap-equivalent",
        "backing is 8",
        "backing is provider-selected, physical-base is 16777216",
        "alignment-frames is 1",
    ] {
        let changed = source.replacen("backing is provider-selected", bad, 1);
        let error = analyze_systems_for_compiler(
            &changed, &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        ).unwrap_err();
        assert_eq!(error.code, "E-SYSTEMS-TRANSLATION-EDIT", "{bad}: {error:?}");
    }
    for count in [0, 4, 6, 8] {
        assert!(analyze_systems_for_compiler(
            &SOURCE.replacen("frame-count is 5,", &format!("frame-count is {count},"), 1),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        ).is_err(), "legacy count {count}");
    }
}

#[test]
fn provider_selected_hierarchy_rejects_wrong_owner_and_unsupported_template() {
    let source = provider_backing_source();
    for (old, new) in [
        ("owner is sealed-static-elf-construction", "owner is populated-static-elf-construction"),
        ("template is bootstrap-equivalent", "template is kernel-image-protected"),
        ("hierarchy is static-elf-hierarchy-frames", "hierarchy is other-hierarchy"),
        ("unmapped frames release returned-static-elf-hierarchy", "unmapped frames release static-elf-hierarchy-frames"),
    ] {
        assert!(source.contains(old), "{old}");
        assert!(analyze_systems_for_compiler(
            &source.replacen(old, new, 1),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        ).is_err(), "{old} -> {new}");
    }
}
