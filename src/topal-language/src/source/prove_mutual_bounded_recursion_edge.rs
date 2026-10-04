pub(super) fn prove_mutual_bounded_recursion_edge(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    body: &[Statement],
) -> Option<(String, &'static str)> {
    let [(parameter, classifier)] = parameters else {
        return None;
    };
    if classifier != "Int" && classifier != "Nat" {
        return None;
    }
    let [Statement::Expression(Expression::DecisionTable { subject, rules, .. })] = body else {
        return None;
    };
    if !matches!(subject.as_ref(), Expression::Identifier(span) if source.slice(*span) == parameter)
    {
        return None;
    }
    let [base, recursive] = rules.as_slice() else {
        return None;
    };
    let (step, proof_rule) = match (&**classifier, &base.matcher) {
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(bound),
                ..
            },
        ) if parse_integer(source.slice(*bound)).is_some_and(|value| value >= BigInt::from(0)) => (
            CallableKind::Minus,
            "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001",
        ),
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Minus,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001",
        ),
        _ => return None,
    };
    if !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_)) {
        return None;
    }
    let (target, valid) =
        mutual_call_target(source, function_name, parameter, step, &recursive.action);
    let target = target?;
    if !valid
        || contains_self_call(source, &target, &base.action)
        || (classifier == "Nat"
            && step == CallableKind::Minus
            && !nat_decrement_step_limit(source, &base.matcher).is_some_and(|limit| {
                recursive_calls_fit_nat_bound(source, &target, parameter, &recursive.action, &limit)
            }))
    {
        return None;
    }
    Some((target, proof_rule))
}

fn mutual_call_target(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> (Option<String>, bool) {
    match expression {
        Expression::Application { items, .. }
            if matches!(items.as_slice(), [Expression::Identifier(_), _]) =>
        {
            let [Expression::Identifier(target), argument] = items.as_slice() else {
                unreachable!("guard established a unary named application");
            };
            let target = source.slice(*target);
            (
                Some(target.to_owned()),
                target != function_name
                    && is_positive_literal_step(source, parameter, step, argument),
            )
        }
        Expression::Application { items, .. } => combine_mutual_call_targets(
            items
                .iter()
                .map(|item| mutual_call_target(source, function_name, parameter, step, item)),
        ),
        Expression::Product { fields, .. } => {
            combine_mutual_call_targets(fields.iter().map(|field| {
                mutual_call_target(source, function_name, parameter, step, &field.value)
            }))
        }
        Expression::DecisionTable { subject, rules, .. } => combine_mutual_call_targets(
            std::iter::once(mutual_call_target(
                source,
                function_name,
                parameter,
                step,
                subject,
            ))
            .chain(rules.iter().map(|rule| {
                mutual_call_target(source, function_name, parameter, step, &rule.action)
            })),
        ),
        _ => (None, true),
    }
}

fn combine_mutual_call_targets(
    checks: impl Iterator<Item = (Option<String>, bool)>,
) -> (Option<String>, bool) {
    checks.fold(
        (None, true),
        |(target, valid), (next_target, next_valid)| match (target, next_target) {
            (Some(target), Some(next)) => {
                let same = target == next;
                (Some(target), valid && next_valid && same)
            }
            (Some(target), None) | (None, Some(target)) => (Some(target), valid && next_valid),
            (None, None) => (None, valid && next_valid),
        },
    )
}

const MUTUAL_INT_RECURSION_RULE: &str = "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001";
const MUTUAL_INCREASING_INT_RECURSION_RULE: &str =
    "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001";
const MUTUAL_NAT_RECURSION_RULE: &str = "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001";
const MUTUAL_INCREASING_NAT_RECURSION_RULE: &str =
    "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001";

fn is_mutual_recursion_rule(rule: &str) -> bool {
    matches!(
        rule,
        MUTUAL_INT_RECURSION_RULE
            | MUTUAL_INCREASING_INT_RECURSION_RULE
            | MUTUAL_NAT_RECURSION_RULE
            | MUTUAL_INCREASING_NAT_RECURSION_RULE
    )
}

fn recursion_rule_for_call(
    call_stack: &[ActiveCall],
    target: &str,
    target_signature: &str,
    function: &UserFunction,
) -> Option<&'static str> {
    let cycle_start = call_stack
        .iter()
        .position(|active| active.signature == target_signature)?;
    let cycle = &call_stack[cycle_start..];
    if function.recursion_target.is_none() {
        return function.termination_rule;
    }
    let cycle_rule = function.termination_rule?;
    if !is_mutual_recursion_rule(cycle_rule)
        || cycle
            .iter()
            .any(|active| active.termination_rule != Some(cycle_rule))
    {
        return None;
    }
    let internal_edges_match = cycle
        .windows(2)
        .all(|pair| pair[0].recursion_target.as_deref() == Some(pair[1].name.as_str()));
    let closes_cycle = cycle
        .last()
        .and_then(|active| active.recursion_target.as_deref())
        == Some(target);
    (internal_edges_match && closes_cycle).then_some(cycle_rule)
}

fn contains_self_call(source: &SourceText, function_name: &str, expression: &Expression) -> bool {
    let (found, _) = bounded_self_calls(source, function_name, "", CallableKind::Minus, expression);
    found
}

