fn external_record_fields<'a>(
    source: &SourceText,
    expression: &'a Expression,
    context: &str,
) -> Result<BTreeMap<String, &'a Expression>, Diagnostic> {
    let Expression::Product { fields, .. } = expression else {
        return Err(unsupported(
            source,
            expression.span(),
            &format!("{context} attributes outside a labeled record"),
        ));
    };
    let mut result = BTreeMap::new();
    for field in fields {
        let Some(label) = field.label else {
            return Err(unsupported(
                source,
                field.value.span(),
                &format!("unlabeled {context} attribute"),
            ));
        };
        let label = source.slice(label).to_owned();
        if result.insert(label.clone(), &field.value).is_some() {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-RECORD-FIELD",
                field.value.span(),
                format!("`{label}` occurs more than once"),
            ));
        }
    }
    Ok(result)
}

fn require_external_fields(
    source: &SourceText,
    span: Span,
    fields: &BTreeMap<String, &Expression>,
    expected: &[&str],
    context: &str,
) -> Result<(), Diagnostic> {
    let actual = fields.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(unsupported(
            source,
            span,
            &format!("closed {context} field schema"),
        ))
    }
}

fn external_identifier_field(
    source: &SourceText,
    fields: &BTreeMap<String, &Expression>,
    name: &str,
) -> Result<String, Diagnostic> {
    let expression = fields.get(name).expect("required external field");
    let Expression::Identifier(value) = expression else {
        return Err(unsupported(
            source,
            expression.span(),
            &format!("static `{name}` attribute"),
        ));
    };
    Ok(source.slice(*value).to_owned())
}

fn external_nat_field(
    source: &SourceText,
    fields: &BTreeMap<String, &Expression>,
    name: &str,
) -> Result<BigInt, Diagnostic> {
    let expression = fields.get(name).expect("required external field");
    let Expression::Integer(value) = expression else {
        return Err(unsupported(
            source,
            expression.span(),
            &format!("static Nat `{name}` attribute"),
        ));
    };
    let value = parse_integer(source.slice(*value))
        .filter(|value| value >= &BigInt::from(0_u8))
        .ok_or_else(|| {
            unsupported(
                source,
                expression.span(),
                &format!("Nat `{name}` attribute"),
            )
        })?;
    Ok(value)
}

fn external_size_field(
    source: &SourceText,
    fields: &BTreeMap<String, &Expression>,
    name: &str,
) -> Result<u64, Diagnostic> {
    let expression = fields.get(name).expect("required external field");
    let Expression::Measured { value, unit, .. } = expression else {
        return Err(unsupported(
            source,
            expression.span(),
            &format!("exact storage size `{name}`"),
        ));
    };
    let count = parse_integer(source.slice(*value))
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| unsupported(source, *value, &format!("finite storage size `{name}`")))?;
    match source.slice(*unit) {
        "b" => Ok(count),
        "B" => count
            .checked_mul(8)
            .ok_or_else(|| unsupported(source, expression.span(), "finite storage size")),
        _ => Err(unsupported(
            source,
            *unit,
            "storage size outside bits or bytes",
        )),
    }
}

fn external_inclusive_range(
    source: &SourceText,
    expression: &Expression,
) -> Result<(BigInt, BigInt), Diagnostic> {
    let Expression::Application { items, .. } = expression else {
        return Err(unsupported(
            source,
            expression.span(),
            "AddressRange bounds",
        ));
    };
    let [
        Expression::Integer(lower),
        Expression::Callable {
            kind: CallableKind::RangeInclusive,
            ..
        },
        Expression::Integer(upper),
    ] = items.as_slice()
    else {
        return Err(unsupported(
            source,
            expression.span(),
            "inclusive AddressRange bounds",
        ));
    };
    let lower = parse_integer(source.slice(*lower))
        .ok_or_else(|| unsupported(source, *lower, "AddressRange lower bound"))?;
    let upper = parse_integer(source.slice(*upper))
        .ok_or_else(|| unsupported(source, *upper, "AddressRange upper bound"))?;
    Ok((lower, upper))
}

fn validate_compiler_location(
    source: &SourceText,
    span: Span,
    location: &CompilerLocationType,
    offset: &CompilerAddressOffset,
) -> Result<(), Diagnostic> {
    let layout = &location.layout;
    let range = &offset.offset_type.range;
    let bytes = layout.storage_size_bits.div_ceil(8);
    let extent = &range.upper - &range.lower + BigInt::from(1_u8);
    if &offset.offset + BigInt::from(bytes) > extent {
        return Err(source_diagnostic(
            source,
            "E-LOCATION-RANGE",
            span,
            "layout does not fit in the associated address range",
        ));
    }
    let physical_access_bits = range.range_type.minimum_access_size_bits;
    let physical_alignment_bytes = physical_access_bits.div_ceil(8);
    let absolute_address = &range.lower + &offset.offset;
    if !offset
        .offset_type
        .alignment_bytes
        .is_multiple_of(layout.alignment_bytes)
        || physical_access_bits == 0
        || !physical_access_bits.is_multiple_of(8)
        || layout.storage_size_bits < physical_access_bits
        || !layout
            .storage_size_bits
            .is_multiple_of(physical_access_bits)
        || offset.offset_type.alignment_bytes < physical_alignment_bytes
        || &absolute_address % layout.alignment_bytes != BigInt::from(0_u8)
        || &absolute_address % physical_alignment_bytes != BigInt::from(0_u8)
        || range.range_type.medium != "MMIO"
        || range.range_type.caching != "Uncached"
    {
        return Err(unsupported(
            source,
            span,
            "closed aligned uncached MMIO location",
        ));
    }
    Ok(())
}

