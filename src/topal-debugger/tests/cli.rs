use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn navigates_recorded_execution_in_both_directions() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.t"
    );
    let commands = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.debug"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--script", commands, source])
        .output()
        .expect("debugger should run script");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("#0 context.selected [TOPAL-SYN-UNICODE-001]"));
    assert!(stdout.contains(
        "decision #0: context.selected because TOPAL-SYN-UNICODE-001 (design-0;Unicode=17.0.0)"
    ));
    assert!(stdout.contains("#1 source.accepted [TOPAL-SYN-SOURCE-001]"));
    assert!(stdout.contains("> #0 context.selected"));
    assert!(stdout.contains("no value at current execution state"));
    assert!(stdout.contains("basic-history.t:4:1"));
    assert!(stdout.contains("answer is 40"));
    assert!(stdout.contains("breakpoint set at line 3"));
    assert!(stdout.contains("breakpoint set at line 4"));
    assert!(stdout.contains("breakpoint removed from line 3"));
    assert!(stdout.contains("watchpoint set for answer"));
    assert!(stdout.contains("watchpoint removed for answer"));
    assert!(stdout.contains("checkpoint result saved"));
    assert!(stdout.contains("checkpoint result restored"));
    assert!(stdout.contains("checkpoint result deleted"));
    assert!(stdout.contains("#0 <script> before first statement"));
    assert!(stdout.contains("#0 <script> at"));
}

#[test]
fn executes_the_debuggee_only_when_commands_advance_it() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}live-execution.debug"),
            &format!("{root}basic-history.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let before = stdout.find("no value at current execution state").unwrap();
    let binding = stdout.find("answer = 40").unwrap();
    let result = stdout.rfind("\n42\n").unwrap();
    assert!(before < binding && binding < result);
}

#[test]
fn retains_inspectable_history_when_live_execution_fails() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}failing-history.debug"),
            &format!("{root}failing-history.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("error[E-UNBOUND-NAME]: name is not bound"));
    assert!(stdout.contains("answer = 40"));
    assert!(stdout.contains("binding.created [TOPAL-SYN-BIND-001] answer"));
    assert!(stdout.contains("no value at current execution state"));
}

#[test]
fn exposes_intermediate_expression_values_as_reversible_states() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}expression-stepping.debug"),
            &format!("{root}expression-stepping.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for value in ["\n40\n", "\n41\n", "\n42\n"] {
        assert!(
            stdout.contains(value),
            "missing intermediate value {value:?}"
        );
    }
}

#[test]
fn records_reversible_static_function_call_decisions() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}static-function-call.debug"),
            &format!("{root}static-function-call.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let declaration = stdout.find("function.declared").unwrap();
    let entered = stdout.find("function.entered").unwrap();
    let body = stdout.find("root.+(Int,Int)").unwrap();
    let returned = stdout.find("function.returned").unwrap();
    assert!(declaration < entered && entered < body && body < returned);
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_static_function_argument_binding() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}static-unary-function.debug"),
            &format!("{root}static-unary-function.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let bound = stdout.find("function.argument.bound").unwrap();
    let entered = stdout.find("function.entered").unwrap();
    let body = stdout.find("root.+(Int,Int)").unwrap();
    let returned = stdout.find("function.returned").unwrap();
    assert!(bound < entered && entered < body && body < returned);
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_static_product_argument_bindings() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}static-product-function.debug"),
            &format!("{root}static-product-function.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let left = stdout
        .find("function.argument.bound [TOPAL-FUNCTION-STATIC-BINARY-001] left")
        .unwrap();
    let right = stdout
        .find("function.argument.bound [TOPAL-FUNCTION-STATIC-BINARY-001] right")
        .unwrap();
    let entered = stdout.find("function.entered").unwrap();
    let created = stdout
        .find("binding.created [TOPAL-SYN-BIND-001] sum")
        .unwrap();
    let resolved = stdout
        .find("binding.resolved [TOPAL-SYN-BIND-001] sum")
        .unwrap();
    assert!(left < right && right < entered && entered < created && created < resolved);
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_explicit_function_return() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-return.debug"),
            &format!("{root}function-return.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let explicit = stdout.find("function.return.explicit").unwrap();
    let returned = stdout.find("function.returned").unwrap();
    assert!(explicit < returned);
    assert!(!stdout.contains("binding.resolved [TOPAL-SYN-BIND-001] missing"));
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_ordinary_function_execution() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}ordinary-function.debug"),
            &format!("{root}ordinary-function.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.entered [TOPAL-FUNCTION-ORDINARY-001] subtract"));
    assert!(stdout.contains("function.returned [TOPAL-FUNCTION-ORDINARY-001] subtract"));
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_nat_function_execution() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nat-functions.debug"),
            &format!("{root}nat-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("identity (Nat)"));
    assert!(stdout.contains("function.entered [TOPAL-FUNCTION-ORDINARY-001] identity"));
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nat-recursion.debug"),
            &format!("{root}nat-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-001"));
    assert!(stdout.contains("function.recursion.descended"));
    assert!(stdout.contains("\n2\n"));
}

#[test]
fn records_reversible_increasing_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nat-increasing-recursion.debug"),
            &format!("{root}nat-increasing-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"));
    assert!(stdout.contains("function.recursion.descended"));
    assert!(stdout.contains("\n6\n"));
}

#[test]
fn records_reversible_mutual_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nat-mutual-recursion.debug"),
            &format!("{root}nat-mutual-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"));
    assert!(stdout.contains("function.recursion.cycle.proven"));
    assert!(stdout.contains("\n(true, false)\n"));
}

#[test]
fn records_reversible_mutual_increasing_nat_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nat-mutual-increasing-recursion.debug"),
            &format!("{root}nat-mutual-increasing-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"));
    assert!(stdout.contains("function.recursion.cycle.proven"));
    assert!(stdout.contains("\n(true, false)\n"));
}

