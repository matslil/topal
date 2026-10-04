#[test]
fn emits_recursive_nominal_values_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-TYPE-ENUM-001, TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
    // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RECURSIVE-NOMINAL-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-recursive-nominal-values.t"
    );
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-recursive-nominal-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let boxes = main
        .match_indices("call ptr @topal.platform.allocate(i64 4)")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(boxes.len(), 6);
    assert_eq!(main.matches("store i32 0").count(), 4);
    assert_eq!(main.matches("store i32 1").count(), 2);
    assert_eq!(main.matches("alloca { ptr, ptr }, align 8").count(), 2);
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let optional_action = main
        .find("optional.product.equal.payload")
        .expect("Optional equality observes the nominal payload only after its tag");
    let result_action = main
        .find("result.product.equal.payload")
        .expect("Result equality observes the nominal success only after its tag");
    let output = main
        .rfind("call i1 @topal.runtime.optional.is.some(")
        .expect("the final recursive value controls Topal-owned display");
    assert!(
        boxes[0] < boxes[1]
            && boxes[1] < constructed
            && constructed < boxes[2]
            && boxes[2] < boxes[3]
            && boxes[3] < optional_action
            && optional_action < result_action
            && result_action < boxes[4]
            && boxes[4] < boxes[5]
            && boxes[5] < output
    );
    assert!(main.contains("optional.product.equal.tag"));
    assert!(main.contains("result.product.equal.error"));
    assert!(main.matches("load i32, ptr").count() >= 6);
    assert_eq!(main.matches("icmp eq i32").count(), 2);
    assert!(main.contains("phi i1"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
    assert!(llvm.contains(
            "name: \"Generator (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)) Unit (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\""
        ));
    assert!(llvm.contains(
        "name: \"(Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\""
    ));
    assert!(llvm.contains("name: \"Optional Choice\""));
    assert!(llvm.contains("name: \"Result (Choice, lang arithmetic ArithmeticErrorCode)\""));
    assert!(llvm.contains("name: \"Choice\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_int_ranges_across_custom_generator_directions() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-RANGE-BOUNDS-001, TOPAL-RANGE-CLASSIFIER-001,
    // TOPAL-COMPILER-GENERATOR-RANGE-001
    let source = include_str!("../../../../../examples/language/custom-generator-range-values.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-range-values.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.range.make(").count(),
        2
    );
    assert_eq!(
        main.matches("call i1 @topal.runtime.range.int.contains(")
            .count(),
        1
    );
    assert_eq!(
        main.matches("call ptr @topal.runtime.range.int.intersection(")
            .count(),
        1
    );
    let initial = main
        .find("call ptr @topal.runtime.range.make(")
        .expect("generator application evaluates its Range input");
    let constructed = main
        .find("#dbg_value(i32 0")
        .expect("generator application retains its private token");
    let debug_stores = main
        .match_indices("store ptr %v0")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let action = main
        .find("call i1 @topal.runtime.range.int.contains(")
        .expect("foreach evaluates membership over the yielded Range");
    let final_range = main
        .match_indices("call ptr @topal.runtime.range.make(")
        .nth(1)
        .expect("generator constructs its exact final intersection operand")
        .0;
    let intersection = main
        .find("call ptr @topal.runtime.range.int.intersection(")
        .expect("generator computes its final narrowed Range");
    let output = main
        .find("call void @topal.runtime.range.int.print(")
        .expect("the final Range controls Topal-owned display");
    assert_eq!(debug_stores.len(), 2);
    assert!(
        initial < constructed
            && constructed < debug_stores[0]
            && debug_stores[0] < action
            && action < debug_stores[1]
            && debug_stores[1] < final_range
            && final_range < intersection
            && intersection < output
    );
    assert!(main.contains("alloca ptr, align 8"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("DILocalVariable(name: \"interval\""));
    assert!(llvm.contains("name: \"Generator Range Int Unit Range Int\""));
    assert!(llvm.contains("name: \"Range Int\""));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_multiple_yield_custom_generator_in_source_order() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-MULTIPLE-YIELD-001
    let source = include_str!("../../../../../examples/language/custom-multiple-yield-generator.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-multiple-yield-generator.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        3
    );
    assert_eq!(main.matches("store ptr").count(), 2);
    assert!(main.contains("#dbg_value(i32 0"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_custom_generator_early_unit_return_without_action() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-EARLY-RETURN-001,
    // TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-EARLY-RETURN-001
    let source = include_str!("../../../../../examples/language/custom-generator-early-return.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-early-return.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(main.contains("#dbg_value(i32 0"));
    assert!(!main.contains("#dbg_declare(ptr"));
    assert!(!main.contains("store ptr"));
    assert!(!llvm.contains("DILocalVariable(name: \"character\""));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_distinct_custom_generator_final_character_after_action() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FINAL-RETURN-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-final-character.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-final-character.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        3
    );
    let action_store = main.find("store ptr").expect("yield action is invoked");
    let final_value = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("final Character is materialized");
    assert!(action_store < final_value);
    assert!(main.contains("#dbg_value(i32 0"));
    assert_eq!(main.matches("#dbg_declare(ptr").count(), 1);
    assert_eq!(main.matches("store ptr").count(), 1);
    assert!(llvm.contains("name: \"Generator Character Unit Character\""));
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_custom_generator_local_binding_in_lexical_debug_scope() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-LOCAL-BINDING-001,
    // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-LOCAL-BINDING-001
    let source = include_str!("../../../../../examples/language/custom-generator-local-binding.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-local-binding.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert_eq!(main.matches("store ptr").count(), 2);
    assert!(main.contains("#dbg_value(i32 0"));
    assert_eq!(main.matches("#dbg_declare(ptr").count(), 2);
    assert!(llvm.contains("DILocalVariable(name: \"copy\""));
    assert!(llvm.contains("DILexicalBlock("));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_post_resume_local_before_the_next_custom_yield() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-BODY-STATEMENT-001,
    // TOPAL-GENERATOR-LOCAL-BINDING-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-SUSPENSION-001
    let source = include_str!("../../../../../examples/language/custom-generator-suspension.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-suspension.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let materializations = main
        .match_indices("call ptr @topal.runtime.string.make")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    let local_declaration = main
        .match_indices("#dbg_declare(ptr")
        .nth(1)
        .expect("post-resume local has a distinct debug declaration")
        .0;
    let first_action = main
        .find("store ptr")
        .expect("first yield action is invoked");

    assert_eq!(materializations.len(), 3);
    assert!(materializations[1] < first_action);
    assert!(first_action < local_declaration);
    assert!(local_declaration < materializations[2]);
    assert_eq!(main.matches("#dbg_declare(ptr").count(), 2);
    assert_eq!(main.matches("store ptr").count(), 3);
    assert!(llvm.contains("DILocalVariable(name: \"copy\""));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(llvm.contains("name: \"Character\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_unit_resume_binding_after_the_custom_action() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-GENERATOR-RESUME-BINDING-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001
    let source = include_str!("../../../../../examples/language/custom-generator-resume-binding.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-resume-binding.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let action_store = main.find("store ptr").expect("yield action is invoked");
    let resumed_store = main
        .find("store i8 0")
        .expect("successful Unit resume has debug storage");

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        2
    );
    assert!(action_store < resumed_store);
    assert_eq!(main.matches("#dbg_declare(ptr").count(), 2);
    assert_eq!(main.matches("store ptr").count(), 1);
    assert_eq!(main.matches("store i8 0").count(), 1);
    assert!(llvm.contains("DILocalVariable(name: \"resumed\""));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(llvm.contains("name: \"Unit\""));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_function_local_custom_generator_close_without_runtime_state() {
    // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-CLOSE-001,
    // TOPAL-GENERATOR-ERROR-CODE-001, TOPAL-COMPILER-GENERATOR-CLOSE-001
    let source = include_str!("../../../../../examples/language/custom-generator-close.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "abandon")
        .expect("called abandon function is instantiated");
    let llvm = Generator::new(&program, "custom-generator-close.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let close = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(ptr %arg0)",
            function.symbol
        ))
        .expect("close function has one private Character parameter")
        .1
        .split_once("}\n")
        .expect("close function definition terminates")
        .0;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(main.contains(&format!("call fastcc void @{}(ptr", function.symbol)));
    assert!(close.contains("#dbg_value(i32 0"));
    assert!(close.contains("ret void"));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(!llvm.contains("TopalGeneratorErrorHeader"));
    assert!(!llvm.contains("Result (Unit, lang generator GeneratorErrorCode)"));
    assert!(!close.contains("topal.runtime.string.make"));
    assert!(!close.contains("topal.runtime.generator"));
    assert!(!close.contains("call ptr %"));
}

#[test]
fn emits_function_local_custom_generator_close_handler() {
    // TOPAL-GENERATOR-CLOSE-001, TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-GENERATOR-ERROR-CODE-001,
    // TOPAL-COMPILER-GENERATOR-CLOSE-HANDLER-001
    let source = include_str!("../../../../../examples/language/custom-generator-close-handler.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "abandon")
        .expect("called abandon function is instantiated");
    let llvm = Generator::new(&program, "custom-generator-close-handler.t").emit();
    let close = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(ptr %arg0)",
            function.symbol
        ))
        .expect("close function has one private Character parameter")
        .1
        .split_once("}\n")
        .expect("close function definition terminates")
        .0;

    let failure = close
        .find("call ptr @topal.runtime.result.failure(i32 0")
        .expect("close materializes the intrinsic failure Result");
    let payload = close
        .find("call ptr @topal.runtime.result.payload")
        .expect("Error handler selects the failure payload");
    let code = close
        .find("call i32 @topal.runtime.error.code")
        .expect("the selected Error remains observable to GDB");
    let returned = close.find("ret void").expect("handler completes with Unit");
    assert!(failure < payload && payload < code && code < returned);
    assert_eq!(
        close
            .matches("call ptr @topal.runtime.result.failure(i32 0")
            .count(),
        1
    );
    assert!(close.contains("#dbg_declare(ptr"));
    assert!(llvm.contains("DILocalVariable(name: \"resume-result\""));
    assert!(llvm.contains("DILocalVariable(name: \"problem\""));
    assert!(llvm.contains("name: \"lang generator GeneratorErrorCode\""));
    assert!(llvm.contains("DIEnumerator(name: \"generator-closed\", value: 0)"));
    assert!(llvm.contains("name: \"Result (Unit, lang generator GeneratorErrorCode)\""));
    assert!(llvm.contains("name: \"Error (lang generator GeneratorErrorCode)\""));
    assert!(!close.contains("topal.runtime.result.is.error"));
    assert!(!close.contains("topal.runtime.generator"));
    assert!(!close.contains("call ptr %"));
}

#[test]
fn emits_generator_local_function_during_custom_close() {
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
    let llvm = Generator::new(&program, "custom-generator-local-close-handler.t").emit();
    let local = llvm
        .split_once(&format!("define internal fastcc void @{}", cleanup.symbol))
        .expect("module contains the private generator-local cleanup function")
        .1
        .split_once("\n}\n")
        .expect("local cleanup function has a complete definition")
        .0;
    let close = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(ptr %arg0)",
            abandon.symbol
        ))
        .expect("abandon function has one private Character parameter")
        .1
        .split_once("}\n")
        .expect("abandon function definition terminates")
        .0;

    assert!(local.contains("(i32 %arg0) nounwind noinline"));
    assert!(local.contains("store i32 %arg0"));
    assert!(local.contains("#dbg_declare(ptr"));
    assert!(local.contains("ret void"));
    let failure = close
        .find("call ptr @topal.runtime.result.failure(i32 0")
        .expect("close materializes the intrinsic failure Result");
    let payload = close
        .find("call ptr @topal.runtime.result.payload")
        .expect("qualified handler observes the failure payload");
    let code = close
        .find("call i32 @topal.runtime.error.code")
        .expect("qualified handler observes the nominal code");
    let call_text = format!("call fastcc void @{}(i32 0)", cleanup.symbol);
    let call = close
        .find(&call_text)
        .expect("close branch directly invokes cleanup Closed");
    let returned = close
        .find("ret void")
        .expect("abandon completes after local cleanup");
    assert!(failure < payload && payload < code && code < call && call < returned);
    assert!(!close.contains(&format!("call fastcc void @{}(i32 1)", cleanup.symbol)));
    assert!(llvm.contains("DISubprogram(name: \"cleanup\""));
    assert!(llvm.contains(&format!("linkageName: \"{}\"", cleanup.symbol)));
    assert!(llvm.contains("DILocalVariable(name: \"choice\", arg: 1"));
    assert!(llvm.contains("name: \"CloseChoice\""));
    assert!(llvm.contains("DIEnumerator(name: \"Closed\", value: 0)"));
    assert!(llvm.contains("DIEnumerator(name: \"Continued\", value: 1)"));
    assert!(!close.contains("topal.runtime.generator"));
    assert!(!close.contains("call ptr %"));
}

#[test]
fn emits_qualified_custom_generator_close_code_pattern() {
    // TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001,
    // TOPAL-GENERATOR-CLOSE-HANDLER-001,
    // TOPAL-COMPILER-GENERATOR-CLOSE-CODE-PATTERN-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-close-code-pattern.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "abandon")
        .expect("called abandon function is instantiated");
    let llvm = Generator::new(&program, "custom-generator-close-code-pattern.t").emit();
    let close = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(ptr %arg0)",
            function.symbol
        ))
        .expect("close function has one private Character parameter")
        .1
        .split_once("}\n")
        .expect("close function definition terminates")
        .0;

    let failure = close
        .find("call ptr @topal.runtime.result.failure(i32 0")
        .expect("close materializes the intrinsic failure Result");
    let payload = close
        .find("call ptr @topal.runtime.result.payload")
        .expect("qualified handler observes the failure payload");
    let code = close
        .find("call i32 @topal.runtime.error.code")
        .expect("qualified handler observes the nominal code");
    let returned = close.find("ret void").expect("handler completes with Unit");
    assert!(failure < payload && payload < code && code < returned);
    assert_eq!(close.matches("topal.runtime.error.code").count(), 1);
    assert!(llvm.contains("DILocalVariable(name: \"resume-result\""));
    assert!(!llvm.contains("DILocalVariable(name: \"problem\""));
    assert!(llvm.contains("name: \"lang generator GeneratorErrorCode\""));
    assert!(llvm.contains("DIEnumerator(name: \"generator-closed\", value: 0)"));
    assert!(!close.contains("switch i32"));
    assert!(!close.contains("topal.runtime.generator"));
    assert!(!close.contains("call ptr %"));
}

#[test]
fn emits_custom_generator_function_parameter_transfer() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-function-parameter.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("called custom Generator consumer is instantiated");
    let llvm = Generator::new(&program, "custom-generator-function-parameter.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let traversal = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(i32 %arg0)",
            function.symbol
        ))
        .expect("traversal function has one private ownership token")
        .1
        .split_once("}\n")
        .expect("traversal function definition terminates")
        .0;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
    assert_eq!(
        traversal
            .matches("call ptr @topal.runtime.string.make")
            .count(),
        1
    );
    assert!(traversal.contains("alloca i32, align 4"));
    assert!(traversal.contains("store i32 %arg0"));
    assert!(traversal.contains("alloca ptr, align 8"));
    assert!(traversal.contains("#dbg_declare(ptr"));
    assert!(traversal.contains("ret void"));
    assert!(!traversal.contains("generator.foreach.loop"));
    assert!(!traversal.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_custom_generator_character_result_parameter_transfer() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001
    let source = include_str!(
        "../../../../../examples/language/custom-generator-character-return-parameter.t"
    );
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("called result-valued custom Generator consumer is instantiated");
    let llvm = Generator::new(&program, "custom-generator-character-return-parameter.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let traversal = llvm
        .split_once(&format!(
            "define internal fastcc ptr @{}(i32 %arg0)",
            function.symbol
        ))
        .expect("result-valued traversal has one private ownership token")
        .1
        .split_once("}\n")
        .expect("result-valued traversal definition terminates")
        .0;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(main.contains(&format!("call fastcc ptr @{}(i32 0)", function.symbol)));
    assert_eq!(
        traversal
            .matches("call ptr @topal.runtime.string.make")
            .count(),
        2
    );
    let yielded = traversal
        .find("call ptr @topal.runtime.string.make")
        .expect("callee materializes the yielded Character");
    let returned = traversal
        .rfind("call ptr @topal.runtime.string.make")
        .expect("callee materializes the final Character");
    let return_instruction = traversal
        .find("ret ptr")
        .expect("callee returns the final Character descriptor");
    assert!(yielded < returned && returned < return_instruction);
    assert!(traversal.contains("alloca i32, align 4"));
    assert!(traversal.contains("store i32 %arg0"));
    assert!(traversal.contains("alloca ptr, align 8"));
    assert!(traversal.contains("#dbg_declare(ptr"));
    assert!(!traversal.contains("generator.foreach.loop"));
    assert!(!traversal.contains("call ptr %"));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("name: \"Generator Character Unit Character\""));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_custom_generator_parameter_close_without_runtime_state() {
    // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-CLOSE-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-CLOSE-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-parameter-close.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "ignore")
        .expect("called custom Generator closer is instantiated");
    let llvm = Generator::new(&program, "custom-generator-parameter-close.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let close = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(i32 %arg0)",
            function.symbol
        ))
        .expect("close function has one private ownership token")
        .1
        .split_once("}\n")
        .expect("close function definition terminates")
        .0;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
    assert!(close.contains("alloca i32, align 4"));
    assert!(close.contains("store i32 %arg0"));
    assert!(close.contains("#dbg_declare(ptr"));
    assert!(close.contains("ret void"));
    assert!(!close.contains("topal.runtime.string.make"));
    assert!(!close.contains("call "));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(!llvm.contains("TopalGeneratorErrorHeader"));
    assert!(!llvm.contains("Result (Unit, lang generator GeneratorErrorCode)"));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_custom_generator_function_result_transfer() {
    // TOPAL-GENERATOR-FUNCTION-RESULT-001, TOPAL-GENERATOR-SUSPEND-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-function-result.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("called custom Generator factory is instantiated");
    let llvm = Generator::new(&program, "custom-generator-function-result.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let factory = llvm
        .split_once(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            function.symbol
        ))
        .expect("custom Generator factory has one private Character argument")
        .1
        .split_once("}\n")
        .expect("custom Generator factory definition terminates")
        .0;

    assert!(main.contains(&format!("call fastcc i32 @{}(ptr ", function.symbol)));
    assert!(main.contains("#dbg_value(i32"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(factory.contains("alloca ptr, align 8"));
    assert!(factory.contains("store ptr %arg0"));
    assert!(factory.contains("#dbg_declare(ptr"));
    assert!(factory.contains("ret i32 0"));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_custom_generator_character_result_function_result_transfer() {
    // TOPAL-GENERATOR-FUNCTION-RESULT-001,
    // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
    // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-001
    let source =
        include_str!("../../../../../examples/language/custom-generator-character-return-result.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "make")
        .expect("called result-valued custom Generator factory is instantiated");
    let llvm = Generator::new(&program, "custom-generator-character-return-result.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let factory = llvm
        .split_once(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            function.symbol
        ))
        .expect("custom Generator factory has one private Character argument")
        .1
        .split_once("}\n")
        .expect("custom Generator factory definition terminates")
        .0;

    assert_eq!(main.matches("call fastcc i32").count(), 1);
    assert!(main.contains(&format!("call fastcc i32 @{}(ptr ", function.symbol)));
    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        3
    );
    let transfer = main
        .find("call fastcc i32")
        .expect("caller receives the private ownership token");
    let yielded = main[transfer..]
        .find("call ptr @topal.runtime.string.make")
        .map(|offset| transfer + offset)
        .expect("caller materializes the yielded Character after transfer");
    let returned = main
        .rfind("call ptr @topal.runtime.string.make")
        .expect("caller materializes the final Character");
    let output = main
        .find("call void @topal.runtime.string.print")
        .expect("caller prints the final Character");
    assert!(transfer < yielded && yielded < returned && returned < output);
    assert!(main.contains("#dbg_value(i32"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(factory.contains("alloca ptr, align 8"));
    assert!(factory.contains("store ptr %arg0"));
    assert!(factory.contains("#dbg_declare(ptr"));
    assert!(factory.contains("ret i32 0"));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!llvm.contains("call ptr %"));
    assert!(llvm.contains("DILocalVariable(name: \"initial\""));
    assert!(llvm.contains("DILocalVariable(name: \"generated\""));
    assert!(llvm.contains("name: \"Generator Character Unit Character\""));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_transferred_string_character_generator_close_without_runtime_state() {
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-STRING-CHARACTERS-CLOSE-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001
    let source =
        include_str!("../../../../../examples/language/string-character-generator-close.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "ignore")
        .expect("called close function is instantiated");
    let llvm = Generator::new(&program, "string-character-generator-close.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let close = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(i32 %arg0)",
            function.symbol
        ))
        .expect("close function has one private ownership token")
        .1
        .split_once("}\n")
        .expect("close function definition terminates")
        .0;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
    assert!(close.contains("alloca i32, align 4"));
    assert!(close.contains("store i32 %arg0"));
    assert!(close.contains("#dbg_declare(ptr"));
    assert!(close.contains("ret void"));
    assert!(!close.contains("call "));
    assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_specialized_string_character_generator_parameter_traversal() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-PARAMETER-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001
    let source =
        include_str!("../../../../../examples/language/string-character-generator-parameter.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("called traversal function is instantiated");
    let llvm = Generator::new(&program, "string-character-generator-parameter.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let traversal = llvm
        .split_once(&format!(
            "define internal fastcc void @{}(i32 %arg0)",
            function.symbol
        ))
        .expect("traversal function has one private ownership token")
        .1
        .split_once("}\n")
        .expect("traversal function definition terminates")
        .0;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        1
    );
    assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
    assert_eq!(
        traversal
            .matches("call ptr @topal.runtime.string.make")
            .count(),
        3
    );
    assert!(traversal.contains("alloca i32, align 4"));
    assert!(traversal.contains("alloca ptr, align 8"));
    assert!(traversal.contains("#dbg_declare(ptr"));
    assert!(traversal.contains("ret void"));
    assert!(!traversal.contains("generator.foreach.loop"));
    assert!(!traversal.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_specialized_string_character_generator_result_transfer() {
    // TOPAL-STRING-CHARACTERS-FOREACH-001,
    // TOPAL-STRING-CHARACTERS-GENERATOR-001,
    // TOPAL-STRING-CHARACTERS-RESULT-001,
    // TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001
    let source =
        include_str!("../../../../../examples/language/string-character-generator-result.t");
    let program = analyze_for_compiler(source).unwrap();
    let function = program
        .functions
        .iter()
        .find(|function| function.source_name == "generate")
        .expect("called generator factory is instantiated");
    let llvm = Generator::new(&program, "string-character-generator-result.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    let factory = llvm
        .split_once(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            function.symbol
        ))
        .expect("generator factory has one private String argument")
        .1
        .split_once("}\n")
        .expect("generator factory definition terminates")
        .0;

    assert_eq!(
        main.matches("call ptr @topal.runtime.string.make").count(),
        4
    );
    assert!(main.contains(&format!("call fastcc i32 @{}(ptr ", function.symbol)));
    assert!(main.contains("#dbg_value(i32"));
    assert!(main.contains("#dbg_declare(ptr"));
    assert!(factory.contains("alloca ptr, align 8"));
    assert!(factory.contains("store ptr %arg0"));
    assert!(factory.contains("#dbg_declare(ptr"));
    assert!(factory.contains("ret i32 0"));
    assert!(!factory.contains("topal.runtime.string.make"));
    assert!(!main.contains("generator.foreach.loop"));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_lazy_unfold_construction_without_invoking_its_step() {
    // TOPAL-GENERATOR-UNFOLD-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001
    let source = include_str!("../../../../../examples/language/unfold-generator.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "unfold-generator.t").emit();

    assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Generator Int Unit Unit\""));
    assert!(llvm.contains("DIEnumerator(name: \"<Generator Int Unit Unit>\", value: 0)"));
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;
    assert!(!main.contains("topal.runtime.list.int.uncons"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_finite_unfold_collection_as_a_seed_and_list_loop() {
    // TOPAL-GENERATOR-UNFOLD-001, TOPAL-GENERATOR-UNFOLD-COLLECT-001,
    // TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001
    let source = include_str!("../../../../../examples/language/unfold-collect.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "unfold-collect.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    for expected in [
        "generator.unfold.collect.loop",
        "phi ptr",
        "icmp ne ptr",
        "load ptr, ptr",
        "call ptr @topal.platform.allocate(i64 16)",
        "generator.unfold.collect.done",
    ] {
        assert!(main.contains(expected), "missing {expected:?}: {main}");
    }
    assert!(!main.contains("topal.runtime.list.int.uncons"));
    assert!(!main.contains("topal.runtime.optional"));
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_nominal_enums_as_checked_i32_tags_with_dwarf_enumerators() {
    // TOPAL-COMPILER-ENUM-001
    let source = "use language (version is v0.1)\nColor is Enum (Red, Green, Blue)\nnext is fn (value : Color) -> Color\n  value\n    Red then Green\n    Green then Blue\n    Blue then Red\n(next Red, next Green)\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/enums.t").emit();
    assert!(llvm.contains("define internal fastcc i32 @topal.fn.next"));
    assert!(llvm.contains("switch i32 %arg0"));
    assert!(llvm.contains("phi i32"));
    assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Color\""));
    assert!(llvm.contains("DIEnumerator(name: \"Red\", value: 0)"));
    assert!(llvm.contains("DIEnumerator(name: \"Green\", value: 1)"));
    assert!(llvm.contains("DIEnumerator(name: \"Blue\", value: 2)"));
}

#[test]
fn emits_nominal_sums_as_private_tagged_aggregates_with_dwarf() {
    // TOPAL-COMPILER-SUM-001, TOPAL-TYPE-UNION-001,
    // TOPAL-TYPE-VARIANT-001, TOPAL-DECISION-UNION-001
    let source = include_str!("../../../../../examples/language/unions-and-recursive-products.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/unions-and-recursive-products.t").emit();
    assert!(llvm.contains("define internal fastcc { ptr, { ptr, ptr } } @topal.fn.describe"));
    assert!(llvm.contains("({ i32, { ptr, { ptr, ptr } } } %arg0)"));
    assert!(llvm.contains("({ i32, ptr, ptr } %arg0)"));
    assert!(llvm.contains("insertvalue { i32, { ptr, { ptr, ptr } } }"));
    assert!(llvm.contains("extractvalue { i32, { ptr, { ptr, ptr } } } %arg0, 0"));
    assert!(llvm.contains("sum.decision.alternative"));
    assert!(llvm.contains("switch i32"));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalUnion.Message\""));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalVariant.Scalar\""));
    assert!(llvm.contains("DIEnumerator(name: \"Move\", value: 1)"));
    assert!(llvm.contains("DIEnumerator(name: \"at 0\", value: 0)"));
}

#[test]
fn emits_modular_values_as_nominal_int_backed_private_values() {
    // TOPAL-COMPILER-MODULAR-001, TOPAL-NUM-MODULAR-ARITHMETIC-001
    let source = "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nretain is fn (value : ByteCounter) -> ByteCounter\n  value\nstart is ByteCounter 255\nresult is (retain start) + (ByteCounter 1)\nresult\n";
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/modular-values.t").emit();
    assert!(llvm.contains("define internal fastcc ptr @topal.fn.retain"));
    assert!(llvm.contains("call fastcc ptr @topal.fn.retain"));
    assert!(llvm.contains("call ptr @topal.runtime.int.add("));
    assert!(llvm.contains("call ptr @topal.runtime.int.subtract("));
    assert!(llvm.contains("call ptr @topal.runtime.int.modulo("));
    assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalModular.ByteCounter\""));
    assert!(llvm.contains("DW_TAG_typedef, name: \"ByteCounter\""));
}

#[test]
fn emits_dynamic_modular_validation_as_a_native_result() {
    // TOPAL-COMPILER-MODULAR-CONSTRUCTION-001,
    // TOPAL-NUM-MODULAR-CONSTRUCT-001
    let source = include_str!("../../../../../examples/language/modular-checked-construction.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "/source/modular-checked-construction.t").emit();
    assert!(llvm.contains("modular.accepted"));
    assert!(llvm.contains("modular.rejected"));
    assert!(llvm.contains("modular.merge"));
    assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 2);
    assert!(llvm.contains("call ptr @topal.runtime.result.success"));
    assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 0"));
    assert!(llvm.contains(&llvm_bytes(b"root.ByteCounter(Int)")));
    assert!(llvm.contains(
        "DW_TAG_typedef, name: \"Result (ByteCounter, lang arithmetic ArithmeticErrorCode)\""
    ));
}

#[test]
fn emits_ordered_custom_generator_overloads_and_typed_results() {
    // TOPAL-GENERATOR-OVERLOAD-001, TOPAL-GENERATOR-FOREACH-RESULT-001,
    // TOPAL-COMPILER-GENERATOR-OVERLOAD-001
    let source = include_str!("../../../../../examples/language/custom-generator-overloads.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "custom-generator-overloads.t").emit();
    let main = llvm
        .split_once("define internal void @topal.main")
        .expect("module contains generated source entry")
        .1;

    let unary_action = main
        .find("call ptr @topal.runtime.int.add(ptr @.topal.int.0")
        .expect("unary selection executes its Int action");
    let unary_final = main[unary_action..]
        .find("call ptr @topal.runtime.string.make")
        .map(|offset| unary_action + offset)
        .expect("unary traversal constructs its final String after the action");
    let binary_input = main[unary_final + 1..]
        .find("call ptr @topal.runtime.string.make")
        .map(|offset| unary_final + 1 + offset)
        .expect("binary application constructs suffix once");
    let binary_prefix = main[binary_input..]
        .find("call ptr @topal.runtime.int.add")
        .map(|offset| binary_input + offset)
        .expect("binary declaration prefix executes before its yield");
    let binary_action = main[binary_prefix..]
        .find("call i1 @topal.runtime.string.is.empty")
        .map(|offset| binary_prefix + offset)
        .expect("binary String action executes after its prefix and yield");
    let binary_final = main[binary_action..]
        .find("call ptr @topal.runtime.string.make")
        .map(|offset| binary_action + offset)
        .expect("binary traversal constructs its final String last");
    let output = main
        .find("call void @topal.runtime.string.print")
        .expect("both bound results reach Topal-owned output");
    assert!(
        unary_action < unary_final
            && unary_final < binary_input
            && binary_input < binary_prefix
            && binary_prefix < binary_action
            && binary_action < binary_final
            && binary_final < output
    );
    assert_eq!(main.matches("#dbg_value(i32 0").count(), 2);
    for expected in [
        "DILocalVariable(name: \"unary-generated\"",
        "DILocalVariable(name: \"unary-result\"",
        "DILocalVariable(name: \"binary-generated\"",
        "DILocalVariable(name: \"binary-result\"",
        "DILocalVariable(name: \"suffix\"",
        "name: \"Generator Int Unit String\"",
        "name: \"Generator String Unit String\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!main.contains("topal.runtime.generator"));
    assert!(!main.contains("call ptr %"));
}

#[test]
fn emits_generic_custom_generator_function_boundaries() {
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
        .expect("factory specialization exists");
    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("consumer specialization exists");
    let llvm = Generator::new(&program, "custom-generator-generic-function-boundaries.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc i32 @{}(ptr %arg0)",
        make.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(i32 %arg0)",
        consume.symbol
    )));
    assert!(llvm.contains(&format!("call fastcc i32 @{}(ptr", make.symbol)));
    assert!(llvm.contains(&format!("call fastcc ptr @{}(i32", consume.symbol)));

    let consume_body = llvm
        .split_once(&format!(
            "define internal fastcc ptr @{}(i32 %arg0)",
            consume.symbol
        ))
        .expect("module contains consumer")
        .1
        .split_once("\n}\n")
        .expect("consumer has one body")
        .0;
    let action = consume_body
        .find("call ptr @topal.runtime.int.add")
        .expect("consumer invokes the checked Int action");
    let final_string = consume_body[action..]
        .find("call ptr @topal.runtime.string.make")
        .map(|offset| action + offset)
        .expect("consumer constructs the final String after resumption");
    let returned = consume_body[final_string..]
        .find("ret ptr")
        .map(|offset| final_string + offset)
        .expect("consumer returns the final String");
    assert!(action < final_string && final_string < returned);
    for expected in [
        "name: \"Generator Int Unit String\"",
        "DILocalVariable(name: \"initial\", arg: 1",
        "DILocalVariable(name: \"generated\", arg: 1",
        "DILocalVariable(name: \"value\"",
        "DILocalVariable(name: \"result\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!llvm.contains("topal.runtime.generator"));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("call i32 %"));
}

#[test]
fn emits_compound_custom_generator_function_boundaries() {
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
        .expect("compound factory specialization exists");
    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("compound consumer specialization exists");
    let llvm = Generator::new(&program, "custom-generator-compound-function-boundaries.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc i32 @{}({{ ptr, ptr }} %arg0)",
        make.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc {{ ptr, ptr }} @{}(i32 %arg0)",
        consume.symbol
    )));
    assert!(llvm.contains(&format!("call fastcc i32 @{}({{ ptr, ptr }}", make.symbol)));
    assert!(llvm.contains(&format!(
        "call fastcc {{ ptr, ptr }} @{}(i32",
        consume.symbol
    )));

    let consume_body = llvm
        .split_once(&format!(
            "define internal fastcc {{ ptr, ptr }} @{}(i32 %arg0)",
            consume.symbol
        ))
        .expect("module contains compound consumer")
        .1
        .split_once("\n}\n")
        .expect("compound consumer has one body")
        .0;
    let int_action = consume_body
        .find("call i32 @topal.runtime.int.compare")
        .expect("consumer compares the yielded Int field");
    let string_action = consume_body[int_action..]
        .find("call i1 @topal.runtime.string.equal")
        .map(|offset| int_action + offset)
        .expect("consumer compares the yielded String field after the Int field");
    let final_string = consume_body[string_action..]
        .find("call ptr @topal.runtime.string.make")
        .map(|offset| string_action + offset)
        .expect("consumer constructs the final product after resumption");
    let returned = consume_body[final_string..]
        .find("ret { ptr, ptr }")
        .map(|offset| final_string + offset)
        .expect("consumer returns the final product");
    assert!(int_action < string_action && string_action < final_string && final_string < returned);
    for expected in [
        "name: \"Generator (Int, String) Unit (Int, String)\"",
        "name: \"(Int, String)\"",
        "name: \"_0\"",
        "name: \"_1\"",
        "DILocalVariable(name: \"initial\", arg: 1",
        "DILocalVariable(name: \"generated\", arg: 1",
        "DILocalVariable(name: \"value\"",
        "DILocalVariable(name: \"result\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!llvm.contains("topal.runtime.generator"));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("call i32 %"));
    assert!(!llvm.contains("call { ptr, ptr } %"));
}

#[test]
fn emits_nested_custom_generator_function_boundaries() {
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
        .expect("nested factory specialization exists");
    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("nested consumer specialization exists");
    let llvm = Generator::new(&program, "custom-generator-nested-function-boundaries.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc i32 @{}(ptr %arg0)",
        make.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(i32 %arg0)",
        consume.symbol
    )));
    assert!(llvm.contains(&format!("call fastcc i32 @{}(ptr", make.symbol)));
    assert!(llvm.contains(&format!("call fastcc ptr @{}(i32", consume.symbol)));

    let make_body = llvm
        .split_once(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            make.symbol
        ))
        .expect("module contains nested factory")
        .1
        .split_once("\n}\n")
        .expect("nested factory has one body")
        .0;
    assert!(make_body.contains("alloca ptr, align 8"));
    assert!(make_body.contains("#dbg_declare"));

    let consume_body = llvm
        .split_once(&format!(
            "define internal fastcc ptr @{}(i32 %arg0)",
            consume.symbol
        ))
        .expect("module contains nested consumer")
        .1
        .split_once("\n}\n")
        .expect("nested consumer has one body")
        .0;
    let optional_tags = consume_body
        .find("call i1 @topal.runtime.optional.is.some")
        .expect("consumer checks Optional tags before payload access");
    let int_action = consume_body[optional_tags..]
        .find("call i32 @topal.runtime.int.compare")
        .map(|offset| optional_tags + offset)
        .expect("consumer compares the yielded product Int field");
    let string_action = consume_body[int_action..]
        .find("call i1 @topal.runtime.string.equal")
        .map(|offset| int_action + offset)
        .expect("consumer compares the yielded product String field after the Int field");
    let final_result = consume_body[string_action..]
        .find("call ptr @topal.runtime.result.success")
        .map(|offset| string_action + offset)
        .expect("consumer constructs the successful final Result after resumption");
    let returned = consume_body[final_result..]
        .find("ret ptr")
        .map(|offset| final_result + offset)
        .expect("consumer returns the final Result");
    assert!(
        optional_tags < int_action
            && int_action < string_action
            && string_action < final_result
            && final_result < returned
    );
    for expected in [
        "name: \"Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)\"",
        "name: \"Optional (Int, String)\"",
        "name: \"Result ((Int, String), lang arithmetic ArithmeticErrorCode)\"",
        "DILocalVariable(name: \"initial\", arg: 1",
        "DILocalVariable(name: \"generated\", arg: 1",
        "DILocalVariable(name: \"value\"",
        "DILocalVariable(name: \"result\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!llvm.contains("topal.runtime.generator"));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("call i32 %"));
}

#[test]
fn emits_list_custom_generator_function_boundaries() {
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
        .expect("List factory specialization exists");
    let consume = program
        .functions
        .iter()
        .find(|function| function.source_name == "consume")
        .expect("List consumer specialization exists");
    let llvm = Generator::new(&program, "custom-generator-list-values.t").emit();
    assert!(llvm.contains(&format!(
        "define internal fastcc i32 @{}(ptr %arg0)",
        make.symbol
    )));
    assert!(llvm.contains(&format!(
        "define internal fastcc ptr @{}(i32 %arg0)",
        consume.symbol
    )));
    assert!(llvm.contains(&format!("call fastcc i32 @{}(ptr", make.symbol)));
    assert!(llvm.contains(&format!("call fastcc ptr @{}(i32", consume.symbol)));

    let make_body = llvm
        .split_once(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            make.symbol
        ))
        .expect("module contains List factory")
        .1
        .split_once("\n}\n")
        .expect("List factory has one body")
        .0;
    assert!(make_body.contains("alloca ptr, align 8"));
    assert!(make_body.contains("#dbg_declare"));

    let consume_body = llvm
        .split_once(&format!(
            "define internal fastcc ptr @{}(i32 %arg0)",
            consume.symbol
        ))
        .expect("module contains List consumer")
        .1
        .split_once("\n}\n")
        .expect("List consumer has one body")
        .0;
    let action = consume_body
        .find("call ptr @topal.runtime.list.int.entry.count")
        .expect("consumer observes the yielded List before resumption");
    let appended_node = consume_body[action..]
        .find("call ptr @topal.platform.allocate(i64 16)")
        .map(|offset| action + offset)
        .expect("consumer constructs the appended singleton after resumption");
    let final_list = consume_body[appended_node..]
        .find("call ptr @topal.runtime.list.int.concat")
        .map(|offset| appended_node + offset)
        .expect("consumer appends after constructing the singleton");
    let returned = consume_body[final_list..]
        .find("ret ptr")
        .map(|offset| final_list + offset)
        .expect("consumer returns the final List");
    assert!(action < appended_node && appended_node < final_list && final_list < returned);
    for expected in [
        "name: \"Generator List Int Unit List Int\"",
        "name: \"List Int\"",
        "DILocalVariable(name: \"initial\", arg: 1",
        "DILocalVariable(name: \"generated\", arg: 1",
        "DILocalVariable(name: \"values\"",
        "DILocalVariable(name: \"result\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!llvm.contains("topal.runtime.generator"));
    assert!(!llvm.contains("call ptr %"));
    assert!(!llvm.contains("call i32 %"));
}

#[test]
fn emits_private_direct_task_storage_and_transactions_in_source_order() {
    // TOPAL-TASK-LIFECYCLE-001, TOPAL-TASK-STATE-001,
    // TOPAL-TASK-MESSAGE-001, TOPAL-COMPILER-TASK-DIRECT-001
    let source = include_str!("../../../../../examples/language/task-declaration-order.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "task-declaration-order.t").emit();
    assert!(llvm.contains("%topal.TaskStorage = type { i64, i64, ptr }"));
    assert!(llvm.contains("@topal.runtime.task.next.identity = private global i64 1"));
    let constructed = llvm
        .find("call ptr @topal.runtime.task.make")
        .expect("start constructs a distinct Task instance");
    let loaded_for_event = llvm[constructed..]
        .find("call ptr @topal.runtime.task.state.load")
        .map(|offset| constructed + offset)
        .expect("event transaction reads private state");
    let added = llvm[loaded_for_event..]
        .find("call ptr @topal.runtime.int.add")
        .map(|offset| loaded_for_event + offset)
        .expect("event computes the replacement state");
    let replaced = llvm[added..]
        .find("call void @topal.runtime.task.state.replace")
        .map(|offset| added + offset)
        .expect("event commits state atomically");
    let loaded_for_request = llvm[replaced..]
        .find("call ptr @topal.runtime.task.state.load")
        .map(|offset| replaced + offset)
        .expect("request observes the committed state");
    assert!(
        constructed < loaded_for_event
            && loaded_for_event < added
            && added < replaced
            && replaced < loaded_for_request
    );
    for expected in [
        "DW_TAG_typedef, name: \"OrderedCounter\"",
        "DW_TAG_structure_type, name: \"TopalTask.OrderedCounter\"",
        "name: \"identity\"",
        "name: \"terminated\"",
        "name: \"count\"",
        "DILocalVariable(name: \"ordered-counter\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!llvm.contains("@malloc"));
}

#[test]
fn emits_one_yield_task_stream_as_an_ordered_affine_transaction() {
    // TOPAL-TASK-HANDLER-001, TOPAL-TASK-MESSAGE-001,
    // TOPAL-COMPILER-TASK-STREAM-001
    let source = include_str!("../../../../../examples/language/task-message-transactions.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "task-message-transactions.t").emit();
    let constructed = llvm
        .find("call ptr @topal.runtime.task.make")
        .expect("start constructs the stream-capable task");
    let event_load = llvm[constructed..]
        .find("call ptr @topal.runtime.task.state.load")
        .map(|offset| constructed + offset)
        .expect("event loads private state");
    let replaced = llvm[event_load..]
        .find("call void @topal.runtime.task.state.replace")
        .map(|offset| event_load + offset)
        .expect("event commits private state");
    let stream_load = llvm[replaced..]
        .find("call ptr @topal.runtime.task.state.load")
        .map(|offset| replaced + offset)
        .expect("stream yield loads the committed state");
    let stream_result = llvm[stream_load..]
        .find("call ptr @topal.runtime.result.success(ptr null)")
        .map(|offset| stream_load + offset)
        .expect("stream completion commits its Unit result");
    let request_load = llvm[stream_result..]
        .find("call ptr @topal.runtime.task.state.load")
        .map(|offset| stream_result + offset)
        .expect("following request executes after stream completion");
    assert!(
        constructed < event_load
            && event_load < replaced
            && replaced < stream_load
            && stream_load < stream_result
            && stream_result < request_load
    );
    assert_eq!(
        llvm.matches("call ptr @topal.runtime.task.state.load")
            .count(),
        3,
        "stream construction must capture the task without observing state"
    );
    for expected in [
        "DW_TAG_typedef, name: \"Counter\"",
        "DW_TAG_structure_type, name: \"TopalTask.Counter\"",
        "DW_TAG_enumeration_type, name: \"Generator Nat Unit Result (Unit, ())\"",
        "DILocalVariable(name: \"stream\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!llvm.contains("@malloc"));
    assert!(!llvm.contains("topal.runtime.generator"));
}

#[test]
fn emits_checked_external_location_as_ordered_topal_owned_storage() {
    // TOPAL-LAYOUT-CONSTRUCT-001, TOPAL-ADDRESS-RANGE-001,
    // TOPAL-LOCATION-CONSTRUCT-001, TOPAL-LOCATION-READ-001,
    // TOPAL-LOCATION-WRITE-001, TOPAL-COMPILER-EXTERNAL-LOCATION-001
    let source = include_str!("../../../../../examples/language/external-layout-location.t");
    let program = analyze_for_compiler(source).unwrap();
    let llvm = Generator::new(&program, "external-layout-location.t").emit();
    let constructed = llvm
        .find("call ptr @topal.runtime.location.make")
        .expect("checked location allocates Topal-owned storage");
    let written = llvm[constructed..]
        .find("call void @topal.runtime.location.write")
        .map(|offset| constructed + offset)
        .expect("source write remains an ordered runtime call");
    let read = llvm[written..]
        .find("call ptr @topal.runtime.location.read")
        .map(|offset| written + offset)
        .expect("source read follows the write");
    assert!(constructed < written && written < read);
    assert_eq!(
        llvm.matches("call void @topal.runtime.location.write")
            .count(),
        1
    );
    assert_eq!(
        llvm.matches("call ptr @topal.runtime.location.read")
            .count(),
        1
    );
    for expected in [
        "%topal.LocationStorage = type { ptr, ptr, i64, ptr }",
        "DW_TAG_typedef, name: \"UInt32LE\"",
        "DW_TAG_typedef, name: \"ControlLocation\"",
        "DW_TAG_structure_type, name: \"TopalLocation.ControlLocation\"",
        "DILocalVariable(name: \"control\"",
        "DILocalVariable(name: \"stored\"",
    ] {
        assert!(llvm.contains(expected), "missing {expected:?}");
    }
    assert!(!llvm.contains("inttoptr i64 1073741856"));
    assert!(!llvm.contains("@malloc"));
}
