fn reject_v02_language_object_shadowing(
    source: &SourceText,
    session: &Session,
    name: Span,
) -> Result<(), Diagnostic> {
    let name_text = source.slice(name);
    if session.language_version == LanguageVersion::DESIGN_1
        && session.call_stack.is_empty()
        && is_v02_language_owned_object(name_text)
    {
        return Err(diagnostic(
            source,
            "E-LANGUAGE-NAME-CONFLICT",
            name,
            format!(
                "`{name_text}` is supplied by the v0.2 language context and cannot be declared in the root scope"
            ),
        ));
    }
    Ok(())
}

fn is_v02_language_owned_object(name: &str) -> bool {
    matches!(
        name,
        "AtomicCommit"
            | "Compensates"
            | "Consumes"
            | "DeadlineMet"
            | "Durable"
            | "Exclusive"
            | "ImmediateHandler"
            | "MultiShot"
            | "NoAlloc"
            | "OAlloc"
            | "OExec"
            | "Progress"
            | "ReleaseJitter"
            | "ResourceBound"
            | "ResponseWithin"
            | "RetrySafe"
            | "Specialized"
            | "WorstCaseExecution"
    )
}

fn interface_clause_shape(source: &SourceText, clauses: &FunctionClauses) -> InterfaceClauseShape {
    InterfaceClauseShape {
        requires: clauses
            .requires
            .as_deref()
            .map(|expression| source.slice(expression.span()).to_owned()),
        effects: clauses
            .effects
            .as_deref()
            .map(|expression| source.slice(expression.span()).to_owned()),
        guarantees: clauses
            .guarantees
            .as_deref()
            .map(|expression| source.slice(expression.span()).to_owned()),
        result_binding: clauses
            .result_binding
            .map(|binding| source.slice(binding).to_owned()),
        ensures: clauses
            .ensures
            .as_deref()
            .map(|expression| source.slice(expression.span()).to_owned()),
    }
}

fn evaluate_binding_initializer(
    source: &SourceText,
    session: &mut Session,
    initializer: &Expression,
    classifier: Option<Span>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    evaluate_expression_with_optional_context(
        source,
        session,
        initializer,
        classifier.map(|classifier| source.slice(classifier)),
        trace,
    )
}

fn evaluate_expression_with_optional_context(
    source: &SourceText,
    session: &mut Session,
    expression: &Expression,
    expected_classifier: Option<&str>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let Expression::Infinity(span) = expression {
        let negative = source.slice(*span).starts_with('-');
        let classifier = match expected_classifier {
            Some("Int") => "Int",
            Some("Nat") if !negative => "Nat",
            Some("Nat") => {
                return Err(diagnostic(
                    source,
                    "E-INFINITY-CLASSIFIER",
                    *span,
                    "-Infinity does not satisfy Nat",
                ));
            }
            Some("Rational") => "Rational",
            _ => {
                return Err(diagnostic(
                    source,
                    "E-INFINITY-CONTEXT",
                    *span,
                    "this implemented subset requires an explicit Int, Nat, or Rational infinity classifier",
                ));
            }
        };
        trace.record(TraceEvent {
            event: "numeric.infinity.constructed",
            rule: "TOPAL-NUM-INFINITY-001",
            detail: source.slice(*span),
        });
        return Ok(Value::Infinity {
            negative,
            classifier: classifier.into(),
        });
    }
    if let Some(element_classifier) = expected_classifier.and_then(list_element_classifier)
        && let Some(list) =
            evaluate_list_expression(source, session, expression, element_classifier, trace)?
    {
        return Ok(list);
    }
    if let Some(payload_classifier) = expected_classifier.and_then(optional_payload_classifier)
        && let Expression::Application { items, .. } = expression
        && let [Expression::Identifier(constructor), payload] = items.as_slice()
        && source.slice(*constructor) == "Some"
    {
        let payload_classifier = substitute_classifier(payload_classifier, &session.generic_types);
        let value = session.evaluate_expression(source, payload, trace)?;
        if value_has_classifier(&value, &payload_classifier) {
            trace.record(TraceEvent {
                event: "optional.some.constructed",
                rule: "TOPAL-TYPE-OPTIONAL-CONTEXT-001",
                detail: &payload_classifier,
            });
            return Ok(Value::Optional {
                payload_classifier,
                payload: Some(Box::new(value)),
            });
        }
    }
    let contextual_none = expected_classifier
        .and_then(optional_payload_classifier)
        .filter(
            |_| matches!(expression, Expression::Identifier(span) if source.slice(*span) == "None"),
        );
    let Some(payload_classifier) = contextual_none else {
        return session.evaluate_expression(source, expression, trace);
    };
    let payload_classifier = session
        .generic_types
        .get(payload_classifier)
        .map_or(payload_classifier, String::as_str);
    trace.record(TraceEvent {
        event: "optional.none.constructed",
        rule: "TOPAL-TYPE-OPTIONAL-CONTEXT-001",
        detail: payload_classifier,
    });
    Ok(Value::Optional {
        payload_classifier: payload_classifier.to_owned(),
        payload: None,
    })
}