#[test]
fn records_reversible_enum_values() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}enum-values.debug"),
            &format!("{root}enum-values.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("enum.declared [TOPAL-TYPE-ENUM-001] Color"));
    assert!(stdout.contains("\n(Red, Green, true, false)\n"));
}

#[test]
fn records_reversible_enum_function_classification() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}enum-functions.debug"),
            &format!("{root}enum-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("identity (Color)"));
    assert!(stdout.contains("\n(Red, Green)\n"));
}

#[test]
fn records_reversible_exhaustive_enum_decision() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}enum-decisions.debug"),
            &format!("{root}enum-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-ENUM-001"));
    assert!(stdout.contains("decision.rule.considered"));
    assert!(stdout.contains("\n(\"red\", \"green\")\n"));
}

#[test]
fn records_reversible_arithmetic_error_code_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}arithmetic-error-codes.debug"),
            &format!("{root}arithmetic-error-codes.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(
            "namespace.member.selected [TOPAL-NUM-ARITHMETIC-ERROR-001] division-by-zero"
        )
    );
    assert!(stdout.contains("\n(division-by-zero, indeterminate, true)\n"));
}

#[test]
fn records_reversible_successful_result_contract() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}result-success.debug"),
            &format!("{root}result-success.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.result.contract [TOPAL-TYPE-RESULT-001]"));
    assert!(stdout.contains("\nRational ( 3, 2 )\n"));
}

#[test]
fn records_reversible_dynamic_division_error() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}result-division-error.debug"),
            &format!("{root}result-division-error.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result.error.constructed [TOPAL-TYPE-RESULT-001]"));
    assert!(
        stdout.contains("Error ( domain is root./(Rational,Rational), code is division-by-zero )")
    );
}

#[test]
fn records_reversible_negative_rational_exponentiation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}rational-negative-exponent.debug"),
            &format!("{root}rational-negative-exponent.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RAT-NEG-POW-001"));
    assert!(stdout.contains("\nRational ( 4, 9 )\n"));
}

#[test]
fn records_reversible_dynamic_negative_power_error() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}result-negative-power-error.debug"),
            &format!("{root}result-negative-power-error.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.^(Rational,Int);division-by-zero"));
    assert!(stdout.contains("Error ( domain is root.^(Rational,Int), code is division-by-zero )"));
}

#[test]
fn records_reversible_result_error_propagation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}result-error-propagation.debug"),
            &format!("{root}result-error-propagation.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result.error.constructed"));
    assert!(stdout.contains("result.error.propagated"));
    assert!(stdout.contains("domain=root./(Rational,Rational);code=division-by-zero"));
}

#[test]
fn records_reversible_result_decisions() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}result-decisions.debug"),
            &format!("{root}result-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-RESULT-001"));
    assert!(stdout.contains("result.payload.bound"));
    assert!(stdout.contains("\n(\"ok\", \"error\")\n"));
}

#[test]
fn records_reversible_error_field_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}error-field-selection.debug"),
            &format!("{root}error-field-selection.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-ERROR-FIELD-001"));
    assert!(stdout.contains("error.field.selected"));
    assert!(stdout.contains("(division-by-zero, root./(Rational,Rational))"));
}

#[test]
fn records_reversible_error_code_decisions() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}error-code-decisions.debug"),
            &format!("{root}error-code-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-ERROR-CODE-001"));
    assert!(stdout.contains("error.code.matched"));
    assert!(stdout.contains("(\"ok\", \"division by zero\")"));
}

#[test]
fn records_reversible_result_success_projection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}result-success-projection.debug"),
            &format!("{root}result-success-projection.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-RESULT-PROJECT-001"));
    assert!(stdout.contains("result.success.projected"));
    assert!(stdout.contains("result.error.projected"));
}

#[test]
fn records_reversible_exhaustive_error_code_decision() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}exhaustive-error-code-decisions.debug"),
            &format!("{root}exhaustive-error-code-decisions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-DECISION-ERROR-CODE-001"));
    assert!(stdout.contains("error.code.matched"));
    assert!(stdout.contains("(\"ok\", \"zero\")"));
}

#[test]
fn records_reversible_character_classification() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}character-classification.debug"),
            &format!("{root}character-classification.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.argument.bound"));
    assert!(stdout.contains("string.from-character"));
    assert!(stdout.contains("(\"🙂\", \"a\u{301}\")"));
}

#[test]
fn records_reversible_int_euclidean_modulo() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}int-euclidean-modulo.debug"),
            &format!("{root}int-euclidean-modulo.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-INT-MODULO-001"));
    assert!(stdout.contains("TOPAL-NUM-INT-QUOTIENT-MODULO-001"));
    assert!(stdout.contains("root.%(Int,Int);division-by-zero"));
    assert!(stdout.contains("Error ( domain is root.%(Int,Int), code is division-by-zero )"));
    assert!(stdout.contains("Error ( domain is root./%(Int,Int), code is division-by-zero )"));
}

#[test]
fn records_reversible_exact_numeric_absolute() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}exact-numeric-absolute.debug"),
            &format!("{root}exact-numeric-absolute.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.absolute(Int)"));
    assert!(stdout.contains("root.absolute(Rational)"));
    assert!(stdout.contains("(42, 42, Rational ( 5, 4 ), Rational ( 5, 4 ))"));
}

#[test]
fn records_reversible_named_exact_numeric_negation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}exact-numeric-negate.debug"),
            &format!("{root}exact-numeric-negate.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.negate(Int)"));
    assert!(stdout.contains("root.negate(Rational)"));
    assert!(stdout.contains("(-42, 42, Rational ( -5, 4 ), Rational ( 5, 4 ))"));
}

#[test]
fn records_reversible_exact_numeric_zero() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}exact-numeric-zero.debug"),
            &format!("{root}exact-numeric-zero.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.zero(Int)"));
    assert!(stdout.contains("root.zero(Rational)"));
    assert!(stdout.contains("root.one(Int)"));
    assert!(stdout.contains("root.zero(Nat)"));
    assert!(stdout.contains("root.one(Nat)"));
    assert!(stdout.contains("root.one(Rational)"));
    assert!(stdout.contains("(0, 0, Rational ( 0, 1 ), 1, 1, Rational ( 1, 1 ))"));
}

