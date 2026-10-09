#[test]
fn requires_affine_local_interrupt_critical_scope() {
    // TOPAL-SYSTEMS-CRITICAL-001, TOPAL-SYSTEMS-AUTHORITY-001.
    for (source, expected) in [
        (
            SOURCE.replace(
                "domain is local-maskable-interrupts",
                "domain is all-interrupts",
            ),
            "`local-maskable-interrupts` domain",
        ),
        (
            SOURCE.replace(
                "restored is critical restore",
                "restored is unmapped restore",
            ),
            "live affine critical context",
        ),
        (
            SOURCE.replace(
                "restored is critical restore",
                "restored console write \"escaped\"",
            ),
            "bind the context returned by restoration",
        ),
        (
            SOURCE.replace(
                "critical console write \"TOPAL_KERNEL_INTERRUPTS_MASKED\"",
                "critical debug break",
            ),
            "nonblocking console marker",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E-SYSTEMS-CRITICAL");
        assert!(error.message.contains(expected), "{}", error.message);
    }
}
