use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use topal_language::{
    CompilerAggregatePathElement, CompilerBinary, CompilerBlock, CompilerComparisonRule,
    CompilerContainerKind, CompilerEnumRule, CompilerEnumType, CompilerErrorCodeRule,
    CompilerErrorField, CompilerExpression, CompilerExpressionKind, CompilerFallible,
    CompilerFunction, CompilerFunctionResultCapture, CompilerGeneratorCloseHandler,
    CompilerGeneratorLocal, CompilerGeneratorType, CompilerGeneratorYield,
    CompilerListIndexOperation, CompilerListZipOperation, CompilerLocationType,
    CompilerMapCollisionPolicy, CompilerModularType, CompilerParameter, CompilerProgram,
    CompilerStatement, CompilerSumRule, CompilerSumType, CompilerTaskType, CompilerType,
    CompilerValidation, compiler_function_result_capture_storage, display_string_literal,
};
use topal_source::Span;

use crate::{DATA_LAYOUT, TARGET_TRIPLE};

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
        | CompilerExpressionKind::StringCharactersCollect { text, .. } => {
            expression_uses_extended_debug(text)
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
        | CompilerExpressionKind::ResultSuccess(value)
        | CompilerExpressionKind::ResultProject(value)
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
    NestedIntStringCore,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PointerListRuntimeFragment {
    Boolean,
    Rational,
    String,
}

struct Generator<'a> {
    program: &'a CompilerProgram,
    source_name: &'a str,
    globals: Vec<String>,
    functions: Vec<String>,
    next_global: usize,
    needs_infinity_result_runtime: bool,
    pointer_list_runtime_fragments: BTreeSet<PointerListRuntimeFragment>,
    list_int_runtime_fragments: BTreeSet<ListIntRuntimeFragment>,
    current_result_captures: Vec<CompilerFunctionResultCapture>,
    current_function_return_type: Option<String>,
    debug: DebugInfo,
}

impl<'a> Generator<'a> {
    fn new(program: &'a CompilerProgram, source_name: &'a str) -> Self {
        let mut debug = DebugInfo::new(
            source_name,
            program_uses_extended_debug(program),
            program_uses_generator_close_handler(program),
        );
        debug.set_source(program.source.clone());
        if !program.function_value_names.is_empty() {
            debug.enum_type(&function_value_enumeration(program));
        }
        if !program.constraints.is_empty() {
            debug.enum_type(&constraint_value_enumeration(program));
        }
        Self {
            program,
            source_name,
            globals: Vec::new(),
            functions: Vec::new(),
            next_global: 0,
            needs_infinity_result_runtime: false,
            pointer_list_runtime_fragments: BTreeSet::new(),
            list_int_runtime_fragments: BTreeSet::new(),
            current_result_captures: Vec::new(),
            current_function_return_type: None,
            debug,
        }
    }

    fn emit(mut self) -> String {
        for function in &self.program.functions {
            self.emit_function(function);
        }
        self.emit_main();
        let mut module = format!(
            "; Topal native compiler output\nsource_filename = \"{}\"\ntarget datalayout = \"{DATA_LAYOUT}\"\ntarget triple = \"{TARGET_TRIPLE}\"\n\n",
            llvm_string(self.debug.filename())
        );
        module.push_str(
            "@llvm.used = appending global [2 x ptr] [ptr @topal.main, ptr @topal.platform.exit], section \"llvm.metadata\"\n",
        );
        module.push('\n');
        module.push_str(PLATFORM_RUNTIME);
        module.push('\n');
        if self.needs_infinity_result_runtime {
            module.push_str(INFINITY_RESULT_RUNTIME);
            module.push('\n');
        }
        self.emit_pointer_list_runtimes(&mut module);
        if self
            .list_int_runtime_fragments
            .iter()
            .any(|fragment| *fragment != ListIntRuntimeFragment::NestedIntStringCore)
        {
            module.push_str(LIST_INT_LAYOUT);
            module.push('\n');
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::Containment)
        {
            module.push_str(LIST_INT_CONTAINMENT_RUNTIME);
            module.push('\n');
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::Removal)
        {
            module.push_str(LIST_INT_REMOVAL_RUNTIME);
            module.push('\n');
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::Core)
        {
            module.push_str(LIST_INT_CORE_RUNTIME);
            module.push('\n');
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::RangeSelection)
        {
            module.push_str(LIST_INT_RANGE_SELECTION_RUNTIME);
            module.push('\n');
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::Sequence)
        {
            module.push_str(LIST_INT_SEQUENCE_RUNTIME);
            module.push('\n');
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::FundamentalContainers)
        {
            module.push_str(FUNDAMENTAL_CONTAINERS_RUNTIME);
            module.push('\n');
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::NestedIntStringCore)
        {
            module.push_str(LIST_NESTED_INT_STRING_CORE_RUNTIME);
            module.push('\n');
        }
        for global in &self.globals {
            let _ = writeln!(module, "{global}");
        }
        module.push('\n');
        for function in &self.functions {
            module.push_str(function);
            module.push('\n');
        }
        module.push_str(
            "define void @_start() naked noreturn nounwind {\nentry:\n  call void asm sideeffect \"andq $$-16, %rsp\\0A\\09callq topal.main\\0A\\09xorl %edi, %edi\\0A\\09callq topal.platform.exit\", \"~{rax},~{rcx},~{rdx},~{rsi},~{rdi},~{r8},~{r9},~{r10},~{r11},~{memory},~{dirflag},~{fpsr},~{flags}\"()\n  unreachable\n}\n\n",
        );
        module.push_str(&self.debug.finish());
        module
    }

    fn emit_pointer_list_runtimes(&self, module: &mut String) {
        for (fragment, runtime) in [
            (
                PointerListRuntimeFragment::Boolean,
                LIST_BOOLEAN_CORE_RUNTIME,
            ),
            (
                PointerListRuntimeFragment::Rational,
                LIST_RATIONAL_CORE_RUNTIME,
            ),
            (PointerListRuntimeFragment::String, LIST_STRING_CORE_RUNTIME),
        ] {
            if self.pointer_list_runtime_fragments.contains(&fragment) {
                module.push_str(runtime);
                module.push('\n');
            }
        }
    }

    #[allow(clippy::if_not_else, clippy::too_many_lines)] // Capture returns precede exhaustive ordinary returns.
    fn emit_function(&mut self, function: &CompilerFunction) {
        let return_type = function_llvm_return_type(function);
        let parameter_types = function
            .parameters
            .iter()
            .map(|parameter| parameter.value_type.clone())
            .collect::<Vec<_>>();
        let subprogram = self.debug.subprogram(
            &function.source_name,
            &function.symbol,
            function.span,
            &function.result_type,
            &parameter_types,
        );
        let parameters = function
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                format!("{} %arg{index}", llvm_value_type(&parameter.value_type))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let mut body = FunctionBody::new(subprogram);
        let mut environment = BTreeMap::new();
        self.bind_function_parameters(function, &mut body, &mut environment);
        if !function.pattern_identities.is_empty() {
            self.emit_pattern_identity_guards(function, &mut body);
        }
        self.current_result_captures
            .clone_from(&function.result_captures);
        self.current_function_return_type = Some(return_type.clone());
        let result = self.emit_block(&function.body, &mut body, &mut environment);
        self.current_result_captures.clear();
        self.current_function_return_type = None;
        let location = self.debug.location(function.body.result.span, subprogram);
        if !function.result_captures.is_empty() {
            let source_result = self.emit_machine_operand(
                &result,
                &function.result_type,
                &mut body,
                function.body.result.span,
            );
            let mut aggregate = body.instruction(
                &format!("insertvalue {return_type} poison, {source_result}, 0"),
                function.body.result.span,
                &mut self.debug,
            );
            for (index, capture) in function.result_captures.iter().enumerate() {
                let storage_name = match &capture.value.kind {
                    CompilerExpressionKind::Local(storage_name)
                    | CompilerExpressionKind::InfinityLocal { storage_name, .. } => {
                        Some(storage_name)
                    }
                    _ => None,
                };
                let value = storage_name
                    .and_then(|storage_name| function_capture_value(&result, storage_name))
                    .unwrap_or_else(|| {
                        self.emit_expression(&capture.value, &mut body, &environment)
                    });
                let operand = self.emit_machine_operand(
                    &value,
                    &capture.value_type,
                    &mut body,
                    capture.value.span,
                );
                aggregate = body.instruction(
                    &format!(
                        "insertvalue {return_type} {aggregate}, {operand}, {}",
                        index + 1
                    ),
                    capture.value.span,
                    &mut self.debug,
                );
            }
            body.terminator(&format!("ret {return_type} {aggregate}"), location);
        } else {
            match result {
                LlValue::Unit => body.terminator("ret void", location),
                LlValue::StaticDisplay(_) => unreachable!("static Capability function result"),
                LlValue::Completed(value) | LlValue::Effect(value) => {
                    body.terminator(&format!("ret i8 {value}"), location);
                }
                LlValue::Boolean(value) => body.terminator(&format!("ret i1 {value}"), location),
                LlValue::Version { value, .. }
                | LlValue::SerializationStream { stream: value, .. }
                | LlValue::Task { value, .. }
                | LlValue::ExternalLocation { value, .. }
                | LlValue::Int(value)
                | LlValue::Modular { value, .. }
                | LlValue::Rational(value)
                | LlValue::Error(value)
                | LlValue::ErrorDomain(value)
                | LlValue::SourceLocation(value)
                | LlValue::String(value)
                | LlValue::Range { value, .. }
                | LlValue::Result { value, .. }
                | LlValue::Optional { value, .. }
                | LlValue::TraversalControl { value, .. }
                | LlValue::List { value, .. }
                | LlValue::Container { value, .. } => {
                    body.terminator(&format!("ret ptr {value}"), location);
                }
                LlValue::Comparison(value)
                | LlValue::ErrorCode(value)
                | LlValue::Enum { value, .. }
                | LlValue::Function { value, .. }
                | LlValue::Generator { value, .. } => {
                    body.terminator(&format!("ret i32 {value}"), location);
                }
                LlValue::Tuple(fields) => {
                    let CompilerType::Tuple(field_types) = &function.result_type else {
                        unreachable!("checked Tuple result retains its Tuple type")
                    };
                    let aggregate = self.emit_tuple_aggregate(
                        &fields,
                        field_types,
                        &mut body,
                        function.body.result.span,
                    );
                    body.terminator(
                        &format!("ret {} {aggregate}", llvm_value_type(&function.result_type)),
                        location,
                    );
                }
                LlValue::Record { fields, order } => {
                    let CompilerType::Record(field_types) = &function.result_type else {
                        unreachable!("checked Record result retains its Record type")
                    };
                    let aggregate = self.emit_record_aggregate(
                        &fields,
                        &order,
                        field_types,
                        &mut body,
                        function.body.result.span,
                    );
                    body.terminator(
                        &format!("ret {} {aggregate}", llvm_value_type(&function.result_type)),
                        location,
                    );
                }
                LlValue::Sum { .. } => {
                    let aggregate =
                        self.emit_sum_aggregate(&result, &mut body, function.body.result.span);
                    body.terminator(
                        &format!("ret {} {aggregate}", llvm_value_type(&function.result_type)),
                        location,
                    );
                }
            }
        }
        self.functions.push(format!(
            "define internal fastcc {return_type} @{}({parameters}) nounwind noinline !dbg !{subprogram} {{\n{}\n}}\n",
            function.symbol,
            body.render()
        ));
    }

    #[allow(clippy::too_many_lines)] // Keep parameter ABI, retained-value, and DWARF policy together in source order.
    fn bind_function_parameters(
        &mut self,
        function: &CompilerFunction,
        body: &mut FunctionBody,
        environment: &mut BTreeMap<String, LlValue>,
    ) {
        let carries_function_boundary = function
            .parameters
            .iter()
            .any(|parameter| compiler_type_contains_function(&parameter.value_type));
        for (index, parameter) in function.parameters.iter().enumerate() {
            let argument = format!("%arg{index}");
            if parameter.discarded {
                continue;
            }
            let value = match &parameter.value_type {
                CompilerType::Tuple(fields) => {
                    self.emit_tuple_extract(&argument, fields, body, parameter.span)
                }
                CompilerType::Record(fields) => {
                    self.emit_record_extract(&argument, fields, body, parameter.span)
                }
                CompilerType::Sum(sum) => {
                    self.emit_sum_extract(&argument, sum, body, parameter.span)
                }
                CompilerType::Function => LlValue::Enum {
                    value: argument.clone(),
                    enumeration: function_value_enumeration(self.program),
                },
                _ => function_parameter_value(&parameter.value_type, index),
            };
            if !parameter.source_visible {
                environment.insert(parameter.name.clone(), value);
                continue;
            }
            let variable = self.debug.parameter(
                &parameter.name,
                index + 1,
                parameter.span,
                &parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(parameter.span, body.subprogram);
            let retained_character_generator_source = matches!(
                parameter.value_type,
                CompilerType::String | CompilerType::Character
            ) && matches!(&function.result_type, CompilerType::Generator(generator)
            if generator.yield_type.as_ref() == &CompilerType::Character
                && generator.resume_type.as_ref() == &CompilerType::Unit
                && matches!(
                    generator.result_type.as_ref(),
                    CompilerType::Unit | CompilerType::Character
                ));
            let retained_value_generator_source = parameter.value_type == CompilerType::Int
                && matches!(&function.result_type, CompilerType::Generator(generator)
                        if generator.yield_type.as_ref() == &CompilerType::Int
                            && generator.resume_type.as_ref() == &CompilerType::Unit
                            && generator.result_type.as_ref() == &CompilerType::String)
                || matches!(
                    (&parameter.value_type, &function.result_type),
                    (
                        CompilerType::Optional(payload),
                        CompilerType::Generator(generator)
                    ) if matches!(payload.as_ref(), CompilerType::Tuple(fields)
                            if fields == &[CompilerType::Int, CompilerType::String])
                        && generator.yield_type.as_ref() == &parameter.value_type
                        && generator.resume_type.as_ref() == &CompilerType::Unit
                        && matches!(generator.result_type.as_ref(), CompilerType::Result(success)
                            if matches!(success.as_ref(), CompilerType::Tuple(fields)
                                if fields == &[CompilerType::Int, CompilerType::String]))
                )
                || matches!(
                    (&parameter.value_type, &function.result_type),
                    (CompilerType::List(element), CompilerType::Generator(generator))
                        if element.as_ref() == &CompilerType::Int
                            && generator.yield_type.as_ref() == &parameter.value_type
                            && generator.resume_type.as_ref() == &CompilerType::Unit
                            && generator.result_type.as_ref() == &parameter.value_type
                );
            let retained_unused_enum = matches!(parameter.value_type, CompilerType::Enum(_))
                && function.body.statements.is_empty()
                && matches!(function.body.result.kind, CompilerExpressionKind::Unit);
            let retained_context_capture = function_parameter_is_context_capture(parameter);
            if retained_character_generator_source
                || retained_value_generator_source
                || retained_unused_enum
                || retained_context_capture
                || (carries_function_boundary
                    && parameter.value_type.machine_scalar()
                    && parameter.value_type != CompilerType::Unit)
                || matches!(
                    parameter.value_type,
                    CompilerType::Scope
                        | CompilerType::Function
                        | CompilerType::Generator(_)
                        | CompilerType::Tuple(_)
                        | CompilerType::Record(_)
                        | CompilerType::Sum(_)
                )
            {
                Self::emit_parameter_debug_shadow(
                    &argument,
                    &parameter.value_type,
                    variable,
                    location,
                    body,
                );
            } else {
                body.debug_value(&value, variable, location);
            }
            environment.insert(parameter.name.clone(), value);
        }
    }

    fn emit_function_parameter_value(
        &mut self,
        parameter: &CompilerParameter,
        index: usize,
        body: &mut FunctionBody,
    ) -> LlValue {
        let argument = format!("%arg{index}");
        match &parameter.value_type {
            CompilerType::Tuple(fields) => {
                self.emit_tuple_extract(&argument, fields, body, parameter.span)
            }
            CompilerType::Record(fields) => {
                self.emit_record_extract(&argument, fields, body, parameter.span)
            }
            CompilerType::Sum(sum) => self.emit_sum_extract(&argument, sum, body, parameter.span),
            CompilerType::Function => LlValue::Enum {
                value: argument,
                enumeration: function_value_enumeration(self.program),
            },
            _ => function_parameter_value(&parameter.value_type, index),
        }
    }

    fn emit_pattern_identity_guards(
        &mut self,
        function: &CompilerFunction,
        body: &mut FunctionBody,
    ) {
        for identity in &function.pattern_identities {
            let first = &function.parameters[identity.first_parameter];
            let repeated = &function.parameters[identity.repeated_parameter];
            debug_assert_eq!(first.value_type, repeated.value_type);
            let first = self.emit_function_parameter_value(first, identity.first_parameter, body);
            let repeated =
                self.emit_function_parameter_value(repeated, identity.repeated_parameter, body);
            let equal = self.emit_equal(&first, &repeated, body, identity.span);
            let matched = body.label("pattern.identity.matched");
            let mismatch = body.label("pattern.identity.mismatch");
            let location = self.debug.location(identity.span, body.subprogram);
            body.terminator(
                &format!("br i1 {equal}, label %{matched}, label %{mismatch}"),
                location,
            );
            body.start_block(&mismatch);
            body.effect(
                "call void @topal.runtime.pattern.identity.fail()",
                identity.span,
                &mut self.debug,
            );
            body.terminator("unreachable", location);
            body.start_block(&matched);
        }
    }

    fn emit_tuple_aggregate(
        &mut self,
        fields: &[LlValue],
        field_types: &[CompilerType],
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(fields.len(), field_types.len());
        let aggregate_type = llvm_value_type(&CompilerType::Tuple(field_types.to_vec()));
        let mut aggregate = "poison".to_owned();
        for (index, (field, field_type)) in fields.iter().zip(field_types).enumerate() {
            let field = self.emit_machine_operand(field, field_type, body, span);
            aggregate = body.instruction(
                &format!("insertvalue {aggregate_type} {aggregate}, {field}, {index}"),
                span,
                &mut self.debug,
            );
        }
        aggregate
    }

    fn emit_machine_operand(
        &mut self,
        value: &LlValue,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        match (value, value_type) {
            (LlValue::Tuple(fields), CompilerType::Tuple(field_types)) => {
                let aggregate = self.emit_tuple_aggregate(fields, field_types, body, span);
                format!("{} {aggregate}", llvm_value_type(value_type))
            }
            (LlValue::Record { fields, order }, CompilerType::Record(field_types)) => {
                let aggregate = self.emit_record_aggregate(fields, order, field_types, body, span);
                format!("{} {aggregate}", llvm_value_type(value_type))
            }
            (LlValue::Sum { .. }, CompilerType::Sum(_)) => {
                let aggregate = self.emit_sum_aggregate(value, body, span);
                format!("{} {aggregate}", llvm_value_type(value_type))
            }
            _ => value.argument(),
        }
    }

    fn emit_tuple_extract(
        &mut self,
        aggregate: &str,
        field_types: &[CompilerType],
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let aggregate_type = llvm_value_type(&CompilerType::Tuple(field_types.to_vec()));
        LlValue::Tuple(
            field_types
                .iter()
                .enumerate()
                .map(|(index, field_type)| {
                    let field = body.instruction(
                        &format!("extractvalue {aggregate_type} {aggregate}, {index}"),
                        span,
                        &mut self.debug,
                    );
                    self.emit_extracted_machine_value(&field, field_type, body, span)
                })
                .collect(),
        )
    }

    fn emit_extracted_machine_value(
        &mut self,
        value: &str,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        match value_type {
            CompilerType::Tuple(fields) => self.emit_tuple_extract(value, fields, body, span),
            CompilerType::Record(fields) => self.emit_record_extract(value, fields, body, span),
            CompilerType::Sum(sum) => self.emit_sum_extract(value, sum, body, span),
            CompilerType::Function => LlValue::Enum {
                value: value.to_owned(),
                enumeration: function_value_enumeration(self.program),
            },
            _ => machine_value(value_type, value.to_owned()),
        }
    }

    fn emit_record_aggregate(
        &mut self,
        fields: &[(String, LlValue)],
        order: &[String],
        field_types: &[(String, CompilerType)],
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(fields.len(), field_types.len());
        debug_assert_eq!(order.len(), field_types.len());
        let value_type = CompilerType::Record(field_types.to_vec());
        let aggregate_type = llvm_value_type(&value_type);
        let mut aggregate = "poison".to_owned();
        for (index, (label, field_type)) in field_types.iter().enumerate() {
            let field = fields
                .iter()
                .find_map(|(candidate, value)| (candidate == label).then_some(value))
                .unwrap_or_else(|| panic!("checked Record value retains field `{label}`"));
            let field = self.emit_machine_operand(field, field_type, body, span);
            aggregate = body.instruction(
                &format!("insertvalue {aggregate_type} {aggregate}, {field}, {index}"),
                span,
                &mut self.debug,
            );
        }
        for (position, selector) in order.iter().enumerate() {
            aggregate = body.instruction(
                &format!(
                    "insertvalue {aggregate_type} {aggregate}, i32 {selector}, {}",
                    field_types.len() + position
                ),
                span,
                &mut self.debug,
            );
        }
        aggregate
    }

    fn emit_record_extract(
        &mut self,
        aggregate: &str,
        field_types: &[(String, CompilerType)],
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let aggregate_type = llvm_value_type(&CompilerType::Record(field_types.to_vec()));
        let fields = field_types
            .iter()
            .enumerate()
            .map(|(index, (label, field_type))| {
                let field = body.instruction(
                    &format!("extractvalue {aggregate_type} {aggregate}, {index}"),
                    span,
                    &mut self.debug,
                );
                let field = self.emit_extracted_machine_value(&field, field_type, body, span);
                (label.clone(), field)
            })
            .collect();
        let order = (0..field_types.len())
            .map(|position| {
                body.instruction(
                    &format!(
                        "extractvalue {aggregate_type} {aggregate}, {}",
                        field_types.len() + position
                    ),
                    span,
                    &mut self.debug,
                )
            })
            .collect();
        LlValue::Record { fields, order }
    }

    fn emit_sum_aggregate(
        &mut self,
        value: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        let LlValue::Sum { tag, payloads, sum } = value else {
            unreachable!("checked sum aggregate retains its sum value")
        };
        let value_type = CompilerType::Sum(sum.clone());
        let aggregate_type = llvm_value_type(&value_type);
        let mut aggregate = body.instruction(
            &format!("insertvalue {aggregate_type} poison, i32 {tag}, 0"),
            span,
            &mut self.debug,
        );
        let mut field_index = 1;
        for (alternative, payload) in sum.alternatives.iter().zip(payloads) {
            if let Some(payload_type) = &alternative.payload {
                let payload = payload
                    .as_deref()
                    .expect("every represented sum payload has a checked machine value");
                let operand = self.emit_machine_operand(payload, payload_type, body, span);
                aggregate = body.instruction(
                    &format!("insertvalue {aggregate_type} {aggregate}, {operand}, {field_index}"),
                    span,
                    &mut self.debug,
                );
                field_index += 1;
            }
        }
        aggregate
    }

    fn emit_sum_extract(
        &mut self,
        aggregate: &str,
        sum: &CompilerSumType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let aggregate_type = llvm_value_type(&CompilerType::Sum(sum.clone()));
        let tag = body.instruction(
            &format!("extractvalue {aggregate_type} {aggregate}, 0"),
            span,
            &mut self.debug,
        );
        let mut field_index = 1;
        let mut payloads = Vec::with_capacity(sum.alternatives.len());
        for alternative in &sum.alternatives {
            let payload = if let Some(payload_type) = &alternative.payload {
                let field = body.instruction(
                    &format!("extractvalue {aggregate_type} {aggregate}, {field_index}"),
                    span,
                    &mut self.debug,
                );
                field_index += 1;
                Some(Box::new(self.machine_or_aggregate_value(
                    payload_type,
                    &field,
                    body,
                    span,
                )))
            } else {
                None
            };
            payloads.push(payload);
        }
        LlValue::Sum {
            tag,
            payloads,
            sum: sum.clone(),
        }
    }

    fn machine_or_aggregate_value(
        &mut self,
        value_type: &CompilerType,
        value: &str,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        self.emit_extracted_machine_value(value, value_type, body, span)
    }

    fn emit_aggregate_debug_shadow(
        &mut self,
        aggregate: &str,
        value_type: &CompilerType,
        variable: usize,
        location: usize,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let llvm_type = llvm_value_type(value_type);
        let alignment = target_value_layout(value_type).alignment / 8;
        let address = body.instruction(
            &format!("alloca {llvm_type}, align {alignment}"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store {llvm_type} {aggregate}, ptr {address}, align {alignment}"),
            span,
            &mut self.debug,
        );
        body.debug_declare(&address, variable, location);
    }

    fn emit_parameter_debug_shadow(
        aggregate: &str,
        value_type: &CompilerType,
        variable: usize,
        location: usize,
        body: &mut FunctionBody,
    ) {
        let llvm_type = llvm_value_type(value_type);
        let alignment = target_value_layout(value_type).alignment / 8;
        let address =
            body.instruction_without_debug(&format!("alloca {llvm_type}, align {alignment}"));
        body.effect_without_debug(&format!(
            "store {llvm_type} {aggregate}, ptr {address}, align {alignment}"
        ));
        body.debug_declare(&address, variable, location);
    }

    fn emit_main(&mut self) {
        let parameter_types = Vec::new();
        let subprogram = self.debug.subprogram(
            "<top-level>",
            "topal.main",
            self.program.main.result.span,
            &CompilerType::Unit,
            &parameter_types,
        );
        let mut body = FunctionBody::new(subprogram);
        let mut environment = BTreeMap::new();
        let result = self.emit_block(&self.program.main, &mut body, &mut environment);
        self.emit_print(&result, &mut body, self.program.main.result.span);
        self.emit_write_literal("\n", &mut body, self.program.main.result.span);
        let location = self
            .debug
            .location(self.program.main.result.span, subprogram);
        body.terminator("ret void", location);
        self.functions.push(format!(
            "define internal void @topal.main() nounwind noinline !dbg !{subprogram} {{\n{}\n}}\n",
            body.render()
        ));
    }

    #[allow(clippy::too_many_lines)] // Keep binding/debug policy ordered beside source statements.
    fn emit_block(
        &mut self,
        block: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &mut BTreeMap<String, LlValue>,
    ) -> LlValue {
        for statement in &block.statements {
            match statement {
                CompilerStatement::Binding(binding) => {
                    let value = self.emit_expression(&binding.value, body, environment);
                    if let CompilerType::Refined { base, .. } = &binding.value.value_type {
                        let variable = self.debug.local(
                            &binding.name,
                            binding.span,
                            &binding.value.value_type,
                            body.subprogram,
                        );
                        let location = self.debug.location(binding.span, body.subprogram);
                        let machine_value = match base.as_ref() {
                            CompilerType::Int | CompilerType::Nat => value.integer(),
                            _ => unreachable!("checked refined debug base is supported"),
                        };
                        self.emit_aggregate_debug_shadow(
                            machine_value,
                            &binding.value.value_type,
                            variable,
                            location,
                            body,
                            binding.span,
                        );
                    } else if matches!(
                        binding.value.value_type,
                        CompilerType::Array { .. }
                            | CompilerType::Set(_)
                            | CompilerType::Bag(_)
                            | CompilerType::Map { .. }
                    ) {
                        let variable = self.debug.local(
                            &binding.name,
                            binding.span,
                            &binding.value.value_type,
                            body.subprogram,
                        );
                        let location = self.debug.location(binding.span, body.subprogram);
                        self.emit_aggregate_debug_shadow(
                            value.container_pointer(),
                            &binding.value.value_type,
                            variable,
                            location,
                            body,
                            binding.span,
                        );
                    } else if matches!(
                        binding.value.value_type,
                        CompilerType::List(ref element)
                            if !matches!(element.as_ref(), CompilerType::Int)
                    ) {
                        let variable = self.debug.local(
                            &binding.name,
                            binding.span,
                            &binding.value.value_type,
                            body.subprogram,
                        );
                        let location = self.debug.location(binding.span, body.subprogram);
                        self.emit_aggregate_debug_shadow(
                            value.list_pointer(),
                            &binding.value.value_type,
                            variable,
                            location,
                            body,
                            binding.span,
                        );
                    } else if binding.value.value_type == CompilerType::Function
                        && matches!(&binding.value.kind, CompilerExpressionKind::Call { .. })
                    {
                        let variable = self.debug.local(
                            &binding.name,
                            binding.span,
                            &binding.value.value_type,
                            body.subprogram,
                        );
                        let location = self.debug.location(binding.span, body.subprogram);
                        self.emit_aggregate_debug_shadow(
                            value.enumeration(),
                            &binding.value.value_type,
                            variable,
                            location,
                            body,
                            binding.span,
                        );
                    } else if binding.value.value_type.machine_scalar() {
                        let variable = self.debug.local(
                            &binding.name,
                            binding.span,
                            &binding.value.value_type,
                            body.subprogram,
                        );
                        let location = self.debug.location(binding.span, body.subprogram);
                        body.debug_value(&value, variable, location);
                    } else if private_aggregate_value_supported(&binding.value.value_type) {
                        let aggregate = match (&value, &binding.value.value_type) {
                            (LlValue::Tuple(fields), CompilerType::Tuple(field_types)) => {
                                self.emit_tuple_aggregate(fields, field_types, body, binding.span)
                            }
                            (
                                LlValue::Record { fields, order },
                                CompilerType::Record(field_types),
                            ) => self.emit_record_aggregate(
                                fields,
                                order,
                                field_types,
                                body,
                                binding.span,
                            ),
                            (LlValue::Sum { .. }, CompilerType::Sum(_)) => {
                                self.emit_sum_aggregate(&value, body, binding.span)
                            }
                            _ => unreachable!("checked aggregate binding retains its type"),
                        };
                        let variable = self.debug.local(
                            &binding.name,
                            binding.span,
                            &binding.value.value_type,
                            body.subprogram,
                        );
                        let location = self.debug.location(binding.span, body.subprogram);
                        self.emit_aggregate_debug_shadow(
                            &aggregate,
                            &binding.value.value_type,
                            variable,
                            location,
                            body,
                            binding.span,
                        );
                    }
                    extend_function_capture_environment(environment, &value);
                    environment.insert(binding.storage_name.clone(), value);
                }
                CompilerStatement::Discard(expression) => {
                    let _ = self.emit_expression(expression, body, environment);
                }
            }
        }
        self.emit_expression(&block.result, body, environment)
    }

    #[allow(clippy::if_not_else, clippy::too_many_lines)] // Captured calls precede exhaustive ordinary calls.
    fn emit_expression(
        &mut self,
        expression: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        match &expression.kind {
            CompilerExpressionKind::Unit
            | CompilerExpressionKind::Identity(_)
            | CompilerExpressionKind::TypeView(_)
            | CompilerExpressionKind::FunctionView(_)
            | CompilerExpressionKind::LanguageContext(_)
            | CompilerExpressionKind::NativeSerializer(_)
            | CompilerExpressionKind::ExternalMetadata(_) => LlValue::Unit,
            CompilerExpressionKind::Infinity { negative } => {
                let direction = if *negative { "negative" } else { "positive" };
                if expression.value_type == CompilerType::InfiniteRational {
                    LlValue::Rational(body.instruction(
                        &format!(
                            "call ptr @topal.runtime.rational.from.int(ptr @topal.runtime.int.{direction}.infinity)"
                        ),
                        expression.span,
                        &mut self.debug,
                    ))
                } else {
                    LlValue::Int(format!("@topal.runtime.int.{direction}.infinity"))
                }
            }
            CompilerExpressionKind::ExternalLocationConstruct(location) => {
                let range_start = self.emit_int_literal(&location.offset.offset_type.range.lower);
                let offset = self.emit_int_literal(&location.offset.offset);
                LlValue::ExternalLocation {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.location.make(ptr {}, ptr {})",
                            range_start.integer(),
                            offset.integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    location: location.location_type.clone(),
                }
            }
            CompilerExpressionKind::ExternalLocationWrite {
                location, value, ..
            } => {
                let location = self.emit_expression(location, body, environment);
                let value = self.emit_expression(value, body, environment);
                body.effect(
                    &format!(
                        "call void @topal.runtime.location.write(ptr {}, ptr {})",
                        location.location_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::Unit
            }
            CompilerExpressionKind::ExternalLocationRead { location, .. } => {
                let location = self.emit_expression(location, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.location.read(ptr {})",
                        location.location_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::Capability(capability) => {
                LlValue::StaticDisplay(capability.display())
            }
            CompilerExpressionKind::Serialize { bytes, value } => {
                let value = self.emit_expression(value, body, environment);
                let (expected, byte_count) = self.emit_raw_bytes_global(bytes);
                let stream = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.serialization.make(ptr {expected}, i64 {byte_count})"
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::SerializationStream {
                    stream,
                    expected,
                    byte_count: byte_count.to_string(),
                    value: Box::new(value),
                }
            }
            CompilerExpressionKind::Deserialize(stream) => {
                let stream = self.emit_expression(stream, body, environment);
                let LlValue::SerializationStream {
                    stream,
                    expected,
                    byte_count,
                    value,
                } = stream
                else {
                    unreachable!("checked deserialize operand retains its native stream")
                };
                body.effect(
                    &format!(
                        "call void @topal.runtime.serialization.verify(ptr {stream}, ptr {expected}, i64 {byte_count})"
                    ),
                    expression.span,
                    &mut self.debug,
                );
                *value
            }
            CompilerExpressionKind::TaskConstruct { task, initial } => {
                let initial = self.emit_expression(initial, body, environment);
                LlValue::Task {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.task.make(ptr {})",
                            initial.integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    task: task.clone(),
                }
            }
            CompilerExpressionKind::TaskStateLoad { task, .. } => {
                let task = self.emit_expression(task, body, environment);
                let state = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.task.state.load(ptr {})",
                        task.task_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                if let CompilerType::TaskResponse(success) = &expression.value_type {
                    debug_assert_eq!(success.as_ref(), &CompilerType::Nat);
                    LlValue::Result {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.result.success(ptr {state})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        success: success.as_ref().clone(),
                        function_captures: Vec::new(),
                    }
                } else {
                    LlValue::Int(state)
                }
            }
            CompilerExpressionKind::TaskStateReplace { task, value, .. } => {
                let task = self.emit_expression(task, body, environment);
                let value = self.emit_expression(value, body, environment);
                body.effect(
                    &format!(
                        "call void @topal.runtime.task.state.replace(ptr {}, ptr {})",
                        task.task_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::Unit
            }
            CompilerExpressionKind::Completed => LlValue::Completed("0".into()),
            CompilerExpressionKind::Effect => LlValue::Effect("0".into()),
            CompilerExpressionKind::TypeValue(value) => LlValue::Enum {
                value: value.to_string(),
                enumeration: fundamental_type_enumeration(),
            },
            CompilerExpressionKind::Root => LlValue::Enum {
                value: "0".into(),
                enumeration: scope_enumeration(),
            },
            CompilerExpressionKind::LintNamespace => LlValue::Enum {
                value: "1".into(),
                enumeration: scope_enumeration(),
            },
            CompilerExpressionKind::FunctionValue(value) => LlValue::Enum {
                value: value.to_string(),
                enumeration: function_value_enumeration(self.program),
            },
            CompilerExpressionKind::ConstraintValue(value) => LlValue::Enum {
                value: value.to_string(),
                enumeration: constraint_value_enumeration(self.program),
            },
            CompilerExpressionKind::Boolean(value) => LlValue::Boolean(value.to_string()),
            CompilerExpressionKind::Version(value) => self.emit_version_literal(
                [value.major, value.minor, value.patch, value.build],
                value.to_string(),
                body,
                expression.span,
            ),
            CompilerExpressionKind::Int(value) => self.emit_int_literal(value),
            CompilerExpressionKind::IntToModular { value, modular } => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Modular {
                    value: value.integer().to_owned(),
                    modular: modular.clone(),
                }
            }
            CompilerExpressionKind::ModularReduce { value, modular } => {
                let value = self.emit_expression(value, body, environment);
                self.emit_modular_reduce(value.integer(), modular, body, expression.span)
            }
            CompilerExpressionKind::ModularValidate {
                value,
                modular,
                error_span,
            } => self.emit_modular_validation(
                value,
                modular,
                *error_span,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::Rational(value) => {
                self.emit_rational_literal(value, body, expression.span)
            }
            CompilerExpressionKind::String(value) => {
                self.emit_string_literal(value, body, expression.span)
            }
            CompilerExpressionKind::StringEmpty => {
                LlValue::String(self.emit_string_value("", body, expression.span))
            }
            CompilerExpressionKind::StringConcat { left, right } => {
                let left = self.emit_expression(left, body, environment);
                let right = self.emit_expression(right, body, environment);
                LlValue::String(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.string.concat(ptr {}, ptr {})",
                        left.string(),
                        right.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringEmptyPredicate(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.string.is.empty(ptr {})",
                        value.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringUtf8ByteCount(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.string.utf8.byte.count(ptr {})",
                        value.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringCharactersGenerator { text, .. } => {
                let _ = self.emit_expression(text, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked characters construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::StringCharactersCollect { text, .. } => {
                self.emit_expression(text, body, environment)
            }
            CompilerExpressionKind::StringCharactersClose(generator)
            | CompilerExpressionKind::CustomCharacterClose { generator, .. } => {
                let _ = self.emit_expression(generator, body, environment);
                LlValue::Unit
            }
            CompilerExpressionKind::CustomCharacterHandledClose { generator, handler } => self
                .emit_custom_character_handled_close(
                    generator,
                    handler,
                    body,
                    environment,
                    expression.span,
                ),
            CompilerExpressionKind::StringCharactersForeach { .. } => {
                self.emit_string_characters_foreach(expression, body, environment)
            }
            CompilerExpressionKind::CustomCharacterGenerator {
                declaration_span,
                initial_parameter,
                initial,
                prefix,
                ..
            } => {
                let initial = self.emit_expression(initial, body, environment);
                if !prefix.statements.is_empty() {
                    let parent_scope = body.subprogram;
                    body.subprogram = self.debug.lexical_block(*declaration_span, parent_scope);
                    let variable = self.debug.local(
                        &initial_parameter.name,
                        initial_parameter.span,
                        &initial_parameter.value_type,
                        body.subprogram,
                    );
                    let location = self.debug.location(initial_parameter.span, body.subprogram);
                    body.debug_value(&initial, variable, location);
                    let mut prefix_environment = environment.clone();
                    prefix_environment.insert(initial_parameter.name.clone(), initial);
                    let prefix_result = self.emit_block(prefix, body, &mut prefix_environment);
                    debug_assert!(matches!(prefix_result, LlValue::Unit));
                    body.subprogram = parent_scope;
                }
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked custom construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::CustomValueGenerator {
                initial,
                additional_initials,
                ..
            } => {
                let initial = self.emit_expression(initial, body, environment);
                let additional_initials = additional_initials
                    .iter()
                    .map(|initial| self.emit_expression(initial, body, environment))
                    .collect();
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked custom construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: Some(Box::new(initial)),
                    captured_additional_initials: additional_initials,
                }
            }
            CompilerExpressionKind::CustomCharacterForeach {
                source,
                declaration_span,
                characters,
                locals,
                parameter,
                body: action,
                result,
            } => {
                let parent_scope = body.subprogram;
                if !locals.is_empty() {
                    body.subprogram = self.debug.lexical_block(*declaration_span, parent_scope);
                }
                let _ = self.emit_character_sequence_foreach(
                    source,
                    characters,
                    locals.first(),
                    parameter,
                    action,
                    body,
                    environment,
                    expression.span,
                );
                let value = self.emit_expression(result, body, environment);
                body.subprogram = parent_scope;
                value
            }
            CompilerExpressionKind::CustomValueForeach { .. } => {
                self.emit_custom_value_foreach(expression, body, environment)
            }
            CompilerExpressionKind::ErrorCode(value) => LlValue::ErrorCode(value.to_string()),
            CompilerExpressionKind::Enum(value) => {
                let CompilerType::Enum(enumeration) = &expression.value_type else {
                    unreachable!("checked Enum value retains its nominal type")
                };
                LlValue::Enum {
                    value: value.to_string(),
                    enumeration: enumeration.clone(),
                }
            }
            CompilerExpressionKind::IterateGenerator { initial, .. } => {
                let _ = self.emit_expression(initial, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked iterate construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::GeneratorTakeWhile { generator, .. } => {
                let _ = self.emit_expression(generator, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked take-while retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::UnfoldGenerator { seed, .. } => {
                let _ = self.emit_expression(seed, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked unfold construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::IterateGeneratorForeach {
                generator,
                parameter,
                body: action,
            } => self.emit_iterate_foreach(
                generator,
                parameter,
                action,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::GeneratorCollect(generator) => {
                if matches!(
                    generator.kind,
                    CompilerExpressionKind::UnfoldGenerator { .. }
                ) {
                    self.emit_unfold_collect(generator, body, environment, expression.span)
                } else {
                    self.emit_iterate_collect(generator, body, environment, expression.span)
                }
            }
            CompilerExpressionKind::Sum { value, payload } => {
                let CompilerType::Sum(sum) = &expression.value_type else {
                    unreachable!("checked sum value retains its nominal type")
                };
                let mut payloads = sum
                    .alternatives
                    .iter()
                    .map(|alternative| {
                        alternative
                            .payload
                            .as_ref()
                            .map(|payload_type| Box::new(zero_machine_value(payload_type)))
                    })
                    .collect::<Vec<_>>();
                if let Some(payload) = payload {
                    payloads[usize::try_from(*value).expect("u32 sum tag fits usize")] =
                        Some(Box::new(self.emit_expression(payload, body, environment)));
                }
                LlValue::Sum {
                    tag: value.to_string(),
                    payloads,
                    sum: sum.clone(),
                }
            }
            CompilerExpressionKind::Tuple(values) => LlValue::Tuple(
                values
                    .iter()
                    .map(|value| self.emit_expression(value, body, environment))
                    .collect(),
            ),
            CompilerExpressionKind::TupleField { tuple, index } => {
                let LlValue::Tuple(values) = self.emit_expression(tuple, body, environment) else {
                    unreachable!("checked tuple field has a Tuple receiver")
                };
                values
                    .into_iter()
                    .nth(*index)
                    .expect("checked tuple field index remains in bounds")
            }
            CompilerExpressionKind::Record(fields) => {
                let mut fields = fields
                    .iter()
                    .map(|(label, value)| {
                        (
                            label.clone(),
                            self.emit_expression(value, body, environment),
                        )
                    })
                    .collect::<Vec<_>>();
                let canonical_labels = match &expression.value_type {
                    CompilerType::Record(field_types) => field_types
                        .iter()
                        .map(|(label, _)| label.as_str())
                        .collect::<Vec<_>>(),
                    _ => unreachable!("checked Record expression retains its Record type"),
                };
                let order = fields
                    .iter()
                    .map(|(label, _)| {
                        canonical_labels
                            .iter()
                            .position(|candidate| candidate == label)
                            .expect("checked Record type retains every value label")
                            .to_string()
                    })
                    .collect();
                fields.sort_by(|left, right| left.0.cmp(&right.0));
                LlValue::Record { fields, order }
            }
            CompilerExpressionKind::RecordReconstruct { base, replacements } => {
                let LlValue::Record { mut fields, order } =
                    self.emit_expression(base, body, environment)
                else {
                    unreachable!("checked reconstruction base has a Record type")
                };
                for (label, replacement) in replacements {
                    let replacement = self.emit_expression(replacement, body, environment);
                    let (_, value) = fields
                        .iter_mut()
                        .find(|(name, _)| name == label)
                        .unwrap_or_else(|| panic!("checked Record retains field `{label}`"));
                    *value = replacement;
                }
                LlValue::Record { fields, order }
            }
            CompilerExpressionKind::RecordField { record, label } => {
                let LlValue::Record { fields, .. } =
                    self.emit_expression(record, body, environment)
                else {
                    unreachable!("checked field selection has a Record receiver")
                };
                fields
                    .into_iter()
                    .find_map(|(name, value)| (name == *label).then_some(value))
                    .unwrap_or_else(|| panic!("checked Record retains field `{label}`"))
            }
            CompilerExpressionKind::Block(block) => {
                let parent_scope = body.subprogram;
                body.subprogram = self.debug.lexical_block(expression.span, parent_scope);
                let mut nested = environment.clone();
                let value = self.emit_block(block, body, &mut nested);
                body.subprogram = parent_scope;
                value
            }
            CompilerExpressionKind::PrivateBinding {
                storage_name,
                value,
                body: result,
            } => {
                let value = self.emit_expression(value, body, environment);
                let mut nested = environment.clone();
                extend_function_capture_environment(&mut nested, &value);
                nested.insert(storage_name.clone(), value);
                self.emit_expression(result, body, &nested)
            }
            CompilerExpressionKind::Local(name)
            | CompilerExpressionKind::InfinityLocal {
                storage_name: name, ..
            } => environment
                .get(name)
                .unwrap_or_else(|| panic!("checked local `{name}` remains available"))
                .clone(),
            CompilerExpressionKind::Negate(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                match emitted {
                    LlValue::Int(operand) => LlValue::Int(body.instruction(
                        &format!("call ptr @topal.runtime.int.negate(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    LlValue::Rational(operand) => LlValue::Rational(body.instruction(
                        &format!("call ptr @topal.runtime.rational.negate(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    LlValue::Modular { value, modular } => {
                        let value = body.instruction(
                            &format!("call ptr @topal.runtime.int.negate(ptr {value})"),
                            expression.span,
                            &mut self.debug,
                        );
                        self.emit_modular_reduce(&value, &modular, body, expression.span)
                    }
                    _ => unreachable!("checked negate operand is exact numeric"),
                }
            }
            CompilerExpressionKind::Absolute(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                match emitted {
                    LlValue::Int(operand) => LlValue::Int(body.instruction(
                        &format!("call ptr @topal.runtime.int.absolute(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    LlValue::Rational(operand) => LlValue::Rational(body.instruction(
                        &format!("call ptr @topal.runtime.rational.absolute(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    _ => unreachable!("checked absolute operand is exact numeric"),
                }
            }
            CompilerExpressionKind::IntToRational(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.from.int(ptr {})",
                        emitted.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::RationalConstruct {
                numerator,
                denominator,
            } => {
                let numerator = self.emit_expression(numerator, body, environment);
                let denominator = self.emit_expression(denominator, body, environment);
                LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.make(ptr {}, ptr {})",
                        numerator.integer(),
                        denominator.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::RationalToInt(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.numerator(ptr {})",
                        value.rational()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::IntToNat(value)
            | CompilerExpressionKind::ExternalLayoutCoerce { value, .. } => {
                self.emit_expression(value, body, environment)
            }
            CompilerExpressionKind::ResultSuccess(value) => {
                let success = self.emit_expression(value, body, environment);
                let function_captures = match &success {
                    LlValue::Function { captures, .. } => captures.clone(),
                    _ => Vec::new(),
                };
                let payload =
                    self.emit_result_payload(&success, &value.value_type, body, value.span);
                LlValue::Result {
                    value: body.instruction(
                        &format!("call ptr @topal.runtime.result.success(ptr {payload})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    success: value.value_type.clone(),
                    function_captures,
                }
            }
            CompilerExpressionKind::ResultProject(value) => {
                self.emit_result_project(value, body, environment, expression.span)
            }
            CompilerExpressionKind::OptionalSome(value) => {
                let payload_value = self.emit_expression(value, body, environment);
                let function_captures = match &payload_value {
                    LlValue::Function { captures, .. } => captures.clone(),
                    _ => Vec::new(),
                };
                let payload = self.emit_optional_payload_pointer(
                    &payload_value,
                    &value.value_type,
                    body,
                    value.span,
                );
                LlValue::Optional {
                    value: body.instruction(
                        &format!("call ptr @topal.runtime.optional.some(ptr {payload})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: value.value_type.clone(),
                    function_captures,
                }
            }
            CompilerExpressionKind::OptionalNone => {
                let CompilerType::Optional(payload) = &expression.value_type else {
                    unreachable!("checked None retains its Optional classifier")
                };
                LlValue::Optional {
                    value: body.instruction(
                        "call ptr @topal.runtime.optional.none()",
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: payload.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::TraversalControl { finish, value } => {
                let payload = self.emit_expression(value, body, environment);
                let control = body.instruction(
                    "call ptr @topal.platform.allocate(i64 16)",
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store i64 {}, ptr {control}, align 8", u8::from(*finish)),
                    expression.span,
                    &mut self.debug,
                );
                let payload_address = body.instruction(
                    &format!("getelementptr i8, ptr {control}, i64 8"),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!(
                        "store ptr {}, ptr {payload_address}, align 8",
                        payload.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::TraversalControl {
                    value: control,
                    payload: CompilerType::Int,
                }
            }
            CompilerExpressionKind::ListEmpty => {
                let CompilerType::List(element) = &expression.value_type else {
                    unreachable!("checked Empty retains its List classifier")
                };
                LlValue::List {
                    value: "null".into(),
                    element: element.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListEntry { value, remaining } => {
                let value = self.emit_expression(value, body, environment);
                let remaining = self.emit_expression(remaining, body, environment);
                let LlValue::List {
                    value: remaining,
                    element,
                    mut function_captures,
                } = remaining
                else {
                    unreachable!("checked Entry tail retains its List classifier")
                };
                let (allocation_size, next_offset) = match &element {
                    CompilerType::Effect
                    | CompilerType::Boolean
                    | CompilerType::Character
                    | CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Rational
                    | CompilerType::String
                    | CompilerType::Function => (16, 8),
                    CompilerType::Tuple(fields)
                        if matches!(
                            fields.as_slice(),
                            [CompilerType::Int, CompilerType::Int | CompilerType::String,]
                                | [
                                    CompilerType::String,
                                    CompilerType::Int | CompilerType::Function
                                ]
                        ) =>
                    {
                        (24, 16)
                    }
                    CompilerType::List(inner) if compiler_int_string_pair(inner) => (16, 8),
                    _ => unreachable!("checked List element has an admitted node layout"),
                };
                let node = body.instruction(
                    &format!("call ptr @topal.platform.allocate(i64 {allocation_size})"),
                    expression.span,
                    &mut self.debug,
                );
                match (&element, &value) {
                    (CompilerType::Effect, LlValue::Effect(value)) => body.effect(
                        &format!("store i8 {value}, ptr {node}, align 1"),
                        expression.span,
                        &mut self.debug,
                    ),
                    (CompilerType::Boolean, LlValue::Boolean(value)) => body.effect(
                        &format!("store i1 {value}, ptr {node}, align 1"),
                        expression.span,
                        &mut self.debug,
                    ),
                    (CompilerType::Int | CompilerType::Nat, LlValue::Int(value))
                    | (CompilerType::Rational, LlValue::Rational(value))
                    | (CompilerType::Character | CompilerType::String, LlValue::String(value)) => {
                        body.effect(
                            &format!("store ptr {value}, ptr {node}, align 8"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (
                        CompilerType::Function,
                        LlValue::Function { value, .. } | LlValue::Enum { value, .. },
                    ) => body.effect(
                        &format!("store i32 {value}, ptr {node}, align 4"),
                        expression.span,
                        &mut self.debug,
                    ),
                    (CompilerType::Tuple(field_types), LlValue::Tuple(values))
                        if matches!(
                            field_types.as_slice(),
                            [CompilerType::Int, CompilerType::Int | CompilerType::String,]
                                | [
                                    CompilerType::String,
                                    CompilerType::Int | CompilerType::Function
                                ]
                        ) =>
                    {
                        let [left, right] = values.as_slice() else {
                            unreachable!("checked List pair value retains two fields")
                        };
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {node}, align 8",
                                left.int_or_string_pointer()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                        let right_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 8"),
                            expression.span,
                            &mut self.debug,
                        );
                        match right {
                            LlValue::Function { value, .. } | LlValue::Enum { value, .. } => {
                                body.effect(
                                    &format!("store i32 {value}, ptr {right_address}, align 4"),
                                    expression.span,
                                    &mut self.debug,
                                );
                            }
                            _ => body.effect(
                                &format!(
                                    "store ptr {}, ptr {right_address}, align 8",
                                    right.int_or_string_pointer()
                                ),
                                expression.span,
                                &mut self.debug,
                            ),
                        }
                    }
                    (
                        CompilerType::List(inner),
                        LlValue::List {
                            value,
                            element: value_element,
                            ..
                        },
                    ) if compiler_int_string_pair(inner) && value_element == inner.as_ref() => {
                        body.effect(
                            &format!("store ptr {value}, ptr {node}, align 8"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    _ => unreachable!("checked List element has an admitted node layout"),
                }
                let next = body.instruction(
                    &format!("getelementptr i8, ptr {node}, i64 {next_offset}"),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {remaining}, ptr {next}, align 8"),
                    expression.span,
                    &mut self.debug,
                );
                if element == CompilerType::Function || compiler_string_function_pair(&element) {
                    let captures = match value {
                        LlValue::Function { captures, .. } => captures,
                        LlValue::Enum { .. } => Vec::new(),
                        LlValue::Tuple(mut values) if compiler_string_function_pair(&element) => {
                            match values.pop().expect("checked Map pair has a Function value") {
                                LlValue::Function { captures, .. } => captures,
                                LlValue::Enum { .. } => Vec::new(),
                                _ => unreachable!(
                                    "checked Map pair retains callable captures in its value"
                                ),
                            }
                        }
                        _ => unreachable!("checked List entry retains callable captures"),
                    };
                    function_captures.insert(0, captures);
                }
                LlValue::List {
                    value: node,
                    element,
                    function_captures,
                }
            }
            CompilerExpressionKind::ListContainsEntry { list, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Containment);
                let list = self.emit_expression(list, body, environment);
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.list.int.contains.entry(ptr {}, ptr {})",
                        list.list_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListContainsSequence { list, pattern }
            | CompilerExpressionKind::ListContainsSubsequence { list, pattern } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Containment);
                let operation = if matches!(
                    &expression.kind,
                    CompilerExpressionKind::ListContainsSequence { .. }
                ) {
                    "sequence"
                } else {
                    "subsequence"
                };
                let list = self.emit_expression(list, body, environment);
                let pattern = self.emit_expression(pattern, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.list.int.contains.{operation}(ptr {}, ptr {})",
                        list.list_pointer(),
                        pattern.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListRemoveFirst { list, value }
            | CompilerExpressionKind::ListRemoveAll { list, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Removal);
                let operation = if matches!(
                    &expression.kind,
                    CompilerExpressionKind::ListRemoveFirst { .. }
                ) {
                    "first"
                } else {
                    "all"
                };
                let list = self.emit_expression(list, body, environment);
                let LlValue::List {
                    value: list,
                    element,
                    ..
                } = list
                else {
                    unreachable!("checked removal subject is a List")
                };
                let value = self.emit_expression(value, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.remove.{operation}(ptr {}, ptr {})",
                            list,
                            value.integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListPrepend { list, value } => {
                let list = self.emit_expression(list, body, environment);
                let value = self.emit_expression(value, body, environment);
                let node = body.instruction(
                    "call ptr @topal.platform.allocate(i64 16)",
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {node}, align 8", value.integer()),
                    expression.span,
                    &mut self.debug,
                );
                let next = body.instruction(
                    &format!("getelementptr i8, ptr {node}, i64 8"),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {next}, align 8", list.list_pointer()),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::List {
                    value: node,
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListAppend { list, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                let list = self.emit_expression(list, body, environment);
                let value = self.emit_expression(value, body, environment);
                let singleton = body.instruction(
                    "call ptr @topal.platform.allocate(i64 16)",
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {singleton}, align 8", value.integer()),
                    expression.span,
                    &mut self.debug,
                );
                let next = body.instruction(
                    &format!("getelementptr i8, ptr {singleton}, i64 8"),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr null, ptr {next}, align 8"),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.concat(ptr {}, ptr {singleton})",
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListConcat { left, right } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                let left = self.emit_expression(left, body, environment);
                let right = self.emit_expression(right, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.concat(ptr {}, ptr {})",
                            left.list_pointer(),
                            right.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListReverse(value) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                let value = self.emit_expression(value, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.reverse(ptr {})",
                            value.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListEntryCount(value) => {
                let boolean = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Boolean
                );
                let string = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if matches!(element.as_ref(), CompilerType::Character | CompilerType::String)
                );
                let rational = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Rational
                );
                let nested_int_string = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_nested_int_string_list_element(element)
                );
                if boolean {
                    self.pointer_list_runtime_fragments
                        .insert(PointerListRuntimeFragment::Boolean);
                } else if rational {
                    self.pointer_list_runtime_fragments
                        .insert(PointerListRuntimeFragment::Rational);
                } else if string {
                    self.pointer_list_runtime_fragments
                        .insert(PointerListRuntimeFragment::String);
                } else {
                    self.list_int_runtime_fragments
                        .insert(if nested_int_string {
                            ListIntRuntimeFragment::NestedIntStringCore
                        } else {
                            ListIntRuntimeFragment::Core
                        });
                }
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.{}.entry.count(ptr {})",
                        if boolean {
                            "boolean"
                        } else if rational {
                            "rational"
                        } else if string {
                            "string"
                        } else if nested_int_string {
                            "nested.int-string"
                        } else {
                            "int"
                        },
                        value.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListEmptyPredicate(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!("icmp eq ptr {}, null", value.list_pointer()),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListFirst(value)
            | CompilerExpressionKind::ListRest(value)
            | CompilerExpressionKind::ListUncons(value) => {
                let nested_int_string = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_nested_int_string_list_element(element)
                );
                self.list_int_runtime_fragments
                    .insert(if nested_int_string {
                        ListIntRuntimeFragment::NestedIntStringCore
                    } else {
                        ListIntRuntimeFragment::Core
                    });
                let operation = match &expression.kind {
                    CompilerExpressionKind::ListFirst(_) => "first",
                    CompilerExpressionKind::ListRest(_) => "rest",
                    CompilerExpressionKind::ListUncons(_) => "uncons",
                    _ => unreachable!(),
                };
                let value = self.emit_expression(value, body, environment);
                let CompilerType::Optional(payload) = &expression.value_type else {
                    unreachable!("checked List projection returns Optional")
                };
                LlValue::Optional {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.{}.{operation}(ptr {})",
                            if nested_int_string {
                                "nested.int-string"
                            } else {
                                "int"
                            },
                            value.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: payload.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListMap {
                list,
                parameters,
                body: action,
            } => self.emit_list_map(list, parameters, action, body, environment, expression.span),
            CompilerExpressionKind::ListSelect {
                list,
                parameters,
                body: predicate,
            } => self.emit_list_select(
                list,
                parameters,
                predicate,
                body,
                environment,
                expression.span,
                false,
                false,
            ),
            CompilerExpressionKind::ListRangeSelect {
                list,
                range,
                indexes,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::RangeSelection);
                let list = self.emit_expression(list, body, environment);
                let range = self.emit_expression(range, body, environment);
                let operation = if *indexes { "index" } else { "value" };
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.select.{operation}.range(ptr {}, ptr {})",
                            list.list_pointer(),
                            range.range().0
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListForeach {
                list,
                parameter,
                body: action,
            } => {
                self.emit_list_foreach(list, parameter, action, body, environment, expression.span)
            }
            CompilerExpressionKind::ListInsertAt {
                list,
                boundary,
                inserted,
                inserts_list,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                let inserted = self.emit_expression(inserted, body, environment);
                let inserted = if *inserts_list {
                    inserted.list_pointer().to_owned()
                } else {
                    let singleton = body.instruction(
                        "call ptr @topal.platform.allocate(i64 16)",
                        expression.span,
                        &mut self.debug,
                    );
                    body.effect(
                        &format!("store ptr {}, ptr {singleton}, align 8", inserted.integer()),
                        expression.span,
                        &mut self.debug,
                    );
                    let next = body.instruction(
                        &format!("getelementptr i8, ptr {singleton}, i64 8"),
                        expression.span,
                        &mut self.debug,
                    );
                    body.effect(
                        &format!("store ptr null, ptr {next}, align 8"),
                        expression.span,
                        &mut self.debug,
                    );
                    singleton
                };
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.insert.at(ptr {}, i64 {boundary}, ptr {inserted})",
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListIndexOperation {
                list,
                index,
                operation,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                let operation_name = match operation {
                    CompilerListIndexOperation::Split => "split.at",
                    CompilerListIndexOperation::Take => "take",
                    CompilerListIndexOperation::Drop => "drop",
                    CompilerListIndexOperation::Remove => "remove.index",
                };
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.int.{operation_name}(ptr {}, i64 {index})",
                        list.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                if *operation == CompilerListIndexOperation::Split {
                    let suffix_address = body.instruction(
                        &format!("getelementptr i8, ptr {value}, i64 8"),
                        expression.span,
                        &mut self.debug,
                    );
                    LlValue::Tuple(vec![
                        LlValue::List {
                            value: body.instruction(
                                &format!("load ptr, ptr {value}, align 8"),
                                expression.span,
                                &mut self.debug,
                            ),
                            element: CompilerType::Int,
                            function_captures: Vec::new(),
                        },
                        LlValue::List {
                            value: body.instruction(
                                &format!("load ptr, ptr {suffix_address}, align 8"),
                                expression.span,
                                &mut self.debug,
                            ),
                            element: CompilerType::Int,
                            function_captures: Vec::new(),
                        },
                    ])
                } else {
                    LlValue::List {
                        value,
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    }
                }
            }
            CompilerExpressionKind::ListRemoveIndexRange { list, start, end } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.remove.index.range(ptr {}, i64 {start}, i64 {end})",
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListReject {
                list,
                parameters,
                predicate,
                indexes,
            } => self.emit_list_select(
                list,
                parameters,
                predicate,
                body,
                environment,
                expression.span,
                true,
                *indexes,
            ),
            CompilerExpressionKind::ListZip {
                left,
                right,
                operation,
                left_default,
                right_default,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let left = self.emit_expression(left, body, environment);
                let left_default = left_default
                    .as_deref()
                    .map(|value| self.emit_expression(value, body, environment));
                let right = self.emit_expression(right, body, environment);
                let right_default = right_default
                    .as_deref()
                    .map(|value| self.emit_expression(value, body, environment));
                let pair = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]);
                match operation {
                    CompilerListZipOperation::Exact => {
                        let domain = self.emit_string_value(
                            "root.zip-exact(List,List)",
                            body,
                            expression.span,
                        );
                        let source =
                            self.emit_string_value(self.source_name, body, expression.span);
                        let position = self.program.source.position(expression.span.start);
                        LlValue::Result {
                            value: body.instruction(
                                &format!(
                                    "call ptr @topal.runtime.list.int.zip.exact(ptr {}, ptr {}, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                                    left.list_pointer(),
                                    right.list_pointer(),
                                    position.line,
                                    position.column
                                ),
                                expression.span,
                                &mut self.debug,
                            ),
                            success: CompilerType::List(Box::new(pair)),
                            function_captures: Vec::new(),
                        }
                    }
                    CompilerListZipOperation::Shortest => LlValue::List {
                        value: body.instruction(
                            &format!(
                                "call ptr @topal.runtime.list.int.zip.shortest(ptr {}, ptr {})",
                                left.list_pointer(),
                                right.list_pointer()
                            ),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: pair,
                        function_captures: Vec::new(),
                    },
                    CompilerListZipOperation::Longest => LlValue::List {
                        value: body.instruction(
                            &format!(
                                "call ptr @topal.runtime.list.int.zip.longest(ptr {}, ptr {}, ptr {}, ptr {})",
                                left.list_pointer(),
                                left_default
                                    .as_ref()
                                    .expect("checked longest zip has left default")
                                    .integer(),
                                right.list_pointer(),
                                right_default
                                    .as_ref()
                                    .expect("checked longest zip has right default")
                                    .integer()
                            ),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: pair,
                        function_captures: Vec::new(),
                    },
                }
            }
            CompilerExpressionKind::ListUnzip(pairs) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let pairs = self.emit_expression(pairs, body, environment);
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.int.unzip(ptr {})",
                        pairs.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                let right_address = body.instruction(
                    &format!("getelementptr i8, ptr {value}, i64 8"),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![
                    LlValue::List {
                        value: body.instruction(
                            &format!("load ptr, ptr {value}, align 8"),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    },
                    LlValue::List {
                        value: body.instruction(
                            &format!("load ptr, ptr {right_address}, align 8"),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    },
                ])
            }
            CompilerExpressionKind::ListEntries(list) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.entries(ptr {})",
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Record(vec![
                        ("index".into(), CompilerType::Int),
                        ("value".into(), CompilerType::Int),
                    ]),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListCollectString(list) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                let empty = self.emit_string_value("", body, expression.span);
                LlValue::String(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.string.collect(ptr {}, ptr {empty})",
                        list.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ContainerCollect {
                source,
                kind,
                map_policy,
                map_keys,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let source = self.emit_expression(source, body, environment);
                let (function_captures, function_capture_keys) =
                    match (&expression.value_type, &source) {
                        (
                            CompilerType::Array { element, .. },
                            LlValue::List {
                                function_captures, ..
                            },
                        ) if element.as_ref() == &CompilerType::Function => {
                            (function_captures.clone(), Vec::new())
                        }
                        (
                            CompilerType::Map { key, value },
                            LlValue::List {
                                function_captures, ..
                            },
                        ) if key.as_ref() == &CompilerType::String
                            && value.as_ref() == &CompilerType::Function =>
                        {
                            let keys = map_keys
                                .as_ref()
                                .expect("checked Map Function collection retains exact keys");
                            debug_assert_eq!(keys.len(), function_captures.len());
                            let mut positions: BTreeMap<String, usize> = BTreeMap::new();
                            let mut unique_keys = Vec::new();
                            let mut unique_captures: Vec<Vec<(String, LlValue)>> = Vec::new();
                            for (key, captures) in keys.iter().zip(function_captures) {
                                if let Some(index) = positions.get(key).copied() {
                                    if matches!(
                                        map_policy,
                                        Some(CompilerMapCollisionPolicy::KeepLast)
                                    ) {
                                        unique_captures[index].clone_from(captures);
                                    }
                                } else {
                                    positions.insert(key.clone(), unique_keys.len());
                                    unique_keys.push(key.clone());
                                    unique_captures.push(captures.clone());
                                }
                            }
                            (unique_captures, unique_keys)
                        }
                        _ => (Vec::new(), Vec::new()),
                    };
                let call = match kind {
                    CompilerContainerKind::Array
                        if matches!(
                            &expression.value_type,
                            CompilerType::Array { element, .. }
                                if element.as_ref() == &CompilerType::Function
                        ) =>
                    {
                        format!(
                            "call ptr @topal.runtime.container.array.function.collect(ptr {})",
                            source.list_pointer()
                        )
                    }
                    CompilerContainerKind::Array => {
                        format!(
                            "call ptr @topal.runtime.container.array.int.collect(ptr {})",
                            source.list_pointer()
                        )
                    }
                    CompilerContainerKind::Set => format!(
                        "call ptr @topal.runtime.container.set.int.collect(ptr {})",
                        source.list_pointer()
                    ),
                    CompilerContainerKind::Bag => format!(
                        "call ptr @topal.runtime.container.bag.int.collect(ptr {})",
                        source.list_pointer()
                    ),
                    CompilerContainerKind::Map => {
                        let keep_last =
                            matches!(map_policy, Some(CompilerMapCollisionPolicy::KeepLast));
                        let helper = if matches!(
                            &expression.value_type,
                            CompilerType::Map { key, value }
                                if key.as_ref() == &CompilerType::String
                                    && value.as_ref() == &CompilerType::Function
                        ) {
                            "topal.runtime.container.map.string-function.collect"
                        } else {
                            "topal.runtime.container.map.string-int.collect"
                        };
                        format!(
                            "call ptr @{helper}(ptr {}, i1 {keep_last})",
                            source.list_pointer()
                        )
                    }
                };
                LlValue::Container {
                    value: body.instruction(&call, expression.span, &mut self.debug),
                    value_type: expression.value_type.clone(),
                    function_captures,
                    function_capture_keys,
                }
            }
            CompilerExpressionKind::ContainerEntryCount(container) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let container = self.emit_expression(container, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.container.entry.count(ptr {})",
                        container.container_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ContainerEmpty(container) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let container = self.emit_expression(container, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.container.empty(ptr {})",
                        container.container_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ArrayAt { array, index } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let array = self.emit_expression(array, body, environment);
                let LlValue::Container {
                    value: array,
                    value_type,
                    function_captures,
                    ..
                } = array
                else {
                    unreachable!("checked Array access retains its container value")
                };
                let CompilerType::Array { element, .. } = value_type else {
                    unreachable!("checked Array access retains its Array classifier")
                };
                let helper = match element.as_ref() {
                    CompilerType::Int => "topal.runtime.container.array.int.at",
                    CompilerType::Function => "topal.runtime.container.array.function.at",
                    _ => unreachable!("checked Array access has an admitted element classifier"),
                };
                let captures = if element.as_ref() == &CompilerType::Function {
                    function_captures.get(*index).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                };
                LlValue::Optional {
                    value: body.instruction(
                        &format!("call ptr @{helper}(ptr {array}, i64 {index})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: element.as_ref().clone(),
                    function_captures: captures,
                }
            }
            CompilerExpressionKind::SetContains { set, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let set = self.emit_expression(set, body, environment);
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.container.set.int.contains(ptr {}, ptr {})",
                        set.container_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::BagMultiplicity { bag, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let bag = self.emit_expression(bag, body, environment);
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.container.bag.int.multiplicity(ptr {}, ptr {})",
                        bag.container_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::MapLookup {
                mapping,
                key,
                exact_key,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let mapping = self.emit_expression(mapping, body, environment);
                let key = self.emit_expression(key, body, environment);
                let LlValue::Container {
                    value: mapping,
                    value_type,
                    function_captures,
                    function_capture_keys,
                } = mapping
                else {
                    unreachable!("checked Map lookup retains its container value")
                };
                let CompilerType::Map {
                    key: map_key,
                    value: map_value,
                } = value_type
                else {
                    unreachable!("checked Map lookup retains its Map classifier")
                };
                debug_assert_eq!(map_key.as_ref(), &CompilerType::String);
                let helper = match map_value.as_ref() {
                    CompilerType::Int => "topal.runtime.container.map.string-int.lookup",
                    CompilerType::Function => "topal.runtime.container.map.string-function.lookup",
                    _ => unreachable!("checked Map lookup has an admitted value classifier"),
                };
                let captures = if map_value.as_ref() == &CompilerType::Function {
                    let exact_key = exact_key
                        .as_ref()
                        .expect("checked Map Function lookup retains an exact key");
                    function_capture_keys
                        .iter()
                        .position(|candidate| candidate == exact_key)
                        .and_then(|index| function_captures.get(index).cloned())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                LlValue::Optional {
                    value: body.instruction(
                        &format!("call ptr @{helper}(ptr {mapping}, ptr {})", key.string()),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: map_value.as_ref().clone(),
                    function_captures: captures,
                }
            }
            CompilerExpressionKind::ListFold {
                list,
                initial,
                parameters,
                body: action,
            } => self.emit_list_fold(
                list,
                initial,
                parameters,
                action,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ErrorField { error, field } => {
                let error_span = error.span;
                let error = self.emit_expression(error, body, environment);
                let error = match error {
                    LlValue::Error(error) => error,
                    LlValue::Result { value, .. } => {
                        let is_error = body.instruction(
                            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
                            error_span,
                            &mut self.debug,
                        );
                        let valid = body.label("error.field.valid");
                        let invalid = body.label("error.field.invalid");
                        let location = self.debug.location(error_span, body.subprogram);
                        body.terminator(
                            &format!("br i1 {is_error}, label %{valid}, label %{invalid}"),
                            location,
                        );
                        body.start_block(&invalid);
                        body.effect(
                            "call void @topal.platform.exit(i64 70)",
                            error_span,
                            &mut self.debug,
                        );
                        body.terminator("unreachable", location);
                        body.start_block(&valid);
                        body.instruction(
                            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
                            error_span,
                            &mut self.debug,
                        )
                    }
                    _ => unreachable!("checked field subject contains an Error"),
                };
                match field {
                    CompilerErrorField::Code => LlValue::ErrorCode(body.instruction(
                        &format!("call i32 @topal.runtime.error.code(ptr {error})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    CompilerErrorField::Domain => LlValue::ErrorDomain(body.instruction(
                        &format!("call ptr @topal.runtime.error.domain(ptr {error})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    CompilerErrorField::Detail => LlValue::Optional {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.error.detail(ptr {error})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::String,
                        function_captures: Vec::new(),
                    },
                    CompilerErrorField::Cause => LlValue::Optional {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.error.cause(ptr {error})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::Error,
                        function_captures: Vec::new(),
                    },
                    CompilerErrorField::Source => LlValue::Optional {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.error.source(ptr {error})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::SourceLocation,
                        function_captures: Vec::new(),
                    },
                }
            }
            CompilerExpressionKind::Validate {
                operation,
                value,
                error_span,
            } => self.emit_validation(
                *operation,
                value,
                *error_span,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::Fallible {
                operation,
                left,
                right,
                error_span,
            } => self.emit_fallible(
                *operation,
                left,
                right,
                *error_span,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::RangeLower(operand)
            | CompilerExpressionKind::RangeUpper(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let operation = if matches!(&expression.kind, CompilerExpressionKind::RangeLower(_))
                {
                    "lower"
                } else {
                    "upper"
                };
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.range.{operation}(ptr {})",
                        emitted.range().0
                    ),
                    expression.span,
                    &mut self.debug,
                );
                match expression.value_type {
                    CompilerType::Int | CompilerType::InfiniteInt | CompilerType::InfiniteNat => {
                        LlValue::Int(value)
                    }
                    CompilerType::Rational | CompilerType::InfiniteRational => {
                        LlValue::Rational(value)
                    }
                    _ => unreachable!("checked Range bound retains its endpoint type"),
                }
            }
            CompilerExpressionKind::RangeLowerInclusive(operand)
            | CompilerExpressionKind::RangeUpperInclusive(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let operation = if matches!(
                    &expression.kind,
                    CompilerExpressionKind::RangeLowerInclusive(_)
                ) {
                    "lower.inclusive"
                } else {
                    "upper.inclusive"
                };
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.range.{operation}(ptr {})",
                        emitted.range().0
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::RangeEmpty(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let (value, endpoint) = emitted.range();
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.range.{}.empty(ptr {value})",
                        numeric_domain(endpoint)
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::Not(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let operand = emitted.boolean();
                LlValue::Boolean(body.instruction(
                    &format!("xor i1 {operand}, true"),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::Binary {
                operation,
                left,
                right,
            } => {
                let left = self.emit_expression(left, body, environment);
                let right = self.emit_expression(right, body, environment);
                self.emit_binary(*operation, &left, &right, body, expression.span)
            }
            CompilerExpressionKind::Call { symbol, arguments } => {
                let mut argument_environment = environment.clone();
                let values = arguments
                    .iter()
                    .map(|argument| {
                        let value = self.emit_expression(argument, body, &argument_environment);
                        extend_function_capture_environment(&mut argument_environment, &value);
                        value
                    })
                    .collect::<Vec<_>>();
                let target = self
                    .program
                    .functions
                    .iter()
                    .find(|function| function.symbol == *symbol)
                    .expect("checked call target has a generated function");
                let parameter_types = target
                    .parameters
                    .iter()
                    .map(|parameter| parameter.value_type.clone())
                    .collect::<Vec<_>>();
                let result_captures = target.result_captures.clone();
                let function_return_type = function_llvm_return_type(target);
                debug_assert_eq!(values.len(), parameter_types.len());
                let mut machine_arguments = Vec::with_capacity(values.len());
                for (value, value_type) in values.iter().zip(&parameter_types) {
                    machine_arguments.push(self.emit_machine_operand(
                        value,
                        value_type,
                        body,
                        expression.span,
                    ));
                }
                let arguments = machine_arguments.join(", ");
                if !result_captures.is_empty() {
                    let aggregate = body.instruction(
                        &format!("call fastcc {function_return_type} @{symbol}({arguments})"),
                        expression.span,
                        &mut self.debug,
                    );
                    let source_value = body.instruction(
                        &format!("extractvalue {function_return_type} {aggregate}, 0"),
                        expression.span,
                        &mut self.debug,
                    );
                    let mut result = self.emit_extracted_machine_value(
                        &source_value,
                        &expression.value_type,
                        body,
                        expression.span,
                    );
                    for (index, capture) in result_captures.iter().enumerate() {
                        let value = body.instruction(
                            &format!(
                                "extractvalue {function_return_type} {aggregate}, {}",
                                index + 1
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                        let value = self.emit_extracted_machine_value(
                            &value,
                            &capture.value_type,
                            body,
                            expression.span,
                        );
                        attach_function_capture(
                            &mut result,
                            &capture.path,
                            compiler_function_result_capture_storage(
                                symbol,
                                expression.span,
                                index,
                            ),
                            value,
                        );
                    }
                    result
                } else {
                    match expression.value_type {
                        CompilerType::Unit => {
                            body.effect(
                                &format!("call fastcc void @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            LlValue::Unit
                        }
                        CompilerType::Completed => LlValue::Completed(body.instruction(
                            &format!("call fastcc i8 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Effect => LlValue::Effect(body.instruction(
                            &format!("call fastcc i8 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Type => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: fundamental_type_enumeration(),
                        },
                        CompilerType::Scope => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: scope_enumeration(),
                        },
                        CompilerType::Function => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: function_value_enumeration(self.program),
                        },
                        CompilerType::Identity
                        | CompilerType::TypeView
                        | CompilerType::FunctionView
                        | CompilerType::LanguageContext
                        | CompilerType::Capability
                        | CompilerType::NativeSerializer(_)
                        | CompilerType::ExternalMetadata
                        | CompilerType::SerializationStream(_)
                        | CompilerType::Constraint
                        | CompilerType::Refined { .. }
                        | CompilerType::TraversalControl(_)
                        | CompilerType::TaskResponse(_)
                        | CompilerType::Task(_)
                        | CompilerType::ExternalLocation(_) => {
                            unreachable!("checked functions do not return this static object kind")
                        }
                        CompilerType::Generator(ref generator) => LlValue::Generator {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            generator: generator.clone(),
                            captured_initial: None,
                            captured_additional_initials: Vec::new(),
                        },
                        CompilerType::Version => {
                            unreachable!("Version function results are not admitted")
                        }
                        CompilerType::Boolean => LlValue::Boolean(body.instruction(
                            &format!("call fastcc i1 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Int | CompilerType::Nat => LlValue::Int(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::InfiniteInt
                        | CompilerType::InfiniteNat
                        | CompilerType::InfiniteRational => {
                            unreachable!("checked functions do not cross infinity values")
                        }
                        CompilerType::Modular(ref modular) => LlValue::Modular {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            modular: modular.clone(),
                        },
                        CompilerType::Rational => LlValue::Rational(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Comparison => LlValue::Comparison(body.instruction(
                            &format!("call fastcc i32 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::ErrorCode => LlValue::ErrorCode(body.instruction(
                            &format!("call fastcc i32 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Enum(ref enumeration) => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: enumeration.clone(),
                        },
                        CompilerType::Error => LlValue::Error(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::ErrorDomain => LlValue::ErrorDomain(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::SourceLocation => LlValue::SourceLocation(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Character | CompilerType::String => {
                            LlValue::String(body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ))
                        }
                        CompilerType::Range(ref endpoint) => LlValue::Range {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            endpoint: endpoint.as_ref().clone(),
                        },
                        CompilerType::Result(ref success) => LlValue::Result {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            success: success.as_ref().clone(),
                            function_captures: Vec::new(),
                        },
                        CompilerType::Optional(ref payload) => LlValue::Optional {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            payload: payload.as_ref().clone(),
                            function_captures: Vec::new(),
                        },
                        CompilerType::List(ref element) => LlValue::List {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            element: element.as_ref().clone(),
                            function_captures: Vec::new(),
                        },
                        CompilerType::Array { .. }
                        | CompilerType::Set(_)
                        | CompilerType::Bag(_)
                        | CompilerType::Map { .. } => LlValue::Container {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            value_type: expression.value_type.clone(),
                            function_captures: Vec::new(),
                            function_capture_keys: Vec::new(),
                        },
                        CompilerType::Tuple(ref field_types) => {
                            let aggregate_type = llvm_value_type(&expression.value_type);
                            let aggregate = body.instruction(
                                &format!("call fastcc {aggregate_type} @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            self.emit_tuple_extract(&aggregate, field_types, body, expression.span)
                        }
                        CompilerType::Record(ref field_types) => {
                            let aggregate_type = llvm_value_type(&expression.value_type);
                            let aggregate = body.instruction(
                                &format!("call fastcc {aggregate_type} @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            self.emit_record_extract(&aggregate, field_types, body, expression.span)
                        }
                        CompilerType::Sum(ref sum) => {
                            let aggregate_type = llvm_value_type(&expression.value_type);
                            let aggregate = body.instruction(
                                &format!("call fastcc {aggregate_type} @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            self.emit_sum_extract(&aggregate, sum, body, expression.span)
                        }
                    }
                }
            }
            CompilerExpressionKind::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => self.emit_boolean_decision(
                subject,
                when_true,
                when_false,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::OrderedComparisonDecision {
                subject,
                rules,
                otherwise,
            } => self.emit_ordered_comparison_decision(
                subject,
                rules,
                otherwise,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ComparisonValueDecision {
                subject,
                when_less,
                when_equal,
                when_greater,
            } => self.emit_comparison_value_decision(
                subject,
                when_less,
                when_equal,
                when_greater,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::EnumDecision {
                subject,
                rules,
                otherwise,
            } => self.emit_enum_decision(
                subject,
                rules,
                otherwise.as_deref(),
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::SumDecision {
                subject,
                rules,
                otherwise,
            } => self.emit_sum_decision(
                subject,
                rules,
                otherwise.as_deref(),
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ResultDecision {
                subject,
                ok_binding,
                ok_binding_span,
                ok_action,
                error_codes,
                error_fallback,
            } => self.emit_result_decision(
                subject,
                ok_binding,
                *ok_binding_span,
                ok_action,
                error_codes,
                error_fallback.as_ref(),
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::OptionalDecision {
                subject,
                some_binding,
                some_action,
                none_action,
            } => self.emit_optional_decision(
                subject,
                some_binding.as_ref(),
                some_action,
                none_action,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ListDecision {
                subject,
                entry_bindings,
                entry_action,
                empty_action,
            } => self.emit_list_decision(
                subject,
                entry_bindings.as_ref(),
                entry_action,
                empty_action,
                body,
                environment,
                expression.span,
            ),
        }
    }

    fn emit_custom_character_handled_close(
        &mut self,
        generator: &CompilerExpression,
        handler: &CompilerGeneratorCloseHandler,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let _ = self.emit_expression(generator, body, environment);
        let domain = self.emit_string_value("root", body, span);
        let source = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(span.start);
        let result_pointer = body.instruction(
            &format!(
                "call ptr @topal.runtime.result.failure(i32 0, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                position.line, position.column
            ),
            span,
            &mut self.debug,
        );
        let result_type = self.debug.result_unit_generator_error_type;
        let result_variable = self.debug.local_with_type_id(
            &handler.result_binding,
            handler.result_binding_span,
            result_type,
            body.subprogram,
        );
        let result_location = self
            .debug
            .location(handler.result_binding_span, body.subprogram);
        let result_debug_address = body.instruction(
            "alloca ptr, align 8",
            handler.result_binding_span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {result_pointer}, ptr {result_debug_address}, align 8"),
            handler.result_binding_span,
            &mut self.debug,
        );
        body.debug_declare(&result_debug_address, result_variable, result_location);

        let error_pointer = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {result_pointer})"),
            handler.result_binding_span,
            &mut self.debug,
        );
        if let Some(rule) = handler.error_codes.iter().find(|rule| rule.code == 0) {
            let _ = body.instruction(
                &format!("call i32 @topal.runtime.error.code(ptr {error_pointer})"),
                rule.action.span,
                &mut self.debug,
            );
            let result = self.emit_expression(&rule.action, body, environment);
            debug_assert!(matches!(result, LlValue::Unit));
            return result;
        }
        let error = LlValue::Error(error_pointer);
        let error_type = self.debug.generator_error_type;
        let error_variable = self.debug.local_with_type_id(
            &handler.error_binding,
            handler.error_binding_span,
            error_type,
            body.subprogram,
        );
        let error_location = self
            .debug
            .location(handler.error_binding_span, body.subprogram);
        body.debug_value(&error, error_variable, error_location);
        let _ = body.instruction(
            &format!("call i32 @topal.runtime.error.code(ptr {})", error.error()),
            handler.error_action.span,
            &mut self.debug,
        );
        let mut handler_environment = environment.clone();
        handler_environment.insert(handler.error_binding.clone(), error);
        let result = self.emit_expression(&handler.error_action, body, &handler_environment);
        debug_assert!(matches!(result, LlValue::Unit));
        result
    }

    fn emit_result_project(
        &mut self,
        value: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let result = self.emit_expression(value, body, environment);
        let LlValue::Result {
            value,
            success,
            function_captures,
        } = result
        else {
            unreachable!("checked projection operand is Result")
        };
        let is_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
            span,
            &mut self.debug,
        );
        let failure = body.label("result.project.error");
        let success_label = body.label("result.project.ok");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_error}, label %{failure}, label %{success_label}"),
            location,
        );
        body.start_block(&failure);
        if self.current_result_captures.is_empty() {
            body.terminator(&format!("ret ptr {value}"), location);
        } else {
            let return_type = self
                .current_function_return_type
                .clone()
                .expect("function emission retains its LLVM return type");
            let mut aggregate = body.instruction(
                &format!("insertvalue {return_type} poison, ptr {value}, 0"),
                span,
                &mut self.debug,
            );
            for (index, capture) in self.current_result_captures.clone().iter().enumerate() {
                let zero = zero_machine_value(&capture.value_type);
                let operand = self.emit_machine_operand(&zero, &capture.value_type, body, span);
                aggregate = body.instruction(
                    &format!(
                        "insertvalue {return_type} {aggregate}, {operand}, {}",
                        index + 1
                    ),
                    span,
                    &mut self.debug,
                );
            }
            body.terminator(&format!("ret {return_type} {aggregate}"), location);
        }
        body.start_block(&success_label);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            span,
            &mut self.debug,
        );
        let mut success_value = self.result_success_value(&payload, &success, body, span);
        if let LlValue::Function { captures, .. } = &mut success_value {
            *captures = function_captures;
        }
        success_value
    }

    #[allow(clippy::too_many_arguments)] // Optional binding and delayed alternatives stay explicit.
    fn emit_optional_decision(
        &mut self,
        subject: &CompilerExpression,
        some_binding: Option<&(String, Span)>,
        some_action: &CompilerExpression,
        none_action: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let optional = self.emit_expression(subject, body, environment);
        let LlValue::Optional {
            value,
            payload,
            function_captures,
        } = optional
        else {
            unreachable!("checked Optional decision subject is Optional")
        };
        let is_some = body.instruction(
            &format!("call i1 @topal.runtime.optional.is.some(ptr {value})"),
            subject.span,
            &mut self.debug,
        );
        let some_label = body.label("optional.decision.some");
        let none_label = body.label("optional.decision.none");
        let merge = body.label("optional.decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_some}, label %{some_label}, label %{none_label}"),
            location,
        );

        body.start_block(&some_label);
        let mut some_environment = environment.clone();
        if let Some((some_binding, some_binding_span)) = some_binding {
            let payload_pointer = body.instruction(
                &format!("call ptr @topal.runtime.optional.payload(ptr {value})"),
                *some_binding_span,
                &mut self.debug,
            );
            let payload_value = self.emit_optional_payload_value(
                payload_pointer,
                &payload,
                body,
                *some_binding_span,
            );
            let payload_value = if let LlValue::Function {
                value, enumeration, ..
            } = payload_value
            {
                LlValue::Function {
                    value,
                    enumeration,
                    captures: function_captures.clone(),
                }
            } else {
                payload_value
            };
            let variable =
                self.debug
                    .local(some_binding, *some_binding_span, &payload, body.subprogram);
            let binding_location = self.debug.location(*some_binding_span, body.subprogram);
            body.debug_value(&payload_value, variable, binding_location);
            some_environment.insert(some_binding.into(), payload_value);
        }
        let some_value = self.emit_expression(some_action, body, &some_environment);
        let some_predecessor = body.current_block.clone();
        let some_location = self.debug.location(some_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), some_location);

        body.start_block(&none_label);
        let none_value = self.emit_expression(none_action, body, environment);
        let none_predecessor = body.current_block.clone();
        let none_location = self.debug.location(none_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), none_location);

        body.start_block(&merge);
        self.emit_decision_phi(
            &[
                (some_value, some_predecessor),
                (none_value, none_predecessor),
            ],
            body,
            span,
        )
    }

    fn emit_optional_payload_value(
        &mut self,
        value: String,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        match value_type {
            CompilerType::List(element) => LlValue::List {
                value,
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            },
            CompilerType::Enum(enumeration) => LlValue::Enum {
                value: body.instruction(
                    &format!("load i32, ptr {value}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: enumeration.clone(),
            },
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {value}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            CompilerType::Tuple(fields)
                if fields.as_slice() == [CompilerType::Int, CompilerType::String] =>
            {
                let integer = body.instruction(
                    &format!("load ptr, ptr {value}, align 8"),
                    span,
                    &mut self.debug,
                );
                let text_address = body.instruction(
                    &format!("getelementptr i8, ptr {value}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let text = body.instruction(
                    &format!("load ptr, ptr {text_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![LlValue::Int(integer), LlValue::String(text)])
            }
            CompilerType::Tuple(fields)
                if matches!(
                    fields.as_slice(),
                    [CompilerType::Int, CompilerType::List(element)]
                        if element.as_ref() == &CompilerType::Int
                ) =>
            {
                let first = body.instruction(
                    &format!("load ptr, ptr {value}, align 8"),
                    span,
                    &mut self.debug,
                );
                let rest_address = body.instruction(
                    &format!("getelementptr i8, ptr {value}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let rest = body.instruction(
                    &format!("load ptr, ptr {rest_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![
                    LlValue::Int(first),
                    LlValue::List {
                        value: rest,
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    },
                ])
            }
            _ => optional_payload_value(value, value_type),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Keeps typed alternatives and scoped bindings together.
    fn emit_list_decision(
        &mut self,
        subject: &CompilerExpression,
        entry_bindings: Option<&ListEntryBindings>,
        entry_action: &CompilerExpression,
        empty_action: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let list = self.emit_expression(subject, body, environment);
        let LlValue::List {
            value: list,
            element,
            function_captures,
        } = list
        else {
            unreachable!("checked List decision subject is a List")
        };
        let is_empty = body.instruction(
            &format!("icmp eq ptr {list}, null"),
            subject.span,
            &mut self.debug,
        );
        let entry_label = body.label("list.decision.entry");
        let empty_label = body.label("list.decision.empty");
        let merge = body.label("list.decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_empty}, label %{empty_label}, label %{entry_label}"),
            location,
        );

        body.start_block(&entry_label);
        let mut entry_environment = environment.clone();
        if let Some(((first_name, first_span), (rest_name, rest_span))) = entry_bindings {
            let (first_value, rest_function_captures) = match &element {
                CompilerType::Boolean => (
                    LlValue::Boolean(body.instruction(
                        &format!("load i1, ptr {list}, align 1"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Character | CompilerType::String => (
                    LlValue::String(body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Int | CompilerType::Nat => (
                    LlValue::Int(body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Rational => (
                    LlValue::Rational(body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Function => {
                    let first_captures = function_captures.first().cloned().unwrap_or_default();
                    let remaining_captures =
                        function_captures.get(1..).unwrap_or_default().to_vec();
                    (
                        LlValue::Function {
                            value: body.instruction(
                                &format!("load i32, ptr {list}, align 4"),
                                *first_span,
                                &mut self.debug,
                            ),
                            enumeration: function_value_enumeration(self.program),
                            captures: first_captures,
                        },
                        remaining_captures,
                    )
                }
                _ => unreachable!("checked List decision has an admitted element type"),
            };
            let rest_address = body.instruction(
                &format!("getelementptr i8, ptr {list}, i64 8"),
                *rest_span,
                &mut self.debug,
            );
            let rest = body.instruction(
                &format!("load ptr, ptr {rest_address}, align 8"),
                *rest_span,
                &mut self.debug,
            );
            let rest_value = LlValue::List {
                value: rest,
                element: element.clone(),
                function_captures: rest_function_captures,
            };
            let first_variable =
                self.debug
                    .local(first_name, *first_span, &element, body.subprogram);
            let rest_type = CompilerType::List(Box::new(element.clone()));
            let rest_variable =
                self.debug
                    .local(rest_name, *rest_span, &rest_type, body.subprogram);
            let first_location = self.debug.location(*first_span, body.subprogram);
            let rest_location = self.debug.location(*rest_span, body.subprogram);
            body.debug_value(&first_value, first_variable, first_location);
            body.debug_value(&rest_value, rest_variable, rest_location);
            entry_environment.insert(first_name.clone(), first_value);
            entry_environment.insert(rest_name.clone(), rest_value);
        }
        let entry_value = self.emit_expression(entry_action, body, &entry_environment);
        let entry_predecessor = body.current_block.clone();
        let entry_location = self.debug.location(entry_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), entry_location);

        body.start_block(&empty_label);
        let empty_value = self.emit_expression(empty_action, body, environment);
        let empty_predecessor = body.current_block.clone();
        let empty_location = self.debug.location(empty_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), empty_location);

        body.start_block(&merge);
        self.emit_decision_phi(
            &[
                (entry_value, entry_predecessor),
                (empty_value, empty_predecessor),
            ],
            body,
            span,
        )
    }

    fn emit_collection_environment(
        &mut self,
        parameters: &[CompilerParameter],
        values: &[LlValue],
        body: &mut FunctionBody,
        outer: &BTreeMap<String, LlValue>,
    ) -> BTreeMap<String, LlValue> {
        debug_assert_eq!(parameters.len(), values.len());
        let mut environment = outer.clone();
        for (parameter, value) in parameters.iter().zip(values) {
            if parameter.discarded {
                continue;
            }
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(parameter.span, body.subprogram);
            body.debug_value(value, variable, location);
            environment.insert(parameter.name.clone(), value.clone());
        }
        environment
    }

    #[allow(clippy::too_many_lines)] // The loop keeps predicate, publication, and next ordering explicit.
    fn emit_iterate_collect(
        &mut self,
        generator: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let CompilerExpressionKind::GeneratorTakeWhile {
            generator,
            parameters: predicate_parameters,
            predicate,
        } = &generator.kind
        else {
            unreachable!("checked collect retains a bounded iterate generator")
        };
        let CompilerExpressionKind::IterateGenerator {
            initial,
            parameters: next_parameters,
            next,
        } = &generator.kind
        else {
            unreachable!("checked collect retains its direct iterate construction")
        };
        let initial = self
            .emit_expression(initial, body, environment)
            .integer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("generator.collect.loop");
        let accepted = body.label("generator.collect.accepted");
        let first = body.label("generator.collect.first");
        let link = body.label("generator.collect.link");
        let linked = body.label("generator.collect.linked");
        let done = body.label("generator.collect.done");
        let next_current = body.reserve_value();
        let node = body.reserve_value();
        let next_head = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{initial}, %{preheader}], [{next_current}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{node}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let mut predicate_environment = self.emit_collection_environment(
            predicate_parameters,
            &[LlValue::Int(current.clone())],
            body,
            environment,
        );
        let accepted_value = self
            .emit_block(predicate, body, &mut predicate_environment)
            .boolean()
            .to_owned();
        body.terminator(
            &format!("br i1 {accepted_value}, label %{accepted}, label %{done}"),
            location,
        );

        body.start_block(&accepted);
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {current}, ptr {node}, align 8"),
            span,
            &mut self.debug,
        );
        let node_next = body.instruction(
            &format!("getelementptr i8, ptr {node}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr null, ptr {node_next}, align 8"),
            span,
            &mut self.debug,
        );
        let mut next_environment = self.emit_collection_environment(
            next_parameters,
            &[LlValue::Int(current)],
            body,
            environment,
        );
        let next_value = self
            .emit_block(next, body, &mut next_environment)
            .integer()
            .to_owned();
        let has_previous = body.instruction(
            &format!("icmp ne ptr {previous}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {has_previous}, label %{link}, label %{first}"),
            location,
        );

        body.start_block(&first);
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&link);
        let previous_next = body.instruction(
            &format!("getelementptr i8, ptr {previous}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {node}, ptr {previous_next}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&linked);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_current,
            &format!("phi ptr [{next_value}, %{first}], [{next_value}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element: CompilerType::Int,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_lines)] // The loop keeps seed elimination and node publication explicit.
    fn emit_unfold_collect(
        &mut self,
        generator: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let CompilerExpressionKind::UnfoldGenerator {
            seed,
            parameters,
            step,
        } = &generator.kind
        else {
            unreachable!("checked collect retains its unfold construction")
        };
        debug_assert!(step.statements.is_empty());
        debug_assert!(matches!(
            step.result.kind,
            CompilerExpressionKind::ListUncons(_)
        ));
        let initial = self
            .emit_expression(seed, body, environment)
            .list_pointer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("generator.unfold.collect.loop");
        let some = body.label("generator.unfold.collect.some");
        let first = body.label("generator.unfold.collect.first");
        let link = body.label("generator.unfold.collect.link");
        let linked = body.label("generator.unfold.collect.linked");
        let done = body.label("generator.unfold.collect.done");
        let next_seed = body.reserve_value();
        let node = body.reserve_value();
        let next_head = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{initial}, %{preheader}], [{next_seed}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{node}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let _step_environment = self.emit_collection_environment(
            parameters,
            &[LlValue::List {
                value: current.clone(),
                element: CompilerType::Int,
                function_captures: Vec::new(),
            }],
            body,
            environment,
        );
        let has_value = body.instruction(
            &format!("icmp ne ptr {current}, null"),
            step.result.span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {has_value}, label %{some}, label %{done}"),
            location,
        );

        body.start_block(&some);
        let yielded = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            step.result.span,
            &mut self.debug,
        );
        let seed_next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            step.result.span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_seed,
            &format!("load ptr, ptr {seed_next_address}, align 8"),
            step.result.span,
            &mut self.debug,
        );
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {yielded}, ptr {node}, align 8"),
            span,
            &mut self.debug,
        );
        let node_next = body.instruction(
            &format!("getelementptr i8, ptr {node}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr null, ptr {node_next}, align 8"),
            span,
            &mut self.debug,
        );
        let has_previous = body.instruction(
            &format!("icmp ne ptr {previous}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {has_previous}, label %{link}, label %{first}"),
            location,
        );

        body.start_block(&first);
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&link);
        let previous_next = body.instruction(
            &format!("getelementptr i8, ptr {previous}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {node}, ptr {previous_next}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&linked);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element: CompilerType::Int,
            function_captures: Vec::new(),
        }
    }

    fn emit_iterate_foreach(
        &mut self,
        generator: &CompilerExpression,
        parameter: &CompilerParameter,
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let CompilerExpressionKind::GeneratorTakeWhile {
            generator,
            parameters: predicate_parameters,
            predicate,
        } = &generator.kind
        else {
            unreachable!("checked foreach retains a bounded iterate generator")
        };
        let CompilerExpressionKind::IterateGenerator {
            initial,
            parameters: next_parameters,
            next,
        } = &generator.kind
        else {
            unreachable!("checked foreach retains its iterate construction")
        };
        let initial = self
            .emit_expression(initial, body, environment)
            .integer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("generator.foreach.loop");
        let accepted = body.label("generator.foreach.accepted");
        let resumed = body.label("generator.foreach.resumed");
        let done = body.label("generator.foreach.done");
        let next_current = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{initial}, %{preheader}], [{next_current}, %{resumed}]"),
            span,
            &mut self.debug,
        );
        let mut predicate_environment = self.emit_collection_environment(
            predicate_parameters,
            &[LlValue::Int(current.clone())],
            body,
            environment,
        );
        let accepted_value = self
            .emit_block(predicate, body, &mut predicate_environment)
            .boolean()
            .to_owned();
        body.terminator(
            &format!("br i1 {accepted_value}, label %{accepted}, label %{done}"),
            location,
        );

        body.start_block(&accepted);
        let mut action_environment = environment.clone();
        if !parameter.discarded {
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &CompilerType::Int,
                body.subprogram,
            );
            let binding_location = self.debug.location(parameter.span, body.subprogram);
            let value = LlValue::Int(current.clone());
            body.debug_value(&value, variable, binding_location);
            action_environment.insert(parameter.name.clone(), value);
        }
        let action_value = self.emit_block(action, body, &mut action_environment);
        debug_assert!(matches!(action_value, LlValue::Unit));
        let mut next_environment = self.emit_collection_environment(
            next_parameters,
            &[LlValue::Int(current)],
            body,
            environment,
        );
        let next_value = self
            .emit_block(next, body, &mut next_environment)
            .integer()
            .to_owned();
        let next_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{resumed}"), location);

        body.start_block(&resumed);
        body.define_reserved(
            &next_current,
            &format!("phi ptr [{next_value}, %{next_predecessor}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::Unit
    }

    fn emit_list_foreach(
        &mut self,
        list: &CompilerExpression,
        parameter: &CompilerParameter,
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let source = self
            .emit_expression(list, body, environment)
            .list_pointer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.foreach.loop");
        let visit = body.label("list.foreach.visit");
        let resumed = body.label("list.foreach.resumed");
        let done = body.label("list.foreach.done");
        let next_current = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next_current}, %{resumed}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{visit}"),
            location,
        );

        body.start_block(&visit);
        let value = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            parameter.span,
            &mut self.debug,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        let next = body.instruction(
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let mut action_environment = environment.clone();
        if !parameter.discarded {
            let value = LlValue::Int(value);
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &CompilerType::Int,
                body.subprogram,
            );
            let binding_location = self.debug.location(parameter.span, body.subprogram);
            body.debug_value(&value, variable, binding_location);
            action_environment.insert(parameter.name.clone(), value);
        }
        let action_value = self.emit_block(action, body, &mut action_environment);
        debug_assert!(matches!(action_value, LlValue::Unit));
        let predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{resumed}"), location);

        body.start_block(&resumed);
        body.define_reserved(
            &next_current,
            &format!("phi ptr [{next}, %{predecessor}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::Unit
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // The loop keeps node publication explicit.
    fn emit_list_map(
        &mut self,
        list: &CompilerExpression,
        parameters: &[CompilerParameter],
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let LlValue::List {
            value: source,
            element,
            ..
        } = self.emit_expression(list, body, environment)
        else {
            unreachable!("checked map subject retains its List classifier")
        };
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.map.loop");
        let visit = body.label("list.map.visit");
        let first = body.label("list.map.first");
        let link = body.label("list.map.link");
        let linked = body.label("list.map.linked");
        let done = body.label("list.map.done");
        let next = body.reserve_value();
        let node = body.reserve_value();
        let next_head = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{node}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{visit}"),
            location,
        );

        body.start_block(&visit);
        let (values, next_offset) = match &element {
            CompilerType::Int => {
                let value = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                (vec![LlValue::Int(value)], 8)
            }
            CompilerType::Tuple(fields)
                if fields.as_slice() == [CompilerType::Int, CompilerType::Int] =>
            {
                let left = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                let right_address = body.instruction(
                    &format!("getelementptr i8, ptr {current}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let right = body.instruction(
                    &format!("load ptr, ptr {right_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                (vec![LlValue::Int(left), LlValue::Int(right)], 16)
            }
            _ => unreachable!("checked map subject has an admitted List element layout"),
        };
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 {next_offset}"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let mut action_environment =
            self.emit_collection_environment(parameters, &values, body, environment);
        let mapped = self.emit_block(action, body, &mut action_environment);
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {}, ptr {node}, align 8", mapped.integer()),
            span,
            &mut self.debug,
        );
        let node_next = body.instruction(
            &format!("getelementptr i8, ptr {node}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr null, ptr {node_next}, align 8"),
            span,
            &mut self.debug,
        );
        let has_previous = body.instruction(
            &format!("icmp ne ptr {previous}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {has_previous}, label %{link}, label %{first}"),
            location,
        );

        body.start_block(&first);
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&link);
        let previous_next = body.instruction(
            &format!("getelementptr i8, ptr {previous}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {node}, ptr {previous_next}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&linked);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element: CompilerType::Int,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Selection publishes only fresh retained nodes.
    fn emit_list_select(
        &mut self,
        list: &CompilerExpression,
        parameters: &[CompilerParameter],
        predicate: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
        reject: bool,
        indexes: bool,
    ) -> LlValue {
        let source = self
            .emit_expression(list, body, environment)
            .list_pointer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.select.loop");
        let visit = body.label("list.select.visit");
        let selected = body.label("list.select.selected");
        let skipped = body.label("list.select.skipped");
        let first = body.label("list.select.first");
        let link = body.label("list.select.link");
        let selected_merge = body.label("list.select.selected.merge");
        let advance = body.label("list.select.advance");
        let done = body.label("list.select.done");
        let next = body.reserve_value();
        let node = body.reserve_value();
        let selected_head = body.reserve_value();
        let next_head = body.reserve_value();
        let next_previous = body.reserve_value();
        let next_index = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_previous}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let index = body.instruction(
            &format!("phi i64 [0, %{preheader}], [{next_index}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{visit}"),
            location,
        );

        body.start_block(&visit);
        let value = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            span,
            &mut self.debug,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let predicate_value = if indexes {
            LlValue::Int(body.instruction(
                &format!("call ptr @topal.runtime.int.from.u64(i64 {index})"),
                span,
                &mut self.debug,
            ))
        } else {
            LlValue::Int(value.clone())
        };
        let mut predicate_environment =
            self.emit_collection_environment(parameters, &[predicate_value], body, environment);
        let keep = self.emit_block(predicate, body, &mut predicate_environment);
        let keep = if reject {
            body.instruction(
                &format!("xor i1 {}, true", keep.boolean()),
                span,
                &mut self.debug,
            )
        } else {
            keep.boolean().to_owned()
        };
        body.terminator(
            &format!("br i1 {keep}, label %{selected}, label %{skipped}"),
            location,
        );

        body.start_block(&skipped);
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&selected);
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {value}, ptr {node}, align 8"),
            span,
            &mut self.debug,
        );
        let node_next = body.instruction(
            &format!("getelementptr i8, ptr {node}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr null, ptr {node_next}, align 8"),
            span,
            &mut self.debug,
        );
        let has_previous = body.instruction(
            &format!("icmp ne ptr {previous}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {has_previous}, label %{link}, label %{first}"),
            location,
        );

        body.start_block(&first);
        body.terminator(&format!("br label %{selected_merge}"), location);
        body.start_block(&link);
        let previous_next = body.instruction(
            &format!("getelementptr i8, ptr {previous}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {node}, ptr {previous_next}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{selected_merge}"), location);
        body.start_block(&selected_merge);
        body.define_reserved(
            &selected_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);

        body.start_block(&advance);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{head}, %{skipped}], [{selected_head}, %{selected_merge}]"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_previous,
            &format!("phi ptr [{previous}, %{skipped}], [{node}, %{selected_merge}]"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_index,
            &format!("add i64 {index}, 1"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element: CompilerType::Int,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Fold control flow and enclosing LLVM state remain explicit.
    fn emit_list_fold(
        &mut self,
        list: &CompilerExpression,
        initial: &CompilerExpression,
        parameters: &[CompilerParameter],
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let source = self
            .emit_expression(list, body, environment)
            .list_pointer()
            .to_owned();
        let initial = self
            .emit_expression(initial, body, environment)
            .integer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.fold.loop");
        let visit = body.label("list.fold.visit");
        let advance = body.label("list.fold.advance");
        let done = body.label("list.fold.done");
        let next = body.reserve_value();
        let next_state = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let state = body.instruction(
            &format!("phi ptr [{initial}, %{preheader}], [{next_state}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{visit}"),
            location,
        );

        body.start_block(&visit);
        let value = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            span,
            &mut self.debug,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let mut action_environment = self.emit_collection_environment(
            parameters,
            &[LlValue::Int(state.clone()), LlValue::Int(value)],
            body,
            environment,
        );
        let value = self.emit_block(action, body, &mut action_environment);
        let finished_state = if action.result.value_type
            == CompilerType::TraversalControl(Box::new(CompilerType::Int))
        {
            let control = value.traversal_control().0;
            let tag = body.instruction(
                &format!("load i64, ptr {control}, align 8"),
                action.result.span,
                &mut self.debug,
            );
            let payload_address = body.instruction(
                &format!("getelementptr i8, ptr {control}, i64 8"),
                action.result.span,
                &mut self.debug,
            );
            let payload = body.instruction(
                &format!("load ptr, ptr {payload_address}, align 8"),
                action.result.span,
                &mut self.debug,
            );
            let finish = body.instruction(
                &format!("icmp eq i64 {tag}, 1"),
                action.result.span,
                &mut self.debug,
            );
            let continue_label = body.label("list.fold.continue");
            let finish_label = body.label("list.fold.finish");
            body.terminator(
                &format!("br i1 {finish}, label %{finish_label}, label %{continue_label}"),
                location,
            );
            body.start_block(&continue_label);
            body.define_reserved(
                &next_state,
                &format!("freeze ptr {payload}"),
                span,
                &mut self.debug,
            );
            body.terminator(&format!("br label %{advance}"), location);
            body.start_block(&finish_label);
            body.terminator(&format!("br label %{done}"), location);
            Some((payload, finish_label))
        } else {
            body.define_reserved(
                &next_state,
                &format!("freeze ptr {}", value.integer()),
                span,
                &mut self.debug,
            );
            body.terminator(&format!("br label %{advance}"), location);
            None
        };
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        if let Some((finished, predecessor)) = finished_state {
            LlValue::Int(body.instruction(
                &format!("phi ptr [{state}, %{loop_label}], [{finished}, %{predecessor}]"),
                span,
                &mut self.debug,
            ))
        } else {
            LlValue::Int(state)
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Result alternatives retain bindings and delayed actions explicitly.
    fn emit_result_decision(
        &mut self,
        subject: &CompilerExpression,
        ok_binding: &str,
        ok_binding_span: Span,
        ok_action: &CompilerExpression,
        error_codes: &[CompilerErrorCodeRule],
        error_fallback: Option<&(String, Span, Box<CompilerExpression>)>,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let result = self.emit_expression(subject, body, environment);
        let LlValue::Result {
            value,
            success,
            function_captures,
        } = result
        else {
            unreachable!("checked Result decision subject is Result")
        };
        let is_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
            subject.span,
            &mut self.debug,
        );
        let ok_label = body.label("result.decision.ok");
        let error_label = body.label("result.decision.error");
        let merge = body.label("result.decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_error}, label %{error_label}, label %{ok_label}"),
            location,
        );

        let mut branches = Vec::with_capacity(error_codes.len() + 2);
        body.start_block(&ok_label);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            ok_binding_span,
            &mut self.debug,
        );
        let mut success_value =
            self.result_success_value(&payload, &success, body, ok_binding_span);
        if let LlValue::Function { captures, .. } = &mut success_value {
            *captures = function_captures;
        }
        let variable = self
            .debug
            .local(ok_binding, ok_binding_span, &success, body.subprogram);
        let binding_location = self.debug.location(ok_binding_span, body.subprogram);
        body.debug_value(&success_value, variable, binding_location);
        let mut ok_environment = environment.clone();
        ok_environment.insert(ok_binding.into(), success_value);
        let ok_value = self.emit_expression(ok_action, body, &ok_environment);
        let predecessor = body.current_block.clone();
        let ok_location = self.debug.location(ok_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), ok_location);
        branches.push((ok_value, predecessor));

        body.start_block(&error_label);
        let error = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            subject.span,
            &mut self.debug,
        );
        if error_codes.is_empty() {
            self.emit_result_error_fallback(
                error_fallback.expect("complete Result decision has Error fallback"),
                &error,
                &merge,
                body,
                environment,
                &mut branches,
            );
        } else {
            let code = body.instruction(
                &format!("call i32 @topal.runtime.error.code(ptr {error})"),
                subject.span,
                &mut self.debug,
            );
            let labels = error_codes
                .iter()
                .map(|_| body.label("result.decision.code"))
                .collect::<Vec<_>>();
            let default = body.label("result.decision.error.fallback");
            let cases = error_codes
                .iter()
                .zip(&labels)
                .map(|(rule, label)| format!("i32 {}, label %{label}", rule.code))
                .collect::<Vec<_>>()
                .join(" ");
            body.terminator(
                &format!("switch i32 {code}, label %{default} [ {cases} ]"),
                location,
            );
            for (rule, label) in error_codes.iter().zip(labels) {
                body.start_block(&label);
                let action = self.emit_expression(&rule.action, body, environment);
                let predecessor = body.current_block.clone();
                let action_location = self.debug.location(rule.action.span, body.subprogram);
                body.terminator(&format!("br label %{merge}"), action_location);
                branches.push((action, predecessor));
            }
            body.start_block(&default);
            if let Some(fallback) = error_fallback {
                self.emit_result_error_fallback(
                    fallback,
                    &error,
                    &merge,
                    body,
                    environment,
                    &mut branches,
                );
            } else {
                body.effect(
                    "call void @topal.platform.exit(i64 70)",
                    span,
                    &mut self.debug,
                );
                body.terminator("unreachable", location);
            }
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    #[allow(clippy::too_many_arguments)] // Helper keeps the Error binding scoped to its action block.
    fn emit_result_error_fallback(
        &mut self,
        fallback: &(String, Span, Box<CompilerExpression>),
        error: &str,
        merge: &str,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        branches: &mut Vec<(LlValue, String)>,
    ) {
        let (binding, binding_span, action) = fallback;
        let error_value = LlValue::Error(error.into());
        let variable = self.debug.local(
            binding,
            *binding_span,
            &CompilerType::Error,
            body.subprogram,
        );
        let location = self.debug.location(*binding_span, body.subprogram);
        body.debug_value(&error_value, variable, location);
        let mut branch_environment = environment.clone();
        branch_environment.insert(binding.clone(), error_value);
        let value = self.emit_expression(action, body, &branch_environment);
        let predecessor = body.current_block.clone();
        let action_location = self.debug.location(action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), action_location);
        branches.push((value, predecessor));
    }

    #[allow(clippy::too_many_arguments)] // Validation retains explicit Error provenance at the ABI call.
    fn emit_validation(
        &mut self,
        operation: CompilerValidation,
        value: &CompilerExpression,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        if let CompilerValidation::Constraint(tag) = operation {
            return self.emit_constraint_validation(
                tag,
                value,
                error_span,
                body,
                environment,
                span,
            );
        }
        let value = self.emit_expression(value, body, environment);
        let (runtime, domain, success, value) = match operation {
            CompilerValidation::RationalToInt => (
                "rational.try.to.int",
                "root.Int(Rational)",
                CompilerType::Int,
                value.rational(),
            ),
            CompilerValidation::IntToNat => (
                "int.try.to.nat",
                "root.Nat(Int)",
                CompilerType::Nat,
                value.integer(),
            ),
            CompilerValidation::Constraint(_) => {
                unreachable!("Constraint validation was lowered above")
            }
        };
        let domain_global = self.emit_string_value(domain, body, span);
        let source_global = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        LlValue::Result {
            value: body.instruction(
                &format!(
                    "call ptr @topal.runtime.{runtime}(ptr {value}, ptr {domain_global}, ptr {source_global}, i64 {}, i64 {})",
                    position.line, position.column
                ),
                span,
                &mut self.debug,
            ),
            success,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)] // Predicate environment and Error provenance stay explicit.
    fn emit_constraint_validation(
        &mut self,
        tag: u32,
        value: &CompilerExpression,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let constraint =
            self.program.constraints[usize::try_from(tag).expect("u32 tag fits usize")].clone();
        let value = self.emit_expression(value, body, environment);
        let mut predicate_environment = environment.clone();
        predicate_environment.insert(constraint.parameter_storage, value.clone());
        let accepted = self.emit_expression(&constraint.predicate, body, &predicate_environment);
        let accepted_label = body.label("constraint.accepted");
        let rejected_label = body.label("constraint.rejected");
        let merge = body.label("constraint.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!(
                "br i1 {}, label %{accepted_label}, label %{rejected_label}",
                accepted.boolean()
            ),
            location,
        );

        body.start_block(&accepted_label);
        let payload = self.emit_result_payload(&value, &constraint.base_type, body, error_span);
        let success = body.instruction(
            &format!("call ptr @topal.runtime.result.success(ptr {payload})"),
            span,
            &mut self.debug,
        );
        let success_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&rejected_label);
        let domain = format!("root.{}({})", constraint.name, constraint.base_type.name());
        let domain = self.emit_string_value(&domain, body, span);
        let source = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        let failure = body.instruction(
            &format!(
                "call ptr @topal.runtime.result.failure(i32 0, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                position.line, position.column
            ),
            span,
            &mut self.debug,
        );
        let failure_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&merge);
        let value = body.instruction(
            &format!(
                "phi ptr [{success}, %{success_predecessor}], [{failure}, %{failure_predecessor}]"
            ),
            span,
            &mut self.debug,
        );
        LlValue::Result {
            value,
            success: constraint.base_type,
            function_captures: Vec::new(),
        }
    }

    fn emit_modular_validation(
        &mut self,
        value: &CompilerExpression,
        modular: &CompilerModularType,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let value = self.emit_expression(value, body, environment);
        let value = value.integer();
        let lower = self.emit_int_global(&modular.lower);
        let upper = self.emit_int_global(&modular.upper);
        let lower_order = body.instruction(
            &format!("call i32 @topal.runtime.int.compare(ptr {value}, ptr {lower})"),
            span,
            &mut self.debug,
        );
        let upper_order = body.instruction(
            &format!("call i32 @topal.runtime.int.compare(ptr {value}, ptr {upper})"),
            span,
            &mut self.debug,
        );
        let at_or_above = body.instruction(
            &format!("icmp sge i32 {lower_order}, 0"),
            span,
            &mut self.debug,
        );
        let at_or_below = body.instruction(
            &format!("icmp sle i32 {upper_order}, 0"),
            span,
            &mut self.debug,
        );
        let accepted = body.instruction(
            &format!("and i1 {at_or_above}, {at_or_below}"),
            span,
            &mut self.debug,
        );
        let accepted_label = body.label("modular.accepted");
        let rejected_label = body.label("modular.rejected");
        let merge = body.label("modular.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {accepted}, label %{accepted_label}, label %{rejected_label}"),
            location,
        );

        body.start_block(&accepted_label);
        let success = body.instruction(
            &format!("call ptr @topal.runtime.result.success(ptr {value})"),
            span,
            &mut self.debug,
        );
        let success_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&rejected_label);
        let domain = self.emit_string_value(&format!("root.{}(Int)", modular.name), body, span);
        let source = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        let failure = body.instruction(
            &format!(
                "call ptr @topal.runtime.result.failure(i32 0, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                position.line, position.column
            ),
            span,
            &mut self.debug,
        );
        let failure_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&merge);
        let value = body.instruction(
            &format!(
                "phi ptr [{success}, %{success_predecessor}], [{failure}, %{failure_predecessor}]"
            ),
            span,
            &mut self.debug,
        );
        LlValue::Result {
            value,
            success: CompilerType::Modular(modular.clone()),
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)] // Fallible ABI arguments keep structured provenance explicit.
    fn emit_fallible(
        &mut self,
        operation: CompilerFallible,
        left: &CompilerExpression,
        right: &CompilerExpression,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let left = self.emit_expression(left, body, environment);
        let right = self.emit_expression(right, body, environment);
        let (runtime, domain, success, left, right) = match operation {
            CompilerFallible::RationalConstruct => (
                "rational.try.make",
                "root.Rational(Int,Int)",
                CompilerType::Rational,
                left.integer(),
                right.integer(),
            ),
            CompilerFallible::RationalDivide => (
                "rational.try.divide",
                "root./(Rational,Rational)",
                CompilerType::Rational,
                left.rational(),
                right.rational(),
            ),
            CompilerFallible::RationalPower => (
                "rational.try.power",
                "root.^(Rational,Int)",
                CompilerType::Rational,
                left.rational(),
                right.integer(),
            ),
            CompilerFallible::IntModulo => (
                "int.try.modulo",
                "root.%(Int,Int)",
                CompilerType::Int,
                left.integer(),
                right.integer(),
            ),
            CompilerFallible::IntQuotientModulo => (
                "int.try.quotient.modulo",
                "root./%(Int,Int)",
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]),
                left.integer(),
                right.integer(),
            ),
            CompilerFallible::InfinityMultiply => {
                self.needs_infinity_result_runtime = true;
                match (&left, &right) {
                    (LlValue::Int(left), LlValue::Int(right)) => (
                        "int.try.multiply.infinity",
                        "root.*(Int,Int)",
                        CompilerType::Int,
                        left.as_str(),
                        right.as_str(),
                    ),
                    (LlValue::Rational(left), LlValue::Rational(right)) => (
                        "rational.try.multiply.infinity",
                        "root.*(Rational,Rational)",
                        CompilerType::Rational,
                        left.as_str(),
                        right.as_str(),
                    ),
                    _ => {
                        unreachable!("checked dynamic infinity multiplication has one exact domain")
                    }
                }
            }
        };
        let domain_global = self.emit_string_value(domain, body, span);
        let source_global = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        LlValue::Result {
            value: body.instruction(
                &format!(
                    "call ptr @topal.runtime.{runtime}(ptr {left}, ptr {right}, ptr {domain_global}, ptr {source_global}, i64 {}, i64 {})",
                    position.line, position.column
                ),
                span,
                &mut self.debug,
            ),
            success,
            function_captures: Vec::new(),
        }
    }

    fn emit_result_payload(
        &mut self,
        value: &LlValue,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        match (value, value_type) {
            (LlValue::Unit, CompilerType::Unit) => "null".into(),
            (LlValue::Modular { value, modular }, CompilerType::Modular(modular_type))
                if modular == modular_type =>
            {
                value.clone()
            }
            (LlValue::Int(value), CompilerType::Int | CompilerType::Nat)
            | (LlValue::Rational(value), CompilerType::Rational)
            | (LlValue::String(value), CompilerType::String)
            | (LlValue::Range { value, .. }, CompilerType::Range(_))
            | (LlValue::Result { value, .. }, CompilerType::Result(_)) => value.clone(),
            (LlValue::Enum { value, enumeration }, CompilerType::Enum(expected_enumeration))
                if enumeration == expected_enumeration =>
            {
                let storage = body.instruction(
                    "call ptr @topal.platform.allocate(i64 4)",
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store i32 {value}, ptr {storage}, align 4"),
                    span,
                    &mut self.debug,
                );
                storage
            }
            (
                LlValue::Function { value, .. } | LlValue::Enum { value, .. },
                CompilerType::Function,
            ) => {
                let storage = body.instruction(
                    "call ptr @topal.platform.allocate(i64 4)",
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store i32 {value}, ptr {storage}, align 4"),
                    span,
                    &mut self.debug,
                );
                storage
            }
            (LlValue::Tuple(fields), CompilerType::Tuple(types))
                if types.as_slice() == [CompilerType::Int, CompilerType::String] =>
            {
                let [integer, text] = fields.as_slice() else {
                    unreachable!("checked Result product payload has two fields")
                };
                let storage = body.instruction(
                    "call ptr @topal.platform.allocate(i64 16)",
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {storage}, align 8", integer.integer()),
                    span,
                    &mut self.debug,
                );
                let text_address = body.instruction(
                    &format!("getelementptr i8, ptr {storage}, i64 8"),
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {text_address}, align 8", text.string()),
                    span,
                    &mut self.debug,
                );
                storage
            }
            (LlValue::Tuple(fields), CompilerType::Tuple(types))
                if matches!(types.as_slice(), [CompilerType::Int, CompilerType::Int]) =>
            {
                let [quotient, remainder] = fields.as_slice() else {
                    unreachable!("checked quotient/modulo payload has two fields")
                };
                body.instruction(
                    &format!(
                        "call ptr @topal.runtime.int.divmod.pair(ptr {}, ptr {})",
                        quotient.integer(),
                        remainder.integer()
                    ),
                    span,
                    &mut self.debug,
                )
            }
            _ => unreachable!("checked Result success has a pointer payload representation"),
        }
    }

    fn emit_optional_payload_pointer(
        &mut self,
        value: &LlValue,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        let boxed_i32 = match (value, value_type) {
            (LlValue::Enum { value, enumeration }, CompilerType::Enum(expected_enumeration))
                if enumeration == expected_enumeration =>
            {
                Some(value)
            }
            (
                LlValue::Function { value, .. } | LlValue::Enum { value, .. },
                CompilerType::Function,
            ) => Some(value),
            _ => None,
        };
        if let Some(value) = boxed_i32 {
            let storage = body.instruction(
                "call ptr @topal.platform.allocate(i64 4)",
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store i32 {value}, ptr {storage}, align 4"),
                span,
                &mut self.debug,
            );
            return storage;
        }
        if let (LlValue::Tuple(fields), CompilerType::Tuple(types)) = (value, value_type)
            && matches!(types.as_slice(), [CompilerType::Int, CompilerType::String])
        {
            let [integer, text] = fields.as_slice() else {
                unreachable!("checked Optional product payload has two fields")
            };
            let storage = body.instruction(
                "call ptr @topal.platform.allocate(i64 16)",
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {}, ptr {storage}, align 8", integer.integer()),
                span,
                &mut self.debug,
            );
            let text_address = body.instruction(
                &format!("getelementptr i8, ptr {storage}, i64 8"),
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {}, ptr {text_address}, align 8", text.string()),
                span,
                &mut self.debug,
            );
            return storage;
        }
        optional_payload_pointer(value).to_owned()
    }

    fn result_success_value(
        &mut self,
        payload: &str,
        success: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        match success {
            CompilerType::Unit => LlValue::Unit,
            CompilerType::Int | CompilerType::Nat => LlValue::Int(payload.into()),
            CompilerType::Modular(modular) => LlValue::Modular {
                value: payload.into(),
                modular: modular.clone(),
            },
            CompilerType::Rational => LlValue::Rational(payload.into()),
            CompilerType::String => LlValue::String(payload.into()),
            CompilerType::Range(endpoint) => LlValue::Range {
                value: payload.into(),
                endpoint: endpoint.as_ref().clone(),
            },
            CompilerType::Result(nested) => LlValue::Result {
                value: payload.into(),
                success: nested.as_ref().clone(),
                function_captures: Vec::new(),
            },
            CompilerType::List(element) => LlValue::List {
                value: payload.into(),
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            },
            CompilerType::Enum(enumeration) => LlValue::Enum {
                value: body.instruction(
                    &format!("load i32, ptr {payload}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: enumeration.clone(),
            },
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {payload}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            CompilerType::Tuple(types)
                if types.as_slice() == [CompilerType::Int, CompilerType::String] =>
            {
                let integer = body.instruction(
                    &format!("load ptr, ptr {payload}, align 8"),
                    span,
                    &mut self.debug,
                );
                let text_address = body.instruction(
                    &format!("getelementptr i8, ptr {payload}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let text = body.instruction(
                    &format!("load ptr, ptr {text_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![LlValue::Int(integer), LlValue::String(text)])
            }
            CompilerType::Tuple(types)
                if matches!(types.as_slice(), [CompilerType::Int, CompilerType::Int]) =>
            {
                let quotient = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.quotient(ptr {payload})"),
                    span,
                    &mut self.debug,
                );
                let remainder = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.remainder(ptr {payload})"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![LlValue::Int(quotient), LlValue::Int(remainder)])
            }
            _ => unreachable!("checked Result success has a pointer payload representation"),
        }
    }

    #[allow(clippy::too_many_lines)] // Exact domains and result representations are selected together.
    fn emit_binary(
        &mut self,
        operation: CompilerBinary,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        match operation {
            CompilerBinary::Range
            | CompilerBinary::RangeOpen
            | CompilerBinary::RangeInclusive
            | CompilerBinary::RangeOpenInclusive => {
                let (lower_inclusive, upper_inclusive) = match operation {
                    CompilerBinary::Range => (1, 0),
                    CompilerBinary::RangeOpen => (0, 0),
                    CompilerBinary::RangeInclusive => (1, 1),
                    CompilerBinary::RangeOpenInclusive => (0, 1),
                    _ => unreachable!(),
                };
                let (left, endpoint) = numeric_pointer(left);
                let (right, right_endpoint) = numeric_pointer(right);
                debug_assert_eq!(endpoint, right_endpoint);
                LlValue::Range {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.range.make(ptr {left}, ptr {right}, i64 {lower_inclusive}, i64 {upper_inclusive})"
                        ),
                        span,
                        &mut self.debug,
                    ),
                    endpoint: endpoint.clone(),
                }
            }
            CompilerBinary::In | CompilerBinary::Contains => {
                let (range, value) = if operation == CompilerBinary::In {
                    (right, left)
                } else {
                    (left, right)
                };
                let (range, endpoint) = range.range();
                let (value, value_type) = numeric_pointer(value);
                debug_assert_eq!(endpoint, &value_type);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.range.{}.contains(ptr {range}, ptr {value})",
                        numeric_domain(endpoint)
                    ),
                    span,
                    &mut self.debug,
                ))
            }
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply => {
                let name = match operation {
                    CompilerBinary::Add => "add",
                    CompilerBinary::Subtract => "subtract",
                    CompilerBinary::Multiply => "multiply",
                    _ => unreachable!(),
                };
                if let (
                    LlValue::Modular {
                        value: left,
                        modular,
                    },
                    LlValue::Modular {
                        value: right,
                        modular: right_modular,
                    },
                ) = (left, right)
                {
                    debug_assert_eq!(modular, right_modular);
                    let value = body.instruction(
                        &format!("call ptr @topal.runtime.int.{name}(ptr {left}, ptr {right})"),
                        span,
                        &mut self.debug,
                    );
                    return self.emit_modular_reduce(&value, modular, body, span);
                }
                let (domain, left, right) = match (left, right) {
                    (LlValue::Int(left), LlValue::Int(right)) => ("int", left, right),
                    (LlValue::Rational(left), LlValue::Rational(right)) => {
                        ("rational", left, right)
                    }
                    _ => unreachable!("checked arithmetic operands agree"),
                };
                let value = body.instruction(
                    &format!("call ptr @topal.runtime.{domain}.{name}(ptr {left}, ptr {right})"),
                    span,
                    &mut self.debug,
                );
                if domain == "int" {
                    LlValue::Int(value)
                } else {
                    LlValue::Rational(value)
                }
            }
            CompilerBinary::Divide => {
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.divide(ptr {}, ptr {})",
                        left.rational(),
                        right.rational()
                    ),
                    span,
                    &mut self.debug,
                );
                LlValue::Rational(value)
            }
            CompilerBinary::Modulo => LlValue::Int(body.instruction(
                &format!(
                    "call ptr @topal.runtime.int.modulo(ptr {}, ptr {})",
                    left.integer(),
                    right.integer()
                ),
                span,
                &mut self.debug,
            )),
            CompilerBinary::QuotientModulo => {
                let pair = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.int.quotient.modulo(ptr {}, ptr {})",
                        left.integer(),
                        right.integer()
                    ),
                    span,
                    &mut self.debug,
                );
                let quotient = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.quotient(ptr {pair})"),
                    span,
                    &mut self.debug,
                );
                let remainder = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.remainder(ptr {pair})"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![LlValue::Int(quotient), LlValue::Int(remainder)])
            }
            CompilerBinary::Power => match left {
                LlValue::Int(left) => LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.int.power(ptr {left}, ptr {})",
                        right.integer()
                    ),
                    span,
                    &mut self.debug,
                )),
                LlValue::Rational(left) => LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.power(ptr {left}, ptr {})",
                        right.integer()
                    ),
                    span,
                    &mut self.debug,
                )),
                _ => unreachable!("checked power base is exact numeric"),
            },
            CompilerBinary::Compare => {
                LlValue::Comparison(self.emit_total_compare(left, right, body, span))
            }
            CompilerBinary::And => match (left, right) {
                (LlValue::Boolean(left), LlValue::Boolean(right)) => LlValue::Boolean(
                    body.instruction(&format!("and i1 {left}, {right}"), span, &mut self.debug),
                ),
                (
                    LlValue::Range {
                        value: left,
                        endpoint,
                    },
                    LlValue::Range {
                        value: right,
                        endpoint: right_endpoint,
                    },
                ) => {
                    debug_assert_eq!(endpoint, right_endpoint);
                    LlValue::Range {
                        value: body.instruction(
                            &format!(
                                "call ptr @topal.runtime.range.{}.intersection(ptr {left}, ptr {right})",
                                numeric_domain(endpoint)
                            ),
                            span,
                            &mut self.debug,
                        ),
                        endpoint: endpoint.clone(),
                    }
                }
                _ => unreachable!("checked conjunction operands agree"),
            },
            CompilerBinary::Or => LlValue::Boolean(body.instruction(
                &format!("or i1 {}, {}", left.boolean(), right.boolean()),
                span,
                &mut self.debug,
            )),
            CompilerBinary::Xor => LlValue::Boolean(body.instruction(
                &format!("xor i1 {}, {}", left.boolean(), right.boolean()),
                span,
                &mut self.debug,
            )),
            CompilerBinary::Equal | CompilerBinary::NotEqual => {
                let predicate = if operation == CompilerBinary::Equal {
                    "eq"
                } else {
                    "ne"
                };
                let equal = self.emit_equal(left, right, body, span);
                LlValue::Boolean(body.instruction(
                    &format!("icmp {predicate} i1 {equal}, true"),
                    span,
                    &mut self.debug,
                ))
            }
            CompilerBinary::Less
            | CompilerBinary::Greater
            | CompilerBinary::LessEqual
            | CompilerBinary::GreaterEqual => {
                let predicate = match operation {
                    CompilerBinary::Less => "slt",
                    CompilerBinary::Greater => "sgt",
                    CompilerBinary::LessEqual => "sle",
                    CompilerBinary::GreaterEqual => "sge",
                    _ => unreachable!(),
                };
                let comparison = self.emit_total_compare(left, right, body, span);
                LlValue::Boolean(body.instruction(
                    &format!("icmp {predicate} i32 {comparison}, 0"),
                    span,
                    &mut self.debug,
                ))
            }
        }
    }

    #[allow(clippy::too_many_lines)] // Exhaustive structural value lowering keeps representation pairs visible.
    fn emit_equal(
        &mut self,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        match (left, right) {
            (LlValue::Unit, LlValue::Unit) => {
                body.instruction("icmp eq i8 0, 0", span, &mut self.debug)
            }
            (LlValue::Completed(left), LlValue::Completed(right))
            | (LlValue::Effect(left), LlValue::Effect(right)) => body.instruction(
                &format!("icmp eq i8 {left}, {right}"),
                span,
                &mut self.debug,
            ),
            (LlValue::Boolean(left), LlValue::Boolean(right)) => body.instruction(
                &format!("icmp eq i1 {left}, {right}"),
                span,
                &mut self.debug,
            ),
            (LlValue::Comparison(left), LlValue::Comparison(right))
            | (LlValue::ErrorCode(left), LlValue::ErrorCode(right)) => body.instruction(
                &format!("icmp eq i32 {left}, {right}"),
                span,
                &mut self.debug,
            ),
            (
                LlValue::Enum {
                    value: left,
                    enumeration,
                },
                LlValue::Enum {
                    value: right,
                    enumeration: right_enumeration,
                },
            )
            | (
                LlValue::Function {
                    value: left,
                    enumeration,
                    ..
                },
                LlValue::Function {
                    value: right,
                    enumeration: right_enumeration,
                    ..
                },
            ) => {
                debug_assert_eq!(enumeration, right_enumeration);
                body.instruction(
                    &format!("icmp eq i32 {left}, {right}"),
                    span,
                    &mut self.debug,
                )
            }
            (LlValue::String(left), LlValue::String(right)) => body.instruction(
                &format!("call i1 @topal.runtime.string.equal(ptr {left}, ptr {right})"),
                span,
                &mut self.debug,
            ),
            (
                LlValue::List {
                    value: left,
                    element,
                    ..
                },
                LlValue::List {
                    value: right,
                    element: right_element,
                    ..
                },
            ) => self.emit_list_equal(left, right, element, right_element, body, span),
            (
                LlValue::Optional {
                    value: left,
                    payload,
                    ..
                },
                LlValue::Optional {
                    value: right,
                    payload: right_payload,
                    ..
                },
            ) => self.emit_optional_equal(left, right, payload, right_payload, body, span),
            (
                LlValue::Result {
                    value: left,
                    success,
                    ..
                },
                LlValue::Result {
                    value: right,
                    success: right_success,
                    ..
                },
            ) => self.emit_result_equal(left, right, success, right_success, body, span),
            (LlValue::Int(_), LlValue::Int(_))
            | (LlValue::Modular { .. }, LlValue::Modular { .. })
            | (LlValue::Rational(_), LlValue::Rational(_)) => {
                let comparison = self.emit_numeric_compare(left, right, body, span);
                body.instruction(
                    &format!("icmp eq i32 {comparison}, 0"),
                    span,
                    &mut self.debug,
                )
            }
            (LlValue::Tuple(left), LlValue::Tuple(right)) => {
                debug_assert_eq!(left.len(), right.len());
                let mut fields = left.iter().zip(right);
                let Some((left, right)) = fields.next() else {
                    return "true".into();
                };
                let mut equal = self.emit_equal(left, right, body, span);
                for (left, right) in fields {
                    let field_equal = self.emit_equal(left, right, body, span);
                    equal = body.instruction(
                        &format!("and i1 {equal}, {field_equal}"),
                        span,
                        &mut self.debug,
                    );
                }
                equal
            }
            (LlValue::Record { fields: left, .. }, LlValue::Record { fields: right, .. }) => {
                self.emit_record_equal(left, right, body, span)
            }
            (
                LlValue::Sum {
                    tag: left_tag,
                    payloads: left_payloads,
                    sum,
                },
                LlValue::Sum {
                    tag: right_tag,
                    payloads: right_payloads,
                    sum: right_sum,
                },
            ) => {
                debug_assert_eq!(sum, right_sum);
                self.emit_sum_equal(
                    left_tag,
                    left_payloads,
                    right_tag,
                    right_payloads,
                    sum,
                    body,
                    span,
                )
            }
            _ => unreachable!("checked equality values agree"),
        }
    }

    #[allow(clippy::too_many_arguments)] // Both structural operands remain explicit at the lowering boundary.
    fn emit_sum_equal(
        &mut self,
        left_tag: &str,
        left_payloads: &[Option<Box<LlValue>>],
        right_tag: &str,
        right_payloads: &[Option<Box<LlValue>>],
        sum: &CompilerSumType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(left_payloads.len(), sum.alternatives.len());
        debug_assert_eq!(right_payloads.len(), sum.alternatives.len());
        let tags_equal = body.instruction(
            &format!("icmp eq i32 {left_tag}, {right_tag}"),
            span,
            &mut self.debug,
        );
        let compare_tag = body.label("sum.equal.tag");
        let unequal = body.label("sum.equal.unequal");
        let merge = body.label("sum.equal.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {tags_equal}, label %{compare_tag}, label %{unequal}"),
            location,
        );
        body.start_block(&compare_tag);
        let alternatives = sum
            .alternatives
            .iter()
            .map(|_| body.label("sum.equal.alternative"))
            .collect::<Vec<_>>();
        let cases = alternatives
            .iter()
            .enumerate()
            .map(|(tag, label)| format!("i32 {tag}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        body.terminator(
            &format!("switch i32 {left_tag}, label %{unequal} [ {cases} ]"),
            location,
        );

        let mut branches = Vec::with_capacity(sum.alternatives.len() + 1);
        for (index, ((alternative, left), right)) in sum
            .alternatives
            .iter()
            .zip(left_payloads)
            .zip(right_payloads)
            .enumerate()
        {
            body.start_block(&alternatives[index]);
            let equal = match (&alternative.payload, left, right) {
                (None, None, None) => "true".to_owned(),
                (Some(_), Some(left), Some(right)) => self.emit_equal(left, right, body, span),
                _ => unreachable!("checked sum equality retains every declared payload"),
            };
            let predecessor = body.current_block.clone();
            body.terminator(&format!("br label %{merge}"), location);
            branches.push((equal, predecessor));
        }
        body.start_block(&unequal);
        let unequal_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);
        branches.push(("false".to_owned(), unequal_predecessor));

        body.start_block(&merge);
        body.instruction(
            &format!(
                "phi i1 {}",
                branches
                    .iter()
                    .map(|(equal, predecessor)| format!("[ {equal}, %{predecessor} ]"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            span,
            &mut self.debug,
        )
    }

    fn emit_list_equal(
        &mut self,
        left: &str,
        right: &str,
        element: &CompilerType,
        right_element: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(element, right_element);
        let runtime = if element == &CompilerType::Boolean {
            self.pointer_list_runtime_fragments
                .insert(PointerListRuntimeFragment::Boolean);
            "boolean"
        } else if matches!(element, CompilerType::Character | CompilerType::String) {
            self.pointer_list_runtime_fragments
                .insert(PointerListRuntimeFragment::String);
            "string"
        } else if element == &CompilerType::Rational {
            self.pointer_list_runtime_fragments
                .insert(PointerListRuntimeFragment::Rational);
            "rational"
        } else if compiler_nested_int_string_list_element(element) {
            self.list_int_runtime_fragments
                .insert(ListIntRuntimeFragment::NestedIntStringCore);
            "nested.int-string"
        } else {
            debug_assert!(matches!(element, CompilerType::Int | CompilerType::Nat));
            self.list_int_runtime_fragments
                .insert(ListIntRuntimeFragment::Core);
            "int"
        };
        body.instruction(
            &format!("call i1 @topal.runtime.list.{runtime}.equal(ptr {left}, ptr {right})"),
            span,
            &mut self.debug,
        )
    }

    fn emit_result_equal(
        &mut self,
        left: &str,
        right: &str,
        success: &CompilerType,
        right_success: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(success, right_success);
        assert!(
            matches!(success, CompilerType::Enum(_))
                || matches!(success, CompilerType::Tuple(fields)
                    if fields.as_slice() == [CompilerType::Int, CompilerType::String])
        );
        let left_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {left})"),
            span,
            &mut self.debug,
        );
        let right_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {right})"),
            span,
            &mut self.debug,
        );
        let any_error = body.instruction(
            &format!("or i1 {left_error}, {right_error}"),
            span,
            &mut self.debug,
        );
        let failure = body.label("result.product.equal.error");
        let success_label = body.label("result.product.equal.payload");
        let merge = body.label("result.product.equal.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {any_error}, label %{failure}, label %{success_label}"),
            location,
        );
        body.start_block(&success_label);
        let left_payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {left})"),
            span,
            &mut self.debug,
        );
        let right_payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {right})"),
            span,
            &mut self.debug,
        );
        let left_value = self.result_success_value(&left_payload, success, body, span);
        let right_value = self.result_success_value(&right_payload, success, body, span);
        let payload_equal = self.emit_equal(&left_value, &right_value, body, span);
        let success_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);
        body.start_block(&failure);
        let failure_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);
        body.start_block(&merge);
        body.instruction(
            &format!(
                "phi i1 [ {payload_equal}, %{success_predecessor} ], [ false, %{failure_predecessor} ]"
            ),
            span,
            &mut self.debug,
        )
    }

    fn emit_optional_equal(
        &mut self,
        left: &str,
        right: &str,
        payload: &CompilerType,
        right_payload: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(payload, right_payload);
        let runtime = match payload {
            CompilerType::Int => "optional.int.equal",
            CompilerType::Rational => "optional.rational.equal",
            CompilerType::String => "optional.string.equal",
            payload
                if matches!(payload, CompilerType::Enum(_) | CompilerType::Function)
                    || matches!(payload, CompilerType::Tuple(fields)
                        if fields.as_slice() == [CompilerType::Int, CompilerType::String]) =>
            {
                let left_some = body.instruction(
                    &format!("call i1 @topal.runtime.optional.is.some(ptr {left})"),
                    span,
                    &mut self.debug,
                );
                let right_some = body.instruction(
                    &format!("call i1 @topal.runtime.optional.is.some(ptr {right})"),
                    span,
                    &mut self.debug,
                );
                let both_some = body.instruction(
                    &format!("and i1 {left_some}, {right_some}"),
                    span,
                    &mut self.debug,
                );
                let payload_label = body.label("optional.product.equal.payload");
                let tag_label = body.label("optional.product.equal.tag");
                let merge_label = body.label("optional.product.equal.merge");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("br i1 {both_some}, label %{payload_label}, label %{tag_label}"),
                    location,
                );
                body.start_block(&payload_label);
                let left_payload = body.instruction(
                    &format!("call ptr @topal.runtime.optional.payload(ptr {left})"),
                    span,
                    &mut self.debug,
                );
                let right_payload = body.instruction(
                    &format!("call ptr @topal.runtime.optional.payload(ptr {right})"),
                    span,
                    &mut self.debug,
                );
                let left_value =
                    self.emit_optional_payload_value(left_payload, payload, body, span);
                let right_value =
                    self.emit_optional_payload_value(right_payload, payload, body, span);
                let payload_equal = self.emit_equal(&left_value, &right_value, body, span);
                let payload_predecessor = body.current_block.clone();
                body.terminator(&format!("br label %{merge_label}"), location);
                body.start_block(&tag_label);
                let tags_equal = body.instruction(
                    &format!("icmp eq i1 {left_some}, {right_some}"),
                    span,
                    &mut self.debug,
                );
                let tag_predecessor = body.current_block.clone();
                body.terminator(&format!("br label %{merge_label}"), location);
                body.start_block(&merge_label);
                return body.instruction(
                    &format!(
                        "phi i1 [ {payload_equal}, %{payload_predecessor} ], [ {tags_equal}, %{tag_predecessor} ]"
                    ),
                    span,
                    &mut self.debug,
                );
            }
            _ => unreachable!("checked Optional equality has canonical evidence"),
        };
        body.instruction(
            &format!("call i1 @topal.runtime.{runtime}(ptr {left}, ptr {right})"),
            span,
            &mut self.debug,
        )
    }

    fn emit_record_equal(
        &mut self,
        left: &[(String, LlValue)],
        right: &[(String, LlValue)],
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(left.len(), right.len());
        let mut equal = None;
        for (label, left) in left {
            let right = right
                .iter()
                .find_map(|(name, value)| (name == label).then_some(value))
                .unwrap_or_else(|| panic!("checked Record equality retains `{label}`"));
            let field_equal = self.emit_equal(left, right, body, span);
            equal = Some(match equal {
                None => field_equal,
                Some(equal) => body.instruction(
                    &format!("and i1 {equal}, {field_equal}"),
                    span,
                    &mut self.debug,
                ),
            });
        }
        equal.unwrap_or_else(|| "true".into())
    }

    fn emit_total_compare(
        &mut self,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        match (left, right) {
            (LlValue::Int(_), LlValue::Int(_))
            | (LlValue::Modular { .. }, LlValue::Modular { .. })
            | (LlValue::Rational(_), LlValue::Rational(_)) => {
                self.emit_numeric_compare(left, right, body, span)
            }
            (LlValue::Tuple(left), LlValue::Tuple(right)) => {
                debug_assert_eq!(left.len(), right.len());
                let mut fields = left.iter().zip(right);
                let Some((left, right)) = fields.next() else {
                    return "0".into();
                };
                let mut comparison = self.emit_total_compare(left, right, body, span);
                for (left, right) in fields {
                    let comparison_block = body.current_block.clone();
                    let compare_next = body.label("tuple.compare.next");
                    let compare_done = body.label("tuple.compare.done");
                    let equal = body.instruction(
                        &format!("icmp eq i32 {comparison}, 0"),
                        span,
                        &mut self.debug,
                    );
                    let location = self.debug.location(span, body.subprogram);
                    body.terminator(
                        &format!("br i1 {equal}, label %{compare_next}, label %{compare_done}"),
                        location,
                    );
                    body.start_block(&compare_next);
                    let next = self.emit_total_compare(left, right, body, span);
                    let next_block = body.current_block.clone();
                    body.terminator(&format!("br label %{compare_done}"), location);
                    body.start_block(&compare_done);
                    comparison = body.instruction(
                        &format!(
                            "phi i32 [ {comparison}, %{comparison_block} ], [ {next}, %{next_block} ]"
                        ),
                        span,
                        &mut self.debug,
                    );
                }
                comparison
            }
            _ => unreachable!("checked total-order values agree"),
        }
    }

    fn emit_numeric_compare(
        &mut self,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        let (domain, left, right) = match (left, right) {
            (LlValue::Int(left), LlValue::Int(right)) => ("int", left, right),
            (
                LlValue::Modular {
                    value: left,
                    modular,
                },
                LlValue::Modular {
                    value: right,
                    modular: right_modular,
                },
            ) => {
                debug_assert_eq!(modular, right_modular);
                ("int", left, right)
            }
            (LlValue::Rational(left), LlValue::Rational(right)) => ("rational", left, right),
            _ => unreachable!("checked numeric comparison operands agree"),
        };
        body.instruction(
            &format!("call i32 @topal.runtime.{domain}.compare(ptr {left}, ptr {right})"),
            span,
            &mut self.debug,
        )
    }

    fn emit_boolean_decision(
        &mut self,
        subject: &CompilerExpression,
        when_true: &CompilerExpression,
        when_false: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let subject = emitted_subject.boolean();
        let true_label = body.label("decision.true");
        let false_label = body.label("decision.false");
        let merge_label = body.label("decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {subject}, label %{true_label}, label %{false_label}"),
            location,
        );

        body.start_block(&true_label);
        let true_value = self.emit_expression(when_true, body, environment);
        let true_predecessor = body.current_block.clone();
        let true_location = self.debug.location(when_true.span, body.subprogram);
        body.terminator(&format!("br label %{merge_label}"), true_location);

        body.start_block(&false_label);
        let false_value = self.emit_expression(when_false, body, environment);
        let false_predecessor = body.current_block.clone();
        let false_location = self.debug.location(when_false.span, body.subprogram);
        body.terminator(&format!("br label %{merge_label}"), false_location);

        body.start_block(&merge_label);
        self.emit_decision_phi(
            &[
                (true_value, true_predecessor),
                (false_value, false_predecessor),
            ],
            body,
            span,
        )
    }

    #[allow(clippy::too_many_arguments)] // Mirrors the checked decision node without hiding evaluation order.
    fn emit_ordered_comparison_decision(
        &mut self,
        subject: &CompilerExpression,
        rules: &[CompilerComparisonRule],
        otherwise: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let subject = self.emit_expression(subject, body, environment);
        let merge = body.label("comparison.decision.merge");
        let mut branches = Vec::with_capacity(rules.len() + 1);
        for rule in rules {
            let action_label = body.label("comparison.decision.action");
            let next_label = body.label("comparison.decision.next");
            let operand = self.emit_expression(&rule.operand, body, environment);
            let comparable_subject = if rule.subject_to_rational {
                LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.from.int(ptr {})",
                        subject.integer()
                    ),
                    rule.span,
                    &mut self.debug,
                ))
            } else {
                subject.clone()
            };
            let predicate = self
                .emit_binary(
                    rule.operation,
                    &comparable_subject,
                    &operand,
                    body,
                    rule.span,
                )
                .boolean()
                .to_owned();
            let location = self.debug.location(rule.span, body.subprogram);
            body.terminator(
                &format!("br i1 {predicate}, label %{action_label}, label %{next_label}"),
                location,
            );
            body.start_block(&action_label);
            let action = self.emit_expression(&rule.action, body, environment);
            let predecessor = body.current_block.clone();
            body.terminator(&format!("br label %{merge}"), location);
            branches.push((action, predecessor));
            body.start_block(&next_label);
        }
        let fallback = self.emit_expression(otherwise, body, environment);
        let fallback_predecessor = body.current_block.clone();
        let fallback_location = self.debug.location(otherwise.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), fallback_location);
        branches.push((fallback, fallback_predecessor));
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    #[allow(clippy::too_many_arguments)] // Mirrors the three closed Comparison alternatives.
    fn emit_comparison_value_decision(
        &mut self,
        subject: &CompilerExpression,
        when_less: &CompilerExpression,
        when_equal: &CompilerExpression,
        when_greater: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let LlValue::Comparison(subject) = emitted_subject else {
            unreachable!("checked decision subject is Comparison")
        };
        let less = body.label("comparison.value.less");
        let equal = body.label("comparison.value.equal");
        let greater = body.label("comparison.value.greater");
        let merge = body.label("comparison.value.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!(
                "switch i32 {subject}, label %{greater} [ i32 -1, label %{less} i32 0, label %{equal} ]"
            ),
            location,
        );
        let mut branches = Vec::with_capacity(3);
        for (label, action) in [
            (&less, when_less),
            (&equal, when_equal),
            (&greater, when_greater),
        ] {
            body.start_block(label);
            let value = self.emit_expression(action, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(action.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    fn emit_enum_decision(
        &mut self,
        subject: &CompilerExpression,
        rules: &[CompilerEnumRule],
        otherwise: Option<&CompilerExpression>,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let LlValue::Enum { value: subject, .. } = emitted_subject else {
            unreachable!("checked decision subject is a nominal Enum")
        };
        let labels = rules
            .iter()
            .map(|_| body.label("enum.decision.alternative"))
            .collect::<Vec<_>>();
        let default = body.label("enum.decision.default");
        let merge = body.label("enum.decision.merge");
        let cases = rules
            .iter()
            .zip(&labels)
            .map(|(rule, label)| format!("i32 {}, label %{label}", rule.value))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {subject}, label %{default} [ {cases} ]"),
            location,
        );
        let mut branches = Vec::with_capacity(rules.len() + usize::from(otherwise.is_some()));
        for (rule, label) in rules.iter().zip(labels) {
            body.start_block(&label);
            let value = self.emit_expression(&rule.action, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(rule.action.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        }
        body.start_block(&default);
        if let Some(otherwise) = otherwise {
            let value = self.emit_expression(otherwise, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(otherwise.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        } else {
            body.effect(
                "call void @topal.platform.exit(i64 70)",
                span,
                &mut self.debug,
            );
            body.terminator("unreachable", location);
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    fn emit_sum_decision(
        &mut self,
        subject: &CompilerExpression,
        rules: &[CompilerSumRule],
        otherwise: Option<&CompilerExpression>,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let LlValue::Sum { tag, payloads, sum } = emitted_subject else {
            unreachable!("checked decision subject is a nominal sum")
        };
        let labels = rules
            .iter()
            .map(|_| body.label("sum.decision.alternative"))
            .collect::<Vec<_>>();
        let default = body.label("sum.decision.default");
        let merge = body.label("sum.decision.merge");
        let cases = rules
            .iter()
            .zip(&labels)
            .map(|(rule, label)| format!("i32 {}, label %{label}", rule.value))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {tag}, label %{default} [ {cases} ]"),
            location,
        );
        let mut branches = Vec::with_capacity(rules.len() + usize::from(otherwise.is_some()));
        for (rule, label) in rules.iter().zip(labels) {
            body.start_block(&label);
            let mut branch_environment = environment.clone();
            if let Some((name, binding_span)) = &rule.binding {
                let index = usize::try_from(rule.value).expect("u32 sum tag fits usize");
                let payload_type = sum.alternatives[index]
                    .payload
                    .as_ref()
                    .expect("checked payload matcher retains its payload type");
                let payload = payloads[index]
                    .as_deref()
                    .expect("checked payload matcher retains its payload value")
                    .clone();
                let variable = self
                    .debug
                    .local(name, *binding_span, payload_type, body.subprogram);
                let binding_location = self.debug.location(*binding_span, body.subprogram);
                if payload_type.machine_scalar() {
                    body.debug_value(&payload, variable, binding_location);
                } else if private_aggregate_value_supported(payload_type) {
                    let aggregate = match (&payload, payload_type) {
                        (LlValue::Tuple(fields), CompilerType::Tuple(field_types)) => {
                            self.emit_tuple_aggregate(fields, field_types, body, *binding_span)
                        }
                        (LlValue::Record { fields, order }, CompilerType::Record(field_types)) => {
                            self.emit_record_aggregate(
                                fields,
                                order,
                                field_types,
                                body,
                                *binding_span,
                            )
                        }
                        (LlValue::Sum { .. }, CompilerType::Sum(_)) => {
                            self.emit_sum_aggregate(&payload, body, *binding_span)
                        }
                        _ => unreachable!("checked sum payload retains its aggregate type"),
                    };
                    self.emit_aggregate_debug_shadow(
                        &aggregate,
                        payload_type,
                        variable,
                        binding_location,
                        body,
                        *binding_span,
                    );
                }
                branch_environment.insert(name.clone(), payload);
            }
            let value = self.emit_expression(&rule.action, body, &branch_environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(rule.action.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        }
        body.start_block(&default);
        if let Some(otherwise) = otherwise {
            let value = self.emit_expression(otherwise, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(otherwise.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        } else {
            body.effect(
                "call void @topal.platform.exit(i64 70)",
                span,
                &mut self.debug,
            );
            body.terminator("unreachable", location);
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    #[allow(clippy::too_many_lines)] // Exhaustive joins preserve checked representation identity.
    fn emit_decision_phi(
        &mut self,
        branches: &[(LlValue, String)],
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let first = &branches.first().expect("complete decision has a branch").0;
        let incoming = |value: fn(&LlValue) -> &str| {
            branches
                .iter()
                .map(|(branch, predecessor)| format!("[{}, %{predecessor}]", value(branch)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        match first {
            LlValue::Unit => LlValue::Unit,
            LlValue::StaticDisplay(_) => {
                unreachable!("static Capability decision results are not admitted")
            }
            LlValue::Completed(_) | LlValue::Effect(_) => {
                let value = body.instruction(
                    &format!("phi i8 {}", incoming(LlValue::singleton)),
                    span,
                    &mut self.debug,
                );
                if matches!(first, LlValue::Completed(_)) {
                    LlValue::Completed(value)
                } else {
                    LlValue::Effect(value)
                }
            }
            LlValue::Boolean(_) => LlValue::Boolean(body.instruction(
                &format!("phi i1 {}", incoming(LlValue::boolean)),
                span,
                &mut self.debug,
            )),
            LlValue::Version { .. } => {
                unreachable!("Version decision results are not admitted")
            }
            LlValue::Function { .. } => {
                unreachable!("captured Function decision results are not admitted")
            }
            LlValue::SerializationStream { .. } => {
                let payload_branches = branches
                    .iter()
                    .map(|(branch, predecessor)| {
                        let LlValue::SerializationStream { value, .. } = branch else {
                            unreachable!(
                                "checked decision branches share a SerializationStream type"
                            )
                        };
                        (value.as_ref().clone(), predecessor.clone())
                    })
                    .collect::<Vec<_>>();
                LlValue::SerializationStream {
                    stream: body.instruction(
                        &format!(
                            "phi ptr {}",
                            incoming(LlValue::serialization_stream_pointer)
                        ),
                        span,
                        &mut self.debug,
                    ),
                    expected: body.instruction(
                        &format!(
                            "phi ptr {}",
                            incoming(LlValue::serialization_expected_pointer)
                        ),
                        span,
                        &mut self.debug,
                    ),
                    byte_count: body.instruction(
                        &format!("phi i64 {}", incoming(LlValue::serialization_byte_count)),
                        span,
                        &mut self.debug,
                    ),
                    value: Box::new(self.emit_decision_phi(&payload_branches, body, span)),
                }
            }
            LlValue::Task { task, .. } => LlValue::Task {
                value: body.instruction(
                    &format!("phi ptr {}", incoming(LlValue::task_pointer)),
                    span,
                    &mut self.debug,
                ),
                task: task.clone(),
            },
            LlValue::ExternalLocation { location, .. } => LlValue::ExternalLocation {
                value: body.instruction(
                    &format!("phi ptr {}", incoming(LlValue::location_pointer)),
                    span,
                    &mut self.debug,
                ),
                location: location.clone(),
            },
            LlValue::Int(_) => LlValue::Int(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::integer)),
                span,
                &mut self.debug,
            )),
            LlValue::Modular { modular, .. } => LlValue::Modular {
                value: body.instruction(
                    &format!("phi ptr {}", incoming(LlValue::modular_pointer)),
                    span,
                    &mut self.debug,
                ),
                modular: modular.clone(),
            },
            LlValue::Rational(_) => LlValue::Rational(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::rational)),
                span,
                &mut self.debug,
            )),
            LlValue::String(_) => LlValue::String(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::string)),
                span,
                &mut self.debug,
            )),
            LlValue::Error(_) => LlValue::Error(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::error)),
                span,
                &mut self.debug,
            )),
            LlValue::ErrorDomain(_) => LlValue::ErrorDomain(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::error_domain)),
                span,
                &mut self.debug,
            )),
            LlValue::SourceLocation(_) => LlValue::SourceLocation(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::source_location)),
                span,
                &mut self.debug,
            )),
            LlValue::Comparison(_) => LlValue::Comparison(body.instruction(
                &format!("phi i32 {}", incoming(LlValue::comparison)),
                span,
                &mut self.debug,
            )),
            LlValue::ErrorCode(_) => LlValue::ErrorCode(body.instruction(
                &format!("phi i32 {}", incoming(LlValue::error_code)),
                span,
                &mut self.debug,
            )),
            LlValue::Enum { enumeration, .. } => LlValue::Enum {
                value: body.instruction(
                    &format!("phi i32 {}", incoming(LlValue::enumeration)),
                    span,
                    &mut self.debug,
                ),
                enumeration: enumeration.clone(),
            },
            LlValue::Generator { generator, .. } => LlValue::Generator {
                value: body.instruction(
                    &format!("phi i32 {}", incoming(LlValue::generator_token)),
                    span,
                    &mut self.debug,
                ),
                generator: generator.clone(),
                captured_initial: None,
                captured_additional_initials: Vec::new(),
            },
            LlValue::Range { endpoint, .. } => {
                let endpoint = endpoint.clone();
                LlValue::Range {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::range_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    endpoint,
                }
            }
            LlValue::Result { success, .. } => {
                let success = success.clone();
                LlValue::Result {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::result_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    success,
                    function_captures: Vec::new(),
                }
            }
            LlValue::Optional { payload, .. } => {
                let payload = payload.clone();
                LlValue::Optional {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::optional_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    payload,
                    function_captures: Vec::new(),
                }
            }
            LlValue::TraversalControl { payload, .. } => {
                let payload = payload.clone();
                LlValue::TraversalControl {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::traversal_control_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    payload,
                }
            }
            LlValue::List { element, .. } => {
                let element = element.clone();
                LlValue::List {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::list_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    element,
                    function_captures: Vec::new(),
                }
            }
            LlValue::Container { value_type, .. } => {
                let value_type = value_type.clone();
                LlValue::Container {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::container_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    value_type,
                    function_captures: Vec::new(),
                    function_capture_keys: Vec::new(),
                }
            }
            LlValue::Sum { sum, .. } => {
                let sum = sum.clone();
                let tags = branches
                    .iter()
                    .map(|(branch, predecessor)| {
                        let LlValue::Sum { tag, .. } = branch else {
                            unreachable!("checked decision branches share a sum type")
                        };
                        format!("[{tag}, %{predecessor}]")
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let tag = body.instruction(&format!("phi i32 {tags}"), span, &mut self.debug);
                let payloads = sum
                    .alternatives
                    .iter()
                    .enumerate()
                    .map(|(index, alternative)| {
                        alternative.payload.as_ref().map(|_| {
                            let payload_branches = branches
                                .iter()
                                .map(|(branch, predecessor)| {
                                    let LlValue::Sum { payloads, .. } = branch else {
                                        unreachable!("checked decision branches share a sum type")
                                    };
                                    (
                                        payloads[index]
                                            .as_deref()
                                            .expect("sum payload slot retains a machine value")
                                            .clone(),
                                        predecessor.clone(),
                                    )
                                })
                                .collect::<Vec<_>>();
                            Box::new(self.emit_decision_phi(&payload_branches, body, span))
                        })
                    })
                    .collect();
                LlValue::Sum { tag, payloads, sum }
            }
            LlValue::Tuple(fields) => LlValue::Tuple(
                (0..fields.len())
                    .map(|index| {
                        let field_branches = branches
                            .iter()
                            .map(|(branch, predecessor)| {
                                let LlValue::Tuple(fields) = branch else {
                                    unreachable!("checked decision branches share a Tuple type")
                                };
                                (fields[index].clone(), predecessor.clone())
                            })
                            .collect::<Vec<_>>();
                        self.emit_decision_phi(&field_branches, body, span)
                    })
                    .collect(),
            ),
            LlValue::Record { fields, order } => {
                let fields = (0..fields.len())
                    .map(|index| {
                        let field_branches = branches
                            .iter()
                            .map(|(branch, predecessor)| {
                                let LlValue::Record { fields, .. } = branch else {
                                    unreachable!("checked decision branches share a Record type")
                                };
                                (fields[index].1.clone(), predecessor.clone())
                            })
                            .collect::<Vec<_>>();
                        (
                            fields[index].0.clone(),
                            self.emit_decision_phi(&field_branches, body, span),
                        )
                    })
                    .collect();
                let order = (0..order.len())
                    .map(|index| {
                        let incoming = branches
                            .iter()
                            .map(|(branch, predecessor)| {
                                let LlValue::Record { order, .. } = branch else {
                                    unreachable!("checked decision branches share a Record type")
                                };
                                format!("[{}, %{predecessor}]", order[index])
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        body.instruction(&format!("phi i32 {incoming}"), span, &mut self.debug)
                    })
                    .collect();
                LlValue::Record { fields, order }
            }
        }
    }

    #[allow(clippy::too_many_lines)] // Source value rendering keeps every admitted representation explicit.
    fn emit_print(&mut self, value: &LlValue, body: &mut FunctionBody, span: Span) {
        match value {
            LlValue::Unit => self.emit_write_literal("()", body, span),
            LlValue::Completed(_) => self.emit_write_literal("Completed", body, span),
            LlValue::Effect(_) => self.emit_write_literal("Effects ()", body, span),
            LlValue::Boolean(value) => {
                let true_label = body.label("print.true");
                let false_label = body.label("print.false");
                let done_label = body.label("print.done");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("br i1 {value}, label %{true_label}, label %{false_label}"),
                    location,
                );
                body.start_block(&true_label);
                self.emit_write_literal("true", body, span);
                body.terminator(&format!("br label %{done_label}"), location);
                body.start_block(&false_label);
                self.emit_write_literal("false", body, span);
                body.terminator(&format!("br label %{done_label}"), location);
                body.start_block(&done_label);
            }
            LlValue::Version { display, .. } | LlValue::StaticDisplay(display) => {
                self.emit_write_literal(display, body, span);
            }
            LlValue::SerializationStream { byte_count, .. } => {
                self.emit_write_literal("SerializationStream ( ", body, span);
                body.effect(
                    &format!("call void @topal.runtime.u64.print(i64 {byte_count})"),
                    span,
                    &mut self.debug,
                );
                self.emit_write_literal(" bytes )", body, span);
            }
            LlValue::Task { task, .. } => {
                self.emit_write_literal(
                    &format!("<{} {}>", task.classifier, task.identity),
                    body,
                    span,
                );
            }
            LlValue::ExternalLocation { location, .. } => {
                self.emit_write_literal(&format!("<{}>", location.identity), body, span);
            }
            LlValue::Int(value) => body.effect(
                &format!("call void @topal.runtime.int.print(ptr {value})"),
                span,
                &mut self.debug,
            ),
            LlValue::Modular { value, modular } => {
                self.emit_write_literal(&modular.name, body, span);
                self.emit_write_literal(" ", body, span);
                body.effect(
                    &format!("call void @topal.runtime.int.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::Rational(value) => body.effect(
                &format!("call void @topal.runtime.rational.print(ptr {value})"),
                span,
                &mut self.debug,
            ),
            LlValue::Comparison(value) => {
                let less = body.label("print.less");
                let equal = body.label("print.equal");
                let greater = body.label("print.greater");
                let done = body.label("print.comparison.done");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!(
                        "switch i32 {value}, label %{greater} [ i32 -1, label %{less} i32 0, label %{equal} ]"
                    ),
                    location,
                );
                for (label, text) in [(&less, "Less"), (&equal, "Equal"), (&greater, "Greater")] {
                    body.start_block(label);
                    self.emit_write_literal(text, body, span);
                    body.terminator(&format!("br label %{done}"), location);
                }
                body.start_block(&done);
            }
            LlValue::Enum { value, enumeration }
            | LlValue::Function {
                value, enumeration, ..
            } => {
                self.emit_print_enum(value, enumeration, body, span);
            }
            LlValue::Generator { generator, .. } => {
                self.emit_write_literal(
                    &format!("<{}>", CompilerType::Generator(generator.clone()).name()),
                    body,
                    span,
                );
            }
            LlValue::Sum { tag, payloads, sum } => {
                let labels = sum
                    .alternatives
                    .iter()
                    .map(|_| body.label("print.sum.alternative"))
                    .collect::<Vec<_>>();
                let invalid = body.label("print.sum.invalid");
                let done = body.label("print.sum.done");
                let cases = labels
                    .iter()
                    .enumerate()
                    .map(|(index, label)| format!("i32 {index}, label %{label}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("switch i32 {tag}, label %{invalid} [ {cases} ]"),
                    location,
                );
                for (index, (label, alternative)) in
                    labels.iter().zip(&sum.alternatives).enumerate()
                {
                    body.start_block(label);
                    self.emit_write_literal(&alternative.name, body, span);
                    if let Some(payload) = payloads[index].as_deref() {
                        self.emit_write_literal(" ", body, span);
                        self.emit_print(payload, body, span);
                    }
                    body.terminator(&format!("br label %{done}"), location);
                }
                body.start_block(&invalid);
                body.effect(
                    "call void @topal.platform.exit(i64 70)",
                    span,
                    &mut self.debug,
                );
                body.terminator("unreachable", location);
                body.start_block(&done);
            }
            LlValue::Range { value, endpoint } => body.effect(
                &format!(
                    "call void @topal.runtime.range.{}.print(ptr {value})",
                    numeric_domain(endpoint)
                ),
                span,
                &mut self.debug,
            ),
            LlValue::Result { value, success, .. } => {
                self.emit_print_result(value, success, body, span);
            }
            LlValue::Optional { value, payload, .. } => {
                self.emit_print_optional(value, payload, body, span);
            }
            LlValue::TraversalControl { value, payload } => {
                self.emit_print_traversal_control(value, payload, body, span);
            }
            LlValue::List { value, element, .. } => {
                self.emit_print_list(value, element, body, span);
            }
            LlValue::Container {
                value, value_type, ..
            } => {
                self.emit_print_container(value, value_type, body, span);
            }
            LlValue::String(value) => {
                body.effect(
                    &format!("call void @topal.runtime.string.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::Error(value) => {
                body.effect(
                    &format!("call void @topal.runtime.error.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::ErrorCode(value) => {
                body.effect(
                    &format!("call void @topal.runtime.error.code.print(i32 {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::ErrorDomain(value) => {
                body.effect(
                    &format!("call void @topal.runtime.string.raw.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::SourceLocation(value) => {
                self.emit_write_literal("(line is ", body, span);
                let line = body.instruction(
                    &format!("call ptr @topal.runtime.source.location.line(ptr {value})"),
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("call void @topal.runtime.int.print(ptr {line})"),
                    span,
                    &mut self.debug,
                );
                self.emit_write_literal(", column is ", body, span);
                let column = body.instruction(
                    &format!("call ptr @topal.runtime.source.location.column(ptr {value})"),
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("call void @topal.runtime.int.print(ptr {column})"),
                    span,
                    &mut self.debug,
                );
                self.emit_write_literal(")", body, span);
            }
            LlValue::Tuple(fields) => {
                self.emit_write_literal("(", body, span);
                for (index, field) in fields.iter().enumerate() {
                    if index != 0 {
                        self.emit_write_literal(", ", body, span);
                    }
                    self.emit_print(field, body, span);
                }
                if fields.len() == 1 {
                    self.emit_write_literal(",", body, span);
                }
                self.emit_write_literal(")", body, span);
            }
            LlValue::Record { fields, order } => {
                self.emit_print_record(fields, order, body, span);
            }
        }
    }

    fn emit_print_container(
        &mut self,
        container: &str,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        match value_type {
            CompilerType::Array { element, .. } => {
                let entries = self.emit_container_entries(container, 8, body, span);
                self.emit_print_sequence_container("Array", &entries, element, body, span);
            }
            CompilerType::Set(element) => {
                let entries = self.emit_container_entries(container, 8, body, span);
                self.emit_print_sequence_container("Set", &entries, element, body, span);
            }
            CompilerType::Bag(element) if element.as_ref() == &CompilerType::Int => {
                let entries = self.emit_container_entries(container, 16, body, span);
                self.emit_print_bag(&entries, body, span);
            }
            CompilerType::Map {
                key,
                value: map_value,
            } if key.as_ref() == &CompilerType::String
                && matches!(
                    map_value.as_ref(),
                    CompilerType::Int | CompilerType::Function
                ) =>
            {
                let entries = self.emit_container_entries(container, 8, body, span);
                self.emit_print_map(&entries, map_value, body, span);
            }
            _ => unreachable!("checked fundamental container has an admitted printer"),
        }
    }

    fn emit_container_entries(
        &mut self,
        container: &str,
        offset: usize,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        let address = body.instruction(
            &format!("getelementptr i8, ptr {container}, i64 {offset}"),
            span,
            &mut self.debug,
        );
        body.instruction(
            &format!("load ptr, ptr {address}, align 8"),
            span,
            &mut self.debug,
        )
    }

    fn emit_print_sequence_container(
        &mut self,
        name: &str,
        entries: &str,
        element: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        debug_assert!(matches!(
            element,
            CompilerType::Int | CompilerType::String | CompilerType::Function
        ));
        self.emit_write_literal(&format!("{name} ("), body, span);
        let initial = body.current_block.clone();
        let loop_label = body.label("print.container.loop");
        let entry = body.label("print.container.entry");
        let first = body.label("print.container.first");
        let separator = body.label("print.container.separator");
        let render = body.label("print.container.render");
        let advance = body.label("print.container.advance");
        let done = body.label("print.container.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{render}.next");
        let current = body.instruction(
            &format!("phi ptr [{entries}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_first = body.instruction(
            &format!("phi i1 [true, %{initial}], [false, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{entry}"),
            location,
        );

        body.start_block(&entry);
        body.terminator(
            &format!("br i1 {is_first}, label %{first}, label %{separator}"),
            location,
        );
        body.start_block(&first);
        body.terminator(&format!("br label %{render}"), location);
        body.start_block(&separator);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{render}"), location);

        body.start_block(&render);
        let value = match element {
            CompilerType::Int => LlValue::Int(body.instruction(
                &format!("load ptr, ptr {current}, align 8"),
                span,
                &mut self.debug,
            )),
            CompilerType::String => LlValue::String(body.instruction(
                &format!("load ptr, ptr {current}, align 8"),
                span,
                &mut self.debug,
            )),
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {current}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            _ => unreachable!("checked sequence container element is supported"),
        };
        self.emit_print(&value, body, span);
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_bag(&mut self, entries: &str, body: &mut FunctionBody, span: Span) {
        self.emit_write_literal("Bag (", body, span);
        let initial = body.current_block.clone();
        let loop_label = body.label("print.bag.loop");
        let entry = body.label("print.bag.entry");
        let first = body.label("print.bag.first");
        let separator = body.label("print.bag.separator");
        let render = body.label("print.bag.render");
        let advance = body.label("print.bag.advance");
        let done = body.label("print.bag.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{render}.next");
        let current = body.instruction(
            &format!("phi ptr [{entries}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_first = body.instruction(
            &format!("phi i1 [true, %{initial}], [false, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{entry}"),
            location,
        );
        body.start_block(&entry);
        body.terminator(
            &format!("br i1 {is_first}, label %{first}, label %{separator}"),
            location,
        );
        body.start_block(&first);
        body.terminator(&format!("br label %{render}"), location);
        body.start_block(&separator);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{render}"), location);

        body.start_block(&render);
        let value = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            span,
            &mut self.debug,
        );
        let count_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        let count = body.instruction(
            &format!("load i64, ptr {count_address}, align 8"),
            span,
            &mut self.debug,
        );
        let count = body.instruction(
            &format!("call ptr @topal.runtime.int.from.u64(i64 {count})"),
            span,
            &mut self.debug,
        );
        self.emit_print(
            &LlValue::Tuple(vec![LlValue::Int(value), LlValue::Int(count)]),
            body,
            span,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 16"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);
        body.start_block(&done);
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_map(
        &mut self,
        entries: &str,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        self.emit_write_literal("Map (", body, span);
        let initial = body.current_block.clone();
        let loop_label = body.label("print.map.loop");
        let entry = body.label("print.map.entry");
        let first = body.label("print.map.first");
        let separator = body.label("print.map.separator");
        let render = body.label("print.map.render");
        let advance = body.label("print.map.advance");
        let done = body.label("print.map.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{render}.next");
        let current = body.instruction(
            &format!("phi ptr [{entries}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_first = body.instruction(
            &format!("phi i1 [true, %{initial}], [false, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{entry}"),
            location,
        );
        body.start_block(&entry);
        body.terminator(
            &format!("br i1 {is_first}, label %{first}, label %{separator}"),
            location,
        );
        body.start_block(&first);
        body.terminator(&format!("br label %{render}"), location);
        body.start_block(&separator);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{render}"), location);

        body.start_block(&render);
        let key = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            span,
            &mut self.debug,
        );
        let value_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        let value = match value_type {
            CompilerType::Int => LlValue::Int(body.instruction(
                &format!("load ptr, ptr {value_address}, align 8"),
                span,
                &mut self.debug,
            )),
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {value_address}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            _ => unreachable!("checked Map value has an admitted printer"),
        };
        self.emit_print(
            &LlValue::Tuple(vec![LlValue::String(key), value]),
            body,
            span,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 16"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);
        body.start_block(&done);
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_record(
        &mut self,
        fields: &[(String, LlValue)],
        order: &[String],
        body: &mut FunctionBody,
        span: Span,
    ) {
        debug_assert_eq!(fields.len(), order.len());
        self.emit_write_literal("(", body, span);
        for (position, selector) in order.iter().enumerate() {
            if position != 0 {
                self.emit_write_literal(", ", body, span);
            }
            if let Ok(index) = selector.parse::<usize>() {
                let (label, field) = &fields[index];
                self.emit_write_literal(label, body, span);
                self.emit_write_literal(" is ", body, span);
                self.emit_print(field, body, span);
            } else {
                self.emit_print_dynamic_record_field(selector, fields, body, span);
            }
        }
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_dynamic_record_field(
        &mut self,
        selector: &str,
        fields: &[(String, LlValue)],
        body: &mut FunctionBody,
        span: Span,
    ) {
        let labels = fields
            .iter()
            .map(|_| body.label("print.record.field"))
            .collect::<Vec<_>>();
        let invalid = body.label("print.record.invalid");
        let done = body.label("print.record.done");
        let cases = labels
            .iter()
            .enumerate()
            .map(|(index, label)| format!("i32 {index}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {selector}, label %{invalid} [ {cases} ]"),
            location,
        );
        for (branch, (label, field)) in labels.iter().zip(fields) {
            body.start_block(branch);
            self.emit_write_literal(label, body, span);
            self.emit_write_literal(" is ", body, span);
            self.emit_print(field, body, span);
            body.terminator(&format!("br label %{done}"), location);
        }
        body.start_block(&invalid);
        body.effect(
            "call void @topal.platform.exit(i64 70)",
            span,
            &mut self.debug,
        );
        body.terminator("unreachable", location);
        body.start_block(&done);
    }

    fn emit_print_enum(
        &mut self,
        value: &str,
        enumeration: &CompilerEnumType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let labels = enumeration
            .alternatives
            .iter()
            .map(|_| body.label("print.enum.alternative"))
            .collect::<Vec<_>>();
        let invalid = body.label("print.enum.invalid");
        let done = body.label("print.enum.done");
        let cases = labels
            .iter()
            .enumerate()
            .map(|(index, label)| format!("i32 {index}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {value}, label %{invalid} [ {cases} ]"),
            location,
        );
        for (label, alternative) in labels.iter().zip(&enumeration.alternatives) {
            body.start_block(label);
            self.emit_write_literal(alternative, body, span);
            body.terminator(&format!("br label %{done}"), location);
        }
        body.start_block(&invalid);
        body.effect(
            "call void @topal.platform.exit(i64 70)",
            span,
            &mut self.debug,
        );
        body.terminator("unreachable", location);
        body.start_block(&done);
    }

    fn emit_print_result(
        &mut self,
        value: &str,
        success: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let is_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
            span,
            &mut self.debug,
        );
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            span,
            &mut self.debug,
        );
        let error = body.label("print.result.error");
        let ok = body.label("print.result.ok");
        let done = body.label("print.result.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_error}, label %{error}, label %{ok}"),
            location,
        );
        body.start_block(&error);
        body.effect(
            &format!("call void @topal.runtime.error.print(ptr {payload})"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&ok);
        let success = self.result_success_value(&payload, success, body, span);
        self.emit_print(&success, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&done);
    }

    fn emit_print_optional(
        &mut self,
        value: &str,
        payload_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let is_some = body.instruction(
            &format!("call i1 @topal.runtime.optional.is.some(ptr {value})"),
            span,
            &mut self.debug,
        );
        let some = body.label("print.optional.some");
        let none = body.label("print.optional.none");
        let done = body.label("print.optional.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_some}, label %{some}, label %{none}"),
            location,
        );
        body.start_block(&some);
        self.emit_write_literal("Some ", body, span);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.optional.payload(ptr {value})"),
            span,
            &mut self.debug,
        );
        let payload = self.emit_optional_payload_value(payload, payload_type, body, span);
        self.emit_print(&payload, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&none);
        self.emit_write_literal("None", body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&done);
    }

    fn emit_print_traversal_control(
        &mut self,
        value: &str,
        payload_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let tag = body.instruction(
            &format!("load i64, ptr {value}, align 8"),
            span,
            &mut self.debug,
        );
        let is_finish = body.instruction(&format!("icmp eq i64 {tag}, 1"), span, &mut self.debug);
        let payload_address = body.instruction(
            &format!("getelementptr i8, ptr {value}, i64 8"),
            span,
            &mut self.debug,
        );
        let payload = body.instruction(
            &format!("load ptr, ptr {payload_address}, align 8"),
            span,
            &mut self.debug,
        );
        let continue_label = body.label("print.traversal.continue");
        let finish_label = body.label("print.traversal.finish");
        let done = body.label("print.traversal.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_finish}, label %{finish_label}, label %{continue_label}"),
            location,
        );
        body.start_block(&continue_label);
        self.emit_write_literal("Continue ", body, span);
        let payload = match payload_type {
            CompilerType::Int => LlValue::Int(payload.clone()),
            _ => unreachable!("checked TraversalControl payload has an admitted printer"),
        };
        self.emit_print(&payload, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&finish_label);
        self.emit_write_literal("Finish ", body, span);
        self.emit_print(&payload, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&done);
    }

    #[allow(clippy::too_many_lines)] // Admitted element layouts retain explicit loads and traversal.
    fn emit_print_list(
        &mut self,
        value: &str,
        element: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let initial = body.current_block.clone();
        let loop_label = body.label("print.list.loop");
        let entry = body.label("print.list.entry");
        let advance = body.label("print.list.advance");
        let empty = body.label("print.list.empty");
        let close_loop = body.label("print.list.close.loop");
        let close_one = body.label("print.list.close.one");
        let done = body.label("print.list.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{entry}.next");
        let next_depth_name = format!("%{entry}.depth.next");
        let current = body.instruction(
            &format!("phi ptr [{value}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let depth = body.instruction(
            &format!("phi i64 [0, %{initial}], [{next_depth_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {is_empty}, label %{empty}, label %{entry}"),
            location,
        );

        body.start_block(&entry);
        self.emit_write_literal("Entry ( ", body, span);
        let (payload, next_offset) = match element {
            CompilerType::Effect => (
                LlValue::Effect(body.instruction(
                    &format!("load i8, ptr {current}, align 1"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Boolean => (
                LlValue::Boolean(body.instruction(
                    &format!("load i1, ptr {current}, align 1"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Int | CompilerType::Nat => (
                LlValue::Int(body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Rational => (
                LlValue::Rational(body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Character | CompilerType::String => (
                LlValue::String(body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Function => (
                LlValue::Function {
                    value: body.instruction(
                        &format!("load i32, ptr {current}, align 4"),
                        span,
                        &mut self.debug,
                    ),
                    enumeration: function_value_enumeration(self.program),
                    captures: Vec::new(),
                },
                8,
            ),
            CompilerType::Tuple(fields)
                if matches!(
                    fields.as_slice(),
                    [CompilerType::Int, CompilerType::Int | CompilerType::String,]
                        | [CompilerType::String, CompilerType::Int]
                ) =>
            {
                let left = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                let right_address = body.instruction(
                    &format!("getelementptr i8, ptr {current}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let right = body.instruction(
                    &format!("load ptr, ptr {right_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                (
                    LlValue::Tuple(vec![
                        match &fields[0] {
                            CompilerType::Int => LlValue::Int(left),
                            CompilerType::String => LlValue::String(left),
                            _ => unreachable!(),
                        },
                        match &fields[1] {
                            CompilerType::Int => LlValue::Int(right),
                            CompilerType::String => LlValue::String(right),
                            _ => unreachable!(),
                        },
                    ]),
                    16,
                )
            }
            CompilerType::List(inner) if compiler_int_string_pair(inner) => (
                LlValue::List {
                    value: body.instruction(
                        &format!("load ptr, ptr {current}, align 8"),
                        span,
                        &mut self.debug,
                    ),
                    element: inner.as_ref().clone(),
                    function_captures: Vec::new(),
                },
                8,
            ),
            CompilerType::Record(fields)
                if fields.as_slice()
                    == [
                        ("index".into(), CompilerType::Int),
                        ("value".into(), CompilerType::Int),
                    ] =>
            {
                let index = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                let value_address = body.instruction(
                    &format!("getelementptr i8, ptr {current}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let value = body.instruction(
                    &format!("load ptr, ptr {value_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                (
                    LlValue::Record {
                        fields: vec![
                            ("index".into(), LlValue::Int(index)),
                            ("value".into(), LlValue::Int(value)),
                        ],
                        order: vec!["0".into(), "1".into()],
                    },
                    24,
                )
            }
            _ => unreachable!("checked List element has an admitted printer"),
        };
        self.emit_print(&payload, body, span);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 {next_offset}"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_depth_name,
            &format!("add i64 {depth}, 1"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&empty);
        self.emit_write_literal("Empty", body, span);
        body.terminator(&format!("br label %{close_loop}"), location);

        body.start_block(&close_loop);
        let close_next_name = format!("%{close_one}.next");
        let remaining = body.instruction(
            &format!("phi i64 [{depth}, %{empty}], [{close_next_name}, %{close_one}]"),
            span,
            &mut self.debug,
        );
        let closed = body.instruction(
            &format!("icmp eq i64 {remaining}, 0"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {closed}, label %{done}, label %{close_one}"),
            location,
        );

        body.start_block(&close_one);
        self.emit_write_literal(" )", body, span);
        body.named_instruction(
            &close_next_name,
            &format!("sub i64 {remaining}, 1"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{close_loop}"), location);
        body.start_block(&done);
    }

    fn emit_write_literal(&mut self, text: &str, body: &mut FunctionBody, span: Span) {
        if text.is_empty() {
            return;
        }
        let (name, length) = self.emit_bytes_global(text);
        body.effect(
            &format!("call void @topal.platform.write_all(ptr {name}, i64 {length})"),
            span,
            &mut self.debug,
        );
    }

    fn emit_bytes_global(&mut self, text: &str) -> (String, usize) {
        self.emit_raw_bytes_global(text.as_bytes())
    }

    fn emit_raw_bytes_global(&mut self, bytes: &[u8]) -> (String, usize) {
        let name = format!(".topal.bytes.{}", self.next_global);
        self.next_global += 1;
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            bytes.len(),
            llvm_bytes(bytes)
        ));
        (format!("@{name}"), bytes.len())
    }

    fn emit_string_literal(&mut self, text: &str, body: &mut FunctionBody, span: Span) -> LlValue {
        LlValue::String(self.emit_string_value(text, body, span))
    }

    fn emit_string_value(&mut self, text: &str, body: &mut FunctionBody, span: Span) -> String {
        let (bytes, length) = self.emit_bytes_global(text);
        let display = display_string_literal(text);
        let (display_bytes, display_length) = self.emit_bytes_global(&display);
        body.instruction(
            &format!(
                "call ptr @topal.runtime.string.make(ptr {bytes}, i64 {length}, ptr {display_bytes}, i64 {display_length})"
            ),
            span,
            &mut self.debug,
        )
    }

    fn emit_string_characters_foreach(
        &mut self,
        traversal: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        let CompilerExpressionKind::StringCharactersForeach {
            source,
            characters,
            parameter,
            body: action,
        } = &traversal.kind
        else {
            unreachable!("checked Character foreach retains its traversal")
        };
        self.emit_character_sequence_foreach(
            source,
            characters,
            None,
            parameter,
            action,
            body,
            environment,
            traversal.span,
        )
    }

    #[allow(clippy::too_many_arguments)] // The checked traversal pieces remain explicit at lowering.
    fn emit_character_sequence_foreach(
        &mut self,
        source: &CompilerExpression,
        characters: &[String],
        generator_local: Option<&CompilerGeneratorLocal>,
        parameter: &CompilerParameter,
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let _ = self.emit_expression(source, body, environment);
        let mut generator_local_address = None;
        let debug_address = (!characters.is_empty() && !parameter.discarded).then(|| {
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &parameter.value_type,
                body.subprogram,
            );
            let address = body.instruction("alloca ptr, align 8", span, &mut self.debug);
            let location = self.debug.location(parameter.span, body.subprogram);
            body.debug_declare(&address, variable, location);
            address
        });
        for (index, character) in characters.iter().enumerate() {
            if let Some(local) = generator_local
                && local.parameter.value_type == CompilerType::Character
                && local.activation_after_resumptions == index
            {
                let parameter = &local.parameter;
                let variable = self.debug.local(
                    &parameter.name,
                    parameter.span,
                    &parameter.value_type,
                    body.subprogram,
                );
                let address =
                    body.instruction("alloca ptr, align 8", parameter.span, &mut self.debug);
                let location = self.debug.location(parameter.span, body.subprogram);
                body.debug_declare(&address, variable, location);
                generator_local_address = Some(address);
            }
            let value = LlValue::String(self.emit_string_value(character, body, span));
            let mut action_environment = environment.clone();
            if let Some(local) = generator_local
                && local.parameter.value_type == CompilerType::Character
                && local.activation_after_resumptions == index
                && let Some(address) = &generator_local_address
            {
                body.effect(
                    &format!("store ptr {}, ptr {address}, align 8", value.string()),
                    local.parameter.span,
                    &mut self.debug,
                );
            }
            if let Some(address) = &debug_address {
                body.effect(
                    &format!("store ptr {}, ptr {address}, align 8", value.string()),
                    parameter.span,
                    &mut self.debug,
                );
            }
            if !parameter.discarded {
                action_environment.insert(parameter.name.clone(), value);
            }
            let action_value = self.emit_block(action, body, &mut action_environment);
            debug_assert!(matches!(action_value, LlValue::Unit));
            if let Some(local) = generator_local
                && local.parameter.value_type == CompilerType::Unit
                && local.activation_after_resumptions == index + 1
            {
                let parameter = &local.parameter;
                let variable = self.debug.local(
                    &parameter.name,
                    parameter.span,
                    &parameter.value_type,
                    body.subprogram,
                );
                let address =
                    body.instruction("alloca i8, align 1", parameter.span, &mut self.debug);
                let location = self.debug.location(parameter.span, body.subprogram);
                body.debug_declare(&address, variable, location);
                body.effect(
                    &format!("store i8 0, ptr {address}, align 1"),
                    parameter.span,
                    &mut self.debug,
                );
            }
        }
        LlValue::Unit
    }

    fn emit_custom_value_action_debug_address(
        &mut self,
        parameter: &CompilerParameter,
        span: Span,
        body: &mut FunctionBody,
    ) -> (String, u64) {
        let variable = self.debug.local(
            &parameter.name,
            parameter.span,
            &parameter.value_type,
            body.subprogram,
        );
        let llvm_type = llvm_value_type(&parameter.value_type);
        let alignment = target_value_layout(&parameter.value_type).alignment / 8;
        let address = body.instruction(
            &format!("alloca {llvm_type}, align {alignment}"),
            span,
            &mut self.debug,
        );
        let location = self.debug.location(parameter.span, body.subprogram);
        body.debug_declare(&address, variable, location);
        (address, alignment)
    }

    #[allow(clippy::too_many_arguments)] // Final source scope keeps every checked value explicit.
    fn emit_custom_value_final_debug(
        &mut self,
        declaration_span: Span,
        initial_parameter: &CompilerParameter,
        initial: &LlValue,
        explicit_return: Option<Span>,
        result_span: Span,
        traversal_span: Span,
        body: &mut FunctionBody,
    ) {
        if initial_parameter.value_type == CompilerType::Unit {
            let address = body.instruction("alloca i8, align 1", traversal_span, &mut self.debug);
            body.effect(
                &format!("store i8 0, ptr {address}, align 1"),
                traversal_span,
                &mut self.debug,
            );
            body.subprogram = self.debug.lexical_block(declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(result_span, body.subprogram);
            body.debug_declare(&address, variable, location);
            body.effect(
                &format!("store i8 0, ptr {address}, align 1"),
                result_span,
                &mut self.debug,
            );
        } else if explicit_return.is_some()
            || matches!(
                initial_parameter.value_type,
                CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Comparison
                    | CompilerType::Enum(_)
                    | CompilerType::Rational
                    | CompilerType::Optional(_)
                    | CompilerType::Range(_)
                    | CompilerType::Result(_)
                    | CompilerType::List(_)
                    | CompilerType::Tuple(_)
            )
        {
            let llvm_type = llvm_value_type(&initial_parameter.value_type);
            let alignment = target_value_layout(&initial_parameter.value_type).alignment / 8;
            let address = body.instruction(
                &format!("alloca {llvm_type}, align {alignment}"),
                traversal_span,
                &mut self.debug,
            );
            let initial_operand = self.emit_machine_operand(
                initial,
                &initial_parameter.value_type,
                body,
                traversal_span,
            );
            body.effect(
                &format!("store {initial_operand}, ptr {address}, align {alignment}"),
                traversal_span,
                &mut self.debug,
            );
            body.subprogram = self.debug.lexical_block(declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                body.subprogram,
            );
            let location = self
                .debug
                .location(explicit_return.unwrap_or(result_span), body.subprogram);
            body.debug_declare(&address, variable, location);
            if matches!(
                initial_parameter.value_type,
                CompilerType::Comparison
                    | CompilerType::Enum(_)
                    | CompilerType::Result(_)
                    | CompilerType::List(_)
                    | CompilerType::Tuple(_)
            ) {
                body.effect(
                    &format!("store {initial_operand}, ptr {address}, align {alignment}"),
                    result_span,
                    &mut self.debug,
                );
            }
        } else if initial_parameter.value_type == CompilerType::Boolean {
            body.subprogram = self.debug.lexical_block(declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(result_span, body.subprogram);
            body.debug_value(initial, variable, location);
        }
    }

    #[allow(clippy::too_many_lines)] // Ordered captures, suspensions, action scopes, and final-value debug lifetimes remain adjacent.
    fn emit_custom_value_foreach(
        &mut self,
        traversal: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        let CompilerExpressionKind::CustomValueForeach {
            source,
            transferred_initial,
            declaration_span,
            initial_parameter,
            additional_initial_parameters,
            prefix,
            yields,
            continuations,
            explicit_return,
            parameter,
            body: action,
            result,
        } = &traversal.kind
        else {
            unreachable!("checked value foreach retains its traversal")
        };
        let source = self.emit_expression(source, body, environment);
        let initial = if let Some(transferred_initial) = transferred_initial {
            self.emit_expression(transferred_initial, body, environment)
        } else {
            source.generator_initial().clone()
        };
        let additional_initials = source.generator_additional_initials().to_vec();
        let traversal_parent_scope = body.subprogram;
        let mut generator_environment = environment.clone();
        generator_environment.insert(initial_parameter.name.clone(), initial.clone());
        for (parameter, value) in additional_initial_parameters
            .iter()
            .zip(&additional_initials)
        {
            generator_environment.insert(parameter.name.clone(), value.clone());
        }
        if !additional_initial_parameters.is_empty() || !prefix.statements.is_empty() {
            body.subprogram = self
                .debug
                .lexical_block(*declaration_span, traversal_parent_scope);
            for (parameter, value) in std::iter::once(initial_parameter.as_ref())
                .chain(additional_initial_parameters)
                .zip(std::iter::once(&initial).chain(&additional_initials))
            {
                let variable = self.debug.local(
                    &parameter.name,
                    parameter.span,
                    &parameter.value_type,
                    body.subprogram,
                );
                let llvm_type = llvm_value_type(&parameter.value_type);
                let alignment = target_value_layout(&parameter.value_type).alignment / 8;
                let address = body.instruction(
                    &format!("alloca {llvm_type}, align {alignment}"),
                    parameter.span,
                    &mut self.debug,
                );
                let location = self.debug.location(parameter.span, body.subprogram);
                body.debug_declare(&address, variable, location);
                let operand =
                    self.emit_machine_operand(value, &parameter.value_type, body, parameter.span);
                body.effect(
                    &format!("store {operand}, ptr {address}, align {alignment}"),
                    parameter.span,
                    &mut self.debug,
                );
            }
            let prefix_value = self.emit_block(prefix, body, &mut generator_environment);
            debug_assert!(matches!(prefix_value, LlValue::Unit));
        }
        let continuation_debug = (!continuations.is_empty()).then(|| {
            let scope = self.debug.lexical_block(*declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                scope,
            );
            (scope, variable)
        });
        let generator_scope = body.subprogram;
        let action_scope = if additional_initial_parameters.is_empty() {
            generator_scope
        } else {
            self.debug
                .lexical_block(parameter.span, traversal_parent_scope)
        };
        let debug_address = (!yields.is_empty() && !parameter.discarded).then(|| {
            body.subprogram = action_scope;
            let address =
                self.emit_custom_value_action_debug_address(parameter, traversal.span, body);
            body.subprogram = generator_scope;
            address
        });
        for (index, yielded) in yields.iter().enumerate() {
            let (value, span) = match yielded {
                CompilerGeneratorYield::Initial(span) => (initial.clone(), *span),
                CompilerGeneratorYield::Value(value) => (
                    self.emit_expression(value, body, &generator_environment),
                    value.span,
                ),
            };
            if let Some((address, alignment)) = &debug_address {
                let operand = self.emit_machine_operand(&value, &parameter.value_type, body, span);
                let store = format!("store {operand}, ptr {address}, align {alignment}");
                body.effect(&store, span, &mut self.debug);
                if parameter.value_type == CompilerType::Boolean {
                    body.effect(&store, parameter.span, &mut self.debug);
                } else if matches!(
                    parameter.value_type,
                    CompilerType::Comparison
                        | CompilerType::Enum(_)
                        | CompilerType::Result(_)
                        | CompilerType::List(_)
                        | CompilerType::Tuple(_)
                ) {
                    let action_span =
                        action
                            .statements
                            .first()
                            .map_or(action.result.span, |statement| match statement {
                                CompilerStatement::Binding(binding) => binding.span,
                                CompilerStatement::Discard(value) => value.span,
                            });
                    body.effect(&store, action_span, &mut self.debug);
                } else if parameter.value_type == CompilerType::Unit {
                    body.effect(&store, action.result.span, &mut self.debug);
                }
            }
            let mut action_environment = environment.clone();
            if !parameter.discarded {
                action_environment.insert(parameter.name.clone(), value);
            }
            body.subprogram = action_scope;
            let action_value = self.emit_block(action, body, &mut action_environment);
            debug_assert!(matches!(action_value, LlValue::Unit));
            body.subprogram = generator_scope;
            if let Some(continuation) = continuations
                .iter()
                .find(|continuation| continuation.after_resumptions == index + 1)
            {
                let parent_scope = body.subprogram;
                if let Some((scope, variable)) = continuation_debug {
                    body.subprogram = scope;
                    let location = self.debug.location(initial_parameter.span, body.subprogram);
                    body.debug_value(&initial, variable, location);
                }
                let mut continuation_environment = generator_environment.clone();
                let continuation_value =
                    self.emit_block(&continuation.body, body, &mut continuation_environment);
                debug_assert!(matches!(continuation_value, LlValue::Unit));
                body.subprogram = parent_scope;
            }
        }
        if additional_initial_parameters.is_empty() {
            self.emit_custom_value_final_debug(
                *declaration_span,
                initial_parameter,
                &initial,
                *explicit_return,
                result.span,
                traversal.span,
                body,
            );
        }
        let result_environment = generator_environment;
        let result = self.emit_expression(result, body, &result_environment);
        body.subprogram = traversal_parent_scope;
        result
    }

    fn emit_int_literal(&mut self, value: &BigInt) -> LlValue {
        LlValue::Int(self.emit_int_global(value))
    }

    fn emit_version_literal(
        &mut self,
        components: [u64; 4],
        display: String,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let fields = components
            .into_iter()
            .map(|component| self.emit_int_global(&BigInt::from(component)))
            .collect::<Vec<_>>();
        let llvm_type = "{ ptr, ptr, ptr, ptr }";
        let value = body.instruction(
            &format!("alloca {llvm_type}, align 8"),
            span,
            &mut self.debug,
        );
        for (index, field) in fields.iter().enumerate() {
            let address = body.instruction(
                &format!("getelementptr {llvm_type}, ptr {value}, i32 0, i32 {index}"),
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {field}, ptr {address}, align 8"),
                span,
                &mut self.debug,
            );
        }
        LlValue::Version { value, display }
    }

    fn emit_int_global(&mut self, value: &BigInt) -> String {
        let (sign, limbs) = value.to_u32_digits();
        let negative = usize::from(sign == Sign::Minus);
        let name = format!(".topal.int.{}", self.next_global);
        self.next_global += 1;
        let values = limbs
            .iter()
            .map(|limb| format!("i32 {limb}"))
            .collect::<Vec<_>>()
            .join(", ");
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant {{ i64, i64, [{} x i32] }} {{ i64 {negative}, i64 {}, [{} x i32] [{values}] }}, align 8",
            limbs.len(),
            limbs.len(),
            limbs.len()
        ));
        format!("@{name}")
    }

    fn emit_modular_reduce(
        &mut self,
        value: &str,
        modular: &CompilerModularType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let lower = self.emit_int_global(&modular.lower);
        let modulus = self.emit_int_global(&(&modular.upper - &modular.lower + BigInt::from(1)));
        let shifted = body.instruction(
            &format!("call ptr @topal.runtime.int.subtract(ptr {value}, ptr {lower})"),
            span,
            &mut self.debug,
        );
        let residue = body.instruction(
            &format!("call ptr @topal.runtime.int.modulo(ptr {shifted}, ptr {modulus})"),
            span,
            &mut self.debug,
        );
        let canonical = body.instruction(
            &format!("call ptr @topal.runtime.int.add(ptr {residue}, ptr {lower})"),
            span,
            &mut self.debug,
        );
        LlValue::Modular {
            value: canonical,
            modular: modular.clone(),
        }
    }

    fn emit_rational_literal(
        &mut self,
        value: &BigRational,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let numerator = self.emit_int_global(value.numer());
        let denominator = self.emit_int_global(value.denom());
        LlValue::Rational(body.instruction(
            &format!("call ptr @topal.runtime.rational.raw(ptr {numerator}, ptr {denominator})"),
            span,
            &mut self.debug,
        ))
    }
}

#[derive(Clone)]
enum LlValue {
    Unit,
    StaticDisplay(String),
    Completed(String),
    Effect(String),
    Boolean(String),
    Version {
        value: String,
        display: String,
    },
    SerializationStream {
        stream: String,
        expected: String,
        byte_count: String,
        value: Box<Self>,
    },
    Task {
        value: String,
        task: CompilerTaskType,
    },
    ExternalLocation {
        value: String,
        location: CompilerLocationType,
    },
    Int(String),
    Modular {
        value: String,
        modular: CompilerModularType,
    },
    Rational(String),
    Comparison(String),
    Error(String),
    ErrorCode(String),
    ErrorDomain(String),
    SourceLocation(String),
    Enum {
        value: String,
        enumeration: CompilerEnumType,
    },
    Function {
        value: String,
        enumeration: CompilerEnumType,
        captures: Vec<(String, Self)>,
    },
    Generator {
        value: String,
        generator: CompilerGeneratorType,
        captured_initial: Option<Box<Self>>,
        captured_additional_initials: Vec<Self>,
    },
    Sum {
        tag: String,
        payloads: Vec<Option<Box<Self>>>,
        sum: CompilerSumType,
    },
    Range {
        value: String,
        endpoint: CompilerType,
    },
    Result {
        value: String,
        success: CompilerType,
        function_captures: Vec<(String, Self)>,
    },
    Optional {
        value: String,
        payload: CompilerType,
        function_captures: Vec<(String, Self)>,
    },
    TraversalControl {
        value: String,
        payload: CompilerType,
    },
    List {
        value: String,
        element: CompilerType,
        function_captures: Vec<Vec<(String, Self)>>,
    },
    Container {
        value: String,
        value_type: CompilerType,
        function_captures: Vec<Vec<(String, Self)>>,
        function_capture_keys: Vec<String>,
    },
    String(String),
    Tuple(Vec<Self>),
    Record {
        fields: Vec<(String, Self)>,
        order: Vec<String>,
    },
}

fn attach_function_capture(
    value: &mut LlValue,
    path: &[CompilerAggregatePathElement],
    storage_name: String,
    capture_value: LlValue,
) {
    let Some((first, rest)) = path.split_first() else {
        match value {
            LlValue::Function { captures, .. } => {
                captures.push((storage_name, capture_value));
            }
            LlValue::Enum {
                value: tag,
                enumeration,
            } => {
                *value = LlValue::Function {
                    value: tag.clone(),
                    enumeration: enumeration.clone(),
                    captures: vec![(storage_name, capture_value)],
                };
            }
            _ => unreachable!("checked capture path ends at a Function value"),
        }
        return;
    };
    match (first, value) {
        (CompilerAggregatePathElement::Tuple(index), LlValue::Tuple(fields)) => {
            attach_function_capture(&mut fields[*index], rest, storage_name, capture_value);
        }
        (CompilerAggregatePathElement::Record(name), LlValue::Record { fields, .. }) => {
            let field = fields
                .iter_mut()
                .find_map(|(label, field)| (label == name).then_some(field))
                .expect("checked capture path names a Record field");
            attach_function_capture(field, rest, storage_name, capture_value);
        }
        (
            CompilerAggregatePathElement::ListEntry(index),
            LlValue::List {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            if function_captures.len() <= *index {
                function_captures.resize_with(*index + 1, Vec::new);
            }
            function_captures[*index].push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::ArrayEntry(index),
            LlValue::Container {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            if function_captures.len() <= *index {
                function_captures.resize_with(*index + 1, Vec::new);
            }
            function_captures[*index].push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::MapValue(key),
            LlValue::Container {
                function_captures,
                function_capture_keys,
                ..
            },
        ) if rest.is_empty() => {
            let index = function_capture_keys
                .iter()
                .position(|candidate| candidate == key)
                .unwrap_or_else(|| {
                    function_capture_keys.push(key.clone());
                    function_captures.push(Vec::new());
                    function_capture_keys.len() - 1
                });
            function_captures[index].push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::OptionalPayload,
            LlValue::Optional {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            function_captures.push((storage_name, capture_value));
        }
        (
            CompilerAggregatePathElement::ResultSuccess,
            LlValue::Result {
                function_captures, ..
            },
        ) if rest.is_empty() => {
            function_captures.push((storage_name, capture_value));
        }
        (CompilerAggregatePathElement::SumPayload(name), LlValue::Sum { payloads, sum, .. }) => {
            let index = sum
                .alternatives
                .iter()
                .position(|alternative| alternative.name == *name)
                .expect("checked capture path names a Sum alternative");
            let payload = payloads[index]
                .as_deref_mut()
                .expect("checked capture path names a payload-bearing Sum alternative");
            attach_function_capture(payload, rest, storage_name, capture_value);
        }
        _ => unreachable!("checked capture path follows its aggregate representation"),
    }
}

fn function_capture_value(value: &LlValue, storage_name: &str) -> Option<LlValue> {
    match value {
        LlValue::Function { captures, .. } => captures
            .iter()
            .find_map(|(name, value)| (name == storage_name).then(|| value.clone())),
        LlValue::Tuple(fields) => fields
            .iter()
            .find_map(|field| function_capture_value(field, storage_name)),
        LlValue::Record { fields, .. } => fields
            .iter()
            .find_map(|(_, field)| function_capture_value(field, storage_name)),
        LlValue::List {
            function_captures, ..
        }
        | LlValue::Container {
            function_captures, ..
        } => function_captures.iter().find_map(|captures| {
            captures
                .iter()
                .find_map(|(name, value)| (name == storage_name).then(|| value.clone()))
        }),
        LlValue::Optional {
            function_captures, ..
        } => function_captures
            .iter()
            .find_map(|(name, value)| (name == storage_name).then(|| value.clone())),
        LlValue::Result {
            function_captures, ..
        } => function_captures
            .iter()
            .find_map(|(name, value)| (name == storage_name).then(|| value.clone())),
        LlValue::Sum { payloads, .. } => payloads.iter().find_map(|payload| {
            payload
                .as_deref()
                .and_then(|payload| function_capture_value(payload, storage_name))
        }),
        _ => None,
    }
}

fn extend_function_capture_environment(
    environment: &mut BTreeMap<String, LlValue>,
    value: &LlValue,
) {
    match value {
        LlValue::Function { captures, .. } => {
            environment.extend(captures.iter().cloned());
        }
        LlValue::Tuple(fields) => {
            for field in fields {
                extend_function_capture_environment(environment, field);
            }
        }
        LlValue::Record { fields, .. } => {
            for (_, field) in fields {
                extend_function_capture_environment(environment, field);
            }
        }
        LlValue::List {
            function_captures, ..
        }
        | LlValue::Container {
            function_captures, ..
        } => {
            for captures in function_captures {
                environment.extend(captures.iter().cloned());
            }
        }
        LlValue::Optional {
            function_captures, ..
        }
        | LlValue::Result {
            function_captures, ..
        } => {
            environment.extend(function_captures.iter().cloned());
        }
        LlValue::Sum { payloads, .. } => {
            for payload in payloads.iter().flatten() {
                extend_function_capture_environment(environment, payload);
            }
        }
        _ => {}
    }
}

impl LlValue {
    fn singleton(&self) -> &str {
        match self {
            Self::Completed(value) | Self::Effect(value) => value,
            _ => unreachable!("checked value is a zero-data singleton"),
        }
    }

    fn integer(&self) -> &str {
        match self {
            Self::Int(value) | Self::Modular { value, .. } => value,
            _ => unreachable!("checked value has an integer representation"),
        }
    }

    fn modular_pointer(&self) -> &str {
        let Self::Modular { value, .. } = self else {
            unreachable!("checked value is modular")
        };
        value
    }

    fn boolean(&self) -> &str {
        let Self::Boolean(value) = self else {
            unreachable!("checked value is Boolean")
        };
        value
    }

    fn rational(&self) -> &str {
        let Self::Rational(value) = self else {
            unreachable!("checked value is Rational")
        };
        value
    }

    fn comparison(&self) -> &str {
        let Self::Comparison(value) = self else {
            unreachable!("checked value is Comparison")
        };
        value
    }

    fn error(&self) -> &str {
        let Self::Error(value) = self else {
            unreachable!("checked value is Error")
        };
        value
    }

    fn error_code(&self) -> &str {
        let Self::ErrorCode(value) = self else {
            unreachable!("checked value is ErrorCode")
        };
        value
    }

    fn error_domain(&self) -> &str {
        let Self::ErrorDomain(value) = self else {
            unreachable!("checked value is ErrorDomain")
        };
        value
    }

    fn source_location(&self) -> &str {
        let Self::SourceLocation(value) = self else {
            unreachable!("checked value is SourceLocation")
        };
        value
    }

    fn enumeration(&self) -> &str {
        match self {
            Self::Enum { value, .. } | Self::Function { value, .. } => value,
            _ => unreachable!("checked value has an enum-like observation tag"),
        }
    }

    fn generator_token(&self) -> &str {
        let Self::Generator { value, .. } = self else {
            unreachable!("checked value is a construction-only Generator")
        };
        value
    }

    fn generator_initial(&self) -> &Self {
        let Self::Generator {
            captured_initial: Some(initial),
            ..
        } = self
        else {
            unreachable!("checked custom value Generator retains its initial value")
        };
        initial
    }

    fn generator_additional_initials(&self) -> &[Self] {
        let Self::Generator {
            captured_additional_initials,
            ..
        } = self
        else {
            unreachable!("checked custom value Generator retains its captured inputs")
        };
        captured_additional_initials
    }

    fn string(&self) -> &str {
        let Self::String(value) = self else {
            unreachable!("checked value is String")
        };
        value
    }

    fn int_or_string_pointer(&self) -> &str {
        match self {
            Self::Int(value) | Self::String(value) => value,
            _ => unreachable!("checked List pair field is Int or String"),
        }
    }

    fn range(&self) -> (&str, &CompilerType) {
        let Self::Range { value, endpoint } = self else {
            unreachable!("checked value is Range")
        };
        (value, endpoint)
    }

    fn range_pointer(&self) -> &str {
        self.range().0
    }

    fn result_pointer(&self) -> &str {
        let Self::Result { value, .. } = self else {
            unreachable!("checked value is Result")
        };
        value
    }

    fn optional_pointer(&self) -> &str {
        let Self::Optional { value, .. } = self else {
            unreachable!("checked value is Optional")
        };
        value
    }

    fn traversal_control(&self) -> (&str, &CompilerType) {
        let Self::TraversalControl { value, payload } = self else {
            unreachable!("checked value is TraversalControl")
        };
        (value, payload)
    }

    fn traversal_control_pointer(&self) -> &str {
        self.traversal_control().0
    }

    fn list_pointer(&self) -> &str {
        let Self::List { value, .. } = self else {
            unreachable!("checked value is List")
        };
        value
    }

    fn container_pointer(&self) -> &str {
        let Self::Container { value, .. } = self else {
            unreachable!("checked value is a fundamental container")
        };
        value
    }

    fn serialization_stream_pointer(&self) -> &str {
        let Self::SerializationStream { stream, .. } = self else {
            unreachable!("checked value is a SerializationStream")
        };
        stream
    }

    fn task_pointer(&self) -> &str {
        let Self::Task { value, .. } = self else {
            unreachable!("checked value is a Task instance")
        };
        value
    }

    fn location_pointer(&self) -> &str {
        let Self::ExternalLocation { value, .. } = self else {
            unreachable!("checked value is an external Location")
        };
        value
    }

    fn serialization_expected_pointer(&self) -> &str {
        let Self::SerializationStream { expected, .. } = self else {
            unreachable!("checked value is a SerializationStream")
        };
        expected
    }

    fn serialization_byte_count(&self) -> &str {
        let Self::SerializationStream { byte_count, .. } = self else {
            unreachable!("checked value is a SerializationStream")
        };
        byte_count
    }

    fn argument(&self) -> String {
        match self {
            Self::Unit => "i8 0".into(),
            Self::StaticDisplay(_) => {
                unreachable!("static Capability values are not call arguments")
            }
            Self::Completed(value) | Self::Effect(value) => format!("i8 {value}"),
            Self::Boolean(value) => format!("i1 {value}"),
            Self::Version { value, .. }
            | Self::SerializationStream { stream: value, .. }
            | Self::Task { value, .. }
            | Self::ExternalLocation { value, .. }
            | Self::Int(value)
            | Self::Modular { value, .. }
            | Self::Rational(value)
            | Self::Error(value)
            | Self::ErrorDomain(value)
            | Self::SourceLocation(value)
            | Self::String(value)
            | Self::Range { value, .. }
            | Self::Result { value, .. }
            | Self::Optional { value, .. }
            | Self::TraversalControl { value, .. }
            | Self::List { value, .. }
            | Self::Container { value, .. } => {
                format!("ptr {value}")
            }
            Self::Comparison(value)
            | Self::ErrorCode(value)
            | Self::Enum { value, .. }
            | Self::Function { value, .. }
            | Self::Generator { value, .. } => {
                format!("i32 {value}")
            }
            Self::Tuple(_) | Self::Record { .. } | Self::Sum { .. } => {
                unreachable!("checked call arguments are scalar")
            }
        }
    }
}

#[allow(clippy::too_many_lines)] // Every admitted representation has an explicit neutral machine value.
fn zero_machine_value(value_type: &CompilerType) -> LlValue {
    match value_type {
        CompilerType::Unit => LlValue::Unit,
        CompilerType::Completed => LlValue::Completed("0".into()),
        CompilerType::Effect => LlValue::Effect("0".into()),
        CompilerType::Boolean => LlValue::Boolean("false".into()),
        CompilerType::Version => {
            unreachable!("Version sum payloads are not admitted")
        }
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat => LlValue::Int("null".into()),
        CompilerType::Modular(modular) => LlValue::Modular {
            value: "null".into(),
            modular: modular.clone(),
        },
        CompilerType::Rational | CompilerType::InfiniteRational => LlValue::Rational("null".into()),
        CompilerType::Comparison => LlValue::Comparison("0".into()),
        CompilerType::Error => LlValue::Error("null".into()),
        CompilerType::ErrorCode => LlValue::ErrorCode("0".into()),
        CompilerType::ErrorDomain => LlValue::ErrorDomain("null".into()),
        CompilerType::SourceLocation => LlValue::SourceLocation("null".into()),
        CompilerType::Enum(enumeration) => LlValue::Enum {
            value: "0".into(),
            enumeration: enumeration.clone(),
        },
        CompilerType::Generator(generator) => LlValue::Generator {
            value: "0".into(),
            generator: generator.clone(),
            captured_initial: None,
            captured_additional_initials: Vec::new(),
        },
        CompilerType::Task(task) => LlValue::Task {
            value: "null".into(),
            task: task.as_ref().clone(),
        },
        CompilerType::ExternalLocation(location) => LlValue::ExternalLocation {
            value: "null".into(),
            location: location.as_ref().clone(),
        },
        CompilerType::Range(endpoint) => LlValue::Range {
            value: "null".into(),
            endpoint: endpoint.as_ref().clone(),
        },
        CompilerType::Result(success) | CompilerType::TaskResponse(success) => LlValue::Result {
            value: "null".into(),
            success: success.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::Optional(payload) => LlValue::Optional {
            value: "null".into(),
            payload: payload.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::TraversalControl(payload) => LlValue::TraversalControl {
            value: "null".into(),
            payload: payload.as_ref().clone(),
        },
        CompilerType::List(element) => LlValue::List {
            value: "null".into(),
            element: element.as_ref().clone(),
            function_captures: Vec::new(),
        },
        value_type @ (CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }) => LlValue::Container {
            value: "null".into(),
            value_type: value_type.clone(),
            function_captures: Vec::new(),
            function_capture_keys: Vec::new(),
        },
        CompilerType::Character | CompilerType::String => LlValue::String("null".into()),
        CompilerType::Refined { base, .. } => zero_machine_value(base),
        CompilerType::Tuple(fields) => {
            LlValue::Tuple(fields.iter().map(zero_machine_value).collect())
        }
        CompilerType::Record(fields) => LlValue::Record {
            fields: fields
                .iter()
                .map(|(label, field)| (label.clone(), zero_machine_value(field)))
                .collect(),
            order: (0..fields.len()).map(|index| index.to_string()).collect(),
        },
        CompilerType::Sum(sum) => LlValue::Sum {
            tag: "0".into(),
            payloads: sum
                .alternatives
                .iter()
                .map(|alternative| {
                    alternative
                        .payload
                        .as_ref()
                        .map(|payload| Box::new(zero_machine_value(payload)))
                })
                .collect(),
            sum: sum.clone(),
        },
        CompilerType::Type | CompilerType::Scope => LlValue::Enum {
            value: "0".into(),
            enumeration: if value_type == &CompilerType::Type {
                fundamental_type_enumeration()
            } else {
                scope_enumeration()
            },
        },
        CompilerType::Function => LlValue::Function {
            value: "0".into(),
            enumeration: CompilerEnumType {
                name: "Function".into(),
                alternatives: Vec::new(),
            },
            captures: Vec::new(),
        },
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata
        | CompilerType::SerializationStream(_)
        | CompilerType::Constraint => {
            unreachable!("static object values are not admitted in sum payloads")
        }
    }
}

fn optional_payload_pointer(value: &LlValue) -> &str {
    match value {
        LlValue::Int(value)
        | LlValue::Rational(value)
        | LlValue::Error(value)
        | LlValue::SourceLocation(value)
        | LlValue::List { value, .. }
        | LlValue::String(value) => value,
        _ => unreachable!("checked Optional payload has a pointer representation"),
    }
}

fn optional_payload_value(value: String, value_type: &CompilerType) -> LlValue {
    match value_type {
        CompilerType::Int => LlValue::Int(value),
        CompilerType::Rational => LlValue::Rational(value),
        CompilerType::Character | CompilerType::String => LlValue::String(value),
        CompilerType::Error => LlValue::Error(value),
        CompilerType::SourceLocation => LlValue::SourceLocation(value),
        _ => unreachable!("checked Optional payload type is supported"),
    }
}

fn numeric_pointer(value: &LlValue) -> (&str, CompilerType) {
    match value {
        LlValue::Int(value) => (value, CompilerType::Int),
        LlValue::Rational(value) => (value, CompilerType::Rational),
        _ => unreachable!("checked value is finite exact numeric"),
    }
}

fn fundamental_type_enumeration() -> CompilerEnumType {
    CompilerEnumType {
        name: "Type".into(),
        alternatives: [
            "Boolean", "Int", "Nat", "Rational", "String", "Unit", "Scope",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    }
}

fn scope_enumeration() -> CompilerEnumType {
    CompilerEnumType {
        name: "Scope".into(),
        alternatives: vec!["<namespace root>".into(), "<namespace lang lint>".into()],
    }
}

fn generator_debug_enumeration(generator: &CompilerGeneratorType) -> CompilerEnumType {
    let name = CompilerType::Generator(generator.clone()).name();
    CompilerEnumType {
        alternatives: vec![format!("<{name}>")],
        name,
    }
}

fn function_value_enumeration(program: &CompilerProgram) -> CompilerEnumType {
    CompilerEnumType {
        name: "Function".into(),
        alternatives: program.function_value_names.clone(),
    }
}

fn constraint_value_enumeration(program: &CompilerProgram) -> CompilerEnumType {
    CompilerEnumType {
        name: "Constraint".into(),
        alternatives: program
            .constraints
            .iter()
            .map(|constraint| format!("<Constraint {}>", constraint.name))
            .collect(),
    }
}

fn numeric_domain(value_type: &CompilerType) -> &'static str {
    match value_type {
        CompilerType::Int => "int",
        CompilerType::Rational => "rational",
        _ => unreachable!("checked Range endpoint is finite exact numeric"),
    }
}

struct FunctionBody {
    lines: Vec<String>,
    next_value: usize,
    next_label: usize,
    current_block: String,
    subprogram: usize,
}

impl FunctionBody {
    fn new(subprogram: usize) -> Self {
        Self {
            lines: vec!["entry:".into()],
            next_value: 0,
            next_label: 0,
            current_block: "entry".into(),
            subprogram,
        }
    }

    fn instruction(&mut self, instruction: &str, span: Span, debug: &mut DebugInfo) -> String {
        let value = self.reserve_value();
        self.define_reserved(&value, instruction, span, debug);
        value
    }

    fn instruction_without_debug(&mut self, instruction: &str) -> String {
        let value = self.reserve_value();
        self.lines.push(format!("  {value} = {instruction}"));
        value
    }

    fn reserve_value(&mut self) -> String {
        let value = format!("%v{}", self.next_value);
        self.next_value += 1;
        value
    }

    fn define_reserved(
        &mut self,
        value: &str,
        instruction: &str,
        span: Span,
        debug: &mut DebugInfo,
    ) {
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {value} = {instruction}, !dbg !{location}"));
    }

    fn effect(&mut self, instruction: &str, span: Span, debug: &mut DebugInfo) {
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {instruction}, !dbg !{location}"));
    }

    fn effect_without_debug(&mut self, instruction: &str) {
        self.lines.push(format!("  {instruction}"));
    }

    fn named_instruction(
        &mut self,
        name: &str,
        instruction: &str,
        span: Span,
        debug: &mut DebugInfo,
    ) {
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {name} = {instruction}, !dbg !{location}"));
    }

    fn terminator(&mut self, instruction: &str, location: usize) {
        self.lines
            .push(format!("  {instruction}, !dbg !{location}"));
    }

    fn debug_value(&mut self, value: &LlValue, variable: usize, location: usize) {
        let value = match value {
            LlValue::Unit => "i8 0".into(),
            LlValue::Completed(value) | LlValue::Effect(value) => format!("i8 {value}"),
            LlValue::Boolean(value) => format!("i1 {value}"),
            LlValue::Version { value, .. }
            | LlValue::SerializationStream { stream: value, .. }
            | LlValue::Task { value, .. }
            | LlValue::ExternalLocation { value, .. }
            | LlValue::Int(value)
            | LlValue::Modular { value, .. }
            | LlValue::Rational(value)
            | LlValue::Error(value)
            | LlValue::ErrorDomain(value)
            | LlValue::SourceLocation(value)
            | LlValue::String(value)
            | LlValue::Range { value, .. }
            | LlValue::Result { value, .. }
            | LlValue::Optional { value, .. }
            | LlValue::TraversalControl { value, .. }
            | LlValue::List { value, .. }
            | LlValue::Container { value, .. } => {
                format!("ptr {value}")
            }
            LlValue::Comparison(value)
            | LlValue::ErrorCode(value)
            | LlValue::Enum { value, .. }
            | LlValue::Function { value, .. }
            | LlValue::Generator { value, .. } => format!("i32 {value}"),
            LlValue::StaticDisplay(_)
            | LlValue::Tuple(_)
            | LlValue::Record { .. }
            | LlValue::Sum { .. } => return,
        };
        self.debug_value_operand(&value, variable, location);
    }

    fn debug_value_operand(&mut self, value: &str, variable: usize, location: usize) {
        self.lines.push(format!(
            "    #dbg_value({value}, !{variable}, !DIExpression(), !{location})"
        ));
    }

    fn debug_declare(&mut self, address: &str, variable: usize, location: usize) {
        self.lines.push(format!(
            "    #dbg_declare(ptr {address}, !{variable}, !DIExpression(), !{location})"
        ));
    }

    fn label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}.{}", self.next_label);
        self.next_label += 1;
        label
    }

    fn start_block(&mut self, label: &str) {
        self.lines.push(format!("{label}:"));
        self.current_block = label.into();
    }

    fn render(&self) -> String {
        self.lines.join("\n")
    }
}

struct DebugInfo {
    nodes: Vec<String>,
    file: usize,
    compile_unit: usize,
    empty: usize,
    unsigned64_type: usize,
    int_type: usize,
    nat_type: usize,
    version_type: usize,
    serialization_stream_type: usize,
    rational_type: usize,
    character_type: usize,
    string_type: usize,
    error_type: usize,
    generator_error_type: usize,
    error_code_type: usize,
    error_domain_type: usize,
    source_location_type: usize,
    int_range_type: usize,
    rational_range_type: usize,
    result_int_type: usize,
    result_nat_type: usize,
    result_rational_type: usize,
    result_string_type: usize,
    result_int_pair_type: usize,
    result_unit_generator_error_type: usize,
    result_header_pointer_type: usize,
    result_types: Vec<(CompilerType, usize)>,
    result_modular_types: Vec<(CompilerModularType, usize)>,
    optional_int_type: usize,
    optional_rational_type: usize,
    optional_character_type: usize,
    optional_string_type: usize,
    optional_error_type: usize,
    optional_source_location_type: usize,
    optional_header_pointer_type: usize,
    optional_types: Vec<(CompilerType, usize)>,
    traversal_control_types: Vec<(CompilerType, usize)>,
    comparison_type: usize,
    boolean_type: usize,
    unit_type: usize,
    completed_type: usize,
    effect_type: usize,
    enum_types: BTreeMap<String, usize>,
    modular_types: Vec<(CompilerModularType, usize)>,
    list_types: Vec<(CompilerType, usize)>,
    container_types: Vec<(CompilerType, usize)>,
    refined_types: Vec<(CompilerType, usize)>,
    tuple_types: Vec<(CompilerType, usize)>,
    record_types: Vec<(CompilerType, usize)>,
    sum_types: Vec<(CompilerType, usize)>,
    task_types: Vec<(CompilerTaskType, usize)>,
    location_types: Vec<(CompilerLocationType, usize)>,
    source: topal_source::SourceText,
    filename: String,
}

impl DebugInfo {
    #[allow(clippy::too_many_lines)] // Initialization keeps the complete emitted DWARF type graph visible.
    fn new(source_name: &str, extended_types: bool, generator_close_types: bool) -> Self {
        let path = Path::new(source_name);
        let filename = path.file_name().map_or_else(
            || source_name.into(),
            |name| name.to_string_lossy().into_owned(),
        );
        let directory = path
            .parent()
            .map_or_else(String::new, |parent| parent.to_string_lossy().into_owned());
        let source = topal_source::SourceText::new("").expect("empty source is valid");
        let mut debug = Self {
            nodes: Vec::new(),
            file: 0,
            compile_unit: 0,
            empty: 0,
            unsigned64_type: 0,
            int_type: 0,
            nat_type: 0,
            version_type: 0,
            serialization_stream_type: 0,
            rational_type: 0,
            character_type: 0,
            string_type: 0,
            error_type: 0,
            generator_error_type: 0,
            error_code_type: 0,
            error_domain_type: 0,
            source_location_type: 0,
            int_range_type: 0,
            rational_range_type: 0,
            result_int_type: 0,
            result_nat_type: 0,
            result_rational_type: 0,
            result_string_type: 0,
            result_int_pair_type: 0,
            result_unit_generator_error_type: 0,
            result_header_pointer_type: 0,
            result_types: Vec::new(),
            result_modular_types: Vec::new(),
            optional_int_type: 0,
            optional_rational_type: 0,
            optional_character_type: 0,
            optional_string_type: 0,
            optional_error_type: 0,
            optional_source_location_type: 0,
            optional_header_pointer_type: 0,
            optional_types: Vec::new(),
            traversal_control_types: Vec::new(),
            comparison_type: 0,
            boolean_type: 0,
            unit_type: 0,
            completed_type: 0,
            effect_type: 0,
            enum_types: BTreeMap::new(),
            modular_types: Vec::new(),
            list_types: Vec::new(),
            container_types: Vec::new(),
            refined_types: Vec::new(),
            tuple_types: Vec::new(),
            record_types: Vec::new(),
            sum_types: Vec::new(),
            task_types: Vec::new(),
            location_types: Vec::new(),
            source,
            filename,
        };
        debug.file = debug.node(format!(
            "!DIFile(filename: \"{}\", directory: \"{}\")",
            llvm_string(&debug.filename),
            llvm_string(&directory)
        ));
        debug.empty = debug.node("!{}".into());
        debug.compile_unit = debug.node(format!(
            "distinct !DICompileUnit(language: DW_LANG_C11, file: !{}, producer: \"topalc {}\", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, enums: !{}, retainedTypes: !{})",
            debug.file,
            env!("CARGO_PKG_VERSION"),
            debug.empty,
            debug.empty
        ));
        let unsigned64 = debug.install_base_integer_types();
        debug.version_type = debug.install_version_type();
        debug.serialization_stream_type = debug.install_serialization_stream_type(unsigned64);
        let numerator = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"numerator\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            debug.file, debug.int_type
        ));
        let denominator = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"denominator\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            debug.file, debug.int_type
        ));
        let rational_members = debug.node(format!("!{{!{numerator}, !{denominator}}}"));
        let rational_storage = debug.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalRationalHeader\", file: !{}, size: 128, align: 64, elements: !{rational_members})",
            debug.file
        ));
        let rational_pointer = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{rational_storage}, size: 64, align: 64)"
        ));
        debug.rational_type = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Rational\", file: !{}, baseType: !{rational_pointer})",
            debug.file
        ));
        if extended_types {
            debug.install_string_and_error_types(unsigned64, generator_close_types);
        }
        debug.install_aggregate_types(unsigned64, extended_types, generator_close_types);
        let less = debug.node("!DIEnumerator(name: \"Less\", value: -1)".into());
        let equal = debug.node("!DIEnumerator(name: \"Equal\", value: 0)".into());
        let greater = debug.node("!DIEnumerator(name: \"Greater\", value: 1)".into());
        let comparison_values = debug.node(format!("!{{!{less}, !{equal}, !{greater}}}"));
        debug.comparison_type = debug.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"Comparison\", file: !{}, size: 32, align: 32, elements: !{comparison_values})",
            debug.file
        ));
        debug.install_zero_data_types();
        debug
    }

    fn install_base_integer_types(&mut self) -> usize {
        let unsigned64 =
            self.node("!DIBasicType(name: \"u64\", size: 64, encoding: DW_ATE_unsigned)".into());
        self.unsigned64_type = unsigned64;
        self.install_integer_types(unsigned64);
        unsigned64
    }

    fn install_zero_data_types(&mut self) {
        self.boolean_type =
            self.node("!DIBasicType(name: \"Boolean\", size: 8, encoding: DW_ATE_boolean)".into());
        self.unit_type =
            self.node("!DIBasicType(name: \"Unit\", size: 8, encoding: DW_ATE_unsigned)".into());
        let completed = self.node("!DIEnumerator(name: \"Completed\", value: 0)".into());
        let completed_values = self.node(format!("!{{!{completed}}}"));
        self.completed_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"Completed\", file: !{}, size: 8, align: 8, elements: !{completed_values})",
            self.file
        ));
        let empty_effect = self.node("!DIEnumerator(name: \"empty\", value: 0)".into());
        let effect_values = self.node(format!("!{{!{empty_effect}}}"));
        self.effect_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"Effect\", file: !{}, size: 8, align: 8, elements: !{effect_values})",
            self.file
        ));
    }

    fn install_aggregate_types(
        &mut self,
        unsigned64: usize,
        extended_types: bool,
        generator_close_types: bool,
    ) {
        self.int_range_type = self.range_type("Range Int", self.int_type, unsigned64);
        self.rational_range_type =
            self.range_type("Range Rational", self.rational_type, unsigned64);
        self.install_result_types(unsigned64, generator_close_types);
        if extended_types {
            self.install_optional_types(unsigned64);
        }
    }

    fn install_integer_types(&mut self, unsigned64: usize) {
        let negative = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"negative\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"length\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{negative}, !{length}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalIntHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.int_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Int\", file: !{}, baseType: !{pointer})",
            self.file
        ));
        self.nat_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Nat\", file: !{}, baseType: !{pointer})",
            self.file
        ));
    }

    fn install_version_type(&mut self) -> usize {
        let members = ["major", "minor", "patch", "build"]
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                self.node(format!(
                    "!DIDerivedType(tag: DW_TAG_member, name: \"{name}\", file: !{}, baseType: !{}, size: 64, align: 64, offset: {})",
                    self.file,
                    self.nat_type,
                    index * 64
                ))
            })
            .collect::<Vec<_>>();
        let members = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalVersionHeader\", file: !{}, size: 256, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Version\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_serialization_stream_type(&mut self, unsigned64: usize) -> usize {
        let byte =
            self.node("!DIBasicType(name: \"u8\", size: 8, encoding: DW_ATE_unsigned_char)".into());
        let byte_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{byte}, size: 64, align: 64)"
        ));
        let data = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"data\", file: !{}, baseType: !{byte_pointer}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let byte_count = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"byte_count\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{data}, !{byte_count}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalSerializationStreamHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"SerializationStream\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_string_and_error_types(&mut self, unsigned64: usize, generator_close_types: bool) {
        let byte =
            self.node("!DIBasicType(name: \"u8\", size: 8, encoding: DW_ATE_unsigned_char)".into());
        let byte_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{byte}, size: 64, align: 64)"
        ));
        let data = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"data\", file: !{}, baseType: !{byte_pointer}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"length\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let display = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"display\", file: !{}, baseType: !{byte_pointer}, size: 64, align: 64, offset: 128)",
            self.file
        ));
        let display_length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"display_length\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 192)",
            self.file
        ));
        let members = self.node(format!(
            "!{{!{data}, !{length}, !{display}, !{display_length}}}"
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalStringHeader\", file: !{}, size: 256, align: 64, elements: !{members})",
            self.file
        ));
        let string_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.string_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"String\", file: !{}, baseType: !{string_pointer})",
            self.file
        ));
        self.character_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Character\", file: !{}, baseType: !{string_pointer})",
            self.file
        ));
        self.error_domain_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"ErrorDomain\", file: !{}, baseType: !{string_pointer})",
            self.file
        ));

        self.install_source_location_type();

        let enumerators = [
            ("out-of-range", 0),
            ("not-representable", 1),
            ("division-by-zero", 2),
            ("indeterminate", 3),
        ]
        .into_iter()
        .map(|(name, value)| self.node(format!("!DIEnumerator(name: \"{name}\", value: {value})")))
        .collect::<Vec<_>>();
        let error_code_values = self.node(format!(
            "!{{{}}}",
            enumerators
                .iter()
                .map(|value| format!("!{value}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        self.error_code_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"lang arithmetic ArithmeticErrorCode\", file: !{}, size: 32, align: 32, elements: !{error_code_values})",
            self.file
        ));
        let domain = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"domain\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.error_domain_type
        ));
        let code = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"code\", file: !{}, baseType: !{}, size: 32, align: 32, offset: 64)",
            self.file, self.error_code_type
        ));
        let error_members = self.node(format!("!{{!{domain}, !{code}}}"));
        let error_storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalErrorHeader\", file: !{}, size: 448, align: 64, elements: !{error_members})",
            self.file
        ));
        let error_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{error_storage}, size: 64, align: 64)"
        ));
        self.error_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Error\", file: !{}, baseType: !{error_pointer})",
            self.file
        ));

        if generator_close_types {
            self.install_generator_error_type();
        }
    }

    fn install_generator_error_type(&mut self) {
        let generator_error_code = self.enum_type(&CompilerEnumType {
            name: "lang generator GeneratorErrorCode".into(),
            alternatives: vec!["generator-closed".into()],
        });
        let generator_domain = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"domain\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.error_domain_type
        ));
        let generator_code = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"code\", file: !{}, baseType: !{generator_error_code}, size: 32, align: 32, offset: 64)",
            self.file
        ));
        let generator_members = self.node(format!("!{{!{generator_domain}, !{generator_code}}}"));
        let generator_storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalGeneratorErrorHeader\", file: !{}, size: 448, align: 64, elements: !{generator_members})",
            self.file
        ));
        let generator_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{generator_storage}, size: 64, align: 64)"
        ));
        self.generator_error_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Error (lang generator GeneratorErrorCode)\", file: !{}, baseType: !{generator_pointer})",
            self.file
        ));
    }

    fn install_source_location_type(&mut self) {
        let line = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"line\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.int_type
        ));
        let column = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"column\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.int_type
        ));
        let members = self.node(format!("!{{!{line}, !{column}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalSourceLocationHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.source_location_type = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"SourceLocation\", file: !{}, baseType: !{pointer})",
            self.file
        ));
    }

    fn range_type(&mut self, name: &str, endpoint_type: usize, flag_type: usize) -> usize {
        let lower = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"lower\", file: !{}, baseType: !{endpoint_type}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let upper = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"upper\", file: !{}, baseType: !{endpoint_type}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let lower_inclusive = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"lower_inclusive\", file: !{}, baseType: !{flag_type}, size: 64, align: 64, offset: 128)",
            self.file
        ));
        let upper_inclusive = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"upper_inclusive\", file: !{}, baseType: !{flag_type}, size: 64, align: 64, offset: 192)",
            self.file
        ));
        let members = self.node(format!(
            "!{{!{lower}, !{upper}, !{lower_inclusive}, !{upper_inclusive}}}"
        ));
        let storage_name = name.replace(' ', "");
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"Topal{storage_name}Header\", file: !{}, size: 256, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{name}\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_result_types(&mut self, unsigned64: usize, generator_close_types: bool) {
        let result_tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_error\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{unsigned64}, size: 64, align: 64)"
        ));
        let result_payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let result_members = self.node(format!("!{{!{result_tag}, !{result_payload}}}"));
        let result_storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalResultHeader\", file: !{}, size: 128, align: 64, elements: !{result_members})",
            self.file
        ));
        let result_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{result_storage}, size: 64, align: 64)"
        ));
        self.result_header_pointer_type = result_pointer;
        self.result_int_type = self.result_type("Int", result_pointer);
        self.result_nat_type = self.result_type("Nat", result_pointer);
        self.result_rational_type = self.result_type("Rational", result_pointer);
        self.result_string_type = self.result_type("String", result_pointer);
        self.result_int_pair_type = self.result_type("(Int, Int)", result_pointer);
        if generator_close_types {
            self.result_unit_generator_error_type = self.node(format!(
                "!DIDerivedType(tag: DW_TAG_typedef, name: \"Result (Unit, lang generator GeneratorErrorCode)\", file: !{}, baseType: !{result_pointer})",
                self.file
            ));
        }
    }

    fn result_type(&mut self, success: &str, pointer: usize) -> usize {
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Result ({success}, lang arithmetic ArithmeticErrorCode)\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn install_optional_types(&mut self, unsigned64: usize) {
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"tag\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 0)",
            self.file
        ));
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{unsigned64}, size: 64, align: 64)"
        ));
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalOptionalHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        self.optional_header_pointer_type = pointer;
        self.optional_int_type = self.optional_type("Int", pointer);
        self.optional_rational_type = self.optional_type("Rational", pointer);
        self.optional_character_type = self.optional_type("Character", pointer);
        self.optional_string_type = self.optional_type("String", pointer);
        self.optional_error_type = self.optional_type("Error", pointer);
        self.optional_source_location_type = self.optional_type("SourceLocation", pointer);
    }

    fn optional_type(&mut self, payload: &str, pointer: usize) -> usize {
        self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Optional {payload}\", file: !{}, baseType: !{pointer})",
            self.file
        ))
    }

    fn set_source(&mut self, source: topal_source::SourceText) {
        self.source = source;
    }

    fn filename(&self) -> &str {
        &self.filename
    }

    fn node(&mut self, value: String) -> usize {
        let id = self.nodes.len();
        self.nodes.push(value);
        id
    }

    #[allow(clippy::too_many_lines)] // Keep the exhaustive semantic-type to DWARF mapping visible.
    fn type_id(&mut self, value_type: &CompilerType) -> usize {
        match value_type {
            CompilerType::Unit => self.unit_type,
            CompilerType::Completed => self.completed_type,
            CompilerType::Effect => self.effect_type,
            CompilerType::Type => self.enum_type(&fundamental_type_enumeration()),
            CompilerType::Scope => self.enum_type(&scope_enumeration()),
            CompilerType::Function => *self
                .enum_types
                .get("Function")
                .expect("checked Function values install their debug type"),
            CompilerType::Identity
            | CompilerType::TypeView
            | CompilerType::FunctionView
            | CompilerType::LanguageContext
            | CompilerType::Capability
            | CompilerType::NativeSerializer(_)
            | CompilerType::ExternalMetadata => {
                unreachable!("static-only compiler values have no runtime debug type")
            }
            CompilerType::SerializationStream(_) => self.serialization_stream_type,
            CompilerType::Constraint => *self
                .enum_types
                .get("Constraint")
                .expect("checked Constraint values install their debug type"),
            CompilerType::Boolean => self.boolean_type,
            CompilerType::Version => self.version_type,
            CompilerType::Int | CompilerType::InfiniteInt => self.int_type,
            CompilerType::Nat | CompilerType::InfiniteNat => self.nat_type,
            CompilerType::Rational | CompilerType::InfiniteRational => self.rational_type,
            CompilerType::Comparison => self.comparison_type,
            CompilerType::Error => self.error_type,
            CompilerType::ErrorCode => self.error_code_type,
            CompilerType::ErrorDomain => self.error_domain_type,
            CompilerType::SourceLocation => self.source_location_type,
            CompilerType::Modular(modular) => self.modular_type(modular),
            CompilerType::Enum(enumeration) => self.enum_type(enumeration),
            CompilerType::Generator(generator) => self.generator_type(generator),
            CompilerType::Task(task) => self.task_type(task),
            CompilerType::ExternalLocation(location) => self.location_type(location),
            CompilerType::Character => self.character_type,
            CompilerType::String => self.string_type,
            CompilerType::Range(endpoint)
                if matches!(
                    endpoint.as_ref(),
                    CompilerType::Int | CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ) =>
            {
                self.int_range_type
            }
            CompilerType::Range(endpoint)
                if matches!(
                    endpoint.as_ref(),
                    CompilerType::Rational | CompilerType::InfiniteRational
                ) =>
            {
                self.rational_range_type
            }
            CompilerType::Range(_) => unreachable!("unsupported Range endpoint reached codegen"),
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Int => {
                self.result_int_type
            }
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Nat => {
                self.result_nat_type
            }
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Rational => {
                self.result_rational_type
            }
            CompilerType::Result(success) if success.as_ref() == &CompilerType::String => {
                self.result_string_type
            }
            CompilerType::Result(success)
                if success.as_ref()
                    == &CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]) =>
            {
                self.result_int_pair_type
            }
            CompilerType::Result(success)
                if matches!(success.as_ref(), CompilerType::Modular(_)) =>
            {
                let CompilerType::Modular(modular) = success.as_ref() else {
                    unreachable!()
                };
                self.result_modular_type(modular)
            }
            CompilerType::Result(success) => self.dynamic_result_type(success),
            CompilerType::TaskResponse(success) => self.task_response_type(success),
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Int => {
                self.optional_int_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Rational => {
                self.optional_rational_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Character => {
                self.optional_character_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::String => {
                self.optional_string_type
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Error => {
                self.optional_error_type
            }
            CompilerType::Optional(payload)
                if payload.as_ref() == &CompilerType::SourceLocation =>
            {
                self.optional_source_location_type
            }
            CompilerType::Optional(payload) => self.dynamic_optional_type(payload),
            CompilerType::TraversalControl(payload) => self.traversal_control_type(payload),
            CompilerType::List(element) => self.list_type(element),
            CompilerType::Array { .. }
            | CompilerType::Set(_)
            | CompilerType::Bag(_)
            | CompilerType::Map { .. } => self.container_type(value_type),
            CompilerType::Refined { constraint, base } => self.refined_type(constraint, base),
            CompilerType::Tuple(fields) => self.tuple_type(fields),
            CompilerType::Record(fields) => self.record_type(fields),
            CompilerType::Sum(sum) => self.sum_type(sum),
        }
    }

    fn task_type(&mut self, task: &CompilerTaskType) -> usize {
        if let Some((_, type_id)) = self.task_types.iter().find(|(known, _)| known == task) {
            return *type_id;
        }
        let identity = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"identity\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let terminated = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"terminated\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.unsigned64_type
        ));
        let state_type = self.type_id(&task.state_type);
        let state = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"{}\", file: !{}, baseType: !{state_type}, size: 64, align: 64, offset: 128)",
            llvm_string(&task.state_name), self.file
        ));
        let members = self.node(format!("!{{!{identity}, !{terminated}, !{state}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalTask.{}\", file: !{}, size: 192, align: 64, elements: !{members})",
            llvm_string(&task.classifier), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&task.classifier),
            self.file
        ));
        self.task_types.push((task.clone(), type_id));
        type_id
    }

    fn location_type(&mut self, location: &CompilerLocationType) -> usize {
        if let Some((_, type_id)) = self
            .location_types
            .iter()
            .find(|(known, _)| known == location)
        {
            return *type_id;
        }
        let range_start = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"range-start\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.nat_type
        ));
        let offset = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"offset\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.nat_type
        ));
        let initialized = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"initialized\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 128)",
            self.file, self.unsigned64_type
        ));
        let layout_value = CompilerType::Refined {
            constraint: location.layout.identity.clone(),
            base: Box::new(CompilerType::Nat),
        };
        let layout_type = self.type_id(&layout_value);
        let value = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"value\", file: !{}, baseType: !{layout_type}, size: 64, align: 64, offset: 192)",
            self.file
        ));
        let members = self.node(format!(
            "!{{!{range_start}, !{offset}, !{initialized}, !{value}}}"
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalLocation.{}\", file: !{}, size: 256, align: 64, elements: !{members})",
            llvm_string(&location.identity), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&location.identity),
            self.file
        ));
        self.location_types.push((location.clone(), type_id));
        type_id
    }

    fn modular_type(&mut self, modular: &CompilerModularType) -> usize {
        if let Some((_, type_id)) = self
            .modular_types
            .iter()
            .find(|(known, _)| known == modular)
        {
            return *type_id;
        }
        let negative = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"negative\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let length = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"length\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
            self.file, self.unsigned64_type
        ));
        let members = self.node(format!("!{{!{negative}, !{length}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalModular.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&modular.name), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&modular.name),
            self.file
        ));
        self.modular_types.push((modular.clone(), type_id));
        type_id
    }

    fn dynamic_optional_type(&mut self, payload: &CompilerType) -> usize {
        let value_type = CompilerType::Optional(Box::new(payload.clone()));
        if let Some((_, type_id)) = self
            .optional_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        if !matches!(payload, CompilerType::Enum(_)) {
            let type_id = self.optional_type(&payload.name(), self.optional_header_pointer_type);
            self.optional_types.push((value_type, type_id));
            return type_id;
        }
        let payload_type = self.type_id(payload);
        let payload_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{payload_type}, size: 64, align: 64)"
        ));
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"tag\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload_member = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{payload_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload_member}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalOptional.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&payload.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.optional_type(&payload.name(), pointer);
        self.optional_types.push((value_type, type_id));
        type_id
    }

    fn dynamic_result_type(&mut self, success: &CompilerType) -> usize {
        let value_type = CompilerType::Result(Box::new(success.clone()));
        if let Some((_, type_id)) = self
            .result_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        if !matches!(success, CompilerType::Enum(_)) {
            let type_id = self.result_type(&success.name(), self.result_header_pointer_type);
            self.result_types.push((value_type, type_id));
            return type_id;
        }
        let success_type = self.type_id(success);
        let success_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{success_type}, size: 64, align: 64)"
        ));
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_error\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{success_pointer}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalResult.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&success.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.result_type(&success.name(), pointer);
        self.result_types.push((value_type, type_id));
        type_id
    }

    fn task_response_type(&mut self, success: &CompilerType) -> usize {
        let value_type = CompilerType::TaskResponse(Box::new(success.clone()));
        if let Some((_, type_id)) = self
            .result_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Result ({}, ())\", file: !{}, baseType: !{})",
            llvm_string(&success.name()), self.file, self.result_header_pointer_type
        ));
        self.result_types.push((value_type, type_id));
        type_id
    }

    fn traversal_control_type(&mut self, payload: &CompilerType) -> usize {
        let value_type = CompilerType::TraversalControl(Box::new(payload.clone()));
        if let Some((_, type_id)) = self
            .traversal_control_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let payload_type = self.type_id(payload);
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_finish\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload_member = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"value\", file: !{}, baseType: !{payload_type}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload_member}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalTraversalControl.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&payload.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"TraversalControl {}\", file: !{}, baseType: !{pointer})",
            llvm_string(&payload.name()), self.file
        ));
        self.traversal_control_types.push((value_type, type_id));
        type_id
    }

    fn list_type(&mut self, element: &CompilerType) -> usize {
        let value_type = CompilerType::List(Box::new(element.clone()));
        if let Some((_, type_id)) = self
            .list_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let element_type = self.type_id(element);
        let element_layout = target_value_layout(element);
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"value\", file: !{}, baseType: !{element_type}, size: {}, align: {}, offset: 0)",
            self.file, element_layout.size, element_layout.alignment
        ));
        let next_offset = align_bits(element_layout.size, 64);
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{}, size: 64, align: 64)",
            self.unsigned64_type
        ));
        let next = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"remaining\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: {next_offset})",
            self.file,
        ));
        let members = self.node(format!("!{{!{payload}, !{next}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalList.{}\", file: !{}, size: {}, align: 64, elements: !{members})",
            llvm_string(&element.name()), self.file, next_offset + 64
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"List {}\", file: !{}, baseType: !{pointer})",
            llvm_string(&element.name()),
            self.file
        ));
        self.list_types.push((value_type, type_id));
        type_id
    }

    fn container_type(&mut self, value_type: &CompilerType) -> usize {
        if let Some((_, type_id)) = self
            .container_types
            .iter()
            .find(|(known, _)| known == value_type)
        {
            return *type_id;
        }
        let entry_count = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"entry_count\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let opaque_pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{}, size: 64, align: 64)",
            self.unsigned64_type
        ));
        let mut members = vec![entry_count];
        let (entries_offset, storage_size) = if matches!(value_type, CompilerType::Bag(_)) {
            let distinct_count = self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"distinct_count\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 64)",
                self.file, self.unsigned64_type
            ));
            members.push(distinct_count);
            (128, 192)
        } else {
            (64, 128)
        };
        let entries = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"entries\", file: !{}, baseType: !{opaque_pointer}, size: 64, align: 64, offset: {entries_offset})",
            self.file
        ));
        members.push(entries);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalContainer.{}\", file: !{}, size: {storage_size}, align: 64, elements: !{elements})",
            llvm_string(&value_type.name()), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{pointer})",
            llvm_string(&value_type.name()),
            self.file
        ));
        self.container_types.push((value_type.clone(), type_id));
        type_id
    }

    fn result_modular_type(&mut self, modular: &CompilerModularType) -> usize {
        if let Some((_, type_id)) = self
            .result_modular_types
            .iter()
            .find(|(known, _)| known == modular)
        {
            return *type_id;
        }
        let modular_type = self.modular_type(modular);
        let tag = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"is_error\", file: !{}, baseType: !{}, size: 64, align: 64, offset: 0)",
            self.file, self.unsigned64_type
        ));
        let payload = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"payload\", file: !{}, baseType: !{modular_type}, size: 64, align: 64, offset: 64)",
            self.file
        ));
        let members = self.node(format!("!{{!{tag}, !{payload}}}"));
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalResult.Modular.{}\", file: !{}, size: 128, align: 64, elements: !{members})",
            llvm_string(&modular.name), self.file
        ));
        let pointer = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        let type_id = self.result_type(&modular.name, pointer);
        self.result_modular_types.push((modular.clone(), type_id));
        type_id
    }

    fn refined_type(&mut self, constraint: &str, base: &CompilerType) -> usize {
        let value_type = CompilerType::Refined {
            constraint: constraint.to_owned(),
            base: Box::new(base.clone()),
        };
        if let Some((_, type_id)) = self
            .refined_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let base_type = self.type_id(base);
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{base_type})",
            llvm_string(constraint),
            self.file
        ));
        self.refined_types.push((value_type, type_id));
        type_id
    }

    fn tuple_type(&mut self, fields: &[CompilerType]) -> usize {
        let value_type = CompilerType::Tuple(fields.to_vec());
        if let Some((_, type_id)) = self
            .tuple_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }

        let mut offset = 0;
        let mut aggregate_alignment = 8;
        let mut members = Vec::with_capacity(fields.len());
        for (index, field) in fields.iter().enumerate() {
            let layout = target_value_layout(field);
            offset = align_bits(offset, layout.alignment);
            aggregate_alignment = aggregate_alignment.max(layout.alignment);
            let field_type = self.type_id(field);
            members.push(self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"_{index}\", file: !{}, baseType: !{field_type}, size: {}, align: {}, offset: {offset})",
                self.file, layout.size, layout.alignment
            )));
            offset += layout.size;
        }
        let size = align_bits(offset, aggregate_alignment);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let type_id = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"{}\", file: !{}, size: {size}, align: {aggregate_alignment}, elements: !{elements})",
            llvm_string(&value_type.name()),
            self.file
        ));
        self.tuple_types.push((value_type, type_id));
        type_id
    }

    fn record_type(&mut self, fields: &[(String, CompilerType)]) -> usize {
        let value_type = CompilerType::Record(fields.to_vec());
        if let Some((_, type_id)) = self
            .record_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }

        let mut offset = 0;
        let mut aggregate_alignment = 8;
        let mut members = Vec::with_capacity(fields.len());
        for (label, field) in fields {
            let layout = target_value_layout(field);
            offset = align_bits(offset, layout.alignment);
            aggregate_alignment = aggregate_alignment.max(layout.alignment);
            let field_type = self.type_id(field);
            members.push(self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"{}\", file: !{}, baseType: !{field_type}, size: {}, align: {}, offset: {offset})",
                llvm_string(label), self.file, layout.size, layout.alignment
            )));
            offset += layout.size;
        }
        for _ in fields {
            offset = align_bits(offset, 32) + 32;
            aggregate_alignment = aggregate_alignment.max(32);
        }
        let size = align_bits(offset, aggregate_alignment);
        debug_assert_eq!(size, target_value_layout(&value_type).size);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let type_id = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"{}\", file: !{}, size: {size}, align: {aggregate_alignment}, elements: !{elements})",
            llvm_string(&value_type.name()),
            self.file
        ));
        self.record_types.push((value_type, type_id));
        type_id
    }

    fn sum_type(&mut self, sum: &CompilerSumType) -> usize {
        let value_type = CompilerType::Sum(sum.clone());
        if let Some((_, type_id)) = self
            .sum_types
            .iter()
            .find(|(known, _)| known == &value_type)
        {
            return *type_id;
        }
        let enumerators = sum
            .alternatives
            .iter()
            .enumerate()
            .map(|(value, alternative)| {
                self.node(format!(
                    "!DIEnumerator(name: \"{}\", value: {value})",
                    llvm_string(&alternative.name)
                ))
            })
            .collect::<Vec<_>>();
        let tag_values = self.node(format!(
            "!{{{}}}",
            enumerators
                .iter()
                .map(|value| format!("!{value}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let tag_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"{}.alternative\", file: !{}, size: 32, align: 32, elements: !{tag_values})",
            llvm_string(&sum.name),
            self.file
        ));
        let mut members = vec![self.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"tag\", file: !{}, baseType: !{tag_type}, size: 32, align: 32, offset: 0)",
            self.file
        ))];
        let mut offset = 32;
        let mut alignment = 32;
        for (index, alternative) in sum.alternatives.iter().enumerate() {
            let Some(payload) = &alternative.payload else {
                continue;
            };
            let layout = target_value_layout(payload);
            offset = align_bits(offset, layout.alignment);
            alignment = alignment.max(layout.alignment);
            let payload_type = self.type_id(payload);
            members.push(self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"payload_{index}\", file: !{}, baseType: !{payload_type}, size: {}, align: {}, offset: {offset})",
                self.file, layout.size, layout.alignment
            )));
            offset += layout.size;
        }
        let size = align_bits(offset, alignment);
        debug_assert_eq!(size, target_value_layout(&value_type).size);
        let elements = self.node(format!(
            "!{{{}}}",
            members
                .iter()
                .map(|member| format!("!{member}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let kind = if sum.positional { "Variant" } else { "Union" };
        let storage = self.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"Topal{kind}.{}\", file: !{}, size: {size}, align: {alignment}, elements: !{elements})",
            llvm_string(&sum.name),
            self.file
        ));
        let type_id = self.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"{}\", file: !{}, baseType: !{storage})",
            llvm_string(&sum.name),
            self.file
        ));
        self.sum_types.push((value_type, type_id));
        type_id
    }

    fn enum_type(&mut self, enumeration: &CompilerEnumType) -> usize {
        if let Some(value_type) = self.enum_types.get(&enumeration.name) {
            return *value_type;
        }
        let enumerators = enumeration
            .alternatives
            .iter()
            .enumerate()
            .map(|(value, name)| {
                self.node(format!(
                    "!DIEnumerator(name: \"{}\", value: {value})",
                    llvm_string(name)
                ))
            })
            .collect::<Vec<_>>();
        let elements = self.node(format!(
            "!{{{}}}",
            enumerators
                .iter()
                .map(|value| format!("!{value}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let value_type = self.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"{}\", file: !{}, size: 32, align: 32, elements: !{elements})",
            llvm_string(&enumeration.name),
            self.file
        ));
        self.enum_types.insert(enumeration.name.clone(), value_type);
        value_type
    }

    fn generator_type(&mut self, generator: &CompilerGeneratorType) -> usize {
        self.enum_type(&generator_debug_enumeration(generator))
    }

    fn subprogram(
        &mut self,
        name: &str,
        linkage_name: &str,
        span: Span,
        result: &CompilerType,
        parameters: &[CompilerType],
    ) -> usize {
        let result_type = if *result == CompilerType::Unit {
            "null".into()
        } else {
            format!("!{}", self.type_id(result))
        };
        let mut types = vec![result_type];
        for parameter in parameters {
            let parameter_type = self.type_id(parameter);
            types.push(format!("!{parameter_type}"));
        }
        let types = self.node(format!("!{{{}}}", types.join(", ")));
        let signature = self.node(format!("!DISubroutineType(types: !{types})"));
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "distinct !DISubprogram(name: \"{}\", linkageName: \"{}\", scope: !{}, file: !{}, line: {}, type: !{}, scopeLine: {}, spFlags: DISPFlagDefinition, unit: !{})",
            llvm_string(name),
            llvm_string(linkage_name),
            self.file,
            self.file,
            position.line,
            signature,
            position.line,
            self.compile_unit
        ))
    }

    fn lexical_block(&mut self, span: Span, scope: usize) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "distinct !DILexicalBlock(scope: !{scope}, file: !{}, line: {}, column: {})",
            self.file, position.line, position.column
        ))
    }

    fn parameter(
        &mut self,
        name: &str,
        argument: usize,
        span: Span,
        value_type: &CompilerType,
        scope: usize,
    ) -> usize {
        self.variable(name, Some(argument), span, value_type, scope)
    }

    fn local(&mut self, name: &str, span: Span, value_type: &CompilerType, scope: usize) -> usize {
        self.variable(name, None, span, value_type, scope)
    }

    fn local_with_type_id(
        &mut self,
        name: &str,
        span: Span,
        value_type: usize,
        scope: usize,
    ) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "!DILocalVariable(name: \"{}\", scope: !{scope}, file: !{}, line: {}, type: !{value_type})",
            llvm_string(name),
            self.file,
            position.line
        ))
    }

    fn variable(
        &mut self,
        name: &str,
        argument: Option<usize>,
        span: Span,
        value_type: &CompilerType,
        scope: usize,
    ) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        let argument = argument.map_or_else(String::new, |value| format!(", arg: {value}"));
        let value_type = self.type_id(value_type);
        self.node(format!(
            "!DILocalVariable(name: \"{}\"{argument}, scope: !{scope}, file: !{}, line: {}, type: !{})",
            llvm_string(name),
            self.file,
            position.line,
            value_type
        ))
    }

    fn location(&mut self, span: Span, scope: usize) -> usize {
        let position = self
            .source
            .position(span.start.min(self.source.as_str().len()));
        self.node(format!(
            "!DILocation(line: {}, column: {}, scope: !{scope})",
            position.line, position.column
        ))
    }

    fn finish(&mut self) -> String {
        let dwarf = self.node("!{i32 2, !\"Dwarf Version\", i32 5}".into());
        let debug_version = self.node("!{i32 2, !\"Debug Info Version\", i32 3}".into());
        let ident = self.node(format!("!{{!\"topalc {}\"}}", env!("CARGO_PKG_VERSION")));
        let mut output = format!(
            "!llvm.dbg.cu = !{{!{}}}\n!llvm.module.flags = !{{!{dwarf}, !{debug_version}}}\n!llvm.ident = !{{!{ident}}}\n",
            self.compile_unit
        );
        for (id, node) in self.nodes.iter().enumerate() {
            let _ = writeln!(output, "!{id} = {node}");
        }
        output
    }
}

fn llvm_type(value_type: &CompilerType) -> String {
    match value_type {
        CompilerType::Unit => "void".into(),
        _ => llvm_value_type(value_type),
    }
}

fn function_parameter_is_context_capture(parameter: &CompilerParameter) -> bool {
    parameter.name.starts_with("root ") || parameter.name.starts_with("@ ")
}

fn compiler_type_contains_function(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Function => true,
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_function),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, field)| compiler_type_contains_function(field)),
        CompilerType::Optional(payload) => compiler_type_contains_function(payload),
        CompilerType::Result(success) => compiler_type_contains_function(success),
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_contains_function)
        }),
        _ => false,
    }
}

fn function_llvm_return_type(function: &CompilerFunction) -> String {
    if function.result_captures.is_empty() {
        llvm_type(&function.result_type)
    } else {
        format!(
            "{{ {} }}",
            std::iter::once(llvm_type(&function.result_type))
                .chain(
                    function
                        .result_captures
                        .iter()
                        .map(|capture| llvm_value_type(&capture.value_type))
                )
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Clone, Copy)]
struct TargetValueLayout {
    size: u64,
    alignment: u64,
}

#[allow(clippy::too_many_lines)] // Every semantic representation has an explicit target layout.
fn target_value_layout(value_type: &CompilerType) -> TargetValueLayout {
    match value_type {
        CompilerType::Unit
        | CompilerType::Completed
        | CompilerType::Effect
        | CompilerType::Boolean => TargetValueLayout {
            size: 8,
            alignment: 8,
        },
        CompilerType::Type
        | CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Constraint
        | CompilerType::Comparison
        | CompilerType::ErrorCode
        | CompilerType::Enum(_)
        | CompilerType::Generator(_) => TargetValueLayout {
            size: 32,
            alignment: 32,
        },
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata => {
            unreachable!("static-only compiler values have no target value layout")
        }
        CompilerType::Version
        | CompilerType::SerializationStream(_)
        | CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat
        | CompilerType::InfiniteRational
        | CompilerType::Modular(_)
        | CompilerType::Rational
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::SourceLocation
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::TaskResponse(_)
        | CompilerType::Optional(_)
        | CompilerType::TraversalControl(_)
        | CompilerType::List(_)
        | CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }
        | CompilerType::Character
        | CompilerType::String
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => TargetValueLayout {
            size: 64,
            alignment: 64,
        },
        CompilerType::Refined { base, .. } => target_value_layout(base),
        CompilerType::Tuple(fields) => {
            let mut size = 0;
            let mut alignment = 8;
            for field in fields {
                let field = target_value_layout(field);
                size = align_bits(size, field.alignment) + field.size;
                alignment = alignment.max(field.alignment);
            }
            TargetValueLayout {
                size: align_bits(size, alignment),
                alignment,
            }
        }
        CompilerType::Record(fields) => {
            let mut size = 0;
            let mut alignment = 8;
            for (_, field) in fields {
                let field = target_value_layout(field);
                size = align_bits(size, field.alignment) + field.size;
                alignment = alignment.max(field.alignment);
            }
            for _ in fields {
                size = align_bits(size, 32) + 32;
                alignment = alignment.max(32);
            }
            TargetValueLayout {
                size: align_bits(size, alignment),
                alignment,
            }
        }
        CompilerType::Sum(sum) => {
            let mut size = 32;
            let mut alignment = 32;
            for payload in sum
                .alternatives
                .iter()
                .filter_map(|alternative| alternative.payload.as_ref())
            {
                let payload = target_value_layout(payload);
                size = align_bits(size, payload.alignment) + payload.size;
                alignment = alignment.max(payload.alignment);
            }
            TargetValueLayout {
                size: align_bits(size, alignment),
                alignment,
            }
        }
    }
}

fn compiler_int_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Int, CompilerType::String]
    )
}

fn compiler_string_function_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::String, CompilerType::Function]
    )
}

fn compiler_nested_int_string_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element) if compiler_int_string_pair(element)
    )
}

fn align_bits(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

fn private_aggregate_value_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata
        | CompilerType::Generator(_)
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => false,
        CompilerType::Tuple(fields) => fields.iter().all(private_aggregate_value_supported),
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, field)| private_aggregate_value_supported(field)),
        CompilerType::Sum(sum) => sum.alternatives.iter().all(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_none_or(private_aggregate_value_supported)
        }),
        _ => true,
    }
}

fn llvm_value_type(value_type: &CompilerType) -> String {
    match value_type {
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata => {
            unreachable!("static-only compiler values have no LLVM value type")
        }
        CompilerType::Unit | CompilerType::Completed | CompilerType::Effect => "i8".into(),
        CompilerType::Boolean => "i1".into(),
        CompilerType::Version
        | CompilerType::SerializationStream(_)
        | CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat
        | CompilerType::InfiniteRational
        | CompilerType::Modular(_)
        | CompilerType::Rational
        | CompilerType::Character
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::SourceLocation
        | CompilerType::String
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::TaskResponse(_)
        | CompilerType::Optional(_)
        | CompilerType::TraversalControl(_)
        | CompilerType::List(_)
        | CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => "ptr".into(),
        CompilerType::Type
        | CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Constraint
        | CompilerType::Comparison
        | CompilerType::ErrorCode
        | CompilerType::Enum(_)
        | CompilerType::Generator(_) => "i32".into(),
        CompilerType::Refined { base, .. } => llvm_value_type(base),
        CompilerType::Tuple(fields) => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(llvm_value_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CompilerType::Record(fields) => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|(_, value_type)| llvm_value_type(value_type))
                .chain(fields.iter().map(|_| "i32".to_owned()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CompilerType::Sum(sum) => format!(
            "{{ {} }}",
            std::iter::once("i32".to_owned())
                .chain(
                    sum.alternatives
                        .iter()
                        .filter_map(|alternative| alternative.payload.as_ref())
                        .map(llvm_value_type)
                )
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn function_parameter_value(value_type: &CompilerType, index: usize) -> LlValue {
    machine_value(value_type, format!("%arg{index}"))
}

#[allow(clippy::too_many_lines)] // Every admitted machine representation stays explicit.
fn machine_value(value_type: &CompilerType, value: String) -> LlValue {
    match value_type {
        CompilerType::Unit => LlValue::Unit,
        CompilerType::Completed => LlValue::Completed(value),
        CompilerType::Effect => LlValue::Effect(value),
        CompilerType::Type => LlValue::Enum {
            value,
            enumeration: fundamental_type_enumeration(),
        },
        CompilerType::Scope => LlValue::Enum {
            value,
            enumeration: scope_enumeration(),
        },
        CompilerType::Function => {
            unreachable!("Function values are not admitted at machine ABI reconstruction points")
        }
        CompilerType::Identity
        | CompilerType::TypeView
        | CompilerType::FunctionView
        | CompilerType::LanguageContext
        | CompilerType::Capability
        | CompilerType::NativeSerializer(_)
        | CompilerType::ExternalMetadata => {
            unreachable!("static-only compiler values have no machine representation")
        }
        CompilerType::SerializationStream(_) => {
            unreachable!("SerializationStream function boundaries are not admitted")
        }
        CompilerType::Version => {
            unreachable!("Version function boundaries are not admitted")
        }
        CompilerType::Constraint => {
            unreachable!("Constraint values are not admitted at machine ABI reconstruction points")
        }
        CompilerType::Boolean => LlValue::Boolean(value),
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::InfiniteInt
        | CompilerType::InfiniteNat => LlValue::Int(value),
        CompilerType::Modular(modular) => LlValue::Modular {
            value,
            modular: modular.clone(),
        },
        CompilerType::Rational | CompilerType::InfiniteRational => LlValue::Rational(value),
        CompilerType::Comparison => LlValue::Comparison(value),
        CompilerType::Error => LlValue::Error(value),
        CompilerType::ErrorCode => LlValue::ErrorCode(value),
        CompilerType::ErrorDomain => LlValue::ErrorDomain(value),
        CompilerType::SourceLocation => LlValue::SourceLocation(value),
        CompilerType::Enum(enumeration) => LlValue::Enum {
            value,
            enumeration: enumeration.clone(),
        },
        CompilerType::Generator(generator) => LlValue::Generator {
            value,
            generator: generator.clone(),
            captured_initial: None,
            captured_additional_initials: Vec::new(),
        },
        CompilerType::Task(task) => LlValue::Task {
            value,
            task: task.as_ref().clone(),
        },
        CompilerType::ExternalLocation(location) => LlValue::ExternalLocation {
            value,
            location: location.as_ref().clone(),
        },
        CompilerType::Character | CompilerType::String => LlValue::String(value),
        CompilerType::Range(endpoint) => LlValue::Range {
            value,
            endpoint: endpoint.as_ref().clone(),
        },
        CompilerType::Result(success) | CompilerType::TaskResponse(success) => LlValue::Result {
            value,
            success: success.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::Optional(payload) => LlValue::Optional {
            value,
            payload: payload.as_ref().clone(),
            function_captures: Vec::new(),
        },
        CompilerType::TraversalControl(payload) => LlValue::TraversalControl {
            value,
            payload: payload.as_ref().clone(),
        },
        CompilerType::List(element) => LlValue::List {
            value,
            element: element.as_ref().clone(),
            function_captures: Vec::new(),
        },
        value_type @ (CompilerType::Array { .. }
        | CompilerType::Set(_)
        | CompilerType::Bag(_)
        | CompilerType::Map { .. }) => LlValue::Container {
            value,
            value_type: value_type.clone(),
            function_captures: Vec::new(),
            function_capture_keys: Vec::new(),
        },
        CompilerType::Refined { base, .. } => machine_value(base, value),
        CompilerType::Tuple(_) | CompilerType::Record(_) | CompilerType::Sum(_) => {
            unreachable!("aggregate machine values require structural lowering")
        }
    }
}

fn llvm_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 3);
    for byte in bytes {
        write!(encoded, "\\{byte:02X}").expect("writing to a String cannot fail");
    }
    encoded
}

fn llvm_string(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b' '..=b'~' if !matches!(byte, b'"' | b'\\') => char::from(byte).to_string(),
            _ => format!("\\{byte:02X}"),
        })
        .collect()
}

const PLATFORM_RUNTIME: &str = include_str!("runtime/linux_x86_64.ll");
const INFINITY_RESULT_RUNTIME: &str = include_str!("runtime/infinity_result.ll");
const LIST_BOOLEAN_CORE_RUNTIME: &str = include_str!("runtime/list_boolean_core.ll");
const LIST_RATIONAL_CORE_RUNTIME: &str = include_str!("runtime/list_rational_core.ll");
const LIST_STRING_CORE_RUNTIME: &str = include_str!("runtime/list_string_core.ll");
const LIST_INT_LAYOUT: &str = include_str!("runtime/list_int_layout.ll");
const LIST_INT_CONTAINMENT_RUNTIME: &str = include_str!("runtime/list_int_containment.ll");
const LIST_INT_REMOVAL_RUNTIME: &str = include_str!("runtime/list_int_removal.ll");
const LIST_INT_CORE_RUNTIME: &str = include_str!("runtime/list_int_core.ll");
const LIST_INT_RANGE_SELECTION_RUNTIME: &str = include_str!("runtime/list_int_range_selection.ll");
const LIST_INT_SEQUENCE_RUNTIME: &str = include_str!("runtime/list_int_sequence.ll");
const FUNDAMENTAL_CONTAINERS_RUNTIME: &str = include_str!("runtime/fundamental_containers.ll");
const LIST_NESTED_INT_STRING_CORE_RUNTIME: &str =
    include_str!("runtime/list_nested_int_string_core.ll");

#[cfg(test)]
mod tests {
    use topal_language::analyze_for_compiler;

    use super::*;

    #[test]
    fn emits_target_platform_runtime_and_debug_metadata() {
        let source = "use language (version is v0.1)\nrank is fn (value : Comparison) -> Int\n  value\n    Less then -1\n    Equal then 0\n    Greater then 1\nminimum is fn (left : Int, right : Int) -> Int\n  left\n    < right then left\n    otherwise right\nvalue is 40 + 2\nproduct is 6 * 7\nratio is 6 / 8\ninterval is 0 ..= 2.5\nordered is product >= value\n(value, product, ratio, 1 in interval, ordered, rank (value <=> product), value minimum product, true, \"Topal\")\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/example.t").emit();
        assert!(llvm.contains("target triple = \"x86_64-unknown-linux-gnu\""));
        assert!(llvm.contains("asm sideeffect \"syscall\""));
        assert!(llvm.contains("define void @_start() naked"));
        assert!(llvm.contains("andq $$-16, %rsp"));
        assert!(llvm.contains("@llvm.used"));
        assert!(llvm.contains("call ptr @topal.runtime.int.add"));
        assert!(llvm.contains("call ptr @topal.runtime.int.multiply"));
        assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
        assert!(llvm.contains("call ptr @topal.runtime.rational.divide"));
        assert!(llvm.contains("call ptr @topal.runtime.range.make"));
        assert!(llvm.contains("call i1 @topal.runtime.range.rational.contains"));
        assert!(llvm.contains("@llvm.ctlz.i32"));
        assert!(llvm.contains("name: \"Rational\""));
        assert!(llvm.contains("name: \"Range Rational\""));
        assert!(llvm.contains("comparison.decision.next"));
        assert!(llvm.contains("comparison.value.less"));
        assert!(llvm.contains("constant { i64, i64, [1 x i32] }"));
        assert!(llvm.contains("\\54\\6F\\70\\61\\6C"));
        assert!(llvm.contains("#dbg_value"));
        assert!(llvm.contains("Dwarf Version"));
    }

    #[test]
    fn emits_private_effect_list_nodes_and_pointer_boundaries() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COMPILER-LIST-EFFECT-001,
        // TOPAL-COMPILER-ABI-001, TOPAL-COMPILER-DEBUG-001
        let source = "use language (version is v0.1)\nretain is fn (rows : List Effect) -> List Effect\n  rows\nrows : List Effect is Entry (Effects (), Empty)\nretain rows\n";
        let program = analyze_for_compiler(source).unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "list-effect-boundary.t").emit();

        assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)")));
        assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
        assert!(llvm.contains("call ptr @topal.platform.allocate(i64 16)"));
        assert!(llvm.contains("store i8 0, ptr"));
        assert!(llvm.contains("getelementptr i8, ptr"));
        assert!(llvm.contains("store ptr null, ptr"));
        assert!(llvm.contains("print.list.loop"));
        assert!(llvm.contains("print.list.close.loop"));
        assert!(llvm.contains("DW_TAG_typedef, name: \"List Effect\""));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalList.Effect\""));
        assert!(llvm.contains("#dbg_value(ptr"));
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains the generated source entry")
            .1;
        assert!(!main.contains("topal.runtime.list.int"));
        assert!(!llvm.contains("topal.runtime.list.nested.int-string"));
        assert!(!llvm.contains("%topal.ListStorage"));
    }

    #[test]
    fn emits_exact_int_list_containment_loops() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-CONTAINS-ENTRY-001,
        // TOPAL-LIST-CONTAINS-SEQUENCE-001, TOPAL-LIST-CONTAINS-SUBSEQUENCE-001,
        // TOPAL-COMPILER-LIST-INT-CONTAINMENT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/list-containment.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "list-containment.t").emit();

        assert!(llvm.contains("%topal.ListStorage = type { ptr, ptr }"));
        assert!(llvm.contains("store ptr "));
        assert!(llvm.contains("define internal i1 @topal.runtime.list.int.contains.entry"));
        assert!(llvm.contains("define internal i1 @topal.runtime.list.int.contains.sequence"));
        assert!(llvm.contains("define internal i1 @topal.runtime.list.int.contains.subsequence"));
        assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
        assert!(llvm.contains("call i1 @topal.runtime.list.int.contains.entry"));
        assert!(llvm.contains("call i1 @topal.runtime.list.int.contains.sequence"));
        assert!(llvm.contains("call i1 @topal.runtime.list.int.contains.subsequence"));
        assert!(!llvm.contains("topal.runtime.list.int.remove"));
        assert!(llvm.contains("DW_TAG_typedef, name: \"List Int\""));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalList.Int\""));
    }

    #[test]
    fn emits_immutable_int_list_removal_loops() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-LIST-REMOVE-FIRST-001,
        // TOPAL-LIST-REMOVE-ALL-001, TOPAL-COMPILER-LIST-INT-REMOVAL-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/list-removal.t"))
                .unwrap();
        let llvm = Generator::new(&program, "list-removal.t").emit();

        assert!(llvm.contains("%topal.ListStorage = type { ptr, ptr }"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.remove.first"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.remove.all"));
        assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
        assert!(llvm.contains("call ptr @topal.platform.allocate(i64 %allocation.length)"));
        assert!(llvm.contains("ret ptr %list"));
        assert!(llvm.contains("ret ptr %remaining"));
        assert!(llvm.contains("call ptr @topal.runtime.list.int.remove.first"));
        assert!(llvm.contains("call ptr @topal.runtime.list.int.remove.all"));
        assert!(!llvm.contains("topal.runtime.list.int.contains"));
        assert!(llvm.contains("DW_TAG_typedef, name: \"List Int\""));
    }

    #[test]
    fn emits_basic_int_list_operations_and_total_decision() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-DECISION-LIST-001,
        // TOPAL-TYPE-LIST-EQUALITY-001, TOPAL-LIST-PREPEND-001,
        // TOPAL-LIST-APPEND-001, TOPAL-LIST-CONCAT-001,
        // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-LIST-EMPTY-PREDICATE-001,
        // TOPAL-LIST-EMPTY-001, TOPAL-LIST-ONE-001, TOPAL-LIST-UNCONS-001,
        // TOPAL-LIST-FIRST-001, TOPAL-LIST-REST-001, TOPAL-LIST-REVERSE-001,
        // TOPAL-COMPILER-LIST-INT-CORE-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/lists.t")).unwrap();
        let llvm = Generator::new(&program, "lists.t").emit();

        assert_eq!(llvm.matches("%topal.ListStorage = type").count(), 1);
        assert!(llvm.contains("%topal.ListUnconsStorage = type { ptr, ptr }"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.concat"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.reverse"));
        assert!(llvm.contains("define internal i1 @topal.runtime.list.int.equal"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.entry.count"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.first"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.rest"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.uncons"));
        assert!(llvm.contains("list.decision.entry"));
        assert!(llvm.contains("list.decision.empty"));
        assert!(llvm.contains("call ptr @topal.runtime.optional.some"));
        assert!(llvm.contains("call ptr @topal.runtime.optional.none"));
        assert!(!llvm.contains("topal.runtime.list.int.contains"));
        assert!(!llvm.contains("topal.runtime.list.int.remove"));
    }

    #[test]
    fn emits_complete_list_sequence_operations_as_private_o0_control_flow() {
        // TOPAL-LIST-BOUNDARY-CHECK-001 through TOPAL-LIST-UNZIP-001,
        // TOPAL-COLLECTION-FOREACH-001, TOPAL-COLLECTION-ENTRIES-001,
        // TOPAL-COLLECTION-COLLECT-LIST-001, TOPAL-COLLECTION-COLLECT-STRING-001,
        // TOPAL-COMPILER-LIST-SEQUENCE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/list-sequence-operations.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "list-sequence-operations.t").emit();

        assert_eq!(llvm.matches("%topal.ListStorage = type").count(), 1);
        for helper in [
            "list.int.insert.at",
            "list.int.split.at",
            "list.int.take",
            "list.int.drop",
            "list.int.remove.index",
            "list.int.remove.index.range",
            "list.int.zip.exact",
            "list.int.zip.shortest",
            "list.int.zip.longest",
            "list.int.unzip",
            "list.int.entries",
            "list.string.collect",
        ] {
            assert!(llvm.contains(helper), "missing {helper}");
        }
        assert!(llvm.contains("list.foreach.loop"));
        assert!(llvm.contains("list.select.loop"));
        assert!(llvm.contains("xor i1"));
        assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 0"));
        assert!(llvm.contains("DW_TAG_typedef, name: \"List String\""));
        assert!(!llvm.contains("declare i8* @malloc"));
        assert!(!llvm.contains("declare i32 @printf"));
    }

    #[test]
    fn emits_fundamental_containers_as_private_o0_collections() {
        // TOPAL-ARRAY-COLLECT-001, TOPAL-SET-COLLECT-001,
        // TOPAL-BAG-COLLECT-001, TOPAL-MAP-COLLECT-001,
        // TOPAL-COLLECTION-ENTRY-COUNT-001,
        // TOPAL-COLLECTION-EMPTY-PREDICATE-001,
        // TOPAL-ARRAY-GET-CHECKED-001, TOPAL-MAP-LOOKUP-001,
        // TOPAL-SET-CONTAINS-001, TOPAL-BAG-MULTIPLICITY-001,
        // TOPAL-COMPILER-FUNDAMENTAL-CONTAINERS-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/fundamental-containers.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "fundamental-containers.t").emit();

        for helper in [
            "container.array.int.collect",
            "container.set.int.collect",
            "container.bag.int.collect",
            "container.map.string-int.collect",
            "container.entry.count",
            "container.empty",
            "container.array.int.at",
            "container.set.int.contains",
            "container.bag.int.multiplicity",
            "container.map.string-int.lookup",
        ] {
            assert!(llvm.contains(helper), "missing {helper}");
        }
        for value_type in ["Array 3 Int", "Set Int", "Bag Int", "Map (String, Int)"] {
            assert!(
                llvm.contains(&format!("DW_TAG_typedef, name: \"{value_type}\"")),
                "missing debug type {value_type}"
            );
        }
        assert!(llvm.contains("%topal.ContainerSequenceHeader = type { i64, ptr }"));
        assert!(llvm.contains("%topal.ContainerBagHeader = type { i64, i64, ptr }"));
        assert!(!llvm.contains("declare i8* @malloc"));
        assert!(!llvm.contains("declare i32 @printf"));
    }

    #[test]
    fn emits_contextual_int_list_functions_as_finite_loops() {
        // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
        // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
        // TOPAL-COMPILER-LIST-INT-FUNCTIONS-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-list-functions.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "anonymous-list-functions.t").emit();

        assert!(llvm.contains("list.map.loop"));
        assert!(llvm.contains("list.select.loop"));
        assert!(llvm.contains("list.fold.loop"));
        assert!(llvm.contains("phi ptr"));
        assert!(llvm.contains("call ptr @topal.platform.allocate(i64 16)"));
        assert!(!llvm.contains("topal.fn.anonymous"));
        assert!(!llvm.contains("topal.runtime.list.int.functions"));
    }

    #[test]
    fn emits_int_pair_list_product_map_without_a_generic_runtime() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-COLLECTION-MAP-001,
        // TOPAL-FUNCTION-ANONYMOUS-001,
        // TOPAL-COMPILER-LIST-INT-PAIR-MAP-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-product-pattern.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "anonymous-product-pattern.t").emit();

        assert!(llvm.contains("call ptr @topal.platform.allocate(i64 24)"));
        assert!(llvm.contains("getelementptr i8, ptr %"));
        assert!(llvm.contains("i64 16"));
        assert!(llvm.contains("list.map.loop"));
        assert!(llvm.contains("DW_TAG_typedef, name: \"List (Int, Int)\""));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalList.(Int, Int)\""));
        assert!(!llvm.contains("topal.runtime.list.pair"));
        assert!(!llvm.contains("topal.fn.anonymous"));
        assert!(!llvm.contains("call ptr %"));

        let bound = "use language (version is v0.1)\ncombine is { (left, right) } left + right\npairs : List (Int, Int) is Entry ((1, 2), Empty)\npairs map combine\n";
        let program = analyze_for_compiler(bound).unwrap();
        let llvm = Generator::new(&program, "bound-product-pattern.t").emit();
        assert!(llvm.contains("list.map.loop"));
        assert!(llvm.contains("<anonymous fn/1>"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_exact_recursive_int_string_list_nodes_and_core_loops() {
        // TOPAL-TYPE-LIST-CONSTRUCT-001, TOPAL-TYPE-LIST-EQUALITY-001,
        // TOPAL-TYPE-LIST-RECURSIVE-001, TOPAL-LIST-FIRST-001,
        // TOPAL-LIST-ENTRY-COUNT-001, TOPAL-COMPILER-LIST-RECURSIVE-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/nested-lists.t"))
                .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "nested-lists.t").emit();

        assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)")));
        assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
        assert!(llvm.contains("call ptr @topal.platform.allocate(i64 24)"));
        assert!(llvm.contains("call ptr @topal.platform.allocate(i64 16)"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.nested.int-string.first"));
        assert!(
            llvm.contains("define internal ptr @topal.runtime.list.nested.int-string.entry.count")
        );
        assert!(llvm.contains("define internal i1 @topal.runtime.list.nested.int-string.equal"));
        assert!(llvm.contains("call i1 @topal.runtime.list.int-string.equal"));
        assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
        assert!(llvm.contains("call i1 @topal.runtime.string.equal"));
        assert!(llvm.contains("print.list.advance"));
        assert!(llvm.contains("DW_TAG_typedef, name: \"List (Int, String)\""));
        assert!(llvm.contains("DW_TAG_typedef, name: \"List List (Int, String)\""));
        assert!(llvm.contains("TopalList.(Int, String)"));
        assert!(llvm.contains("TopalList.List (Int, String)"));
        assert!(!llvm.contains("%topal.ListStorage"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn specializes_bound_anonymous_int_list_functions_without_dispatch() {
        // TOPAL-COLLECTION-MAP-001, TOPAL-COLLECTION-SELECT-001,
        // TOPAL-COLLECTION-FOLD-001, TOPAL-FUNCTION-ANONYMOUS-001,
        // TOPAL-COMPILER-LIST-INT-BOUND-FUNCTIONS-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/bound-anonymous-functions.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "bound-anonymous-functions.t").emit();

        assert!(llvm.contains("list.map.loop"));
        assert!(llvm.contains("list.select.loop"));
        assert!(llvm.contains("list.fold.loop"));
        assert!(llvm.contains("<anonymous fn/1>"));
        assert!(llvm.contains("<anonymous fn/2>"));
        assert!(!llvm.contains("topal.fn.anonymous"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_short_circuiting_int_list_fold_control_inline() {
        // TOPAL-EXEC-TRAVERSAL-CONTROL-001,
        // TOPAL-COMPILER-TRAVERSAL-CONTROL-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/traversal-control.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "traversal-control.t").emit();

        assert!(llvm.contains("store i64 0, ptr"));
        assert!(llvm.contains("store i64 1, ptr"));
        assert!(llvm.contains("list.fold.continue"));
        assert!(llvm.contains("list.fold.finish"));
        assert!(llvm.contains("icmp eq i64"));
        assert!(llvm.contains("DW_TAG_typedef, name: \"TraversalControl Int\""));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalTraversalControl.Int\""));
        assert!(!llvm.contains("topal.runtime.traversal.control"));
        assert!(!llvm.contains("topal.fn.anonymous"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_int_list_range_selection_as_private_finite_loops() {
        // TOPAL-RANGE-VALUE-SELECTION-001, TOPAL-RANGE-INDEX-SELECTION-001,
        // TOPAL-COMPILER-RANGE-SELECTION-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/range-selection.t"))
                .unwrap();
        let llvm = Generator::new(&program, "range-selection.t").emit();

        assert!(llvm.contains("call ptr @topal.runtime.list.int.select.value.range"));
        assert!(llvm.contains("call ptr @topal.runtime.list.int.select.index.range"));
        assert!(llvm.contains("define internal ptr @topal.runtime.list.int.select.range"));
        assert!(llvm.contains("call i1 @topal.runtime.range.int.contains"));
        assert!(llvm.contains("call ptr @topal.runtime.int.from.u64"));
        assert!(llvm.contains("c\"\\6F\\70\\61\""));
        assert!(!llvm.contains("RangeSelectionOf"));
        assert!(!llvm.contains("SliceOf"));
    }

    #[test]
    fn erases_diagnostic_controls_before_llvm_lowering() {
        // TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001, TOPAL-SYN-DIAG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/diagnostic-controls.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "diagnostic-controls.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module defines the source entry point")
            .1;
        assert_eq!(main.matches("call ptr @topal.runtime.int.add(").count(), 1);
        assert!(!llvm.contains("disable-warning"));
        assert!(!llvm.contains("disable-diagnostic"));
        assert!(!llvm.contains("topal.runtime.diagnostic"));
    }

    #[test]
    fn emits_nat_comparison_with_the_exact_int_representation() {
        // TOPAL-COMPILER-NAT-COMPARISON-001
        let source = include_str!("../../../examples/language/nat-equality-and-ordering.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/nat-equality-and-ordering.t").emit();
        assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 12);
        assert!(
            llvm.matches("call i32 @topal.runtime.rational.compare")
                .count()
                >= 1
        );
        assert!(!llvm.contains("topal.runtime.nat.compare"));
        assert!(llvm.contains("name: \"Nat\""));
        assert!(llvm.contains("define internal fastcc i32 @topal.fn.compare_2dnat.0"));
        assert!(llvm.contains("DILocalVariable(name: \"left\", arg: 1"));
        assert!(llvm.contains("DILocalVariable(name: \"right\", arg: 2"));
    }

    #[test]
    fn emits_root_local_infinity_sentinels_and_exact_range_operations() {
        // TOPAL-NUM-INFINITY-001, TOPAL-RANGE-BOUNDS-001,
        // TOPAL-RANGE-INTERSECTION-001,
        // TOPAL-COMPILER-INFINITY-001
        let source = include_str!("../../../examples/language/infinity-values-and-ranges.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/infinity-values-and-ranges.t").emit();

        assert!(llvm.contains("@topal.runtime.int.positive.infinity = private constant"));
        assert!(llvm.contains("@topal.runtime.int.negative.infinity = private constant"));
        assert!(llvm.contains("c\"+Infinity\""));
        assert!(llvm.contains("c\"-Infinity\""));
        assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 8);
        assert!(llvm.matches("call ptr @topal.runtime.range.make").count() >= 2);
        assert!(llvm.contains("call i1 @topal.runtime.range.int.contains"));
        assert!(llvm.contains("name: \"Int\""));
        assert!(llvm.contains("name: \"Nat\""));
        assert!(llvm.contains("name: \"Range Int\""));
        assert!(!llvm.contains("@topal.runtime.int.add(ptr @topal.runtime.int.positive.infinity"));
    }

    #[test]
    fn emits_runtime_constructed_rational_infinities_and_exact_ranges() {
        // TOPAL-NUM-INFINITY-001, TOPAL-RANGE-RATIONAL-001,
        // TOPAL-COMPILER-INFINITY-001
        let source =
            include_str!("../../../examples/language/rational-infinity-values-and-ranges.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/rational-infinity-values-and-ranges.t").emit();

        assert!(
            llvm.matches("call ptr @topal.runtime.rational.from.int(ptr @topal.runtime.int.")
                .count()
                >= 3
        );
        assert!(llvm.contains("%either.infinity = or i1"));
        assert!(
            llvm.matches("call i32 @topal.runtime.rational.compare")
                .count()
                >= 8
        );
        assert!(llvm.contains("call i1 @topal.runtime.range.rational.contains"));
        assert!(llvm.contains("call ptr @topal.runtime.range.rational.intersection"));
        assert!(llvm.contains("name: \"Rational\""));
        assert!(llvm.contains("name: \"Range Rational\""));
        assert!(!llvm.contains("@topal.runtime.rational.positive.infinity ="));
        assert!(!llvm.contains("@topal.runtime.rational.negative.infinity ="));
    }

    #[test]
    fn emits_o0_infinity_arithmetic_through_validating_runtime_paths() {
        // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-COMPILER-INFINITY-001
        let source = include_str!("../../../examples/language/infinity-arithmetic.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/infinity-arithmetic.t").emit();

        for operation in ["add", "subtract", "multiply", "negate", "absolute"] {
            assert!(
                llvm.contains(&format!("call ptr @topal.runtime.int.{operation}")),
                "missing Int infinity operation {operation}"
            );
        }
        for operation in ["add", "subtract", "multiply", "negate", "absolute"] {
            assert!(
                llvm.contains(&format!("call ptr @topal.runtime.rational.{operation}")),
                "missing Rational infinity operation {operation}"
            );
        }
        assert!(llvm.contains("%either.infinity = or i1"));
        assert!(llvm.contains("label %indeterminate.infinity"));
        assert!(llvm.contains("label %make.infinity"));
        assert!(llvm.contains("name: \"Int\""));
        assert!(llvm.contains("name: \"Rational\""));
        assert!(!llvm.contains("try.multiply.infinity"));
    }

    #[test]
    fn emits_dynamic_infinity_multiplication_through_result_runtime_paths() {
        // TOPAL-NUM-INFINITY-ARITHMETIC-001, TOPAL-TYPE-RESULT-001,
        // TOPAL-COMPILER-INFINITY-001
        let source = include_str!("../../../examples/language/dynamic-infinity-results.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/dynamic-infinity-results.t").emit();

        assert_eq!(
            llvm.matches("call ptr @topal.runtime.int.try.multiply.infinity")
                .count(),
            2
        );
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.rational.try.multiply.infinity")
                .count(),
            2
        );
        assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 3"));
        assert!(llvm.contains("%exactly.one.infinity = xor i1"));
        assert!(llvm.contains("br i1 %valid, label %validate.zero, label %invalid"));
        assert!(llvm.contains("name: \"Result (Int, lang arithmetic ArithmeticErrorCode)\""));
        assert!(llvm.contains("name: \"Result (Rational, lang arithmetic ArithmeticErrorCode)\""));
    }

    #[test]
    fn emits_lexical_block_scope_metadata() {
        // TOPAL-EXEC-BLOCK-001, TOPAL-COMPILER-BLOCK-001
        let source =
            "use language (version is v0.1)\nvalue is 40\n{\n  value is 41\n  value + 1\n}\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/block.t").emit();
        assert!(llvm.contains("distinct !DILexicalBlock"));
        assert!(llvm.contains("DILocalVariable(name: \"value\""));
    }

    #[test]
    fn emits_completed_as_a_retained_zero_data_result() {
        // TOPAL-EXEC-COMPLETED-001
        let source = "use language (version is v0.1)\nfinish is fn () -> Completed\n  result is Completed\n  result\nretain is fn (value : Completed) -> Completed\n  value\n(finish (), retain Completed, Completed = Completed)\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/completed.t").emit();
        assert!(llvm.contains("define internal fastcc i8 @topal.fn.finish.0"));
        assert!(llvm.contains("call fastcc i8 @topal.fn.finish.0"));
        assert!(llvm.contains("call fastcc i8 @topal.fn.retain.1(i8 0)"));
        assert!(llvm.contains("icmp eq i8 0, 0"));
        assert!(llvm.contains("ret i8 0"));
        assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Completed\""));
    }

    #[test]
    fn emits_empty_effect_as_a_distinct_zero_data_scalar() {
        // TOPAL-EFFECT-EMPTY-001, TOPAL-EFFECT-IDENTITY-001,
        // TOPAL-EFFECT-BOUNDARY-001, TOPAL-EFFECT-PRODUCT-001
        for source in [
            include_str!("../../../examples/language/empty-effects.t"),
            include_str!("../../../examples/language/effect-classifier.t"),
            include_str!("../../../examples/language/effect-identity.t"),
            include_str!("../../../examples/language/effect-products.t"),
            include_str!("../../../examples/language/unit-effect-value.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, "empty-effects.t").emit();
            assert!(llvm.contains("name: \"Effect\""));
            assert!(llvm.contains("DIEnumerator(name: \"empty\", value: 0)"));
            assert!(!llvm.contains("topal.runtime.effect"));
        }

        let source = include_str!("../../../examples/language/effect-function-boundary.t");
        let program = analyze_for_compiler(source).unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "effect-function-boundary.t").emit();
        assert!(llvm.contains(&format!("define internal fastcc i8 @{symbol}(i8 %arg0)")));
        assert!(llvm.contains(&format!("call fastcc i8 @{symbol}(i8 0)")));
        assert!(!llvm.contains("topal.runtime.effect"));

        let equality =
            analyze_for_compiler(include_str!("../../../examples/language/effect-identity.t"))
                .unwrap();
        let llvm = Generator::new(&equality, "effect-identity.t").emit();
        assert!(llvm.contains("icmp eq i8 0, 0"));
    }

    #[test]
    fn emits_recursive_private_tuple_function_results_and_debug_types() {
        // TOPAL-EXEC-COMPLETION-EFFECT-VALUE-001,
        // TOPAL-COMPILER-TUPLE-RESULT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/completion-effect-value.t"
        ))
        .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "completion-effect-value.t").emit();
        assert!(llvm.contains(&format!("define internal fastcc {{ i8, i8 }} @{symbol}()")));
        assert!(llvm.contains("insertvalue { i8, i8 } poison, i8 0, 0"));
        assert!(llvm.contains("insertvalue { i8, i8 } %v0, i8 0, 1"));
        assert!(llvm.contains(&format!("call fastcc {{ i8, i8 }} @{symbol}()")));
        assert!(llvm.contains("extractvalue { i8, i8 }"));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"(Completed, Effect)\""));
        assert!(llvm.contains("name: \"_0\""));
        assert!(llvm.contains("name: \"_1\""));

        let nested = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn static () -> ((Int, Boolean), String)\n  ((42, true), \"Topal\")\nmake ()\n",
        )
        .unwrap();
        let llvm = Generator::new(&nested, "nested-tuple-result.t").emit();
        assert!(llvm.contains("fastcc { { ptr, i1 }, ptr }"));
        assert!(llvm.contains("insertvalue { ptr, i1 }"));
        assert!(llvm.contains("extractvalue { { ptr, i1 }, ptr }"));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"((Int, Boolean), String)\", file:"));
        assert!(llvm.contains("size: 192, align: 64"));
    }

    #[test]
    fn emits_fieldwise_phi_nodes_for_tuple_decision_results() {
        // TOPAL-COMPILER-TUPLE-DECISION-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/tuple-decision-results.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "tuple-decision-results.t").emit();
        for function in program
            .functions
            .iter()
            .filter(|function| function.source_name.starts_with("choose-"))
        {
            let definition = llvm
                .split_once(&format!("@{}(", function.symbol))
                .unwrap()
                .1
                .split_once("\n}\n")
                .unwrap()
                .0;
            assert_eq!(definition.matches("phi ptr").count(), 2, "{definition}");
            assert!(!definition.contains("phi {"), "{definition}");
        }

        let nested = analyze_for_compiler(
            "use language (version is v0.1)\nselect is fn (condition : Boolean) -> ((Int, Boolean), String)\n  condition\n    true then ((1, true), \"yes\")\n    false then ((0, false), \"no\")\nselect true\n",
        )
        .unwrap();
        let symbol = &nested.functions[0].symbol;
        let llvm = Generator::new(&nested, "nested-tuple-decision.t").emit();
        let definition = llvm
            .split_once(&format!("@{symbol}("))
            .unwrap()
            .1
            .split_once("\n}\n")
            .unwrap()
            .0;
        assert_eq!(definition.matches("phi ptr").count(), 2, "{definition}");
        assert_eq!(definition.matches("phi i1").count(), 1, "{definition}");
        assert!(!definition.contains("phi {"), "{definition}");
    }

    #[test]
    fn emits_exact_private_tuple_parameter_prototypes_and_debug_shadows() {
        // TOPAL-COMPILER-TUPLE-PARAMETER-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/tuple-function-parameters.t"
        ))
        .unwrap();
        let symbol = |name: &str| {
            program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
                .symbol
                .as_str()
        };
        let llvm = Generator::new(&program, "tuple-function-parameters.t").emit();

        let retain = symbol("retain");
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ {{ ptr, i1 }}, ptr }} @{retain}({{ {{ ptr, i1 }}, ptr }} %arg0)"
        )));
        assert!(llvm.contains(&format!(
            "call fastcc {{ {{ ptr, i1 }}, ptr }} @{retain}({{ {{ ptr, i1 }}, ptr }}"
        )));
        assert!(llvm.contains("extractvalue { { ptr, i1 }, ptr } %arg0, 0"));
        assert!(llvm.contains("store { { ptr, i1 }, ptr } %arg0"));
        assert!(llvm.contains("#dbg_declare(ptr"));

        let choose = symbol("choose");
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ ptr, ptr }} @{choose}({{ ptr, ptr }} %arg0, i1 %arg1)"
        )));
        assert!(llvm.contains(&format!(
            "call fastcc {{ ptr, ptr }} @{choose}({{ ptr, ptr }}"
        )));

        let tuple_first = symbol("tuple-first");
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{tuple_first}({{ ptr, ptr }} %arg0)"
        )));
        let fields_first = symbol("fields-first");
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{fields_first}(ptr %arg0, ptr %arg1)"
        )));

        let discard = symbol("discard-pair");
        let discard_definition = llvm
            .split_once(&format!("@{discard}("))
            .unwrap()
            .1
            .split_once("\n}\n")
            .unwrap()
            .0;
        assert!(discard_definition.starts_with("{ i8, i8 } %arg0)"));
        assert!(!discard_definition.contains("extractvalue"));
        assert!(!llvm.contains("DILocalVariable(name: \"_\""));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"((Int, Boolean), String)\", file:"));
    }

    #[test]
    fn emits_order_preserving_private_record_boundaries_and_debug_types() {
        // TOPAL-COMPILER-RECORD-BOUNDARY-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/record-function-boundaries.t"
        ))
        .unwrap();
        let symbol = |name: &str| {
            program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
                .symbol
                .as_str()
        };
        let llvm = Generator::new(&program, "record-function-boundaries.t").emit();
        let person_type = "{ i1, ptr, i32, i32 }";

        let retain = symbol("retain-person");
        assert!(llvm.contains(&format!(
            "define internal fastcc {person_type} @{retain}({person_type} %arg0)"
        )));
        assert!(llvm.contains(&format!(
            "call fastcc {person_type} @{retain}({person_type}"
        )));
        assert!(llvm.contains(&format!("extractvalue {person_type} %arg0, 0")));
        assert!(llvm.contains(&format!("extractvalue {person_type} %arg0, 3")));
        assert!(llvm.contains(&format!("store {person_type} %arg0")));

        for name in [
            "choose-person",
            "choose-ordered",
            "choose-comparison",
            "choose-enum",
            "choose-optional",
            "choose-result",
        ] {
            let choice_definition = llvm
                .split_once(&format!("@{}(", symbol(name)))
                .unwrap()
                .1
                .split_once("\n}\n")
                .unwrap()
                .0;
            assert_eq!(
                choice_definition.matches("phi i32").count(),
                2,
                "{name}: {choice_definition}"
            );
            assert!(!choice_definition.contains("phi {"), "{name}");
        }
        assert!(llvm.contains("switch i32"));

        assert!(llvm.contains("{ { i1, ptr, i32, i32 }, ptr, i32, i32 }"));
        assert!(
            llvm.contains(
                "DW_TAG_structure_type, name: \"(active : Boolean, name : String)\", file:"
            )
        );
        assert!(llvm.contains("name: \"active\""));
        assert!(llvm.contains("name: \"name\""));
        assert!(llvm.contains("#dbg_declare(ptr"));
        assert!(!llvm.contains("topal.runtime.record"));
    }

    #[test]
    fn emits_fundamental_type_values_as_closed_private_tags() {
        // TOPAL-ABSTRACTION-TYPE-VALUE-001,
        // TOPAL-ABSTRACTION-TYPE-IDENTITY-001,
        // TOPAL-ABSTRACTION-TYPE-BOUNDARY-001
        for source in [
            include_str!("../../../examples/language/type-values.t"),
            include_str!("../../../examples/language/type-classifier.t"),
            include_str!("../../../examples/language/type-identity.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, "type-values.t").emit();
            assert!(!llvm.contains("topal.runtime.type"));
        }

        let classified =
            analyze_for_compiler(include_str!("../../../examples/language/type-classifier.t"))
                .unwrap();
        let llvm = Generator::new(&classified, "type-classifier.t").emit();
        assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Type\""));
        for (name, value) in [
            ("Boolean", 0),
            ("Int", 1),
            ("Nat", 2),
            ("Rational", 3),
            ("String", 4),
            ("Unit", 5),
            ("Scope", 6),
        ] {
            assert!(llvm.contains(&format!("DIEnumerator(name: \"{name}\", value: {value})")));
        }

        let boundary = analyze_for_compiler(include_str!(
            "../../../examples/language/type-function-boundary.t"
        ))
        .unwrap();
        let symbol = &boundary.functions[0].symbol;
        let llvm = Generator::new(&boundary, "type-function-boundary.t").emit();
        assert!(llvm.contains(&format!("define internal fastcc i32 @{symbol}(i32 %arg0)")));
        assert!(llvm.contains(&format!("call fastcc i32 @{symbol}(i32 1)")));

        let identity =
            analyze_for_compiler(include_str!("../../../examples/language/type-identity.t"))
                .unwrap();
        let llvm = Generator::new(&identity, "type-identity.t").emit();
        assert!(llvm.contains("icmp eq i32 1, 1"));
        assert!(llvm.contains("icmp eq i32 1, 4"));
    }

    #[test]
    fn emits_named_constraint_values_as_closed_private_tags() {
        // TOPAL-ABSTRACTION-CONSTRAINT-CLASSIFIER-001,
        // TOPAL-TYPE-CONSTRAINT-001, TOPAL-COMPILER-CONSTRAINT-VALUE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/constraint-classifier.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "constraint-classifier.t").emit();
        assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Constraint\""));
        assert!(llvm.contains("DIEnumerator(name: \"<Constraint Positive>\", value: 0)"));
        assert!(llvm.contains("DIEnumerator(name: \"<Constraint rule>\", value: 1)"));
        assert!(llvm.contains("#dbg_value(i32 0"));
        assert!(llvm.contains("#dbg_value(i32 1"));
        assert!(!llvm.contains("topal.runtime.constraint"));
        assert!(!llvm.contains("define internal fastcc i1 @topal.constraint"));
    }

    #[test]
    fn emits_constraint_validation_with_erased_base_storage_and_result_paths() {
        // TOPAL-TYPE-CONSTRAINT-VALIDATE-001,
        // TOPAL-COMPILER-CONSTRAINT-VALIDATE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/constraints-and-derived-capabilities.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "constraints-and-derived-capabilities.t").emit();
        assert!(llvm.contains("DIDerivedType(tag: DW_TAG_typedef, name: \"Positive\""));
        assert!(llvm.contains("constraint.accepted"));
        assert!(llvm.contains("constraint.rejected"));
        assert!(llvm.contains("constraint.merge"));
        assert!(llvm.contains("call ptr @topal.runtime.result.success"));
        assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 0"));
        assert!(llvm.contains("phi ptr"));
        assert!(llvm.contains("#dbg_declare(ptr"));
        assert!(!llvm.contains("topal.runtime.constraint"));
    }

    #[test]
    fn emits_root_namespace_identity_and_statically_qualified_call() {
        // TOPAL-COMPILER-ROOT-NAMESPACE-001, TOPAL-NAMESPACE-ROOT-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/root-namespace.t"))
                .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "root-namespace.t").emit();
        assert!(llvm.contains(&llvm_bytes(b"root")));
        assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));
    }

    #[test]
    fn emits_use_namespace_as_the_existing_private_scope_identity() {
        // TOPAL-COMPILER-NAMESPACE-USE-001, TOPAL-NAMESPACE-USE-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/use-namespace.t"))
                .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "use-namespace.t").emit();
        assert!(llvm.contains(&format!("call fastcc ptr @{symbol}(ptr")));
        assert!(llvm.contains("!DILocalVariable(name: \"current\""));
        assert!(!llvm.contains("topal.runtime.use"));
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_function_namespace_aliases_as_direct_private_calls() {
        // TOPAL-COMPILER-NAMESPACE-FUNCTION-ALIAS-001,
        // TOPAL-NAMESPACE-ALIAS-001, TOPAL-NAMESPACE-OVERLOAD-001
        for source in [
            include_str!("../../../examples/language/namespace-alias.t"),
            include_str!("../../../examples/language/namespace-overloads.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let symbols = program
                .functions
                .iter()
                .map(|function| function.symbol.clone())
                .collect::<Vec<_>>();
            let llvm = Generator::new(&program, "namespace-alias.t").emit();
            for symbol in symbols {
                assert!(llvm.lines().any(|line| {
                    line.contains("call fastcc") && line.contains(&format!("@{symbol}("))
                }));
            }
            assert!(!llvm.contains("topal.runtime.namespace"));
            assert!(!llvm.contains("topal.runtime.scope"));
        }
    }

    #[test]
    fn emits_namespace_qualified_generator_without_runtime_lookup() {
        // TOPAL-COMPILER-NAMESPACE-GENERATOR-001,
        // TOPAL-NAMESPACE-GENERATOR-001, TOPAL-GENERATOR-FOREACH-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/namespace-generator.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "namespace-generator.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert!(main.contains("#dbg_value(i32 0"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("name: \"Scope\""));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("topal.runtime.namespace"));
        assert!(!main.contains("topal.runtime.scope"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_namespace_data_members_as_stable_ssa_references() {
        // TOPAL-COMPILER-NAMESPACE-DATA-001, TOPAL-NAMESPACE-SNAPSHOT-001
        let program = analyze_for_compiler(
            "use language (version is v0.1)\nanswer is 40 + 2\napi is root\n{\n  answer is 0\n  (api answer, root answer, answer)\n}\n",
        )
        .unwrap();
        let llvm = Generator::new(&program, "namespace-data.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module defines the source entry point")
            .1;
        assert_eq!(
            main.matches("call ptr @topal.runtime.int.add(").count(),
            1,
            "the captured initializer must execute exactly once"
        );
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));

        for source in [
            include_str!("../../../examples/language/namespace-alias-chain.t"),
            include_str!("../../../examples/language/namespace-snapshot.t"),
            include_str!("../../../examples/language/scope-classifier.t"),
            include_str!("../../../examples/language/published-root-member.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, "namespace-data.t").emit();
            assert!(!llvm.contains("topal.runtime.namespace"));
            assert!(!llvm.contains("topal.runtime.scope"));
        }
    }

    #[test]
    fn emits_scope_parameters_with_private_data_environment_arguments() {
        // TOPAL-COMPILER-NAMESPACE-BOUNDARY-001,
        // TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001
        let source = "use language (version is v0.1)\nanswer is 40 + 2\nincrement is fn (value : Int) -> Int\n  value + 1\nread-answer is fn (api : Scope) -> Int\n  api answer\napply is fn (api : Scope, value : Int) -> Int\n  api increment value\nforward is fn (api : Scope) -> Int\n  read-answer api\n(read-answer root, apply root 41, forward root)\n";
        let program = analyze_for_compiler(source).unwrap();
        let forward = program
            .functions
            .iter()
            .find(|function| function.source_name == "forward")
            .unwrap();
        let llvm = Generator::new(&program, "scope-parameter.t").emit();
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(i32 %arg0, ptr %arg1)",
            forward.symbol
        )));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr") && line.contains("(i32 %arg0, ptr %arg1)")
        }));
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module defines the source entry point")
            .1;
        assert_eq!(
            main.matches("call ptr @topal.runtime.int.add(").count(),
            1,
            "the captured namespace initializer executes once"
        );
        assert!(llvm.contains("!DILocalVariable(name: \"api\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"api answer\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn forwards_live_root_data_as_exact_private_arguments() {
        // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
        // TOPAL-NAMESPACE-ROOT-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-root-data-forwarding.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "function-root-data-forwarding.t").emit();
        for name in ["read", "relay", "forward"] {
            let symbol = &program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
                .symbol;
            assert!(llvm.contains(&format!(
                "define internal fastcc {{ ptr, ptr, ptr }} @{symbol}(ptr %arg0, ptr %arg1, ptr %arg2)"
            )));
        }
        let read = &program
            .functions
            .iter()
            .find(|function| function.source_name == "read")
            .unwrap()
            .symbol;
        let relay = &program
            .functions
            .iter()
            .find(|function| function.source_name == "relay")
            .unwrap()
            .symbol;
        assert!(llvm.contains(&format!(
            "call fastcc {{ ptr, ptr, ptr }} @{read}(ptr %arg0, ptr %arg1, ptr %arg2)"
        )));
        assert!(llvm.contains(&format!(
            "call fastcc {{ ptr, ptr, ptr }} @{relay}(ptr %arg0, ptr %arg1, ptr %arg2)"
        )));
        assert_eq!(
            llvm.matches("!DILocalVariable(name: \"root label\", arg: 2")
                .count(),
            3
        );
        assert_eq!(
            llvm.matches("!DILocalVariable(name: \"root answer\", arg: 3")
                .count(),
            3
        );
        assert!(llvm.matches("alloca ptr, align 8").count() >= 6);
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_defining_context_as_a_private_direct_capture_parameter() {
        // TOPAL-COMPILER-CONTEXT-CAPTURE-001, TOPAL-CONTEXT-SELECT-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/constructed-context.t"
        ))
        .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "constructed-context.t").emit();
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{symbol}(ptr %arg0, ptr %arg1)"
        )));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr")
                && line.contains(&format!("@{symbol}(ptr @.topal.int.1, ptr @.topal.int.0)"))
        }));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"@ offset\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.context"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));

        let computed = analyze_for_compiler(
            "use language (version is v0.1)\noffset is 20 + 20\nadd-offset is fn (value : Int) -> Int\n  value + @ offset\nadd-offset 2\n",
        )
        .unwrap();
        let llvm = Generator::new(&computed, "computed-context.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module defines the source entry point")
            .1;
        assert_eq!(
            main.matches("call ptr @topal.runtime.int.add(").count(),
            1,
            "the defining-context initializer must execute exactly once"
        );
    }

    #[test]
    fn forwards_defining_context_as_exact_private_arguments() {
        // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
        // TOPAL-CONTEXT-SELECT-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/defining-context-forwarding.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "defining-context-forwarding.t").emit();
        for name in ["read", "relay", "forward"] {
            let symbol = &program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
                .symbol;
            assert!(llvm.contains(&format!(
                "define internal fastcc {{ ptr, ptr, ptr }} @{symbol}(ptr %arg0, ptr %arg1, ptr %arg2)"
            )));
        }
        let read = &program
            .functions
            .iter()
            .find(|function| function.source_name == "read")
            .unwrap()
            .symbol;
        let relay = &program
            .functions
            .iter()
            .find(|function| function.source_name == "relay")
            .unwrap()
            .symbol;
        assert!(llvm.contains(&format!(
            "call fastcc {{ ptr, ptr, ptr }} @{read}(ptr %arg0, ptr %arg1, ptr %arg2)"
        )));
        assert!(llvm.contains(&format!(
            "call fastcc {{ ptr, ptr, ptr }} @{relay}(ptr %arg0, ptr %arg1, ptr %arg2)"
        )));
        assert_eq!(
            llvm.matches("!DILocalVariable(name: \"@ offset\", arg: 2")
                .count(),
            3
        );
        assert_eq!(
            llvm.matches("!DILocalVariable(name: \"@ label\", arg: 3")
                .count(),
            3
        );
        assert!(llvm.matches("alloca ptr, align 8").count() >= 6);
        assert!(!llvm.contains("topal.runtime.context"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn forwards_exact_scalar_environments_across_proven_recursion() {
        // TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001,
        // TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001,
        // TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001,
        // TOPAL-COMPILER-DEBUG-001
        let source = include_str!("../../../examples/language/recursive-scalar-environments.t")
            .replace("(cycle-even 3, cycle-odd 3)", "cycle-even 3");
        let program = analyze_for_compiler(&source).unwrap();
        let llvm = Generator::new(&program, "recursive-scalar-environments.t").emit();
        assert_eq!(program.functions.len(), 2);
        let even = program
            .functions
            .iter()
            .find(|function| function.source_name == "cycle-even")
            .unwrap();
        let odd = program
            .functions
            .iter()
            .find(|function| function.source_name == "cycle-odd")
            .unwrap();
        for function in [even, odd] {
            assert!(llvm.contains(&format!(
                "define internal fastcc {{ i1, ptr, ptr }} @{}(ptr %arg0, ptr %arg1, ptr %arg2) nounwind noinline",
                function.symbol
            )));
        }
        for (caller, callee) in [(even, odd), (odd, even)] {
            let definition = llvm
                .split_once(&format!("@{}(", caller.symbol))
                .expect("caller definition is emitted")
                .1;
            assert!(definition.contains(&format!(
                "call fastcc {{ i1, ptr, ptr }} @{}(ptr %v4, ptr %arg1, ptr %arg2)",
                callee.symbol
            )));
        }
        assert_eq!(
            llvm.matches("!DILocalVariable(name: \"@ captured\", arg: 2")
                .count(),
            2
        );
        assert_eq!(
            llvm.matches("!DILocalVariable(name: \"root live\", arg: 3")
                .count(),
            2
        );
        assert!(!llvm.contains("topal.runtime.context"));
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn erases_static_empty_effect_view_before_llvm_lowering() {
        // TOPAL-FUNCTION-EFFECT-BOUND-001, TOPAL-EFFECT-CONTAIN-001,
        // TOPAL-INTRO-STATIC-001, TOPAL-INTRO-VIEW-001,
        // TOPAL-COMPILER-FUNCTION-EMPTY-EFFECT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-effect-bound.t"
        ))
        .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "function-effect-bound.t").emit();
        assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)")));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr") && line.contains(&format!("@{symbol}("))
        }));
        assert!(!llvm.contains("FunctionView"));
        assert!(!llvm.contains("signature"));
        assert!(!llvm.contains("topal.runtime.introspection"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn erases_static_introspection_and_lowers_numeric_version() {
        // TOPAL-INTRO-QUALIFIED-001, TOPAL-INTRO-STATIC-001,
        // TOPAL-INTRO-VIEW-001, TOPAL-INTRO-CONTEXT-001,
        // TOPAL-INTRO-RELATION-001, TOPAL-COMPILER-STATIC-INTROSPECTION-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/static-introspection.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "static-introspection.t").emit();
        assert!(llvm.contains("alloca { ptr, ptr, ptr, ptr }, align 8"));
        assert!(llvm.contains("name: \"Version\""));
        assert!(llvm.contains("name: \"major\""));
        assert!(llvm.contains("name: \"minor\""));
        assert!(llvm.contains("name: \"patch\""));
        assert!(llvm.contains("name: \"build\""));
        assert!(!llvm.contains("integer-identity"));
        assert!(!llvm.contains("integer-view"));
        assert!(!llvm.contains("current-context"));
        assert!(!llvm.contains("topal.runtime.introspection"));
        assert!(!llvm.contains("topal.runtime.version"));
    }

    #[test]
    fn lowers_the_lint_namespace_as_an_authority_free_scope_value() {
        // TOPAL-SYN-CONTEXT-001, TOPAL-LINT-VARIANT-001,
        // TOPAL-COMPILER-LINT-VARIANT-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/lint-language-variant.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "lint-language-variant.t").emit();

        assert!(llvm.contains(&llvm_bytes(b"<namespace lang lint>")));
        assert!(!llvm.contains("topal.runtime.lint"));
        assert!(!llvm.contains("topal.runtime.namespace"));
        assert!(!llvm.contains("topal.runtime.scope"));
        assert!(!llvm.contains("call ptr %"));

        let debug_program = analyze_for_compiler(
            "use language (version is v0.1, features is (lint))\nlint-scope : Scope is lang lint\nlint-scope\n",
        )
        .unwrap();
        let debug_llvm = Generator::new(&debug_program, "lint-scope-debug.t").emit();
        assert!(debug_llvm.contains("!DIEnumerator(name: \"<namespace root>\", value: 0)"));
        assert!(debug_llvm.contains("!DIEnumerator(name: \"<namespace lang lint>\", value: 1)"));
    }

    #[test]
    fn embeds_and_validates_canonical_native_serialization() {
        // TOPAL-SER-HEADER-001 through TOPAL-SER-DESER-001,
        // TOPAL-COMPILER-NATIVE-SERIALIZATION-001,
        // TOPAL-COMPILER-PLATFORM-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/native-serialization.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "native-serialization.t").emit();

        assert!(llvm.contains("%topal.SerializationStreamStorage = type { ptr, i64 }"));
        assert!(llvm.contains(&llvm_bytes(b"TOPALSER")));
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.serialization.make")
                .count(),
            1
        );
        assert_eq!(
            llvm.matches("call void @topal.runtime.serialization.verify")
                .count(),
            1
        );
        assert!(llvm.contains("%copy = call ptr @topal.platform.allocate(i64 %length)"));
        assert!(llvm.contains("%actual.byte = load i8, ptr %actual.pointer"));
        assert!(llvm.contains("%expected.byte = load i8, ptr %expected.pointer"));
        assert!(llvm.contains("define internal void @topal.runtime.u64.print"));
        assert!(llvm.contains("name: \"SerializationStream\""));
        assert!(llvm.contains("name: \"TopalSerializationStreamHeader\""));
        assert!(llvm.contains("name: \"byte_count\""));
        assert!(!llvm.contains("declare ptr @serialize"));
        assert!(!llvm.contains("declare ptr @deserialize"));
        assert!(!llvm.contains("call ptr %"));

        let display =
            analyze_for_compiler("use language (version is v0.1)\nv0.1 (lang serialize) true\n")
                .unwrap();
        let display_llvm = Generator::new(&display, "native-stream-display.t").emit();
        assert!(display_llvm.contains("call void @topal.runtime.u64.print(i64 "));
    }

    #[test]
    fn folds_and_erases_static_capability_composition() {
        // TOPAL-CAPABILITY-EVIDENCE-001, TOPAL-CAPABILITY-COHERENCE-001,
        // TOPAL-CAPABILITY-COMPOSE-001, TOPAL-COMPILER-CAPABILITY-COMPOSE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/capability-composition.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "capability-composition.t").emit();
        assert!(llvm.contains(&llvm_bytes(
            b"Equality and Ordering or Foldable and Membership"
        )));
        assert!(!llvm.contains("Comparable"));
        assert!(!llvm.contains("Searchable"));
        assert!(!llvm.contains("ComparableOrSearchable"));
        assert!(!llvm.contains("Capability"));
        assert!(!llvm.contains("topal.runtime.capability"));
        assert!(!llvm.contains("topal.runtime.evidence"));
    }

    #[test]
    fn erases_function_interface_evidence_before_direct_llvm_lowering() {
        // TOPAL-INTERFACE-SHAPE-001, TOPAL-INTERFACE-IMPLEMENTATION-001,
        // TOPAL-COMPILER-FUNCTION-INTERFACE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-interface.t"
        ))
        .unwrap();
        assert_eq!(program.interfaces[0].identity, "root.Parser");
        assert_eq!(
            program.interface_implementations[0].operations[0].declaration_identity,
            "root.parse:ordinary(String)"
        );
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "function-interface.t").emit();
        assert!(llvm.contains(&format!("define internal fastcc i1 @{symbol}(ptr %arg0)")));
        assert!(llvm.contains(&format!("call fastcc i1 @{symbol}(ptr %")));
        assert!(!llvm.contains("root.Parser"));
        assert!(!llvm.contains("Parser"));
        assert!(!llvm.contains("Interface"));
        assert!(!llvm.contains("topal.runtime.interface"));
        assert!(!llvm.contains("topal.runtime.evidence"));
    }

    #[test]
    fn emits_named_function_values_with_direct_retained_calls() {
        // TOPAL-COMPILER-NAMED-FUNCTION-VALUE-001, TOPAL-FUNCTION-VALUE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/named-function-values.t"
        ))
        .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "named-function-values.t").emit();
        assert!(
            llvm.lines().any(|line| {
                line.contains("call fastcc") && line.contains(&format!("@{symbol}("))
            })
        );
        assert!(llvm.contains("!DIEnumerator(name: \"<fn increment>\", value: 0)"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("call ptr %"));

        let displayed = analyze_for_compiler(
            "use language (version is v0.1)\nfirst is fn (value : Int) -> Int\n  value\nsecond is fn (value : Int) -> Int\n  value\n(first, second)\n",
        )
        .unwrap();
        let llvm = Generator::new(&displayed, "function-display.t").emit();
        assert!(llvm.contains(&llvm_bytes(b"<fn first>")));
        assert!(llvm.contains(&llvm_bytes(b"<fn second>")));
    }

    #[test]
    fn emits_symbolic_callable_values_as_direct_operations() {
        // TOPAL-COMPILER-SYMBOLIC-CALLABLE-VALUE-001,
        // TOPAL-FUNCTION-CALLABLE-VALUE-001
        let program =
            analyze_for_compiler(include_str!("../../../examples/language/callable-values.t"))
                .unwrap();
        let llvm = Generator::new(&program, "callable-values.t").emit();
        assert!(llvm.contains("call ptr @topal.runtime.int.add("));
        assert!(llvm.contains("call ptr @topal.runtime.int.negate("));
        assert!(llvm.contains("call i32 @topal.runtime.int.compare("));
        assert!(llvm.contains("!DIEnumerator(name: \"+\", value: 0)"));
        assert!(llvm.contains("!DIEnumerator(name: \"-\", value: 1)"));
        assert!(llvm.contains("!DIEnumerator(name: \"<=>\", value: 2)"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("call ptr %"));

        let displayed = analyze_for_compiler(
            "use language (version is v0.1)\nadd is +\nnegate is -\ncompare-values is <=>\n(add, negate, compare-values)\n",
        )
        .unwrap();
        let llvm = Generator::new(&displayed, "callable-display.t").emit();
        assert!(llvm.contains(&llvm_bytes(b"+")));
        assert!(llvm.contains(&llvm_bytes(b"-")));
        assert!(llvm.contains(&llvm_bytes(b"<=>")));
    }

    #[test]
    fn emits_every_symbolic_callable_value_without_dispatch() {
        // TOPAL-COMPILER-SYMBOLIC-CALLABLE-EXPANDED-001,
        // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-TYPE-CALL-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/expanded-callable-values.t"
        ))
        .unwrap();
        let selector = program
            .functions
            .iter()
            .find(|function| function.source_name == "select")
            .unwrap();
        let llvm = Generator::new(&program, "expanded-callable-values.t").emit();

        for operation in [
            "topal.runtime.int.compare",
            "topal.runtime.int.multiply",
            "topal.runtime.rational.divide",
            "topal.runtime.int.quotient.modulo",
            "topal.runtime.int.modulo",
            "topal.runtime.int.power",
            "topal.runtime.range.make",
        ] {
            assert!(
                llvm.contains(&format!("call ptr @{operation}"))
                    || llvm.contains(&format!("call i32 @{operation}"))
            );
        }
        assert!(llvm.contains(&format!(
            "define internal fastcc i32 @{}(i32 %arg0)",
            selector.symbol
        )));
        assert!(llvm.contains(&format!("call fastcc i32 @{}(i32", selector.symbol)));
        for (name, value) in [
            ("=", 4),
            ("/=", 5),
            ("*", 10),
            ("/", 11),
            ("/%", 12),
            ("%", 13),
            ("^", 14),
            ("..", 15),
            ("<..", 16),
            ("..=", 17),
            ("<..=", 18),
        ] {
            assert!(llvm.contains(&format!("!DIEnumerator(name: \"{name}\", value: {value})")));
        }
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_function_inputs_as_private_tags_with_direct_specialization() {
        // TOPAL-COMPILER-FUNCTION-PARAMETER-001,
        // TOPAL-FUNCTION-CALLABLE-VALUE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-value-boundary.t"
        ))
        .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "function-value-boundary.t").emit();
        assert!(llvm.contains(&format!("define internal fastcc ptr @{symbol}(i32 %arg0)")));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr") && line.contains(&format!("@{symbol}(i32"))
        }));
        assert!(llvm.contains("call ptr @topal.runtime.int.add("));
        assert!(llvm.contains("!DILocalVariable(name: \"operation\""));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_closed_function_results_as_private_tags_with_direct_application() {
        // TOPAL-COMPILER-FUNCTION-RESULT-001,
        // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-FUNCTION-VALUE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-results.t"
        ))
        .unwrap();
        let selectors = program
            .functions
            .iter()
            .filter(|function| function.source_name == "select")
            .collect::<Vec<_>>();
        assert_eq!(selectors.len(), 2);
        let llvm = Generator::new(&program, "function-results.t").emit();
        for selector in selectors {
            assert!(llvm.contains(&format!(
                "define internal fastcc i32 @{}(i32 %arg0)",
                selector.symbol
            )));
            assert!(llvm.contains(&format!("call fastcc i32 @{}(i32", selector.symbol)));
        }
        let increment = program
            .functions
            .iter()
            .find(|function| function.source_name == "increment")
            .unwrap();
        assert!(llvm.contains(&format!("call fastcc ptr @{}(", increment.symbol)));
        assert!(llvm.contains("call ptr @topal.runtime.int.add("));
        assert!(llvm.contains("!DILocalVariable(name: \"selected\""));
        assert!(llvm.contains("!DILocalVariable(name: \"addition\""));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_direct_anonymous_functions_without_a_closure_runtime() {
        // TOPAL-COMPILER-ANONYMOUS-DIRECT-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-function-application.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "anonymous-function-application.t").emit();
        for function in &program.functions {
            assert!(llvm.contains(&format!("define internal fastcc ptr @{}(", function.symbol)));
            assert!(llvm.lines().any(|line| {
                line.contains("call fastcc ptr") && line.contains(&format!("@{}(", function.symbol))
            }));
        }
        assert!(llvm.contains("!DIEnumerator(name: \"<anonymous fn/1>\", value: 18)"));
        assert!(llvm.contains("!DIEnumerator(name: \"<anonymous fn/2>\", value: 19)"));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_anonymous_captures_and_results_as_exact_private_calls() {
        // TOPAL-COMPILER-ANONYMOUS-CAPTURE-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-function-captures.t"
        ))
        .unwrap();
        let captured = program
            .functions
            .iter()
            .find(|function| {
                function.source_name == "<anonymous fn/1>"
                    && function
                        .parameters
                        .iter()
                        .any(|parameter| parameter.name == "offset")
            })
            .unwrap();
        let factory = program
            .functions
            .iter()
            .find(|function| function.source_name == "make-double")
            .unwrap();
        let returned = program
            .functions
            .iter()
            .find(|function| {
                function.source_name == "<anonymous fn/1>"
                    && function.parameters.len() == 1
                    && function.parameters[0].name == "value"
            })
            .unwrap();
        let llvm = Generator::new(&program, "anonymous-function-captures.t").emit();

        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1)",
            captured.symbol
        )));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr")
                && line.contains(&format!("@{}(ptr %arg1, ptr %arg0)", captured.symbol))
        }));
        assert!(llvm.contains(&format!("define internal fastcc i32 @{}()", factory.symbol)));
        assert!(llvm.contains(&format!("call fastcc i32 @{}()", factory.symbol)));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(ptr %arg0)",
            returned.symbol
        )));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr") && line.contains(&format!("@{}(", returned.symbol))
        }));
        assert!(llvm.contains("!DILocalVariable(name: \"offset\", arg: 2"));
        assert!(llvm.contains("!DILocalVariable(name: \"twice\""));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_captured_function_parameters_as_hidden_direct_arguments() {
        // TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-NESTED-001,
        // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/capturing-function-parameters.t"
        ))
        .unwrap();
        let scalar_anonymous = program
            .functions
            .iter()
            .find(|function| {
                function.source_name == "<anonymous fn/1>"
                    && function
                        .parameters
                        .iter()
                        .map(|parameter| parameter.name.as_str())
                        .eq(["input", "left", "right"])
            })
            .unwrap();
        let pair_boundary = program
            .functions
            .iter()
            .find(|function| function.source_name == "apply-pair")
            .unwrap();
        let nested = program
            .functions
            .iter()
            .find(|function| function.source_name == "add")
            .unwrap();
        let llvm = Generator::new(&program, "capturing-function-parameters.t").emit();

        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
            scalar_anonymous.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ ptr, ptr }} @{}(i32 %arg0, ptr %arg1, {{ ptr, ptr }} %arg2)",
            pair_boundary.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
            nested.symbol
        )));
        assert!(llvm.contains("!DIEnumerator(name: \"<fn add>\""));
        assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 2"));
        assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 3"));
        assert!(llvm.contains("!DILocalVariable(name: \"pair\", arg: 2"));
        assert!(!llvm.contains("DILocalVariable(name: \"operation capture"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_captured_function_results_as_exact_private_aggregates() {
        // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-FUNCTION-VALUE-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/capturing-function-results.t"
        ))
        .unwrap();
        let scalar_factory = program
            .functions
            .iter()
            .find(|function| function.source_name == "make-scalars")
            .unwrap();
        let pair_factory = program
            .functions
            .iter()
            .find(|function| function.source_name == "make-pair")
            .unwrap();
        let forwarding = program
            .functions
            .iter()
            .find(|function| function.source_name == "return-operation")
            .unwrap();
        let llvm = Generator::new(&program, "capturing-function-results.t").emit();

        assert!(llvm.contains(&format!(
            "define internal fastcc {{ i32, ptr, ptr }} @{}(ptr %arg0, ptr %arg1)",
            scalar_factory.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ i32, {{ ptr, ptr }} }} @{}({{ ptr, ptr }} %arg0)",
            pair_factory.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ i32, ptr, ptr }} @{}(i32 %arg0, ptr %arg1, ptr %arg2)",
            forwarding.symbol
        )));
        assert!(llvm.contains("call fastcc { i32, ptr, ptr }"));
        assert!(llvm.contains("insertvalue { i32, ptr, ptr } poison, i32"));
        assert!(llvm.contains("extractvalue { i32, ptr, ptr }"));
        assert!(llvm.contains("insertvalue { i32, { ptr, ptr } }"));
        assert!(llvm.contains("extractvalue { i32, { ptr, ptr } }"));
        assert!(llvm.contains("!DILocalVariable(name: \"scalar-operation\""));
        assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 2"));
        assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 3"));
        assert!(llvm.contains("!DILocalVariable(name: \"pair\", arg: 2"));
        assert!(!llvm.contains("DILocalVariable(name: \"topal.function.result"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_function_result_chains_as_once_only_direct_calls() {
        // TOPAL-COMPILER-FUNCTION-RESULT-CHAIN-001,
        // TOPAL-FUNCTION-CALLABLE-VALUE-001, TOPAL-FUNCTION-VALUE-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-result-chains.t"
        ))
        .unwrap();
        let factories = program
            .functions
            .iter()
            .filter(|function| {
                matches!(
                    function.source_name.as_str(),
                    "make-offset" | "make-pair" | "make-closed" | "return-operation"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(factories.len(), 6);
        let llvm = Generator::new(&program, "function-result-chains.t").emit();

        for factory in factories {
            assert_eq!(
                llvm.matches(&format!("@{}(", factory.symbol)).count(),
                2,
                "factory {} must have one definition and one direct call",
                factory.symbol
            );
        }
        assert!(llvm.contains("call fastcc { i32, ptr }"));
        assert!(llvm.contains("call fastcc { i32, { ptr, ptr } }"));
        assert!(llvm.contains("extractvalue { i32, ptr }"));
        assert!(llvm.contains("!DILocalVariable(name: \"offset\", arg: 2"));
        assert!(llvm.contains("!DILocalVariable(name: \"pair\", arg: 2"));
        assert!(!llvm.contains("DILocalVariable(name: \"topal.function.chain"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_anonymous_product_patterns_as_flat_once_only_private_calls() {
        // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-TYPE-PRODUCT-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/anonymous-product-functions.t"
        ))
        .unwrap();
        let make_pair = program
            .functions
            .iter()
            .find(|function| function.source_name == "make-pair")
            .unwrap();
        let captured = program
            .functions
            .iter()
            .find(|function| {
                function.source_name == "<anonymous fn/1>"
                    && function
                        .parameters
                        .iter()
                        .any(|parameter| parameter.name == "offset")
            })
            .unwrap();
        let mixed = program
            .functions
            .iter()
            .find(|function| {
                function.source_name == "<anonymous fn/2>"
                    && function
                        .parameters
                        .iter()
                        .any(|parameter| parameter.name == "extra")
            })
            .unwrap();
        let llvm = Generator::new(&program, "anonymous-product-functions.t").emit();

        assert_eq!(
            llvm.matches(&format!(
                "call fastcc {{ ptr, ptr }} @{}(",
                make_pair.symbol
            ))
            .count(),
            1
        );
        assert!(llvm.contains("extractvalue { ptr, ptr }"));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
            captured.symbol
        )));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr")
                && line.contains(&format!("@{}(", captured.symbol))
                && line.contains("ptr %arg0")
        }));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
            mixed.symbol
        )));
        assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 2"));
        assert!(llvm.contains("!DILocalVariable(name: \"offset\", arg: 3"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));

        let heterogeneous = analyze_for_compiler(
            "use language (version is v0.1)\nchoose : Function is { (condition, text) } text\nchoose (true, \"kept\")\n",
        )
        .unwrap();
        let function = &heterogeneous.functions[0];
        let heterogeneous_llvm = Generator::new(&heterogeneous, "heterogeneous-product.t").emit();
        assert!(heterogeneous_llvm.contains(&format!(
            "define internal fastcc ptr @{}(i1 %arg0, ptr %arg1)",
            function.symbol
        )));
        assert!(heterogeneous_llvm.contains(&format!(
            "call fastcc ptr @{}(i1 true, ptr ",
            function.symbol
        )));
    }

    #[test]
    fn emits_nested_anonymous_patterns_as_recursive_once_only_projections() {
        // TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001,
        // TOPAL-COMPILER-ANONYMOUS-PRODUCT-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/nested-anonymous-patterns.t"
        ))
        .unwrap();
        let make_values = program
            .functions
            .iter()
            .find(|function| function.source_name == "make-values")
            .unwrap();
        let nested = program
            .functions
            .iter()
            .find(|function| {
                function.source_name == "<anonymous fn/1>"
                    && function
                        .parameters
                        .iter()
                        .map(|parameter| parameter.name.as_str())
                        .eq(["left", "middle", "right"])
            })
            .unwrap();
        let llvm = Generator::new(&program, "nested-anonymous-patterns.t").emit();

        assert_eq!(
            llvm.matches(&format!("@{}(", make_values.symbol)).count(),
            2,
            "nested Tuple factory must have one definition and one call"
        );
        assert!(llvm.contains("extractvalue { ptr, { ptr, ptr } }"));
        assert!(llvm.contains("extractvalue { ptr, ptr }"));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(ptr %arg0, ptr %arg1, ptr %arg2)",
            nested.symbol
        )));
        assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 1);
        for (name, argument) in [("left", 1), ("middle", 2), ("right", 3)] {
            assert!(llvm.contains(&format!(
                "!DILocalVariable(name: \"{name}\", arg: {argument}"
            )));
        }
        assert!(!llvm.contains("DILocalVariable(name: \"topal.anonymous.argument"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_function_aggregates_with_exact_private_direct_boundaries() {
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
        // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
        // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/function-aggregate-boundaries.t"
        ))
        .unwrap();
        let symbol = |name: &str| {
            program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
                .symbol
                .clone()
        };
        let make_tuple = symbol("make-tuple");
        let apply_tuple = symbol("apply-tuple");
        let make_record = symbol("make-record");
        let apply_record = symbol("apply-record");
        let make_nested = symbol("make-nested");
        let apply_nested = symbol("apply-nested");
        let llvm = Generator::new(&program, "function-aggregate-boundaries.t").emit();

        for expected in [
            format!("define internal fastcc {{ i32, ptr }} @{make_tuple}(i32 %arg0, ptr %arg1)"),
            format!("define internal fastcc ptr @{apply_tuple}({{ i32, ptr }} %arg0)"),
            format!(
                "define internal fastcc {{ i32, ptr, i32, i32 }} @{make_record}(i32 %arg0, ptr %arg1)"
            ),
            format!("define internal fastcc ptr @{apply_record}({{ i32, ptr, i32, i32 }} %arg0)"),
            format!(
                "define internal fastcc {{ {{ i32, ptr }}, i32 }} @{make_nested}(i32 %arg0, ptr %arg1)"
            ),
            format!("define internal fastcc ptr @{apply_nested}({{ {{ i32, ptr }}, i32 }} %arg0)"),
        ] {
            assert!(llvm.contains(&expected), "missing {expected:?}");
        }
        assert!(llvm.contains("extractvalue { i32, ptr }"));
        assert!(llvm.contains("extractvalue { i32, ptr, i32, i32 }"));
        assert!(llvm.contains("extractvalue { { i32, ptr }, i32 }"));
        assert!(llvm.contains("DW_TAG_member, name: \"operation\""));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
    }

    #[test]
    fn emits_capture_bearing_function_aggregates_with_path_ordered_private_transport() {
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-001,
        // TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001,
        // TOPAL-FUNCTION-VALUE-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/capturing-function-aggregate-boundaries.t"
        ))
        .unwrap();
        let symbol = |name: &str| {
            program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
                .symbol
                .clone()
        };
        let make_record = symbol("make-record");
        let apply_record = symbol("apply-record");
        let forward_record = symbol("forward-record");
        let make_tuple = symbol("make-tuple");
        let apply_tuple = symbol("apply-tuple");
        let apply_one = symbol("apply-one");
        let llvm = Generator::new(&program, "capturing-function-aggregate-boundaries.t").emit();

        let record = "{ i32, i32, ptr, i32, i32, i32 }";
        let extended_record = "{ { i32, i32, ptr, i32, i32, i32 }, ptr, ptr }";
        for expected in [
            format!(
                "define internal fastcc {extended_record} @{make_record}(ptr %arg0, ptr %arg1)"
            ),
            format!(
                "define internal fastcc {{ ptr, ptr }} @{apply_record}({record} %arg0, ptr %arg1, ptr %arg2)"
            ),
            format!(
                "define internal fastcc {extended_record} @{forward_record}({record} %arg0, ptr %arg1, ptr %arg2)"
            ),
            format!("define internal fastcc {{ {{ i32, ptr }}, ptr }} @{make_tuple}(ptr %arg0)"),
            format!("define internal fastcc ptr @{apply_tuple}({{ i32, ptr }} %arg0, ptr %arg1)"),
            format!(
                "define internal fastcc ptr @{apply_one}({{ i32, ptr, i32, i32 }} %arg0, ptr %arg1)"
            ),
        ] {
            assert!(llvm.contains(&expected), "missing {expected:?}");
        }
        assert!(llvm.contains(&format!("call fastcc {extended_record} @{make_record}(")));
        assert!(llvm.contains(&format!(
            "call fastcc {{ ptr, ptr }} @{apply_record}({record}"
        )));
        assert!(llvm.contains("insertvalue { { i32, i32, ptr, i32, i32, i32 }, ptr, ptr }"));
        assert!(llvm.contains("extractvalue { { i32, i32, ptr, i32, i32, i32 }, ptr, ptr }"));
        assert!(llvm.contains("insertvalue { { i32, ptr }, ptr }"));
        assert!(llvm.contains("extractvalue { { i32, ptr }, ptr }"));
        assert!(llvm.contains("DW_TAG_member, name: \"operation\""));
        assert!(!llvm.contains("DILocalVariable(name: \"operation capture"));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
    }

    #[test]
    fn emits_escaping_nested_function_environments_as_exact_private_results() {
        // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
        // TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001,
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/escaping-nested-function-environments.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "escaping-nested-function-environments.t").emit();

        assert_eq!(
            llvm.matches(
                "define internal fastcc { i32, ptr, ptr, ptr } @topal.fn.make_2doperation."
            )
            .count(),
            4
        );
        assert!(llvm.contains(
            "define internal fastcc { i32, { ptr, ptr } } @topal.fn.make_2dpair_2doperation."
        ));
        assert!(llvm.contains(
            "define internal fastcc { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord."
        ));
        assert!(llvm.contains("define internal fastcc ptr @topal.fn.increase."));
        assert!(llvm.contains("call fastcc { i32, ptr, ptr, ptr }"));
        assert!(llvm.contains("call fastcc { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr }"));
        assert!(llvm.contains("extractvalue { i32, ptr, ptr, ptr }"));
        assert!(llvm.contains("extractvalue { { i32, ptr, i32, i32 }, ptr, ptr, ptr, ptr }"));
        for name in ["offset", "pair", "@ context-offset", "root live-offset"] {
            assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
        }
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("topal.runtime.environment"));
    }

    #[test]
    fn emits_optional_function_environments_as_exact_private_paths() {
        // TOPAL-COMPILER-OPTIONAL-FUNCTION-001,
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
        // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/optional-function-environments.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "optional-function-environments.t").emit();

        assert_eq!(
            llvm.matches(
                "define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2doptional."
            )
            .count(),
            5
        );
        assert!(llvm.contains(
            "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
        ));
        assert!(llvm.contains(
            "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr, ptr } @topal.fn.make_2drecord."
        ));
        assert!(llvm.contains("call ptr @topal.runtime.optional.some(ptr"));
        assert!(llvm.contains("call ptr @topal.runtime.optional.none()"));
        assert_eq!(
            llvm.matches("call void @topal.runtime.pattern.identity.fail()")
                .count(),
            4
        );
        for name in [
            "candidate",
            "operation",
            "offset",
            "@ context-offset",
            "root live-offset",
        ] {
            assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
        }
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("topal.runtime.environment"));
    }

    #[test]
    fn emits_sum_function_environments_as_exact_private_paths() {
        // TOPAL-COMPILER-SUM-FUNCTION-001,
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
        // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/sum-function-environments.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "sum-function-environments.t").emit();

        assert_eq!(
            llvm.matches(
                "define internal fastcc { { i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2doperation."
            )
            .count(),
            5
        );
        assert!(llvm.contains(
            "define internal fastcc { { i32, i32, ptr }, ptr, ptr, ptr } @topal.fn.make_2dchoice."
        ));
        assert_eq!(
            llvm.matches("call void @topal.runtime.pattern.identity.fail()")
                .count(),
            4
        );
        for name in [
            "candidate",
            "operation",
            "offset",
            "@ context-offset",
            "root live-offset",
        ] {
            assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
        }
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("topal.runtime.environment"));
    }

    #[test]
    fn emits_result_function_environments_as_exact_private_paths() {
        // TOPAL-COMPILER-RESULT-FUNCTION-001,
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
        // TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/result-function-environments.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "result-function-environments.t").emit();

        assert_eq!(
            llvm.matches("define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.make_2dresult.")
                .count(),
            7
        );
        assert_eq!(
            llvm.matches(
                "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.make_2dfallible."
            )
            .count(),
            2
        );
        assert!(
            llvm.contains(
                "define internal fastcc { ptr, ptr, ptr, ptr } @topal.fn.project_2dresult."
            )
        );
        assert!(llvm.contains(
            "define internal fastcc { ptr, ptr, ptr, ptr, ptr, ptr } @topal.fn.project_2dresult."
        ));
        assert!(llvm.contains(
            "define internal fastcc { { ptr, ptr, i32, i32 }, ptr, ptr, ptr } @topal.fn.make_2drecord."
        ));
        assert!(llvm.contains(
            "define internal fastcc { { ptr, ptr }, ptr, ptr, ptr } @topal.fn.return_2dtuple."
        ));
        assert!(llvm.contains("call ptr @topal.runtime.result.success(ptr"));
        assert!(llvm.contains("insertvalue { ptr, ptr, ptr, ptr, ptr, ptr } poison, ptr"));
        assert!(llvm.contains("insertvalue { ptr, ptr, ptr, ptr, ptr, ptr } %"));
        assert!(llvm.contains(", ptr null, 5"));
        assert!(llvm.contains("ret { ptr, ptr, ptr, ptr, ptr, ptr }"));
        for name in [
            "candidate",
            "operation",
            "offset",
            "@ context-offset",
            "root live-offset",
        ] {
            assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\"")));
        }
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("topal.runtime.environment"));
    }

    #[test]
    fn emits_repeated_anonymous_patterns_as_exact_private_guards() {
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
        // TOPAL-TYPE-MATCH-001, TOPAL-FUNCTION-ANONYMOUS-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/repeated-anonymous-patterns.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "repeated-anonymous-patterns.t").emit();

        assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
        assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 5);
        assert!(llvm.contains("call i32 @topal.runtime.int.compare(ptr %arg0, ptr %arg1)"));
        assert!(llvm.contains("call i1 @topal.runtime.string.equal(ptr %arg0, ptr %arg1)"));
        assert!(llvm.contains("icmp eq i32 %arg0, %arg1"));
        for name in ["value", "text", "operation"] {
            assert!(llvm.contains(&format!("!DILocalVariable(name: \"{name}\", arg: 1")));
            assert!(!llvm.contains(&format!("!DILocalVariable(name: \"{name}\", arg: 2")));
        }
        assert!(!llvm.contains("topal.runtime.pattern.identity.match"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_repeated_anonymous_aggregate_values_as_structural_private_guards() {
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
        // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/repeated-anonymous-aggregate-patterns.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "repeated-anonymous-aggregate-patterns.t").emit();

        assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 4);
        assert!(llvm.contains("extractvalue { ptr, ptr } %arg0, 0"));
        assert!(llvm.contains("extractvalue { ptr, ptr } %arg1, 1"));
        assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
        assert!(llvm.contains("call i1 @topal.runtime.string.equal"));
        assert!(llvm.contains("call i1 @topal.runtime.optional.int.equal(ptr %arg0, ptr %arg1)"));
        assert!(llvm.contains("call i1 @topal.runtime.list.int.equal(ptr %arg0, ptr %arg1)"));
        assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(!llvm.contains("!DILocalVariable(name: \"value\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.pattern.identity.aggregate"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_capture_free_function_aggregate_pattern_identity_as_direct_guards() {
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001,
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/repeated-function-aggregate-patterns.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "repeated-function-aggregate-patterns.t").emit();

        assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 3);
        assert!(llvm.contains("extractvalue { i32, ptr } %arg0, 0"));
        assert!(llvm.contains("extractvalue { i32, ptr } %arg1, 0"));
        assert!(llvm.contains("extractvalue { i32, ptr, i32, i32 } %arg0, 0"));
        assert!(llvm.contains("icmp eq i32"));
        assert!(llvm.contains("call i32 @topal.runtime.int.compare"));
        assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(!llvm.contains("topal.runtime.pattern.identity.aggregate"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_captured_anonymous_function_pattern_identity_as_direct_guards() {
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001,
        // TOPAL-FUNCTION-ANONYMOUS-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/repeated-captured-function-patterns.t"
        ))
        .unwrap();
        let guarded = program
            .functions
            .iter()
            .filter(|function| function.pattern_identities.len() == 2)
            .collect::<Vec<_>>();
        assert_eq!(guarded.len(), 2);
        let llvm = Generator::new(&program, "repeated-captured-function-patterns.t").emit();

        assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 4);
        for function in guarded {
            let signature = format!(
                "define internal fastcc ptr @{}(i32 %arg0, i32 %arg1, ptr %arg2, ptr %arg3)",
                function.symbol
            );
            let start = llvm.find(&signature).unwrap();
            let remaining = &llvm[start..];
            let end = remaining[1..]
                .find("\ndefine ")
                .map_or(remaining.len(), |offset| offset + 1);
            let body = &remaining[..end];
            let identity = body.find("icmp eq i32 %arg0, %arg1").unwrap();
            let capture = body
                .find("call i32 @topal.runtime.int.compare(ptr %arg2, ptr %arg3)")
                .unwrap();
            assert!(identity < capture);
        }

        let ordered_program = analyze_for_compiler(
            "use language (version is v0.1)\nmake is fn (offset : Int, marker : String) -> Function\n  operation : Function is { value } (value + offset, marker)\n  operation\nrepeat : Function is { operation, operation } 42\nrepeat (make (1, \"same\"), make (1, \"same\"))\n",
        )
        .unwrap();
        let ordered = ordered_program
            .functions
            .iter()
            .find(|function| function.pattern_identities.len() == 3)
            .unwrap();
        let ordered_llvm = Generator::new(&ordered_program, "ordered-captures.t").emit();
        let signature = format!(
            "define internal fastcc ptr @{}(i32 %arg0, i32 %arg1, ptr %arg2, ptr %arg3, ptr %arg4, ptr %arg5)",
            ordered.symbol
        );
        let start = ordered_llvm.find(&signature).unwrap();
        let body = &ordered_llvm[start..];
        let source_identity = body.find("icmp eq i32 %arg0, %arg1").unwrap();
        let first_capture = body
            .find("call i1 @topal.runtime.string.equal(ptr %arg2, ptr %arg4)")
            .unwrap();
        let second_capture = body
            .find("call i32 @topal.runtime.int.compare(ptr %arg3, ptr %arg5)")
            .unwrap();
        assert!(source_identity < first_capture && first_capture < second_capture);

        assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
        assert!(llvm.contains("!DILocalVariable(name: \"operation\", arg: 1"));
        assert!(!llvm.contains("!DILocalVariable(name: \"operation\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_captured_named_function_pattern_identity_as_direct_guards() {
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001,
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001,
        // TOPAL-FUNCTION-NESTED-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/repeated-captured-named-function-patterns.t"
        ))
        .unwrap();
        let guarded = program
            .functions
            .iter()
            .find(|function| function.pattern_identities.len() == 2)
            .unwrap();
        let llvm = Generator::new(&program, "repeated-captured-named-function-patterns.t").emit();
        let signature = format!(
            "define internal fastcc ptr @{}(i32 %arg0, i32 %arg1, ptr %arg2, ptr %arg3)",
            guarded.symbol
        );
        let start = llvm.find(&signature).unwrap();
        let body = &llvm[start..];
        let source_identity = body.find("icmp eq i32 %arg0, %arg1").unwrap();
        let capture_identity = body
            .find("call i32 @topal.runtime.int.compare(ptr %arg2, ptr %arg3)")
            .unwrap();
        assert!(source_identity < capture_identity);
        assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
        assert!(llvm.contains("!DILocalVariable(name: \"operation\", arg: 1"));
        assert!(!llvm.contains("!DILocalVariable(name: \"operation\", arg: 2"));
        assert!(llvm.contains("@topal.fn.increase"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_captured_function_aggregate_pattern_identity_as_direct_guards() {
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001,
        // TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001,
        // TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001,
        // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/repeated-captured-function-aggregate-patterns.t"
        ))
        .unwrap();
        let guarded = program
            .functions
            .iter()
            .filter(|function| function.pattern_identities.len() == 2)
            .collect::<Vec<_>>();
        assert_eq!(guarded.len(), 3);
        let llvm =
            Generator::new(&program, "repeated-captured-function-aggregate-patterns.t").emit();

        assert_eq!(llvm.matches("\npattern.identity.mismatch.").count(), 6);
        for function in guarded {
            let start = llvm
                .find(&format!("define internal fastcc ptr @{}(", function.symbol))
                .unwrap();
            let remaining = &llvm[start..];
            let end = remaining[1..]
                .find("\ndefine ")
                .map_or(remaining.len(), |offset| offset + 1);
            let body = &remaining[..end];
            let source_identity = body.find("icmp eq i32").unwrap();
            let capture_identity = body
                .find("call i32 @topal.runtime.int.compare(ptr %arg2, ptr %arg3)")
                .unwrap();
            assert!(source_identity < capture_identity);
        }
        assert!(llvm.contains("call void @topal.runtime.pattern.identity.fail()"));
        assert!(llvm.contains("!DILocalVariable(name: \"package\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(!llvm.contains("!DILocalVariable(name: \"package\", arg: 2"));
        assert!(!llvm.contains("!DILocalVariable(name: \"value\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_nested_functions_with_exact_private_capture_parameters() {
        // TOPAL-COMPILER-NESTED-FUNCTION-001, TOPAL-FUNCTION-NESTED-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/nested-functions.t"
        ))
        .unwrap();
        let symbol = |name: &str| {
            program
                .functions
                .iter()
                .find(|function| function.source_name == name)
                .unwrap()
                .symbol
                .as_str()
        };
        let nested = symbol("add-input");
        let outer = symbol("answer");
        let llvm = Generator::new(&program, "nested-functions.t").emit();

        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{nested}(ptr %arg0, ptr %arg1)"
        )));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr")
                && line.contains(&format!("@{nested}(ptr @.topal.int.0, ptr %arg0)"))
        }));
        assert!(llvm.contains(&format!("define internal fastcc ptr @{outer}(ptr %arg0)")));
        assert!(llvm.contains("!DISubprogram(name: \"add-input\""));
        assert!(llvm.contains("!DISubprogram(name: \"answer\""));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"input\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.closure"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_packaged_fields_as_an_exact_flat_private_signature() {
        // TOPAL-COMPILER-PACKAGED-OPERAND-001,
        // TOPAL-FUNCTION-PACKAGED-OPERAND-001, TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/packaged-function-operand.t"
        ))
        .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "packaged-function-operand.t").emit();
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{symbol}(ptr %arg0, ptr %arg1)"
        )));
        assert!(llvm.lines().any(|line| {
            line.contains("call fastcc ptr") && line.contains(&format!("@{symbol}(ptr"))
        }));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"fallback\", arg: 2"));
        assert!(!llvm.contains(" byval("));
        assert!(!llvm.contains(" sret("));
        assert!(!llvm.contains("topal.runtime.package"));
    }

    #[test]
    fn emits_discarded_parameter_without_debug_binding() {
        // TOPAL-TYPE-MATCH-001, TOPAL-COMPILER-PATTERN-001
        let source = "use language (version is v0.1)\nsecond is fn (_ : Int, value : Int) -> Int\n  value\nsecond (0, 42)\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/discard.t").emit();
        assert!(
            llvm.contains("define internal fastcc ptr @topal.fn.second.0(ptr %arg0, ptr %arg1)")
        );
        assert!(!llvm.contains("DILocalVariable(name: \"_\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\", arg: 2"));
    }

    #[test]
    fn emits_result_decisions_and_native_string_error_metadata() {
        let source = "use language (version is v0.1)\ndivide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)\n  left / right\ndescribe is fn (denominator : Rational) -> String\n  1.0 divide denominator\n    Ok value then \"ok\"\n    Error (code is lang arithmetic division-by-zero) then \"zero\"\n    Error problem then \"other\"\nproblem is 1.0 divide 0.0\n(describe 0.0, problem code, problem domain)\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/result-decision.t").emit();
        assert!(llvm.contains("%topal.StringStorage = type { ptr, i64, ptr, i64 }"));
        assert!(llvm.contains("call ptr @topal.runtime.string.make"));
        assert!(llvm.contains("result.decision.ok"));
        assert!(llvm.contains("result.decision.code"));
        assert!(llvm.contains("call i32 @topal.runtime.error.code"));
        assert!(llvm.contains("error.field.invalid"));
        assert!(llvm.contains("call ptr @topal.runtime.error.domain"));
        assert!(llvm.contains("name: \"String\""));
        assert!(llvm.contains("name: \"Error\""));
        assert!(llvm.contains("name: \"ErrorDomain\""));
        assert!(llvm.contains("name: \"lang arithmetic ArithmeticErrorCode\""));
    }

    #[test]
    fn emits_optional_structured_error_fields_and_source_metadata() {
        // TOPAL-COMPILER-ERROR-OPTIONAL-FIELDS-001, TOPAL-ERROR-FIELD-001,
        // TOPAL-COMPILER-DEBUG-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/optional-result-composition.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "/source/optional-result-composition.t").emit();
        assert!(llvm.contains("%topal.SourceLocationStorage = type { ptr, ptr }"));
        assert!(llvm.contains("call ptr @topal.runtime.error.detail"));
        assert!(llvm.contains("call ptr @topal.runtime.error.cause"));
        assert!(llvm.contains("call ptr @topal.runtime.error.source"));
        assert!(llvm.contains("call ptr @topal.runtime.source.location.line"));
        assert!(llvm.contains("call ptr @topal.runtime.source.location.column"));
        assert!(llvm.contains("name: \"Optional Error\""));
        assert!(llvm.contains("name: \"Optional SourceLocation\""));
        assert!(llvm.contains("name: \"SourceLocation\""));
    }

    #[test]
    fn emits_optional_values_decisions_equality_and_debug_metadata() {
        // TOPAL-COMPILER-OPTIONAL-001
        let source = include_str!("../../../examples/language/optional-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/optional-values.t").emit();
        assert!(llvm.contains("%topal.OptionalStorage = type { i64, ptr }"));
        assert!(llvm.contains("call ptr @topal.runtime.optional.some"));
        assert!(llvm.contains("call ptr @topal.runtime.optional.none"));
        assert!(llvm.contains("optional.decision.some"));
        assert!(llvm.contains("optional.decision.none"));
        assert!(llvm.contains("call i1 @topal.runtime.optional.int.equal"));
        assert!(llvm.contains("name: \"Optional Int\""));
        assert!(llvm.contains("name: \"Optional String\""));
    }

    #[test]
    fn emits_optional_rational_values_with_exact_equality() {
        // TOPAL-COMPILER-OPTIONAL-RATIONAL-001
        let source = include_str!("../../../examples/language/optional-rational-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/optional-rational-values.t").emit();
        assert_eq!(
            llvm.matches("call i1 @topal.runtime.optional.rational.equal")
                .count(),
            6
        );
        assert!(llvm.contains("call i32 @topal.runtime.rational.compare"));
        assert!(llvm.contains("name: \"Optional Rational\""));
        assert!(llvm.contains("define internal fastcc ptr @topal.fn.preserve.0(ptr %arg0)"));
        assert!(llvm.contains("optional.decision.some"));
        assert!(llvm.contains("optional.decision.none"));
    }

    #[test]
    fn emits_static_character_evidence_as_the_string_carrier() {
        // TOPAL-COMPILER-CHARACTER-001
        let source = include_str!("../../../examples/language/character-classification.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/character-classification.t").emit();
        assert!(llvm.contains("name: \"Character\""));
        assert!(llvm.contains("define internal fastcc ptr @topal.fn.identity.0(ptr %arg0)"));
        assert_eq!(
            llvm.matches("call i1 @topal.runtime.string.equal").count(),
            5
        );
        assert!(!llvm.contains("runtime.character.validate"));
    }

    #[test]
    fn folds_closed_character_counting_and_indexing() {
        // TOPAL-COMPILER-CHARACTER-OBSERVATION-001
        let source = include_str!("../../../examples/language/string-character-at.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/string-character-at.t").emit();
        let generated = llvm
            .split_once("@topal.fn.")
            .expect("regression has a generated source function")
            .1;
        assert_eq!(
            generated
                .matches("call ptr @topal.runtime.optional.some")
                .count(),
            4
        );
        assert_eq!(
            generated
                .matches("call ptr @topal.runtime.optional.none")
                .count(),
            3
        );
        assert!(llvm.contains("name: \"Optional Character\""));
        assert!(llvm.contains("define internal fastcc ptr @topal.fn.describe.0(ptr %arg0)"));
        assert!(llvm.contains("optional.decision.some"));
        assert!(llvm.contains("optional.decision.none"));
        assert!(!llvm.contains("runtime.character.count"));
        assert!(!llvm.contains("runtime.character.at"));
    }

    #[test]
    fn folds_closed_pinned_unicode_transformations() {
        // TOPAL-COMPILER-UNICODE-FOLD-001
        for (source, name, expected) in [
            (
                include_str!("../../../examples/language/string-uppercase.t"),
                "string-uppercase.t",
                "STRASSE ΣΣ",
            ),
            (
                include_str!("../../../examples/language/string-lowercase.t"),
                "string-lowercase.t",
                "i\u{307}ς",
            ),
            (
                include_str!("../../../examples/language/string-case-fold.t"),
                "string-case-fold.t",
                "strasse σσ",
            ),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, name).emit();
            assert!(llvm.contains(&llvm_bytes(expected.as_bytes())));
            assert!(!llvm.contains("runtime.unicode"));
        }
        for (source, name) in [
            (
                include_str!("../../../examples/language/string-normalization.t"),
                "string-normalization.t",
            ),
            (
                include_str!("../../../examples/language/string-normalization-nfd.t"),
                "string-normalization-nfd.t",
            ),
            (
                include_str!("../../../examples/language/string-canonical-equality.t"),
                "string-canonical-equality.t",
            ),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, name).emit();
            assert!(!llvm.contains("runtime.unicode"));
        }
    }

    #[test]
    fn lowers_anonymous_records_without_a_runtime_abi() {
        // TOPAL-COMPILER-RECORD-001, TOPAL-TYPE-PRODUCT-001
        let source = include_str!("../../../examples/language/strings-and-products.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "strings-and-products.t").emit();
        assert!(llvm.contains(&llvm_bytes(b"name")));
        assert!(llvm.contains(&llvm_bytes(b"active")));
        assert!(llvm.contains(&llvm_bytes(b"Ada")));
        assert!(!llvm.contains("runtime.record"));
    }

    #[test]
    fn lowers_record_reconstruction_without_a_runtime_abi() {
        // TOPAL-COMPILER-RECONSTRUCT-001, TOPAL-TYPE-RECONSTRUCT-001
        let source = include_str!("../../../examples/language/record-reconstruction.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "record-reconstruction.t").emit();
        assert!(llvm.contains(&llvm_bytes(b"Ada")));
        assert!(!llvm.contains("runtime.record"));
        assert!(!llvm.contains("runtime.reconstruct"));
    }

    #[test]
    fn lowers_recursive_structural_comparisons() {
        // TOPAL-COMPILER-STRUCTURAL-COMPARISON-001
        let source = include_str!("../../../examples/language/equality-and-ordering.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "equality-and-ordering.t").emit();
        assert!(llvm.contains("tuple.compare.next"));
        assert!(llvm.contains("phi i32"));
        assert!(llvm.contains("@topal.runtime.rational.compare"));
        assert!(llvm.contains("@topal.runtime.string.equal"));
        assert!(!llvm.contains("runtime.tuple"));
        assert!(!llvm.contains("runtime.record"));
    }

    #[test]
    fn emits_utf8_byte_counts_through_the_native_string_descriptor() {
        // TOPAL-COMPILER-STRING-UTF8-BYTE-COUNT-001
        let source = include_str!("../../../examples/language/string-utf8-byte-count.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/string-utf8-byte-count.t").emit();
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.string.utf8.byte.count")
                .count(),
            3
        );
        assert!(llvm.contains("define internal ptr @topal.runtime.int.from.u64"));
        assert!(llvm.contains("lshr i64 %source, 32"));
        assert!(llvm.contains("select i1 %has.high, i64 2, i64 1"));
        assert!(llvm.contains("ret ptr @topal.runtime.int.zero"));
    }

    #[test]
    fn emits_exact_string_and_derived_optional_string_equality() {
        // TOPAL-COMPILER-STRING-EQUALITY-001
        let source = include_str!("../../../examples/language/string-exact-equality.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/string-exact-equality.t").emit();
        assert_eq!(
            llvm.matches("call i1 @topal.runtime.string.equal").count(),
            5
        );
        assert_eq!(
            llvm.matches("call i1 @topal.runtime.optional.string.equal")
                .count(),
            4
        );
        assert!(llvm.contains("%same.length = icmp eq i64 %left.length, %right.length"));
        assert!(llvm.contains("%same.byte = icmp eq i8 %left.byte, %right.byte"));
    }

    #[test]
    fn emits_freestanding_string_construction_concatenation_and_emptiness() {
        // TOPAL-COMPILER-STRING-CONSTRUCTION-001
        let source = include_str!("../../../examples/language/string-construction.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/string-construction.t").emit();
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.string.concat")
                .count(),
            5
        );
        assert_eq!(
            llvm.matches("call i1 @topal.runtime.string.is.empty")
                .count(),
            2
        );
        assert!(llvm.contains("call void @topal.runtime.string.dynamic.print"));
        assert!(llvm.contains("call i1 @topal.runtime.string.has.delimiter"));
        assert!(llvm.contains("call ptr @topal.platform.allocate(i64 %allocation.length)"));
        assert_eq!(llvm.matches("call void @llvm.memcpy.inline").count(), 2);
        assert!(llvm.contains(
            "call ptr @topal.runtime.string.make(ptr %data, i64 %length, ptr null, i64 0)"
        ));
    }

    #[test]
    fn emits_recursive_positional_product_equality_from_field_evidence() {
        // TOPAL-COMPILER-TUPLE-EQUALITY-001
        let source = include_str!("../../../examples/language/tuple-equality.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/tuple-equality.t").emit();
        assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 5);
        assert!(
            llvm.matches("call i32 @topal.runtime.rational.compare")
                .count()
                >= 3
        );
        assert!(llvm.matches("call i1 @topal.runtime.string.equal").count() >= 5);
        assert!(
            llvm.matches("call i1 @topal.runtime.optional.int.equal")
                .count()
                >= 2
        );
        assert!(
            llvm.matches("call i1 @topal.runtime.optional.string.equal")
                .count()
                >= 3
        );
        assert!(llvm.matches("and i1").count() >= 20);
    }

    #[test]
    fn emits_distinct_overload_and_static_function_instances() {
        let source = "use language (version is v0.1)\ndescribe is fn (value : Int) -> String\n  \"integer\"\ndescribe is fn (value : String) -> String\n  describe 42\nanswer is fn static () -> Int\n  42\nadd is fn static (left : Int, right : Int) -> Int\n  left + right\n(describe \"Topal\", answer (), 20 add 22)\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/function-overloads.t").emit();
        assert_eq!(llvm.matches("!DISubprogram(name: \"describe\"").count(), 2);
        assert!(llvm.contains("!DISubprogram(name: \"answer\""));
        assert!(llvm.contains("!DISubprogram(name: \"add\""));
        assert_eq!(
            llvm.matches("define internal fastcc ptr @topal.fn.describe")
                .count(),
            2
        );
    }

    #[test]
    fn emits_same_named_cross_overload_edge_without_a_cycle() {
        // TOPAL-FUNCTION-RECURSION-OVERLOAD-IDENTITY-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/overload-recursion-identity.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "overload-recursion-identity.t").emit();
        let integer = "topal.fn.describe.0";
        let string = "topal.fn.describe.1";
        let integer_definition = llvm
            .find(&format!("define internal fastcc ptr @{integer}"))
            .unwrap();
        let string_definition = llvm
            .find(&format!("define internal fastcc ptr @{string}"))
            .unwrap();
        assert!(integer_definition < string_definition);
        assert_eq!(
            llvm.matches(&format!("call fastcc ptr @{integer}(ptr "))
                .count(),
            1
        );
        assert_eq!(
            llvm.matches(&format!("call fastcc ptr @{string}(ptr "))
                .count(),
            1
        );
        assert_eq!(llvm.matches("!DISubprogram(name: \"describe\"").count(), 2);
    }

    #[test]
    fn emits_closed_mutual_int_cycles_with_exact_private_edges() {
        // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001,
        // TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001
        for source in [
            include_str!("../../../examples/language/mutual-int-recursion.t"),
            include_str!("../../../examples/language/mutual-increasing-int-recursion.t"),
            include_str!("../../../examples/language/mutual-multiple-recursive-calls.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, "mutual-int-recursion.t").emit();
            for function in &program.functions {
                assert_eq!(
                    llvm.matches(&format!(
                        "define internal fastcc {} @{}(",
                        llvm_type(&function.result_type),
                        function.symbol
                    ))
                    .count(),
                    1
                );
            }
            assert!(llvm.matches("call fastcc").count() >= program.functions.len());
            assert!(llvm.contains("nounwind noinline"));
            assert!(!llvm.contains("norecurse"));
        }
    }

    #[test]
    fn emits_proven_mutual_nat_cycles_without_runtime_revalidation() {
        // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001,
        // TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001
        for source in [
            include_str!("../../../examples/language/nat-mutual-recursion.t"),
            include_str!("../../../examples/language/nat-mutual-increasing-recursion.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, "nat-mutual-recursion.t").emit();
            for function in &program.functions {
                assert_eq!(function.parameters[0].value_type, CompilerType::Nat);
                assert_eq!(
                    llvm.matches(&format!(
                        "define internal fastcc i1 @{}(ptr %arg0)",
                        function.symbol
                    ))
                    .count(),
                    1
                );
            }
            assert!(llvm.matches("call fastcc i1").count() >= program.functions.len());
            assert_eq!(
                llvm.matches("call ptr @topal.runtime.int.try.to.nat")
                    .count(),
                0
            );
            assert!(llvm.contains("nounwind noinline"));
            assert!(!llvm.contains("norecurse"));
        }
    }

    #[test]
    fn emits_forward_callee_before_its_caller() {
        // TOPAL-FUNCTION-FORWARD-DECLARATION-001, TOPAL-COMPILER-FUNCTION-001
        let source = include_str!("../../../examples/language/forward-function-declarations.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "forward-function-declarations.t").emit();
        let decorate_definition = llvm
            .find("define internal fastcc ptr @topal.fn.decorate.0")
            .unwrap();
        let render_definition = llvm
            .find("define internal fastcc ptr @topal.fn.render.1")
            .unwrap();
        assert!(decorate_definition < render_definition);
        assert!(llvm.contains("call fastcc ptr @topal.fn.decorate.0(ptr %arg0)"));
    }

    #[test]
    fn emits_proven_direct_recursion_with_one_exact_private_signature() {
        // TOPAL-FUNCTION-RECURSION-INT-001
        for (source, name, recursive_calls) in [
            (
                include_str!("../../../examples/language/decreasing-int-recursion.t"),
                "decreasing-int-recursion.t",
                1,
            ),
            (
                include_str!("../../../examples/language/multiple-recursive-calls.t"),
                "multiple-recursive-calls.t",
                2,
            ),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let symbol = &program.functions[0].symbol;
            let llvm = Generator::new(&program, name).emit();
            assert_eq!(
                llvm.matches(&format!("define internal fastcc ptr @{symbol}"))
                    .count(),
                1
            );
            assert_eq!(
                llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
                    .count(),
                recursive_calls
            );
            assert!(llvm.contains("nounwind noinline"));
            assert!(!llvm.contains("norecurse"));
        }
    }

    #[test]
    fn emits_proven_increasing_recursion_with_exact_private_signatures() {
        // TOPAL-FUNCTION-RECURSION-INT-INCREASING-001
        for source in [
            include_str!("../../../examples/language/increasing-int-recursion.t"),
            include_str!("../../../examples/language/positive-recursion-steps.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, "increasing-int-recursion.t").emit();
            for function in &program.functions {
                let symbol = &function.symbol;
                assert_eq!(
                    llvm.matches(&format!("define internal fastcc ptr @{symbol}"))
                        .count(),
                    1
                );
                assert_eq!(
                    llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
                        .count(),
                    1
                );
            }
            assert!(llvm.contains("nounwind noinline"));
            assert!(!llvm.contains("norecurse"));
        }
    }

    #[test]
    fn emits_proven_nat_recursion_without_runtime_revalidation() {
        // TOPAL-FUNCTION-RECURSION-NAT-001,
        // TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001
        for source in [
            include_str!("../../../examples/language/nat-recursion.t"),
            include_str!("../../../examples/language/nat-increasing-recursion.t"),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let symbol = &program.functions[0].symbol;
            let llvm = Generator::new(&program, "nat-recursion.t").emit();
            assert_eq!(
                llvm.matches(&format!("define internal fastcc ptr @{symbol}(ptr %arg0)"))
                    .count(),
                1
            );
            assert_eq!(
                llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
                    .count(),
                1
            );
            assert_eq!(
                llvm.matches("call ptr @topal.runtime.int.try.to.nat")
                    .count(),
                0
            );
            assert!(llvm.contains("nounwind noinline"));
            assert!(!llvm.contains("norecurse"));
        }
    }

    #[test]
    fn emits_explicit_measure_recursion_with_one_complete_signature() {
        // TOPAL-FUNCTION-DECREASES-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/explicit-multi-parameter-decreases.t"
        ))
        .unwrap();
        let symbol = &program.functions[0].symbol;
        let llvm = Generator::new(&program, "explicit-multi-parameter-decreases.t").emit();
        assert_eq!(
            llvm.matches(&format!(
                "define internal fastcc ptr @{symbol}(ptr %arg0, ptr %arg1)"
            ))
            .count(),
            1
        );
        assert_eq!(
            llvm.matches(&format!("call fastcc ptr @{symbol}(ptr %"))
                .count(),
            1
        );
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.int.try.to.nat")
                .count(),
            0
        );
        assert!(llvm.contains("nounwind noinline"));
        assert!(!llvm.contains("norecurse"));
    }

    #[test]
    fn emits_layout_policies_as_closed_nominal_tags_without_a_layout_runtime() {
        // TOPAL-LAYOUT-ENDIAN-001, TOPAL-LAYOUT-ACCESS-001,
        // TOPAL-LAYOUT-BIT-ORDER-001, TOPAL-LAYOUT-PACKING-001,
        // TOPAL-LAYOUT-FIELD-ORDER-001, TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001,
        // TOPAL-LAYOUT-ABSENCE-POLICY-001,
        // TOPAL-COMPILER-LAYOUT-POLICY-001
        let source = "use language (version is v0.1)\nendian is Big\naccess is Reserved\nbits is LeastSignificantFirst\npacking is Packed\nfields is Declared\npayload is Overlay\nabsence is NoTerminator\n(endian, access, bits, packing, fields, payload, absence)\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "layout-policy-values.t").emit();

        for type_name in [
            "Endian",
            "Access",
            "BitOrder",
            "Packing",
            "FieldOrder",
            "PayloadPlacement",
            "LayoutPolicy",
        ] {
            assert!(llvm.contains(&format!("DW_TAG_enumeration_type, name: \"{type_name}\"")));
        }
        for alternative in [
            "Big",
            "Reserved",
            "LeastSignificantFirst",
            "Packed",
            "Declared",
            "Overlay",
            "NoTerminator",
        ] {
            assert!(llvm.contains(&format!("DIEnumerator(name: \"{alternative}\"")));
        }
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        assert!(!main.contains("topal.platform.allocate"));
        assert!(!main.contains("topal.runtime.layout"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_generator_error_code_as_a_closed_nominal_tag_without_a_generator_runtime() {
        // TOPAL-GENERATOR-ERROR-CODE-001,
        // TOPAL-COMPILER-GENERATOR-ERROR-CODE-001
        let program = analyze_for_compiler(include_str!(
            "../../../examples/language/generator-error-codes.t"
        ))
        .unwrap();
        let llvm = Generator::new(&program, "generator-error-codes.t").emit();

        assert!(
            llvm.contains("DW_TAG_enumeration_type, name: \"lang generator GeneratorErrorCode\"")
        );
        assert!(llvm.contains("DIEnumerator(name: \"generator-closed\", value: 0)"));
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("topal.platform.allocate"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_lazy_iterate_construction_as_a_private_debug_token_without_invoking_functions() {
        // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
        // TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001
        for (source, name) in [
            (
                include_str!("../../../examples/language/iterate-generator.t"),
                "iterate-generator.t",
            ),
            (
                include_str!("../../../examples/language/iterate-take-while.t"),
                "iterate-take-while.t",
            ),
        ] {
            let program = analyze_for_compiler(source).unwrap();
            let llvm = Generator::new(&program, name).emit();

            assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Generator Int Unit Unit\""));
            assert!(llvm.contains("DIEnumerator(name: \"<Generator Int Unit Unit>\", value: 0)"));
            let main = llvm
                .split_once("define internal void @topal.main")
                .expect("module contains generated source entry")
                .1;
            assert!(!main.contains("topal.runtime.generator"));
            assert!(!main.contains("topal.platform.allocate"));
            assert!(!main.contains("topal.runtime.int.add"));
            assert!(!main.contains("icmp"));
            assert!(!main.contains("call ptr %"));
        }

        let once = analyze_for_compiler(
            "use language (version is v0.1)\ninitial is fn () -> Int\n  0\nnumbers is (initial ()) iterate ({ value } value + 1)\nnumbers\n",
        )
        .unwrap();
        let llvm = Generator::new(&once, "iterate-initial-once.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        assert_eq!(
            main.matches("call fastcc ptr @topal.fn.initial.").count(),
            1
        );
    }

    #[test]
    fn emits_bounded_iterate_collection_as_an_ordered_list_loop() {
        // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
        // TOPAL-GENERATOR-COLLECT-001,
        // TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001
        let source = include_str!("../../../examples/language/generated-collect.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "generated-collect.t").emit();

        for expected in [
            "generator.collect.loop",
            "generator.collect.accepted",
            "generator.collect.done",
            "call ptr @topal.platform.allocate(i64 16)",
            "call ptr @topal.runtime.int.add",
            "icmp slt i32",
            "List Int",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}: {llvm}");
        }
        assert!(!llvm.contains("topal.runtime.generator"));
        assert!(!llvm.contains("call ptr %"));
    }

    #[test]
    fn emits_bounded_iterate_foreach_as_an_ordered_unit_loop() {
        // TOPAL-GENERATOR-ITERATE-001, TOPAL-GENERATOR-TAKE-WHILE-001,
        // TOPAL-GENERATOR-ITERATE-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001
        let source = include_str!("../../../examples/language/generated-foreach.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "generated-foreach.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        for expected in [
            "generator.foreach.loop",
            "phi ptr",
            "call i32 @topal.runtime.int.compare",
            "generator.foreach.accepted",
            "call ptr @topal.runtime.int.add",
            "generator.foreach.resumed",
            "generator.foreach.done",
            "#dbg_value(i8 0",
        ] {
            assert!(main.contains(expected), "missing {expected:?}: {main}");
        }
        let predicate = main.find("call i32 @topal.runtime.int.compare").unwrap();
        let action = main.find("generator.foreach.accepted").unwrap();
        let next = main.find("call ptr @topal.runtime.int.add").unwrap();
        assert!(predicate < action && action < next);
        assert!(!main.contains("topal.platform.allocate"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_closed_string_character_collection_as_exact_source_identity() {
        // TOPAL-STRING-CHARACTERS-COLLECT-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001
        let source = include_str!("../../../examples/language/string-character-traversal.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "string-character-traversal.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(llvm.contains(&format!("c\"{}\"", llvm_bytes("a\u{301}👩‍🔬🇸🇪".as_bytes()))));
        assert!(llvm.contains("name: \"String\""));
        assert!(!main.contains("topal.runtime.string.concat"));
        assert!(!main.contains("topal.runtime.list"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_closed_string_character_foreach_in_preserved_order() {
        // TOPAL-STRING-CHARACTERS-COLLECT-001,
        // TOPAL-STRING-CHARACTERS-FOREACH-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001
        let source = include_str!("../../../examples/language/string-character-foreach.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "string-character-foreach.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            4
        );
        let mut previous = 0;
        for character in ["a\u{301}", "👩‍🔬", "🇸🇪"] {
            let encoded = format!("c\"{}\"", llvm_bytes(character.as_bytes()));
            let position = llvm[previous..].find(&encoded).map_or_else(
                || panic!("missing {character:?} bytes: {llvm}"),
                |position| previous + position,
            );
            assert!(position >= previous);
            previous = position + encoded.len();
        }
        assert!(main.contains("alloca ptr, align 8"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert_eq!(main.matches("store ptr").count(), 3);
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));

        let empty = analyze_for_compiler(
            "use language (version is v0.1)\ncharacters \"\" foreach { character }\n  _ is String character\n",
        )
        .unwrap();
        let empty_llvm = Generator::new(&empty, "empty-string-character-foreach.t").emit();
        let empty_main = empty_llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        assert_eq!(
            empty_main
                .matches("call ptr @topal.runtime.string.make")
                .count(),
            1
        );
    }

    #[test]
    fn emits_named_string_character_generator_as_a_private_linear_token() {
        // TOPAL-STRING-CHARACTERS-COLLECT-001,
        // TOPAL-STRING-CHARACTERS-FOREACH-001,
        // TOPAL-STRING-CHARACTERS-GENERATOR-001,
        // TOPAL-STRING-CHARACTERS-CLASSIFIER-001,
        // TOPAL-STRING-CHARACTERS-LINEAR-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-GENERATOR-001
        let source = include_str!("../../../examples/language/string-named-character-generator.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "string-named-character-generator.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            4
        );
        assert!(main.contains("#dbg_value(i32 0"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_single_yield_custom_generator_without_runtime_state() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001
        let source = include_str!("../../../examples/language/custom-single-yield-generator.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-single-yield-generator.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert!(main.contains("#dbg_value(i32 0"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_custom_generator_string_input_prefix_before_suspension() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
        // TOPAL-COMPILER-GENERATOR-STRING-INPUT-001
        let source = include_str!("../../../examples/language/custom-generator-string-input.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-string-input.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert_eq!(
            main.matches("call i1 @topal.runtime.string.is.empty")
                .count(),
            1
        );
        let input = main
            .find("call ptr @topal.runtime.string.make")
            .expect("generator application evaluates its String input");
        let predicate = main
            .find("call i1 @topal.runtime.string.is.empty")
            .expect("generator application executes the retained prefix");
        let suspended = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private suspended token");
        let yielded = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("traversal materializes the yielded Character");
        assert!(input < predicate && predicate < suspended && suspended < yielded);
        assert!(main.contains("#dbg_value(ptr"));
        assert!(main.contains("#dbg_value(i1"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"initial-is-empty\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_custom_generator_string_yields_without_runtime_state() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FOREACH-001, TOPAL-STRING-EMPTY-PREDICATE-001,
        // TOPAL-COMPILER-GENERATOR-STRING-YIELD-001
        let source = include_str!("../../../examples/language/custom-generator-string-yield.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-string-yield.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert_eq!(
            main.matches("call i1 @topal.runtime.string.is.empty")
                .count(),
            2
        );
        let input = main
            .find("call ptr @topal.runtime.string.make")
            .expect("generator application evaluates its String input");
        let suspended = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private suspended token");
        let first_action = main
            .find("call i1 @topal.runtime.string.is.empty")
            .expect("the first yielded String reaches the action");
        let literal = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("the second suspension materializes its String literal");
        let second_action = main
            .rfind("call i1 @topal.runtime.string.is.empty")
            .expect("the second yielded String reaches the action");
        assert!(
            input < suspended
                && suspended < first_action
                && first_action < literal
                && literal < second_action
        );
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"text\""));
        assert!(llvm.contains("name: \"Generator String Unit Unit\""));
        assert!(llvm.contains("name: \"String\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_custom_generator_final_string_after_resumption() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-FINAL-STRING-001
        let source = include_str!("../../../examples/language/custom-generator-string-return.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-string-return.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert_eq!(
            main.matches("call i1 @topal.runtime.string.is.empty")
                .count(),
            1
        );
        let input = main
            .find("call ptr @topal.runtime.string.make")
            .expect("generator application evaluates its String input");
        let suspended = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private suspended token");
        let action = main
            .find("call i1 @topal.runtime.string.is.empty")
            .expect("the yielded String reaches the action");
        let result = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("traversal materializes the distinct final String");
        let output = main
            .find("call void @topal.runtime.string.print")
            .expect("the final String is observed as the program result");
        assert!(input < suspended && suspended < action && action < result && result < output);
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"text\""));
        assert!(llvm.contains("name: \"Generator String Unit String\""));
        assert!(llvm.contains("name: \"String\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_discarded_computation_between_string_yields() {
        // TOPAL-GENERATOR-BODY-STATEMENT-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-RESUME-DISCARD-001
        let source =
            include_str!("../../../examples/language/custom-generator-discard-between-yields.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-discard-between-yields.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        let predicates = main
            .match_indices("call i1 @topal.runtime.string.is.empty")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert_eq!(predicates.len(), 3);
        let input = main
            .find("call ptr @topal.runtime.string.make")
            .expect("generator application evaluates its String input");
        let suspended = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private suspended token");
        let literal = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("the second suspension materializes its String literal");
        assert!(
            input < suspended
                && suspended < predicates[0]
                && predicates[0] < predicates[1]
                && predicates[1] < literal
                && literal < predicates[2]
        );
        assert!(main.contains("#dbg_value(ptr"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"text\""));
        assert!(llvm.contains("name: \"Generator String Unit Unit\""));
        assert!(llvm.contains("name: \"String\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_explicit_string_return_without_invoking_the_action() {
        // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-FINAL-RETURN-001,
        // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-EXPLICIT-RETURN-001
        let source = include_str!("../../../examples/language/custom-generator-explicit-return.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-explicit-return.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert_eq!(
            main.matches("call i1 @topal.runtime.string.is.empty")
                .count(),
            0
        );
        let input = main
            .find("call ptr @topal.runtime.string.make")
            .expect("generator application evaluates its String input");
        let completed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private completed token");
        let result = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("traversal materializes the explicit String return");
        let output = main
            .find("call void @topal.runtime.string.print")
            .expect("the explicit return becomes the program result");
        assert!(input < completed && completed < result && result < output);
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(!llvm.contains("DILocalVariable(name: \"text\""));
        assert!(llvm.contains("name: \"Generator String Unit String\""));
        assert!(llvm.contains("name: \"String\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_explicit_string_return_after_yield_action_and_resumption() {
        // TOPAL-GENERATOR-EXPLICIT-RETURN-001, TOPAL-GENERATOR-RESUMPTION-001,
        // TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-RETURN-AFTER-YIELD-001
        let source =
            include_str!("../../../examples/language/custom-generator-return-after-yield.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-return-after-yield.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert_eq!(
            main.matches("call i1 @topal.runtime.string.is.empty")
                .count(),
            1
        );
        let input = main
            .find("call ptr @topal.runtime.string.make")
            .expect("generator application evaluates its String input");
        let suspended = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private suspended token");
        let action = main
            .find("call i1 @topal.runtime.string.is.empty")
            .expect("foreach invokes the String action");
        let result = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("resumed generator materializes its explicit String return");
        let output = main
            .find("call void @topal.runtime.string.print")
            .expect("the explicit return becomes the program result");
        assert!(input < suspended && suspended < action && action < result && result < output);
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"text\""));
        assert!(llvm.contains("name: \"Generator String Unit String\""));
        assert!(llvm.contains("name: \"String\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_boolean_values_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-BOOLEAN-001
        let source = include_str!("../../../examples/language/custom-generator-boolean-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-boolean-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(main.matches("xor i1 true, true").count(), 2);
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let yielded = main
            .find("store i1 true")
            .expect("foreach retains the yielded Boolean for debugging");
        let negations = main
            .match_indices("xor i1 true, true")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let output = main
            .find("br i1 %")
            .expect("the final Boolean controls Topal-owned display");
        assert!(
            constructed < yielded
                && yielded < negations[0]
                && negations[0] < negations[1]
                && negations[1] < output
        );
        assert!(main.contains("alloca i1, align 1"));
        assert_eq!(main.matches("store i1 true").count(), 2);
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(main.contains("#dbg_value(i1 true"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\""));
        assert!(llvm.contains("name: \"Generator Boolean Unit Boolean\""));
        assert!(llvm.contains("name: \"Boolean\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_custom_generator_final_boolean_decision_after_resumption() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-DECISION-BOOLEAN-001,
        // TOPAL-COMPILER-GENERATOR-FINAL-DECISION-001
        let source = include_str!("../../../examples/language/custom-generator-final-decision.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-final-decision.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let yielded = main
            .find("store i1 true")
            .expect("foreach retains the yielded Boolean for debugging");
        let action = main
            .find("xor i1 true, true")
            .expect("foreach action executes before the generator final decision");
        let decision = main
            .find("br i1 true, label %decision.true")
            .expect("final result retains its O0 Boolean decision");
        let selected = main
            .find("phi ptr")
            .expect("the final decision joins its two String actions");
        let output = main
            .find("call void @topal.runtime.string.print")
            .expect("the selected final String reaches Topal-owned display");
        assert!(
            constructed < yielded
                && yielded < action
                && action < decision
                && decision < selected
                && selected < output
        );
        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert!(main.contains("decision.true"));
        assert!(main.contains("decision.false"));
        assert!(main.contains("decision.merge"));
        assert!(main.contains("alloca i1, align 1"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(main.contains("#dbg_value(i1 true"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\""));
        assert!(llvm.contains("name: \"Generator Boolean Unit String\""));
        assert!(llvm.contains("name: \"Boolean\""));
        assert!(llvm.contains("name: \"String\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_generator_local_enum_and_direct_function_after_resumption() {
        // TOPAL-GENERATOR-LOCAL-FUNCTION-001, TOPAL-GENERATOR-LOCAL-ENUM-001,
        // TOPAL-GENERATOR-SUSPEND-001, TOPAL-FUNCTION-ORDINARY-001,
        // TOPAL-COMPILER-GENERATOR-LOCAL-FUNCTION-001
        let source = include_str!("../../../examples/language/custom-generator-local-function.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-local-function.t").emit();
        let local = llvm
            .split_once("define internal fastcc ptr @topal.fn.label.0")
            .expect("module contains the private generator-local function")
            .1
            .split_once("\n}\n")
            .expect("local function has a complete definition")
            .0;
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert!(local.contains("(i32 %arg0) nounwind noinline"));
        assert!(local.contains("#dbg_value(i32 %arg0"));
        assert!(local.contains("switch i32 %arg0"));
        assert!(local.contains("i32 0, label %enum.decision.alternative.0"));
        assert!(local.contains("i32 1, label %enum.decision.alternative.1"));
        assert_eq!(
            local.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert!(local.contains("phi ptr"));
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let yielded = main
            .find("store i1 true")
            .expect("foreach retains the yielded Boolean for debugging");
        let action = main
            .find("xor i1 true, true")
            .expect("foreach action executes before resumption");
        let call = main
            .find("call fastcc ptr @topal.fn.label.0(i32 0)")
            .expect("resumption invokes the retained local function directly");
        let output = main
            .find("call void @topal.runtime.string.print")
            .expect("the local function result reaches Topal-owned display");
        assert!(constructed < yielded && yielded < action && action < call && call < output);
        assert!(llvm.contains("DISubprogram(name: \"label\""));
        assert!(llvm.contains("linkageName: \"topal.fn.label.0\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\", arg: 1"));
        assert!(llvm.contains("name: \"Choice\""));
        assert!(llvm.contains("name: \"Accepted\""));
        assert!(llvm.contains("name: \"Rejected\""));
        assert!(llvm.contains("name: \"Generator Boolean Unit String\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
        assert!(!llvm.contains("define fastcc ptr @topal.fn.label.0"));
    }

    #[test]
    fn emits_arbitrary_precision_ints_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-INT-001
        let source = include_str!("../../../examples/language/custom-generator-int-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-int-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(main.matches("call ptr @topal.runtime.int.add(").count(), 2);
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store ptr @.topal.int.0")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let additions = main
            .match_indices("call ptr @topal.runtime.int.add(")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let output = main
            .find("call void @topal.runtime.int.print")
            .expect("the final exact Int controls Topal-owned display");
        assert_eq!(debug_stores.len(), 2);
        assert!(
            constructed < debug_stores[0]
                && debug_stores[0] < additions[0]
                && additions[0] < debug_stores[1]
                && debug_stores[1] < additions[1]
                && additions[1] < output
        );
        assert!(main.contains("alloca ptr, align 8"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\""));
        assert!(llvm.contains("name: \"Generator Int Unit Int\""));
        assert!(llvm.contains("name: \"Int\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_nonnegative_nats_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-NUM-NAT-CONSTRUCT-001, TOPAL-COMPILER-GENERATOR-NAT-001
        let source = include_str!("../../../examples/language/custom-generator-nat-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-nat-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(main.matches("call ptr @topal.runtime.int.add(").count(), 2);
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store ptr @.topal.int.0")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let additions = main
            .match_indices("call ptr @topal.runtime.int.add(")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let output = main
            .find("call void @topal.runtime.int.print")
            .expect("the final exact Nat controls Topal-owned display");
        assert_eq!(debug_stores.len(), 2);
        assert!(
            constructed < debug_stores[0]
                && debug_stores[0] < additions[0]
                && additions[0] < debug_stores[1]
                && debug_stores[1] < additions[1]
                && additions[1] < output
        );
        assert!(main.contains("alloca ptr, align 8"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(!main.contains("@topal.runtime.int.try.to.nat"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\""));
        assert!(llvm.contains("name: \"Generator Nat Unit Nat\""));
        assert!(llvm.contains("name: \"Nat\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_nominal_enum_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-ENUM-001, TOPAL-COMPILER-GENERATOR-ENUM-001
        let source = include_str!("../../../examples/language/custom-generator-enum-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-enum-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store i32 0")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let action = main
            .find("icmp eq i32 0, 0")
            .expect("foreach compares the yielded nominal alternative");
        let output = main
            .find("switch i32 1")
            .expect("the distinct final alternative controls Topal-owned display");
        assert_eq!(debug_stores.len(), 4);
        assert!(
            constructed < debug_stores[0]
                && debug_stores[0] < debug_stores[1]
                && debug_stores[1] < action
                && action < debug_stores[2]
                && debug_stores[2] < debug_stores[3]
                && debug_stores[3] < output
        );
        assert!(main.contains("alloca i32, align 4"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"choice\""));
        assert!(llvm.contains("name: \"Generator Choice Unit Choice\""));
        assert!(llvm.contains("name: \"Choice\""));
        assert!(llvm.contains("DIEnumerator(name: \"First\", value: 0)"));
        assert!(llvm.contains("DIEnumerator(name: \"Second\", value: 1)"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_ordered_product_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-PRODUCT-001, TOPAL-COMPILER-GENERATOR-PRODUCT-001
        let source = include_str!("../../../examples/language/custom-generator-product-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-product-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store { ptr, ptr }")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let int_equal = main
            .find("call i32 @topal.runtime.int.compare")
            .expect("foreach compares the yielded Int field");
        let string_equal = main
            .find("call i1 @topal.runtime.string.equal")
            .expect("foreach compares the yielded String field");
        let output = main
            .find("call void @topal.runtime.int.print")
            .expect("the distinct final product controls Topal-owned display");
        assert_eq!(debug_stores.len(), 4);
        assert_eq!(main.matches("insertvalue { ptr, ptr }").count(), 4);
        assert_eq!(main.matches("alloca { ptr, ptr }, align 8").count(), 2);
        assert!(
            constructed < debug_stores[0]
                && debug_stores[0] < debug_stores[1]
                && debug_stores[1] < int_equal
                && int_equal < string_equal
                && string_equal < debug_stores[2]
                && debug_stores[2] < debug_stores[3]
                && debug_stores[3] < output
        );
        assert!(main.contains("and i1"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\""));
        assert!(llvm.contains("name: \"Generator (Int, String) Unit (Int, String)\""));
        assert!(llvm.contains("name: \"(Int, String)\""));
        assert!(llvm.contains("name: \"_0\""));
        assert!(llvm.contains("name: \"_1\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_result_rational_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RESULT-001
        let source = include_str!("../../../examples/language/custom-generator-result-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-result-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.rational.from.int")
                .count(),
            3
        );
        assert_eq!(
            main.matches("call ptr @topal.runtime.result.success")
                .count(),
            1
        );
        assert_eq!(
            main.matches("call ptr @topal.runtime.rational.try.divide")
                .count(),
            1
        );
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store ptr")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let division = main
            .find("call ptr @topal.runtime.rational.try.divide")
            .expect("the final Result performs the checked division");
        assert_eq!(debug_stores.len(), 4);
        assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
        assert!(
            constructed < debug_stores[0]
                && debug_stores[0] < debug_stores[1]
                && debug_stores[1] < debug_stores[2]
                && debug_stores[2] < debug_stores[3]
                && debug_stores[3] < division
        );
        assert!(main.contains("@topal.runtime.result.is.error"));
        assert!(main.contains("@topal.runtime.error.print"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
        assert!(llvm.contains("name: \"Generator Result (Rational, lang arithmetic ArithmeticErrorCode) Unit Result (Rational, lang arithmetic ArithmeticErrorCode)\""));
        assert!(llvm.contains("name: \"Result (Rational, lang arithmetic ArithmeticErrorCode)\""));
        assert!(!main.contains("result.project"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_comparison_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-DECISION-COMPARISON-001, TOPAL-COMPILER-GENERATOR-COMPARISON-001
        let source =
            include_str!("../../../examples/language/custom-generator-comparison-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-comparison-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call i32 @topal.runtime.int.compare").count(),
            3
        );
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store i32")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let final_comparison = main
            .rfind("call i32 @topal.runtime.int.compare")
            .expect("the final Comparison is evaluated after resumption");
        assert_eq!(debug_stores.len(), 4);
        assert_eq!(main.matches("alloca i32, align 4").count(), 2);
        assert!(
            constructed < debug_stores[0]
                && debug_stores[0] < debug_stores[1]
                && debug_stores[1] < debug_stores[2]
                && debug_stores[2] < debug_stores[3]
                && debug_stores[3] < final_comparison
        );
        assert!(main.contains("icmp eq i32"));
        assert!(main.contains("switch i32"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"comparison\""));
        assert!(llvm.contains("name: \"Generator Comparison Unit Comparison\""));
        assert!(llvm.contains("name: \"Comparison\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_exact_rationals_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-COMPILER-GENERATOR-RATIONAL-001
        let source = include_str!("../../../examples/language/custom-generator-rational-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-rational-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.rational.make(")
                .count(),
            3
        );
        assert_eq!(
            main.matches("call ptr @topal.runtime.rational.add(")
                .count(),
            2
        );
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store ptr %v0")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let rational_constructions = main
            .match_indices("call ptr @topal.runtime.rational.make(")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let additions = main
            .match_indices("call ptr @topal.runtime.rational.add(")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let output = main
            .find("call void @topal.runtime.rational.print")
            .expect("the final exact Rational controls Topal-owned display");
        assert_eq!(debug_stores.len(), 2);
        assert!(
            rational_constructions[0] < constructed
                && constructed < debug_stores[0]
                && debug_stores[0] < rational_constructions[1]
                && rational_constructions[1] < additions[0]
                && additions[0] < debug_stores[1]
                && debug_stores[1] < rational_constructions[2]
                && rational_constructions[2] < additions[1]
                && additions[1] < output
        );
        assert!(main.contains("alloca ptr, align 8"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"value\""));
        assert!(llvm.contains("name: \"Generator Rational Unit Rational\""));
        assert!(llvm.contains("name: \"Rational\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_unit_across_every_custom_generator_direction() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-UNIT-001
        let source = include_str!("../../../examples/language/custom-generator-unit-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-unit-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let slots = main
            .match_indices("alloca i8, align 1")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let stores = main
            .match_indices("store i8 0")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let output = main
            .find("call void @topal.platform.write_all")
            .expect("the final Unit controls Topal-owned display");
        assert_eq!(slots.len(), 2);
        assert_eq!(stores.len(), 4);
        assert!(
            constructed < slots[0]
                && slots[0] < stores[0]
                && stores[0] < stores[1]
                && stores[1] < slots[1]
                && slots[1] < stores[2]
                && stores[2] < stores[3]
                && stores[3] < output
        );
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"signal\""));
        assert!(llvm.contains("name: \"Generator Unit Unit Unit\""));
        assert!(llvm.contains("name: \"Unit\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("topal.platform.allocate"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_optional_int_values_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-OPTIONAL-BOUNDARY-001, TOPAL-COMPILER-GENERATOR-OPTIONAL-001
        let source = include_str!("../../../examples/language/custom-generator-optional-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-optional-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.optional.some(")
                .count(),
            2
        );
        assert_eq!(
            main.matches("call i1 @topal.runtime.optional.int.equal(")
                .count(),
            1
        );
        assert_eq!(
            main.matches("call ptr @topal.runtime.optional.none()")
                .count(),
            1
        );
        let initial = main
            .find("call ptr @topal.runtime.optional.some(")
            .expect("generator application evaluates its Optional input");
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store ptr %v0")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let action_some = main
            .match_indices("call ptr @topal.runtime.optional.some(")
            .nth(1)
            .expect("foreach constructs the canonical Optional action operand")
            .0;
        let action = main
            .find("call i1 @topal.runtime.optional.int.equal(")
            .expect("foreach compares the yielded Optional value");
        let result = main
            .find("call ptr @topal.runtime.optional.none()")
            .expect("generator materializes its final Optional alternative");
        let output = main
            .find("call i1 @topal.runtime.optional.is.some(")
            .expect("the final Optional controls Topal-owned display");
        assert_eq!(debug_stores.len(), 2);
        assert!(
            initial < constructed
                && constructed < debug_stores[0]
                && debug_stores[0] < action_some
                && action_some < action
                && action < debug_stores[1]
                && debug_stores[1] < result
                && result < output
        );
        assert!(main.contains("alloca ptr, align 8"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
        assert!(llvm.contains("name: \"Generator Optional Int Unit Optional Int\""));
        assert!(llvm.contains("name: \"Optional Int\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_nested_optional_product_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
        // TOPAL-COMPILER-GENERATOR-NESTED-OPTIONAL-001
        let source =
            include_str!("../../../examples/language/custom-generator-nested-optional-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-nested-optional-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let boxes = main
            .match_indices("call ptr @topal.platform.allocate(i64 16)")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let optional_values = main
            .match_indices("call ptr @topal.runtime.optional.some(")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert_eq!(boxes.len(), 3);
        assert_eq!(optional_values.len(), 3);
        assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let action = main
            .find("optional.product.equal.payload")
            .expect("foreach structurally compares the Optional product payload");
        let output = main
            .rfind("call i1 @topal.runtime.optional.is.some(")
            .expect("the final Optional controls Topal-owned display");
        assert!(
            boxes[0] < optional_values[0]
                && optional_values[0] < constructed
                && constructed < boxes[1]
                && boxes[1] < optional_values[1]
                && optional_values[1] < action
                && action < boxes[2]
                && boxes[2] < optional_values[2]
                && optional_values[2] < output
        );
        assert!(main.contains("call i32 @topal.runtime.int.compare"));
        assert!(main.contains("call i1 @topal.runtime.string.equal"));
        assert!(main.contains("phi i1"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
        assert!(
            llvm.contains("name: \"Generator Optional (Int, String) Unit Optional (Int, String)\"")
        );
        assert!(llvm.contains("name: \"Optional (Int, String)\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_nested_absent_optional_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001, TOPAL-TYPE-PRODUCT-001,
        // TOPAL-COMPILER-GENERATOR-NESTED-NONE-001
        let source =
            include_str!("../../../examples/language/custom-generator-nested-none-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-nested-none-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let absent = main
            .match_indices("call ptr @topal.runtime.optional.none()")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert_eq!(absent.len(), 3);
        assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
        assert!(!main.contains("call ptr @topal.platform.allocate(i64 16)"));
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let action = main
            .find("optional.product.equal.payload")
            .expect("foreach retains the guarded structural equality path");
        let output = main
            .rfind("call i1 @topal.runtime.optional.is.some(")
            .expect("the final absent Optional controls Topal-owned display");
        assert!(
            absent[0] < constructed
                && constructed < absent[1]
                && absent[1] < action
                && action < absent[2]
                && absent[2] < output
        );
        assert!(main.contains("and i1"));
        assert!(main.contains("optional.product.equal.tag"));
        assert!(main.contains("phi i1"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
        assert!(
            llvm.contains("name: \"Generator Optional (Int, String) Unit Optional (Int, String)\"")
        );
        assert!(llvm.contains("name: \"Optional (Int, String)\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_nested_result_product_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-RESULT-001, TOPAL-TYPE-PRODUCT-001,
        // TOPAL-COMPILER-GENERATOR-NESTED-RESULT-001
        let source =
            include_str!("../../../examples/language/custom-generator-nested-result-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-nested-result-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let boxes = main
            .match_indices("call ptr @topal.platform.allocate(i64 16)")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let results = main
            .match_indices("call ptr @topal.runtime.result.success(")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert_eq!(boxes.len(), 3);
        assert_eq!(results.len(), 3);
        assert_eq!(main.matches("alloca ptr, align 8").count(), 2);
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let action = main
            .find("result.product.equal.payload")
            .expect("foreach structurally compares the successful product payload");
        let output = main
            .rfind("call i1 @topal.runtime.result.is.error(")
            .expect("the final Result controls Topal-owned display");
        assert!(
            boxes[0] < results[0]
                && results[0] < constructed
                && constructed < boxes[1]
                && boxes[1] < results[1]
                && results[1] < action
                && action < boxes[2]
                && boxes[2] < results[2]
                && results[2] < output
        );
        assert!(main.contains("call i32 @topal.runtime.int.compare"));
        assert!(main.contains("call i1 @topal.runtime.string.equal"));
        assert!(main.contains("phi i1"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
        assert!(llvm.contains(
            "name: \"Generator Result ((Int, String), lang arithmetic ArithmeticErrorCode) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)\""
        ));
        assert!(
            llvm.contains("name: \"Result ((Int, String), lang arithmetic ArithmeticErrorCode)\"")
        );
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_recursive_nominal_values_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-TYPE-ENUM-001, TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
        // TOPAL-TYPE-RESULT-001, TOPAL-COMPILER-GENERATOR-RECURSIVE-NOMINAL-001
        let source =
            include_str!("../../../examples/language/custom-generator-recursive-nominal-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-recursive-nominal-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let boxes = main
            .match_indices("call ptr @topal.platform.allocate(i64 4)")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert_eq!(boxes.len(), 6);
        assert_eq!(main.matches("store i32 0").count(), 4);
        assert_eq!(main.matches("store i32 1").count(), 2);
        assert_eq!(main.matches("alloca { ptr, ptr }, align 8").count(), 2);
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let optional_action = main
            .find("optional.product.equal.payload")
            .expect("Optional equality observes the nominal payload only after its tag");
        let result_action = main
            .find("result.product.equal.payload")
            .expect("Result equality observes the nominal success only after its tag");
        let output = main
            .rfind("call i1 @topal.runtime.optional.is.some(")
            .expect("the final recursive value controls Topal-owned display");
        assert!(
            boxes[0] < boxes[1]
                && boxes[1] < constructed
                && constructed < boxes[2]
                && boxes[2] < boxes[3]
                && boxes[3] < optional_action
                && optional_action < result_action
                && result_action < boxes[4]
                && boxes[4] < boxes[5]
                && boxes[5] < output
        );
        assert!(main.contains("optional.product.equal.tag"));
        assert!(main.contains("result.product.equal.error"));
        assert!(main.matches("load i32, ptr").count() >= 6);
        assert_eq!(main.matches("icmp eq i32").count(), 2);
        assert!(main.contains("phi i1"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"candidate\""));
        assert!(llvm.contains(
            "name: \"Generator (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)) Unit (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\""
        ));
        assert!(llvm.contains(
            "name: \"(Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))\""
        ));
        assert!(llvm.contains("name: \"Optional Choice\""));
        assert!(llvm.contains("name: \"Result (Choice, lang arithmetic ArithmeticErrorCode)\""));
        assert!(llvm.contains("name: \"Choice\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_int_ranges_across_custom_generator_directions() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-RANGE-BOUNDS-001, TOPAL-RANGE-CLASSIFIER-001,
        // TOPAL-COMPILER-GENERATOR-RANGE-001
        let source = include_str!("../../../examples/language/custom-generator-range-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-range-values.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.range.make(").count(),
            2
        );
        assert_eq!(
            main.matches("call i1 @topal.runtime.range.int.contains(")
                .count(),
            1
        );
        assert_eq!(
            main.matches("call ptr @topal.runtime.range.int.intersection(")
                .count(),
            1
        );
        let initial = main
            .find("call ptr @topal.runtime.range.make(")
            .expect("generator application evaluates its Range input");
        let constructed = main
            .find("#dbg_value(i32 0")
            .expect("generator application retains its private token");
        let debug_stores = main
            .match_indices("store ptr %v0")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let action = main
            .find("call i1 @topal.runtime.range.int.contains(")
            .expect("foreach evaluates membership over the yielded Range");
        let final_range = main
            .match_indices("call ptr @topal.runtime.range.make(")
            .nth(1)
            .expect("generator constructs its exact final intersection operand")
            .0;
        let intersection = main
            .find("call ptr @topal.runtime.range.int.intersection(")
            .expect("generator computes its final narrowed Range");
        let output = main
            .find("call void @topal.runtime.range.int.print(")
            .expect("the final Range controls Topal-owned display");
        assert_eq!(debug_stores.len(), 2);
        assert!(
            initial < constructed
                && constructed < debug_stores[0]
                && debug_stores[0] < action
                && action < debug_stores[1]
                && debug_stores[1] < final_range
                && final_range < intersection
                && intersection < output
        );
        assert!(main.contains("alloca ptr, align 8"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("DILocalVariable(name: \"interval\""));
        assert!(llvm.contains("name: \"Generator Range Int Unit Range Int\""));
        assert!(llvm.contains("name: \"Range Int\""));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_multiple_yield_custom_generator_in_source_order() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-MULTIPLE-YIELD-001
        let source = include_str!("../../../examples/language/custom-multiple-yield-generator.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-multiple-yield-generator.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            3
        );
        assert_eq!(main.matches("store ptr").count(), 2);
        assert!(main.contains("#dbg_value(i32 0"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_custom_generator_early_unit_return_without_action() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-EARLY-RETURN-001,
        // TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-EARLY-RETURN-001
        let source = include_str!("../../../examples/language/custom-generator-early-return.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-early-return.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(main.contains("#dbg_value(i32 0"));
        assert!(!main.contains("#dbg_declare(ptr"));
        assert!(!main.contains("store ptr"));
        assert!(!llvm.contains("DILocalVariable(name: \"character\""));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_distinct_custom_generator_final_character_after_action() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-FINAL-RETURN-001,
        // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001
        let source = include_str!("../../../examples/language/custom-generator-final-character.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-final-character.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            3
        );
        let action_store = main.find("store ptr").expect("yield action is invoked");
        let final_value = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("final Character is materialized");
        assert!(action_store < final_value);
        assert!(main.contains("#dbg_value(i32 0"));
        assert_eq!(main.matches("#dbg_declare(ptr").count(), 1);
        assert_eq!(main.matches("store ptr").count(), 1);
        assert!(llvm.contains("name: \"Generator Character Unit Character\""));
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_custom_generator_local_binding_in_lexical_debug_scope() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-LOCAL-BINDING-001,
        // TOPAL-GENERATOR-SUSPEND-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-LOCAL-BINDING-001
        let source = include_str!("../../../examples/language/custom-generator-local-binding.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-local-binding.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert_eq!(main.matches("store ptr").count(), 2);
        assert!(main.contains("#dbg_value(i32 0"));
        assert_eq!(main.matches("#dbg_declare(ptr").count(), 2);
        assert!(llvm.contains("DILocalVariable(name: \"copy\""));
        assert!(llvm.contains("DILexicalBlock("));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_post_resume_local_before_the_next_custom_yield() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-BODY-STATEMENT-001,
        // TOPAL-GENERATOR-LOCAL-BINDING-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FOREACH-001, TOPAL-COMPILER-GENERATOR-SUSPENSION-001
        let source = include_str!("../../../examples/language/custom-generator-suspension.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-suspension.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let materializations = main
            .match_indices("call ptr @topal.runtime.string.make")
            .map(|(position, _)| position)
            .collect::<Vec<_>>();
        let local_declaration = main
            .match_indices("#dbg_declare(ptr")
            .nth(1)
            .expect("post-resume local has a distinct debug declaration")
            .0;
        let first_action = main
            .find("store ptr")
            .expect("first yield action is invoked");

        assert_eq!(materializations.len(), 3);
        assert!(materializations[1] < first_action);
        assert!(first_action < local_declaration);
        assert!(local_declaration < materializations[2]);
        assert_eq!(main.matches("#dbg_declare(ptr").count(), 2);
        assert_eq!(main.matches("store ptr").count(), 3);
        assert!(llvm.contains("DILocalVariable(name: \"copy\""));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(llvm.contains("name: \"Character\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_unit_resume_binding_after_the_custom_action() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-RESUME-BINDING-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001
        let source = include_str!("../../../examples/language/custom-generator-resume-binding.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-resume-binding.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let action_store = main.find("store ptr").expect("yield action is invoked");
        let resumed_store = main
            .find("store i8 0")
            .expect("successful Unit resume has debug storage");

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            2
        );
        assert!(action_store < resumed_store);
        assert_eq!(main.matches("#dbg_declare(ptr").count(), 2);
        assert_eq!(main.matches("store ptr").count(), 1);
        assert_eq!(main.matches("store i8 0").count(), 1);
        assert!(llvm.contains("DILocalVariable(name: \"resumed\""));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(llvm.contains("name: \"Unit\""));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_function_local_custom_generator_close_without_runtime_state() {
        // TOPAL-GENERATOR-DECLARATION-001, TOPAL-GENERATOR-CLOSE-001,
        // TOPAL-GENERATOR-ERROR-CODE-001, TOPAL-COMPILER-GENERATOR-CLOSE-001
        let source = include_str!("../../../examples/language/custom-generator-close.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "abandon")
            .expect("called abandon function is instantiated");
        let llvm = Generator::new(&program, "custom-generator-close.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let close = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(ptr %arg0)",
                function.symbol
            ))
            .expect("close function has one private Character parameter")
            .1
            .split_once("}\n")
            .expect("close function definition terminates")
            .0;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(main.contains(&format!("call fastcc void @{}(ptr", function.symbol)));
        assert!(close.contains("#dbg_value(i32 0"));
        assert!(close.contains("ret void"));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(!llvm.contains("TopalGeneratorErrorHeader"));
        assert!(!llvm.contains("Result (Unit, lang generator GeneratorErrorCode)"));
        assert!(!close.contains("topal.runtime.string.make"));
        assert!(!close.contains("topal.runtime.generator"));
        assert!(!close.contains("call ptr %"));
    }

    #[test]
    fn emits_function_local_custom_generator_close_handler() {
        // TOPAL-GENERATOR-CLOSE-001, TOPAL-GENERATOR-CLOSE-HANDLER-001,
        // TOPAL-GENERATOR-ERROR-CODE-001,
        // TOPAL-COMPILER-GENERATOR-CLOSE-HANDLER-001
        let source = include_str!("../../../examples/language/custom-generator-close-handler.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "abandon")
            .expect("called abandon function is instantiated");
        let llvm = Generator::new(&program, "custom-generator-close-handler.t").emit();
        let close = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(ptr %arg0)",
                function.symbol
            ))
            .expect("close function has one private Character parameter")
            .1
            .split_once("}\n")
            .expect("close function definition terminates")
            .0;

        let failure = close
            .find("call ptr @topal.runtime.result.failure(i32 0")
            .expect("close materializes the intrinsic failure Result");
        let payload = close
            .find("call ptr @topal.runtime.result.payload")
            .expect("Error handler selects the failure payload");
        let code = close
            .find("call i32 @topal.runtime.error.code")
            .expect("the selected Error remains observable to GDB");
        let returned = close.find("ret void").expect("handler completes with Unit");
        assert!(failure < payload && payload < code && code < returned);
        assert_eq!(
            close
                .matches("call ptr @topal.runtime.result.failure(i32 0")
                .count(),
            1
        );
        assert!(close.contains("#dbg_declare(ptr"));
        assert!(llvm.contains("DILocalVariable(name: \"resume-result\""));
        assert!(llvm.contains("DILocalVariable(name: \"problem\""));
        assert!(llvm.contains("name: \"lang generator GeneratorErrorCode\""));
        assert!(llvm.contains("DIEnumerator(name: \"generator-closed\", value: 0)"));
        assert!(llvm.contains("name: \"Result (Unit, lang generator GeneratorErrorCode)\""));
        assert!(llvm.contains("name: \"Error (lang generator GeneratorErrorCode)\""));
        assert!(!close.contains("topal.runtime.result.is.error"));
        assert!(!close.contains("topal.runtime.generator"));
        assert!(!close.contains("call ptr %"));
    }

    #[test]
    fn emits_generator_local_function_during_custom_close() {
        // TOPAL-GENERATOR-LOCAL-FUNCTION-001, TOPAL-GENERATOR-LOCAL-ENUM-001,
        // TOPAL-GENERATOR-CLOSE-001, TOPAL-GENERATOR-CLOSE-HANDLER-001,
        // TOPAL-COMPILER-GENERATOR-LOCAL-CLOSE-001
        let source =
            include_str!("../../../examples/language/custom-generator-local-close-handler.t");
        let program = analyze_for_compiler(source).unwrap();
        let cleanup = program
            .functions
            .iter()
            .find(|function| function.source_name == "cleanup")
            .expect("generator-local cleanup function is retained");
        let abandon = program
            .functions
            .iter()
            .find(|function| function.source_name == "abandon")
            .expect("calling abandon function is instantiated");
        let llvm = Generator::new(&program, "custom-generator-local-close-handler.t").emit();
        let local = llvm
            .split_once(&format!("define internal fastcc void @{}", cleanup.symbol))
            .expect("module contains the private generator-local cleanup function")
            .1
            .split_once("\n}\n")
            .expect("local cleanup function has a complete definition")
            .0;
        let close = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(ptr %arg0)",
                abandon.symbol
            ))
            .expect("abandon function has one private Character parameter")
            .1
            .split_once("}\n")
            .expect("abandon function definition terminates")
            .0;

        assert!(local.contains("(i32 %arg0) nounwind noinline"));
        assert!(local.contains("store i32 %arg0"));
        assert!(local.contains("#dbg_declare(ptr"));
        assert!(local.contains("ret void"));
        let failure = close
            .find("call ptr @topal.runtime.result.failure(i32 0")
            .expect("close materializes the intrinsic failure Result");
        let payload = close
            .find("call ptr @topal.runtime.result.payload")
            .expect("qualified handler observes the failure payload");
        let code = close
            .find("call i32 @topal.runtime.error.code")
            .expect("qualified handler observes the nominal code");
        let call_text = format!("call fastcc void @{}(i32 0)", cleanup.symbol);
        let call = close
            .find(&call_text)
            .expect("close branch directly invokes cleanup Closed");
        let returned = close
            .find("ret void")
            .expect("abandon completes after local cleanup");
        assert!(failure < payload && payload < code && code < call && call < returned);
        assert!(!close.contains(&format!("call fastcc void @{}(i32 1)", cleanup.symbol)));
        assert!(llvm.contains("DISubprogram(name: \"cleanup\""));
        assert!(llvm.contains(&format!("linkageName: \"{}\"", cleanup.symbol)));
        assert!(llvm.contains("DILocalVariable(name: \"choice\", arg: 1"));
        assert!(llvm.contains("name: \"CloseChoice\""));
        assert!(llvm.contains("DIEnumerator(name: \"Closed\", value: 0)"));
        assert!(llvm.contains("DIEnumerator(name: \"Continued\", value: 1)"));
        assert!(!close.contains("topal.runtime.generator"));
        assert!(!close.contains("call ptr %"));
    }

    #[test]
    fn emits_qualified_custom_generator_close_code_pattern() {
        // TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001,
        // TOPAL-GENERATOR-CLOSE-HANDLER-001,
        // TOPAL-COMPILER-GENERATOR-CLOSE-CODE-PATTERN-001
        let source =
            include_str!("../../../examples/language/custom-generator-close-code-pattern.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "abandon")
            .expect("called abandon function is instantiated");
        let llvm = Generator::new(&program, "custom-generator-close-code-pattern.t").emit();
        let close = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(ptr %arg0)",
                function.symbol
            ))
            .expect("close function has one private Character parameter")
            .1
            .split_once("}\n")
            .expect("close function definition terminates")
            .0;

        let failure = close
            .find("call ptr @topal.runtime.result.failure(i32 0")
            .expect("close materializes the intrinsic failure Result");
        let payload = close
            .find("call ptr @topal.runtime.result.payload")
            .expect("qualified handler observes the failure payload");
        let code = close
            .find("call i32 @topal.runtime.error.code")
            .expect("qualified handler observes the nominal code");
        let returned = close.find("ret void").expect("handler completes with Unit");
        assert!(failure < payload && payload < code && code < returned);
        assert_eq!(close.matches("topal.runtime.error.code").count(), 1);
        assert!(llvm.contains("DILocalVariable(name: \"resume-result\""));
        assert!(!llvm.contains("DILocalVariable(name: \"problem\""));
        assert!(llvm.contains("name: \"lang generator GeneratorErrorCode\""));
        assert!(llvm.contains("DIEnumerator(name: \"generator-closed\", value: 0)"));
        assert!(!close.contains("switch i32"));
        assert!(!close.contains("topal.runtime.generator"));
        assert!(!close.contains("call ptr %"));
    }

    #[test]
    fn emits_custom_generator_function_parameter_transfer() {
        // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001
        let source =
            include_str!("../../../examples/language/custom-generator-function-parameter.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("called custom Generator consumer is instantiated");
        let llvm = Generator::new(&program, "custom-generator-function-parameter.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let traversal = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(i32 %arg0)",
                function.symbol
            ))
            .expect("traversal function has one private ownership token")
            .1
            .split_once("}\n")
            .expect("traversal function definition terminates")
            .0;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
        assert_eq!(
            traversal
                .matches("call ptr @topal.runtime.string.make")
                .count(),
            1
        );
        assert!(traversal.contains("alloca i32, align 4"));
        assert!(traversal.contains("store i32 %arg0"));
        assert!(traversal.contains("alloca ptr, align 8"));
        assert!(traversal.contains("#dbg_declare(ptr"));
        assert!(traversal.contains("ret void"));
        assert!(!traversal.contains("generator.foreach.loop"));
        assert!(!traversal.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_custom_generator_character_result_parameter_transfer() {
        // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
        // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001
        let source = include_str!(
            "../../../examples/language/custom-generator-character-return-parameter.t"
        );
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("called result-valued custom Generator consumer is instantiated");
        let llvm = Generator::new(&program, "custom-generator-character-return-parameter.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let traversal = llvm
            .split_once(&format!(
                "define internal fastcc ptr @{}(i32 %arg0)",
                function.symbol
            ))
            .expect("result-valued traversal has one private ownership token")
            .1
            .split_once("}\n")
            .expect("result-valued traversal definition terminates")
            .0;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(main.contains(&format!("call fastcc ptr @{}(i32 0)", function.symbol)));
        assert_eq!(
            traversal
                .matches("call ptr @topal.runtime.string.make")
                .count(),
            2
        );
        let yielded = traversal
            .find("call ptr @topal.runtime.string.make")
            .expect("callee materializes the yielded Character");
        let returned = traversal
            .rfind("call ptr @topal.runtime.string.make")
            .expect("callee materializes the final Character");
        let return_instruction = traversal
            .find("ret ptr")
            .expect("callee returns the final Character descriptor");
        assert!(yielded < returned && returned < return_instruction);
        assert!(traversal.contains("alloca i32, align 4"));
        assert!(traversal.contains("store i32 %arg0"));
        assert!(traversal.contains("alloca ptr, align 8"));
        assert!(traversal.contains("#dbg_declare(ptr"));
        assert!(!traversal.contains("generator.foreach.loop"));
        assert!(!traversal.contains("call ptr %"));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("name: \"Generator Character Unit Character\""));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_custom_generator_parameter_close_without_runtime_state() {
        // TOPAL-GENERATOR-FUNCTION-PARAMETER-001, TOPAL-GENERATOR-CLOSE-001,
        // TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-CLOSE-001
        let source = include_str!("../../../examples/language/custom-generator-parameter-close.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "ignore")
            .expect("called custom Generator closer is instantiated");
        let llvm = Generator::new(&program, "custom-generator-parameter-close.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let close = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(i32 %arg0)",
                function.symbol
            ))
            .expect("close function has one private ownership token")
            .1
            .split_once("}\n")
            .expect("close function definition terminates")
            .0;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
        assert!(close.contains("alloca i32, align 4"));
        assert!(close.contains("store i32 %arg0"));
        assert!(close.contains("#dbg_declare(ptr"));
        assert!(close.contains("ret void"));
        assert!(!close.contains("topal.runtime.string.make"));
        assert!(!close.contains("call "));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(!llvm.contains("TopalGeneratorErrorHeader"));
        assert!(!llvm.contains("Result (Unit, lang generator GeneratorErrorCode)"));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_custom_generator_function_result_transfer() {
        // TOPAL-GENERATOR-FUNCTION-RESULT-001, TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001
        let source = include_str!("../../../examples/language/custom-generator-function-result.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "make")
            .expect("called custom Generator factory is instantiated");
        let llvm = Generator::new(&program, "custom-generator-function-result.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let factory = llvm
            .split_once(&format!(
                "define internal fastcc i32 @{}(ptr %arg0)",
                function.symbol
            ))
            .expect("custom Generator factory has one private Character argument")
            .1
            .split_once("}\n")
            .expect("custom Generator factory definition terminates")
            .0;

        assert!(main.contains(&format!("call fastcc i32 @{}(ptr ", function.symbol)));
        assert!(main.contains("#dbg_value(i32"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(factory.contains("alloca ptr, align 8"));
        assert!(factory.contains("store ptr %arg0"));
        assert!(factory.contains("#dbg_declare(ptr"));
        assert!(factory.contains("ret i32 0"));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_custom_generator_character_result_function_result_transfer() {
        // TOPAL-GENERATOR-FUNCTION-RESULT-001,
        // TOPAL-GENERATOR-FINAL-RETURN-001, TOPAL-GENERATOR-FOREACH-001,
        // TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-001
        let source =
            include_str!("../../../examples/language/custom-generator-character-return-result.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "make")
            .expect("called result-valued custom Generator factory is instantiated");
        let llvm = Generator::new(&program, "custom-generator-character-return-result.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let factory = llvm
            .split_once(&format!(
                "define internal fastcc i32 @{}(ptr %arg0)",
                function.symbol
            ))
            .expect("custom Generator factory has one private Character argument")
            .1
            .split_once("}\n")
            .expect("custom Generator factory definition terminates")
            .0;

        assert_eq!(main.matches("call fastcc i32").count(), 1);
        assert!(main.contains(&format!("call fastcc i32 @{}(ptr ", function.symbol)));
        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            3
        );
        let transfer = main
            .find("call fastcc i32")
            .expect("caller receives the private ownership token");
        let yielded = main[transfer..]
            .find("call ptr @topal.runtime.string.make")
            .map(|offset| transfer + offset)
            .expect("caller materializes the yielded Character after transfer");
        let returned = main
            .rfind("call ptr @topal.runtime.string.make")
            .expect("caller materializes the final Character");
        let output = main
            .find("call void @topal.runtime.string.print")
            .expect("caller prints the final Character");
        assert!(transfer < yielded && yielded < returned && returned < output);
        assert!(main.contains("#dbg_value(i32"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(factory.contains("alloca ptr, align 8"));
        assert!(factory.contains("store ptr %arg0"));
        assert!(factory.contains("#dbg_declare(ptr"));
        assert!(factory.contains("ret i32 0"));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!llvm.contains("call ptr %"));
        assert!(llvm.contains("DILocalVariable(name: \"initial\""));
        assert!(llvm.contains("DILocalVariable(name: \"generated\""));
        assert!(llvm.contains("name: \"Generator Character Unit Character\""));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_transferred_string_character_generator_close_without_runtime_state() {
        // TOPAL-STRING-CHARACTERS-GENERATOR-001,
        // TOPAL-STRING-CHARACTERS-PARAMETER-001,
        // TOPAL-STRING-CHARACTERS-CLOSE-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001
        let source = include_str!("../../../examples/language/string-character-generator-close.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "ignore")
            .expect("called close function is instantiated");
        let llvm = Generator::new(&program, "string-character-generator-close.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let close = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(i32 %arg0)",
                function.symbol
            ))
            .expect("close function has one private ownership token")
            .1
            .split_once("}\n")
            .expect("close function definition terminates")
            .0;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
        assert!(close.contains("alloca i32, align 4"));
        assert!(close.contains("store i32 %arg0"));
        assert!(close.contains("#dbg_declare(ptr"));
        assert!(close.contains("ret void"));
        assert!(!close.contains("call "));
        assert!(llvm.contains("name: \"Generator Character Unit Unit\""));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_specialized_string_character_generator_parameter_traversal() {
        // TOPAL-STRING-CHARACTERS-FOREACH-001,
        // TOPAL-STRING-CHARACTERS-GENERATOR-001,
        // TOPAL-STRING-CHARACTERS-PARAMETER-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001
        let source =
            include_str!("../../../examples/language/string-character-generator-parameter.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("called traversal function is instantiated");
        let llvm = Generator::new(&program, "string-character-generator-parameter.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let traversal = llvm
            .split_once(&format!(
                "define internal fastcc void @{}(i32 %arg0)",
                function.symbol
            ))
            .expect("traversal function has one private ownership token")
            .1
            .split_once("}\n")
            .expect("traversal function definition terminates")
            .0;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            1
        );
        assert!(main.contains(&format!("call fastcc void @{}(i32 0)", function.symbol)));
        assert_eq!(
            traversal
                .matches("call ptr @topal.runtime.string.make")
                .count(),
            3
        );
        assert!(traversal.contains("alloca i32, align 4"));
        assert!(traversal.contains("alloca ptr, align 8"));
        assert!(traversal.contains("#dbg_declare(ptr"));
        assert!(traversal.contains("ret void"));
        assert!(!traversal.contains("generator.foreach.loop"));
        assert!(!traversal.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_specialized_string_character_generator_result_transfer() {
        // TOPAL-STRING-CHARACTERS-FOREACH-001,
        // TOPAL-STRING-CHARACTERS-GENERATOR-001,
        // TOPAL-STRING-CHARACTERS-RESULT-001,
        // TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001
        let source = include_str!("../../../examples/language/string-character-generator-result.t");
        let program = analyze_for_compiler(source).unwrap();
        let function = program
            .functions
            .iter()
            .find(|function| function.source_name == "generate")
            .expect("called generator factory is instantiated");
        let llvm = Generator::new(&program, "string-character-generator-result.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        let factory = llvm
            .split_once(&format!(
                "define internal fastcc i32 @{}(ptr %arg0)",
                function.symbol
            ))
            .expect("generator factory has one private String argument")
            .1
            .split_once("}\n")
            .expect("generator factory definition terminates")
            .0;

        assert_eq!(
            main.matches("call ptr @topal.runtime.string.make").count(),
            4
        );
        assert!(main.contains(&format!("call fastcc i32 @{}(ptr ", function.symbol)));
        assert!(main.contains("#dbg_value(i32"));
        assert!(main.contains("#dbg_declare(ptr"));
        assert!(factory.contains("alloca ptr, align 8"));
        assert!(factory.contains("store ptr %arg0"));
        assert!(factory.contains("#dbg_declare(ptr"));
        assert!(factory.contains("ret i32 0"));
        assert!(!factory.contains("topal.runtime.string.make"));
        assert!(!main.contains("generator.foreach.loop"));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_lazy_unfold_construction_without_invoking_its_step() {
        // TOPAL-GENERATOR-UNFOLD-001,
        // TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001
        let source = include_str!("../../../examples/language/unfold-generator.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "unfold-generator.t").emit();

        assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Generator Int Unit Unit\""));
        assert!(llvm.contains("DIEnumerator(name: \"<Generator Int Unit Unit>\", value: 0)"));
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;
        assert!(!main.contains("topal.runtime.list.int.uncons"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_finite_unfold_collection_as_a_seed_and_list_loop() {
        // TOPAL-GENERATOR-UNFOLD-001, TOPAL-GENERATOR-UNFOLD-COLLECT-001,
        // TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001
        let source = include_str!("../../../examples/language/unfold-collect.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "unfold-collect.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        for expected in [
            "generator.unfold.collect.loop",
            "phi ptr",
            "icmp ne ptr",
            "load ptr, ptr",
            "call ptr @topal.platform.allocate(i64 16)",
            "generator.unfold.collect.done",
        ] {
            assert!(main.contains(expected), "missing {expected:?}: {main}");
        }
        assert!(!main.contains("topal.runtime.list.int.uncons"));
        assert!(!main.contains("topal.runtime.optional"));
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_nominal_enums_as_checked_i32_tags_with_dwarf_enumerators() {
        // TOPAL-COMPILER-ENUM-001
        let source = "use language (version is v0.1)\nColor is Enum (Red, Green, Blue)\nnext is fn (value : Color) -> Color\n  value\n    Red then Green\n    Green then Blue\n    Blue then Red\n(next Red, next Green)\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/enums.t").emit();
        assert!(llvm.contains("define internal fastcc i32 @topal.fn.next"));
        assert!(llvm.contains("switch i32 %arg0"));
        assert!(llvm.contains("phi i32"));
        assert!(llvm.contains("DW_TAG_enumeration_type, name: \"Color\""));
        assert!(llvm.contains("DIEnumerator(name: \"Red\", value: 0)"));
        assert!(llvm.contains("DIEnumerator(name: \"Green\", value: 1)"));
        assert!(llvm.contains("DIEnumerator(name: \"Blue\", value: 2)"));
    }

    #[test]
    fn emits_nominal_sums_as_private_tagged_aggregates_with_dwarf() {
        // TOPAL-COMPILER-SUM-001, TOPAL-TYPE-UNION-001,
        // TOPAL-TYPE-VARIANT-001, TOPAL-DECISION-UNION-001
        let source = include_str!("../../../examples/language/unions-and-recursive-products.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/unions-and-recursive-products.t").emit();
        assert!(llvm.contains("define internal fastcc { ptr, { ptr, ptr } } @topal.fn.describe"));
        assert!(llvm.contains("({ i32, { ptr, { ptr, ptr } } } %arg0)"));
        assert!(llvm.contains("({ i32, ptr, ptr } %arg0)"));
        assert!(llvm.contains("insertvalue { i32, { ptr, { ptr, ptr } } }"));
        assert!(llvm.contains("extractvalue { i32, { ptr, { ptr, ptr } } } %arg0, 0"));
        assert!(llvm.contains("sum.decision.alternative"));
        assert!(llvm.contains("switch i32"));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalUnion.Message\""));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalVariant.Scalar\""));
        assert!(llvm.contains("DIEnumerator(name: \"Move\", value: 1)"));
        assert!(llvm.contains("DIEnumerator(name: \"at 0\", value: 0)"));
    }

    #[test]
    fn emits_modular_values_as_nominal_int_backed_private_values() {
        // TOPAL-COMPILER-MODULAR-001, TOPAL-NUM-MODULAR-ARITHMETIC-001
        let source = "use language (version is v0.1)\nByteCounter is ModNat (0 ..= 255)\nretain is fn (value : ByteCounter) -> ByteCounter\n  value\nstart is ByteCounter 255\nresult is (retain start) + (ByteCounter 1)\nresult\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/modular-values.t").emit();
        assert!(llvm.contains("define internal fastcc ptr @topal.fn.retain"));
        assert!(llvm.contains("call fastcc ptr @topal.fn.retain"));
        assert!(llvm.contains("call ptr @topal.runtime.int.add("));
        assert!(llvm.contains("call ptr @topal.runtime.int.subtract("));
        assert!(llvm.contains("call ptr @topal.runtime.int.modulo("));
        assert!(llvm.contains("DW_TAG_structure_type, name: \"TopalModular.ByteCounter\""));
        assert!(llvm.contains("DW_TAG_typedef, name: \"ByteCounter\""));
    }

    #[test]
    fn emits_dynamic_modular_validation_as_a_native_result() {
        // TOPAL-COMPILER-MODULAR-CONSTRUCTION-001,
        // TOPAL-NUM-MODULAR-CONSTRUCT-001
        let source = include_str!("../../../examples/language/modular-checked-construction.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/modular-checked-construction.t").emit();
        assert!(llvm.contains("modular.accepted"));
        assert!(llvm.contains("modular.rejected"));
        assert!(llvm.contains("modular.merge"));
        assert!(llvm.matches("call i32 @topal.runtime.int.compare").count() >= 2);
        assert!(llvm.contains("call ptr @topal.runtime.result.success"));
        assert!(llvm.contains("call ptr @topal.runtime.result.failure(i32 0"));
        assert!(llvm.contains(&llvm_bytes(b"root.ByteCounter(Int)")));
        assert!(llvm.contains(
            "DW_TAG_typedef, name: \"Result (ByteCounter, lang arithmetic ArithmeticErrorCode)\""
        ));
    }

    #[test]
    fn emits_ordered_custom_generator_overloads_and_typed_results() {
        // TOPAL-GENERATOR-OVERLOAD-001, TOPAL-GENERATOR-FOREACH-RESULT-001,
        // TOPAL-COMPILER-GENERATOR-OVERLOAD-001
        let source = include_str!("../../../examples/language/custom-generator-overloads.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "custom-generator-overloads.t").emit();
        let main = llvm
            .split_once("define internal void @topal.main")
            .expect("module contains generated source entry")
            .1;

        let unary_action = main
            .find("call ptr @topal.runtime.int.add(ptr @.topal.int.0")
            .expect("unary selection executes its Int action");
        let unary_final = main[unary_action..]
            .find("call ptr @topal.runtime.string.make")
            .map(|offset| unary_action + offset)
            .expect("unary traversal constructs its final String after the action");
        let binary_input = main[unary_final + 1..]
            .find("call ptr @topal.runtime.string.make")
            .map(|offset| unary_final + 1 + offset)
            .expect("binary application constructs suffix once");
        let binary_prefix = main[binary_input..]
            .find("call ptr @topal.runtime.int.add")
            .map(|offset| binary_input + offset)
            .expect("binary declaration prefix executes before its yield");
        let binary_action = main[binary_prefix..]
            .find("call i1 @topal.runtime.string.is.empty")
            .map(|offset| binary_prefix + offset)
            .expect("binary String action executes after its prefix and yield");
        let binary_final = main[binary_action..]
            .find("call ptr @topal.runtime.string.make")
            .map(|offset| binary_action + offset)
            .expect("binary traversal constructs its final String last");
        let output = main
            .find("call void @topal.runtime.string.print")
            .expect("both bound results reach Topal-owned output");
        assert!(
            unary_action < unary_final
                && unary_final < binary_input
                && binary_input < binary_prefix
                && binary_prefix < binary_action
                && binary_action < binary_final
                && binary_final < output
        );
        assert_eq!(main.matches("#dbg_value(i32 0").count(), 2);
        for expected in [
            "DILocalVariable(name: \"unary-generated\"",
            "DILocalVariable(name: \"unary-result\"",
            "DILocalVariable(name: \"binary-generated\"",
            "DILocalVariable(name: \"binary-result\"",
            "DILocalVariable(name: \"suffix\"",
            "name: \"Generator Int Unit String\"",
            "name: \"Generator String Unit String\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!main.contains("topal.runtime.generator"));
        assert!(!main.contains("call ptr %"));
    }

    #[test]
    fn emits_generic_custom_generator_function_boundaries() {
        // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
        // TOPAL-GENERATOR-FUNCTION-RESULT-001,
        // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
        // TOPAL-COMPILER-GENERATOR-FUNCTION-BOUNDARY-001
        let source = include_str!(
            "../../../examples/language/custom-generator-generic-function-boundaries.t"
        );
        let program = analyze_for_compiler(source).unwrap();
        let make = program
            .functions
            .iter()
            .find(|function| function.source_name == "make")
            .expect("factory specialization exists");
        let consume = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("consumer specialization exists");
        let llvm =
            Generator::new(&program, "custom-generator-generic-function-boundaries.t").emit();
        assert!(llvm.contains(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            make.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(i32 %arg0)",
            consume.symbol
        )));
        assert!(llvm.contains(&format!("call fastcc i32 @{}(ptr", make.symbol)));
        assert!(llvm.contains(&format!("call fastcc ptr @{}(i32", consume.symbol)));

        let consume_body = llvm
            .split_once(&format!(
                "define internal fastcc ptr @{}(i32 %arg0)",
                consume.symbol
            ))
            .expect("module contains consumer")
            .1
            .split_once("\n}\n")
            .expect("consumer has one body")
            .0;
        let action = consume_body
            .find("call ptr @topal.runtime.int.add")
            .expect("consumer invokes the checked Int action");
        let final_string = consume_body[action..]
            .find("call ptr @topal.runtime.string.make")
            .map(|offset| action + offset)
            .expect("consumer constructs the final String after resumption");
        let returned = consume_body[final_string..]
            .find("ret ptr")
            .map(|offset| final_string + offset)
            .expect("consumer returns the final String");
        assert!(action < final_string && final_string < returned);
        for expected in [
            "name: \"Generator Int Unit String\"",
            "DILocalVariable(name: \"initial\", arg: 1",
            "DILocalVariable(name: \"generated\", arg: 1",
            "DILocalVariable(name: \"value\"",
            "DILocalVariable(name: \"result\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!llvm.contains("topal.runtime.generator"));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("call i32 %"));
    }

    #[test]
    fn emits_compound_custom_generator_function_boundaries() {
        // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
        // TOPAL-GENERATOR-FUNCTION-RESULT-001,
        // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
        // TOPAL-TYPE-PRODUCT-001,
        // TOPAL-COMPILER-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001
        let source = include_str!(
            "../../../examples/language/custom-generator-compound-function-boundaries.t"
        );
        let program = analyze_for_compiler(source).unwrap();
        let make = program
            .functions
            .iter()
            .find(|function| function.source_name == "make")
            .expect("compound factory specialization exists");
        let consume = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("compound consumer specialization exists");
        let llvm =
            Generator::new(&program, "custom-generator-compound-function-boundaries.t").emit();
        assert!(llvm.contains(&format!(
            "define internal fastcc i32 @{}({{ ptr, ptr }} %arg0)",
            make.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc {{ ptr, ptr }} @{}(i32 %arg0)",
            consume.symbol
        )));
        assert!(llvm.contains(&format!("call fastcc i32 @{}({{ ptr, ptr }}", make.symbol)));
        assert!(llvm.contains(&format!(
            "call fastcc {{ ptr, ptr }} @{}(i32",
            consume.symbol
        )));

        let consume_body = llvm
            .split_once(&format!(
                "define internal fastcc {{ ptr, ptr }} @{}(i32 %arg0)",
                consume.symbol
            ))
            .expect("module contains compound consumer")
            .1
            .split_once("\n}\n")
            .expect("compound consumer has one body")
            .0;
        let int_action = consume_body
            .find("call i32 @topal.runtime.int.compare")
            .expect("consumer compares the yielded Int field");
        let string_action = consume_body[int_action..]
            .find("call i1 @topal.runtime.string.equal")
            .map(|offset| int_action + offset)
            .expect("consumer compares the yielded String field after the Int field");
        let final_string = consume_body[string_action..]
            .find("call ptr @topal.runtime.string.make")
            .map(|offset| string_action + offset)
            .expect("consumer constructs the final product after resumption");
        let returned = consume_body[final_string..]
            .find("ret { ptr, ptr }")
            .map(|offset| final_string + offset)
            .expect("consumer returns the final product");
        assert!(
            int_action < string_action && string_action < final_string && final_string < returned
        );
        for expected in [
            "name: \"Generator (Int, String) Unit (Int, String)\"",
            "name: \"(Int, String)\"",
            "name: \"_0\"",
            "name: \"_1\"",
            "DILocalVariable(name: \"initial\", arg: 1",
            "DILocalVariable(name: \"generated\", arg: 1",
            "DILocalVariable(name: \"value\"",
            "DILocalVariable(name: \"result\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!llvm.contains("topal.runtime.generator"));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("call i32 %"));
        assert!(!llvm.contains("call { ptr, ptr } %"));
    }

    #[test]
    fn emits_nested_custom_generator_function_boundaries() {
        // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
        // TOPAL-GENERATOR-FUNCTION-RESULT-001,
        // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
        // TOPAL-TYPE-OPTIONAL-CONSTRUCT-001,
        // TOPAL-TYPE-RESULT-001,
        // TOPAL-TYPE-PRODUCT-001,
        // TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001
        let source = include_str!(
            "../../../examples/language/custom-generator-nested-function-boundaries.t"
        );
        let program = analyze_for_compiler(source).unwrap();
        let make = program
            .functions
            .iter()
            .find(|function| function.source_name == "make")
            .expect("nested factory specialization exists");
        let consume = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("nested consumer specialization exists");
        let llvm = Generator::new(&program, "custom-generator-nested-function-boundaries.t").emit();
        assert!(llvm.contains(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            make.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(i32 %arg0)",
            consume.symbol
        )));
        assert!(llvm.contains(&format!("call fastcc i32 @{}(ptr", make.symbol)));
        assert!(llvm.contains(&format!("call fastcc ptr @{}(i32", consume.symbol)));

        let make_body = llvm
            .split_once(&format!(
                "define internal fastcc i32 @{}(ptr %arg0)",
                make.symbol
            ))
            .expect("module contains nested factory")
            .1
            .split_once("\n}\n")
            .expect("nested factory has one body")
            .0;
        assert!(make_body.contains("alloca ptr, align 8"));
        assert!(make_body.contains("#dbg_declare"));

        let consume_body = llvm
            .split_once(&format!(
                "define internal fastcc ptr @{}(i32 %arg0)",
                consume.symbol
            ))
            .expect("module contains nested consumer")
            .1
            .split_once("\n}\n")
            .expect("nested consumer has one body")
            .0;
        let optional_tags = consume_body
            .find("call i1 @topal.runtime.optional.is.some")
            .expect("consumer checks Optional tags before payload access");
        let int_action = consume_body[optional_tags..]
            .find("call i32 @topal.runtime.int.compare")
            .map(|offset| optional_tags + offset)
            .expect("consumer compares the yielded product Int field");
        let string_action = consume_body[int_action..]
            .find("call i1 @topal.runtime.string.equal")
            .map(|offset| int_action + offset)
            .expect("consumer compares the yielded product String field after the Int field");
        let final_result = consume_body[string_action..]
            .find("call ptr @topal.runtime.result.success")
            .map(|offset| string_action + offset)
            .expect("consumer constructs the successful final Result after resumption");
        let returned = consume_body[final_result..]
            .find("ret ptr")
            .map(|offset| final_result + offset)
            .expect("consumer returns the final Result");
        assert!(
            optional_tags < int_action
                && int_action < string_action
                && string_action < final_result
                && final_result < returned
        );
        for expected in [
            "name: \"Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)\"",
            "name: \"Optional (Int, String)\"",
            "name: \"Result ((Int, String), lang arithmetic ArithmeticErrorCode)\"",
            "DILocalVariable(name: \"initial\", arg: 1",
            "DILocalVariable(name: \"generated\", arg: 1",
            "DILocalVariable(name: \"value\"",
            "DILocalVariable(name: \"result\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!llvm.contains("topal.runtime.generator"));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("call i32 %"));
    }

    #[test]
    fn emits_list_custom_generator_function_boundaries() {
        // TOPAL-GENERATOR-DECLARATION-001,
        // TOPAL-GENERATOR-SUSPEND-001,
        // TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001,
        // TOPAL-GENERATOR-FUNCTION-RESULT-001,
        // TOPAL-GENERATOR-FUNCTION-PARAMETER-001,
        // TOPAL-GENERATOR-FOREACH-RESULT-001,
        // TOPAL-TYPE-LIST-CONSTRUCT-001,
        // TOPAL-LIST-APPEND-001,
        // TOPAL-LIST-ENTRY-COUNT-001,
        // TOPAL-COMPILER-GENERATOR-LIST-FUNCTION-BOUNDARY-001
        let source = include_str!("../../../examples/language/custom-generator-list-values.t");
        let program = analyze_for_compiler(source).unwrap();
        let make = program
            .functions
            .iter()
            .find(|function| function.source_name == "make")
            .expect("List factory specialization exists");
        let consume = program
            .functions
            .iter()
            .find(|function| function.source_name == "consume")
            .expect("List consumer specialization exists");
        let llvm = Generator::new(&program, "custom-generator-list-values.t").emit();
        assert!(llvm.contains(&format!(
            "define internal fastcc i32 @{}(ptr %arg0)",
            make.symbol
        )));
        assert!(llvm.contains(&format!(
            "define internal fastcc ptr @{}(i32 %arg0)",
            consume.symbol
        )));
        assert!(llvm.contains(&format!("call fastcc i32 @{}(ptr", make.symbol)));
        assert!(llvm.contains(&format!("call fastcc ptr @{}(i32", consume.symbol)));

        let make_body = llvm
            .split_once(&format!(
                "define internal fastcc i32 @{}(ptr %arg0)",
                make.symbol
            ))
            .expect("module contains List factory")
            .1
            .split_once("\n}\n")
            .expect("List factory has one body")
            .0;
        assert!(make_body.contains("alloca ptr, align 8"));
        assert!(make_body.contains("#dbg_declare"));

        let consume_body = llvm
            .split_once(&format!(
                "define internal fastcc ptr @{}(i32 %arg0)",
                consume.symbol
            ))
            .expect("module contains List consumer")
            .1
            .split_once("\n}\n")
            .expect("List consumer has one body")
            .0;
        let action = consume_body
            .find("call ptr @topal.runtime.list.int.entry.count")
            .expect("consumer observes the yielded List before resumption");
        let appended_node = consume_body[action..]
            .find("call ptr @topal.platform.allocate(i64 16)")
            .map(|offset| action + offset)
            .expect("consumer constructs the appended singleton after resumption");
        let final_list = consume_body[appended_node..]
            .find("call ptr @topal.runtime.list.int.concat")
            .map(|offset| appended_node + offset)
            .expect("consumer appends after constructing the singleton");
        let returned = consume_body[final_list..]
            .find("ret ptr")
            .map(|offset| final_list + offset)
            .expect("consumer returns the final List");
        assert!(action < appended_node && appended_node < final_list && final_list < returned);
        for expected in [
            "name: \"Generator List Int Unit List Int\"",
            "name: \"List Int\"",
            "DILocalVariable(name: \"initial\", arg: 1",
            "DILocalVariable(name: \"generated\", arg: 1",
            "DILocalVariable(name: \"values\"",
            "DILocalVariable(name: \"result\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!llvm.contains("topal.runtime.generator"));
        assert!(!llvm.contains("call ptr %"));
        assert!(!llvm.contains("call i32 %"));
    }

    #[test]
    fn emits_private_direct_task_storage_and_transactions_in_source_order() {
        // TOPAL-TASK-LIFECYCLE-001, TOPAL-TASK-STATE-001,
        // TOPAL-TASK-MESSAGE-001, TOPAL-COMPILER-TASK-DIRECT-001
        let source = include_str!("../../../examples/language/task-declaration-order.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "task-declaration-order.t").emit();
        assert!(llvm.contains("%topal.TaskStorage = type { i64, i64, ptr }"));
        assert!(llvm.contains("@topal.runtime.task.next.identity = private global i64 1"));
        let constructed = llvm
            .find("call ptr @topal.runtime.task.make")
            .expect("start constructs a distinct Task instance");
        let loaded_for_event = llvm[constructed..]
            .find("call ptr @topal.runtime.task.state.load")
            .map(|offset| constructed + offset)
            .expect("event transaction reads private state");
        let added = llvm[loaded_for_event..]
            .find("call ptr @topal.runtime.int.add")
            .map(|offset| loaded_for_event + offset)
            .expect("event computes the replacement state");
        let replaced = llvm[added..]
            .find("call void @topal.runtime.task.state.replace")
            .map(|offset| added + offset)
            .expect("event commits state atomically");
        let loaded_for_request = llvm[replaced..]
            .find("call ptr @topal.runtime.task.state.load")
            .map(|offset| replaced + offset)
            .expect("request observes the committed state");
        assert!(
            constructed < loaded_for_event
                && loaded_for_event < added
                && added < replaced
                && replaced < loaded_for_request
        );
        for expected in [
            "DW_TAG_typedef, name: \"OrderedCounter\"",
            "DW_TAG_structure_type, name: \"TopalTask.OrderedCounter\"",
            "name: \"identity\"",
            "name: \"terminated\"",
            "name: \"count\"",
            "DILocalVariable(name: \"ordered-counter\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!llvm.contains("@malloc"));
    }

    #[test]
    fn emits_one_yield_task_stream_as_an_ordered_affine_transaction() {
        // TOPAL-TASK-HANDLER-001, TOPAL-TASK-MESSAGE-001,
        // TOPAL-COMPILER-TASK-STREAM-001
        let source = include_str!("../../../examples/language/task-message-transactions.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "task-message-transactions.t").emit();
        let constructed = llvm
            .find("call ptr @topal.runtime.task.make")
            .expect("start constructs the stream-capable task");
        let event_load = llvm[constructed..]
            .find("call ptr @topal.runtime.task.state.load")
            .map(|offset| constructed + offset)
            .expect("event loads private state");
        let replaced = llvm[event_load..]
            .find("call void @topal.runtime.task.state.replace")
            .map(|offset| event_load + offset)
            .expect("event commits private state");
        let stream_load = llvm[replaced..]
            .find("call ptr @topal.runtime.task.state.load")
            .map(|offset| replaced + offset)
            .expect("stream yield loads the committed state");
        let stream_result = llvm[stream_load..]
            .find("call ptr @topal.runtime.result.success(ptr null)")
            .map(|offset| stream_load + offset)
            .expect("stream completion commits its Unit result");
        let request_load = llvm[stream_result..]
            .find("call ptr @topal.runtime.task.state.load")
            .map(|offset| stream_result + offset)
            .expect("following request executes after stream completion");
        assert!(
            constructed < event_load
                && event_load < replaced
                && replaced < stream_load
                && stream_load < stream_result
                && stream_result < request_load
        );
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.task.state.load")
                .count(),
            3,
            "stream construction must capture the task without observing state"
        );
        for expected in [
            "DW_TAG_typedef, name: \"Counter\"",
            "DW_TAG_structure_type, name: \"TopalTask.Counter\"",
            "DW_TAG_enumeration_type, name: \"Generator Nat Unit Result (Unit, ())\"",
            "DILocalVariable(name: \"stream\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!llvm.contains("@malloc"));
        assert!(!llvm.contains("topal.runtime.generator"));
    }

    #[test]
    fn emits_checked_external_location_as_ordered_topal_owned_storage() {
        // TOPAL-LAYOUT-CONSTRUCT-001, TOPAL-ADDRESS-RANGE-001,
        // TOPAL-LOCATION-CONSTRUCT-001, TOPAL-LOCATION-READ-001,
        // TOPAL-LOCATION-WRITE-001, TOPAL-COMPILER-EXTERNAL-LOCATION-001
        let source = include_str!("../../../examples/language/external-layout-location.t");
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "external-layout-location.t").emit();
        let constructed = llvm
            .find("call ptr @topal.runtime.location.make")
            .expect("checked location allocates Topal-owned storage");
        let written = llvm[constructed..]
            .find("call void @topal.runtime.location.write")
            .map(|offset| constructed + offset)
            .expect("source write remains an ordered runtime call");
        let read = llvm[written..]
            .find("call ptr @topal.runtime.location.read")
            .map(|offset| written + offset)
            .expect("source read follows the write");
        assert!(constructed < written && written < read);
        assert_eq!(
            llvm.matches("call void @topal.runtime.location.write")
                .count(),
            1
        );
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.location.read")
                .count(),
            1
        );
        for expected in [
            "%topal.LocationStorage = type { ptr, ptr, i64, ptr }",
            "DW_TAG_typedef, name: \"UInt32LE\"",
            "DW_TAG_typedef, name: \"ControlLocation\"",
            "DW_TAG_structure_type, name: \"TopalLocation.ControlLocation\"",
            "DILocalVariable(name: \"control\"",
            "DILocalVariable(name: \"stored\"",
        ] {
            assert!(llvm.contains(expected), "missing {expected:?}");
        }
        assert!(!llvm.contains("inttoptr i64 1073741856"));
        assert!(!llvm.contains("@malloc"));
    }
}
