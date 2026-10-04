fn supported_generator_value_classifier(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    matches!(
        classifier,
        "Unit"
            | "Boolean"
            | "Character"
            | "Comparison"
            | "Constraint"
            | "Effect"
            | "Error"
            | "Int"
            | "MessageContext"
            | "Nat"
            | "Rational"
            | "Scope"
            | "String"
            | "Range Int"
            | "Range Rational"
    ) || enum_types.contains_key(classifier)
        || optional_payload_classifier(classifier)
            .is_some_and(|payload| supported_generator_value_classifier(payload, enum_types))
        || list_element_classifier(classifier)
            .is_some_and(|element| supported_generator_value_classifier(element, enum_types))
        || tuple_classifiers(classifier).is_some_and(|items| {
            items
                .into_iter()
                .all(|item| supported_generator_value_classifier(item, enum_types))
        })
        || result_success_classifier(classifier)
            .is_some_and(|success| supported_generator_value_classifier(success, enum_types))
}

fn is_arithmetic_error_code(code: &str) -> bool {
    matches!(
        code,
        "out-of-range" | "not-representable" | "division-by-zero" | "indeterminate"
    )
}

fn error_code_classifier(code: &str) -> &'static str {
    if is_arithmetic_error_code(code) {
        "lang arithmetic ArithmeticErrorCode"
    } else {
        "lang generator GeneratorErrorCode"
    }
}

fn result_classifier_parts(classifier: &str) -> Option<(&str, &str)> {
    let contents = classifier
        .trim()
        .strip_prefix("Result")?
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let comma = top_level_comma(contents)?;
    Some((contents[..comma].trim(), contents[comma + 1..].trim()))
}

fn result_success_classifier(classifier: &str) -> Option<&str> {
    let (success, errors) = result_classifier_parts(classifier)?;
    let errors = errors.split_whitespace().collect::<Vec<_>>().join(" ");
    matches!(
        errors.as_str(),
        "lang arithmetic ArithmeticErrorCode" | "()"
    )
    .then_some(success)
}

fn optional_payload_classifier(classifier: &str) -> Option<&str> {
    classifier.trim().strip_prefix("Optional ").map(str::trim)
}

fn list_element_classifier(classifier: &str) -> Option<&str> {
    classifier.trim().strip_prefix("List ").map(str::trim)
}

fn tuple_classifiers(classifier: &str) -> Option<Vec<&str>> {
    let contents = classifier.trim().strip_prefix('(')?.strip_suffix(')')?;
    let mut classifiers = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0_usize;
    for (offset, character) in contents.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                classifiers.push(contents[start..offset].trim());
                start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    classifiers.push(contents[start..].trim());
    (classifiers.len() > 1 && classifiers.iter().all(|item| !item.is_empty()))
        .then_some(classifiers)
}

fn top_level_comma(text: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => return Some(offset),
            _ => {}
        }
    }
    None
}

const fn function_rule(is_static: bool, parameter_count: usize) -> &'static str {
    if !is_static {
        return "TOPAL-FUNCTION-ORDINARY-001";
    }
    match parameter_count {
        0 => "TOPAL-FUNCTION-STATIC-NULLARY-001",
        1 => "TOPAL-FUNCTION-STATIC-UNARY-001",
        _ => "TOPAL-FUNCTION-STATIC-BINARY-001",
    }
}

fn function_accepts(parameters: &[(String, String)], argument: &Value) -> bool {
    match parameters {
        [] => matches!(argument, Value::Unit),
        [(_, classifier)] => value_has_classifier(argument, classifier),
        parameters => {
            let Value::Tuple(arguments) = argument else {
                return false;
            };
            arguments.len() == parameters.len()
                && parameters
                    .iter()
                    .zip(arguments)
                    .all(|((_, classifier), argument)| value_has_classifier(argument, classifier))
        }
    }
}

fn user_function_accepts(function: &UserFunction, argument: &Value) -> bool {
    let arguments = match function.parameters.as_slice() {
        [] => return matches!(argument, Value::Unit),
        [_] => std::slice::from_ref(argument),
        parameters => {
            let Value::Tuple(arguments) = argument else {
                return false;
            };
            if arguments.len() != parameters.len() {
                return false;
            }
            arguments
        }
    };
    let mut generic_types = BTreeMap::new();
    function.parameters.iter().enumerate().zip(arguments).all(
        |((index, (_, classifier)), argument)| {
            if let Some(fields) = function.parameter_packages.get(&index) {
                package_generic_accepts(fields, argument, &mut generic_types)
            } else {
                generic_parameter_accepts(argument, classifier, &mut generic_types)
                    || value_matches_substituted_classifier(
                        argument,
                        classifier,
                        &function.generic_names,
                        &mut generic_types,
                    )
            }
        },
    )
}

fn generic_capability_classifier(classifier: &str) -> Option<(&str, &str)> {
    let contents = classifier.trim().strip_prefix('(')?.strip_suffix(')')?;
    let (name, capability) = contents.split_once(':')?;
    let name = name.trim();
    let capability = capability.trim();
    (!name.is_empty() && !capability.is_empty()).then_some((name, capability))
}

