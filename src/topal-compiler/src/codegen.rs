use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use topal_language::{
    CompilerBinary, CompilerBlock, CompilerComparisonRule, CompilerEnumRule, CompilerEnumType,
    CompilerErrorCodeRule, CompilerErrorField, CompilerExpression, CompilerExpressionKind,
    CompilerFallible, CompilerFunction, CompilerProgram, CompilerStatement, CompilerType,
    CompilerValidation, display_string_literal,
};
use topal_source::Span;

use crate::{DATA_LAYOUT, TARGET_TRIPLE};

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

fn block_uses_extended_debug(block: &CompilerBlock) -> bool {
    block.statements.iter().any(|statement| match statement {
        CompilerStatement::Binding(binding) => expression_uses_extended_debug(&binding.value),
        CompilerStatement::Discard(expression) => expression_uses_extended_debug(expression),
    }) || expression_uses_extended_debug(&block.result)
}

fn type_uses_extended_debug(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Character
        | CompilerType::String
        | CompilerType::Error
        | CompilerType::ErrorCode
        | CompilerType::ErrorDomain
        | CompilerType::Optional(_) => true,
        CompilerType::Range(endpoint) | CompilerType::Result(endpoint) => {
            type_uses_extended_debug(endpoint)
        }
        CompilerType::Tuple(fields) => fields.iter().any(type_uses_extended_debug),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, value_type)| type_uses_extended_debug(value_type)),
        CompilerType::Unit
        | CompilerType::Completed
        | CompilerType::Effect
        | CompilerType::Type
        | CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Constraint
        | CompilerType::Boolean
        | CompilerType::Int
        | CompilerType::Nat
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
        | CompilerExpressionKind::OptionalDecision { .. }
        | CompilerExpressionKind::ErrorCode(_) => true,
        CompilerExpressionKind::Tuple(fields)
        | CompilerExpressionKind::Call {
            arguments: fields, ..
        } => fields.iter().any(expression_uses_extended_debug),
        CompilerExpressionKind::Record(fields) => fields
            .iter()
            .any(|(_, value)| expression_uses_extended_debug(value)),
        CompilerExpressionKind::RecordReconstruct { base, replacements } => {
            expression_uses_extended_debug(base)
                || replacements
                    .iter()
                    .any(|(_, value)| expression_uses_extended_debug(value))
        }
        CompilerExpressionKind::Block(block) => block_uses_extended_debug(block),
        CompilerExpressionKind::Negate(value)
        | CompilerExpressionKind::Absolute(value)
        | CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::RationalToInt(value)
        | CompilerExpressionKind::IntToNat(value)
        | CompilerExpressionKind::ResultSuccess(value)
        | CompilerExpressionKind::ResultProject(value)
        | CompilerExpressionKind::OptionalSome(value)
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
        CompilerExpressionKind::StringConcat { left, right }
        | CompilerExpressionKind::RationalConstruct {
            numerator: left,
            denominator: right,
        }
        | CompilerExpressionKind::Fallible { left, right, .. }
        | CompilerExpressionKind::Binary { left, right, .. } => {
            expression_uses_extended_debug(left) || expression_uses_extended_debug(right)
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
        CompilerExpressionKind::Unit
        | CompilerExpressionKind::Completed
        | CompilerExpressionKind::Effect
        | CompilerExpressionKind::TypeValue(_)
        | CompilerExpressionKind::Root
        | CompilerExpressionKind::FunctionValue(_)
        | CompilerExpressionKind::ConstraintValue(_)
        | CompilerExpressionKind::Boolean(_)
        | CompilerExpressionKind::Int(_)
        | CompilerExpressionKind::Rational(_)
        | CompilerExpressionKind::Enum(_)
        | CompilerExpressionKind::OptionalNone
        | CompilerExpressionKind::Local(_) => false,
    }
}

struct Generator<'a> {
    program: &'a CompilerProgram,
    source_name: &'a str,
    globals: Vec<String>,
    functions: Vec<String>,
    next_global: usize,
    debug: DebugInfo,
}

