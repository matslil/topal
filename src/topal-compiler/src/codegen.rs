use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use topal_language::{
    CompilerBinary, CompilerBlock, CompilerExpression, CompilerExpressionKind, CompilerFunction,
    CompilerProgram, CompilerStatement, CompilerType, display_string_literal,
};
use topal_source::Span;

use crate::{DATA_LAYOUT, TARGET_TRIPLE};

pub fn emit_llvm(program: &CompilerProgram, source_name: &str) -> String {
    Generator::new(program, source_name).emit()
}

struct Generator<'a> {
    program: &'a CompilerProgram,
    globals: Vec<String>,
    functions: Vec<String>,
    next_global: usize,
    debug: DebugInfo,
}

impl<'a> Generator<'a> {
    fn new(program: &'a CompilerProgram, source_name: &str) -> Self {
        let mut debug = DebugInfo::new(source_name);
        debug.set_source(program.source.clone());
        Self {
            program,
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
        for global in &self.globals {
            let _ = writeln!(module, "{global}");
        }
        module.push('\n');
        module.push_str(PLATFORM_RUNTIME);
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
                format!("{} %arg{index}", llvm_parameter_type(&parameter.value_type))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let mut body = FunctionBody::new(subprogram);
        let mut environment = BTreeMap::new();
        for (index, parameter) in function.parameters.iter().enumerate() {
            let value = match parameter.value_type {
                CompilerType::Unit => LlValue::Unit,
                CompilerType::Boolean => LlValue::Boolean(format!("%arg{index}")),
                CompilerType::Int => LlValue::Int(format!("%arg{index}")),
                CompilerType::Rational => LlValue::Rational(format!("%arg{index}")),
                CompilerType::Comparison => LlValue::Comparison(format!("%arg{index}")),
                _ => unreachable!("shared model restricts machine parameters"),
            };
            let variable = self.debug.parameter(
                &parameter.name,
                index + 1,
                parameter.span,
                &parameter.value_type,
                subprogram,
            );
            let location = self.debug.location(parameter.span, subprogram);
            body.debug_value(&value, variable, location);
            environment.insert(parameter.name.clone(), value);
        }
        let result = self.emit_block(&function.body, &mut body, &mut environment);
        let location = self.debug.location(function.body.result.span, subprogram);
        match result {
            LlValue::Unit => body.terminator("ret void", location),
            LlValue::Boolean(value) => body.terminator(&format!("ret i1 {value}"), location),
            LlValue::Int(value) | LlValue::Rational(value) => {
                body.terminator(&format!("ret ptr {value}"), location);
            }
            LlValue::Comparison(value) => body.terminator(&format!("ret i32 {value}"), location),
            _ => unreachable!("shared model restricts machine results"),
        }
        self.functions.push(format!(
            "define internal fastcc {return_type} @{}({parameters}) nounwind noinline !dbg !{subprogram} {{\n{}\n}}\n",
            function.symbol,
            body.render()
        ));
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
                    }
                    environment.insert(binding.name.clone(), value);
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
            CompilerExpressionKind::Boolean(value) => LlValue::Boolean(value.to_string()),
            CompilerExpressionKind::Int(value) => self.emit_int_literal(value),
            CompilerExpressionKind::Rational(value) => {
                self.emit_rational_literal(value, body, expression.span)
            }
            CompilerExpressionKind::String(value) => LlValue::String(value.clone()),
            CompilerExpressionKind::Tuple(values) => LlValue::Tuple(
                values
                    .iter()
                    .map(|value| self.emit_expression(value, body, environment))
                    .collect(),
            ),
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
                let arguments = values
                    .iter()
                    .map(LlValue::argument)
                    .collect::<Vec<_>>()
                    .join(", ");
                match expression.value_type {
                    CompilerType::Unit => {
                        body.effect(
                            &format!("call fastcc void @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        );
                        LlValue::Unit
                    }
                    CompilerType::Boolean => LlValue::Boolean(body.instruction(
                        &format!("call fastcc i1 @{symbol}({arguments})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    CompilerType::Int => LlValue::Int(body.instruction(
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
                    _ => unreachable!("shared model restricts call result types"),
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
                LlValue::Comparison(self.emit_numeric_compare(left, right, body, span))
            }
            CompilerBinary::And => LlValue::Boolean(body.instruction(
                &format!("and i1 {}, {}", left.boolean(), right.boolean()),
                span,
                &mut self.debug,
            )),
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
                let instruction = match (left, right) {
                    (LlValue::Unit, LlValue::Unit) => format!("icmp {predicate} i8 0, 0"),
                    (LlValue::Boolean(left), LlValue::Boolean(right)) => {
                        format!("icmp {predicate} i1 {left}, {right}")
                    }
                    (LlValue::Comparison(left), LlValue::Comparison(right)) => {
                        format!("icmp {predicate} i32 {left}, {right}")
                    }
                    (LlValue::Int(_), LlValue::Int(_))
                    | (LlValue::Rational(_), LlValue::Rational(_)) => format!(
                        "icmp {predicate} i32 {}, 0",
                        self.emit_numeric_compare(left, right, body, span)
                    ),
                    _ => unreachable!("checked equality types agree"),
                };
                LlValue::Boolean(body.instruction(&instruction, span, &mut self.debug))
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
                let comparison = self.emit_numeric_compare(left, right, body, span);
                LlValue::Boolean(body.instruction(
                    &format!("icmp {predicate} i32 {comparison}, 0"),
                    span,
                    &mut self.debug,
                ))
            }
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
        match (true_value, false_value) {
            (LlValue::Unit, LlValue::Unit) => LlValue::Unit,
            (LlValue::Boolean(left), LlValue::Boolean(right)) => {
                LlValue::Boolean(body.instruction(
                    &format!(
                        "phi i1 [{left}, %{true_predecessor}], [{right}, %{false_predecessor}]"
                    ),
                    span,
                    &mut self.debug,
                ))
            }
            (LlValue::Int(left), LlValue::Int(right)) => LlValue::Int(body.instruction(
                &format!("phi ptr [{left}, %{true_predecessor}], [{right}, %{false_predecessor}]"),
                span,
                &mut self.debug,
            )),
            (LlValue::Rational(left), LlValue::Rational(right)) => {
                LlValue::Rational(body.instruction(
                    &format!(
                        "phi ptr [{left}, %{true_predecessor}], [{right}, %{false_predecessor}]"
                    ),
                    span,
                    &mut self.debug,
                ))
            }
            (LlValue::Comparison(left), LlValue::Comparison(right)) => {
                LlValue::Comparison(body.instruction(
                    &format!(
                        "phi i32 [{left}, %{true_predecessor}], [{right}, %{false_predecessor}]"
                    ),
                    span,
                    &mut self.debug,
                ))
            }
            _ => unreachable!("checked decision branch types agree"),
        }
    }

    fn emit_print(&mut self, value: &LlValue, body: &mut FunctionBody, span: Span) {
        match value {
            LlValue::Unit => self.emit_write_literal("()", body, span),
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
            LlValue::String(value) => {
                self.emit_write_literal(&display_string_literal(value), body, span);
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
        }
    }

    fn emit_write_literal(&mut self, text: &str, body: &mut FunctionBody, span: Span) {
        if text.is_empty() {
            return;
        }
        let name = format!(".topal.bytes.{}", self.next_global);
        self.next_global += 1;
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            text.len(),
            llvm_bytes(text.as_bytes())
        ));
        body.effect(
            &format!(
                "call void @topal.platform.write_all(ptr @{name}, i64 {})",
                text.len()
            ),
            span,
            &mut self.debug,
        );
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
    Boolean(String),
    Int(String),
    Rational(String),
    Comparison(String),
    String(String),
    Tuple(Vec<Self>),
}

impl LlValue {
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

    fn argument(&self) -> String {
        match self {
            Self::Unit => "i8 0".into(),
            Self::Boolean(value) => format!("i1 {value}"),
            Self::Int(value) | Self::Rational(value) => format!("ptr {value}"),
            Self::Comparison(value) => format!("i32 {value}"),
            _ => unreachable!("checked call arguments are scalar"),
        }
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
            LlValue::Boolean(value) => format!("i1 {value}"),
            LlValue::Int(value) | LlValue::Rational(value) => format!("ptr {value}"),
            LlValue::Comparison(value) => format!("i32 {value}"),
            LlValue::String(_) | LlValue::Tuple(_) => return,
        };
        self.lines.push(format!(
            "    #dbg_value({value}, !{variable}, !DIExpression(), !{location})"
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
    rational_type: usize,
    comparison_type: usize,
    boolean_type: usize,
    unit_type: usize,
    source: topal_source::SourceText,
    filename: String,
}

impl DebugInfo {
    fn new(source_name: &str) -> Self {
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
            rational_type: 0,
            comparison_type: 0,
            boolean_type: 0,
            unit_type: 0,
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
        let negative = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"negative\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 0)",
            debug.file
        ));
        let length = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_member, name: \"length\", file: !{}, baseType: !{unsigned64}, size: 64, align: 64, offset: 64)",
            debug.file
        ));
        let members = debug.node(format!("!{{!{negative}, !{length}}}"));
        let storage = debug.node(format!(
            "!DICompositeType(tag: DW_TAG_structure_type, name: \"TopalIntHeader\", file: !{}, size: 128, align: 64, elements: !{members})",
            debug.file
        ));
        let pointer = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{storage}, size: 64, align: 64)"
        ));
        debug.int_type = debug.node(format!(
            "!DIDerivedType(tag: DW_TAG_typedef, name: \"Int\", file: !{}, baseType: !{pointer})",
            debug.file
        ));
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
        let less = debug.node("!DIEnumerator(name: \"Less\", value: -1)".into());
        let equal = debug.node("!DIEnumerator(name: \"Equal\", value: 0)".into());
        let greater = debug.node("!DIEnumerator(name: \"Greater\", value: 1)".into());
        let comparison_values = debug.node(format!("!{{!{less}, !{equal}, !{greater}}}"));
        debug.comparison_type = debug.node(format!(
            "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"Comparison\", file: !{}, size: 32, align: 32, elements: !{comparison_values})",
            debug.file
        ));
        debug.boolean_type =
            debug.node("!DIBasicType(name: \"Boolean\", size: 8, encoding: DW_ATE_boolean)".into());
        debug.unit_type =
            debug.node("!DIBasicType(name: \"Unit\", size: 8, encoding: DW_ATE_unsigned)".into());
        debug
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

