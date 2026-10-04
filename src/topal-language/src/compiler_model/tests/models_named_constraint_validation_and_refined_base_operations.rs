#[test]
fn models_named_constraint_validation_and_refined_base_operations() {
    // TOPAL-TYPE-CONSTRAINT-001, TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
    // TOPAL-COMPILER-CONSTRAINT-VALIDATE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/constraints-and-derived-capabilities.t"
    ))
    .unwrap();
    let refined = CompilerType::Refined {
        constraint: "Positive".into(),
        base: Box::new(CompilerType::Int),
    };
    for statement in &program.main.statements[1..=2] {
        let CompilerStatement::Binding(binding) = statement else {
            panic!("expected a refined binding")
        };
        assert_eq!(binding.value.value_type, refined);
        assert!(matches!(binding.value.kind, CompilerExpressionKind::Int(_)));
    }
    let validate = program
        .functions
        .iter()
        .find(|function| function.source_name == "validate")
        .unwrap();
    assert_eq!(
        validate.result_type,
        CompilerType::Result(Box::new(CompilerType::Int))
    );
    assert!(matches!(
        validate.body.result.kind,
        CompilerExpressionKind::Validate {
            operation: CompilerValidation::Constraint(0),
            ..
        }
    ));
    let CompilerExpressionKind::Tuple(observations) = &program.main.result.kind else {
        panic!("expected constraint observations")
    };
    assert_eq!(observations[0].value_type, refined);
    assert_eq!(observations[1].value_type, CompilerType::Boolean);
    assert_eq!(observations[2].value_type, CompilerType::Boolean);
    assert_eq!(observations[3].value_type, CompilerType::Int);
    assert_eq!(
        observations[4].value_type,
        CompilerType::Result(Box::new(CompilerType::Int))
    );

    let rejected = "use language (version is v0.1)\nPositive is Int constraint { value } value > 0\nPositive 0\n";
    assert_eq!(
        analyze_for_compiler(rejected).unwrap_err().code,
        "E-CONSTRAINT-REJECTED"
    );
    let wrong_base = "use language (version is v0.1)\nPositive is Int constraint { value } value > 0\nPositive \"one\"\n";
    assert_eq!(
        analyze_for_compiler(wrong_base).unwrap_err().code,
        "E-TYPE-MISMATCH"
    );

    let boolean = analyze_for_compiler(include_str!(
        "../../../../../tests/standard-library/harness.t"
    ))
    .unwrap();
    assert_eq!(boolean.constraints[0].base_type, CompilerType::Boolean);
    assert_eq!(
        boolean.main.result.value_type,
        CompilerType::Refined {
            constraint: "Pass".into(),
            base: Box::new(CompilerType::Boolean),
        }
    );

    let string = analyze_for_compiler(
            "use language (version is v0.1)\nNonempty is String constraint { value } value != \"\"\nname : Nonempty is Nonempty \"Topal\"\nname\n",
        )
        .unwrap();
    assert_eq!(string.constraints[0].base_type, CompilerType::String);
    assert_eq!(string.main.result.value_type.name(), "Nonempty");
}

#[test]
fn models_the_executable_root_namespace_and_qualified_function() {
    // TOPAL-COMPILER-ROOT-NAMESPACE-001, TOPAL-NAMESPACE-ROOT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/root-namespace.t"
    ))
    .unwrap();
    assert_eq!(program.functions.len(), 1);
    let CompilerExpressionKind::Tuple(fields) = &program.main.result.kind else {
        panic!("expected the root namespace observation product")
    };
    assert!(matches!(fields[0].kind, CompilerExpressionKind::Root));
    assert_eq!(fields[0].value_type, CompilerType::Scope);
    assert!(matches!(
        fields[1].kind,
        CompilerExpressionKind::Call { .. }
    ));
    assert_eq!(exact_int(&fields[1]), Some(BigInt::from(42)));

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\n{\n  increment is 100\n  root increment 41\n}\n",
        )
        .unwrap();
    assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));

    for (source, expected) in [
        (
            "use language (version is v0.1)\nroot is 1\n",
            "E-DUPLICATE-BINDING",
        ),
        (
            "use language (version is v0.1)\nroot is fn () -> Int\n  1\n",
            "E-DUPLICATE-BINDING",
        ),
        (
            "use language (version is v0.1)\nKind is Enum (root)\nKind\n",
            "E-COMPILER-UNSUPPORTED",
        ),
    ] {
        assert_eq!(analyze_for_compiler(source).unwrap_err().code, expected);
    }
}

