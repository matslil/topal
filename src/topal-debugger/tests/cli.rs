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
