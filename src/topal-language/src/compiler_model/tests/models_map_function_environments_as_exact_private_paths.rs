#[test]
fn models_map_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-MAP-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-MAP-COLLECT-001, TOPAL-MAP-LOOKUP-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/map-function-environments.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Map Function environment results")
    };
    assert_eq!(results.len(), 16);
    for result in &results[..12] {
        assert_eq!(result.value_type, CompilerType::Int);
    }
    assert_eq!(results[12].value_type, CompilerType::Nat);
    assert_eq!(results[13].value_type, CompilerType::Int);
    assert_eq!(results[14].value_type, CompilerType::Boolean);
    assert_eq!(
        results[15].value_type,
        CompilerType::Map {
            key: Box::new(CompilerType::String),
            value: Box::new(CompilerType::Function),
        }
    );

    let factories = program
        .functions
        .iter()
        .filter(|function| function.source_name == "make-map")
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 6);
    assert!(factories.iter().all(|function| {
        function.result_captures.len() == 3
            && function.result_captures.iter().all(|capture| {
                capture.path == [CompilerAggregatePathElement::MapValue("increase".into())]
            })
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
                CompilerAggregatePathElement::MapValue("increase".into()),
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
                CompilerAggregatePathElement::MapValue("increase".into()),
            ]
    }));
    let keep_first = program
        .functions
        .iter()
        .find(|function| function.source_name == "keep-first-map")
        .unwrap();
    assert!(keep_first.result_captures.is_empty());
    let keep_last = program
        .functions
        .iter()
        .find(|function| function.source_name == "keep-last-map")
        .unwrap();
    assert!(keep_last.result_captures.iter().all(|capture| {
        capture.path == [CompilerAggregatePathElement::MapValue("operation".into())]
    }));

    for rejected in [
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\ndecrement is fn (value : Int) -> Int\n  value - 1\nleft is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nright is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", decrement), Empty)\n  collect-map pairs resolving reject\nchoose is fn (flag : Boolean) -> Map (String, Function)\n  flag\n    true then left ()\n    false then right ()\nchoose true\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nwrap is fn (operation : Function) -> Map (String, Function)\n  nested is fn (value : Int) -> Int\n    operation value\n  pairs : List (String, Function) is Entry ((\"operation\", nested), Empty)\n  collect-map pairs resolving reject\nwrap increment\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nwrap is fn (candidate : Map (String, Function)) -> Map (String, Function)\n  nested is fn (value : Int) -> Int\n    map-lookup (candidate, \"operation\")\n      Some operation then operation value\n      None then 0\n  pairs : List (String, Function) is Entry ((\"nested\", nested), Empty)\n  collect-map pairs resolving reject\nwrap (source ())\n",
        "use language (version is v0.1)\nmake is fn (offset : Int) -> Map (String, Function)\n  increase is fn (value : Int) -> Int\n    value + offset\n  pairs : List (String, Function) is Entry ((\"operation\", increase), Empty)\n  collect-map pairs resolving reject\nsame : Function is { candidate, candidate } 1\nsame (make 1, make 1)\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nsource is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((\"operation\", increment), Empty)\n  collect-map pairs resolving reject\nselect is fn (flag : Boolean) -> String\n  flag\n    true then \"operation\"\n    false then \"missing\"\nlookup is fn (candidate : Map (String, Function), key : String) -> Int\n  map-lookup (candidate, key)\n    Some operation then operation 1\n    None then 0\nlookup (source (), select true)\n",
        "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nmake is fn (key : String) -> Map (String, Function)\n  pairs : List (String, Function) is Entry ((key, increment), Empty)\n  collect-map pairs resolving reject\nmake \"operation\"\n",
        "use language (version is v0.1)\nempty is fn () -> Map (String, Function)\n  pairs : List (String, Function) is Empty\n  collect-map pairs resolving reject\nempty ()\n",
    ] {
        let diagnostic = analyze_for_compiler(rejected).unwrap_err();
        assert_eq!(diagnostic.code, "E-COMPILER-UNSUPPORTED");
    }
}

