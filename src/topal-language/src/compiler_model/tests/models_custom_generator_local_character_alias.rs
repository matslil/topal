#[test]
fn models_custom_generator_local_character_alias() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-LOCAL-BINDING-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-BINDING-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-local-binding.t"
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
                        locals,
                        ..
                    },
                    ..
                },
                ..
            }),
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    characters: yielded,
                    locals: traversal_locals,
                    parameter,
                    ..
                },
                value_type: CompilerType::Unit,
                ..
            })
        ] if declaration == "copy-once"
            && characters == &[String::from("T")]
            && locals.len() == 1
            && locals[0].parameter.name == "copy"
            && locals[0].parameter.value_type == CompilerType::Character
            && locals[0].activation_after_resumptions == 0
            && traversal_locals == locals
            && yielded == &[String::from("T")]
            && parameter.value_type == CompilerType::Character
    ));

    for source in [
        "use language (version is v0.1)\ncopy-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  copy : Int is initial\n  _ is yield copy\n  ()\ngenerated is copy-once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\ncopy-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  copy : Character is initial\n  _ is 1\n  _ is yield copy\n  ()\ngenerated is copy-once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let escaped = "use language (version is v0.1)\ncopy-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  copy : Character is initial\n  _ is yield copy\n  ()\ngenerated is copy-once \"T\"\ngenerated foreach { character }\n  _ is String character\n_ is String copy\n";
    assert_eq!(
        analyze_for_compiler(escaped).unwrap_err().code,
        "E-UNBOUND-NAME"
    );
}

#[test]
fn models_custom_generator_local_activation_after_resume() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-BODY-STATEMENT-001,
    // TOPAL-GENERATOR-LOCAL-BINDING-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-SUSPENSION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-suspension.t"
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
                        locals,
                        ..
                    },
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
        ] if declaration == "pause-twice"
            && characters == &[String::from("T"), String::from("T")]
            && locals.len() == 1
            && locals[0].parameter.name == "copy"
            && locals[0].parameter.value_type == CompilerType::Character
            && locals[0].activation_after_resumptions == 1
            && traversal_locals == locals
            && yielded == characters
            && parameter.value_type == CompilerType::Character
            && body.result.value_type == CompilerType::Unit
    ));

    for source in [
        "use language (version is v0.1)\npause is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  copy : Character is initial\n  ()\ngenerated is pause \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\npause is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  copy : Character is initial\n  other : Character is initial\n  _ is yield other\n  ()\ngenerated is pause \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\npause is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  copy : Character is \"U\"\n  _ is yield copy\n  ()\ngenerated is pause \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\npause is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  copy : Character is initial\n  _ is yield initial\n  ()\ngenerated is pause \"T\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_custom_generator_unit_resume_binding() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-RESUME-BINDING-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-resume-binding.t"
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
                        locals,
                        ..
                    },
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
        ] if declaration == "bind-resume"
            && characters == &[String::from("T")]
            && locals.len() == 1
            && locals[0].parameter.name == "resumed"
            && locals[0].parameter.value_type == CompilerType::Unit
            && locals[0].activation_after_resumptions == 1
            && traversal_locals == locals
            && yielded == characters
            && parameter.value_type == CompilerType::Character
            && body.result.value_type == CompilerType::Unit
    ));

    for source in [
        "use language (version is v0.1)\nbind-resume is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  resumed : Character is yield initial\n  resumed\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nbind-resume is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  resumed is yield initial\n  ()\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nbind-resume is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  resumed is yield initial\n  other\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nbind-resume is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  resumed is yield initial\n  resumed\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let escaped = "use language (version is v0.1)\nbind-resume is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  resumed is yield initial\n  resumed\ngenerated is bind-resume \"T\"\ngenerated foreach { character }\n  _ is String character\nresumed\n";
    assert_eq!(
        analyze_for_compiler(escaped).unwrap_err().code,
        "E-UNBOUND-NAME"
    );
}