fn compiler_type_is_static_only(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Identity
            | CompilerType::TypeView
            | CompilerType::FunctionView
            | CompilerType::LanguageContext
            | CompilerType::Capability
            | CompilerType::NativeSerializer(_)
            | CompilerType::ExternalMetadata
    )
}

fn compiler_type_contains_static_only(value_type: &CompilerType) -> bool {
    compiler_type_is_static_only(value_type)
        || match value_type {
            CompilerType::Range(value)
            | CompilerType::Result(value)
            | CompilerType::TaskResponse(value)
            | CompilerType::Optional(value)
            | CompilerType::List(value)
            | CompilerType::SerializationStream(value)
            | CompilerType::TraversalControl(value)
            | CompilerType::Refined { base: value, .. } => {
                compiler_type_contains_static_only(value)
            }
            CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_static_only),
            CompilerType::Record(fields) => fields
                .iter()
                .any(|(_, field)| compiler_type_contains_static_only(field)),
            CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
                alternative
                    .payload
                    .as_ref()
                    .is_some_and(compiler_type_contains_static_only)
            }),
            _ => false,
        }
}

fn compiler_type_contains_external_location(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::ExternalLocation(_) => true,
        CompilerType::Range(value)
        | CompilerType::Result(value)
        | CompilerType::TaskResponse(value)
        | CompilerType::Optional(value)
        | CompilerType::List(value)
        | CompilerType::SerializationStream(value)
        | CompilerType::TraversalControl(value)
        | CompilerType::Refined { base: value, .. } => {
            compiler_type_contains_external_location(value)
        }
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_external_location),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, field)| compiler_type_contains_external_location(field)),
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_contains_external_location)
        }),
        _ => false,
    }
}

fn compiler_type_contains_generator(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Generator(_) => true,
        CompilerType::Range(value)
        | CompilerType::Result(value)
        | CompilerType::TaskResponse(value)
        | CompilerType::Optional(value)
        | CompilerType::List(value)
        | CompilerType::TraversalControl(value)
        | CompilerType::Refined { base: value, .. } => compiler_type_contains_generator(value),
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_generator),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, field)| compiler_type_contains_generator(field)),
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_contains_generator)
        }),
        _ => false,
    }
}

fn anonymous_body_capture(
    source: &SourceText,
    parameters: &[AnonymousPattern],
    body: &Expression,
    environment: &BTreeMap<String, BindingFacts>,
) -> Option<String> {
    let mut parameter_names = BTreeSet::new();
    for parameter in parameters {
        collect_anonymous_pattern_names(source, parameter, &mut parameter_names);
    }
    environment
        .keys()
        .find(|name| {
            !parameter_names.contains(name.as_str())
                && expression_mentions_environment_binding(source, body, name)
        })
        .cloned()
}

fn expression_mentions_environment_binding(
    source: &SourceText,
    expression: &Expression,
    name: &str,
) -> bool {
    if let Some(member_name) = name.strip_prefix("@ ") {
        expression_context_member_span(source, expression, member_name).is_some()
    } else if let Some(member_name) = name.strip_prefix("root ") {
        expression_root_member_span(source, expression, member_name).is_some()
    } else {
        expression_mentions_name(source, expression, name)
    }
}

fn collect_anonymous_pattern_names<'a>(
    source: &'a SourceText,
    pattern: &AnonymousPattern,
    names: &mut BTreeSet<&'a str>,
) {
    match pattern {
        AnonymousPattern::Binding(binding) => {
            names.insert(source.slice(*binding));
        }
        AnonymousPattern::Product { fields, .. } => {
            for field in fields {
                collect_anonymous_pattern_names(source, field, names);
            }
        }
    }
}

fn anonymous_pattern_contains_product(pattern: &AnonymousPattern) -> bool {
    match pattern {
        AnonymousPattern::Binding(_) => false,
        AnonymousPattern::Product { .. } => true,
    }
}

fn directly_collectable_list_uncons_unfold(
    generator: &CompilerExpression,
    environment: &BTreeMap<String, BindingFacts>,
) -> bool {
    let CompilerExpressionKind::UnfoldGenerator {
        seed,
        parameters,
        step,
    } = &generator.kind
    else {
        return false;
    };
    let [parameter] = parameters.as_slice() else {
        return false;
    };
    let CompilerExpressionKind::ListUncons(argument) = &step.result.kind else {
        return false;
    };
    let CompilerExpressionKind::Local(seed) = &seed.kind else {
        return false;
    };
    step.statements.is_empty()
        && binding_facts_by_storage(environment, seed).is_some()
        && matches!(&argument.kind, CompilerExpressionKind::Local(name) if name == &parameter.name)
}

type AnonymousBodyRef<'a> = (&'a [AnonymousPattern], &'a Expression, Span);

fn direct_bounded_iterate_functions<'a>(
    source: &SourceText,
    expression: &'a Expression,
) -> Option<(AnonymousBodyRef<'a>, AnonymousBodyRef<'a>)> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    if let [
        _,
        Expression::Identifier(iterate),
        Expression::AnonymousFunction {
            parameters: next_parameters,
            body: next_body,
            span: next_span,
        },
        Expression::Identifier(take_while),
        Expression::AnonymousFunction {
            parameters: predicate_parameters,
            body: predicate_body,
            span: predicate_span,
        },
    ] = items.as_slice()
        && source.slice(*iterate) == "iterate"
        && source.slice(*take_while) == "take-while"
    {
        return Some((
            (next_parameters, next_body, *next_span),
            (predicate_parameters, predicate_body, *predicate_span),
        ));
    }
    let [iterate, Expression::Identifier(take_while), predicate] = items.as_slice() else {
        return None;
    };
    let Expression::AnonymousFunction {
        parameters: predicate_parameters,
        body: predicate_body,
        span: predicate_span,
    } = predicate
    else {
        return None;
    };
    let Expression::Application { items, .. } = iterate else {
        return None;
    };
    let [
        _,
        Expression::Identifier(iterate),
        Expression::AnonymousFunction {
            parameters: next_parameters,
            body: next_body,
            span: next_span,
        },
    ] = items.as_slice()
    else {
        return None;
    };
    (source.slice(*iterate) == "iterate" && source.slice(*take_while) == "take-while").then_some((
        (next_parameters, next_body, *next_span),
        (predicate_parameters, predicate_body, *predicate_span),
    ))
}

