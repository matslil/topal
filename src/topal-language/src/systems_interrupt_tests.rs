#[test]
fn requires_typed_local_notification_entry_completion_and_wait() {
    // TOPAL-SYSTEMS-LOCAL-INTERRUPT-001.
    for (source, code, expected) in [
        (
            SOURCE.replace(
                "  local-notification is lang systems external-interrupt-entry local-notification-handler,\n",
                "",
            ),
            "E-SYSTEMS-ENTRY-SET",
            "local-notification",
        ),
        (
            SOURCE.replace(
                "context : LocalNotificationInterruptContext",
                "context : DebugBreakContext",
            ),
            "E-SYSTEMS-HANDLER",
            "LocalNotificationInterruptContext",
        ),
        (
            SOURCE.replace(
                "completed is context local notification complete",
                "completed is context local notification acknowledge",
            ),
            "E-SYSTEMS-LOCAL-INTERRUPT",
            "consume",
        ),
        (
            SOURCE.replace("  completed resume", "  context resume"),
            "E-SYSTEMS-DISPOSITION",
            "not a disposition",
        ),
        (
            SOURCE.replace(
                "pending is restored local notification send",
                "pending is stale local notification send",
            ),
            "E-SYSTEMS-LOCAL-INTERRUPT",
            "current processor context",
        ),
        (
            SOURCE.replace(
                "resumed is pending local notification wait",
                "resumed is other local notification wait",
            ),
            "E-SYSTEMS-LOCAL-INTERRUPT",
            "matching pending session",
        ),
        (
            SOURCE.replace(
                "resumed console write \"TOPAL_KERNEL_INTERRUPT_OK\"",
                "resumed console write \"TOPAL_KERNEL_INTERRUPT_EARLY\"",
            ),
            "E-SYSTEMS-LOCAL-INTERRUPT",
            "exact interrupt success marker",
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
