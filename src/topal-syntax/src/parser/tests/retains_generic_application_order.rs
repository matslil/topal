use super::*;
use crate::lex;

#[test]
fn retains_generic_application_order() {
    let source = SourceText::new("1 + 2 * 3").unwrap();
    let parsed = parse(&source, &lex(&source));
    let Statement::Expression(Expression::Application { items, .. }) = &parsed.statements[0] else {
        panic!("expected application");
    };
    assert_eq!(items.len(), 5);
    assert!(matches!(
        items[1],
        Expression::Callable {
            kind: CallableKind::Plus,
            ..
        }
    ));
    assert!(matches!(
        items[3],
        Expression::Callable {
            kind: CallableKind::Multiply,
            ..
        }
    ));
    assert!(parsed.diagnostics.is_empty());
}

#[test]
fn parses_the_zero_field_product_as_unit() {
    let source = SourceText::new("()").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        parsed.statements,
        vec![Statement::Expression(Expression::Unit(Span::new(0, 2)))]
    );
}

#[test]
fn parses_inline_single_statement_block_before_closing_brace() {
    let source = SourceText::new("{ return 42 }").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements.as_slice(),
        [Statement::Expression(Expression::Block { statements, .. })]
            if matches!(statements.as_slice(), [Statement::Return { .. }])
    ));
}

#[test]
fn distinguishes_grouping_and_positional_products() {
    let grouped_source = SourceText::new("(1)").unwrap();
    let grouped = parse(&grouped_source, &lex(&grouped_source));
    assert!(matches!(
        grouped.statements[0],
        Statement::Expression(Expression::Integer(_))
    ));

    let tuple_source = SourceText::new("(1, 2,)").unwrap();
    let tuple = parse(&tuple_source, &lex(&tuple_source));
    let Statement::Expression(Expression::Product { fields, .. }) = &tuple.statements[0] else {
        panic!("expected tuple");
    };
    assert_eq!(fields.len(), 2);
    assert!(tuple.diagnostics.is_empty());
}

#[test]
fn ignores_newlines_inside_parentheses() {
    let source = SourceText::new("(\n1,\n2\n)").unwrap();
    let parsed = parse(&source, &lex(&source));
    let Statement::Expression(Expression::Product { fields, .. }) = &parsed.statements[0] else {
        panic!("expected tuple");
    };
    assert_eq!(fields.len(), 2);
    assert!(parsed.diagnostics.is_empty());
}

#[test]
fn retains_labels_on_record_product_fields() {
    let source = SourceText::new("(name is \"Ada\", active is true)").unwrap();
    let parsed = parse(&source, &lex(&source));
    let Statement::Expression(Expression::Product { fields, .. }) = &parsed.statements[0] else {
        panic!("expected product");
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(source.slice(fields[0].label.unwrap()), "name");
    assert_eq!(source.slice(fields[1].label.unwrap()), "active");
    assert!(parsed.diagnostics.is_empty());
}

#[test]
fn rejects_products_that_mix_positional_and_labeled_fields() {
    let source = SourceText::new("(1, name is \"Ada\")").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert_eq!(parsed.diagnostics[0].code, "E-MIXED-PRODUCT-FIELDS");
    assert_eq!(source.slice(parsed.diagnostics[0].span), "name");
}

#[test]
fn retains_incomplete_parentheses_for_recovery() {
    let source = SourceText::new("(\n1,").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-RPAREN");
}

#[test]
fn retains_named_and_prefix_application_terms() {
    let named_source = SourceText::new("print value").unwrap();
    let named = parse(&named_source, &lex(&named_source));
    let Statement::Expression(Expression::Application { items, .. }) = &named.statements[0] else {
        panic!("expected named application");
    };
    assert_eq!(items.len(), 2);

    let prefix_source = SourceText::new("- 42").unwrap();
    let prefix = parse(&prefix_source, &lex(&prefix_source));
    let Statement::Expression(Expression::Application { items, .. }) = &prefix.statements[0] else {
        panic!("expected prefix application");
    };
    assert!(matches!(
        items[0],
        Expression::Callable {
            kind: CallableKind::Minus,
            ..
        }
    ));
}

#[test]
fn retains_same_line_and_continued_function_effect_bounds() {
    for text in [
        "read is fn ( value : Int ) -> Int : Effects ()\n  value\n",
        "read is fn ( value : Int ) -> Int\n  : Effects ()\n  value\n",
    ] {
        let source = SourceText::new(text).unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics, []);
        let Statement::Function {
            effect_bound: Some(bound),
            ..
        } = parsed.statements[0]
        else {
            panic!("expected a function effect bound");
        };
        assert_eq!(source.slice(bound), "Effects ()");
    }
}

