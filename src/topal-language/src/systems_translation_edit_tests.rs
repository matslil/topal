#[test]
fn requires_sealed_affine_active_translation_edits() {
    // TOPAL-SYSTEMS-MAPPING-001, TOPAL-SYSTEMS-AUTHORITY-001,
    // TOPAL-SYSTEMS-QUALIFY-001.
    for (source, code, expected) in [
        (
            SOURCE.replace(
                "placement is provider-selected",
                "placement is source-selected",
            ),
            "E-SYSTEMS-TRANSLATION-EDIT",
            "must be `provider-selected`",
        ),
        (
            SOURCE.replace(
                "map-edit translation commit",
                "other-edit translation commit",
            ),
            "E-SYSTEMS-TRANSLATION-EDIT",
            "current affine edit",
        ),
        (
            SOURCE.replace(
                "removed-frames is unmap-edit kernel unmap dynamic-mapping",
                "removed-frames is unmap-edit kernel unmap other-mapping",
            ),
            "E-SYSTEMS-TRANSLATION-EDIT",
            "live mapping",
        ),
        (
            SOURCE.replace(
                "unmap-edit translation commit",
                "map-edit translation commit",
            ),
            "E-SYSTEMS-TRANSLATION-EDIT",
            "current affine edit",
        ),
        (
            SOURCE.replace(
                "unmapped frames release removed-frames",
                "edited frames release removed-frames",
            ),
            "E-SYSTEMS-FRAMES",
            "live allocator and affine extent",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, code);
        assert!(error.message.contains(expected), "{}", error.message);
    }
}
