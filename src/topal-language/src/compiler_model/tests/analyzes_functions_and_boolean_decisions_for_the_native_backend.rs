use super::*;

#[test]
fn analyzes_functions_and_boolean_decisions_for_the_native_backend() {
    let source = "use language (version is v0.1)\nchoose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise 0\n(choose true, choose false)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.main.result.value_type.name(), "(Int, Int)");
}

#[test]
fn erases_valid_diagnostic_controls_after_shared_validation() {
    // TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001, TOPAL-SYN-DIAG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/diagnostic-controls.t"
    ))
    .unwrap();
    assert_eq!(program.main.statements.len(), 2);
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

    let invalid = "use language (version is v0.1)\nlang push-disable-warning unclosed\n()\n";
    assert_eq!(
        analyze_for_compiler(invalid).unwrap_err().code,
        "E-DIAGNOSTIC-CONTROL-UNCLOSED"
    );
}

#[test]
fn preserves_arbitrary_integer_ranges_for_the_native_backend() {
    let source = "use language (version is v0.1)\n9223372036854775807 + 1\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.int_range,
        Some(IntRange::exact(BigInt::from(i64::MAX) + 1))
    );
}

#[test]
fn models_finite_exact_conversion_division_and_power() {
    let source = "use language (version is v0.1)\n(1 + 0.5, 6 / 8, -17 % 5, 2 ^ 16, 1 <=> 2)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Rational, Rational, Int, Int, Comparison)"
    );
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected a tuple")
    };
    assert_eq!(
        values[0].rational_value,
        Some(BigRational::new(BigInt::from(3), BigInt::from(2)))
    );
    assert_eq!(values[2].int_range, Some(IntRange::exact(BigInt::from(3))));
    assert_eq!(
        values[3].int_range,
        Some(IntRange::exact(BigInt::from(65_536)))
    );
}

#[test]
fn models_dynamic_nat_power_exponents() {
    let source = "use language (version is v0.1)\npower-of-two is fn (exponent : Nat) -> Int\n  2 ^ exponent\npower-of-two 3\n";
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "power-of-two")
        .expect("power specialization is emitted");
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Power,
        right,
        ..
    } = &function.body.result.kind
    else {
        panic!("expected the power operation")
    };
    assert_eq!(right.value_type, CompilerType::Nat);
}

#[test]
fn models_nat_comparison_and_retains_closed_arithmetic_evidence() {
    // TOPAL-NUM-NAT-001, TOPAL-TYPE-EQUALITY-001,
    // TOPAL-TYPE-ORDERING-001, TOPAL-NUM-THREE-WAY-COMPARE-001
    let source = include_str!("../../../../../examples/language/nat-equality-and-ordering.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Comparison, Boolean, Boolean)"
    );
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected a comparison result tuple")
    };
    let CompilerExpressionKind::Binary { left, right, .. } = &values[0].kind else {
        panic!("expected mixed Nat/Int equality")
    };
    assert_eq!(left.value_type, CompilerType::Int);
    assert_eq!(right.value_type, CompilerType::Int);
    let CompilerExpressionKind::Binary { left, right, .. } = &values[1].kind else {
        panic!("expected mixed Nat/Rational equality")
    };
    assert_eq!(left.value_type, CompilerType::Rational);
    assert_eq!(right.value_type, CompilerType::Rational);
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "compare-nat")
        .unwrap();
    let CompilerExpressionKind::Binary {
        operation,
        left,
        right,
    } = &function.body.result.kind
    else {
        panic!("expected Nat three-way comparison")
    };
    assert_eq!(*operation, CompilerBinary::Compare);
    assert_eq!(left.value_type, CompilerType::Int);
    assert_eq!(right.value_type, CompilerType::Int);
    let CompilerExpressionKind::Binary { left, right, .. } = &values[8].kind else {
        panic!("expected derived product equality")
    };
    assert_eq!(left.value_type.name(), "(Nat, Nat)");
    assert_eq!(right.value_type.name(), "(Nat, Nat)");

    let arithmetic = "use language (version is v0.1)\none : Nat is 1\none + one\n";
    let arithmetic = analyze_for_compiler(arithmetic).unwrap();
    assert_eq!(arithmetic.main.result.value_type, CompilerType::Nat);
    assert_eq!(
        arithmetic.main.result.int_range,
        Some(IntRange::exact(BigInt::from(2)))
    );
}

#[test]
fn rejects_statically_zero_exact_divisors() {
    let source = "use language (version is v0.1)\n1 / 0\n";
    assert_eq!(
        analyze_for_compiler(source).unwrap_err().code,
        "E-DIVISION-BY-ZERO"
    );
}

#[test]
fn models_finite_exact_ranges_and_observations() {
    let source = "use language (version is v0.1)\ninterval is 0 ..= 2.5\npreserve is fn (value : Range Rational) -> Range Rational\n  value\nkept is preserve interval\n(1 in kept, kept contains 3, empty? (2 .. 2), range-lower kept, range-upper-inclusive? kept, kept and (1.0 <.. 4.0))\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Boolean, Boolean, Boolean, Rational, Boolean, Range Rational)"
    );
    assert!(
        program
            .functions
            .iter()
            .any(|function| function.result_type.name() == "Range Rational")
    );
}