fn generic_parameter_accepts(
    argument: &Value,
    classifier: &str,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if let Some((name, capability)) = generic_capability_classifier(classifier) {
        if !value_has_capability(argument, capability) {
            return false;
        }
        if let Some(existing) = generic_types.get(name) {
            return value_has_classifier(argument, existing)
                || structural_value_classifier(argument) == *existing;
        }
        let actual = structural_value_classifier(argument);
        generic_types.insert(name.to_owned(), actual);
        return true;
    }
    if let Some(expected) = generic_types.get(classifier) {
        return value_has_classifier(argument, expected)
            || structural_value_classifier(argument) == *expected;
    }
    if let Some(payload_classifier) = applied_classifier(classifier, "Optional") {
        let Value::Optional {
            payload_classifier: actual,
            ..
        } = argument
        else {
            return false;
        };
        return generic_classifier_accepts_name(actual, payload_classifier, generic_types);
    }
    if let Some(element) = applied_classifier(classifier, "List") {
        let Value::List {
            element_classifier, ..
        } = argument
        else {
            return false;
        };
        return generic_classifier_accepts_name(element_classifier, element, generic_types);
    }
    if let Some(endpoint) = applied_classifier(classifier, "Range") {
        let actual = match argument {
            Value::IntRange { .. } => "Int",
            Value::RationalRange { .. } | Value::InfiniteRationalRange { .. } => "Rational",
            _ => return false,
        };
        return generic_classifier_accepts_name(actual, endpoint, generic_types);
    }
    if let Some((success, codes)) = result_classifier_parts(classifier) {
        if let Value::Error { code, .. } = argument {
            return generic_classifier_accepts_name(
                error_code_classifier(code),
                codes,
                generic_types,
            );
        }
        return generic_classifier_accepts_name(
            &structural_value_classifier(argument),
            success,
            generic_types,
        );
    }
    if function_classifier_parts(classifier).is_some() {
        return value_classifier(argument) == "Function";
    }
    value_has_classifier(argument, classifier)
}

fn generic_classifier_accepts_name(
    actual: &str,
    expected: &str,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if let Some((name, _)) = generic_capability_classifier(expected) {
        return generic_types
            .insert(name.to_owned(), actual.to_owned())
            .is_none_or(|existing| existing == actual);
    }
    if let Some(bound) = generic_types.get(expected) {
        return bound == actual;
    }
    if let (Some(actual_payload), Some(expected_payload)) = (
        applied_classifier(actual, "Optional"),
        applied_classifier(expected, "Optional"),
    ) {
        return generic_classifier_accepts_name(actual_payload, expected_payload, generic_types);
    }
    if let (Some(actual_element), Some(expected_element)) = (
        applied_classifier(actual, "List"),
        applied_classifier(expected, "List"),
    ) {
        return generic_classifier_accepts_name(actual_element, expected_element, generic_types);
    }
    if let (Some(actual_endpoint), Some(expected_endpoint)) = (
        applied_classifier(actual, "Range"),
        applied_classifier(expected, "Range"),
    ) {
        return generic_classifier_accepts_name(actual_endpoint, expected_endpoint, generic_types);
    }
    if let (Some(actual_payload), Some(expected_payload)) = (
        result_classifier_parts(actual),
        result_classifier_parts(expected),
    ) {
        return generic_classifier_accepts_name(
            actual_payload.0,
            expected_payload.0,
            generic_types,
        ) && generic_classifier_accepts_name(
            actual_payload.1,
            expected_payload.1,
            generic_types,
        );
    }
    if let (Some(actual_items), Some(expected_items)) =
        (tuple_classifiers(actual), tuple_classifiers(expected))
    {
        return actual_items.len() == expected_items.len()
            && actual_items
                .into_iter()
                .zip(expected_items)
                .all(|(actual, expected)| {
                    generic_classifier_accepts_name(actual, expected, generic_types)
                });
    }
    actual == substitute_classifier(expected, generic_types)
}

