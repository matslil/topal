#[test]
fn requires_typed_affine_deadline_event_lifecycle() {
    // TOPAL-SYSTEMS-DEADLINE-EVENT-001.
    for (source, code, expected) in [
        (
            SOURCE.replace(
                "  deadline-notification is lang systems external-interrupt-entry deadline-notification-handler\n",
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
        (
            SOURCE.replace(
                "resumed deadline notification arm deadline",
                "stale deadline notification arm deadline",
            ),
            "E-SYSTEMS-DEADLINE-EVENT",
            "resumed context",
        ),
        (
            SOURCE.replace(
                "ArmedDeadline InitialMonotonicClock",
                "ArmedDeadline OtherClock",
            ),
            "E-SYSTEMS-DEADLINE-EVENT",
            "same-clock",
        ),
        (
            SOURCE.replace(
                "resumed deadline notification wait armed",
                "resumed deadline notification wait other",
            ),
            "E-SYSTEMS-DEADLINE-EVENT",
            "matching affine armed event",
        ),
        (
            SOURCE.replace(
                "deadline-resumed console write \"TOPAL_KERNEL_DEADLINE_OK\"",
                "deadline-resumed console write \"TOPAL_KERNEL_DEADLINE_EARLY\"",
            ),
            "E-SYSTEMS-DEADLINE-EVENT",
            "exact deadline success marker",
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
