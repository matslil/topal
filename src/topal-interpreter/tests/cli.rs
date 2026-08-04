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
fn version_exposes_the_language_context_unicode_version() {
    let output = run(&["--version"], "");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "topal 0.1.0 (language design-0; Unicode 17.0.0)\n"
    );
}

#[test]
fn script_rejects_non_nfc_identifier_without_rewriting_it() {
    let output = run(&[], "e\u{301} is 1\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NON-NFC-TOKEN")
    );
}

#[test]
fn script_rejects_non_nfc_literal_tag() {
    let output = run(&[], "tag\u{301}\"value\"tag\u{301}\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NON-NFC-TOKEN")
    );
}

#[test]
fn script_preserves_non_nfc_string_contents() {
    let output = run(&[], "\"e\u{301}\"\n");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "\"e\u{301}\"\n");
}

#[test]
fn test_trace_records_the_pinned_unicode_context() {
    let output = run(&["--test"], "1\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"context.selected\""));
    assert!(trace.contains("\"rule\":\"TOPAL-SYN-UNICODE-001\""));
    assert!(trace.contains("\"detail\":\"design-0;Unicode=17.0.0\""));
}

#[test]
fn script_mode_is_default() {
    let output = run(&[], "123456789012345678901234567890\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"123456789012345678901234567890\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn every_mode_evaluates_boolean_literals() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "true\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&[], "false\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"false\n");
}

#[test]
fn boolean_literal_spellings_cannot_be_rebound() {
    let output = run(&[], "true is 1\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-RESERVED-BOOLEAN-LITERAL")
    );
}

#[test]
fn test_trace_explains_boolean_literal_construction() {
    let output = run(&["--test"], "false\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"token.boolean\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-BOOLEAN-001\""));
    assert!(trace.contains("\"detail\":\"false\""));
    assert!(trace.contains("\"detail\":\"Boolean\""));
}

#[test]
fn every_mode_evaluates_fundamental_equality() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(
            arguments,
            "(1, \"same\", true, ()) = (1, \"same\", true, ())\n",
        );
        assert!(output.status.success());
        assert_eq!(output.stdout, b"true\n");
    }

    let output = run(&[], "\"e\u{301}\" = \"é\"\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"false\n");
}

#[test]
fn equality_uses_canonical_exact_conversion() {
    let output = run(&["--test"], "1 = 1.0\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"true\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    let conversion = trace.find("\"event\":\"conversion.applied\"").unwrap();
    let selection = trace.find("\"event\":\"operator.selected\"").unwrap();
    assert!(conversion < selection);
    assert!(trace.contains("\"event\":\"evaluation.equal\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-EQUALITY-001\""));
}

#[test]
fn equality_requires_a_shared_operation() {
    let output = run(&[], "true = 1\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );

    let output = run(&[], "(1,) = (1, 2)\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-NO-APPLICABLE-OVERLOAD")
    );
}

#[test]
fn every_mode_evaluates_the_unit_product() {
    let script = run(&[], "()\n");
    assert!(script.status.success());
    assert_eq!(script.stdout, b"()\n");

    let interactive = run(&["--interactive"], "()\n");
    assert!(interactive.status.success());
    assert_eq!(interactive.stdout, b"()\n");

    let test = run(&["--test"], "()\n");
    assert!(test.status.success());
    assert_eq!(test.stdout, b"()\n");
    let trace = String::from_utf8(test.stderr).unwrap();
    assert!(trace.contains("\"event\":\"product.unit\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-PRODUCT-001\""));
    assert!(trace.contains("\"detail\":\"Tuple()\""));
}

#[test]
fn every_mode_evaluates_positional_products() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "(1, \"two\", ())\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"(1, \"two\", ())\n");
    }

    let grouped = run(&[], "(1)\n");
    assert!(grouped.status.success());
    assert_eq!(grouped.stdout, b"1\n");

    let singleton = run(&[], "(1,)\n");
    assert!(singleton.status.success());
    assert_eq!(singleton.stdout, b"(1,)\n");

    let traced = run(&["--test"], "(1, 2)\n");
    let trace = String::from_utf8(traced.stderr).unwrap();
    assert!(trace.contains("\"event\":\"product.tuple\""));
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-PRODUCT-001\""));
    assert!(trace.contains("\"detail\":\"fields=2\""));
}

#[test]
fn every_mode_evaluates_multiline_products() {
    let source = "(\n1,\n(\n2\n),\n3\n)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert_eq!(output.stdout, b"(1, 2, 3)\n");
    }
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
    let output = run(&["--interactive"], "1 % 2\n2\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"2\n");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNKNOWN-TOKEN")
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
    let output = run(&[], "1 % 2\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNKNOWN-TOKEN")
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

