#[test]
fn retains_and_erases_exact_function_interface_evidence() {
    // TOPAL-INTERFACE-SHAPE-001, TOPAL-INTERFACE-IMPLEMENTATION-001,
    // TOPAL-COMPILER-FUNCTION-INTERFACE-001
    let source = include_str!("../../../../../examples/language/function-interface.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(program.interfaces.len(), 1);
    assert_eq!(program.interfaces[0].identity, "root.Parser");
    assert_eq!(
        program.interfaces[0].operations,
        [CompilerInterfaceOperation {
            name: "parse".into(),
            parameters: vec![CompilerType::String],
            result: CompilerType::Boolean,
        }]
    );
    assert_eq!(program.interface_implementations.len(), 1);
    assert_eq!(
        program.interface_implementations[0],
        CompilerInterfaceImplementation {
            interface_identity: "root.Parser".into(),
            operations: vec![CompilerInterfaceOperationEvidence {
                role: "parse".into(),
                declaration_identity: "root.parse:ordinary(String)".into(),
                declared_effects: None,
            }],
        }
    );
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].source_name, "parse");
    assert_eq!(program.main.result.value_type, CompilerType::Boolean);
    let interpreted = crate::source::Session::new()
        .evaluate_source_file(source, &mut std::io::sink())
        .unwrap();
    assert_eq!(interpreted.to_string(), "true");

    let multi_source = "use language (version is v0.1)\nService is Interface\n  zed is fn (value : Int) -> Int\n  alpha is fn (value : String) -> Boolean\nService\n  zed is fn (value : Int) -> Int\n    value\n  alpha is fn (value : String) -> Boolean\n    : Effects ()\n    value = \"ok\"\nalpha \"ok\"\n";
    let multi = analyze_for_compiler(multi_source).unwrap();
    assert_eq!(
        multi.interfaces[0]
            .operations
            .iter()
            .map(|operation| operation.name.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "zed"]
    );
    assert_eq!(
        multi.interface_implementations[0]
            .operations
            .iter()
            .map(|operation| operation.role.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "zed"]
    );
    assert_eq!(
        multi.interface_implementations[0].operations[0].declared_effects,
        Some(CompilerEffectRow {
            identities: Vec::new(),
        })
    );
    assert_eq!(
        multi.interface_implementations[0].operations[1].declared_effects,
        None
    );

    for invalid in [
        "use language (version is v0.1)\nParser\n  parse is fn (source : String) -> Boolean\n    true\nparse \"ok\"\n",
        "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  other is fn (source : String) -> Boolean\n    true\nother \"ok\"\n",
        "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\n  other is fn (source : String) -> Boolean\nParser\n  parse is fn (source : String) -> Boolean\n    true\nparse \"ok\"\n",
        "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  parse is fn (source : String) -> Boolean\n    true\n  other is fn (source : String) -> Boolean\n    false\nparse \"ok\"\n",
        "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  parse is fn (source : Int) -> Boolean\n    true\nparse 1\n",
        "use language (version is v0.1)\nParser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  parse is fn (source : String) -> Boolean\n    true\n  parse is fn (source : String) -> Boolean\n    false\nparse \"ok\"\n",
    ] {
        assert!(
            matches!(
                analyze_for_compiler(invalid).unwrap_err().code.as_str(),
                "E-UNKNOWN-INTERFACE" | "E-INTERFACE-IMPLEMENTATION"
            ),
            "{invalid}"
        );
    }
    let nested = "use language (version is v0.1)\nconstruct is fn () -> Unit\n  Parser is Interface\n    parse is fn (source : String) -> Boolean\n  ()\nconstruct ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_named_function_values_without_restarting_lookup() {
    // TOPAL-COMPILER-NAMED-FUNCTION-VALUE-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-FUNCTION-OVERLOAD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/named-function-values.t"
    ))
    .unwrap();
    assert_eq!(
        program.function_value_names,
        std::iter::once("<fn increment>".to_owned())
            .chain(
                COMPILER_SYMBOLIC_CALLABLES
                    .iter()
                    .map(|(_, name)| (*name).to_owned()),
            )
            .collect::<Vec<_>>()
    );
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::FunctionValue(0),
                value_type: CompilerType::Function,
                ..
            },
            ..
        })]
    ));
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

    let chained = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\noperation : Function is increment\nagain is operation\n{\n  increment is 100\n  again 41\n}\n",
        )
        .unwrap();
    assert_eq!(exact_int(&chained.main.result), Some(BigInt::from(42)));

    let snapshot = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\noperation is identity\nidentity is fn (value : String) -> String\n  value\n(operation 42, identity \"Topal\")\n",
        )
        .unwrap();
    let CompilerExpressionKind::Tuple(values) = &snapshot.main.result.kind else {
        panic!("expected captured and live function result product")
    };
    assert_eq!(values[0].value_type, CompilerType::Int);
    assert_eq!(values[1].value_type, CompilerType::String);

    let rejected = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\noperation is identity\nidentity is fn (value : String) -> String\n  value\noperation \"Topal\"\n",
        )
        .unwrap_err();
    assert_eq!(rejected.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn models_symbolic_callable_values_and_function_classification() {
    // TOPAL-COMPILER-SYMBOLIC-CALLABLE-VALUE-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001
    let values = analyze_for_compiler(include_str!(
        "../../../../../examples/language/callable-values.t"
    ))
    .unwrap();
    assert_eq!(
        values.function_value_names,
        COMPILER_SYMBOLIC_CALLABLES
            .iter()
            .map(|(_, name)| (*name).to_owned())
            .collect::<Vec<_>>()
    );
    let CompilerExpressionKind::Tuple(results) = &values.main.result.kind else {
        panic!("expected callable result product")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(-5)));
    assert_eq!(results[2].value_type, CompilerType::Comparison);

    let classified = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-classifier.t"
    ))
    .unwrap();
    assert_eq!(exact_int(&classified.main.result), Some(BigInt::from(42)));

    let chained = analyze_for_compiler(
            "use language (version is v0.1)\nadd : Function is +\noperation is add\n(operation (20, 22), operation (1, 2))\n",
        )
        .unwrap();
    let CompilerExpressionKind::Tuple(results) = &chained.main.result.kind else {
        panic!("expected chained symbolic results")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(3)));

    let binary_minus = analyze_for_compiler(
        "use language (version is v0.1)\nsubtract is -\n(subtract (9, 4), subtract 5)\n",
    )
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &binary_minus.main.result.kind else {
        panic!("expected unary and binary minus results")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(5)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(-5)));

    let rejected =
        analyze_for_compiler("use language (version is v0.1)\nadd is +\nadd 1\n").unwrap_err();
    assert_eq!(rejected.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn models_every_symbolic_callable_value_as_a_direct_operation() {
    // TOPAL-COMPILER-SYMBOLIC-CALLABLE-EXPANDED-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/expanded-callable-values.t"
    ))
    .unwrap();
    assert_eq!(
        program.function_value_names,
        std::iter::once("<fn select>".to_owned())
            .chain(
                COMPILER_SYMBOLIC_CALLABLES
                    .iter()
                    .map(|(_, name)| (*name).to_owned()),
            )
            .collect::<Vec<_>>()
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected complete symbolic result product")
    };
    assert_eq!(results.len(), 16);
    assert!(
        results[..6]
            .iter()
            .all(|result| result.value_type == CompilerType::Boolean)
    );
    assert!(
        results[..6]
            .iter()
            .all(|result| matches!(result.kind, CompilerExpressionKind::Binary { .. }))
    );
    assert_eq!(exact_int(&results[6]), Some(BigInt::from(42)));
    assert_eq!(
        results[7].rational_value,
        Some(BigRational::new(BigInt::from(3), BigInt::from(4)))
    );
    assert!(matches!(
        results[8].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::QuotientModulo,
            ..
        }
    ));
    assert_eq!(
        results[8].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int])
    );
    assert_eq!(exact_int(&results[9]), Some(BigInt::from(3)));
    assert_eq!(exact_int(&results[10]), Some(BigInt::from(1024)));
    assert!(
        results[11..15].iter().all(|result| {
            result.value_type == CompilerType::Range(Box::new(CompilerType::Int))
        })
    );
    assert_eq!(exact_int(&results[15]), Some(BigInt::from(42)));
    assert!(program.functions.iter().any(|function| {
        function.source_name == "select"
            && function.result_type == CompilerType::Function
            && matches!(function.body.result.kind, CompilerExpressionKind::Local(_))
    }));
}

