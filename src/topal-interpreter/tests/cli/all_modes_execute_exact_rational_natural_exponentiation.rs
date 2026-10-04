#[test]
fn all_modes_execute_exact_rational_natural_exponentiation() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "(1.5 ^ 3, 0.0 ^ 0)\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"(Rational ( 27, 8 ), Rational ( 1, 1 ))\n");
    }

    let output = run(&["--test"], "1.5 ^ 3\n");
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(trace.contains("\"detail\":\"root.^(Rational,Nat)\""));
    assert!(trace.contains("\"rule\":\"TOPAL-NUM-RAT-POW-001\""));
    assert!(!trace.contains("conversion.applied"));
}

#[test]
fn every_mode_executes_exact_negative_rational_exponents() {
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, "1.5 ^ -2\n");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Rational ( 4, 9 )\n");
    }
    let trace = String::from_utf8(run(&["--test"], "1.5 ^ -2\n").stderr).unwrap();
    assert!(trace.contains("root.^(Rational,Int)"));
    assert!(trace.contains("TOPAL-NUM-RAT-NEG-POW-001"));
}

#[test]
fn every_mode_returns_dynamic_negative_power_error() {
    let source = "power is fn (base : Rational, exponent : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  base ^ exponent\n0.0 power -1\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output
                .stdout
                .ends_with(b"Error ( domain is root.^(Rational,Int), code is division-by-zero )\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("result.error.constructed"));
    assert!(trace.contains("root.^(Rational,Int);division-by-zero"));
}

#[test]
fn result_errors_propagate_unchanged_across_calls() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nouter is fn () -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  1.0 divide 0.0\nouter ()\n";
    let output = run(&["--test"], source);
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        b"Error ( domain is root./(Rational,Rational), code is division-by-zero )\n"
    );
    let trace = String::from_utf8(output.stderr).unwrap();
    assert_eq!(trace.matches("result.error.constructed").count(), 1);
    assert_eq!(trace.matches("result.error.propagated").count(), 2);
    assert!(trace.contains("domain=root./(Rational,Rational);code=division-by-zero"));
}

#[test]
fn every_mode_executes_exhaustive_result_decisions() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error problem then \"error\"\n(describe 2.0, describe 0.0)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.ends_with(b"(\"ok\", \"error\")\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-DECISION-RESULT-001"));
    assert!(trace.contains("result.payload.bound"));
}

#[test]
fn every_mode_selects_structured_error_fields() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproblem is 1.0 divide 0.0\n(problem code, problem domain)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output
                .stdout
                .ends_with(b"(division-by-zero, root./(Rational,Rational))\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("error.field.selected").count(), 2);
    assert!(trace.contains("TOPAL-ERROR-FIELD-001"));
}

#[test]
fn every_mode_matches_qualified_error_codes() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n    Error problem then \"other\"\n(describe 2.0, describe 0.0)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(\"ok\", \"zero\")\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-DECISION-ERROR-CODE-001"));
    assert!(trace.contains("error.code.matched"));
}

#[test]
fn every_mode_projects_result_through_classified_binding() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproject is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  quotient : Rational is 1.0 divide denominator\n  quotient + 1.0\n(project 2.0, project 0.0)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(Rational ( 3, 2 ), Error ( domain is root./(Rational,Rational), code is division-by-zero ))\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("result.success.projected"));
    assert!(trace.contains("result.error.projected"));
}

#[test]
fn infallible_projection_diagnostic_explains_available_repairs() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nbad is fn (denominator : Rational) -> Rational\n  quotient : Rational is 1.0 divide denominator\n  quotient\nbad 0.0\n";
    let output = run(&[], source);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-RESULT-PROJECTION-INFALLIBLE]"));
    assert!(diagnostic.contains("cannot propagate a failed Result"));
    assert!(diagnostic.contains("help: change the function result to `Result (T, Codes)`"));
    assert!(diagnostic.contains("quotient : Rational is 1.0 divide denominator"));
}

#[test]
fn every_mode_accepts_exhaustive_arithmetic_code_decision() {
    let source = include_str!("../../../../examples/language/exhaustive-error-code-decisions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(\"ok\", \"zero\")\n"));
    }
}

#[test]
fn incomplete_error_code_decision_reports_missing_alternatives() {
    let source = "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n";
    let output = run(&[], source);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-INCOMPLETE-ERROR-CODE-DECISION]"));
    assert!(diagnostic.contains("out-of-range, not-representable, indeterminate"));
    assert!(diagnostic.contains("help: add each missing qualified code pattern"));
}

#[test]
fn duplicate_error_code_pattern_points_to_unreachable_case() {
    let source = "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"first\"\n    Error ( code is lang arithmetic division-by-zero ) then \"second\"\n    Error problem then \"other\"\n";
    let output = run(&[], source);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-DUPLICATE-ERROR-CODE-PATTERN]"));
    assert!(diagnostic.contains("matched more than once"));
    assert!(diagnostic.contains("help: remove the later duplicate pattern"));
}

#[test]
fn error_code_pattern_after_fallback_has_ordering_help() {
    let source = "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error problem then \"other\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n";
    let output = run(&[], source);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-UNREACHABLE-ERROR-CODE-PATTERN]"));
    assert!(diagnostic.contains("unreachable after `Error problem`"));
    assert!(diagnostic.contains("help: move qualified code patterns before"));
}

