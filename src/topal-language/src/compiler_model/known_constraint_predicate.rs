fn known_constraint_predicate(
    predicate: &CompilerExpression,
    parameter_storage: &str,
    argument: &CompilerExpression,
) -> Option<bool> {
    match &predicate.kind {
        CompilerExpressionKind::Boolean(value) => Some(*value),
        CompilerExpressionKind::Local(name) if name == parameter_storage => {
            let CompilerExpressionKind::Boolean(value) = &argument.kind else {
                return None;
            };
            Some(*value)
        }
        CompilerExpressionKind::Not(value) => Some(!known_constraint_predicate(
            value,
            parameter_storage,
            argument,
        )?),
        CompilerExpressionKind::StringEmptyPredicate(value) => {
            Some(known_constraint_string(value, parameter_storage, argument)?.is_empty())
        }
        CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual)
            && left.value_type == CompilerType::String
            && right.value_type == CompilerType::String =>
        {
            let equal = known_constraint_string(left, parameter_storage, argument)?
                == known_constraint_string(right, parameter_storage, argument)?;
            Some(if *operation == CompilerBinary::Equal {
                equal
            } else {
                !equal
            })
        }
        CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual)
            && left.value_type == CompilerType::Boolean
            && right.value_type == CompilerType::Boolean =>
        {
            let equal = known_constraint_boolean(left, parameter_storage, argument)?
                == known_constraint_boolean(right, parameter_storage, argument)?;
            Some(if *operation == CompilerBinary::Equal {
                equal
            } else {
                !equal
            })
        }
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::And,
            left,
            right,
        } => Some(
            known_constraint_predicate(left, parameter_storage, argument)?
                & known_constraint_predicate(right, parameter_storage, argument)?,
        ),
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Or,
            left,
            right,
        } => Some(
            known_constraint_predicate(left, parameter_storage, argument)?
                | known_constraint_predicate(right, parameter_storage, argument)?,
        ),
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Xor,
            left,
            right,
        } => Some(
            known_constraint_predicate(left, parameter_storage, argument)?
                ^ known_constraint_predicate(right, parameter_storage, argument)?,
        ),
        CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } if matches!(
            operation,
            CompilerBinary::Equal
                | CompilerBinary::NotEqual
                | CompilerBinary::Less
                | CompilerBinary::Greater
                | CompilerBinary::LessEqual
                | CompilerBinary::GreaterEqual
        ) =>
        {
            let left = known_constraint_numeric(left, parameter_storage, argument)?;
            let right = known_constraint_numeric(right, parameter_storage, argument)?;
            Some(match operation {
                CompilerBinary::Equal => left == right,
                CompilerBinary::NotEqual => left != right,
                CompilerBinary::Less => left < right,
                CompilerBinary::Greater => left > right,
                CompilerBinary::LessEqual => left <= right,
                CompilerBinary::GreaterEqual => left >= right,
                _ => unreachable!("guard selected a Boolean comparison"),
            })
        }
        _ => None,
    }
}

fn known_constraint_boolean(
    expression: &CompilerExpression,
    parameter_storage: &str,
    argument: &CompilerExpression,
) -> Option<bool> {
    match &expression.kind {
        CompilerExpressionKind::Boolean(value) => Some(*value),
        CompilerExpressionKind::Local(name) if name == parameter_storage => {
            let CompilerExpressionKind::Boolean(value) = &argument.kind else {
                return None;
            };
            Some(*value)
        }
        _ => None,
    }
}

fn known_constraint_string<'a>(
    expression: &'a CompilerExpression,
    parameter_storage: &str,
    argument: &'a CompilerExpression,
) -> Option<&'a str> {
    match &expression.kind {
        CompilerExpressionKind::String(value) => Some(value),
        CompilerExpressionKind::Local(name) if name == parameter_storage => {
            let CompilerExpressionKind::String(value) = &argument.kind else {
                return None;
            };
            Some(value)
        }
        _ => None,
    }
}

fn known_constraint_numeric(
    expression: &CompilerExpression,
    parameter_storage: &str,
    argument: &CompilerExpression,
) -> Option<BigRational> {
    match &expression.kind {
        CompilerExpressionKind::Local(name) if name == parameter_storage => argument
            .rational_value
            .clone()
            .or_else(|| exact_int(argument).map(BigRational::from_integer)),
        CompilerExpressionKind::Int(value) => Some(BigRational::from_integer(value.clone())),
        CompilerExpressionKind::Rational(value) => Some(value.clone()),
        CompilerExpressionKind::Negate(value) => Some(-known_constraint_numeric(
            value,
            parameter_storage,
            argument,
        )?),
        CompilerExpressionKind::Absolute(value) => Some(rational_absolute(
            &known_constraint_numeric(value, parameter_storage, argument)?,
        )),
        CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::RationalToInt(value)
        | CompilerExpressionKind::IntToNat(value)
        | CompilerExpressionKind::IntToNatBoundary(value) => {
            known_constraint_numeric(value, parameter_storage, argument)
        }
        CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } if matches!(
            operation,
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply
        ) =>
        {
            let left = known_constraint_numeric(left, parameter_storage, argument)?;
            let right = known_constraint_numeric(right, parameter_storage, argument)?;
            Some(match operation {
                CompilerBinary::Add => left + right,
                CompilerBinary::Subtract => left - right,
                CompilerBinary::Multiply => left * right,
                _ => unreachable!("guard selected exact arithmetic"),
            })
        }
        _ => None,
    }
}

fn is_exact_comparable(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Int
            | CompilerType::Nat
            | CompilerType::InfiniteInt
            | CompilerType::InfiniteNat
            | CompilerType::InfiniteRational
            | CompilerType::Rational
    )
}

fn forget_nat_evidence(mut expression: CompilerExpression) -> CompilerExpression {
    if expression.value_type == CompilerType::Nat {
        expression.value_type = CompilerType::Int;
    } else if expression.value_type == CompilerType::InfiniteNat {
        expression.value_type = CompilerType::InfiniteInt;
    }
    expression
}