#[test]
fn models_ordered_and_comparison_value_decisions() {
    let source = "use language (version is v0.1)\nrank is fn (value : Comparison) -> Int\n  value\n    Less then -1\n    Equal then 0\n    Greater then 1\nlocate is fn (value : Int, pivot : Rational) -> Int\n  value\n    < pivot - 0.5 then -1\n    = pivot then 0\n    otherwise 1\n(rank (1 <=> 2), 0 locate 1.5)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert!(program.functions.iter().any(|function| matches!(
        function.body.result.kind,
        CompilerExpressionKind::ComparisonValueDecision { .. }
    )));
    assert!(program.functions.iter().any(|function| matches!(
        function.body.result.kind,
        CompilerExpressionKind::OrderedComparisonDecision { .. }
    )));
}

#[test]
fn models_dynamic_arithmetic_results_without_reclassifying_parameter_errors_as_static() {
    // TOPAL-COMPILER-RESULT-001
    let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  Rational (numerator, denominator)\nmodulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  left % right\n(1.0 divide 2.0, 1.0 divide 0.0, 1 ratio 0, 17 modulo 0)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Rational, lang arithmetic ArithmeticErrorCode), Result (Int, lang arithmetic ArithmeticErrorCode))"
    );
    assert!(program.functions.iter().any(|function| matches!(
        function.body.result.kind,
        CompilerExpressionKind::Fallible { .. }
    )));
    assert!(program.functions.iter().any(|function| matches!(
        function.body.result.kind,
        CompilerExpressionKind::ResultSuccess(_)
    )));
}

#[test]
fn models_dynamic_infinity_multiplication_as_fallible_results() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-TYPE-RESULT-001,
    // TOPAL-COMPILER-INFINITY-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/dynamic-infinity-results.t"
    ))
    .unwrap();
    let binding = |name: &str| {
        program
            .main
            .statements
            .iter()
            .find_map(|statement| match statement {
                CompilerStatement::Binding(binding) if binding.name == name => Some(binding),
                _ => None,
            })
            .unwrap_or_else(|| panic!("the shared regression binds `{name}`"))
    };

    for (name, success_type) in [
        ("int-success", CompilerType::Int),
        ("int-failure", CompilerType::Int),
        ("rational-success", CompilerType::Rational),
        ("rational-failure", CompilerType::Rational),
    ] {
        let value = &binding(name).value;
        assert_eq!(
            value.value_type,
            CompilerType::Result(Box::new(success_type))
        );
        assert!(matches!(
            value.kind,
            CompilerExpressionKind::Fallible {
                operation: CompilerFallible::InfinityMultiply,
                ..
            }
        ));
    }
}

#[test]
fn models_optional_construction_boundaries_decisions_and_equality() {
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-OPTIONAL-CONTEXT-001,
    // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001
    let source = "use language (version is v0.1)\nmissing : Optional Int is None\npreserve is fn (candidate : Optional Int) -> Optional Int\n  candidate\nabsent is fn () -> Optional Int\n  None\ndescribe is fn (candidate : Optional Int) -> String\n  candidate\n    Some payload then \"present\"\n    None then \"absent\"\n(Some 42, Some \"present\", None Int, None String, missing, preserve (Some 7), preserve (None Int), absent (), describe (Some 7), describe missing, (None Int) = (None Int), (Some 7) != (Some 8))\n";
    let program = analyze_for_compiler(source).unwrap();
    assert!(program.functions.iter().any(|function| {
        function.source_name == "preserve"
            && function.parameters[0].value_type
                == CompilerType::Optional(Box::new(CompilerType::Int))
            && function.result_type == CompilerType::Optional(Box::new(CompilerType::Int))
    }));
    assert!(program.functions.iter().any(|function| matches!(
        function.body.result.kind,
        CompilerExpressionKind::OptionalDecision { .. }
    )));
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Tuple(_)
    ));
}