#[test]
fn rule_after_otherwise_has_ordering_help() {
    let source = "choose is fn (condition : Boolean) -> Int\n  condition\n    otherwise 0\n    true then 1\n";
    let output = run(&[], source);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-UNREACHABLE-DECISION-RULE]"));
    assert!(diagnostic.contains("unreachable after `otherwise`"));
    assert!(diagnostic.contains("help: move `otherwise` after every specific matcher"));
}

#[test]
fn every_mode_classifies_unicode_characters() {
    let source = include_str!("../../../../examples/language/character-classification.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with("(\"🙂\", \"a\u{301}\", true, true, true)\n".as_bytes())
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("string.from-character").count(), 3);
}

#[test]
fn character_classifier_diagnostic_reports_observed_count() {
    let output = run(&[], "invalid : Character is \"ab\"\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-CHARACTER-CLASSIFIER]"));
    assert!(diagnostic.contains("this String contains 2"));
    assert!(diagnostic.contains("help: use a String containing exactly one Unicode grapheme"));
}

#[test]
fn every_mode_executes_euclidean_int_modulo() {
    let source = include_str!("../../../../examples/language/int-euclidean-modulo.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(
            b"(2, 3, 2, (-4, 3), (-3, 2), Error ( domain is root.%(Int,Int), code is division-by-zero ), Error ( domain is root./%(Int,Int), code is division-by-zero ))\n"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-NUM-INT-MODULO-001"));
    assert!(trace.contains("root.%(Int,Int);division-by-zero"));
    assert!(trace.contains("TOPAL-NUM-INT-QUOTIENT-MODULO-001"));
    assert!(trace.contains("root./%(Int,Int);division-by-zero"));
}

#[test]
fn every_mode_executes_finite_exact_division_and_comparison() {
    let source =
        include_str!("../../../../examples/language/finite-exact-division-and-comparison.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(
            b"(2, 3, 2, (-4, 3), (-3, 2), Rational ( 12345678901234567890123456789, 1 ), 52, Less, Greater, true, Rational ( 5, 2 ), Rational ( -5, 2 ))\n"
        ));
    }
}

#[test]
fn every_mode_executes_exact_numeric_absolute() {
    let source = include_str!("../../../../examples/language/exact-numeric-absolute.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(42, 42, Rational ( 5, 4 ), Rational ( 5, 4 ))\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.absolute(Int)"));
    assert!(trace.contains("root.absolute(Rational)"));
}

#[test]
fn every_mode_executes_named_exact_numeric_negation() {
    let source = include_str!("../../../../examples/language/exact-numeric-negate.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(-42, 42, Rational ( -5, 4 ), Rational ( 5, 4 ))\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.negate(Int)"));
    assert!(trace.contains("root.negate(Rational)"));
}

#[test]
fn every_mode_constructs_exact_numeric_zero() {
    let source = include_str!("../../../../examples/language/exact-numeric-zero.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(0, 0, Rational ( 0, 1 ), 1, 1, Rational ( 1, 1 ))\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.zero(Int)"));
    assert!(trace.contains("root.zero(Rational)"));
    assert!(trace.contains("root.one(Int)"));
    assert!(trace.contains("root.zero(Nat)"));
    assert!(trace.contains("root.one(Nat)"));
    assert!(trace.contains("root.one(Rational)"));
}

#[test]
fn every_mode_executes_exact_three_way_comparison() {
    let source = include_str!("../../../../examples/language/exact-three-way-comparison.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(Less, Equal, Greater, Less, \"less\", \"equal\", \"greater\")\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-NUM-THREE-WAY-COMPARE-001"));
    assert!(trace.contains("Int->Rational:left"));
    assert!(trace.contains("TOPAL-DECISION-ENUM-001"));
}