#[test]
fn records_reversible_exact_three_way_comparison() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}exact-three-way-comparison.debug"),
            &format!("{root}exact-three-way-comparison.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-THREE-WAY-COMPARE-001"));
    assert!(stdout.contains("Int->Rational:left"));
    assert!(stdout.contains("TOPAL-DECISION-ENUM-001"));
    assert!(stdout.contains("(Less, Equal, Greater, Less, \"less\", \"equal\", \"greater\")"));
}

#[test]
fn records_reversible_exact_rational_int_narrowing() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}exact-rational-int-narrowing.debug"),
            &format!("{root}exact-rational-int-narrowing.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-INT-EXACT-001"));
    assert!(stdout.contains("Rational->Int:exact"));
    assert!(stdout.contains("(50, -3)"));
}

#[test]
fn records_reversible_dynamic_rational_int_validation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}dynamic-rational-int-validation.debug"),
            &format!("{root}dynamic-rational-int-validation.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001"));
    assert!(stdout.contains("Rational->Int:validated"));
    assert!(stdout.contains("root.Int(Rational);not-representable"));
    assert!(
        stdout.contains("(50, Error ( domain is root.Int(Rational), code is not-representable ))")
    );
}

#[test]
fn records_reversible_checked_int_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}int-checked-construction.debug"),
            &format!("{root}int-checked-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-INT-CONSTRUCT-001"));
    assert!(stdout.contains("Int->Int:identity"));
    assert!(stdout.contains("Rational->Int:exact"));
    assert!(stdout.contains("root.Int(Rational);not-representable"));
    assert!(
        stdout
            .contains("(7, 6, Error ( domain is root.Int(Rational), code is not-representable ))")
    );
}

#[test]
fn records_reversible_checked_nat_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nat-checked-construction.debug"),
            &format!("{root}nat-checked-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-NAT-CONSTRUCT-001"));
    assert!(stdout.contains("Int->Nat:nonnegative"));
    assert!(stdout.contains("root.Nat(Int);out-of-range"));
    assert!(stdout.contains("(7, 6, Error ( domain is root.Nat(Int), code is out-of-range ))"));
}

#[test]
fn records_reversible_canonical_rational_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}rational-exact-construction.debug"),
            &format!("{root}rational-exact-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-001"));
    assert!(stdout.contains("numeric.rational.constructed"));
    assert!(stdout.contains("Int->Rational:explicit"));
    assert!(
        stdout.contains(
            "(Rational ( 7, 1 ), Rational ( 1, 2 ), Rational ( -1, 2 ), Rational ( 0, 1 ))"
        )
    );
}

#[test]
fn records_reversible_dynamic_rational_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}dynamic-rational-construction.debug"),
            &format!("{root}dynamic-rational-construction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"));
    assert!(stdout.contains("root.Rational(Int,Int);division-by-zero"));
    assert!(stdout.contains("root.Rational(Int,Int);indeterminate"));
    assert!(stdout.contains("Rational ( 1, 2 )"));
}

#[test]
fn records_reversible_inclusive_int_ranges() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}inclusive-int-ranges.debug"),
            &format!("{root}inclusive-int-ranges.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-RANGE-INCLUSIVE-001"));
    assert!(stdout.contains("TOPAL-RANGE-MEMBERSHIP-001"));
    assert!(stdout.contains("range.constructed"));
    assert!(stdout.contains("range.membership.tested"));
    assert!(stdout.contains("TOPAL-RANGE-INTERSECTION-001"));
    assert!(stdout.contains("(0 .. 10, 5 .. 10, 20 .. 10, true, false, false)"));
}

#[test]
fn records_reversible_rational_ranges() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}rational-ranges.debug"),
            &format!("{root}rational-ranges.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Int->Rational:left"));
    assert!(stdout.contains("Int->Rational:membership"));
    assert!(stdout.contains("Rational ( 0, 1 ) .. Rational ( 5, 2 )"));
    assert!(stdout.contains("TOPAL-RANGE-INTERSECTION-001"));
}

#[test]
fn records_reversible_boolean_not() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}boolean-logic.debug"),
            &format!("{root}boolean-logic.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-BOOLEAN-LOGIC-001"));
    assert!(stdout.contains("root.not(Boolean)"));
    assert!(stdout.contains("root.and(Boolean,Boolean)"));
    assert!(stdout.contains("and:eager"));
    assert!(stdout.contains("root.or(Boolean,Boolean)"));
    assert!(stdout.contains("or:eager"));
    assert!(stdout.contains("root.xor(Boolean,Boolean)"));
    assert!(stdout.contains("xor:eager"));
    assert!(stdout.contains("(false, true, true, false, false, false, true, true, true, false, false, true, true, false)"));
}

#[test]
fn records_reversible_explicit_optional_construction() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}optional-values.debug"),
            &format!("{root}optional-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-OPTIONAL-CONSTRUCT-001"));
    assert!(stdout.contains("optional.some.constructed"));
    assert!(stdout.contains("optional.none.constructed"));
    assert!(stdout.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001"));
    assert!(stdout.contains("preserve"));
    assert!(stdout.contains("absent"));
    assert!(stdout.contains("TOPAL-DECISION-OPTIONAL-001"));
    assert!(stdout.contains("optional.payload.bound"));
    assert!(stdout.contains("TOPAL-TYPE-OPTIONAL-EQUALITY-001"));
    assert!(stdout.contains(
        "(Some 42, Some \"present\", None, None, None, Some 7, None, None, \"present\", \"absent\", true, true, false, true)"
    ));
}