#[test]
fn models_specialized_function_input_boundaries() {
    // TOPAL-COMPILER-FUNCTION-PARAMETER-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-value-boundary.t"
    ))
    .unwrap();
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
    assert_eq!(
        program.functions[0].parameters[0].value_type,
        CompilerType::Function
    );
    assert!(matches!(
        program.functions[0].body.result.kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            ..
        }
    ));

    let aliased = analyze_for_compiler(
            "use language (version is v0.1)\napply-pair is fn (operation : Function) -> Int\n  operation (20, 22)\nadd is +\napply-pair add\n",
        )
        .unwrap();
    assert_eq!(exact_int(&aliased.main.result), Some(BigInt::from(42)));

    let named = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\napply is fn (operation : Function) -> Int\n  operation 41\napply increment\n",
        )
        .unwrap();
    assert_eq!(exact_int(&named.main.result), Some(BigInt::from(42)));
    assert!(
        named
            .functions
            .iter()
            .any(|function| function.source_name == "increment")
    );

    let two_specializations = analyze_for_compiler(
            "use language (version is v0.1)\napply is fn (operation : Function) -> Int\n  operation (9, 4)\nadd is +\nsubtract is -\n(apply add, apply subtract)\n",
        )
        .unwrap();
    let CompilerExpressionKind::Tuple(results) = &two_specializations.main.result.kind else {
        panic!("expected two Function specializations")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(13)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(5)));
    assert_eq!(
        two_specializations
            .functions
            .iter()
            .filter(|function| function.source_name == "apply")
            .count(),
        2
    );
}

