#[test]
fn models_contextual_exact_infinities_and_explicit_int_range_endpoints() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-COMPARE-001,
    // TOPAL-NUM-INFINITY-ARITHMETIC-001,
    // TOPAL-NUM-THREE-WAY-COMPARE-001, TOPAL-RANGE-BOUNDS-001,
    // TOPAL-RANGE-INTERSECTION-001, TOPAL-COMPILER-INFINITY-001
    let source = include_str!("../../../../../examples/language/infinity-values-and-ranges.t");
    let program = analyze_for_compiler(source).unwrap();
    let bindings = program
        .main
        .statements
        .iter()
        .filter_map(|statement| match statement {
            CompilerStatement::Binding(binding) => Some(binding),
            CompilerStatement::Discard(_) => None,
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        bindings[0].value,
        CompilerExpression {
            kind: CompilerExpressionKind::Infinity { negative: true },
            value_type: CompilerType::InfiniteInt,
            ..
        }
    ));
    assert!(matches!(
        bindings[1].value,
        CompilerExpression {
            kind: CompilerExpressionKind::Infinity { negative: false },
            value_type: CompilerType::InfiniteInt,
            ..
        }
    ));
    assert!(matches!(
        bindings[2].value,
        CompilerExpression {
            kind: CompilerExpressionKind::Infinity { negative: false },
            value_type: CompilerType::InfiniteNat,
            ..
        }
    ));
    assert_eq!(
        bindings[3].value.value_type,
        CompilerType::Range(Box::new(CompilerType::InfiniteInt))
    );
    assert!(matches!(
        program.main.result.value_type,
        CompilerType::Tuple(_)
    ));
    let arithmetic =
        analyze_for_compiler("use language (version is v0.1)\nupper : Int is +Infinity\nupper + 1")
            .unwrap();
    assert_eq!(arithmetic.main.result.value_type, CompilerType::InfiniteInt);
    assert_eq!(
        compiler_infinity_direction(&arithmetic.main.result),
        Some(false)
    );

    for (invalid, expected) in [
        (
            "use language (version is v0.1)\n+Infinity",
            "E-INFINITY-CONTEXT",
        ),
        (
            "use language (version is v0.1)\ninvalid : Nat is -Infinity\ninvalid",
            "E-INFINITY-CLASSIFIER",
        ),
        (
            "use language (version is v0.1)\npub exposed : Int is +Infinity\nexposed",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\nlower : Int is -Infinity\nupper : Int is +Infinity\nwhole is lower ..= upper\ncontains-zero is fn () -> Boolean\n  0 in @ whole\ncontains-zero ()",
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(
            analyze_for_compiler(invalid).unwrap_err().code,
            expected,
            "unexpected diagnostic for {invalid:?}"
        );
    }
}

#[test]
fn models_contextual_rational_infinities_and_exact_range_endpoints() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-COMPARE-001,
    // TOPAL-NUM-THREE-WAY-COMPARE-001, TOPAL-RANGE-RATIONAL-001,
    // TOPAL-COMPILER-INFINITY-001
    let source =
        include_str!("../../../../../examples/language/rational-infinity-values-and-ranges.t");
    let program = analyze_for_compiler(source).unwrap();
    let bindings = program
        .main
        .statements
        .iter()
        .filter_map(|statement| match statement {
            CompilerStatement::Binding(binding) => Some(binding),
            CompilerStatement::Discard(_) => None,
        })
        .collect::<Vec<_>>();
    for binding in &bindings[0..3] {
        assert!(matches!(
            binding.value,
            CompilerExpression {
                kind: CompilerExpressionKind::Infinity { .. },
                value_type: CompilerType::InfiniteRational,
                ..
            }
        ));
    }
    assert_eq!(
        bindings[5].value.value_type,
        CompilerType::Range(Box::new(CompilerType::InfiniteRational))
    );
    assert_eq!(
        bindings[6].value.value_type,
        CompilerType::Range(Box::new(CompilerType::InfiniteRational))
    );
    assert!(matches!(
        program.main.result.value_type,
        CompilerType::Tuple(_)
    ));

    for (invalid, expected) in [
        (
            "use language (version is v0.1)\nupper : Rational is +Infinity\nupper + Rational (1, 1)",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\ninteger : Int is +Infinity\nratio : Rational is +Infinity\ninteger = ratio",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\npub exposed : Rational is +Infinity\nexposed",
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(
            analyze_for_compiler(invalid).unwrap_err().code,
            expected,
            "unexpected diagnostic for {invalid:?}"
        );
    }
}