#[test]
fn models_optional_rational_values_and_equality() {
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-OPTIONAL-CONTEXT-001,
    // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001
    let source = include_str!("../../../../../examples/language/optional-rational-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let optional_rational = CompilerType::Optional(Box::new(CompilerType::Rational));
    assert!(program.functions.iter().any(|function| {
        function.source_name == "preserve"
            && function.parameters[0].value_type == optional_rational
            && function.result_type == optional_rational
    }));
    assert!(program.functions.iter().any(|function| matches!(
        function.body.result.kind,
        CompilerExpressionKind::OptionalDecision { .. }
    )));
    assert_eq!(
        program.main.result.value_type.name(),
        "(Optional Rational, Optional Rational, Boolean, Boolean, Boolean, Boolean, Boolean, String, String)"
    );
}

#[test]
fn models_static_character_constraint_evidence() {
    // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-STRING-CHARACTER-CLASSIFIER-001,
    // TOPAL-STRING-FROM-CHARACTER-001, TOPAL-TYPE-EQUALITY-001
    let source = include_str!("../../../../../examples/language/character-classification.t");
    let program = analyze_for_compiler(source).unwrap();
    assert!(program.functions.iter().any(|function| {
        function.source_name == "identity"
            && function.parameters[0].value_type == CompilerType::Character
            && function.result_type == CompilerType::Character
    }));
    assert_eq!(
        program.main.result.value_type,
        CompilerType::Tuple(vec![
            CompilerType::String,
            CompilerType::String,
            CompilerType::Boolean,
            CompilerType::Boolean,
            CompilerType::Boolean,
        ])
    );

    let invalid =
        analyze_for_compiler("use language (version is v0.1)\ninvalid : Character is \"ab\"\n")
            .unwrap_err();
    assert_eq!(invalid.code, "E-CHARACTER-CLASSIFIER");
    assert!(invalid.message.contains("contains 2"));
    let dynamic = "use language (version is v0.1)\nretain is fn (value : String) -> Character\n  value\nretain \"a\"\n";
    assert_eq!(
        analyze_for_compiler(dynamic).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_closed_character_counting_and_indexing() {
    // TOPAL-STRING-CHARACTER-COUNT-001, TOPAL-STRING-ENTRY-COUNT-001,
    // TOPAL-STRING-CHARACTER-AT-001, TOPAL-TYPE-OPTIONAL-BOUNDARY-001,
    // TOPAL-DECISION-OPTIONAL-001
    let source = include_str!("../../../../../examples/language/string-character-at.t");
    let program = analyze_for_compiler(source).unwrap();
    assert!(program.functions.iter().any(|function| {
        function.source_name == "describe"
            && function.parameters[0].value_type
                == CompilerType::Optional(Box::new(CompilerType::Character))
            && function.result_type == CompilerType::String
    }));
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected the observation tuple")
    };
    assert_eq!(values.len(), 9);
    assert!(matches!(values[0].kind, CompilerExpressionKind::Int(_)));
    assert!(matches!(values[1].kind, CompilerExpressionKind::Int(_)));
    assert!(matches!(
        values[2].kind,
        CompilerExpressionKind::OptionalSome(_)
    ));
    assert!(matches!(
        values[5].kind,
        CompilerExpressionKind::OptionalNone
    ));
}

#[test]
fn models_closed_pinned_unicode_transformations() {
    // TOPAL-STRING-UPPER-001, TOPAL-STRING-LOWER-001,
    // TOPAL-STRING-CASE-FOLD-001, TOPAL-STRING-NORMALIZE-NFC-001,
    // TOPAL-STRING-NORMALIZE-NFD-001,
    // TOPAL-STRING-CANONICAL-EQUALITY-001
    for (source, expected) in [
        (
            include_str!("../../../../../examples/language/string-uppercase.t"),
            "STRASSE ΣΣ",
        ),
        (
            include_str!("../../../../../examples/language/string-lowercase.t"),
            "i\u{307}ς",
        ),
        (
            include_str!("../../../../../examples/language/string-case-fold.t"),
            "strasse σσ",
        ),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(
            exact_string(&program.main.result).as_deref(),
            Some(expected)
        );
    }

    for source in [
        include_str!("../../../../../examples/language/string-normalization.t"),
        include_str!("../../../../../examples/language/string-normalization-nfd.t"),
        include_str!("../../../../../examples/language/string-canonical-equality.t"),
    ] {
        assert!(analyze_for_compiler(source).is_ok());
    }
    let canonical = analyze_for_compiler(include_str!(
        "../../../../../examples/language/string-canonical-equality.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(values) = &canonical.main.result.kind else {
        panic!("expected the canonical-equality tuple")
    };
    assert!(matches!(
        values[1].kind,
        CompilerExpressionKind::Boolean(true)
    ));
    assert!(matches!(
        values[2].kind,
        CompilerExpressionKind::Boolean(false)
    ));

    let specialized = analyze_for_compiler(
            "use language (version is v0.1)\ntransform is fn (value : String) -> String\n  upper value\ntransform \"a\"\n",
        )
        .unwrap();
    assert_eq!(
        exact_string(&specialized.functions[0].body.result).as_deref(),
        Some("A")
    );

    for source in [
        "use language (version is v0.1)\nidentity is fn (value : String) -> String\n  value\nupper (identity \"a\")\n",
        "use language (version is v0.1)\nidentity is fn (value : String) -> String\n  value\n(identity \"a\") normalize NFC\n",
        "use language (version is v0.1)\nidentity is fn (value : String) -> String\n  value\n(identity \"a\") canonically-equals \"a\"\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_anonymous_record_construction_and_selection() {
    // TOPAL-TYPE-PRODUCT-001, TOPAL-COMPILER-RECORD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/strings-and-products.t"
    ))
    .unwrap();
    let person = program
        .main
        .statements
        .iter()
        .find_map(|statement| match statement {
            CompilerStatement::Binding(binding) if binding.name == "person" => Some(binding),
            _ => None,
        })
        .expect("the shared regression binds person");
    let CompilerExpressionKind::Record(fields) = &person.value.kind else {
        panic!("expected a checked anonymous Record")
    };
    assert_eq!(
        fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["name", "active"]
    );
    let CompilerType::Record(field_types) = &person.value.value_type else {
        panic!("expected an inferred Record type")
    };
    assert_eq!(
        field_types
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["active", "name"]
    );
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::RecordField { label, .. },
                value_type: CompilerType::String,
                ..
            },
            ..
        }) if name == "person-name" && label == "name"
    )));

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\ntext is \"outer\"\nrecord is (name is text)\n{\n  text is \"inner\"\n  upper (record name)\n}\n",
        )
        .unwrap();
    let CompilerExpressionKind::Block(block) = &shadowed.main.result.kind else {
        panic!("expected a lexical block")
    };
    assert_eq!(exact_string(&block.result).as_deref(), Some("OUTER"));

    let code_field = analyze_for_compiler(
        "use language (version is v0.1)\nrecord is (code is 7)\nrecord code\n",
    )
    .unwrap();
    assert_eq!(exact_int(&code_field.main.result), Some(BigInt::from(7)));

    let duplicate =
        analyze_for_compiler("use language (version is v0.1)\nvalue is (a is 1, a is 2)\nvalue\n")
            .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-RECORD-FIELD");
    let absent =
        analyze_for_compiler("use language (version is v0.1)\nvalue is (a is 1)\nvalue b\n")
            .unwrap_err();
    assert_eq!(absent.code, "E-NO-SUCH-RECORD-FIELD");
}

