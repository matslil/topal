#[test]
fn models_non_escaping_nested_functions_with_private_captures() {
    // TOPAL-COMPILER-NESTED-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001,
    // TOPAL-FUNCTION-NESTED-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/nested-functions.t"
    ))
    .unwrap();
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
    let nested = program
        .functions
        .iter()
        .find(|function| function.source_name == "add-input")
        .expect("nested specialization is emitted");
    assert_eq!(nested.parameters.len(), 2);
    assert_eq!(nested.parameters[0].name, "value");
    assert_eq!(nested.parameters[1].name, "input");
    assert!(
        nested
            .parameters
            .iter()
            .all(|parameter| parameter.value_type == CompilerType::Int)
    );
    let outer = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .expect("outer specialization is emitted");
    let CompilerExpressionKind::Call { arguments, .. } = &outer.body.result.kind else {
        panic!("outer body directly calls the nested specialization")
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(name) if name == "input"
    ));

    let discarded = analyze_for_compiler(
            "use language (version is v0.1)\nconsume is fn (_ : Function) -> Int\n  1\nouter is fn (input : Int) -> Int\n  helper is fn (value : Int) -> Int\n    value + input\n  consume helper\nouter 1\n",
        )
        .unwrap();
    assert_eq!(exact_int(&discarded.main.result), Some(BigInt::from(1)));

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nouter is fn (value : Int) -> Int\n  helper is fn (value : Int) -> Int\n    value\n  helper 42\nouter 1\n",
        )
        .unwrap();
    let nested = shadowed
        .functions
        .iter()
        .find(|function| function.source_name == "helper")
        .unwrap();
    assert_eq!(nested.parameters.len(), 1);
    assert_eq!(nested.parameters[0].name, "value");
    assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));
}

#[test]
fn models_complete_header_forward_function_calls() {
    // TOPAL-FUNCTION-FORWARD-DECLARATION-001, TOPAL-COMPILER-FUNCTION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/forward-function-declarations.t"
    ))
    .unwrap();
    assert_eq!(
        program
            .functions
            .iter()
            .map(|function| function.source_name.as_str())
            .collect::<Vec<_>>(),
        ["decorate", "render"]
    );
    let CompilerExpressionKind::Call { symbol, .. } = &program.functions[1].body.result.kind else {
        panic!("expected the earlier function to call the later declaration")
    };
    assert_eq!(symbol, &program.functions[0].symbol);
}

#[test]
fn models_only_structurally_proven_decreasing_int_recursion() {
    // TOPAL-FUNCTION-RECURSION-INT-001,
    // TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/decreasing-int-recursion.t"
    ))
    .unwrap();
    assert_eq!(program.functions.len(), 1);
    let function = &program.functions[0];
    assert_eq!(function.source_name, "sum-down");
    assert!(function.parameters[0].int_range.is_none());
    let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
        &function.body.result.kind
    else {
        panic!("expected the structurally proven recursion decision")
    };
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Add,
        right,
        ..
    } = &otherwise.kind
    else {
        panic!("expected the recursive action")
    };
    let CompilerExpressionKind::Call { symbol, .. } = &right.kind else {
        panic!("expected the direct recursive edge")
    };
    assert_eq!(symbol, &function.symbol);

    assert!(
        analyze_for_compiler(include_str!(
            "../../../../../examples/language/multiple-recursive-calls.t"
        ))
        .is_ok()
    );

    for source in [
        "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    <= 0 then 0\n    otherwise loop (value - 0)\nloop 1\n",
        "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    <= 0 then loop (value - 1)\n    otherwise loop (value - 1)\nloop 1\n",
        "use language (version is v0.1)\nfirst is fn (value : Int) -> Int\n  second value\nsecond is fn (value : Int) -> Int\n  first value\nfirst 1\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_only_structurally_proven_increasing_int_recursion() {
    // TOPAL-FUNCTION-RECURSION-INT-INCREASING-001,
    // TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/increasing-int-recursion.t"
    ))
    .unwrap();
    assert_eq!(program.functions.len(), 1);
    let function = &program.functions[0];
    assert_eq!(function.source_name, "distance-up");
    assert!(function.parameters[0].int_range.is_none());
    let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
        &function.body.result.kind
    else {
        panic!("expected the structurally proven recursion decision")
    };
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Add,
        right,
        ..
    } = &otherwise.kind
    else {
        panic!("expected the recursive action")
    };
    let CompilerExpressionKind::Call { symbol, .. } = &right.kind else {
        panic!("expected the direct recursive edge")
    };
    assert_eq!(symbol, &function.symbol);

    assert!(
        analyze_for_compiler(include_str!(
            "../../../../../examples/language/positive-recursion-steps.t"
        ))
        .is_ok()
    );

    for source in [
        "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise loop (value + 0)\nloop (-1)\n",
        "use language (version is v0.1)\nloop is fn (value : Int) -> Int\n  value\n    >= 0 then 0\n    otherwise loop (value - 1)\nloop (-1)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_only_range_preserving_nat_recursion() {
    // TOPAL-FUNCTION-RECURSION-NAT-001,
    // TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001
    for (source, name, operation) in [
        (
            include_str!("../../../../../examples/language/nat-recursion.t"),
            "count-down",
            CompilerBinary::Subtract,
        ),
        (
            include_str!("../../../../../examples/language/nat-increasing-recursion.t"),
            "advance",
            CompilerBinary::Add,
        ),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        assert_eq!(program.functions.len(), 1);
        let function = &program.functions[0];
        assert_eq!(function.source_name, name);
        assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
        assert_eq!(function.result_type, CompilerType::Nat);
        assert!(function.parameters[0].int_range.is_none());
        let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
            &function.body.result.kind
        else {
            panic!("expected the structurally proven Nat decision")
        };
        let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
            panic!("expected the direct recursive edge")
        };
        assert_eq!(symbol, &function.symbol);
        let [argument] = arguments.as_slice() else {
            panic!("expected one recursive Nat argument")
        };
        let CompilerExpressionKind::IntToNat(argument) = &argument.kind else {
            panic!("expected proof-backed Nat evidence")
        };
        assert!(matches!(
            argument.kind,
            CompilerExpressionKind::Binary {
                operation: actual,
                ..
            } if actual == operation
        ));
    }

    let unsafe_overshoot = "use language (version is v0.1)\nloop is fn (value : Nat) -> Nat\n  value\n    <= 0 then value\n    otherwise loop (value - 2)\nloop 3\n";
    assert!(analyze_for_compiler(unsafe_overshoot).is_err());
}