#[test]
fn every_mode_narrows_closed_exact_rational_to_int() {
    let source = include_str!("../../../../examples/language/exact-rational-int-narrowing.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(50, -3)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("TOPAL-NUM-RATIONAL-INT-EXACT-001").count(), 2);

    let diagnostic = run(&[], "half : Int is 1 / 2\n");
    assert!(!diagnostic.status.success());
    let diagnostic = String::from_utf8(diagnostic.stderr).unwrap();
    assert!(diagnostic.contains("error[E-RATIONAL-NOT-EXACT-INT]"));
    assert!(diagnostic.contains("denominator 2"));
    assert!(diagnostic.contains("help: use an exactly divisible expression"));
}

#[test]
fn every_mode_validates_dynamic_rational_to_int() {
    let source = include_str!("../../../../examples/language/dynamic-rational-int-validation.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(
            b"(50, Error ( domain is root.Int(Rational), code is not-representable ))\n"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001"));
    assert!(trace.contains("Rational->Int:validated"));
    assert!(trace.contains("root.Int(Rational);not-representable"));
    assert!(trace.contains("result.error.projected"));
}

#[test]
fn every_mode_executes_checked_int_construction() {
    let source = include_str!("../../../../examples/language/int-checked-construction.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(
            b"(7, 6, Error ( domain is root.Int(Rational), code is not-representable ))\n"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("TOPAL-NUM-INT-CONSTRUCT-001").count(), 3);
    assert!(trace.contains("Int->Int:identity"));
    assert!(trace.contains("Rational->Int:exact"));
    assert!(trace.contains("root.Int(Rational);not-representable"));
}

#[test]
fn every_mode_executes_checked_nat_construction() {
    let source = include_str!("../../../../examples/language/nat-checked-construction.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(7, 6, Error ( domain is root.Nat(Int), code is out-of-range ))\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("TOPAL-NUM-NAT-CONSTRUCT-001").count(), 3);
    assert!(trace.contains("Int->Nat:nonnegative"));
    assert!(trace.contains("root.Nat(Int);out-of-range"));

    let diagnostic = run(&[], "Nat -1\n");
    assert!(!diagnostic.status.success());
    let diagnostic = String::from_utf8(diagnostic.stderr).unwrap();
    assert!(diagnostic.contains("error[E-NAT-OUT-OF-RANGE]"));
    assert!(diagnostic.contains("help: use a provably nonnegative Int"));
}

#[test]
fn every_mode_compares_nat_constraint_values() {
    let source = include_str!("../../../../examples/language/nat-equality-and-ordering.t");
    let expected = b"(true, true, true, true, true, true, true, Less, true, true)\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-EQUALITY-001"));
    assert!(trace.contains("TOPAL-NUM-COMPARE-001"));
    assert!(trace.contains("TOPAL-NUM-THREE-WAY-COMPARE-001"));
}

#[test]
fn every_mode_constructs_canonical_rationals() {
    let source = include_str!("../../../../examples/language/rational-exact-construction.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(
            b"(Rational ( 7, 1 ), Rational ( 1, 2 ), Rational ( -1, 2 ), Rational ( 0, 1 ))\n"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("TOPAL-NUM-RATIONAL-CONSTRUCT-001").count(), 3);
    assert!(trace.contains("Int->Rational:explicit"));

    let diagnostic = run(&[], "Rational (1, 0)\n");
    assert!(!diagnostic.status.success());
    let diagnostic = String::from_utf8(diagnostic.stderr).unwrap();
    assert!(diagnostic.contains("error[E-DIVISION-BY-ZERO]"));
    assert!(diagnostic.contains("help: use a divisor that is provably nonzero"));
}

#[test]
fn every_mode_constructs_dynamic_rationals() {
    let source = include_str!("../../../../examples/language/dynamic-rational-construction.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(Rational ( 1, 2 ), Error ( domain is root.Rational(Int,Int), code is division-by-zero ), Error ( domain is root.Rational(Int,Int), code is indeterminate ))\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(
        trace
            .matches("TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001")
            .count(),
        3
    );
    assert!(trace.contains("root.Rational(Int,Int);division-by-zero"));
    assert!(trace.contains("root.Rational(Int,Int);indeterminate"));

    let diagnostic = run(&[], "Rational (0, 0)\n");
    assert!(!diagnostic.status.success());
    let diagnostic = String::from_utf8(diagnostic.stderr).unwrap();
    assert!(diagnostic.contains("error[E-INDETERMINATE-RATIONAL]"));
    assert!(diagnostic.contains("help: use a nonzero denominator"));
}

#[test]
fn every_mode_constructs_all_int_range_endpoint_forms() {
    let source = include_str!("../../../../examples/language/inclusive-int-ranges.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(0 ..= 10, 0 .. 10, 0 <.. 10, 0 <..= 10, true, false, false, false, false, true, 5 ..= 10, 20 ..= 10, true, false, false)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("TOPAL-RANGE-BOUNDS-001").count(), 7);
    assert_eq!(trace.matches("TOPAL-RANGE-MEMBERSHIP-001").count(), 9);
    assert!(trace.contains("\"detail\":\"nonempty\""));
    assert!(trace.contains("\"detail\":\"empty\""));
    assert!(trace.contains("\"detail\":\"accepted\""));
    assert!(trace.contains("\"detail\":\"rejected\""));
    assert_eq!(trace.matches("TOPAL-RANGE-INTERSECTION-001").count(), 2);
}

#[test]
fn every_mode_constructs_and_tests_rational_ranges() {
    let source = include_str!("../../../../examples/language/rational-ranges.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(Rational ( 0, 1 ) ..= Rational ( 5, 2 ), Rational ( 1, 1 ) ..= Rational ( 5, 2 ), true, true, false)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Int->Rational:left"));
    assert!(trace.contains("Int->Rational:membership"));
    assert_eq!(trace.matches("TOPAL-RANGE-MEMBERSHIP-001").count(), 3);
    assert!(trace.contains("TOPAL-RANGE-INTERSECTION-001"));
    assert!(
        trace
            .contains("\"detail\":\"(Range Rational, Range Rational, Boolean, Boolean, Boolean)\"")
    );
}

#[test]
fn every_mode_observes_finite_range_bounds_and_emptiness() {
    let source = include_str!("../../../../examples/language/finite-range-observation.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(
            b"(true, false, -2, 3, false, true, Rational ( 1, 2 ), Rational ( 2, 1 ), true, false, 0 <..= 2, 0 .. 2, 0 ..= 1, false)\n"
        ));
    }
}

