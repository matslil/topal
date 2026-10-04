type ListEntryBindings = ((String, Span), (String, Span));

pub fn emit_llvm(program: &CompilerProgram, source_name: &str) -> String {
    Generator::new(program, source_name).emit()
}

fn program_uses_extended_debug(program: &CompilerProgram) -> bool {
    block_uses_extended_debug(&program.main)
        || program.functions.iter().any(|function| {
            type_uses_extended_debug(&function.result_type)
                || function
                    .parameters
                    .iter()
                    .any(|parameter| type_uses_extended_debug(&parameter.value_type))
                || block_uses_extended_debug(&function.body)
        })
}

fn program_uses_generator_close_handler(program: &CompilerProgram) -> bool {
    program.functions.iter().any(|function| {
        function.body.statements.iter().any(|statement| {
            matches!(
                statement,
                CompilerStatement::Discard(CompilerExpression {
                    kind: CompilerExpressionKind::CustomCharacterHandledClose { .. },
                    ..
                })
            )
        })
    })
}

fn block_uses_extended_debug(block: &CompilerBlock) -> bool {
    block.statements.iter().any(|statement| match statement {
        CompilerStatement::Binding(binding) => expression_uses_extended_debug(&binding.value),
        CompilerStatement::Discard(expression) => expression_uses_extended_debug(expression),
    }) || expression_uses_extended_debug(&block.result)
}

fn type_uses_extended_debug(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Refined { base, .. } => type_uses_extended_debug(base),
        CompilerType::Character
        | CompilerType::String
        | CompilerType::Error
        | CompilerType::ErrorCode
        | CompilerType::ErrorDomain
        | CompilerType::SourceLocation
        | CompilerType::Version
        | CompilerType::SerializationStream(_)
        | CompilerType::Modular(_)
        | CompilerType::Optional(_)
        | CompilerType::TraversalControl(_)
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => true,
        CompilerType::Range(endpoint)
        | CompilerType::Result(endpoint)
        | CompilerType::TaskResponse(endpoint)
        | CompilerType::List(endpoint)
        | CompilerType::Set(endpoint)
        | CompilerType::Bag(endpoint) => type_uses_extended_debug(endpoint),
        CompilerType::Array { element, .. } => type_uses_extended_debug(element),
        CompilerType::Map { key, value } => {
            type_uses_extended_debug(key) || type_uses_extended_debug(value)
        }
        CompilerType::Generator(generator) => {
            type_uses_extended_debug(&generator.yield_type)
                || type_uses_extended_debug(&generator.resume_type)
                || type_uses_extended_debug(&generator.result_type)
        }
        CompilerType::Tuple(fields) => fields.iter().any(type_uses_extended_debug),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, value_type)| type_uses_extended_debug(value_type)),
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(type_uses_extended_debug)
        }),
        CompilerType::Unit
        | CompilerType::Completed
        | CompilerType::Effect
        | CompilerType::Type
        | CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata
        | CompilerType::Constraint
        | CompilerType::Boolean
        | CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat
        | CompilerType::InfiniteRational
        | CompilerType::Rational
        | CompilerType::Comparison
        | CompilerType::Enum(_) => false,
    }
}

