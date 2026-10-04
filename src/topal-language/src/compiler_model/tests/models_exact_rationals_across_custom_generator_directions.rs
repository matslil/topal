#[test]
#[allow(clippy::too_many_lines)] // Accepted graph and exact rejection matrix stay together.
fn models_exact_rationals_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-RATIONAL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-rational-values.t"
    ))
    .unwrap();
    let one_third = BigRational::new(BigInt::from(1), BigInt::from(3));
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    declaration,
                    initial_parameter,
                    initial,
                    yields,
                    continuations,
                    explicit_return: None,
                    result,
                    ..
                },
                value_type: CompilerType::Generator(generator_type),
                ..
            },
            ..
        })] if name == "generated"
            && declaration == "next"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::Rational
            && matches!(initial.kind, CompilerExpressionKind::RationalConstruct { .. })
            && initial.rational_value.as_ref() == Some(&one_third)
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(
                result.kind,
                CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    ref left,
                    ref right,
                } if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                    if name == "initial")
                    && matches!(right.kind, CompilerExpressionKind::RationalConstruct { .. })
                    && right.rational_value.as_ref() == Some(&one_third)
            )
            && *generator_type.yield_type == CompilerType::Rational
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::Rational
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach {
                yields,
                parameter,
                body,
                result,
                ..
            },
            value_type: CompilerType::Rational,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && parameter.value_type == CompilerType::Rational
            && matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Add,
                        left,
                        right,
                    },
                    ..
                })] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                    if name == "value")
                    && matches!(right.kind, CompilerExpressionKind::RationalConstruct { .. })
                    && right.rational_value.as_ref() == Some(&one_third)
            )
            && matches!(
                result.kind,
                CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    ref left,
                    ref right,
                } if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                    if name == "initial")
                    && matches!(right.kind, CompilerExpressionKind::RationalConstruct { .. })
                    && right.rational_value.as_ref() == Some(&one_third)
            )
    ));

    for source in [
        "use language (version is v0.1)\nnext is generator (initial : Rational)\n  yields Rational\n  resumes Unit\n  -> Rational\n  _ is yield (Rational (1, 3))\n  initial + (Rational (1, 3))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n",
        "use language (version is v0.1)\nnext is generator (initial : Rational)\n  yields Rational\n  resumes Unit\n  -> Rational\n  _ is yield initial\n  _ is yield initial\n  initial + (Rational (1, 3))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n",
        "use language (version is v0.1)\nnext is generator (initial : Rational)\n  yields Rational\n  resumes Unit\n  -> Rational\n  _ is yield initial\n  initial\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n",
        "use language (version is v0.1)\nnext is generator (initial : Rational)\n  yields Rational\n  resumes Unit\n  -> Rational\n  _ is yield initial\n  initial + (Rational (1, 2))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n",
        "use language (version is v0.1)\nnext is generator (initial : Rational)\n  yields Rational\n  resumes Unit\n  -> Rational\n  _ is yield initial\n  initial + (Rational (1, 3))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 2))\n",
        "use language (version is v0.1)\nnext is generator (initial : Rational)\n  yields Rational\n  resumes Unit\n  -> Rational\n  _ is yield initial\n  initial + (Rational (1, 3))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (2, 6))\n",
        "use language (version is v0.1)\nnext is generator (initial : Rational)\n  yields Rational\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_unit_across_every_custom_generator_direction() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-UNIT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-unit-values.t"
    ))
    .unwrap();
    assert!(matches!(
        program.main.statements.as_slice(),
        [
            CompilerStatement::Binding(CompilerBinding {
                name,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::CustomValueGenerator {
                        declaration,
                        initial_parameter,
                        initial,
                        yields,
                        continuations,
                        explicit_return: None,
                        result,
                        ..
                    },
                    value_type: CompilerType::Generator(generator_type),
                    ..
                },
                ..
            }),
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomValueForeach {
                    initial_parameter: traversal_initial_parameter,
                    yields: traversal_yields,
                    parameter,
                    body,
                    result: traversal_result,
                    ..
                },
                value_type: CompilerType::Unit,
                ..
            })
        ] if name == "generated"
            && declaration == "pulse"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::Unit
            && matches!(initial.kind, CompilerExpressionKind::Unit)
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(result.kind, CompilerExpressionKind::Unit)
            && *generator_type.yield_type == CompilerType::Unit
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::Unit
            && traversal_initial_parameter.name == "initial"
            && traversal_initial_parameter.value_type == CompilerType::Unit
            && matches!(traversal_yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "signal"
            && parameter.value_type == CompilerType::Unit
            && body.statements.is_empty()
            && matches!(body.result.kind, CompilerExpressionKind::Local(ref name)
                if name == "signal")
            && matches!(traversal_result.kind, CompilerExpressionKind::Unit)
    ));
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Unit
    ));

    for source in [
        "use language (version is v0.1)\npulse is generator (initial : Unit)\n  yields Unit\n  resumes Unit\n  -> Unit\n  _ is yield ()\n  ()\ngenerated is pulse ()\ngenerated foreach { signal }\n  signal\n",
        "use language (version is v0.1)\npulse is generator (initial : Unit)\n  yields Unit\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is yield initial\n  ()\ngenerated is pulse ()\ngenerated foreach { signal }\n  signal\n",
        "use language (version is v0.1)\npulse is generator (initial : Unit)\n  yields Unit\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  initial\ngenerated is pulse ()\ngenerated foreach { signal }\n  signal\n",
        "use language (version is v0.1)\npulse is generator (initial : Unit)\n  yields Unit\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is pulse ()\ngenerated foreach { signal }\n  ()\n",
        "use language (version is v0.1)\npulse is generator (initial : Unit)\n  yields Unit\n  resumes Unit\n  -> Boolean\n  _ is yield initial\n  false\ngenerated is pulse ()\ngenerated foreach { signal }\n  signal\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Accepted graph and exact rejection matrix stay together.
fn models_optional_int_values_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-COMPILER-GENERATOR-OPTIONAL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-optional-values.t"
    ))
    .unwrap();
    let optional_int = CompilerType::Optional(Box::new(CompilerType::Int));
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    declaration,
                    initial_parameter,
                    initial,
                    yields,
                    continuations,
                    explicit_return: None,
                    result,
                    ..
                },
                value_type: CompilerType::Generator(generator_type),
                ..
            },
            ..
        })] if name == "generated"
            && declaration == "optional"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == optional_int
            && matches!(initial.kind, CompilerExpressionKind::OptionalSome(ref payload)
                if matches!(payload.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(7)))
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(result.kind, CompilerExpressionKind::OptionalNone)
            && result.value_type == optional_int
            && *generator_type.yield_type == optional_int
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == optional_int
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach {
                yields,
                parameter,
                body,
                result,
                ..
            },
            value_type,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "candidate"
            && parameter.value_type == optional_int
            && matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                })] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                    if name == "candidate")
                    && matches!(right.kind, CompilerExpressionKind::OptionalSome(ref payload)
                        if matches!(payload.kind, CompilerExpressionKind::Int(ref value)
                            if value == &BigInt::from(7)))
            )
            && matches!(body.result.kind, CompilerExpressionKind::Unit)
            && matches!(result.kind, CompilerExpressionKind::OptionalNone)
            && result.value_type == optional_int
            && value_type == &optional_int
    ));

    let absent_input = analyze_for_compiler(
            "use language (version is v0.1)\noptional is generator (initial : Optional Int)\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n  _ is yield initial\n  None Int\ngenerated is optional (None Int)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n",
        )
        .unwrap();
    assert!(matches!(
        &absent_input.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator { initial, .. },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::OptionalNone)
            && initial.value_type == optional_int
    ));

    for source in [
        "use language (version is v0.1)\noptional is generator (initial : Optional Int)\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n  _ is yield (Some 7)\n  None Int\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n",
        "use language (version is v0.1)\noptional is generator (initial : Optional Int)\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n  _ is yield initial\n  _ is yield initial\n  None Int\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n",
        "use language (version is v0.1)\noptional is generator (initial : Optional Int)\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n  _ is yield initial\n  initial\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n",
        "use language (version is v0.1)\noptional is generator (initial : Optional Int)\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n  _ is yield initial\n  Some 8\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n",
        "use language (version is v0.1)\noptional is generator (initial : Optional Int)\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n  _ is yield initial\n  None Int\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 8)\n",
        "use language (version is v0.1)\noptional is generator (initial : Optional Int)\n  yields Optional Int\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Accepted graph and exact rejection matrix stay together.
fn models_int_ranges_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-RANGE-BOUNDS-001, TOPAL-RANGE-CLASSIFIER-001,
    // TOPAL-COMPILER-GENERATOR-RANGE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-range-values.t"
    ))
    .unwrap();
    let range_int = CompilerType::Range(Box::new(CompilerType::Int));
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    declaration,
                    initial_parameter,
                    initial,
                    yields,
                    continuations,
                    explicit_return: None,
                    result,
                    ..
                },
                value_type: CompilerType::Generator(generator_type),
                ..
            },
            ..
        })] if name == "generated"
            && declaration == "narrow"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == range_int
            && matches!(initial.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::RangeInclusive,
                ref left,
                ref right,
            } if matches!(left.kind, CompilerExpressionKind::Int(ref value)
                if value == &BigInt::from(0))
                && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(10)))
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(result.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::And,
                ref left,
                ref right,
            } if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
                && matches!(right.kind, CompilerExpressionKind::Binary {
                    operation: CompilerBinary::RangeInclusive,
                    ..
                }))
            && result.value_type == range_int
            && *generator_type.yield_type == range_int
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == range_int
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach {
                yields,
                parameter,
                body,
                result,
                ..
            },
            value_type,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "interval"
            && parameter.value_type == range_int
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::In,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Int(ref value)
                if value == &BigInt::from(5))
                && matches!(right.kind, CompilerExpressionKind::Local(ref name)
                    if name == "interval"))
            && matches!(body.result.kind, CompilerExpressionKind::Unit)
            && matches!(result.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::And,
                ..
            })
            && result.value_type == range_int
            && value_type == &range_int
    ));

    let expression_input = analyze_for_compiler(
            "use language (version is v0.1)\nnarrow is generator (initial : Range Int)\n  yields Range Int\n  resumes Unit\n  -> Range Int\n  _ is yield initial\n  initial and (5 ..= 15)\ngenerated is narrow ((0 ..= 12) and (2 ..= 10))\ngenerated foreach { interval }\n  _ is 5 in interval\n",
        )
        .unwrap();
    assert!(matches!(
        &expression_input.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator { initial, .. },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::Binary {
            operation: CompilerBinary::And,
            ..
        })
    ));

    for source in [
        "use language (version is v0.1)\nnarrow is generator (initial : Range Int)\n  yields Range Int\n  resumes Unit\n  -> Range Int\n  _ is yield (0 ..= 10)\n  initial and (5 ..= 15)\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n",
        "use language (version is v0.1)\nnarrow is generator (initial : Range Int)\n  yields Range Int\n  resumes Unit\n  -> Range Int\n  _ is yield initial\n  _ is yield initial\n  initial and (5 ..= 15)\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n",
        "use language (version is v0.1)\nnarrow is generator (initial : Range Int)\n  yields Range Int\n  resumes Unit\n  -> Range Int\n  _ is yield initial\n  initial\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n",
        "use language (version is v0.1)\nnarrow is generator (initial : Range Int)\n  yields Range Int\n  resumes Unit\n  -> Range Int\n  _ is yield initial\n  initial and (6 ..= 15)\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n",
        "use language (version is v0.1)\nnarrow is generator (initial : Range Int)\n  yields Range Int\n  resumes Unit\n  -> Range Int\n  _ is yield initial\n  initial and (5 ..= 15)\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 6 in interval\n",
        "use language (version is v0.1)\nnarrow is generator (initial : Range Int)\n  yields Range Int\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_owned_close_of_transferred_string_character_generator() {
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-STRING-CHARACTERS-CLOSE-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/string-character-generator-close.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "ignore")
        .expect("called close function is instantiated");
    assert!(matches!(
        function.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type,
            ..
        }] if name == "generated" && is_character_unit_generator_type(value_type)
    ));
    assert!(matches!(
        function.body.statements.as_slice(),
        [CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersClose(generator),
            value_type: CompilerType::Unit,
            ..
        })] if matches!(generator.kind, CompilerExpressionKind::Local(ref name) if name == "generated")
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Call { arguments, .. },
            value_type: CompilerType::Unit,
            ..
        } if matches!(arguments.as_slice(), [CompilerExpression {
            kind: CompilerExpressionKind::Local(_),
            value_type,
            ..
        }] if is_character_unit_generator_type(value_type))
    ));

    for source in [
        "use language (version is v0.1)\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  _ is 1\ngenerated is characters \"a\"\nignore generated\n",
        "use language (version is v0.1)\nignore is fn (left : Generator Character Unit Unit, right : Generator Character Unit Unit) -> Unit\n  ()\nleft is characters \"a\"\nright is characters \"b\"\nignore (left, right)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_specialized_string_character_generator_parameter_traversal() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/string-character-generator-parameter.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("called traversal function is instantiated");
    assert!(matches!(
        function.parameters.as_slice(),
        [CompilerParameter { value_type, .. }]
            if is_character_unit_generator_type(value_type)
    ));
    assert!(matches!(
        function.body.statements.as_slice(),
        [CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersForeach {
                source,
                characters,
                parameter,
                ..
            },
            ..
        })] if matches!(source.kind, CompilerExpressionKind::Local(ref name) if name == "generated")
            && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
            && parameter.value_type == CompilerType::Character
    ));
    assert!(!function.body.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersClose(_),
            ..
        })
    )));

    let distinct = analyze_for_compiler(
            "use language (version is v0.1)\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\nfirst is characters \"a\"\n_ is consume first\nsecond is characters \"🇸🇪\"\nconsume second\n",
        )
        .unwrap();
    let traversals = distinct
        .functions
        .iter()
        .filter(|function| function.source_name == "consume")
        .map(|function| match &function.body.statements[0] {
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersForeach { characters, .. },
                ..
            }) => characters.clone(),
            statement => panic!("expected specialized traversal, found {statement:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        traversals,
        [vec![String::from("a")], vec![String::from("🇸🇪")]]
    );

    let extra_statement = "use language (version is v0.1)\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  _ is 1\n  generated foreach { character }\n    _ is String character\ngenerated is characters \"a\"\nconsume generated\n";
    assert_eq!(
        analyze_for_compiler(extra_statement).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
    let direct_function_traversal = "use language (version is v0.1)\nconsume is fn () -> Unit\n  characters \"a\" foreach { character }\n    _ is String character\nconsume ()\n";
    assert_eq!(
        analyze_for_compiler(direct_function_traversal)
            .unwrap_err()
            .code,
        "E-COMPILER-UNSUPPORTED"
    );
    let nested_transfer = "use language (version is v0.1)\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\nouter is fn () -> Unit\n  generated is characters \"a\"\n  consume generated\nouter ()\n";
    let nested_transfer = analyze_for_compiler(nested_transfer).unwrap_err();
    assert_eq!(nested_transfer.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        nested_transfer
            .message
            .contains("nested Character Generator parameter transfer")
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Generator result admission and its rejected boundaries stay together.
fn models_specialized_string_character_generator_result_transfer() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-RESULT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/string-character-generator-result.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "generate")
        .expect("called generator factory is instantiated");
    assert!(matches!(
        function.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type: CompilerType::String,
            ..
        }] if name == "text"
    ));
    assert!(is_character_unit_generator_type(&function.result_type));
    assert!(function.body.statements.is_empty());
    assert!(matches!(
        &function.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersGenerator {
                text,
                characters,
            },
            ..
        } if matches!(text.kind, CompilerExpressionKind::Local(ref name) if name == "text")
            && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
    ));
    assert!(matches!(
        program.main.statements.as_slice(),
        [
            CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Call { .. },
                    value_type,
                    ..
                },
                ..
            }),
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersForeach { characters, .. },
                ..
            })
        ] if is_character_unit_generator_type(value_type)
            && characters == &["a\u{301}", "👩‍🔬", "🇸🇪"]
    ));

    let distinct = analyze_for_compiler(
            "use language (version is v0.1)\ngenerate is fn (text : String) -> Generator Character Unit Unit\n  characters text\nfirst is generate \"a\"\nfirst foreach { character }\n  _ is String character\nsecond is generate \"🇸🇪\"\nsecond foreach { character }\n  _ is String character\n",
        )
        .unwrap();
    let traversals = distinct
        .main
        .statements
        .iter()
        .filter_map(|statement| match statement {
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersForeach { characters, .. },
                ..
            }) => Some(characters.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        traversals,
        [vec![String::from("a")], vec![String::from("🇸🇪")]]
    );
    let factory_symbols = distinct
        .functions
        .iter()
        .filter(|function| function.source_name == "generate")
        .map(|function| function.symbol.as_str())
        .collect::<Vec<_>>();
    assert_eq!(factory_symbols.len(), 2);
    assert_ne!(factory_symbols[0], factory_symbols[1]);

    for source in [
        "use language (version is v0.1)\ngenerate is fn (text : String) -> Generator Character Unit Unit\n  _ is 1\n  characters text\ngenerated is generate \"a\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\ngenerate is fn (text : String, ignored : Int) -> Generator Character Unit Unit\n  characters text\ngenerated is generate (\"a\", 0)\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\ngenerate is fn (text : String) -> Generator Character Unit Unit\n  characters text\nouter is fn (text : String) -> Generator Character Unit Unit\n  generate text\ngenerated is outer \"a\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
    let returned_text = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ngenerate is fn (text : String) -> Generator Character Unit Unit\n  characters text\ngenerated is generate (identity \"a\")\ngenerated foreach { character }\n  _ is String character\n",
        )
        .unwrap();
    assert!(
        returned_text
            .main
            .statements
            .iter()
            .any(|statement| matches!(
                statement,
                CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::StringCharactersForeach { characters, .. },
                    ..
                }) if characters == &[String::from("a")]
            ))
    );
    let unbound_result = analyze_for_compiler(
            "use language (version is v0.1)\ngenerate is fn (text : String) -> Generator Character Unit Unit\n  characters text\ngenerate \"a\"\n",
        )
        .unwrap_err();
    assert_eq!(unbound_result.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        unbound_result
            .message
            .contains("unbound returned Generator and close delivery")
    );
}