#[test]
fn models_repeated_anonymous_pattern_names_as_exact_identity_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-anonymous-patterns.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected repeated-pattern results")
    };
    assert_eq!(results.len(), 5);
    assert_eq!(exact_int(&results[0]), Some(BigInt::from(42)));
    assert_eq!(results[1].value_type, CompilerType::Int);
    assert!(matches!(
        results[1].kind,
        CompilerExpressionKind::PrivateBinding { .. }
    ));
    assert_eq!(exact_int(&results[2]), Some(BigInt::from(7)));
    assert_eq!(results[3].value_type, CompilerType::String);
    assert_eq!(exact_int(&results[4]), Some(BigInt::from(42)));

    let guarded = program
        .functions
        .iter()
        .filter(|function| !function.pattern_identities.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 5);
    assert!(guarded.iter().all(|function| {
        function.pattern_identities.as_slice()
            == [CompilerPatternIdentity {
                first_parameter: 0,
                repeated_parameter: 1,
                span: function.parameters[1].span,
            }]
            && !function.parameters[0].discarded
            && function.parameters[1].discarded
            && function.parameters[0].name == function.parameters[1].name
    }));

    let classifier_mismatch = analyze_for_compiler(
            "use language (version is v0.1)\noperation : Function is { (value, value) } value\noperation (1, 1.0)\n",
        )
        .unwrap_err();
    assert_eq!(
        classifier_mismatch.code,
        "E-ANONYMOUS-PATTERN-IDENTITY-CLASSIFIER"
    );

    let unsupported_identity = analyze_for_compiler(
            "use language (version is v0.1)\noperation : Function is { (value, value) } value\noperation (0 .. 1, 0 .. 1)\n",
        )
        .unwrap_err();
    assert_eq!(unsupported_identity.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_repeated_anonymous_aggregate_values_as_exact_identity_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-anonymous-aggregate-patterns.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected repeated aggregate-pattern results")
    };
    assert_eq!(results.len(), 4);
    assert!(matches!(results[0].value_type, CompilerType::Tuple(_)));
    assert!(matches!(results[1].value_type, CompilerType::Record(_)));
    assert!(matches!(results[2].value_type, CompilerType::Optional(_)));
    assert!(matches!(results[3].value_type, CompilerType::List(_)));

    let guarded = program
        .functions
        .iter()
        .filter(|function| !function.pattern_identities.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 4);
    assert!(guarded.iter().all(|function| {
        function.pattern_identities.as_slice()
            == [CompilerPatternIdentity {
                first_parameter: 0,
                repeated_parameter: 1,
                span: function.parameters[1].span,
            }]
            && !function.parameters[0].discarded
            && function.parameters[1].discarded
            && function.parameters[0].value_type == function.parameters[1].value_type
    }));

    let unsupported_result = analyze_for_compiler(
            "use language (version is v0.1)\ndivide is fn (value : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  1.0 / value\noperation : Function is { value, value } value\noperation (divide 2.0, divide 2.0)\n",
        )
        .unwrap_err();
    assert_eq!(unsupported_result.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        unsupported_result
            .message
            .contains("repeated anonymous pattern identity for `Result")
    );
}

#[test]
fn models_repeated_sum_values_as_active_payload_identity_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-SUM-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-sum-patterns.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected repeated Sum-pattern results")
    };
    assert_eq!(results.len(), 5);
    assert!(
        results
            .iter()
            .all(|result| matches!(result.value_type, CompilerType::Sum(_)))
    );

    let guarded = program
        .functions
        .iter()
        .filter(|function| !function.pattern_identities.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 5);
    assert!(guarded.iter().all(|function| {
        function.pattern_identities.as_slice()
            == [CompilerPatternIdentity {
                first_parameter: 0,
                repeated_parameter: 1,
                span: function.parameters[1].span,
            }]
            && matches!(function.parameters[0].value_type, CompilerType::Sum(_))
            && function.parameters[0].value_type == function.parameters[1].value_type
            && compiler_equality_supported(&function.parameters[0].value_type)
    }));

    let unsupported = analyze_for_compiler(
            "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\nrepeat : Function is { value, value } value\nwindow : Window is Bounded (0 ..= 1)\nrepeat (window, window)\n",
        )
        .unwrap_err();
    assert_eq!(unsupported.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        unsupported
            .message
            .contains("repeated anonymous pattern identity for `Window`"),
        "{unsupported:?}"
    );
}

#[test]
fn models_capture_free_function_aggregate_pattern_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001, TOPAL-TYPE-MATCH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-function-aggregate-patterns.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected repeated Function-aggregate pattern results")
    };
    assert_eq!(results.len(), 3);
    assert!(
        results
            .iter()
            .all(|result| exact_int(result) == Some(BigInt::from(42)))
    );

    let guarded = program
        .functions
        .iter()
        .filter(|function| !function.pattern_identities.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 3);
    assert!(guarded.iter().all(|function| {
        function.pattern_identities.as_slice()
            == [CompilerPatternIdentity {
                first_parameter: 0,
                repeated_parameter: 1,
                span: function.parameters[1].span,
            }]
            && compiler_type_is_function_aggregate(&function.parameters[0].value_type)
            && function.parameters[0].value_type == function.parameters[1].value_type
    }));
}

#[test]
fn models_captured_anonymous_function_pattern_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-TYPE-MATCH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-captured-function-patterns.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected repeated captured-Function pattern results")
    };
    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .all(|result| exact_int(result) == Some(BigInt::from(42)))
    );

    let guarded = program
        .functions
        .iter()
        .filter(|function| function.pattern_identities.len() == 2)
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 2);
    for function in guarded {
        assert_eq!(function.parameters.len(), 4);
        assert_eq!(function.parameters[0].value_type, CompilerType::Function);
        assert_eq!(function.parameters[1].value_type, CompilerType::Function);
        assert_eq!(function.parameters[2].value_type, CompilerType::Int);
        assert_eq!(function.parameters[3].value_type, CompilerType::Int);
        assert!(function.parameters[0].source_visible);
        assert!(
            function.parameters[1..]
                .iter()
                .all(|parameter| !parameter.source_visible)
        );
        assert_eq!(
            function
                .pattern_identities
                .iter()
                .map(|identity| (identity.first_parameter, identity.repeated_parameter))
                .collect::<Vec<_>>(),
            [(0, 1), (2, 3)]
        );
    }

    let ordered = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn (offset : Int, marker : String) -> Function\n  operation : Function is { value } (value + offset, marker)\n  operation\nrepeat : Function is { operation, operation } 42\nrepeat (make (1, \"same\"), make (1, \"same\"))\n",
        )
        .unwrap();
    let ordered = ordered
        .functions
        .iter()
        .find(|function| function.pattern_identities.len() == 3)
        .unwrap();
    assert_eq!(
        ordered
            .parameters
            .iter()
            .map(|parameter| parameter.value_type.clone())
            .collect::<Vec<_>>(),
        [
            CompilerType::Function,
            CompilerType::Function,
            CompilerType::String,
            CompilerType::Int,
            CompilerType::String,
            CompilerType::Int,
        ]
    );
    assert_eq!(
        ordered
            .pattern_identities
            .iter()
            .map(|identity| (identity.first_parameter, identity.repeated_parameter))
            .collect::<Vec<_>>(),
        [(0, 1), (2, 4), (3, 5)]
    );

    let unsupported_capture = analyze_for_compiler(
            "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\nmake is fn (window : Window) -> Function\n  operation : Function is { value } window\n  operation\n\nrepeat : Function is { operation, operation } 42\nrepeat (make (Bounded (0 ..= 1)), make (Bounded (0 ..= 1)))\n",
        )
        .unwrap_err();
    assert_eq!(
        unsupported_capture.code, "E-COMPILER-UNSUPPORTED",
        "{unsupported_capture:?}"
    );
    assert!(
        unsupported_capture
            .message
            .contains("without exact capture equality"),
        "{unsupported_capture:?}"
    );
}