fn evaluate_list_expression(
    source: &SourceText,
    session: &mut Session,
    expression: &Expression,
    element_classifier: &str,
    trace: &mut impl TraceSink,
) -> Result<Option<Value>, Diagnostic> {
    let grouped = tuple_classifiers(element_classifier);
    let element_classifier = grouped
        .as_ref()
        .filter(|items| items.len() == 1)
        .map_or(element_classifier, |items| items[0]);
    let substituted_element_classifier =
        substitute_classifier(element_classifier, &session.generic_types);
    let element_classifier = substituted_element_classifier.as_str();
    if matches!(expression, Expression::Identifier(span) if source.slice(*span) == "Empty") {
        trace.record(TraceEvent {
            event: "list.empty.constructed",
            rule: "TOPAL-TYPE-LIST-CONSTRUCT-001",
            detail: element_classifier,
        });
        return Ok(Some(Value::List {
            element_classifier: element_classifier.to_owned(),
            entries: Vec::new(),
        }));
    }
    let Expression::Application { items, span } = expression else {
        return Ok(None);
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Ok(None);
    };
    if source.slice(*constructor) != "Entry" {
        return Ok(None);
    }
    if fields.len() != 2 || fields.iter().any(|field| field.label.is_some()) {
        return Err(diagnostic(
            source,
            "E-LIST-ENTRY-SHAPE",
            *span,
            "Entry requires exactly `(value, remaining-list)`",
        )
        .with_help("write `Entry ( value, remaining-list )`"));
    }
    let entry = session.evaluate_expression(source, &fields[0].value, trace)?;
    if !value_has_classifier(&entry, element_classifier) {
        let found = structural_value_classifier(&entry);
        return Err(diagnostic(
            source,
            "E-LIST-ENTRY-CLASSIFIER",
            fields[0].value.span(),
            format!(
                "list entry has classifier `{found}`, but this list requires `{element_classifier}`"
            ),
        )
        .with_help(format!("use a `{element_classifier}` value for this entry")));
    }
    let Some(Value::List { mut entries, .. }) =
        evaluate_list_expression(source, session, &fields[1].value, element_classifier, trace)?
    else {
        return Err(diagnostic(
            source,
            "E-LIST-REMAINDER",
            fields[1].value.span(),
            "Entry requires another List as its remaining value",
        )
        .with_help("end the constructor chain with `Empty`"));
    };
    entries.insert(0, entry);
    trace.record(TraceEvent {
        event: "list.entry.constructed",
        rule: "TOPAL-TYPE-LIST-CONSTRUCT-001",
        detail: element_classifier,
    });
    Ok(Some(Value::List {
        element_classifier: element_classifier.to_owned(),
        entries,
    }))
}

pub(super) fn direct_expression_returns_from_function(expression: &Expression) -> bool {
    let Expression::Block { statements, .. } = expression else {
        return false;
    };
    statements.iter().any(|statement| match statement {
        Statement::Return { .. } => true,
        Statement::Binding { value, .. }
        | Statement::Discard { value, .. }
        | Statement::Expression(value) => direct_expression_returns_from_function(value),
        _ => false,
    })
}

pub(super) fn is_supported_returning_boolean_action_shape(rules: &[DecisionRule]) -> bool {
    let complete = match rules {
        [rule] => matches!(rule.matcher, DecisionMatcher::Otherwise(_)),
        [first, second] => match (&first.matcher, &second.matcher) {
            (
                DecisionMatcher::Boolean {
                    value: first_value, ..
                },
                DecisionMatcher::Boolean {
                    value: second_value,
                    ..
                },
            ) => first_value != second_value,
            (DecisionMatcher::Boolean { .. }, DecisionMatcher::Otherwise(_)) => true,
            _ => false,
        },
        _ => false,
    };
    complete
        && rules
            .iter()
            .all(|rule| direct_expression_returns_from_function(&rule.action))
}

pub(super) fn is_supported_returning_enum_fallback_action_shape(
    source: &SourceText,
    rules: &[DecisionRule],
) -> bool {
    let Some((fallback, enum_rules)) = rules.split_last() else {
        return false;
    };
    let mut alternatives = BTreeSet::new();
    !enum_rules.is_empty()
        && matches!(fallback.matcher, DecisionMatcher::Otherwise(_))
        && enum_rules.iter().all(|rule| {
            matches!(rule.matcher, DecisionMatcher::Identifier(matcher) if alternatives.insert(source.slice(matcher)))
        })
        && rules
            .iter()
            .all(|rule| direct_expression_returns_from_function(&rule.action))
}

pub(super) fn is_supported_returning_exhaustive_enum_action_shape(
    source: &SourceText,
    rules: &[DecisionRule],
) -> bool {
    let mut alternatives = BTreeSet::new();
    !rules.is_empty()
        && rules.iter().all(|rule| {
            matches!(rule.matcher, DecisionMatcher::Identifier(matcher) if alternatives.insert(source.slice(matcher)))
                && direct_expression_returns_from_function(&rule.action)
        })
}

pub(super) fn is_supported_returning_optional_action_shape(rules: &[DecisionRule]) -> bool {
    let mut some = false;
    let mut none = false;
    let mut fallback = false;
    for (index, rule) in rules.iter().enumerate() {
        match rule.matcher {
            DecisionMatcher::Optional {
                some: true,
                binding: Some(_),
                ..
            } if !some && !fallback => some = true,
            DecisionMatcher::Optional {
                some: false,
                binding: None,
                ..
            } if !none && !fallback => none = true,
            DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => fallback = true,
            _ => return false,
        }
        if !direct_expression_returns_from_function(&rule.action) {
            return false;
        }
    }
    fallback || (some && none)
}

pub(super) fn is_supported_returning_result_action_shape(rules: &[DecisionRule]) -> bool {
    let [first, second] = rules else {
        return false;
    };
    matches!(
        (&first.matcher, &second.matcher),
        (
            DecisionMatcher::Result {
                error: first_error,
                ..
            },
            DecisionMatcher::Result {
                error: second_error,
                ..
            }
        ) if first_error != second_error
    ) && rules
        .iter()
        .all(|rule| direct_expression_returns_from_function(&rule.action))
}