#[test]
fn every_mode_evaluates_boolean_not() {
    let source = include_str!("../../../../examples/language/boolean-logic.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(false, true, true, false, false, false, true, true, true, false, false, true, true, false)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("TOPAL-TYPE-BOOLEAN-LOGIC-001").count(), 14);
    assert!(trace.contains("root.not(Boolean)"));
    assert_eq!(trace.matches("root.and(Boolean,Boolean)").count(), 4);
    assert!(trace.contains("and:eager"));
    assert_eq!(trace.matches("root.or(Boolean,Boolean)").count(), 4);
    assert!(trace.contains("or:eager"));
    assert_eq!(trace.matches("root.xor(Boolean,Boolean)").count(), 4);
    assert!(trace.contains("xor:eager"));
}

#[test]
fn every_mode_constructs_explicit_optional_values() {
    let source = include_str!("../../../../examples/language/optional-values.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(Some 42, Some \"present\", None, None, None, Some 7, None, None, \"present\", \"absent\", true, true, false, true)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(
        trace.matches("TOPAL-TYPE-OPTIONAL-CONSTRUCT-001").count(),
        15
    );
    assert!(trace.contains("optional.some.constructed"));
    assert!(trace.contains("optional.none.constructed"));
    assert!(trace.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001"));
    assert!(trace.contains("preserve"));
    assert!(trace.contains("absent"));
    assert_eq!(trace.matches("TOPAL-TYPE-OPTIONAL-CONTEXT-001").count(), 2);
    assert_eq!(trace.matches("TOPAL-DECISION-OPTIONAL-001").count(), 6);
    assert!(trace.contains("optional.payload.bound"));
    assert_eq!(trace.matches("TOPAL-TYPE-OPTIONAL-EQUALITY-001").count(), 4);
}

#[test]
fn every_mode_uses_optional_rational_values() {
    let source = include_str!("../../../../examples/language/optional-rational-values.t");
    let expected = b"(Some Rational ( 7, 2 ), None, true, true, true, true, true, \"some Rational\", \"no Rational\")\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-OPTIONAL-CONSTRUCT-001"));
    assert!(trace.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001"));
    assert!(trace.contains("TOPAL-DECISION-OPTIONAL-001"));
    assert_eq!(trace.matches("TOPAL-TYPE-OPTIONAL-EQUALITY-001").count(), 6);
}

#[test]
fn every_mode_indexes_user_perceived_string_characters() {
    let source = include_str!("../../../../examples/language/string-character-at.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output.stdout.ends_with(
                "(3, 3, Some \"a\u{301}\", Some \"👩‍🔬\", Some \"🇸🇪\", None, None, \"👩‍🔬\", \"missing\")\n"
                    .as_bytes()
            )
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-STRING-CHARACTER-COUNT-001"));
    assert!(trace.contains("TOPAL-STRING-ENTRY-COUNT-001"));
    assert_eq!(trace.matches("TOPAL-STRING-CHARACTER-AT-001").count(), 7);
    assert!(trace.contains("\"detail\":\"Some\""));
    assert!(trace.contains("\"detail\":\"None\""));
    assert!(trace.contains("TOPAL-DECISION-OPTIONAL-001"));
    assert!(trace.contains("TOPAL-STRING-FROM-CHARACTER-001"));
}

#[test]
fn every_mode_applies_universal_unicode_uppercase() {
    let source = include_str!("../../../../examples/language/string-uppercase.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with("\"STRASSE ΣΣ\"\n".as_bytes()));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.upper(String)"));
    assert!(trace.contains("TOPAL-STRING-UPPER-001"));
}

#[test]
fn every_mode_applies_universal_unicode_lowercase() {
    let source = include_str!("../../../../examples/language/string-lowercase.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with("\"i\u{307}ς\"\n".as_bytes()));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.lower(String)"));
    assert!(trace.contains("TOPAL-STRING-LOWER-001"));
}

#[test]
fn every_mode_applies_full_universal_unicode_case_folding() {
    let source = include_str!("../../../../examples/language/string-case-fold.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with("\"strasse σσ\"\n".as_bytes()));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.case-fold(String)"));
    assert!(trace.contains("TOPAL-STRING-CASE-FOLD-001"));
}

#[test]
fn every_mode_compares_canonical_string_equivalence() {
    let source = include_str!("../../../../examples/language/string-canonical-equality.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(false, true, false)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.canonically-equals(String,String)"));
    assert_eq!(
        trace.matches("TOPAL-STRING-CANONICAL-EQUALITY-001").count(),
        2
    );
}

#[test]
fn every_mode_compares_exact_strings_and_optional_strings() {
    let source = include_str!("../../../../examples/language/string-exact-equality.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(false, true, true, true, true, true, true, false)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("TOPAL-TYPE-EQUALITY-001").count(), 8);
    assert_eq!(trace.matches("TOPAL-TYPE-OPTIONAL-EQUALITY-001").count(), 4);
}

#[test]
fn every_mode_constructs_concatenates_and_tests_strings() {
    let source = include_str!("../../../../examples/language/string-construction.t");
    let expected = b"(\"\", true, false, true, \"e\xcc\x81\", text\"say \"hello\"!\"text, text__\"value \"text and \"text_ marker!\"text__, \"ab\", \"adjacent literals\")\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(expected));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-STRING-EMPTY-001"));
    assert!(trace.contains("TOPAL-STRING-LITERAL-COMPOSE-001"));
    assert_eq!(trace.matches("TOPAL-STRING-CONCAT-001").count(), 5);
    assert_eq!(trace.matches("TOPAL-STRING-EMPTY-PREDICATE-001").count(), 2);
}