fn substitute_classifier(classifier: &str, generic_types: &BTreeMap<String, String>) -> String {
    if let Some(concrete) = generic_types.get(classifier) {
        return concrete.clone();
    }
    for constructor in ["Optional", "List", "Range"] {
        if let Some(payload) = applied_classifier(classifier, constructor) {
            return format!(
                "{constructor} {}",
                substitute_classifier(payload, generic_types)
            );
        }
    }
    if let Some((success, codes)) = result_classifier_parts(classifier) {
        return format!(
            "Result {} {}",
            substitute_classifier(success, generic_types),
            substitute_classifier(codes, generic_types)
        );
    }
    if let Some(items) = tuple_classifiers(classifier) {
        return format!(
            "({})",
            items
                .into_iter()
                .map(|item| substitute_classifier(item, generic_types))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    classifier.to_owned()
}

fn value_has_capability(value: &Value, capability: &str) -> bool {
    match capability {
        "Type" => true,
        "Equality" => values_equal(value.clone(), value.clone(), &mut Vec::new()).is_some(),
        "PartialOrder" | "TotalOrder" | "Ordering" => {
            values_compare(value.clone(), value.clone(), &mut Vec::new()).is_some()
        }
        _ => false,
    }
}

fn generic_result_accepts(function: &UserFunction, scope: &Session, value: &Value) -> bool {
    let mut generic_types = BTreeMap::new();
    populate_function_generics(function, scope, &mut generic_types);
    value_matches_substituted_classifier(
        value,
        &function.result,
        &function.generic_names,
        &mut generic_types,
    )
}

fn populate_function_generics(
    function: &UserFunction,
    scope: &Session,
    generic_types: &mut BTreeMap<String, String>,
) {
    for (index, (parameter, classifier)) in function.parameters.iter().enumerate() {
        if let Some(fields) = function.parameter_packages.get(&index) {
            for field in fields {
                if let Some(argument) = scope.bindings.get(&field.name) {
                    let _ = generic_parameter_accepts(argument, &field.classifier, generic_types);
                }
            }
        } else if let Some(argument) = scope.bindings.get(parameter) {
            if !generic_parameter_accepts(argument, classifier, generic_types) {
                let _ = value_matches_substituted_classifier(
                    argument,
                    classifier,
                    &function.generic_names,
                    generic_types,
                );
            }
            if let (Some((_, expected_result)), Value::NamedFunction(named_function)) =
                (function_classifier_parts(classifier), argument)
                && named_function.candidates.len() == 1
            {
                let _ = bind_named_classifier_generics(
                    &named_function.candidates[0].result,
                    expected_result,
                    &function.generic_names,
                    generic_types,
                );
            }
        }
    }
}

fn generic_capability_value_matches(
    value: &Value,
    classifier: &str,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    let Some((name, capability)) = generic_capability_classifier(classifier) else {
        return false;
    };
    if !value_has_capability(value, capability) {
        return false;
    }
    if let Some(existing) = generic_types.get(name) {
        return value_has_classifier(value, existing)
            || structural_value_classifier(value) == *existing;
    }
    let actual = structural_value_classifier(value);
    generic_types.insert(name.to_owned(), actual);
    true
}

fn generic_classifier_accepts_declared_name(
    actual: &str,
    expected: &str,
    generic_names: &BTreeSet<String>,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if let Some((name, _)) = generic_capability_classifier(expected) {
        if let Some(existing) = generic_types.get(name) {
            return existing == actual;
        }
        generic_types.insert(name.to_owned(), actual.to_owned());
        return true;
    }
    if generic_names.contains(expected) {
        if let Some(existing) = generic_types.get(expected) {
            return existing == actual;
        }
        generic_types.insert(expected.to_owned(), actual.to_owned());
        return true;
    }
    for constructor in ["Optional", "List", "Range"] {
        if let (Some(actual_payload), Some(expected_payload)) = (
            applied_classifier(actual, constructor),
            applied_classifier(expected, constructor),
        ) {
            return generic_classifier_accepts_declared_name(
                actual_payload,
                expected_payload,
                generic_names,
                generic_types,
            );
        }
    }
    if let (Some(actual_payload), Some(expected_payload)) = (
        result_classifier_parts(actual),
        result_classifier_parts(expected),
    ) {
        return generic_classifier_accepts_declared_name(
            actual_payload.0,
            expected_payload.0,
            generic_names,
            generic_types,
        ) && generic_classifier_accepts_declared_name(
            actual_payload.1,
            expected_payload.1,
            generic_names,
            generic_types,
        );
    }
    if let (Some(actual_items), Some(expected_items)) =
        (tuple_classifiers(actual), tuple_classifiers(expected))
    {
        return actual_items.len() == expected_items.len()
            && actual_items
                .into_iter()
                .zip(expected_items)
                .all(|(actual, expected)| {
                    generic_classifier_accepts_declared_name(
                        actual,
                        expected,
                        generic_names,
                        generic_types,
                    )
                });
    }
    generic_classifier_accepts_name(actual, expected, generic_types)
}

fn value_matches_substituted_classifier(
    value: &Value,
    classifier: &str,
    generic_names: &BTreeSet<String>,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if let Some(expected) = generic_types.get(classifier) {
        return value_has_classifier(value, expected)
            || structural_value_classifier(value) == *expected;
    }
    if generic_capability_classifier(classifier).is_some() {
        return generic_capability_value_matches(value, classifier, generic_types);
    }
    if generic_names.contains(classifier) {
        generic_types.insert(classifier.to_owned(), structural_value_classifier(value));
        return true;
    }
    if let Some(expected_payload) = applied_classifier(classifier, "Optional") {
        let Value::Optional {
            payload_classifier, ..
        } = value
        else {
            return false;
        };
        if generic_names.contains(expected_payload) && !generic_types.contains_key(expected_payload)
        {
            generic_types.insert(expected_payload.to_owned(), payload_classifier.clone());
            return true;
        }
        return generic_classifier_accepts_declared_name(
            payload_classifier,
            expected_payload,
            generic_names,
            generic_types,
        );
    }
    if let Some(element) = applied_classifier(classifier, "List") {
        let Value::List {
            element_classifier, ..
        } = value
        else {
            return false;
        };
        return generic_classifier_accepts_declared_name(
            element_classifier,
            element,
            generic_names,
            generic_types,
        );
    }
    if let Some(endpoint) = applied_classifier(classifier, "Range") {
        let actual = match value {
            Value::IntRange { .. } => "Int",
            Value::RationalRange { .. } | Value::InfiniteRationalRange { .. } => "Rational",
            _ => return false,
        };
        return generic_classifier_accepts_declared_name(
            actual,
            endpoint,
            generic_names,
            generic_types,
        );
    }
    if let Some((success, codes)) = result_classifier_parts(classifier) {
        if let Value::Error { code, .. } = value {
            if generic_names.contains(codes) && !generic_types.contains_key(codes) {
                generic_types.insert(codes.to_owned(), error_code_classifier(code).to_owned());
                return true;
            }
            return generic_classifier_accepts_declared_name(
                error_code_classifier(code),
                codes,
                generic_names,
                generic_types,
            );
        }
        return value_matches_substituted_classifier(value, success, generic_names, generic_types);
    }
    if let (Value::Tuple(values), Some(classifiers)) = (value, tuple_classifiers(classifier)) {
        return values.len() == classifiers.len()
            && values.iter().zip(classifiers).all(|(value, classifier)| {
                value_matches_substituted_classifier(
                    value,
                    classifier,
                    generic_names,
                    generic_types,
                )
            });
    }
    value_has_classifier(value, classifier)
}

fn supported_generic_classifier(
    classifier: &str,
    generic_names: &BTreeSet<String>,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
    modular_names: &BTreeSet<String>,
) -> bool {
    supported_value_classifier(classifier, enum_types)
        || modular_names.contains(classifier)
        || generic_capability_classifier(classifier).is_some()
        || generic_names.contains(classifier)
        || applied_classifier(classifier, "Optional").is_some_and(|payload| {
            supported_generic_classifier(payload, generic_names, enum_types, modular_names)
                || generic_capability_classifier(payload).is_some()
        })
        || applied_classifier(classifier, "List").is_some_and(|element| {
            supported_generic_classifier(element, generic_names, enum_types, modular_names)
                || generic_capability_classifier(element).is_some()
        })
        || applied_classifier(classifier, "Range").is_some_and(|endpoint| {
            supported_generic_classifier(endpoint, generic_names, enum_types, modular_names)
                || generic_capability_classifier(endpoint).is_some()
        })
        || result_success_classifier(classifier).is_some_and(|success| {
            supported_generic_classifier(success, generic_names, enum_types, modular_names)
                || generic_capability_classifier(success).is_some()
        })
        || result_classifier_parts(classifier).is_some_and(|(success, codes)| {
            (supported_generic_classifier(success, generic_names, enum_types, modular_names)
                || generic_capability_classifier(success).is_some())
                && (supported_generic_classifier(codes, generic_names, enum_types, modular_names)
                    || generic_capability_classifier(codes).is_some())
        })
        || function_classifier_parts(classifier).is_some_and(|(input, result)| {
            supported_generic_classifier(input, generic_names, enum_types, modular_names)
                && supported_generic_classifier(result, generic_names, enum_types, modular_names)
        })
        || tuple_classifiers(classifier).is_some_and(|items| {
            items.into_iter().all(|item| {
                supported_generic_classifier(item, generic_names, enum_types, modular_names)
            })
        })
}

fn applied_classifier<'a>(classifier: &'a str, constructor: &str) -> Option<&'a str> {
    let payload = classifier
        .trim()
        .strip_prefix(constructor)?
        .strip_prefix(' ')
        .map(str::trim)
        .filter(|payload| !payload.is_empty())?;
    Some(strip_classifier_group(payload))
}