#[test]
fn models_only_structurally_verified_explicit_integer_measure() {
    // TOPAL-FUNCTION-DECREASES-001
    let source =
        include_str!("../../../../../examples/language/explicit-multi-parameter-decreases.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(program.functions.len(), 1);
    let function = &program.functions[0];
    assert_eq!(function.source_name, "repeat-add");
    assert_eq!(function.parameters.len(), 2);
    assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
    assert_eq!(function.parameters[1].value_type, CompilerType::Int);
    assert!(
        function
            .parameters
            .iter()
            .all(|parameter| parameter.int_range.is_none())
    );
    let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
        &function.body.result.kind
    else {
        panic!("expected the measured recursion decision")
    };
    let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
        panic!("expected the measured recursive edge")
    };
    assert_eq!(symbol, &function.symbol);
    let [count, total] = arguments.as_slice() else {
        panic!("expected the complete measured recursive state")
    };
    assert!(matches!(count.kind, CompilerExpressionKind::IntToNat(_)));
    assert!(matches!(
        total.kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            ..
        }
    ));

    for invalid in [
        source.replace("Decreases count", "Decreases total"),
        source.replace("count - 1", "count - 0"),
    ] {
        assert!(analyze_for_compiler(&invalid).is_err());
    }
}

#[test]
fn models_interpreter_proven_euclidean_recursion_without_fallible_modulo() {
    // TOPAL-FUNCTION-RECURSION-EUCLIDEAN-001, TOPAL-NUM-INT-MODULO-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/euclidean-gcd.t"
    ))
    .unwrap();
    let function = &program.functions[0];
    assert_eq!(function.source_name, "gcd");
    assert_eq!(function.result_type, CompilerType::Nat);
    let CompilerExpressionKind::OrderedComparisonDecision {
        rules, otherwise, ..
    } = &function.body.result.kind
    else {
        panic!("expected Euclidean comparison decision")
    };
    assert!(matches!(
        rules[0].action.kind,
        CompilerExpressionKind::IntToNat(_)
    ));
    let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
        panic!("expected direct Euclidean recursive edge")
    };
    assert_eq!(symbol, &function.symbol);
    assert!(matches!(
        arguments[1].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Modulo,
            ..
        }
    ));
}

