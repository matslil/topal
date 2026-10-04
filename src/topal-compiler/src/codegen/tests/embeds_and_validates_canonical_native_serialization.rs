#[test]
fn embeds_and_validates_canonical_native_serialization() {
    // TOPAL-SER-HEADER-001 through TOPAL-SER-DESER-001,
    // TOPAL-COMPILER-NATIVE-SERIALIZATION-001,
    // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/native-serialization.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "native-serialization.t").emit();

    assert!(llvm.contains("%topal.SerializationStreamStorage = type { ptr, i64 }"));
    assert!(llvm.contains(&llvm_bytes(b"TOPALSER")));
    assert_eq!(
        llvm.matches("call ptr @topal.runtime.serialization.make")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call void @topal.runtime.serialization.verify")
            .count(),
        1
    );
    assert!(llvm.contains("%copy = call ptr @topal.platform.allocate(i64 %length)"));
    assert!(llvm.contains("%actual.byte = load i8, ptr %actual.pointer"));
    assert!(llvm.contains("%expected.byte = load i8, ptr %expected.pointer"));
    assert!(llvm.contains("define internal void @topal.runtime.u64.print"));
    assert!(llvm.contains("name: \"SerializationStream\""));
    assert!(llvm.contains("name: \"TopalSerializationStreamHeader\""));
    assert!(llvm.contains("name: \"byte_count\""));
    assert!(!llvm.contains("declare ptr @serialize"));
    assert!(!llvm.contains("declare ptr @deserialize"));
    assert!(!llvm.contains("call ptr %"));

    let display =
        analyze_for_compiler("use language (version is v0.1)\nv0.1 (lang serialize) true\n")
            .unwrap();
    let display_llvm = Generator::new(&display, "native-stream-display.t").emit();
    assert!(display_llvm.contains("call void @topal.runtime.u64.print(i64 "));
}

#[test]
fn folds_and_erases_static_capability_composition() {
    // TOPAL-CAPABILITY-EVIDENCE-001, TOPAL-CAPABILITY-COHERENCE-001,
    // TOPAL-CAPABILITY-COMPOSE-001, TOPAL-COMPILER-CAPABILITY-COMPOSE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capability-composition.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "capability-composition.t").emit();
    assert!(llvm.contains(&llvm_bytes(
        b"Equality and Ordering or Foldable and Membership"
    )));
    assert!(!llvm.contains("Comparable"));
    assert!(!llvm.contains("Searchable"));
    assert!(!llvm.contains("ComparableOrSearchable"));
    assert!(!llvm.contains("Capability"));
    assert!(!llvm.contains("topal.runtime.capability"));
    assert!(!llvm.contains("topal.runtime.evidence"));
}

#[test]
fn erases_function_interface_evidence_before_direct_llvm_lowering() {
    // TOPAL-INTERFACE-SHAPE-001, TOPAL-INTERFACE-IMPLEMENTATION-001,
    // TOPAL-COMPILER-FUNCTION-INTERFACE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-interface.t"
    ))
    .unwrap();
    assert_eq!(program.interfaces[0].identity, "root.Parser");
    assert_eq!(
        program.interface_implementations[0].operations[0].declaration_identity,
        "root.parse:ordinary(String)"
    );
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "function-interface.t").emit();
    assert!(llvm.contains(&format!("define internal fastcc i1 @{symbol}(ptr %arg0)")));
    assert!(llvm.contains(&format!("call fastcc i1 @{symbol}(ptr %")));
    assert!(!llvm.contains("root.Parser"));
    assert!(!llvm.contains("Parser"));
    assert!(!llvm.contains("Interface"));
    assert!(!llvm.contains("topal.runtime.interface"));
    assert!(!llvm.contains("topal.runtime.evidence"));
}

