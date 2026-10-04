fn infer_unfold_yield_classifier(seed: &Value, step: &Value) -> String {
    let Value::List {
        element_classifier, ..
    } = seed
    else {
        return "Value".into();
    };
    let Value::AnonymousFunction(function) = step else {
        return "Value".into();
    };
    let [CapturedPattern::Binding(parameter)] = function.parameters.as_slice() else {
        return "Value".into();
    };
    let Expression::Application { items, .. } = function.body.as_ref() else {
        return "Value".into();
    };
    let [
        Expression::Identifier(operation),
        Expression::Identifier(argument),
    ] = items.as_slice()
    else {
        return "Value".into();
    };
    if function.source.slice(*operation) == "uncons"
        && function.source.slice(*argument) == parameter
    {
        element_classifier.clone()
    } else {
        "Value".into()
    }
}

fn bind_anonymous_pattern(
    source: &SourceText,
    invocation: &mut Session,
    matched_bindings: &mut BTreeMap<String, Value>,
    pattern: &CapturedPattern,
    argument: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    match pattern {
        CapturedPattern::Binding(name) => {
            bind_anonymous_name(
                source,
                invocation,
                matched_bindings,
                name,
                argument,
                span,
                trace,
            )?;
        }
        CapturedPattern::Product(fields) => {
            let Value::Tuple(values) = argument else {
                return Err(diagnostic(
                    source,
                    "E-ANONYMOUS-PRODUCT-PATTERN",
                    span,
                    "anonymous product pattern requires a positional product",
                ));
            };
            if values.len() != fields.len() {
                return Err(diagnostic(
                    source,
                    "E-ANONYMOUS-PRODUCT-PATTERN",
                    span,
                    format!(
                        "anonymous product pattern expects {} fields, found {}",
                        fields.len(),
                        values.len()
                    ),
                ));
            }
            for (field, value) in fields.iter().zip(values) {
                bind_anonymous_pattern(
                    source,
                    invocation,
                    matched_bindings,
                    field,
                    value,
                    span,
                    trace,
                )?;
            }
        }
    }
    Ok(())
}

fn capture_anonymous_pattern(source: &SourceText, pattern: &AnonymousPattern) -> CapturedPattern {
    match pattern {
        AnonymousPattern::Binding(span) => CapturedPattern::Binding(source.slice(*span).to_owned()),
        AnonymousPattern::Product { fields, .. } => CapturedPattern::Product(
            fields
                .iter()
                .map(|field| capture_anonymous_pattern(source, field))
                .collect(),
        ),
    }
}

fn bind_anonymous_name(
    source: &SourceText,
    invocation: &mut Session,
    matched_bindings: &mut BTreeMap<String, Value>,
    name: &str,
    argument: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    if name == "_" {
        return Ok(());
    }
    if let Some(first) = matched_bindings.get(name) {
        if first != &argument {
            return Err(diagnostic(
                source,
                "E-ANONYMOUS-PATTERN-IDENTITY",
                span,
                format!("repeated pattern name `{name}` requires the same exact value"),
            ));
        }
        trace.record(TraceEvent {
            event: "pattern.identity.matched",
            rule: "TOPAL-TYPE-MATCH-001",
            detail: name,
        });
        return Ok(());
    }
    matched_bindings.insert(name.to_owned(), argument.clone());
    invocation.bindings.insert(name.to_owned(), argument);
    Ok(())
}

fn known_enum_alternatives(session: &Session, type_name: &str) -> Option<BTreeSet<String>> {
    if type_name == "Comparison" {
        return Some(
            ["Less", "Equal", "Greater"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        );
    }
    session.enum_types.get(type_name).cloned()
}
