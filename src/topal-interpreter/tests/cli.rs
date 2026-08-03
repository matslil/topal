//! Functional conformance tests for TOPAL-SYN-SOURCE-001,
//! TOPAL-SYN-NUM-001, TOPAL-SYN-GRAMMAR-001, TOPAL-SYN-BIND-001,
//! TOPAL-NUM-LITERAL-001, TOPAL-NUM-ADD-001, and TOPAL-INTP-MODE-001 through
//! TOPAL-INTP-MODE-003.

use std::io::Write;
use std::process::{Command, Stdio};

fn run(arguments: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_topal"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn script_mode_is_default() {
    let output = run(&[], "123456789012345678901234567890\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"123456789012345678901234567890\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn script_mode_ignores_hashbang_launcher_line() {
    let output = run(&[], "#!/usr/bin/env topal\n42\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
}

#[test]
fn interactive_mode_evaluates_each_input() {
    let output = run(&["--interactive"], "1\n2\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1\n2\n");
}

#[test]
fn interactive_mode_recovers_after_diagnostic() {
    let output = run(&["--interactive"], "1 * 2\n2\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"2\n");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNSUPPORTED-SYNTAX")
    );
}

#[test]
fn interactive_mode_preserves_bindings() {
    let output = run(&["--interactive"], "answer is 42\nanswer\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"()\n42\n");
}

#[test]
fn test_mode_emits_stable_decisions() {
    let output = run(&["--test"], "42\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"schema\":\"topal.test-trace/1\""));
    assert!(trace.contains("\"event\":\"token.integer\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-LITERAL-001\""));
    assert!(trace.contains("\"event\":\"evaluation.result\""));
    assert!(trace.contains("\"detail\":\"Int\""));
}

#[test]
fn unsupported_syntax_is_explicit() {
    let output = run(&[], "1 * 2\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNSUPPORTED-SYNTAX")
    );
}

#[test]
fn script_executes_bindings_in_source_order() {
    let output = run(&[], "answer is 42\nanswer\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
}

#[test]
fn script_executes_arbitrary_precision_based_integer() {
    let output = run(&[], "0xFFFF_FFFF_FFFF_FFFF_FFFF\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1208925819614629174706175\n");
}

#[test]
fn test_mode_preserves_based_literal_lexeme_in_trace() {
    let output = run(&["--test"], "0b1010_1100\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"172\n");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("\"detail\":\"0b1010_1100\"")
    );
}

#[test]
fn script_rejects_discarded_expression_values() {
    let output = run(&[], "1\n2\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-DISCARDED-VALUE")
    );
}

#[test]
fn test_trace_explains_binding_decisions() {
    let output = run(&["--test"], "answer is 42\nanswer\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"binding.created\""));
    assert!(trace.contains("\"event\":\"binding.resolved\""));
    assert!(trace.contains("\"rule\":\"TOPAL-SYN-BIND-001\""));
}

#[test]
fn all_modes_execute_signed_exact_addition() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "-0x1 + 1_000\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"999\n");
    }
}

#[test]
fn test_trace_explains_exact_addition() {
    let output = run(&["--test"], "40 + 2\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"operator.selected\""));
    assert!(trace.contains("\"detail\":\"root.+(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.add\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-ADD-001\""));
}