#[test]
fn emits_named_function_values_with_direct_retained_calls() {
    // TOPAL-COMPILER-NAMED-FUNCTION-VALUE-001, TOPAL-FUNCTION-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/named-function-values.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "named-function-values.t").emit();
    assert!(
        llvm.lines()
            .any(|line| { line.contains("call fastcc") && line.contains(&format!("@{symbol}(")) })
    );
    assert!(llvm.contains("!DIEnumerator(name: \"<fn increment>\", value: 0)"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("call ptr %"));

    let displayed = analyze_for_compiler(
            "use language (version is v0.1)\nfirst is fn (value : Int) -> Int\n  value\nsecond is fn (value : Int) -> Int\n  value\n(first, second)\n",
        )
        .unwrap();
    let llvm = Generator::new(&displayed, "function-display.t").emit();
    assert!(llvm.contains(&llvm_bytes(b"<fn first>")));
    assert!(llvm.contains(&llvm_bytes(b"<fn second>")));
}

#[test]
fn emits_symbolic_callable_values_as_direct_operations() {
    // TOPAL-COMPILER-SYMBOLIC-CALLABLE-VALUE-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/callable-values.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "callable-values.t").emit();
    assert!(llvm.contains("call ptr @topal.runtime.int.add("));
    assert!(llvm.contains("call ptr @topal.runtime.int.negate("));
    assert!(llvm.contains("call i32 @topal.runtime.int.compare("));
    assert!(llvm.contains("!DIEnumerator(name: \"+\", value: 0)"));
    assert!(llvm.contains("!DIEnumerator(name: \"-\", value: 1)"));
    assert!(llvm.contains("!DIEnumerator(name: \"<=>\", value: 2)"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("call ptr %"));

    let displayed = analyze_for_compiler(
            "use language (version is v0.1)\nadd is +\nnegate is -\ncompare-values is <=>\n(add, negate, compare-values)\n",
        )
        .unwrap();
    let llvm = Generator::new(&displayed, "callable-display.t").emit();
    assert!(llvm.contains(&llvm_bytes(b"+")));
    assert!(llvm.contains(&llvm_bytes(b"-")));
    assert!(llvm.contains(&llvm_bytes(b"<=>")));
}

#[test]
fn emits_every_symbolic_callable_value_without_dispatch() {
    // TOPAL-COMPILER-SYMBOLIC-CALLABLE-EXPANDED-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-TYPE-CALL-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/expanded-callable-values.t"
    ))
    .unwrap();
    let selector = program
        .functions
        .iter()
        .find(|function| function.source_name == "select")
        .unwrap();
    let llvm = Generator::new(&program, "expanded-callable-values.t").emit();

    for operation in [
        "topal.runtime.int.compare",
        "topal.runtime.int.multiply",
        "topal.runtime.rational.divide",
        "topal.runtime.int.quotient.modulo",
        "topal.runtime.int.modulo",
        "topal.runtime.int.power",
        "topal.runtime.range.make",
    ] {
        assert!(
            llvm.contains(&format!("call ptr @{operation}"))
                || llvm.contains(&format!("call i32 @{operation}"))
        );
    }
    assert!(llvm.contains(&format!(
        "define internal fastcc i32 @{}(i32 %arg0)",
        selector.symbol
    )));
    assert!(llvm.contains(&format!("call fastcc i32 @{}(i32", selector.symbol)));
    for (name, value) in [
        ("=", 4),
        ("/=", 5),
        ("*", 10),
        ("/", 11),
        ("/%", 12),
        ("%", 13),
        ("^", 14),
        ("..", 15),
        ("<..", 16),
        ("..=", 17),
        ("<..=", 18),
    ] {
        assert!(llvm.contains(&format!("!DIEnumerator(name: \"{name}\", value: {value})")));
    }
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_function_inputs_as_private_tags_with_direct_specialization() {
    // TOPAL-COMPILER-FUNCTION-PARAMETER-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-value-boundary.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "function-value-boundary.t").emit();
    assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(i32 %arg0)")));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr") && line.contains(&format!("@{symbol}(i32"))
    }));
    assert!(llvm.contains("call ptr @topal.runtime.int.add("));
    assert!(llvm.contains("!DILocalVariable(name: \"operation\""));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_closed_function_results_as_private_tags_with_direct_application() {
    // TOPAL-COMPILER-FUNCTION-RESULT-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-FUNCTION-VALUE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-results.t"
    ))
    .unwrap();
    let selectors = program
        .functions
        .iter()
        .filter(|function| function.source_name == "select")
        .collect::<Vec<_>>();
    assert_eq!(selectors.len(), 2);
    let llvm = Generator::new(&program, "function-results.t").emit();
    for selector in selectors {
        assert!(llvm.contains(&format!(
            "define internal fastcc i32 @{}(i32 %arg0)",
            selector.symbol
        )));
        assert!(llvm.contains(&format!("call fastcc i32 @{}(i32", selector.symbol)));
    }
    let increment = program
        .functions
        .iter()
        .find(|function| function.source_name == "increment")
        .unwrap();
    assert!(llvm.contains(&format!("call fastcc ptr @{}(", increment.symbol)));
    assert!(llvm.contains("call ptr @topal.runtime.int.add("));
    assert!(llvm.contains("!DILocalVariable(name: \"selected\""));
    assert!(llvm.contains("!DILocalVariable(name: \"addition\""));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_direct_anonymous_functions_without_a_closure_runtime() {
    // TOPAL-COMPILER-ANONYMOUS-DIRECT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-function-application.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "anonymous-function-application.t").emit();
    for function in &program.functions {
        assert!(llvm.contains(&format!("define internal fastcc ptr @{}(", function.symbol)));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr") && line.contains(&format!("@{}(", function.symbol))
        }));
    }
    assert!(llvm.contains("!DIEnumerator(name: \"<anonymous fn/1>\", value: 18)"));
    assert!(llvm.contains("!DIEnumerator(name: \"<anonymous fn/2>\", value: 19)"));
    assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 2"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_anonymous_captures_and_results_as_exact_private_calls() {
    // TOPAL-COMPILER-ANONYMOUS-CAPTURE-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-function-captures.t"
    ))
    .unwrap();
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
    let factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-double")
        .unwrap();
    let returned = program
        .functions
        .iter()
        .find(|function| {
            function.source_name == "<anonymous fn/1>"
                && function.parameters.len() == 1
                && function.parameters[0].name == "value"
        })
        .unwrap();
    let llvm = Generator::new(&program, "anonymous-function-captures.t").emit();

    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1)",
        captured.symbol
    )));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr")
            && line.contains(&format!("@{}(ptr %arg1, ptr %arg0)", captured.symbol))
    }));
    assert!(llvm.contains(&format!("define internal fastcc i32 @{}()", factory.symbol)));
    assert!(llvm.contains(&format!("call fastcc i32 @{}()", factory.symbol)));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0)",
        returned.symbol
    )));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr") && line.contains(&format!("@{}(", returned.symbol))
    }));
    assert!(llvm.contains("!DILocalVariable(name: \"offset\", arg: 2"));
    assert!(llvm.contains("!DILocalVariable(name: \"twice\""));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_captured_function_parameters_as_hidden_direct_arguments() {
    // TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-NESTED-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capturing-function-parameters.t"
    ))
    .unwrap();
    let scalar_anonymous = program
        .functions
        .iter()
        .find(|function| {
            function.source_name == "<anonymous fn/1>"
                && function
                    .parameters
                    .iter()
                    .map(|parameter| parameter.name.as_str())
                    .eq(["input", "left", "right"])
        })
        .unwrap();
    let pair_boundary = program
        .functions
        .iter()
        .find(|function| function.source_name == "apply-pair")
        .unwrap();
    let nested = program
        .functions
        .iter()
        .find(|function| function.source_name == "add")
        .unwrap();
    let llvm = Generator::new(&program, "capturing-function-parameters.t").emit();

    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
        scalar_anonymous.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc {{ ptr, ptr }} @{}(i32 %arg0, ptr %arg1, {{ ptr, ptr }} %arg2)",
        pair_boundary.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
        nested.symbol
    )));
    assert!(llvm.contains("!DIEnumerator(name: \"<fn add>\""));
    assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 2"));
    assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 3"));
    assert!(llvm.contains("!DILocalVariable(name: \"pair\", arg: 2"));
    assert!(!llvm.contains("DILocalVariable(name: \"operation capture"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_captured_function_results_as_exact_private_aggregates() {
    // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capturing-function-results.t"
    ))
    .unwrap();
    let scalar_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-scalars")
        .unwrap();
    let pair_factory = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-pair")
        .unwrap();
    let forwarding = program
        .functions
        .iter()
        .find(|function| function.source_name == "return-operation")
        .unwrap();
    let llvm = Generator::new(&program, "capturing-function-results.t").emit();

    assert!(llvm.contains(&format!(
        "define internal fastcc {{ i32, ptr, ptr }} @{}(ptr %arg0, ptr %arg1)",
        scalar_factory.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc {{ i32, {{ ptr, ptr }} }} @{}({{ ptr, ptr }} %arg0)",
        pair_factory.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc {{ i32, ptr, ptr }} @{}(i32 %arg0, ptr %arg1, ptr %arg2)",
        forwarding.symbol
    )));
    assert!(llvm.contains("call fastcc { i32, ptr, ptr }"));
    assert!(llvm.contains("insertvalue { i32, ptr, ptr } poison, i32"));
    assert!(llvm.contains("extractvalue { i32, ptr, ptr }"));
    assert!(llvm.contains("insertvalue { i32, { ptr, ptr } }"));
    assert!(llvm.contains("extractvalue { i32, { ptr, ptr } }"));
    assert!(llvm.contains("!DILocalVariable(name: \"scalar-operation\""));
    assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 2"));
    assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 3"));
    assert!(llvm.contains("!DILocalVariable(name: \"pair\", arg: 2"));
    assert!(!llvm.contains("DILocalVariable(name: \"topal.function.result"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_function_result_chains_as_once_only_direct_calls() {
    // TOPAL-COMPILER-FUNCTION-RESULT-CHAIN-001,
    // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-FUNCTION-VALUE-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-result-chains.t"
    ))
    .unwrap();
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
    let llvm = Generator::new(&program, "function-result-chains.t").emit();

    for factory in factories {
        assert_eq!(
            llvm.matches(&format!("@{}(", factory.symbol)).count(),
            2,
            "factory {} must have one definition and one direct call",
            factory.symbol
        );
    }
    assert!(llvm.contains("call fastcc { i32, ptr }"));
    assert!(llvm.contains("call fastcc { i32, { ptr, ptr } }"));
    assert!(llvm.contains("extractvalue { i32, ptr }"));
    assert!(llvm.contains("!DILocalVariable(name: \"offset\", arg: 2"));
    assert!(llvm.contains("!DILocalVariable(name: \"pair\", arg: 2"));
    assert!(!llvm.contains("DILocalVariable(name: \"topal.function.chain"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_anonymous_product_patterns_as_flat_once_only_private_calls() {
    // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/anonymous-product-functions.t"
    ))
    .unwrap();
    let make_pair = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-pair")
        .unwrap();
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
    let mixed = program
        .functions
        .iter()
        .find(|function| {
            function.source_name == "<anonymous fn/2>"
                && function
                    .parameters
                    .iter()
                    .any(|parameter| parameter.name == "extra")
        })
        .unwrap();
    let llvm = Generator::new(&program, "anonymous-product-functions.t").emit();

    assert_eq!(
        llvm.matches(&format!(
            "call fastcc {{ ptr, ptr }} @{}(",
            make_pair.symbol
        ))
        .count(),
        1
    );
    assert!(llvm.contains("extractvalue { ptr, ptr }"));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
        captured.symbol
    )));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr")
            && line.contains(&format!("@{}(", captured.symbol))
            && line.contains("ptr %arg0")
    }));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
        mixed.symbol
    )));
    assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 2"));
    assert!(llvm.contains("!DILocalVariable(name: \"offset\", arg: 3"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));

    let heterogeneous = analyze_for_compiler(
            "use language (version is v0.1)\nchoose : Function is { (condition, text) } text\nchoose (true, \"kept\")\n",
        )
        .unwrap();
    let function = &heterogeneous.functions[0];
    let heterogeneous_llvm = Generator::new(&heterogeneous, "heterogeneous-product.t").emit();
    assert!(heterogeneous_llvm.contains(&format!(
        "define internal fastcc ptr @{}(i1 %arg0, ptr %arg1)",
        function.symbol
    )));
    assert!(heterogeneous_llvm.contains(&format!(
        "call fastcc ptr @{}(i1 true, ptr ",
        function.symbol
    )));
}