pub(super) fn is_supported_returning_error_code_action_shape(rules: &[DecisionRule]) -> bool {
    let mut ok = 0;
    let mut error_fallback = 0;
    let mut error_codes = 0;
    for rule in rules {
        match rule.matcher {
            DecisionMatcher::Result { error: false, .. } => ok += 1,
            DecisionMatcher::Result { error: true, .. } => error_fallback += 1,
            DecisionMatcher::ErrorCode { .. } => error_codes += 1,
            _ => return false,
        }
        if !direct_expression_returns_from_function(&rule.action) {
            return false;
        }
    }
    ok == 1 && error_fallback <= 1 && error_codes > 0
}

pub(super) fn is_supported_returning_list_action_shape(
    source: &SourceText,
    rules: &[DecisionRule],
) -> bool {
    let [first, second] = rules else {
        return false;
    };
    let complete = matches!(
        (&first.matcher, &second.matcher),
        (
            DecisionMatcher::ListEmpty(_),
            DecisionMatcher::ListEntry { .. }
        ) | (
            DecisionMatcher::ListEntry { .. },
            DecisionMatcher::ListEmpty(_)
        )
    );
    complete
        && rules.iter().all(|rule| {
            let distinct_bindings = match rule.matcher {
                DecisionMatcher::ListEntry { first, rest, .. } => {
                    source.slice(first) != source.slice(rest)
                }
                _ => true,
            };
            distinct_bindings && direct_expression_returns_from_function(&rule.action)
        })
}

pub(super) fn is_supported_returning_comparison_value_action_shape(
    source: &SourceText,
    rules: &[DecisionRule],
) -> bool {
    let mut alternatives = BTreeSet::new();
    let mut fallback = false;
    for (index, rule) in rules.iter().enumerate() {
        match rule.matcher {
            DecisionMatcher::Identifier(matcher)
                if !fallback
                    && matches!(source.slice(matcher), "Less" | "Equal" | "Greater")
                    && alternatives.insert(source.slice(matcher)) => {}
            DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => fallback = true,
            _ => return false,
        }
        if !direct_expression_returns_from_function(&rule.action) {
            return false;
        }
    }
    fallback || alternatives.len() == 3
}

pub(super) fn is_supported_returning_ordered_comparison_action_shape(
    rules: &[DecisionRule],
) -> bool {
    let Some((fallback, comparison_rules)) = rules.split_last() else {
        return false;
    };
    !comparison_rules.is_empty()
        && matches!(fallback.matcher, DecisionMatcher::Otherwise(_))
        && comparison_rules
            .iter()
            .all(|rule| matches!(rule.matcher, DecisionMatcher::Comparison { .. }))
        && rules
            .iter()
            .all(|rule| direct_expression_returns_from_function(&rule.action))
}