#[test]
fn models_total_exact_infinity_arithmetic_and_static_indeterminate_rejection() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-COMPILER-INFINITY-001
    let source = include_str!("../../../../../examples/language/infinity-arithmetic.t");
    let program = analyze_for_compiler(source).unwrap();
    let bindings = program
        .main
        .statements
        .iter()
        .filter_map(|statement| match statement {
            CompilerStatement::Binding(binding) => Some(binding),
            CompilerStatement::Discard(_) => None,
        })
        .collect::<Vec<_>>();
    for (binding, value_type, negative) in [
        (&bindings[5], CompilerType::InfiniteInt, true),
        (&bindings[6], CompilerType::InfiniteInt, false),
        (&bindings[7], CompilerType::InfiniteRational, false),
        (&bindings[8], CompilerType::InfiniteRational, true),
    ] {
        assert_eq!(binding.value.value_type, value_type);
        assert_eq!(compiler_infinity_direction(&binding.value), Some(negative));
    }

    for (invalid, expected) in [
        (
            "use language (version is v0.1)\nupper : Int is +Infinity\nlower : Int is -Infinity\nupper + lower",
            "E-INDETERMINATE-INFINITY",
        ),
        (
            "use language (version is v0.1)\nupper : Int is +Infinity\nupper - upper",
            "E-INDETERMINATE-INFINITY",
        ),
        (
            "use language (version is v0.1)\nupper : Int is +Infinity\n0 * upper",
            "E-INDETERMINATE-INFINITY",
        ),
        (
            "use language (version is v0.1)\ninteger : Int is +Infinity\nratio : Rational is +Infinity\ninteger + ratio",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\nupper : Int is +Infinity\nupper / 2",
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(
            analyze_for_compiler(invalid).unwrap_err().code,
            expected,
            "unexpected diagnostic for {invalid:?}"
        );
    }
}

#[test]
fn models_private_infinity_function_and_aggregate_boundaries() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-INFINITY-ARITHMETIC-001,
    // TOPAL-COMPILER-INFINITY-001
    let source = include_str!("../../../../../examples/language/infinity-private-boundaries.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(program.functions.len(), 9);

    let identity_int = program
        .functions
        .iter()
        .find(|function| function.source_name == "identity-int")
        .unwrap();
    assert_eq!(
        identity_int.parameters[0].value_type,
        CompilerType::InfiniteInt
    );
    assert_eq!(identity_int.result_type, CompilerType::InfiniteInt);

    let identity_nat = program
        .functions
        .iter()
        .find(|function| function.source_name == "identity-nat")
        .unwrap();
    assert_eq!(
        identity_nat.parameters[0].value_type,
        CompilerType::InfiniteNat
    );
    assert_eq!(identity_nat.result_type, CompilerType::InfiniteNat);

    let identity_rational = program
        .functions
        .iter()
        .find(|function| function.source_name == "identity-rational")
        .unwrap();
    assert_eq!(
        identity_rational.parameters[0].value_type,
        CompilerType::InfiniteRational
    );
    assert_eq!(
        identity_rational.result_type,
        CompilerType::InfiniteRational
    );

    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    let pair_type = CompilerType::Tuple(vec![
        CompilerType::InfiniteInt,
        CompilerType::InfiniteRational,
    ]);
    assert_eq!(return_pair.parameters[0].value_type, pair_type);
    assert_eq!(
        return_pair.result_type,
        return_pair.parameters[0].value_type
    );

    let capture_positive = program
        .functions
        .iter()
        .find(|function| function.source_name == "capture-positive")
        .unwrap();
    assert_eq!(
        capture_positive.parameters[0].value_type,
        CompilerType::InfiniteInt
    );
    assert_eq!(capture_positive.result_type, CompilerType::InfiniteInt);

    let widened_natural = program
        .functions
        .iter()
        .filter(|function| function.source_name == "identity-int")
        .nth(1)
        .unwrap();
    assert_eq!(
        widened_natural.parameters[0].value_type,
        CompilerType::InfiniteInt
    );
    assert_eq!(widened_natural.result_type, CompilerType::InfiniteInt);

    assert!(matches!(
        program.main.result.value_type,
        CompilerType::Tuple(ref fields)
            if fields.iter().any(compiler_type_contains_infinity)
    ));

    let recursive = "use language (version is v0.1)\ncount is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise count (value - 1)\nupper : Int is +Infinity\ncount upper";
    assert_eq!(
        analyze_for_compiler(recursive).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}