#[test]
fn emits_nested_anonymous_patterns_as_recursive_once_only_projections() {
    // TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001,
    // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/nested-anonymous-patterns.t"
    ))
    .unwrap();
    let make_values = program
        .functions
        .iter()
        .find(|function| function.source_name == "make-values")
        .unwrap();
    let nested = program
        .functions
        .iter()
        .find(|function| {
            function.source_name == "<anonymous fn/1>"
                && function
                    .parameters
                    .iter()
                    .map(|parameter| parameter.name.as_str())
                    .eq(["left", "middle", "right"])
        })
        .unwrap();
    let llvm = Generator::new(&program, "nested-anonymous-patterns.t").emit();

    assert_eq!(
        llvm.matches(&format!("@{}(", make_values.symbol)).count(),
        2,
        "nested Tuple factory must have one definition and one call"
    );
    assert!(llvm.contains("extractvalue { ptr, { ptr, ptr } }"));
    assert!(llvm.contains("extractvalue { ptr, ptr }"));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
        nested.symbol
    )));
    assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 1);
    for (name, argument) in [("left", 1), ("middle", 2), ("right", 3)] {
        assert!(llvm.contains(&format!(
            "!DILocalVariable(name: \"{name}\", arg: {argument}"
        )));
    }
    assert!(!llvm.contains("DILocalVariable(name: \"topal.anonymous.argument"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_function_aggregates_with_exact_private_direct_boundaries() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
    // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/function-aggregate-boundaries.t"
    ))
    .unwrap();
    let symbol = |name: &str| {
        program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
            .symbol
            .clone()
    };
    let make_tuple = symbol("make-tuple");
    let apply_tuple = symbol("apply-tuple");
    let make_record = symbol("make-record");
    let apply_record = symbol("apply-record");
    let make_nested = symbol("make-nested");
    let apply_nested = symbol("apply-nested");
    let llvm = Generator::new(&program, "function-aggregate-boundaries.t").emit();

    for expected in [
        format!("define internal fastcc {{ i32, ptr }} @{make_tuple}(i32 %arg0, ptr %arg1)"),
        format!("define internal fastcc ptr @{apply_tuple}({{ i32, ptr }} %arg0)"),
        format!(
            "define internal fastcc {{ i32, ptr, i32, i32 }} @{make_record}(i32 %arg0, ptr %arg1)"
        ),
        format!("define internal fastcc ptr @{apply_record}({{ i32, ptr, i32, i32 }} %arg0)"),
        format!(
            "define internal fastcc {{ {{ i32, ptr }}, i32 }} @{make_nested}(i32 %arg0, ptr %arg1)"
        ),
        format!("define internal fastcc ptr @{apply_nested}({{ {{ i32, ptr }}, i32 }} %arg0)"),
    ] {
        assert!(llvm.contains(&expected), "missing {expected:?}");
    }
    assert!(llvm.contains("extractvalue { i32, ptr }"));
    assert!(llvm.contains("extractvalue { i32, ptr, i32, i32 }"));
    assert!(llvm.contains("extractvalue { { i32, ptr }, i32 }"));
    assert!(llvm.contains("DW_TAG_member, name: \"operation\""));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
}

