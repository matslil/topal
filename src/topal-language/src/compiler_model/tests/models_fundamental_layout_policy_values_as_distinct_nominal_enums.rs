#[test]
fn models_fundamental_layout_policy_values_as_distinct_nominal_enums() {
    // TOPAL-LAYOUT-ENDIAN-001, TOPAL-LAYOUT-ACCESS-001,
    // TOPAL-LAYOUT-BIT-ORDER-001, TOPAL-LAYOUT-PACKING-001,
    // TOPAL-LAYOUT-FIELD-ORDER-001, TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001,
    // TOPAL-LAYOUT-ABSENCE-POLICY-001,
    // TOPAL-COMPILER-LAYOUT-POLICY-001
    let cases = [
        (
            include_str!("../../../../../examples/language/layout-endian.t"),
            "Endian",
            vec![0, 1],
        ),
        (
            include_str!("../../../../../examples/language/layout-access.t"),
            "Access",
            vec![0, 1, 2, 3],
        ),
        (
            include_str!("../../../../../examples/language/layout-bit-order.t"),
            "BitOrder",
            vec![0, 1],
        ),
        (
            include_str!("../../../../../examples/language/layout-packing.t"),
            "Packing",
            vec![0, 1],
        ),
        (
            include_str!("../../../../../examples/language/layout-field-order.t"),
            "FieldOrder",
            vec![0],
        ),
        (
            include_str!("../../../../../examples/language/layout-payload-placement.t"),
            "PayloadPlacement",
            vec![0, 1],
        ),
        (
            include_str!("../../../../../examples/language/layout-absence-policies.t"),
            "LayoutPolicy",
            vec![0, 1],
        ),
    ];
    for (source, expected_type, expected_values) in cases {
        let program = analyze_for_compiler(source).unwrap();
        let values = match &program.main.result.kind {
            CompilerExpressionKind::Enum(value) => vec![(&program.main.result, *value)],
            CompilerExpressionKind::Tuple(fields) => fields
                .iter()
                .map(|field| {
                    let CompilerExpressionKind::Enum(value) = field.kind else {
                        panic!("layout policy example contains only enum values")
                    };
                    (field, value)
                })
                .collect(),
            _ => panic!("layout policy example returns one enum or a Tuple"),
        };
        assert_eq!(
            values.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
            expected_values
        );
        assert!(values.iter().all(|(expression, _)| matches!(
            &expression.value_type,
            CompilerType::Enum(enumeration) if enumeration.name == expected_type
        )));
    }

    let mismatch = "use language (version is v0.1)\nLittle = Natural\n";
    assert_eq!(
        analyze_for_compiler(mismatch).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
}

#[test]
fn models_nominal_enum_values_functions_and_exhaustive_decisions() {
    // TOPAL-TYPE-ENUM-001, TOPAL-DECISION-ENUM-001
    let source = "use language (version is v0.1)\nColor is Enum (Red, Green, Blue)\nnext is fn (value : Color) -> Color\n  value\n    Red then Green\n    Green then Blue\n    Blue then Red\nfavorite : Color is Red\n(next favorite, next Green, Red = Green)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Color, Color, Boolean)"
    );
    let enumeration = CompilerEnumType {
        name: "Color".into(),
        alternatives: vec!["Red".into(), "Green".into(), "Blue".into()],
    };
    assert!(program.functions.iter().any(|function| {
        function.parameters[0].value_type == CompilerType::Enum(enumeration.clone())
            && matches!(
                &function.body.result.kind,
                CompilerExpressionKind::EnumDecision {
                    rules,
                    otherwise: None,
                    ..
                } if rules.len() == 3
            )
    }));
}

