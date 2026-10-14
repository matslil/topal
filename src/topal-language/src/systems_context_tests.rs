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
        [
            SYSTEMS_USER_SYSCALL_DISPATCH,
            SYSTEMS_USER_COPY_FROM,
            SYSTEMS_USER_COPY_TO,
        ]
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
            SOURCE.replace("uname-machine is \"x86_64\"", "uname-machine is \"aarch64\""),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("nanosleep-nanoseconds is 0", "nanosleep-nanoseconds is 1"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("write-descriptor is 1", "write-descriptor is 2"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("write-maximum-bytes is 256", "write-maximum-bytes is 255"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("read-descriptor is 0", "read-descriptor is 1"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("read-maximum-bytes is 256", "read-maximum-bytes is 255"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("brk-data-page-bytes is 4096", "brk-data-page-bytes is 8192"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_COPY_TO_OK",
                "TOPAL_KERNEL_USER_COPY_TO_EARLY",
            ),
            "E-SYSTEMS-CONTEXT-TRANSFER",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_COPY_FROM_OK",
                "TOPAL_KERNEL_USER_COPY_FROM_EARLY",
            ),
            "E-SYSTEMS-CONTEXT-TRANSFER",
        ),
        (
            SOURCE.replace(
                "completion-resumed user context enter user-probe",
                "completion-resumed user context enter other-probe",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("backing is user-frames", "backing is other-frames"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("layout is initial-probe", "layout is other-layout"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("address-space is user-space", "address-space is other-space"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            replace_last(SOURCE, "frame-count is 1", "frame-count is 2"),
            "E-SYSTEMS-FRAMES",
        ),
        (
            SOURCE.replace("backing is scratch-frame", "backing is other-scratch"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("layout is probe-code", "layout is other-layout"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "user mapping install entry trampoline mapped-scratch",
                "user mapping install entry trampoline other-mapping",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "user mapping protect read only stored-scratch",
                "user mapping protect read only mapped-scratch",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "user mapping verify entry trampoline protected-scratch",
                "user mapping verify entry trampoline stored-scratch",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("mapping is executable-scratch", "mapping is stored-scratch"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "user mapping permit execute verified-scratch",
                "user mapping permit execute protected-scratch",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "completion-resumed frames release returned-data",
                "completion-resumed frames release other-scratch",
            ),
            "E-SYSTEMS-FRAMES",
        ),
        (
            SOURCE.replace("backing is data-frame", "backing is scratch-frame"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "user mapping store byte mapped-data",
                "user mapping store byte mapped-scratch",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "user mapping protect read only stored-data",
                "user mapping protect read only stored-scratch",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "mapping is writable-data",
                "mapping is executable-scratch",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "user mapping permit writable data verified-data",
                "user mapping permit writable data verified-scratch",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace("layout is probe-data", "layout is probe-code"),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_READY",
                "TOPAL_KERNEL_USER_IMAGE_NOT_READY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_BOOT_RESOURCE_OK",
                "TOPAL_KERNEL_BOOT_RESOURCE_NOT_OK",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK",
                "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_NOT_OK",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK",
                "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_NOT_OK",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK",
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_NOT_OK",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK",
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_NOT_OK",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK",
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_NOT_OK",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_BSS_OK",
                "TOPAL_KERNEL_USER_IMAGE_BSS_NOT_OK",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK",
                "TOPAL_KERNEL_USER_IMAGE_ENTRY_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_STACK_OK",
                "TOPAL_KERNEL_USER_IMAGE_STACK_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_AUXV_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_ELF_OK",
                "TOPAL_KERNEL_USER_IMAGE_ELF_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_DATA_OK",
                "TOPAL_KERNEL_USER_IMAGE_DATA_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK",
                "TOPAL_KERNEL_USER_IMAGE_COPY_TO_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK",
                "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK",
                "TOPAL_KERNEL_USER_IMAGE_SYSCALL_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_IMAGE_RETAINED",
                "TOPAL_KERNEL_USER_IMAGE_NOT_RETAINED",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_USER_MAPPING_OK",
                "TOPAL_KERNEL_USER_MAPPING_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "TOPAL_KERNEL_FRAME_DISJOINT_OK",
                "TOPAL_KERNEL_FRAME_DISJOINT_EARLY",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "completion-resumed frames release returned-frames",
                "completion-resumed frames release user-frames",
            ),
            "E-SYSTEMS-FRAMES",
        ),
        (
            SOURCE.replace(
                "revoked-space is completion-resumed user execution complete user-completed",
                "completion-resumed user execution complete user-completed",
            ),
            "E-SYSTEMS-USER-CONTEXT",
        ),
        (
            SOURCE.replace(
                "returned-frames is completion-resumed user address space reclaim revoked-space",
                "returned-frames is completion-resumed user address space reclaim other-space",
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

fn replace_last(source: &str, from: &str, to: &str) -> String {
    let (prefix, suffix) = source.rsplit_once(from).expect("fixture contains probe text");
    format!("{prefix}{to}{suffix}")
}