impl<'a> Generator<'a> {
    fn new(program: &'a CompilerProgram, source_name: &'a str) -> Self {
        let mut debug = DebugInfo::new(source_name, program_uses_extended_debug(program));
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

    fn emit_function(&mut self, function: &CompilerFunction) {
        let return_type = llvm_type(&function.result_type);
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
        let result = self.emit_block(&function.body, &mut body, &mut environment);
        let location = self.debug.location(function.body.result.span, subprogram);
        match result {
            LlValue::Unit => body.terminator("ret void", location),
            LlValue::Completed(value) | LlValue::Effect(value) => {
                body.terminator(&format!("ret i8 {value}"), location);
            }
            LlValue::Boolean(value) => body.terminator(&format!("ret i1 {value}"), location),
            LlValue::Int(value)
            | LlValue::Rational(value)
            | LlValue::Error(value)
            | LlValue::ErrorDomain(value)
            | LlValue::String(value)
            | LlValue::Range { value, .. }
            | LlValue::Result { value, .. }
            | LlValue::Optional { value, .. } => {
                body.terminator(&format!("ret ptr {value}"), location);
            }
            LlValue::Comparison(value)
            | LlValue::ErrorCode(value)
            | LlValue::Enum { value, .. } => {
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
        }
        self.functions.push(format!(
            "define internal fastcc {return_type} @{}({parameters}) nounwind noinline !dbg !{subprogram} {{\n{}\n}}\n",
            function.symbol,
            body.render()
        ));
    }

    fn bind_function_parameters(
        &mut self,
        function: &CompilerFunction,
        body: &mut FunctionBody,
        environment: &mut BTreeMap<String, LlValue>,
    ) {
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
                CompilerType::Function => LlValue::Enum {
                    value: argument.clone(),
                    enumeration: function_value_enumeration(self.program),
                },
                _ => function_parameter_value(&parameter.value_type, index),
            };
            let variable = self.debug.parameter(
                &parameter.name,
                index + 1,
                parameter.span,
                &parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(parameter.span, body.subprogram);
            if matches!(
                parameter.value_type,
                CompilerType::Function | CompilerType::Tuple(_) | CompilerType::Record(_)
            ) {
                self.emit_aggregate_debug_shadow(
                    &argument,
                    &parameter.value_type,
                    variable,
                    location,
                    body,
                    parameter.span,
                );
            } else {
                body.debug_value(&value, variable, location);
            }
            environment.insert(parameter.name.clone(), value);
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
                    match field_type {
                        CompilerType::Tuple(nested_types) => {
                            self.emit_tuple_extract(&field, nested_types, body, span)
                        }
                        CompilerType::Record(nested_types) => {
                            self.emit_record_extract(&field, nested_types, body, span)
                        }
                        _ => machine_value(field_type, field),
                    }
                })
                .collect(),
        )
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
                let field = match field_type {
                    CompilerType::Tuple(nested_types) => {
                        self.emit_tuple_extract(&field, nested_types, body, span)
                    }
                    CompilerType::Record(nested_types) => {
                        self.emit_record_extract(&field, nested_types, body, span)
                    }
                    _ => machine_value(field_type, field),
                };
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
                    if binding.value.value_type.machine_scalar() {
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
                    environment.insert(binding.storage_name.clone(), value);
                }
                CompilerStatement::Discard(expression) => {
                    let _ = self.emit_expression(expression, body, environment);
                }
            }
        }
        self.emit_expression(&block.result, body, environment)
    }

    #[allow(clippy::too_many_lines)] // Exhaustive checked-model lowering keeps every admitted form explicit.
    fn emit_expression(
        &mut self,
        expression: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        match &expression.kind {
            CompilerExpressionKind::Unit => LlValue::Unit,
            CompilerExpressionKind::Completed => LlValue::Completed("0".into()),
            CompilerExpressionKind::Effect => LlValue::Effect("0".into()),
            CompilerExpressionKind::TypeValue(value) => LlValue::Enum {
                value: value.to_string(),
                enumeration: fundamental_type_enumeration(),
            },
            CompilerExpressionKind::Root => LlValue::Enum {
                value: "0".into(),
                enumeration: root_scope_enumeration(),
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
            CompilerExpressionKind::Int(value) => self.emit_int_literal(value),
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
            CompilerExpressionKind::Tuple(values) => LlValue::Tuple(
                values
                    .iter()
                    .map(|value| self.emit_expression(value, body, environment))
                    .collect(),
            ),
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
            CompilerExpressionKind::Local(name) => environment
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
            CompilerExpressionKind::IntToNat(value) => {
                self.emit_expression(value, body, environment)
            }
            CompilerExpressionKind::ResultSuccess(value) => {
                let success = self.emit_expression(value, body, environment);
                let payload =
                    self.emit_result_payload(&success, &value.value_type, body, value.span);
                LlValue::Result {
                    value: body.instruction(
                        &format!("call ptr @topal.runtime.result.success(ptr {payload})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    success: value.value_type.clone(),
                }
            }
            CompilerExpressionKind::ResultProject(value) => {
                self.emit_result_project(value, body, environment, expression.span)
            }
            CompilerExpressionKind::OptionalSome(value) => {
                let payload_value = self.emit_expression(value, body, environment);
                let payload = optional_payload_pointer(&payload_value);
                LlValue::Optional {
                    value: body.instruction(
                        &format!("call ptr @topal.runtime.optional.some(ptr {payload})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: value.value_type.clone(),
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
                }
            }
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
                    CompilerType::Int => LlValue::Int(value),
                    CompilerType::Rational => LlValue::Rational(value),
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
                let values = arguments
                    .iter()
                    .map(|argument| self.emit_expression(argument, body, environment))
                    .collect::<Vec<_>>();
                let parameter_types = self
                    .program
                    .functions
                    .iter()
                    .find(|function| function.symbol == *symbol)
                    .expect("checked call target has a generated function")
                    .parameters
                    .iter()
                    .map(|parameter| parameter.value_type.clone())
                    .collect::<Vec<_>>();
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
                        enumeration: root_scope_enumeration(),
                    },
                    CompilerType::Function | CompilerType::Constraint => {
                        unreachable!("checked functions do not return this static object kind")
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
                    },
                    CompilerType::Optional(ref payload) => LlValue::Optional {
                        value: body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        payload: payload.as_ref().clone(),
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
        }
    }

    fn emit_result_project(
        &mut self,
        value: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let result = self.emit_expression(value, body, environment);
        let LlValue::Result { value, success } = result else {
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
        body.terminator(&format!("ret ptr {value}"), location);
        body.start_block(&success_label);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            span,
            &mut self.debug,
        );
        self.result_success_value(&payload, &success, body, span)
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
        let LlValue::Optional { value, payload } = optional else {
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
            let payload_value = optional_payload_value(payload_pointer, &payload);
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
        let LlValue::Result { value, success } = result else {
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
        let success_value = self.result_success_value(&payload, &success, body, ok_binding_span);
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
            (LlValue::Int(value), CompilerType::Int | CompilerType::Nat)
            | (LlValue::Rational(value), CompilerType::Rational)
            | (LlValue::String(value), CompilerType::String)
            | (LlValue::Range { value, .. }, CompilerType::Range(_))
            | (LlValue::Result { value, .. }, CompilerType::Result(_)) => value.clone(),
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
            CompilerType::Rational => LlValue::Rational(payload.into()),
            CompilerType::String => LlValue::String(payload.into()),
            CompilerType::Range(endpoint) => LlValue::Range {
                value: payload.into(),
                endpoint: endpoint.as_ref().clone(),
            },
            CompilerType::Result(nested) => LlValue::Result {
                value: payload.into(),
                success: nested.as_ref().clone(),
            },
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
                LlValue::Optional {
                    value: left,
                    payload,
                },
                LlValue::Optional {
                    value: right,
                    payload: right_payload,
                },
            ) => {
                debug_assert_eq!(payload, right_payload);
                let runtime = match payload {
                    CompilerType::Int => "optional.int.equal",
                    CompilerType::Rational => "optional.rational.equal",
                    CompilerType::String => "optional.string.equal",
                    _ => unreachable!("checked Optional equality has canonical evidence"),
                };
                body.instruction(
                    &format!("call i1 @topal.runtime.{runtime}(ptr {left}, ptr {right})"),
                    span,
                    &mut self.debug,
                )
            }
            (LlValue::Int(_), LlValue::Int(_)) | (LlValue::Rational(_), LlValue::Rational(_)) => {
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
            _ => unreachable!("checked equality values agree"),
        }
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
            (LlValue::Int(_), LlValue::Int(_)) | (LlValue::Rational(_), LlValue::Rational(_)) => {
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
            LlValue::Int(_) => LlValue::Int(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::integer)),
                span,
                &mut self.debug,
            )),
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
                }
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
            LlValue::Int(value) => body.effect(
                &format!("call void @topal.runtime.int.print(ptr {value})"),
                span,
                &mut self.debug,
            ),
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
            LlValue::Enum { value, enumeration } => {
                self.emit_print_enum(value, enumeration, body, span);
            }
            LlValue::Range { value, endpoint } => body.effect(
                &format!(
                    "call void @topal.runtime.range.{}.print(ptr {value})",
                    numeric_domain(endpoint)
                ),
                span,
                &mut self.debug,
            ),
            LlValue::Result { value, success } => {
                self.emit_print_result(value, success, body, span);
            }
            LlValue::Optional { value, payload } => {
                self.emit_print_optional(value, payload, body, span);
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
        let payload = optional_payload_value(payload, payload_type);
        self.emit_print(&payload, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&none);
        self.emit_write_literal("None", body, span);
        body.terminator(&format!("br label %{done}"), location);
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
        let name = format!(".topal.bytes.{}", self.next_global);
        self.next_global += 1;
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            text.len(),
            llvm_bytes(text.as_bytes())
        ));
        (format!("@{name}"), text.len())
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

    fn emit_int_literal(&mut self, value: &BigInt) -> LlValue {
        LlValue::Int(self.emit_int_global(value))
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
    Completed(String),
    Effect(String),
    Boolean(String),
    Int(String),
    Rational(String),
    Comparison(String),
    Error(String),
    ErrorCode(String),
    ErrorDomain(String),
    Enum {
        value: String,
        enumeration: CompilerEnumType,
    },
    Range {
        value: String,
        endpoint: CompilerType,
    },
    Result {
        value: String,
        success: CompilerType,
    },
    Optional {
        value: String,
        payload: CompilerType,
    },
    String(String),
    Tuple(Vec<Self>),
    Record {
        fields: Vec<(String, Self)>,
        order: Vec<String>,
    },
}

impl LlValue {
    fn singleton(&self) -> &str {
        match self {
            Self::Completed(value) | Self::Effect(value) => value,
            _ => unreachable!("checked value is a zero-data singleton"),
        }
    }

    fn integer(&self) -> &str {
        let Self::Int(value) = self else {
            unreachable!("checked value is Int")
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

    fn enumeration(&self) -> &str {
        let Self::Enum { value, .. } = self else {
            unreachable!("checked value is a nominal Enum")
        };
        value
    }

    fn string(&self) -> &str {
        let Self::String(value) = self else {
            unreachable!("checked value is String")
        };
        value
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

    fn argument(&self) -> String {
        match self {
            Self::Unit => "i8 0".into(),
            Self::Completed(value) | Self::Effect(value) => format!("i8 {value}"),
            Self::Boolean(value) => format!("i1 {value}"),
            Self::Int(value)
            | Self::Rational(value)
            | Self::Error(value)
            | Self::ErrorDomain(value)
            | Self::String(value)
            | Self::Range { value, .. }
            | Self::Result { value, .. }
            | Self::Optional { value, .. } => {
                format!("ptr {value}")
            }
            Self::Comparison(value) | Self::ErrorCode(value) | Self::Enum { value, .. } => {
                format!("i32 {value}")
            }
            Self::Tuple(_) | Self::Record { .. } => {
                unreachable!("checked call arguments are scalar")
            }
        }
    }
}

fn optional_payload_pointer(value: &LlValue) -> &str {
    match value {
        LlValue::Int(value) | LlValue::Rational(value) | LlValue::String(value) => value,
        _ => unreachable!("checked Optional payload has a pointer representation"),
    }
}

fn optional_payload_value(value: String, value_type: &CompilerType) -> LlValue {
    match value_type {
        CompilerType::Int => LlValue::Int(value),
        CompilerType::Rational => LlValue::Rational(value),
        CompilerType::Character | CompilerType::String => LlValue::String(value),
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

fn root_scope_enumeration() -> CompilerEnumType {
    CompilerEnumType {
        name: "Scope".into(),
        alternatives: vec!["<namespace root>".into()],
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
        let value = format!("%v{}", self.next_value);
        self.next_value += 1;
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {value} = {instruction}, !dbg !{location}"));
        value
    }

    fn effect(&mut self, instruction: &str, span: Span, debug: &mut DebugInfo) {
        let location = debug.location(span, self.subprogram);
        self.lines
            .push(format!("  {instruction}, !dbg !{location}"));
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
            LlValue::Int(value)
            | LlValue::Rational(value)
            | LlValue::Error(value)
            | LlValue::ErrorDomain(value)
            | LlValue::String(value)
            | LlValue::Range { value, .. }
            | LlValue::Result { value, .. }
            | LlValue::Optional { value, .. } => {
                format!("ptr {value}")
            }
            LlValue::Comparison(value)
            | LlValue::ErrorCode(value)
            | LlValue::Enum { value, .. } => format!("i32 {value}"),
            LlValue::Tuple(_) | LlValue::Record { .. } => return,
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
    int_type: usize,
    nat_type: usize,
    rational_type: usize,
    character_type: usize,
    string_type: usize,
    error_type: usize,
    error_code_type: usize,
    error_domain_type: usize,
    int_range_type: usize,
    rational_range_type: usize,
    result_int_type: usize,
    result_nat_type: usize,
    result_rational_type: usize,
    result_string_type: usize,
    result_int_pair_type: usize,
    optional_int_type: usize,
    optional_rational_type: usize,
    optional_character_type: usize,
    optional_string_type: usize,
    comparison_type: usize,
    boolean_type: usize,
    unit_type: usize,
    completed_type: usize,
    effect_type: usize,
    enum_types: BTreeMap<String, usize>,
    tuple_types: Vec<(CompilerType, usize)>,
    record_types: Vec<(CompilerType, usize)>,
    source: topal_source::SourceText,
    filename: String,
}

impl DebugInfo {
    fn new(source_name: &str, extended_types: bool) -> Self {
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
            int_type: 0,
            nat_type: 0,
            rational_type: 0,
            character_type: 0,
            string_type: 0,
            error_type: 0,
            error_code_type: 0,
            error_domain_type: 0,
            int_range_type: 0,
            rational_range_type: 0,
            result_int_type: 0,
            result_nat_type: 0,
            result_rational_type: 0,
            result_string_type: 0,
            result_int_pair_type: 0,
            optional_int_type: 0,
            optional_rational_type: 0,
            optional_character_type: 0,
            optional_string_type: 0,
            comparison_type: 0,
            boolean_type: 0,
            unit_type: 0,
            completed_type: 0,
            effect_type: 0,
            enum_types: BTreeMap::new(),
            tuple_types: Vec::new(),
            record_types: Vec::new(),
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
        let unsigned64 =
            debug.node("!DIBasicType(name: \"u64\", size: 64, encoding: DW_ATE_unsigned)".into());
        debug.install_integer_types(unsigned64);
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
            debug.install_string_and_error_types(unsigned64);
        }
        debug.install_aggregate_types(unsigned64, extended_types);
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

    fn install_aggregate_types(&mut self, unsigned64: usize, extended_types: bool) {
        self.int_range_type = self.range_type("Range Int", self.int_type, unsigned64);
        self.rational_range_type =
            self.range_type("Range Rational", self.rational_type, unsigned64);
        self.install_result_types(unsigned64);
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

    fn install_string_and_error_types(&mut self, unsigned64: usize) {
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

    fn install_result_types(&mut self, unsigned64: usize) {
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
        self.result_int_type = self.result_type("Int", result_pointer);
        self.result_nat_type = self.result_type("Nat", result_pointer);
        self.result_rational_type = self.result_type("Rational", result_pointer);
        self.result_string_type = self.result_type("String", result_pointer);
        self.result_int_pair_type = self.result_type("(Int, Int)", result_pointer);
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
        self.optional_int_type = self.optional_type("Int", pointer);
        self.optional_rational_type = self.optional_type("Rational", pointer);
        self.optional_character_type = self.optional_type("Character", pointer);
        self.optional_string_type = self.optional_type("String", pointer);
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

    fn type_id(&mut self, value_type: &CompilerType) -> usize {
        match value_type {
            CompilerType::Unit => self.unit_type,
            CompilerType::Completed => self.completed_type,
            CompilerType::Effect => self.effect_type,
            CompilerType::Type => self.enum_type(&fundamental_type_enumeration()),
            CompilerType::Scope => self.enum_type(&root_scope_enumeration()),
            CompilerType::Function => *self
                .enum_types
                .get("Function")
                .expect("checked Function values install their debug type"),
            CompilerType::Constraint => *self
                .enum_types
                .get("Constraint")
                .expect("checked Constraint values install their debug type"),
            CompilerType::Boolean => self.boolean_type,
            CompilerType::Int => self.int_type,
            CompilerType::Nat => self.nat_type,
            CompilerType::Rational => self.rational_type,
            CompilerType::Comparison => self.comparison_type,
            CompilerType::Error => self.error_type,
            CompilerType::ErrorCode => self.error_code_type,
            CompilerType::ErrorDomain => self.error_domain_type,
            CompilerType::Enum(enumeration) => self.enum_type(enumeration),
            CompilerType::Character => self.character_type,
            CompilerType::String => self.string_type,
            CompilerType::Range(endpoint) if endpoint.as_ref() == &CompilerType::Int => {
                self.int_range_type
            }
            CompilerType::Range(endpoint) if endpoint.as_ref() == &CompilerType::Rational => {
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
            CompilerType::Result(_) => {
                unreachable!("unsupported Result success type reached codegen")
            }
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
            CompilerType::Optional(_) => {
                unreachable!("unsupported Optional payload type reached codegen")
            }
            CompilerType::Tuple(fields) => self.tuple_type(fields),
            CompilerType::Record(fields) => self.record_type(fields),
        }
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

#[derive(Clone, Copy)]
struct TargetValueLayout {
    size: u64,
    alignment: u64,
}

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
        | CompilerType::Enum(_) => TargetValueLayout {
            size: 32,
            alignment: 32,
        },
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::Rational
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::Optional(_)
        | CompilerType::Character
        | CompilerType::String => TargetValueLayout {
            size: 64,
            alignment: 64,
        },
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
    }
}

fn align_bits(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

fn private_aggregate_value_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::Tuple(fields) => fields.iter().all(private_aggregate_value_supported),
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, field)| private_aggregate_value_supported(field)),
        _ => true,
    }
}

fn llvm_value_type(value_type: &CompilerType) -> String {
    match value_type {
        CompilerType::Unit | CompilerType::Completed | CompilerType::Effect => "i8".into(),
        CompilerType::Boolean => "i1".into(),
        CompilerType::Int
        | CompilerType::Nat
        | CompilerType::Rational
        | CompilerType::Character
        | CompilerType::Error
        | CompilerType::ErrorDomain
        | CompilerType::String
        | CompilerType::Range(_)
        | CompilerType::Result(_)
        | CompilerType::Optional(_) => "ptr".into(),
        CompilerType::Type
        | CompilerType::Scope
        | CompilerType::Function
        | CompilerType::Constraint
        | CompilerType::Comparison
        | CompilerType::ErrorCode
        | CompilerType::Enum(_) => "i32".into(),
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
    }
}

fn function_parameter_value(value_type: &CompilerType, index: usize) -> LlValue {
    machine_value(value_type, format!("%arg{index}"))
}

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
            enumeration: root_scope_enumeration(),
        },
        CompilerType::Function => {
            unreachable!("Function values are not admitted at machine ABI reconstruction points")
        }
        CompilerType::Constraint => {
            unreachable!("Constraint values are not admitted at machine ABI reconstruction points")
        }
        CompilerType::Boolean => LlValue::Boolean(value),
        CompilerType::Int | CompilerType::Nat => LlValue::Int(value),
        CompilerType::Rational => LlValue::Rational(value),
        CompilerType::Comparison => LlValue::Comparison(value),
        CompilerType::Error => LlValue::Error(value),
        CompilerType::ErrorCode => LlValue::ErrorCode(value),
        CompilerType::ErrorDomain => LlValue::ErrorDomain(value),
        CompilerType::Enum(enumeration) => LlValue::Enum {
            value,
            enumeration: enumeration.clone(),
        },
        CompilerType::Character | CompilerType::String => LlValue::String(value),
        CompilerType::Range(endpoint) => LlValue::Range {
            value,
            endpoint: endpoint.as_ref().clone(),
        },
        CompilerType::Result(success) => LlValue::Result {
            value,
            success: success.as_ref().clone(),
        },
        CompilerType::Optional(payload) => LlValue::Optional {
            value,
            payload: payload.as_ref().clone(),
        },
        CompilerType::Tuple(_) | CompilerType::Record(_) => {
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
        assert!(llvm.contains("!DIEnumerator(name: \"<anonymous fn/1>\", value: 3)"));
        assert!(llvm.contains("!DIEnumerator(name: \"<anonymous fn/2>\", value: 4)"));
        assert!(llvm.contains("!DILocalVariable(name: \"value\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"left\", arg: 1"));
        assert!(llvm.contains("!DILocalVariable(name: \"right\", arg: 2"));
        assert!(!llvm.contains("topal.runtime.function"));
        assert!(!llvm.contains("topal.runtime.closure"));
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
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.optional.some")
                .count(),
            4
        );
        assert_eq!(
            llvm.matches("call ptr @topal.runtime.optional.none")
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
}
