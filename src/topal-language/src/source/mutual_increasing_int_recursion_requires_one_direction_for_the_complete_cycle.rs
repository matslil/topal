#[test]
fn mutual_increasing_int_recursion_requires_one_direction_for_the_complete_cycle() {
    let source = "even-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then true\n    otherwise odd-up (value + 1)\nodd-up is fn (value : Int) -> Boolean\n  value\n    >= 0 then false\n    otherwise even-up (value + 1)\n(even-up (-6), odd-up (-6))\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    assert!(trace.iter().any(|event| {
        event.contains("function.recursion.cycle.proven")
            && event.contains("TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001")
    }));

    let mixed = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    >= 10 then false\n    otherwise first (value + 1)\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(mixed.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn same_named_distinct_overloads_are_not_recursive() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  (describe 42) concat \":\" concat value\ndescribe \"Topal\"\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "\"integer:Topal\"");
    let string = trace
        .iter()
        .position(|event| event.contains("describe (String)"))
        .unwrap();
    let integer = trace
        .iter()
        .position(|event| event.contains("describe (Int)"))
        .unwrap();
    assert!(string < integer);
}

#[test]
fn bounded_int_recursion_accepts_only_positive_literal_progress() {
    let value = Session::new()
        .evaluate(
            "down is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise 1 + (down (value - 3))\nup is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (up (value + 2))\n(down 7, up (-5))\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(3, 3)");

    let mutual = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 2)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 3)\nfirst 7\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(mutual.to_string(), "false");

    for invalid_step in ["0", "-1"] {
        let source = format!(
            "stuck is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise stuck (value - {invalid_step})\nstuck 1\n"
        );
        let error = Session::new()
            .evaluate(&source, &mut std::io::sink())
            .unwrap_err();
        assert_eq!(error.code, "E-UNPROVEN-RECURSION");
    }
}

#[test]
fn every_recursive_call_in_one_action_must_progress() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "branch-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (branch-count (value - 1)) + (branch-count (value - 2))\nbranch-count 3\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "5");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count()
            > 2
    );

    let error = Session::new()
        .evaluate(
            "unsafe-branch is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (unsafe-branch (value - 1)) + (unsafe-branch value)\nunsafe-branch 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn every_call_on_one_mutual_edge_must_share_target_and_progress() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "first-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second-count (value - 1)) + (second-count (value - 2))\nsecond-count is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first-count (value - 1)\nfirst-count 3\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "3");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count()
            > 1
    );

    let error = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second (value - 1)) + (second value)\nsecond is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first (value - 1)\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");

    let different_target = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise (second (value - 1)) + (third (value - 1))\nsecond is fn (value : Int) -> Int\n  value\n    <= 0 then 1\n    otherwise first (value - 1)\nthird is fn (value : Int) -> Int\n  value\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(different_target.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn comparison_decision_uses_subject_as_left_operand() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "minimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise missing\n42 minimum 50\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(trace.iter().any(|event| {
        event.contains("decision.rule.selected") && event.contains("TOPAL-DECISION-COMPARISON-001")
    }));
    assert!(!trace.iter().any(|event| event.contains("missing")));
}

#[test]
fn decreasing_int_recursion_executes_only_after_structural_proof() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "sum-down is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise value + (sum-down (value - 1))\nsum-down 5\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "15");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.recursion.proven"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count(),
        5
    );

    let unproven = Session::new()
        .evaluate(
            "wrong is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise wrong (value + 1)\nwrong 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(unproven.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn increasing_int_recursion_executes_only_after_structural_proof() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "distance-up is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise 1 + (distance-up (value + 1))\ndistance-up (-5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "5");
    assert!(trace.iter().any(|event| {
        event.contains("function.recursion.proven")
            && event.contains("TOPAL-FUNCTION-RECURSION-INT-INCREASING-001")
    }));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.recursion.descended"))
            .count(),
        5
    );
}

#[test]
fn comparison_matcher_evaluates_complete_operand_expression() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "within is fn (value : Int, limit : Int) -> Boolean\n  value\n    < limit + 1 then true\n    otherwise false\n(5 within 5, 6 within 5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    let addition = trace
        .iter()
        .position(|event| event.contains("root.+(Int,Int)"))
        .unwrap();
    let comparison = trace
        .iter()
        .position(|event| event.contains("root.<(TotalOrder,TotalOrder)"))
        .unwrap();
    assert!(addition < comparison);
}

