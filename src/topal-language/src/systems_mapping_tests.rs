#[test]
fn requires_opaque_affine_kernel_mapping_policy_and_bounds() {
    // TOPAL-SYSTEMS-MAPPING-001, TOPAL-SYSTEMS-AUTHORITY-001,
    // TOPAL-SYSTEMS-QUALIFY-001.
    for (source, expected) in [
        (
            SOURCE.replace("rights is read-write", "rights is read-only"),
            "must be `read-write`",
        ),
        (
            SOURCE.replace("execution is denied", "execution is allowed"),
            "must be `denied`",
        ),
        (
            SOURCE.replace("memory-kind is normal", "memory-kind is device"),
            "must be `normal`",
        ),
        (
            SOURCE.replace(
                "mapping byte store (offset-bytes is 0",
                "mapping byte store (offset-bytes is 4096",
            ),
            "outside the one-frame mapping",
        ),
        (
            SOURCE.replace(
                "frames is memory kernel unmap mapping",
                "frames is memory kernel unmap other",
            ),
            "affine mapping",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E-SYSTEMS-MAPPING");
        assert!(error.message.contains(expected), "{}", error.message);
    }
}