#[test]
fn models_explicit_early_return_and_skips_the_tail() {
    // TOPAL-FUNCTION-RETURN-001
    let source = "use language (version is v0.1)\nanswer is fn static () -> Int\n  return 40 + 2\n  0\nanswer ()\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(program.main.result.value_type, CompilerType::Int);
    assert!(program.functions.iter().any(|function| {
        function.source_name == "answer"
            && matches!(
                function.body.result.kind,
                CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    ..
                }
            )
    }));

    let outside = "use language (version is v0.1)\nreturn 42\n";
    assert_eq!(
        analyze_for_compiler(outside).unwrap_err().code,
        "E-RETURN-OUTSIDE-FUNCTION"
    );
}

#[test]
fn models_unconditional_lexical_block_return_as_the_function_result() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COMPILER-LEXICAL-RETURN-001
    let source = "use language (version is v0.1)\nanswer is fn (value : Int) -> Int\n  {\n    adjusted is value + 1\n    return adjusted\n    999\n    }\n  1000\nanswer 41\n";
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    let CompilerExpressionKind::Block(block) = &function.body.result.kind else {
        panic!("expected the return-bearing lexical block to become the function result")
    };
    assert!(matches!(
        block.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding { name, .. })] if name == "adjusted"
    ));
    assert!(matches!(
        &block.result.kind,
        CompilerExpressionKind::Local(name) if name == "adjusted"
    ));
    assert!(function.body.statements.is_empty());

    let abandoned_classifier = "use language (version is v0.1)\nanswer is fn () -> Int\n  abandoned : String is {\n    return 42\n    }\n  0\nanswer ()\n";
    let program = analyze_for_compiler(abandoned_classifier).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);

    let embedded = "use language (version is v0.1)\nanswer is fn () -> Int\n  Character (String {\n    return 41\n    })\nanswer ()\n";
    let error = analyze_for_compiler(embedded).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_return_bearing_block_as_an_explicit_return_operand() {
    // TOPAL-FUNCTION-RETURN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERAND-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-block-operand.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    let CompilerExpressionKind::Block(block) = &function.body.result.kind else {
        panic!("expected the explicit return operand to retain its lexical block")
    };
    assert!(block.statements.is_empty());
    assert!(matches!(
        block.result.kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            ..
        }
    ));
    assert!(function.body.statements.is_empty());

    let normal = "use language (version is v0.1)\nanswer is fn (value : Int) -> Int\n  return { value + 1 }\nanswer 41\n";
    let program = analyze_for_compiler(normal).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
}

#[test]
fn models_return_bearing_direct_symbolic_operands_in_source_order() {
    // TOPAL-FUNCTION-RETURN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-operator-operand.t"
    ))
    .unwrap();
    let right = program
        .functions
        .iter()
        .find(|function| function.source_name == "right-exit")
        .unwrap();
    let CompilerExpressionKind::ExitSequence { preceding, result } = &right.body.result.kind else {
        panic!("expected the evaluated left operand before the right-side exit")
    };
    assert!(matches!(
        preceding.as_slice(),
        [CompilerExpression {
            kind: CompilerExpressionKind::Call { .. },
            ..
        }]
    ));
    assert!(matches!(result.kind, CompilerExpressionKind::Block(_)));
    assert!(right.body.statements.is_empty());

    let left = program
        .functions
        .iter()
        .find(|function| function.source_name == "left-exit")
        .unwrap();
    assert!(matches!(
        left.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(left.body.statements.is_empty());

    let abandoned_classifier = "use language (version is v0.1)\nanswer is fn () -> Int\n  abandoned : String is 1 + { return 42 }\n  0\nanswer ()\n";
    let program = analyze_for_compiler(abandoned_classifier).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::ExitSequence { .. }
    ));

    let conditional = "use language (version is v0.1)\nanswer is fn () -> Int\n  true\n    true then { return 42 }\n    false then 0\nanswer ()\n";
    let error = analyze_for_compiler(conditional).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
    assert!(error.message.contains("direct statement position"));
}