fn bounded_self_calls(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> (bool, bool) {
    match expression {
        Expression::Application { items, .. } if matches!(items.first(), Some(Expression::Identifier(span)) if source.slice(*span) == function_name) =>
        {
            let valid = matches!(items.as_slice(), [_, argument] if is_positive_literal_step(source, parameter, step, argument));
            (true, valid)
        }
        Expression::Application { items, .. } => combine_call_checks(
            items
                .iter()
                .map(|item| bounded_self_calls(source, function_name, parameter, step, item)),
        ),
        Expression::Product { fields, .. } => {
            combine_call_checks(fields.iter().map(|field| {
                bounded_self_calls(source, function_name, parameter, step, &field.value)
            }))
        }
        Expression::DecisionTable { subject, rules, .. } => combine_call_checks(
            std::iter::once(bounded_self_calls(
                source,
                function_name,
                parameter,
                step,
                subject,
            ))
            .chain(rules.iter().map(|rule| {
                bounded_self_calls(source, function_name, parameter, step, &rule.action)
            })),
        ),
        _ => (false, true),
    }
}

fn combine_call_checks(checks: impl Iterator<Item = (bool, bool)>) -> (bool, bool) {
    checks.fold((false, true), |(found, valid), (next_found, next_valid)| {
        (found || next_found, valid && next_valid)
    })
}

fn is_positive_literal_step(
    source: &SourceText,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> bool {
    matches!(
        expression,
        Expression::Application { items, .. }
            if matches!(
                items.as_slice(),
                [
                    Expression::Identifier(name),
                    Expression::Callable { kind, .. },
                    Expression::Integer(amount)
                ] if source.slice(*name) == parameter
                    && *kind == step
                    && parse_integer(source.slice(*amount)).is_some_and(|value| value > BigInt::from(0_u8))
            )
    )
}

fn literal_step_value(
    source: &SourceText,
    parameter: &str,
    step: CallableKind,
    expression: &Expression,
) -> Option<BigInt> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    let [
        Expression::Identifier(name),
        Expression::Callable { kind, .. },
        Expression::Integer(amount),
    ] = items.as_slice()
    else {
        return None;
    };
    (source.slice(*name) == parameter && *kind == step)
        .then(|| parse_integer(source.slice(*amount)))
        .flatten()
        .filter(|value| value > &BigInt::from(0_u8))
}

fn supported_value_classifier(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    matches!(
        classifier,
        "Boolean"
            | "Character"
            | "Completed"
            | "Comparison"
            | "Constraint"
            | "Effect"
            | "Error"
            | "ErrorCode"
            | "Generator Character Unit Unit"
            | "Generator Character Unit Character"
            | "Generator String Unit Unit"
            | "Generator String Unit Character"
            | "Generator String Unit String"
            | "Generator Character Unit String"
            | "Function"
            | "Int"
            | "MessageContext"
            | "Nat"
            | "Range Int"
            | "Range Rational"
            | "Rational"
            | "Scope"
            | "String"
            | "Type"
            | "Unit"
    ) || enum_types.contains_key(classifier)
        || generator_classifiers(classifier).is_some_and(|(yielded, resumed, returned)| {
            resumed == "Unit"
                && supported_generator_value_classifier(yielded, enum_types)
                && supported_generator_value_classifier(returned, enum_types)
        })
        || optional_payload_classifier(classifier)
            .is_some_and(|payload| supported_value_classifier(payload, enum_types))
        || list_element_classifier(classifier)
            .is_some_and(|element| supported_value_classifier(element, enum_types))
        || array_classifier_parts(classifier)
            .is_some_and(|(_, element)| supported_value_classifier(element, enum_types))
        || applied_classifier(classifier, "Set")
            .is_some_and(|element| supported_value_classifier(element, enum_types))
        || applied_classifier(classifier, "Bag")
            .is_some_and(|element| supported_value_classifier(element, enum_types))
        || map_classifier_parts(classifier).is_some_and(|(key, value)| {
            supported_value_classifier(key, enum_types)
                && supported_value_classifier(value, enum_types)
        })
        || tuple_classifiers(classifier).is_some_and(|items| {
            items
                .into_iter()
                .all(|item| supported_value_classifier(item, enum_types))
        })
        || record_classifiers(classifier).is_some_and(|fields| {
            fields
                .into_iter()
                .all(|(_, field)| supported_value_classifier(field, enum_types))
        })
        || result_success_classifier(classifier)
            .is_some_and(|success| supported_value_classifier(success, enum_types))
}

fn record_classifiers(classifier: &str) -> Option<Vec<(&str, &str)>> {
    let contents = classifier
        .trim()
        .strip_prefix("Record")?
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let fields = split_top_level_classifier_items(contents)?;
    let mut labels = BTreeSet::new();
    fields
        .into_iter()
        .map(|field| {
            let (label, classifier) = split_top_level_classifier_field(field)?;
            labels.insert(label).then_some((label, classifier))
        })
        .collect()
}

fn array_classifier_parts(classifier: &str) -> Option<(usize, &str)> {
    let (count, element) = binary_classifier_parts(classifier, "Array")?;
    Some((count.parse().ok()?, element))
}

fn map_classifier_parts(classifier: &str) -> Option<(&str, &str)> {
    binary_classifier_parts(classifier, "Map")
}

fn binary_classifier_parts<'a>(
    classifier: &'a str,
    constructor: &str,
) -> Option<(&'a str, &'a str)> {
    let contents = classifier
        .trim()
        .strip_prefix(constructor)?
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let fields = split_top_level_classifier_items(contents)?;
    let [first, second] = fields.as_slice() else {
        return None;
    };
    Some((*first, *second))
}

fn split_top_level_classifier_items(contents: &str) -> Option<Vec<&str>> {
    let mut items = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0_usize;
    for (offset, character) in contents.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                items.push(contents[start..offset].trim());
                start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    items.push(contents[start..].trim());
    items.iter().all(|item| !item.is_empty()).then_some(items)
}

fn split_top_level_classifier_field(field: &str) -> Option<(&str, &str)> {
    let mut depth = 0_usize;
    for (offset, character) in field.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ':' if depth == 0 => {
                let label = field[..offset].trim();
                let classifier = field[offset + character.len_utf8()..].trim();
                return (!label.is_empty() && !classifier.is_empty())
                    .then_some((label, classifier));
            }
            _ => {}
        }
    }
    None
}