#[test]
fn emits_capture_bearing_function_aggregates_with_path_ordered_private_transport() {
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
    // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
    // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/capturing-function-aggregate-boundaries.t"
    ))
    .unwrap();
    let symbol = |name: &str| {
        program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
            .symbol
            .clone()
    };
    let make_record = symbol("make-record");
    let apply_record = symbol("apply-record");
    let forward_record = symbol("forward-record");
    let make_tuple = symbol("make-tuple");
    let apply_tuple = symbol("apply-tuple");
    let apply_one = symbol("apply-one");
    let llvm = Generator::new(&program, "capturing-function-aggregate-boundaries.t").emit();

    let record = "{ i32, i32, ptr, i32, i32, i32 }";
    let extended_record = "{ { i32, i32, ptr, i32, i32, i32 }, ptr, ptr }";
    for expected in [
        format!("define internal fastcc {extended_record} @{make_record}(ptr %arg0, ptr %arg1)"),
        format!(
            "define internal fastcc {{ ptr, ptr }} @{apply_record}({record} %arg0, ptr %arg1, ptr %arg2)"
        ),
        format!(
            "define internal fastcc {extended_record} @{forward_record}({record} %arg0, ptr %arg1, ptr %arg2)"
        ),
        format!("define internal fastcc {{ {{ i32, ptr }}, ptr }} @{make_tuple}(ptr %arg0)"),
        format!("define internal fastcc ptr @{apply_tuple}({{ i32, ptr }} %arg0, ptr %arg1)"),
        format!(
            "define internal fastcc ptr @{apply_one}({{ i32, ptr, i32, i32 }} %arg0, ptr %arg1)"
        ),
    ] {
        assert!(llvm.contains(&expected), "missing {expected:?}");
    }
    assert!(llvm.contains(&format!("call fastcc {extended_record} @{make_record}(")));
    assert!(llvm.contains(&format!(
        "call fastcc {{ ptr, ptr }} @{apply_record}({record}"
    )));
    assert!(llvm.contains("insertvalue { { i32, i32, ptr, i32, i32, i32 }, ptr, ptr }"));
    assert!(llvm.contains("extractvalue { { i32, i32, ptr, i32, i32, i32 }, ptr, ptr }"));
    assert!(llvm.contains("insertvalue { { i32, ptr }, ptr }"));
    assert!(llvm.contains("extractvalue { { i32, ptr }, ptr }"));
    assert!(llvm.contains("DW_TAG_member, name: \"operation\""));
    assert!(!llvm.contains("DILocalVariable(name: \"operation capture"));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
}

#[test]
fn emits_escaping_nested_function_environments_as_exact_private_results() {
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/escaping-nested-function-environments.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "escaping-nested-function-environments.t").emit();

    assert_eq!(
        llvm.matches("define internal fastcc { i32, ptr, ptr, ptr } @topal.fn.make_2doperation.")
            .count(),
        4
    );
    assert!(llvm.contains(
        "define internal fastcc { i32, { ptr, ptr } } @topal.fn.make_2dpair_2doperation."
    ));
    assert!(llvm.contains(
            "define internal fastcc { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord."
        ));
    assert!(llvm.contains("define internal fastcc ptr @topal.fn.increase."));
    assert!(llvm.contains("call fastcc { i32, ptr, ptr, ptr }"));
    assert!(llvm.contains("call fastcc { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr }"));
    assert!(llvm.contains("extractvalue { i32, ptr, ptr, ptr }"));
    assert!(llvm.contains("extractvalue { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr }"));
    for name in ["offset", "pair", "@ context-offset", "root live-offset"] {
        assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
    }
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("topal.runtime.environment"));
}

#[test]
fn emits_optional_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-OPTIONAL-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/optional-function-environments.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "optional-function-environments.t").emit();

    assert_eq!(
        llvm.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2doptional.")
            .count(),
        5
    );
    assert!(llvm.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(llvm.contains(
            "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord."
        ));
    assert!(llvm.contains("call ptr @topal.runtime.optional.some(ptr"));
    assert!(llvm.contains("call ptr @topal.runtime.optional.none()"));
    assert_eq!(
        llvm.matches("call void @topal.runtime.pattern.identity.fail()")
            .count(),
        4
    );
    for name in [
        "candidate",
        "operation",
        "offset",
        "@ context-offset",
        "root live-offset",
    ] {
        assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
    }
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("topal.runtime.environment"));
}

#[test]
fn emits_sum_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-SUM-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/sum-function-environments.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "sum-function-environments.t").emit();

    assert_eq!(
        llvm.matches(
            "define internal fastcc { { i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2doperation."
        )
        .count(),
        5
    );
    assert!(llvm.contains(
        "define internal fastcc { { i32, i32, ptr }, ptr, ptr, ptr } @topal.fn.make_2dchoice."
    ));
    assert_eq!(
        llvm.matches("call void @topal.runtime.pattern.identity.fail()")
            .count(),
        4
    );
    for name in [
        "candidate",
        "operation",
        "offset",
        "@ context-offset",
        "root live-offset",
    ] {
        assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
    }
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("topal.runtime.environment"));
}

