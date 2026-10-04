#[test]
fn emits_required_nat_validation_for_dynamic_list_insertion() {
    // TOPAL-NUM-NAT-001, TOPAL-COMPILER-NAT-ARITHMETIC-001
    let source = "use language (version is v0.1)\ndecrement is fn (values : List Nat) -> List Nat\n  values append ((entry-count values) - 1)\nempty : List Nat is Empty\ndecrement empty\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "dynamic-list-nat-boundary.t").emit();
    assert!(llvm.contains("call ptr @topal.runtime.int.try.to.nat"));
    assert!(llvm.contains("nat.boundary.error"));
    assert!(llvm.contains("call void @topal.platform.exit(i64 1)"));
}

#[test]
fn emits_forward_callee_before_its_caller() {
    // TOPAL-FUNCTION-FORWARD-DECLARATION-001, TOPAL-COMPILER-FUNCTION-001
    let source = include_str!("../../../../../examples/language/forward-function-declarations.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "forward-function-declarations.t").emit();
    let decorate_definition = llvm
        .find("define internal fastcc ptr @topal.fn.decorate.0")
        .unwrap();
    let render_definition = llvm
        .find("define internal fastcc ptr @topal.fn.render.1")
        .unwrap();
    assert!(decorate_definition < render_definition);
    assert!(llvm.contains("call fastcc ptr @topal.fn.decorate.0(ptr %arg0)"));
}

#[test]
fn emits_proven_direct_recursion_with_one_exact_private_signature() {
    // TOPAL-FUNCTION-RECURSION-INT-001
    for (source, name, recursive_calls) in [
        (
            include_str!("../../../../../examples/language/decreasing-int-recursion.t"),
            "decreasing-int-recursion.t",
            1,
        ),
        (
            include_str!("../../../../../examples/language/multiple-recursive-calls.t"),
            "multiple-recursive-calls.t",
            2,
        ),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, name).emit();
        assert_eq!(
            llvm.matches(&format!("define internal fastcc ptr @{symbol}"))
                .count(),
            1
        );
        assert_eq!(
            llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
                .count(),
            recursive_calls
        );
        assert!(llvm.contains("nounwind noinline"));
        assert!(!llvm.contains("norecurse"));
    }
}

#[test]
fn emits_proven_increasing_recursion_with_exact_private_signatures() {
    // TOPAL-FUNCTION-RECURSION-INT-INCREASING-001
    for source in [
        include_str!("../../../../../examples/language/increasing-int-recursion.t"),
        include_str!("../../../../../examples/language/positive-recursion-steps.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "increasing-int-recursion.t").emit();
        for function in &program.functions {
            let symbol = &function.symbol;
            assert_eq!(
                llvm.matches(&format!("define internal fastcc ptr @{symbol}"))
                    .count(),
                1
            );
            assert_eq!(
                llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
                    .count(),
                1
            );
        }
        assert!(llvm.contains("nounwind noinline"));
        assert!(!llvm.contains("norecurse"));
    }
}

#[test]
fn emits_proven_nat_recursion_without_runtime_revalidation() {
    // TOPAL-FUNCTION-RECURSION-NAT-001,
    // TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001
    for source in [
        include_str!("../../../../../examples/language/nat-recursion.t"),
        include_str!("../../../../../examples/language/nat-increasing-recursion.t"),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "nat-recursion.t").emit();
        assert_eq!(
            llvm.matches(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)"))
                .count(),
            1
        );
        assert_eq!(
            llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
                .count(),
            1
        );
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.int.try.to.nat")
                .count(),
            0
        );
        assert!(llvm.contains("nounwind noinline"));
        assert!(!llvm.contains("norecurse"));
    }
}

#[test]
fn emits_explicit_measure_recursion_with_one_complete_signature() {
    // TOPAL-FUNCTION-DECREASES-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/explicit-multi-parameter-decreases.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "explicit-multi-parameter-decreases.t").emit();
    assert_eq!(
        llvm.matches(&format!(
            "define internal fastcc ptr @{symbol}(ptr %arg0, ptr %arg1)"
        ))
        .count(),
        1
    );
    assert_eq!(
        llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call ptr @topal.runtime.int.try.to.nat")
            .count(),
        0
    );
    assert!(llvm.contains("nounwind noinline"));
    assert!(!llvm.contains("norecurse"));
}