#[test]
fn models_closed_specialized_function_results() {
    // TOPAL-COMPILER-FUNCTION-RESULT-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-FUNCTION-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-results.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected named and symbolic Function-result applications")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(42)));
    let selectors = program
        .functions
        .iter()
        .filter(|function| function.source_name == "select")
        .collect::<Vec<_>>();
    assert_eq!(selectors.len(), 2);
    assert!(selectors.iter().all(|function| {
        function.result_type == CompilerType::Function
            && matches!(
                function.body.result,
                CompilerExpression {
                    kind: CompilerExpressionKind::Local(_),
                    value_type: CompilerType::Function,
                    ..
                }
            )
    }));

    let static_symbolic = analyze_for_compiler(
            "use language (version is v0.1)\nselect is fn static (operation : Function) -> Function\n  operation\naddition is select +\naddition (20, 22)\n",
        )
        .unwrap();
    assert_eq!(
        exact_int(&static_symbolic.main.result),
        Some(BigInt::from(42))
    );

    let escaping_nested = analyze_for_compiler(
            "use language (version is v0.1)\nouter is fn () -> Function\n  inner is fn (value : Int) -> Int\n    value + 1\n  inner\noperation is outer ()\noperation 41\n",
        )
        .unwrap();
    assert_eq!(
        exact_int(&escaping_nested.main.result),
        Some(BigInt::from(42))
    );
    let factory = escaping_nested
        .functions
        .iter()
        .find(|function| function.source_name == "outer")
        .unwrap();
    assert_eq!(factory.result_type, CompilerType::Function);
    assert!(factory.result_captures.is_empty());
}

#[test]
fn models_direct_non_capturing_anonymous_functions() {
    // TOPAL-COMPILER-ANONYMOUS-DIRECT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-SYN-GRAMMAR-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-function-application.t"
    ))
    .unwrap();
    assert_eq!(
        program.function_value_names,
        COMPILER_SYMBOLIC_CALLABLES
            .iter()
            .map(|(_, name)| (*name).to_owned())
            .chain(["<anonymous fn/1>".into(), "<anonymous fn/2>".into()])
            .collect::<Vec<_>>()
    );
    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.functions[0].source_name, "<anonymous fn/1>");
    assert_eq!(program.functions[0].parameters.len(), 1);
    assert_eq!(program.functions[1].source_name, "<anonymous fn/2>");
    assert_eq!(program.functions[1].parameters.len(), 2);
    assert!(
        program
            .functions
            .iter()
            .flat_map(|function| &function.parameters)
            .all(|parameter| parameter.value_type == CompilerType::Int)
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected two direct anonymous calls")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(42)));
    assert!(results.iter().all(|result| match &result.kind {
        CompilerExpressionKind::Call { .. } => true,
        CompilerExpressionKind::PrivateBinding { body, .. } => {
            matches!(body.kind, CompilerExpressionKind::Call { .. })
        }
        _ => false,
    }));

    let left_associative = analyze_for_compiler(
        "use language (version is v0.1)\ncalculate is { value } value + 3 * 4\ncalculate 2\n",
    )
    .unwrap();
    assert_eq!(
        exact_int(&left_associative.main.result),
        Some(BigInt::from(20))
    );

    let contextual = analyze_for_compiler(
            "use language (version is v0.1)\napply is fn (operation : Function) -> Int\n  operation 41\napply { value } value + 1\n",
        )
        .unwrap();
    assert_eq!(exact_int(&contextual.main.result), Some(BigInt::from(42)));
    assert!(
        contextual
            .functions
            .iter()
            .any(|function| function.source_name == "<anonymous fn/1>")
    );

    let arity = analyze_for_compiler(
        "use language (version is v0.1)\ncombine is { left, right } left + right\ncombine 42\n",
    )
    .unwrap_err();
    assert_eq!(arity.code, "E-ANONYMOUS-ARGUMENT-PACKAGE");
}

#[test]
fn models_private_anonymous_captures_and_noncapturing_results() {
    // TOPAL-COMPILER-ANONYMOUS-CAPTURE-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-function-captures.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected captured and returned anonymous calls")
    };
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(42)));

    let captured = program
        .functions
        .iter()
        .find(|function| {
            function.source_name == "<anonymous fn/1>"
                && function
                    .parameters
                    .iter()
                    .any(|parameter| parameter.name == "offset")
        })
        .unwrap();
    assert_eq!(captured.parameters.len(), 2);
    assert_eq!(captured.parameters[0].name, "input");
    assert_eq!(captured.parameters[1].name, "offset");
    assert!(
        captured
            .parameters
            .iter()
            .all(|parameter| parameter.value_type == CompilerType::Int)
    );

    let factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-double")
        .unwrap();
    assert_eq!(factory.result_type, CompilerType::Function);
    assert!(matches!(
        factory.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::FunctionValue(_),
            value_type: CompilerType::Function,
            ..
        }
    ));
    assert!(program.functions.iter().any(|function| {
        function.source_name == "<anonymous fn/1>"
            && function.parameters.len() == 1
            && function.parameters[0].name == "value"
    }));

    let rejected = "use language (version is v0.1)\nwith-shadow is fn (offset : Int) -> Int\n  operation : Function is { value } value + offset\n  {\n    offset is 100\n    operation 41\n  }\nwith-shadow 1\n";
    assert_eq!(
        analyze_for_compiler(rejected).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_captured_function_parameters_as_private_hidden_arguments() {
    // TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-NESTED-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capturing-function-parameters.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected capturing Function-parameter results")
    };
    assert_eq!(results.len(), 4);
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(42)));
    assert_eq!(
        results[2].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
    );
    assert_eq!(exact_int(&results[3]), Some(BigInt::from(42)));

    let forwarded = program
        .functions
        .iter()
        .filter(|function| function.source_name == "forward-int")
        .collect::<Vec<_>>();
    assert_eq!(forwarded.len(), 3);
    assert!(forwarded.iter().all(|function| {
        function.parameters[0].value_type == CompilerType::Function
            && function.parameters[0].source_visible
            && function.parameters[1].source_visible
            && function.parameters[2..]
                .iter()
                .all(|parameter| !parameter.source_visible)
    }));
    assert!(
        forwarded
            .iter()
            .any(|function| function.parameters.len() == 3)
    );
    assert_eq!(
        forwarded
            .iter()
            .filter(|function| function.parameters.len() == 4)
            .count(),
        2
    );

    let pair_boundary = program
        .functions
        .iter()
        .find(|function| function.source_name == "apply-pair")
        .unwrap();
    assert_eq!(pair_boundary.parameters.len(), 3);
    assert!(!pair_boundary.parameters[2].source_visible);
    assert_eq!(
        pair_boundary.parameters[2].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
    );
    assert!(program.functions.iter().any(|function| {
        function.source_name == "add"
            && function.parameters.len() == 3
            && function
                .parameters
                .iter()
                .all(|parameter| parameter.source_visible)
    }));
}