#[test]
fn emits_result_function_environments_as_exact_private_paths() {
    // TOPAL-COMPILER-RESULT-FUNCTION-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/result-function-environments.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "result-function-environments.t").emit();

    assert_eq!(
        llvm.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dresult.")
            .count(),
        7
    );
    assert_eq!(
        llvm.matches(
            "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.make_2dfallible."
        )
        .count(),
        2
    );
    assert!(
        llvm.contains("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.project_2dresult.")
    );
    assert!(llvm.contains(
        "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.project_2dresult."
    ));
    assert!(llvm.contains(
        "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
    ));
    assert!(llvm.contains(
        "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
    ));
    assert!(llvm.contains("call ptr @topal.runtime.result.success(ptr"));
    assert!(llvm.contains("insertvalue { ptr, ptr, ptr, ptr, ptr, ptr } poison, ptr"));
    assert!(llvm.contains("insertvalue { ptr, ptr, ptr, ptr, ptr, ptr } %"));
    assert!(llvm.contains(", ptr null, 5"));
    assert!(llvm.contains("ret { ptr, ptr, ptr, ptr, ptr, ptr }"));
    for name in [
        "candidate",
        "operation",
        "offset",
        "@ context-offset",
        "root live-offset",
    ] {
        assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
    }
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("topal.runtime.environment"));
}

#[test]
fn emits_repeated_anonymous_patterns_as_exact_private_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-anonymous-patterns.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "repeated-anonymous-patterns.t").emit();

    assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
    assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 5);
    assert!(llvm.contains("call i32 @topal.runtime.int.compare(ptr %arg0, ptr %arg1)"));
    assert!(llvm.contains("call i1 @topal.runtime.string.equal(ptr %arg0, ptr %arg1)"));
    assert!(llvm.contains("icmp eq i32 %arg0, %arg1"));
    for name in ["value", "text", "operation"] {
        assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\", arg: 1")));
        assert!(!llvm.contains(&format!("!DILocalVariable(name: \"{name}\", arg: 2")));
    }
    assert!(!llvm.contains("topal.runtime.pattern.identity.match"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_repeated_anonymous_aggregate_values_as_structural_private_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-anonymous-aggregate-patterns.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "repeated-anonymous-aggregate-patterns.t").emit();

    assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 4);
    assert!(llvm.contains("extractvalue { ptr, ptr } %arg0, 0"));
    assert!(llvm.contains("extractvalue { ptr, ptr } %arg1, 1"));
    assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
    assert!(llvm.contains("call i1 @topal.runtime.string.equal"));
    assert!(llvm.contains("call i1 @topal.runtime.optional.int.equal(ptr %arg0, ptr %arg1)"));
    assert!(llvm.contains("call i1 @topal.runtime.list.int.equal(ptr %arg0, ptr %arg1)"));
    assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
    assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
    assert!(!llvm.contains("!DILocalVariable(name: \"value\", arg: 2"));
    assert!(!llvm.contains("topal.runtime.pattern.identity.aggregate"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_capture_free_function_aggregate_pattern_identity_as_direct_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-function-aggregate-patterns.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "repeated-function-aggregate-patterns.t").emit();

    assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 3);
    assert!(llvm.contains("extractvalue { i32, ptr } %arg0, 0"));
    assert!(llvm.contains("extractvalue { i32, ptr } %arg1, 0"));
    assert!(llvm.contains("extractvalue { i32, ptr, i32, i32 } %arg0, 0"));
    assert!(llvm.contains("icmp eq i32"));
    assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
    assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
    assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
    assert!(!llvm.contains("topal.runtime.pattern.identity.aggregate"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_captured_anonymous_function_pattern_identity_as_direct_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-captured-function-patterns.t"
    ))
    .unwrap();
    let guarded = program
        .functions
        .iter()
        .filter(|function| function.pattern_identities.len() == 2)
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 2);
    let llvm = Generator::new(&program, "repeated-captured-function-patterns.t").emit();

    assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 4);
    for function in guarded {
        let signature = format!(
            "define internal fastcc ptr @{}(i32 %arg0, i32 %arg1, ptr %arg2, ptr %arg3)",
            function.symbol
        );
        let start = llvm.find(&signature).unwrap();
        let remaining = &llvm[start..];
        let end = remaining[1..]
            .find("\ndefine ")
            .map_or(remaining.len(), |offset| offset + 1);
        let body = &remaining[..end];
        let identity = body.find("icmp eq i32 %arg0, %arg1").unwrap();
        let capture = body
            .find("call i32 @topal.runtime.int.compare(ptr %arg2, ptr %arg3)")
            .unwrap();
        assert!(identity < capture);
    }

    let ordered_program = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn (offset : Int, marker : String) -> Function\n  operation : Function is { value } (value + offset, marker)\n  operation\nrepeat : Function is { operation, operation } 42\nrepeat (make (1, \"same\"), make (1, \"same\"))\n",
        )
        .unwrap();
    let ordered = ordered_program
        .functions
        .iter()
        .find(|function| function.pattern_identities.len() == 3)
        .unwrap();
    let ordered_llvm = Generator::new(&ordered_program, "ordered-captures.t").emit();
    let signature = format!(
        "define internal fastcc ptr @{}(i32 %arg0, i32 %arg1, ptr %arg2, ptr %arg3, ptr %arg4, ptr %arg5)",
        ordered.symbol
    );
    let start = ordered_llvm.find(&signature).unwrap();
    let body = &ordered_llvm[start..];
    let source_identity = body.find("icmp eq i32 %arg0, %arg1").unwrap();
    let first_capture = body
        .find("call i1 @topal.runtime.string.equal(ptr %arg2, ptr %arg4)")
        .unwrap();
    let second_capture = body
        .find("call i32 @topal.runtime.int.compare(ptr %arg3, ptr %arg5)")
        .unwrap();
    assert!(source_identity < first_capture && first_capture < second_capture);

    assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
    assert!(llvm.contains("!DILocalVariable(name: \"operation\", arg: 1"));
    assert!(!llvm.contains("!DILocalVariable(name: \"operation\", arg: 2"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_captured_named_function_pattern_identity_as_direct_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
    // TOPAL-FUNCTION-NESTED-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-captured-named-function-patterns.t"
    ))
    .unwrap();
    let guarded = program
        .functions
        .iter()
        .find(|function| function.pattern_identities.len() == 2)
        .unwrap();
    let llvm = Generator::new(&program, "repeated-captured-named-function-patterns.t").emit();
    let signature = format!(
        "define internal fastcc ptr @{}(i32 %arg0, i32 %arg1, ptr %arg2, ptr %arg3)",
        guarded.symbol
    );
    let start = llvm.find(&signature).unwrap();
    let body = &llvm[start..];
    let source_identity = body.find("icmp eq i32 %arg0, %arg1").unwrap();
    let capture_identity = body
        .find("call i32 @topal.runtime.int.compare(ptr %arg2, ptr %arg3)")
        .unwrap();
    assert!(source_identity < capture_identity);
    assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
    assert!(llvm.contains("!DILocalVariable(name: \"operation\", arg: 1"));
    assert!(!llvm.contains("!DILocalVariable(name: \"operation\", arg: 2"));
    assert!(llvm.contains("@topal.fn.increase"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_captured_function_aggregate_pattern_identity_as_direct_guards() {
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
    // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
    // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/repeated-captured-function-aggregate-patterns.t"
    ))
    .unwrap();
    let guarded = program
        .functions
        .iter()
        .filter(|function| function.pattern_identities.len() == 2)
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 3);
    let llvm = Generator::new(&program, "repeated-captured-function-aggregate-patterns.t").emit();

    assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 6);
    for function in guarded {
        let start = llvm
            .find(&format!("define internal fastcc ptr @{}(", function.symbol))
            .unwrap();
        let remaining = &llvm[start..];
        let end = remaining[1..]
            .find("\ndefine ")
            .map_or(remaining.len(), |offset| offset + 1);
        let body = &remaining[..end];
        let source_identity = body.find("icmp eq i32").unwrap();
        let capture_identity = body
            .find("call i32 @topal.runtime.int.compare(ptr %arg2, ptr %arg3)")
            .unwrap();
        assert!(source_identity < capture_identity);
    }
    assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
    assert!(llvm.contains("!DILocalVariable(name: \"package\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
    assert!(!llvm.contains("!DILocalVariable(name: \"package\", arg: 2"));
    assert!(!llvm.contains("!DILocalVariable(name: \"value\", arg: 2"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_nested_functions_with_exact_private_capture_parameters() {
    // TOPAL-COMPILER-NESTED-FUNCTION-001, TOPAL-FUNCTION-NESTED-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/nested-functions.t"
    ))
    .unwrap();
    let symbol = |name: &str| {
        program
            .functions
            .iter()
            .find(|function| function.source_name == name)
            .unwrap()
            .symbol
            .as_str()
    };
    let nested = symbol("add-input");
    let outer = symbol("answer");
    let llvm = Generator::new(&program, "nested-functions.t").emit();

    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{nested}(ptr %arg0, ptr %arg1)"
    )));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr")
            && line.contains(&format!("@{nested}(ptr @.topal.int.0, ptr %arg0)"))
    }));
    assert!(llvm.contains(&format!("define internal fastcc ptr @{outer}(ptr %arg0)")));
    assert!(llvm.contains("!DISubprogram(name: \"add-input\""));
    assert!(llvm.contains("!DISubprogram(name: \"answer\""));
    assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"input\", arg: 2"));
    assert!(!llvm.contains("topal.runtime.closure"));
    assert!(!llvm.contains("topal.runtime.function"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_packaged_fields_as_an_exact_flat_private_signature() {
    // TOPAL-COMPILER-PACKAGED-OPERAND-001,
    // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/packaged-function-operand.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "packaged-function-operand.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{symbol}(ptr %arg0, ptr %arg1)"
    )));
    assert!(llvm.lines().any(|line| {
        line.contains("call fastcc ptr") && line.contains(&format!("@{symbol}(ptr"))
    }));
    assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
    assert!(llvm.contains("!DILocalVariable(name: \"fallback\", arg: 2"));
    assert!(!llvm.contains(" byval("));
    assert!(!llvm.contains(" sret("));
    assert!(!llvm.contains("topal.runtime.package"));
}