#[test]
fn models_use_of_root_and_root_alias_namespaces() {
    // TOPAL-COMPILER-NAMESPACE-USE-001, TOPAL-NAMESPACE-USE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/use-namespace.t"
    ))
    .unwrap();
    assert!(matches!(
        program.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::Root,
                value_type: CompilerType::Scope,
                ..
            },
            ..
        })]
    ));
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

    let alias = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\napi is root\ncurrent is use api\ncurrent increment 41\n",
        )
        .unwrap();
    assert_eq!(exact_int(&alias.main.result), Some(BigInt::from(42)));

    let data = analyze_for_compiler(
        "use language (version is v0.1)\nanswer is 42\ncurrent is use root\ncurrent answer\n",
    )
    .unwrap();
    assert_eq!(exact_int(&data.main.result), Some(BigInt::from(42)));

    let stale = analyze_for_compiler(
        "use language (version is v0.1)\ncurrent is use root\nlater is 42\ncurrent later\n",
    )
    .unwrap_err();
    assert_eq!(stale.code, "E-COMPILER-UNSUPPORTED");

    let non_namespace =
        analyze_for_compiler("use language (version is v0.1)\nuse 42\n").unwrap_err();
    assert_eq!(non_namespace.code, "E-USE-NON-NAMESPACE");

    let function_body = analyze_for_compiler(
        "use language (version is v0.1)\nprobe is fn () -> Scope\n  use root\nprobe ()\n",
    )
    .unwrap_err();
    assert_eq!(function_body.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_function_namespace_aliases_chains_and_snapshots() {
    // TOPAL-COMPILER-NAMESPACE-FUNCTION-ALIAS-001,
    // TOPAL-NAMESPACE-ALIAS-001, TOPAL-NAMESPACE-SNAPSHOT-001,
    // TOPAL-NAMESPACE-OVERLOAD-001, TOPAL-NAMESPACE-CLASSIFIER-001,
    // TOPAL-NAMESPACE-ALIAS-CHAIN-001
    let alias = analyze_for_compiler(include_str!(
        "../../../../../examples/language/namespace-alias.t"
    ))
    .unwrap();
    assert!(matches!(
        alias.main.statements.as_slice(),
        [CompilerStatement::Binding(CompilerBinding {
            value: CompilerExpression {
                kind: CompilerExpressionKind::Root,
                value_type: CompilerType::Scope,
                ..
            },
            ..
        })]
    ));
    assert!(matches!(
        alias.main.result.kind,
        CompilerExpressionKind::Call { .. }
    ));

    let overloads = analyze_for_compiler(include_str!(
        "../../../../../examples/language/namespace-overloads.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(values) = &overloads.main.result.kind else {
        panic!("expected qualified overload result product")
    };
    assert_eq!(values.len(), 2);
    assert!(
        values
            .iter()
            .all(|value| matches!(value.kind, CompilerExpressionKind::Call { .. }))
    );

    let chained = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nfirst is root\nsecond : Scope is first\nsecond increment 41\n",
        )
        .unwrap();
    assert_eq!(exact_int(&chained.main.result), Some(BigInt::from(42)));

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\napi is root\n{\n  increment is 100\n  api increment 41\n}\n",
        )
        .unwrap();
    assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));

    let snapshot = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\napi is root\nidentity is fn (value : String) -> String\n  value\n(api identity 42, root identity \"Topal\")\n",
        )
        .unwrap();
    let CompilerExpressionKind::Tuple(values) = &snapshot.main.result.kind else {
        panic!("expected snapshot and live-root result product")
    };
    assert_eq!(values[0].value_type, CompilerType::Int);
    assert_eq!(values[1].value_type, CompilerType::String);

    let rejected = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int\n  value\napi is root\nidentity is fn (value : String) -> String\n  value\napi identity \"Topal\"\n",
        )
        .unwrap_err();
    assert_eq!(rejected.code, "E-NO-APPLICABLE-OVERLOAD");
}