#[test]
fn nested_function_captures_outer_parameter_without_leaking() {
    let mut session = Session::new();
    let mut trace = Vec::new();
    let value = session
        .evaluate(
            "answer is fn (input : Int) -> Int\n  add-input is fn (value : Int) -> Int\n    value + input\n  add-input 2\nanswer 40\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let outer_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("answer"))
        .unwrap();
    let nested_declaration = trace
        .iter()
        .position(|event| event.contains("function.declared") && event.contains("add-input"))
        .unwrap();
    let nested_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("add-input"))
        .unwrap();
    assert!(outer_entry < nested_declaration && nested_declaration < nested_entry);

    let error = session
        .evaluate("add-input 2\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn structured_error_fields_retain_code_type_and_domain_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproblem is 1.0 divide 0.0\n(problem code, problem domain)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value,
        Value::Tuple(vec![
            Value::Enum {
                type_name: "lang arithmetic ArithmeticErrorCode".into(),
                alternative: "division-by-zero".into(),
            },
            Value::ErrorDomain("root./(Rational,Rational)".into()),
        ])
    );
    assert_eq!(
        value.to_string(),
        "(division-by-zero, root./(Rational,Rational))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("error.field.selected"))
            .count(),
        2
    );
}

#[test]
fn qualified_error_code_pattern_selects_without_using_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n    Error problem then \"other\"\ndescribe 0.0\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("zero".into()));
    assert!(trace.iter().any(|event| {
        event.contains("error.code.matched") && event.contains("TOPAL-DECISION-ERROR-CODE-001")
    }));
}

#[test]
fn classified_binding_projects_success_and_propagates_error() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nproject is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  quotient : Rational is 1.0 divide denominator\n  quotient + 1.0\n(project 2.0, project 0.0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 3, 2 ), Error ( domain is root./(Rational,Rational), code is division-by-zero ))"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("result.success.projected"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("result.error.projected"))
    );
}

#[test]
fn classified_binding_rejects_error_propagation_from_infallible_function() {
    let error = Session::new()
        .evaluate(
            "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nbad is fn (denominator : Rational) -> Rational\n  quotient : Rational is 1.0 divide denominator\n  quotient\nbad 0.0\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RESULT-PROJECTION-INFALLIBLE");
    assert!(error.message.contains("returning `Rational`"));
    assert!(
        error
            .help
            .is_some_and(|help| help.contains("match the Error"))
    );
}

#[test]
fn character_classifier_uses_pinned_grapheme_segmentation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "identity is fn (value : Character) -> Character\n  value\ncomposed : Character is \"a\u{301}\"\n(String (identity \"🙂\"), String composed)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"🙂\", \"a\u{301}\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-FROM-CHARACTER-001"))
            .count(),
        2
    );

    let error = Session::new()
        .evaluate("invalid : Character is \"ab\"\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-CHARACTER-CLASSIFIER");
    assert!(error.message.contains("contains 2"));
}

#[test]
fn int_modulo_is_euclidean_and_dynamic_zero_returns_error() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "modulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  left % right\nquotient-modulo is fn (left : Int, right : Int) -> Result ((Int, Int), lang arithmetic ArithmeticErrorCode)\n  left /% right\n(17 % 5, -17 % 5, 17 % -5, -17 /% 5, 17 /% -5, 17 modulo 0, 17 quotient-modulo 0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(2, 3, 2, (-4, 3), (-3, 2), Error ( domain is root.%(Int,Int), code is division-by-zero ), Error ( domain is root./%(Int,Int), code is division-by-zero ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-MODULO-001"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-QUOTIENT-MODULO-001"))
            .count(),
        2
    );
}

#[test]
fn exact_numeric_absolute_retains_operand_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(absolute -42, absolute 42, absolute -1.25, absolute 1.25)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(42, 42, Rational ( 5, 4 ), Rational ( 5, 4 ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-ABS-001"))
            .count(),
        4
    );
}