fn expression_is_closed(expression: &Expression) -> bool {
    match expression {
        Expression::Block { statements, .. } => {
            statements.iter().all(|statement| match statement {
                Statement::Binding { value, .. }
                | Statement::Discard { value, .. }
                | Statement::Return { value, .. }
                | Statement::Expression(value) => expression_is_closed(value),
                _ => false,
            })
        }
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Infinity(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Callable { .. } => true,
        Expression::Product { fields, .. } => fields
            .iter()
            .all(|field| expression_is_closed(&field.value)),
        Expression::Application { items, .. } => items.iter().all(expression_is_closed),
        Expression::AnonymousFunction { body, .. } => expression_is_closed(body),
        Expression::DecisionTable { .. }
        | Expression::Identifier(_)
        | Expression::ContextIdentifier(_)
        | Expression::Discard(_) => false,
    }
}

fn narrow_rational_to_int(
    source: &SourceText,
    initializer: &Expression,
    value: Value,
    classifier: &str,
    return_classifier: Option<&str>,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if classifier != "Int" {
        return Ok(value);
    }
    let Value::Rational(value) = value else {
        return Ok(value);
    };
    if value.denom() != &BigInt::from(1) {
        if !expression_is_closed(initializer)
            && return_classifier.and_then(result_success_classifier) == Some("Int")
        {
            let position = source.position(initializer.span().start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-NUM-RATIONAL-INT-VALIDATE-001",
                detail: "root.Int(Rational);not-representable",
            });
            return Ok(Value::Error {
                domain: "root.Int(Rational)".to_owned(),
                code: "not-representable".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-RATIONAL-NOT-EXACT-INT",
            initializer.span(),
            format!(
                "exact Rational result has denominator {}, so it cannot satisfy Int",
                value.denom()
            ),
        ));
    }
    trace.record(TraceEvent {
        event: "conversion.applied",
        rule: if expression_is_closed(initializer) {
            "TOPAL-NUM-RATIONAL-INT-EXACT-001"
        } else {
            "TOPAL-NUM-RATIONAL-INT-VALIDATE-001"
        },
        detail: if expression_is_closed(initializer) {
            "Rational->Int:exact"
        } else {
            "Rational->Int:validated"
        },
    });
    Ok(Value::Int(value.numer().clone()))
}

fn construct_int(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Rational(value) = value else {
        if matches!(value, Value::Int(_)) {
            trace.record(TraceEvent {
                event: "numeric.int.constructed",
                rule: "TOPAL-NUM-INT-CONSTRUCT-001",
                detail: "Int->Int:identity",
            });
            return Ok(value);
        }
        return Err(diagnostic(
            source,
            "E-INT-CONSTRUCTOR-OPERAND",
            operand.span(),
            "Int construction requires an exact numeric operand",
        ));
    };
    if value.denom() == &BigInt::from(1) {
        trace.record(TraceEvent {
            event: "numeric.int.constructed",
            rule: "TOPAL-NUM-INT-CONSTRUCT-001",
            detail: "Rational->Int:exact",
        });
        return Ok(Value::Int(value.numer().clone()));
    }
    if expression_is_closed(operand) {
        return Err(diagnostic(
            source,
            "E-RATIONAL-NOT-EXACT-INT",
            operand.span(),
            format!(
                "exact Rational operand has denominator {}, so Int cannot represent it",
                value.denom()
            ),
        ));
    }
    let position = source.position(operand.span().start);
    trace.record(TraceEvent {
        event: "result.error.constructed",
        rule: "TOPAL-NUM-INT-CONSTRUCT-001",
        detail: "root.Int(Rational);not-representable",
    });
    Ok(Value::Error {
        domain: "root.Int(Rational)".to_owned(),
        code: "not-representable".to_owned(),
        line: position.line,
        column: position.column,
    })
}

fn construct_nat(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Int(value) = value else {
        return Err(diagnostic(
            source,
            "E-NAT-CONSTRUCTOR-OPERAND",
            operand.span(),
            "Nat construction requires an Int operand",
        ));
    };
    if value >= BigInt::from(0) {
        trace.record(TraceEvent {
            event: "numeric.nat.constructed",
            rule: "TOPAL-NUM-NAT-CONSTRUCT-001",
            detail: "Int->Nat:nonnegative",
        });
        return Ok(Value::Int(value));
    }
    if expression_is_closed(operand) {
        return Err(diagnostic(
            source,
            "E-NAT-OUT-OF-RANGE",
            operand.span(),
            "a negative Int is outside the Nat constraint",
        ));
    }
    let position = source.position(operand.span().start);
    trace.record(TraceEvent {
        event: "result.error.constructed",
        rule: "TOPAL-NUM-NAT-CONSTRUCT-001",
        detail: "root.Nat(Int);out-of-range",
    });
    Ok(Value::Error {
        domain: "root.Nat(Int)".to_owned(),
        code: "out-of-range".to_owned(),
        line: position.line,
        column: position.column,
    })
}

fn construct_rational(
    source: &SourceText,
    operand: &Expression,
    value: Value,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let Value::Int(value) = value {
        trace.record(TraceEvent {
            event: "numeric.rational.constructed",
            rule: "TOPAL-NUM-INT-RATIONAL-CONVERT-001",
            detail: "Int->Rational:explicit",
        });
        return Ok(Value::Rational(BigRational::from_integer(value)));
    }
    let Value::Tuple(values) = value else {
        return Err(diagnostic(
            source,
            "E-RATIONAL-CONSTRUCTOR-PRODUCT",
            operand.span(),
            "Rational construction requires a positional (numerator, denominator) product",
        ));
    };
    let [Value::Int(numerator), Value::Int(denominator)] = values.as_slice() else {
        return Err(diagnostic(
            source,
            "E-RATIONAL-CONSTRUCTOR-COMPONENTS",
            operand.span(),
            "Rational numerator and denominator must both be Int values",
        ));
    };
    if denominator == &BigInt::from(0) {
        let code = if numerator == &BigInt::from(0) {
            "indeterminate"
        } else {
            "division-by-zero"
        };
        if expression_is_closed(operand) {
            let (diagnostic_code, message) = if code == "indeterminate" {
                (
                    "E-INDETERMINATE-RATIONAL",
                    "Rational (0, 0) does not determine one numeric value",
                )
            } else {
                (
                    "E-DIVISION-BY-ZERO",
                    "a finite Rational constructor requires a nonzero denominator",
                )
            };
            return Err(diagnostic(source, diagnostic_code, operand.span(), message));
        }
        let position = source.position(operand.span().start);
        trace.record(TraceEvent {
            event: "result.error.constructed",
            rule: "TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001",
            detail: if code == "indeterminate" {
                "root.Rational(Int,Int);indeterminate"
            } else {
                "root.Rational(Int,Int);division-by-zero"
            },
        });
        return Ok(Value::Error {
            domain: "root.Rational(Int,Int)".to_owned(),
            code: code.to_owned(),
            line: position.line,
            column: position.column,
        });
    }
    let value = BigRational::new(numerator.clone(), denominator.clone());
    trace.record(TraceEvent {
        event: "numeric.rational.constructed",
        rule: if expression_is_closed(operand) {
            "TOPAL-NUM-RATIONAL-CONSTRUCT-001"
        } else {
            "TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001"
        },
        detail: if expression_is_closed(operand) {
            "canonical"
        } else {
            "canonical:validated"
        },
    });
    Ok(Value::Rational(value))
}

const fn cover(first: Span, second: Span) -> Span {
    Span {
        start: first.start,
        end: second.end,
    }
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::Binding { name, value, .. } => cover(*name, value.span()),
        Statement::LanguageSelection { span, .. }
        | Statement::LibrarySelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. } => *span,
        Statement::StateField { name, classifier } => cover(*name, *classifier),
        Statement::Discard { span, value } => cover(*span, value.span()),
        Statement::Return { keyword, value } => cover(*keyword, value.span()),
        Statement::Expression(expression) => expression.span(),
    }
}

fn declaration_name<'a>(source: &'a SourceText, statement: &Statement) -> Option<&'a str> {
    declaration_name_span(statement).map(|name| source.slice(name))
}