#[test]
fn rejects_invalid_nominal_enum_declarations_and_decisions() {
    // TOPAL-TYPE-ENUM-001, TOPAL-DECISION-ENUM-001
    let duplicate = "use language (version is v0.1)\nColor is Enum (Red, Red)\nRed\n";
    assert_eq!(
        analyze_for_compiler(duplicate).unwrap_err().code,
        "E-DUPLICATE-ENUM-ALTERNATIVE"
    );

    let type_as_alternative =
        "use language (version is v0.1)\nColor is Enum (Color, Green)\nGreen\n";
    assert_eq!(
        analyze_for_compiler(type_as_alternative).unwrap_err().code,
        "E-DUPLICATE-ENUM-ALTERNATIVE"
    );

    let incomplete = "use language (version is v0.1)\nColor is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"red\"\nname Green\n";
    assert_eq!(
        analyze_for_compiler(incomplete).unwrap_err().code,
        "E-INCOMPLETE-DECISION"
    );

    let nominal_mismatch = "use language (version is v0.1)\nColor is Enum (Red, Green)\nSignal is Enum (Stop, Go)\n(Red = Stop)\n";
    assert_eq!(
        analyze_for_compiler(nominal_mismatch).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );

    let before_declaration =
        "use language (version is v0.1)\nvalue is Red\nColor is Enum (Red, Green)\nvalue\n";
    assert_eq!(
        analyze_for_compiler(before_declaration).unwrap_err().code,
        "E-UNBOUND-NAME"
    );

    let nested = "use language (version is v0.1)\nmake is fn () -> Unit\n  Local is Enum (First, Second)\n  ()\nmake ()\n";
    assert_eq!(
        analyze_for_compiler(nested).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_nominal_union_and_variant_payload_decisions() {
    // TOPAL-TYPE-UNION-001, TOPAL-TYPE-VARIANT-001,
    // TOPAL-DECISION-UNION-001
    let source = include_str!("../../../../../examples/language/unions-and-recursive-products.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "((Int, (Int, Int)), (Int, (Int, Int)), String, String)"
    );
    let describe = program
        .functions
        .iter()
        .find(|function| function.source_name == "describe")
        .expect("shared regression instantiates describe");
    let CompilerType::Sum(message) = &describe.parameters[0].value_type else {
        panic!("describe retains the nominal Message type")
    };
    assert!(!message.positional);
    assert_eq!(message.alternatives.len(), 2);
    assert!(message.alternatives[0].payload.is_none());
    let CompilerExpressionKind::SumDecision { rules, .. } = &describe.body.result.kind else {
        panic!("describe lowers a sum decision")
    };
    assert_eq!(rules.len(), 2);
    assert!(rules.iter().any(|rule| rule.binding.is_some()));

    let show = program
        .functions
        .iter()
        .find(|function| function.source_name == "show-scalar")
        .expect("shared regression instantiates show-scalar");
    let CompilerType::Sum(scalar) = &show.parameters[0].value_type else {
        panic!("show-scalar retains the nominal Scalar type")
    };
    assert!(scalar.positional);
    assert!(
        scalar
            .alternatives
            .iter()
            .all(|alternative| alternative.payload.is_some())
    );
}

#[test]
fn models_derived_nominal_sum_equality() {
    // TOPAL-COMPILER-SUM-EQUALITY-001, TOPAL-TYPE-SUM-EQUALITY-001,
    // TOPAL-TYPE-EQUALITY-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/sum-equality.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("expected Sum equality results")
    };
    assert_eq!(results.len(), 11);
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Boolean)
    );

    for name in ["same-token", "same-choice"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap_or_else(|| panic!("shared regression instantiates {name}"));
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(
            function.parameters[0].value_type,
            function.parameters[1].value_type
        );
        assert!(compiler_equality_supported(
            &function.parameters[0].value_type
        ));
        assert!(matches!(
            function.body.result.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Equal,
                ..
            }
        ));
    }

    let unsupported = analyze_for_compiler(
            "use language (version is v0.1)\nHolder is Union\n  Blank\n  Window : Range Int\n\nleft : Holder is Blank\nright : Holder is Blank\nleft = right\n",
        )
        .unwrap_err();
    assert_eq!(unsupported.code, "E-COMPILER-UNSUPPORTED");
    assert!(unsupported.message.contains("equality for this value type"));

    let nominal = analyze_for_compiler(
            "use language (version is v0.1)\nLeft is Union\n  LeftEmpty\n\nRight is Union\n  RightEmpty\n\nleft : Left is LeftEmpty\nright : Right is RightEmpty\nleft = right\n",
        )
        .unwrap_err();
    assert_eq!(nominal.code, "E-TYPE-MISMATCH");
}

#[test]
fn rejects_invalid_nominal_sum_construction_and_matching() {
    // TOPAL-TYPE-UNION-001, TOPAL-TYPE-VARIANT-001,
    // TOPAL-DECISION-UNION-001
    let wrong_payload = "use language (version is v0.1)\nMessage is Union\n  Move : Int\n\nvalue is Move true\nvalue\n";
    assert_eq!(
        analyze_for_compiler(wrong_payload).unwrap_err().code,
        "E-UNION-PAYLOAD-CLASSIFIER"
    );
    let invalid_index =
        "use language (version is v0.1)\nScalar is Variant (String, Int)\nScalar at 2 42\n";
    assert_eq!(
        analyze_for_compiler(invalid_index).unwrap_err().code,
        "E-VARIANT-INDEX"
    );
    let incomplete = "use language (version is v0.1)\nMessage is Union\n  Stop\n  Move : Int\n\nread is fn (message : Message) -> Int\n  message\n    Stop then 0\nread Stop\n";
    assert_eq!(
        analyze_for_compiler(incomplete).unwrap_err().code,
        "E-INCOMPLETE-DECISION"
    );
    let foreign_variant = "use language (version is v0.1)\nScalar is Variant (String, Int)\nOther is Variant (String, Int)\nread is fn (scalar : Scalar) -> String\n  scalar\n    Other at 0 text then text\n    otherwise \"number\"\nread (Scalar at 0 \"text\")\n";
    assert_eq!(
        analyze_for_compiler(foreign_variant).unwrap_err().code,
        "E-VARIANT-TYPE"
    );
}

#[test]
fn models_nominal_modular_construction_reduction_and_arithmetic() {
    // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001,
    // TOPAL-NUM-MODULAR-REDUCE-001, TOPAL-NUM-MODULAR-ARITHMETIC-001
    let source = include_str!("../../../../../examples/language/modular-numbers.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(ByteCounter, SignedByte, ByteCounter, SignedByte, Boolean, Boolean)"
    );
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("modular regression retains its result product")
    };
    assert!(matches!(
        values[0].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            ..
        }
    ));
    let CompilerExpressionKind::Binary { left, .. } = &values[0].kind else {
        unreachable!("checked above")
    };
    assert!(matches!(
        left.kind,
        CompilerExpressionKind::IntToModular { .. }
    ));
    assert!(matches!(
        values[2].kind,
        CompilerExpressionKind::ModularReduce { .. }
    ));
    assert_eq!(exact_int(&values[0]), Some(BigInt::from(0)));
    assert_eq!(exact_int(&values[1]), Some(BigInt::from(-128)));
    assert_eq!(exact_int(&values[2]), Some(BigInt::from(255)));
    assert_eq!(exact_int(&values[3]), Some(BigInt::from(-128)));
}

#[test]
fn models_named_ranges_and_dynamic_checked_modular_construction() {
    // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001
    let source = include_str!("../../../../../examples/language/modular-checked-construction.t");
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Result (ByteCounter, lang arithmetic ArithmeticErrorCode), Result (ByteCounter, lang arithmetic ArithmeticErrorCode), lang arithmetic ArithmeticErrorCode, ErrorDomain, Optional SourceLocation)"
    );
    assert!(program.functions.iter().any(|function| {
        function.source_name == "construct"
            && matches!(
                function.body.result.kind,
                CompilerExpressionKind::ResultSuccess(_)
            )
    }));
    assert!(program.functions.iter().any(|function| {
        function.source_name == "construct"
            && matches!(
                function.body.result.kind,
                CompilerExpressionKind::ModularValidate { .. }
            )
    }));
}