#[test]
fn named_numeric_negate_matches_exact_additive_inverse() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(negate 42, negate -42, negate 1.25, negate -1.25)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(-42, 42, Rational ( -5, 4 ), Rational ( 5, 4 ))"
    );
    assert!(trace.iter().any(|event| event.contains("root.negate(Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.negate(Rational)"))
    );
}

#[test]
fn exact_numeric_zero_uses_explicit_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(zero Int, zero Nat, zero Rational, one Int, one Nat, one Rational)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(0, 0, Rational ( 0, 1 ), 1, 1, Rational ( 1, 1 ))"
    );
    assert!(trace.iter().any(|event| event.contains("root.zero(Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.zero(Rational)"))
    );
    assert!(trace.iter().any(|event| event.contains("root.one(Int)")));
    assert!(trace.iter().any(|event| event.contains("root.zero(Nat)")));
    assert!(trace.iter().any(|event| event.contains("root.one(Nat)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.one(Rational)"))
    );
}

#[test]
fn exact_three_way_comparison_returns_nominal_alternatives() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("describe is fn (value : Comparison) -> String\n  value\n    Less then \"less\"\n    Equal then \"equal\"\n    Greater then \"greater\"\n(1 <=> 2, 2 <=> 2, 3 <=> 2, 1 <=> 1.5, describe (1 <=> 2), describe (2 <=> 2), describe (3 <=> 2))\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Less, Equal, Greater, Less, \"less\", \"equal\", \"greater\")"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-THREE-WAY-COMPARE-001"))
            .count(),
        7
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("decision.rule.selected"))
            .filter(|event| event.contains("TOPAL-DECISION-ENUM-001"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:left"))
    );
}

#[test]
fn closed_exact_rational_narrows_to_int_without_rounding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "fifty : Int is 100 / 2\nnegative-three : Int is -9 / 3\n(fifty, negative-three)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(50, -3)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-INT-EXACT-001"))
            .count(),
        2
    );

    let error = Session::new()
        .evaluate("half : Int is 1 / 2\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RATIONAL-NOT-EXACT-INT");
    assert!(error.message.contains("denominator 2"));
}

#[test]
fn dynamic_rational_to_int_validation_returns_typed_result() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "halve is fn (value : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  half : Int is value / 2\n  half\n(halve 100, halve 3)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(50, Error ( domain is root.Int(Rational), code is not-representable ))"
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001")
            && event.contains("Rational->Int:validated")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-RATIONAL-INT-VALIDATE-001")
            && event.contains("root.Int(Rational);not-representable")
    }));
}

#[test]
fn checked_int_construction_is_exact_and_fallible() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "as-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  Int value\n(Int 7, as-int 6.0, as-int 1.5)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(7, 6, Error ( domain is root.Int(Rational), code is not-representable ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-INT-CONSTRUCT-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Int 1.5\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RATIONAL-NOT-EXACT-INT");
}

#[test]
fn checked_nat_construction_validates_the_nonnegative_constraint() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "as-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  Nat value\n(Nat 7, as-nat 6, as-nat -1)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(7, 6, Error ( domain is root.Nat(Int), code is out-of-range ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-NAT-CONSTRUCT-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Nat -1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NAT-OUT-OF-RANGE");
}

#[test]
fn closed_rational_construction_canonicalizes_components() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(Rational 7, Rational (2, 4), Rational (2, -4), Rational (0, 5))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 7, 1 ), Rational ( 1, 2 ), Rational ( -1, 2 ), Rational ( 0, 1 ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-001"))
            .count(),
        3
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-NUM-INT-RATIONAL-CONVERT-001")
            && event.contains("Int->Rational:explicit")
    }));

    let error = Session::new()
        .evaluate("Rational (1, 0)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-DIVISION-BY-ZERO");
}

#[test]
fn dynamic_rational_construction_distinguishes_zero_failures() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "ratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  Rational (numerator, denominator)\n(1 ratio 2, 1 ratio 0, 0 ratio 0)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 1, 2 ), Error ( domain is root.Rational(Int,Int), code is division-by-zero ), Error ( domain is root.Rational(Int,Int), code is indeterminate ))"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"))
            .count(),
        3
    );

    let error = Session::new()
        .evaluate("Rational (0, 0)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-INDETERMINATE-RATIONAL");
}