#[test]
fn models_return_bearing_direct_product_fields_in_source_order() {
    // TOPAL-FUNCTION-RETURN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-product-field.t"
    ))
    .unwrap();
    for name in ["tuple-exit", "record-exit"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        let CompilerExpressionKind::ExitSequence { preceding, result } = &function.body.result.kind
        else {
            panic!("expected the evaluated product prefix before the field exit")
        };
        assert!(matches!(
            preceding.as_slice(),
            [
                CompilerExpression {
                    kind: CompilerExpressionKind::Call { .. },
                    ..
                },
                CompilerExpression {
                    kind: CompilerExpressionKind::Call { .. },
                    ..
                }
            ]
        ));
        assert!(matches!(result.kind, CompilerExpressionKind::Block(_)));
        assert!(function.body.statements.is_empty());
    }

    let first = "use language (version is v0.1)\nanswer is fn () -> Int\n  ({ return 42 }, missing)\n  0\nanswer ()\n";
    let program = analyze_for_compiler(first).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));

    let abandoned_classifier = "use language (version is v0.1)\nanswer is fn () -> Int\n  abandoned : String is (1, { return 42 }, missing)\n  0\nanswer ()\n";
    let program = analyze_for_compiler(abandoned_classifier).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::ExitSequence { .. }
    ));

    let nested_call = "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\nanswer is fn () -> Int\n  identity (1, { return 42 })\nanswer ()\n";
    let error = analyze_for_compiler(nested_call).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
    assert!(error.message.contains("direct statement position"));
}

#[test]
fn models_return_bearing_direct_named_call_arguments_in_source_order() {
    // TOPAL-FUNCTION-RETURN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-call-argument.t"
    ))
    .unwrap();
    let right = program
        .functions
        .iter()
        .find(|function| function.source_name == "right-exit")
        .unwrap();
    let CompilerExpressionKind::ExitSequence { preceding, result } = &right.body.result.kind else {
        panic!("expected the evaluated left argument before the right-side exit")
    };
    assert!(matches!(
        preceding.as_slice(),
        [CompilerExpression {
            kind: CompilerExpressionKind::Call { .. },
            ..
        }]
    ));
    assert!(matches!(result.kind, CompilerExpressionKind::Block(_)));
    assert!(right.body.statements.is_empty());

    for name in ["left-exit", "unary-exit"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Block(_)
        ));
        assert!(function.body.statements.is_empty());
    }

    let abandoned_classifier = "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\nanswer is fn () -> Int\n  abandoned : String is identity { return 42 }\n  0\nanswer ()\n";
    let program = analyze_for_compiler(abandoned_classifier).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);

    let overloaded = "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\nidentity is fn (value : String) -> String\n  value\nanswer is fn () -> Int\n  identity { return 42 }\nanswer ()\n";
    let error = analyze_for_compiler(overloaded).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
    assert!(error.message.contains("direct statement position"));
}

#[test]
fn models_return_bearing_optional_constructor_argument() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-optional-constructor.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  Some (1, { return 42 })\nanswer ()\n";
    let error = analyze_for_compiler(nested).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
    assert!(error.message.contains("direct statement position"));
}

#[test]
fn models_return_bearing_strict_unary_constructor_arguments() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-UNION-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-unary-constructor.t"
    ))
    .unwrap();
    for name in [
        "string-exit",
        "int-exit",
        "nat-exit",
        "rational-exit",
        "union-exit",
    ] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(function.body.result.value_type, CompilerType::Int);
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Block(_)
        ));
        assert!(function.body.statements.is_empty());
    }

    let positional = "use language (version is v0.1)\nChoice is Variant (Int)\nanswer is fn () -> Int\n  Choice at 0 { return 42 }\nanswer ()\n";
    let error = analyze_for_compiler(positional).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
    assert!(error.message.contains("application"));
}

#[test]
fn models_return_bearing_positional_variant_argument() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-VARIANT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-variant-constructor.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let invalid = "use language (version is v0.1)\nChoice is Variant (Int)\n\nanswer is fn () -> Int\n  Choice at 1 { return 42 }\nanswer ()\n";
    let error = analyze_for_compiler(invalid).unwrap_err();
    assert_eq!(error.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_return_bearing_character_constructor_argument() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-STRING-CHARACTER-CLASSIFIER-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-character-constructor.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  abandoned : Character is Character (String { return 42 })\n  1000\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_named_constraint_argument() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001,
    // TOPAL-COMPILER-CONSTRAINT-FUNDAMENTAL-BASES-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-constraint-constructor.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let forward = "use language (version is v0.1)\nanswer is fn () -> Int\n  Positive { return 42 }\nPositive is Int constraint { candidate } candidate > 0\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(forward).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let non_int = "use language (version is v0.1)\nNonempty is String constraint { candidate } candidate = candidate\nanswer is fn () -> Int\n  Nonempty { return 42 }\nanswer ()\n";
    let program = analyze_for_compiler(non_int).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
}