#[test]
fn rejects_invalid_modular_construction() {
    // TOPAL-NUM-MODULAR-TYPE-001, TOPAL-NUM-MODULAR-CONSTRUCT-001
    let invalid_range = "use language (version is v0.1)\nDigit is ModNat (1 ..= 9)\nDigit 1\n";
    assert_eq!(
        analyze_for_compiler(invalid_range).unwrap_err().code,
        "E-MODULAR-RANGE"
    );

    let outside = "use language (version is v0.1)\nDigit is ModNat (0 ..= 9)\nDigit 10\n";
    assert_eq!(
        analyze_for_compiler(outside).unwrap_err().code,
        "E-MODULAR-OUT-OF-RANGE"
    );

    let nominal_mismatch = "use language (version is v0.1)\nDigit is ModNat (0 ..= 9)\nHour is ModNat (0 ..= 23)\n(Digit 1) + (Hour 1)\n";
    assert_eq!(
        analyze_for_compiler(nominal_mismatch).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
}

#[test]
fn admits_dependency_only_library_selection_and_shared_diagnostics() {
    // TOPAL-SYN-LIBRARY-001, TOPAL-LIB-DEPENDENCY-001,
    // TOPAL-COMPILER-LIBRARY-DEPENDENCY-001
    let source = "use language (version is v0.1)\nuse library std (version is v0.1)\n()\n";
    analyze_for_compiler(source).unwrap();
    analyze_for_compiler(include_str!(
        "../../../../../examples/data-transfer/packet-filter.t"
    ))
    .unwrap();

    assert_eq!(
        analyze_for_compiler(
            "use language (version is v0.1)\nuse library other (version is v0.1)\n()\n"
        )
        .unwrap_err()
        .code,
        "E-UNSUPPORTED-LIBRARY"
    );
    assert_eq!(
        analyze_for_compiler(
            "use language (version is v0.1)\nuse library std (version is v0.2)\n()\n"
        )
        .unwrap_err()
        .code,
        "E-UNSUPPORTED-LIBRARY-VERSION"
    );
    assert_eq!(
            analyze_for_compiler("use language (version is v0.1)\nuse library std (version is v0.1)\nuse library std (version is v0.1)\n()\n")
                .unwrap_err()
                .code,
            "E-DUPLICATE-LIBRARY"
        );
}

#[test]
fn compiles_published_source_module_functions_and_private_helpers() {
    // TOPAL-LIB-SOURCE-001, TOPAL-NAMESPACE-USE-001,
    // TOPAL-COMPILER-LIBRARY-SOURCE-001
    let source = "use language (version is v0.1)\nuse library std (version is v0.1)\naccepted? is std checks values accepted?\naccepted? 7\n";
    let module = CompilerSourceModule {
            identity: vec!["std".into(), "checks".into(), "values".into()],
            source_name: "library/std/checks/values.t".into(),
            source: "#!/usr/bin/env topal\nuse language (version is v0.1)\npositive? is fn (value : Nat) -> Boolean\n  value > 0\npub accepted? is fn (value : Nat) -> Boolean\n  positive? value\n"
                .into(),
        };
    let program = analyze_for_compiler_with_modules(source, &[module]).unwrap();
    assert_eq!(program.primary_source_end, source.len());
    assert_eq!(program.dependencies.len(), 1);
    assert_eq!(program.dependencies[0].identity, "std.checks.values");
    assert!(program.dependencies[0].source_text.starts_with("#!"));
    assert_eq!(program.main.result.value_type, CompilerType::Boolean);
    assert!(
        program
            .functions
            .iter()
            .any(|function| function.source_name == "std.checks.values.accepted?")
    );
    assert!(
        program
            .functions
            .iter()
            .any(|function| function.source_name == "std.checks.values.positive?")
    );

    let private = "use language (version is v0.1)\nuse library std (version is v0.1)\nstd checks values positive? 1\n";
    assert_eq!(
            analyze_for_compiler_with_modules(
                private,
                &[CompilerSourceModule {
                    identity: vec!["std".into(), "checks".into(), "values".into()],
                    source_name: "library/std/checks/values.t".into(),
                    source: "use language (version is v0.1)\npositive? is fn (value : Nat) -> Boolean\n  value > 0\npub accepted? is fn (value : Nat) -> Boolean\n  positive? value\n"
                        .into(),
                }],
            )
            .unwrap_err()
            .code,
            "E-COMPILER-UNSUPPORTED"
        );
}

#[test]
fn compiles_proven_nat_recursion_in_source_modules() {
    // TOPAL-LIB-SOURCE-001, TOPAL-FUNCTION-RECURSION-NAT-001,
    // TOPAL-COMPILER-LIBRARY-SOURCE-001
    let source = "use language (version is v0.1)\nuse library std (version is v0.1)\nfactorial is std maths factorial\nfactorial 5\n";
    let module = CompilerSourceModule {
            identity: vec!["std".into(), "maths".into()],
            source_name: "library/std/maths.t".into(),
            source: "use language (version is v0.1)\npub factorial is fn (count : Nat) -> Nat\n  count\n    <= 1 then 1\n    otherwise count * (factorial (count - 1))\n"
                .into(),
        };
    let program = analyze_for_compiler_with_modules(source, &[module]).unwrap();
    let factorial = program
        .functions
        .iter()
        .find(|function| function.source_name == "std.maths.factorial")
        .expect("qualified recursive specialization is emitted");
    let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
        &factorial.body.result.kind
    else {
        panic!("expected the structurally proven Nat decision")
    };
    let CompilerExpressionKind::Binary { right, .. } = &otherwise.kind else {
        panic!("expected the recursive multiplication")
    };
    let CompilerExpressionKind::Call { arguments, .. } = &right.kind else {
        panic!("expected the qualified recursive call")
    };
    assert!(matches!(
        arguments.as_slice(),
        [CompilerExpression {
            kind: CompilerExpressionKind::IntToNat(_),
            ..
        }]
    ));
}

#[test]
fn models_ordered_custom_generator_overloads_and_final_result_bindings() {
    // TOPAL-GENERATOR-OVERLOAD-001, TOPAL-GENERATOR-FOREACH-RESULT-001,
    // TOPAL-COMPILER-GENERATOR-OVERLOAD-001
    let source = include_str!("../../../../../examples/language/custom-generator-overloads.t");
    let program = analyze_for_compiler(source).unwrap();
    let [
        CompilerStatement::Binding(unary_generator),
        CompilerStatement::Binding(unary_result),
        CompilerStatement::Binding(binary_generator),
        CompilerStatement::Binding(binary_result),
    ] = program.main.statements.as_slice()
    else {
        panic!("overload regression retains four ordered root bindings")
    };
    assert!(matches!(
        &unary_generator.value,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueGenerator {
                declaration,
                initial_parameter,
                initial,
                additional_initial_parameters,
                additional_initials,
                yields,
                result,
                ..
            },
            value_type: CompilerType::Generator(generator),
            ..
        } if declaration == "select"
            && initial_parameter.value_type == CompilerType::Int
            && exact_int(initial) == Some(BigInt::from(7))
            && additional_initial_parameters.is_empty()
            && additional_initials.is_empty()
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && exact_string(result).as_deref() == Some("unary")
            && *generator.yield_type == CompilerType::Int
            && *generator.result_type == CompilerType::String
    ));
    assert!(matches!(
        &binary_generator.value,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueGenerator {
                initial_parameter,
                initial,
                additional_initial_parameters,
                additional_initials,
                prefix,
                yields,
                result,
                ..
            },
            value_type: CompilerType::Generator(generator),
            ..
        } if initial_parameter.name == "value"
            && exact_int(initial) == Some(BigInt::from(7))
            && matches!(additional_initial_parameters.as_slice(), [parameter]
                if parameter.name == "suffix"
                    && parameter.value_type == CompilerType::String)
            && matches!(additional_initials.as_slice(), [suffix]
                if exact_string(suffix).as_deref() == Some("item"))
            && matches!(prefix.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Add,
                        ..
                    },
                    ..
                }
            )])
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Value(value)]
                if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                    if name == "suffix"))
            && exact_string(result).as_deref() == Some("binary")
            && *generator.yield_type == CompilerType::String
            && *generator.result_type == CompilerType::String
    ));
    for binding in [unary_result, binary_result] {
        assert_eq!(binding.value.value_type, CompilerType::String);
        assert!(matches!(
            binding.value.kind,
            CompilerExpressionKind::CustomValueForeach { .. }
        ));
    }
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Tuple(values),
            value_type: CompilerType::Tuple(types),
            ..
        } if values.len() == 2
            && types == &[CompilerType::String, CompilerType::String]
    ));
}