#[test]
fn models_captured_named_function_pattern_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-FUNCTION-NESTED-001, TOPAL-TYPE-MATCH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-captured-named-function-patterns.t"
    ))
    .unwrap();
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
    let guarded = program
        .functions
        .iter()
        .find(|function| function.pattern_identities.len() == 2)
        .expect("captured nested Function repetition retains both guards");
    assert_eq!(guarded.parameters.len(), 4);
    assert_eq!(guarded.parameters[0].value_type, CompilerType::Function);
    assert_eq!(guarded.parameters[1].value_type, CompilerType::Function);
    assert_eq!(guarded.parameters[2].value_type, CompilerType::Int);
    assert_eq!(guarded.parameters[3].value_type, CompilerType::Int);
    assert_eq!(
        guarded
            .pattern_identities
            .iter()
            .map(|identity| (identity.first_parameter, identity.repeated_parameter))
            .collect::<Vec<_>>(),
        [(0, 1), (2, 3)]
    );

    let differing = analyze_for_compiler(
            "use language (version is v0.1)\nToken is Union\n  Value : Int\n\ncompare is fn (token : Token) -> Int\n  left is fn (value : Int) -> Token\n    token\n  right is fn (value : Int) -> Token\n    token\n  repeat : Function is { operation, operation } 42\n  repeat (left, right)\ncompare (Value 1)\n",
        )
        .unwrap();
    let differing_guard = differing
        .functions
        .iter()
        .find(|function| function.pattern_identities.len() == 1)
        .expect("different nested declarations retain only their source guard");
    assert_eq!(differing_guard.parameters.len(), 3);
    assert_eq!(
        differing_guard
            .pattern_identities
            .iter()
            .map(|identity| (identity.first_parameter, identity.repeated_parameter))
            .collect::<Vec<_>>(),
        [(0, 1)]
    );

    let unsupported = analyze_for_compiler(
            "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\ncompare is fn (window : Window) -> Int\n  operation is fn (value : Int) -> Window\n    window\n  repeat : Function is { function, function } 42\n  repeat (operation, operation)\ncompare (Bounded (0 ..= 1))\n",
        )
        .unwrap_err();
    assert_eq!(unsupported.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        unsupported
            .message
            .contains("without exact capture equality"),
        "{unsupported:?}"
    );
}

#[test]
fn models_captured_function_aggregate_pattern_identity() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-captured-function-aggregate-patterns.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected repeated captured Function-aggregate pattern results")
    };
    assert_eq!(results.len(), 3);
    assert!(
        results
            .iter()
            .all(|result| exact_int(result) == Some(BigInt::from(42)))
    );

    let guarded = program
        .functions
        .iter()
        .filter(|function| function.pattern_identities.len() == 2)
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 3);
    for function in guarded {
        assert_eq!(function.parameters.len(), 4);
        assert!(compiler_type_is_function_aggregate(
            &function.parameters[0].value_type
        ));
        assert_eq!(
            function.parameters[0].value_type,
            function.parameters[1].value_type
        );
        assert_eq!(function.parameters[2].value_type, CompilerType::Int);
        assert_eq!(function.parameters[3].value_type, CompilerType::Int);
        assert!(function.parameters[0].source_visible);
        assert!(
            function.parameters[1..]
                .iter()
                .all(|parameter| !parameter.source_visible)
        );
        assert_eq!(
            function
                .pattern_identities
                .iter()
                .map(|identity| (identity.first_parameter, identity.repeated_parameter))
                .collect::<Vec<_>>(),
            [(0, 1), (2, 3)]
        );
    }

    let differing_callables = analyze_for_compiler(
            "use language (version is v0.1)\nToken is Union\n  Value : Int\n\nmake-left is fn (token : Token) -> Record (operation : Function)\n  operation : Function is { value } token\n  (operation is operation)\nmake-right is fn (token : Token) -> Record (operation : Function)\n  operation : Function is { value } token\n  (operation is operation)\nrepeat : Function is { package, package } 42\nrepeat (make-left (Value 1), make-right (Value 1))\n",
        )
        .unwrap();
    let differing_guard = differing_callables
        .functions
        .iter()
        .find(|function| function.pattern_identities.len() == 1)
        .expect("differing callable identities retain the aggregate guard");
    assert_eq!(differing_guard.parameters.len(), 3);
    assert_eq!(
        differing_guard
            .pattern_identities
            .iter()
            .map(|identity| (identity.first_parameter, identity.repeated_parameter))
            .collect::<Vec<_>>(),
        [(0, 1)]
    );

    let unsupported_capture = analyze_for_compiler(
            "use language (version is v0.1)\nWindow is Union\n  Bounded : Range Int\n\nmake is fn (window : Window) -> Record (operation : Function)\n  operation : Function is { value } window\n  (operation is operation)\n\nrepeat : Function is { package, package } 42\nrepeat (make (Bounded (0 ..= 1)), make (Bounded (0 ..= 1)))\n",
        )
        .unwrap_err();
    assert_eq!(unsupported_capture.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        unsupported_capture
            .message
            .contains("without exact capture equality"),
        "{unsupported_capture:?}"
    );
}