#[test]
fn int_ranges_preserve_all_endpoint_forms_and_allow_empty_ranges() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("half-open is 0 .. 10\nopen is 0 <.. 10\nclosed is 0 ..= 10\nlower-open is 0 <..= 10\nempty-interval is 10 .. 10\n(half-open, 0 in half-open, 10 in half-open, 0 in open, 10 in open, 0 in closed, 10 in closed, 0 in lower-open, 10 in lower-open, empty? empty-interval, range-lower-inclusive? lower-open, range-upper-inclusive? lower-open)\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(0 .. 10, true, false, false, false, true, true, false, true, true, false, true)"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-RANGE-BOUNDS-001"))
            .count(),
        5
    );
    assert!(trace.iter().any(|event| event.contains("empty")));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-RANGE-MEMBERSHIP-001"))
            .count(),
        8
    );
}

#[test]
fn boolean_not_is_eager_and_type_checked() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("(not true, not false, not (not true))\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, true)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-BOOLEAN-LOGIC-001"))
            .count(),
        4
    );
    let error = Session::new()
        .evaluate("not 1\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-BOOLEAN-NOT-OPERAND");
}

#[test]
fn boolean_and_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true and true, true and false, false and true, false and false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false, false, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("and:eager"))
            .count(),
        4
    );
}

#[test]
fn boolean_or_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true or true, true or false, false or true, false or false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, true, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("or:eager"))
            .count(),
        4
    );
}

#[test]
fn boolean_xor_implements_the_eager_truth_table() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(true xor true, true xor false, false xor true, false xor false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("xor:eager"))
            .count(),
        4
    );
}

#[test]
fn explicit_optional_constructors_preserve_payload_classifiers() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(Some 42, Some \"present\", None Int, None String)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some 42, Some \"present\", None, None)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-CONSTRUCT-001"))
            .count(),
        4
    );
}

#[test]
fn contextual_none_uses_the_binding_classifier() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("missing : Optional Int is None\nmissing\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "None");
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001") && event.contains("Int")
    }));

    let error = Session::new()
        .evaluate("None\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn optional_values_cross_matching_function_boundaries() {
    let value = Session::new()
        .evaluate(
            "preserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\n(preserve (Some 7), preserve (None Int))\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some 7, None)");

    let error = Session::new()
        .evaluate(
            "preserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\npreserve (None String)\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FUNCTION-ARGUMENT-TYPE");
}

#[test]
fn contextual_none_uses_function_result_classifiers() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "implicit is fn () -> Optional Int\n  None\nexplicit is fn () -> Optional String\n  return None\n(implicit (), explicit ())\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(None, None)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-CONTEXT-001"))
            .count(),
        2
    );
}

#[test]
fn optional_decisions_bind_only_present_payloads() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (candidate : Optional Int) -> String\n  candidate\n    Some payload then \"present\"\n    None then \"absent\"\n(describe (Some 7), describe (None Int))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"present\", \"absent\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("decision.rule.selected"))
            .filter(|event| event.contains("TOPAL-DECISION-OPTIONAL-001"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("optional.payload.bound"))
            .count(),
        1
    );
}

#[test]
fn optional_equality_uses_nominal_payload_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "((None Int) = (None Int), (Some 7) = (Some 7), (Some 7) = (None Int), (Some 7) != (Some 8))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, true, false, true)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-TYPE-OPTIONAL-EQUALITY-001"))
            .count(),
        4
    );

    let error = Session::new()
        .evaluate("(None Int) = (None String)\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn string_character_at_returns_optional_grapheme_clusters() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "text is \"a\u{301}👩‍🔬🇸🇪\"\n(text character-at 0, text character-at 1, text character-at 2, text character-at -1, text character-at 3)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some \"a\u{301}\", Some \"👩‍🔬\", Some \"🇸🇪\", None, None)"
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-CHARACTER-AT-001"))
            .count(),
        5
    );
}

#[test]
fn optional_decisions_consume_indexed_characters() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (candidate : Optional Character) -> String\n  candidate\n    Some character then String character\n    None then \"missing\"\n(describe (\"👩‍🔬\" character-at 0), describe (\"👩‍🔬\" character-at 1))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"👩‍🔬\", \"missing\")");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-OPTIONAL-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-FROM-CHARACTER-001"))
    );
}