#[test]
fn models_captured_function_results_as_private_aggregate_returns() {
    // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capturing-function-results.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected capturing Function-result applications")
    };
    assert_eq!(results.len(), 4);
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(
        results[1].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
    );
    assert_eq!(exact_int(&results[2]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[3]), Some(BigInt::from(42)));

    let scalar_factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-scalars")
        .collect::<Vec<_>>();
    assert_eq!(scalar_factories.len(), 2);
    assert!(scalar_factories.iter().all(|function| {
        function.result_type == CompilerType::Function
            && function
                .result_captures
                .iter()
                .map(|capture| capture.name.as_str())
                .eq(["left", "right"])
            && function
                .result_captures
                .iter()
                .all(|capture| capture.value_type == CompilerType::Int)
    }));
    let pair_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-pair")
        .unwrap();
    assert_eq!(pair_factory.result_captures.len(), 1);
    assert_eq!(pair_factory.result_captures[0].name, "pair");
    assert_eq!(
        pair_factory.result_captures[0].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
    );
    for function_name in ["return-operation", "make-forwarded"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == function_name)
            .unwrap();
        assert_eq!(function.result_captures.len(), 2);
        assert!(
            function
                .result_captures
                .iter()
                .all(|capture| capture.value_type == CompilerType::Int)
        );
    }

    let rejected = "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nmake is fn (operation : Function) -> Function\n  { value } operation value\nresult is make increment\nresult 41\n";
    let diagnostic = analyze_for_compiler(rejected).unwrap_err();
    assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    assert!(diagnostic.message.contains("Function result"));
}

#[test]
fn models_left_associative_function_result_chains() {
    // TOPAL-COMPILER-FUNCTION-RESULT-CHAIN-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-result-chains.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Function-result chain tuple")
    };
    assert_eq!(results.len(), 6);
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[1]), Some(BigInt::from(42)));
    assert_eq!(
        results[2].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
    );
    assert_eq!(exact_int(&results[3]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[4]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[5]), Some(BigInt::from(42)));
    assert!(
        results
            .iter()
            .all(|result| { matches!(result.kind, CompilerExpressionKind::PrivateBinding { .. }) })
    );

    let factories = program
        .functions
        .iter()
        .filter(|function| {
            matches!(
                function.source_name.as_str(),
                "make-offset" | "make-pair" | "make-closed" | "return-operation"
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 6);
    assert!(
        factories
            .iter()
            .all(|function| function.result_type == CompilerType::Function)
    );
    assert_eq!(
        factories
            .iter()
            .filter(|function| !function.result_captures.is_empty())
            .count(),
        3
    );

    let diagnostic = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn (offset : Int) -> Function\n  { value } value + offset\nmake 1 41 0\n",
        )
        .unwrap_err();
    assert_eq!(diagnostic.code, "E-NO-APPLICABLE-OVERLOAD");
    assert!(
        diagnostic
            .message
            .contains("application chain produced `Int`")
    );
}