#[test]
fn emits_discarded_parameter_without_debug_binding() {
    // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-PATTERN-001
    let source = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (0, 42)\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/discard.t").emit();
    assert!(llvm.contains("define internal fastcc ptr @topal.fn.second.0(ptr %arg0, ptr %arg1)"));
    assert!(!llvm.contains("DILocalVariable(name: \"_\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\", arg: 2"));
}

#[test]
fn emits_result_decisions_and_native_string_error_metadata() {
    let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error problem then \"other\"\nproblem is 1.0 divide 0.0\n(describe 0.0, problem code, problem domain)\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/result-decision.t").emit();
    assert!(llvm.contains("%topal.StringStorage = type { ptr, i64, ptr, i64 }"));
    assert!(llvm.contains("call ptr @topal.runtime.string.make"));
    assert!(llvm.contains("result.decision.ok"));
    assert!(llvm.contains("result.decision.code"));
    assert!(llvm.contains("call i32 @topal.runtime.error.code"));
    assert!(llvm.contains("error.field.invalid"));
    assert!(llvm.contains("call ptr @topal.runtime.error.domain"));
    assert!(llvm.contains("name: \"String\""));
    assert!(llvm.contains("name: \"Error\""));
    assert!(llvm.contains("name: \"ErrorDomain\""));
    assert!(llvm.contains("name: \"lang arithmetic ArithmeticErrorCode\""));
}

#[test]
fn emits_optional_structured_error_fields_and_source_metadata() {
    // TOPAL-COMPILER-ERROR-OPTIONAL-FIELDS-001, TOPAL-ERROR-FIELD-001,
    // TOPAL-COMPILER-DEBUG-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/optional-result-composition.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "/source/optional-result-composition.t").emit();
    assert!(llvm.contains("%topal.SourceLocationStorage = type { ptr, ptr }"));
    assert!(llvm.contains("call ptr @topal.runtime.error.detail"));
    assert!(llvm.contains("call ptr @topal.runtime.error.cause"));
    assert!(llvm.contains("call ptr @topal.runtime.error.source"));
    assert!(llvm.contains("call ptr @topal.runtime.source.location.line"));
    assert!(llvm.contains("call ptr @topal.runtime.source.location.column"));
    assert!(llvm.contains("name: \"Optional Error\""));
    assert!(llvm.contains("name: \"Optional SourceLocation\""));
    assert!(llvm.contains("name: \"SourceLocation\""));
}

