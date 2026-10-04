#[test]
#[allow(clippy::too_many_lines)] // Explicit completion provenance and rejections are one scenario.
fn models_custom_generator_explicit_string_return_before_yield() {
    // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-FINAL-RETURN-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-EXPLICIT-RETURN-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-explicit-return.t"
    ))
    .unwrap();
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
                    explicit_return: Some(return_span),
                    result,
                    ..
                },
                value_type: CompilerType::Generator(generator_type),
                ..
            },
            ..
        })] if name == "generated"
            && declaration == "done"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::String
            && exact_string(initial).as_deref() == Some("unused")
            && yields.is_empty()
            && continuations.is_empty()
            && program.source.slice(*return_span) == "return"
            && exact_string(result).as_deref() == Some("done")
            && *generator_type.yield_type == CompilerType::String
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::String
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach {
                yields,
                continuations,
                explicit_return: Some(return_span),
                parameter,
                body,
                result,
                ..
            },
            value_type: CompilerType::String,
            ..
        } if yields.is_empty()
            && continuations.is_empty()
            && program.source.slice(*return_span) == "return"
            && parameter.name == "text"
            && parameter.value_type == CompilerType::String
            && matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::StringEmptyPredicate(value),
                    ..
                })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                    if name == "text")
            )
            && exact_string(result).as_deref() == Some("done")
    ));

    let dynamic = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ndone is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  return \"done\"\ngenerated is done (identity \"unused\")\ngenerated foreach { text }\n  _ is empty? text\n",
        )
        .unwrap();
    assert!(matches!(
        &dynamic.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    initial,
                    yields,
                    explicit_return: Some(_),
                    ..
                },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::Call { .. })
            && yields.is_empty()
    ));

    for source in [
        "use language (version is v0.1)\ndone is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  \"done\"\ngenerated is done \"unused\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ndone is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  return initial\ngenerated is done \"unused\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ndone is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  return ()\ngenerated is done \"unused\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ndone is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  return ()\ngenerated is done \"unused\"\ngenerated foreach { text }\n  _ is empty? text\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_explicit_string_return_after_generator_resumption() {
    // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-RESUMPTION-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-RETURN-AFTER-YIELD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-return-after-yield.t"
    ))
    .unwrap();
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
                    explicit_return: Some(return_span),
                    result,
                    ..
                },
                value_type: CompilerType::Generator(generator_type),
                ..
            },
            ..
        })] if name == "generated"
            && declaration == "finish"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::String
            && exact_string(initial).as_deref() == Some("item")
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && program.source.slice(*return_span) == "return"
            && exact_string(result).as_deref() == Some("done")
            && *generator_type.yield_type == CompilerType::String
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::String
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach {
                yields,
                continuations,
                explicit_return: Some(return_span),
                parameter,
                body,
                result,
                ..
            },
            value_type: CompilerType::String,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && program.source.slice(*return_span) == "return"
            && parameter.name == "text"
            && parameter.value_type == CompilerType::String
            && matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::StringEmptyPredicate(value),
                    ..
                })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                    if name == "text")
            )
            && exact_string(result).as_deref() == Some("done")
    ));

    for source in [
        "use language (version is v0.1)\nfinish is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  _ is yield \"again\"\n  return \"done\"\ngenerated is finish \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\nfinish is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield \"item\"\n  return \"done\"\ngenerated is finish \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\nfinish is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  return initial\ngenerated is finish \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\nfinish is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  _ is empty? initial\n  return \"done\"\ngenerated is finish \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_boolean_values_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-BOOLEAN-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-boolean-values.t"
    ))
    .unwrap();
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
            && declaration == "invert"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::Boolean
            && matches!(initial.kind, CompilerExpressionKind::Boolean(true))
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(
                result.kind,
                CompilerExpressionKind::Not(ref value)
                    if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                        if name == "initial")
            )
            && *generator_type.yield_type == CompilerType::Boolean
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::Boolean
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
            value_type: CompilerType::Boolean,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && parameter.value_type == CompilerType::Boolean
            && matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::Not(value),
                    ..
                })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                    if name == "value")
            )
            && matches!(
                result.kind,
                CompilerExpressionKind::Not(ref value)
                    if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                        if name == "initial")
            )
    ));

    let dynamic = analyze_for_compiler(
            "use language (version is v0.1)\ninvert is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> Boolean\n  _ is yield initial\n  not initial\ngenerated is invert (not false)\ngenerated foreach { value }\n  _ is not value\n",
        )
        .unwrap();
    assert!(matches!(
        &dynamic.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator { initial, .. },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::Not(_))
    ));

    for source in [
        "use language (version is v0.1)\ninvert is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> Boolean\n  _ is yield false\n  not initial\ngenerated is invert true\ngenerated foreach { value }\n  _ is not value\n",
        "use language (version is v0.1)\ninvert is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> Boolean\n  _ is yield initial\n  _ is yield initial\n  not initial\ngenerated is invert true\ngenerated foreach { value }\n  _ is not value\n",
        "use language (version is v0.1)\ninvert is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> Boolean\n  _ is yield initial\n  initial\ngenerated is invert true\ngenerated foreach { value }\n  _ is not value\n",
        "use language (version is v0.1)\ninvert is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is invert true\ngenerated foreach { value }\n  _ is not value\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Exact post-resume decision and rejection matrix stay together.
fn models_custom_generator_final_boolean_decision() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-DECISION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-final-decision.t"
    ))
    .unwrap();
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
            && declaration == "describe"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::Boolean
            && matches!(initial.kind, CompilerExpressionKind::Boolean(true))
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(
                result.kind,
                CompilerExpressionKind::BooleanDecision {
                    ref subject,
                    ref when_true,
                    ref when_false,
                } if matches!(subject.kind, CompilerExpressionKind::Local(ref name)
                        if name == "initial")
                    && exact_string(when_true).as_deref() == Some("accepted")
                    && exact_string(when_false).as_deref() == Some("rejected")
            )
            && *generator_type.yield_type == CompilerType::Boolean
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::String
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
            value_type: CompilerType::String,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && parameter.value_type == CompilerType::Boolean
            && matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::Not(value),
                    ..
                })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                    if name == "value")
            )
            && matches!(
                result.kind,
                CompilerExpressionKind::BooleanDecision {
                    ref subject,
                    ref when_true,
                    ref when_false,
                } if matches!(subject.kind, CompilerExpressionKind::Local(ref name)
                        if name == "initial")
                    && exact_string(when_true).as_deref() == Some("accepted")
                    && exact_string(when_false).as_deref() == Some("rejected")
            )
    ));

    let dynamic = analyze_for_compiler(
            "use language (version is v0.1)\ndescribe is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> String\n  _ is yield initial\n  initial\n    true then \"accepted\"\n    otherwise \"rejected\"\ngenerated is describe (not false)\ngenerated foreach { value }\n  _ is not value\n",
        )
        .unwrap();
    assert!(matches!(
        &dynamic.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator { initial, .. },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::Not(_))
    ));

    for source in [
        "use language (version is v0.1)\ndescribe is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> String\n  _ is yield false\n  initial\n    true then \"accepted\"\n    otherwise \"rejected\"\ngenerated is describe true\ngenerated foreach { value }\n  _ is not value\n",
        "use language (version is v0.1)\ndescribe is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> String\n  _ is yield initial\n  initial\n    false then \"rejected\"\n    otherwise \"accepted\"\ngenerated is describe true\ngenerated foreach { value }\n  _ is not value\n",
        "use language (version is v0.1)\ndescribe is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> String\n  _ is yield initial\n  initial\n    true then \"yes\"\n    otherwise \"rejected\"\ngenerated is describe true\ngenerated foreach { value }\n  _ is not value\n",
        "use language (version is v0.1)\ndescribe is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> String\n  _ is yield initial\n  \"accepted\"\ngenerated is describe true\ngenerated foreach { value }\n  _ is not value\n",
        "use language (version is v0.1)\ndescribe is generator (initial : Boolean)\n  yields Boolean\n  resumes Unit\n  -> String\n  _ is yield initial\n  not initial\n    true then \"accepted\"\n    otherwise \"rejected\"\ngenerated is describe true\ngenerated foreach { value }\n  _ is not value\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Retained declarations, traversal, and rejection matrix form one model contract.
fn models_generator_local_enum_and_function_across_suspension() {
    // TOPAL-GENERATOR-LOCAL-FUNCTION-001, TOPAL-GENERATOR-LOCAL-ENUM-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-FUNCTION-ORDINARY-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-FUNCTION-001
    let source = include_str!("../../../../../examples/language/custom-generator-local-function.t");
    let program = analyze_for_compiler(source).unwrap();
    let [function] = program.functions.as_slice() else {
        panic!("one retained generator-local function instance")
    };
    assert_eq!(function.source_name, "label");
    assert!(function.symbol.starts_with("topal.fn.label."));
    assert_eq!(function.result_type, CompilerType::String);
    assert!(matches!(
        function.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type: CompilerType::Enum(CompilerEnumType {
                name: enum_name,
                alternatives,
            }),
            ..
        }] if name == "value"
            && enum_name == "Choice"
            && alternatives == &[String::from("Accepted"), String::from("Rejected")]
    ));
    assert!(matches!(
        &function.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::EnumDecision {
                subject,
                rules,
                otherwise: None,
            },
            value_type: CompilerType::String,
            ..
        } if matches!(subject.kind, CompilerExpressionKind::Local(ref name)
                if name == "value")
            && matches!(rules.as_slice(), [accepted, rejected]
                if accepted.value == 0
                    && exact_string(&accepted.action).as_deref() == Some("accepted")
                    && rejected.value == 1
                    && exact_string(&rejected.action).as_deref() == Some("rejected"))
    ));
    let symbol = function.symbol.as_str();
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    declaration,
                    yields,
                    result,
                    ..
                },
                ..
            },
            ..
        })] if name == "generated"
            && declaration == "describe"
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && matches!(result.kind,
                CompilerExpressionKind::Call { symbol: ref call_symbol, ref arguments }
                    if call_symbol == symbol
                        && matches!(arguments.as_slice(), [CompilerExpression {
                            kind: CompilerExpressionKind::Enum(0),
                            value_type: CompilerType::Enum(_),
                            ..
                        }]))
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach {
                parameter,
                body,
                result,
                ..
            },
            value_type: CompilerType::String,
            ..
        } if parameter.name == "value"
            && matches!(body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::Not(value),
                    ..
                })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                    if name == "value"))
            && matches!(result.kind,
                CompilerExpressionKind::Call { symbol: ref call_symbol, .. }
                    if call_symbol == symbol)
    ));

    for malformed in [
        source.replace(
            "Choice is Enum ( Accepted, Rejected )",
            "Choice is Enum ( Rejected, Accepted )",
        ),
        source.replace("value : Choice", "value : Boolean"),
        source.replace("Rejected then \"rejected\"", "Rejected then \"declined\""),
        source.replace("_ is yield initial", "_ is yield false"),
        source.replace("label Accepted", "label Rejected"),
    ] {
        assert_eq!(
            analyze_for_compiler(&malformed).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let escaped = format!(
        "{}label Accepted\n",
        source
            .split_once("generated is")
            .expect("fixture constructs its generator")
            .0
    );
    assert_eq!(
        analyze_for_compiler(&escaped).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Accepted graph and exact rejection matrix stay together.
fn models_arbitrary_precision_ints_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-INT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-int-values.t"
    ))
    .unwrap();
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
            && initial_parameter.value_type == CompilerType::Int
            && matches!(
                initial.kind,
                CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::parse_bytes(b"999999999999999999999999999999", 10).unwrap()
            )
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
                    && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                        if value == &BigInt::from(1))
            )
            && *generator_type.yield_type == CompilerType::Int
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::Int
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
            value_type: CompilerType::Int,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && parameter.value_type == CompilerType::Int
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
                    && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                        if value == &BigInt::from(1))
            )
            && matches!(
                result.kind,
                CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    ref left,
                    ref right,
                } if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                    if name == "initial")
                    && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                        if value == &BigInt::from(1))
            )
    ));

    let expression_input = analyze_for_compiler(
            "use language (version is v0.1)\nnext is generator (initial : Int)\n  yields Int\n  resumes Unit\n  -> Int\n  _ is yield initial\n  initial + 1\ngenerated is next (40 + 2)\ngenerated foreach { value }\n  _ is value + 1\n",
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
            operation: CompilerBinary::Add,
            ..
        })
    ));

    for source in [
        "use language (version is v0.1)\nnext is generator (initial : Int)\n  yields Int\n  resumes Unit\n  -> Int\n  _ is yield 1\n  initial + 1\ngenerated is next 41\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Int)\n  yields Int\n  resumes Unit\n  -> Int\n  _ is yield initial\n  _ is yield initial\n  initial + 1\ngenerated is next 41\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Int)\n  yields Int\n  resumes Unit\n  -> Int\n  _ is yield initial\n  initial\ngenerated is next 41\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Int)\n  yields Int\n  resumes Unit\n  -> Int\n  _ is yield initial\n  initial + 2\ngenerated is next 41\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Int)\n  yields Int\n  resumes Unit\n  -> Int\n  _ is yield initial\n  initial + 1\ngenerated is next 41\ngenerated foreach { value }\n  _ is value + 2\n",
        "use language (version is v0.1)\nnext is generator (initial : Int)\n  yields Int\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is next 41\ngenerated foreach { value }\n  _ is value + 1\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Accepted graph and exact rejection matrix stay together.
fn models_nonnegative_nats_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-NUM-NAT-CONSTRUCT-001, TOPAL-COMPILER-GENERATOR-NAT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-nat-values.t"
    ))
    .unwrap();
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
            && initial_parameter.value_type == CompilerType::Nat
            && matches!(initial.kind, CompilerExpressionKind::IntToNat(ref value)
                if matches!(value.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(7)))
            && initial.value_type == CompilerType::Nat
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(result.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                ref left,
                ref right,
            } if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
                && left.value_type == CompilerType::Nat
                && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(1)))
            && result.value_type == CompilerType::Nat
            && *generator_type.yield_type == CompilerType::Nat
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::Nat
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
            value_type: CompilerType::Nat,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "value"
            && parameter.value_type == CompilerType::Nat
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Add,
                        left,
                        right,
                    },
                    value_type: CompilerType::Nat,
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "value")
                && left.value_type == CompilerType::Nat
                && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(1)))
            && matches!(result.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                ref left,
                ref right,
            } if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
                && left.value_type == CompilerType::Nat
                && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(1)))
            && result.value_type == CompilerType::Nat
    ));

    let expression_input = analyze_for_compiler(
            "use language (version is v0.1)\nnext is generator (initial : Nat)\n  yields Nat\n  resumes Unit\n  -> Nat\n  _ is yield initial\n  initial + 1\ngenerated is next (Nat (6 + 1))\ngenerated foreach { value }\n  _ is value + 1\n",
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
        }) if matches!(initial.kind, CompilerExpressionKind::IntToNat(ref value)
            if matches!(value.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                ..
            }))
    ));

    for source in [
        "use language (version is v0.1)\nnext is generator (initial : Nat)\n  yields Nat\n  resumes Unit\n  -> Nat\n  _ is yield (Nat 1)\n  initial + 1\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Nat)\n  yields Nat\n  resumes Unit\n  -> Nat\n  _ is yield initial\n  _ is yield initial\n  initial + 1\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Nat)\n  yields Nat\n  resumes Unit\n  -> Nat\n  _ is yield initial\n  initial\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Nat)\n  yields Nat\n  resumes Unit\n  -> Nat\n  _ is yield initial\n  initial + 2\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n",
        "use language (version is v0.1)\nnext is generator (initial : Nat)\n  yields Nat\n  resumes Unit\n  -> Nat\n  _ is yield initial\n  initial + 1\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 2\n",
        "use language (version is v0.1)\nnext is generator (initial : Nat)\n  yields Nat\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Accepted nominal graph and rejection matrix stay together.
fn models_nominal_enum_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-ENUM-001, TOPAL-COMPILER-GENERATOR-ENUM-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-enum-values.t"
    ))
    .unwrap();
    let choice = CompilerType::Enum(CompilerEnumType {
        name: "Choice".into(),
        alternatives: vec!["First".into(), "Second".into()],
    });
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
            && declaration == "choose"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == choice
            && matches!(initial.kind, CompilerExpressionKind::Enum(0))
            && initial.value_type == choice
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(result.kind, CompilerExpressionKind::Enum(1))
            && result.value_type == choice
            && *generator_type.yield_type == choice
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == choice
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
            && parameter.name == "choice"
            && parameter.value_type == choice
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "choice")
                && left.value_type == choice
                && matches!(right.kind, CompilerExpressionKind::Enum(0))
                && right.value_type == choice)
            && matches!(result.kind, CompilerExpressionKind::Enum(1))
            && result.value_type == choice
            && value_type == &choice
    ));

    for source in [
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nchoose is generator (initial : Choice)\n  yields Choice\n  resumes Unit\n  -> Choice\n  _ is yield Second\n  Second\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nchoose is generator (initial : Choice)\n  yields Choice\n  resumes Unit\n  -> Choice\n  _ is yield initial\n  _ is yield initial\n  Second\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nchoose is generator (initial : Choice)\n  yields Choice\n  resumes Unit\n  -> Choice\n  _ is yield initial\n  initial\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nchoose is generator (initial : Choice)\n  yields Choice\n  resumes Unit\n  -> Choice\n  _ is yield initial\n  First\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nchoose is generator (initial : Choice)\n  yields Choice\n  resumes Unit\n  -> Choice\n  _ is yield initial\n  Second\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = Second\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nchoose is generator (initial : Choice)\n  yields Choice\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Ordered product graph and rejection matrix stay together.
fn models_ordered_product_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-PRODUCT-001, TOPAL-COMPILER-GENERATOR-PRODUCT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-product-values.t"
    ))
    .unwrap();
    let product = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]);
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
            && declaration == "pair"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == product
            && matches!(&initial.kind, CompilerExpressionKind::Tuple(fields)
                if matches!(fields.as_slice(), [int, text]
                    if exact_int(int).as_ref() == Some(&BigInt::from(7))
                        && exact_string(text).as_deref() == Some("item")))
            && initial.value_type == product
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(&result.kind, CompilerExpressionKind::Tuple(fields)
                if matches!(fields.as_slice(), [int, text]
                    if exact_int(int).as_ref() == Some(&BigInt::from(8))
                        && exact_string(text).as_deref() == Some("done")))
            && result.value_type == product
            && *generator_type.yield_type == product
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == product
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
            && parameter.name == "value"
            && parameter.value_type == product
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "value")
                && left.value_type == product
                && matches!(&right.kind, CompilerExpressionKind::Tuple(fields)
                    if matches!(fields.as_slice(), [int, text]
                        if exact_int(int).as_ref() == Some(&BigInt::from(7))
                            && exact_string(text).as_deref() == Some("item")))
                && right.value_type == product)
            && matches!(&result.kind, CompilerExpressionKind::Tuple(fields)
                if matches!(fields.as_slice(), [int, text]
                    if exact_int(int).as_ref() == Some(&BigInt::from(8))
                        && exact_string(text).as_deref() == Some("done")))
            && result.value_type == product
            && value_type == &product
    ));

    for source in [
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n  _ is yield (7, \"item\")\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n  _ is yield initial\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n  _ is yield initial\n  initial\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n  _ is yield initial\n  (9, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n  _ is yield initial\n  (8, \"other\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (8, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"other\")\n",
        "use language (version is v0.1)\npair is generator (initial : (Int, String))\n  yields (String, Int)\n  resumes Unit\n  -> (Int, String)\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Result propagation graph and rejection matrix stay together.
fn models_result_rational_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RESULT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-result-values.t"
    ))
    .unwrap();
    let result_type = CompilerType::Result(Box::new(CompilerType::Rational));
    let one = BigRational::from_integer(BigInt::from(1));
    let zero = BigRational::from_integer(BigInt::from(0));
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
            && declaration == "attempt"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == result_type
            && matches!(&initial.kind, CompilerExpressionKind::ResultSuccess(value)
                if value.rational_value.as_ref() == Some(&one))
            && initial.value_type == result_type
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && matches!(&result.kind, CompilerExpressionKind::Fallible {
                operation: CompilerFallible::RationalDivide,
                left,
                right,
                ..
            } if left.rational_value.as_ref() == Some(&one)
                && right.rational_value.as_ref() == Some(&zero))
            && result.value_type == result_type
            && *generator_type.yield_type == result_type
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == result_type
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
            && parameter.value_type == result_type
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Boolean(true),
                    value_type: CompilerType::Boolean,
                    ..
                }
            )])
            && matches!(&result.kind, CompilerExpressionKind::Fallible {
                operation: CompilerFallible::RationalDivide,
                left,
                right,
                ..
            } if left.rational_value.as_ref() == Some(&one)
                && right.rational_value.as_ref() == Some(&zero))
            && result.value_type == result_type
            && value_type == &result_type
    ));

    for source in [
        "use language (version is v0.1)\nattempt is generator (initial : Result (Rational, lang arithmetic ArithmeticErrorCode))\n  yields Result (Rational, lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  _ is yield (Rational 1)\n  initial / (Rational 0)\ngenerated is attempt (Rational 1)\ngenerated foreach { candidate }\n  _ is candidate = candidate\n",
        "use language (version is v0.1)\nattempt is generator (initial : Result (Rational, lang arithmetic ArithmeticErrorCode))\n  yields Result (Rational, lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  _ is yield initial\n  initial / (Rational 0)\ngenerated is attempt (Rational 1)\ngenerated foreach { candidate }\n  _ is candidate = candidate\n",
        "use language (version is v0.1)\nattempt is generator (initial : Result (Rational, lang arithmetic ArithmeticErrorCode))\n  yields Result (Rational, lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  initial / (Rational 1)\ngenerated is attempt (Rational 1)\ngenerated foreach { candidate }\n  _ is candidate = candidate\n",
        "use language (version is v0.1)\nattempt is generator (initial : Result (Rational, lang arithmetic ArithmeticErrorCode))\n  yields Result (Rational, lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  initial\ngenerated is attempt (Rational 1)\ngenerated foreach { candidate }\n  _ is candidate = candidate\n",
        "use language (version is v0.1)\nattempt is generator (initial : Result (Rational, lang arithmetic ArithmeticErrorCode))\n  yields Result (Rational, lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  initial / (Rational 0)\ngenerated is attempt (Rational 2)\ngenerated foreach { candidate }\n  _ is candidate = candidate\n",
        "use language (version is v0.1)\nattempt is generator (initial : Result (Rational, lang arithmetic ArithmeticErrorCode))\n  yields Result (Rational, lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  initial / (Rational 0)\ngenerated is attempt (Rational 1)\ngenerated foreach { candidate }\n  _ is candidate = candidate\n  _ is candidate = candidate\n",
        "use language (version is v0.1)\nattempt is generator (initial : Result (Rational, lang arithmetic ArithmeticErrorCode))\n  yields Result (Rational, lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is attempt (Rational 1)\ngenerated foreach { candidate }\n  _ is candidate = candidate\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Nominal Comparison graph and rejection matrix stay together.
fn models_comparison_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-DECISION-COMPARISON-001, TOPAL-COMPILER-GENERATOR-COMPARISON-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-comparison-values.t"
    ))
    .unwrap();
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
            && declaration == "order"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::Comparison
            && exact_int_comparison_value(initial, 1, 2)
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_int_comparison_value(result, 3, 2)
            && *generator_type.yield_type == CompilerType::Comparison
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::Comparison
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
            value_type: CompilerType::Comparison,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "comparison"
            && parameter.value_type == CompilerType::Comparison
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "comparison")
                && exact_int_comparison_value(right, 1, 2))
            && exact_int_comparison_value(result, 3, 2)
    ));

    for source in [
        "use language (version is v0.1)\norder is generator (initial : Comparison)\n  yields Comparison\n  resumes Unit\n  -> Comparison\n  _ is yield (1 <=> 2)\n  3 <=> 2\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n",
        "use language (version is v0.1)\norder is generator (initial : Comparison)\n  yields Comparison\n  resumes Unit\n  -> Comparison\n  _ is yield initial\n  _ is yield initial\n  3 <=> 2\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n",
        "use language (version is v0.1)\norder is generator (initial : Comparison)\n  yields Comparison\n  resumes Unit\n  -> Comparison\n  _ is yield initial\n  2 <=> 3\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n",
        "use language (version is v0.1)\norder is generator (initial : Comparison)\n  yields Comparison\n  resumes Unit\n  -> Comparison\n  _ is yield initial\n  3 <=> 2\ngenerated is order (2 <=> 1)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n",
        "use language (version is v0.1)\norder is generator (initial : Comparison)\n  yields Comparison\n  resumes Unit\n  -> Comparison\n  _ is yield initial\n  3 <=> 2\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (2 <=> 1)\n",
        "use language (version is v0.1)\norder is generator (initial : Comparison)\n  yields Comparison\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Recursive Optional/product graph and rejection matrix stay together.
fn models_nested_optional_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-OPTIONAL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-nested-optional-values.t"
    ))
    .unwrap();
    let optional_product = CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
        CompilerType::Int,
        CompilerType::String,
    ])));
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
            && declaration == "pair"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == optional_product
            && exact_optional_int_string_value(initial, 7, "item")
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_optional_int_string_value(result, 8, "done")
            && *generator_type.yield_type == optional_product
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == optional_product
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
            && parameter.value_type == optional_product
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "candidate")
                && exact_optional_int_string_value(right, 7, "item"))
            && exact_optional_int_string_value(result, 8, "done")
            && value_type == &optional_product
    ));

    for source in [
        "use language (version is v0.1)\npair is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield (Some (7, \"item\"))\n  Some (8, \"done\")\ngenerated is pair (Some (7, \"item\"))\ngenerated foreach { candidate }\n  _ is candidate = (Some (7, \"item\"))\n",
        "use language (version is v0.1)\npair is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  _ is yield initial\n  Some (8, \"done\")\ngenerated is pair (Some (7, \"item\"))\ngenerated foreach { candidate }\n  _ is candidate = (Some (7, \"item\"))\n",
        "use language (version is v0.1)\npair is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  Some (9, \"done\")\ngenerated is pair (Some (7, \"item\"))\ngenerated foreach { candidate }\n  _ is candidate = (Some (7, \"item\"))\n",
        "use language (version is v0.1)\npair is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  Some (8, \"done\")\ngenerated is pair (Some (8, \"item\"))\ngenerated foreach { candidate }\n  _ is candidate = (Some (7, \"item\"))\n",
        "use language (version is v0.1)\npair is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  Some (8, \"done\")\ngenerated is pair (Some (7, \"item\"))\ngenerated foreach { candidate }\n  _ is candidate = (Some (7, \"other\"))\n",
        "use language (version is v0.1)\npair is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is pair (Some (7, \"item\"))\ngenerated foreach { candidate }\n  _ is candidate = (Some (7, \"item\"))\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Recursive absent Optional graph and rejection matrix stay together.
fn models_nested_absent_optional_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-NONE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-nested-none-values.t"
    ))
    .unwrap();
    let optional_product = CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
        CompilerType::Int,
        CompilerType::String,
    ])));
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
            && declaration == "absent"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == optional_product
            && exact_optional_int_string_none_value(initial)
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_optional_int_string_none_value(result)
            && *generator_type.yield_type == optional_product
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == optional_product
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
            && parameter.value_type == optional_product
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "candidate")
                && exact_optional_int_string_none_value(right))
            && exact_optional_int_string_none_value(result)
            && value_type == &optional_product
    ));

    for source in [
        "use language (version is v0.1)\nabsent is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield (None (Int, String))\n  None (Int, String)\ngenerated is absent (None (Int, String))\ngenerated foreach { candidate }\n  _ is candidate = (None (Int, String))\n",
        "use language (version is v0.1)\nabsent is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  _ is yield initial\n  None (Int, String)\ngenerated is absent (None (Int, String))\ngenerated foreach { candidate }\n  _ is candidate = (None (Int, String))\n",
        "use language (version is v0.1)\nabsent is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  Some (8, \"done\")\ngenerated is absent (None (Int, String))\ngenerated foreach { candidate }\n  _ is candidate = (None (Int, String))\n",
        "use language (version is v0.1)\nabsent is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  None (Int, String)\ngenerated is absent (Some (7, \"item\"))\ngenerated foreach { candidate }\n  _ is candidate = (None (Int, String))\n",
        "use language (version is v0.1)\nabsent is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  None (Int, String)\ngenerated is absent (None (Int, String))\ngenerated foreach { candidate }\n  _ is candidate = (Some (7, \"item\"))\n",
        "use language (version is v0.1)\nabsent is generator (initial : Optional (Int, String))\n  yields Optional (Int, String)\n  resumes Unit\n  -> Optional (Int, String)\n  _ is yield initial\n  None (Int, Int)\ngenerated is absent (None (Int, String))\ngenerated foreach { candidate }\n  _ is candidate = (None (Int, String))\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Recursive Result/product graph and rejection matrix stay together.
fn models_nested_result_product_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-RESULT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-nested-result-values.t"
    ))
    .unwrap();
    let result_product = nested_result_product_type();
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
            && declaration == "pair"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == result_product
            && exact_nested_result_product_value(initial, 7, "item")
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_nested_result_product_value(result, 8, "done")
            && *generator_type.yield_type == result_product
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == result_product
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
            && parameter.value_type == result_product
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "candidate")
                && exact_nested_result_product_value(right, 7, "item"))
            && exact_nested_result_product_value(result, 8, "done")
            && value_type == &result_product
    ));

    for source in [
        "use language (version is v0.1)\npair is generator (initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode))\n  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  _ is yield (7, \"item\")\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { candidate }\n  _ is candidate = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode))\n  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { candidate }\n  _ is candidate = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode))\n  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  (9, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { candidate }\n  _ is candidate = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode))\n  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (8, \"item\")\ngenerated foreach { candidate }\n  _ is candidate = (7, \"item\")\n",
        "use language (version is v0.1)\npair is generator (initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode))\n  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { candidate }\n  _ is candidate = (7, \"other\")\n",
        "use language (version is v0.1)\npair is generator (initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode))\n  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is pair (7, \"item\")\ngenerated foreach { candidate }\n  _ is candidate = (7, \"item\")\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Recursive nominal graph and exact rejection matrix stay together.
fn models_recursive_nominal_values_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-ENUM-001, TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RECURSIVE-NOMINAL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-recursive-nominal-values.t"
    ))
    .unwrap();
    let enumeration = CompilerEnumType {
        name: "Choice".to_owned(),
        alternatives: vec!["First".to_owned(), "Second".to_owned()],
    };
    let recursive = recursive_nominal_generator_type(&enumeration);
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
            && declaration == "both"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == recursive
            && exact_recursive_nominal_value(initial, 0)
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && continuations.is_empty()
            && exact_recursive_nominal_value(result, 1)
            && *generator_type.yield_type == recursive
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == recursive
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
            && parameter.value_type == recursive
            && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
                CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Equal,
                        left,
                        right,
                    },
                    ..
                }
            )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == "candidate")
                && exact_recursive_nominal_value(right, 0))
            && exact_recursive_nominal_value(result, 1)
            && value_type == &recursive
    ));

    for source in [
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nboth is generator (initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)))\n  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  resumes Unit\n  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  _ is yield (Some First, First)\n  (Some Second, Second)\ngenerated is both (Some First, First)\ngenerated foreach { candidate }\n  _ is candidate = (Some First, First)\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nboth is generator (initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)))\n  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  resumes Unit\n  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  _ is yield initial\n  _ is yield initial\n  (Some Second, Second)\ngenerated is both (Some First, First)\ngenerated foreach { candidate }\n  _ is candidate = (Some First, First)\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nboth is generator (initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)))\n  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  resumes Unit\n  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  _ is yield initial\n  (Some First, First)\ngenerated is both (Some First, First)\ngenerated foreach { candidate }\n  _ is candidate = (Some First, First)\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nboth is generator (initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)))\n  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  resumes Unit\n  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  _ is yield initial\n  (Some Second, First)\ngenerated is both (Some First, First)\ngenerated foreach { candidate }\n  _ is candidate = (Some First, First)\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nboth is generator (initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)))\n  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  resumes Unit\n  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  _ is yield initial\n  (Some Second, Second)\ngenerated is both (Some Second, Second)\ngenerated foreach { candidate }\n  _ is candidate = (Some First, First)\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nboth is generator (initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)))\n  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  resumes Unit\n  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  _ is yield initial\n  (Some Second, Second)\ngenerated is both (Some First, First)\ngenerated foreach { candidate }\n  _ is candidate = (Some Second, Second)\n",
        "use language (version is v0.1)\nChoice is Enum (Second, First)\nboth is generator (initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)))\n  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  resumes Unit\n  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\n  _ is yield initial\n  (Some Second, Second)\ngenerated is both (Some First, First)\ngenerated foreach { candidate }\n  _ is candidate = (Some First, First)\n",
        "use language (version is v0.1)\nChoice is Enum (First, Second)\nboth is generator (initial : (Optional Choice, Optional Choice))\n  yields (Optional Choice, Optional Choice)\n  resumes Unit\n  -> (Optional Choice, Optional Choice)\n  _ is yield initial\n  (Some Second, Some Second)\ngenerated is both (Some First, Some First)\ngenerated foreach { candidate }\n  _ is candidate = (Some First, Some First)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}