fn strip_classifier_group(classifier: &str) -> &str {
    let Some(inner) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    else {
        return classifier;
    };
    let mut depth = 0_i32;
    for character in inner.chars() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' | ':' if depth == 0 => return classifier,
            _ => {}
        }
    }
    if depth == 0 { inner.trim() } else { classifier }
}

fn function_classifier_parts(classifier: &str) -> Option<(&str, &str)> {
    let classifier = classifier
        .trim()
        .strip_prefix("static ")
        .unwrap_or(classifier.trim());
    let signature = classifier.strip_prefix("fn (")?;
    let boundary = signature.rfind(") -> ")?;
    Some((
        signature[..boundary].trim(),
        signature[boundary + 5..].trim(),
    ))
}

fn collect_generic_names(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
    modular_names: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    if let Some((name, _)) = generic_capability_classifier(classifier) {
        names.insert(name.to_owned());
        return;
    }
    if let Some(payload) = applied_classifier(classifier, "Optional") {
        collect_generic_names(payload, enum_types, modular_names, names);
        return;
    }
    if let Some(element) = applied_classifier(classifier, "List") {
        collect_generic_names(element, enum_types, modular_names, names);
        return;
    }
    if let Some(endpoint) = applied_classifier(classifier, "Range") {
        collect_generic_names(endpoint, enum_types, modular_names, names);
        return;
    }
    if let Some((success, codes)) = result_classifier_parts(classifier) {
        collect_generic_names(success, enum_types, modular_names, names);
        collect_generic_names(codes, enum_types, modular_names, names);
        return;
    }
    if let Some((input, result)) = function_classifier_parts(classifier) {
        collect_generic_names(input, enum_types, modular_names, names);
        collect_function_result_generic_names(result, enum_types, modular_names, names);
        return;
    }
    if let Some(items) = tuple_classifiers(classifier) {
        for item in items {
            collect_generic_names(item, enum_types, modular_names, names);
        }
    }
}