#[test]
fn emits_optional_values_decisions_equality_and_debug_metadata() {
    // TOPAL-COMPILER-OPTIONAL-001
    let source = include_str!("../../../../../examples/language/optional-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/optional-values.t").emit();
    assert!(llvm.contains("%topal.OptionalStorage = type { i64, ptr }"));
    assert!(llvm.contains("call ptr @topal.runtime.optional.some"));
    assert!(llvm.contains("call ptr @topal.runtime.optional.none"));
    assert!(llvm.contains("optional.decision.some"));
    assert!(llvm.contains("optional.decision.none"));
    assert!(llvm.contains("call i1 @topal.runtime.optional.int.equal"));
    assert!(llvm.contains("name: \"Optional Int\""));
    assert!(llvm.contains("name: \"Optional String\""));
}

#[test]
fn emits_optional_rational_values_with_exact_equality() {
    // TOPAL-COMPILER-OPTIONAL-RATIONAL-001
    let source = include_str!("../../../../../examples/language/optional-rational-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/optional-rational-values.t").emit();
    assert_eq!(
        llvm.matches("call i1 @topal.runtime.optional.rational.equal")
            .count(),
        6
    );
    assert!(llvm.contains("call i32 @topal.runtime.rational.compare"));
    assert!(llvm.contains("name: \"Optional Rational\""));
    assert!(llvm.contains("define internal fastcc ptr @topal.fn.preserve.0(ptr %arg0)"));
    assert!(llvm.contains("optional.decision.some"));
    assert!(llvm.contains("optional.decision.none"));
}

#[test]
fn emits_static_character_evidence_as_the_string_carrier() {
    // TOPAL-COMPILER-CHARACTER-001
    let source = include_str!("../../../../../examples/language/character-classification.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/character-classification.t").emit();
    assert!(llvm.contains("name: \"Character\""));
    assert!(llvm.contains("define internal fastcc ptr @topal.fn.identity.0(ptr %arg0)"));
    assert_eq!(
        llvm.matches("call i1 @topal.runtime.string.equal").count(),
        5
    );
    assert!(!llvm.contains("runtime.character.validate"));
}

#[test]
fn folds_closed_character_counting_and_indexing() {
    // TOPAL-COMPILER-CHARACTER-OBSERVATION-001
    let source = include_str!("../../../../../examples/language/string-character-at.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/string-character-at.t").emit();
    let generated = llvm
        .split_once("@topal.fn.")
        .expect("regression has a generated source function")
        .1;
    assert_eq!(
        generated
            .matches("call ptr @topal.runtime.optional.some")
            .count(),
        4
    );
    assert_eq!(
        generated
            .matches("call ptr @topal.runtime.optional.none")
            .count(),
        3
    );
    assert!(llvm.contains("name: \"Optional Character\""));
    assert!(llvm.contains("define internal fastcc ptr @topal.fn.describe.0(ptr %arg0)"));
    assert!(llvm.contains("optional.decision.some"));
    assert!(llvm.contains("optional.decision.none"));
    assert!(!llvm.contains("runtime.character.count"));
    assert!(!llvm.contains("runtime.character.at"));
}

#[test]
fn folds_closed_pinned_unicode_transformations() {
    // TOPAL-COMPILER-UNICODE-FOLD-001
    for (source, name, expected) in [
        (
            include_str!("../../../../../examples/language/string-uppercase.t"),
            "string-uppercase.t",
            "STRASSE ΣΣ",
        ),
        (
            include_str!("../../../../../examples/language/string-lowercase.t"),
            "string-lowercase.t",
            "i\u{307}ς",
        ),
        (
            include_str!("../../../../../examples/language/string-case-fold.t"),
            "string-case-fold.t",
            "strasse σσ",
        ),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, name).emit();
        assert!(llvm.contains(&llvm_bytes(expected.as_bytes())));
        assert!(!llvm.contains("runtime.unicode"));
    }
    for (source, name) in [
        (
            include_str!("../../../../../examples/language/string-normalization.t"),
            "string-normalization.t",
        ),
        (
            include_str!("../../../../../examples/language/string-normalization-nfd.t"),
            "string-normalization-nfd.t",
        ),
        (
            include_str!("../../../../../examples/language/string-canonical-equality.t"),
            "string-canonical-equality.t",
        ),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, name).emit();
        assert!(!llvm.contains("runtime.unicode"));
    }
}