fn declaration_name_span(statement: &Statement) -> Option<Span> {
    let name = match statement {
        Statement::Binding { name, .. }
        | Statement::Implementation { name, .. }
        | Statement::Function { name, .. }
        | Statement::Generator { name, .. }
        | Statement::Union { name, .. }
        | Statement::Interface { name, .. } => *name,
        Statement::Published { declaration, .. } => return declaration_name_span(declaration),
        _ => return None,
    };
    Some(name)
}

fn supported_generator_body(source: &SourceText, body: &[Statement]) -> bool {
    if !matches!(
        body.last(),
        Some(Statement::Expression(_) | Statement::Return { .. })
    ) {
        return false;
    }
    for statement in &body[..body.len().saturating_sub(1)] {
        if yielded_statement(source, statement).is_none()
            && !matches!(
                statement,
                Statement::Published { .. }
                    | Statement::DiagnosticControl { .. }
                    | Statement::Binding { .. }
                    | Statement::ContextAssignment { .. }
                    | Statement::Discard { .. }
                    | Statement::Function { .. }
                    | Statement::Return { .. }
            )
        {
            return false;
        }
    }
    true
}

fn discarded_yield_expression<'a>(
    source: &SourceText,
    statement: &'a Statement,
) -> Option<&'a Expression> {
    let Statement::Discard {
        value: Expression::Application { items, .. },
        ..
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(keyword), yielded] = items.as_slice() else {
        return None;
    };
    (source.slice(*keyword) == "yield").then_some(yielded)
}

fn yielded_statement<'a>(
    source: &SourceText,
    statement: &'a Statement,
) -> Option<(Option<Span>, &'a Expression)> {
    if let Some(expression) = discarded_yield_expression(source, statement) {
        return Some((None, expression));
    }
    if let Statement::Expression(Expression::Application { items, .. }) = statement
        && let [Expression::Identifier(keyword), yielded] = items.as_slice()
        && source.slice(*keyword) == "yield"
    {
        return Some((None, yielded));
    }
    let Statement::Binding {
        name,
        value: Expression::Application { items, .. },
        ..
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(keyword), yielded] = items.as_slice() else {
        return None;
    };
    (source.slice(*keyword) == "yield").then_some((Some(*name), yielded))
}

#[allow(clippy::too_many_arguments)]
fn advance_custom_generator(
    source: &SourceText,
    body: &[Statement],
    cursor: &mut usize,
    scope: &mut Session,
    pending_yield: &mut Option<Box<Value>>,
    resume_binding: &mut Option<String>,
    returned: &mut Option<Value>,
    yield_classifier: &str,
    return_classifier: &str,
    name: &str,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    while *cursor < body.len() {
        let statement = &body[*cursor];
        *cursor += 1;
        if let Some((binding, expression)) = yielded_statement(source, statement) {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, yield_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-YIELD-TYPE",
                    expression.span(),
                    name,
                    "yielded",
                    yield_classifier,
                    &value,
                ));
            }
            *pending_yield = Some(Box::new(value));
            *resume_binding = binding.map(|span| source.slice(span).to_owned());
            trace.record(TraceEvent {
                event: "generator.suspended",
                rule: "TOPAL-GENERATOR-SUSPEND-001",
                detail: name,
            });
            return Ok(());
        }
        if let Statement::Expression(expression) = statement {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, return_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-RETURN-TYPE",
                    expression.span(),
                    name,
                    "returned",
                    return_classifier,
                    &value,
                ));
            }
            *returned = Some(value);
            return Ok(());
        }
        if let Statement::Return {
            value: expression, ..
        } = statement
        {
            let value = scope.evaluate_expression(source, expression, trace)?;
            if !value_has_classifier(&value, return_classifier) {
                return Err(generator_classifier_diagnostic(
                    source,
                    "E-GENERATOR-RETURN-TYPE",
                    expression.span(),
                    name,
                    "returned",
                    return_classifier,
                    &value,
                ));
            }
            trace.record(TraceEvent {
                event: "generator.return.explicit",
                rule: "TOPAL-GENERATOR-EXPLICIT-RETURN-001",
                detail: return_classifier,
            });
            *returned = Some(value);
            *cursor = body.len();
            return Ok(());
        }
        let mut execution = Execution {
            source: source.clone(),
            statements: vec![statement.clone()],
            cursor: 0,
            result_classifier: None,
            return_classifier: None,
        };
        match execution.step(scope, trace)? {
            ExecutionStep::Advanced { .. } | ExecutionStep::Complete(_) => {}
            ExecutionStep::Returned { .. } => unreachable!("generator bindings cannot return"),
        }
    }
    Ok(())
}

fn generator_return_rule(
    origin: &str,
    empty: bool,
    returned: &str,
    traversal_rule: &'static str,
) -> &'static str {
    if origin != "root.characters" && returned != "Unit" {
        "TOPAL-GENERATOR-FINAL-RETURN-001"
    } else if origin != "root.characters" && empty {
        "TOPAL-GENERATOR-EARLY-RETURN-001"
    } else {
        traversal_rule
    }
}

fn foreach_source_diagnostic(source: &SourceText, span: Span) -> Diagnostic {
    diagnostic(
        source,
        "E-FOREACH-SOURCE",
        span,
        "the implemented foreach subset requires `characters text`",
    )
}

fn consumed_generator_diagnostic(source: &SourceText, span: Span, name: &str) -> Diagnostic {
    diagnostic(
        source,
        "E-GENERATOR-CONSUMED",
        span,
        format!("generator `{name}` was already consumed"),
    )
    .with_help("construct a fresh generator before traversing it again")
}