fn compiler_equality_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Unit
        | CompilerType::Completed
        | CompilerType::Effect
        | CompilerType::Type
        | CompilerType::Boolean
        | CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat
        | CompilerType::InfiniteRational
        | CompilerType::Rational
        | CompilerType::Comparison
        | CompilerType::ErrorCode
        | CompilerType::Character
        | CompilerType::String
        | CompilerType::Modular(_)
        | CompilerType::Enum(_) => true,
        CompilerType::Optional(payload) => {
            matches!(
                payload.as_ref(),
                CompilerType::Int | CompilerType::Rational | CompilerType::String
            ) || compiler_nested_int_list_element(payload.as_ref())
                || matches!(payload.as_ref(), CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Int, CompilerType::String])
        }
        CompilerType::List(element) => {
            compiler_list_observation_element_supported(element.as_ref())
                || compiler_nested_int_list_element(element.as_ref())
                || compiler_nested_int_string_list_element(element.as_ref())
        }
        CompilerType::Tuple(fields) => fields.iter().all(compiler_equality_supported),
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, value_type)| compiler_equality_supported(value_type)),
        CompilerType::Sum(sum) => sum.alternatives.iter().all(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_none_or(compiler_equality_supported)
        }),
        CompilerType::Refined { base, .. } => compiler_equality_supported(base),
        CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata
        | CompilerType::SerializationStream(_)
        | CompilerType::Constraint
        | CompilerType::Version
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::SourceLocation
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::TaskResponse(_)
        | CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }
        | CompilerType::TraversalControl(_)
        | CompilerType::Generator(_)
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => false,
    }
}

fn compiler_ordering_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat
        | CompilerType::Rational
        | CompilerType::Modular(_) => true,
        CompilerType::Tuple(fields) => fields.iter().all(compiler_ordering_supported),
        _ => false,
    }
}

fn require_optional_payload(
    source: &SourceText,
    span: Span,
    value_type: &CompilerType,
) -> Result<(), Diagnostic> {
    if matches!(
        value_type,
        CompilerType::Int
            | CompilerType::Nat
            | CompilerType::Rational
            | CompilerType::String
            | CompilerType::Error
            | CompilerType::SourceLocation
            | CompilerType::Function
            | CompilerType::List(_)
    ) || compiler_integer_pair(value_type)
        || compiler_int_triple(value_type)
        || compiler_string_string_list_int_triple(value_type)
        || compiler_int_int_string_int_tuple(value_type)
        || compiler_int_list_pair(value_type)
        || compiler_list_string_rational_pair(value_type)
        || matches!(value_type, CompilerType::Tuple(fields)
        if fields.as_slice() == [CompilerType::Int, CompilerType::String]
            || matches!(fields.as_slice(), [CompilerType::Int, CompilerType::List(element)]
                if element.as_ref() == &CompilerType::Int))
    {
        Ok(())
    } else {
        Err(unsupported(source, span, "Optional payload type"))
    }
}

fn require_exact_numeric(
    source: &SourceText,
    span: Span,
    value_type: &CompilerType,
) -> Result<(), Diagnostic> {
    if is_exact_numeric(value_type) {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            span,
            format!("expected Int or Rational, found {}", value_type.name()),
        ))
    }
}

fn exact_int(expression: &CompilerExpression) -> Option<BigInt> {
    let range = expression.int_range.as_ref()?;
    (range.lower == range.upper).then(|| range.lower.clone())
}

fn numeric_local_storage(expression: &CompilerExpression) -> Option<&str> {
    match &expression.kind {
        CompilerExpressionKind::Local(name) => Some(name),
        CompilerExpressionKind::IntToNat(value)
        | CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::RationalToInt(value) => numeric_local_storage(value),
        _ => None,
    }
}

fn compiler_expression_nonnegative_with_locals(
    expression: &CompilerExpression,
    nonnegative: &BTreeSet<&str>,
) -> bool {
    if expression.value_type == CompilerType::Nat
        || expression
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= BigInt::from(0))
    {
        return true;
    }
    match &expression.kind {
        CompilerExpressionKind::Local(name) => nonnegative.contains(name.as_str()),
        CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add | CompilerBinary::Multiply,
            left,
            right,
        } => {
            compiler_expression_nonnegative_with_locals(left, nonnegative)
                && compiler_expression_nonnegative_with_locals(right, nonnegative)
        }
        CompilerExpressionKind::BooleanDecision {
            when_true,
            when_false,
            ..
        } => {
            compiler_expression_nonnegative_with_locals(when_true, nonnegative)
                && compiler_expression_nonnegative_with_locals(when_false, nonnegative)
        }
        CompilerExpressionKind::OrderedComparisonDecision {
            rules, otherwise, ..
        } => {
            rules
                .iter()
                .all(|rule| compiler_expression_nonnegative_with_locals(&rule.action, nonnegative))
                && compiler_expression_nonnegative_with_locals(otherwise, nonnegative)
        }
        _ => false,
    }
}

