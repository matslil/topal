#[test]
fn custom_generator_transfers_boolean_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "invert is generator ( initial : Boolean )\n  yields Boolean\n  resumes Unit\n  -> Boolean\n\n  _ is yield initial\n  not initial\ngenerated is invert true\ngenerated foreach { value }\n  _ is not value\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(false));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Boolean Unit Boolean"))
    );
}

#[test]
fn custom_generator_preserves_int_values() {
    let value = Session::new().evaluate("next is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> Int\n\n  _ is yield initial\n  initial + 1\ngenerated is next 999999999999999999999999999999\ngenerated foreach { value }\n  _ is value + 1\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "1000000000000000000000000000000");
}

#[test]
fn custom_generator_preserves_rational_values() {
    let value = Session::new().evaluate("next is generator ( initial : Rational )\n  yields Rational\n  resumes Unit\n  -> Rational\n\n  _ is yield initial\n  initial + (Rational (1, 3))\ngenerated is next (Rational (1, 3))\ngenerated foreach { value }\n  _ is value + (Rational (1, 3))\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "Rational ( 2, 3 )");
}

#[test]
fn custom_generator_transfers_unit_values() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("pulse is generator ( initial : Unit )\n  yields Unit\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()\ngenerated is pulse ()\ngenerated foreach { signal }\n  signal\n", &mut trace).unwrap();
    assert_eq!(value, Value::Unit);
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Unit Unit Unit"))
    );
}

#[test]
fn custom_generator_preserves_optional_values() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("optional is generator ( initial : Optional Int )\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n\n  _ is yield initial\n  None Int\ngenerated is optional (Some 7)\ngenerated foreach { candidate }\n  _ is candidate = (Some 7)\n", &mut trace).unwrap();
    assert_eq!(value.to_string(), "None");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Optional Int Unit Optional Int"))
    );
}

#[test]
fn custom_generator_preserves_range_values() {
    let value = Session::new().evaluate("narrow is generator ( initial : Range Int )\n  yields Range Int\n  resumes Unit\n  -> Range Int\n\n  _ is yield initial\n  initial and (5 ..= 15)\ngenerated is narrow (0 ..= 10)\ngenerated foreach { interval }\n  _ is 5 in interval\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "5 ..= 10");
}

#[test]
fn custom_generator_preserves_nat_constraint() {
    let value = Session::new().evaluate("next is generator ( initial : Nat )\n  yields Nat\n  resumes Unit\n  -> Nat\n\n  _ is yield initial\n  initial + 1\ngenerated is next (Nat 7)\ngenerated foreach { value }\n  _ is value + 1\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "8");
}

#[test]
fn custom_generator_preserves_enum_identity() {
    let mut trace = Vec::new();
    let value = Session::new().evaluate("Choice is Enum ( First, Second )\nchoose is generator ( initial : Choice )\n  yields Choice\n  resumes Unit\n  -> Choice\n\n  _ is yield initial\n  Second\ngenerated is choose First\ngenerated foreach { choice }\n  _ is choice = First\n", &mut trace).unwrap();
    assert_eq!(value.to_string(), "Second");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Generator Choice Unit Choice"))
    );
}

#[test]
fn custom_generator_preserves_product_values() {
    let value = Session::new().evaluate("pair is generator ( initial : (Int, String) )\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n\n  _ is yield initial\n  (8, \"done\")\ngenerated is pair (7, \"item\")\ngenerated foreach { value }\n  _ is value = (7, \"item\")\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn custom_generator_returns_structured_result_error() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-result-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert!(matches!(value, Value::Error { ref code, .. } if code == "division-by-zero"));
}

#[test]
fn custom_generator_preserves_comparison_identity() {
    let value = Session::new().evaluate("order is generator ( initial : Comparison )\n  yields Comparison\n  resumes Unit\n  -> Comparison\n\n  _ is yield initial\n  3 <=> 2\ngenerated is order (1 <=> 2)\ngenerated foreach { comparison }\n  _ is comparison = (1 <=> 2)\n", &mut Vec::new()).unwrap();
    assert_eq!(value.to_string(), "Greater");
}

#[test]
fn custom_generator_preserves_nested_optional_product() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-nested-optional-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "Some (8, \"done\")");
}