#[test]
fn models_namespace_qualified_generator_application() {
    // TOPAL-COMPILER-NAMESPACE-GENERATOR-001,
    // TOPAL-NAMESPACE-GENERATOR-001, TOPAL-NAMESPACE-SNAPSHOT-001,
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/namespace-generator.t"
    ))
    .unwrap();
    assert!(matches!(
        program.main.statements.as_slice(),
        [
            CompilerStatement::Binding(CompilerBinding {
                name: api,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::Root,
                    value_type: CompilerType::Scope,
                    ..
                },
                ..
            }),
            CompilerStatement::Binding(CompilerBinding {
                name: generated,
                value: CompilerExpression {
                    kind: CompilerExpressionKind::CustomCharacterGenerator {
                        declaration,
                        declaration_namespace,
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
                    ..
                },
                ..
            })
        ] if api == "api"
            && generated == "generated"
            && declaration == "once"
            && declaration_namespace == "root"
            && characters == &[String::from("T")]
            && is_character_unit_generator_type(value_type)
            && matches!(source.kind, CompilerExpressionKind::Local(_))
            && yielded == &[String::from("T")]
    ));

    let direct = analyze_for_compiler(
            "use language (version is v0.1)\nonce is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is root once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        )
        .unwrap();
    let CompilerStatement::Binding(CompilerBinding { value, .. }) = &direct.main.statements[0]
    else {
        panic!("expected a directly qualified generator binding")
    };
    assert!(matches!(
        &value.kind,
        CompilerExpressionKind::CustomCharacterGenerator {
            declaration_namespace,
            ..
        } if declaration_namespace == "root"
    ));

    let qualified_overloads =
        include_str!("../../../../../examples/language/custom-generator-overloads.t")
            .replace(
                "unary-generated is select 7",
                "api is root\nunary-generated is api select 7",
            )
            .replace(
                "binary-generated is select (7, \"item\")",
                "binary-generated is api select (7, \"item\")",
            );
    let overloads = analyze_for_compiler(&qualified_overloads).unwrap();
    let selected_namespaces = overloads
        .main
        .statements
        .iter()
        .filter_map(|statement| {
            let CompilerStatement::Binding(CompilerBinding { value, .. }) = statement else {
                return None;
            };
            let CompilerExpressionKind::CustomValueGenerator {
                declaration_namespace,
                ..
            } = &value.kind
            else {
                return None;
            };
            Some(declaration_namespace.as_str())
        })
        .collect::<Vec<_>>();
    assert_eq!(selected_namespaces, ["root", "root"]);

    let stale = analyze_for_compiler(
            "use language (version is v0.1)\napi is root\nonce is generator (initial : Character)\n  yields Character\n  resumes Unit\n  -> Unit\n  _ is yield initial\n  ()\ngenerated is api once \"T\"\ngenerated foreach { character }\n  _ is String character\n",
        )
        .unwrap_err();
    assert_eq!(stale.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_root_and_alias_data_member_snapshots() {
    // TOPAL-COMPILER-NAMESPACE-DATA-001, TOPAL-NAMESPACE-ROOT-001,
    // TOPAL-NAMESPACE-ALIAS-001, TOPAL-NAMESPACE-SNAPSHOT-001,
    // TOPAL-NAMESPACE-CLASSIFIER-001, TOPAL-NAMESPACE-ALIAS-CHAIN-001
    for source in [
        include_str!("../../../../../examples/language/namespace-alias-chain.t"),
        include_str!("../../../../../examples/language/namespace-snapshot.t"),
        include_str!("../../../../../examples/language/scope-classifier.t"),
        include_str!("../../../../../examples/language/published-root-member.t"),
    ] {
        analyze_for_compiler(source).unwrap();
    }

    let snapshot = analyze_for_compiler(include_str!(
        "../../../../../examples/language/namespace-snapshot.t"
    ))
    .unwrap();
    let CompilerExpressionKind::Tuple(values) = &snapshot.main.result.kind else {
        panic!("expected earlier-alias and live-root data product")
    };
    assert_eq!(exact_int(&values[0]), Some(BigInt::from(41)));
    assert_eq!(exact_int(&values[1]), Some(BigInt::from(42)));

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\napi is root\n{\n  answer is 0\n  (root answer, api answer, answer)\n}\n",
        )
        .unwrap();
    let CompilerExpressionKind::Block(block) = &shadowed.main.result.kind else {
        panic!("expected lexical shadow block")
    };
    let CompilerExpressionKind::Tuple(values) = &block.result.kind else {
        panic!("expected qualified and lexical result product")
    };
    assert_eq!(exact_int(&values[0]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&values[1]), Some(BigInt::from(42)));
    assert_eq!(exact_int(&values[2]), Some(BigInt::from(0)));
    let CompilerExpressionKind::Local(root_storage) = &values[0].kind else {
        panic!("expected stable root storage reference")
    };
    let CompilerExpressionKind::Local(local_storage) = &values[2].kind else {
        panic!("expected lexical storage reference")
    };
    assert_ne!(root_storage, local_storage);

    let rejected = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\napi is root\nread is fn () -> Int\n  api answer\nread ()\n",
        )
        .unwrap_err();
    assert_eq!(rejected.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_scope_parameters_as_specialized_private_environments() {
    // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
    // TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001
    let program = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 42\nincrement is fn (value : Int) -> Int\n  value + 1\nread-answer is fn (api : Scope) -> Int\n  api answer\napply is fn (api : Scope, value : Int) -> Int\n  api increment value\nforward is fn (api : Scope) -> Int\n  read-answer api\n(read-answer root, apply root 41, forward root)\n",
        )
        .unwrap();
    let CompilerExpressionKind::Tuple(values) = &program.main.result.kind else {
        panic!("expected Scope-boundary result product")
    };
    assert!(
        values
            .iter()
            .all(|value| exact_int(value) == Some(BigInt::from(42)))
    );
    let forward = program
        .functions
        .iter()
        .find(|function| function.source_name == "forward")
        .expect("forwarding Scope specialization exists");
    assert_eq!(forward.parameters.len(), 2);
    assert_eq!(forward.parameters[0].value_type, CompilerType::Scope);
    assert_eq!(forward.parameters[1].name, "api answer");
    assert_eq!(forward.parameters[1].value_type, CompilerType::Int);
    let CompilerExpressionKind::Call { arguments, .. } = &forward.body.result.kind else {
        panic!("forwarding body retains a direct private call")
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(name) if name == "api answer"
    ));

    let stale = analyze_for_compiler(
            "use language (version is v0.1)\napi is root\nanswer is 42\nread-answer is fn (scope : Scope) -> Int\n  scope answer\nread-answer api\n",
        )
        .unwrap_err();
    assert_eq!(stale.code, "E-COMPILER-UNSUPPORTED");

    let unsupported_member = analyze_for_compiler(
            "use language (version is v0.1)\nnested is root\nread is fn (api : Scope) -> Int\n  api nested\nread root\n",
        )
        .unwrap_err();
    assert_eq!(unsupported_member.code, "E-COMPILER-UNSUPPORTED");

    let local_root = analyze_for_compiler(
            "use language (version is v0.1)\naccept is fn (_ : Scope) -> Int\n  1\nwrapper is fn () -> Int\n  accept root\nwrapper ()\n",
        )
        .unwrap_err();
    assert_eq!(local_root.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_private_scalar_defining_context_capture() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-001, TOPAL-CONTEXT-SELECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/constructed-context.t"
    ))
    .unwrap();
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));
    let function = &program.functions[0];
    assert_eq!(function.parameters.len(), 2);
    assert_eq!(function.parameters[0].name, "value");
    assert_eq!(function.parameters[1].name, "@ offset");
    let CompilerExpressionKind::Call { arguments, .. } = &program.main.result.kind else {
        panic!("expected direct context-capturing call")
    };
    assert_eq!(arguments.len(), 2);
    assert_eq!(exact_int(&arguments[0]), Some(BigInt::from(2)));
    assert_eq!(exact_int(&arguments[1]), Some(BigInt::from(40)));
    let [CompilerStatement::Binding(root_binding)] = program.main.statements.as_slice() else {
        panic!("expected one defining-context root binding")
    };
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == &root_binding.storage_name
    ));

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nadd-offset is fn (value : Int) -> Int\n  value + @ offset\n{\n  offset is 100\n  add-offset 2\n}\n",
        )
        .unwrap();
    assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));

    let later = analyze_for_compiler(
            "use language (version is v0.1)\nadd-offset is fn (value : Int) -> Int\n  value + @ offset\noffset is 40\nadd-offset 2\n",
        )
        .unwrap_err();
    assert_eq!(later.code, "E-COMPILER-UNSUPPORTED");

    let outside = analyze_for_compiler("use language (version is v0.1)\noffset is 40\n@ offset\n")
        .unwrap_err();
    assert_eq!(outside.code, "E-CONTEXT-SELECTION");
}

#[test]
fn models_private_scalar_defining_context_forwarding() {
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-001, TOPAL-CONTEXT-SELECT-001
    let forwarded = analyze_for_compiler(include_str!(
        "../../../../../examples/language/defining-context-forwarding.t"
    ))
    .unwrap();
    for name in ["read", "relay", "forward"] {
        let function = forwarded
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(function.parameters.len(), 3);
        assert_eq!(function.parameters[0].name, "offset");
        assert_eq!(function.parameters[1].name, "@ offset");
        assert_eq!(function.parameters[2].name, "@ label");
    }
    let relay = forwarded
        .functions
        .iter()
        .find(|function| function.source_name == "relay")
        .unwrap();
    let CompilerExpressionKind::Call { arguments, .. } = &relay.body.result.kind else {
        panic!("relay retains its direct forwarded call")
    };
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(name) if name == "@ offset"
    ));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(name) if name == "@ label"
    ));

    let overloaded = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nread is fn (value : Nat) -> Int\n  @ offset\nread is fn (value : Int) -> Int\n  0\nwrapper is fn (value : Int) -> Int\n  read value\nwrapper 0\n",
        )
        .unwrap_err();
    assert_eq!(overloaded.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        overloaded
            .message
            .contains("overload-dependent defining-context capture forwarding")
    );

    let aliased = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nread is fn () -> Int\n  @ offset\nwrapper is fn () -> Int\n  operation is read\n  operation ()\nwrapper ()\n",
        )
        .unwrap();
    let wrapper = aliased
        .functions
        .iter()
        .find(|function| function.source_name == "wrapper")
        .unwrap();
    assert_eq!(wrapper.parameters.len(), 1);
    assert_eq!(wrapper.parameters[0].name, "@ offset");

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 1\nread is fn (value : Int) -> Int\n  value + @ offset\nwrapper is fn () -> Int\n  read is +\n  read (20, 22)\nwrapper ()\n",
        )
        .unwrap();
    let wrapper = shadowed
        .functions
        .iter()
        .find(|function| function.source_name == "wrapper")
        .unwrap();
    assert!(wrapper.parameters.is_empty());
    assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));
}