#[test]
fn models_return_bearing_named_modular_argument() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-modular-constructor.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let forward = "use language (version is v0.1)\nanswer is fn () -> Int\n  Counter { return 42 }\nCounter is ModNat (0 ..= 255)\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(forward).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nested = "use language (version is v0.1)\nCounter is ModNat (0 ..= 255)\nanswer is fn () -> Int\n  Counter ({ return 42 }, 0)\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_modular_reduction_operand() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-NUM-MODULAR-REDUCE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-modular-reduction.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let forward = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 } modulo Counter\nCounter is ModNat (0 ..= 255)\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(forward).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let unknown = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 } modulo Missing\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(unknown).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nested = "use language (version is v0.1)\nCounter is ModNat (0 ..= 255)\nanswer is fn () -> Int\n  ({ return 42 }, 0) modulo Counter\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_unary_list_collect_source() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-COLLECTION-COLLECT-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-list-collect.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let other_collector = "use language (version is v0.1)\nanswer is fn () -> Int\n  collect-map { return 42 } choosing reject\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(other_collector).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let infix = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 } collect Set\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(infix).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  collect ({ return 42 }, 0)\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_unordered_collect_sources() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-SET-COLLECT-001,
    // TOPAL-BAG-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-unordered-collect.t"
    ))
    .unwrap();
    for name in ["set-exit", "bag-exit"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(function.body.result.value_type, CompilerType::Int);
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Block(_)
        ));
        assert!(function.body.statements.is_empty());
    }

    let map = "use language (version is v0.1)\nanswer is fn () -> Int\n  collect-map { return 42 } choosing reject\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(map).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  collect-set ({ return 42 }, 0)\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_map_collect_source() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-MAP-COLLECT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-map-collect.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let invalid_policy = "use language (version is v0.1)\nanswer is fn () -> Int\n  collect-map { return 42 } resolving merge\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(invalid_policy).unwrap_err().code,
        "E-MAP-COLLISION-POLICY"
    );
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  collect-map ({ return 42 }, 0) resolving reject\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_infix_collect_sources() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-ARRAY-COLLECT-001,
    // TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-INFIX-COLLECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-infix-collect.t"
    ))
    .unwrap();
    for name in ["array-exit", "string-exit"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(function.body.result.value_type, CompilerType::Int);
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Block(_)
        ));
        assert!(function.body.statements.is_empty());
    }

    let invalid_target = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 } collect Set\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(invalid_target).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  ({ return 42 }, 0) collect Array\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_boolean_decision_subject() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-DECISION-SUBJECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-decision-subject.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  ({ return 42 }, true)\n    true then 0\n    otherwise 1\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_exhaustive_boolean_decision_subject() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-EXHAUSTIVE-BOOLEAN-SUBJECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-exhaustive-boolean-subject.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let reverse_order = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 }\n    true then 0\n    false then 1\nanswer ()\n";
    analyze_for_compiler(reverse_order).unwrap();
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  ({ return 42 }, true)\n    false then 0\n    true then 1\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_comparison_decision_subject() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-DECISION-SUBJECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-comparison-decision-subject.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let single_rule = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 }\n    >= 0 then 0\n    otherwise 1\nanswer ()\n";
    analyze_for_compiler(single_rule).unwrap();
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  ({ return 42 }, 0)\n    < 0 then 0\n    otherwise 1\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_fallback_decision_subject() {
    // TOPAL-FUNCTION-RETURN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-FALLBACK-DECISION-SUBJECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-fallback-decision-subject.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "answer")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let otherwise_only = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 }\n    otherwise 0\nanswer ()\n";
    analyze_for_compiler(otherwise_only).unwrap();
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  ({ return 42 }, 0)\n    Some payload then payload\n    otherwise 0\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_return_bearing_complete_decision_subjects() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-DECISION-RESULT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPLETE-DECISION-SUBJECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-complete-decision-subject.t"
    ))
    .unwrap();
    for name in ["optional-exit", "result-exit", "list-exit"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(function.body.result.value_type, CompilerType::Int);
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Block(_)
        ));
        assert!(function.body.statements.is_empty());
    }

    let identifier_rules = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 }\n    Red then 0\nanswer ()\n";
    analyze_for_compiler(identifier_rules).unwrap();
    let nested = "use language (version is v0.1)\nanswer is fn () -> Int\n  ({ return 42 }, 0)\n    Some payload then payload\n    None then 0\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_boolean_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-BOOLEAN-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-boolean-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "choose")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::BooleanDecision {
        when_true,
        when_false,
        ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their Boolean decision")
    };
    assert!(matches!(when_true.kind, CompilerExpressionKind::Block(_)));
    assert!(matches!(when_false.kind, CompilerExpressionKind::Block(_)));
    assert!(function.body.statements.is_empty());

    let otherwise = "use language (version is v0.1)\nchoose is fn (value : Boolean) -> Int\n  value\n    false then { return 40 }\n    otherwise { return 41 }\n  1000\nchoose true\n";
    analyze_for_compiler(otherwise).unwrap();
    let mixed = "use language (version is v0.1)\nchoose is fn (value : Boolean) -> Int\n  value\n    false then { return 40 }\n    true then 41\n  1000\nchoose true\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nonnumeric = "use language (version is v0.1)\nchoose is fn (value : Boolean) -> Int\n  value\n    = 0 then { return 40 }\n    otherwise { return 41 }\nchoose true\n";
    assert_eq!(
        analyze_for_compiler(nonnumeric).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_comparison_value_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-COMPARISON-VALUE-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-comparison-value-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "choose")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::ComparisonValueDecision {
        when_less,
        when_equal,
        when_greater,
        ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their Comparison decision")
    };
    for action in [when_less, when_equal, when_greater] {
        assert!(matches!(action.kind, CompilerExpressionKind::Block(_)));
    }
    assert!(function.body.statements.is_empty());

    let otherwise = "use language (version is v0.1)\nchoose is fn (value : Comparison) -> Int\n  value\n    Less then { return 40 }\n    otherwise { return 41 }\n  1000\nchoose (2 <=> 2)\n";
    analyze_for_compiler(otherwise).unwrap();
    let mixed = "use language (version is v0.1)\nchoose is fn (value : Comparison) -> Int\n  value\n    Less then { return 40 }\n    Equal then { return 41 }\n    Greater then 42\n  1000\nchoose (3 <=> 2)\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_ordered_comparison_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-COMPARISON-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ORDERED-COMPARISON-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-ordered-comparison-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "choose")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::OrderedComparisonDecision {
        rules, otherwise, ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their ordered comparison decision")
    };
    assert_eq!(rules.len(), 2);
    assert!(
        rules
            .iter()
            .all(|rule| matches!(rule.action.kind, CompilerExpressionKind::Block(_)))
    );
    assert!(matches!(otherwise.kind, CompilerExpressionKind::Block(_)));
    assert!(function.body.statements.is_empty());

    let single = "use language (version is v0.1)\nchoose is fn (value : Int) -> Int\n  value\n    < 0 then { return 40 }\n    otherwise { return 41 }\n  1000\nchoose 2\n";
    analyze_for_compiler(single).unwrap();
    let rational = "use language (version is v0.1)\nchoose is fn (value : Rational) -> Int\n  value\n    < 0 then { return 40 }\n    otherwise { return 41 }\n  1000\nchoose (Rational (1, 2))\n";
    analyze_for_compiler(rational).unwrap();
    let mixed = "use language (version is v0.1)\nchoose is fn (value : Int) -> Int\n  value\n    < 0 then { return 40 }\n    = 0 then { return 41 }\n    otherwise 42\n  1000\nchoose 2\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_final_fallback_enum_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-FALLBACK-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-enum-fallback-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "choose")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::EnumDecision {
        rules, otherwise, ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their Enum decision")
    };
    assert_eq!(rules.len(), 2);
    assert!(
        rules
            .iter()
            .all(|rule| matches!(rule.action.kind, CompilerExpressionKind::Block(_)))
    );
    assert!(matches!(
        otherwise.as_deref().map(|action| &action.kind),
        Some(CompilerExpressionKind::Block(_))
    ));
    assert!(function.body.statements.is_empty());

    let reserved = "use language (version is v0.1)\nDirection is Enum (Less, Other)\nchoose is fn (value : Direction) -> Int\n  value\n    Less then { return 40 }\n    otherwise { return 41 }\n  1000\nchoose Other\n";
    analyze_for_compiler(reserved).unwrap();
    let mixed = "use language (version is v0.1)\nColor is Enum (Red, Green)\nchoose is fn (value : Color) -> Int\n  value\n    Red then { return 40 }\n    otherwise 41\n  1000\nchoose Green\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let unknown = "use language (version is v0.1)\nColor is Enum (Red, Green)\nchoose is fn (value : Color) -> Int\n  value\n    Blue then { return 40 }\n    otherwise { return 41 }\nchoose Green\n";
    assert_eq!(
        analyze_for_compiler(unknown).unwrap_err().code,
        "E-UNKNOWN-ENUM-ALTERNATIVE"
    );
}