#[test]
fn models_private_anonymous_product_patterns() {
    // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-TYPE-PRODUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-product-functions.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected four anonymous product calls")
    };
    assert_eq!(results.len(), 4);
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Int)
    );
    assert_eq!(exact_int(&results[2]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[3]), Some(BigInt::from(42)));

    let anonymous = program
        .functions
        .iter()
        .filter(|function| function.source_name.starts_with("<anonymous fn/"))
        .collect::<Vec<_>>();
    assert_eq!(anonymous.len(), 4);
    assert!(anonymous.iter().any(|function| {
        function
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .eq(["left", "right", "offset"])
    }));
    assert!(anonymous.iter().any(|function| {
        function
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .eq(["left", "right", "extra"])
    }));
    assert!(anonymous.iter().all(|function| {
        function
            .parameters
            .iter()
            .all(|parameter| parameter.value_type == CompilerType::Int)
    }));

    let CompilerExpressionKind::PrivateBinding { value, body, .. } = &results[0].kind else {
        panic!("expected once-only materialization of the Tuple-producing call")
    };
    assert!(matches!(value.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected a direct anonymous call after destructuring")
    };
    assert_eq!(arguments.len(), 2);
    assert!(
        arguments
            .iter()
            .all(|argument| matches!(argument.kind, CompilerExpressionKind::TupleField { .. }))
    );

    let heterogeneous = analyze_for_compiler(
            "use language (version is v0.1)\nchoose : Function is { (condition, text) } text\nchoose (true, \"kept\")\n",
        )
        .unwrap();
    let function = &heterogeneous.functions[0];
    assert_eq!(
        function
            .parameters
            .iter()
            .map(|parameter| parameter.value_type.clone())
            .collect::<Vec<_>>(),
        [CompilerType::Boolean, CompilerType::String]
    );
    assert_eq!(function.result_type, CompilerType::String);
    assert_eq!(heterogeneous.main.result.value_type, CompilerType::String);

    let boundary = analyze_for_compiler(
            "use language (version is v0.1)\napply is fn (operation : Function, values : (Int, Int)) -> Int\n  operation values\napply ({ (left, right) } left + right, (20, 22))\n",
        )
        .unwrap();
    assert_eq!(boundary.main.result.value_type, CompilerType::Int);
    assert!(boundary.functions.iter().any(|function| {
        function.source_name == "<anonymous fn/1>"
            && function
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .eq(["left", "right"])
    }));

    for (source, code) in [
        (
            "use language (version is v0.1)\noperation : Function is { (left, right) } left + right\noperation 42\n",
            "E-ANONYMOUS-PRODUCT-PATTERN",
        ),
        (
            "use language (version is v0.1)\noperation : Function is { (left, right) } left + right\noperation (1, 2, 3)\n",
            "E-ANONYMOUS-PRODUCT-PATTERN",
        ),
    ] {
        assert_eq!(analyze_for_compiler(source).unwrap_err().code, code);
    }
}

#[test]
fn models_recursive_anonymous_product_patterns() {
    // TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001,
    // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-TYPE-PRODUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/nested-anonymous-patterns.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected five nested-pattern calls")
    };
    assert_eq!(results.len(), 5);
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Int)
    );
    assert_eq!(exact_int(&results[3]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&results[4]), Some(BigInt::from(42)));
    assert!(matches!(
        results[0].kind,
        CompilerExpressionKind::PrivateBinding { .. }
    ));

    let anonymous = program
        .functions
        .iter()
        .filter(|function| function.source_name.starts_with("<anonymous fn/"))
        .collect::<Vec<_>>();
    assert_eq!(anonymous.len(), 5);
    assert!(anonymous.iter().any(|function| {
        function
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .eq(["left", "middle", "right"])
    }));
    assert!(anonymous.iter().any(|function| {
        function
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .eq(["left", "right", "tail", "offset"])
    }));
    assert!(anonymous.iter().any(|function| {
        function
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .eq(["left", "middle", "right", "extra"])
    }));
    assert!(anonymous.iter().any(|function| {
        function.pattern_identities.as_slice()
            == [CompilerPatternIdentity {
                first_parameter: 0,
                repeated_parameter: 1,
                span: function.parameters[1].span,
            }]
            && function.parameters[2].discarded
    }));

    for (source, detail) in [
        (
            "use language (version is v0.1)\noperation : Function is { (left, (middle, right)) } left + middle + right\noperation (1, 2)\n",
            "requires a positional product",
        ),
        (
            "use language (version is v0.1)\noperation : Function is { (left, (middle, right)) } left + middle + right\noperation (1, (2, 3, 4))\n",
            "expects 2 fields, found 3",
        ),
    ] {
        let diagnostic = analyze_for_compiler(source).unwrap_err();
        assert_eq!(diagnostic.code, "E-ANONYMOUS-PRODUCT-PATTERN");
        assert!(diagnostic.message.contains(detail), "{diagnostic:?}");
    }
}

#[test]
fn models_exact_function_values_inside_private_aggregates() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
    // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-TYPE-PRODUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-aggregate-boundaries.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected four Function-aggregate applications")
    };
    assert_eq!(results.len(), 4);
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Int)
    );
    let function_aggregate_results = program
        .functions
        .iter()
        .filter(|function| {
            matches!(
                function.source_name.as_str(),
                "make-tuple" | "make-record" | "make-nested"
            )
        })
        .map(|function| &function.result_type)
        .collect::<Vec<_>>();
    assert_eq!(function_aggregate_results.len(), 3);
    assert!(
        function_aggregate_results
            .iter()
            .all(|result| compiler_type_is_function_aggregate(result))
    );
    assert!(program.functions.iter().any(|function| {
        function.source_name == "apply-nested"
            && matches!(function.parameters.as_slice(), [parameter]
                    if compiler_type_is_function_aggregate(&parameter.value_type))
    }));
}

