#[test]
fn declares_and_calls_static_nullary_functions() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  40 + 2\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let declared = trace
        .iter()
        .position(|event| event.contains("function.declared"))
        .unwrap();
    let entered = trace
        .iter()
        .position(|event| event.contains("function.entry"))
        .unwrap();
    let returned = trace
        .iter()
        .position(|event| event.contains("function.exit"))
        .unwrap();
    assert!(declared < entered && entered < returned);
}

#[test]
fn static_function_body_uses_declaration_order_lexical_bindings() {
    let value = Session::new()
        .evaluate(
            "base is 40\nanswer is fn static () -> Int\n  base + 2\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  later + 2\nlater is 40\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn static_unary_function_binds_a_typed_local_parameter() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "increment is fn static (input : Int) -> Int\n  input + 1\nincrement 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("function.argument.bound") && event.contains("input") })
    );

    let error = Session::new()
        .evaluate(
            "increment is fn static (input : Int) -> Int\n  input + 1\ninput\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn static_product_function_binds_typed_parameters_in_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "subtract is fn static (left : Int, right : Int) -> Int\n  left - right\n50 subtract 8\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let bindings = trace
        .iter()
        .filter(|event| event.contains("function.argument.bound"))
        .collect::<Vec<_>>();
    assert!(bindings[0].contains("left"));
    assert!(bindings[1].contains("right"));

    let error = Session::new()
        .evaluate(
            "bad is fn static (value : Int, value : Int) -> Int\n  value\nbad (1, 2)\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DUPLICATE-FUNCTION-PARAMETER");
}

#[test]
fn function_block_bindings_are_local_to_each_invocation() {
    let mut trace = Vec::new();
    let mut session = Session::new();
    let value = session
        .evaluate(
            "answer is fn static () -> Int\n  local is 40 + 2\n  local\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let created = trace
        .iter()
        .position(|event| event.contains("binding.bind") && event.contains("local"))
        .unwrap();
    let resolved = trace
        .iter()
        .position(|event| event.contains("binding.resolved") && event.contains("local"))
        .unwrap();
    assert!(created < resolved);

    let error = session
        .evaluate("local\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");

    let error = session
        .evaluate(
            "invalid is fn static () -> Int\n  1\n  2\ninvalid ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-DISCARDED-VALUE");
}

#[test]
fn explicit_return_skips_later_function_statements() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn static () -> Int\n  return 40 + 2\n  missing\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.return.explicit"))
    );
    assert!(!trace.iter().any(|event| event.contains("missing")));

    let error = Session::new()
        .evaluate("return 42\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn lexical_block_return_completes_the_nearest_function() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn (value : Int) -> Int\n  adjusted is value + 1\n  { return adjusted }\n  1000\nanswer 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.return.explicit"))
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));

    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  abandoned : String is {\n    return 42\n    }\n  0\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate("{\n  return 42\n}\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn return_operand_block_propagates_its_inner_return_once() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-block-operand.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));

    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn (value : Int) -> Int\n  return { value + 1 }\nanswer 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
}

#[test]
fn operator_operand_blocks_propagate_returns_in_source_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-operator-operand.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 43)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.entry") && event.contains("preceding"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("missing")));

    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  abandoned : String is 1 + { return 42 }\n  0\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
}

#[test]
fn product_field_blocks_propagate_returns_in_source_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-product-field.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 43)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.entry") && event.contains("preceding"))
            .count(),
        4
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("missing")));

    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  abandoned : String is (1, { return 42 }, missing)\n  0\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
}

#[test]
fn named_call_argument_blocks_propagate_returns_in_source_order() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-call-argument.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 43, 44)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.entry") && event.contains("preceding"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| {
        event.contains("function.entry")
            && (event.contains("combine") || event.contains("identity"))
    }));
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("missing")));

    let value = Session::new()
        .evaluate(
            "identity is fn (value : Int) -> Int\n  value\nanswer is fn () -> Int\n  abandoned : String is identity { return 42 }\n  0\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
}