#[test]
fn models_lazy_list_int_unfold_with_distinct_seed_and_yield_types() {
    // TOPAL-GENERATOR-UNFOLD-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/unfold-generator.t"
    ))
    .unwrap();
    assert_eq!(program.main.result.value_type, int_unit_generator_type());
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::UnfoldGenerator {
                    seed,
                    parameters,
                    step,
                },
                ..
            },
            ..
        }) if name == "generated"
            && matches!(seed.kind, CompilerExpressionKind::Local(_))
            && parameters.len() == 1
            && matches!(step.result.kind, CompilerExpressionKind::ListUncons(_))
    )));

    for (invalid, code) in [
        (
            "use language (version is v0.1)\n0 unfold ({ value } value)\n",
            "E-TYPE-MISMATCH",
        ),
        (
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\nvalues unfold ({ remaining } remaining)\n",
            "E-TYPE-MISMATCH",
        ),
        (
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\ngenerated is values unfold ({ remaining } uncons remaining)\n(generated, generated)\n",
            "E-GENERATOR-CONSUMED",
        ),
        (
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\ngenerated is values unfold ({ remaining } uncons remaining)\n()\n",
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(analyze_for_compiler(invalid).unwrap_err().code, code);
    }
}

#[test]
fn models_direct_finite_list_uncons_unfold_collection() {
    // TOPAL-GENERATOR-UNFOLD-001, TOPAL-GENERATOR-UNFOLD-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/unfold-collect.t"
    ))
    .unwrap();
    assert_eq!(
        program.main.result.value_type,
        CompilerType::List(Box::new(CompilerType::Int))
    );
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::GeneratorCollect(ref generator)
            if matches!(generator.kind, CompilerExpressionKind::UnfoldGenerator { .. })
    ));

    let moved = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Entry (2, Empty))\ngenerated is values unfold ({ remaining } uncons remaining)\nmoved is generated\ncollect moved\n",
        )
        .unwrap();
    assert_eq!(
        moved.main.result.value_type,
        CompilerType::List(Box::new(CompilerType::Int))
    );

    let non_finite = analyze_for_compiler(
            "use language (version is v0.1)\nvalues : List Int is Entry (1, Empty)\ncollect (values unfold ({ remaining } Some (1, remaining)))\n",
        )
        .unwrap_err();
    assert_eq!(non_finite.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_lexical_blocks_with_fresh_shadowing_scope() {
    // TOPAL-SYN-GRAMMAR-001, TOPAL-EXEC-BLOCK-001
    let source = "use language (version is v0.1)\nColor is Enum (Red, Green)\nidentity is fn (number : Int) -> Int\n  number\nempty is {}\nvalue is 40\nshadow is {\n  Red is value + 1\n  identity is Red + 1\n  (Red, identity)\n}\n(empty, shadow, Red, identity 43, value)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Unit, (Int, Int), Color, Int, Int)"
    );
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected a tuple")
    };
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::Block(_),
                ..
            },
            ..
        }) if name == "shadow"
    )));
    assert!(matches!(values[2].kind, CompilerExpressionKind::Enum(0)));
    assert!(matches!(
        values[3].kind,
        CompilerExpressionKind::Call { .. }
    ));
    assert_eq!(exact_int(&values[4]), Some(BigInt::from(40)));

    let closed_zero = "use language (version is v0.1)\n1 / {\n  zero is 0\n  zero\n}\n";
    assert_eq!(
        analyze_for_compiler(closed_zero).unwrap_err().code,
        "E-DIVISION-BY-ZERO"
    );
}