#[test]
fn models_capture_bearing_function_aggregate_boundaries() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
    // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-TYPE-PRODUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capturing-function-aggregate-boundaries.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected four capture-bearing Function-aggregate applications")
    };
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|result| match &result.value_type {
        CompilerType::Int => true,
        CompilerType::Tuple(fields) => {
            fields == &[CompilerType::Int, CompilerType::Int]
        }
        _ => false,
    }));

    let function = |name: &str| {
        program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap_or_else(|| panic!("missing generated `{name}` function"))
    };
    let capture_paths = |name: &str| {
        function(name)
            .result_captures
            .iter()
            .map(|capture| capture.path.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        capture_paths("make-record"),
        [
            vec![CompilerAggregatePathElement::Record("operation".into())],
            vec![CompilerAggregatePathElement::Record("scale".into())],
        ]
    );
    assert_eq!(
        capture_paths("forward-record"),
        [
            vec![CompilerAggregatePathElement::Record("operation".into())],
            vec![CompilerAggregatePathElement::Record("scale".into())],
        ]
    );
    assert_eq!(
        capture_paths("make-tuple"),
        [vec![CompilerAggregatePathElement::Tuple(0)]]
    );

    for (name, hidden_capture_count) in [
        ("apply-record", 2),
        ("forward-record", 2),
        ("apply-tuple", 1),
    ] {
        let generated = function(name);
        assert!(compiler_type_is_function_aggregate(
            &generated.parameters[0].value_type
        ));
        assert_eq!(
            generated
                .parameters
                .iter()
                .filter(|parameter| !parameter.source_visible)
                .count(),
            hidden_capture_count
        );
    }
    assert!(program.functions.iter().any(|generated| {
        generated.source_name == "apply-one"
            && compiler_type_is_function_aggregate(&generated.parameters[0].value_type)
            && generated
                .parameters
                .iter()
                .filter(|parameter| !parameter.source_visible)
                .count()
                == 1
    }));

    let rejected = "use language (version is v0.1)\nmake is fn (operation : Function) -> Record (wrapped : Function)\n  wrapped : Function is { value } operation value\n  (wrapped is wrapped)\noffset is 1\ncaptured : Function is { value } value + offset\nmake captured\n";
    let diagnostic = analyze_for_compiler(rejected).unwrap_err();
    assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        diagnostic.message.contains("private representation"),
        "{diagnostic:?}"
    );
}

#[test]
fn models_escaping_nested_function_environments_as_private_results() {
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-FUNCTION-NESTED-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/escaping-nested-function-environments.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected escaping nested Function applications")
    };
    assert_eq!(results.len(), 6);
    for (result, expected) in results[..5].iter().zip([43, 44, 45, 46, 47]) {
        assert_eq!(exact_int(result), Some(BigInt::from(expected)));
    }
    assert_eq!(
        results[5].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
    );

    let scalar_factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-operation")
        .collect::<Vec<_>>();
    assert_eq!(scalar_factories.len(), 4);
    assert!(scalar_factories.iter().all(|function| {
        function.result_type == CompilerType::Function
            && function
                .result_captures
                .iter()
                .map(|capture| capture.name.as_str())
                .eq(["offset", "@ context-offset", "root live-offset"])
            && function
                .result_captures
                .iter()
                .all(|capture| capture.path.is_empty())
    }));

    let record_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-record")
        .unwrap();
    assert_eq!(record_factory.result_captures.len(), 4);
    assert!(record_factory.result_captures.iter().all(|capture| {
        capture.path == [CompilerAggregatePathElement::Record("operation".into())]
    }));
    let pair_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-pair-operation")
        .unwrap();
    assert_eq!(pair_factory.result_captures.len(), 1);
    assert_eq!(pair_factory.result_captures[0].name, "pair");
    assert_eq!(
        pair_factory.result_captures[0].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
    );
}

#[test]
fn models_optional_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-OPTIONAL-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-TYPE-MATCH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/optional-function-environments.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Optional Function environment results")
    };
    assert_eq!(results.len(), 13);
    for index in [0, 1, 2, 3, 4, 6, 7, 8, 9, 10] {
        assert_eq!(results[index].value_type, CompilerType::Int);
    }
    for index in [5, 11, 12] {
        assert_eq!(
            results[index].value_type,
            CompilerType::Optional(Box::new(CompilerType::Function))
        );
    }

    let factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-optional")
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 5);
    assert!(factories.iter().all(|function| {
        function.result_captures.len() == 3
            && function
                .result_captures
                .iter()
                .all(|capture| capture.path == [CompilerAggregatePathElement::OptionalPayload])
    }));

    let record_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-record")
        .unwrap();
    assert!(record_factory.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Record("candidate".into()),
                CompilerAggregatePathElement::OptionalPayload,
            ]
    }));
    let tuple_forwarder = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-tuple")
        .unwrap();
    assert!(tuple_forwarder.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Tuple(0),
                CompilerAggregatePathElement::OptionalPayload,
            ]
    }));
    let repeated = program
        .functions
        .iter()
        .find(|function| {
            function.source_name == "<anonymous fn/2>" && function.pattern_identities.len() == 4
        })
        .unwrap();
    assert_eq!(
        repeated.parameters[0].value_type,
        repeated.parameters[1].value_type
    );

    for rejected in [
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nchoose is fn (flag : Boolean) -> Optional Function\n  flag\n    true then Some increment\n    false then Some decrement\nchoose true\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Optional Function\n  nested is fn (value : Int) -> Int\n    operation value\n  Some nested\napply is fn (candidate : Optional Function) -> Int\n  candidate\n    Some operation then operation 41\n    None then 0\napply (wrap increment)\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (candidate : Optional Function) -> Optional Function\n  nested is fn (value : Int) -> Int\n    candidate\n      Some operation then operation value\n      None then 0\n  Some nested\nwrap (Some increment)\n",
    ] {
        let diagnostic = analyze_for_compiler(rejected).unwrap_err();
        assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    }
}