#[test]
fn optional_constructor_argument_block_propagates_return() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-optional-constructor.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("optional.some.constructed"))
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));
}

#[test]
fn strict_unary_constructor_argument_blocks_propagate_returns() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-unary-constructor.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 43, 44, 45, 46)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        5
    );
    for event in [
        "string.from-character",
        "numeric.int.constructed",
        "numeric.nat.constructed",
        "numeric.rational.constructed",
        "union.constructed",
    ] {
        assert!(!trace.iter().any(|entry| entry.contains(event)), "{event}");
    }
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));
}

#[test]
fn positional_variant_argument_block_propagates_return_after_index_validation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-variant-constructor.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("variant.constructed"))
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    let invalid = "use language (version is v0.1)\nChoice is Variant (Int)\n\nanswer is fn () -> Int\n  Choice at 1 { return 42 }\nanswer ()\n";
    let error = Session::new()
        .evaluate(invalid, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn character_constructor_argument_block_propagates_return_before_validation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-character-constructor.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("constraint.validated"))
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));
}

#[test]
fn named_constraint_argument_block_propagates_return_before_validation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-constraint-constructor.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("constraint.validated"))
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    let forward = "use language (version is v0.1)\nanswer is fn () -> Int\n  Positive { return 42 }\nPositive is Int constraint { candidate } candidate > 0\nanswer ()\n";
    let error = Session::new()
        .evaluate(forward, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn named_modular_argument_block_propagates_return_before_validation() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-modular-constructor.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("numeric.modular.constructed"))
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    let forward = "use language (version is v0.1)\nanswer is fn () -> Int\n  Counter { return 42 }\nCounter is ModNat (0 ..= 255)\nanswer ()\n";
    let error = Session::new()
        .evaluate(forward, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-UNBOUND-NAME");
}

#[test]
fn modular_reduction_operand_block_propagates_return_before_reduction() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-modular-reduction.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(
        !trace
            .iter()
            .any(|event| event.contains("numeric.modular.reduced"))
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    let forward = "use language (version is v0.1)\nanswer is fn () -> Int\n  { return 42 } modulo Counter\nCounter is ModNat (0 ..= 255)\nanswer ()\n";
    let error = Session::new()
        .evaluate(forward, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn unary_list_collect_source_block_propagates_return_before_materialization() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-list-collect.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("list.collected")));
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    let error = Session::new()
        .evaluate("collect { return 42 }\n", &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn unordered_collect_source_blocks_propagate_return_before_materialization() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-unordered-collect.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        2
    );
    assert!(!trace.iter().any(|event| event.contains("set.collected")));
    assert!(!trace.iter().any(|event| event.contains("bag.collected")));
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    for operation in ["collect-set", "collect-bag"] {
        let error = Session::new()
            .evaluate(
                &format!("{operation} {{ return 42 }}\n"),
                &mut std::io::sink(),
            )
            .unwrap_err();
        assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
    }
}

#[test]
fn map_collect_source_block_propagates_return_before_materialization() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-map-collect.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("map.collected")));
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    let invalid_policy =
        "answer is fn () -> Int\n  collect-map { return 42 } resolving merge\nanswer ()\n";
    let error = Session::new()
        .evaluate(invalid_policy, &mut std::io::sink())
        .unwrap_err();
    assert_eq!(error.code, "E-MAP-COLLISION-POLICY");

    let error = Session::new()
        .evaluate(
            "collect-map { return 42 } resolving reject\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn infix_collect_source_blocks_propagate_return_before_materialization() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-infix-collect.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        2
    );
    assert!(!trace.iter().any(|event| event.contains("array.collected")));
    assert!(!trace.iter().any(|event| event.contains("string.collected")));
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("abandoned")));

    for target in ["Array", "String"] {
        let error = Session::new()
            .evaluate(
                &format!("{{ return 42 }} collect {target}\n"),
                &mut std::io::sink(),
            )
            .unwrap_err();
        assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
    }
}