#[test]
fn models_one_closed_scalar_packaged_function_operand() {
    // TOPAL-COMPILER-PACKAGED-OPERAND-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/packaged-function-operand.t"
    ))
    .unwrap();
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
    let function = &program.functions[0];
    assert_eq!(function.source_name, "sum");
    assert_eq!(function.parameters.len(), 2);
    assert_eq!(function.parameters[0].name, "value");
    assert_eq!(function.parameters[1].name, "fallback");
    let CompilerExpressionKind::Call { arguments, .. } = &program.main.result.kind else {
        panic!("expected a normalized packaged call")
    };
    assert_eq!(arguments.len(), 2);
    assert_eq!(exact_int(&arguments[0]), Some(BigInt::from(40)));
    assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(2)));

    for (call, expected) in [
        ("sum (value is 40, fallback is 5)", 45),
        ("sum (fallback is 2, value is 40)", 42),
        ("sum (40, 2)", 42),
    ] {
        let source = format!(
            "use language (version is v0.1)\nsum is fn ((value : Int, fallback : Int default 2)) -> Int\n  value + fallback\n{call}\n"
        );
        let program = analyze_for_compiler(&source).unwrap();
        assert_eq!(
            exact_int(&program.main.result),
            Some(BigInt::from(expected))
        );
    }

    let missing = analyze_for_compiler(
            "use language (version is v0.1)\nsum is fn ((value : Int, fallback : Int default 2)) -> Int\n  value + fallback\nsum (fallback is 2)\n",
        )
        .unwrap_err();
    assert_eq!(missing.code, "E-NO-APPLICABLE-OVERLOAD");

    let rejected = "use language (version is v0.1)\nsum is fn ((value : Int, fallback : Int default value)) -> Int\n  value + fallback\nsum (value is 40)\n";
    assert_eq!(
        analyze_for_compiler(rejected).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_labeled_package_association_in_declaration_order() {
    // TOPAL-COMPILER-PACKAGED-ASSOCIATION-ORDER-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/packaged-function-association-order.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected packaged association results")
    };
    assert_eq!(results.len(), 3);
    assert!(
        results
            .iter()
            .all(|result| exact_int(result) == Some(BigInt::from(42)))
    );

    let CompilerExpressionKind::PrivateBinding {
        storage_name: right_storage,
        value: right_value,
        body,
    } = &results[0].kind
    else {
        panic!("reordered fields evaluate their first source expression once")
    };
    let CompilerExpressionKind::PrivateBinding {
        storage_name: left_storage,
        value: left_value,
        body,
    } = &body.kind
    else {
        panic!("reordered fields evaluate their second source expression once")
    };
    assert!(matches!(
        right_value.kind,
        CompilerExpressionKind::Call { .. }
    ));
    assert!(matches!(
        left_value.kind,
        CompilerExpressionKind::Call { .. }
    ));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected flattened direct packaged call")
    };
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == left_storage
    ));
    assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(1)));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(storage) if storage == right_storage
    ));

    let aligned_omission = analyze_for_compiler(
            "use language (version is v0.1)\ncombine is fn ((left : Int, offset : Int default 1, right : Int)) -> Int\n  left + offset + right\ncombine (left is 39, right is 2)\n",
        )
        .unwrap();
    let CompilerExpressionKind::PrivateBinding {
        storage_name: left_storage,
        body,
        ..
    } = &aligned_omission.main.result.kind
    else {
        panic!("a supplied field before a non-trailing omission is retained once")
    };
    let CompilerExpressionKind::PrivateBinding {
        storage_name: right_storage,
        body,
        ..
    } = &body.kind
    else {
        panic!("a supplied field after a non-trailing omission is retained once")
    };
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected a declaration-order call after supplied values")
    };
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == left_storage
    ));
    assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(1)));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(storage) if storage == right_storage
    ));
}

#[test]
fn models_compound_packaged_operands_in_source_and_declaration_order() {
    // TOPAL-COMPILER-COMPOUND-PACKAGED-OPERAND-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/compound-packaged-function-operands.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected compound packaged results")
    };
    assert_eq!(results.len(), 5);
    assert!(
        results
            .iter()
            .all(|result| exact_int(result) == Some(BigInt::from(42)))
    );

    let CompilerExpressionKind::PrivateBinding {
        storage_name: left_offset_storage,
        value: left_offset,
        body,
    } = &results[0].kind
    else {
        panic!("the first left-package field is retained first")
    };
    assert_eq!(exact_int(left_offset), Some(BigInt::from(1)));
    let CompilerExpressionKind::PrivateBinding {
        storage_name: left_storage,
        value: left_value,
        body,
    } = &body.kind
    else {
        panic!("the second left-package field is retained second")
    };
    assert!(matches!(
        left_value.kind,
        CompilerExpressionKind::Call { .. }
    ));
    let CompilerExpressionKind::PrivateBinding {
        storage_name: right_storage,
        value: right_value,
        body,
    } = &body.kind
    else {
        panic!("the right-package field is retained after the left package")
    };
    assert!(matches!(
        right_value.kind,
        CompilerExpressionKind::Call { .. }
    ));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one flattened direct compound-package call")
    };
    assert_eq!(arguments.len(), 4);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == left_storage
    ));
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == left_offset_storage
    ));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(storage) if storage == right_storage
    ));
    assert_eq!(exact_int(&arguments[3]), Some(BigInt::from(0)));

    let CompilerExpressionKind::PrivateBinding {
        storage_name: value_storage,
        value,
        body,
    } = &results[3].kind
    else {
        panic!("the packaged mixed operand is retained first")
    };
    assert!(matches!(value.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::PrivateBinding {
        storage_name: factor_storage,
        value: factor,
        body,
    } = &body.kind
    else {
        panic!("the ordinary mixed operand is retained second")
    };
    assert!(matches!(factor.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one flattened direct mixed-package call")
    };
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == value_storage
    ));
    assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(2)));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(storage) if storage == factor_storage
    ));
}