#[test]
fn rejects_invalid_custom_generator_overload_boundaries() {
    // TOPAL-GENERATOR-OVERLOAD-001, TOPAL-GENERATOR-FOREACH-RESULT-001,
    // TOPAL-COMPILER-GENERATOR-OVERLOAD-001
    let source = include_str!("../../../../../examples/language/custom-generator-overloads.t");
    let duplicate = source.replacen(
        "select is generator ( value : Int, suffix : String )",
        "select is generator ( value : Int )",
        1,
    );
    assert_eq!(
        analyze_for_compiler(&duplicate).unwrap_err().code,
        "E-DUPLICATE-GENERATOR-OVERLOAD"
    );

    let reversed = source.replacen("select (7, \"item\")", "select (\"item\", 7)", 1);
    let reversed = analyze_for_compiler(&reversed).unwrap_err();
    assert_eq!(reversed.code, "E-NO-APPLICABLE-GENERATOR-OVERLOAD");
    assert!(reversed.message.contains("(String, Int)"));
    assert!(reversed.message.contains("(Int); (Int, String)"));

    let wrong_result = source.replacen("binary-result : String", "binary-result : Int", 1);
    assert_eq!(
        analyze_for_compiler(&wrong_result).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );

    let altered_prefix = source.replacen("_ is value + 1", "_ is value + 2", 1);
    assert_eq!(
        analyze_for_compiler(&altered_prefix).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );

    let altered_input = source.replacen("select 7", "select 8", 1);
    assert_eq!(
        analyze_for_compiler(&altered_input).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Factory and consumer ownership provenance form one boundary scenario.
fn models_custom_generator_generic_function_boundaries() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-COMPILER-GENERATOR-FUNCTION-BOUNDARY-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-generic-function-boundaries.t"
    );
    let program = analyze_for_compiler(source).unwrap();
    let make = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("called Generator factory is specialized");
    assert!(matches!(
        make.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type: CompilerType::Int,
            ..
        }] if name == "initial"
    ));
    assert!(is_admitted_value_boundary_generator_type(&make.result_type));
    assert!(make.body.statements.is_empty());
    assert!(matches!(
        &make.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueGenerator {
                declaration,
                initial_parameter,
                initial,
                additional_initial_parameters,
                prefix,
                yields,
                continuations,
                explicit_return: None,
                result,
                ..
            },
            ..
        } if declaration == "numbers"
            && initial_parameter.name == "initial"
            && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
            && additional_initial_parameters.is_empty()
            && prefix.statements.is_empty()
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_string(result).as_deref() == Some("done")
    ));

    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("called Generator consumer is specialized");
    assert!(matches!(
        consume.parameters.as_slice(),
        [CompilerParameter { name, value_type, .. }]
            if name == "generated"
                && is_admitted_value_boundary_generator_type(value_type)
    ));
    assert!(matches!(
        consume.body.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomValueForeach {
                            source,
                            transferred_initial: Some(initial),
                            yields,
                            parameter,
                            body,
                            result,
                            ..
                        },
                    value_type: CompilerType::String,
                    ..
                },
            ..
        })] if name == "result"
            && matches!(source.kind, CompilerExpressionKind::Local(ref name)
                if name == "generated")
            && exact_int(initial) == Some(BigInt::from(7))
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && exact_int_value_generator_action(parameter, body)
            && exact_string(result).as_deref() == Some("done")
    ));
    assert!(matches!(
        &consume.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Local(name),
            value_type: CompilerType::String,
            ..
        } if name == "result"
    ));
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind: CompilerExpressionKind::Call { arguments, .. },
                    value_type,
                    ..
                },
            ..
        })] if name == "generated"
            && is_admitted_value_boundary_generator_type(value_type)
            && matches!(arguments.as_slice(), [argument]
                if exact_int(argument) == Some(BigInt::from(7)))
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Call { arguments, .. },
            value_type: CompilerType::String,
            ..
        } if matches!(arguments.as_slice(), [CompilerExpression {
            kind: CompilerExpressionKind::Local(name),
            value_type,
            ..
        }] if name.starts_with("topal.root.")
            && is_admitted_value_boundary_generator_type(value_type))
    ));
}