fn reject_static_value_containment(
    source: &SourceText,
    value: &CompilerExpression,
) -> Result<(), Diagnostic> {
    if compiler_type_contains_static_only(&value.value_type)
        && !compiler_type_is_static_only(&value.value_type)
    {
        Err(unsupported(
            source,
            value.span,
            "containment of a static compiler value",
        ))
    } else {
        Ok(())
    }
}

fn compiler_function_result_supported(value_type: &CompilerType) -> bool {
    if matches!(
        value_type,
        CompilerType::Scope
            | CompilerType::Function
            | CompilerType::Capability
            | CompilerType::Constraint
            | CompilerType::Version
            | CompilerType::Refined { .. }
    ) {
        return false;
    }
    if value_type.machine_scalar() {
        return compiler_abi_type_supported(value_type);
    }
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.iter().all(|field| {
                field == &CompilerType::Function || compiler_function_result_supported(field)
            })
    ) || matches!(
        value_type,
        CompilerType::Record(fields)
            if fields
                .iter()
                .all(|(_, field)| {
                    field == &CompilerType::Function || compiler_function_result_supported(field)
                })
    ) || matches!(
        value_type,
        CompilerType::Sum(sum)
            if sum.alternatives.iter().all(|alternative| alternative
                .payload
                .as_ref()
                .is_none_or(|payload| {
                    payload == &CompilerType::Function
                        || compiler_function_result_supported(payload)
                }))
    )
}

fn compiler_environment_capture_supported(value_type: &CompilerType) -> bool {
    compiler_function_result_supported(value_type) && !compiler_type_contains_function(value_type)
}

fn compiler_type_contains_function(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Function => true,
        CompilerType::SerializationStream(value)
        | CompilerType::Range(value)
        | CompilerType::Result(value)
        | CompilerType::TaskResponse(value)
        | CompilerType::Optional(value)
        | CompilerType::List(value)
        | CompilerType::Set(value)
        | CompilerType::Bag(value)
        | CompilerType::TraversalControl(value)
        | CompilerType::Refined { base: value, .. } => compiler_type_contains_function(value),
        CompilerType::Array { element, .. } => compiler_type_contains_function(element),
        CompilerType::Map { key, value } => {
            compiler_type_contains_function(key) || compiler_type_contains_function(value)
        }
        CompilerType::Generator(generator) => {
            compiler_type_contains_function(&generator.yield_type)
                || compiler_type_contains_function(&generator.resume_type)
                || compiler_type_contains_function(&generator.result_type)
        }
        CompilerType::Task(task) => {
            compiler_type_contains_function(&task.state_type)
                || task.handlers.iter().any(|handler| {
                    compiler_type_contains_function(&handler.payload_type)
                        || compiler_type_contains_function(&handler.response_type)
                        || handler.stream_type.as_ref().is_some_and(|stream| {
                            compiler_type_contains_function(&stream.yield_type)
                                || compiler_type_contains_function(&stream.resume_type)
                                || compiler_type_contains_function(&stream.result_type)
                        })
                })
        }
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_function),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, field)| compiler_type_contains_function(field)),
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_contains_function)
        }),
        _ => false,
    }
}

fn compiler_type_is_function_aggregate(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Tuple(fields) => fields.iter().any(|field| {
            field == &CompilerType::Function || compiler_type_is_function_aggregate(field)
        }),
        CompilerType::Record(fields) => fields.iter().any(|(_, field)| {
            field == &CompilerType::Function || compiler_type_is_function_aggregate(field)
        }),
        CompilerType::List(element) | CompilerType::Array { element, .. } => {
            element.as_ref() == &CompilerType::Function
        }
        CompilerType::Map { key, value } => {
            key.as_ref() == &CompilerType::String && value.as_ref() == &CompilerType::Function
        }
        CompilerType::Optional(payload) => payload.as_ref() == &CompilerType::Function,
        CompilerType::Result(success) => success.as_ref() == &CompilerType::Function,
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_is_sum_function_aggregate)
        }),
        _ => false,
    }
}

fn compiler_type_is_sum_function_aggregate(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Function => true,
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_is_sum_function_aggregate),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, field)| compiler_type_is_sum_function_aggregate(field)),
        CompilerType::Optional(payload) => payload.as_ref() == &CompilerType::Function,
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_is_sum_function_aggregate)
        }),
        // Result Function is admitted directly, not inside a nominal Sum.
        _ => false,
    }
}