#[test]
fn models_structured_packaged_fields_in_source_and_declaration_order() {
    // TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/structured-packaged-function-fields.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected structured packaged results")
    };
    assert_eq!(results.len(), 4);

    let CompilerExpressionKind::PrivateBinding {
        storage_name: person_storage,
        value: person,
        body,
    } = &results[0].kind
    else {
        panic!("the source-first Record field is retained first")
    };
    assert!(matches!(person.value_type, CompilerType::Record(_)));
    assert!(matches!(person.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::PrivateBinding {
        storage_name: pair_storage,
        value: pair,
        body,
    } = &body.kind
    else {
        panic!("the source-second Tuple field is retained second")
    };
    assert!(matches!(pair.value_type, CompilerType::Tuple(_)));
    assert!(matches!(pair.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one declaration-order structured package call")
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == pair_storage
    ));
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == person_storage
    ));

    let CompilerExpressionKind::PrivateBinding { body, .. } = &results[2].kind else {
        panic!("the explicit Tuple field is retained before its default")
    };
    let CompilerExpressionKind::PrivateBinding {
        storage_name: enabled_storage,
        value: enabled,
        body,
    } = &body.kind
    else {
        panic!("the explicit ordinary operand is retained before the default")
    };
    assert!(matches!(
        enabled.kind,
        CompilerExpressionKind::Boolean(false)
    ));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one structured call with a closed Record default")
    };
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        arguments[1].kind,
        CompilerExpressionKind::Record(_)
    ));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(storage) if storage == enabled_storage
    ));
}

#[test]
fn models_sum_packaged_fields_in_source_and_declaration_order() {
    // TOPAL-COMPILER-SUM-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/sum-packaged-function-fields.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Sum packaged results")
    };
    assert_eq!(results.len(), 4);
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Int)
    );

    let CompilerExpressionKind::PrivateBinding {
        storage_name: fallback_storage,
        value: fallback,
        body,
    } = &results[0].kind
    else {
        panic!("the source-first scalar field is retained first")
    };
    assert_eq!(exact_int(fallback), Some(BigInt::from(0)));
    let CompilerExpressionKind::PrivateBinding {
        storage_name: message_storage,
        value: message,
        body,
    } = &body.kind
    else {
        panic!("the source-second Sum field is retained second")
    };
    assert!(matches!(message.value_type, CompilerType::Sum(_)));
    assert!(matches!(message.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one declaration-order Sum package call")
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == message_storage
    ));
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == fallback_storage
    ));

    let CompilerExpressionKind::PrivateBinding { body, .. } = &results[1].kind else {
        panic!("the explicit fallback is retained before the Sum default")
    };
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one Sum package call with a closed default")
    };
    assert!(matches!(
        arguments[0].kind,
        CompilerExpressionKind::Sum { .. }
    ));
}

#[test]
fn models_function_packaged_fields_with_exact_callable_facts() {
    // TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-packaged-fields.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Function packaged results")
    };
    assert_eq!(results.len(), 4);
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Int)
    );

    let CompilerExpressionKind::PrivateBinding {
        storage_name: value_storage,
        value,
        body,
    } = &results[0].kind
    else {
        panic!("the source-first scalar field is retained first")
    };
    assert!(matches!(value.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::PrivateBinding {
        storage_name: operation_storage,
        value: operation,
        body,
    } = &body.kind
    else {
        panic!("the source-second Function field is retained second")
    };
    assert_eq!(operation.value_type, CompilerType::Function);
    assert!(matches!(
        operation.kind,
        CompilerExpressionKind::Call { .. }
    ));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one declaration-order Function package call")
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == operation_storage
    ));
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == value_storage
    ));

    let captured = program
        .functions
        .iter()
        .find(|function| function.source_name == "apply-captured")
        .expect("capturing Function package call is specialized");
    assert_eq!(captured.parameters.len(), 3);
    assert_eq!(captured.parameters[0].value_type, CompilerType::Function);
    assert!(captured.parameters[0].source_visible);
    assert_eq!(captured.parameters[2].value_type, CompilerType::Int);
    assert!(!captured.parameters[2].source_visible);
}