#[test]
fn models_function_local_custom_generator_close() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-CLOSE-001,
    // TOPAL-GENERATOR-ERROR-CODE-001, TOPAL-COMPILER-GENERATOR-CLOSE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-close.t"
    ))
    .unwrap();
    let abandon = program
        .functions
        .iter()
        .find(|function| function.source_name == "abandon")
        .expect("called abandon function is instantiated");
    assert!(matches!(
        (abandon.parameters.as_slice(), abandon.body.statements.as_slice()),
        (
            [CompilerParameter {
                value_type: CompilerType::Character,
                ..
            }],
            [
                CompilerStatement::Binding(CompilerBinding {
                    name,
                    value: CompilerExpression {
                        kind: CompilerExpressionKind::CustomCharacterGenerator {
                            declaration,
                            characters,
                            locals,
                            result,
                            ..
                        },
                        ..
                    },
                    ..
                }),
                CompilerStatement::Discard(CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomCharacterClose {
                            generator,
                            provenance,
                            close_domain,
                        },
                    value_type: CompilerType::Unit,
                    ..
                })
            ]
        ) if name == "generated"
            && declaration == "pause-once"
            && characters == &[String::from("T")]
            && locals.is_empty()
            && result.value_type == CompilerType::Unit
            && matches!(generator.kind, CompilerExpressionKind::Local(_))
            && matches!(provenance.kind, CompilerExpressionKind::CustomCharacterGenerator { .. })
            && close_domain == "root"
    ));
    assert_eq!(abandon.body.result.value_type, CompilerType::Unit);

    for source in [
        "use language (version is v0.1)\npause-twice is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is yield initial\n  ()\nabandon is fn (initial : Character) -> Unit\n  generated is pause-twice initial\n  ()\nabandon \"T\"\n",
        "use language (version is v0.1)\ncopy-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  copy : Character is initial\n  _ is yield copy\n  ()\nabandon is fn (initial : Character) -> Unit\n  generated is copy-once initial\n  ()\nabandon \"T\"\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nabandon is fn (initial : Character) -> Unit\n  first is pause-once initial\n  second is pause-once initial\n  ()\nabandon \"T\"\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nabandon is fn (initial : Character) -> Unit\n  generated is pause-once initial\n  return ()\nabandon \"T\"\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nabandon is fn (initial : Character) -> Character\n  generated is pause-once initial\n  initial\n_ is abandon \"T\"\n()\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_function_local_custom_generator_close_handler() {
    // TOPAL-GENERATOR-CLOSE-001, TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-GENERATOR-ERROR-CODE-001,
    // TOPAL-COMPILER-GENERATOR-CLOSE-HANDLER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-close-handler.t"
    ))
    .unwrap();
    let abandon = program
        .functions
        .iter()
        .find(|function| function.source_name == "abandon")
        .expect("called abandon function is instantiated");
    let [
        CompilerStatement::Binding(CompilerBinding {
            value:
                CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomCharacterGenerator {
                            close_handler: Some(construction_handler),
                            ..
                        },
                    ..
                },
            ..
        }),
        CompilerStatement::Discard(CompilerExpression {
            kind:
                CompilerExpressionKind::CustomCharacterHandledClose {
                    handler: close_handler,
                    ..
                },
            ..
        }),
    ] = abandon.body.statements.as_slice()
    else {
        panic!("function retains one handled custom close after construction")
    };
    assert_eq!(construction_handler, close_handler);
    assert_eq!(close_handler.result_binding, "resume-result");
    assert!(close_handler.error_codes.is_empty());
    assert_eq!(close_handler.error_binding, "problem");
    assert_eq!(close_handler.ok_binding, "resumed");
    assert_eq!(
        close_handler.error_code_type,
        CompilerEnumType {
            name: "lang generator GeneratorErrorCode".into(),
            alternatives: vec!["generator-closed".into()],
        }
    );
    assert!(matches!(
        close_handler.error_action.kind,
        CompilerExpressionKind::Unit
    ));
    assert!(matches!(
        close_handler.ok_action.kind,
        CompilerExpressionKind::Unit
    ));

    for source in [
        "use language (version is v0.1)\nhandle-close is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  resume-result is yield initial\n  resume-result\n    Error problem then ()\n    Ok resumed then ()\ngenerated is handle-close \"T\"\n()\n",
        "use language (version is v0.1)\nhandle-close is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  resume-result is yield initial\n  resume-result\n    Error problem then problem\n    Ok resumed then ()\nabandon is fn (initial : Character) -> Unit\n  generated is handle-close initial\n  ()\nabandon \"T\"\n",
        "use language (version is v0.1)\nhandle-close is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  resume-result is yield initial\n  resume-result\n    Error problem then ()\n    Ok resumed then ()\ngenerated is handle-close \"T\"\ngenerated foreach { value }\n  _ is String value\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_qualified_custom_generator_close_code_pattern() {
    // TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001,
    // TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-COMPILER-GENERATOR-CLOSE-CODE-PATTERN-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-close-code-pattern.t"
    ))
    .unwrap();
    let abandon = program
        .functions
        .iter()
        .find(|function| function.source_name == "abandon")
        .expect("called abandon function is instantiated");
    let [
        CompilerStatement::Binding(CompilerBinding {
            value:
                CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomCharacterGenerator {
                            close_handler: Some(construction_handler),
                            ..
                        },
                    ..
                },
            ..
        }),
        CompilerStatement::Discard(CompilerExpression {
            kind:
                CompilerExpressionKind::CustomCharacterHandledClose {
                    handler: close_handler,
                    ..
                },
            ..
        }),
    ] = abandon.body.statements.as_slice()
    else {
        panic!("function retains one code-qualified handled close")
    };
    assert_eq!(construction_handler, close_handler);
    let [code_rule] = close_handler.error_codes.as_slice() else {
        panic!("handler retains exactly one qualified close-code rule")
    };
    assert_eq!(code_rule.code, 0);
    assert!(matches!(
        code_rule.action.kind,
        CompilerExpressionKind::Unit
    ));
    assert_eq!(close_handler.error_binding, "problem");
    assert_eq!(close_handler.ok_binding, "resumed");
    assert_eq!(
        close_handler.error_code_type.name,
        "lang generator GeneratorErrorCode"
    );

    for (source, expected) in [
            (
                include_str!("../../../../../examples/language/custom-generator-close-code-pattern.t")
                    .replace(
                        "lang generator generator-closed",
                        "lang arithmetic division-by-zero",
                    ),
                "E-COMPILER-UNSUPPORTED",
            ),
            (
                include_str!("../../../../../examples/language/custom-generator-close-code-pattern.t")
                    .replace(
                        "    Error problem then ()\n    Ok resumed then ()",
                        "    Ok resumed then ()",
                    ),
                "E-UNSUPPORTED-INCOMPLETE-DECISION",
            ),
            (
                include_str!("../../../../../examples/language/custom-generator-close-code-pattern.t")
                    .replace(
                        "    Error problem then ()",
                        "    Error problem then ()\n    Error ( code is lang generator generator-closed ) then ()",
                    ),
                "E-UNREACHABLE-ERROR-CODE-PATTERN",
            ),
        ] {
            assert_eq!(
                analyze_for_compiler(&source).unwrap_err().code,
                expected
            );
        }
}