#[test]
fn models_private_live_root_data_capture() {
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-001, TOPAL-NAMESPACE-ROOT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-root-data.t"
    ))
    .unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "read")
        .expect("read is instantiated");
    assert_eq!(function.parameters.len(), 3);
    assert_eq!(function.parameters[0].name, "answer");
    assert_eq!(function.parameters[1].name, "root label");
    assert_eq!(function.parameters[2].name, "root answer");
    assert!(
        function
            .parameters
            .iter()
            .all(|parameter| parameter.source_visible)
    );
    let CompilerExpressionKind::Tuple(fields) = &function.body.result.kind else {
        panic!("read returns its three selected values")
    };
    assert!(matches!(
        &fields[0].kind,
        CompilerExpressionKind::Local(name) if name == "root answer"
    ));
    assert!(matches!(
        &fields[1].kind,
        CompilerExpressionKind::Local(name) if name == "answer"
    ));
    assert!(matches!(
        &fields[2].kind,
        CompilerExpressionKind::Local(name) if name == "root label"
    ));
    let CompilerExpressionKind::Call { arguments, .. } = &program.main.result.kind else {
        panic!("expected direct root-data-capturing call")
    };
    assert_eq!(arguments.len(), 3);
    assert_eq!(exact_int(&arguments[0]), Some(BigInt::from(0)));
    let root_bindings = program
        .main
        .statements
        .iter()
        .filter_map(|statement| match statement {
            CompilerStatement::Binding(binding) => Some(binding),
            CompilerStatement::Discard(_) => None,
        })
        .map(|binding| (binding.name.as_str(), binding.storage_name.as_str()))
        .collect::<BTreeMap<_, _>>();
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(storage) if storage == root_bindings["label"]
    ));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(storage) if storage == root_bindings["answer"]
    ));

    let aggregate = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is (40, 2)\nread is fn () -> (Int, Int)\n  root answer\nread ()\n",
        )
        .unwrap();
    let function = aggregate
        .functions
        .iter()
        .find(|function| function.source_name == "read")
        .unwrap();
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(function.parameters[0].name, "root answer");
    assert_eq!(
        function.parameters[0].value_type,
        CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int])
    );
}

#[test]
fn models_private_live_root_data_forwarding() {
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-001, TOPAL-NAMESPACE-ROOT-001
    let forwarded = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-root-data-forwarding.t"
    ))
    .unwrap();
    for name in ["read", "relay", "forward"] {
        let function = forwarded
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(function.parameters.len(), 3);
        assert_eq!(function.parameters[0].name, "answer");
        assert_eq!(function.parameters[1].name, "root label");
        assert_eq!(function.parameters[2].name, "root answer");
    }
    let relay = forwarded
        .functions
        .iter()
        .find(|function| function.source_name == "relay")
        .unwrap();
    let CompilerExpressionKind::Call { arguments, .. } = &relay.body.result.kind else {
        panic!("relay retains its direct forwarded call")
    };
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        &arguments[1].kind,
        CompilerExpressionKind::Local(name) if name == "root label"
    ));
    assert!(matches!(
        &arguments[2].kind,
        CompilerExpressionKind::Local(name) if name == "root answer"
    ));

    let overloaded = analyze_for_compiler(
            "use language (version is v0.1)\nread is fn (value : Nat) -> Int\n  root answer\nread is fn (value : Int) -> Int\n  0\nwrapper is fn (value : Int) -> Int\n  read value\nanswer is 42\nwrapper 0\n",
        )
        .unwrap_err();
    assert_eq!(overloaded.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        overloaded
            .message
            .contains("overload-dependent root-data capture forwarding")
    );

    let aliased = analyze_for_compiler(
            "use language (version is v0.1)\nread is fn () -> Int\n  root answer\nwrapper is fn () -> Int\n  operation is read\n  operation ()\nanswer is 42\nwrapper ()\n",
        )
        .unwrap();
    let wrapper = aliased
        .functions
        .iter()
        .find(|function| function.source_name == "wrapper")
        .unwrap();
    assert_eq!(wrapper.parameters.len(), 1);
    assert_eq!(wrapper.parameters[0].name, "root answer");

    let shadowed = analyze_for_compiler(
            "use language (version is v0.1)\nread is fn (value : Int) -> Int\n  root answer\nwrapper is fn () -> Int\n  read is +\n  read (20, 22)\nanswer is 1\nwrapper ()\n",
        )
        .unwrap();
    let wrapper = shadowed
        .functions
        .iter()
        .find(|function| function.source_name == "wrapper")
        .unwrap();
    assert!(wrapper.parameters.is_empty());
    assert_eq!(exact_int(&shadowed.main.result), Some(BigInt::from(42)));
}