fn function_aggregate_facts_exact(value_type: &CompilerType, facts: &StaticValueFacts) -> bool {
    match value_type {
        CompilerType::Function => facts.callable.is_some(),
        CompilerType::Tuple(fields) => {
            fields.len() == facts.tuple_fields.len()
                && fields
                    .iter()
                    .zip(&facts.tuple_fields)
                    .all(|(field, facts)| function_aggregate_facts_exact(field, facts))
        }
        CompilerType::Record(fields) => {
            fields.len() == facts.record_fields.len()
                && fields.iter().all(|(name, field)| {
                    facts
                        .record_fields
                        .get(name)
                        .is_some_and(|facts| function_aggregate_facts_exact(field, facts))
                })
        }
        CompilerType::List(element) if element.as_ref() == &CompilerType::Function => {
            facts.list_entries.as_ref().is_some_and(|entries| {
                entries
                    .iter()
                    .all(|facts| function_aggregate_facts_exact(element, facts))
            })
        }
        CompilerType::Array { count, element } if element.as_ref() == &CompilerType::Function => {
            facts.array_entries.as_ref().is_some_and(|entries| {
                entries.len() == *count
                    && entries
                        .iter()
                        .all(|facts| function_aggregate_facts_exact(element, facts))
            })
        }
        CompilerType::Map { key, value }
            if key.as_ref() == &CompilerType::String
                && value.as_ref() == &CompilerType::Function =>
        {
            facts.map_entries.as_ref().is_some_and(|entries| {
                entries
                    .iter()
                    .all(|(_, facts)| function_aggregate_facts_exact(value.as_ref(), facts))
            })
        }
        CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Function => {
            match facts.optional.as_ref() {
                Some(CompilerOptionalFacts::None) => true,
                Some(CompilerOptionalFacts::Some(payload_facts)) => {
                    function_aggregate_facts_exact(payload, payload_facts)
                }
                None => false,
            }
        }
        CompilerType::Sum(sum) => facts.sum.as_ref().is_some_and(|sum_facts| {
            let Some(alternative) = usize::try_from(sum_facts.alternative)
                .ok()
                .and_then(|index| sum.alternatives.get(index))
            else {
                return false;
            };
            if alternative.name != sum_facts.alternative_name {
                return false;
            }
            match (&alternative.payload, sum_facts.payload.as_deref()) {
                (Some(payload), Some(payload_facts)) => {
                    function_aggregate_facts_exact(payload, payload_facts)
                }
                (None, None) => true,
                _ => false,
            }
        }),
        CompilerType::Result(success) if success.as_ref() == &CompilerType::Function => facts
            .result
            .as_ref()
            .is_some_and(|result| function_aggregate_facts_exact(success, &result.success)),
        _ => true,
    }
}

fn same_callable_identity(left: &CompilerCallableFacts, right: &CompilerCallableFacts) -> bool {
    match (left, right) {
        (
            CompilerCallableFacts::Named {
                name: left_name,
                declarations: left_declarations,
                ..
            },
            CompilerCallableFacts::Named {
                name: right_name,
                declarations: right_declarations,
                ..
            },
        ) => {
            left_name == right_name
                && left_declarations
                    .iter()
                    .map(|declaration| declaration.span)
                    .eq(right_declarations
                        .iter()
                        .map(|declaration| declaration.span))
        }
        (CompilerCallableFacts::Symbolic(left), CompilerCallableFacts::Symbolic(right)) => {
            left == right
        }
        (
            CompilerCallableFacts::Anonymous {
                span: left_span, ..
            },
            CompilerCallableFacts::Anonymous {
                span: right_span, ..
            },
        ) => left_span == right_span,
        _ => false,
    }
}

fn function_aggregate_facts_same_callable_identity(
    value_type: &CompilerType,
    left: &StaticValueFacts,
    right: &StaticValueFacts,
) -> bool {
    match value_type {
        CompilerType::Function => left
            .callable
            .as_ref()
            .zip(right.callable.as_ref())
            .is_some_and(|(left, right)| same_callable_identity(left, right)),
        CompilerType::Tuple(fields) => {
            fields.len() == left.tuple_fields.len()
                && fields.len() == right.tuple_fields.len()
                && fields
                    .iter()
                    .zip(&left.tuple_fields)
                    .zip(&right.tuple_fields)
                    .all(|((field, left), right)| {
                        function_aggregate_facts_same_callable_identity(field, left, right)
                    })
        }
        CompilerType::Record(fields) => {
            fields.len() == left.record_fields.len()
                && fields.len() == right.record_fields.len()
                && fields.iter().all(|(name, field)| {
                    left.record_fields
                        .get(name)
                        .zip(right.record_fields.get(name))
                        .is_some_and(|(left, right)| {
                            function_aggregate_facts_same_callable_identity(field, left, right)
                        })
                })
        }
        CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Function => {
            match (left.optional.as_ref(), right.optional.as_ref()) {
                (Some(CompilerOptionalFacts::None), Some(CompilerOptionalFacts::None)) => true,
                (
                    Some(CompilerOptionalFacts::Some(left)),
                    Some(CompilerOptionalFacts::Some(right)),
                ) => function_aggregate_facts_same_callable_identity(payload, left, right),
                _ => false,
            }
        }
        CompilerType::Sum(sum) => match (left.sum.as_ref(), right.sum.as_ref()) {
            (Some(left), Some(right)) if left.alternative == right.alternative => {
                let Some(alternative) = usize::try_from(left.alternative)
                    .ok()
                    .and_then(|index| sum.alternatives.get(index))
                else {
                    return false;
                };
                if alternative.name != left.alternative_name
                    || alternative.name != right.alternative_name
                {
                    return false;
                }
                match (
                    &alternative.payload,
                    left.payload.as_deref(),
                    right.payload.as_deref(),
                ) {
                    (Some(payload), Some(left), Some(right)) => {
                        function_aggregate_facts_same_callable_identity(payload, left, right)
                    }
                    (None, None, None) => true,
                    _ => false,
                }
            }
            _ => false,
        },
        _ => true,
    }
}