#[test]
fn models_function_aggregate_packaged_fields_with_exact_callable_facts() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-aggregate-packaged-fields.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Function-aggregate packaged results")
    };
    assert_eq!(results.len(), 3);
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Int)
    );

    let CompilerExpressionKind::PrivateBinding {
        storage_name: addend_storage,
        value: addend,
        body,
    } = &results[0].kind
    else {
        panic!("the source-first scalar field is retained first")
    };
    assert!(matches!(addend.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::PrivateBinding {
        storage_name: bundle_storage,
        value: bundle,
        body,
    } = &body.kind
    else {
        panic!("the source-second Function aggregate field is retained second")
    };
    assert!(compiler_type_is_function_aggregate(&bundle.value_type));
    assert!(matches!(bundle.kind, CompilerExpressionKind::Call { .. }));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one declaration-order Function aggregate package call")
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == bundle_storage
    ));
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == addend_storage
    ));

    let captured = program
        .functions
        .iter()
        .find(|function| function.source_name == "apply-tuple")
        .expect("capturing Function aggregate package call is specialized");
    assert_eq!(captured.parameters.len(), 3);
    assert!(compiler_type_is_function_aggregate(
        &captured.parameters[0].value_type
    ));
    assert!(captured.parameters[0].source_visible);
    assert_eq!(captured.parameters[2].value_type, CompilerType::Int);
    assert!(!captured.parameters[2].source_visible);
}

#[test]
#[allow(clippy::too_many_lines)] // One model check covers ordering for all represented container fields.
fn models_container_packaged_fields_in_source_and_declaration_order() {
    // TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/container-packaged-fields.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected represented-container packaged results")
    };
    assert_eq!(results.len(), 2);

    let CompilerExpressionKind::PrivateBinding {
        storage_name: span_storage,
        value: span,
        body,
    } = &results[0].kind
    else {
        panic!("the source-first Range field is retained first")
    };
    assert_eq!(
        span.value_type,
        CompilerType::Range(Box::new(CompilerType::Int))
    );
    let CompilerExpressionKind::PrivateBinding {
        storage_name: outcome_storage,
        value: outcome,
        body,
    } = &body.kind
    else {
        panic!("the source-second Result field is retained second")
    };
    assert_eq!(
        outcome.value_type,
        CompilerType::Result(Box::new(CompilerType::Int))
    );
    let CompilerExpressionKind::PrivateBinding {
        storage_name: maybe_storage,
        value: maybe,
        body,
    } = &body.kind
    else {
        panic!("the source-third Optional field is retained third")
    };
    assert_eq!(
        maybe.value_type,
        CompilerType::Optional(Box::new(CompilerType::Int))
    );
    let CompilerExpressionKind::PrivateBinding {
        storage_name: values_storage,
        value: values,
        body,
    } = &body.kind
    else {
        panic!("the source-fourth List field is retained fourth")
    };
    assert_eq!(
        values.value_type,
        CompilerType::List(Box::new(CompilerType::Int))
    );
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one declaration-order container package call")
    };
    assert_eq!(arguments.len(), 4);
    for (argument, storage) in [
        (&arguments[0], values_storage),
        (&arguments[1], maybe_storage),
        (&arguments[2], outcome_storage),
        (&arguments[3], span_storage),
    ] {
        assert!(matches!(
            &argument.kind,
            CompilerExpressionKind::Local(actual) if actual == storage
        ));
    }

    let CompilerExpressionKind::PrivateBinding {
        storage_name: second_span_storage,
        body,
        ..
    } = &results[1].kind
    else {
        panic!("the second call retains its explicit Range first")
    };
    let CompilerExpressionKind::PrivateBinding {
        storage_name: second_outcome_storage,
        body,
        ..
    } = &body.kind
    else {
        panic!("the second call retains its explicit Result second")
    };
    let CompilerExpressionKind::PrivateBinding {
        storage_name: second_values_storage,
        body,
        ..
    } = &body.kind
    else {
        panic!("the second call retains its explicit List third")
    };
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected the closed Optional default after explicit values")
    };
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == second_values_storage
    ));
    assert_eq!(
        arguments[1].value_type,
        CompilerType::Optional(Box::new(CompilerType::Int))
    );
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(storage) if storage == second_outcome_storage
    ));
    assert!(matches!(
        &arguments[3].kind,
        CompilerExpressionKind::Local(storage) if storage == second_span_storage
    ));
}