#[test]
fn retains_v02_function_clauses_around_their_arrow() {
    let text = "withdraw is fn ( account : Int, amount : Int )\n  requires ( amount <= account )\n  effects ( Effects () )\n  guarantees ( Prefer NoAlloc )\n-> result : Int\n  ensures ( result <= account )\n  result\n";
    let source = SourceText::new(text).unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function {
        result, clauses, ..
    } = &parsed.statements[0]
    else {
        panic!("expected function");
    };
    assert_eq!(source.slice(*result), "Int");
    assert_eq!(
        source.slice(clauses.result_binding.expect("result binding")),
        "result"
    );
    assert!(clauses.requires.is_some());
    assert!(clauses.effects.is_some());
    assert!(clauses.guarantees.is_some());
    assert!(clauses.ensures.is_some());
}

#[test]
fn keeps_exclusive_and_consumes_on_parameters() {
    let text = "rewrite is fn ( input : String : Exclusive ) -> String\n  input\n";
    let source = SourceText::new(text).unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { parameters, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    assert_eq!(
        source.slice(parameters[0].qualifier.expect("qualifier")),
        "Exclusive"
    );
}

#[test]
fn rejects_out_of_order_function_clauses() {
    let text = "bad is fn ()\n  guarantees NoAlloc\n  requires true\n-> Unit\n  ()\n";
    let source = SourceText::new(text).unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E-FUNCTION-CLAUSE-ORDER")
    );
}

#[test]
fn retains_packaged_operand_fields_and_defaults() {
    let source = SourceText::new(
        "choose is fn ( ( value : Int, fallback : Int default 0 ) ) -> Int\n  value\n",
    )
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert_eq!(parsed.diagnostics, []);
    let Statement::Function { parameters, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].fields.len(), 2);
    assert!(parameters[0].fields[0].default.is_none());
    assert!(parameters[0].fields[1].default.is_some());
}

#[test]
fn retains_task_implementation_state_handlers_and_context_assignment() {
    let source = SourceText::new(
            "Counter is Task (queue-size is 10)\ncounter-service is Counter\n  count : Nat\n  start is fn ( initial : Nat ) -> Completed\n    @ count is initial\n    Completed\n",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert_eq!(parsed.diagnostics, []);
    let Statement::Implementation { declarations, .. } = &parsed.statements[1] else {
        panic!("expected task implementation");
    };
    assert!(matches!(declarations[0], Statement::StateField { .. }));
    let Statement::Function { body, .. } = &declarations[1] else {
        panic!("expected lifecycle handler");
    };
    assert!(matches!(body[0], Statement::ContextAssignment { .. }));
}

#[test]
fn retains_incomplete_application_for_semantic_recovery() {
    let source = SourceText::new("1 +").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(matches!(
        parsed.statements[0],
        Statement::Expression(Expression::Application { .. })
    ));
    assert!(parsed.diagnostics.is_empty());
}

#[test]
fn recovers_incomplete_binding() {
    let source = SourceText::new("answer is").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-EXPRESSION");
}

#[test]
fn rejects_boolean_literal_as_a_binding_name() {
    let source = SourceText::new("true is 1").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert_eq!(parsed.diagnostics[0].code, "E-RESERVED-BOOLEAN-LITERAL");
}

#[test]
fn parses_static_nullary_function_with_indented_body() {
    let source = SourceText::new("answer is fn static () -> Int\n  40 + 2\nanswer ()").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty());
    let Statement::Function {
        name, result, body, ..
    } = &parsed.statements[0]
    else {
        panic!("expected static function declaration");
    };
    assert_eq!(source.slice(*name), "answer");
    assert_eq!(source.slice(*result), "Int");
    assert!(matches!(
        body[0],
        Statement::Expression(Expression::Application { .. })
    ));
    assert!(matches!(parsed.statements[1], Statement::Expression(_)));
}

