#[test]
fn requires_typed_affine_deadline_event_lifecycle() {
    // TOPAL-SYSTEMS-DEADLINE-EVENT-001.
    for (source, code, expected) in [
        (
            SOURCE.replace(
                "  deadline-notification is lang systems external-interrupt-entry deadline-notification-handler,\n",
                "",
            ),
            "E-SYSTEMS-ENTRY-SET",
            "deadline-notification",
        ),
        (
            SOURCE.replace(
                "context : DeadlineInterruptContext InitialMonotonicClock",
                "context : DeadlineInterruptContext OtherClock",
            ),
            "E-SYSTEMS-HANDLER",
            "DeadlineInterruptContext InitialMonotonicClock",
        ),
        (
            SOURCE.replace(
                "completed is context deadline notification complete",
                "completed is context deadline notification acknowledge",
            ),
            "E-SYSTEMS-DEADLINE-EVENT",
            "consume",
        ),
        (
            SOURCE.replace(
                "completed preempt current kernel context",
                "completed resume",
            ),
            "E-SYSTEMS-DEADLINE-EVENT",
            "preempt",
        ),
        (
            SOURCE.replace(
                "second deadline after 1[ms]",
                "first deadline after 1[ms]",
            ),
            "E-SYSTEMS-DEADLINE-EVENT",
            "second observation",
        ),
        (
            SOURCE.replace("second deadline after 1[ms]", "second deadline after 2[ms]"),
            "E-SYSTEMS-DEADLINE-EVENT",
            "exact duration",
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