fn function_aggregate_binding_facts_exact(value_type: &CompilerType, facts: &BindingFacts) -> bool {
    match value_type {
        CompilerType::Tuple(fields) => {
            fields.len() == facts.tuple_fields.len()
                && fields
                    .iter()
                    .zip(&facts.tuple_fields)
                    .all(|(field, facts)| function_aggregate_facts_exact(field, facts))
        }
        CompilerType::Record(fields) => {
            fields.len() == facts.record_fields.len()
                && fields.iter().all(|(name, field)| {
                    facts
                        .record_fields
                        .get(name)
                        .is_some_and(|facts| function_aggregate_facts_exact(field, facts))
                })
        }
        CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Function => {
            match facts.optional.as_ref() {
                Some(CompilerOptionalFacts::None) => true,
                Some(CompilerOptionalFacts::Some(payload_facts)) => {
                    function_aggregate_facts_exact(payload, payload_facts)
                }
                None => false,
            }
        }
        CompilerType::Sum(sum) => facts.sum.as_ref().is_some_and(|sum_facts| {
            let Some(alternative) = usize::try_from(sum_facts.alternative)
                .ok()
                .and_then(|index| sum.alternatives.get(index))
            else {
                return false;
            };
            if alternative.name != sum_facts.alternative_name {
                return false;
            }
            match (&alternative.payload, sum_facts.payload.as_deref()) {
                (Some(payload), Some(payload_facts)) => {
                    function_aggregate_facts_exact(payload, payload_facts)
                }
                (None, None) => true,
                _ => false,
            }
        }),
        CompilerType::Result(success) if success.as_ref() == &CompilerType::Function => facts
            .result
            .as_ref()
            .is_some_and(|result| function_aggregate_facts_exact(success, &result.success)),
        CompilerType::Map { key, value }
            if key.as_ref() == &CompilerType::String
                && value.as_ref() == &CompilerType::Function =>
        {
            facts.map_entries.as_ref().is_some_and(|entries| {
                entries
                    .iter()
                    .all(|(_, facts)| function_aggregate_facts_exact(value.as_ref(), facts))
            })
        }
        _ => false,
    }
}