#[test]
fn models_proof_backed_recursive_scalar_environments() {
    // TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001
    let source = include_str!("../../../../../examples/language/recursive-scalar-environments.t")
        .replace("(cycle-even 3, cycle-odd 3)", "cycle-even 3");
    let program = analyze_for_compiler(&source).unwrap();
    assert_eq!(program.functions.len(), 2);
    let even = program
        .functions
        .iter()
        .find(|function| function.source_name == "cycle-even")
        .unwrap();
    let odd = program
        .functions
        .iter()
        .find(|function| function.source_name == "cycle-odd")
        .unwrap();
    for (function, target) in [(even, odd), (odd, even)] {
        assert_eq!(function.parameters.len(), 3);
        assert_eq!(function.parameters[0].name, "value");
        assert_eq!(function.parameters[1].name, "@ captured");
        assert_eq!(function.parameters[2].name, "root live");
        let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
            &function.body.result.kind
        else {
            panic!("expected a proven mutual recursion decision")
        };
        let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
            panic!("expected a proven recursive edge")
        };
        assert_eq!(symbol, &target.symbol);
        assert_eq!(arguments.len(), 3);
        assert!(matches!(
            &arguments[1].kind,
            CompilerExpressionKind::Local(name) if name == "@ captured"
        ));
        assert!(matches!(
            &arguments[2].kind,
            CompilerExpressionKind::Local(name) if name == "root live"
        ));
    }

    let direct = analyze_for_compiler(
            "use language (version is v0.1)\ncaptured is 40\nwalk is fn (value : Int) -> (Int, Int)\n  value\n    <= 0 then (@ captured, root live)\n    otherwise walk (value - 1)\nlive is 2\nwalk 2\n",
        )
        .unwrap();
    assert_eq!(direct.functions.len(), 1);
    assert_eq!(
        direct.functions[0]
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        ["value", "@ captured", "root live"]
    );

    let measured = analyze_for_compiler(
            "use language (version is v0.1)\ncaptured is 40\nrepeat is fn (count : Nat, total : Int) -> (Int, Int, Int) : Decreases count\n  count\n    <= 0 then (total, @ captured, root live)\n    otherwise repeat (count - 1, total + 3)\nlive is 2\nrepeat (4, 0)\n",
        )
        .unwrap();
    assert_eq!(measured.functions.len(), 1);
    assert_eq!(
        measured.functions[0]
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        ["count", "total", "@ captured", "root live"]
    );

    let unproven = analyze_for_compiler(
            "use language (version is v0.1)\ncaptured is 40\nloop is fn (value : Int) -> Int\n  value\n    <= 0 then @ captured\n    otherwise loop value\nloop 1\n",
        )
        .unwrap_err();
    assert_eq!(unproven.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_private_represented_aggregate_environments() {
    // TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/aggregate-environments.t"
    ))
    .unwrap();
    let pair_type = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]);
    let record_type = CompilerType::Record(vec![
        ("amount".into(), CompilerType::Int),
        ("enabled".into(), CompilerType::Boolean),
    ]);
    for (name, capture_name, capture_type) in [
        ("select-context-pair", "@ context-pair", pair_type.clone()),
        ("forward-context-pair", "@ context-pair", pair_type.clone()),
        (
            "select-context-record",
            "@ context-record",
            record_type.clone(),
        ),
        (
            "forward-context-record",
            "@ context-record",
            record_type.clone(),
        ),
        ("select-root-pair", "root live-pair", pair_type.clone()),
        ("forward-root-pair", "root live-pair", pair_type.clone()),
        (
            "select-root-record",
            "root live-record",
            record_type.clone(),
        ),
        (
            "forward-root-record",
            "root live-record",
            record_type.clone(),
        ),
    ] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        let capture = function.parameters.last().unwrap();
        assert_eq!(capture.name, capture_name);
        assert_eq!(capture.value_type, capture_type);
    }
    for (name, capture_name) in [
        ("select-context-token", "@ context-token"),
        ("forward-context-token", "@ context-token"),
        ("select-root-token", "root live-token"),
        ("forward-root-token", "root live-token"),
    ] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        let capture = function.parameters.last().unwrap();
        assert_eq!(capture.name, capture_name);
        assert!(matches!(capture.value_type, CompilerType::Sum(_)));
    }

    let mutual = analyze_for_compiler(
            "use language (version is v0.1)\ncontext-pair is (40, \"context\")\ncycle-even is fn (value : Int) -> (Int, String)\n  value\n    <= 0 then @ context-pair\n    otherwise cycle-odd (value - 1)\ncycle-odd is fn (value : Int) -> (Int, String)\n  value\n    <= 0 then root live-pair\n    otherwise cycle-even (value - 1)\nlive-pair is (7, \"root\")\ncycle-even 3\n",
        )
        .unwrap();
    for name in ["cycle-even", "cycle-odd"] {
        let function = mutual
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(
            function
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            ["value", "@ context-pair", "root live-pair"]
        );
        assert_eq!(function.parameters[1].value_type, pair_type);
        assert_eq!(function.parameters[2].value_type, pair_type);
    }

    let callable = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nbundle is (increment, 40)\nread is fn () -> (Function, Int)\n  @ bundle\nread ()\n",
        )
        .unwrap_err();
    assert_eq!(callable.code, "E-COMPILER-UNSUPPORTED");

    let root_callable = analyze_for_compiler(
            "use language (version is v0.1)\nincrement is fn (value : Int) -> Int\n  value + 1\nread is fn () -> (Function, Int)\n  root bundle\nbundle is (increment, 40)\nread ()\n",
        )
        .unwrap_err();
    assert_eq!(root_callable.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
#[allow(clippy::too_many_lines)] // One model test compares every selected overload vector plus mutual and rejection boundaries.
fn models_overload_selected_private_environments() {
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
    // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/overload-environments.t"
    ))
    .unwrap();
    for (name, explicit_type, captures) in [
        (
            "choose-context",
            CompilerType::Int,
            vec!["@ context-number"],
        ),
        (
            "choose-context",
            CompilerType::String,
            vec!["@ context-label"],
        ),
        ("choose-pair", CompilerType::Int, vec!["@ context-pair"]),
        ("choose-pair", CompilerType::String, vec!["root live-pair"]),
        ("choose-root", CompilerType::Int, vec!["root live-number"]),
        ("choose-root", CompilerType::String, vec!["root live-label"]),
        (
            "cross",
            CompilerType::Int,
            vec!["@ context-number", "root live-number"],
        ),
        (
            "cross",
            CompilerType::String,
            vec!["@ context-number", "root live-number"],
        ),
        (
            "choose-product",
            CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]),
            vec!["@ context-number"],
        ),
        (
            "choose-product",
            CompilerType::Record(vec![
                ("amount".into(), CompilerType::Int),
                ("label".into(), CompilerType::String),
            ]),
            vec!["root live-label"],
        ),
    ] {
        let function = program
            .functions
            .iter()
            .find(|function| {
                function.source_name == name && function.parameters[0].value_type == explicit_type
            })
            .unwrap();
        assert_eq!(
            function.parameters[1..]
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            captures
        );
    }
    for (name, captures) in [
        ("forward-context-number", vec!["@ context-number"]),
        ("forward-context-label", vec!["@ context-label"]),
        ("forward-context-pair", vec!["@ context-pair"]),
        ("forward-root-pair", vec!["root live-pair"]),
        ("forward-root-number", vec!["root live-number"]),
        ("forward-root-label", vec!["root live-label"]),
        (
            "forward-cross",
            vec!["@ context-number", "root live-number"],
        ),
        ("forward-product-tuple", vec!["@ context-number"]),
        ("forward-product-record", vec!["root live-label"]),
    ] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(
            function
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            captures
        );
    }

    let mutual = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\neven is fn (value : Int) -> Int\n  value\n    <= 0 then @ offset\n    otherwise odd (value - 1)\neven is fn (value : String) -> Int\n  1\nodd is fn (value : Int) -> Int\n  value\n    <= 0 then @ offset\n    otherwise even (value - 1)\nodd is fn (value : String) -> Int\n  0\n(even 2, odd \"selected\")\n",
        )
        .unwrap();
    for name in ["even", "odd"] {
        let integer = mutual
            .functions
            .iter()
            .find(|function| {
                function.source_name == name
                    && function.parameters[0].value_type == CompilerType::Int
            })
            .unwrap();
        assert_eq!(integer.parameters[1].name, "@ offset");
    }
    let string = mutual
        .functions
        .iter()
        .find(|function| {
            function.source_name == "odd"
                && function.parameters[0].value_type == CompilerType::String
        })
        .unwrap();
    assert_eq!(string.parameters.len(), 1);

    let context_ambiguous = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  select value\nforward 0\n",
        )
        .unwrap_err();
    assert!(
        context_ambiguous
            .message
            .contains("overload-dependent defining-context capture forwarding")
    );

    let root_ambiguous = analyze_for_compiler(
            "use language (version is v0.1)\nselect is fn (value : Nat) -> Int\n  root answer\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  select value\nanswer is 40\nforward 0\n",
        )
        .unwrap_err();
    assert!(
        root_ambiguous
            .message
            .contains("overload-dependent root-data capture forwarding")
    );

    let qualified = analyze_for_compiler(
            "use language (version is v0.1)\nselect is fn (value : Int) -> Int\n  root answer\nselect is fn (value : String) -> Int\n  0\nforward is fn () -> Int\n  root select 0\nanswer is 40\nforward ()\n",
        )
        .unwrap_err();
    assert!(
        qualified
            .message
            .contains("overload-dependent root-data capture forwarding")
    );

    let locally_inferred = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nselect is fn (value : Int) -> Int\n  @ offset\nselect is fn (value : String) -> Int\n  0\nforward is fn () -> Int\n  value is 1\n  select value\nforward ()\n",
        )
        .unwrap_err();
    assert!(
        locally_inferred
            .message
            .contains("overload-dependent defining-context capture forwarding")
    );
}

