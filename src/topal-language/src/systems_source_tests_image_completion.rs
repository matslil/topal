#[test]
fn requires_exact_static_elf_entropy_request_completion_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-REQUEST-COMPLETION-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-queue-activation,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-entropy-request-completion-coexistence",
            "policy is synthetic-entropy-request-completion",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy request-completion")
                || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_sample_capture_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SAMPLE-CAPTURE-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-request-completion,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "sample is initial-virtio-rng-sample,",
            "sample is synthetic-entropy-sample,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-entropy-sample-capture-coexistence",
            "policy is synthetic-entropy-sample-capture",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy-sample capture")
                || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_sample_quality_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SAMPLE-QUALITY-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-sample-capture,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "evidence is initial-q35-virtio-rng-trust-evidence,",
            "evidence is synthetic-trust-evidence,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-entropy-sample-quality-coexistence",
            "policy is synthetic-entropy-sample-quality",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy-sample quality")
                || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_pool_lifecycle_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-POOL-LIFECYCLE-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-sample-quality,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "pool is initial-entropy-pool,",
            "pool is synthetic-entropy-pool,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-entropy-pool-lifecycle-coexistence",
            "policy is synthetic-entropy-pool-lifecycle",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy-pool lifecycle")
                || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_entropy_generator_lifecycle_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-GENERATOR-LIFECYCLE-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-pool-lifecycle,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "generator is initial-entropy-generator,",
            "generator is synthetic-entropy-generator,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-entropy-generator-lifecycle-coexistence",
            "policy is synthetic-entropy-generator-lifecycle",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK",
            "TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("entropy-generator lifecycle")
                || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_initial_random_delivery_coexistence() {
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-RANDOM-DELIVERY-COEXISTENCE-001.
    for source in [
        SOURCE.replace(
            "owner is entropy-acquiring-static-process-after-generator-lifecycle,",
            "owner is entropy-pending-static-process,",
        ),
        SOURCE.replace(
            "delivery is initial-auxv-random-delivery,",
            "delivery is synthetic-random-delivery,",
        ),
        SOURCE.replace(
            "policy is bounded-static-elf-initial-random-delivery-coexistence",
            "policy is synthetic-initial-random-delivery",
        ),
        SOURCE.replace(
            "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK",
            "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_BAD",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert!(
            error.message.contains("initial-random") || error.message.contains("exact marker"),
            "{}",
            error.message
        );
    }
}

#[test]
fn requires_exact_static_elf_process_entry() {
    // TOPAL-SYSTEMS-STATIC-ELF-PROCESS-ENTRY-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-BRK-QUERY-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-BRK-GROWTH-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-X64-FS-BASE-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CLEAR-TID-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-EMPTY-ROBUST-LIST-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-RSEQ-FALLBACK-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-STACK-LIMIT-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-READLINKAT-FALLBACK-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-GETRANDOM-001,
    // TOPAL-SYSTEMS-STATIC-ELF-REUSABLE-GETRANDOM-001,
    // TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-QUERY-001,
    // TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-GROWTH-001,
    // TOPAL-SYSTEMS-STATIC-ELF-POST-RANDOM-BRK-PAGE-ALIGNMENT-001,
    // TOPAL-SYSTEMS-STATIC-ELF-RELRO-MPROTECT-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-PROCESS-NAME-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-GETUID-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CONFIG-STAT-MISS-001,
    // TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CREDENTIAL-TAIL-001,
    // TOPAL-SYSTEMS-STATIC-ELF-APPLET-NEWLINE-WRITE-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-GETPID-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-SIGCHLD-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-GETPPID-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-EMPTY-INPUT-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-EXIT-COMMAND-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-ECHO-COMMAND-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-CONSOLE-FSTAT-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-DEV-TTY-OPEN-MISS-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-STANDARD-TERMINAL-PROBES-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-TERMINAL-WARNING-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-MAILPATH-STAT-MISS-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-BANNER-001,
    // TOPAL-SYSTEMS-STATIC-ELF-SHELL-POST-BANNER-TERMINAL-PROBE-001.
    for source in [
            SOURCE.replace("abi is linux-x86-64-process,", "abi is synthetic-process,"),
            SOURCE.replace(
                "startup is initial-brk-query-growth-x64-fs-base-clear-tid-empty-robust-list-rseq-fallback-stack-limit-readlinkat-fallback-getrandom-repeated-brk-query-growth-page-alignment-relro-mprotect-process-name-getuid-config-stat-miss-credential-tail-applet-newline-write-shell-getpid-sigchld-disposition-getppid-uname-root-getcwd-tcgets-miss-signal-defaults-quit-ignore-sealed-echo-command-console-fstat-dev-tty-open-miss-standard-terminal-probes-terminal-warning-mailpath-stat-miss-banner-post-banner-terminal-probe-post-banner-root-getcwd-post-banner-geteuid-profile-open-profile-fstat-profile-read-profile-close-prompt-refill-input-pwd-command-cd-root,",
                "startup is synthetic-startup,",
            ),
            SOURCE.replace(
                "terminal is exit-group-zero,",
                "terminal is exit-group-one,",
            ),
            SOURCE.replace(
                "policy is bounded-static-elf-shell-cd-root-command",
                "policy is synthetic-process-entry",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CLEAR_TID_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CLEAR_TID_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_EMPTY_ROBUST_LIST_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_EMPTY_ROBUST_LIST_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RSEQ_FALLBACK_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RSEQ_FALLBACK_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_STACK_LIMIT_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_STACK_LIMIT_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_READLINKAT_FALLBACK_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_READLINKAT_FALLBACK_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETRANDOM_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETRANDOM_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_REUSABLE_GETRANDOM_OK",
                "TOPAL_KERNEL_STATIC_ELF_REUSABLE_GETRANDOM_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_OK",
                "TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CONFIG_STAT_PROFILE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CONFIG_STAT_PROFILE_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CREDENTIAL_TAIL_PROFILE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CREDENTIAL_TAIL_PROFILE_BAD",
            ),
            SOURCE.replace(
                "TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_BAD",
            ),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert!(
                error.message.contains("process entry") || error.message.contains("exact marker"),
                "{}",
                error.message
            );
        }
}

#[test]
fn rejects_wrong_handler_signatures_and_open_entry_sets() {
    // TOPAL-SYSTEMS-ENTRY-001, TOPAL-SYSTEMS-VOCABULARY-001.
    let wrong_signature =
        SOURCE.replace("context : BootstrapContext", "context : DebugBreakContext");
    assert_eq!(
        analyze_systems_for_compiler(
            &wrong_signature,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err()
        .code,
        "E-SYSTEMS-HANDLER"
    );

    let open_entries = SOURCE.replace(
            "  deadline-notification is lang systems external-interrupt-entry deadline-notification-handler,\n",
            "  deadline-notification is lang systems external-interrupt-entry deadline-notification-handler,\n  interrupt is lang systems interrupt-entry interrupt-handler,\n",
        );
    assert_eq!(
        analyze_systems_for_compiler(
            &open_entries,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err()
        .code,
        "E-SYSTEMS-ENTRY-SET"
    );
}

#[test]
fn rejects_missing_malformed_and_open_bootstrap_storage() {
    // TOPAL-SYSTEMS-VOCABULARY-001, TOPAL-SYSTEMS-STORAGE-001.
    let missing = SOURCE.replace(STORAGE_DECLARATION, "");
    assert_eq!(
        analyze_systems_for_compiler(
            &missing,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err()
        .code,
        "E-SYSTEMS-ENTRY-SET"
    );

    let duplicate = SOURCE.replace(
        STORAGE_DECLARATION,
        &format!("{STORAGE_DECLARATION}{STORAGE_DECLARATION}"),
    );
    assert_eq!(
        analyze_systems_for_compiler(
            &duplicate,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err()
        .code,
        "E-SYSTEMS-ENTRY-SET"
    );

    for malformed in [
        SOURCE.replace("    capacity-bytes is 65536,\n", ""),
        SOURCE.replace("capacity-bytes is 65536", "capacity-bytes is 0"),
        SOURCE.replace("alignment-bytes is 4096", "alignment-bytes is 3"),
        SOURCE.replace("alignment-bytes is 4096", "alignment-bytes is 131072"),
        SOURCE.replace("capacity-bytes is 65536", "capacity-bytes is capacity"),
        SOURCE.replace(
            "alignment-bytes is 4096",
            "alignment-bytes is 4096,\n    physical-address is 4096",
        ),
    ] {
        assert_eq!(
            analyze_systems_for_compiler(
                &malformed,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-STORAGE"
        );
    }
}

#[test]
fn a_fatal_debug_break_disposition_stops_the_bootstrap_model() {
    let fatal_debug = SOURCE.replace("context resume", "context fatal \"debug fatal\"");
    let program = analyze_systems_for_compiler(
        &fatal_debug,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let transitions = model_systems_transitions(&program).unwrap();
    assert_eq!(
        transitions.last(),
        Some(&CompilerSystemsTransition::Fatal {
            message: "debug fatal".into(),
        })
    );
    assert!(
        !transitions.contains(&CompilerSystemsTransition::ConsoleWrite {
            text: "TOPAL_KERNEL_FAULT_RESUMED".into(),
        })
    );
}

include!("systems_mapping_tests.rs");
include!("systems_translation_tests.rs");
include!("systems_translation_edit_tests.rs");
include!("systems_critical_tests.rs");
include!("systems_interrupt_tests.rs");
include!("systems_clock_tests.rs");
include!("systems_deadline_tests.rs");
include!("systems_context_tests.rs");