    fn type_id(&self, value_type: &CompilerType) -> usize {
        match value_type {
            CompilerType::Unit => self.unit_type,
            CompilerType::Boolean => self.boolean_type,
            CompilerType::Int => self.int_type,
            CompilerType::Rational => self.rational_type,
            CompilerType::Comparison => self.comparison_type,
            CompilerType::String | CompilerType::Tuple(_) => {
                unreachable!("structural values have no native debug representation yet")
            }
        }
    }

    fn subprogram(
        &mut self,
        name: &str,
        linkage_name: &str,
        span: Span,
        result: &CompilerType,
        parameters: &[CompilerType],
    ) -> usize {
        let mut types = vec![if *result == CompilerType::Unit {
            "null".into()
        } else {
            format!("!{}", self.type_id(result))
        }];
        types.extend(
            parameters
                .iter()
                .map(|parameter| format!("!{}", self.type_id(parameter))),
        );
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
        self.node(format!(
            "!DILocalVariable(name: \"{}\"{argument}, scope: !{scope}, file: !{}, line: {}, type: !{})",
            llvm_string(name),
            self.file,
            position.line,
            self.type_id(value_type)
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

fn llvm_type(value_type: &CompilerType) -> &'static str {
    match value_type {
        CompilerType::Unit => "void",
        CompilerType::Boolean => "i1",
        CompilerType::Int | CompilerType::Rational => "ptr",
        CompilerType::Comparison => "i32",
        _ => unreachable!("shared model restricts function ABI types"),
    }
}

fn llvm_parameter_type(value_type: &CompilerType) -> &'static str {
    match value_type {
        CompilerType::Unit => "i8",
        CompilerType::Boolean => "i1",
        CompilerType::Int | CompilerType::Rational => "ptr",
        CompilerType::Comparison => "i32",
        _ => unreachable!("shared model restricts function ABI types"),
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
        let source = "use language (version is v0.1)\nvalue is 40 + 2\nproduct is 6 * 7\nratio is 6 / 8\nordered is product >= value\n(value, product, ratio, ordered, true, \"Topal\")\n";
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
        assert!(llvm.contains("@llvm.ctlz.i32"));
        assert!(llvm.contains("name: \"Rational\""));
        assert!(llvm.contains("constant { i64, i64, [1 x i32] }"));
        assert!(llvm.contains("\\54\\6F\\70\\61\\6C"));
        assert!(llvm.contains("#dbg_value"));
        assert!(llvm.contains("Dwarf Version"));
    }
}