#[test]
fn boolean_decision_subject_block_propagates_return_before_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-decision-subject.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("decision.rule")));
    assert!(!trace.iter().any(|event| event.contains("1000")));

    let error = Session::new()
        .evaluate(
            "{ return 42 }\n  true then false\n  otherwise true\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn exhaustive_boolean_decision_subject_block_propagates_return_before_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-exhaustive-boolean-subject.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("decision.rule")));
    assert!(!trace.iter().any(|event| event.contains("1000")));

    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  { return 42 }\n    true then 0\n    false then 1\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate(
            "{ return 42 }\n  false then 0\n  true then 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn comparison_decision_subject_block_propagates_return_before_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-comparison-decision-subject.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("decision.rule")));
    assert!(!trace.iter().any(|event| event.contains("1000")));

    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  { return 42 }\n    >= 0 then 0\n    otherwise 1\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate(
            "{ return 42 }\n  < 0 then 0\n  otherwise 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn fallback_decision_subject_block_propagates_return_before_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-fallback-decision-subject.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("decision.rule")));
    assert!(!trace.iter().any(|event| event.contains("1000")));

    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  { return 42 }\n    otherwise 0\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate(
            "{ return 42 }\n  Some payload then payload\n  otherwise 0\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn complete_decision_subject_blocks_propagate_return_before_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-complete-decision-subject.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(40, 41, 42)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        3
    );
    assert!(!trace.iter().any(|event| event.contains("decision.rule")));
    assert!(!trace.iter().any(|event| event.contains("1000")));

    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  { return 42 }\n    Red then 0\nanswer ()\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");

    let error = Session::new()
        .evaluate(
            "{ return 42 }\n  Some payload then payload\n  None then 0\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-RETURN-OUTSIDE-FUNCTION");
}

#[test]
fn complete_boolean_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-boolean-decision-actions.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(40, 41)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("decision.rule.selected"))
            .count(),
        2
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn complete_comparison_value_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-comparison-value-decision-actions.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(40, 41, 42)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-ENUM-001")
            })
            .count(),
        3
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn complete_ordered_comparison_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-ordered-comparison-decision-actions.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(40, 41, 42)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-COMPARISON-001")
            })
            .count(),
        3
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn final_fallback_enum_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-enum-fallback-decision-actions.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(40, 41, 42)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-ENUM-001")
            })
            .count(),
        3
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn exhaustive_enum_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-exhaustive-enum-decision-actions.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(43, 44, 45)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-ENUM-001")
            })
            .count(),
        3
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn optional_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-optional-decision-actions.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(43, 40, 44, 45)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        4
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-OPTIONAL-001")
            })
            .count(),
        4
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("optional.payload.bound"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
    assert!(!trace.iter().any(|event| event.contains("1001")));

    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "choose is fn (candidate : Optional Int) -> Int\n  candidate\n    otherwise { return 42 }\n  1000\nchoose (None Int)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(trace.iter().any(|event| {
        event.contains("decision.rule.selected") && event.contains("TOPAL-DECISION-OPTIONAL-001")
    }));
    assert!(!trace.iter().any(|event| {
        event.contains("decision.rule.selected") && event.contains("TOPAL-DECISION-BOOLEAN-001")
    }));
}

#[test]
fn result_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-result-decision-actions.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, true)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-RESULT-001")
            })
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("result.payload.bound"))
            .count(),
        2
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn error_code_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!(
                "../../../../examples/language/function-return-error-code-decision-actions.t"
            ),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(true, false, true, 0, 3, 4)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        6
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-RESULT-001")
            })
            .count(),
        6
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("error.code.matched"))
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("result.payload.bound"))
            .count(),
        3
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn list_decision_actions_propagate_return_after_selection() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            include_str!("../../../../examples/language/function-return-list-decision-actions.t"),
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(43, 40)");
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("function.return.explicit"))
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| {
                event.contains("decision.rule.selected")
                    && event.contains("TOPAL-DECISION-LIST-001")
            })
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event.contains("list.entry.decomposed"))
            .count(),
        1
    );
    assert!(!trace.iter().any(|event| event.contains("1000")));
}

