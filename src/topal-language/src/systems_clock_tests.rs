#[test]
fn requires_two_typed_monotonic_clock_observations_before_success() {
    // TOPAL-SYSTEMS-MONOTONIC-CLOCK-001.
    for (source, code, expected) in [
        (
            SOURCE.replace(
                "first : Instant InitialMonotonicClock",
                "first : Instant OtherClock",
            ),
            "E-SYSTEMS-MONOTONIC-CLOCK",
            "initial monotonic clock classifier",
        ),
        (
            SOURCE.replace(
                "first : Instant InitialMonotonicClock is resumed monotonic clock now",
                "first : Instant InitialMonotonicClock is stale monotonic clock now",
            ),
            "E-SYSTEMS-MONOTONIC-CLOCK",
            "resumed context",
        ),
        (
            SOURCE.replace(
                "second : Instant InitialMonotonicClock",
                "first : Instant InitialMonotonicClock",
            ),
            "E-SYSTEMS-MONOTONIC-CLOCK",
            "distinct instant bindings",
        ),
        (
            SOURCE.replace("resumed monotonic clock now", "resumed monotonic clock read"),
            "E-SYSTEMS-MONOTONIC-CLOCK",
            "initial monotonic clock classifier",
        ),
        (
            SOURCE.replace(
                "resumed console write \"TOPAL_KERNEL_TIME_OK\"",
                "resumed console write \"TOPAL_KERNEL_TIME_EARLY\"",
            ),
            "E-SYSTEMS-MONOTONIC-CLOCK",
            "exact time success marker",
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

    let missing = SOURCE.replace(
        "                                                                                  second : Instant InitialMonotonicClock is resumed monotonic clock now\n",
        "",
    );
    assert!(
        analyze_systems_for_compiler(
            &missing,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .is_err()
    );
}