#[test]
fn all_modes_execute_exact_natural_exponentiation() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "2 ^ 100\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"1267650600228229401496703205376\n");
    }
}

#[test]
fn zero_to_zero_is_empty_product() {
    let output = run(&["--test"], "0 ^ 0\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.^(Int,Nat)\""));
    assert!(trace.contains("\"event\":\"evaluation.power\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-POW-001\""));
}

#[test]
fn exponentiation_has_no_hidden_precedence() {
    assert_eq!(run(&[], "2 + 3 ^ 2\n").stdout, b"25\n");
    assert_eq!(run(&[], "2 + (3 ^ 2)\n").stdout, b"11\n");
}

#[test]
fn negative_exponent_is_rejected_for_int_overload() {
    let output = run(&["--test"], "2 ^ -1\n");
    assert!(!output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"obligation.refuted\""));
    assert!(trace.contains("\"detail\":\"exponent.finite-nat\""));
    assert!(!trace.contains("root.^(Int,Nat)"));
    assert!(trace.contains("E-NO-APPLICABLE-OVERLOAD at 1:5"));
}

#[test]
fn all_modes_construct_exact_rational_literals() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "-6.022e-24\n");
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            b"Rational ( -3011, 500000000000000000000000000 )\n"
        );
    }
}

#[test]
fn rational_literal_trace_retains_exact_lexeme() {
    let output = run(&["--test"], "1_000.000_125\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"token.rational\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-RATIONAL-LITERAL-001\""));
    assert!(trace.contains("\"detail\":\"1_000.000_125\""));
    assert!(trace.contains("\"detail\":\"Rational\""));
}

#[test]
fn all_modes_execute_exact_rational_arithmetic_left_to_right() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "0.5 + 0.25 * 2.0\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Rational ( 3, 2 )\n");
    }
    let grouped = run(&[], "0.5 + (0.25 * 2.0)\n");
    assert_eq!(grouped.stdout, b"Rational ( 1, 1 )\n");
}

#[test]
fn rational_arithmetic_trace_identifies_overload_and_rule() {
    let output = run(&["--test"], "1.5 / 0.25\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root./(Rational,Rational)\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-RAT-DIV-001\""));
    assert!(trace.contains("\"event\":\"obligation.proved\""));
}

#[test]
fn all_modes_execute_mixed_exact_arithmetic() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "1 + 0.5 * 2\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Rational ( 3, 1 )\n");
    }
}

#[test]
fn conversion_trace_precedes_rational_overload_selection() {
    let output = run(&["--test"], "1 + 0.5\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    let conversion = trace.find("\"event\":\"conversion.applied\"").unwrap();
    let selection = trace
        .find("\"detail\":\"root.+(Rational,Rational)\"")
        .unwrap();
    assert!(conversion < selection);
    assert!(trace.contains("\"rule\":\"TOPAL-TYPE-CONVERT-001\""));
    assert!(trace.contains("\"detail\":\"Int->Rational:left\""));
}

#[test]
fn mixed_power_does_not_promote_exponent_contract() {
    let output = run(&["--test"], "2.0 ^ 2\n");
    assert!(!output.status.success());
    assert!(
        !String::from_utf8(output.stderr)
            .unwrap()
            .contains("conversion.applied")
    );
}

#[test]
fn all_modes_preserve_literal_string_contents() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "text\"He said \"hello\". {value} \\n\"text\n");
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            b"text\"He said \"hello\". {value} \\n\"text\n"
        );
    }
}

#[test]
fn interactive_mode_accumulates_multiline_string() {
    let output = run(&["--interactive"], "\"first\nsecond\"\n");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\"first\nsecond\"\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn string_trace_retains_complete_tagged_lexeme() {
    let output = run(&["--test"], "tag\"a \"quote\"\"tag\n");
    assert!(output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"token.string\""));
    assert!(trace.contains("\"rule\":\"TOPAL-SYN-STRING-001\""));
    assert!(trace.contains("\"detail\":\"tag\\\"a "));
    assert!(trace.contains("\"detail\":\"String\""));
}

#[test]
fn unterminated_string_is_rejected_recoverably() {
    let output = run(&[], "tag\"unfinished\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("E-UNTERMINATED-STRING")
    );
}

#[test]
fn rational_zero_division_trace_refutes_obligation() {
    let output = run(&["--test"], "1.0 / 0.0\n");
    assert!(!output.status.success());
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"event\":\"obligation.refuted\""));
    assert!(!trace.contains("root./(Rational,Rational)"));
}