#[test]
fn models_local_named_function_environments() {
    // TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/local-function-environments.t"
    ))
    .unwrap();
    let alias_values = program
        .functions
        .iter()
        .find(|function| function.source_name == "alias-values")
        .unwrap();
    assert_eq!(
        alias_values
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        [
            "@ context-number",
            "@ context-label",
            "@ context-pair",
            "root live-number",
            "root live-label",
            "root live-pair",
        ]
    );
    let nested_values = program
        .functions
        .iter()
        .find(|function| function.source_name == "nested-values")
        .unwrap();
    assert_eq!(
        nested_values
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        ["@ context-number", "root live-number"]
    );
    for (name, capture) in [
        ("nested-context", "@ context-number"),
        ("nested-root", "root live-number"),
    ] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.parameters[1].name, capture);
    }

    let ambiguous = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\nforward is fn (value : Int) -> Int\n  operation is select\n  operation value\nforward 0\n",
        )
        .unwrap_err();
    assert!(
        ambiguous
            .message
            .contains("overload-dependent defining-context capture forwarding")
    );
}

#[test]
#[allow(clippy::too_many_lines)] // One model test compares scalar, aggregate, anonymous, nested, and rejection boundaries.
fn models_function_environment_boundaries() {
    // TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001,
    // TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-environment-boundaries.t"
    ))
    .unwrap();

    let boundary_vectors = [
        vec![
            "operation",
            "operation capture @ context-number",
            "operation capture @ context-label",
        ],
        vec![
            "operation",
            "operation capture root live-number",
            "operation capture root live-label",
        ],
        vec![
            "operation",
            "operation capture @ context-pair",
            "operation capture root live-pair",
        ],
        vec![
            "operation",
            "operation capture @ context-number",
            "operation capture root live-number",
        ],
    ];
    let return_operations = program
        .functions
        .iter()
        .filter(|function| function.source_name == "return-operation")
        .collect::<Vec<_>>();
    assert_eq!(return_operations.len(), boundary_vectors.len());
    for expected in boundary_vectors {
        let function = return_operations
            .iter()
            .find(|function| {
                function
                    .parameters
                    .iter()
                    .map(|parameter| parameter.name.as_str())
                    .eq(expected.iter().copied())
            })
            .unwrap_or_else(|| panic!("missing Function boundary vector {expected:?}"));
        assert_eq!(
            function
                .result_captures
                .iter()
                .map(|capture| capture.name.as_str())
                .collect::<Vec<_>>(),
            expected[1..]
                .iter()
                .map(|name| name.strip_prefix("operation capture ").unwrap())
                .collect::<Vec<_>>()
        );
        assert!(
            function
                .result_captures
                .iter()
                .all(|capture| capture.path.is_empty())
        );
    }

    let apply_record = program
        .functions
        .iter()
        .find(|function| function.source_name == "apply-record")
        .unwrap();
    assert_eq!(
        apply_record
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        [
            "package",
            "package field context-operation capture @ context-number",
            "package field context-operation capture @ context-label",
            "package field pair-operation capture @ context-pair",
            "package field pair-operation capture root live-pair",
            "package field root-operation capture root live-number",
            "package field root-operation capture root live-label",
        ]
    );

    for name in ["make-anonymous", "make-anonymous-record", "use-nested"] {
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap();
        assert_eq!(
            function
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            ["@ context-number", "root live-number"]
        );
    }
    let nested = program
        .functions
        .iter()
        .find(|function| function.source_name == "increase")
        .unwrap();
    assert_eq!(
        nested
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        ["value", "@ context-number", "root live-number"]
    );

    let ambiguous = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 40\nselect is fn (value : Nat) -> Int\n  @ offset\nselect is fn (value : Int) -> Int\n  0\napply is fn (operation : Function, value : Int) -> Int\n  operation value\napply (select, 0)\n",
        )
        .unwrap_err();
    assert!(
        ambiguous
            .message
            .contains("value-fact-dependent named Function environment selection")
    );
}