#[test]
fn custom_generator_preserves_nested_result_product() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-nested-result-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn custom_generator_preserves_nested_absent_optional() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-nested-none-values.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert!(
        matches!(value, Value::Optional { ref payload_classifier, payload: None } if payload_classifier == "(Int, String)")
    );
}

#[test]
fn custom_generators_preserve_recursive_nominal_classifiers() {
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/custom-generator-recursive-nominal-values.t"
            ),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Some Second, Second)");
}

#[test]
fn custom_generator_selects_final_decision_after_resuming() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-final-decision.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("accepted".into()));
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let selected = trace
        .iter()
        .position(|event| event.contains("decision.rule.selected"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("generator.returned"))
        .unwrap();
    assert!(resumed < selected && selected < returned);
}

#[test]
fn generator_return_mismatch_reports_expected_and_found_classifiers() {
    let error = Session::new()
        .evaluate(
            "invalid is generator ( initial : Boolean )\n  yields Boolean\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  42\ngenerated is invalid true\ngenerated foreach { value }\n  _ is not value\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-RETURN-TYPE");
    assert!(error.message.contains("returned `Int`"));
    assert!(error.message.contains("requires `String`"));
    assert!(error.help.as_deref().unwrap().contains("produce `String`"));
}

#[test]
fn custom_generator_retains_local_function_across_resumption() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-local-function.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("accepted".into()));
    let declared_enum = trace
        .iter()
        .position(|event| event.contains("enum.declared") && event.contains("Choice"))
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let called = trace
        .iter()
        .rposition(|event| event.contains("function.entry"))
        .unwrap();
    assert!(declared_enum < resumed && resumed < called);
}

#[test]
fn custom_generator_restores_local_declarations_during_close() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-local-close-handler.t"),
            &mut trace,
        )
        .unwrap();
    let close_bound = trace
        .iter()
        .position(|event| event.contains("generator.close.bound"))
        .unwrap();
    let entered = trace
        .iter()
        .rposition(|event| event.contains("function.entry"))
        .unwrap();
    let closed = trace
        .iter()
        .position(|event| event.contains("generator.closed"))
        .unwrap();
    assert!(close_bound < entered && entered < closed);
}

#[test]
fn custom_generator_selects_unary_and_binary_overloads() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-overloads.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"unary\", \"binary\")");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("generator.selected"))
            .count(),
        2
    );
    assert!(trace.iter().any(|event| event.contains("Int, String")));
}

#[test]
fn duplicate_generator_input_signature_is_rejected() {
    let error = Session::new()
        .evaluate(
            "same is generator ( value : Int )\n  yields Int\n  resumes Unit\n  -> Unit\n\n  _ is yield value\n  ()\nsame is generator ( other : Int )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield \"duplicate\"\n  \"duplicate\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DUPLICATE-GENERATOR-OVERLOAD");
}

#[test]
fn generator_overload_error_lists_available_inputs() {
    let error = Session::new()
        .evaluate(
            "select is generator ( value : Int )\n  yields Int\n  resumes Unit\n  -> Unit\n\n  _ is yield value\n  ()\ngenerated is select true\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-GENERATOR");
    assert!(error.message.contains("Boolean"));
    assert!(error.help.as_deref().unwrap().contains("Int"));
}

#[test]
fn foreach_result_binding_is_available_to_later_statements() {
    let value = Session::new()
        .evaluate(
            "once is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is once 7\nresult is generated foreach { value }\n  _ is value + 1\nempty? result\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(false));
}

#[test]
fn classified_foreach_result_reports_mismatch() {
    let error = Session::new()
        .evaluate(
            "once is generator ( initial : Int )\n  yields Int\n  resumes Unit\n  -> String\n\n  _ is yield initial\n  \"done\"\ngenerated is once 7\nresult : Int is generated foreach { value }\n  _ is value + 1\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-FOREACH-RESULT-CLASSIFIER");
    assert!(error.message.contains("returned `String`"));
    assert!(error.message.contains("requires `Int`"));
}

#[test]
fn custom_generator_crosses_generic_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/custom-generator-generic-function-boundaries.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::String("done".into()));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.result.transferred"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("generator.parameter.transferred"))
    );
}

