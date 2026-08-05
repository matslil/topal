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