#[test]
fn every_mode_compares_recursive_positional_products() {
    let source = include_str!("../../../../examples/language/tuple-equality.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(true, false, false, true, true)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-EQUALITY-001"));
}

#[test]
fn every_mode_collects_unicode_character_traversal() {
    let source = include_str!("../../../../examples/language/string-character-traversal.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with("\"a\u{301}👩‍🔬🇸🇪\"\n".as_bytes()));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("generator.yielded").count(), 3);
    assert!(trace.contains("TOPAL-STRING-CHARACTERS-COLLECT-001"));
}

#[test]
fn every_mode_foreach_consumes_character_generator() {
    let source = include_str!("../../../../examples/language/string-character-foreach.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        if arguments.is_empty() {
            assert!(output.stdout.ends_with(b"()\n"));
        }
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("generator.yielded").count(), 3);
    assert_eq!(trace.matches("generator.resumed").count(), 3);
    assert!(trace.contains("TOPAL-STRING-CHARACTERS-FOREACH-001"));
}

#[test]
fn every_mode_rejects_non_unit_foreach_action() {
    let source = "characters \"Topal\" foreach { character }\n  String character\n\n";
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        let rendered = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            rendered.contains("E-FOREACH-ACTION-RESULT"),
            "{arguments:?}: {rendered}"
        );
    }
}

#[test]
fn every_mode_consumes_named_character_generator() {
    let source = include_str!("../../../../examples/language/string-named-character-generator.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("generator.started"));
    assert!(trace.contains("generator.consumed"));
    assert_eq!(trace.matches("generator.yielded").count(), 3);
}

#[test]
fn every_mode_consumes_returned_character_generator() {
    let source = include_str!("../../../../examples/language/string-character-generator-result.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("function.exit"));
    assert!(trace.contains("generator.consumed"));
}

#[test]
fn every_mode_transfers_generator_parameter() {
    let source =
        include_str!("../../../../examples/language/string-character-generator-parameter.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-STRING-CHARACTERS-PARAMETER-001"));
    assert!(trace.contains("generator.parameter.transferred"));
}

#[test]
fn every_mode_closes_abandoned_generator_parameter() {
    let source = include_str!("../../../../examples/language/string-character-generator-close.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-STRING-CHARACTERS-CLOSE-001"));
    assert!(trace.contains("generator.closed"));
    assert!(trace.contains("domain=root;code=generator-closed;generator=root.characters"));
}

#[test]
fn every_mode_constructs_qualified_generator_error_code() {
    let source = include_str!("../../../../examples/language/generator-error-codes.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(generator-closed, true)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-ERROR-CODE-001"));
    assert!(trace.contains("namespace.member.selected"));
}