fn generator_classifiers(classifier: &str) -> Option<(&str, &str, &str)> {
    let contents = classifier.trim().strip_prefix("Generator")?;
    let (yielded, contents) = take_classifier(contents)?;
    let (resumed, contents) = take_classifier(contents)?;
    let (returned, remainder) = take_classifier(contents)?;
    remainder
        .trim()
        .is_empty()
        .then_some((yielded, resumed, returned))
}

fn take_classifier(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    if text.starts_with('(') {
        let end = parenthesized_end(text)?;
        return Some((&text[..end], &text[end..]));
    }
    let head_end = text.find(char::is_whitespace).unwrap_or(text.len());
    let head = &text[..head_end];
    if head.is_empty() {
        return None;
    }
    if head == "Result" {
        let tail = text[head_end..].trim_start();
        let result_end = parenthesized_end(tail)?;
        let end = text.len() - tail.len() + result_end;
        return Some((&text[..end], &text[end..]));
    }
    let arity = match head {
        "Optional" | "Range" | "List" => 1,
        "Generator" => 3,
        _ => 0,
    };
    let mut remainder = &text[head_end..];
    for _ in 0..arity {
        let (_, next) = take_classifier(remainder)?;
        remainder = next;
    }
    let end = text.len() - remainder.len();
    Some((text[..end].trim_end(), remainder))
}

fn parenthesized_end(text: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(offset + character.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

fn accepted_source(input: &str, trace: &mut impl TraceSink) -> Result<SourceText, Diagnostic> {
    let source = SourceText::new(input).map_err(|error| {
        let (line, column) = raw_position(input, error.span.start);
        let mut diagnostic = Diagnostic::error(error.code, line, column, error.message)
            .with_source_excerpt(
                raw_source_line(input, line),
                marker_width(input, error.span),
            );
        if let Some(help) = diagnostic_help(error.code) {
            diagnostic = diagnostic.with_help(help);
        }
        diagnostic
    })?;
    trace.record(TraceEvent {
        event: "source.accepted",
        rule: "TOPAL-SYN-SOURCE-001",
        detail: "unicode source normalized",
    });
    Ok(source)
}

fn expected_statement(input: &str) -> Diagnostic {
    Diagnostic::error("E-EXPECTED-EXPRESSION", 1, 1, "expected a statement")
        .with_source_excerpt(raw_source_line(input, 1), 1)
        .with_help(
            diagnostic_help("E-EXPECTED-EXPRESSION")
                .expect("expected-expression diagnostic has stable help"),
        )
}

fn record_result(trace: &mut impl TraceSink, value: &Value) {
    let classifier = structural_value_classifier(value);
    trace.record(TraceEvent {
        event: "evaluation.result",
        rule: "TOPAL-SYN-GRAMMAR-001",
        detail: &classifier,
    });
}

fn validate_layout_attributes(
    source: &SourceText,
    span: Span,
    semantic: &str,
    attributes: &[(String, Value)],
) -> Result<(), Diagnostic> {
    const COMMON: &[&str] = &["storage-size", "encoding", "endian", "access", "alignment"];
    const EXTRA: &[&str] = &[
        "bit-order",
        "unit-size",
        "canonical",
        "length",
        "false-pattern",
        "true-pattern",
        "bias",
        "numerator-layout",
        "denominator-layout",
        "integer-layout",
        "quantum",
        "exponent-bits",
        "fraction-bits",
        "exponent-bias",
        "subnormal",
        "infinity",
        "signed-zero",
        "nan",
        "termination",
        "padding",
        "packing",
        "field-order",
        "tag-layout",
        "tags",
        "payload-placement",
        "element-layout",
        "stride",
        "entry-layout",
        "ordering",
        "measurement-unit",
    ];
    for (name, _) in attributes {
        if !COMMON.contains(&name.as_str()) && !EXTRA.contains(&name.as_str()) {
            return Err(diagnostic(
                source,
                "E-LAYOUT-UNKNOWN-FIELD",
                span,
                format!("`{name}` is not a layout attribute"),
            ));
        }
    }
    let field = |name: &str| {
        attributes
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value)
    };
    if let Some(size) = field("storage-size")
        && !matches!(size, Value::SizeBits(bits) if bits >= &BigInt::from(0))
    {
        return Err(diagnostic(
            source,
            "E-LAYOUT-STORAGE-SIZE",
            span,
            "storage-size requires a nonnegative bit or byte size",
        ));
    }
    if semantic == "Unit" {
        if field("storage-size").is_some_and(|value| value != &Value::SizeBits(BigInt::from(0))) {
            return Err(diagnostic(
                source,
                "E-LAYOUT-UNIT-SIZE",
                span,
                "Layout Unit has exactly 0[b] storage",
            ));
        }
    } else if matches!(
        semantic,
        "Boolean" | "Nat" | "Int" | "Rational" | "String" | "Character"
    ) && field("encoding").is_none()
    {
        return Err(diagnostic(
            source,
            "E-LAYOUT-ENCODING",
            span,
            format!("Layout {semantic} requires an encoding"),
        ));
    }
    Ok(())
}

fn sync_stream_task_state(
    session: &Session,
    owner: Option<&str>,
    state: Option<&BTreeMap<String, Value>>,
) {
    let (Some(_), Some(state), Some(Value::TaskInstance(instance))) = (
        owner,
        state,
        owner.and_then(|name| session.bindings.get(name)),
    ) else {
        return;
    };
    instance.borrow_mut().state = state.clone();
}

fn layout_access(layout: &LayoutValue) -> &str {
    layout
        .attributes
        .iter()
        .find_map(|(name, value)| {
            (name == "access")
                .then_some(value)
                .and_then(|value| match value {
                    Value::Enum { alternative, .. } => Some(alternative.as_str()),
                    _ => None,
                })
        })
        .unwrap_or("ReadWrite")
}

fn validate_address_offset(
    source: &SourceText,
    span: Span,
    attributes: &[(String, Value)],
    offset: &BigInt,
) -> Result<(), Diagnostic> {
    if let Some(Value::Int(alignment)) = attributes
        .iter()
        .find_map(|(name, value)| (name == "alignment").then_some(value))
        && (alignment <= &BigInt::from(0) || offset % alignment != BigInt::from(0))
    {
        return Err(diagnostic(
            source,
            "E-ADDRESS-OFFSET-ALIGNMENT",
            span,
            "address offset does not satisfy its byte alignment",
        ));
    }
    if let Some(Value::AddressRange { lower, upper, .. }) = attributes
        .iter()
        .find_map(|(name, value)| (name == "range").then_some(value))
        && offset > &(upper - lower)
    {
        return Err(diagnostic(
            source,
            "E-ADDRESS-OFFSET-RANGE",
            span,
            "address offset lies outside its associated range",
        ));
    }
    Ok(())
}

fn validate_location_fit(
    source: &SourceText,
    span: Span,
    layout: &LayoutValue,
    offset_attributes: &[(String, Value)],
    offset: &BigInt,
) -> Result<(), Diagnostic> {
    let size_bits = layout.attributes.iter().find_map(|(name, value)| {
        (name == "storage-size")
            .then_some(value)
            .and_then(|value| match value {
                Value::SizeBits(bits) => Some(bits),
                _ => None,
            })
    });
    if let (Some(bits), Some(Value::AddressRange { lower, upper, .. })) = (
        size_bits,
        offset_attributes
            .iter()
            .find_map(|(name, value)| (name == "range").then_some(value)),
    ) {
        let bytes = (bits + BigInt::from(7)) / BigInt::from(8);
        if offset + bytes > upper - lower + BigInt::from(1) {
            return Err(diagnostic(
                source,
                "E-LOCATION-RANGE",
                span,
                "layout does not fit in the associated address range",
            ));
        }
    }
    Ok(())
}

fn coerce_layout_value(
    source: &SourceText,
    span: Span,
    layout: &LayoutValue,
    value: Value,
) -> Result<Value, Diagnostic> {
    if let Value::LayoutBacked {
        layout: existing, ..
    } = &value
        && existing.as_ref() == layout
    {
        return Ok(value);
    }
    if !value_has_classifier(&value, &layout.semantic) {
        return Err(diagnostic(
            source,
            "E-LAYOUT-SEMANTIC-VALUE",
            span,
            format!(
                "Layout {} requires a {} value",
                layout.semantic, layout.semantic
            ),
        ));
    }
    if let Value::Int(integer) = &value
        && let Some(Value::SizeBits(bits)) = layout
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "storage-size").then_some(value))
    {
        let unsigned = layout.attributes.iter().any(|(name, value)| {
            name == "encoding"
                && matches!(value, Value::Enum { alternative, .. } if alternative == "UnsignedBinary")
        });
        let limit = usize::try_from(bits)
            .ok()
            .map(|width| BigInt::from(1_u8) << width);
        if unsigned
            && (integer < &BigInt::from(0) || limit.as_ref().is_some_and(|limit| integer >= limit))
        {
            return Err(diagnostic(
                source,
                "E-LAYOUT-NOT-REPRESENTABLE",
                span,
                "integer is not representable by the selected layout",
            ));
        }
    }
    Ok(Value::LayoutBacked {
        layout: Box::new(layout.clone()),
        value: Box::new(value),
    })
}