#[test]
fn models_immutable_record_reconstruction() {
    // TOPAL-TYPE-RECONSTRUCT-001, TOPAL-COMPILER-RECONSTRUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/record-reconstruction.t"
    ))
    .unwrap();
    let updated = program
        .main
        .statements
        .iter()
        .find_map(|statement| match statement {
            CompilerStatement::Binding(binding) if binding.name == "updated" => Some(binding),
            _ => None,
        })
        .expect("the shared regression binds updated");
    let CompilerExpressionKind::RecordReconstruct { base, replacements } = &updated.value.kind
    else {
        panic!("expected checked Record reconstruction")
    };
    assert_eq!(updated.value.value_type, base.value_type);
    assert_eq!(replacements.len(), 1);
    assert_eq!(replacements[0].0, "age");
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected the observation tuple")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(36)));
    assert_eq!(exact_int(&results[2]), Some(BigInt::from(37)));

    let converted = analyze_for_compiler(
            "use language (version is v0.1)\nrecord is (score is Rational (1, 2))\nupdated is record with (score is 1)\nupdated score\n",
        )
        .unwrap();
    let CompilerStatement::Binding(converted) = &converted.main.statements[1] else {
        panic!("expected updated binding")
    };
    let CompilerExpressionKind::RecordReconstruct { replacements, .. } = &converted.value.kind
    else {
        panic!("expected converted reconstruction")
    };
    assert!(matches!(
        replacements[0].1.kind,
        CompilerExpressionKind::IntToRational(_)
    ));

    for (source, code) in [
        (
            "use language (version is v0.1)\n1 with (a is 2)\n",
            "E-RECONSTRUCT-NON-RECORD",
        ),
        (
            "use language (version is v0.1)\nrecord is (a is 1)\nrecord with (a is 2, a is 3)\n",
            "E-DUPLICATE-RECONSTRUCTION-FIELD",
        ),
        (
            "use language (version is v0.1)\nrecord is (a is 1)\nrecord with (b is 2)\n",
            "E-NO-SUCH-RECORD-FIELD",
        ),
        (
            "use language (version is v0.1)\nrecord is (a is 1)\nrecord with (a is \"wrong\")\n",
            "E-TYPE-MISMATCH",
        ),
    ] {
        assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
    }
}

#[test]
fn models_recursive_structural_comparisons() {
    // TOPAL-TYPE-EQUALITY-001, TOPAL-TYPE-ORDERING-001,
    // TOPAL-NUM-INT-RATIONAL-CONVERT-001,
    // TOPAL-COMPILER-STRUCTURAL-COMPARISON-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/equality-and-ordering.t"
    ))
    .unwrap();
    let binding = |name: &str| {
        program
            .main
            .statements
            .iter()
            .find_map(|statement| match statement {
                CompilerStatement::Binding(binding) if binding.name == name => Some(binding),
                _ => None,
            })
            .unwrap_or_else(|| panic!("the shared regression binds `{name}`"))
    };
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Less,
        left,
        right,
    } = &binding("ordered").value.kind
    else {
        panic!("expected derived tuple ordering")
    };
    let ordered_type = CompilerType::Tuple(vec![
        CompilerType::Rational,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]),
    ]);
    assert_eq!(left.value_type, ordered_type);
    assert_eq!(right.value_type, ordered_type);

    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Equal,
        left,
        right,
    } = &binding("same-record").value.kind
    else {
        panic!("expected derived Record equality")
    };
    let record_type = CompilerType::Record(vec![
        ("name".into(), CompilerType::String),
        ("score".into(), CompilerType::Rational),
    ]);
    assert_eq!(left.value_type, record_type);
    assert_eq!(right.value_type, record_type);
    let CompilerExpressionKind::Record(left_fields) = &left.kind else {
        panic!("expected the left Record")
    };
    let CompilerExpressionKind::Record(right_fields) = &right.kind else {
        panic!("expected the right Record")
    };
    assert_eq!(left_fields[0].0, "name");
    assert_eq!(right_fields[0].0, "score");

    let shape_error =
        analyze_for_compiler("use language (version is v0.1)\n(a is 1) = (b is 1)\n").unwrap_err();
    assert_eq!(shape_error.code, "E-NO-APPLICABLE-OVERLOAD");
    for source in [
        "use language (version is v0.1)\n(value is (1 .. 2)) = (value is (1 .. 2))\n",
        "use language (version is v0.1)\n(true, 1) < (false, 1)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }
}

#[test]
fn models_prospective_utf8_string_byte_counts() {
    // TOPAL-TYPE-CALL-001, TOPAL-STRING-UTF8-BYTE-COUNT-001
    let source = include_str!("../../../../../examples/language/string-utf8-byte-count.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program
            .main
            .statements
            .iter()
            .filter(|statement| matches!(
                statement,
                CompilerStatement::Binding(CompilerBinding {
                    value: CompilerExpression {
                        kind: CompilerExpressionKind::StringUtf8ByteCount(_),
                        ..
                    },
                    ..
                })
            ))
            .count(),
        3
    );
    assert_eq!(program.main.result.value_type.name(), "(Int, Int, Int)");
}

#[test]
fn models_exact_string_and_derived_optional_string_equality() {
    // TOPAL-TYPE-EQUALITY-001, TOPAL-TYPE-OPTIONAL-EQUALITY-001
    let source = include_str!("../../../../../examples/language/string-exact-equality.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean)"
    );
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected equality result tuple")
    };
    assert!(values.iter().all(|value| matches!(
        value.kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal | CompilerBinary::NotEqual,
            ..
        }
    )));

    let unsupported_field = "use language (version is v0.1)\nleft is (1 .. 2, true)\nright is (1 .. 2, true)\nleft = right\n";
    assert_eq!(
        analyze_for_compiler(unsupported_field).unwrap_err().code,
        "E-NO-APPLICABLE-OVERLOAD"
    );
}