#[test]
fn upper_uses_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("upper \"Straße σς\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"STRASSE ΣΣ\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-UPPER-001"))
    );
}

#[test]
fn lower_uses_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("lower \"İΣ\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"i\u{307}ς\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-LOWER-001"))
    );
}

#[test]
fn case_fold_uses_full_locale_independent_unicode_mapping() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("case-fold \"Straße Σς\"\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"strasse σσ\"");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CASE-FOLD-001"))
    );
}

#[test]
fn canonical_string_equality_preserves_exact_equality_distinction() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "composed is \"é\"\ndecomposed is \"e\u{301}\"\n(composed = decomposed, composed canonically-equals decomposed, composed canonically-equals \"e\")\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(false, true, false)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("TOPAL-STRING-CANONICAL-EQUALITY-001"))
            .count(),
        2
    );
}

#[test]
fn character_traversal_collects_the_exact_preserved_string() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("characters \"a\u{301}👩‍🔬🇸🇪\" collect String\n", &mut trace)
        .unwrap();
    assert_eq!(value.to_string(), "\"a\u{301}👩‍🔬🇸🇪\"");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CHARACTERS-COLLECT-001"))
    );
}

#[test]
fn foreach_consumes_character_generator_with_unit() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "characters \"a\u{301}👩‍🔬🇸🇪\" foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.resumed"))
            .count(),
        3
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.returned") && event.contains("Unit"))
    );
}

#[test]
fn foreach_rejects_non_unit_action_result() {
    let error = Session::new()
        .evaluate(
            "characters \"Topal\" foreach { character }\n  String character\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FOREACH-ACTION-RESULT");
}

#[test]
fn named_character_generator_is_consumed_linearly() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "generated is characters \"a\u{301}👩‍🔬🇸🇪\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.started"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.consumed"))
    );
}

#[test]
fn character_generator_accepts_its_explicit_classifier() {
    let value = Session::new()
        .evaluate(
            "generated : Generator Character Unit Unit is characters \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
}

#[test]
fn function_returns_fresh_character_generator() {
    let value = Session::new()
        .evaluate(
            "generate is fn (text : String) -> Generator Character Unit Unit\n  characters text\ngenerated is generate \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
}

#[test]
fn reused_character_generator_reports_consumption() {
    let error = Session::new()
        .evaluate(
            "generated is characters \"Topal\"\ngenerated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-CONSUMED");
    assert_eq!(
        error.help.as_deref(),
        Some("construct a fresh generator before traversing it again")
    );
}

#[test]
fn generator_parameter_transfers_linear_continuation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "consume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is characters \"Topal\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.parameter.transferred"))
    );
}

#[test]
fn abandoned_generator_parameter_closes_at_function_boundary() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "ignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\ngenerated is characters \"Topal\"\nignore generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-STRING-CHARACTERS-CLOSE-001"))
    );
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-GENERATOR-ERROR-CODE-001")
            && event.contains("domain=root")
            && event.contains("generator=root.characters")
    }));
}

#[test]
fn generator_error_code_has_qualified_nominal_identity() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "code is lang generator generator-closed\n(code, code = (lang generator generator-closed))\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(generator-closed, true)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-ERROR-CODE-001"))
    );
}

#[test]
fn named_single_yield_generator_is_traversable() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is yield initial\n  ()\ngenerated is once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.declared"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.started"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        2
    );
}

#[test]
fn named_generator_yield_reads_local_binding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "copy-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  copy : Character is initial\n  _ is yield copy\n  ()\ngenerated is copy-once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(trace.iter().any(|event| event.contains("binding.bind")));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        1
    );
}

#[test]
fn named_generator_can_return_before_first_yield() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "nothing is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  ()\ngenerated is nothing \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("generator.yielded"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.returned"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-EARLY-RETURN-001"))
    );
}