#[test]
fn every_mode_traverses_custom_multiple_yield_generator() {
    let source = include_str!("../../../../examples/language/custom-multiple-yield-generator.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let rendered = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!rendered.contains("error["), "{arguments:?}: {rendered}");
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-DECLARATION-001"));
    assert!(trace.contains("generator.declared"));
    assert!(trace.contains("generator.started"));
    assert_eq!(trace.matches("generator.yielded").count(), 2);
    assert_eq!(trace.matches("generator.resumed").count(), 2);
    assert!(trace.contains("TOPAL-GENERATOR-FOREACH-001"));
}

#[test]
fn every_mode_uses_custom_generator_local_binding() {
    let source = include_str!("../../../../examples/language/custom-generator-local-binding.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("error["));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("binding.bind"));
    assert!(trace.contains("TOPAL-GENERATOR-FOREACH-001"));
}

#[test]
fn every_mode_traverses_generator_returning_before_yield() {
    let source = include_str!("../../../../examples/language/custom-generator-early-return.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("error["));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("generator.yielded").count(), 0);
    assert!(trace.contains("generator.returned"));
    assert!(trace.contains("TOPAL-GENERATOR-EARLY-RETURN-001"));
}

#[test]
fn every_mode_observes_distinct_generator_final_character() {
    let source = include_str!("../../../../examples/language/custom-generator-final-character.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"R\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
    assert!(trace.contains("generator.returned") && trace.contains("Character"));
}

#[test]
fn every_mode_suspends_custom_generator_between_yields() {
    let source = include_str!("../../../../examples/language/custom-generator-suspension.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert_eq!(trace.matches("generator.suspended").count(), 2);
    let resumed = trace.find("generator.resumed").unwrap();
    let local = trace
        .find("\"event\":\"binding.bind\",\"rule\":\"TOPAL-SYN-BIND-001\",\"detail\":\"copy\"")
        .unwrap();
    let second_suspend = trace.rfind("generator.suspended").unwrap();
    assert!(resumed < local && local < second_suspend);
}

#[test]
fn every_mode_binds_successful_unit_resumption() {
    let source = include_str!("../../../../examples/language/custom-generator-resume-binding.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-RESUME-BINDING-001"));
    let resumed = trace.find("generator.resumed").unwrap();
    let bound = trace.find("generator.resume.bound").unwrap();
    let resolved = trace.rfind("\"detail\":\"resumed\"").unwrap();
    assert!(resumed < bound && bound < resolved);
}

#[test]
fn every_mode_closes_abandoned_custom_generator() {
    let source = include_str!("../../../../examples/language/custom-generator-close.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-CLOSE-001"));
    assert!(trace.contains("domain=root;code=generator-closed;generator=root.pause-once"));
}

#[test]
fn every_mode_runs_custom_generator_close_handler() {
    let source = include_str!("../../../../examples/language/custom-generator-close-handler.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-CLOSE-HANDLER-001"));
    assert!(trace.contains("generator.close.bound"));
    assert!(trace.contains("domain=root;code=generator-closed;generator=root.handle-close"));
    assert!(trace.contains("decision.rule.selected"));
}

#[test]
fn every_mode_matches_qualified_generator_close_code() {
    let source =
        include_str!("../../../../examples/language/custom-generator-close-code-pattern.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001"));
    assert!(trace.contains("generator.error.code.matched"));
    assert!(trace.contains("rule=0"));
}

#[test]
fn every_mode_consumes_custom_generator_returned_by_function() {
    let source = include_str!("../../../../examples/language/custom-generator-function-result.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"));
    assert!(trace.contains("generator.result.transferred"));
    assert!(trace.contains("generator.yielded"));
}

#[test]
fn every_mode_transfers_custom_generator_function_parameter() {
    let source =
        include_str!("../../../../examples/language/custom-generator-function-parameter.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(trace.contains("generator.parameter.transferred"));
    assert_eq!(trace.matches("generator.yielded").count(), 1);
}

#[test]
fn every_mode_closes_unconsumed_custom_generator_parameter() {
    let source = include_str!("../../../../examples/language/custom-generator-parameter-close.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(trace.contains("TOPAL-GENERATOR-CLOSE-001"));
    assert!(trace.contains("domain=root;code=generator-closed;generator=root.pause-once"));
}

#[test]
fn every_mode_transfers_character_returning_generator_parameter() {
    let source =
        include_str!("../../../../examples/language/custom-generator-character-return-parameter.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"R\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"));
    assert!(trace.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
}

#[test]
fn every_mode_consumes_character_returning_generator_function_result() {
    let source =
        include_str!("../../../../examples/language/custom-generator-character-return-result.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"R\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"));
    assert!(trace.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
}

#[test]
fn every_mode_starts_custom_generator_with_string_input() {
    let source = include_str!("../../../../examples/language/custom-generator-string-input.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("root.empty?(String)"));
    assert!(trace.contains("generator.suspended"));
}

#[test]
fn every_mode_traverses_custom_string_yields() {
    let source = include_str!("../../../../examples/language/custom-generator-string-yield.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator String Unit Unit"));
    assert_eq!(trace.matches("generator.yielded").count(), 2);
    assert_eq!(trace.matches("generator.resumed").count(), 2);
}

#[test]
fn every_mode_observes_distinct_generator_final_string() {
    let source = include_str!("../../../../examples/language/custom-generator-string-return.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"done\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator String Unit String"));
    assert!(trace.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
}

#[test]
fn every_mode_executes_discarded_computation_between_yields() {
    let source =
        include_str!("../../../../examples/language/custom-generator-discard-between-yields.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    let resumed = trace.find("generator.resumed").unwrap();
    let tested = resumed + trace[resumed..].find("string.empty.tested").unwrap();
    let suspended = trace.rfind("generator.suspended").unwrap();
    assert!(resumed < tested && tested < suspended);
}

#[test]
fn every_mode_executes_explicit_generator_return() {
    let source = include_str!("../../../../examples/language/custom-generator-explicit-return.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"done\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-GENERATOR-EXPLICIT-RETURN-001"));
    assert_eq!(trace.matches("generator.yielded").count(), 0);
}

#[test]
fn every_mode_returns_explicitly_after_generator_resumption() {
    let source =
        include_str!("../../../../examples/language/custom-generator-return-after-yield.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"done\"\n"));
    }
    let output = run(&["--test"], source);
    let trace = String::from_utf8(output.stderr).unwrap();
    let resumed = trace.find("generator.resumed").unwrap();
    let returned = trace.find("generator.return.explicit").unwrap();
    assert!(resumed < returned);
}

#[test]
fn every_mode_traverses_boolean_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-boolean-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"false\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator Boolean Unit Boolean"));
    assert!(trace.contains("generator.yielded") && trace.contains("Boolean"));
    assert!(trace.contains("TOPAL-GENERATOR-FINAL-RETURN-001"));
}

#[test]
fn every_mode_traverses_arbitrary_precision_int_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-int-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"1000000000000000000000000000000\n")
        );
    }
}

#[test]
fn every_mode_traverses_exact_rational_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-rational-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"Rational ( 2, 3 )\n"));
    }
}

#[test]
fn every_mode_traverses_unit_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-unit-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator Unit Unit Unit"));
    assert_eq!(trace.matches("generator.yielded").count(), 1);
}

#[test]
fn every_mode_traverses_optional_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-optional-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"None\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator Optional Int Unit Optional Int"));
    assert!(trace.contains("Some 7"));
}

#[test]
fn every_mode_traverses_range_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-range-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"5 ..= 10\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains(
        "\"event\":\"evaluation.result\",\"rule\":\"TOPAL-SYN-GRAMMAR-001\",\"detail\":\"Range Int\""
    ));
}