#[test]
fn models_sum_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-SUM-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-TYPE-MATCH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/sum-function-environments.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Sum Function environment results")
    };
    assert_eq!(results.len(), 13);
    for index in [0, 1, 2, 3, 4, 6, 7, 8, 9, 10] {
        assert_eq!(results[index].value_type, CompilerType::Int);
    }
    for index in [5, 11, 12] {
        assert!(matches!(results[index].value_type, CompilerType::Sum(_)));
    }

    let factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-operation")
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 5);
    assert!(factories.iter().all(|function| {
        function.result_captures.len() == 3
            && function.result_captures.iter().all(|capture| {
                capture.path == [CompilerAggregatePathElement::SumPayload("Apply".into())]
            })
    }));
    let choice_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-choice")
        .unwrap();
    let CompilerType::Sum(choice) = &choice_factory.result_type else {
        panic!("expected a Choice result")
    };
    let choice_payload = choice.alternatives[0].name.clone();
    assert!(choice_factory.result_captures.iter().all(|capture| {
        capture.path
            == [CompilerAggregatePathElement::SumPayload(
                choice_payload.clone(),
            )]
    }));
    let repeated = program
        .functions
        .iter()
        .find(|function| {
            function.source_name == "<anonymous fn/2>" && function.pattern_identities.len() == 4
        })
        .unwrap();
    assert_eq!(
        repeated.parameters[0].value_type,
        repeated.parameters[1].value_type
    );

    let record = analyze_for_compiler(
            "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\ncontext-offset is 40\nmake is fn (offset : Int) -> Record (candidate : Operation, value : Int)\n  increase is fn (operand : Int) -> Int\n    operand + offset + @ context-offset + (root live-offset)\n  (candidate is Apply increase, value is 1)\napply is fn (package : Record (candidate : Operation, value : Int)) -> Int\n  package candidate\n    Apply operation then operation (package value)\n    Missing then 0\nlive-offset is 1\napply (make 1)\n",
        )
        .unwrap();
    let record_factory = record
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .unwrap();
    assert!(record_factory.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Record("candidate".into()),
                CompilerAggregatePathElement::SumPayload("Apply".into()),
            ]
    }));

    for rejected in [
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nchoose is fn (flag : Boolean) -> Operation\n  flag\n    true then Apply increment\n    false then Apply decrement\nchoose true\n",
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Operation\n  nested is fn (value : Int) -> Int\n    operation value\n  Apply nested\napply is fn (candidate : Operation) -> Int\n  candidate\n    Apply operation then operation 41\n    Missing then 0\napply (wrap increment)\n",
        "use language (version is v0.1)\nOperation is Union\n  Apply : Function\n  Missing\n\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (candidate : Operation) -> Operation\n  nested is fn (value : Int) -> Int\n    candidate\n      Apply operation then operation value\n      Missing then 0\n  Apply nested\nwrap (Apply increment)\n",
    ] {
        let diagnostic = analyze_for_compiler(rejected).unwrap_err();
        assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    }
}

#[test]
fn models_result_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-RESULT-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-ARITHMETIC-RESULT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/result-function-environments.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Result Function environment results")
    };
    assert_eq!(results.len(), 16);
    for index in [0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13] {
        assert_eq!(results[index].value_type, CompilerType::Int);
    }
    for index in [5, 14, 15] {
        assert_eq!(
            results[index].value_type,
            CompilerType::Result(Box::new(CompilerType::Function))
        );
    }

    let factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-result")
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 7);
    assert!(factories.iter().all(|function| {
        function.result_captures.len() == 3
            && function
                .result_captures
                .iter()
                .all(|capture| capture.path == [CompilerAggregatePathElement::ResultSuccess])
    }));

    let record_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-record")
        .unwrap();
    assert!(record_factory.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Record("candidate".into()),
                CompilerAggregatePathElement::ResultSuccess,
            ]
    }));
    let tuple_forwarder = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-tuple")
        .unwrap();
    assert!(tuple_forwarder.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Tuple(0),
                CompilerAggregatePathElement::ResultSuccess,
            ]
    }));
    let fallible = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-fallible")
        .collect::<Vec<_>>();
    assert_eq!(fallible.len(), 2);
    assert!(fallible.iter().all(|function| {
        function.result_captures.len() == 5
            && function
                .result_captures
                .iter()
                .all(|capture| capture.path == [CompilerAggregatePathElement::ResultSuccess])
    }));
    let projected = program
        .functions
        .iter()
        .filter(|function| function.source_name == "project-result")
        .collect::<Vec<_>>();
    assert_eq!(projected.len(), 2);
    assert!(projected.iter().all(|function| {
        !function.result_captures.is_empty()
            && function
                .result_captures
                .iter()
                .all(|capture| capture.path == [CompilerAggregatePathElement::ResultSuccess])
    }));

    for rejected in [
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increment\nright is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  decrement\nchoose is fn (flag : Boolean) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  nested is fn (value : Int) -> Int\n    operation value\n  nested\nwrap increment\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increment\nwrap is fn (candidate : Result (Function, lang arithmetic ArithmeticErrorCode)) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  nested is fn (value : Int) -> Int\n    candidate\n      Ok operation then operation value\n      Error problem then 0\n  nested\nwrap (source ())\n",
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Result (Function, lang arithmetic ArithmeticErrorCode)\n  increase is fn (value : Int) -> Int\n    value + offset\n  increase\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
    ] {
        let diagnostic = analyze_for_compiler(rejected).unwrap_err();
        assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    }
}