#[test]
fn rejects_invalid_custom_generator_generic_function_boundaries() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-COMPILER-GENERATOR-FUNCTION-BOUNDARY-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-generic-function-boundaries.t"
    );
    for invalid in [
        source.replacen("make 7", "make 8", 1),
        source.replacen("numbers initial", "numbers (initial + 1)", 1),
        source.replacen("\"done\"", "\"later\"", 1),
        source.replacen("value + 1", "value + 2", 1),
        source.replacen("consume generated", "consume (numbers 7)", 1),
    ] {
        assert_eq!(
            analyze_for_compiler(&invalid).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Aggregate factory and consumer provenance form one boundary scenario.
fn models_custom_generator_compound_function_boundaries() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-compound-function-boundaries.t"
    );
    let program = analyze_for_compiler(source).unwrap();
    let make = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("compound Generator factory is specialized");
    assert!(matches!(
        make.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type,
            ..
        }] if name == "initial" && compiler_int_string_pair(value_type)
    ));
    assert_eq!(make.result_type, product_boundary_generator_type());
    assert!(make.body.statements.is_empty());
    assert!(matches!(
        &make.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueGenerator {
                declaration,
                initial_parameter,
                initial,
                additional_initial_parameters,
                prefix,
                yields,
                continuations,
                explicit_return: None,
                result,
                ..
            },
            value_type,
            ..
        } if declaration == "pairs"
            && initial_parameter.name == "initial"
            && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
            && additional_initial_parameters.is_empty()
            && prefix.statements.is_empty()
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_int_string_product(result, 8, "done")
            && value_type == &product_boundary_generator_type()
    ));

    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("compound Generator consumer is specialized");
    assert!(matches!(
        consume.parameters.as_slice(),
        [CompilerParameter { name, value_type, .. }]
            if name == "generated" && value_type == &product_boundary_generator_type()
    ));
    assert!(matches!(
        consume.body.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomValueForeach {
                            source,
                            transferred_initial: Some(initial),
                            yields,
                            parameter,
                            body,
                            result,
                            ..
                        },
                    value_type,
                    ..
                },
            ..
        })] if name == "result"
            && matches!(source.kind, CompilerExpressionKind::Local(ref name)
                if name == "generated")
            && exact_int_string_product(initial, 7, "item")
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && exact_int_string_product_value_generator_action(parameter, body)
            && exact_int_string_product(result, 8, "done")
            && compiler_int_string_pair(value_type)
    ));
    assert!(matches!(
        &consume.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Local(name),
            value_type,
            ..
        } if name == "result" && compiler_int_string_pair(value_type)
    ));
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind: CompilerExpressionKind::Call { arguments, .. },
                    value_type,
                    ..
                },
            ..
        })] if name == "generated"
            && value_type == &product_boundary_generator_type()
            && matches!(arguments.as_slice(), [argument]
                if exact_int_string_product(argument, 7, "item"))
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Call { arguments, .. },
            value_type,
            ..
        } if compiler_int_string_pair(value_type)
            && matches!(arguments.as_slice(), [CompilerExpression {
                kind: CompilerExpressionKind::Local(name),
                value_type,
                ..
            }] if name.starts_with("topal.root.")
                && value_type == &product_boundary_generator_type())
    ));
}