#[test]
#[allow(clippy::too_many_lines)] // One model check covers ordering for all exact collection fields.
fn models_collection_packaged_fields_in_source_and_declaration_order() {
    // TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/collection-packaged-fields.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected exact-collection packaged results")
    };
    assert_eq!(results.len(), 2);

    let CompilerExpressionKind::PrivateBinding {
        storage_name: scores_storage,
        value: scores,
        body,
    } = &results[0].kind
    else {
        panic!("the source-first Map field is retained first")
    };
    assert_eq!(
        scores.value_type,
        CompilerType::Map {
            key: Box::new(CompilerType::String),
            value: Box::new(CompilerType::Int),
        }
    );
    let CompilerExpressionKind::PrivateBinding {
        storage_name: occurrences_storage,
        value: occurrences,
        body,
    } = &body.kind
    else {
        panic!("the source-second Bag field is retained second")
    };
    assert_eq!(
        occurrences.value_type,
        CompilerType::Bag(Box::new(CompilerType::Int))
    );
    let CompilerExpressionKind::PrivateBinding {
        storage_name: members_storage,
        value: members,
        body,
    } = &body.kind
    else {
        panic!("the source-third Set field is retained third")
    };
    assert_eq!(
        members.value_type,
        CompilerType::Set(Box::new(CompilerType::Int))
    );
    let CompilerExpressionKind::PrivateBinding {
        storage_name: array_storage,
        value: array,
        body,
    } = &body.kind
    else {
        panic!("the source-fourth Array field is retained fourth")
    };
    assert_eq!(
        array.value_type,
        CompilerType::Array {
            count: 3,
            element: Box::new(CompilerType::Int),
        }
    );
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected one declaration-order collection package call")
    };
    assert_eq!(arguments.len(), 4);
    for (argument, storage) in [
        (&arguments[0], array_storage),
        (&arguments[1], members_storage),
        (&arguments[2], occurrences_storage),
        (&arguments[3], scores_storage),
    ] {
        assert!(matches!(
            &argument.kind,
            CompilerExpressionKind::Local(actual) if actual == storage
        ));
    }

    let CompilerExpressionKind::Call { arguments, .. } = &results[1].kind else {
        panic!("expected the positional collection package call")
    };
    assert_eq!(arguments.len(), 4);
    assert_eq!(
        arguments[0].value_type,
        CompilerType::Array {
            count: 3,
            element: Box::new(CompilerType::Int),
        }
    );
    assert_eq!(
        arguments[1].value_type,
        CompilerType::Set(Box::new(CompilerType::Int))
    );
    assert_eq!(
        arguments[2].value_type,
        CompilerType::Bag(Box::new(CompilerType::Int))
    );
    assert_eq!(
        arguments[3].value_type,
        CompilerType::Map {
            key: Box::new(CompilerType::String),
            value: Box::new(CompilerType::Int),
        }
    );
    assert!(
        arguments
            .iter()
            .all(|argument| matches!(argument.kind, CompilerExpressionKind::Call { .. }))
    );

    let unsupported = "use language (version is v0.1)\nidentity is fn (values : Set String) -> Set String\n  values\nidentity (collect-set Entry (\"Ada\", Empty))\n";
    assert_eq!(
        analyze_for_compiler(unsupported).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_scope_packaged_fields_as_exact_private_environments() {
    // TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001,
    // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/scope-packaged-fields.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Scope-packaged results")
    };
    assert_eq!(results.len(), 3);

    let CompilerExpressionKind::PrivateBinding {
        storage_name: value_storage,
        value,
        body,
    } = &results[0].kind
    else {
        panic!("the source-first value field is retained first")
    };
    assert_eq!(value.value_type, CompilerType::Int);
    let CompilerExpressionKind::PrivateBinding {
        storage_name: scope_storage,
        value: scope,
        body,
    } = &body.kind
    else {
        panic!("the source-second Scope field is retained second")
    };
    assert_eq!(scope.value_type, CompilerType::Scope);
    assert!(matches!(scope.kind, CompilerExpressionKind::Local(_)));
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected a declaration-order Scope package call")
    };
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        &arguments[0].kind,
        CompilerExpressionKind::Local(storage) if storage == scope_storage
    ));
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == value_storage
    ));
    assert_eq!(arguments[2].value_type, CompilerType::Int);

    let CompilerExpressionKind::Call { arguments, .. } = &results[1].kind else {
        panic!("expected the positional Scope package call")
    };
    assert_eq!(arguments.len(), 3);
    assert_eq!(arguments[0].value_type, CompilerType::Scope);
    assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(41)));
    assert_eq!(arguments[2].value_type, CompilerType::Int);

    let CompilerExpressionKind::PrivateBinding {
        storage_name: default_value_storage,
        body,
        ..
    } = &results[2].kind
    else {
        panic!("the explicit field precedes the closed Scope default")
    };
    let CompilerExpressionKind::Call { arguments, .. } = &body.kind else {
        panic!("expected the defaulted Scope package call")
    };
    assert_eq!(arguments.len(), 3);
    assert!(matches!(arguments[0].kind, CompilerExpressionKind::Root));
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == default_value_storage
    ));
    assert_eq!(arguments[2].value_type, CompilerType::Int);

    let specialized = program
        .functions
        .iter()
        .find(|function| function.source_name == "observe")
        .expect("Scope package call is specialized");
    assert_eq!(specialized.parameters.len(), 3);
    assert_eq!(specialized.parameters[0].name, "scope");
    assert_eq!(specialized.parameters[0].value_type, CompilerType::Scope);
    assert!(specialized.parameters[0].source_visible);
    assert_eq!(specialized.parameters[1].name, "value");
    assert!(specialized.parameters[1].source_visible);
    assert_eq!(specialized.parameters[2].name, "scope answer");
    assert_eq!(specialized.parameters[2].value_type, CompilerType::Int);
    assert!(specialized.parameters[2].source_visible);

    let live_root = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\naccept is fn ((api : Scope, value : Int)) -> Int\n  api answer + value\nwrapper is fn () -> Int\n  accept (api is root, value is 0)\nwrapper ()\n",
        )
        .unwrap_err();
    assert_eq!(live_root.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        live_root
            .message
            .contains("function-body live root Scope argument"),
        "{live_root:?}"
    );

    let non_root = analyze_for_compiler(
            "use language (version is v0.1, features is (lint))\nlint-scope : Scope is lang lint\naccept is fn ((api : Scope)) -> Int\n  0\naccept (api is lint-scope)\n",
        )
        .unwrap_err();
    assert_eq!(non_root.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        non_root.message.contains("non-root Scope argument"),
        "{non_root:?}"
    );
}

#[test]
fn models_discarded_function_parameters_without_binding_them() {
    // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-PATTERN-001
    let source = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (0, 42)\n";
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "second")
        .unwrap();
    assert!(function.parameters[0].discarded);
    assert!(!function.parameters[1].discarded);
    assert!(matches!(
        function.body.result.kind,
        CompilerExpressionKind::Local(ref name) if name == "value"
    ));

    let mismatch = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (\"ignored\", 42)\n";
    assert_eq!(
        analyze_for_compiler(mismatch).unwrap_err().code,
        "E-NO-APPLICABLE-OVERLOAD"
    );
}