#[test]
fn models_string_construction_concatenation_and_emptiness() {
    // TOPAL-STRING-EMPTY-001, TOPAL-STRING-LITERAL-COMPOSE-001,
    // TOPAL-STRING-CONCAT-001, TOPAL-STRING-EMPTY-PREDICATE-001
    let source = include_str!("../../../../../examples/language/string-construction.t");
    let program = analyze_for_compiler(source).unwrap();
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::StringEmpty,
                ..
            },
            ..
        })
    )));
    assert_eq!(
        program
            .main
            .statements
            .iter()
            .filter(|statement| matches!(
                statement,
                CompilerStatement::Binding(CompilerBinding {
                    value: CompilerExpression {
                        kind: CompilerExpressionKind::StringConcat { .. },
                        ..
                    },
                    ..
                })
            ))
            .count(),
        4
    );
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::String(value),
                ..
            },
            ..
        }) if value == "adjacent literals"
    )));
    assert_eq!(
        program.main.result.value_type.name(),
        "(String, Boolean, Boolean, Boolean, String, String, String, String, String)"
    );
}

#[test]
fn models_recursive_positional_product_equality() {
    // TOPAL-TYPE-PRODUCT-001, TOPAL-TYPE-EQUALITY-001
    let source = include_str!("../../../../../examples/language/tuple-equality.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Boolean, Boolean, Boolean, Boolean, Boolean)"
    );
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected equality result tuple")
    };
    assert!(values.iter().all(|value| matches!(
        value.kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal | CompilerBinary::NotEqual,
            ..
        }
    )));
}

#[test]
fn rejects_equality_between_distinct_optional_classifiers() {
    // TOPAL-TYPE-OPTIONAL-EQUALITY-001
    let source = "use language (version is v0.1)\n(None Int) = (None String)\n";
    assert_eq!(
        analyze_for_compiler(source).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
}

#[test]
fn models_optional_otherwise_as_the_missing_alternative_without_a_payload_binding() {
    // TOPAL-DECISION-OPTIONAL-001
    let source = "use language (version is v0.1)\ndescribe is fn (candidate : Optional Int) -> Int\n  candidate\n    None then 0\n    otherwise 1\n(describe (Some 42), describe (None Int))\n";
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "describe")
        .unwrap();
    let CompilerExpressionKind::OptionalDecision { some_binding, .. } = &function.body.result.kind
    else {
        panic!("expected Optional decision")
    };
    assert!(some_binding.is_none());
}

#[test]
fn models_exact_validation_and_contextual_result_projection() {
    // TOPAL-COMPILER-RESULT-001
    let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\nincrement is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  quotient : Rational is 1.0 divide denominator\n  quotient + 1.0\nas-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)\n  Int value\nas-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)\n  Nat value\n(increment 0.0, as-int 1.5, as-nat -1)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert!(program.functions.iter().any(|function| {
        function.body.statements.iter().any(|statement| {
            matches!(
                statement,
                CompilerStatement::Binding(CompilerBinding {
                    value: CompilerExpression {
                        kind: CompilerExpressionKind::ResultProject(_),
                        ..
                    },
                    ..
                })
            )
        })
    }));

    let repeated = "use language (version is v0.1)\nColor is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"first\"\n    Red then 42\n    Green then \"green\"\nname Red\n";
    let repeated = analyze_for_compiler(repeated).unwrap();
    assert!(repeated.functions.iter().any(|function| matches!(
        &function.body.result.kind,
        CompilerExpressionKind::EnumDecision { rules, .. } if rules.len() == 2
    )));
    assert!(program.functions.iter().any(|function| matches!(
        function.body.result.kind,
        CompilerExpressionKind::Validate { .. }
    )));
}

#[test]
fn models_result_decisions_and_structured_error_observation() {
    // TOPAL-DECISION-RESULT-001, TOPAL-DECISION-ERROR-CODE-001,
    // TOPAL-ERROR-FIELD-001
    let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error problem then \"other\"\nexhaustive is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic out-of-range) then \"range\"\n    Error (code is lang arithmetic not-representable) then \"representation\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error (code is lang arithmetic indeterminate) then \"indeterminate\"\nproblem is 1.0 divide 0.0\n(describe 0.0, exhaustive 0.0, problem code, problem domain)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(String, String, lang arithmetic ArithmeticErrorCode, ErrorDomain)"
    );
    assert!(program.functions.iter().any(|function| matches!(
        &function.body.result.kind,
        CompilerExpressionKind::ResultDecision {
            error_codes,
            error_fallback: Some(_),
            ..
        } if error_codes.len() == 1
    )));
    assert!(program.functions.iter().any(|function| matches!(
        &function.body.result.kind,
        CompilerExpressionKind::ResultDecision {
            error_codes,
            error_fallback: None,
            ..
        } if error_codes.len() == 4
    )));
    let CompilerExpressionKind::Tuple(fields) = &program.main.result.kind else {
        panic!("expected top-level tuple")
    };
    assert!(matches!(
        fields[2].kind,
        CompilerExpressionKind::ErrorField {
            field: CompilerErrorField::Code,
            ..
        }
    ));
    assert!(matches!(
        fields[3].kind,
        CompilerExpressionKind::ErrorField {
            field: CompilerErrorField::Domain,
            ..
        }
    ));
}