#[test]
#[allow(clippy::too_many_lines)] // Restored declarations and exact rejection matrix form one contract.
fn models_generator_local_declarations_during_custom_close() {
    // TOPAL-GENERATOR-LOCAL-FUNCTION-001, TOPAL-GENERATOR-LOCAL-ENUM-001,
    // TOPAL-GENERATOR-CLOSE-001, TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-CLOSE-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-local-close-handler.t");
    let program = analyze_for_compiler(source).unwrap();
    let cleanup = program
        .functions
        .iter()
        .find(|function| function.source_name == "cleanup")
        .expect("generator-local cleanup function is retained");
    let abandon = program
        .functions
        .iter()
        .find(|function| function.source_name == "abandon")
        .expect("calling abandon function is instantiated");
    assert!(cleanup.symbol.starts_with("topal.fn.cleanup."));
    assert_eq!(cleanup.result_type, CompilerType::Unit);
    assert!(matches!(
        cleanup.body.result.kind,
        CompilerExpressionKind::Unit
    ));
    assert!(matches!(
        cleanup.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type: CompilerType::Enum(CompilerEnumType {
                name: enum_name,
                alternatives,
            }),
            ..
        }] if name == "choice"
            && enum_name == "CloseChoice"
            && alternatives == &[String::from("Closed"), String::from("Continued")]
    ));

    let [
        CompilerStatement::Binding(CompilerBinding {
            value:
                CompilerExpression {
                    kind:
                        CompilerExpressionKind::CustomCharacterGenerator {
                            close_handler: Some(construction_handler),
                            ..
                        },
                    ..
                },
            ..
        }),
        CompilerStatement::Discard(CompilerExpression {
            kind:
                CompilerExpressionKind::CustomCharacterHandledClose {
                    handler: close_handler,
                    ..
                },
            ..
        }),
    ] = abandon.body.statements.as_slice()
    else {
        panic!("function retains one locally handled custom close")
    };
    assert_eq!(construction_handler, close_handler);
    let [closed_rule] = close_handler.error_codes.as_slice() else {
        panic!("local close handler retains its qualified close branch")
    };
    assert_eq!(closed_rule.code, 0);
    let symbol = cleanup.symbol.as_str();
    assert!(matches!(
        &closed_rule.action.kind,
        CompilerExpressionKind::Call {
            symbol: call_symbol,
            arguments,
        } if call_symbol == symbol
            && matches!(arguments.as_slice(), [CompilerExpression {
                kind: CompilerExpressionKind::Enum(0),
                value_type: CompilerType::Enum(_),
                ..
            }])
    ));
    assert!(matches!(
        close_handler.error_action.kind,
        CompilerExpressionKind::Unit
    ));
    assert!(matches!(
        &close_handler.ok_action.kind,
        CompilerExpressionKind::Call {
            symbol: call_symbol,
            arguments,
        } if call_symbol == symbol
            && matches!(arguments.as_slice(), [CompilerExpression {
                kind: CompilerExpressionKind::Enum(1),
                value_type: CompilerType::Enum(_),
                ..
            }])
    ));

    for malformed in [
        source.replace(
            "CloseChoice is Enum ( Closed, Continued )",
            "CloseChoice is Enum ( Continued, Closed )",
        ),
        source.replace("choice : CloseChoice", "choice : Boolean"),
        source.replace("    ()\n  resume-result", "    choice\n  resume-result"),
        source.replace(
            "resume-result is yield initial",
            "resume-result is yield false",
        ),
        source.replace("then cleanup Closed", "then cleanup Continued"),
        source.replace("Error problem then ()", "Error problem then cleanup Closed"),
        source.replace("then cleanup Continued", "then cleanup Closed"),
        source.replace(
            "handle-close is generator",
            "CloseChoice is Enum ( Closed, Continued )\nhandle-close is generator",
        ),
    ] {
        assert_eq!(
            analyze_for_compiler(&malformed).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Transfer provenance and fail-closed parameter shapes stay one scenario.
fn models_custom_generator_function_parameter_transfer() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-function-parameter.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("called custom Generator consumer is instantiated");
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
            kind: CompilerExpressionKind::CustomCharacterForeach {
                source,
                characters,
                locals,
                parameter,
                result,
                ..
            },
            ..
        })] if matches!(source.kind, CompilerExpressionKind::Local(ref name)
                if name == "generated")
            && characters == &[String::from("T")]
            && locals.is_empty()
            && parameter.value_type == CompilerType::Character
            && matches!(result.kind, CompilerExpressionKind::Unit)
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

    let distinct = analyze_for_compiler(
            "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\nfirst is pause-once \"A\"\n_ is consume first\nsecond is pause-once \"B\"\nconsume second\n",
        )
        .unwrap();
    let consumers = distinct
        .functions
        .iter()
        .filter(|function| function.source_name == "consume")
        .collect::<Vec<_>>();
    assert_eq!(consumers.len(), 2);
    assert_ne!(consumers[0].symbol, consumers[1].symbol);
    let traversals = consumers
        .iter()
        .map(|function| match &function.body.statements[0] {
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach { characters, .. },
                ..
            }) => characters.clone(),
            statement => panic!("expected custom traversal, found {statement:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        traversals,
        [vec![String::from("A")], vec![String::from("B")]]
    );

    for source in [
        "use language (version is v0.1)\npause-twice is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is yield initial\n  ()\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is pause-twice \"T\"\nconsume generated\n",
        "use language (version is v0.1)\ncopy-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  copy : Character is initial\n  _ is yield copy\n  ()\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is copy-once \"T\"\nconsume generated\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nconsume is fn static (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is pause-once \"T\"\nconsume generated\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  _ is ()\n  generated foreach { character }\n    _ is String character\ngenerated is pause-once \"T\"\nconsume generated\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\nconsume (pause-once \"T\")\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let consumed = "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nconsume is fn (generated : Generator Character Unit Unit) -> Unit\n  generated foreach { character }\n    _ is String character\ngenerated is pause-once \"T\"\n_ is consume generated\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(consumed).unwrap_err().code,
        "E-GENERATOR-CONSUMED"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Result-direction provenance and fail-closed transfer shapes stay one scenario.
fn models_custom_generator_character_result_parameter_transfer() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-character-return-parameter.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("called result-valued custom Generator consumer is instantiated");
    assert!(matches!(
        function.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type,
            ..
        }] if name == "generated"
            && is_character_generator_type_with_result(
                value_type,
                &CompilerType::Character
            )
    ));
    assert!(function.body.statements.is_empty());
    assert!(matches!(
        &function.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomCharacterForeach {
                source,
                characters,
                locals,
                parameter,
                result,
                ..
            },
            value_type: CompilerType::Character,
            ..
        } if matches!(source.kind, CompilerExpressionKind::Local(ref name)
                if name == "generated")
            && characters == &[String::from("Y")]
            && locals.is_empty()
            && parameter.value_type == CompilerType::Character
            && exact_string(result).as_deref() == Some("R")
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Call { arguments, .. },
            value_type: CompilerType::Character,
            ..
        } if matches!(arguments.as_slice(), [CompilerExpression {
            kind: CompilerExpressionKind::Local(_),
            value_type,
            ..
        }] if is_character_generator_type_with_result(
            value_type,
            &CompilerType::Character
        ))
    ));

    let distinct = analyze_for_compiler(
            "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nconsume is fn (generated : Generator Character Unit Character) -> Character\n  generated foreach { character }\n    _ is String character\nfirst is yield-return \"A\"\n_ is consume first\nsecond is yield-return \"B\"\nconsume second\n",
        )
        .unwrap();
    let consumers = distinct
        .functions
        .iter()
        .filter(|function| function.source_name == "consume")
        .collect::<Vec<_>>();
    assert_eq!(consumers.len(), 2);
    assert_ne!(consumers[0].symbol, consumers[1].symbol);
    let traversals = consumers
        .iter()
        .map(|function| match &function.body.result.kind {
            CompilerExpressionKind::CustomCharacterForeach {
                characters, result, ..
            } => (characters.clone(), exact_string(result)),
            expression => panic!("expected result-valued traversal, found {expression:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        traversals,
        [
            (vec![String::from("A")], Some(String::from("R"))),
            (vec![String::from("B")], Some(String::from("R"))),
        ]
    );

    for source in [
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nconsume is fn static (generated : Generator Character Unit Character) -> Character\n  generated foreach { character }\n    _ is String character\ngenerated is yield-return \"Y\"\nconsume generated\n",
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nconsume is fn (generated : Generator Character Unit Character) -> Character\n  _ is ()\n  generated foreach { character }\n    _ is String character\ngenerated is yield-return \"Y\"\nconsume generated\n",
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nconsume is fn (generated : Generator Character Unit Character) -> Character\n  _ is generated foreach { character }\n    _ is String character\n  \"Q\"\ngenerated is yield-return \"Y\"\nconsume generated\n",
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nconsume is fn (generated : Generator Character Unit Character) -> Character\n  generated foreach { character }\n    _ is String character\nconsume (yield-return \"Y\")\n",
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nignore is fn (generated : Generator Character Unit Character) -> Unit\n  ()\ngenerated is yield-return \"Y\"\nignore generated\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let consumed = "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nconsume is fn (generated : Generator Character Unit Character) -> Character\n  generated foreach { character }\n    _ is String character\ngenerated is yield-return \"Y\"\n_ is consume generated\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(consumed).unwrap_err().code,
        "E-GENERATOR-CONSUMED"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Close provenance and fail-closed transfer shapes stay one scenario.
fn models_custom_generator_parameter_close_transfer() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-CLOSE-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-CLOSE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-parameter-close.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "ignore")
        .expect("called custom Generator closer is instantiated");
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
            kind: CompilerExpressionKind::CustomCharacterClose {
                generator,
                provenance,
                close_domain,
            },
            value_type: CompilerType::Unit,
            ..
        })] if matches!(generator.kind, CompilerExpressionKind::Local(ref name)
                if name == "generated")
            && matches!(
                provenance.kind,
                CompilerExpressionKind::CustomCharacterGenerator {
                    ref declaration,
                    ref characters,
                    ref locals,
                    close_handler: None,
                    ref result,
                    ..
                } if declaration == "pause-once"
                    && characters == &[String::from("T")]
                    && locals.is_empty()
                    && matches!(result.kind, CompilerExpressionKind::Unit)
            )
            && close_domain == "root"
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

    let distinct = analyze_for_compiler(
            "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\nfirst is pause-once \"A\"\n_ is ignore first\nsecond is pause-once \"B\"\nignore second\n",
        )
        .unwrap();
    let closers = distinct
        .functions
        .iter()
        .filter(|function| function.source_name == "ignore")
        .collect::<Vec<_>>();
    assert_eq!(closers.len(), 2);
    assert_ne!(closers[0].symbol, closers[1].symbol);
    let closed_characters = closers
        .iter()
        .map(|function| match &function.body.statements[0] {
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterClose { provenance, .. },
                ..
            }) => match &provenance.kind {
                CompilerExpressionKind::CustomCharacterGenerator { characters, .. } => {
                    characters.clone()
                }
                expression => panic!("expected custom provenance, found {expression:?}"),
            },
            statement => panic!("expected custom close, found {statement:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        closed_characters,
        [vec![String::from("A")], vec![String::from("B")]]
    );

    for source in [
        "use language (version is v0.1)\npause-twice is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is yield initial\n  ()\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\ngenerated is pause-twice \"T\"\nignore generated\n",
        "use language (version is v0.1)\ncopy-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  copy : Character is initial\n  _ is yield copy\n  ()\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\ngenerated is copy-once \"T\"\nignore generated\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nignore is fn static (generated : Generator Character Unit Unit) -> Unit\n  ()\ngenerated is pause-once \"T\"\nignore generated\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  _ is ()\n  ()\ngenerated is pause-once \"T\"\nignore generated\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nignore is fn (generated : Generator Character Unit Unit) -> Character\n  \"T\"\ngenerated is pause-once \"T\"\n_ is ignore generated\n()\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\nignore (pause-once \"T\")\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let consumed = "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nignore is fn (generated : Generator Character Unit Unit) -> Unit\n  ()\ngenerated is pause-once \"T\"\n_ is ignore generated\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(consumed).unwrap_err().code,
        "E-GENERATOR-CONSUMED"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Transfer provenance and fail-closed factory shapes stay one scenario.
fn models_custom_generator_function_result_transfer() {
    // TOPAL-GENERATOR-FUNCTION-RESULT-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-function-result.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("called custom Generator factory is instantiated");
    assert!(matches!(
        function.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type: CompilerType::Character,
            ..
        }] if name == "initial"
    ));
    assert!(is_character_unit_generator_type(&function.result_type));
    assert!(function.body.statements.is_empty());
    assert!(matches!(
        &function.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomCharacterGenerator {
                declaration,
                initial,
                characters,
                locals,
                close_handler: None,
                result,
                ..
            },
            ..
        } if declaration == "pause-once"
            && matches!(initial.kind, CompilerExpressionKind::Local(ref name) if name == "initial")
            && characters == &["T"]
            && locals.is_empty()
            && matches!(result.kind, CompilerExpressionKind::Unit)
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
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    characters,
                    ..
                },
                ..
            })
        ] if is_character_unit_generator_type(value_type) && characters == &["T"]
    ));

    let distinct = analyze_for_compiler(
            "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nmake is fn (initial : Character) -> Generator Character Unit Unit\n  pause-once initial\nfirst is make \"A\"\nfirst foreach { character }\n  _ is String character\nsecond is make \"B\"\nsecond foreach { character }\n  _ is String character\n",
        )
        .unwrap();
    let traversals = distinct
        .main
        .statements
        .iter()
        .filter_map(|statement| match statement {
            CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach { characters, .. },
                ..
            }) => Some(characters.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        traversals,
        [vec![String::from("A")], vec![String::from("B")]]
    );
    let factory_symbols = distinct
        .functions
        .iter()
        .filter(|function| function.source_name == "make")
        .map(|function| function.symbol.as_str())
        .collect::<Vec<_>>();
    assert_eq!(factory_symbols.len(), 2);
    assert_ne!(factory_symbols[0], factory_symbols[1]);

    for source in [
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nmake is fn static (initial : Character) -> Generator Character Unit Unit\n  pause-once initial\ngenerated is make \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nmake is fn (initial : Character, ignored : Int) -> Generator Character Unit Unit\n  pause-once initial\ngenerated is make (\"T\", 0)\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nmake is fn (initial : Character) -> Generator Character Unit Unit\n  pause-once \"T\"\ngenerated is make \"U\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nmake is fn (initial : Character) -> Generator Character Unit Unit\n  _ is ()\n  pause-once initial\ngenerated is make \"T\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
    let unbound_result = analyze_for_compiler(
            "use language (version is v0.1)\npause-once is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nmake is fn (initial : Character) -> Generator Character Unit Unit\n  pause-once initial\nmake \"T\"\n",
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
#[allow(clippy::too_many_lines)] // Final-direction provenance and fail-closed factory shapes stay one scenario.
fn models_custom_generator_character_result_function_result_transfer() {
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-character-return-result.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("called result-valued custom Generator factory is instantiated");
    assert!(matches!(
        function.parameters.as_slice(),
        [CompilerParameter {
            name,
            value_type: CompilerType::Character,
            ..
        }] if name == "initial"
    ));
    assert!(is_character_generator_type_with_result(
        &function.result_type,
        &CompilerType::Character
    ));
    assert!(function.body.statements.is_empty());
    assert!(matches!(
        &function.body.result,
        CompilerExpression {
            kind: CompilerExpressionKind::CustomCharacterGenerator {
                declaration,
                initial,
                characters,
                locals,
                close_handler: None,
                result,
                ..
            },
            value_type,
            ..
        } if declaration == "yield-return"
            && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                if name == "initial")
            && characters == &[String::from("Y")]
            && locals.is_empty()
            && exact_string(result).as_deref() == Some("R")
            && is_character_generator_type_with_result(
                value_type,
                &CompilerType::Character
            )
    ));
    assert!(matches!(
        (program.main.statements.as_slice(), &program.main.result),
        (
            [CompilerStatement::Binding(CompilerBinding {
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Call { .. },
                    value_type,
                    ..
                },
                ..
            })],
            CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    source,
                    characters,
                    result,
                    ..
                },
                value_type: CompilerType::Character,
                ..
            }
        ) if is_character_generator_type_with_result(
                value_type,
                &CompilerType::Character
            )
            && matches!(source.kind, CompilerExpressionKind::Local(_))
            && characters == &[String::from("Y")]
            && exact_string(result).as_deref() == Some("R")
    ));

    for source in [
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nmake is fn static (initial : Character) -> Generator Character Unit Character\n  yield-return initial\ngenerated is make \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nmake is fn (initial : Character, ignored : Int) -> Generator Character Unit Character\n  yield-return initial\ngenerated is make (\"Y\", 0)\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nmake is fn (initial : Character) -> Generator Character Unit Character\n  yield-return \"Q\"\ngenerated is make \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nmake is fn (initial : Character) -> Generator Character Unit Character\n  _ is ()\n  yield-return initial\ngenerated is make \"Y\"\ngenerated foreach { character }\n  _ is String character\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let unbound_result = analyze_for_compiler(
            "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nmake is fn (initial : Character) -> Generator Character Unit Character\n  yield-return initial\nmake \"Y\"\n",
        )
        .unwrap_err();
    assert_eq!(unbound_result.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        unbound_result
            .message
            .contains("unbound returned Generator and close delivery")
    );

    let consumed = "use language (version is v0.1)\nyield-return is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\nmake is fn (initial : Character) -> Generator Character Unit Character\n  yield-return initial\ngenerated is make \"Y\"\n_ is generated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(consumed).unwrap_err().code,
        "E-COMPILER-UNSUPPORTED"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Pre-yield state and the fail-closed String-input shapes form one scenario.
fn models_custom_generator_string_initial_input() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-GENERATOR-STRING-INPUT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-string-input.t"
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
                        initial_parameter,
                        initial,
                        prefix,
                        characters,
                        locals,
                        result,
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
                    parameter,
                    body,
                    ..
                },
                value_type: CompilerType::Unit,
                ..
            })
        ] if name == "generated"
            && declaration == "from-text"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::String
            && exact_string(initial).as_deref() == Some("Topal")
            && matches!(
                prefix.as_ref(),
                CompilerBlock {
                    statements,
                    result: CompilerExpression {
                        kind: CompilerExpressionKind::Unit,
                        ..
                    },
                } if matches!(
                    statements.as_slice(),
                    [CompilerStatement::Binding(CompilerBinding {
                        name,
                        value: CompilerExpression {
                            kind: CompilerExpressionKind::StringEmptyPredicate(operand),
                            value_type: CompilerType::Boolean,
                            ..
                        },
                        ..
                    })] if name == "initial-is-empty"
                        && matches!(operand.kind, CompilerExpressionKind::Local(ref name)
                            if name == "initial")
                )
            )
            && characters == &[String::from("T")]
            && locals.is_empty()
            && matches!(result.kind, CompilerExpressionKind::Unit)
            && is_character_unit_generator_type(value_type)
            && yielded == &[String::from("T")]
            && parameter.name == "character"
            && parameter.value_type == CompilerType::Character
            && body.result.value_type == CompilerType::Unit
    ));

    let dynamic = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\nfrom-text is generator (initial : String)\n  yields Character\n  resumes Unit\n  -> Unit\n  tested : Boolean is empty? initial\n  _ is yield \"T\"\n  ()\ngenerated is from-text (identity \"Topal\")\ngenerated foreach { character }\n  _ is String character\n",
        )
        .unwrap();
    assert!(matches!(
        &dynamic.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterGenerator { initial, .. },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::Call { .. })
    ));

    for source in [
        "use language (version is v0.1)\nfrom-text is generator (initial : String)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield \"T\"\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nfrom-text is generator (initial : String)\n  yields Character\n  resumes Unit\n  -> Unit\n  tested : Int is empty? initial\n  _ is yield \"T\"\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nfrom-text is generator (initial : String)\n  yields Character\n  resumes Unit\n  -> Unit\n  tested : Boolean is empty? \"\"\n  _ is yield \"T\"\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nfrom-text is generator (initial : String)\n  yields Character\n  resumes Unit\n  -> Unit\n  tested : Boolean is empty? initial\n  _ is yield initial\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\n",
        "use language (version is v0.1)\nfrom-text is generator (initial : String)\n  yields Character\n  resumes Unit\n  -> Unit\n  tested : Boolean is empty? initial\n  _ is yield \"T\"\n  ()\nmake is fn (text : String) -> Unit\n  generated is from-text text\n  generated foreach { character }\n    _ is String character\nmake \"Topal\"\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let consumed_twice = "use language (version is v0.1)\nfrom-text is generator (initial : String)\n  yields Character\n  resumes Unit\n  -> Unit\n  tested : Boolean is empty? initial\n  _ is yield \"T\"\n  ()\ngenerated is from-text \"Topal\"\ngenerated foreach { character }\n  _ is String character\ngenerated foreach { character }\n  _ is String character\n";
    assert_eq!(
        analyze_for_compiler(consumed_twice).unwrap_err().code,
        "E-GENERATOR-CONSUMED"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Ordered yields and fail-closed declaration shapes form one scenario.
fn models_custom_generator_string_yields() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-GENERATOR-STRING-YIELD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-string-yield.t"
    ))
    .unwrap();
    let statements = &program.main.statements;
    assert!(
        matches!(
            statements.as_slice(),
            [
                CompilerStatement::Binding(CompilerBinding {
                    name,
                    value: CompilerExpression {
                        kind: CompilerExpressionKind::CustomValueGenerator {
                            declaration,
                            initial_parameter,
                            initial,
                            yields,
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
                        yields: traversed,
                        parameter,
                        body,
                        result: traversal_result,
                        ..
                    },
                    value_type: CompilerType::Unit,
                    ..
                })
            ] if name == "generated"
                && declaration == "texts"
                && initial_parameter.name == "initial"
                && initial_parameter.value_type == CompilerType::String
                && exact_string(initial).as_deref() == Some("Topal")
                && matches!(
                    yields.as_slice(),
                    [
                        CompilerGeneratorYield::Initial(_),
                        CompilerGeneratorYield::Value(value)
                    ] if matches!(
                        value.as_ref(),
                        CompilerExpression {
                            kind: CompilerExpressionKind::String(value),
                            value_type: CompilerType::String,
                            ..
                        } if value.is_empty()
                    )
                )
                && traversed == yields
                && matches!(result.kind, CompilerExpressionKind::Unit)
                && matches!(traversal_result.kind, CompilerExpressionKind::Unit)
                && *generator_type.yield_type == CompilerType::String
                && *generator_type.resume_type == CompilerType::Unit
                && *generator_type.result_type == CompilerType::Unit
                && parameter.name == "text"
                && parameter.value_type == CompilerType::String
                && matches!(
                    body.statements.as_slice(),
                    [CompilerStatement::Discard(CompilerExpression {
                        kind: CompilerExpressionKind::StringEmptyPredicate(value),
                        value_type: CompilerType::Boolean,
                        ..
                    })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                        if name == "text")
                )
                && matches!(body.result.kind, CompilerExpressionKind::Unit)
        ),
        "{statements:#?}"
    );

    let dynamic = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ntexts is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is yield \"\"\n  ()\ngenerated is texts (identity \"Topal\")\ngenerated foreach { text }\n  _ is empty? text\n",
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
        }) if matches!(initial.kind, CompilerExpressionKind::Call { .. })
    ));

    for source in [
        "use language (version is v0.1)\ntexts is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  ()\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ntexts is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield (identity initial)\n  ()\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ntexts is generator (initial : String)\n  yields String\n  resumes Int\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ntexts is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  initial\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ntexts is generator (_ : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield \"Topal\"\n  ()\ngenerated is texts \"ignored\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ntexts is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\nconsume is fn (text : String) -> Unit\n  generated is texts text\n  generated foreach { yielded }\n    _ is empty? yielded\nconsume \"Topal\"\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }

    let consumed_twice = "use language (version is v0.1)\ntexts is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is texts \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\ngenerated foreach { text }\n  _ is empty? text\n";
    let consumed_twice = analyze_for_compiler(consumed_twice).unwrap_err();
    assert_eq!(
        consumed_twice.code, "E-GENERATOR-CONSUMED",
        "{consumed_twice:?}"
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Final-value provenance and its fail-closed forms are one scenario.
fn models_custom_generator_final_string() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-STRING-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-string-return.t"
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
                    result,
                    ..
                },
                value_type: CompilerType::Generator(generator_type),
                ..
            },
            ..
        })] if name == "generated"
            && declaration == "text-result"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::String
            && exact_string(initial).as_deref() == Some("item")
            && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
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
                parameter,
                body,
                result,
                ..
            },
            value_type: CompilerType::String,
            ..
        } if matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
            && parameter.name == "text"
            && parameter.value_type == CompilerType::String
            && matches!(
                body.statements.as_slice(),
                [CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::StringEmptyPredicate(value),
                    value_type: CompilerType::Boolean,
                    ..
                })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                    if name == "text")
            )
            && exact_string(result).as_deref() == Some("done")
    ));

    let dynamic = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ntext-result is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  \"done\"\ngenerated is text-result (identity \"item\")\ngenerated foreach { text }\n  _ is empty? text\n",
        )
        .unwrap();
    assert!(matches!(
        &dynamic.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    initial, result, ..
                },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::Call { .. })
            && exact_string(result).as_deref() == Some("done")
    ));

    let multiple = analyze_for_compiler(
            "use language (version is v0.1)\ntext-result is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  _ is yield \"more\"\n  \"done\"\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
        )
        .unwrap();
    assert!(matches!(
        &multiple.main.result.kind,
        CompilerExpressionKind::CustomValueForeach { yields, result, .. }
            if yields.len() == 2 && exact_string(result).as_deref() == Some("done")
    ));

    for source in [
        "use language (version is v0.1)\ntext-result is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  ()\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ntext-result is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  identity \"done\"\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ntext-result is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  \"done\"\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ntext-result is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Character\n  _ is yield initial\n  \"R\"\ngenerated is text-result \"item\"\ngenerated foreach { text }\n  _ is empty? text\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Resume placement and its fail-closed forms are one scenario.
fn models_discarded_computation_between_string_yields() {
    // TOPAL-GENERATOR-BODY-STATEMENT-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-RESUME-DISCARD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/custom-generator-discard-between-yields.t"
    ))
    .unwrap();
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            name,
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    initial_parameter,
                    initial,
                    yields,
                    continuations,
                    result,
                    ..
                },
                value_type: CompilerType::Generator(generator_type),
                ..
            },
            ..
        }), CompilerStatement::Discard(_)] if name == "generated"
            && initial_parameter.name == "initial"
            && initial_parameter.value_type == CompilerType::String
            && exact_string(initial).as_deref() == Some("Topal")
            && matches!(
                yields.as_slice(),
                [
                    CompilerGeneratorYield::Initial(_),
                    CompilerGeneratorYield::Value(value),
                ] if exact_string(value).as_deref() == Some("")
            )
            && matches!(
                continuations.as_slice(),
                [CompilerGeneratorContinuation {
                    after_resumptions: 1,
                    body: CompilerBlock {
                        statements,
                        result: continuation_result,
                    },
                }] if matches!(
                    statements.as_slice(),
                    [CompilerStatement::Discard(CompilerExpression {
                        kind: CompilerExpressionKind::StringEmptyPredicate(value),
                        value_type: CompilerType::Boolean,
                        ..
                    })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                        if name == "initial")
                ) && continuation_result.value_type == CompilerType::Unit
            )
            && result.value_type == CompilerType::Unit
            && *generator_type.yield_type == CompilerType::String
            && *generator_type.resume_type == CompilerType::Unit
            && *generator_type.result_type == CompilerType::Unit
    ));
    assert!(matches!(
        &program.main.statements[1],
        CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::CustomValueForeach {
                initial_parameter,
                yields,
                continuations,
                result,
                ..
            },
            value_type: CompilerType::Unit,
            ..
        }) if initial_parameter.name == "initial"
            && yields.len() == 2
            && matches!(
                continuations.as_slice(),
                [CompilerGeneratorContinuation {
                    after_resumptions: 1,
                    ..
                }]
            )
            && result.value_type == CompilerType::Unit
    ));

    let dynamic = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is empty? initial\n  _ is yield \"\"\n  ()\ngenerated is inspect-between (identity \"Topal\")\ngenerated foreach { text }\n  _ is empty? text\n",
        )
        .unwrap();
    assert!(matches!(
        &dynamic.main.statements[0],
        CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    initial, continuations, ..
                },
                ..
            },
            ..
        }) if matches!(initial.kind, CompilerExpressionKind::Call { .. })
            && continuations.len() == 1
    ));

    for source in [
        "use language (version is v0.1)\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is empty? initial\n  _ is yield initial\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is yield \"\"\n  _ is empty? initial\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is empty? initial\n  _ is empty? initial\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  observed is empty? initial\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is empty? \"Topal\"\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\nidentity is fn (text : String) -> String\n  text\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  _ is empty? (identity initial)\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
        "use language (version is v0.1)\ninspect-between is generator (initial : String)\n  yields String\n  resumes Unit\n  -> String\n  _ is yield initial\n  _ is empty? initial\n  _ is yield \"\"\n  \"done\"\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}