fn collect_function_result_generic_names(
    classifier: &str,
    enum_types: &BTreeMap<String, BTreeSet<String>>,
    modular_names: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    if let Some(payload) = applied_classifier(classifier, "Optional") {
        collect_function_result_generic_names(payload, enum_types, modular_names, names);
    } else if let Some(element) = applied_classifier(classifier, "List") {
        collect_function_result_generic_names(element, enum_types, modular_names, names);
    } else if let Some(endpoint) = applied_classifier(classifier, "Range") {
        collect_function_result_generic_names(endpoint, enum_types, modular_names, names);
    } else if let Some((success, codes)) = result_classifier_parts(classifier) {
        collect_function_result_generic_names(success, enum_types, modular_names, names);
        collect_function_result_generic_names(codes, enum_types, modular_names, names);
    } else if let Some(items) = tuple_classifiers(classifier) {
        for item in items {
            collect_function_result_generic_names(item, enum_types, modular_names, names);
        }
    } else if !supported_value_classifier(classifier, enum_types)
        && !modular_names.contains(classifier)
        && classifier.chars().all(char::is_alphanumeric)
    {
        names.insert(classifier.to_owned());
    }
}

fn package_generic_accepts(
    fields: &[UserParameterField],
    argument: &Value,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    match argument {
        Value::Record(values) => {
            values
                .iter()
                .all(|(label, _)| fields.iter().any(|field| field.name == *label))
                && fields.iter().all(|field| {
                    values
                        .iter()
                        .find(|(label, _)| *label == field.name)
                        .map_or(field.default.is_some(), |(_, value)| {
                            generic_parameter_accepts(value, &field.classifier, generic_types)
                        })
                })
        }
        Value::Tuple(values) => {
            values.len() == fields.len()
                && fields.iter().zip(values).all(|(field, value)| {
                    generic_parameter_accepts(value, &field.classifier, generic_types)
                })
        }
        _ => false,
    }
}

fn bind_function_arguments(
    scope: &mut Session,
    function: &UserFunction,
    argument: Value,
    trace: &mut impl TraceSink,
    rule: &'static str,
) -> Result<(), Diagnostic> {
    let arguments = match (function.parameters.as_slice(), argument) {
        ([], Value::Unit) => return Ok(()),
        ([_], argument) => vec![argument],
        (_, Value::Tuple(arguments)) => arguments,
        _ => unreachable!("selected overload has already validated its argument"),
    };
    for (index, ((parameter, _), argument)) in function.parameters.iter().zip(arguments).enumerate()
    {
        if let Some(fields) = function.parameter_packages.get(&index) {
            bind_package_fields(scope, &function.source, fields, argument, trace, rule)?;
            continue;
        }
        if parameter == "_" {
            trace.record(TraceEvent {
                event: "function.argument.discarded",
                rule: "TOPAL-TYPE-MATCH-001",
                detail: "_",
            });
            continue;
        }
        scope.bindings.insert(parameter.clone(), argument);
        scope.declared_names.insert(parameter.clone());
        trace.record(TraceEvent {
            event: "function.argument.bound",
            rule,
            detail: parameter,
        });
    }
    Ok(())
}

fn bind_named_classifier_generics(
    actual: &str,
    expected: &str,
    generic_names: &BTreeSet<String>,
    generic_types: &mut BTreeMap<String, String>,
) -> bool {
    if generic_names.contains(expected) {
        return generic_types
            .insert(expected.to_owned(), actual.to_owned())
            .is_none_or(|existing| existing == actual);
    }
    for constructor in ["Optional", "List", "Range"] {
        if let (Some(actual_inner), Some(expected_inner)) = (
            applied_classifier(actual, constructor),
            applied_classifier(expected, constructor),
        ) {
            return bind_named_classifier_generics(
                actual_inner,
                expected_inner,
                generic_names,
                generic_types,
            );
        }
    }
    generic_classifier_accepts_name(actual, expected, generic_types)
}

fn bind_package_fields(
    scope: &mut Session,
    source: &SourceText,
    fields: &[UserParameterField],
    argument: Value,
    trace: &mut impl TraceSink,
    rule: &'static str,
) -> Result<(), Diagnostic> {
    let supplied = match argument {
        Value::Record(values) => values.into_iter().collect::<BTreeMap<_, _>>(),
        Value::Tuple(values) => fields
            .iter()
            .map(|field| field.name.clone())
            .zip(values)
            .collect(),
        _ => unreachable!("selected package has validated its argument"),
    };
    for field in fields {
        let value = if let Some(value) = supplied.get(&field.name) {
            value.clone()
        } else {
            let default = field
                .default
                .as_ref()
                .expect("selected package requires a supplied field or default");
            let value = scope.evaluate_expression(source, default, trace)?;
            trace.record(TraceEvent {
                event: "function.argument.defaulted",
                rule: "TOPAL-FUNCTION-PACKAGED-OPERAND-001",
                detail: &field.name,
            });
            value
        };
        if field.name == "_" {
            continue;
        }
        scope.bindings.insert(field.name.clone(), value);
        scope.declared_names.insert(field.name.clone());
        trace.record(TraceEvent {
            event: "function.argument.bound",
            rule,
            detail: &field.name,
        });
    }
    Ok(())
}