fn function_aggregate_binding_facts_same_callable_identity(
    value_type: &CompilerType,
    left: &BindingFacts,
    right: &StaticValueFacts,
) -> bool {
    match value_type {
        CompilerType::Tuple(fields) => {
            fields.len() == left.tuple_fields.len()
                && fields.len() == right.tuple_fields.len()
                && fields
                    .iter()
                    .zip(&left.tuple_fields)
                    .zip(&right.tuple_fields)
                    .all(|((field, left), right)| {
                        function_aggregate_facts_same_callable_identity(field, left, right)
                    })
        }
        CompilerType::Record(fields) => {
            fields.len() == left.record_fields.len()
                && fields.len() == right.record_fields.len()
                && fields.iter().all(|(name, field)| {
                    left.record_fields
                        .get(name)
                        .zip(right.record_fields.get(name))
                        .is_some_and(|(left, right)| {
                            function_aggregate_facts_same_callable_identity(field, left, right)
                        })
                })
        }
        CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Function => {
            match (left.optional.as_ref(), right.optional.as_ref()) {
                (Some(CompilerOptionalFacts::None), Some(CompilerOptionalFacts::None)) => true,
                (
                    Some(CompilerOptionalFacts::Some(left)),
                    Some(CompilerOptionalFacts::Some(right)),
                ) => function_aggregate_facts_same_callable_identity(payload, left, right),
                _ => false,
            }
        }
        CompilerType::Sum(sum) => match (left.sum.as_ref(), right.sum.as_ref()) {
            (Some(left), Some(right)) if left.alternative == right.alternative => {
                let Some(alternative) = usize::try_from(left.alternative)
                    .ok()
                    .and_then(|index| sum.alternatives.get(index))
                else {
                    return false;
                };
                if alternative.name != left.alternative_name
                    || alternative.name != right.alternative_name
                {
                    return false;
                }
                match (
                    &alternative.payload,
                    left.payload.as_deref(),
                    right.payload.as_deref(),
                ) {
                    (Some(payload), Some(left), Some(right)) => {
                        function_aggregate_facts_same_callable_identity(payload, left, right)
                    }
                    (None, None, None) => true,
                    _ => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

fn named_callable_capture_binding(capture: &CompilerContextCapture) -> Option<BindingFacts> {
    let (CompilerExpressionKind::Local(storage_name)
    | CompilerExpressionKind::InfinityLocal { storage_name, .. }) = &capture.argument.kind
    else {
        return None;
    };
    is_function_result_capture_storage(storage_name).then(|| BindingFacts {
        storage_name: storage_name.clone(),
        origin: capture.span.start,
        runtime_bound: true,
        value_type: capture.value_type.clone(),
        int_range: capture.int_range.clone(),
        rational_value: capture.rational_value.clone(),
        infinity_negative: None,
        string_value: None,
        string_characters: None,
        closed_int_range: None,
        list_count: None,
        list_string_keys: None,
        list_string_characters: None,
        list_entries: None,
        array_entries: None,
        map_entries: None,
        tuple_fields: Vec::new(),
        record_fields: BTreeMap::new(),
        optional: None,
        sum: None,
        result: None,
        namespace: None,
        callable: None,
        static_capability: None,
    })
}

fn returned_function_capture_bindings(facts: &StaticValueFacts) -> Vec<BindingFacts> {
    let mut returned = Vec::new();
    match &facts.callable {
        Some(CompilerCallableFacts::Anonymous { captures, .. }) => {
            returned.extend(
                captures
                    .values()
                    .filter(|capture| is_function_result_capture_storage(&capture.storage_name))
                    .cloned(),
            );
        }
        Some(CompilerCallableFacts::Named { captures, .. }) => {
            returned.extend(captures.iter().filter_map(named_callable_capture_binding));
        }
        Some(CompilerCallableFacts::Symbolic(_)) | None => {}
    }
    for field in &facts.tuple_fields {
        returned.extend(returned_function_capture_bindings(field));
    }
    for field in facts.record_fields.values() {
        returned.extend(returned_function_capture_bindings(field));
    }
    if let Some(entries) = &facts.list_entries {
        for entry in entries {
            returned.extend(returned_function_capture_bindings(entry));
        }
    }
    if let Some(entries) = &facts.array_entries {
        for entry in entries {
            returned.extend(returned_function_capture_bindings(entry));
        }
    }
    if let Some(entries) = &facts.map_entries {
        for (_, entry) in entries {
            returned.extend(returned_function_capture_bindings(entry));
        }
    }
    if let Some(CompilerOptionalFacts::Some(payload)) = &facts.optional {
        returned.extend(returned_function_capture_bindings(payload));
    }
    if let Some(CompilerSumFacts {
        payload: Some(payload),
        ..
    }) = &facts.sum
    {
        returned.extend(returned_function_capture_bindings(payload));
    }
    if let Some(result) = &facts.result {
        returned.extend(returned_function_capture_bindings(&result.success));
    }
    returned
}

fn aggregate_callable_at_path_mut<'a>(
    facts: &'a mut StaticValueFacts,
    path: &[CompilerAggregatePathElement],
) -> Option<&'a mut CompilerCallableFacts> {
    let Some((first, rest)) = path.split_first() else {
        return facts.callable.as_mut();
    };
    match first {
        CompilerAggregatePathElement::Tuple(index) => {
            aggregate_callable_at_path_mut(facts.tuple_fields.get_mut(*index)?, rest)
        }
        CompilerAggregatePathElement::Record(name) => {
            aggregate_callable_at_path_mut(facts.record_fields.get_mut(name)?, rest)
        }
        CompilerAggregatePathElement::ListEntry(index) => {
            aggregate_callable_at_path_mut(facts.list_entries.as_mut()?.get_mut(*index)?, rest)
        }
        CompilerAggregatePathElement::ArrayEntry(index) => {
            aggregate_callable_at_path_mut(facts.array_entries.as_mut()?.get_mut(*index)?, rest)
        }
        CompilerAggregatePathElement::MapValue(key) => {
            let (_, value) = facts
                .map_entries
                .as_mut()?
                .iter_mut()
                .find(|(candidate, _)| candidate == key)?;
            aggregate_callable_at_path_mut(value, rest)
        }
        CompilerAggregatePathElement::OptionalPayload => {
            let CompilerOptionalFacts::Some(payload) = facts.optional.as_mut()? else {
                return None;
            };
            aggregate_callable_at_path_mut(payload, rest)
        }
        CompilerAggregatePathElement::SumPayload(name) => {
            let sum = facts.sum.as_mut()?;
            if &sum.alternative_name != name {
                return None;
            }
            aggregate_callable_at_path_mut(sum.payload.as_mut()?, rest)
        }
        CompilerAggregatePathElement::ResultSuccess => {
            aggregate_callable_at_path_mut(&mut facts.result.as_mut()?.success, rest)
        }
    }
}

fn remap_returned_callable_capture(
    callable: &mut CompilerCallableFacts,
    result_capture: &CompilerFunctionResultCapture,
    storage_name: String,
    span: Span,
) {
    let remapped = match callable {
        CompilerCallableFacts::Anonymous { captures, .. } => captures
            .get_mut(&result_capture.name)
            .is_some_and(|capture| {
                capture.storage_name.clone_from(&storage_name);
                capture.origin = span.start;
                true
            }),
        CompilerCallableFacts::Named { captures, .. } => captures
            .iter_mut()
            .find(|capture| capture.parameter_name == result_capture.name)
            .is_some_and(|capture| {
                capture.argument = CompilerExpression {
                    kind: CompilerExpressionKind::Local(storage_name),
                    value_type: result_capture.value_type.clone(),
                    int_range: None,
                    rational_value: None,
                    span,
                };
                true
            }),
        CompilerCallableFacts::Symbolic(_) => false,
    };
    debug_assert!(remapped, "checked result capture retains its callable path");
}

fn remap_returned_callable_captures(
    callable: &mut CompilerCallableFacts,
    result_captures: &[CompilerFunctionResultCapture],
    symbol: &str,
    span: Span,
) {
    let callable_capture_count = match callable {
        CompilerCallableFacts::Anonymous { captures, .. } => captures.len(),
        CompilerCallableFacts::Named { captures, .. } => captures.len(),
        CompilerCallableFacts::Symbolic(_) => 0,
    };
    if callable_capture_count == 0 {
        return;
    }
    debug_assert_eq!(
        callable_capture_count,
        result_captures
            .iter()
            .filter(|capture| capture.path.is_empty())
            .count()
    );
    for (index, result_capture) in result_captures.iter().enumerate() {
        if result_capture.path.is_empty() {
            remap_returned_callable_capture(
                callable,
                result_capture,
                compiler_function_result_capture_storage(symbol, span, index),
                span,
            );
        }
    }
}

fn remap_returned_aggregate_captures(
    facts: &mut StaticValueFacts,
    result_captures: &[CompilerFunctionResultCapture],
    symbol: &str,
    span: Span,
) {
    for (index, result_capture) in result_captures.iter().enumerate() {
        let callable = aggregate_callable_at_path_mut(facts, &result_capture.path)
            .expect("checked Function aggregate result retains its callable path");
        remap_returned_callable_capture(
            callable,
            result_capture,
            compiler_function_result_capture_storage(symbol, span, index),
            span,
        );
    }
}

fn compiler_function_parameter_supported(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Scope | CompilerType::Function)
        || is_admitted_function_generator_type(value_type)
        || compiler_function_result_supported(value_type)
}

fn compiler_packaged_field_supported(value_type: &CompilerType) -> bool {
    compiler_function_parameter_supported(value_type)
        && (value_type.machine_scalar()
            || matches!(
                value_type,
                CompilerType::Tuple(_) | CompilerType::Record(_) | CompilerType::Sum(_)
            ))
}

fn compiler_repeated_pattern_identity_supported(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Unit
            | CompilerType::Completed
            | CompilerType::Effect
            | CompilerType::Type
            | CompilerType::Function
            | CompilerType::Boolean
            | CompilerType::Int
            | CompilerType::Nat
            | CompilerType::Modular(_)
            | CompilerType::Rational
            | CompilerType::Comparison
            | CompilerType::ErrorCode
            | CompilerType::Enum(_)
            | CompilerType::Character
            | CompilerType::String
    ) || (matches!(
        value_type,
        CompilerType::Tuple(_)
            | CompilerType::Record(_)
            | CompilerType::Optional(_)
            | CompilerType::List(_)
            | CompilerType::Sum(_)
    ) && (compiler_repeated_structural_identity_supported(value_type)
        || compiler_repeated_function_aggregate_identity_supported(value_type)))
}

fn compiler_repeated_structural_identity_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Tuple(fields) => fields
            .iter()
            .all(compiler_repeated_structural_identity_supported),
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, field)| compiler_repeated_structural_identity_supported(field)),
        CompilerType::Sum(sum) => sum.alternatives.iter().all(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_none_or(compiler_repeated_structural_identity_supported)
        }),
        _ => compiler_equality_supported(value_type),
    }
}