#[test]
fn models_explicit_empty_function_effect_bound_and_static_view() {
    // TOPAL-FUNCTION-EFFECT-BOUND-001, TOPAL-EFFECT-CONTAIN-001,
    // TOPAL-INTRO-STATIC-001, TOPAL-INTRO-VIEW-001,
    // TOPAL-COMPILER-FUNCTION-EMPTY-EFFECT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-effect-bound.t"
    ))
    .unwrap();
    let empty = CompilerEffectRow {
        identities: Vec::new(),
    };
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].declared_effects, Some(empty.clone()));
    let [CompilerStatement::Binding(signature)] = program.main.statements.as_slice() else {
        panic!("the static Function view is the only lowered root binding")
    };
    assert_eq!(signature.value.value_type, CompilerType::FunctionView);
    let CompilerExpressionKind::FunctionView(view) = &signature.value.kind else {
        panic!("the static Function view retains checked metadata")
    };
    assert_eq!(view.identity, "root.identity");
    assert_eq!(view.inputs, ["Int"]);
    assert_eq!(view.output, "Int");
    assert!(!view.is_static);
    assert_eq!(view.declared_effects, Some(empty));
    assert_eq!(exact_int(&program.main.result), Some(BigInt::from(42)));

    let nonempty = analyze_for_compiler(
            "use language (version is v0.1)\nread is fn (value : Int) -> Int : Read value\n  value\n1\n",
        )
        .unwrap_err();
    assert_eq!(nonempty.code, "E-COMPILER-UNSUPPORTED");

    let runtime_view = analyze_for_compiler(
            "use language (version is v0.1)\nidentity is fn (value : Int) -> Int : Effects ()\n  value\nsignature is lang view identity\nsignature\n",
        )
        .unwrap_err();
    assert_eq!(runtime_view.code, "E-COMPILER-UNSUPPORTED");
}

#[test]
fn models_closed_static_introspection_and_runtime_version() {
    // TOPAL-INTRO-QUALIFIED-001, TOPAL-INTRO-STATIC-001,
    // TOPAL-INTRO-VIEW-001, TOPAL-INTRO-CONTEXT-001,
    // TOPAL-INTRO-RELATION-001, TOPAL-COMPILER-STATIC-INTROSPECTION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/static-introspection.t"
    ))
    .unwrap();
    let [
        CompilerStatement::Binding(identity_binding),
        CompilerStatement::Binding(view_binding),
        CompilerStatement::Binding(context_binding),
    ] = program.main.statements.as_slice()
    else {
        panic!("the three static values retain checked metadata")
    };
    assert_eq!(identity_binding.value.value_type, CompilerType::Identity);
    let CompilerExpressionKind::Identity(identity) = &identity_binding.value.kind else {
        panic!("lang identity retains a typed compiler value")
    };
    assert_eq!(identity.kind, ObjectKind::Type);
    assert_eq!(identity.canonical, "type:Int");

    assert_eq!(view_binding.value.value_type, CompilerType::TypeView);
    let CompilerExpressionKind::TypeView(view) = &view_binding.value.kind else {
        panic!("lang view retains a typed compiler value")
    };
    assert_eq!(view.form, CompilerTypeViewForm::Primitive);
    assert_eq!(view.identity, "Int");

    assert_eq!(
        context_binding.value.value_type,
        CompilerType::LanguageContext
    );
    let CompilerExpressionKind::LanguageContext(context) = &context_binding.value.kind else {
        panic!("lang context retains a typed compiler value")
    };
    assert_eq!(context.language, "topal");
    assert_eq!(context.version, LanguageVersion::DESIGN_0);
    assert!(context.features.is_empty());

    let CompilerExpressionKind::Tuple(result) = &program.main.result.kind else {
        panic!("the regression result is a typed Tuple")
    };
    assert!(matches!(
        result[0].kind,
        CompilerExpressionKind::Boolean(true)
    ));
    assert!(matches!(
        result[1].kind,
        CompilerExpressionKind::Boolean(false)
    ));
    assert!(matches!(
        result[2].kind,
        CompilerExpressionKind::Version(LanguageVersion::DESIGN_0)
    ));
    assert_eq!(
        program.main.result.value_type,
        CompilerType::Tuple(vec![
            CompilerType::Boolean,
            CompilerType::Boolean,
            CompilerType::Version,
        ])
    );

    for source in [
        "use language (version is v0.1)\nidentity is lang identity Int\nidentity\n",
        "use language (version is v0.1)\nview is lang view Int\nview\n",
        "use language (version is v0.1)\ncontext is lang context\ncontext\n",
        "use language (version is v0.1)\nlang identity 42\n",
        "use language (version is v0.1)\n1 lang same-object 1\n",
        "use language (version is v0.1)\nuse language (version is v0.1)\nlang version\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED"
        );
    }
}

#[test]
fn models_the_authority_free_lint_language_variant() {
    // TOPAL-SYN-CONTEXT-001, TOPAL-LINT-VARIANT-001,
    // TOPAL-COMPILER-LINT-VARIANT-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/lint-language-variant.t"
    ))
    .unwrap();
    assert_eq!(program.language_features, ["lint"]);
    assert_eq!(program.main.result.value_type, CompilerType::Scope);
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::LintNamespace
    ));

    let context = analyze_for_compiler(
            "use language (version is v0.1, features is (lint, lint))\ncontext is lang context\nlang lint\n",
        )
        .unwrap();
    assert_eq!(context.language_features, ["lint"]);
    let [CompilerStatement::Binding(context_binding)] = context.main.statements.as_slice() else {
        panic!("the selected context retains one static binding")
    };
    let CompilerExpressionKind::LanguageContext(context) = &context_binding.value.kind else {
        panic!("lang context retains checked feature metadata")
    };
    assert_eq!(context.features, ["lint"]);

    let missing = analyze_for_compiler("use language (version is v0.1)\nlang lint\n").unwrap_err();
    assert_eq!(missing.code, "E-LINT-VARIANT");
    let unsupported = analyze_for_compiler(
        "use language (version is v0.1, features is (debug, lint))\nlang lint\n",
    )
    .unwrap_err();
    assert_eq!(unsupported.code, "E-COMPILER-UNSUPPORTED");

    for source in [
        "use language (version is v0.1)\nv0.1 (lang serialize) ()\n",
        "use language (version is v0.1)\ntext is \"Topal\"\nv0.1 (lang serialize) text\n",
        "use language (version is v0.1)\nv0.1 (lang serialize) (1, \"two\")\n",
    ] {
        let admitted = analyze_for_compiler(source).unwrap();
        assert!(matches!(
            admitted.main.result.value_type,
            CompilerType::SerializationStream(_)
        ));
    }
}