#[test]
fn models_completed_as_distinct_zero_data_evidence() {
    // TOPAL-EXEC-COMPLETED-001
    let source = "use language (version is v0.1)\nfinish is fn () -> Completed\n  Completed\nretain is fn (value : Completed) -> Completed\n  value\n(finish (), retain Completed, Completed = Completed)\n";
    let program = analyze_for_compiler(source).unwrap();
    assert_eq!(
        program.main.result.value_type.name(),
        "(Completed, Completed, Boolean)"
    );
    assert!(program.functions.iter().all(|function| {
        function.result_type == CompilerType::Completed
            && function.body.result.value_type == CompilerType::Completed
    }));
    assert_ne!(CompilerType::Completed, CompilerType::Unit);
}

#[test]
fn models_empty_effect_as_a_distinct_inert_scalar() {
    // TOPAL-EFFECT-EMPTY-001, TOPAL-EFFECT-CLASSIFIER-001,
    // TOPAL-EFFECT-BOUNDARY-001, TOPAL-EFFECT-PRODUCT-001
    let boundary = analyze_for_compiler(include_str!(
        "../../../../../examples/language/effect-function-boundary.t"
    ))
    .unwrap();
    assert_eq!(boundary.functions.len(), 1);
    let function = &boundary.functions[0];
    assert_eq!(function.parameters[0].value_type, CompilerType::Effect);
    assert_eq!(function.result_type, CompilerType::Effect);
    assert_eq!(function.body.result.value_type, CompilerType::Effect);
    assert!(matches!(
        boundary.main.result.kind,
        CompilerExpressionKind::Call { .. }
    ));

    let pair = analyze_for_compiler(include_str!(
        "../../../../../examples/language/effect-products.t"
    ))
    .unwrap();
    assert_eq!(
        pair.main.result.value_type,
        CompilerType::Tuple(vec![CompilerType::Effect, CompilerType::Effect])
    );
    let unit = analyze_for_compiler(include_str!(
        "../../../../../examples/language/unit-effect-value.t"
    ))
    .unwrap();
    assert_eq!(
        unit.main.result.value_type,
        CompilerType::Tuple(vec![CompilerType::Unit, CompilerType::Effect])
    );
    assert_ne!(CompilerType::Effect, CompilerType::Unit);
    assert_ne!(CompilerType::Effect, CompilerType::Completed);

    for source in [
        include_str!("../../../../../examples/language/empty-effects.t"),
        include_str!("../../../../../examples/language/effect-classifier.t"),
        include_str!("../../../../../examples/language/effect-identity.t"),
    ] {
        assert!(analyze_for_compiler(source).is_ok());
    }
}