#[test]
fn rejects_invalid_custom_generator_compound_function_boundaries() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-compound-function-boundaries.t"
    );
    for invalid in [
        source.replacen("make (7, \"item\")", "make (8, \"item\")", 1),
        source.replacen("pairs initial", "pairs (8, \"item\")", 1),
        source.replacen("(8, \"done\")", "(9, \"done\")", 1),
        source.replacen("value = (7, \"item\")", "value = (8, \"item\")", 1),
        source.replacen("consume generated", "consume (pairs (7, \"item\"))", 1),
    ] {
        assert_eq!(
            analyze_for_compiler(&invalid).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Recursive factory and consumer provenance form one boundary scenario.
fn models_custom_generator_nested_function_boundaries() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-TYPE-RESULT-001,
    // TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-nested-function-boundaries.t"
    );
    let program = analyze_for_compiler(source).unwrap();
    let make = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("nested Generator factory is specialized");
    assert!(matches!(
        make.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type,
            ..
        }] if name == "initial" && value_type == &nested_optional_product_type()
    ));
    assert_eq!(make.result_type, nested_boundary_generator_type());
    assert!(make.body.statements.is_empty());
    assert!(matches!(
        &make.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueGenerator {
                declaration,
                initial_parameter,
                initial,
                additional_initial_parameters,
                prefix,
                yields,
                continuations,
                explicit_return: None,
                result,
                ..
            },
            value_type,
            ..
        } if declaration == "pairs"
            && initial_parameter.name == "initial"
            && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
            && additional_initial_parameters.is_empty()
            && prefix.statements.is_empty()
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_nested_result_product_value(result, 8, "done")
            && value_type == &nested_boundary_generator_type()
    ));

    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("nested Generator consumer is specialized");
    assert!(matches!(
        consume.parameters.as_slice(),
        [CompilerParameter { name, value_type, .. }]
            if name == "generated" && value_type == &nested_boundary_generator_type()
    ));
    assert!(matches!(
        consume.body.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomValueForeach {
                            source,
                            transferred_initial: Some(initial),
                            yields,
                            parameter,
                            body,
                            result,
                            ..
                        },
                    value_type,
                    ..
                },
            ..
        })] if name == "result"
            && matches!(source.kind, CompilerExpressionKind::Local(ref name)
                if name == "generated")
            && exact_optional_int_string_value(initial, 7, "item")
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && exact_nested_optional_value_generator_action(parameter, body)
            && exact_nested_result_product_value(result, 8, "done")
            && value_type == &nested_result_product_type()
    ));
    assert!(matches!(
        &consume.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Local(name),
            value_type,
            ..
        } if name == "result" && value_type == &nested_result_product_type()
    ));
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind: CompilerExpressionKind::Call { arguments, .. },
                    value_type,
                    ..
                },
            ..
        })] if name == "generated"
            && value_type == &nested_boundary_generator_type()
            && matches!(arguments.as_slice(), [argument]
                if exact_optional_int_string_value(argument, 7, "item"))
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Call { arguments, .. },
            value_type,
            ..
        } if value_type == &nested_result_product_type()
            && matches!(arguments.as_slice(), [CompilerExpression {
                kind: CompilerExpressionKind::Local(name),
                value_type,
                ..
            }] if name.starts_with("topal.root.")
                && value_type == &nested_boundary_generator_type())
    ));
}

#[test]
fn rejects_invalid_custom_generator_nested_function_boundaries() {
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-TYPE-RESULT-001,
    // TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-nested-function-boundaries.t"
    );
    for invalid in [
        source.replacen("make (Some (7, \"item\"))", "make (Some (8, \"item\"))", 1),
        source.replacen("pairs initial", "pairs (Some (8, \"item\"))", 1),
        source.replacen("(8, \"done\")", "(9, \"done\")", 1),
        source.replacen(
            "value = (Some (7, \"item\"))",
            "value = (Some (8, \"item\"))",
            1,
        ),
        source.replacen(
            "consume generated",
            "consume (pairs (Some (7, \"item\")))",
            1,
        ),
    ] {
        assert_eq!(
            analyze_for_compiler(&invalid).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Factory, traversal, and List provenance form one boundary scenario.
fn models_custom_generator_list_values() {
    // TOPAL-GENERATOR-DECLARATION-001,
    // TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-GENERATOR-FOREACH-RESULT-001,
    // TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-LIST-APPEND-001,
    // TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-COMPILER-GENERATOR-LIST-FUNCTION-BOUNDARY-001
    let source = include_str!("../../../../../examples/language/custom-generator-list-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let make = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("List Generator factory is specialized");
    assert!(matches!(
        make.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type,
            ..
        }] if name == "initial" && value_type == &int_list_type()
    ));
    assert_eq!(make.result_type, list_boundary_generator_type());
    assert!(make.body.statements.is_empty());
    assert!(matches!(
        &make.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueGenerator {
                declaration,
                initial_parameter,
                initial,
                additional_initial_parameters,
                prefix,
                yields,
                continuations,
                explicit_return: None,
                result,
                ..
            },
            value_type,
            ..
        } if declaration == "relay"
            && initial_parameter.name == "initial"
            && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
            && additional_initial_parameters.is_empty()
            && prefix.statements.is_empty()
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_int_list_append(result, "initial", 9)
            && value_type == &list_boundary_generator_type()
    ));

    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("List Generator consumer is specialized");
    assert!(matches!(
        consume.parameters.as_slice(),
        [CompilerParameter { name, value_type, .. }]
            if name == "generated" && value_type == &list_boundary_generator_type()
    ));
    assert!(matches!(
        consume.body.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomValueForeach {
                            source,
                            transferred_initial: Some(initial),
                            yields,
                            parameter,
                            body,
                            result,
                            ..
                        },
                    value_type,
                    ..
                },
            ..
        })] if name == "result"
            && matches!(source.kind, CompilerExpressionKind::Local(ref name)
                if name == "generated")
            && exact_singleton_int_list(initial, 7)
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "values"
            && exact_list_value_generator_action(parameter, body)
            && exact_int_list_append(result, "initial", 9)
            && value_type == &int_list_type()
    ));
    assert!(matches!(
        &consume.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Local(name),
            value_type,
            ..
        } if name == "result" && value_type == &int_list_type()
    ));
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value:
                CompilerExpression {
                    kind: CompilerExpressionKind::Call { arguments, .. },
                    value_type,
                    ..
                },
            ..
        })] if name == "generated"
            && value_type == &list_boundary_generator_type()
            && matches!(arguments.as_slice(), [argument]
                if exact_singleton_int_list(argument, 7))
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Call { arguments, .. },
            value_type,
            ..
        } if value_type == &int_list_type()
            && matches!(arguments.as_slice(), [CompilerExpression {
                kind: CompilerExpressionKind::Local(name),
                value_type,
                ..
            }] if name.starts_with("topal.root.")
                && value_type == &list_boundary_generator_type())
    ));
}