#[test]
fn models_optional_structured_error_fields() {
    // TOPAL-COMPILER-ERROR-OPTIONAL-FIELDS-001, TOPAL-ERROR-FIELD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/optional-result-composition.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(fields) = &program.main.result.kind else {
        panic!("expected top-level tuple")
    };
    for (index, field, payload) in [
        (4, CompilerErrorField::Detail, CompilerType::String),
        (5, CompilerErrorField::Cause, CompilerType::Error),
        (6, CompilerErrorField::Source, CompilerType::SourceLocation),
    ] {
        assert_eq!(
            fields[index].value_type,
            CompilerType::Optional(Box::new(payload))
        );
        assert!(matches!(
            fields[index].kind,
            CompilerExpressionKind::ErrorField { field: actual, .. } if actual == field
        ));
    }
}

#[test]
fn models_qualified_arithmetic_error_code_values() {
    // TOPAL-NUM-ARITHMETIC-ERROR-001
    let source = "use language (version is v0.1)\nretain is fn (value : ErrorCode) -> ErrorCode\n  value\n(retain (lang arithmetic division-by-zero), lang arithmetic indeterminate, (lang arithmetic out-of-range) = (lang arithmetic out-of-range))\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(lang arithmetic ArithmeticErrorCode, lang arithmetic ArithmeticErrorCode, Boolean)"
    );
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected a tuple")
    };
    assert!(matches!(
        values[1].kind,
        CompilerExpressionKind::ErrorCode(3)
    ));
}

#[test]
fn models_qualified_generator_error_code_as_a_nominal_value() {
    // TOPAL-GENERATOR-ERROR-CODE-001,
    // TOPAL-COMPILER-GENERATOR-ERROR-CODE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/generator-error-codes.t"
    ))
    .unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(lang generator GeneratorErrorCode, Boolean)"
    );
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::Enum(0),
                value_type: CompilerType::Enum(enumeration),
                ..
            },
            ..
        }) if enumeration.name == "lang generator GeneratorErrorCode"
            && enumeration.alternatives == ["generator-closed"]
    )));

    let unknown = "use language (version is v0.1)\nlang generator generator-reopened\n";
    assert_eq!(
        analyze_for_compiler(unknown).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let arithmetic_mismatch = "use language (version is v0.1)\n(lang generator generator-closed) = (lang arithmetic division-by-zero)\n";
    assert_eq!(
        analyze_for_compiler(arithmetic_mismatch).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
}

#[test]
fn models_lazy_int_iterate_construction_without_invoking_captured_functions() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001
    let unbounded = analyze_for_compiler(include_str!(
        "../../../../../examples/language/iterate-generator.t"
    ))
    .unwrap();
    assert_eq!(unbounded.main.result.value_type, int_unit_generator_type());
    assert!(unbounded.main.statements.iter().any(|statement| matches!(
            statement,
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::IterateGenerator {
                        initial,
                        parameters,
                        next,
                    },
                    ..
                },
                ..
            }) if matches!(initial.kind, CompilerExpressionKind::Int(ref value) if value == &BigInt::from(0))
                && parameters.len() == 1
                && matches!(next.result.kind, CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    ..
                })
        )));

    let bounded = analyze_for_compiler(include_str!(
        "../../../../../examples/language/iterate-take-while.t"
    ))
    .unwrap();
    assert_eq!(bounded.main.result.value_type, int_unit_generator_type());
    assert!(bounded.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::GeneratorTakeWhile {
                    generator,
                    parameters,
                    predicate,
                },
                ..
            },
            ..
        }) if matches!(generator.kind, CompilerExpressionKind::IterateGenerator { .. })
            && parameters.len() == 1
            && matches!(predicate.result.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::Less,
                ..
            })
    )));

    let moved = analyze_for_compiler(
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\nmoved is numbers\nmoved\n",
        )
        .unwrap();
    assert_eq!(moved.main.result.value_type, int_unit_generator_type());

    for (source, code) in [
        (
            "use language (version is v0.1)\n0 iterate ({ value } value < 1)\n",
            "E-TYPE-MISMATCH",
        ),
        (
            "use language (version is v0.1)\n0 iterate ({ value } value + 1) take-while ({ value } value + 1)\n",
            "E-TYPE-MISMATCH",
        ),
        (
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\n(numbers, numbers)\n",
            "E-GENERATOR-CONSUMED",
        ),
        (
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\n()\n",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\n(numbers,)\n",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\nroot numbers\n",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\n(0 iterate ({ value } value + 1)) = (1 iterate ({ value } value + 1))\n",
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
    }
}