#[test]
fn models_contextual_effect_list_construction() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COMPILER-LIST-EFFECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/effect-list.t"
    ))
    .unwrap();
    let [CompilerStatement::Binding(rows)] = program.main.statements.as_slice() else {
        panic!("shared regression binds one List value")
    };
    assert_eq!(
        rows.value.value_type,
        CompilerType::List(Box::new(CompilerType::Effect))
    );
    let CompilerExpressionKind::ListEntry { value, remaining } = &rows.value.kind else {
        panic!("expected contextual Entry construction")
    };
    assert!(matches!(value.kind, CompilerExpressionKind::Effect));
    assert!(matches!(remaining.kind, CompilerExpressionKind::ListEmpty));
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Local(ref storage) if storage == &rows.storage_name
    ));

    for invalid in [
        "use language (version is v0.1)\nrows : List Effect is Entry (Completed, Empty)\nrows\n",
        "use language (version is v0.1)\nrows : List Effect is Entry (Effects (), Effects ())\nrows\n",
    ] {
        assert_eq!(
            analyze_for_compiler(invalid).unwrap_err().code,
            "E-TYPE-MISMATCH"
        );
    }
}

#[test]
fn models_int_list_containment_without_erasing_list_identity() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-CONTAINS-ENTRY-001,
    // TOPAL-LIST-CONTAINS-SEQUENCE-001, TOPAL-LIST-CONTAINS-SUBSEQUENCE-001,
    // TOPAL-COMPILER-LIST-INT-CONTAINMENT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-containment.t"
    ))
    .unwrap();
    for statement in &program.main.statements {
        let CompilerStatement::Binding(binding) = statement else {
            continue;
        };
        assert_eq!(
            binding.value.value_type,
            CompilerType::List(Box::new(CompilerType::Int))
        );
    }
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared containment regression returns a Tuple")
    };
    assert_eq!(results.len(), 6);
    assert!(matches!(
        results[0].kind,
        CompilerExpressionKind::ListContainsEntry { .. }
    ));
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::ListContainsSequence { .. }
    ));
    assert!(matches!(
        results[3].kind,
        CompilerExpressionKind::ListContainsSubsequence { .. }
    ));
    assert!(
        results
            .iter()
            .all(|result| result.value_type == CompilerType::Boolean)
    );

    let mismatch = "use language (version is v0.1)\nvalues : List Int is Empty\nvalues contains-entry \"no\"\n";
    assert_eq!(
        analyze_for_compiler(mismatch).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let unavailable = "use language (version is v0.1)\nvalues : List Effect is Empty\nvalues contains-entry Effects ()\n";
    assert_eq!(
        analyze_for_compiler(unavailable).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_immutable_int_list_removal() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-REMOVE-FIRST-001,
    // TOPAL-LIST-REMOVE-ALL-001, TOPAL-COMPILER-LIST-INT-REMOVAL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-removal.t"
    ))
    .unwrap();
    let [CompilerStatement::Binding(values)] = program.main.statements.as_slice() else {
        panic!("shared removal regression binds one List value")
    };
    let list_int = CompilerType::List(Box::new(CompilerType::Int));
    assert_eq!(values.value.value_type, list_int);
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared removal regression returns a Tuple")
    };
    assert_eq!(results.len(), 3);
    assert!(matches!(
        results[0].kind,
        CompilerExpressionKind::ListRemoveFirst { .. }
    ));
    assert!(
        results[1..]
            .iter()
            .all(|result| matches!(result.kind, CompilerExpressionKind::ListRemoveAll { .. }))
    );
    assert!(results.iter().all(|result| result.value_type == list_int));

    let mismatch =
        "use language (version is v0.1)\nvalues : List Int is Empty\nvalues remove-all \"no\"\n";
    assert_eq!(
        analyze_for_compiler(mismatch).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );
    let unavailable = "use language (version is v0.1)\nvalues : List Effect is Empty\nvalues remove-first Effects ()\n";
    assert_eq!(
        analyze_for_compiler(unavailable).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_complete_closed_list_sequence_operations() {
    // TOPAL-LIST-BOUNDARY-CHECK-001 through TOPAL-LIST-UNZIP-001,
    // TOPAL-COLLECTION-FOREACH-001, TOPAL-COLLECTION-ENTRIES-001,
    // TOPAL-COLLECTION-COLLECT-LIST-001, TOPAL-COLLECTION-COLLECT-STRING-001,
    // TOPAL-COMPILER-LIST-SEQUENCE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-sequence-operations.t"
    ))
    .unwrap();
    assert!(program.main.statements.iter().any(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::ListForeach { .. },
                ..
            },
            ..
        })
    )));
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared sequence regression returns a Tuple")
    };
    assert_eq!(results.len(), 16);
    assert!(matches!(
        results[0].kind,
        CompilerExpressionKind::ListInsertAt {
            inserts_list: false,
            ..
        }
    ));
    assert!(matches!(
        results[1].kind,
        CompilerExpressionKind::ListInsertAt {
            inserts_list: true,
            ..
        }
    ));
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::ListIndexOperation {
            operation: CompilerListIndexOperation::Split,
            ..
        }
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListRemoveIndexRange {
            start: 1,
            end: 3,
            ..
        }
    ));
    assert!(matches!(
        results[7].kind,
        CompilerExpressionKind::ListReject { indexes: true, .. }
    ));
    assert!(matches!(
        results[8].kind,
        CompilerExpressionKind::ListReject { indexes: false, .. }
    ));
    assert!(matches!(
        results[9].kind,
        CompilerExpressionKind::ListZip {
            operation: CompilerListZipOperation::Exact,
            ..
        }
    ));
    assert!(matches!(
        results[12].kind,
        CompilerExpressionKind::ListUnzip(_)
    ));
    assert!(matches!(
        results[13].kind,
        CompilerExpressionKind::ListEntries(_)
    ));
    assert!(matches!(
        results[15].kind,
        CompilerExpressionKind::ListCollectString(_)
    ));

    for invalid in [
        "use language (version is v0.1)\nvalues : List Int is one 1\nvalues insert-at 2 9\n",
        "use language (version is v0.1)\nvalues : List Int is one 1\nvalues take 2\n",
        "use language (version is v0.1)\nvalues : List Int is one 1\nvalues remove 1\n",
        "use language (version is v0.1)\nvalues : List Int is one 1\nvalues remove-indexes (0 ..= 1)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(invalid).unwrap_err().code,
            "E-LIST-BOUNDARY-OUT-OF-RANGE"
        );
    }
}