#[test]
fn rejects_invalid_custom_generator_list_values() {
    // TOPAL-GENERATOR-DECLARATION-001,
    // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
    // TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-LIST-APPEND-001,
    // TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-COMPILER-GENERATOR-LIST-FUNCTION-BOUNDARY-001
    let source = include_str!("../../../../../examples/language/custom-generator-list-values.t");
    for invalid in [
        source.replacen("make (one 7)", "make (one 8)", 1),
        source.replacen("relay initial", "relay (one 8)", 1),
        source.replacen("initial append 9", "initial append 8", 1),
        source.replacen("entry-count values", "empty? values", 1),
        source.replacen("consume generated", "consume (relay (one 7))", 1),
    ] {
        assert_eq!(
            analyze_for_compiler(&invalid).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_direct_task_identity_state_and_messages() {
    // TOPAL-TASK-DEFINITION-001, TOPAL-TASK-LIFECYCLE-001,
    // TOPAL-TASK-STATE-001, TOPAL-TASK-MESSAGE-001,
    // TOPAL-COMPILER-TASK-DIRECT-001
    let source = include_str!("../../../../../examples/language/task-declaration-order.t");
    let program = analyze_for_compiler(source).unwrap();
    let [task] = program.tasks.as_slice() else {
        panic!("one direct task definition is retained")
    };
    assert_eq!(task.classifier, "OrderedCounter");
    assert_eq!(task.definition, "ordered-counter-service");
    assert_eq!(task.identity, "ordered-counter");
    assert_eq!(task.queue_size, Some(4));
    assert_eq!(task.state_name, "count");
    assert_eq!(task.state_type.as_ref(), &CompilerType::Nat);
    assert!(task.handlers.iter().any(|handler| {
        handler.name == "increment" && handler.kind == CompilerTaskHandlerKind::Event
    }));
    assert!(task.handlers.iter().any(|handler| {
        handler.name == "current" && handler.kind == CompilerTaskHandlerKind::Request
    }));
    assert!(matches!(
        program.main.statements.as_slice(),
        [
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::TaskConstruct { .. },
                    value_type: CompilerType::Task(_),
                    ..
                },
                ..
            }),
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::TaskStateReplace { value, .. },
                value_type: CompilerType::Unit,
                ..
            })
        ] if matches!(value.kind, CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add,
            ..
        })
    ));
    assert!(matches!(
        program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::TaskStateLoad { .. },
            value_type: CompilerType::TaskResponse(success),
            ..
        } if success.as_ref() == &CompilerType::Nat
    ));
    let CompilerStatement::Discard(CompilerExpression {
        kind: CompilerExpressionKind::TaskStateReplace { message, .. },
        ..
    }) = &program.main.statements[1]
    else {
        unreachable!("checked event transaction is retained")
    };
    assert_eq!(message.operation, "increment");
    assert_eq!(message.transaction_identity, 1);
    let CompilerExpressionKind::TaskStateLoad { message, .. } = &program.main.result.kind else {
        unreachable!("checked request transaction is retained")
    };
    assert_eq!(message.operation, "current");
    assert_eq!(message.transaction_identity, 2);
}

#[test]
fn rejects_task_shapes_outside_the_direct_transaction_increment() {
    // TOPAL-COMPILER-TASK-DIRECT-001
    let source = include_str!("../../../../../examples/language/task-declaration-order.t");
    for invalid in [
            source.replacen("count : Nat", "count : Int", 1),
            source.replacen("  count : Nat", "  count : Nat\n  previous : Nat", 1),
            source.replacen("@ count + amount", "@ count - amount", 1),
            source.replacen("_ : MessageContext", "context : MessageContext", 1),
            source.replacen(
                "terminate is fn (_ : String) -> Unit\n    ()",
                "terminate is fn (_ : MessageContext, amount : Nat) -> Unit\n    @ count is @ count + amount",
                1,
            ),
            source.replacen(
                "ordered-counter current ()",
                "ordered-counter terminate \"done\"",
                1,
            ),
        ] {
            assert_eq!(
                analyze_for_compiler(&invalid).unwrap_err().code,
                "E-COMPILER-UNSUPPORTED"
            );
        }
}

#[test]
fn models_one_yield_task_stream_transaction() {
    // TOPAL-TASK-HANDLER-001, TOPAL-TASK-MESSAGE-001,
    // TOPAL-COMPILER-TASK-STREAM-001
    let source = include_str!("../../../../../examples/language/task-message-transactions.t");
    let program = analyze_for_compiler(source).unwrap();
    let [task] = program.tasks.as_slice() else {
        panic!("one stream-capable task definition is retained")
    };
    let stream = task
        .handlers
        .iter()
        .find(|handler| handler.name == "values")
        .expect("task metadata retains the stream handler");
    assert_eq!(stream.kind, CompilerTaskHandlerKind::Stream);
    assert_eq!(
        stream.stream_type,
        Some(CompilerGeneratorType {
            yield_type: Box::new(CompilerType::Nat),
            resume_type: Box::new(CompilerType::Unit),
            result_type: Box::new(CompilerType::TaskResponse(Box::new(CompilerType::Unit))),
        })
    );
    let [
        CompilerStatement::Binding(_),
        CompilerStatement::Discard(_),
        CompilerStatement::Binding(CompilerBinding {
            value:
                CompilerExpression {
                    kind: CompilerExpressionKind::CustomValueGenerator { .. },
                    ..
                },
            ..
        }),
        CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach { yields, result, .. },
            value_type: CompilerType::TaskResponse(success),
            ..
        }),
    ] = program.main.statements.as_slice()
    else {
        panic!("task stream construction and traversal are retained")
    };
    let [CompilerGeneratorYield::Value(yielded)] = yields.as_slice() else {
        panic!("task stream retains one state yield")
    };
    let CompilerExpressionKind::TaskStateLoad { message, .. } = &yielded.kind else {
        panic!("task stream yield reacquires private state")
    };
    assert_eq!(message.operation, "values");
    assert_eq!(message.transaction_identity, 2);
    assert_eq!(success.as_ref(), &CompilerType::Unit);
    assert!(matches!(
        result.kind,
        CompilerExpressionKind::ResultSuccess(_)
    ));
    let CompilerExpressionKind::TaskStateLoad { message, .. } = &program.main.result.kind else {
        panic!("request follows the completed stream transaction")
    };
    assert_eq!(message.operation, "current");
    assert_eq!(message.transaction_identity, 3);
}