#[test]
fn every_mode_traverses_nat_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-nat-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"8\n"));
    }
}

#[test]
fn every_mode_traverses_enum_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-enum-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"Second\n"));
    }
}

#[test]
fn every_mode_traverses_product_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-product-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(8, \"done\")\n"));
    }
}

#[test]
fn every_mode_traverses_result_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-result-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("division-by-zero"));
    }
}

#[test]
fn every_mode_traverses_comparison_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-comparison-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"Greater\n"));
    }
}

#[test]
fn every_mode_traverses_nested_optional_generator_values() {
    let source =
        include_str!("../../../../examples/language/custom-generator-nested-optional-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"Some (8, \"done\")\n"));
    }
}

#[test]
fn every_mode_traverses_nested_result_generator_values() {
    let source =
        include_str!("../../../../examples/language/custom-generator-nested-result-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(8, \"done\")\n"));
    }
}

#[test]
fn every_mode_traverses_nested_absent_optional_values() {
    let source =
        include_str!("../../../../examples/language/custom-generator-nested-none-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"None\n"));
    }
}

#[test]
fn every_mode_traverses_recursive_nominal_generator_values() {
    let source =
        include_str!("../../../../examples/language/custom-generator-recursive-nominal-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(Some Second, Second)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Optional Choice"));
    assert!(trace.contains("Result (Choice, lang arithmetic ArithmeticErrorCode)"));
}

#[test]
fn every_mode_selects_generator_final_decision() {
    let source = include_str!("../../../../examples/language/custom-generator-final-decision.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"accepted\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    let resumed = trace.find("generator.resumed").unwrap();
    let selected = trace.find("decision.rule.selected").unwrap();
    let returned = trace.find("generator.returned").unwrap();
    assert!(resumed < selected && selected < returned);
}

#[test]
fn generator_classifier_error_is_actionable_in_script_mode() {
    let source = "invalid is generator ( initial : Boolean )\n  yields Boolean\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  42\ngenerated is invalid true\ngenerated foreach { value }\n  _ is not value\n";
    let output = run(&[], source);
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("returned `Int`, but its declaration requires `String`"));
    assert!(error.contains("help: produce `String` here"));
}

#[test]
fn every_mode_retains_generator_local_function() {
    let source = include_str!("../../../../examples/language/custom-generator-local-function.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"accepted\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    let declared_enum = trace.find("enum.declared").unwrap();
    let resumed = trace.find("generator.resumed").unwrap();
    let entered = trace.rfind("function.entry").unwrap();
    assert!(declared_enum < resumed && resumed < entered);
}

#[test]
fn every_mode_restores_generator_local_close_handler() {
    let source =
        include_str!("../../../../examples/language/custom-generator-local-close-handler.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        assert!(run(arguments, source).status.success());
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    let close_bound = trace.find("generator.close.bound").unwrap();
    let entered = trace.rfind("function.entry").unwrap();
    let closed = trace.find("generator.closed").unwrap();
    assert!(close_bound < entered && entered < closed);
    assert!(trace.contains("domain=root;code=generator-closed;generator=root.handle-close"));
}

#[test]
fn every_mode_selects_generator_overloads() {
    let source = include_str!("../../../../examples/language/custom-generator-overloads.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(\"unary\", \"binary\")\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("generator.selected"));
    assert!(trace.contains("generator.argument.bound"));
    assert_eq!(trace.matches("generator.foreach.result.bound").count(), 2);
    assert!(trace.contains("Int, String"));
}

#[test]
fn every_mode_transfers_generic_generator_function_boundaries() {
    let source = include_str!(
        "../../../../examples/language/custom-generator-generic-function-boundaries.t"
    );
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"\"done\"\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator Int Unit String"));
    assert!(trace.contains("generator.result.transferred"));
    assert!(trace.contains("generator.parameter.transferred"));
    assert!(trace.contains("generator.foreach.result.bound"));
}

#[test]
fn every_mode_transfers_compound_generator_function_boundaries() {
    let source = include_str!(
        "../../../../examples/language/custom-generator-compound-function-boundaries.t"
    );
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(8, \"done\")\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator (Int, String) Unit (Int, String)"));
    assert!(trace.contains("generator.result.transferred"));
    assert!(trace.contains("generator.parameter.transferred"));
}

#[test]
fn every_mode_transfers_nested_generator_function_boundaries() {
    let source =
        include_str!("../../../../examples/language/custom-generator-nested-function-boundaries.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(8, \"done\")\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    let classifier = "Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)";
    assert!(trace.contains(&format!(
        "\"event\":\"generator.result.transferred\",\"rule\":\"TOPAL-GENERATOR-FUNCTION-RESULT-001\",\"detail\":\"{classifier}\""
    )));
    assert!(trace.contains(&format!(
        "\"event\":\"generator.parameter.transferred\",\"rule\":\"TOPAL-GENERATOR-FUNCTION-PARAMETER-001\",\"detail\":\"{classifier}\""
    )));
}

