#[test]
fn requires_typed_affine_kernel_context_round_trip() {
    // TOPAL-SYSTEMS-CONTEXT-001.
    let program = analyze_systems_for_compiler(
        SOURCE,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    assert_eq!(
        program.kernel_thread.kind,
        CompilerSystemsEntryKind::ResumedKernelThread
    );
    assert_eq!(
        program.kernel_thread.handler.effects,
        [SYSTEMS_CONSOLE_WRITE, SYSTEMS_KERNEL_CONTEXT_RETIRE]
    );

    for (source, expected) in [
        (
            SOURCE.replace(
                "  kernel-thread is lang systems resumed-thread-entry kernel-thread-handler\n",
                "",
            ),
            "kernel-thread",
        ),
        (
            SOURCE.replace(
                "context : KernelThreadContext InitialProcessor",
                "context : KernelThreadContext OtherProcessor",
            ),
            "KernelThreadContext InitialProcessor",
        ),
        (
            SOURCE.replace(
                "context console write \"TOPAL_KERNEL_CONTEXT_ENTERED\"",
                "context console write \"TOPAL_KERNEL_CONTEXT_EARLY\"",
            ),
            "exact context-entry marker",
        ),
        (
            SOURCE.replace(
                "context kernel context retire to caller",
                "context kernel context retire to other",
            ),
            "matching suspended caller",
        ),
        (
            SOURCE.replace("byte-count is 16384", "byte-count is 8192"),
            "16 KiB",
        ),
        (
            SOURCE.replace("entry is kernel-thread", "entry is other-thread"),
            "typed `kernel-thread` entry",
        ),
        (
            SOURCE.replace(
                "deadline-resumed kernel context transfer worker",
                "deadline-resumed kernel context transfer other",
            ),
            "matching suspended context",
        ),
        (
            SOURCE.replace(
                "deadline-resumed console write \"TOPAL_KERNEL_CONTEXT_RESUMED\"",
                "deadline-resumed console write \"TOPAL_KERNEL_CONTEXT_EARLY\"",
            ),
            "exact resumed-caller marker",
        ),
        (
            SOURCE.replace(
                "context-resumed is completed kernel context reclaim",
                "context-resumed is other kernel context reclaim",
            ),
            "matching completed transfer",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            matches!(error.code.as_str(), "E-SYSTEMS-CONTEXT-TRANSFER" | "E-SYSTEMS-ENTRY-SET"),
            "{}: {}",
            error.code,
            error.message
        );
        assert!(error.message.contains(expected), "{}", error.message);
    }
}