fn consume_generator_argument(source: &SourceText, session: &mut Session, expression: &Expression) {
    let Expression::Application { items, .. } = expression else {
        return;
    };
    let [
        Expression::Identifier(function),
        Expression::Identifier(argument),
    ] = items.as_slice()
    else {
        return;
    };
    let function_name = source.slice(*function);
    let argument_name = source.slice(*argument);
    let accepts_generator = session
        .functions
        .get(function_name)
        .is_some_and(|candidates| {
            candidates.iter().any(|candidate| {
                matches!(
                    candidate.parameters.as_slice(),
                    [(_, classifier)] if classifier.starts_with("Generator ")
                )
            })
        });
    if accepts_generator
        && matches!(
            session.bindings.get(argument_name),
            Some(
                Value::CharacterGenerator { .. }
                    | Value::CharacterReturningGenerator { .. }
                    | Value::SuspendedGenerator { .. }
            )
        )
    {
        session.bindings.remove(argument_name);
        session.declared_names.remove(argument_name);
        session.consumed_names.insert(argument_name.to_owned());
    }
}

#[allow(clippy::too_many_lines)] // Close delivery, handler execution, and trace order stay auditable together.
fn close_remaining_character_generators(
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    let generators = session
        .bindings
        .iter()
        .filter(|(_, value)| {
            matches!(
                value,
                Value::CharacterGenerator { .. }
                    | Value::CharacterReturningGenerator { .. }
                    | Value::SuspendedGenerator { .. }
            )
        })
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    for name in generators {
        let value = session
            .bindings
            .remove(&name)
            .expect("collected binding exists");
        session.declared_names.remove(&name);
        session.consumed_names.insert(name.clone());
        let origin = match &value {
            Value::CharacterGenerator { origin, .. }
            | Value::CharacterReturningGenerator { origin, .. }
            | Value::SuspendedGenerator { origin, .. } => origin.clone(),
            _ => unreachable!("only generators were collected"),
        };
        let detail = format!("domain=root;code=generator-closed;generator={origin}");
        trace.record(TraceEvent {
            event: "generator.close.signaled",
            rule: "TOPAL-GENERATOR-ERROR-CODE-001",
            detail: &detail,
        });
        if let Value::SuspendedGenerator {
            source,
            body,
            mut cursor,
            bindings,
            scope_state,
            pending_yield: _,
            resume_binding,
            returned,
            return_classifier,
            yield_classifier,
            ..
        } = value
            && let Some(resume_binding) = resume_binding
        {
            let mut pending_yield = None;
            let yield_span = body
                .get(cursor.saturating_sub(1))
                .map_or(Span::new(0, 0), statement_span);
            let position = source.position(yield_span.start);
            let mut scope = session.clone();
            scope.bindings = *bindings;
            *scope.functions = scope_state.functions;
            scope.declared_names = scope_state.declared_names;
            scope.local_function_names = scope_state.local_function_names;
            scope.enum_types = scope_state.enum_types;
            scope.bindings.insert(
                resume_binding.clone(),
                Value::Error {
                    domain: "root".into(),
                    code: "generator-closed".into(),
                    line: position.line,
                    column: position.column,
                },
            );
            scope.declared_names.insert(resume_binding.clone());
            trace.record(TraceEvent {
                event: "generator.close.bound",
                rule: "TOPAL-GENERATOR-CLOSE-HANDLER-001",
                detail: &resume_binding,
            });
            let mut handled_return = returned.map(|value| *value);
            let mut next_resume_binding = None;
            advance_custom_generator(
                &source,
                &body,
                &mut cursor,
                &mut scope,
                &mut pending_yield,
                &mut next_resume_binding,
                &mut handled_return,
                &yield_classifier,
                &return_classifier,
                origin.rsplit('.').next().unwrap_or(&origin),
                trace,
            )?;
            if pending_yield.is_some() {
                return Err(diagnostic(
                    &source,
                    "E-GENERATOR-YIELD-AFTER-CLOSE",
                    body.get(cursor.saturating_sub(1))
                        .map_or(yield_span, statement_span),
                    "a generator cannot yield again after observing `generator-closed`",
                ));
            }
        }
        trace.record(TraceEvent {
            event: "generator.closed",
            rule: if origin == "root.characters" {
                "TOPAL-STRING-CHARACTERS-CLOSE-001"
            } else {
                "TOPAL-GENERATOR-CLOSE-001"
            },
            detail: &origin,
        });
    }
    Ok(())
}

fn enum_alternatives(source: &SourceText, expression: &Expression) -> Option<Vec<(String, Span)>> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Enum" {
        return None;
    }
    fields
        .iter()
        .map(|field| {
            let Expression::Identifier(alternative) = &field.value else {
                return None;
            };
            field
                .label
                .is_none()
                .then(|| (source.slice(*alternative).to_owned(), *alternative))
        })
        .collect()
}

fn variant_alternatives(source: &SourceText, expression: &Expression) -> Option<Vec<String>> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Variant" {
        return None;
    }
    fields
        .iter()
        .map(|field| {
            field
                .label
                .is_none()
                .then(|| classifier_expression(source, &field.value))
                .flatten()
        })
        .collect()
}