#[test]
fn parses_one_typed_static_function_parameter() {
    let source =
        SourceText::new("increment is fn static (input : Int) -> Int\n  input + 1\nincrement 41")
            .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty());
    let Statement::Function { parameters, .. } = &parsed.statements[0] else {
        panic!("expected one static parameter");
    };
    assert_eq!(parameters.len(), 1);
    assert_eq!(source.slice(parameters[0].name), "input");
    assert_eq!(source.slice(parameters[0].classifier), "Int");
}

#[test]
fn parses_multiple_typed_static_function_parameters() {
    let source = SourceText::new(
        "add is fn static (left : Int, right : Int) -> Int\n  left + right\n20 add 22",
    )
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty());
    let Statement::Function { parameters, .. } = &parsed.statements[0] else {
        panic!("expected static function parameters");
    };
    assert_eq!(parameters.len(), 2);
    assert_eq!(source.slice(parameters[0].name), "left");
    assert_eq!(source.slice(parameters[1].name), "right");
}

#[test]
fn preserves_record_function_boundary_classifiers() {
    // TOPAL-COMPILER-RECORD-BOUNDARY-001, TOPAL-TYPE-PRODUCT-001
    let source = SourceText::new(
            "retain is fn (value : Record (active : Boolean, name : String)) -> Record (active : Boolean, name : String)\n  value\nretain (name is \"Ada\", active is true)",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function {
        parameters, result, ..
    } = &parsed.statements[0]
    else {
        panic!("expected function declaration");
    };
    assert_eq!(
        source.slice(parameters[0].classifier),
        "Record (active : Boolean, name : String)"
    );
    assert_eq!(
        source.slice(*result),
        "Record (active : Boolean, name : String)"
    );
}

#[test]
fn preserves_structural_higher_order_generic_classifiers() {
    let source = SourceText::new(
            "map is fn (candidate : Optional (Input : Type), transformation : fn (Input) -> Output) -> Optional Output\n  candidate",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function {
        parameters, result, ..
    } = &parsed.statements[0]
    else {
        panic!("expected function declaration");
    };
    assert_eq!(
        source.slice(parameters[0].classifier),
        "Optional (Input : Type)"
    );
    assert_eq!(
        source.slice(parameters[1].classifier),
        "fn (Input) -> Output"
    );
    assert_eq!(source.slice(*result), "Optional Output");
}

#[test]
fn rejects_more_than_two_function_operands() {
    let source =
        SourceText::new("invalid is fn static (one : Int, two : Int, three : Int) -> Int\n  one")
            .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert_eq!(parsed.diagnostics[0].code, "E-FUNCTION-OPERAND-COUNT");
}

#[test]
fn parses_a_multi_statement_function_body() {
    let source =
        SourceText::new("answer is fn static () -> Int\n  value is 40 + 2\n  value\nanswer ()")
            .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty());
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected static function body");
    };
    assert_eq!(body.len(), 2);
    assert!(matches!(body[0], Statement::Binding { .. }));
    assert!(matches!(body[1], Statement::Expression(_)));
    assert_eq!(parsed.statements.len(), 2);
}

#[test]
fn parses_explicit_return_as_a_function_statement() {
    let source =
        SourceText::new("answer is fn static () -> Int\n  return 42\n  0\nanswer ()").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty());
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected static function body");
    };
    assert!(matches!(body[0], Statement::Return { .. }));
}

#[test]
fn distinguishes_ordinary_and_static_function_headers() {
    let ordinary_source = SourceText::new("ordinary is fn () -> Int\n  42").unwrap();
    let ordinary = parse(&ordinary_source, &lex(&ordinary_source));
    let Statement::Function { is_static, .. } = ordinary.statements[0] else {
        panic!("expected ordinary function");
    };
    assert!(!is_static);

    let static_source = SourceText::new("compile is fn static () -> Int\n  42").unwrap();
    let static_function = parse(&static_source, &lex(&static_source));
    let Statement::Function { is_static, .. } = static_function.statements[0] else {
        panic!("expected static function");
    };
    assert!(is_static);
}