#[test]
fn emits_euclidean_recursion_without_dynamic_arithmetic_checks() {
    // TOPAL-FUNCTION-RECURSION-EUCLIDEAN-001, TOPAL-NUM-INT-MODULO-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/euclidean-gcd.t"
    ))
    .unwrap();
    let symbol = &program.functions[0].symbol;
    let llvm = Generator::new(&program, "euclidean-gcd.t").emit();
    assert_eq!(
        llvm.matches(&format!(
            "define internal fastcc ptr @{symbol}(ptr %arg0, ptr %arg1)"
        ))
        .count(),
        1
    );
    assert_eq!(
        llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
            .count(),
        1
    );
    assert!(!llvm.contains("call ptr @topal.runtime.int.try.modulo"));
    assert!(!llvm.contains("call ptr @topal.runtime.int.try.to.nat"));
}

#[test]
fn emits_layout_policies_as_closed_nominal_tags_without_a_layout_runtime() {
    // TOPAL-LAYOUT-ENDIAN-001, TOPAL-LAYOUT-ACCESS-001,
    // TOPAL-LAYOUT-BIT-ORDER-001, TOPAL-LAYOUT-PACKING-001,
    // TOPAL-LAYOUT-FIELD-ORDER-001, TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001,
    // TOPAL-LAYOUT-ABSENCE-POLICY-001,
    // TOPAL-COMPILER-LAYOUT-POLICY-001
    let source = "use language (version is v0.1)\nendian is Big\naccess is Reserved\nbits is LeastSignificantFirst\npacking is Packed\nfields is Declared\npayload is Overlay\nabsence is NoTerminator\n(endian, access, bits, packing, fields, payload, absence)\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "layout-policy-values.t").emit();

    for type_name in [
        "Endian",
        "Access",
        "BitOrder",
        "Packing",
        "FieldOrder",
        "PayloadPlacement",
        "LayoutPolicy",
    ] {
        assert!(llvm.contains(&format!("DW_TAG_enumeration_type, name: \"{type_name}\"")));
    }
    for alternative in [
        "Big",
        "Reserved",
        "LeastSignificantFirst",
        "Packed",
        "Declared",
        "Overlay",
        "NoTerminator",
    ] {
        assert!(llvm.contains(&format!("DIEnumerator(name: \"{alternative}\"")));
    }
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    assert!(!main.contains("topal.platform.allocate"));
    assert!(!main.contains("topal.runtime.layout"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_generator_error_code_as_a_closed_nominal_tag_without_a_generator_runtime() {
    // TOPAL-GENERATOR-ERROR-CODE-001,
    // TOPAL-COMPILER-GENERATOR-ERROR-CODE-001
    let program = analyze_for_compiler(include_str!(
        "../../../../../examples/language/generator-error-codes.t"
    ))
    .unwrap();
    let llvm = Generator::new(&program, "generator-error-codes.t").emit();

    assert!(llvm.contains("DW_TAG_enumeration_type, name: \"lang generator GeneratorErrorCode\""));
    assert!(llvm.contains("DIEnumerator(name: \"generator-closed\", value: 0)"));
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("topal.platform.allocate"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_lazy_iterate_construction_as_a_private_debug_token_without_invoking_functions() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001
    for (source, name) in [
        (
            include_str!("../../../../../examples/language/iterate-generator.t"),
            "iterate-generator.t",
        ),
        (
            include_str!("../../../../../examples/language/iterate-take-while.t"),
            "iterate-take-while.t",
        ),
    ] {
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, name).emit();

        assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Generator Int Unit Unit\""));
        assert!(llvm.contains("DIEnumerator(name: \"<Generator Int Unit Unit>\", value: 0)"));
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("topal.platform.allocate"));
        assert!(!main.contains("topal.runtime.int.add"));
        assert!(!main.contains("icmp"));
        assert!(!main.contains("call ptr %"));
    }

    let once = analyze_for_compiler(
            "use language (version is v0.1)\ninitial is fn () -> Int\n  0\nnumbers is (initial ()) iterate ({ value } value + 1)\nnumbers\n",
        )
        .unwrap();
    let llvm = Generator::new(&once, "iterate-initial-once.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    assert_eq!(
        main.matches("call fastcc ptr @topal.fn.initial.").count(),
        1
    );
}

#[test]
fn emits_bounded_iterate_collection_as_an_ordered_list_loop() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001
    let source = include_str!("../../../../../examples/language/generated-collect.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "generated-collect.t").emit();

    for expected in [
        "generator.collect.loop",
        "generator.collect.accepted",
        "generator.collect.done",
        "call ptr @topal.platform.allocate(i64 16)",
        "call ptr @topal.runtime.int.add",
        "icmp slt i32",
        "List Int",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}: {llvm}");
    }
    assert!(!llvm.contains("topal.runtime.generator"));
    assert!(!llvm.contains("call ptr %"));
}

