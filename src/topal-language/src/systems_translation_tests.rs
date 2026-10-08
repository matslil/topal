#[test]
fn requires_sealed_affine_translation_lifecycle() {
    // TOPAL-SYSTEMS-MAPPING-001, TOPAL-SYSTEMS-AUTHORITY-001,
    // TOPAL-SYSTEMS-QUALIFY-001.
    for (source, expected) in [
        (
            SOURCE.replace("template is bootstrap-equivalent", "template is empty"),
            "must be `bootstrap-equivalent`",
        ),
        (
            SOURCE.replace("page-policy is provider-selected", "page-policy is four-level"),
            "must be `provider-selected`",
        ),
        (
            SOURCE.replace(
                "memory translation commit update",
                "memory translation commit other",
            ),
            "affine resource",
        ),
        (
            SOURCE.replace(
                "memory translation activate space",
                "memory translation activate update",
            ),
            "affine resource",
        ),
        (
            SOURCE.replace(
                "translated console write \"TOPAL_KERNEL_TRANSLATION_ACTIVE\"",
                "memory console write \"TOPAL_KERNEL_TRANSLATION_ACTIVE\"",
            ),
            "not admitted",
        ),
        (
            SOURCE.replace(
                "failure fatal \"translation activation failed\"",
                "failure console write \"continued\"",
            ),
            "final operation is not a disposition",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            matches!(
                error.code.as_str(),
                "E-SYSTEMS-TRANSLATION" | "E-SYSTEMS-DISPOSITION" | "E-SYSTEMS-OPERATION"
            ),
            "{}: {}",
            error.code,
            error.message
        );
        assert!(error.message.contains(expected), "{}", error.message);
    }
}