#[test]
fn records_reversible_string_character_indexing() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-character-at.debug"),
            &format!("{root}string-character-at.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTER-AT-001"));
    assert!(stdout.contains("string.character-at"));
    assert!(stdout.contains("Some \"👩‍🔬\""));
    assert!(stdout.contains("None, None"));
    assert!(stdout.contains("TOPAL-DECISION-OPTIONAL-001"));
    assert!(stdout.contains("TOPAL-STRING-FROM-CHARACTER-001"));
    assert!(stdout.contains("\"👩‍🔬\", \"missing\""));
}

#[test]
fn records_reversible_universal_unicode_uppercase() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-uppercase.debug"),
            &format!("{root}string-uppercase.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-UPPER-001"));
    assert!(stdout.contains("string.uppercased"));
    assert!(stdout.contains("\"STRASSE ΣΣ\""));
}

#[test]
fn records_reversible_universal_unicode_lowercase() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-lowercase.debug"),
            &format!("{root}string-lowercase.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-LOWER-001"));
    assert!(stdout.contains("string.lowercased"));
    assert!(stdout.contains("\"i\u{307}ς\""));
}

#[test]
fn records_reversible_full_unicode_case_folding() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-case-fold.debug"),
            &format!("{root}string-case-fold.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CASE-FOLD-001"));
    assert!(stdout.contains("string.case-folded"));
    assert!(stdout.contains("\"strasse σσ\""));
}

#[test]
fn records_reversible_canonical_string_equality() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-canonical-equality.debug"),
            &format!("{root}string-canonical-equality.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CANONICAL-EQUALITY-001"));
    assert!(stdout.contains("string.canonical-equality.compared"));
    assert!(stdout.contains("(false, true, false)"));
}

#[test]
fn records_reversible_character_traversal_collection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-character-traversal.debug"),
            &format!("{root}string-character-traversal.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-COLLECT-001"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("\"a\u{301}👩‍🔬🇸🇪\""));
}

#[test]
fn records_reversible_character_generator_foreach() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-character-foreach.debug"),
            &format!("{root}string-character-foreach.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-FOREACH-001"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("generator.resumed"));
    assert!(stdout.contains("generator.returned"));
}

#[test]
fn records_reversible_named_character_generator_consumption() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-named-character-generator.debug"),
            &format!("{root}string-named-character-generator.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-GENERATOR-001"));
    assert!(stdout.contains("generator.started"));
    assert!(stdout.contains("generator.consumed"));
}

#[test]
fn records_reversible_returned_character_generator() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-character-generator-result.debug"),
            &format!("{root}string-character-generator-result.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-RESULT-001"));
    assert!(stdout.contains("function.returned"));
    assert!(stdout.contains("generator.consumed"));
}

#[test]
fn records_reversible_generator_parameter_transfer() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-character-generator-parameter.debug"),
            &format!("{root}string-character-generator-parameter.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-PARAMETER-001"));
    assert!(stdout.contains("generator.parameter.transferred"));
    assert!(stdout.contains("generator.consumed"));
}

#[test]
fn records_reversible_abandoned_generator_close() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}string-character-generator-close.debug"),
            &format!("{root}string-character-generator-close.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-STRING-CHARACTERS-CLOSE-001"));
    assert!(stdout.contains("generator.closed"));
    assert!(stdout.contains("domain=root;code=generator-closed;generator=root.characters"));
}

#[test]
fn records_reversible_generator_error_code_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}generator-error-codes.debug"),
            &format!("{root}generator-error-codes.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-ERROR-CODE-001"));
    assert!(stdout.contains("generator-closed"));
}

#[test]
fn records_custom_multiple_yield_generator_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-multiple-yield-generator.debug"),
            &format!("{root}custom-multiple-yield-generator.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-DECLARATION-001"));
    assert!(stdout.contains("generator.declared"));
    assert!(stdout.contains("generator.started"));
    assert!(stdout.contains("generator.yielded"));
    assert_eq!(stdout.matches("generator.yielded").count(), 2);
    assert_eq!(stdout.matches("generator.resumed").count(), 2);
}

#[test]
fn records_custom_generator_local_binding_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-local-binding.debug"),
            &format!("{root}custom-generator-local-binding.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("binding.created"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("TOPAL-GENERATOR-FOREACH-001"));
}

#[test]
fn records_generator_return_before_yield_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-early-return.debug"),
            &format!("{root}custom-generator-early-return.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("generator.yielded"));
    assert!(stdout.contains("generator.returned"));
    assert!(stdout.contains("TOPAL-GENERATOR-EARLY-RETURN-001"));
}

#[test]
fn records_distinct_generator_final_character_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-final-character.debug"),
            &format!("{root}custom-generator-final-character.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
    assert!(stdout.contains("generator.yielded"));
    assert!(stdout.contains("\"R\""));
}

#[test]
fn records_custom_generator_suspension_order_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-suspension.debug"),
            &format!("{root}custom-generator-suspension.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("generator.suspended").count(), 2);
    assert!(stdout.contains("TOPAL-GENERATOR-SUSPEND-001"));
}

#[test]
fn records_unit_resume_binding_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-resume-binding.debug"),
            &format!("{root}custom-generator-resume-binding.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-RESUME-BINDING-001"));
    assert!(stdout.contains("generator.resume.bound"));
}

#[test]
fn records_abandoned_custom_generator_close_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-close.debug"),
            &format!("{root}custom-generator-close.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-001"));
    assert!(stdout.contains("domain=root;code=generator-closed;generator=root.pause-once"));
}

#[test]
fn records_custom_generator_close_handler_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-close-handler.debug"),
            &format!("{root}custom-generator-close-handler.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-HANDLER-001"));
    assert!(stdout.contains("generator.close.bound"));
    assert!(stdout.contains("decision.rule.selected"));
}

#[test]
fn records_qualified_generator_close_code_match_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-close-code-pattern.debug"),
            &format!("{root}custom-generator-close-code-pattern.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001"));
    assert!(stdout.contains("generator.error.code.matched"));
}