#[test]
fn emits_bounded_iterate_foreach_as_an_ordered_unit_loop() {
    // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
    // TOPAL-GENERATOR-ITERATE-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001
    let source = include_str!("../../../../../examples/language/generated-foreach.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "generated-foreach.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    for expected in [
        "generator.foreach.loop",
        "phi ptr",
        "call i32 @topal.runtime.int.compare",
        "generator.foreach.accepted",
        "call ptr @topal.runtime.int.add",
        "generator.foreach.resumed",
        "generator.foreach.done",
        "#dbg_value(i8 0",
    ] {
        assert!(main.contains(expected), "missing {expected:?}: {main}");
    }
    let predicate = main.find("call i32 @topal.runtime.int.compare").unwrap();
    let action = main.find("generator.foreach.accepted").unwrap();
    let next = main.find("call ptr @topal.runtime.int.add").unwrap();
    assert!(predicate < action && action < next);
    assert!(!main.contains("topal.platform.allocate"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_closed_string_character_collection_as_exact_source_identity() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001
    let source = include_str!("../../../../../examples/language/string-character-traversal.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "string-character-traversal.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(llvm.contains(&format!("c\"{}\"", llvm_bytes("a\u{301}👩‍🔬🇸🇪".as_bytes()))));
    assert!(llvm.contains("name: \"String\""));
    assert!(!main.contains("topal.runtime.string.concat"));
    assert!(!main.contains("topal.runtime.list"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_closed_string_character_foreach_in_preserved_order() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001
    let source = include_str!("../../../../../examples/language/string-character-foreach.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "string-character-foreach.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        4
    );
    let mut previous = 0;
    for character in ["a\u{301}", "👩‍🔬", "🇸🇪"] {
        let encoded = format!("c\"{}\"", llvm_bytes(character.as_bytes()));
        let position = llvm[previous..].find(&encoded).map_or_else(
            || panic!("missing {character:?} bytes: {llvm}"),
            |position| previous + position,
        );
        assert!(position >= previous);
        previous = position + encoded.len();
    }
    assert!(main.contains("alloca ptr, align 8"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert_eq!(main.matches("store ptr").count(), 3);
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));

    let empty = analyze_for_compiler(
            "use language (version is v0.1)\ncharacters \"\" foreach { character }\n  _ is String character\n",
        )
        .unwrap();
    let empty_llvm = Generator::new(&empty, "empty-string-character-foreach.t").emit();
    let empty_main = empty_llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    assert_eq!(
        empty_main
            .matches("call ptr @topal.runtime.string.make")
            .count(),
        1
    );
}

#[test]
fn emits_named_string_character_generator_as_a_private_linear_token() {
    // TOPAL-STRING-CHARACTERS-COLLECT-001,
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-CLASSIFIER-001,
    // TOPAL-STRING-CHARACTERS-LINEAR-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-GENERATOR-001
    let source =
        include_str!("../../../../../examples/language/string-named-character-generator.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "string-named-character-generator.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        4
    );
    assert!(main.contains("#dbg_value(i32 0"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_single_yield_custom_generator_without_runtime_state() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001
    let source = include_str!("../../../../../examples/language/custom-single-yield-generator.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-single-yield-generator.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert!(main.contains("#dbg_value(i32 0"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_custom_generator_string_input_prefix_before_suspension() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-GENERATOR-STRING-INPUT-001
    let source = include_str!("../../../../../examples/language/custom-generator-string-input.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-string-input.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert_eq!(
        main.matches("call i1 @topal.runtime.string.is.empty")
            .count(),
        1
    );
    let input = main
        .find("call ptr @topal.runtime.string.make")
        .expect("generator application evaluates its String input");
    let predicate = main
        .find("call i1 @topal.runtime.string.is.empty")
        .expect("generator application executes the retained prefix");
    let suspended = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private suspended token");
    let yielded = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("traversal materializes the yielded Character");
    assert!(input < predicate && predicate < suspended && suspended < yielded);
    assert!(main.contains("#dbg_value(ptr"));
    assert!(main.contains("#dbg_value(i1"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"initial-is-empty\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_custom_generator_string_yields_without_runtime_state() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
    // TOPAL-COMPILER-GENERATOR-STRING-YIELD-001
    let source = include_str!("../../../../../examples/language/custom-generator-string-yield.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-string-yield.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert_eq!(
        main.matches("call i1 @topal.runtime.string.is.empty")
            .count(),
        2
    );
    let input = main
        .find("call ptr @topal.runtime.string.make")
        .expect("generator application evaluates its String input");
    let suspended = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private suspended token");
    let first_action = main
        .find("call i1 @topal.runtime.string.is.empty")
        .expect("the first yielded String reaches the action");
    let literal = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("the second suspension materializes its String literal");
    let second_action = main
        .rfind("call i1 @topal.runtime.string.is.empty")
        .expect("the second yielded String reaches the action");
    assert!(
        input < suspended
            && suspended < first_action
            && first_action < literal
            && literal < second_action
    );
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"text\""));
    assert!(llvm.contains("name: \"Generator String Unit Unit\""));
    assert!(llvm.contains("name: \"String\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_custom_generator_final_string_after_resumption() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-STRING-001
    let source = include_str!("../../../../../examples/language/custom-generator-string-return.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-string-return.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert_eq!(
        main.matches("call i1 @topal.runtime.string.is.empty")
            .count(),
        1
    );
    let input = main
        .find("call ptr @topal.runtime.string.make")
        .expect("generator application evaluates its String input");
    let suspended = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private suspended token");
    let action = main
        .find("call i1 @topal.runtime.string.is.empty")
        .expect("the yielded String reaches the action");
    let result = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("traversal materializes the distinct final String");
    let output = main
        .find("call void @topal.runtime.string.print")
        .expect("the final String is observed as the program result");
    assert!(input < suspended && suspended < action && action < result && result < output);
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"text\""));
    assert!(llvm.contains("name: \"Generator String Unit String\""));
    assert!(llvm.contains("name: \"String\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_discarded_computation_between_string_yields() {
    // TOPAL-GENERATOR-BODY-STATEMENT-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-RESUME-DISCARD-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-discard-between-yields.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-discard-between-yields.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    let predicates = main
        .match_indices("call i1 @topal.runtime.string.is.empty")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(predicates.len(), 3);
    let input = main
        .find("call ptr @topal.runtime.string.make")
        .expect("generator application evaluates its String input");
    let suspended = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private suspended token");
    let literal = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("the second suspension materializes its String literal");
    assert!(
        input < suspended
            && suspended < predicates[0]
            && predicates[0] < predicates[1]
            && predicates[1] < literal
            && literal < predicates[2]
    );
    assert!(main.contains("#dbg_value(ptr"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"text\""));
    assert!(llvm.contains("name: \"Generator String Unit Unit\""));
    assert!(llvm.contains("name: \"String\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_explicit_string_return_without_invoking_the_action() {
    // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-FINAL-RETURN-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-EXPLICIT-RETURN-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-explicit-return.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-explicit-return.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert_eq!(
        main.matches("call i1 @topal.runtime.string.is.empty")
            .count(),
        0
    );
    let input = main
        .find("call ptr @topal.runtime.string.make")
        .expect("generator application evaluates its String input");
    let completed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private completed token");
    let result = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("traversal materializes the explicit String return");
    let output = main
        .find("call void @topal.runtime.string.print")
        .expect("the explicit return becomes the program result");
    assert!(input < completed && completed < result && result < output);
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(!llvm.contains("DILocalVariable(name: \"text\""));
    assert!(llvm.contains("name: \"Generator String Unit String\""));
    assert!(llvm.contains("name: \"String\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_explicit_string_return_after_yield_action_and_resumption() {
    // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-RESUMPTION-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-RETURN-AFTER-YIELD-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-return-after-yield.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-return-after-yield.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert_eq!(
        main.matches("call i1 @topal.runtime.string.is.empty")
            .count(),
        1
    );
    let input = main
        .find("call ptr @topal.runtime.string.make")
        .expect("generator application evaluates its String input");
    let suspended = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private suspended token");
    let action = main
        .find("call i1 @topal.runtime.string.is.empty")
        .expect("foreach invokes the String action");
    let result = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("resumed generator materializes its explicit String return");
    let output = main
        .find("call void @topal.runtime.string.print")
        .expect("the explicit return becomes the program result");
    assert!(input < suspended && suspended < action && action < result && result < output);
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"text\""));
    assert!(llvm.contains("name: \"Generator String Unit String\""));
    assert!(llvm.contains("name: \"String\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_boolean_values_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-BOOLEAN-001
    let source = include_str!("../../../../../examples/language/custom-generator-boolean-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-boolean-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(main.matches("xor i1 true, true").count(), 2);
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let yielded = main
        .find("store i1 true")
        .expect("foreach retains the yielded Boolean for debugging");
    let negations = main
        .match_indices("xor i1 true, true")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let output = main
        .find("br i1 %")
        .expect("the final Boolean controls Topal-owned display");
    assert!(
        constructed < yielded
            && yielded < negations[0]
            && negations[0] < negations[1]
            && negations[1] < output
    );
    assert!(main.contains("alloca i1, align 1"));
    assert_eq!(main.matches("store i1 true").count(), 2);
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(main.contains("#dbg_value(i1 true"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\""));
    assert!(llvm.contains("name: \"Generator Boolean Unit Boolean\""));
    assert!(llvm.contains("name: \"Boolean\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_custom_generator_final_boolean_decision_after_resumption() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-DECISION-001
    let source = include_str!("../../../../../examples/language/custom-generator-final-decision.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-final-decision.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let yielded = main
        .find("store i1 true")
        .expect("foreach retains the yielded Boolean for debugging");
    let action = main
        .find("xor i1 true, true")
        .expect("foreach action executes before the generator final decision");
    let decision = main
        .find("br i1 true, label %decision.true")
        .expect("final result retains its O0 Boolean decision");
    let selected = main
        .find("phi ptr")
        .expect("the final decision joins its two String actions");
    let output = main
        .find("call void @topal.runtime.string.print")
        .expect("the selected final String reaches Topal-owned display");
    assert!(
        constructed < yielded
            && yielded < action
            && action < decision
            && decision < selected
            && selected < output
    );
    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert!(main.contains("decision.true"));
    assert!(main.contains("decision.false"));
    assert!(main.contains("decision.merge"));
    assert!(main.contains("alloca i1, align 1"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(main.contains("#dbg_value(i1 true"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\""));
    assert!(llvm.contains("name: \"Generator Boolean Unit String\""));
    assert!(llvm.contains("name: \"Boolean\""));
    assert!(llvm.contains("name: \"String\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_generator_local_enum_and_direct_function_after_resumption() {
    // TOPAL-GENERATOR-LOCAL-FUNCTION-001, TOPAL-GENERATOR-LOCAL-ENUM-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-FUNCTION-ORDINARY-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-FUNCTION-001
    let source = include_str!("../../../../../examples/language/custom-generator-local-function.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-local-function.t").emit();
    let local = llvm
        .split_once("define internal fastcc ptr @topal.fn.label.0")
        .expect("module contains the private generator-local function")
        .1
        .split_once("\n}\n")
        .expect("local function has a complete definition")
        .0;
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert!(local.contains("(i32 %arg0) nounwind noinline"));
    assert!(local.contains("#dbg_value(i32 %arg0"));
    assert!(local.contains("switch i32 %arg0"));
    assert!(local.contains("i32 0, label %enum.decision.alternative.0"));
    assert!(local.contains("i32 1, label %enum.decision.alternative.1"));
    assert_eq!(
        local.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert!(local.contains("phi ptr"));
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let yielded = main
        .find("store i1 true")
        .expect("foreach retains the yielded Boolean for debugging");
    let action = main
        .find("xor i1 true, true")
        .expect("foreach action executes before resumption");
    let call = main
        .find("call fastcc ptr @topal.fn.label.0(i32 0)")
        .expect("resumption invokes the retained local function directly");
    let output = main
        .find("call void @topal.runtime.string.print")
        .expect("the local function result reaches Topal-owned display");
    assert!(constructed < yielded && yielded < action && action < call && call < output);
    assert!(llvm.contains("DISubprogram(name: \"label\""));
    assert!(llvm.contains("linkageName: \"topal.fn.label.0\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\", arg: 1"));
    assert!(llvm.contains("name: \"Choice\""));
    assert!(llvm.contains("name: \"Accepted\""));
    assert!(llvm.contains("name: \"Rejected\""));
    assert!(llvm.contains("name: \"Generator Boolean Unit String\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
    assert!(!llvm.contains("define fastcc ptr @topal.fn.label.0"));
}

#[test]
fn emits_arbitrary_precision_ints_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-INT-001
    let source = include_str!("../../../../../examples/language/custom-generator-int-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-int-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(main.matches("call ptr @topal.runtime.int.add(").count(), 2);
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store ptr @.topal.int.0")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let additions = main
        .match_indices("call ptr @topal.runtime.int.add(")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let output = main
        .find("call void @topal.runtime.int.print")
        .expect("the final exact Int controls Topal-owned display");
    assert_eq!(debug_stores.len(), 2);
    assert!(
        constructed < debug_stores[0]
            && debug_stores[0] < additions[0]
            && additions[0] < debug_stores[1]
            && debug_stores[1] < additions[1]
            && additions[1] < output
    );
    assert!(main.contains("alloca ptr, align 8"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\""));
    assert!(llvm.contains("name: \"Generator Int Unit Int\""));
    assert!(llvm.contains("name: \"Int\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_nonnegative_nats_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-NUM-NAT-CONSTRUCT-001, TOPAL-COMPILER-GENERATOR-NAT-001
    let source = include_str!("../../../../../examples/language/custom-generator-nat-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-nat-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(main.matches("call ptr @topal.runtime.int.add(").count(), 2);
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store ptr @.topal.int.0")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let additions = main
        .match_indices("call ptr @topal.runtime.int.add(")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let output = main
        .find("call void @topal.runtime.int.print")
        .expect("the final exact Nat controls Topal-owned display");
    assert_eq!(debug_stores.len(), 2);
    assert!(
        constructed < debug_stores[0]
            && debug_stores[0] < additions[0]
            && additions[0] < debug_stores[1]
            && debug_stores[1] < additions[1]
            && additions[1] < output
    );
    assert!(main.contains("alloca ptr, align 8"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(!main.contains("@topal.runtime.int.try.to.nat"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\""));
    assert!(llvm.contains("name: \"Generator Nat Unit Nat\""));
    assert!(llvm.contains("name: \"Nat\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_nominal_enum_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-ENUM-001, TOPAL-COMPILER-GENERATOR-ENUM-001
    let source = include_str!("../../../../../examples/language/custom-generator-enum-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-enum-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store i32 0")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let action = main
        .find("icmp eq i32 0, 0")
        .expect("foreach compares the yielded nominal alternative");
    let output = main
        .find("switch i32 1")
        .expect("the distinct final alternative controls Topal-owned display");
    assert_eq!(debug_stores.len(), 4);
    assert!(
        constructed < debug_stores[0]
            && debug_stores[0] < debug_stores[1]
            && debug_stores[1] < action
            && action < debug_stores[2]
            && debug_stores[2] < debug_stores[3]
            && debug_stores[3] < output
    );
    assert!(main.contains("alloca i32, align 4"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"choice\""));
    assert!(llvm.contains("name: \"Generator Choice Unit Choice\""));
    assert!(llvm.contains("name: \"Choice\""));
    assert!(llvm.contains("DIEnumerator(name: \"First\", value: 0)"));
    assert!(llvm.contains("DIEnumerator(name: \"Second\", value: 1)"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_ordered_product_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-PRODUCT-001, TOPAL-COMPILER-GENERATOR-PRODUCT-001
    let source = include_str!("../../../../../examples/language/custom-generator-product-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-product-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store { ptr, ptr }")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let int_equal = main
        .find("call i32 @topal.runtime.int.compare")
        .expect("foreach compares the yielded Int field");
    let string_equal = main
        .find("call i1 @topal.runtime.string.equal")
        .expect("foreach compares the yielded String field");
    let output = main
        .find("call void @topal.runtime.int.print")
        .expect("the distinct final product controls Topal-owned display");
    assert_eq!(debug_stores.len(), 4);
    assert_eq!(main.matches("insertvalue { ptr, ptr }").count(), 4);
    assert_eq!(main.matches("alloca { ptr, ptr }, align 8").count(), 2);
    assert!(
        constructed < debug_stores[0]
            && debug_stores[0] < debug_stores[1]
            && debug_stores[1] < int_equal
            && int_equal < string_equal
            && string_equal < debug_stores[2]
            && debug_stores[2] < debug_stores[3]
            && debug_stores[3] < output
    );
    assert!(main.contains("and i1"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\""));
    assert!(llvm.contains("name: \"Generator (Int, String) Unit (Int, String)\""));
    assert!(llvm.contains("name: \"(Int, String)\""));
    assert!(llvm.contains("name: \"_0\""));
    assert!(llvm.contains("name: \"_1\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_result_rational_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RESULT-001
    let source = include_str!("../../../../../examples/language/custom-generator-result-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-result-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.rational.from.int")
            .count(),
        3
    );
    assert_eq!(
        main.matches("call ptr @topal.runtime.result.success")
            .count(),
        1
    );
    assert_eq!(
        main.matches("call ptr @topal.runtime.rational.try.divide")
            .count(),
        1
    );
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store ptr")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let division = main
        .find("call ptr @topal.runtime.rational.try.divide")
        .expect("the final Result performs the checked division");
    assert_eq!(debug_stores.len(), 4);
    assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
    assert!(
        constructed < debug_stores[0]
            && debug_stores[0] < debug_stores[1]
            && debug_stores[1] < debug_stores[2]
            && debug_stores[2] < debug_stores[3]
            && debug_stores[3] < division
    );
    assert!(main.contains("@topal.runtime.result.is.error"));
    assert!(main.contains("@topal.runtime.error.print"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
    assert!(llvm.contains("name: \"Generator Result (Rational, lang arithmetic ArithmeticErrorCode) Unit Result (Rational, lang arithmetic ArithmeticErrorCode)\""));
    assert!(llvm.contains("name: \"Result (Rational, lang arithmetic ArithmeticErrorCode)\""));
    assert!(!main.contains("result.project"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_comparison_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-DECISION-COMPARISON-001, TOPAL-COMPILER-GENERATOR-COMPARISON-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-comparison-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-comparison-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call i32 @topal.runtime.int.compare").count(),
        3
    );
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store i32")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let final_comparison = main
        .rfind("call i32 @topal.runtime.int.compare")
        .expect("the final Comparison is evaluated after resumption");
    assert_eq!(debug_stores.len(), 4);
    assert_eq!(main.matches("alloca i32, align 4").count(), 2);
    assert!(
        constructed < debug_stores[0]
            && debug_stores[0] < debug_stores[1]
            && debug_stores[1] < debug_stores[2]
            && debug_stores[2] < debug_stores[3]
            && debug_stores[3] < final_comparison
    );
    assert!(main.contains("icmp eq i32"));
    assert!(main.contains("switch i32"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"comparison\""));
    assert!(llvm.contains("name: \"Generator Comparison Unit Comparison\""));
    assert!(llvm.contains("name: \"Comparison\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_exact_rationals_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-RATIONAL-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-rational-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-rational-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.rational.make(")
            .count(),
        3
    );
    assert_eq!(
        main.matches("call ptr @topal.runtime.rational.add(")
            .count(),
        2
    );
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store ptr %v0")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let rational_constructions = main
        .match_indices("call ptr @topal.runtime.rational.make(")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let additions = main
        .match_indices("call ptr @topal.runtime.rational.add(")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let output = main
        .find("call void @topal.runtime.rational.print")
        .expect("the final exact Rational controls Topal-owned display");
    assert_eq!(debug_stores.len(), 2);
    assert!(
        rational_constructions[0] < constructed
            && constructed < debug_stores[0]
            && debug_stores[0] < rational_constructions[1]
            && rational_constructions[1] < additions[0]
            && additions[0] < debug_stores[1]
            && debug_stores[1] < rational_constructions[2]
            && rational_constructions[2] < additions[1]
            && additions[1] < output
    );
    assert!(main.contains("alloca ptr, align 8"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"value\""));
    assert!(llvm.contains("name: \"Generator Rational Unit Rational\""));
    assert!(llvm.contains("name: \"Rational\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_unit_across_every_custom_generator_direction() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-UNIT-001
    let source = include_str!("../../../../../examples/language/custom-generator-unit-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-unit-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let slots = main
        .match_indices("alloca i8, align 1")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let stores = main
        .match_indices("store i8 0")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let output = main
        .find("call void @topal.platform.write_all")
        .expect("the final Unit controls Topal-owned display");
    assert_eq!(slots.len(), 2);
    assert_eq!(stores.len(), 4);
    assert!(
        constructed < slots[0]
            && slots[0] < stores[0]
            && stores[0] < stores[1]
            && stores[1] < slots[1]
            && slots[1] < stores[2]
            && stores[2] < stores[3]
            && stores[3] < output
    );
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"signal\""));
    assert!(llvm.contains("name: \"Generator Unit Unit Unit\""));
    assert!(llvm.contains("name: \"Unit\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("topal.platform.allocate"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_optional_int_values_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-COMPILER-GENERATOR-OPTIONAL-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-optional-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-optional-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.optional.some(")
            .count(),
        2
    );
    assert_eq!(
        main.matches("call i1 @topal.runtime.optional.int.equal(")
            .count(),
        1
    );
    assert_eq!(
        main.matches("call ptr @topal.runtime.optional.none()")
            .count(),
        1
    );
    let initial = main
        .find("call ptr @topal.runtime.optional.some(")
        .expect("generator application evaluates its Optional input");
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store ptr %v0")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let action_some = main
        .match_indices("call ptr @topal.runtime.optional.some(")
        .nth(1)
        .expect("foreach constructs the canonical Optional action operand")
        .0;
    let action = main
        .find("call i1 @topal.runtime.optional.int.equal(")
        .expect("foreach compares the yielded Optional value");
    let result = main
        .find("call ptr @topal.runtime.optional.none()")
        .expect("generator materializes its final Optional alternative");
    let output = main
        .find("call i1 @topal.runtime.optional.is.some(")
        .expect("the final Optional controls Topal-owned display");
    assert_eq!(debug_stores.len(), 2);
    assert!(
        initial < constructed
            && constructed < debug_stores[0]
            && debug_stores[0] < action_some
            && action_some < action
            && action < debug_stores[1]
            && debug_stores[1] < result
            && result < output
    );
    assert!(main.contains("alloca ptr, align 8"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
    assert!(llvm.contains("name: \"Generator Optional Int Unit Optional Int\""));
    assert!(llvm.contains("name: \"Optional Int\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_nested_optional_product_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-OPTIONAL-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-nested-optional-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-nested-optional-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let boxes = main
        .match_indices("call ptr @topal.platform.allocate(i64 16)")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let optional_values = main
        .match_indices("call ptr @topal.runtime.optional.some(")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(boxes.len(), 3);
    assert_eq!(optional_values.len(), 3);
    assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let action = main
        .find("optional.product.equal.payload")
        .expect("foreach structurally compares the Optional product payload");
    let output = main
        .rfind("call i1 @topal.runtime.optional.is.some(")
        .expect("the final Optional controls Topal-owned display");
    assert!(
        boxes[0] < optional_values[0]
            && optional_values[0] < constructed
            && constructed < boxes[1]
            && boxes[1] < optional_values[1]
            && optional_values[1] < action
            && action < boxes[2]
            && boxes[2] < optional_values[2]
            && optional_values[2] < output
    );
    assert!(main.contains("call i32 @topal.runtime.int.compare"));
    assert!(main.contains("call i1 @topal.runtime.string.equal"));
    assert!(main.contains("phi i1"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
    assert!(
        llvm.contains("name: \"Generator Optional (Int, String) Unit Optional (Int, String)\"")
    );
    assert!(llvm.contains("name: \"Optional (Int, String)\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_nested_absent_optional_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-NONE-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-nested-none-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-nested-none-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let absent = main
        .match_indices("call ptr @topal.runtime.optional.none()")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(absent.len(), 3);
    assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
    assert!(!main.contains("call ptr @topal.platform.allocate(i64 16)"));
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let action = main
        .find("optional.product.equal.payload")
        .expect("foreach retains the guarded structural equality path");
    let output = main
        .rfind("call i1 @topal.runtime.optional.is.some(")
        .expect("the final absent Optional controls Topal-owned display");
    assert!(
        absent[0] < constructed
            && constructed < absent[1]
            && absent[1] < action
            && action < absent[2]
            && absent[2] < output
    );
    assert!(main.contains("and i1"));
    assert!(main.contains("optional.product.equal.tag"));
    assert!(main.contains("phi i1"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
    assert!(
        llvm.contains("name: \"Generator Optional (Int, String) Unit Optional (Int, String)\"")
    );
    assert!(llvm.contains("name: \"Optional (Int, String)\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_nested_result_product_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-TYPE-PRODUCT-001,
    // TOPAL-COMPILER-GENERATOR-NESTED-RESULT-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-nested-result-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-nested-result-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let boxes = main
        .match_indices("call ptr @topal.platform.allocate(i64 16)")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let results = main
        .match_indices("call ptr @topal.runtime.result.success(")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(boxes.len(), 3);
    assert_eq!(results.len(), 3);
    assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let action = main
        .find("result.product.equal.payload")
        .expect("foreach structurally compares the successful product payload");
    let output = main
        .rfind("call i1 @topal.runtime.result.is.error(")
        .expect("the final Result controls Topal-owned display");
    assert!(
        boxes[0] < results[0]
            && results[0] < constructed
            && constructed < boxes[1]
            && boxes[1] < results[1]
            && results[1] < action
            && action < boxes[2]
            && boxes[2] < results[2]
            && results[2] < output
    );
    assert!(main.contains("call i32 @topal.runtime.int.compare"));
    assert!(main.contains("call i1 @topal.runtime.string.equal"));
    assert!(main.contains("phi i1"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
    assert!(llvm.contains(
            "name: \"Generator Result ((Int, String), lang arithmetic ArithmeticErrorCode) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)\""
        ));
    assert!(llvm.contains("name: \"Result ((Int, String), lang arithmetic ArithmeticErrorCode)\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}