#[test]
fn models_direct_bounded_int_iterate_collection() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/generated-collect.t"
    ))
    .unwrap();
    assert_eq!(
        program.main.result.value_type,
        CompilerType::List(Box::new(CompilerType::Int))
    );
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::GeneratorCollect(generator),
                ..
            },
            ..
        }) if name == "digits"
            && matches!(generator.kind, CompilerExpressionKind::GeneratorTakeWhile {
                generator: ref source,
                ..
            } if matches!(source.kind, CompilerExpressionKind::IterateGenerator { .. }))
    )));
    analyze_for_compiler(
            "use language (version is v0.1)\ncollect ((0 iterate ({ value } value + 1)) take-while ({ value } value < 2))\n",
        )
        .unwrap();

    let unbounded = analyze_for_compiler(
        "use language (version is v0.1)\ncollect (0 iterate ({ value } value + 1))\n",
    )
    .unwrap_err();
    assert_eq!(unbounded.code, "E-UNBOUNDED-GENERATOR-COLLECT");

    let indirect = analyze_for_compiler(
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1) take-while ({ value } value < 5)\ncollect numbers\n",
        )
        .unwrap_err();
    assert_eq!(indirect.code, "E-COMPILER-UNSUPPORTED");

    for captured in [
        "use language (version is v0.1)\nstep is 1\ncollect (0 iterate ({ value } value + step) take-while ({ value } value < 5))\n",
        "use language (version is v0.1)\nlimit is 5\ncollect (0 iterate ({ value } value + 1) take-while ({ value } value < limit))\n",
    ] {
        assert_eq!(
            analyze_for_compiler(captured).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_capture_free_bounded_int_iterate_foreach() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-ITERATE-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/generated-foreach.t"
    ))
    .unwrap();
    assert_eq!(program.main.result.value_type, CompilerType::Unit);
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::IterateGeneratorForeach {
                    generator,
                    parameter,
                    body,
                },
                ..
            },
            ..
        }) if name == "completed"
            && parameter.name == "digit"
            && body.result.value_type == CompilerType::Unit
            && matches!(generator.kind, CompilerExpressionKind::GeneratorTakeWhile { .. })
    )));

    for (source, code) in [
        (
            "use language (version is v0.1)\nnumbers is 0 iterate ({ value } value + 1)\ncompleted is numbers foreach { digit }\n  _ is digit\ncompleted\n",
            "E-UNBOUNDED-GENERATOR-TRAVERSAL",
        ),
        (
            "use language (version is v0.1)\nstart is 0\ndigits is start iterate ({ value } value + 1) take-while ({ value } value < 2)\ncompleted is digits foreach { digit }\n  _ is digit\ncompleted\n",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\nstep is 1\ndigits is 0 iterate ({ value } value + step) take-while ({ value } value < 2)\ncompleted is digits foreach { digit }\n  _ is digit\ncompleted\n",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\nlimit is 1\ndigits is 0 iterate ({ value } value + 1) take-while ({ value } value < 2)\ncompleted is digits foreach { digit }\n  _ is digit + limit\ncompleted\n",
            "E-UNBOUND-NAME",
        ),
    ] {
        assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
    }
}

#[test]
fn models_closed_string_character_foreach() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/string-character-foreach.t"
    ))
    .unwrap();
    assert_eq!(program.main.result.value_type, CompilerType::Unit);
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersForeach {
                characters,
                parameter,
                body,
                ..
            },
            ..
        }) if characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
            && parameter.name == "character"
            && parameter.value_type == CompilerType::Character
            && body.result.value_type == CompilerType::Unit
    )));

    for (source, code) in [
        (
            "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ncharacters (identity \"a\") foreach { character }\n  _ is String character\n",
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            "use language (version is v0.1)\noutside is \"x\"\ncharacters \"a\" foreach { character }\n  _ is outside\n",
            "E-UNBOUND-NAME",
        ),
        (
            "use language (version is v0.1)\ncharacters \"a\" foreach { character }\n  String character\n",
            "E-TYPE-MISMATCH",
        ),
    ] {
        assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
    }

    let empty = analyze_for_compiler(
            "use language (version is v0.1)\ncharacters \"\" foreach { character }\n  _ is String character\n",
        )
        .unwrap();
    assert!(matches!(
        &empty.main.statements[0],
        CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersForeach { characters, .. },
            ..
        }) if characters.is_empty()
    ));
}