#[test]
fn named_generator_returns_character_after_yields() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-then-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\ngenerated is yield-then-return \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_defers_post_yield_binding_until_resume() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-twice is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  copy : Character is initial\n  _ is yield copy\n  ()\ngenerated is pause-twice \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let local = trace
        .iter()
        .position(|event| event.contains("binding.bind") && event.contains("copy"))
        .unwrap();
    let second_suspend = trace
        .iter()
        .rposition(|event| event.contains("generator.suspended"))
        .unwrap();
    assert!(resumed < local && local < second_suspend);
}

#[test]
fn custom_generator_binds_unit_resume_after_yield() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "bind-resume is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resumed is yield initial\n  resumed\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let bound = trace
        .iter()
        .position(|event| event.contains("generator.resume.bound"))
        .unwrap();
    let resolved = trace
        .iter()
        .rposition(|event| event.contains("binding.resolved") && event.contains("resumed"))
        .unwrap();
    assert!(resumed < bound && bound < resolved);
}

#[test]
fn abandoned_custom_generator_keeps_domain_separate_from_provenance() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is pause-once initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert!(trace.iter().any(|event| {
        event.contains("domain=root;code=generator-closed;generator=root.pause-once")
    }));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-CLOSE-001"))
    );
}

#[test]
fn abandoned_custom_generator_handles_close_result() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "handle-close is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resume-result is yield initial\n  resume-result\n    Error problem then ()\n    Ok resumed then ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is handle-close initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.close.bound"))
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
}

#[test]
fn custom_generator_matches_qualified_close_code() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "handle-code is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  result is yield initial\n  result\n    Error ( code is lang generator generator-closed ) then ()\n    Error problem then ()\n    Ok resumed then ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is handle-code initial\n  ()\nabandon \"T\"\n",
            &mut trace,
        )
        .unwrap();
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
}

#[test]
fn function_transfers_custom_generator_result_to_caller() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nmake is fn ( initial : Character ) -> Generator Character Unit Unit\n  pause-once initial\ngenerated is make \"T\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"))
    );
    assert!(
        !trace.iter().any(|event| {
            event.contains("generator.closed") && event.contains("root.pause-once")
        })
    );
}

#[test]
fn function_parameter_receives_custom_generator_ownership() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nconsume is fn ( generated : Generator Character Unit Unit ) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is pause-once \"T\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"))
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        1
    );
}

#[test]
fn function_closes_unconsumed_custom_generator_parameter() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "pause-once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\nignore is fn ( generated : Generator Character Unit Unit ) -> Unit\n  ()\ngenerated is pause-once \"T\"\nignore generated\n",
            &mut trace,
        )
        .unwrap();
    assert!(trace.iter().any(|event| {
        event.contains("TOPAL-GENERATOR-CLOSE-001") && event.contains("root.pause-once")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("domain=root;code=generator-closed;generator=root.pause-once")
    }));
}

#[test]
fn function_parameter_preserves_generator_final_character() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\nconsume is fn ( generated : Generator Character Unit Character ) -> Character\n  generated foreach { character }\n    _ is String character\ngenerated is yield-return \"Y\"\nconsume generated\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-PARAMETER-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn function_result_preserves_generator_final_character() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "yield-return is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Character\n\n  _ is yield initial\n  \"R\"\nmake is fn ( initial : Character ) -> Generator Character Unit Character\n  yield-return initial\ngenerated is make \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("R".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FUNCTION-RESULT-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_accepts_string_initial_input() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "from-text is generator ( initial : String )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  initial-is-empty : Boolean is empty? initial\n  _ is yield \"T\"\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("root.empty?(String)"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.suspended"))
    );
}

#[test]
fn custom_generator_yields_strings() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "texts is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is yield \"\"\n  ()\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Unit);
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.yielded"))
            .count(),
        2
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator String Unit Unit"))
    );
}

#[test]
fn custom_generator_returns_distinct_string() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "text-result is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator String Unit String"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-FINAL-RETURN-001"))
    );
}

#[test]
fn custom_generator_returns_explicitly_before_yielding() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "done is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  return \"done\"\ngenerated is done \"unused\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-GENERATOR-EXPLICIT-RETURN-001"))
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("generator.yielded"))
    );
}

#[test]
fn custom_generator_returns_explicitly_after_resuming() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "finish is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  return \"done\"\ngenerated is finish \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("generator.return.explicit"))
        .unwrap();
    assert!(resumed < returned);
}