#[test]
fn compound_generator_crosses_function_boundaries() {
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/custom-generator-compound-function-boundaries.t"
            ),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
}

#[test]
fn nested_generator_crosses_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/custom-generator-nested-function-boundaries.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(8, \"done\")");
    let classifier = "Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)";
    assert!(trace.iter().any(|event| {
        event.contains("generator.result.transferred") && event.contains(classifier)
    }));
    assert!(trace.iter().any(|event| {
        event.contains("generator.parameter.transferred") && event.contains(classifier)
    }));
}

#[test]
fn list_generator_crosses_function_boundaries() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/custom-generator-list-values.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "Entry ( 7, Entry ( 9, Empty ) )");
    assert!(trace.iter().any(|event| {
        event.contains("generator.result.transferred")
            && event.contains("Generator List Int Unit List Int")
    }));
    assert!(trace.iter().any(|event| {
        event.contains("generator.parameter.transferred")
            && event.contains("Generator List Int Unit List Int")
    }));
}

#[test]
fn custom_generator_executes_discard_after_resume() {
    let mut trace = Vec::new();
    Session::new()
        .evaluate(
            "inspect-between is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  _ is empty? initial\n  _ is yield \"\"\n  ()\ngenerated is inspect-between \"Topal\"\ngenerated foreach { text }\n  _ is empty? text\n",
            &mut trace,
        )
        .unwrap();
    let resumed = trace
        .iter()
        .position(|event| event.contains("generator.resumed"))
        .unwrap();
    let tested = trace
        .iter()
        .enumerate()
        .skip(resumed + 1)
        .find_map(|(index, event)| event.contains("string.empty.tested").then_some(index))
        .unwrap();
    let suspended = trace
        .iter()
        .rposition(|event| event.contains("generator.suspended"))
        .unwrap();
    assert!(resumed < tested && tested < suspended);
}

#[test]
fn custom_generator_cannot_yield_after_close_result() {
    let error = Session::new()
        .evaluate(
            "invalid-close is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  resume-result is yield initial\n  _ is yield initial\n  ()\nabandon is fn ( initial : Character ) -> Unit\n  generated is invalid-close initial\n  ()\nabandon \"T\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-GENERATOR-YIELD-AFTER-CLOSE");
    assert!(error.message.contains("cannot yield again"));
}

#[test]
fn rational_ranges_use_exact_canonical_conversion() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate("interval is 0 .. 2.5\n(interval, 1.5 in interval, interval contains 2, 3 in interval)\n", &mut trace)
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Rational ( 0, 1 ) .. Rational ( 5, 2 ), true, true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:left"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("Int->Rational:membership"))
    );
}

#[test]
fn lists_construct_compare_and_decompose() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/lists.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some 6, Some 6, Some Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) ), None, None, 5, false, true, true, Some (6, Entry ( 7, Entry ( 8, Entry ( 9, Entry ( 10, Empty ) ) ) )), Some 10, true)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("list.entry.constructed"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("list.entry.decomposed"))
    );
    assert!(trace.iter().any(|event| event.contains("equality.list")));
    for event in [
        "list.prepended",
        "list.appended",
        "list.concatenated",
        "list.entry-count",
        "list.empty.tested",
        "list.empty.constructed",
        "list.singleton.constructed",
        "list.uncons",
        "list.first",
        "list.rest",
        "list.reversed",
    ] {
        assert!(trace.iter().any(|record| record.contains(event)), "{event}");
    }
}

