use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

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
            LlValue::Int(value) => body.terminator(&format!("ret i64 {value}"), location),
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

    fn emit_expression(
        &mut self,
        expression: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        match &expression.kind {
            CompilerExpressionKind::Unit => LlValue::Unit,
            CompilerExpressionKind::Boolean(value) => LlValue::Boolean(value.to_string()),
            CompilerExpressionKind::Int(value) => LlValue::Int(value.to_string()),
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
                let operand = emitted.integer();
                LlValue::Int(body.instruction(
                    &format!("sub i64 0, {operand}"),
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
                        &format!("call fastcc i64 @{symbol}({arguments})"),
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

    fn emit_binary(
        &mut self,
        operation: CompilerBinary,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let instruction = match operation {
            CompilerBinary::Add => format!("add i64 {}, {}", left.integer(), right.integer()),
            CompilerBinary::Subtract => {
                format!("sub i64 {}, {}", left.integer(), right.integer())
            }
            CompilerBinary::Multiply => {
                format!("mul i64 {}, {}", left.integer(), right.integer())
            }
            CompilerBinary::And => {
                format!("and i1 {}, {}", left.boolean(), right.boolean())
            }
            CompilerBinary::Or => format!("or i1 {}, {}", left.boolean(), right.boolean()),
            CompilerBinary::Xor => {
                format!("xor i1 {}, {}", left.boolean(), right.boolean())
            }
            CompilerBinary::Equal | CompilerBinary::NotEqual => {
                let predicate = if operation == CompilerBinary::Equal {
                    "eq"
                } else {
                    "ne"
                };
                match (&left, &right) {
                    (LlValue::Unit, LlValue::Unit) => format!("icmp {predicate} i8 0, 0"),
                    (LlValue::Boolean(left), LlValue::Boolean(right)) => {
                        format!("icmp {predicate} i1 {left}, {right}")
                    }
                    (LlValue::Int(left), LlValue::Int(right)) => {
                        format!("icmp {predicate} i64 {left}, {right}")
                    }
                    _ => unreachable!("checked equality types agree"),
                }
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
                format!(
                    "icmp {predicate} i64 {}, {}",
                    left.integer(),
                    right.integer()
                )
            }
        };
        let value = body.instruction(&instruction, span, &mut self.debug);
        if matches!(
            operation,
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply
        ) {
            LlValue::Int(value)
        } else {
            LlValue::Boolean(value)
        }
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
                &format!("phi i64 [{left}, %{true_predecessor}], [{right}, %{false_predecessor}]"),
                span,
                &mut self.debug,
            )),
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
                &format!("call void @topal.runtime.print.i64(i64 {value})"),
                span,
                &mut self.debug,
            ),
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
}

#[derive(Clone)]
enum LlValue {
    Unit,
    Boolean(String),
    Int(String),
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

    fn argument(&self) -> String {
        match self {
            Self::Unit => "i8 0".into(),
            Self::Boolean(value) => format!("i1 {value}"),
            Self::Int(value) => format!("i64 {value}"),
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
            LlValue::Int(value) => format!("i64 {value}"),
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
        debug.int_type =
            debug.node("!DIBasicType(name: \"Int\", size: 64, encoding: DW_ATE_signed)".into());
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
        CompilerType::Int => "i64",
        _ => unreachable!("shared model restricts function ABI types"),
    }
}

fn llvm_parameter_type(value_type: &CompilerType) -> &'static str {
    match value_type {
        CompilerType::Unit => "i8",
        CompilerType::Boolean => "i1",
        CompilerType::Int => "i64",
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

const PLATFORM_RUNTIME: &str = r#"; topal.platform.linux-x86_64/1
define internal i64 @topal.platform.write(i64 %fd, ptr %buffer, i64 %length) nounwind noinline {
entry:
  %result = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 1, i64 %fd, ptr %buffer, i64 %length)
  ret i64 %result
}

define internal void @topal.platform.exit(i64 %status) noreturn nounwind noinline {
entry:
  %ignored = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},~{rcx},~{r11},~{memory}"(i64 60, i64 %status)
  unreachable
}

define internal void @topal.platform.write_all(ptr %buffer, i64 %length) nounwind noinline {
entry:
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %done, label %loop
loop:
  %offset = phi i64 [0, %entry], [%next, %progress], [%offset, %retry]
  %remaining = sub i64 %length, %offset
  %cursor = getelementptr i8, ptr %buffer, i64 %offset
  %written = call i64 @topal.platform.write(i64 1, ptr %cursor, i64 %remaining)
  %positive = icmp sgt i64 %written, 0
  br i1 %positive, label %progress, label %error
progress:
  %next = add i64 %offset, %written
  %complete = icmp eq i64 %next, %length
  br i1 %complete, label %done, label %loop
error:
  %interrupted = icmp eq i64 %written, -4
  br i1 %interrupted, label %retry, label %failure
retry:
  br label %loop
failure:
  call void @topal.platform.exit(i64 74)
  unreachable
done:
  ret void
}

define internal void @topal.runtime.print.i64(i64 %value) nounwind noinline {
entry:
  %buffer = alloca [20 x i8], align 1
  %negative = icmp slt i64 %value, 0
  %negated = sub i64 0, %value
  %magnitude = select i1 %negative, i64 %negated, i64 %value
  br i1 %negative, label %sign, label %digits.entry
sign:
  %minus = alloca i8, align 1
  store i8 45, ptr %minus, align 1
  call void @topal.platform.write_all(ptr %minus, i64 1)
  br label %digits.entry
digits.entry:
  %zero = icmp eq i64 %magnitude, 0
  br i1 %zero, label %zero.digit, label %digits.loop
zero.digit:
  %zero.pointer = getelementptr [20 x i8], ptr %buffer, i64 0, i64 19
  store i8 48, ptr %zero.pointer, align 1
  br label %emit
digits.loop:
  %current = phi i64 [%magnitude, %digits.entry], [%quotient, %digits.loop]
  %index = phi i64 [20, %digits.entry], [%next.index, %digits.loop]
  %next.index = sub i64 %index, 1
  %remainder = urem i64 %current, 10
  %digit = trunc i64 %remainder to i8
  %ascii = add i8 %digit, 48
  %digit.pointer = getelementptr [20 x i8], ptr %buffer, i64 0, i64 %next.index
  store i8 %ascii, ptr %digit.pointer, align 1
  %quotient = udiv i64 %current, 10
  %more = icmp ne i64 %quotient, 0
  br i1 %more, label %digits.loop, label %emit
emit:
  %start = phi i64 [19, %zero.digit], [%next.index, %digits.loop]
  %count = sub i64 20, %start
  %first = getelementptr [20 x i8], ptr %buffer, i64 0, i64 %start
  call void @topal.platform.write_all(ptr %first, i64 %count)
  ret void
}
"#;

#[cfg(test)]
mod tests {
    use topal_language::analyze_for_compiler;

    use super::*;

    #[test]
    fn emits_target_platform_runtime_and_debug_metadata() {
        let source = "use language (version is v0.1)\nvalue is 40 + 2\nproduct is 6 * 7\nordered is product >= value\n(value, product, ordered, true, \"Topal\")\n";
        let program = analyze_for_compiler(source).unwrap();
        let llvm = Generator::new(&program, "/source/example.t").emit();
        assert!(llvm.contains("target triple = \"x86_64-unknown-linux-gnu\""));
        assert!(llvm.contains("asm sideeffect \"syscall\""));
        assert!(llvm.contains("define void @_start() naked"));
        assert!(llvm.contains("andq $$-16, %rsp"));
        assert!(llvm.contains("@llvm.used"));
        assert!(llvm.contains("add i64 40, 2"));
        assert!(llvm.contains("mul i64 6, 7"));
        assert!(llvm.contains("icmp sge i64"));
        assert!(llvm.contains("\\54\\6F\\70\\61\\6C"));
        assert!(llvm.contains("#dbg_value"));
        assert!(llvm.contains("Dwarf Version"));
    }
}