#[test]
fn rejects_task_streams_outside_the_one_yield_increment() {
    // TOPAL-COMPILER-TASK-STREAM-001
    let source = include_str!("../../../../../examples/language/task-message-transactions.t");
    for (invalid, expected) in [
        (
            source.replacen("yields Nat", "yields Int", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen("resumes Unit", "resumes Nat", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen("yield @ count", "yield 1", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source
                .replacen("start is fn", "initialize is fn", 1)
                .replacen("values is generator", "start is generator", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source
                .replacen("terminate is fn", "cleanup is fn", 1)
                .replacen("values is generator", "terminate is generator", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen(
                "stream foreach { value }\n  ()",
                "stream foreach { value }\n  _ is value + 1\n  ()",
                1,
            ),
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(analyze_for_compiler(&invalid).unwrap_err().code, expected);
    }
}

#[test]
fn models_closed_external_layout_location_access() {
    // TOPAL-LAYOUT-SIZE-001, TOPAL-LAYOUT-CONSTRUCT-001,
    // TOPAL-ADDRESS-RANGE-001, TOPAL-LOCATION-CONSTRUCT-001,
    // TOPAL-LOCATION-READ-001, TOPAL-LOCATION-WRITE-001,
    // TOPAL-COMPILER-EXTERNAL-LOCATION-001
    let source = include_str!("../../../../../examples/language/external-layout-location.t");
    let program = analyze_for_compiler(source).unwrap();
    let layout =
        program
            .main
            .statements
            .iter()
            .find_map(|statement| match statement {
                CompilerStatement::Binding(CompilerBinding {
                    name,
                    value:
                        CompilerExpression {
                            kind:
                                CompilerExpressionKind::ExternalMetadata(
                                    CompilerExternalMetadata::Layout(layout),
                                ),
                            ..
                        },
                    ..
                }) if name == "UInt32LE" => Some(layout),
                _ => None,
            })
            .expect("UInt32LE metadata is retained");
    assert_eq!(layout.storage_size_bits, 32);
    assert_eq!(layout.encoding.as_deref(), Some("UnsignedBinary"));
    assert_eq!(layout.endian.as_deref(), Some("Little"));
    assert_eq!(layout.access, "ReadWrite");
    assert_eq!(layout.alignment_bytes, 4);
    assert!(matches!(
        layout.family,
        CompilerExternalLayoutFamily::UnsignedNat
    ));

    let location = program
        .main
        .statements
        .iter()
        .find_map(|statement| match statement {
            CompilerStatement::Binding(CompilerBinding {
                name,
                value:
                    CompilerExpression {
                        kind: CompilerExpressionKind::ExternalLocationConstruct(location),
                        ..
                    },
                ..
            }) if name == "control" => Some(location),
            _ => None,
        })
        .expect("checked location is retained");
    assert_eq!(location.location_type.identity, "ControlLocation");
    assert_eq!(location.offset.offset_type.range.identity, "device");
    assert_eq!(location.offset.offset, BigInt::from(32));
    assert_eq!(
        location.offset.offset_type.range.lower,
        BigInt::from(0x4000_0000_u64)
    );
    assert!(matches!(
        program.main.statements.last(),
        Some(CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::ExternalLocationWrite { .. },
            ..
        }))
    ));
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::ExternalLocationRead { .. }
    ));
    assert_eq!(program.main.result.value_type.name(), "UInt32LE");
}

#[test]
fn rejects_external_locations_outside_the_closed_increment() {
    // TOPAL-COMPILER-EXTERNAL-LOCATION-001
    let source = include_str!("../../../../../examples/language/external-layout-location.t");
    for (invalid, expected) in [
        (
            source.replacen("storage-size is 32[b]", "storage-size is 16[b]", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen("alignment is 4", "alignment is 3", 1),
            "E-ADDRESS-OFFSET-ALIGNMENT",
        ),
        (
            source.replacen("DeviceOffset 32", "DeviceOffset 65536", 1),
            "E-ADDRESS-OFFSET-RANGE",
        ),
        (
            source.replacen("access is ReadWrite", "access is ReadOnly", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen("UInt32LE 42", "UInt32LE 0x100000000", 1),
            "E-LAYOUT-NOT-REPRESENTABLE",
        ),
        (
            source.replacen(
                "tags is (none is 0, some is 1)",
                "tags is (none is 18446744073709551616, some is 1)",
                1,
            ),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen("first is UInt32LE", "left is UInt32LE", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen("0x40000000 ..=", "0x40000001 ..=", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen("control write stored\nread control", "control", 1),
            "E-COMPILER-UNSUPPORTED",
        ),
        (
            source.replacen(
                "control write stored\nread control",
                "copy is control\nread copy",
                1,
            ),
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(analyze_for_compiler(&invalid).unwrap_err().code, expected);
    }
}