fn compiler_repeated_function_aggregate_identity_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Function => true,
        CompilerType::Tuple(fields) => fields
            .iter()
            .all(compiler_repeated_function_aggregate_identity_supported),
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, field)| compiler_repeated_function_aggregate_identity_supported(field)),
        CompilerType::Optional(payload) => {
            compiler_repeated_function_aggregate_identity_supported(payload)
        }
        CompilerType::Sum(sum) => sum.alternatives.iter().all(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_none_or(compiler_repeated_function_aggregate_identity_supported)
        }),
        _ => compiler_equality_supported(value_type),
    }
}

fn validate_repeated_anonymous_pattern_parameter(
    source: &SourceText,
    parameters: &[CompilerParameter],
    name: &str,
    discarded: bool,
    value_type: &CompilerType,
    span: Span,
) -> Result<Option<usize>, Diagnostic> {
    let Some((first_parameter, first)) = (!discarded)
        .then(|| {
            parameters
                .iter()
                .enumerate()
                .find(|(_, parameter)| !parameter.discarded && parameter.name == name)
        })
        .flatten()
    else {
        return Ok(None);
    };
    if first.value_type != *value_type {
        return Err(source_diagnostic(
            source,
            "E-ANONYMOUS-PATTERN-IDENTITY-CLASSIFIER",
            span,
            format!(
                "repeated pattern name `{name}` requires `{}`, found `{}`",
                first.value_type.name(),
                value_type.name()
            ),
        ));
    }
    if !compiler_repeated_pattern_identity_supported(value_type) {
        return Err(unsupported(
            source,
            span,
            &format!(
                "repeated anonymous pattern identity for `{}`",
                value_type.name()
            ),
        ));
    }
    Ok(Some(first_parameter))
}

fn append_repeated_capture_identities(
    source: &SourceText,
    span: Span,
    source_parameter_count: usize,
    (first_start, first_end): (usize, usize),
    (repeated_start, repeated_end): (usize, usize),
    captures: &[CompilerContextCapture],
    identities: &mut Vec<CompilerPatternIdentity>,
) -> Result<(), Diagnostic> {
    if first_end - first_start != repeated_end - repeated_start {
        return Err(unsupported(
            source,
            span,
            "repeated captured Function identity with inconsistent capture schema",
        ));
    }
    for (first_capture, repeated_capture) in
        (first_start..first_end).zip(repeated_start..repeated_end)
    {
        let first = &captures[first_capture];
        let repeated = &captures[repeated_capture];
        if first.value_type != repeated.value_type
            || !compiler_equality_supported(&first.value_type)
        {
            return Err(unsupported(
                source,
                span,
                "repeated captured Function identity without exact capture equality",
            ));
        }
        identities.push(CompilerPatternIdentity {
            first_parameter: source_parameter_count + first_capture,
            repeated_parameter: source_parameter_count + repeated_capture,
            span,
        });
    }
    Ok(())
}

fn data_member_expression(facts: &CompilerDataMemberFacts, span: Span) -> CompilerExpression {
    CompilerExpression {
        kind: facts.infinity_negative.map_or_else(
            || CompilerExpressionKind::Local(facts.storage_name.clone()),
            |negative| CompilerExpressionKind::InfinityLocal {
                storage_name: facts.storage_name.clone(),
                negative,
            },
        ),
        value_type: facts.value_type.clone(),
        int_range: facts.int_range.clone(),
        rational_value: facts.rational_value.clone(),
        span,
    }
}

fn binding_facts_by_storage<'a>(
    environment: &'a BTreeMap<String, BindingFacts>,
    storage_name: &str,
) -> Option<&'a BindingFacts> {
    environment
        .values()
        .find(|facts| facts.storage_name == storage_name)
}