fn bind_generator_arguments(
    scope: &mut Session,
    parameters: &[(String, String)],
    argument: Value,
    trace: &mut impl TraceSink,
) {
    let arguments = match (parameters, argument) {
        ([_], argument) => vec![argument],
        (_, Value::Tuple(arguments)) => arguments,
        _ => unreachable!("selected generator overload has validated its argument"),
    };
    for ((parameter, _), argument) in parameters.iter().zip(arguments) {
        if parameter == "_" {
            trace.record(TraceEvent {
                event: "generator.argument.discarded",
                rule: "TOPAL-TYPE-MATCH-001",
                detail: "_",
            });
            continue;
        }
        scope.bindings.insert(parameter.clone(), argument);
        scope.declared_names.insert(parameter.clone());
        trace.record(TraceEvent {
            event: "generator.argument.bound",
            rule: "TOPAL-GENERATOR-OVERLOAD-001",
            detail: parameter,
        });
    }
}

fn no_applicable_generator(
    source: &SourceText,
    name: &str,
    argument_span: Span,
    argument: &Value,
    candidates: &[UserGenerator],
) -> Diagnostic {
    let found = structural_value_classifier(argument);
    let expected = candidates
        .iter()
        .map(|candidate| {
            candidate
                .parameters
                .iter()
                .map(|(_, classifier)| classifier.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect::<Vec<_>>()
        .join(" or ");
    diagnostic(
        source,
        "E-NO-APPLICABLE-GENERATOR",
        argument_span,
        format!("no `{name}` generator overload accepts `{found}`"),
    )
    .with_help(format!("available input classifiers: {expected}"))
}

fn no_applicable_overload(
    source: &SourceText,
    name: &str,
    argument_span: Span,
    argument: &Value,
    candidates: &[UserFunction],
    static_context: bool,
) -> Diagnostic {
    let eligible = candidates
        .iter()
        .filter(|function| !static_context || function.is_static)
        .collect::<Vec<_>>();
    if let [function] = eligible.as_slice() {
        match function.parameters.as_slice() {
            [] => {
                return diagnostic(
                    source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    argument_span,
                    format!("nullary function `{name}` requires ()"),
                );
            }
            [(parameter, classifier)] => {
                return diagnostic(
                    source,
                    "E-FUNCTION-ARGUMENT-TYPE",
                    argument_span,
                    format!("argument for `{parameter}` is outside `{classifier}`"),
                );
            }
            parameters => {
                let Value::Tuple(arguments) = argument else {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-SHAPE",
                        argument_span,
                        format!(
                            "function `{name}` requires a positional product with {} fields",
                            parameters.len()
                        ),
                    );
                };
                if arguments.len() != parameters.len() {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-ARITY",
                        argument_span,
                        format!(
                            "function `{name}` requires {} arguments but received {}",
                            parameters.len(),
                            arguments.len()
                        ),
                    );
                }
                if let Some(((parameter, classifier), _)) = parameters
                    .iter()
                    .zip(arguments)
                    .find(|((_, classifier), argument)| !value_has_classifier(argument, classifier))
                {
                    return diagnostic(
                        source,
                        "E-FUNCTION-ARGUMENT-TYPE",
                        argument_span,
                        format!("argument for `{parameter}` is outside `{classifier}`"),
                    );
                }
            }
        }
    }
    let signatures = eligible
        .iter()
        .map(|function| {
            let inputs = function
                .parameters
                .iter()
                .map(|(_, classifier)| classifier.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name} ({inputs})")
        })
        .collect::<Vec<_>>()
        .join(", ");
    diagnostic(
        source,
        "E-NO-APPLICABLE-OVERLOAD",
        argument_span,
        format!(
            "no overload of `{name}` accepts {} in this context",
            value_classifier(argument)
        ),
    )
    .with_help(format!("available overloads: {signatures}"))
}

fn function_signature(name: &str, function: &UserFunction) -> String {
    let inputs = function
        .parameters
        .iter()
        .map(|(_, classifier)| classifier.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let staticness = if function.is_static { " static" } else { "" };
    format!("{name}{staticness} ({inputs})")
}

fn validate_parameter_names(
    source: &SourceText,
    parameters: &[FunctionParameter],
) -> Result<(), Diagnostic> {
    let flattened = parameters
        .iter()
        .flat_map(|parameter| {
            if parameter.fields.is_empty() {
                std::slice::from_ref(parameter)
            } else {
                parameter.fields.as_slice()
            }
        })
        .collect::<Vec<_>>();
    for (index, parameter) in flattened.iter().enumerate() {
        let name = source.slice(parameter.name);
        if name == "_" {
            continue;
        }
        if flattened[..index]
            .iter()
            .any(|earlier| source.slice(earlier.name) == name)
        {
            return Err(diagnostic(
                source,
                "E-DUPLICATE-FUNCTION-PARAMETER",
                parameter.name,
                format!("parameter `{name}` is already declared in this function"),
            ));
        }
    }
    Ok(())
}

pub(super) fn prove_int_recursion(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    body: &[Statement],
) -> Option<&'static str> {
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
        ) if parse_integer(source.slice(*bound)).is_some_and(|value| value >= BigInt::from(0)) => {
            (CallableKind::Minus, "TOPAL-FUNCTION-RECURSION-NAT-001")
        }
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001",
        ),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (CallableKind::Minus, "TOPAL-FUNCTION-RECURSION-INT-001"),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (
            CallableKind::Plus,
            "TOPAL-FUNCTION-RECURSION-INT-INCREASING-001",
        ),
        _ => return None,
    };
    if !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_))
        || contains_self_call(source, function_name, &base.action)
    {
        return None;
    }
    let (found, valid) =
        bounded_self_calls(source, function_name, parameter, step, &recursive.action);
    let nat_step_limit = nat_decrement_step_limit(source, &base.matcher);
    let preserves_nat = classifier != "Nat"
        || step == CallableKind::Plus
        || nat_step_limit.is_some_and(|limit| {
            recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                &recursive.action,
                &limit,
            )
        });
    (found && valid && preserves_nat).then_some(proof_rule)
}