fn evaluate_arithmetic_error_code(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Option<Value> {
    let [
        Expression::Identifier(lang),
        Expression::Identifier(arithmetic),
        Expression::Identifier(code),
    ] = items
    else {
        return None;
    };
    if source.slice(*lang) != "lang" || source.slice(*arithmetic) != "arithmetic" {
        return None;
    }
    let code = source.slice(*code);
    if !matches!(
        code,
        "out-of-range" | "not-representable" | "division-by-zero" | "indeterminate"
    ) {
        return None;
    }
    trace.record(TraceEvent {
        event: "namespace.member.selected",
        rule: "TOPAL-NUM-ARITHMETIC-ERROR-001",
        detail: code,
    });
    Some(Value::Enum {
        type_name: "lang arithmetic ArithmeticErrorCode".to_owned(),
        alternative: code.to_owned(),
    })
}

fn evaluate_generator_error_code(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Option<Value> {
    let [
        Expression::Identifier(lang),
        Expression::Identifier(generator),
        Expression::Identifier(code),
    ] = items
    else {
        return None;
    };
    if source.slice(*lang) != "lang"
        || source.slice(*generator) != "generator"
        || source.slice(*code) != "generator-closed"
    {
        return None;
    }
    trace.record(TraceEvent {
        event: "namespace.member.selected",
        rule: "TOPAL-GENERATOR-ERROR-CODE-001",
        detail: "generator-closed",
    });
    Some(Value::Enum {
        type_name: "lang generator GeneratorErrorCode".to_owned(),
        alternative: "generator-closed".to_owned(),
    })
}

fn declare_enum(
    source: &SourceText,
    name: Span,
    expression: &Expression,
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Result<Option<(Value, Span)>, Diagnostic> {
    let Some(alternatives) = enum_alternatives(source, expression) else {
        return Ok(None);
    };
    let name_text = source.slice(name);
    let mut seen = BTreeSet::new();
    for (alternative, span) in &alternatives {
        if !seen.insert(alternative.as_str())
            || session.declared_names.contains(alternative)
            || alternative == name_text
        {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-ENUM-ALTERNATIVE",
                *span,
                format!("enum alternative `{alternative}` is already declared in this scope"),
            ));
        }
    }
    session.declared_names.insert(name_text.to_owned());
    session.enum_types.insert(
        name_text.to_owned(),
        alternatives
            .iter()
            .map(|(alternative, _)| alternative.clone())
            .collect(),
    );
    for (alternative, _) in alternatives {
        session.bindings.insert(
            alternative.clone(),
            Value::Enum {
                type_name: name_text.to_owned(),
                alternative: alternative.clone(),
            },
        );
        session.declared_names.insert(alternative);
    }
    trace.record(TraceEvent {
        event: "enum.declared",
        rule: "TOPAL-TYPE-ENUM-001",
        detail: name_text,
    });
    Ok(Some((Value::Unit, cover(name, expression.span()))))
}

fn declare_union(
    source: &SourceText,
    session: &mut Session,
    name: Span,
    alternatives: &[topal_syntax::UnionAlternative],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<(Value, Span), Diagnostic> {
    let type_name = source.slice(name);
    if session.declared_names.contains(type_name) {
        return Err(diagnostic(
            source,
            "E-DUPLICATE-UNION",
            name,
            "name is already declared",
        ));
    }
    let mut declared = BTreeMap::new();
    for alternative in alternatives {
        let alternative_name = source.slice(alternative.name);
        if declared.contains_key(alternative_name) {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-UNION-ALTERNATIVE",
                alternative.name,
                "Union alternative occurs more than once",
            ));
        }
        let classifier = alternative
            .classifier
            .map(|classifier| source.slice(classifier).to_owned());
        declared.insert(alternative_name.to_owned(), classifier.clone());
        session.declared_names.insert(alternative_name.to_owned());
    }
    session.union_types.insert(type_name.to_owned(), declared);
    session.declared_names.insert(type_name.to_owned());
    let supports_equality = session.classifier_supports_equality(type_name);
    for alternative in alternatives
        .iter()
        .filter(|alternative| alternative.classifier.is_none())
    {
        let alternative_name = source.slice(alternative.name);
        session.bindings.insert(
            alternative_name.to_owned(),
            Value::Union(Box::new(UnionValue {
                type_name: type_name.to_owned(),
                alternative: alternative_name.to_owned(),
                payload_classifier: None,
                payload: None,
                supports_equality,
            })),
        );
    }
    trace.record(TraceEvent {
        event: "union.declared",
        rule: "TOPAL-TYPE-UNION-001",
        detail: type_name,
    });
    Ok((Value::Unit, span))
}

fn declare_variant(
    source: &SourceText,
    name: Span,
    expression: &Expression,
    session: &mut Session,
    trace: &mut impl TraceSink,
) -> Option<(Value, Span)> {
    let alternatives = variant_alternatives(source, expression)?;
    let type_name = source.slice(name);
    let declared = alternatives
        .into_iter()
        .enumerate()
        .map(|(index, classifier)| (format!("at {index}"), Some(classifier)))
        .collect();
    session.union_types.insert(type_name.to_owned(), declared);
    session.declared_names.insert(type_name.to_owned());
    trace.record(TraceEvent {
        event: "variant.declared",
        rule: "TOPAL-TYPE-VARIANT-001",
        detail: type_name,
    });
    Some((Value::Unit, expression.span()))
}