fn decision_binding_environment(
    environment: &BTreeMap<String, BindingFacts>,
    name: &str,
    value_type: CompilerType,
    origin: usize,
) -> BTreeMap<String, BindingFacts> {
    let mut branch = environment.clone();
    branch.insert(
        name.to_owned(),
        BindingFacts {
            storage_name: name.to_owned(),
            origin,
            runtime_bound: true,
            value_type,
            int_range: None,
            rational_value: None,
            infinity_negative: None,
            string_value: None,
            string_characters: None,
            closed_int_range: None,
            list_count: None,
            list_string_keys: None,
            list_string_characters: None,
            list_entries: None,
            array_entries: None,
            map_entries: None,
            tuple_fields: Vec::new(),
            record_fields: BTreeMap::new(),
            optional: None,
            sum: None,
            result: None,
            namespace: None,
            callable: None,
            static_capability: None,
        },
    );
    branch
}

fn arithmetic_error_code(namespace: &str, vocabulary: &str, code: &str) -> Option<u32> {
    if namespace != "lang" || vocabulary != "arithmetic" {
        return None;
    }
    match code {
        "out-of-range" => Some(0),
        "not-representable" => Some(1),
        "division-by-zero" => Some(2),
        "indeterminate" => Some(3),
        _ => None,
    }
}

fn generator_error_code(
    namespace: &str,
    vocabulary: &str,
    code: &str,
) -> Option<(CompilerEnumType, u32)> {
    if namespace != "lang" || vocabulary != "generator" || code != "generator-closed" {
        return None;
    }
    Some((
        CompilerEnumType {
            name: "lang generator GeneratorErrorCode".to_owned(),
            alternatives: vec!["generator-closed".to_owned()],
        },
        0,
    ))
}

fn int_unit_generator_type() -> CompilerType {
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(CompilerType::Int),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(CompilerType::Unit),
    })
}

fn value_boundary_generator_type() -> CompilerType {
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(CompilerType::Int),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(CompilerType::String),
    })
}

fn product_boundary_generator_type() -> CompilerType {
    let product = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]);
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(product.clone()),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(product),
    })
}

fn nested_optional_product_type() -> CompilerType {
    CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
        CompilerType::Int,
        CompilerType::String,
    ])))
}

fn nested_boundary_generator_type() -> CompilerType {
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(nested_optional_product_type()),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(nested_result_product_type()),
    })
}

fn int_list_type() -> CompilerType {
    CompilerType::List(Box::new(CompilerType::Int))
}

fn list_boundary_generator_type() -> CompilerType {
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(int_list_type()),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(int_list_type()),
    })
}

fn character_unit_generator_type() -> CompilerType {
    character_generator_type(CompilerType::Unit)
}

fn character_generator_type(result_type: CompilerType) -> CompilerType {
    CompilerType::Generator(CompilerGeneratorType {
        yield_type: Box::new(CompilerType::Character),
        resume_type: Box::new(CompilerType::Unit),
        result_type: Box::new(result_type),
    })
}

fn is_character_unit_generator_type(value_type: &CompilerType) -> bool {
    is_character_generator_type_with_result(value_type, &CompilerType::Unit)
}

fn is_admitted_character_generator_type(value_type: &CompilerType) -> bool {
    is_character_unit_generator_type(value_type)
        || is_character_generator_type_with_result(value_type, &CompilerType::Character)
}

fn is_admitted_value_boundary_generator_type(value_type: &CompilerType) -> bool {
    value_type == &int_unit_generator_type()
        || matches!(
            value_type,
            CompilerType::Generator(CompilerGeneratorType {
                yield_type,
                resume_type,
                result_type,
            }) if yield_type.as_ref() == &CompilerType::Int
                && resume_type.as_ref() == &CompilerType::Unit
                && result_type.as_ref() == &CompilerType::String
        )
        || value_type == &product_boundary_generator_type()
        || value_type == &nested_boundary_generator_type()
        || value_type == &list_boundary_generator_type()
}

fn is_admitted_function_generator_type(value_type: &CompilerType) -> bool {
    is_admitted_character_generator_type(value_type)
        || is_admitted_value_boundary_generator_type(value_type)
}

fn is_admitted_function_value_result_type(value_type: &CompilerType) -> bool {
    value_type == &CompilerType::String
        || compiler_int_string_pair(value_type)
        || compiler_int_list_pair(value_type)
        || value_type == &nested_result_product_type()
        || value_type == &int_list_type()
}

fn is_character_generator_type_with_result(
    value_type: &CompilerType,
    expected_result: &CompilerType,
) -> bool {
    matches!(
        value_type,
        CompilerType::Generator(CompilerGeneratorType {
            yield_type,
            resume_type,
            result_type,
        }) if yield_type.as_ref() == &CompilerType::Character
            && resume_type.as_ref() == &CompilerType::Unit
            && result_type.as_ref() == expected_result
    )
}

fn comparison_binary(kind: CallableKind) -> Option<CompilerBinary> {
    match kind {
        CallableKind::Equal => Some(CompilerBinary::Equal),
        CallableKind::NotEqual => Some(CompilerBinary::NotEqual),
        CallableKind::Less => Some(CompilerBinary::Less),
        CallableKind::Greater => Some(CompilerBinary::Greater),
        CallableKind::LessEqual => Some(CompilerBinary::LessEqual),
        CallableKind::GreaterEqual => Some(CompilerBinary::GreaterEqual),
        _ => None,
    }
}

fn is_exact_numeric(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Int
            | CompilerType::InfiniteInt
            | CompilerType::InfiniteNat
            | CompilerType::InfiniteRational
            | CompilerType::Rational
    )
}

fn is_int_range_endpoint(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Int | CompilerType::InfiniteInt)
}

fn is_rational_range_endpoint(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Rational | CompilerType::InfiniteRational
    )
}

fn forget_refined_evidence(mut expression: CompilerExpression) -> CompilerExpression {
    let CompilerType::Refined { base, .. } = expression.value_type else {
        return expression;
    };
    expression.value_type = *base;
    expression
}