#[test]
fn ordinary_runtime_function_uses_ordinary_trace_rule() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "subtract is fn (left : Int, right : Int) -> Int\n  left - right\n50 subtract 8\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .filter(|event| event.contains("function."))
            .all(|event| event.contains("TOPAL-FUNCTION-ORDINARY-001")
                || event.contains("TOPAL-TYPE-CALL-001"))
    );
}

#[test]
fn nat_classifiers_accept_only_nonnegative_int_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "identity is fn (value : Nat) -> Nat\n  value\nidentity 42\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(trace.iter().any(|event| event.contains("identity (Nat)")));

    let argument_error = Session::new()
        .evaluate(
            "identity is fn (value : Nat) -> Nat\n  value\nidentity -1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(argument_error.code, "E-FUNCTION-ARGUMENT-TYPE");

    let result_error = Session::new()
        .evaluate(
            "negative is fn () -> Nat\n  -1\nnegative ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(result_error.code, "E-FUNCTION-RESULT-TYPE");
}

#[test]
fn proves_unit_step_nat_recursion_without_overshoot() {
    let source = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 0 then 0\n    otherwise count-down (value - 1)\ncount-down 3\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "0"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-001"))
    );

    let error = Session::new()
        .evaluate(
            &source.replace("value - 1", "value - 2"),
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn proves_nat_decrement_when_the_bound_prevents_overshoot() {
    let safe = "count-down is fn (value : Nat) -> Nat\n  value\n    <= 2 then value\n    otherwise count-down (value - 3)\ncount-down 8\n";
    assert_eq!(
        Session::new()
            .evaluate(safe, &mut std::io::sink())
            .unwrap()
            .to_string(),
        "2"
    );
    let error = Session::new()
        .evaluate(
            &safe.replace("value - 3", "value - 4"),
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(error.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn proves_increasing_nat_recursion_with_positive_steps() {
    let source = "advance is fn (value : Nat) -> Nat\n  value\n    >= 5 then value\n    otherwise advance (value + 2)\nadvance 0\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "6"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"))
    );
}

#[test]
fn proves_closed_mutual_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"))
    );
}

#[test]
fn proves_closed_mutual_increasing_nat_recursion() {
    let source = "even is fn (value : Nat) -> Boolean\n  value\n    >= 6 then true\n    otherwise odd (value + 1)\nodd is fn (value : Nat) -> Boolean\n  value\n    >= 6 then false\n    otherwise even (value + 1)\n(even 0, odd 0)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(true, false)"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"))
    );
}

#[test]
fn declares_nominal_payload_free_enum_values() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "Color is Enum (Red, Green, Blue)\n(Red, Green, Red = Red, Red = Green)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(Red, Green, true, false)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-TYPE-ENUM-001"))
    );
}

#[test]
fn validates_enum_function_parameters_and_results() {
    let source = "Color is Enum (Red, Green)\nidentity is fn (value : Color) -> Color\n  value\n(identity Red, identity Green)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(Red, Green)"
    );
    assert!(trace.iter().any(|event| event.contains("identity (Color)")));
}

#[test]
fn executes_only_complete_enum_decisions() {
    let source = "Color is Enum (Red, Green)\nname is fn (value : Color) -> String\n  value\n    Red then \"red\"\n    Green then \"green\"\nname Green\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "\"green\""
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-ENUM-001"))
    );
}

#[test]
fn resolves_namespaced_arithmetic_error_codes_without_a_domain() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "(lang arithmetic division-by-zero) = (lang arithmetic division-by-zero)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(true));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-NUM-ARITHMETIC-ERROR-001"))
    );
}

#[test]
fn matches_both_result_paths_exhaustively() {
    let source = "divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error problem then \"error\"\n(describe 2.0, describe 0.0)\n";
    let mut trace = Vec::new();
    assert_eq!(
        Session::new()
            .evaluate(source, &mut trace)
            .unwrap()
            .to_string(),
        "(\"ok\", \"error\")"
    );
    assert!(
        trace
            .iter()
            .any(|event| event.contains("TOPAL-DECISION-RESULT-001"))
    );
}