fn binding_expression(facts: &BindingFacts, span: Span) -> CompilerExpression {
    if let Some(value) = &facts.string_value
        && matches!(
            facts.value_type,
            CompilerType::String | CompilerType::Character
        )
    {
        return CompilerExpression {
            kind: CompilerExpressionKind::String(value.clone()),
            value_type: facts.value_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        };
    }
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

fn compiler_list_literal(
    element: CompilerType,
    values: Vec<CompilerExpression>,
    span: Span,
) -> CompilerExpression {
    let value_type = CompilerType::List(Box::new(element));
    values.into_iter().rev().fold(
        CompilerExpression {
            kind: CompilerExpressionKind::ListEmpty,
            value_type: value_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
        |remaining, value| CompilerExpression {
            kind: CompilerExpressionKind::ListEntry {
                value: Box::new(value),
                remaining: Box::new(remaining),
            },
            value_type: value_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
    )
}

fn compiler_int_literal(value: BigInt, span: Span) -> CompilerExpression {
    CompilerExpression {
        kind: CompilerExpressionKind::Int(value.clone()),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(value)),
        rational_value: None,
        span,
    }
}

fn compiler_int_rows_literal(rows: Vec<Vec<BigInt>>, span: Span) -> CompilerExpression {
    let rows = rows
        .into_iter()
        .map(|row| {
            compiler_list_literal(
                CompilerType::Int,
                row.into_iter()
                    .map(|value| compiler_int_literal(value, span))
                    .collect(),
                span,
            )
        })
        .collect();
    compiler_list_literal(int_list_type(), rows, span)
}

fn compiler_int_tuple_list_literal(
    rows: Vec<Vec<BigInt>>,
    width: usize,
    span: Span,
) -> CompilerExpression {
    let tuple_type = CompilerType::Tuple(vec![CompilerType::Int; width]);
    let values = rows
        .into_iter()
        .map(|row| CompilerExpression {
            kind: CompilerExpressionKind::Tuple(
                row.into_iter()
                    .map(|value| compiler_int_literal(value, span))
                    .collect(),
            ),
            value_type: tuple_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        })
        .collect();
    compiler_list_literal(tuple_type, values, span)
}

fn compiler_vertical_ints_literal(text: &str, span: Span) -> CompilerExpression {
    let lines = text.lines().collect::<Vec<_>>();
    let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let values = (0..width)
        .map(|column| {
            let digits = lines
                .iter()
                .filter_map(|line| line.as_bytes().get(column).copied())
                .filter(u8::is_ascii_digit)
                .map(char::from)
                .collect::<String>();
            let kind = if digits.is_empty() {
                CompilerExpressionKind::OptionalNone
            } else {
                CompilerExpressionKind::OptionalSome(Box::new(compiler_int_literal(
                    digits
                        .parse::<BigInt>()
                        .expect("ASCII decimal digits form an Int"),
                    span,
                )))
            };
            CompilerExpression {
                kind,
                value_type: CompilerType::Optional(Box::new(CompilerType::Int)),
                int_range: None,
                rational_value: None,
                span,
            }
        })
        .collect();
    compiler_list_literal(
        CompilerType::Optional(Box::new(CompilerType::Int)),
        values,
        span,
    )
}

#[must_use]
pub fn compiler_function_result_capture_storage(symbol: &str, span: Span, index: usize) -> String {
    format!(
        "topal.function.result.{symbol}.{}.capture.{index}",
        span.start
    )
}

fn is_function_result_capture_storage(storage_name: &str) -> bool {
    storage_name.starts_with("topal.function.result.")
}

fn compiler_expression_is_closed(expression: &CompilerExpression) -> bool {
    compiler_expression_is_closed_with(expression, &BTreeSet::new())
}

#[allow(clippy::too_many_lines)] // Exhaustive closure classification keeps every checked form visible.
fn compiler_expression_is_closed_with(
    expression: &CompilerExpression,
    bound: &BTreeSet<String>,
) -> bool {
    match &expression.kind {
        CompilerExpressionKind::Unit
        | CompilerExpressionKind::Completed
        | CompilerExpressionKind::Effect
        | CompilerExpressionKind::TypeValue(_)
        | CompilerExpressionKind::Root
        | CompilerExpressionKind::LintNamespace
        | CompilerExpressionKind::FunctionValue(_)
        | CompilerExpressionKind::Identity(_)
        | CompilerExpressionKind::TypeView(_)
        | CompilerExpressionKind::FunctionView(_)
        | CompilerExpressionKind::LanguageContext(_)
        | CompilerExpressionKind::Capability(_)
        | CompilerExpressionKind::NativeSerializer(_)
        | CompilerExpressionKind::ExternalMetadata(_)
        | CompilerExpressionKind::ExternalLocationConstruct(_)
        | CompilerExpressionKind::ConstraintValue(_)
        | CompilerExpressionKind::Boolean(_)
        | CompilerExpressionKind::Version(_)
        | CompilerExpressionKind::Int(_)
        | CompilerExpressionKind::Infinity { .. }
        | CompilerExpressionKind::Rational(_)
        | CompilerExpressionKind::String(_)
        | CompilerExpressionKind::StringEmpty
        | CompilerExpressionKind::ErrorCode(_)
        | CompilerExpressionKind::Enum(_)
        | CompilerExpressionKind::OptionalNone
        | CompilerExpressionKind::ListEmpty => true,
        CompilerExpressionKind::ExternalLocationWrite {
            location, value, ..
        } => {
            compiler_expression_is_closed_with(location, bound)
                && compiler_expression_is_closed_with(value, bound)
        }
        CompilerExpressionKind::ListEntry { value, remaining } => {
            compiler_expression_is_closed_with(value, bound)
                && compiler_expression_is_closed_with(remaining, bound)
        }
        CompilerExpressionKind::Sum { payload, .. } => payload
            .as_deref()
            .is_none_or(|value| compiler_expression_is_closed_with(value, bound)),
        CompilerExpressionKind::Tuple(values) => values
            .iter()
            .all(|value| compiler_expression_is_closed_with(value, bound)),
        CompilerExpressionKind::TupleField { tuple, .. } => {
            compiler_expression_is_closed_with(tuple, bound)
        }
        CompilerExpressionKind::Record(fields) => fields
            .iter()
            .all(|(_, value)| compiler_expression_is_closed_with(value, bound)),
        CompilerExpressionKind::RecordReconstruct { base, replacements } => {
            compiler_expression_is_closed_with(base, bound)
                && replacements
                    .iter()
                    .all(|(_, value)| compiler_expression_is_closed_with(value, bound))
        }
        CompilerExpressionKind::Block(block) => compiler_block_is_closed(block, bound),
        CompilerExpressionKind::ExitSequence { preceding, result } => {
            preceding
                .iter()
                .all(|value| compiler_expression_is_closed_with(value, bound))
                && compiler_expression_is_closed_with(result, bound)
        }
        CompilerExpressionKind::PrivateBinding {
            storage_name,
            value,
            body,
        } => {
            if !compiler_expression_is_closed_with(value, bound) {
                return false;
            }
            let mut nested = bound.clone();
            nested.insert(storage_name.clone());
            compiler_expression_is_closed_with(body, &nested)
        }
        CompilerExpressionKind::Local(name)
        | CompilerExpressionKind::InfinityLocal {
            storage_name: name, ..
        } => bound.contains(name),
        CompilerExpressionKind::TaskConstruct { .. }
        | CompilerExpressionKind::TaskStateLoad { .. }
        | CompilerExpressionKind::TaskStateReplace { .. }
        | CompilerExpressionKind::Call { .. }
        | CompilerExpressionKind::Fallible { .. }
        | CompilerExpressionKind::Validate { .. }
        | CompilerExpressionKind::ModularValidate { .. }
        | CompilerExpressionKind::SumDecision { .. }
        | CompilerExpressionKind::ResultDecision { .. }
        | CompilerExpressionKind::OptionalDecision { .. }
        | CompilerExpressionKind::ListDecision { .. }
        | CompilerExpressionKind::ListMap { .. }
        | CompilerExpressionKind::ListSelect { .. }
        | CompilerExpressionKind::ListForeach { .. }
        | CompilerExpressionKind::ListReject { .. }
        | CompilerExpressionKind::ListFold { .. }
        | CompilerExpressionKind::IterateGenerator { .. }
        | CompilerExpressionKind::GeneratorTakeWhile { .. }
        | CompilerExpressionKind::UnfoldGenerator { .. }
        | CompilerExpressionKind::StringCharactersGenerator { .. }
        | CompilerExpressionKind::StringRangeCharactersGenerator { .. }
        | CompilerExpressionKind::StringProvenanceCharactersGenerator { .. }
        | CompilerExpressionKind::StringCharactersForeach { .. }
        | CompilerExpressionKind::CustomCharacterGenerator { .. }
        | CompilerExpressionKind::CustomCharacterForeach { .. }
        | CompilerExpressionKind::CustomValueGenerator { .. }
        | CompilerExpressionKind::CustomValueForeach { .. }
        | CompilerExpressionKind::IterateGeneratorForeach { .. }
        | CompilerExpressionKind::GeneratorCollect(_) => false,
        CompilerExpressionKind::Serialize { value, .. }
        | CompilerExpressionKind::Deserialize(value)
        | CompilerExpressionKind::ExternalLayoutCoerce { value, .. }
        | CompilerExpressionKind::ExternalLocationRead {
            location: value, ..
        }
        | CompilerExpressionKind::IntToModular { value, .. }
        | CompilerExpressionKind::ModularReduce { value, .. }
        | CompilerExpressionKind::Negate(value)
        | CompilerExpressionKind::Absolute(value)
        | CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::RationalToInt(value)
        | CompilerExpressionKind::IntToNat(value)
        | CompilerExpressionKind::IntToNatBoundary(value)
        | CompilerExpressionKind::ResultSuccess(value)
        | CompilerExpressionKind::ResultError(value)
        | CompilerExpressionKind::ResultProject(value)
        | CompilerExpressionKind::ResultProjectBoundary(value)
        | CompilerExpressionKind::OptionalSome(value)
        | CompilerExpressionKind::ListReverse(value)
        | CompilerExpressionKind::ListEntryCount(value)
        | CompilerExpressionKind::ListEmptyPredicate(value)
        | CompilerExpressionKind::ListFirst(value)
        | CompilerExpressionKind::ListRest(value)
        | CompilerExpressionKind::ListUncons(value)
        | CompilerExpressionKind::ListIndexOperation { list: value, .. }
        | CompilerExpressionKind::ListRemoveIndexRange { list: value, .. }
        | CompilerExpressionKind::ListUnzip(value)
        | CompilerExpressionKind::ListEntries(value)
        | CompilerExpressionKind::ListCollectString(value)
        | CompilerExpressionKind::ContainerCollect { source: value, .. }
        | CompilerExpressionKind::ContainerEntryCount(value)
        | CompilerExpressionKind::ContainerEmpty(value)
        | CompilerExpressionKind::ArrayAt { array: value, .. }
        | CompilerExpressionKind::TraversalControl { value, .. }
        | CompilerExpressionKind::StringEmptyPredicate(value)
        | CompilerExpressionKind::StringUtf8ByteCount(value)
        | CompilerExpressionKind::StringCharacterCount(value)
        | CompilerExpressionKind::StringUnicodeScalarValue(value)
        | CompilerExpressionKind::StringDynamicScalarCharactersCollect(value)
        | CompilerExpressionKind::CharacterAsciiDecimalDigit(value)
        | CompilerExpressionKind::CharacterUnicodeWhitespace(value)
        | CompilerExpressionKind::StringCharactersCollect { text: value, .. }
        | CompilerExpressionKind::StringProvenanceCharactersCollect { text: value, .. }
        | CompilerExpressionKind::StringCharactersClose(value)
        | CompilerExpressionKind::CustomCharacterClose {
            generator: value, ..
        }
        | CompilerExpressionKind::RecordField { record: value, .. }
        | CompilerExpressionKind::ErrorField { error: value, .. }
        | CompilerExpressionKind::RangeLower(value)
        | CompilerExpressionKind::RangeUpper(value)
        | CompilerExpressionKind::RangeLowerInclusive(value)
        | CompilerExpressionKind::RangeUpperInclusive(value)
        | CompilerExpressionKind::RangeEmpty(value)
        | CompilerExpressionKind::Not(value) => compiler_expression_is_closed_with(value, bound),
        CompilerExpressionKind::CustomCharacterHandledClose { generator, handler } => {
            compiler_expression_is_closed_with(generator, bound)
                && handler
                    .error_codes
                    .iter()
                    .all(|rule| compiler_expression_is_closed_with(&rule.action, bound))
                && compiler_expression_is_closed_with(&handler.error_action, bound)
                && compiler_expression_is_closed_with(&handler.ok_action, bound)
        }
        CompilerExpressionKind::RationalConstruct {
            numerator,
            denominator,
        } => {
            compiler_expression_is_closed_with(numerator, bound)
                && compiler_expression_is_closed_with(denominator, bound)
        }
        CompilerExpressionKind::StringConcat { left, right }
        | CompilerExpressionKind::ListContainsEntry {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListContainsSequence {
            list: left,
            pattern: right,
        }
        | CompilerExpressionKind::ListContainsSubsequence {
            list: left,
            pattern: right,
        }
        | CompilerExpressionKind::ListRemoveFirst {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListRemoveAll {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListPrepend {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListAppend {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListInsertAt {
            list: left,
            inserted: right,
            ..
        }
        | CompilerExpressionKind::ListInsertEverywhere {
            list: left,
            value: right,
        }
        | CompilerExpressionKind::ListCartesianStringInt { left, right }
        | CompilerExpressionKind::ListRangeSelect {
            list: left,
            range: right,
            ..
        }
        | CompilerExpressionKind::ListConcat { left, right }
        | CompilerExpressionKind::SetContains {
            set: left,
            value: right,
        }
        | CompilerExpressionKind::BagMultiplicity {
            bag: left,
            value: right,
        }
        | CompilerExpressionKind::MapLookup {
            mapping: left,
            key: right,
            ..
        }
        | CompilerExpressionKind::Binary { left, right, .. } => {
            compiler_expression_is_closed_with(left, bound)
                && compiler_expression_is_closed_with(right, bound)
        }
        CompilerExpressionKind::StringRangeSelect { text, range, .. }
        | CompilerExpressionKind::StringRangeCharactersCollect { text, range, .. } => {
            compiler_expression_is_closed_with(text, bound)
                && compiler_expression_is_closed_with(range, bound)
        }
        CompilerExpressionKind::ListZip {
            left,
            right,
            left_default,
            right_default,
            ..
        } => {
            compiler_expression_is_closed_with(left, bound)
                && compiler_expression_is_closed_with(right, bound)
                && left_default
                    .as_deref()
                    .is_none_or(|value| compiler_expression_is_closed_with(value, bound))
                && right_default
                    .as_deref()
                    .is_none_or(|value| compiler_expression_is_closed_with(value, bound))
        }
        CompilerExpressionKind::BooleanDecision {
            subject,
            when_true,
            when_false,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && compiler_expression_is_closed_with(when_true, bound)
                && compiler_expression_is_closed_with(when_false, bound)
        }
        CompilerExpressionKind::OrderedComparisonDecision {
            subject,
            rules,
            otherwise,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && rules.iter().all(|rule| {
                    compiler_expression_is_closed_with(&rule.operand, bound)
                        && compiler_expression_is_closed_with(&rule.action, bound)
                })
                && compiler_expression_is_closed_with(otherwise, bound)
        }
        CompilerExpressionKind::ComparisonValueDecision {
            subject,
            when_less,
            when_equal,
            when_greater,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && compiler_expression_is_closed_with(when_less, bound)
                && compiler_expression_is_closed_with(when_equal, bound)
                && compiler_expression_is_closed_with(when_greater, bound)
        }
        CompilerExpressionKind::EnumDecision {
            subject,
            rules,
            otherwise,
        } => {
            compiler_expression_is_closed_with(subject, bound)
                && rules
                    .iter()
                    .all(|rule| compiler_expression_is_closed_with(&rule.action, bound))
                && otherwise
                    .as_deref()
                    .is_none_or(|value| compiler_expression_is_closed_with(value, bound))
        }
    }
}

fn compiler_block_is_closed(block: &CompilerBlock, outer: &BTreeSet<String>) -> bool {
    let mut bound = outer.clone();
    for statement in &block.statements {
        match statement {
            CompilerStatement::Binding(binding) => {
                if !compiler_expression_is_closed_with(&binding.value, &bound) {
                    return false;
                }
                bound.insert(binding.storage_name.clone());
            }
            CompilerStatement::Discard(expression) => {
                if !compiler_expression_is_closed_with(expression, &bound) {
                    return false;
                }
            }
        }
    }
    compiler_expression_is_closed_with(&block.result, &bound)
}

fn compiler_block_is_closed_over_parameters(
    parameters: &[CompilerParameter],
    block: &CompilerBlock,
) -> bool {
    let bound = parameters
        .iter()
        .filter(|parameter| !parameter.discarded)
        .map(|parameter| parameter.name.clone())
        .collect();
    compiler_block_is_closed(block, &bound)
}

fn into_rational(expression: CompilerExpression) -> CompilerExpression {
    if expression.value_type == CompilerType::Rational {
        return expression;
    }
    if expression.value_type == CompilerType::InfiniteRational {
        let mut expression = expression;
        expression.value_type = CompilerType::Rational;
        return expression;
    }
    debug_assert!(!matches!(
        expression.value_type,
        CompilerType::InfiniteInt | CompilerType::InfiniteNat
    ));
    let span = expression.span;
    let rational_value = exact_int(&expression).map(BigRational::from_integer);
    CompilerExpression {
        kind: CompilerExpressionKind::IntToRational(Box::new(expression)),
        value_type: CompilerType::Rational,
        int_range: None,
        rational_value,
        span,
    }
}

fn forget_character_evidence(mut expression: CompilerExpression) -> CompilerExpression {
    debug_assert_eq!(expression.value_type, CompilerType::Character);
    expression.value_type = CompilerType::String;
    expression
}

fn adapt_decision_pair(left: &mut CompilerExpression, right: &mut CompilerExpression) {
    if let (CompilerType::Tuple(left_types), CompilerType::Tuple(right_types)) =
        (&left.value_type, &right.value_type)
        && left_types.len() == right_types.len()
    {
        let common = left_types
            .iter()
            .zip(right_types)
            .map(|(left, right)| decision_common_type(left, right))
            .collect::<Option<Vec<_>>>();
        if let Some(common) = common {
            if let CompilerExpressionKind::Tuple(fields) = &mut left.kind {
                for (field, value_type) in fields.iter_mut().zip(&common) {
                    field.value_type = value_type.clone();
                }
            }
            if let CompilerExpressionKind::Tuple(fields) = &mut right.kind {
                for (field, value_type) in fields.iter_mut().zip(&common) {
                    field.value_type = value_type.clone();
                }
            }
            left.value_type = CompilerType::Tuple(common.clone());
            right.value_type = CompilerType::Tuple(common);
            return;
        }
    }
    match (&mut left.kind, &mut right.kind) {
        (
            CompilerExpressionKind::Tuple(left_fields),
            CompilerExpressionKind::Tuple(right_fields),
        ) if left_fields.len() == right_fields.len() => {
            for (left, right) in left_fields.iter_mut().zip(right_fields.iter_mut()) {
                adapt_decision_pair(left, right);
            }
            left.value_type = CompilerType::Tuple(
                left_fields
                    .iter()
                    .map(|field| field.value_type.clone())
                    .collect(),
            );
            right.value_type = left.value_type.clone();
        }
        _ if left.value_type == CompilerType::Nat && right.value_type == CompilerType::Int => {
            left.value_type = CompilerType::Int;
        }
        _ if left.value_type == CompilerType::Int && right.value_type == CompilerType::Nat => {
            right.value_type = CompilerType::Int;
        }
        _ if left.value_type == CompilerType::Character
            && right.value_type == CompilerType::String =>
        {
            left.value_type = CompilerType::String;
        }
        _ if left.value_type == CompilerType::String
            && right.value_type == CompilerType::Character =>
        {
            right.value_type = CompilerType::String;
        }
        _ => {}
    }
}

fn decision_common_type(left: &CompilerType, right: &CompilerType) -> Option<CompilerType> {
    if left == right {
        return Some(left.clone());
    }
    match (left, right) {
        (CompilerType::Nat, CompilerType::Int) | (CompilerType::Int, CompilerType::Nat) => {
            Some(CompilerType::Int)
        }
        (CompilerType::Character, CompilerType::String)
        | (CompilerType::String, CompilerType::Character) => Some(CompilerType::String),
        (CompilerType::Tuple(left), CompilerType::Tuple(right)) if left.len() == right.len() => {
            left.iter()
                .zip(right)
                .map(|(left, right)| decision_common_type(left, right))
                .collect::<Option<Vec<_>>>()
                .map(CompilerType::Tuple)
        }
        _ => None,
    }
}

fn retain_character_evidence(mut expression: CompilerExpression) -> Option<CompilerExpression> {
    if expression.value_type == CompilerType::Character {
        return Some(expression);
    }
    if expression.value_type != CompilerType::String
        || exact_string(&expression).is_none_or(|text| character_count(&text) != 1)
    {
        return None;
    }
    expression.value_type = CompilerType::Character;
    Some(expression)
}

fn exact_string(expression: &CompilerExpression) -> Option<String> {
    match &expression.kind {
        CompilerExpressionKind::String(value) => Some(value.clone()),
        CompilerExpressionKind::StringEmpty => Some(String::new()),
        CompilerExpressionKind::StringConcat { left, right } => {
            let mut value = exact_string(left)?;
            value.push_str(&exact_string(right)?);
            Some(value)
        }
        CompilerExpressionKind::StringCharactersCollect { text, .. } => exact_string(text),
        _ => None,
    }
}

fn exact_int_entries(facts: &StaticValueFacts) -> Option<Vec<BigInt>> {
    facts
        .list_entries
        .as_ref()?
        .iter()
        .map(|entry| {
            let range = entry.int_range.as_ref()?;
            (range.lower == range.upper).then(|| range.lower.clone())
        })
        .collect()
}

fn exact_nested_int_entries(facts: &StaticValueFacts) -> Option<Vec<Vec<BigInt>>> {
    facts
        .list_entries
        .as_ref()?
        .iter()
        .map(exact_int_entries)
        .collect()
}

fn exact_rational_entries(facts: &StaticValueFacts) -> Option<Vec<BigRational>> {
    facts
        .list_entries
        .as_ref()?
        .iter()
        .map(|entry| entry.rational_value.clone())
        .collect()
}

fn exact_closed_range_values(range: &ClosedIntRange, limit: usize) -> Option<Vec<BigInt>> {
    let mut current = &range.lower + u8::from(!range.lower_inclusive);
    let finish = &range.upper - u8::from(!range.upper_inclusive);
    let mut values = Vec::new();
    while current <= finish {
        if values.len() == limit {
            return None;
        }
        values.push(current.clone());
        current += 1;
    }
    Some(values)
}

fn compiler_int_list_literal(values: &[BigInt], span: Span) -> CompilerExpression {
    let list_type = int_list_type();
    values.iter().rev().fold(
        CompilerExpression {
            kind: CompilerExpressionKind::ListEmpty,
            value_type: list_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
        |remaining, value| CompilerExpression {
            kind: CompilerExpressionKind::ListEntry {
                value: Box::new(CompilerExpression {
                    kind: CompilerExpressionKind::Int(value.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(value.clone())),
                    rational_value: None,
                    span,
                }),
                remaining: Box::new(remaining),
            },
            value_type: list_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
    )
}

fn compiler_nested_int_list_literal(values: &[Vec<BigInt>], span: Span) -> CompilerExpression {
    let list_type = CompilerType::List(Box::new(int_list_type()));
    values.iter().rev().fold(
        CompilerExpression {
            kind: CompilerExpressionKind::ListEmpty,
            value_type: list_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
        |remaining, value| CompilerExpression {
            kind: CompilerExpressionKind::ListEntry {
                value: Box::new(compiler_int_list_literal(value, span)),
                remaining: Box::new(remaining),
            },
            value_type: list_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
    )
}

fn compiler_rational_list_literal(values: &[BigRational], span: Span) -> CompilerExpression {
    let list_type = CompilerType::List(Box::new(CompilerType::Rational));
    values.iter().rev().fold(
        CompilerExpression {
            kind: CompilerExpressionKind::ListEmpty,
            value_type: list_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
        |remaining, value| CompilerExpression {
            kind: CompilerExpressionKind::ListEntry {
                value: Box::new(CompilerExpression {
                    kind: CompilerExpressionKind::Rational(value.clone()),
                    value_type: CompilerType::Rational,
                    int_range: None,
                    rational_value: Some(value.clone()),
                    span,
                }),
                remaining: Box::new(remaining),
            },
            value_type: list_type.clone(),
            int_range: None,
            rational_value: None,
            span,
        },
    )
}

fn compiler_index_literal(
    value: usize,
    value_type: CompilerType,
    span: Span,
) -> CompilerExpression {
    let value = BigInt::from(value);
    CompilerExpression {
        kind: CompilerExpressionKind::Int(value.clone()),
        value_type,
        int_range: Some(IntRange::exact(value)),
        rational_value: None,
        span,
    }
}

fn compiler_optional_literal(
    value: Option<CompilerExpression>,
    payload: CompilerType,
    span: Span,
) -> CompilerExpression {
    CompilerExpression {
        kind: value.map_or(CompilerExpressionKind::OptionalNone, |value| {
            CompilerExpressionKind::OptionalSome(Box::new(value))
        }),
        value_type: CompilerType::Optional(Box::new(payload)),
        int_range: None,
        rational_value: None,
        span,
    }
}

fn exact_binary_search<T: Ord>(values: &[T], sought: &T) -> Option<usize> {
    let (mut lower, mut upper) = (0, values.len());
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        match values[middle].cmp(sought) {
            std::cmp::Ordering::Equal => return Some(middle),
            std::cmp::Ordering::Less => lower = middle + 1,
            std::cmp::Ordering::Greater => upper = middle,
        }
    }
    None
}

fn exact_ordered_int_call(
    name: &str,
    values: &[BigInt],
    sought: Option<&BigInt>,
    count: Option<usize>,
    right: Option<Vec<BigInt>>,
    span: Span,
) -> Option<CompilerExpression> {
    let lower = sought.map(|sought| values.partition_point(|value| value < sought));
    let upper = sought.map(|sought| values.partition_point(|value| value <= sought));
    match name {
        "std.ordered.lower-bound" => Some(compiler_index_literal(lower?, CompilerType::Int, span)),
        "std.ordered.upper-bound" => Some(compiler_index_literal(upper?, CompilerType::Int, span)),
        "std.ordered.equal-range" => Some(CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation: CompilerBinary::Range,
                left: Box::new(compiler_index_literal(lower?, CompilerType::Int, span)),
                right: Box::new(compiler_index_literal(upper?, CompilerType::Int, span)),
            },
            value_type: CompilerType::Range(Box::new(CompilerType::Int)),
            int_range: None,
            rational_value: None,
            span,
        }),
        "std.ordered.binary-search" => Some(compiler_optional_literal(
            exact_binary_search(values, sought?)
                .map(|index| compiler_index_literal(index, CompilerType::Nat, span)),
            CompilerType::Nat,
            span,
        )),
        "std.ordered.merge" => {
            let mut merged = values.to_vec();
            merged.extend(right?);
            merged.sort();
            Some(compiler_int_list_literal(&merged, span))
        }
        "std.ordered.smallest" => {
            let mut selected = values.to_vec();
            selected.sort();
            selected.truncate(count?.min(selected.len()));
            Some(compiler_int_list_literal(&selected, span))
        }
        "std.ordered.nth" => {
            let mut selected = values.to_vec();
            selected.sort();
            Some(compiler_optional_literal(
                selected
                    .get(count?)
                    .cloned()
                    .map(|value| CompilerExpression {
                        kind: CompilerExpressionKind::Int(value.clone()),
                        value_type: CompilerType::Int,
                        int_range: Some(IntRange::exact(value)),
                        rational_value: None,
                        span,
                    }),
                CompilerType::Int,
                span,
            ))
        }
        _ => None,
    }
}

fn exact_ordered_rational_call(
    name: &str,
    values: &[BigRational],
    sought: Option<&BigRational>,
    count: Option<usize>,
    right: Option<Vec<BigRational>>,
    span: Span,
) -> Option<CompilerExpression> {
    match name {
        "std.ordered.binary-search" => Some(compiler_optional_literal(
            exact_binary_search(values, sought?)
                .map(|index| compiler_index_literal(index, CompilerType::Nat, span)),
            CompilerType::Nat,
            span,
        )),
        "std.ordered.merge" => {
            let mut merged = values.to_vec();
            merged.extend(right?);
            merged.sort();
            Some(compiler_rational_list_literal(&merged, span))
        }
        "std.ordered.smallest" => {
            let mut selected = values.to_vec();
            selected.sort();
            selected.truncate(count?.min(selected.len()));
            Some(compiler_rational_list_literal(&selected, span))
        }
        "std.ordered.nth" => {
            let mut selected = values.to_vec();
            selected.sort();
            Some(compiler_optional_literal(
                selected
                    .get(count?)
                    .cloned()
                    .map(|value| CompilerExpression {
                        kind: CompilerExpressionKind::Rational(value.clone()),
                        value_type: CompilerType::Rational,
                        int_range: None,
                        rational_value: Some(value),
                        span,
                    }),
                CompilerType::Rational,
                span,
            ))
        }
        _ => None,
    }
}

fn adapt_call_argument(
    expected: &CompilerType,
    argument: &CompilerExpression,
) -> Option<CompilerExpression> {
    if expected == &argument.value_type {
        return Some(argument.clone());
    }
    if matches!(&argument.value_type, CompilerType::Refined { base, .. }
        if base.as_ref() == expected)
    {
        let mut value = argument.clone();
        value.value_type = expected.clone();
        return Some(value);
    }
    match (expected, &argument.value_type) {
        (CompilerType::Character, CompilerType::String) => {
            retain_character_evidence(argument.clone())
        }
        (CompilerType::String, CompilerType::Character) => {
            Some(forget_character_evidence(argument.clone()))
        }
        (CompilerType::Int, CompilerType::Nat) => {
            let mut value = argument.clone();
            value.value_type = CompilerType::Int;
            Some(value)
        }
        (CompilerType::Rational, CompilerType::Int | CompilerType::Nat) => {
            Some(into_rational(argument.clone()))
        }
        (CompilerType::Int, CompilerType::Rational) => {
            let rational = argument.rational_value.as_ref()?;
            (rational.denom() == &BigInt::from(1)).then(|| CompilerExpression {
                kind: CompilerExpressionKind::RationalToInt(Box::new(argument.clone())),
                value_type: CompilerType::Int,
                int_range: Some(IntRange::exact(rational.numer().clone())),
                rational_value: None,
                span: argument.span,
            })
        }
        (CompilerType::Nat, CompilerType::Int) => argument
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= BigInt::from(0))
            .then(|| CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(argument.clone())),
                value_type: CompilerType::Nat,
                int_range: argument.int_range.clone(),
                rational_value: None,
                span: argument.span,
            }),
        (CompilerType::Nat, CompilerType::Rational) => {
            let rational = argument.rational_value.as_ref()?;
            (rational.denom() == &BigInt::from(1) && rational.numer() >= &BigInt::from(0)).then(
                || {
                    let integer = CompilerExpression {
                        kind: CompilerExpressionKind::RationalToInt(Box::new(argument.clone())),
                        value_type: CompilerType::Int,
                        int_range: Some(IntRange::exact(rational.numer().clone())),
                        rational_value: None,
                        span: argument.span,
                    };
                    CompilerExpression {
                        kind: CompilerExpressionKind::IntToNat(Box::new(integer)),
                        value_type: CompilerType::Nat,
                        int_range: Some(IntRange::exact(rational.numer().clone())),
                        rational_value: None,
                        span: argument.span,
                    }
                },
            )
        }
        _ => None,
    }
}

fn adapt_function_call_argument(
    expected: &CompilerType,
    argument: &CompilerExpression,
) -> Option<CompilerExpression> {
    if compiler_infinity_evidence_compatible(expected, &argument.value_type) {
        return Some(argument.clone());
    }
    if let (CompilerType::Tuple(expected_fields), CompilerType::Tuple(actual_fields)) =
        (expected, &argument.value_type)
        && expected_fields.len() == actual_fields.len()
        && expected_fields
            .iter()
            .zip(actual_fields)
            .all(|(expected, actual)| {
                expected == actual
                    || matches!((expected, actual), (CompilerType::Int, CompilerType::Nat))
            })
    {
        let mut adapted = argument.clone();
        adapted.value_type = expected.clone();
        return Some(adapted);
    }
    if let (CompilerType::Tuple(expected), CompilerExpressionKind::Tuple(values)) =
        (expected, &argument.kind)
        && expected.len() == values.len()
    {
        let values = expected
            .iter()
            .zip(values)
            .map(|(expected, value)| adapt_function_call_argument(expected, value))
            .collect::<Option<Vec<_>>>()?;
        return Some(CompilerExpression {
            kind: CompilerExpressionKind::Tuple(values),
            value_type: CompilerType::Tuple(expected.clone()),
            int_range: None,
            rational_value: None,
            span: argument.span,
        });
    }
    if let (CompilerType::Record(expected), CompilerExpressionKind::Record(values)) =
        (expected, &argument.kind)
        && expected.len() == values.len()
    {
        let values = expected
            .iter()
            .zip(values)
            .map(|((expected_name, expected), (actual_name, value))| {
                (expected_name == actual_name)
                    .then(|| {
                        adapt_function_call_argument(expected, value)
                            .map(|value| (actual_name.clone(), value))
                    })
                    .flatten()
            })
            .collect::<Option<Vec<_>>>()?;
        return Some(CompilerExpression {
            kind: CompilerExpressionKind::Record(values),
            value_type: CompilerType::Record(expected.clone()),
            int_range: None,
            rational_value: None,
            span: argument.span,
        });
    }
    if matches!(
        (expected, &argument.value_type),
        (CompilerType::Int, CompilerType::InfiniteNat)
    ) {
        let mut value = argument.clone();
        value.value_type = CompilerType::InfiniteInt;
        return Some(value);
    }
    adapt_call_argument(expected, argument).or_else(|| {
        compiler_infinity_evidence_compatible(expected, &argument.value_type)
            .then(|| argument.clone())
    })
}

fn compiler_infinity_evidence_compatible(expected: &CompilerType, actual: &CompilerType) -> bool {
    if expected == actual {
        return true;
    }
    match (expected, actual) {
        (CompilerType::Int, CompilerType::InfiniteInt)
        | (CompilerType::Nat, CompilerType::InfiniteNat)
        | (CompilerType::Rational, CompilerType::InfiniteRational) => true,
        (CompilerType::Tuple(expected), CompilerType::Tuple(actual)) => {
            expected.len() == actual.len()
                && expected.iter().zip(actual).all(|(expected, actual)| {
                    compiler_infinity_evidence_compatible(expected, actual)
                })
        }
        (CompilerType::Record(expected), CompilerType::Record(actual)) => {
            expected.len() == actual.len()
                && expected.iter().zip(actual).all(
                    |((expected_name, expected), (actual_name, actual))| {
                        expected_name == actual_name
                            && compiler_infinity_evidence_compatible(expected, actual)
                    },
                )
        }
        _ => false,
    }
}

fn capture_argument_adaptation_may_depend_on_facts(
    expected: &CompilerType,
    argument: &CompilerExpression,
) -> bool {
    if !matches!(argument.kind, CompilerExpressionKind::Local(_)) {
        return false;
    }
    matches!(
        (expected, &argument.value_type),
        (CompilerType::Character, CompilerType::String)
            | (
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Rational
            )
            | (CompilerType::Nat, CompilerType::Int)
    )
}

fn validate_list_custom_generator_initial(
    source: &SourceText,
    parameter: &CompilerParameter,
    argument: &CompilerExpression,
    generator_name: &str,
) -> Result<(), Diagnostic> {
    if parameter.value_type == int_list_type()
        && !exact_singleton_int_list(argument, 7)
        && !matches!(&argument.kind, CompilerExpressionKind::Local(name)
            if generator_name == "relay" && name == "initial")
    {
        return Err(unsupported(
            source,
            argument.span,
            "List Int custom generator input outside exact one 7 or the admitted factory parameter",
        ));
    }
    Ok(())
}