fn value_classifier(value: &Value) -> &'static str {
    match value {
        Value::Boolean(_) => "Boolean",
        Value::Version(_) => "Version",
        Value::SerializationStream(_) => "SerializationStream",
        Value::ObjectDescription { .. } => "ObjectDescription",
        Value::TaskType(_)
        | Value::AddressRangeType(_)
        | Value::AddressOffsetType(_)
        | Value::LayoutType(_)
        | Value::LayoutFactory(_)
        | Value::LocationType(_)
        | Value::Type(_)
        | Value::ModularType(_) => "Type",
        Value::TaskDefinition(_) => "TaskDefinition",
        Value::TaskInstance(_) => "Task",
        Value::SizeBits(_) => "Size",
        Value::AddressRange { .. } => "AddressRange",
        Value::AddressOffset { .. } => "AddressOffset",
        Value::Location { .. } => "Location",
        Value::LayoutBacked { .. } => "Layout",
        Value::Effects(_) => "Effect",
        Value::Infinity { classifier, .. } if classifier == "Nat" => "Nat",
        Value::Infinity { classifier, .. } if classifier == "Rational" => "Rational",
        Value::Int(_) | Value::Infinity { .. } => "Int",
        Value::Rational(_) => "Rational",
        Value::IntRange { .. }
        | Value::InfiniteIntRange { .. }
        | Value::RationalRange { .. }
        | Value::InfiniteRationalRange { .. } => "Range",
        Value::Optional { .. } => "Optional",
        Value::List { .. } => "List",
        Value::Callable(_)
        | Value::NamedFunction(_)
        | Value::AnonymousFunction(_)
        | Value::NativeSerializer(_) => "Function",
        Value::Namespace(_) => "Scope",
        Value::Array { .. } => "Array",
        Value::Set { .. } => "Set",
        Value::Bag { .. } => "Bag",
        Value::Map { .. } => "Map",
        Value::CharacterReturningGenerator { .. } => "Generator Character Unit Character",
        Value::IterateGenerator { .. } | Value::UnfoldGenerator { .. } => "Generator",
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } if yield_classifier == "String" && return_classifier == "String" => {
            "Generator String Unit String"
        }
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } if yield_classifier == "String" && return_classifier == "Character" => {
            "Generator String Unit Character"
        }
        Value::SuspendedGenerator {
            yield_classifier, ..
        } if yield_classifier == "String" => "Generator String Unit Unit",
        Value::SuspendedGenerator {
            return_classifier, ..
        } if return_classifier == "String" => "Generator Character Unit String",
        Value::SuspendedGenerator {
            return_classifier, ..
        } if return_classifier == "Character" => "Generator Character Unit Character",
        Value::CharacterGenerator { .. } | Value::SuspendedGenerator { .. } => {
            "Generator Character Unit Unit"
        }
        Value::String(_) => "String",
        Value::Tuple(_) => "Tuple",
        Value::Record(_) => "Record",
        Value::Enum { .. } => "Enum",
        Value::Union(_) => "Union",
        Value::Constraint(_) => "Constraint",
        Value::Capability(_) => "Capability",
        Value::Interface(_) => "Interface",
        Value::Introspection(value) => match value.as_ref() {
            IntrospectionValue::Identity { .. } => "lang Identity",
            IntrospectionValue::TypeView { .. } => "lang TypeView",
            IntrospectionValue::FunctionView { .. } => "lang FunctionView",
            IntrospectionValue::ScopeView { .. } => "lang ScopeView",
            IntrospectionValue::ConstraintView { .. } => "lang ConstraintView",
            IntrospectionValue::EffectView { .. } => "lang EffectView",
            IntrospectionValue::ProtocolView { .. } => "lang ProtocolView",
            IntrospectionValue::DeclarationView { .. } => "lang DeclarationView",
            IntrospectionValue::LanguageContext { .. } => "lang LanguageContext",
        },
        Value::Refined { .. } => "Refined",
        Value::Modular { .. } => "Modular",
        Value::ErrorDomain(_) => "ErrorDomain",
        Value::Error { .. } => "Error",
        Value::Continue(_) | Value::Finish(_) => "TraversalControl",
        Value::Completed => "Completed",
        Value::Unit => "Unit",
    }
}