#[test]
fn nested_function_calls_preserve_staticness_and_detect_cycles() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "answer is fn () -> Int\n  increment 41\nincrement is fn (input : Int) -> Int\n  input + 1\nanswer ()\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let outer_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("answer"))
        .unwrap();
    let inner_entry = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("increment"))
        .unwrap();
    let inner_return = trace
        .iter()
        .position(|event| event.contains("function.exit") && event.contains("increment"))
        .unwrap();
    let outer_return = trace
        .iter()
        .position(|event| event.contains("function.exit") && event.contains("answer"))
        .unwrap();
    assert!(outer_entry < inner_entry && inner_entry < inner_return && inner_return < outer_return);

    let recursion = Session::new()
        .evaluate(
            "again is fn () -> Int\n  again ()\nagain ()\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(recursion.code, "E-UNPROVEN-RECURSION");
}

#[test]
fn function_local_binding_shadows_capture_without_leaking() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "value is 40\nanswer is fn () -> Int\n  value is 42\n  value\n(answer (), value)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 40)");

    let duplicate = Session::new()
        .evaluate(
            "bad is fn (value : Int) -> Int\n  value is 42\n  value\nbad 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-BINDING");
}

#[test]
fn overload_selection_uses_input_classifier_and_rejects_duplicate_signature() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "describe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  value\n(describe 42, describe \"Topal\")\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(\"integer\", \"Topal\")");
    assert!(trace.iter().any(|event| event.contains("describe (Int)")));
    assert!(
        trace
            .iter()
            .any(|event| event.contains("describe (String)"))
    );

    let duplicate = Session::new()
        .evaluate(
            "same is fn (first : Int) -> Int\n  first\nsame is fn (second : Int) -> String\n  \"duplicate\"\nsame 1\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(duplicate.code, "E-DUPLICATE-FUNCTION-OVERLOAD");
}

#[test]
fn boolean_decision_evaluates_only_selected_action() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise missing\nchoose true\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
    assert!(!trace.iter().any(|event| event.contains("missing")));
}

#[test]
fn exhaustive_boolean_decision_selects_both_literal_cases() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    false then 0\n(choose true, choose false)\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "(42, 0)");
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=0") })
    );
    assert!(
        trace
            .iter()
            .any(|event| { event.contains("decision.rule.selected") && event.contains("rule=1") })
    );
}

#[test]
fn earlier_function_body_calls_later_declaration() {
    let mut trace = Vec::new();
    let value = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Int\n  second value\nsecond is fn (value : Int) -> Int\n  value + 1\nfirst 41\n",
            &mut trace,
        )
        .unwrap();
    assert_eq!(value.to_string(), "42");
    let first = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("first"))
        .unwrap();
    let second = trace
        .iter()
        .position(|event| event.contains("function.entry") && event.contains("second"))
        .unwrap();
    assert!(first < second);
}

#[test]
fn mutual_int_recursion_executes_only_when_every_cycle_edge_decreases() {
    let source = "even is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\n(even 6, odd 6)\n";
    let mut trace = Vec::new();
    let value = Session::new().evaluate(source, &mut trace).unwrap();
    assert_eq!(value.to_string(), "(true, false)");
    assert!(
        trace
            .iter()
            .any(|event| event.contains("function.recursion.cycle.proven"))
    );

    let three_member = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise third (value - 1)\nthird is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first (value - 1)\nfirst 3\n",
            &mut std::io::sink(),
        )
        .unwrap();
    assert_eq!(three_member.to_string(), "true");

    let invalid = Session::new()
        .evaluate(
            "first is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise second (value - 1)\nsecond is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise first value\nfirst 2\n",
            &mut std::io::sink(),
        )
        .unwrap_err();
    assert_eq!(invalid.code, "E-UNPROVEN-RECURSION");
}