#[test]
fn models_closed_string_character_collection() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/string-character-traversal.t"
    ))
    .unwrap();
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersCollect { text, characters },
            value_type: CompilerType::String,
            ..
        } if matches!(text.kind, CompilerExpressionKind::String(_))
            && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
    ));
    assert_eq!(
        exact_string(&program.main.result).as_deref(),
        Some("a\u{301}👩‍🔬🇸🇪")
    );

    let empty =
        analyze_for_compiler("use language (version is v0.1)\ncharacters \"\" collect String\n")
            .unwrap();
    assert!(matches!(
        &empty.main.result.kind,
        CompilerExpressionKind::StringCharactersCollect { characters, .. }
            if characters.is_empty()
    ));

    let dynamic = "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ncharacters (identity \"a\") collect String\n";
    assert_eq!(
        analyze_for_compiler(dynamic).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_named_linear_string_character_generator() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-CLASSIFIER-001,
    // TOPAL-STRING-CHARACTERS-LINEAR-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-GENERATOR-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/string-named-character-generator.t"
    ))
    .unwrap();
    assert!(matches!(
        &program.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersGenerator {
                    characters,
                    ..
                },
                value_type: CompilerType::Generator(CompilerGeneratorType {
                    yield_type,
                    resume_type,
                    result_type,
                }),
                ..
            },
            ..
        }) if name == "generated"
            && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
            && yield_type.as_ref() == &CompilerType::Character
            && resume_type.as_ref() == &CompilerType::Unit
            && result_type.as_ref() == &CompilerType::Unit
    ));
    assert!(matches!(
        &program.main.statements[1],
        CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersForeach {
                source,
                characters,
                ..
            },
            ..
        }) if matches!(source.kind, CompilerExpressionKind::Local(_))
            && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
    ));

    let consumed_twice = "use language (version is v0.1)\ngenerated : Generator Character Unit Unit is characters \"a\"\ngenerated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(consumed_twice).unwrap_err().code,
        "E-GENERATOR-CONSUMED"
    );
    let abandoned = "use language (version is v0.1)\ngenerated : Generator Character Unit Unit is characters \"a\"\n";
    assert_eq!(
        analyze_for_compiler(abandoned).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let dynamic = "use language (version is v0.1)\nonce is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nidentity is fn (character : Character) -> Character\n  character\ninitial is identity \"T\"\ngenerated is once initial\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(dynamic).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_root_single_yield_custom_generator() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-single-yield-generator.t"
    ))
    .unwrap();
    assert!(matches!(
        program.main.statements.as_slice(),
        [
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::CustomCharacterGenerator {
                        declaration,
                        initial,
                        characters,
                        ..
                    },
                    value_type,
                    ..
                },
                ..
            }),
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    source,
                    characters: yielded,
                    parameter,
                    ..
                },
                value_type: CompilerType::Unit,
                ..
            })
        ] if name == "generated"
            && declaration == "once"
            && initial.value_type == CompilerType::Character
            && characters == &[String::from("T")]
            && is_character_unit_generator_type(value_type)
            && matches!(source.kind, CompilerExpressionKind::Local(_))
            && yielded == &[String::from("T")]
            && parameter.name == "character"
            && parameter.value_type == CompilerType::Character
    ));

    for source in [
        "use language (version is v0.1)\nonce is generator (initial : Character, other : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is once (\"T\", \"U\")\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nonce is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  initial\ngenerated is once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let consumed_twice = "use language (version is v0.1)\nonce is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is once \"T\"\ngenerated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(consumed_twice).unwrap_err().code,
        "E-GENERATOR-CONSUMED"
    );
    let abandoned = "use language (version is v0.1)\nonce is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is once \"T\"\n";
    assert_eq!(
        analyze_for_compiler(abandoned).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_root_multiple_yield_custom_generator() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-MULTIPLE-YIELD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-multiple-yield-generator.t"
    ))
    .unwrap();
    assert!(matches!(
        program.main.statements.as_slice(),
        [
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::CustomCharacterGenerator {
                        declaration,
                        characters,
                        ..
                    },
                    ..
                },
                ..
            }),
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    characters: yielded,
                    parameter,
                    ..
                },
                value_type: CompilerType::Unit,
                ..
            })
        ] if declaration == "twice"
            && characters == &[String::from("T"), String::from("T")]
            && yielded == &[String::from("T"), String::from("T")]
            && parameter.value_type == CompilerType::Character
    ));

    let intervening_statement = "use language (version is v0.1)\ntwice is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is 1\n  _ is yield initial\n  ()\ngenerated is twice \"T\"\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(intervening_statement)
            .unwrap_err()
            .code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_custom_generator_returning_unit_before_yield() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-EARLY-RETURN-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-EARLY-RETURN-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-early-return.t"
    ))
    .unwrap();
    assert!(matches!(
        program.main.statements.as_slice(),
        [
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::CustomCharacterGenerator {
                        declaration,
                        initial,
                        characters,
                        locals,
                        ..
                    },
                    value_type,
                    ..
                },
                ..
            }),
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    characters: yielded,
                    locals: traversal_locals,
                    parameter,
                    body,
                    ..
                },
                value_type: CompilerType::Unit,
                ..
            })
        ] if declaration == "nothing"
            && initial.value_type == CompilerType::Character
            && characters.is_empty()
            && locals.is_empty()
            && is_character_unit_generator_type(value_type)
            && yielded.is_empty()
            && traversal_locals.is_empty()
            && parameter.value_type == CompilerType::Character
            && body.result.value_type == CompilerType::Unit
    ));

    for source in [
        "use language (version is v0.1)\nnothing is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  copy : Character is initial\n  ()\ngenerated is nothing \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nnothing is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is initial\n  ()\ngenerated is nothing \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nnothing is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  initial\ngenerated is nothing \"T\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let invalid_action = "use language (version is v0.1)\nnothing is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  ()\ngenerated is nothing \"T\"\ngenerated foreach { character }\n  character\n";
    assert_eq!(
        analyze_for_compiler(invalid_action).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
}

#[test]
fn models_distinct_custom_generator_final_character() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FINAL-RETURN-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-final-character.t"
    ))
    .unwrap();
    assert!(matches!(
        (program.main.statements.as_slice(), &program.main.result),
        (
            [CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::CustomCharacterGenerator {
                        declaration,
                        initial,
                        characters,
                        result: generated_result,
                        ..
                    },
                    value_type,
                    ..
                },
                ..
            })],
            CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    characters: yielded,
                    parameter,
                    body,
                    result: traversal_result,
                    ..
                },
                value_type: CompilerType::Character,
                ..
            }
        ) if declaration == "yield-then-return"
            && exact_string(initial).as_deref() == Some("Y")
            && characters == &[String::from("Y")]
            && exact_string(generated_result).as_deref() == Some("R")
            && is_character_generator_type_with_result(value_type, &CompilerType::Character)
            && yielded == &[String::from("Y")]
            && parameter.value_type == CompilerType::Character
            && body.result.value_type == CompilerType::Unit
            && exact_string(traversal_result).as_deref() == Some("R")
    ));

    let empty_final = "use language (version is v0.1)\nyield-then-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"\"\ngenerated is yield-then-return \"Y\"\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(empty_final).unwrap_err().code,
        "E-CHARACTER-CLASSIFIER"
    );

    for source in [
        "use language (version is v0.1)\nyield-then-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  initial\ngenerated is yield-then-return \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nreturn-only is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  \"R\"\ngenerated is return-only \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nyield-then-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\ngenerated is yield-then-return \"Y\"\nreturned is generated foreach { character }\n  _ is String character\nreturned\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}