#[test]
fn records_custom_generator_function_result_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-function-result.debug"),
            &format!("{root}custom-generator-function-result.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"));
    assert!(stdout.contains("generator.function.returned"));
    assert!(stdout.contains("generator.yielded"));
}

#[test]
fn records_custom_generator_function_parameter_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-function-parameter.debug"),
            &format!("{root}custom-generator-function-parameter.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(stdout.contains("generator.parameter.transferred"));
    assert!(stdout.contains("generator.yielded"));
}

#[test]
fn records_unconsumed_custom_generator_parameter_close_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-parameter-close.debug"),
            &format!("{root}custom-generator-parameter-close.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(stdout.contains("TOPAL-GENERATOR-CLOSE-001"));
    assert!(stdout.contains("generator.closed"));
}

#[test]
fn records_character_returning_generator_parameter_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-character-return-parameter.debug"),
            &format!("{root}custom-generator-character-return-parameter.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(stdout.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
    assert!(stdout.contains("\"R\""));
}

#[test]
fn records_character_returning_generator_function_result_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-character-return-result.debug"),
            &format!("{root}custom-generator-character-return-result.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"));
    assert!(stdout.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
    assert!(stdout.contains("\"R\""));
}

#[test]
fn records_custom_generator_string_input_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-string-input.debug"),
            &format!("{root}custom-generator-string-input.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.empty?(String)"));
    assert!(stdout.contains("generator.suspended"));
}

#[test]
fn records_custom_string_yields_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-string-yield.debug"),
            &format!("{root}custom-generator-string-yield.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("generator.yielded").count(), 2);
    assert_eq!(stdout.matches("generator.resumed").count(), 2);
}

#[test]
fn records_distinct_generator_final_string_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-string-return.debug"),
            &format!("{root}custom-generator-string-return.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator String Unit String"));
    assert!(stdout.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
    assert!(stdout.contains("\"done\""));
}

#[test]
fn records_discarded_computation_between_yields_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-discard-between-yields.debug"),
            &format!("{root}custom-generator-discard-between-yields.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("string.empty.tested"));
    assert_eq!(stdout.matches("generator.suspended").count(), 2);
}

#[test]
fn records_explicit_generator_return_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-explicit-return.debug"),
            &format!("{root}custom-generator-explicit-return.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-GENERATOR-EXPLICIT-RETURN-001"));
    assert!(!stdout.contains("generator.yielded"));
    assert!(stdout.contains("\"done\""));
}

#[test]
fn records_explicit_return_after_generator_resumption_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-return-after-yield.debug"),
            &format!("{root}custom-generator-return-after-yield.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let resumed = stdout.find("generator.resumed").unwrap();
    let returned = stdout.find("generator.return.explicit").unwrap();
    assert!(resumed < returned);
    assert!(stdout.contains("\"done\""));
}

#[test]
fn records_boolean_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-boolean-values.debug"),
            &format!("{root}custom-generator-boolean-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Boolean Unit Boolean"));
    assert!(stdout.contains("generator.suspended"));
    assert!(stdout.contains("false"));
}

#[test]
fn records_int_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-int-values.debug"),
            &format!("{root}custom-generator-int-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Int Unit Int"));
    assert!(stdout.contains("1000000000000000000000000000000"));
}

#[test]
fn records_rational_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-rational-values.debug"),
            &format!("{root}custom-generator-rational-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Rational Unit Rational"));
    assert!(stdout.contains("Rational ( 2, 3 )"));
}

#[test]
fn records_unit_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-unit-values.debug"),
            &format!("{root}custom-generator-unit-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Unit Unit Unit"));
    assert!(stdout.contains("generator.suspended"));
}

#[test]
fn records_optional_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-optional-values.debug"),
            &format!("{root}custom-generator-optional-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Optional Int Unit Optional Int"));
    assert!(stdout.contains("Some 7"));
    assert!(stdout.contains("None"));
}

#[test]
fn records_range_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-range-values.debug"),
            &format!("{root}custom-generator-range-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Range Int Unit Range Int"));
    assert!(stdout.contains("5 .. 10"));
}

#[test]
fn records_nat_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-nat-values.debug"),
            &format!("{root}custom-generator-nat-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Nat Unit Nat"));
    assert!(stdout.contains('8'));
}

#[test]
fn records_enum_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-enum-values.debug"),
            &format!("{root}custom-generator-enum-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Choice Unit Choice"));
    assert!(stdout.contains("First"));
    assert!(stdout.contains("Second"));
}

#[test]
fn records_product_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-product-values.debug"),
            &format!("{root}custom-generator-product-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator (Int, String) Unit (Int, String)"));
    assert!(stdout.contains("(8, \"done\")"));
}

#[test]
fn records_result_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-result-values.debug"),
            &format!("{root}custom-generator-result-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("division-by-zero"));
    assert!(stdout.contains("result.error.constructed"));
}

#[test]
fn records_comparison_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-comparison-values.debug"),
            &format!("{root}custom-generator-comparison-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Comparison Unit Comparison"));
    assert!(stdout.contains("Less"));
    assert!(stdout.contains("Greater"));
}

#[test]
fn records_nested_optional_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-nested-optional-values.debug"),
            &format!("{root}custom-generator-nested-optional-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Optional (Int, String) Unit Optional (Int, String)"));
    assert!(stdout.contains("Some (8, \"done\")"));
}

#[test]
fn records_nested_result_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-nested-result-values.debug"),
            &format!("{root}custom-generator-nested-result-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Result ((Int, String), lang arithmetic ArithmeticErrorCode)"));
    assert!(stdout.contains("(8, \"done\")"));
}

#[test]
fn records_nested_absent_optional_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-nested-none-values.debug"),
            &format!("{root}custom-generator-nested-none-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("optional.none.constructed"));
    assert!(stdout.contains("(Int, String)"));
}

#[test]
fn records_recursive_nominal_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-recursive-nominal-values.debug"),
            &format!("{root}custom-generator-recursive-nominal-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Optional Choice"));
    assert!(stdout.contains("Result (Choice, lang arithmetic ArithmeticErrorCode)"));
    assert!(stdout.contains("(Some Second, Second)"));
}