#[test]
fn models_closed_fundamental_containers_and_queries() {
    // TOPAL-ARRAY-COLLECT-001, TOPAL-SET-COLLECT-001,
    // TOPAL-BAG-COLLECT-001, TOPAL-MAP-COLLECT-001,
    // TOPAL-COLLECTION-ENTRY-COUNT-001, TOPAL-COLLECTION-EMPTY-PREDICATE-001,
    // TOPAL-ARRAY-GET-CHECKED-001, TOPAL-MAP-LOOKUP-001,
    // TOPAL-SET-CONTAINS-001, TOPAL-BAG-MULTIPLICITY-001,
    // TOPAL-COMPILER-FUNDAMENTAL-CONTAINERS-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/fundamental-containers.t"
    ))
    .unwrap();
    let collections = program
        .main
        .statements
        .iter()
        .filter_map(|statement| match statement {
            CompilerStatement::Binding(CompilerBinding {
                value:
                    CompilerExpression {
                        kind: CompilerExpressionKind::ContainerCollect { kind, .. },
                        ..
                    },
                ..
            }) => Some(*kind),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        collections,
        [
            CompilerContainerKind::Array,
            CompilerContainerKind::Set,
            CompilerContainerKind::Bag,
            CompilerContainerKind::Map,
        ]
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared fundamental-container regression returns a Tuple")
    };
    assert_eq!(results.len(), 16);
    assert!(matches!(
        results[9].kind,
        CompilerExpressionKind::ArrayAt { index: 1, .. }
    ));
    assert!(matches!(
        results[11].kind,
        CompilerExpressionKind::SetContains { .. }
    ));
    assert!(matches!(
        results[12].kind,
        CompilerExpressionKind::BagMultiplicity { .. }
    ));
    assert!(matches!(
        results[14].kind,
        CompilerExpressionKind::MapLookup { .. }
    ));

    let duplicate_reject = "use language (version is v0.1)\npairs : List (String, Int) is Entry ((\"Ada\", 1), Entry ((\"Ada\", 2), Empty))\ncollect-map pairs resolving reject\n";
    assert_eq!(
        analyze_for_compiler(duplicate_reject).unwrap_err().code,
        "E-MAP-KEY-COLLISION"
    );
    let unique_reject = "use language (version is v0.1)\npairs : List (String, Int) is Entry ((\"Ada\", 1), Entry ((\"Lin\", 2), Empty))\ncollect-map pairs resolving reject\n";
    assert!(analyze_for_compiler(unique_reject).is_ok());
}