fn structural_value_classifier(value: &Value) -> String {
    match value {
        Value::IntRange { .. } | Value::InfiniteIntRange { .. } => "Range Int".into(),
        Value::Infinity { classifier, .. } => classifier.clone(),
        Value::RationalRange { .. } | Value::InfiniteRationalRange { .. } => {
            "Range Rational".into()
        }
        Value::Tuple(values) => format!(
            "({})",
            values
                .iter()
                .map(structural_value_classifier)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Optional {
            payload_classifier, ..
        } => format!("Optional {payload_classifier}"),
        Value::List {
            element_classifier, ..
        } => format!("List {element_classifier}"),
        Value::Array {
            element_classifier,
            entries,
        } => format!("Array {} {element_classifier}", entries.len()),
        Value::Set {
            element_classifier, ..
        } => format!("Set {element_classifier}"),
        Value::Bag {
            element_classifier, ..
        } => format!("Bag {element_classifier}"),
        Value::Map {
            key_classifier,
            value_classifier,
            ..
        } => format!("Map ({key_classifier}, {value_classifier})"),
        Value::SuspendedGenerator {
            yield_classifier,
            return_classifier,
            ..
        } => format!("Generator {yield_classifier} Unit {return_classifier}"),
        Value::IterateGenerator { classifier, .. } => {
            format!("Generator {classifier} Unit Unit")
        }
        Value::UnfoldGenerator {
            yield_classifier, ..
        } => format!("Generator {yield_classifier} Unit Unit"),
        Value::Enum { type_name, .. } | Value::Modular { type_name, .. } => type_name.clone(),
        Value::Union(union) => union.type_name.clone(),
        Value::Constraint(constraint) => format!("Constraint {}", constraint.base_classifier),
        Value::Refined { constraint, .. } => constraint.clone(),
        Value::Type(name) => name.clone(),
        Value::Effects(_) => "Effect".into(),
        Value::ModularType(kind) => kind.name.clone().unwrap_or_else(|| "Type".into()),
        Value::LayoutBacked { layout, .. } => layout.semantic.clone(),
        _ => value_classifier(value).to_owned(),
    }
}

fn classifier_name(source: &SourceText, expression: &Expression) -> Option<String> {
    match expression {
        Expression::Identifier(span) => Some(source.slice(*span).to_owned()),
        _ => None,
    }
}

fn introspection_identity(value: &Value) -> Option<String> {
    match value {
        Value::Type(name) => Some(format!("type:{name}")),
        Value::ModularType(kind) => Some(format!(
            "type:{}:{}..{}",
            if kind.signed { "ModInt" } else { "ModNat" },
            kind.lower,
            kind.upper
        )),
        Value::NamedFunction(function) => Some(format!("function:root.{}", function.name)),
        Value::Callable(kind) => Some(format!("function:root.{}", callable_name(*kind))),
        Value::Namespace(namespace) => Some(format!("scope:{}", namespace.name)),
        Value::Constraint(constraint) => constraint
            .name
            .as_ref()
            .map(|name| format!("constraint:root.{name}")),
        Value::Capability(alternatives) => Some(format!("capability:{alternatives:?}")),
        Value::Effects(effects) => Some(format!("effect-set:{effects:?}")),
        Value::Interface(interface) => Some(format!("interface:root.{}", interface.name)),
        Value::Introspection(value) => match value.as_ref() {
            IntrospectionValue::Identity { canonical, .. } => Some(canonical.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn introspection_view(source: &SourceText, value: Value, span: Span) -> Result<Value, Diagnostic> {
    let view = match value {
        Value::Type(identity) => IntrospectionValue::TypeView {
            form: "PrimitiveType".into(),
            identity,
        },
        Value::ModularType(kind) => IntrospectionValue::TypeView {
            form: "RefinedType".into(),
            identity: kind.name.clone().unwrap_or_else(|| {
                format!(
                    "{} {}..{}",
                    if kind.signed { "ModInt" } else { "ModNat" },
                    kind.lower,
                    kind.upper
                )
            }),
        },
        Value::NamedFunction(function) => {
            let Some(first) = function.candidates.first() else {
                return Err(diagnostic(
                    source,
                    "E-INTROSPECTION-EMPTY-FUNCTION",
                    span,
                    "function view has no declared overload",
                ));
            };
            IntrospectionValue::FunctionView {
                identity: format!("root.{}", function.name),
                inputs: first
                    .parameters
                    .iter()
                    .map(|(_, classifier)| classifier.clone())
                    .collect(),
                output: first.result.clone(),
                is_static: first.is_static,
                effects: first.metadata.effect_bound.iter().cloned().collect(),
            }
        }
        Value::Namespace(namespace) => {
            let mut members = namespace
                .bindings
                .keys()
                .chain(namespace.functions.keys())
                .chain(namespace.generators.keys())
                .cloned()
                .collect::<Vec<_>>();
            members.sort();
            members.dedup();
            IntrospectionValue::ScopeView {
                identity: namespace.name.clone(),
                members,
            }
        }
        Value::Constraint(constraint) => IntrospectionValue::ConstraintView {
            identity: constraint
                .name
                .clone()
                .unwrap_or_else(|| constraint.base_classifier.clone()),
            base: constraint.base_classifier,
        },
        Value::Effects(identities) => IntrospectionValue::EffectView { identities },
        Value::Interface(interface) => IntrospectionValue::ProtocolView {
            identity: format!("root.{}", interface.name),
            operations: interface.functions.keys().cloned().collect(),
        },
        _ => {
            return Err(diagnostic(
                source,
                "E-STATIC-INTROSPECTION-SUBJECT",
                span,
                "lang view requires a statically known Type, Function, Scope, Constraint, Effect, or Protocol",
            ));
        }
    };
    Ok(Value::Introspection(Box::new(view)))
}

fn stream_for_value(
    version: LanguageVersion,
    value: &Value,
) -> Result<SerializationStream, &'static str> {
    let mut types = Vec::new();
    let mut identities = BTreeMap::new();
    let (type_id, value) = serialize_language_value(value, &mut types, &mut identities)?;
    Ok(SerializationStream {
        header: SerializationHeader {
            language_identity: "topal".into(),
            language_version: version,
            byte_order: if cfg!(target_endian = "little") {
                StreamByteOrder::Little
            } else {
                StreamByteOrder::Big
            },
            streaming: false,
        },
        types,
        events: vec![SerializedEvent { type_id, value }],
    })
}

#[allow(clippy::too_many_lines)] // Each supported semantic schema remains explicit at the authority-free boundary.
fn serialize_language_value(
    value: &Value,
    types: &mut Vec<TypeDefinition>,
    identities: &mut BTreeMap<String, usize>,
) -> Result<(usize, SerializedValue), &'static str> {
    let (identity, definition, serialized) = match value {
        Value::Unit => (
            "Unit".to_owned(),
            TypeDefinition::Unit {
                identity: "Unit".into(),
            },
            SerializedValue::Unit,
        ),
        Value::Boolean(value) => (
            "Boolean".to_owned(),
            TypeDefinition::Boolean {
                identity: "Boolean".into(),
            },
            SerializedValue::Boolean(*value),
        ),
        Value::Int(value) => (
            "Int".to_owned(),
            TypeDefinition::Int {
                identity: "Int".into(),
                signed: true,
                width_bits: 0,
            },
            SerializedValue::ArbitraryInt(value.clone()),
        ),
        Value::Rational(value) => {
            let (numerator_id, numerator) =
                serialize_language_value(&Value::Int(value.numer().clone()), types, identities)?;
            let (denominator_id, denominator) =
                serialize_language_value(&Value::Int(value.denom().clone()), types, identities)?;
            let mut schema_payload = Vec::new();
            put_protocol_uvarint(numerator_id, &mut schema_payload);
            put_protocol_uvarint(denominator_id, &mut schema_payload);
            (
                "Rational".into(),
                TypeDefinition::ObjectDescription {
                    identity: "Rational".into(),
                    kind: 3,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![numerator, denominator]),
            )
        }
        Value::String(value) => (
            "String".to_owned(),
            TypeDefinition::Text {
                identity: "String".into(),
            },
            SerializedValue::Text(value.clone()),
        ),
        Value::Tuple(values) => {
            let mut components = Vec::with_capacity(values.len());
            let mut encoded = Vec::with_capacity(values.len());
            for value in values {
                let (id, value) = serialize_language_value(value, types, identities)?;
                components.push(id);
                encoded.push(value);
            }
            let identity = structural_value_classifier(value);
            (
                identity.clone(),
                TypeDefinition::Tuple {
                    identity,
                    components,
                },
                SerializedValue::Product(encoded),
            )
        }
        Value::Record(fields) => {
            let mut definitions = Vec::with_capacity(fields.len());
            let mut encoded = Vec::with_capacity(fields.len());
            for (label, value) in fields {
                let (id, value) = serialize_language_value(value, types, identities)?;
                definitions.push((label.clone(), id));
                encoded.push(value);
            }
            let identity = structural_value_classifier(value);
            (
                identity.clone(),
                TypeDefinition::Record {
                    identity,
                    fields: definitions,
                },
                SerializedValue::Product(encoded),
            )
        }
        Value::List {
            element_classifier,
            entries,
        }
        | Value::Array {
            element_classifier,
            entries,
        } => {
            let (element, encoded) = serialize_homogeneous_values(entries, types, identities)?;
            let identity = match value {
                Value::List { .. } => format!("List {element_classifier}"),
                Value::Array { .. } => format!("Array {} {element_classifier}", entries.len()),
                _ => unreachable!(),
            };
            (
                identity.clone(),
                TypeDefinition::Sequence { identity, element },
                SerializedValue::Sequence(encoded),
            )
        }
        Value::Bag {
            element_classifier,
            entries,
        } => {
            let values = entries
                .iter()
                .flat_map(|(value, count)| std::iter::repeat_n(value.clone(), *count))
                .collect::<Vec<_>>();
            let (element, encoded) = serialize_homogeneous_values(&values, types, identities)?;
            let identity = format!("Bag {element_classifier}");
            (
                identity.clone(),
                TypeDefinition::Sequence { identity, element },
                SerializedValue::Sequence(encoded),
            )
        }
        Value::Set {
            element_classifier,
            entries,
        } => {
            let values = entries.clone();
            let (element, mut encoded) = serialize_homogeneous_values(&values, types, identities)?;
            encoded.sort();
            let mut schema_payload = Vec::new();
            put_protocol_uvarint(element, &mut schema_payload);
            put_protocol_text("semantic-total-order", &mut schema_payload);
            let identity = format!("Set {element_classifier}");
            (
                identity.clone(),
                TypeDefinition::ObjectDescription {
                    identity,
                    kind: 11,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![SerializedValue::Sequence(encoded)]),
            )
        }
        Value::Map {
            key_classifier,
            value_classifier,
            entries,
        } => {
            let mut key_type = None;
            let mut value_type = None;
            let mut encoded = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                let (key_id, key) = serialize_language_value(key, types, identities)?;
                let (value_id, value) = serialize_language_value(value, types, identities)?;
                if key_type.replace(key_id).is_some_and(|id| id != key_id)
                    || value_type
                        .replace(value_id)
                        .is_some_and(|id| id != value_id)
                {
                    return Err("map entries do not share one key and value serialization schema");
                }
                encoded.push(SerializedValue::Product(vec![key, value]));
            }
            let (key_type, value_type) = (
                key_type.ok_or("an empty Map needs explicit serializable schemas")?,
                value_type.unwrap(),
            );
            encoded.sort_by(|left, right| match (left, right) {
                (SerializedValue::Product(left), SerializedValue::Product(right)) => {
                    left[0].cmp(&right[0])
                }
                _ => Ordering::Equal,
            });
            let mut schema_payload = Vec::new();
            put_protocol_uvarint(key_type, &mut schema_payload);
            put_protocol_uvarint(value_type, &mut schema_payload);
            put_protocol_text("semantic-total-order", &mut schema_payload);
            let identity = format!("Map ({key_classifier}, {value_classifier})");
            (
                identity.clone(),
                TypeDefinition::ObjectDescription {
                    identity,
                    kind: 12,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![SerializedValue::Sequence(encoded)]),
            )
        }
        _ => {
            let (schema, description) =
                serialize_language_value(&Value::String(value.to_string()), types, identities)?;
            let identity = format!("description:{}", structural_value_classifier(value));
            let kind = format!("{:?}", value.object_kind());
            let mut schema_payload = Vec::new();
            put_protocol_text(&kind, &mut schema_payload);
            put_protocol_uvarint(schema, &mut schema_payload);
            (
                identity.clone(),
                TypeDefinition::ObjectDescription {
                    identity,
                    kind: 15,
                    schema_payload,
                },
                SerializedValue::ObjectDescription(vec![description]),
            )
        }
    };
    if let Some(id) = identities.get(&identity) {
        return Ok((*id, serialized));
    }
    let id = types.len();
    types.push(definition);
    identities.insert(identity, id);
    Ok((id, serialized))
}

fn serialize_homogeneous_values(
    values: &[Value],
    types: &mut Vec<TypeDefinition>,
    identities: &mut BTreeMap<String, usize>,
) -> Result<(usize, Vec<SerializedValue>), &'static str> {
    let mut element = None;
    let mut encoded = Vec::with_capacity(values.len());
    for value in values {
        let (id, value) = serialize_language_value(value, types, identities)?;
        if element.replace(id).is_some_and(|existing| existing != id) {
            return Err("collection entries do not share one native serialization type");
        }
        encoded.push(value);
    }
    let element =
        element.ok_or("an empty collection needs an explicit serializable element schema")?;
    Ok((element, encoded))
}

fn put_protocol_uvarint(mut value: usize, output: &mut Vec<u8>) {
    loop {
        let mut byte = u8::try_from(value & 0x7f).expect("seven-bit payload fits in u8");
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn put_protocol_text(value: &str, output: &mut Vec<u8>) {
    put_protocol_uvarint(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}

fn value_from_serialized(event: &SerializedEvent, types: &[TypeDefinition]) -> Option<Value> {
    deserialize_language_value(event.type_id, &event.value, types)
}

#[allow(clippy::too_many_lines)] // Schema reconstruction mirrors the explicit serializer cases.
fn deserialize_language_value(
    type_id: usize,
    value: &SerializedValue,
    types: &[TypeDefinition],
) -> Option<Value> {
    match (types.get(type_id)?, value) {
        (TypeDefinition::Unit { .. }, SerializedValue::Unit) => Some(Value::Unit),
        (TypeDefinition::Boolean { .. }, SerializedValue::Boolean(value)) => {
            Some(Value::Boolean(*value))
        }
        (TypeDefinition::Int { .. }, SerializedValue::Int(value)) => {
            Some(Value::Int(BigInt::from(*value)))
        }
        (TypeDefinition::Int { .. }, SerializedValue::ArbitraryInt(value)) => {
            Some(Value::Int(value.clone()))
        }
        (TypeDefinition::Text { .. }, SerializedValue::Text(value)) => {
            Some(Value::String(value.clone()))
        }
        (TypeDefinition::Tuple { components, .. }, SerializedValue::Product(values)) => {
            Some(Value::Tuple(
                components
                    .iter()
                    .zip(values)
                    .map(|(id, value)| deserialize_language_value(*id, value, types))
                    .collect::<Option<Vec<_>>>()?,
            ))
        }
        (TypeDefinition::Record { fields, .. }, SerializedValue::Product(values)) => {
            Some(Value::Record(
                fields
                    .iter()
                    .zip(values)
                    .map(|((label, id), value)| {
                        Some((
                            label.clone(),
                            deserialize_language_value(*id, value, types)?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?,
            ))
        }
        (
            TypeDefinition::ObjectDescription {
                identity, kind: 3, ..
            },
            SerializedValue::ObjectDescription(values),
        ) if identity == "Rational" => {
            let [
                SerializedValue::ArbitraryInt(numerator),
                SerializedValue::ArbitraryInt(denominator),
            ] = values.as_slice()
            else {
                return None;
            };
            Some(Value::Rational(BigRational::new(
                numerator.clone(),
                denominator.clone(),
            )))
        }
        (TypeDefinition::Sequence { identity, element }, SerializedValue::Sequence(values)) => {
            let entries = values
                .iter()
                .map(|value| deserialize_language_value(*element, value, types))
                .collect::<Option<Vec<_>>>()?;
            if let Some(classifier) = identity.strip_prefix("List ") {
                Some(Value::List {
                    element_classifier: classifier.into(),
                    entries,
                })
            } else if let Some(rest) = identity.strip_prefix("Array ") {
                let (_, classifier) = rest.split_once(' ')?;
                Some(Value::Array {
                    element_classifier: classifier.into(),
                    entries,
                })
            } else {
                identity.strip_prefix("Bag ").map(|classifier| Value::Bag {
                    element_classifier: classifier.into(),
                    entries: entries.into_iter().map(|value| (value, 1)).collect(),
                })
            }
        }
        (
            TypeDefinition::ObjectDescription {
                identity,
                kind: 11,
                schema_payload,
            },
            SerializedValue::ObjectDescription(values),
        ) => {
            let [SerializedValue::Sequence(values)] = values.as_slice() else {
                return None;
            };
            let (element, _) = read_protocol_uvarint(schema_payload)?;
            let entries = values
                .iter()
                .map(|value| deserialize_language_value(element, value, types))
                .collect::<Option<Vec<_>>>()?;
            Some(Value::Set {
                element_classifier: identity.strip_prefix("Set ")?.into(),
                entries,
            })
        }
        (
            TypeDefinition::ObjectDescription {
                identity,
                kind: 12,
                schema_payload,
            },
            SerializedValue::ObjectDescription(values),
        ) => {
            let [SerializedValue::Sequence(values)] = values.as_slice() else {
                return None;
            };
            let (key_type, used) = read_protocol_uvarint(schema_payload)?;
            let (value_type, _) = read_protocol_uvarint(&schema_payload[used..])?;
            let entries = values
                .iter()
                .map(|entry| {
                    let SerializedValue::Product(pair) = entry else {
                        return None;
                    };
                    let [key, value] = pair.as_slice() else {
                        return None;
                    };
                    Some((
                        deserialize_language_value(key_type, key, types)?,
                        deserialize_language_value(value_type, value, types)?,
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            let classifiers = identity.strip_prefix("Map (")?.strip_suffix(')')?;
            let (key_classifier, value_classifier) = classifiers.split_once(", ")?;
            Some(Value::Map {
                key_classifier: key_classifier.into(),
                value_classifier: value_classifier.into(),
                entries,
            })
        }
        (
            TypeDefinition::ObjectDescription {
                identity,
                kind: 15,
                schema_payload,
            },
            SerializedValue::ObjectDescription(values),
        ) => {
            let (kind, used) = read_protocol_text(schema_payload)?;
            let (schema, _) = read_protocol_uvarint(&schema_payload[used..])?;
            let [description] = values.as_slice() else {
                return None;
            };
            Some(Value::ObjectDescription {
                identity: identity.clone(),
                kind,
                value: Box::new(deserialize_language_value(schema, description, types)?),
            })
        }
        _ => None,
    }
}

fn read_protocol_uvarint(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0_usize;
    for (index, byte) in bytes.iter().copied().enumerate() {
        let shift = index.checked_mul(7)?;
        value |= usize::from(byte & 0x7f).checked_shl(u32::try_from(shift).ok()?)?;
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
    }
    None
}

fn read_protocol_text(bytes: &[u8]) -> Option<(String, usize)> {
    let (length, used) = read_protocol_uvarint(bytes)?;
    let end = used.checked_add(length)?;
    Some((std::str::from_utf8(bytes.get(used..end)?).ok()?.into(), end))
}

fn generator_classifier_diagnostic(
    source: &SourceText,
    code: &'static str,
    span: Span,
    name: &str,
    action: &str,
    expected: &str,
    value: &Value,
) -> Diagnostic {
    let found = structural_value_classifier(value);
    diagnostic(
        source,
        code,
        span,
        format!("generator `{name}` {action} `{found}`, but its declaration requires `{expected}`"),
    )
    .with_help(format!(
        "produce `{expected}` here or change the generator's declared classifier from `{expected}`"
    ))
}

fn classifier_expression(source: &SourceText, expression: &Expression) -> Option<String> {
    match expression {
        Expression::Identifier(span) => Some(source.slice(*span).to_owned()),
        Expression::Product { fields, .. }
            if fields.len() > 1 && fields.iter().all(|field| field.label.is_none()) =>
        {
            Some(format!(
                "({})",
                fields
                    .iter()
                    .map(|field| classifier_expression(source, &field.value))
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            ))
        }
        _ => None,
    }
}

fn evaluate_boolean_literal(source: &SourceText, span: Span, trace: &mut impl TraceSink) -> Value {
    let lexeme = source.slice(span);
    trace.record(TraceEvent {
        event: "token.boolean",
        rule: "TOPAL-TYPE-BOOLEAN-001",
        detail: lexeme,
    });
    Value::Boolean(lexeme == "true")
}

fn evaluate_integer_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let text = source.slice(span);
    let value = parse_integer(text)
        .ok_or_else(|| diagnostic(source, "E-NUMERIC-LITERAL", span, "invalid integer literal"))?;
    trace.record(TraceEvent {
        event: "token.integer",
        rule: "TOPAL-NUM-LITERAL-001",
        detail: text,
    });
    Ok(Value::Int(value))
}