#[test]
fn records_generator_final_decision_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-final-decision.debug"),
            &format!("{root}custom-generator-final-decision.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let resumed = stdout.find("generator.resumed").unwrap();
    let selected = stdout.find("decision.rule.selected").unwrap();
    let returned = stdout.find("generator.returned").unwrap();
    assert!(resumed < selected && selected < returned);
    assert!(stdout.contains("\"accepted\""));
}

#[test]
fn records_generator_local_function_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-local-function.debug"),
            &format!("{root}custom-generator-local-function.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let declared_enum = stdout.find("enum.declared").unwrap();
    let resumed = stdout.find("generator.resumed").unwrap();
    let entered = stdout.rfind("function.entered").unwrap();
    assert!(declared_enum < resumed && resumed < entered);
    assert!(stdout.contains("\"accepted\""));
}

#[test]
fn records_generator_local_close_handler_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-local-close-handler.debug"),
            &format!("{root}custom-generator-local-close-handler.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let close_bound = stdout.find("generator.close.bound").unwrap();
    let entered = stdout.rfind("function.entered").unwrap();
    let closed = stdout.find("generator.closed").unwrap();
    assert!(close_bound < entered && entered < closed);
    assert!(stdout.contains("CloseChoice"));
}

#[test]
fn records_generator_overload_selection_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-overloads.debug"),
            &format!("{root}custom-generator-overloads.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("generator.selected").count(), 2);
    assert!(stdout.contains("Int, String"));
    assert_eq!(stdout.matches("generator.foreach.result.bound").count(), 2);
    assert!(stdout.contains("(\"unary\", \"binary\")"));
}

#[test]
fn records_generic_generator_function_boundaries_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-generic-function-boundaries.debug"),
            &format!("{root}custom-generator-generic-function-boundaries.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Int Unit String"));
    assert!(stdout.contains("generator.function.returned"));
    assert!(stdout.contains("generator.parameter.transferred"));
    assert!(stdout.contains("\"done\""));
}

#[test]
fn records_compound_generator_function_boundaries_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-compound-function-boundaries.debug"),
            &format!("{root}custom-generator-compound-function-boundaries.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator (Int, String) Unit (Int, String)"));
    assert!(stdout.contains("(8, \"done\")"));
}

#[test]
fn records_nested_generator_function_boundaries_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-nested-function-boundaries.debug"),
            &format!("{root}custom-generator-nested-function-boundaries.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator Optional (Int, String) Unit Result"));
    assert!(stdout.contains("(8, \"done\")"));
}

#[test]
fn records_list_generator_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-list-values.debug"),
            &format!("{root}custom-generator-list-values.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generator List Int Unit List Int"));
    assert!(stdout.contains("TOPAL-LIST-APPEND-001"));
    assert!(stdout.contains("Entry ( 7, Entry ( 9, Empty ) )"));
}

#[test]
fn records_yield_after_custom_close_failure_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}custom-generator-yield-after-close.debug"),
            &format!("{root}custom-generator-yield-after-close.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("E-GENERATOR-YIELD-AFTER-CLOSE"));
    assert!(stdout.contains("generator.close.bound"));
}

#[test]
fn records_consumed_generator_failure_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}generator-consumed.debug"),
            &format!("{root}generator-consumed.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("E-GENERATOR-CONSUMED"));
    assert!(stdout.contains("generator `generated` was already consumed"));
    assert!(stdout.contains("generator.consumed"));
}

#[test]
fn records_reversible_nested_function_call_order() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-call-chain.debug"),
            &format!("{root}function-call-chain.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let outer_entry = stdout
        .find("function.entered [TOPAL-FUNCTION-ORDINARY-001] answer")
        .unwrap();
    let inner_entry = stdout
        .find("function.entered [TOPAL-FUNCTION-ORDINARY-001] increment")
        .unwrap();
    let inner_return = stdout
        .find("function.returned [TOPAL-FUNCTION-ORDINARY-001] increment")
        .unwrap();
    let outer_return = stdout
        .find("function.returned [TOPAL-FUNCTION-ORDINARY-001] answer")
        .unwrap();
    assert!(outer_entry < inner_entry && inner_entry < inner_return && inner_return < outer_return);
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn records_reversible_function_local_shadowing() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-local-shadowing.debug"),
            &format!("{root}function-local-shadowing.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let entered = stdout.find("function.entered").unwrap();
    let local = stdout[entered..]
        .find("binding.created [TOPAL-SYN-BIND-001] value")
        .unwrap()
        + entered;
    let returned = stdout.find("function.returned").unwrap();
    assert!(entered < local && local < returned);
    assert!(stdout.contains("\n(42, 40)\n"));
}

#[test]
fn records_reversible_function_overload_reasons() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}function-overloads.debug"),
            &format!("{root}function-overloads.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let integer = stdout
        .find("function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] describe (Int)")
        .unwrap();
    let string = stdout
        .find("function.overload.selected [TOPAL-FUNCTION-OVERLOAD-001] describe (String)")
        .unwrap();
    assert!(integer < string);
    assert!(stdout.contains("\n(\"integer\", \"Topal\")\n"));
}

#[test]
fn records_reversible_boolean_decision_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}boolean-decision.debug"),
            &format!("{root}boolean-decision.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first = stdout
        .find("decision.rule.selected [TOPAL-DECISION-BOOLEAN-001] rule=0")
        .unwrap();
    let fallback = stdout
        .find("decision.rule.selected [TOPAL-DECISION-BOOLEAN-001] rule=1")
        .unwrap();
    assert!(first < fallback);
    assert!(stdout.contains("\n(42, 0)\n"));
}

