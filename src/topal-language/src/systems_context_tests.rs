#[test]
fn requires_typed_affine_kernel_context_round_trip() {
    // TOPAL-SYSTEMS-CONTEXT-001, TOPAL-SYSTEMS-CONTEXT-002,
    // TOPAL-SYSTEMS-CONTEXT-003, TOPAL-SYSTEMS-CONTEXT-004,
    // TOPAL-SYSTEMS-CONTEXT-005, TOPAL-SYSTEMS-CONTEXT-006.
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
        [
            SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION,
            SYSTEMS_CONSOLE_WRITE,
            SYSTEMS_KERNEL_CONTEXT_RETIRE,
        ]
    );
    assert_eq!(
        program.completion_thread.handler.effects,
        [
            SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
            SYSTEMS_CONSOLE_WRITE,
            SYSTEMS_KERNEL_CONTEXT_RETIRE,
        ]
    );
    assert_eq!(
        program.terminal_thread.handler.effects,
        [SYSTEMS_CONSOLE_WRITE, SYSTEMS_KERNEL_CONTEXT_RETIRE]
    );
    assert_eq!(
        program.user_syscall.kind,
        CompilerSystemsEntryKind::UserSyscall
    );
    assert_eq!(
        program.user_syscall.handler.effects,
        [SYSTEMS_USER_SYSCALL_DISPATCH]
    );

    for (source, expected) in [
        (
            SOURCE.replace(
                "  kernel-thread is lang systems resumed-thread-entry kernel-thread-handler,\n",
                "",
            ),
            "kernel-thread",
        ),
        (
            SOURCE.replace(
                "  terminal-thread is lang systems resumed-thread-entry terminal-thread-handler,\n",
                "",
            ),
            "terminal-thread",
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
                "context console write \"TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED\"",
                "context console write \"TOPAL_KERNEL_CONTEXT_EARLY\"",
            ),
            "exact entry marker",
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
                "resumed kernel context transfer selected-cooperative until deadline",
                "resumed kernel context transfer other until deadline",
            ),
            "FIFO-selected context",
        ),
        (
            SOURCE.replace("capacity is 3", "capacity is 2"),
            "capacity three",
        ),
        (
            SOURCE.replace(
                "runnable kernel runnable enqueue cooperative-worker",
                "runnable kernel runnable enqueue terminal-worker",
            ),
            "selected suspended context",
        ),
        (
            SOURCE.replace(
                "resumed console write \"TOPAL_KERNEL_DEADLINE_PREEMPTED\"",
                "resumed console write \"TOPAL_KERNEL_CONTEXT_EARLY\"",
            ),
            "exact success marker",
        ),
        (
            SOURCE.replace(
                "selected-cooperative until deadline",
                "selected-cooperative until other-deadline",
            ),
            "exact deadline",
        ),
        (
            SOURCE.replace(
                "preempted kernel context take preempted",
                "other kernel context take preempted",
            ),
            "exact completed outcome",
        ),
        (
            SOURCE.replace(
                "context-resumed is cooperative-completed kernel context reclaim",
                "context-resumed is other kernel context reclaim",
            ),
            "matching completed transfer",
        ),
        (
            SOURCE.replace(
                "completion-resumed kernel runnable queue consume empty runnable",
                "completion-resumed kernel runnable queue consume empty other",
            ),
            "exact empty queue",
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

#[test]
fn requires_exact_affine_user_syscall_round_trip() {
    // TOPAL-SYSTEMS-CONTEXT-006 negative conformance.
    for (source, expected_code) in [
        (
            SOURCE.replace(
                "  user-syscall is lang systems user-syscall-entry user-syscall-handler\n",
                "",
            ),
            "E-SYSTEMS-ENTRY-SET",
        ),
        (
            SOURCE.replace("process-identity is 1", "process-identity is 2"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("getpid-result is 1", "getpid-result is 2"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "completion-resumed user context enter user-probe",
                "completion-resumed user context enter other-probe",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("context : InitialUserSyscallContext", "context : OtherContext"),
            "E-SYSTEMS-HANDLER",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, expected_code, "{}", error.message);
    }
}