#[test]
fn models_canonical_native_serialization_and_validated_reconstruction() {
    // TOPAL-SER-HEADER-001 through TOPAL-SER-DESER-001,
    // TOPAL-COMPILER-NATIVE-SERIALIZATION-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/native-serialization.t"
    ))
    .unwrap();
    let [
        CompilerStatement::Binding(serializer),
        CompilerStatement::Binding(stream),
    ] = program.main.statements.as_slice()
    else {
        panic!("native serialization regression retains its two bindings")
    };
    assert_eq!(
        serializer.value.value_type,
        CompilerType::NativeSerializer(LanguageVersion::DESIGN_0)
    );
    let CompilerExpressionKind::Serialize { bytes, value } = &stream.value.kind else {
        panic!("stream binding retains canonical bytes and its once-evaluated value")
    };
    assert_eq!(bytes.len(), 76);
    assert_eq!(&bytes[..8], b"TOPALSER");
    assert!(matches!(value.kind, CompilerExpressionKind::Record(_)));
    let decoded = deserialize_native(bytes, SerializationLimits::default()).unwrap();
    assert_eq!(decoded.header.language_identity, "topal");
    assert_eq!(decoded.header.language_version, LanguageVersion::DESIGN_0);
    assert_eq!(decoded.header.byte_order, StreamByteOrder::Little);
    assert_eq!(decoded.types.len(), 3);
    assert!(matches!(
        decoded.types.as_slice(),
        [
            TypeDefinition::Int { identity, signed: true, width_bits: 0 },
            TypeDefinition::Boolean { identity: boolean },
            TypeDefinition::Record { identity: record, fields },
        ] if identity == "Int"
            && boolean == "Boolean"
            && record == "Record"
            && fields == &[("answer".into(), 0), ("accepted".into(), 1)]
    ));
    assert!(matches!(
        &program.main.result,
        CompilerExpression {
            kind: CompilerExpressionKind::Deserialize(stream),
            value_type: CompilerType::Record(fields),
            ..
        } if matches!(stream.kind, CompilerExpressionKind::Local(_))
            && fields == &vec![
                ("accepted".into(), CompilerType::Boolean),
                ("answer".into(), CompilerType::Int),
            ]
    ));

    let invalid_version =
        analyze_for_compiler("use language (version is v0.1)\n42 (lang serialize) true\n")
            .unwrap_err();
    assert_eq!(invalid_version.code, "E-SERIALIZATION-VERSION");
    let unknown_version =
        analyze_for_compiler("use language (version is v0.1)\nunknown (lang serialize) true\n")
            .unwrap_err();
    assert_eq!(unknown_version.code, "E-UNBOUND-NAME");
    let unsupported = analyze_for_compiler(
        "use language (version is v0.1)\nv0.1 (lang serialize) (lang version)\n",
    )
    .unwrap_err();
    assert_eq!(unsupported.code, "E-COMPILER-UNSUPPORTED");
    for source in [
        "use language (version is v0.1)\nv0.1 (lang serialize) (1, (2, 3))\n",
        "use language (version is v0.1)\nv0.1 (lang serialize) (outer is (answer is 42), accepted is true)\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED",
            "{source}"
        );
    }
}

#[test]
fn models_closed_static_capability_composition() {
    // TOPAL-CAPABILITY-EVIDENCE-001, TOPAL-CAPABILITY-COHERENCE-001,
    // TOPAL-CAPABILITY-COMPOSE-001, TOPAL-COMPILER-CAPABILITY-COMPOSE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capability-composition.t"
    ))
    .unwrap();
    let [
        CompilerStatement::Binding(comparable),
        CompilerStatement::Binding(searchable),
        CompilerStatement::Binding(alternatives),
    ] = program.main.statements.as_slice()
    else {
        panic!("the three root Capability bindings retain checked metadata")
    };
    for binding in [comparable, searchable, alternatives] {
        assert_eq!(binding.value.value_type, CompilerType::Capability);
    }
    let CompilerExpressionKind::Capability(comparable) = &comparable.value.kind else {
        panic!("Comparable is a checked Capability")
    };
    assert_eq!(
        comparable.alternatives,
        [vec!["Equality".to_owned(), "Ordering".to_owned()]]
    );
    let CompilerExpressionKind::Capability(searchable) = &searchable.value.kind else {
        panic!("Searchable is a checked Capability")
    };
    assert_eq!(
        searchable.alternatives,
        [vec!["Foldable".to_owned(), "Membership".to_owned()]]
    );
    let CompilerExpressionKind::Capability(result) = &program.main.result.kind else {
        panic!("the final result retains canonical Capability alternatives")
    };
    assert_eq!(
        result.alternatives,
        [
            vec!["Equality".to_owned(), "Ordering".to_owned()],
            vec!["Foldable".to_owned(), "Membership".to_owned()],
        ]
    );
    assert_eq!(
        result.display(),
        "Equality and Ordering or Foldable and Membership"
    );

    let canonical_source = "use language (version is v0.1)\nSame is Equality and Equality\nRepeated : Capability is Same or Equality\nReversed : Capability is Membership or Repeated\nReversed\n";
    let idempotent = analyze_for_compiler(canonical_source).unwrap();
    let CompilerExpressionKind::Capability(result) = &idempotent.main.result.kind else {
        panic!("the classified alias remains a Capability")
    };
    assert_eq!(
        result.alternatives,
        [vec!["Equality".to_owned()], vec!["Membership".to_owned()]]
    );
    let interpreted = crate::source::Session::new()
        .evaluate_source_file(canonical_source, &mut std::io::sink())
        .unwrap();
    assert_eq!(interpreted.to_string(), result.display());

    for source in [
        "use language (version is v0.1)\n(Equality, true)\n",
        "use language (version is v0.1)\nchoose is fn static () -> Capability\n  Equality\nchoose ()\n",
        "use language (version is v0.1)\nEquality xor Ordering\n",
    ] {
        assert_eq!(
            analyze_for_compiler(source).unwrap_err().code,
            "E-COMPILER-UNSUPPORTED",
            "{source}"
        );
    }
    assert_eq!(
        analyze_for_compiler(
            "use language (version is v0.1)\nvalue : Boolean is Equality\nvalue\n"
        )
        .unwrap_err()
        .code,
        "E-TYPE-MISMATCH"
    );
}
