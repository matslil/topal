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
    let output = run(&["--interactive"], "1 ^ 2\n2\n");
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
    let output = run(&[], "1 ^ 2\n");
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

#[test]
fn all_modes_execute_exact_subtraction() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "1_000 - 0x1 - 1\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"998\n");
    }
}

#[test]
fn test_trace_distinguishes_literal_sign_and_prefix_negation() {
    let literal = run(&["--test"], "-42\n");
    let literal_trace = String::from_utf8(literal.stderr).unwrap();
    assert!(literal_trace.contains("\"detail\":\"-42\""));
    assert!(!literal_trace.contains("evaluation.negate"));

    let prefix = run(&["--test"], "- 42\n");
    let prefix_trace = String::from_utf8(prefix.stderr).unwrap();
    assert!(prefix_trace.contains("\"detail\":\"root.-(Int)\""));
    assert!(prefix_trace.contains("\"rule\":\"TOPAL-NUM-NEG-001\""));
}

#[test]
fn test_trace_explains_binary_subtraction() {
    let output = run(&["--test"], "10 - 3\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.-(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.subtract\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-SUB-001\""));
}

#[test]
fn all_modes_execute_exact_multiplication_left_to_right() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "2 + 3 * 4\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"20\n");
    }
    let grouped = run(&[], "2 + (3 * 4)\n");
    assert!(grouped.status.success());
    assert_eq!(grouped.stdout, b"14\n");
}

#[test]
fn test_trace_explains_exact_multiplication() {
    let output = run(&["--test"], "6 * 7\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.*(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.multiply\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-MUL-001\""));
}

#[test]
fn all_modes_execute_exact_rational_division() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "-6 / -0x8\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Rational ( 3, 4 )\n");
    }
}

#[test]
fn division_retains_rational_type_for_whole_result() {
    let output = run(&[], "6 / 3\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Rational ( 2, 1 )\n");
}

#[test]
fn division_by_zero_is_rejected() {
    let output = run(&[], "1 / 0\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-DIVISION-BY-ZERO")
    );
}

#[test]
fn test_trace_explains_exact_division() {
    let output = run(&["--test"], "6 / 8\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"obligation.proved\""));
    assert!(trace.contains("\"detail\":\"divisor.nonzero\""));
    assert!(trace.contains("\"detail\":\"root./(Int,Int)\""));
    assert!(trace.contains("\"event\":\"evaluation.divide\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-DIV-001\""));
    assert!(trace.contains("\"detail\":\"Rational\""));
}

#[test]
fn test_trace_explains_zero_division_rejection() {
    let output = run(&["--test"], "1 / 0\n");
    assert!(!output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"obligation.refuted\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-DIVZERO-001\""));
    assert!(!trace.contains("root./(Int,Int)"));
    assert!(!trace.contains("evaluation.divide"));
    assert!(trace.contains("E-DIVISION-BY-ZERO at 1:5"));
}