#[test]
fn models_basic_int_list_operations_and_total_decomposition() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-PREPEND-001,
    // TOPAL-LIST-APPEND-001, TOPAL-LIST-CONCAT-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-LIST-EMPTY-001, TOPAL-LIST-ONE-001, TOPAL-LIST-UNCONS-001,
    // TOPAL-LIST-FIRST-001, TOPAL-LIST-REST-001, TOPAL-LIST-REVERSE-001,
    // TOPAL-COMPILER-LIST-INT-CORE-001
    let program =
        analyze_for_compiler(include_str!("../../../../../examples/language/lists.t")).unwrap();
    let list_int = CompilerType::List(Box::new(CompilerType::Int));
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].parameters[0].value_type, list_int);
    assert_eq!(
        program.functions[0].result_type,
        CompilerType::Optional(Box::new(CompilerType::Int))
    );
    assert!(matches!(
        program.functions[0].body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    assert_eq!(program.main.statements.len(), 8);
    assert!(program.main.statements.iter().all(|statement| matches!(
        statement,
        CompilerStatement::Binding(CompilerBinding { value, .. }) if value.value_type == list_int
    )));
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared List regression returns a Tuple")
    };
    assert_eq!(results.len(), 12);
    assert!(matches!(
        results[1].kind,
        CompilerExpressionKind::ListFirst(_)
    ));
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::ListRest(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[6].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert!(matches!(
        results[8].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[9].kind,
        CompilerExpressionKind::ListUncons(_)
    ));

    let mismatch =
        "use language (version is v0.1)\nvalues : List Int is Empty\nvalues append \"no\"\n";
    assert_eq!(
        analyze_for_compiler(mismatch).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );

    let incomplete = "use language (version is v0.1)\ninspect is fn (values : List Int) -> Int\n  values\n    Empty then 0\ninspect Empty\n";
    assert_eq!(
        analyze_for_compiler(incomplete).unwrap_err().code,
        "E-UNSUPPORTED-INCOMPLETE-DECISION"
    );

    let duplicate_binding = "use language (version is v0.1)\ninspect is fn (values : List Int) -> Int\n  values\n    Empty then 0\n    Entry (value, value) then value\nvalues : List Int is Entry (1, Empty)\ninspect values\n";
    assert_eq!(
        analyze_for_compiler(duplicate_binding).unwrap_err().code,
        "E-DUPLICATE-BINDING"
    );
}

#[test]
fn models_boolean_lists_across_private_boundaries() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-BOOLEAN-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-boolean-values.t"
    ))
    .unwrap();
    let list_boolean = CompilerType::List(Box::new(CompilerType::Boolean));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list_boolean);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_list = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-list")
        .unwrap();
    assert_eq!(return_list.result_type, list_boolean);
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list_boolean.clone(), CompilerType::Boolean])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list_boolean.clone()),
            ("fallback".into(), CompilerType::Boolean),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Boolean List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list_boolean);

    let unsupported_transform = "use language (version is v0.1)\nvalues : List Boolean is Entry (true, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported_transform)
            .unwrap_err()
            .code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_string_lists_across_private_boundaries() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-STRING-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-string-values.t"
    ))
    .unwrap();
    let list_string = CompilerType::List(Box::new(CompilerType::String));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list_string);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_list = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-list")
        .unwrap();
    assert_eq!(return_list.result_type, list_string);
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list_string.clone(), CompilerType::String])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list_string.clone()),
            ("fallback".into(), CompilerType::String),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared String List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list_string);

    let unsupported_transform = "use language (version is v0.1)\nvalues : List String is Entry (\"Top\", Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported_transform)
            .unwrap_err()
            .code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