#[test]
fn records_reversible_exhaustive_boolean_decision_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}exhaustive-boolean-decision.debug"),
            &format!("{root}exhaustive-boolean-decision.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let truth = stdout
        .find("decision.rule.selected [TOPAL-DECISION-BOOLEAN-001] rule=0")
        .unwrap();
    let falsehood = stdout
        .find("decision.rule.selected [TOPAL-DECISION-BOOLEAN-001] rule=1")
        .unwrap();
    assert!(truth < falsehood);
    assert!(stdout.contains("\n(\"enabled\", \"disabled\")\n"));
}

#[test]
fn records_reversible_call_to_later_function_declaration() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}forward-function-declaration.debug"),
            &format!("{root}forward-function-declaration.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let render = stdout
        .find("function.entered [TOPAL-FUNCTION-ORDINARY-001] render")
        .unwrap();
    let decorate = stdout
        .find("function.entered [TOPAL-FUNCTION-ORDINARY-001] decorate")
        .unwrap();
    assert!(render < decorate);
    assert!(stdout.contains("\n\"[Topal]\"\n"));
}

#[test]
fn records_reversible_mutual_int_recursion_proof() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}mutual-int-recursion.debug"),
            &format!("{root}mutual-int-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let candidate = stdout.find("function.recursion.edge.candidate").unwrap();
    let proof = stdout.find("function.recursion.cycle.proven").unwrap();
    let descent = stdout.find("function.recursion.descended").unwrap();
    assert!(candidate < proof && proof < descent);
    assert!(stdout.contains("\n(true, false)\n"));
}

#[test]
fn records_reversible_mutual_increasing_int_recursion_proof() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}mutual-increasing-int-recursion.debug"),
            &format!("{root}mutual-increasing-int-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001"));
    assert!(stdout.contains("function.recursion.cycle.proven"));
    assert!(stdout.contains("\n(true, false)\n"));
}

#[test]
fn records_reversible_calls_between_distinct_overloads() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}overload-recursion-identity.debug"),
            &format!("{root}overload-recursion-identity.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let string = stdout.find("describe (String)").unwrap();
    let integer = stdout.find("describe (Int)").unwrap();
    assert!(string < integer);
    assert!(stdout.contains("\n\"integer:Topal\"\n"));
}

#[test]
fn records_reversible_positive_literal_recursion_steps() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}positive-recursion-steps.debug"),
            &format!("{root}positive-recursion-steps.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-INT-001"));
    assert!(stdout.contains("TOPAL-FUNCTION-RECURSION-INT-INCREASING-001"));
    assert!(stdout.contains("\n(3, 3)\n"));
}

#[test]
fn records_reversible_multiple_recursive_calls() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}multiple-recursive-calls.debug"),
            &format!("{root}multiple-recursive-calls.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.matches("function.recursion.descended").count() > 2);
    assert!(stdout.contains("\n5\n"));
}

#[test]
fn records_reversible_multiple_calls_on_a_mutual_edge() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}mutual-multiple-recursive-calls.debug"),
            &format!("{root}mutual-multiple-recursive-calls.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.matches("function.recursion.descended").count() > 1);
    assert!(stdout.contains("\n3\n"));
}

#[test]
fn records_reversible_rational_natural_exponentiation() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}rational-exponentiation.debug"),
            &format!("{root}rational-exponentiation.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("root.^(Rational,Nat)"));
    assert!(stdout.contains("TOPAL-NUM-RAT-POW-001"));
    assert!(stdout.contains("\n(Rational ( 27, 8 ), Rational ( 1, 1 ))\n"));
}

#[test]
fn records_reversible_comparison_decision_selection() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}comparison-decision.debug"),
            &format!("{root}comparison-decision.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("decision.rule.selected [TOPAL-DECISION-COMPARISON-001] rule=0"));
    assert!(stdout.contains("decision.rule.selected [TOPAL-DECISION-COMPARISON-001] rule=1"));
    assert!(stdout.contains("\n(42, 50)\n"));
}

#[test]
fn records_reversible_decreasing_int_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}decreasing-int-recursion.debug"),
            &format!("{root}decreasing-int-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let proven = stdout.find("function.recursion.proven").unwrap();
    let descended = stdout.find("function.recursion.descended").unwrap();
    let nested = stdout[descended..].find("function.entered").unwrap() + descended;
    assert!(proven < descended && descended < nested);
    assert_eq!(stdout.matches("function.recursion.descended").count(), 5);
    assert!(stdout.contains("\n15\n"));
}

#[test]
fn records_reversible_increasing_int_recursion() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}increasing-int-recursion.debug"),
            &format!("{root}increasing-int-recursion.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("function.recursion.proven [TOPAL-FUNCTION-RECURSION-INT-INCREASING-001]")
    );
    assert_eq!(stdout.matches("function.recursion.descended").count(), 5);
    assert!(stdout.contains("\n5\n"));
}

#[test]
fn records_reversible_comparison_operand_expression() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}decision-operand-expression.debug"),
            &format!("{root}decision-operand-expression.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let addition = stdout.find("root.+(Int,Int)").unwrap();
    let comparison = stdout.find("root.<(TotalOrder,TotalOrder)").unwrap();
    assert!(addition < comparison);
    assert!(stdout.contains("\n(true, false)\n"));
}

#[test]
fn records_reversible_nested_lexical_function() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nested-function.debug"),
            &format!("{root}nested-function.t"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let outer = stdout
        .find("function.entered [TOPAL-FUNCTION-ORDINARY-001] answer")
        .unwrap();
    let declared = stdout
        .find("function.declared [TOPAL-FUNCTION-ORDINARY-001] add-input")
        .unwrap();
    let nested = stdout
        .find("function.entered [TOPAL-FUNCTION-ORDINARY-001] add-input")
        .unwrap();
    assert!(outer < declared && declared < nested);
    assert!(stdout.contains("\n42\n"));
}

#[test]
fn evaluates_inspection_expressions_without_mutating_history() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}expression-inspection.debug"),
            &format!("{root}basic-history.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\n40\n"));
    assert!(stdout.contains("\n42\n"));
    assert!(stdout.contains("\n(40, 42)\n"));
    assert!(stdout.contains("error[E-UNBOUND-NAME]: name is not bound"));
}