#[test]
fn models_all_returning_exhaustive_enum_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ENUM-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ENUM-EXHAUSTIVE-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-exhaustive-enum-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "choose")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::EnumDecision {
        rules, otherwise, ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their exhaustive Enum decision")
    };
    assert_eq!(rules.len(), 3);
    assert!(
        rules
            .iter()
            .all(|rule| matches!(rule.action.kind, CompilerExpressionKind::Block(_)))
    );
    assert!(otherwise.is_none());
    assert!(function.body.statements.is_empty());

    let reserved = "use language (version is v0.1)\nDirection is Enum (Less, Other)\nchoose is fn (value : Direction) -> Int\n  value\n    Other then { return 41 }\n    Less then { return 40 }\n  1000\nchoose Other\n";
    analyze_for_compiler(reserved).unwrap();
    let incomplete = "use language (version is v0.1)\nColor is Enum (Red, Green)\nchoose is fn (value : Color) -> Int\n  value\n    Red then { return 40 }\n  1000\nchoose Red\n";
    assert_eq!(
        analyze_for_compiler(incomplete).unwrap_err().code,
        "E-INCOMPLETE-DECISION"
    );
    let unknown = "use language (version is v0.1)\nColor is Enum (Red, Green)\nchoose is fn (value : Color) -> Int\n  value\n    Red then { return 40 }\n    Blue then { return 41 }\n  1000\nchoose Red\n";
    assert_eq!(
        analyze_for_compiler(unknown).unwrap_err().code,
        "E-UNKNOWN-ENUM-ALTERNATIVE"
    );
    let mixed = "use language (version is v0.1)\nColor is Enum (Red, Green)\nchoose is fn (value : Color) -> Int\n  value\n    Red then { return 40 }\n    Green then 41\n  1000\nchoose Green\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_optional_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-OPTIONAL-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-optional-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "explicit")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::OptionalDecision {
        some_binding,
        some_action,
        none_action,
        ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their Optional decision")
    };
    assert_eq!(
        some_binding.as_ref().map(|(name, _)| name.as_str()),
        Some("payload")
    );
    assert!(matches!(some_action.kind, CompilerExpressionKind::Block(_)));
    assert!(matches!(none_action.kind, CompilerExpressionKind::Block(_)));
    assert!(function.body.statements.is_empty());

    let otherwise_only = "use language (version is v0.1)\nchoose is fn (value : Optional Int) -> Int\n  value\n    otherwise { return 42 }\n  1000\nchoose (None Int)\n";
    analyze_for_compiler(otherwise_only).unwrap();
    let mixed = "use language (version is v0.1)\nchoose is fn (value : Optional Int) -> Int\n  value\n    Some payload then { return payload }\n    None then 41\n  1000\nchoose (Some 40)\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nonoptional = "use language (version is v0.1)\nchoose is fn (value : Int) -> Int\n  value\n    Some payload then { return payload }\n    None then { return 41 }\nchoose 40\n";
    assert_eq!(
        analyze_for_compiler(nonoptional).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_result_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-RESULT-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-RESULT-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-result-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "choose")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Boolean);
    let CompilerExpressionKind::ResultDecision {
        ok_binding,
        ok_action,
        error_codes,
        error_fallback,
        ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their Result decision")
    };
    assert_eq!(ok_binding, "quotient");
    assert!(matches!(ok_action.kind, CompilerExpressionKind::Block(_)));
    assert!(error_codes.is_empty());
    let (error_binding, _, error_action) = error_fallback.as_ref().expect("complete Error action");
    assert_eq!(error_binding, "problem");
    assert!(matches!(
        error_action.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let reversed = "use language (version is v0.1)\ndivide is fn (candidate : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  1.0 / candidate\nchoose is fn (candidate : Rational) -> Int\n  divide candidate\n    Error problem then { return 41 }\n    Ok quotient then { return 40 }\n  1000\nchoose 1.0\n";
    analyze_for_compiler(reversed).unwrap();
    let mixed = "use language (version is v0.1)\ndivide is fn (candidate : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  1.0 / candidate\nchoose is fn (candidate : Rational) -> Int\n  divide candidate\n    Ok quotient then { return 40 }\n    Error problem then 41\n  1000\nchoose 1.0\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nonresult = "use language (version is v0.1)\nchoose is fn (candidate : Int) -> Int\n  candidate\n    Ok value then { return 40 }\n    Error problem then { return 41 }\nchoose 1\n";
    assert_eq!(
        analyze_for_compiler(nonresult).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_error_code_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-ERROR-CODE-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-ERROR-CODE-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-error-code-decision-actions.t"
    ))
    .unwrap();
    let recover = program
        .functions
        .iter()
        .find(|function| function.source_name == "recover")
        .unwrap();
    assert_eq!(recover.body.result.value_type, CompilerType::Boolean);
    let CompilerExpressionKind::ResultDecision {
        ok_binding,
        ok_action,
        error_codes,
        error_fallback,
        ..
    } = &recover.body.result.kind
    else {
        panic!("all-returning actions retain their Error-code decision")
    };
    assert_eq!(ok_binding, "quotient");
    assert!(matches!(ok_action.kind, CompilerExpressionKind::Block(_)));
    assert_eq!(error_codes.len(), 1);
    assert!(matches!(
        error_codes[0].action.kind,
        CompilerExpressionKind::Block(_)
    ));
    let (error_binding, _, error_action) = error_fallback.as_ref().unwrap();
    assert_eq!(error_binding, "problem");
    assert!(matches!(
        error_action.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(recover.body.statements.is_empty());

    let classify = program
        .functions
        .iter()
        .find(|function| function.source_name == "classify")
        .unwrap();
    assert_eq!(classify.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::ResultDecision {
        error_codes,
        error_fallback,
        ..
    } = &classify.body.result.kind
    else {
        panic!("exhaustive Error-code actions retain their Result decision")
    };
    assert_eq!(
        error_codes.iter().map(|rule| rule.code).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(error_fallback.is_none());
    assert!(classify.body.statements.is_empty());

    let mixed = "use language (version is v0.1)\nratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  Rational (numerator, denominator)\nchoose is fn (numerator : Int, denominator : Int) -> Int\n  ratio (numerator, denominator)\n    Ok value then { return 0 }\n    Error ( code is lang arithmetic division-by-zero ) then { return 1 }\n    Error problem then 2\n  1000\nchoose (1, 0)\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nonresult = "use language (version is v0.1)\nchoose is fn (candidate : Int) -> Int\n  candidate\n    Ok value then { return 0 }\n    Error ( code is lang arithmetic division-by-zero ) then { return 1 }\n    Error problem then { return 2 }\nchoose 1\n";
    assert_eq!(
        analyze_for_compiler(nonresult).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_all_returning_list_decision_actions() {
    // TOPAL-FUNCTION-RETURN-001, TOPAL-DECISION-LIST-001,
    // TOPAL-COMPILER-LEXICAL-RETURN-LIST-DECISION-ACTIONS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-return-list-decision-actions.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "choose")
        .unwrap();
    assert_eq!(function.body.result.value_type, CompilerType::Int);
    let CompilerExpressionKind::ListDecision {
        entry_bindings,
        entry_action,
        empty_action,
        ..
    } = &function.body.result.kind
    else {
        panic!("all-returning actions retain their List decision")
    };
    let ((first_name, _), (rest_name, _)) = entry_bindings.as_ref().expect("complete Entry action");
    assert_eq!(first_name, "first");
    assert_eq!(rest_name, "rest");
    assert!(matches!(
        entry_action.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(matches!(
        empty_action.kind,
        CompilerExpressionKind::Block(_)
    ));
    assert!(function.body.statements.is_empty());

    let reversed = "use language (version is v0.1)\nchoose is fn (candidate : List Int) -> Int\n  candidate\n    Empty then { return 40 }\n    Entry (first, rest) then { return first }\n  1000\nvalues : List Int is Entry (42, Empty)\nchoose values\n";
    analyze_for_compiler(reversed).unwrap();
    let mixed = "use language (version is v0.1)\nchoose is fn (candidate : List Int) -> Int\n  candidate\n    Entry (first, rest) then { return first }\n    Empty then 40\n  1000\nvalues : List Int is Entry (42, Empty)\nchoose values\n";
    assert_eq!(
        analyze_for_compiler(mixed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nonlist = "use language (version is v0.1)\nchoose is fn (candidate : Int) -> Int\n  candidate\n    Entry (first, rest) then { return first }\n    Empty then { return 40 }\nchoose 1\n";
    assert_eq!(
        analyze_for_compiler(nonlist).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let duplicate_binding = "use language (version is v0.1)\nchoose is fn (candidate : List Int) -> Int\n  candidate\n    Entry (value, value) then { return value }\n    Empty then { return 40 }\nvalues : List Int is Entry (42, Empty)\nchoose values\n";
    assert_eq!(
        analyze_for_compiler(duplicate_binding).unwrap_err().code,
        "E-DUPLICATE-BINDING"
    );
}