#[test]
fn lists_with_the_same_element_classifier_and_different_lengths_are_unequal() {
    let value = Session::new()
        .evaluate(
            "left : List Int is one 1\nright : List Int is Entry (1, Entry (2, Empty))\nleft = right\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(false));
}

#[test]
fn first_and_rest_reject_non_lists() {
    for operation in ["first", "rest"] {
        let error = Session::new()
            .evaluate(&format!("{operation} 7\n"), &mut Vec::new())
            .unwrap_err();
        assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
        assert!(error.message.contains("requires a List"));
    }
}

#[test]
fn recursive_list_classifiers_cross_function_boundaries() {
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/nested-lists.t"),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(
        value.to_string(),
        "(Some Entry ( (7, \"seven\"), Empty ), 1, true)"
    );
}

#[test]
fn list_containment_distinguishes_entry_sequence_and_subsequence() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/list-containment.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false, true, true, false, false)");
    for rule in [
        "TOPAL-LIST-CONTAINS-ENTRY-001",
        "TOPAL-LIST-CONTAINS-SEQUENCE-001",
        "TOPAL-LIST-CONTAINS-SUBSEQUENCE-001",
    ] {
        assert!(trace.iter().any(|event| event.contains(rule)), "{rule}");
    }
}

#[test]
fn list_containment_requires_compatible_classifiers() {
    let error = Session::new()
        .evaluate(
            "numbers : List Int is one 1\ntexts : List String is one \"one\"\nnumbers contains-sequence texts\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-CONTAINMENT-CLASSIFIER");
    assert!(error.message.contains("List String"));
}

#[test]
fn list_value_removal_preserves_retained_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/list-removal.t"),
            &mut trace,
        )
        .unwrap();
    assert!(
        value
            .to_string()
            .contains("Entry ( 1, Entry ( 3, Entry ( 2")
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-LIST-REMOVE-FIRST-001"))
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-LIST-REMOVE-ALL-001"))
    );
}

#[test]
fn list_value_removal_rejects_wrong_classifier() {
    let error = Session::new()
        .evaluate(
            "values : List Int is one 1\nvalues remove-first \"1\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-REMOVAL-CLASSIFIER");
}