#[test]
fn script_mode_reports_command_file_errors() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.t"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--script", "missing-debug-commands", source])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot read command script missing-debug-commands")
    );
}

#[test]
fn script_mode_rejects_unknown_commands_with_a_line_diagnostic() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/debugger/basic-history.t"
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args(["--script", "-", source])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"step\nnot-a-command\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr).unwrap().contains(
            "<stdin>:2: error[D-UNKNOWN-COMMAND]: unknown debugger command `not-a-command`"
        )
    );
}

#[test]
fn reports_debuggee_diagnostics_with_its_source_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .arg("missing-topal-debugger-example.t")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot read missing-topal-debugger-example.t")
    );
}

#[test]
fn records_list_construction_and_decomposition_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}lists.debug"),
            &format!("{root}lists.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-LIST-CONSTRUCT-001"));
    assert!(stdout.contains("TOPAL-DECISION-LIST-001"));
    assert!(stdout.contains("TOPAL-LIST-CONCAT-001"));
    assert!(stdout.contains("TOPAL-LIST-ENTRY-COUNT-001"));
    assert!(stdout.contains("TOPAL-LIST-EMPTY-001"));
    assert!(stdout.contains("TOPAL-LIST-ONE-001"));
    assert!(stdout.contains("TOPAL-LIST-UNCONS-001"));
    assert!(stdout.contains("TOPAL-LIST-FIRST-001"));
    assert!(stdout.contains("TOPAL-LIST-REST-001"));
    assert!(stdout.contains("TOPAL-LIST-REVERSE-001"));
    assert!(stdout.contains("Some (6, Entry ( 7"));
}

#[test]
fn records_recursive_list_classifiers_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}nested-lists.debug"),
            &format!("{root}nested-lists.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("List List (Int, String)"));
    assert!(stdout.contains("Some Entry ( (7, \"seven\"), Empty )"));
}

#[test]
fn records_list_containment_laws_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-containment.debug"),
            &format!("{root}list-containment.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-LIST-CONTAINS-ENTRY-001"));
    assert!(stdout.contains("TOPAL-LIST-CONTAINS-SEQUENCE-001"));
    assert!(stdout.contains("TOPAL-LIST-CONTAINS-SUBSEQUENCE-001"));
    assert!(stdout.contains("(true, false, true, true, false, false)"));
}

#[test]
fn records_list_value_removal_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-removal.debug"),
            &format!("{root}list-removal.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-LIST-REMOVE-FIRST-001"));
    assert!(stdout.contains("TOPAL-LIST-REMOVE-ALL-001"));
}

#[test]
fn records_contextual_anonymous_list_functions_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}anonymous-list-functions.debug"),
            &format!("{root}anonymous-list-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(stdout.contains("Entry ( 2, Entry ( 4, Entry ( 6"));
}

#[test]
fn records_list_sequence_operations_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}list-sequence-operations.debug"),
            &format!("{root}list-sequence-operations.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-LIST-INSERT-AT-001"));
    assert!(stdout.contains("TOPAL-COLLECTION-ENTRIES-001"));
}

#[test]
fn records_fundamental_container_collection_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}fundamental-containers.debug"),
            &format!("{root}fundamental-containers.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-ARRAY-COLLECT-001"));
    assert!(stdout.contains("TOPAL-MAP-COLLECT-001"));
}

#[test]
fn records_payload_union_decisions_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}unions-and-recursive-products.debug"),
            &format!("{root}unions-and-recursive-products.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-UNION-001"));
    assert!(stdout.contains("TOPAL-DECISION-UNION-001"));
}

#[test]
fn records_constraint_evidence_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}constraints-and-derived-capabilities.debug"),
            &format!("{root}constraints-and-derived-capabilities.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-CONSTRAINT-VALIDATE-001"));
    assert!(stdout.contains("constraint->base"));
}

#[test]
fn records_optional_result_composition_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}optional-result-composition.debug"),
            &format!("{root}optional-result-composition.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-RESULT-PROJECT-001"));
    assert!(stdout.contains("TOPAL-ERROR-FIELD-001"));
}

#[test]
fn records_modular_arithmetic_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}modular-numbers.debug"),
            &format!("{root}modular-numbers.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-NUM-MODULAR-REDUCE-001"));
    assert!(stdout.contains("TOPAL-NUM-MODULAR-ARITHMETIC-001"));
}

#[test]
fn records_range_selection_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}range-selection.debug"),
            &format!("{root}range-selection.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-RANGE-VALUE-SELECTION-001"));
    assert!(stdout.contains("TOPAL-RANGE-INDEX-SELECTION-001"));
}

#[test]
fn records_completion_evidence_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}completed-evidence.debug"),
            &format!("{root}completed-evidence.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-EXEC-COMPLETED-001"));
    assert!(stdout.contains("Completed"));
}

#[test]
fn records_immutable_reconstruction_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}record-reconstruction.debug"),
            &format!("{root}record-reconstruction.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-TYPE-RECONSTRUCT-001"));
    assert!(stdout.contains("age is 37"));
}

#[test]
fn records_bound_anonymous_function_values_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}bound-anonymous-functions.debug"),
            &format!("{root}bound-anonymous-functions.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(stdout.contains("<anonymous fn/1>"));
}

#[test]
fn records_direct_anonymous_function_application_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}anonymous-function-application.debug"),
            &format!("{root}anonymous-function-application.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function.anonymous.called"));
    assert!(stdout.contains("<anonymous fn/2>"));
}

#[test]
fn records_short_circuiting_traversal_control_reversibly() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugger/");
    let output = Command::new(env!("CARGO_BIN_EXE_topal-debug"))
        .args([
            "--script",
            &format!("{root}traversal-control.debug"),
            &format!("{root}traversal-control.t"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("TOPAL-EXEC-TRAVERSAL-CONTROL-001"));
    assert!(stdout.contains("traversal.finished"));
}