pub(super) fn prove_explicit_parameter_recursion(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    effect_bound: Option<&str>,
    body: &[Statement],
) -> Option<&'static str> {
    let measure = explicit_single_measure(effect_bound?)?;
    let measure_index = parameters.iter().position(|(name, classifier)| {
        name == measure && matches!(classifier.as_str(), "Int" | "Nat")
    })?;
    let (parameter, classifier) = &parameters[measure_index];
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
    let (step, proof_rule) = match (classifier.as_str(), &base.matcher) {
        (
            "Nat",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(bound),
                ..
            },
        ) if parse_integer(source.slice(*bound)).is_some_and(|value| value >= BigInt::from(0)) => {
            (CallableKind::Minus, "TOPAL-FUNCTION-DECREASES-001")
        }
        (
            "Nat" | "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::GreaterEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (CallableKind::Plus, "TOPAL-FUNCTION-DECREASES-001"),
        (
            "Int",
            DecisionMatcher::Comparison {
                kind: CallableKind::LessEqual,
                operand: Expression::Integer(_),
                ..
            },
        ) => (CallableKind::Minus, "TOPAL-FUNCTION-DECREASES-001"),
        _ => return None,
    };
    if !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_))
        || contains_self_call(source, function_name, &base.action)
    {
        return None;
    }
    let (found, valid) = measured_self_calls(
        source,
        function_name,
        parameter,
        measure_index,
        parameters.len(),
        step,
        &recursive.action,
    );
    let nat_step_limit = nat_decrement_step_limit(source, &base.matcher);
    let preserves_nat = classifier != "Nat"
        || step == CallableKind::Plus
        || nat_step_limit.is_some_and(|limit| {
            measured_recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                measure_index,
                parameters.len(),
                &recursive.action,
                &limit,
            )
        });
    (found && valid && preserves_nat).then_some(proof_rule)
}

pub(super) fn explicit_single_measure(effect_bound: &str) -> Option<&str> {
    let measure = effect_bound.trim().strip_prefix("Decreases")?.trim();
    let measure = measure
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(measure)
        .trim();
    (!measure.is_empty()
        && measure
            .chars()
            .all(|character| character == '-' || character.is_alphanumeric()))
    .then_some(measure)
}

pub(super) fn explicit_absolute_measure(effect_bound: &str) -> Option<&str> {
    let measure = effect_bound.trim().strip_prefix("Decreases")?.trim();
    let measure = measure
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(measure)
        .trim();
    let measure = measure.strip_prefix("absolute")?.trim();
    (!measure.is_empty()
        && measure
            .chars()
            .all(|character| character == '-' || character.is_alphanumeric()))
    .then_some(measure)
}

fn recursive_call_argument<'a>(
    source: &SourceText,
    function_name: &str,
    parameter_index: usize,
    parameter_count: usize,
    expression: &'a Expression,
) -> Option<&'a Expression> {
    let Expression::Application { items, .. } = expression else {
        return None;
    };
    if !matches!(items.first(), Some(Expression::Identifier(span)) if source.slice(*span) == function_name)
    {
        return None;
    }
    let argument = items.get(1)?;
    if parameter_count == 1 {
        return (items.len() == 2).then_some(argument);
    }
    let Expression::Product { fields, .. } = argument else {
        return None;
    };
    (items.len() == 2 && fields.len() == parameter_count).then(|| &fields[parameter_index].value)
}

fn measured_self_calls(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    parameter_index: usize,
    parameter_count: usize,
    step: CallableKind,
    expression: &Expression,
) -> (bool, bool) {
    if let Some(argument) = recursive_call_argument(
        source,
        function_name,
        parameter_index,
        parameter_count,
        expression,
    ) {
        return (
            true,
            is_positive_literal_step(source, parameter, step, argument),
        );
    }
    match expression {
        Expression::Application { items, .. } => combine_call_checks(items.iter().map(|item| {
            measured_self_calls(
                source,
                function_name,
                parameter,
                parameter_index,
                parameter_count,
                step,
                item,
            )
        })),
        Expression::Product { fields, .. } => combine_call_checks(fields.iter().map(|field| {
            measured_self_calls(
                source,
                function_name,
                parameter,
                parameter_index,
                parameter_count,
                step,
                &field.value,
            )
        })),
        Expression::DecisionTable { subject, rules, .. } => combine_call_checks(
            std::iter::once(measured_self_calls(
                source,
                function_name,
                parameter,
                parameter_index,
                parameter_count,
                step,
                subject,
            ))
            .chain(rules.iter().map(|rule| {
                measured_self_calls(
                    source,
                    function_name,
                    parameter,
                    parameter_index,
                    parameter_count,
                    step,
                    &rule.action,
                )
            })),
        ),
        _ => (false, true),
    }
}