#[test]
fn parses_complete_boolean_decision_table() {
    let source = SourceText::new(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    otherwise 0\nchoose true",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
        panic!("expected decision table");
    };
    assert_eq!(rules.len(), 2);
    assert!(matches!(
        rules[0].matcher,
        DecisionMatcher::Boolean { value: true, .. }
    ));
    assert!(matches!(rules[1].matcher, DecisionMatcher::Otherwise(_)));
}

#[test]
fn parses_exhaustive_boolean_decision_without_otherwise() {
    let source = SourceText::new(
            "choose is fn (condition : Boolean) -> Int\n  condition\n    true then 42\n    false then 0\nchoose false",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
        panic!("expected decision table");
    };
    assert_eq!(rules.len(), 2);
    assert!(matches!(
        rules[0].matcher,
        DecisionMatcher::Boolean { value: true, .. }
    ));
    assert!(matches!(
        rules[1].matcher,
        DecisionMatcher::Boolean { value: false, .. }
    ));
}

#[test]
fn preserves_named_enum_decision_matchers() {
    let source = SourceText::new(
            "name is fn (value : Color) -> String\n  value\n    Red then \"red\"\n    Green then \"green\"",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty());
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
        panic!("expected function decision table");
    };
    assert!(matches!(rules[0].matcher, DecisionMatcher::Identifier(_)));
    assert!(matches!(rules[1].matcher, DecisionMatcher::Identifier(_)));
}

#[test]
fn parses_a_body_calling_a_later_function_declaration() {
    let source = SourceText::new(
            "first is fn (value : Int) -> Int\n  second value\nsecond is fn (value : Int) -> Int\n  value\nfirst 42",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(parsed.statements[0], Statement::Function { .. }));
    assert!(matches!(parsed.statements[1], Statement::Function { .. }));
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        unreachable!();
    };
    assert!(matches!(
        &body[0],
        Statement::Expression(Expression::Application { .. })
    ));
}

#[test]
fn parses_mutually_recursive_function_declarations() {
    let source = SourceText::new(
            "even is fn (value : Int) -> Boolean\n  value\n    <= 0 then true\n    otherwise odd (value - 1)\nodd is fn (value : Int) -> Boolean\n  value\n    <= 0 then false\n    otherwise even (value - 1)\neven 4",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(parsed.statements[0], Statement::Function { .. }));
    assert!(matches!(parsed.statements[1], Statement::Function { .. }));
}

#[test]
fn parses_comparison_decision_matcher() {
    let source = SourceText::new(
            "minimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise right\nminimum (1, 2)",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
        panic!("expected decision table");
    };
    assert!(matches!(
        &rules[0].matcher,
        DecisionMatcher::Comparison {
            kind: CallableKind::Less,
            ..
        }
    ));
}

#[test]
fn parses_complete_comparison_operand_before_then() {
    let source = SourceText::new(
            "within is fn (value : Int, limit : Int) -> Boolean\n  value\n    < limit + 1 then true\n    otherwise false\n1 within 1",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
        panic!("expected decision table");
    };
    assert!(matches!(
        &rules[0].matcher,
        DecisionMatcher::Comparison {
            operand: Expression::Application { .. },
            ..
        }
    ));
}

#[test]
fn parses_nested_function_declaration_in_body() {
    let source = SourceText::new(
            "answer is fn (input : Int) -> Int\n  helper is fn (value : Int) -> Int\n    value + input\n  helper 2\nanswer 40",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected outer function");
    };
    assert!(matches!(body[0], Statement::Function { .. }));
    assert!(matches!(body[1], Statement::Expression(_)));
}