#[allow(clippy::unnested_or_patterns)] // Keep each exact value/classifier association explicit.
fn value_has_classifier(value: &Value, classifier: &str) -> bool {
    if let Some(matches) = infinity_has_classifier(value, classifier) {
        return matches;
    }
    if classifier == "MessageContext"
        && matches!(value, Value::Record(fields)
            if fields.iter().any(|(name, _)| name == "session-id")
                && fields.iter().any(|(name, _)| name == "sender"))
    {
        return true;
    }
    if let Value::LayoutBacked { layout, value } = value {
        return classifier == layout.semantic || value_has_classifier(value, classifier);
    }
    if let Value::Refined {
        constraint,
        base_classifier,
        value,
    } = value
    {
        return classifier == constraint
            || (classifier == base_classifier && value_has_classifier(value, base_classifier));
    }
    if let Value::SuspendedGenerator {
        yield_classifier,
        return_classifier,
        ..
    } = value
    {
        return classifier == format!("Generator {yield_classifier} Unit {return_classifier}");
    }
    if let Value::IterateGenerator {
        classifier: yielded,
        ..
    } = value
    {
        return classifier == format!("Generator {yielded} Unit Unit");
    }
    if let Value::UnfoldGenerator {
        yield_classifier, ..
    } = value
    {
        return classifier == format!("Generator {yield_classifier} Unit Unit");
    }
    if let Some(matches) = represented_value_has_classifier(value, classifier) {
        return matches;
    }
    if let Some(success) = result_success_classifier(classifier) {
        return matches!(value, Value::Error { code, .. } if is_arithmetic_error_code(code))
            || value_has_classifier(value, success);
    }
    if let Some(matches) = record_value_has_classifier(value, classifier) {
        return matches;
    }
    if let (Value::Tuple(values), Some(classifiers)) = (value, tuple_classifiers(classifier)) {
        return values.len() == classifiers.len()
            && values
                .iter()
                .zip(classifiers)
                .all(|(value, classifier)| value_has_classifier(value, classifier));
    }
    if let Some(requested_kind) = classifier_object_kind(classifier) {
        return value.object_kind().satisfies(requested_kind);
    }
    match (value, classifier) {
        (Value::Boolean(_), "Boolean")
        | (Value::Int(_), "Int")
        | (Value::Rational(_), "Rational")
        | (Value::IntRange { .. }, "Range Int")
        | (Value::InfiniteIntRange { .. }, "Range Int")
        | (Value::RationalRange { .. } | Value::InfiniteRationalRange { .. }, "Range Rational")
        | (Value::CharacterGenerator { .. }, "Generator Character Unit Unit")
        | (Value::CharacterReturningGenerator { .. }, "Generator Character Unit Character")
        | (Value::String(_), "String")
        | (Value::Error { .. }, "Error")
        | (Value::Continue(_) | Value::Finish(_), "TraversalControl")
        | (Value::Completed, "Completed")
        | (Value::Unit, "Unit") => true,
        (Value::String(value), "Character") => character_count(value) == 1,
        (Value::Int(value), "Nat") => value >= &BigInt::from(0),
        (Value::Enum { type_name, .. }, "ErrorCode") => {
            type_name == "lang arithmetic ArithmeticErrorCode"
        }
        (Value::Enum { type_name, .. } | Value::Modular { type_name, .. }, classifier) => {
            type_name == classifier
        }
        (Value::Union(union), classifier) => union.type_name == classifier,
        _ => false,
    }
}

fn represented_value_has_classifier(value: &Value, classifier: &str) -> Option<bool> {
    match value {
        Value::Optional {
            payload_classifier, ..
        } => optional_payload_classifier(classifier).map(|expected| payload_classifier == expected),
        Value::List {
            element_classifier,
            entries,
        } => list_element_classifier(classifier).map(|expected| {
            element_classifier == expected
                && entries
                    .iter()
                    .all(|entry| value_has_classifier(entry, expected))
        }),
        Value::Array {
            element_classifier,
            entries,
        } => array_classifier_parts(classifier).map(|(count, expected)| {
            entries.len() == count
                && element_classifier == expected
                && entries
                    .iter()
                    .all(|entry| value_has_classifier(entry, expected))
        }),
        Value::Set {
            element_classifier,
            entries,
        } => applied_classifier(classifier, "Set").map(|expected| {
            element_classifier == expected
                && entries
                    .iter()
                    .all(|entry| value_has_classifier(entry, expected))
        }),
        Value::Bag {
            element_classifier,
            entries,
        } => applied_classifier(classifier, "Bag").map(|expected| {
            element_classifier == expected
                && entries
                    .iter()
                    .all(|(entry, _)| value_has_classifier(entry, expected))
        }),
        Value::Map {
            key_classifier,
            value_classifier,
            entries,
        } => map_classifier_parts(classifier).map(|(expected_key, expected_value)| {
            key_classifier == expected_key
                && value_classifier == expected_value
                && entries.iter().all(|(key, value)| {
                    value_has_classifier(key, expected_key)
                        && value_has_classifier(value, expected_value)
                })
        }),
        _ => None,
    }
}

fn infinity_has_classifier(value: &Value, classifier: &str) -> Option<bool> {
    let Value::Infinity {
        negative,
        classifier: infinity_classifier,
    } = value
    else {
        return None;
    };
    Some(if infinity_classifier == "Rational" {
        classifier == "Rational"
    } else {
        classifier == "Int" || (!negative && classifier == "Nat")
    })
}

fn classifier_object_kind(classifier: &str) -> Option<ObjectKind> {
    match classifier {
        "Type" => Some(ObjectKind::Type),
        "Function" => Some(ObjectKind::Function),
        "Constraint" => Some(ObjectKind::Constraint),
        "Capability" => Some(ObjectKind::Capability),
        "Effect" => Some(ObjectKind::Effect),
        "Scope" => Some(ObjectKind::Scope),
        _ => None,
    }
}

fn record_value_has_classifier(value: &Value, classifier: &str) -> Option<bool> {
    let Value::Record(values) = value else {
        return None;
    };
    let fields = record_classifiers(classifier)?;
    Some(
        values.len() == fields.len()
            && fields.iter().all(|(label, classifier)| {
                values
                    .iter()
                    .find_map(|(candidate, value)| (candidate == label).then_some(value))
                    .is_some_and(|value| value_has_classifier(value, classifier))
            }),
    )
}