fn measured_recursive_calls_fit_nat_bound(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    parameter_index: usize,
    parameter_count: usize,
    expression: &Expression,
    maximum_step: &BigInt,
) -> bool {
    if let Some(argument) = recursive_call_argument(
        source,
        function_name,
        parameter_index,
        parameter_count,
        expression,
    ) {
        return literal_step_value(source, parameter, CallableKind::Minus, argument)
            .is_some_and(|step| step <= *maximum_step);
    }
    match expression {
        Expression::Application { items, .. } => items.iter().all(|item| {
            measured_recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                parameter_index,
                parameter_count,
                item,
                maximum_step,
            )
        }),
        Expression::Product { fields, .. } => fields.iter().all(|field| {
            measured_recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                parameter_index,
                parameter_count,
                &field.value,
                maximum_step,
            )
        }),
        Expression::DecisionTable { subject, rules, .. } => {
            measured_recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                parameter_index,
                parameter_count,
                subject,
                maximum_step,
            ) && rules.iter().all(|rule| {
                measured_recursive_calls_fit_nat_bound(
                    source,
                    function_name,
                    parameter,
                    parameter_index,
                    parameter_count,
                    &rule.action,
                    maximum_step,
                )
            })
        }
        _ => true,
    }
}

pub(super) fn prove_euclidean_recursion(
    source: &SourceText,
    function_name: &str,
    parameters: &[(String, String)],
    effect_bound: Option<&str>,
    body: &[Statement],
) -> Option<&'static str> {
    let [(left, left_classifier), (right, right_classifier)] = parameters else {
        return None;
    };
    if left_classifier != "Int"
        || right_classifier != "Int"
        || effect_bound.and_then(explicit_absolute_measure) != Some(right)
    {
        return None;
    }
    let [Statement::Expression(Expression::DecisionTable { subject, rules, .. })] = body else {
        return None;
    };
    if !matches!(subject.as_ref(), Expression::Identifier(span) if source.slice(*span) == right) {
        return None;
    }
    let [base, recursive] = rules.as_slice() else {
        return None;
    };
    if !matches!(&base.matcher, DecisionMatcher::Comparison {
        kind: CallableKind::Equal,
        operand: Expression::Integer(zero),
        ..
    } if parse_integer(source.slice(*zero)).is_some_and(|value| value == BigInt::from(0)))
        || !matches!(&recursive.matcher, DecisionMatcher::Otherwise(_))
        || contains_self_call(source, function_name, &base.action)
    {
        return None;
    }
    let Expression::Application { items, .. } = &recursive.action else {
        return None;
    };
    let [
        Expression::Identifier(callee),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    let [first, second] = fields.as_slice() else {
        return None;
    };
    let modulo_is_measure_reducing = matches!(
        &second.value,
        Expression::Application { items, .. }
            if matches!(items.as_slice(), [
                Expression::Identifier(dividend),
                Expression::Callable { kind: CallableKind::Modulo, .. },
                Expression::Identifier(divisor)
            ] if source.slice(*dividend) == left && source.slice(*divisor) == right)
    );
    (source.slice(*callee) == function_name
        && matches!(&first.value, Expression::Identifier(span) if source.slice(*span) == right)
        && modulo_is_measure_reducing)
        .then_some("TOPAL-FUNCTION-RECURSION-EUCLIDEAN-001")
}

fn nat_decrement_step_limit(source: &SourceText, matcher: &DecisionMatcher) -> Option<BigInt> {
    let DecisionMatcher::Comparison {
        kind: CallableKind::LessEqual,
        operand: Expression::Integer(bound),
        ..
    } = matcher
    else {
        return None;
    };
    parse_integer(source.slice(*bound)).map(|bound| bound + BigInt::from(1))
}

fn recursive_calls_fit_nat_bound(
    source: &SourceText,
    function_name: &str,
    parameter: &str,
    expression: &Expression,
    maximum_step: &BigInt,
) -> bool {
    match expression {
        Expression::Application { items, .. } if matches!(items.first(), Some(Expression::Identifier(span)) if source.slice(*span) == function_name) =>
        {
            matches!(items.as_slice(), [_, Expression::Application { items, .. }]
                if matches!(items.as_slice(), [Expression::Identifier(name), Expression::Callable { kind: CallableKind::Minus, .. }, Expression::Integer(amount)]
                    if source.slice(*name) == parameter && parse_integer(source.slice(*amount)).is_some_and(|step| step > BigInt::from(0) && step <= *maximum_step)))
        }
        Expression::Application { items, .. } => items.iter().all(|item| {
            recursive_calls_fit_nat_bound(source, function_name, parameter, item, maximum_step)
        }),
        Expression::Product { fields, .. } => fields.iter().all(|field| {
            recursive_calls_fit_nat_bound(
                source,
                function_name,
                parameter,
                &field.value,
                maximum_step,
            )
        }),
        Expression::DecisionTable { subject, rules, .. } => {
            recursive_calls_fit_nat_bound(source, function_name, parameter, subject, maximum_step)
                && rules.iter().all(|rule| {
                    recursive_calls_fit_nat_bound(
                        source,
                        function_name,
                        parameter,
                        &rule.action,
                        maximum_step,
                    )
                })
        }
        _ => true,
    }
}