#[test]
fn every_mode_transfers_list_generator_values() {
    let source = include_str!("../../../../examples/language/custom-generator-list-values.t");
    for arguments in [&[][..], &["--test"][..], &["--interactive"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"Entry ( 7, Entry ( 9, Empty ) )\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("Generator List Int Unit List Int"));
    assert!(trace.contains("generator.result.transferred"));
    assert!(trace.contains("generator.parameter.transferred"));
    assert!(trace.contains("TOPAL-LIST-APPEND-001"));
}

#[test]
fn every_mode_rejects_yield_after_custom_close() {
    let source = include_str!(
        "../../../../examples/language-diagnostics/custom-generator-yield-after-close.t"
    );
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        let rendered = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(rendered.contains("E-GENERATOR-YIELD-AFTER-CLOSE"));
        assert!(rendered.contains("cannot yield again after observing"));
    }
}

#[test]
fn script_mode_explains_consumed_generator_reuse() {
    let source = include_str!("../../../../examples/language-diagnostics/generator-consumed.t");
    let output = run(&[], source);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-GENERATOR-CONSUMED]"));
    assert!(diagnostic.contains("generator `generated` was already consumed"));
    assert!(diagnostic.contains("construct a fresh generator"));
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

#[test]
fn every_mode_constructs_compares_and_decomposes_lists() {
    let source = include_str!("../../../../examples/language/lists.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"(Some 6, Some 6, Some Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) ), None, None, 5, false, true, true, Some (6, Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) )), Some 10, true)\n"));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-TYPE-LIST-CONSTRUCT-001"));
    assert!(trace.contains("TOPAL-DECISION-LIST-001"));
    assert!(trace.contains("TOPAL-TYPE-LIST-EQUALITY-001"));
    assert!(trace.contains("TOPAL-LIST-PREPEND-001"));
    assert!(trace.contains("TOPAL-LIST-APPEND-001"));
    assert!(trace.contains("TOPAL-LIST-CONCAT-001"));
    assert!(trace.contains("TOPAL-LIST-ENTRY-COUNT-001"));
    assert!(trace.contains("TOPAL-LIST-EMPTY-PREDICATE-001"));
    assert!(trace.contains("TOPAL-LIST-EMPTY-001"));
    assert!(trace.contains("TOPAL-LIST-ONE-001"));
    assert!(trace.contains("TOPAL-LIST-UNCONS-001"));
    assert!(trace.contains("TOPAL-LIST-FIRST-001"));
    assert!(trace.contains("TOPAL-LIST-REST-001"));
    assert!(trace.contains("TOPAL-LIST-REVERSE-001"));
}

#[test]
fn script_mode_explains_invalid_list_entries() {
    let output = run(&[], "values : List Int is Entry ( \"bad\", Empty )\n");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("error[E-LIST-ENTRY-CLASSIFIER]"));
    assert!(diagnostic.contains("this list requires `Int`"));
    assert!(diagnostic.contains("use a `Int` value"));
}

#[test]
fn every_mode_preserves_recursive_list_classifiers() {
    let source = include_str!("../../../../examples/language/nested-lists.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(Some Entry ( (7, \"seven\"), Empty ), 1, true)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("List List (Int, String)"));
    assert!(trace.contains("TOPAL-TYPE-LIST-EQUALITY-001"));
}

#[test]
fn every_mode_distinguishes_list_containment_laws() {
    let source = include_str!("../../../../examples/language/list-containment.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            output
                .stdout
                .ends_with(b"(true, false, true, true, false, false)\n")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-LIST-CONTAINS-ENTRY-001"));
    assert!(trace.contains("TOPAL-LIST-CONTAINS-SEQUENCE-001"));
    assert!(trace.contains("TOPAL-LIST-CONTAINS-SUBSEQUENCE-001"));
}

#[test]
fn every_mode_removes_list_values_by_explicit_law() {
    let source = include_str!("../../../../examples/language/list-removal.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("Entry ( 1, Entry ( 3")
        );
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-LIST-REMOVE-FIRST-001"));
    assert!(trace.contains("TOPAL-LIST-REMOVE-ALL-001"));
}

#[test]
fn every_mode_executes_contextual_anonymous_list_functions() {
    let source = include_str!("../../../../examples/language/anonymous-list-functions.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(
            "(Entry ( 2, Entry ( 4, Entry ( 6, Empty ) ) ), Entry ( 2, Entry ( 3, Empty ) ), 6)"
        ));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    assert!(trace.contains("TOPAL-FUNCTION-ANONYMOUS-001"));
    assert!(trace.contains("TOPAL-COLLECTION-FOLD-001"));
}

#[test]
fn every_mode_executes_complete_list_sequence_operations() {
    let source = include_str!("../../../../examples/language/list-sequence-operations.t");
    for arguments in [&[][..], &["--interactive"][..], &["--test"][..]] {
        let output = run(arguments, source);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("domain is root.zip-exact(List,List)"));
        assert!(stdout.contains("\"Topal\""));
    }
    let trace = String::from_utf8(run(&["--test"], source).stderr).unwrap();
    for rule in [
        "TOPAL-LIST-INSERT-AT-001",
        "TOPAL-LIST-SPLIT-AT-001",
        "TOPAL-LIST-REMOVE-INDEXES-001",
        "TOPAL-LIST-ZIP-LONGEST-001",
        "TOPAL-LIST-UNZIP-001",
        "TOPAL-COLLECTION-FOREACH-001",
        "TOPAL-COLLECTION-COLLECT-STRING-001",
    ] {
        assert!(trace.contains(rule), "missing {rule}");
    }
}
