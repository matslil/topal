#[test]
fn executes_the_shared_task_declaration_order_example_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("task-declaration-order.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("task.state.replaced [TOPAL-TASK-STATE-001] count"));
    assert!(stdout.contains("message.sent [TOPAL-CONC-INTERACT-001] transaction="));
    assert!(stdout.contains("\n3\n"));
}

#[test]
fn executes_the_shared_lint_language_variant_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("lint-language-variant.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("lint.context.viewed [TOPAL-SYN-CONTEXT-001] lang lint"));
    assert!(stdout.contains("<namespace lang lint>"));
}

#[test]
fn records_reversible_checked_location_access() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("external-layout-location.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("location.written [TOPAL-LOCATION-WRITE-001] control"));
    assert!(stdout.contains("location.read [TOPAL-LOCATION-READ-001] control"));
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_contextual_infinity_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("infinity-values-and-ranges.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("numeric.infinity.constructed [TOPAL-NUM-INFINITY-001]"));
    assert!(stdout.contains("-Infinity ..= +Infinity"));
}

#[test]
fn records_reversible_contextual_rational_infinity_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("rational-infinity-values-and-ranges.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("numeric.infinity.constructed [TOPAL-NUM-INFINITY-001]"));
    assert!(stdout.contains("Rational ( -1, 1 ) ..= +Infinity"));
}

#[test]
fn records_reversible_exact_infinity_arithmetic() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("infinity-arithmetic.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("numeric.infinity.arithmetic [TOPAL-NUM-INFINITY-ARITHMETIC-001]"));
    assert!(stdout.contains("(+Infinity, -Infinity, +Infinity"));
}

#[test]
fn records_reversible_private_infinity_boundaries() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("infinity-private-boundaries.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("numeric.infinity.constructed [TOPAL-NUM-INFINITY-001]"));
    assert!(stdout.contains("function.entry [TOPAL-FUNCTION-ORDINARY-001]"));
    assert!(
        stdout.contains(
            "(integer is +Infinity, ratio is -Infinity), -Infinity, +Infinity, +Infinity)"
        )
    );
}

#[test]
fn records_reversible_dynamic_infinity_results() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}scripts/finish-and-reverse.debug"),
            &language_example("dynamic-infinity-results.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("numeric.infinity.arithmetic [TOPAL-NUM-INFINITY-ARITHMETIC-001]"));
    assert!(stdout.contains("result.error.constructed [TOPAL-TYPE-RESULT-001]"));
    assert!(
        stdout.contains("(-Infinity, Error ( domain is root.*(Int,Int), code is indeterminate )")
    );
}

#[test]
fn records_reversible_broad_unicode_identifier_bindings() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}unicode-identifiers.debug"),
            &language_example("unicode-identifiers.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("binding.bind [TOPAL-SYN-BIND-001] 🙂"));
    assert!(stdout.contains("binding.bind [TOPAL-SYN-BIND-001] left+right"));
    assert!(stdout.contains("\n40\n"));
    assert!(stdout.contains("\n2\n"));
}

#[test]
fn help_prints_source_standard_library_and_builtin_documentation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let library = concat!(env!("CARGO_MANIFEST_DIR"), "/../../library");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--library-root",
            library,
            "--script",
            &format!("{root}documentation-help.debug"),
            &format!("{root}documentation-help.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Return the documented answer"));
    assert!(stdout.contains("Return the smaller value"));
    assert!(stdout.contains("Test whether an HTTP method has safe semantics"));
    assert!(stdout.contains("Arbitrary-precision signed integers"));
    assert!(stdout.contains("next: advance to the next location in the current source file"));
    assert!(stdout.contains("no visible bindings"));
    assert!(stdout.contains("step | reverse-step"));
}