#[allow(clippy::too_many_lines)] // Exhaustive debug classification keeps every checked form visible.
fn expression_uses_extended_debug(expression: &CompilerExpression) -> bool {
    if type_uses_extended_debug(&expression.value_type) {
        return true;
    }
    match &expression.kind {
        CompilerExpressionKind::String(_)
        | CompilerExpressionKind::StringEmpty
        | CompilerExpressionKind::ErrorField { .. }
        | CompilerExpressionKind::ResultDecision { .. }
        | CompilerExpressionKind::CustomCharacterHandledClose { .. }
        | CompilerExpressionKind::OptionalDecision { .. }
        | CompilerExpressionKind::ListDecision { .. }
        | CompilerExpressionKind::ErrorCode(_) => true,
        CompilerExpressionKind::Tuple(fields)
        | CompilerExpressionKind::Call {
            arguments: fields, ..
        } => fields.iter().any(expression_uses_extended_debug),
        CompilerExpressionKind::TupleField { tuple, .. } => expression_uses_extended_debug(tuple),
        CompilerExpressionKind::Record(fields) => fields
            .iter()
            .any(|(_, value)| expression_uses_extended_debug(value)),
        CompilerExpressionKind::RecordReconstruct { base, replacements } => {
            expression_uses_extended_debug(base)
                || replacements
                    .iter()
                    .any(|(_, value)| expression_uses_extended_debug(value))
        }
        CompilerExpressionKind::ListMap { list, body, .. }
        | CompilerExpressionKind::ListSelect { list, body, .. }
        | CompilerExpressionKind::ListForeach { list, body, .. }
        | CompilerExpressionKind::ListReject {
            list,
            predicate: body,
            ..
        } => expression_uses_extended_debug(list) || block_uses_extended_debug(body),
        CompilerExpressionKind::IterateGenerator { initial, next, .. } => {
            expression_uses_extended_debug(initial) || block_uses_extended_debug(next)
        }
        CompilerExpressionKind::GeneratorTakeWhile {
            generator,
            predicate,
            ..
        } => expression_uses_extended_debug(generator) || block_uses_extended_debug(predicate),
        CompilerExpressionKind::UnfoldGenerator { seed, step, .. } => {
            expression_uses_extended_debug(seed) || block_uses_extended_debug(step)
        }
        CompilerExpressionKind::StringCharactersGenerator { text, .. }
        | CompilerExpressionKind::StringCharactersCollect { text, .. }
        | CompilerExpressionKind::StringDynamicScalarCharactersCollect(text)
        | CompilerExpressionKind::StringProvenanceCharactersGenerator { text, .. }
        | CompilerExpressionKind::StringProvenanceCharactersCollect { text, .. } => {
            expression_uses_extended_debug(text)
        }
        CompilerExpressionKind::StringRangeCharactersGenerator { text, range, .. }
        | CompilerExpressionKind::StringRangeCharactersCollect { text, range, .. }
        | CompilerExpressionKind::StringRangeSelect { text, range, .. } => {
            expression_uses_extended_debug(text) || expression_uses_extended_debug(range)
        }
        CompilerExpressionKind::StringCharactersClose(generator)
        | CompilerExpressionKind::GeneratorCollect(generator) => {
            expression_uses_extended_debug(generator)
        }
        CompilerExpressionKind::CustomCharacterClose {
            generator,
            provenance,
            ..
        } => {
            expression_uses_extended_debug(generator) || expression_uses_extended_debug(provenance)
        }
        CompilerExpressionKind::CustomCharacterGenerator {
            initial,
            prefix,
            result,
            ..
        } => {
            expression_uses_extended_debug(initial)
                || block_uses_extended_debug(prefix)
                || expression_uses_extended_debug(result)
        }
        CompilerExpressionKind::CustomValueGenerator {
            initial,
            additional_initials,
            prefix,
            yields,
            continuations,
            result,
            ..
        } => {
            expression_uses_extended_debug(initial)
                || additional_initials
                    .iter()
                    .any(expression_uses_extended_debug)
                || block_uses_extended_debug(prefix)
                || yields.iter().any(|yielded| match yielded {
                    CompilerGeneratorYield::Initial(_) => false,
                    CompilerGeneratorYield::Value(value) => expression_uses_extended_debug(value),
                })
                || continuations
                    .iter()
                    .any(|continuation| block_uses_extended_debug(&continuation.body))
                || expression_uses_extended_debug(result)
        }
        CompilerExpressionKind::StringCharactersForeach { source, body, .. } => {
            expression_uses_extended_debug(source) || block_uses_extended_debug(body)
        }
        CompilerExpressionKind::CustomCharacterForeach {
            source,
            body,
            result,
            ..
        } => {
            expression_uses_extended_debug(source)
                || block_uses_extended_debug(body)
                || expression_uses_extended_debug(result)
        }
        CompilerExpressionKind::CustomValueForeach {
            source,
            transferred_initial,
            prefix,
            yields,
            continuations,
            body,
            result,
            ..
        } => {
            expression_uses_extended_debug(source)
                || transferred_initial
                    .as_deref()
                    .is_some_and(expression_uses_extended_debug)
                || block_uses_extended_debug(prefix)
                || yields.iter().any(|yielded| match yielded {
                    CompilerGeneratorYield::Initial(_) => false,
                    CompilerGeneratorYield::Value(value) => expression_uses_extended_debug(value),
                })
                || continuations
                    .iter()
                    .any(|continuation| block_uses_extended_debug(&continuation.body))
                || block_uses_extended_debug(body)
                || expression_uses_extended_debug(result)
        }
        CompilerExpressionKind::IterateGeneratorForeach {
            generator, body, ..
        } => expression_uses_extended_debug(generator) || block_uses_extended_debug(body),
        CompilerExpressionKind::ListFold {
            list,
            initial,
            body,
            ..
        } => {
            expression_uses_extended_debug(list)
                || expression_uses_extended_debug(initial)
                || block_uses_extended_debug(body)
        }
        CompilerExpressionKind::Sum { payload, .. } => payload
            .as_deref()
            .is_some_and(expression_uses_extended_debug),
        CompilerExpressionKind::Block(block) => block_uses_extended_debug(block),
        CompilerExpressionKind::PrivateBinding { value, body, .. } => {
            expression_uses_extended_debug(value) || expression_uses_extended_debug(body)
        }
        CompilerExpressionKind::IntToModular { value, .. }
        | CompilerExpressionKind::ExternalLayoutCoerce { value, .. }
        | CompilerExpressionKind::ExternalLocationRead {
            location: value, ..
        }
        | CompilerExpressionKind::Serialize { value, .. }
        | CompilerExpressionKind::Deserialize(value)
        | CompilerExpressionKind::ModularReduce { value, .. }
        | CompilerExpressionKind::ModularValidate { value, .. }
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
        | CompilerExpressionKind::TraversalControl { value, .. }
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
        | CompilerExpressionKind::StringEmptyPredicate(value)
        | CompilerExpressionKind::StringUtf8ByteCount(value)
        | CompilerExpressionKind::StringCharacterCount(value)
        | CompilerExpressionKind::StringUnicodeScalarValue(value)
        | CompilerExpressionKind::CharacterAsciiDecimalDigit(value)
        | CompilerExpressionKind::CharacterUnicodeWhitespace(value)
        | CompilerExpressionKind::RecordField { record: value, .. }
        | CompilerExpressionKind::RangeLower(value)
        | CompilerExpressionKind::RangeUpper(value)
        | CompilerExpressionKind::RangeLowerInclusive(value)
        | CompilerExpressionKind::RangeUpperInclusive(value)
        | CompilerExpressionKind::RangeEmpty(value)
        | CompilerExpressionKind::Not(value)
        | CompilerExpressionKind::Validate { value, .. } => expression_uses_extended_debug(value),
        CompilerExpressionKind::TaskConstruct { initial, .. } => {
            expression_uses_extended_debug(initial)
        }
        CompilerExpressionKind::TaskStateLoad { task, .. } => expression_uses_extended_debug(task),
        CompilerExpressionKind::TaskStateReplace { task, value, .. } => {
            expression_uses_extended_debug(task) || expression_uses_extended_debug(value)
        }
        CompilerExpressionKind::ExternalLocationWrite {
            location, value, ..
        } => expression_uses_extended_debug(location) || expression_uses_extended_debug(value),
        CompilerExpressionKind::ExitSequence { preceding, result } => {
            preceding.iter().any(expression_uses_extended_debug)
                || expression_uses_extended_debug(result)
        }
        CompilerExpressionKind::StringConcat { left, right }
        | CompilerExpressionKind::ListEntry {
            value: left,
            remaining: right,
        }
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
        | CompilerExpressionKind::RationalConstruct {
            numerator: left,
            denominator: right,
        }
        | CompilerExpressionKind::Fallible { left, right, .. }
        | CompilerExpressionKind::Binary { left, right, .. } => {
            expression_uses_extended_debug(left) || expression_uses_extended_debug(right)
        }
        CompilerExpressionKind::ListZip {
            left,
            right,
            left_default,
            right_default,
            ..
        } => {
            expression_uses_extended_debug(left)
                || expression_uses_extended_debug(right)
                || left_default
                    .as_deref()
                    .is_some_and(expression_uses_extended_debug)
                || right_default
                    .as_deref()
                    .is_some_and(expression_uses_extended_debug)
        }
        CompilerExpressionKind::BooleanDecision {
            subject,
            when_true,
            when_false,
        } => {
            expression_uses_extended_debug(subject)
                || expression_uses_extended_debug(when_true)
                || expression_uses_extended_debug(when_false)
        }
        CompilerExpressionKind::OrderedComparisonDecision {
            subject,
            rules,
            otherwise,
        } => {
            expression_uses_extended_debug(subject)
                || rules.iter().any(|rule| {
                    expression_uses_extended_debug(&rule.operand)
                        || expression_uses_extended_debug(&rule.action)
                })
                || expression_uses_extended_debug(otherwise)
        }
        CompilerExpressionKind::ComparisonValueDecision {
            subject,
            when_less,
            when_equal,
            when_greater,
        } => {
            expression_uses_extended_debug(subject)
                || expression_uses_extended_debug(when_less)
                || expression_uses_extended_debug(when_equal)
                || expression_uses_extended_debug(when_greater)
        }
        CompilerExpressionKind::EnumDecision {
            subject,
            rules,
            otherwise,
        } => {
            expression_uses_extended_debug(subject)
                || rules
                    .iter()
                    .any(|rule| expression_uses_extended_debug(&rule.action))
                || otherwise
                    .as_deref()
                    .is_some_and(expression_uses_extended_debug)
        }
        CompilerExpressionKind::SumDecision {
            subject,
            rules,
            otherwise,
        } => {
            expression_uses_extended_debug(subject)
                || rules
                    .iter()
                    .any(|rule| expression_uses_extended_debug(&rule.action))
                || otherwise
                    .as_deref()
                    .is_some_and(expression_uses_extended_debug)
        }
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
        | CompilerExpressionKind::Enum(_)
        | CompilerExpressionKind::OptionalNone
        | CompilerExpressionKind::ListEmpty
        | CompilerExpressionKind::Local(_)
        | CompilerExpressionKind::InfinityLocal { .. } => false,
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ListIntRuntimeFragment {
    Containment,
    Removal,
    Core,
    RangeSelection,
    Sequence,
    FundamentalContainers,
    NestedIntCore,
    NestedIntStringCore,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ScalarListRuntimeFragment {
    Unit,
    Completed,
    Effect,
    Type,
    Enum,
    Modular,
    OptionalInt,
    OptionalRational,
    OptionalString,
    IntPair,
    IntTriple,
    IntIntStringIntTuple,
    IntIntBooleanPair,
    IntRange,
    IntStringPair,
    StringIntPair,
    StringPair,
    BooleanStringPair,
    Boolean,
    Comparison,
    ErrorCode,
    Rational,
    String,
}

struct Generator<'a> {
    program: &'a CompilerProgram,
    source_name: &'a str,
    globals: Vec<String>,
    functions: Vec<String>,
    foreign_declarations: BTreeSet<String>,
    next_global: usize,
    needs_infinity_result_runtime: bool,
    scalar_list_runtime_fragments: BTreeSet<ScalarListRuntimeFragment>,
    list_int_runtime_fragments: BTreeSet<ListIntRuntimeFragment>,
    current_result_captures: Vec<CompilerFunctionResultCapture>,
    current_function_return_type: Option<String>,
    debug: DebugInfo,
}