fn models_character_lists_across_private_boundaries() {
    // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
    // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-ENTRY-COUNT-001,
    // TOPAL-LIST-EMPTY-PREDICATE-001, TOPAL-COMPILER-LIST-CHARACTER-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-character-values.t"
    ))
    .unwrap();
    let list_character = CompilerType::List(Box::new(CompilerType::Character));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list_character);
    assert_eq!(head.result_type, CompilerType::Character);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_list = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-list")
        .unwrap();
    assert_eq!(return_list.result_type, list_character);
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list_character.clone(), CompilerType::Character])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list_character.clone()),
            ("fallback".into(), CompilerType::Character),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Character List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list_character);

    let transform = "use language (version is v0.1)\nvalues : List Character is Entry (\"A\", Empty)\nvalues reverse\n";
    assert!(matches!(
        analyze_for_compiler(transform).unwrap().main.result.kind,
        CompilerExpressionKind::ListReverse(_)
    ));
}

#[test]
fn models_nat_lists_across_private_boundaries() {
    // TOPAL-NUM-NAT-001, TOPAL-TYPE-LIST-CONSTRUCT-001,
    // TOPAL-DECISION-LIST-001, TOPAL-TYPE-LIST-EQUALITY-001,
    // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-LIST-NAT-CORE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/list-nat-values.t"
    ))
    .unwrap();
    let list_nat = CompilerType::List(Box::new(CompilerType::Nat));
    let head = program
        .functions
        .iter()
        .find(|function| function.source_name == "head-or")
        .unwrap();
    assert_eq!(head.parameters[0].value_type, list_nat);
    assert_eq!(head.result_type, CompilerType::Nat);
    assert!(matches!(
        head.body.result.kind,
        CompilerExpressionKind::ListDecision { .. }
    ));
    let return_list = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-list")
        .unwrap();
    assert_eq!(return_list.result_type, list_nat);
    let return_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-pair")
        .unwrap();
    assert_eq!(
        return_pair.result_type,
        CompilerType::Tuple(vec![list_nat.clone(), CompilerType::Nat])
    );
    let return_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-record")
        .unwrap();
    assert_eq!(
        return_record.result_type,
        CompilerType::Record(vec![
            ("candidate".into(), list_nat.clone()),
            ("fallback".into(), CompilerType::Nat),
        ])
    );
    let CompilerExpressionKind::Tuple(results) = &program.main.result.kind else {
        panic!("shared Nat List regression returns a Tuple")
    };
    assert_eq!(results.len(), 10);
    assert!(matches!(
        results[2].kind,
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Equal,
            ..
        }
    ));
    assert!(matches!(
        results[4].kind,
        CompilerExpressionKind::ListEntryCount(_)
    ));
    assert!(matches!(
        results[5].kind,
        CompilerExpressionKind::ListEmptyPredicate(_)
    ));
    assert_eq!(results[9].value_type, list_nat);

    let unsupported_transform =
        "use language (version is v0.1)\nvalues : List Nat is Entry (1, Empty)\nvalues reverse\n";
    assert_eq!(
        analyze_for_compiler(unsupported_transform)
            .unwrap_err()
            .code,
        "E-COMPILER-UNSUPPORTED"
    );
}