#[test]
fn parses_qualified_error_code_decision_matcher() {
    let source = SourceText::new(
            "describe is fn (attempt : Result) -> Int\n  attempt\n    Ok value then value\n    Error ( code is lang arithmetic division-by-zero ) then 0\n    Error problem then 1",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { body, .. } = &parsed.statements[0] else {
        panic!("expected function");
    };
    let Statement::Expression(Expression::DecisionTable { rules, .. }) = &body[0] else {
        panic!("expected decision table");
    };
    let DecisionMatcher::ErrorCode {
        namespace,
        vocabulary,
        code,
        ..
    } = rules[1].matcher
    else {
        panic!("expected Error code matcher");
    };
    assert_eq!(source.slice(namespace), "lang");
    assert_eq!(source.slice(vocabulary), "arithmetic");
    assert_eq!(source.slice(code), "division-by-zero");
}

#[test]
fn parses_classified_binding() {
    let source = SourceText::new("value : Rational is operation input").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Binding {
        name, classifier, ..
    } = parsed.statements[0]
    else {
        panic!("expected binding");
    };
    assert_eq!(source.slice(name), "value");
    assert_eq!(source.slice(classifier.unwrap()), "Rational");
}

#[test]
fn parses_named_generator_declaration() {
    let source = SourceText::new(
            "once is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  _ is yield initial\n  ()",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Generator {
        name,
        parameters,
        yielded,
        resumed,
        result,
        body,
        ..
    } = &parsed.statements[0]
    else {
        panic!("expected generator");
    };
    assert_eq!(source.slice(*name), "once");
    assert_eq!(parameters.len(), 1);
    assert_eq!(source.slice(*yielded), "Character");
    assert_eq!(source.slice(*resumed), "Unit");
    assert_eq!(source.slice(*result), "Unit");
    assert_eq!(body.len(), 2);
}

#[test]
fn parses_multi_input_generator_declaration() {
    let source = SourceText::new(
            "select is generator ( value : Int, suffix : String )\n  yields String\n  resumes Unit\n  -> String\n\n  _ is yield suffix\n  \"done\"",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Generator { parameters, .. } = &parsed.statements[0] else {
        panic!("expected generator");
    };
    assert_eq!(parameters.len(), 2);
    assert_eq!(source.slice(parameters[0].classifier), "Int");
    assert_eq!(source.slice(parameters[1].classifier), "String");
}

#[test]
fn parses_foreach_result_binding() {
    let source =
        SourceText::new("result is generated foreach { value }\n  _ is value + 1").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Foreach {
        result,
        source: generator,
        ..
    } = &parsed.statements[0]
    else {
        panic!("expected foreach");
    };
    assert_eq!(source.slice(result.unwrap().0), "result");
    assert_eq!(source.slice(generator.span()), "generated");
}

#[test]
fn parses_classified_foreach_result_binding() {
    let source =
        SourceText::new("result : String is generated foreach { value }\n  _ is value + 1")
            .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Foreach {
        result: Some((name, Some(classifier))),
        ..
    } = &parsed.statements[0]
    else {
        panic!("expected classified foreach result");
    };
    assert_eq!(source.slice(*name), "result");
    assert_eq!(source.slice(*classifier), "String");
}

#[test]
fn parses_explicit_generator_return() {
    let source = SourceText::new(
            "done is generator ( initial : String )\n  yields String\n  resumes Unit\n  -> String\n\n  return \"done\"",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Generator { body, .. } = &parsed.statements[0] else {
        panic!("expected generator");
    };
    assert!(matches!(body.as_slice(), [Statement::Return { .. }]));
}

#[test]
fn parses_optional_generator_classifiers() {
    let source = SourceText::new(
            "optional is generator ( initial : Optional Int )\n  yields Optional Int\n  resumes Unit\n  -> Optional Int\n\n  _ is yield initial\n  None Int",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Generator {
        parameters,
        yielded,
        result,
        ..
    } = &parsed.statements[0]
    else {
        panic!("expected generator");
    };
    assert_eq!(source.slice(parameters[0].classifier), "Optional Int");
    assert_eq!(source.slice(*yielded), "Optional Int");
    assert_eq!(source.slice(*result), "Optional Int");
}

#[test]
fn parses_product_generator_classifiers() {
    let source = SourceText::new("pair is generator ( initial : (Int, String) )\n  yields (Int, String)\n  resumes Unit\n  -> (Int, String)\n\n  _ is yield initial\n  (8, \"done\")").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Generator {
        parameters,
        yielded,
        result,
        ..
    } = &parsed.statements[0]
    else {
        panic!("expected generator")
    };
    assert_eq!(source.slice(parameters[0].classifier), "(Int, String)");
    assert_eq!(source.slice(*yielded), "(Int, String)");
    assert_eq!(source.slice(*result), "(Int, String)");
}

#[test]
fn parses_result_generator_classifiers() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/custom-generator-result-values.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn parses_compound_generator_function_classifiers() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/custom-generator-compound-function-boundaries.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn parses_nested_generator_function_classifiers() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/custom-generator-nested-function-boundaries.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn parses_list_generator_function_classifiers() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/custom-generator-list-values.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn parses_nested_optional_generator_classifiers() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/custom-generator-nested-optional-values.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn accepts_complete_closed_arithmetic_error_code_set() {
    let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic out-of-range ) then \"range\"\n    Error ( code is lang arithmetic not-representable ) then \"representation\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"\n    Error ( code is lang arithmetic indeterminate ) then \"indeterminate\"",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn incomplete_arithmetic_error_decision_lists_missing_codes() {
    let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    let diagnostic = parsed.diagnostics.first().unwrap();
    assert_eq!(diagnostic.code, "E-INCOMPLETE-ERROR-CODE-DECISION");
    assert!(diagnostic.message.contains("out-of-range"));
    assert!(diagnostic.message.contains("not-representable"));
    assert!(diagnostic.message.contains("indeterminate"));
}

#[test]
fn duplicate_arithmetic_error_code_pattern_is_rejected() {
    let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"first\"\n    Error ( code is lang arithmetic division-by-zero ) then \"second\"\n    Error problem then \"other\"",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    let diagnostic = parsed.diagnostics.first().unwrap();
    assert_eq!(diagnostic.code, "E-DUPLICATE-ERROR-CODE-PATTERN");
    assert!(diagnostic.message.contains("division-by-zero"));
    assert_eq!(source.slice(diagnostic.span), "division-by-zero");
}

#[test]
fn parses_qualified_generator_error_code_pattern() {
    let source = SourceText::new(
            "handle is generator ( initial : Character )\n  yields Character\n  resumes Unit\n  -> Unit\n\n  result is yield initial\n  result\n    Error ( code is lang generator generator-closed ) then ()\n    Error problem then ()\n    Ok resumed then ()",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn error_code_pattern_after_generic_fallback_is_rejected() {
    let source = SourceText::new(
            "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error problem then \"other\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\"",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    let diagnostic = parsed.diagnostics.first().unwrap();
    assert_eq!(diagnostic.code, "E-UNREACHABLE-ERROR-CODE-PATTERN");
    assert_eq!(
        source.slice(diagnostic.span),
        "Error ( code is lang arithmetic division-by-zero )"
    );
}

#[test]
fn rule_after_otherwise_is_rejected() {
    let source = SourceText::new(
        "choose is fn (condition : Boolean) -> Int\n  condition\n    otherwise 0\n    true then 1",
    )
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    let diagnostic = parsed.diagnostics.first().unwrap();
    assert_eq!(diagnostic.code, "E-UNREACHABLE-DECISION-RULE");
    assert_eq!(source.slice(diagnostic.span), "true");
}

#[test]
fn parses_list_construction_and_total_decomposition() {
    let source = SourceText::new(include_str!("../../../../../examples/language/lists.t")).unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn parses_recursive_list_classifiers() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/nested-lists.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn parses_collection_function_classifiers() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/collection-packaged-fields.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Function { result, .. } = &parsed.statements[1] else {
        panic!("expected the Array-producing function")
    };
    assert_eq!(source.slice(*result), "Array (3, Int)");
}

#[test]
fn parses_contextual_anonymous_list_functions() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/anonymous-list-functions.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn parses_recursive_anonymous_product_patterns() {
    // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-SYN-GRAMMAR-001
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/nested-anonymous-patterns.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::Binding {
        value:
            Expression::AnonymousFunction {
                parameters,
                body: _,
                span: _,
            },
        ..
    } = &parsed.statements[2]
    else {
        panic!("expected the nested anonymous Function binding")
    };
    let [AnonymousPattern::Product { fields, .. }] = parameters.as_slice() else {
        panic!("expected one outer product pattern")
    };
    assert!(matches!(
        fields.as_slice(),
        [AnonymousPattern::Binding(_), AnonymousPattern::Product { fields: nested, .. }]
            if nested.len() == 2
    ));
}

#[test]
fn parses_payload_unions_and_positional_variants() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/unions-and-recursive-products.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn malformed_source_corpus_never_panics() {
    for input in ["(", "{", "fn", "x is", "x\n  then", "\0", "😀 ( , )"] {
        if let Ok(source) = SourceText::new(input) {
            let _ = parse(&source, &lex(&source));
        }
    }
}

#[test]
fn parses_balanced_diagnostic_controls() {
    let source = SourceText::new(include_str!(
        "../../../../../examples/language/diagnostic-controls.t"
    ))
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements[1],
        Statement::DiagnosticControl {
            operation: DiagnosticControlKind::DisableNext,
            ..
        }
    ));
}

#[test]
fn rejects_unbalanced_diagnostic_controls() {
    for (input, code) in [
        (
            "lang pop-disable-warning unused",
            "E-DIAGNOSTIC-CONTROL-UNDERFLOW",
        ),
        (
            "lang push-disable-warning unused\n()",
            "E-DIAGNOSTIC-CONTROL-UNCLOSED",
        ),
        ("lang disable-warning unused", "E-DIAGNOSTIC-CONTROL-TARGET"),
    ] {
        let source = SourceText::new(input).unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.iter().any(|error| error.code == code));
    }
}

#[test]
fn requires_matching_structured_diagnostic_identities() {
    let source = SourceText::new(
            "lang push-disable-diagnostic ( lang best-practice task state-machine )\n()\nlang pop-disable-diagnostic ( lang best-practice task declaration-order )",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|error| error.code == "E-DIAGNOSTIC-CONTROL-MISMATCH")
    );
}

#[test]
fn preserves_publication_on_declarations() {
    let source = SourceText::new("pub answer is 42").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements.as_slice(),
        [Statement::Published { declaration, .. }]
            if matches!(declaration.as_ref(), Statement::Binding { .. })
    ));
}

#[test]
fn rejects_publication_on_expressions() {
    let source = SourceText::new("pub 42").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|error| error.code == "E-PUBLICATION-TARGET")
    );
}