#[test]
fn models_ordered_overloads_and_static_call_boundaries() {
    // TOPAL-FUNCTION-OVERLOAD-001, TOPAL-FUNCTION-STATIC-NULLARY-001,
    // TOPAL-FUNCTION-STATIC-UNARY-001, TOPAL-FUNCTION-STATIC-BINARY-001
    let source = "use language (version is v0.1)\ndescribe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  describe 42\nanswer is fn static () -> Int\n  42\nadd is fn static (left : Int, right : Int) -> Int\n  left + right\n(describe \"Topal\", answer (), 20 add 22)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program
            .functions
            .iter()
            .filter(|function| function.source_name == "describe")
            .count(),
        2
    );
    assert_eq!(
        program
            .functions
            .iter()
            .filter(|function| function.is_static)
            .count(),
        2
    );

    let duplicate = "use language (version is v0.1)\nsame is fn (first : Int) -> Int\n  first\nsame is fn (second : Int) -> String\n  \"duplicate\"\nsame 1\n";
    assert_eq!(
        analyze_for_compiler(duplicate).unwrap_err().code,
        "E-DUPLICATE-FUNCTION-OVERLOAD"
    );

    let invalid_static_call = "use language (version is v0.1)\nruntime is fn () -> Int\n  42\nanswer is fn static () -> Int\n  runtime ()\nanswer ()\n";
    assert_eq!(
        analyze_for_compiler(invalid_static_call).unwrap_err().code,
        "E-NO-APPLICABLE-OVERLOAD"
    );
}

#[test]
fn models_same_named_cross_overload_call_as_acyclic() {
    // TOPAL-FUNCTION-RECURSION-OVERLOAD-IDENTITY-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/overload-recursion-identity.t"
    ))
    .unwrap();
    let integer = program
        .functions
        .iter()
        .find(|function| function.parameters[0].value_type == CompilerType::Int)
        .unwrap();
    let string = program
        .functions
        .iter()
        .find(|function| function.parameters[0].value_type == CompilerType::String)
        .unwrap();
    assert_ne!(integer.symbol, string.symbol);
    let CompilerExpressionKind::StringConcat { left, .. } = &string.body.result.kind else {
        panic!("expected outer String concatenation")
    };
    let CompilerExpressionKind::StringConcat { left, .. } = &left.kind else {
        panic!("expected inner String concatenation")
    };
    let CompilerExpressionKind::Call { symbol, arguments } = &left.kind else {
        panic!("expected the cross-overload call")
    };
    assert_eq!(symbol, &integer.symbol);
    assert_ne!(symbol, &string.symbol);
    assert_eq!(arguments[0].value_type, CompilerType::Int);
}

#[test]
fn models_only_complete_uniform_mutual_int_cycles() {
    // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001,
    // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001
    let source = include_str!("../../../../../examples/language/mutual-int-recursion.t")
        .replace("(even 6, odd 6)", "even 6");
    let program = analyze_for_compiler(&source).unwrap();
    assert_eq!(program.functions.len(), 2);
    let even = program
        .functions
        .iter()
        .find(|function| function.source_name == "even")
        .unwrap();
    let odd = program
        .functions
        .iter()
        .find(|function| function.source_name == "odd")
        .unwrap();
    for (function, target) in [(even, odd), (odd, even)] {
        assert!(function.parameters[0].int_range.is_none());
        let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
            &function.body.result.kind
        else {
            panic!("expected a proven mutual recursion decision")
        };
        let CompilerExpressionKind::Call { symbol, .. } = &otherwise.kind else {
            panic!("expected the next mutual edge")
        };
        assert_eq!(symbol, &target.symbol);
    }
    assert!(
        analyze_for_compiler(include_str!(
            "../../../../../examples/language/mutual-increasing-int-recursion.t"
        ))
        .is_ok()
    );
    assert!(
        analyze_for_compiler(include_str!(
            "../../../../../examples/language/mutual-multiple-recursive-calls.t"
        ))
        .is_ok()
    );

    for invalid in [
        "use language (version is v0.1)\nfirst is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    >= 0 then false\n    otherwise first (value + 1)\nfirst 2\n",
        "use language (version is v0.1)\nfirst is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 0)\nfirst 2\n",
    ] {
        assert_eq!(
            analyze_for_compiler(invalid).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_only_range_preserving_mutual_nat_cycles() {
    // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001,
    // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001
    let source = include_str!("../../../../../examples/language/nat-mutual-recursion.t")
        .replace("(even 8, odd 8)", "even 8");
    let program = analyze_for_compiler(&source).unwrap();
    assert_eq!(program.functions.len(), 2);
    let even = program
        .functions
        .iter()
        .find(|function| function.source_name == "even")
        .unwrap();
    let odd = program
        .functions
        .iter()
        .find(|function| function.source_name == "odd")
        .unwrap();
    for (function, target) in [(even, odd), (odd, even)] {
        assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
        assert!(function.parameters[0].int_range.is_none());
        let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
            &function.body.result.kind
        else {
            panic!("expected a proven mutual Nat recursion decision")
        };
        let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
            panic!("expected the next mutual Nat edge")
        };
        assert_eq!(symbol, &target.symbol);
        let [argument] = arguments.as_slice() else {
            panic!("expected one recursive Nat argument")
        };
        assert!(matches!(argument.kind, CompilerExpressionKind::IntToNat(_)));
    }

    assert!(
        analyze_for_compiler(include_str!(
            "../../../../../examples/language/nat-mutual-increasing-recursion.t"
        ))
        .is_ok()
    );

    let unsafe_overshoot = include_str!("../../../../../examples/language/nat-mutual-recursion.t")
        .replace("<= 2", "<= 1");
    assert_eq!(
        analyze_for_compiler(&unsafe_overshoot).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}