#[test]
fn lowers_anonymous_records_without_a_runtime_abi() {
    // TOPAL-COMPILER-RECORD-001, TOPAL-TYPE-PRODUCT-001
    let source = include_str!("../../../../../examples/language/strings-and-products.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "strings-and-products.t").emit();
    assert!(llvm.contains(&llvm_bytes(b"name")));
    assert!(llvm.contains(&llvm_bytes(b"active")));
    assert!(llvm.contains(&llvm_bytes(b"Ada")));
    assert!(!llvm.contains("runtime.record"));
}

#[test]
fn lowers_record_reconstruction_without_a_runtime_abi() {
    // TOPAL-COMPILER-RECONSTRUCT-001, TOPAL-TYPE-RECONSTRUCT-001
    let source = include_str!("../../../../../examples/language/record-reconstruction.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "record-reconstruction.t").emit();
    assert!(llvm.contains(&llvm_bytes(b"Ada")));
    assert!(!llvm.contains("runtime.record"));
    assert!(!llvm.contains("runtime.reconstruct"));
}

#[test]
fn lowers_recursive_structural_comparisons() {
    // TOPAL-COMPILER-STRUCTURAL-COMPARISON-001
    let source = include_str!("../../../../../examples/language/equality-and-ordering.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "equality-and-ordering.t").emit();
    assert!(llvm.contains("tuple.compare.next"));
    assert!(llvm.contains("phi i32"));
    assert!(llvm.contains("@topal.runtime.rational.compare"));
    assert!(llvm.contains("@topal.runtime.string.equal"));
    assert!(!llvm.contains("runtime.tuple"));
    assert!(!llvm.contains("runtime.record"));
}

#[test]
fn emits_utf8_byte_counts_through_the_native_string_descriptor() {
    // TOPAL-COMPILER-STRING-UTF8-BYTE-COUNT-001
    let source = include_str!("../../../../../examples/language/string-utf8-byte-count.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/string-utf8-byte-count.t").emit();
    assert_eq!(
        llvm.matches("call ptr @topal.runtime.string.utf8.byte.count")
            .count(),
        3
    );
    assert!(llvm.contains("define internal ptr @topal.runtime.int.from.u64"));
    assert!(llvm.contains("lshr i64 %source, 32"));
    assert!(llvm.contains("select i1 %has.high, i64 2, i64 1"));
    assert!(llvm.contains("ret ptr @topal.runtime.int.zero"));
}

#[test]
fn emits_exact_string_and_derived_optional_string_equality() {
    // TOPAL-COMPILER-STRING-EQUALITY-001
    let source = include_str!("../../../../../examples/language/string-exact-equality.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/string-exact-equality.t").emit();
    assert_eq!(
        llvm.matches("call i1 @topal.runtime.string.equal").count(),
        5
    );
    assert_eq!(
        llvm.matches("call i1 @topal.runtime.optional.string.equal")
            .count(),
        4
    );
    assert!(llvm.contains("%same.length = icmp eq i64 %left.length, %right.length"));
    assert!(llvm.contains("%same.byte = icmp eq i8 %left.byte, %right.byte"));
}

#[test]
fn emits_freestanding_string_construction_concatenation_and_emptiness() {
    // TOPAL-COMPILER-STRING-CONSTRUCTION-001
    let source = include_str!("../../../../../examples/language/string-construction.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/string-construction.t").emit();
    assert_eq!(
        llvm.matches("call ptr @topal.runtime.string.concat")
            .count(),
        6
    );
    assert_eq!(
        llvm.matches("call i1 @topal.runtime.string.is.empty")
            .count(),
        2
    );
    assert!(llvm.contains("call void @topal.runtime.string.dynamic.print"));
    assert!(llvm.contains("call i1 @topal.runtime.string.has.delimiter"));
    assert!(llvm.contains("call ptr @topal.platform.allocate(i64 %allocation.length)"));
    assert_eq!(llvm.matches("call void @llvm.memcpy.inline").count(), 2);
    assert!(
        llvm.contains(
            "call ptr @topal.runtime.string.make(ptr %data, i64 %length, ptr null, i64 0)"
        )
    );
}

#[test]
fn emits_recursive_positional_product_equality_from_field_evidence() {
    // TOPAL-COMPILER-TUPLE-EQUALITY-001
    let source = include_str!("../../../../../examples/language/tuple-equality.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/tuple-equality.t").emit();
    assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 5);
    assert!(
        llvm.matches("call i32 @topal.runtime.rational.compare")
            .count()
            >= 3
    );
    assert!(llvm.matches("call i1 @topal.runtime.string.equal").count() >= 5);
    assert!(
        llvm.matches("call i1 @topal.runtime.optional.int.equal")
            .count()
            >= 2
    );
    assert!(
        llvm.matches("call i1 @topal.runtime.optional.string.equal")
            .count()
            >= 3
    );
    assert!(llvm.matches("and i1").count() >= 20);
}

#[test]
fn emits_distinct_overload_and_static_function_instances() {
    let source = "use language (version is v0.1)\ndescribe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  describe 42\nanswer is fn static () -> Int\n  42\nadd is fn static (left : Int, right : Int) -> Int\n  left + right\n(describe \"Topal\", answer (), 20 add 22)\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/function-overloads.t").emit();
    assert_eq!(llvm.matches("!DISubprogram(name: \"describe\"").count(), 2);
    assert!(llvm.contains("!DISubprogram(name: \"answer\""));
    assert!(llvm.contains("!DISubprogram(name: \"add\""));
    assert_eq!(
        llvm.matches("define internal fastcc ptr @topal.fn.describe")
            .count(),
        2
    );
}

#[test]
fn emits_same_named_cross_overload_edge_without_a_cycle() {
    // TOPAL-FUNCTION-RECURSION-OVERLOAD-IDENTITY-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/overload-recursion-identity.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "overload-recursion-identity.t").emit();
    let integer = "topal.fn.describe.0";
    let string = "topal.fn.describe.1";
    let integer_definition = llvm
        .find(&format!("define internal fastcc ptr @{integer}"))
        .unwrap();
    let string_definition = llvm
        .find(&format!("define internal fastcc ptr @{string}"))
        .unwrap();
    assert!(integer_definition < string_definition);
    assert_eq!(
        llvm.matches(&format!("call fastcc ptr @{integer}(ptr "))
            .count(),
        1
    );
    assert_eq!(
        llvm.matches(&format!("call fastcc ptr @{string}(ptr "))
            .count(),
        1
    );
    assert_eq!(llvm.matches("!DISubprogram(name: \"describe\"").count(), 2);
}

#[test]
fn emits_closed_mutual_int_cycles_with_exact_private_edges() {
    // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001,
    // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001
    for source in [
        include_str!("../../../../../examples/language/mutual-int-recursion.t"),
        include_str!("../../../../../examples/language/mutual-increasing-int-recursion.t"),
        include_str!("../../../../../examples/language/mutual-multiple-recursive-calls.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "mutual-int-recursion.t").emit();
        for function in &program.functions {
            assert_eq!(
                llvm.matches(&format!(
                    "define internal fastcc {} @{}(",
                    llvm_type(&function.result_type),
                    function.symbol
                ))
                .count(),
                1
            );
        }
        assert!(llvm.matches("call fastcc").count() >= program.functions.len());
        assert!(llvm.contains("nounwind noinline"));
        assert!(!llvm.contains("norecurse"));
    }
}

#[test]
fn emits_proven_mutual_nat_cycles_without_runtime_revalidation() {
    // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001,
    // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001
    for source in [
        include_str!("../../../../../examples/language/nat-mutual-recursion.t"),
        include_str!("../../../../../examples/language/nat-mutual-increasing-recursion.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "nat-mutual-recursion.t").emit();
        for function in &program.functions {
            assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
            assert_eq!(
                llvm.matches(&format!(
                    "define internal fastcc i1 @{}(ptr %arg0)",
                    function.symbol
                ))
                .count(),
                1
            );
        }
        assert!(llvm.matches("call fastcc i1").count() >= program.functions.len());
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.int.try.to.nat")
                .count(),
            0
        );
        assert!(llvm.contains("nounwind noinline"));
        assert!(!llvm.contains("norecurse"));
    }
}
