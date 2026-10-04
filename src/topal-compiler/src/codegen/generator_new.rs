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
            foreign_declarations: BTreeSet::new(),
            next_global: 0,
            needs_infinity_result_runtime: false,
            scalar_list_runtime_fragments: BTreeSet::new(),
            list_int_runtime_fragments: BTreeSet::new(),
            current_result_captures: Vec::new(),
            current_function_return_type: None,
            debug,
        }
    }

    fn close_list_runtime_dependencies(&mut self) {
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::NestedIntStringCore)
        {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::IntStringPair);
        }
        if self
            .list_int_runtime_fragments
            .contains(&ListIntRuntimeFragment::NestedIntCore)
        {
            self.list_int_runtime_fragments
                .insert(ListIntRuntimeFragment::Core);
            self.list_int_runtime_fragments
                .insert(ListIntRuntimeFragment::Sequence);
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
            "@llvm.used = appending global [3 x ptr] [ptr @topal.main, ptr @topal.platform.exit, ptr @memset], section \"llvm.metadata\"\n",
        );
        module.push('\n');
        module.push_str(PLATFORM_RUNTIME);
        module.push('\n');
        self.emit_foreign_declarations(&mut module);
        if self.needs_infinity_result_runtime {
            module.push_str(INFINITY_RESULT_RUNTIME);
            module.push('\n');
        }
        self.close_list_runtime_dependencies();
        self.emit_scalar_list_runtimes(&mut module);
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
            .contains(&ListIntRuntimeFragment::NestedIntCore)
        {
            module.push_str(LIST_NESTED_INT_CORE_RUNTIME);
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

    fn emit_foreign_declarations(&self, module: &mut String) {
        for declaration in &self.foreign_declarations {
            let _ = writeln!(module, "{declaration}");
        }
        if !self.foreign_declarations.is_empty() {
            module.push('\n');
        }
    }

    fn emit_scalar_list_runtimes(&self, module: &mut String) {
        for (fragment, runtime) in [
            (ScalarListRuntimeFragment::Unit, LIST_UNIT_CORE_RUNTIME),
            (
                ScalarListRuntimeFragment::Completed,
                LIST_COMPLETED_CORE_RUNTIME,
            ),
            (ScalarListRuntimeFragment::Effect, LIST_EFFECT_CORE_RUNTIME),
            (ScalarListRuntimeFragment::Type, LIST_TYPE_CORE_RUNTIME),
            (ScalarListRuntimeFragment::Enum, LIST_ENUM_CORE_RUNTIME),
            (
                ScalarListRuntimeFragment::Modular,
                LIST_MODULAR_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::OptionalInt,
                LIST_OPTIONAL_INT_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::OptionalRational,
                LIST_OPTIONAL_RATIONAL_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::OptionalString,
                LIST_OPTIONAL_STRING_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::IntPair,
                LIST_INT_PAIR_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::IntTriple,
                LIST_INT_TRIPLE_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::IntIntStringIntTuple,
                LIST_INT_INT_STRING_INT_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::IntIntBooleanPair,
                LIST_INT_INT_BOOLEAN_PAIR_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::IntRange,
                LIST_INT_RANGE_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::IntStringPair,
                LIST_INT_STRING_PAIR_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::StringIntPair,
                LIST_STRING_INT_PAIR_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::StringPair,
                LIST_STRING_PAIR_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::BooleanStringPair,
                LIST_BOOLEAN_STRING_PAIR_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::Boolean,
                LIST_BOOLEAN_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::Comparison,
                LIST_COMPARISON_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::ErrorCode,
                LIST_ERROR_CODE_CORE_RUNTIME,
            ),
            (
                ScalarListRuntimeFragment::Rational,
                LIST_RATIONAL_CORE_RUNTIME,
            ),
            (ScalarListRuntimeFragment::String, LIST_STRING_CORE_RUNTIME),
        ] {
            if self.scalar_list_runtime_fragments.contains(&fragment) {
                module.push_str(runtime);
                module.push('\n');
            }
        }
    }

    #[allow(clippy::if_not_else, clippy::too_many_lines)] // Capture returns precede exhaustive ordinary returns.
    fn emit_function(&mut self, function: &CompilerFunction) {
        if function.foreign.is_some() {
            self.emit_foreign_function(function);
            return;
        }
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

    fn emit_foreign_function(&mut self, function: &CompilerFunction) {
        let foreign = function
            .foreign
            .as_ref()
            .expect("foreign adapter retains checked C metadata");
        debug_assert_eq!(foreign.parameters.len(), function.parameters.len());
        let external_result = match foreign.result {
            CompilerCValue::Void => "void",
            CompilerCValue::SignedInt32 => "i32",
        };
        let external_parameters = foreign
            .parameters
            .iter()
            .map(|value| match value {
                CompilerCValue::SignedInt32 => "i32",
                CompilerCValue::Void => unreachable!("validated C parameters are non-void"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.foreign_declarations.insert(format!(
            "declare {external_result} @{}({external_parameters})",
            foreign.external_symbol
        ));

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
        let mut c_arguments = Vec::with_capacity(function.parameters.len());
        for (index, value) in foreign.parameters.iter().enumerate() {
            match value {
                CompilerCValue::SignedInt32 => c_arguments.push(body.instruction(
                    &format!("call i32 @topal.runtime.int.to.c.i32(ptr %arg{index})"),
                    function.parameters[index].span,
                    &mut self.debug,
                )),
                CompilerCValue::Void => unreachable!("validated C parameters are non-void"),
            }
        }
        let arguments = c_arguments
            .iter()
            .map(|argument| format!("i32 {argument}"))
            .collect::<Vec<_>>()
            .join(", ");
        let location = self.debug.location(function.body.result.span, subprogram);
        match foreign.result {
            CompilerCValue::Void => {
                body.effect(
                    &format!("call void @{}({arguments})", foreign.external_symbol),
                    function.body.result.span,
                    &mut self.debug,
                );
                body.terminator("ret void", location);
            }
            CompilerCValue::SignedInt32 => {
                let result = body.instruction(
                    &format!("call i32 @{}({arguments})", foreign.external_symbol),
                    function.body.result.span,
                    &mut self.debug,
                );
                let extended = body.instruction(
                    &format!("sext i32 {result} to i64"),
                    function.body.result.span,
                    &mut self.debug,
                );
                let adapted = body.instruction(
                    &format!("call ptr @topal.runtime.int.from.i64(i64 {extended})"),
                    function.body.result.span,
                    &mut self.debug,
                );
                body.terminator(&format!("ret ptr {adapted}"), location);
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
                || matches!(&parameter.value_type, CompilerType::List(element)
                if matches!(element.as_ref(), CompilerType::Tuple(fields)
                        if matches!(fields.as_slice(), [
                            CompilerType::Int | CompilerType::String,
                            CompilerType::Int | CompilerType::String
                        ])))
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
                        match base.as_ref() {
                            CompilerType::Boolean => {
                                body.debug_value(&value, variable, location);
                            }
                            CompilerType::Int | CompilerType::Nat => {
                                self.emit_aggregate_debug_shadow(
                                    value.integer(),
                                    &binding.value.value_type,
                                    variable,
                                    location,
                                    body,
                                    binding.span,
                                );
                            }
                            CompilerType::Rational => {
                                self.emit_aggregate_debug_shadow(
                                    value.rational(),
                                    &binding.value.value_type,
                                    variable,
                                    location,
                                    body,
                                    binding.span,
                                );
                            }
                            CompilerType::String => {
                                self.emit_aggregate_debug_shadow(
                                    value.string(),
                                    &binding.value.value_type,
                                    variable,
                                    location,
                                    body,
                                    binding.span,
                                );
                            }
                            _ => unreachable!("checked refined debug base is supported"),
                        }
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
}