#[test]
fn uncons_is_total_for_empty_lists_and_rejects_other_values() {
    let value = Session::new()
        .evaluate("uncons (empty List Int)\n", &mut Vec::new())
        .unwrap();
    assert_eq!(value.to_string(), "None");

    let error = Session::new()
        .evaluate("uncons 7\n", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-NO-APPLICABLE-OVERLOAD");
    assert!(error.message.contains("requires a List"));
}

#[test]
fn explicit_empty_and_singleton_lists_preserve_numeric_one() {
    let value = Session::new()
        .evaluate(
            "empty-values is empty List String\nsingleton is one \"Topal\"\n(empty-values, singleton, one Int)\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Empty, Entry ( \"Topal\", Empty ), 1)");
}

#[test]
fn list_operations_reject_incompatible_classifiers() {
    let entry = Session::new()
        .evaluate(
            "values : List Int is Empty\nvalues append \"bad\"\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(entry.code, "E-LIST-ENTRY-CLASSIFIER");
    assert!(entry.message.contains("requires `Int`"));

    let concat = Session::new()
        .evaluate(
            "numbers : List Int is Empty\ntexts : List String is Empty\nnumbers concat texts\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(concat.code, "E-LIST-CONCAT-CLASSIFIER");
    assert!(concat.message.contains("List String"));
}

#[test]
fn list_entry_classifier_mismatch_is_precise() {
    let error = Session::new()
        .evaluate(
            "values : List Int is Entry ( \"bad\", Empty )\n",
            &mut Vec::new(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-ENTRY-CLASSIFIER");
    assert!(error.message.contains("requires `Int`"));
    assert!(error.help.unwrap().contains("use a `Int` value"));
}

#[test]
fn list_remainder_must_be_a_list() {
    let error = Session::new()
        .evaluate("values : List Int is Entry ( 7, 8 )\n", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-LIST-REMAINDER");
    assert!(error.help.unwrap().contains("Empty"));
}

#[test]
fn loaded_modules_expose_only_published_members() {
    let mut session = Session::new();
    session
        .load_module(
            "math",
            "use language (\n  version is v0.1\n)\nprivate-value is 40\npub answer is private-value + 2\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(
        session.evaluate("math answer", &mut Vec::new()).unwrap(),
        Value::Int(BigInt::from(42))
    );
    let error = session
        .evaluate("math private-value", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-NAMESPACE-MEMBER-NOT-FOUND");
}

#[test]
fn published_functions_capture_private_named_functions() {
    let mut session = Session::new();
    session
        .load_module(
            "math",
            "use language (\n  version is v0.1\n)\nincrement is fn (value : Int) -> Int\n  value + 1\npub answer is fn (value : Int) -> Int\n  increment value\n",
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(
        session.evaluate("math answer 41", &mut Vec::new()).unwrap(),
        Value::Int(BigInt::from(42))
    );
    let error = session
        .evaluate("math increment 41", &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.code, "E-NAMESPACE-MEMBER-NOT-FOUND");
}

#[test]
fn interface_implementations_require_exact_shapes() {
    let source = "Parser is Interface\n  parse is fn (source : String) -> Boolean\nParser\n  other is fn (source : String) -> Boolean\n    true\n()";
    let error = Session::new()
        .evaluate(source, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-INTERFACE-IMPLEMENTATION");
}

#[test]
fn v02_interface_contracts_are_retained_and_required_of_implementations() {
    let source = "use language ( version is v0.2 )\nParser is Interface\n  parse is fn (source : String) requires true -> result : Boolean ensures result\nParser\n  parse is fn (source : String) requires true -> result : Boolean ensures result\n    true\nparse \"input\"";
    let value = Session::new()
        .evaluate_source_file(source, &mut std::io::sink())
        .unwrap();
    assert_eq!(value, Value::Boolean(true));

    let mismatch = "use language ( version is v0.2 )\nParser is Interface\n  parse is fn (source : String) requires true -> result : Boolean ensures result\nParser\n  parse is fn (source : String) -> Boolean\n    true\n()";
    let error = Session::new()
        .evaluate_source_file(mismatch, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-INTERFACE-IMPLEMENTATION");
}

#[test]
fn exact_infinities_require_context_and_order_range_endpoints() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-COMPARE-001,
    // TOPAL-NUM-INFINITY-ARITHMETIC-001,
    // TOPAL-NUM-THREE-WAY-COMPARE-001, TOPAL-RANGE-BOUNDS-001,
    // TOPAL-RANGE-MEMBERSHIP-001, TOPAL-RANGE-INTERSECTION-001,
    // TOPAL-RANGE-BOUND-001
    let source = include_str!("../../../../examples/language/infinity-values-and-ranges.t");
    assert_eq!(
        Session::new()
            .evaluate_source_file(source, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "(-Infinity, +Infinity, +Infinity, true, true, Less, true, -Infinity, +Infinity, true, true, true, -Infinity ..= +Infinity, 0, false, 0 ..= +Infinity, false, 0 ..= 10, false, true)"
    );
    assert_eq!(
        Session::new()
            .evaluate("+Infinity", &mut std::io::sink())
            .unwrap_err()
            .code,
        "E-INFINITY-CONTEXT"
    );
    assert_eq!(
        Session::new()
            .evaluate("invalid : Nat is -Infinity\ninvalid", &mut std::io::sink(),)
            .unwrap_err()
            .code,
        "E-INFINITY-CLASSIFIER"
    );
    assert_eq!(
        Session::new()
            .evaluate("upper : Int is +Infinity\nupper + 1", &mut std::io::sink())
            .unwrap(),
        Value::Infinity {
            negative: false,
            classifier: "Int".into(),
        }
    );
}

#[test]
fn rational_infinities_preserve_their_domain_and_exact_range_endpoints() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-COMPARE-001,
    // TOPAL-NUM-THREE-WAY-COMPARE-001, TOPAL-RANGE-RATIONAL-001,
    // TOPAL-RANGE-MEMBERSHIP-001, TOPAL-RANGE-INTERSECTION-001,
    // TOPAL-RANGE-BOUND-001
    let source =
        include_str!("../../../../examples/language/rational-infinity-values-and-ranges.t");
    assert_eq!(
        Session::new()
            .evaluate_source_file(source, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "(-Infinity, +Infinity, true, true, Less, true, -Infinity, +Infinity, true, true, true, -Infinity ..= +Infinity, Rational ( 0, 1 ), false, Rational ( 0, 1 ) ..= +Infinity, false, Rational ( -2, 1 ) .. Rational ( 2, 1 ), false, true, Rational ( -1, 1 ) ..= +Infinity)"
    );
    assert_eq!(
        Session::new()
            .evaluate(
                "identity is fn (value : Rational) -> Rational\n  return value\nupper-bound : Rational is +Infinity\nidentity upper-bound",
                &mut std::io::sink(),
            )
            .unwrap(),
        Value::Infinity {
            negative: false,
            classifier: "Rational".into(),
        }
    );
    for invalid in [
        "upper : Rational is +Infinity\nupper + Rational (1, 1)",
        "integer : Int is +Infinity\nratio : Rational is +Infinity\ninteger = ratio",
    ] {
        assert_eq!(
            Session::new()
                .evaluate(invalid, &mut std::io::sink())
                .unwrap_err()
                .code,
            "E-NO-APPLICABLE-OVERLOAD",
            "unexpected diagnostic for {invalid:?}"
        );
    }
}

#[test]
fn exact_infinity_arithmetic_preserves_direction_and_rejects_indeterminate_forms() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001
    let source = include_str!("../../../../examples/language/infinity-arithmetic.t");
    assert_eq!(
        Session::new()
            .evaluate_source_file(source, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "(+Infinity, -Infinity, +Infinity, +Infinity, -Infinity, +Infinity, -Infinity, +Infinity, -Infinity, +Infinity, +Infinity, +Infinity, +Infinity, -Infinity, -Infinity, +Infinity, -Infinity, -Infinity, +Infinity)"
    );
    for (invalid, expected) in [
        (
            "upper : Int is +Infinity\nlower : Int is -Infinity\nupper + lower",
            "E-INDETERMINATE-INFINITY",
        ),
        (
            "upper : Int is +Infinity\nupper - upper",
            "E-INDETERMINATE-INFINITY",
        ),
        (
            "upper : Int is +Infinity\n0 * upper",
            "E-INDETERMINATE-INFINITY",
        ),
        (
            "upper : Rational is +Infinity\nupper * (Rational (0, 1))",
            "E-INDETERMINATE-INFINITY",
        ),
        (
            "integer : Int is +Infinity\nratio : Rational is +Infinity\ninteger + ratio",
            "E-NO-APPLICABLE-OVERLOAD",
        ),
        (
            "upper : Int is +Infinity\nupper / 2",
            "E-NO-APPLICABLE-OVERLOAD",
        ),
    ] {
        assert_eq!(
            Session::new()
                .evaluate(invalid, &mut std::io::sink())
                .unwrap_err()
                .code,
            expected,
            "unexpected diagnostic for {invalid:?}"
        );
    }
}

#[test]
fn exact_infinities_cross_private_function_and_aggregate_boundaries() {
    // TOPAL-NUM-INFINITY-001, TOPAL-NUM-INFINITY-ARITHMETIC-001
    let source = include_str!("../../../../examples/language/infinity-private-boundaries.t");
    assert_eq!(
        Session::new()
            .evaluate_source_file(source, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "(+Infinity, +Infinity, -Infinity, +Infinity, (+Infinity, -Infinity), (integer is +Infinity, ratio is -Infinity), -Infinity, +Infinity, +Infinity)"
    );
}

#[test]
fn dynamic_infinity_multiplication_returns_indeterminate_results() {
    // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-TYPE-RESULT-001
    let source = include_str!("../../../../examples/language/dynamic-infinity-results.t");
    assert_eq!(
        Session::new()
            .evaluate_source_file(source, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "(-Infinity, Error ( domain is root.*(Int,Int), code is indeterminate ), +Infinity, Error ( domain is root.*(Rational,Rational), code is indeterminate ))"
    );
}