#[test]
fn models_list_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-LIST-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-TYPE-LIST-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-function-environments.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected List Function environment results")
    };
    assert_eq!(results.len(), 13);
    for index in [0, 1, 2, 3, 4, 6, 7, 8, 9, 10] {
        assert_eq!(results[index].value_type, CompilerType::Int);
    }
    for index in [5, 11, 12] {
        assert_eq!(
            results[index].value_type,
            CompilerType::List(Box::new(CompilerType::Function))
        );
    }

    let factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-list")
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 6);
    assert!(factories.iter().all(|function| {
        function.result_captures.len() == 3
            && function
                .result_captures
                .iter()
                .all(|capture| capture.path == [CompilerAggregatePathElement::ListEntry(1)])
    }));

    let record_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-record")
        .unwrap();
    assert!(record_factory.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Record("candidate".into()),
                CompilerAggregatePathElement::ListEntry(1),
            ]
    }));
    let tuple_forwarder = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-tuple")
        .unwrap();
    assert!(tuple_forwarder.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Tuple(0),
                CompilerAggregatePathElement::ListEntry(1),
            ]
    }));

    for rejected in [
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> List Function\n  Entry (increment, Empty)\nright is fn () -> List Function\n  Entry (decrement, Empty)\nchoose is fn (flag : Boolean) -> List Function\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> List Function\n  nested is fn (value : Int) -> Int\n    operation value\n  Entry (nested, Empty)\nwrap increment\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> List Function\n  Entry (increment, Empty)\nwrap is fn (candidate : List Function) -> List Function\n  nested is fn (value : Int) -> Int\n    candidate\n      Entry (operation, remaining) then operation value\n      Empty then 0\n  Entry (nested, Empty)\nwrap (source ())\n",
        "use language (version is v0.1)\nmake is fn (offset : Int) -> List Function\n  increase is fn (value : Int) -> Int\n    value + offset\n  Entry (increase, Empty)\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
    ] {
        let diagnostic = analyze_for_compiler(rejected).unwrap_err();
        assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    }
}

#[test]
fn models_array_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-ARRAY-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-ARRAY-COLLECT-001, TOPAL-ARRAY-GET-CHECKED-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/array-function-environments.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Array Function environment results")
    };
    assert_eq!(results.len(), 15);
    for index in [0, 1, 2, 3, 4, 5, 7, 8, 9, 11] {
        assert_eq!(results[index].value_type, CompilerType::Int);
    }
    assert_eq!(results[10].value_type, CompilerType::Nat);
    assert_eq!(results[12].value_type, CompilerType::Boolean);
    assert_eq!(
        results[6].value_type,
        CompilerType::Array {
            count: 1,
            element: Box::new(CompilerType::Function),
        }
    );
    assert_eq!(
        results[13].value_type,
        CompilerType::Array {
            count: 2,
            element: Box::new(CompilerType::Function),
        }
    );
    assert_eq!(
        results[14].value_type,
        CompilerType::Array {
            count: 0,
            element: Box::new(CompilerType::Function),
        }
    );

    let factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-array")
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 6);
    assert!(factories.iter().all(|function| {
        function.result_captures.len() == 3
            && function
                .result_captures
                .iter()
                .all(|capture| capture.path == [CompilerAggregatePathElement::ArrayEntry(1)])
    }));

    let record_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-record")
        .unwrap();
    assert!(record_factory.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Record("candidate".into()),
                CompilerAggregatePathElement::ArrayEntry(1),
            ]
    }));
    let tuple_forwarder = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-tuple")
        .unwrap();
    assert!(tuple_forwarder.result_captures.iter().all(|capture| {
        capture.path
            == [
                CompilerAggregatePathElement::Tuple(0),
                CompilerAggregatePathElement::ArrayEntry(1),
            ]
    }));

    for rejected in [
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Array (1, Function)\n  values : List Function is Entry (increment, Empty)\n  values collect Array\nright is fn () -> Array (1, Function)\n  values : List Function is Entry (decrement, Empty)\n  values collect Array\nchoose is fn (flag : Boolean) -> Array (1, Function)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Array (1, Function)\n  nested is fn (value : Int) -> Int\n    operation value\n  values : List Function is Entry (nested, Empty)\n  values collect Array\nwrap increment\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Array (1, Function)\n  values : List Function is Entry (increment, Empty)\n  values collect Array\nwrap is fn (candidate : Array (1, Function)) -> Array (1, Function)\n  nested is fn (value : Int) -> Int\n    array-at? (candidate, 0)\n      Some operation then operation value\n      None then 0\n  values : List Function is Entry (nested, Empty)\n  values collect Array\nwrap (source ())\n",
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Array (1, Function)\n  increase is fn (value : Int) -> Int\n    value + offset\n  values : List Function is Entry (increase, Empty)\n  values collect Array\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
    ] {
        let diagnostic = analyze_for_compiler(rejected).unwrap_err();
        assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    }
}