#[test]
fn parses_brand_neutral_language_selection() {
    let source = SourceText::new("use language (\n  version is v0.1\n)\n42").unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements.first(),
        Some(Statement::LanguageSelection { version, .. })
            if source.slice(*version) == "v0.1"
    ));
    let source = SourceText::new(
        "use language ( version is v0.1 )\nuse library std ( version is v0.1 )\nstd min (2, 1)",
    )
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements.get(1),
        Some(Statement::LibrarySelection { name, version, .. })
            if source.slice(*name) == "std" && source.slice(*version) == "v0.1"
    ));
}

#[test]
fn preserves_language_variant_features() {
    let source = SourceText::new(
        "use language (\n  version is v0.1,\n  features is ( debug, testing )\n)\n42",
    )
    .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements.first(),
        Some(Statement::LanguageSelection { features, .. })
            if features.iter().map(|span| source.slice(*span)).collect::<Vec<_>>()
                == ["debug", "testing"]
    ));
}

#[test]
fn parses_function_interface_shapes_without_bodies() {
    let source =
        SourceText::new("Parser is Interface\n  parse is fn (source : String) -> Boolean\nParser")
            .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements.first(),
        Some(Statement::Interface { functions, .. }) if functions.len() == 1
    ));
}

#[test]
fn retains_v02_clauses_on_interface_operations() {
    let source = SourceText::new(
            "Parser is Interface\n  parse is fn (source : String) requires true effects (Effects ()) -> result : Boolean ensures result\nParser",
        )
        .unwrap();
    let parsed = parse(&source, &lex(&source));
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Some(Statement::Interface { functions, .. }) = parsed.statements.first() else {
        panic!("expected interface");
    };
    assert_eq!(functions.len(), 1);
    assert!(functions[0].clauses.requires.is_some());
    assert!(functions[0].clauses.effects.is_some());
    assert_eq!(
        source.slice(functions[0].clauses.result_binding.expect("result binding")),
        "result"
    );
    assert!(functions[0].clauses.ensures.is_some());
}
