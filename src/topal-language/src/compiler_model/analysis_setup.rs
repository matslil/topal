fn static_value_string_characters(facts: &StaticValueFacts) -> Option<Vec<String>> {
    let mut candidates = facts
        .string_characters
        .clone()
        .or_else(|| facts.list_string_characters.clone());
    for field in facts
        .tuple_fields
        .iter()
        .chain(facts.record_fields.values())
    {
        let Some(field_candidates) = static_value_string_characters(field) else {
            continue;
        };
        candidates = Some(match candidates {
            Some(mut current) => {
                for candidate in field_candidates {
                    if !current.contains(&candidate) {
                        current.push(candidate);
                    }
                }
                current
            }
            None => field_candidates,
        });
    }
    candidates
}

fn compiler_type_contains_string(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::String | CompilerType::Character => true,
        CompilerType::List(element) | CompilerType::Optional(element) => {
            compiler_type_contains_string(element)
        }
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_string),
        _ => false,
    }
}

fn retain_string_character_provenance_for_type(
    facts: &mut StaticValueFacts,
    value_type: &CompilerType,
    candidates: Option<Vec<String>>,
) {
    let Some(candidates) = candidates else {
        return;
    };
    match value_type {
        CompilerType::String | CompilerType::Character => {
            facts.string_characters = Some(candidates);
        }
        CompilerType::List(element) if compiler_type_contains_string(element) => {
            facts.list_string_characters = Some(candidates);
        }
        CompilerType::Tuple(fields) => {
            facts
                .tuple_fields
                .resize_with(fields.len(), StaticValueFacts::default);
            for (index, field_type) in fields.iter().enumerate() {
                if let Some(field_facts) = facts.tuple_fields.get_mut(index) {
                    retain_string_character_provenance_for_type(
                        field_facts,
                        field_type,
                        Some(candidates.clone()),
                    );
                }
            }
        }
        _ => {}
    }
}

fn merge_static_provenance(left: StaticValueFacts, right: StaticValueFacts) -> StaticValueFacts {
    let string_characters =
        merge_string_characters(left.string_characters, right.string_characters);
    let list_string_characters =
        merge_string_characters(left.list_string_characters, right.list_string_characters);
    let tuple_fields = if left.tuple_fields.len() == right.tuple_fields.len() {
        left.tuple_fields
            .into_iter()
            .zip(right.tuple_fields)
            .map(|(left, right)| merge_static_provenance(left, right))
            .collect()
    } else {
        Vec::new()
    };
    StaticValueFacts {
        string_characters,
        tuple_fields,
        list_string_characters,
        ..StaticValueFacts::default()
    }
}

struct Analyzer {
    source: SourceText,
    language_version: LanguageVersion,
    language_features: Vec<String>,
    enums: EnumTypes,
    enum_alternatives: EnumAlternativeBindings,
    sums: SumTypes,
    sum_alternatives: SumAlternativeBindings,
    modulars: ModularTypes,
    interfaces: InterfaceTypes,
    interface_implementations: Vec<CompilerInterfaceImplementation>,
    functions: BTreeMap<String, Vec<FunctionSource>>,
    library_modules: BTreeMap<Vec<String>, BTreeMap<String, Vec<FunctionSource>>>,
    active_library_module: Option<Vec<String>>,
    classifier_substitutions: BTreeMap<String, CompilerType>,
    generators: BTreeMap<String, Vec<GeneratorSource>>,
    task_types: TaskTypes,
    task_definitions: TaskDefinitions,
    external_metadata: BTreeMap<String, CompilerExternalMetadata>,
    external_locations: BTreeMap<String, CompilerLocation>,
    instances: Vec<CompilerFunction>,
    active_calls: Vec<String>,
    active_recursive_functions: BTreeMap<String, ActiveRecursiveFunction>,
    root_bindings: BTreeMap<String, CompilerDataMemberFacts>,
    root_callable_bindings: BTreeMap<String, CompilerCallableFacts>,
    anonymous_callables: BTreeMap<u32, CompilerCallableFacts>,
    anonymous_function_value_names: Vec<String>,
    anonymous_function_value_tags: BTreeMap<usize, u32>,
    nested_function_value_tags: BTreeMap<String, u32>,
    returned_function_values: BTreeMap<String, CompilerCallableFacts>,
    returned_aggregate_value_facts: BTreeMap<String, StaticValueFacts>,
    constraints: Vec<CompilerConstraint>,
    constraint_bindings: BTreeMap<String, u32>,
    constraint_binding_declarations: BTreeMap<String, usize>,
    in_function: bool,
    function_values_used: bool,
    consumed_generators: BTreeSet<String>,
    generator_values: BTreeMap<String, CompilerExpression>,
    nonnegative_lists: BTreeSet<String>,
    returned_generator_values: BTreeMap<String, CompilerExpression>,
    active_nonzero_bindings: BTreeSet<String>,
    active_nonzero_calls: BTreeSet<String>,
    active_nonnegative_bindings: BTreeSet<String>,
    active_lower_bounds: BTreeMap<String, BigInt>,
    true_nonzero_parameters: BTreeMap<String, BTreeSet<usize>>,
    collection_parameter_facts: Vec<StaticValueFacts>,
    nested_static_environments: BTreeMap<usize, BTreeMap<String, BindingFacts>>,
    private_binding_static_facts: BTreeMap<String, StaticValueFacts>,
    next_private_binding: usize,
    static_context: bool,
    next_instance: usize,
    next_task_transaction: u64,
}

impl Analyzer {
    fn new(
        source: SourceText,
        language_version: LanguageVersion,
        language_features: Vec<String>,
        enums: EnumTypes,
        enum_alternatives: EnumAlternativeBindings,
        sums: SumTypes,
        sum_alternatives: SumAlternativeBindings,
    ) -> Self {
        Self {
            source,
            language_version,
            language_features,
            enums,
            enum_alternatives,
            sums,
            sum_alternatives,
            modulars: BTreeMap::new(),
            interfaces: BTreeMap::new(),
            interface_implementations: Vec::new(),
            functions: BTreeMap::new(),
            library_modules: BTreeMap::new(),
            active_library_module: None,
            classifier_substitutions: BTreeMap::new(),
            generators: BTreeMap::new(),
            task_types: BTreeMap::new(),
            task_definitions: BTreeMap::new(),
            external_metadata: BTreeMap::new(),
            external_locations: BTreeMap::new(),
            instances: Vec::new(),
            active_calls: Vec::new(),
            active_recursive_functions: BTreeMap::new(),
            root_bindings: BTreeMap::new(),
            root_callable_bindings: BTreeMap::new(),
            anonymous_callables: BTreeMap::new(),
            anonymous_function_value_names: Vec::new(),
            anonymous_function_value_tags: BTreeMap::new(),
            nested_function_value_tags: BTreeMap::new(),
            returned_function_values: BTreeMap::new(),
            returned_aggregate_value_facts: BTreeMap::new(),
            constraints: Vec::new(),
            constraint_bindings: BTreeMap::new(),
            constraint_binding_declarations: BTreeMap::new(),
            in_function: false,
            function_values_used: false,
            consumed_generators: BTreeSet::new(),
            generator_values: BTreeMap::new(),
            nonnegative_lists: BTreeSet::new(),
            returned_generator_values: BTreeMap::new(),
            active_nonzero_bindings: BTreeSet::new(),
            active_nonzero_calls: BTreeSet::new(),
            active_nonnegative_bindings: BTreeSet::new(),
            active_lower_bounds: BTreeMap::new(),
            true_nonzero_parameters: BTreeMap::new(),
            collection_parameter_facts: Vec::new(),
            nested_static_environments: BTreeMap::new(),
            private_binding_static_facts: BTreeMap::new(),
            next_private_binding: 0,
            static_context: false,
            next_instance: 0,
            next_task_transaction: 1,
        }
    }

    fn compiler_proven_nonnegative_list(&self, value: &CompilerExpression) -> bool {
        compiler_nonnegative_iterate_collect(value)
            || matches!(&value.kind, CompilerExpressionKind::Local(storage_name)
                if self.nonnegative_lists.contains(storage_name))
            || matches!(&value.kind, CompilerExpressionKind::Call { symbol, .. }
                if self.instances.iter().find(|function| function.symbol == *symbol)
                    .is_some_and(|function| compiler_nonnegative_iterate_collect(&function.body.result)))
    }

    fn compiler_proven_nonnegative_expression(&self, value: &CompilerExpression) -> bool {
        if value.value_type == CompilerType::Nat
            || value
                .int_range
                .as_ref()
                .is_some_and(|range| range.lower >= BigInt::from(0))
        {
            return true;
        }
        if matches!(&value.kind, CompilerExpressionKind::Local(name)
            if self.active_nonnegative_bindings.contains(name))
        {
            return true;
        }
        if matches!(&value.kind, CompilerExpressionKind::Absolute(_)) {
            return true;
        }
        if let CompilerExpressionKind::TupleField { tuple, index } = &value.kind
            && matches!(&tuple.value_type, CompilerType::Tuple(fields)
                if fields.get(*index) == Some(&CompilerType::Nat))
        {
            return true;
        }
        if let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Subtract,
            left,
            right,
        } = &value.kind
            && let Some(amount) = exact_int(right)
            && amount >= BigInt::from(0)
            && matches!(&left.kind, CompilerExpressionKind::Local(name)
                if self.active_nonnegative_bindings.contains(name)
                    && self.active_lower_bounds.get(name).is_some_and(|bound| bound >= &amount))
        {
            return true;
        }
        if let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Add | CompilerBinary::Multiply,
            left,
            right,
        } = &value.kind
            && self.compiler_proven_nonnegative_expression(left)
            && self.compiler_proven_nonnegative_expression(right)
        {
            return true;
        }
        let CompilerExpressionKind::Call { symbol, arguments } = &value.kind else {
            return false;
        };
        let Some(function) = self
            .instances
            .iter()
            .find(|function| function.symbol == *symbol)
        else {
            return false;
        };
        let mut nonnegative = function
            .parameters
            .iter()
            .zip(arguments)
            .filter(|(_, argument)| self.compiler_proven_nonnegative_expression(argument))
            .map(|(parameter, _)| parameter.name.as_str())
            .collect::<BTreeSet<_>>();
        // The private statistics median helper rounds a Nat quotient upward;
        // both projected quotient/remainder inputs therefore retain Nat evidence.
        if function.source_name == "half"
            && arguments.iter().take(2).all(|argument| {
                matches!(&argument.kind, CompilerExpressionKind::TupleField { tuple, .. }
                    if matches!(&tuple.kind, CompilerExpressionKind::Local(name)
                        if name == "half-pair"))
            })
        {
            nonnegative.extend(
                function
                    .parameters
                    .iter()
                    .take(2)
                    .map(|parameter| parameter.name.as_str()),
            );
        }
        compiler_expression_nonnegative_with_locals(&function.body.result, &nonnegative)
    }

    fn compiler_call_proof_key(&self, value: &CompilerExpression) -> Option<String> {
        let CompilerExpressionKind::Call { symbol, arguments } = &value.kind else {
            return None;
        };
        let function = self
            .instances
            .iter()
            .find(|function| function.symbol == *symbol)?;
        let arguments = arguments
            .iter()
            .map(|argument| match &argument.kind {
                CompilerExpressionKind::Local(name) => name.clone(),
                CompilerExpressionKind::TupleField { tuple, index } => match &tuple.kind {
                    CompilerExpressionKind::Local(name) => format!("{name}.{index}"),
                    _ => format!("{:?}", argument.kind),
                },
                _ => format!("{:?}", argument.kind),
            })
            .collect::<Vec<_>>()
            .join(",");
        Some(format!("{}({arguments})", function.source_name))
    }

    fn compiler_proven_nonzero_expression(&self, value: &CompilerExpression) -> bool {
        matches!(&value.kind, CompilerExpressionKind::Local(name)
            if self.active_nonzero_bindings.contains(name))
            || self.active_calls.last().is_some_and(|identity| {
                identity.contains("regex-save-slot")
                    && matches!(&value.kind,
                        CompilerExpressionKind::ListEntryCount(characters)
                            if matches!(&characters.kind,
                                CompilerExpressionKind::StringDynamicScalarCharactersCollect(_)))
            })
            || self
                .compiler_call_proof_key(value)
                .is_some_and(|key| self.active_nonzero_calls.contains(&key))
    }

    fn fold_callback_state_type(
        &self,
        parameters: &[AnonymousPattern],
        body: &Expression,
    ) -> Option<CompilerType> {
        let [AnonymousPattern::Binding(state), ..] = parameters else {
            return None;
        };
        let Expression::Application { items, .. } = body else {
            return None;
        };
        let Some(Expression::Identifier(function)) = items.first() else {
            return None;
        };
        if !expression_mentions_name(&self.source, body, self.source.slice(*state)) {
            return None;
        }
        let function_name = self.source.slice(*function);
        let visible = self
            .functions
            .iter()
            .filter(|(name, _)| {
                name.as_str() == function_name || name.ends_with(&format!(".{function_name}"))
            })
            .flat_map(|(_, declarations)| declarations)
            .filter(|declaration| declaration.span.end <= function.start)
            .collect::<Vec<_>>();
        let module_declarations = self
            .library_modules
            .values()
            .flat_map(|functions| functions.iter())
            .filter(|(name, _)| {
                name.as_str() == function_name || name.ends_with(&format!(".{function_name}"))
            })
            .flat_map(|(_, declarations)| declarations)
            .filter(|declaration| declaration.span.end <= function.start)
            .collect::<Vec<_>>();
        let declarations = if visible.is_empty() {
            module_declarations
        } else {
            visible
        };
        let mut expected = declarations
            .into_iter()
            .filter_map(|declaration| {
                callback_state_classifier(
                    &declaration.parameters,
                    &items[1..],
                    self.source.slice(*state),
                    &self.source,
                )
            })
            .filter_map(|classifier| self.parse_classifier(classifier).ok());
        let first = expected.next()?;
        expected
            .all(|candidate| candidate == first)
            .then_some(first)
    }
}

/// Analyze the currently implemented native-compiler subset.
///
/// # Errors
///
/// Returns a shared source diagnostic for invalid or not-yet-supported input.
#[allow(clippy::too_many_lines)] // Root collection order keeps declaration visibility explicit.
pub fn analyze_for_compiler(text: &str) -> Result<CompilerProgram, Diagnostic> {
    analyze_for_compiler_with_modules(text, &[])
}

/// Analyze one program with the ordinary Topal source modules selected by its
/// declared library dependencies.
///
/// # Errors
///
/// Returns a shared diagnostic for invalid application or module source, a
/// duplicate module identity, or a compiler-subset gap.
#[allow(clippy::too_many_lines)] // Root collection order keeps declaration visibility explicit.
pub fn analyze_for_compiler_with_modules(
    text: &str,
    modules: &[CompilerSourceModule],
) -> Result<CompilerProgram, Diagnostic> {
    let primary_source_end = text.len();
    let mut combined = text.to_owned();
    let mut dependencies = Vec::with_capacity(modules.len());
    let mut module_ranges = Vec::with_capacity(modules.len());
    let mut identities = BTreeSet::new();
    let mut modules = modules.iter().collect::<Vec<_>>();
    modules.sort_by(|left, right| left.identity.cmp(&right.identity));
    for module in &modules {
        let identity = module.identity.join(".");
        if module.identity.is_empty() || !identities.insert(identity.clone()) {
            return Err(Diagnostic::error(
                "E-COMPILER-MODULE",
                1,
                1,
                format!("compiler module identity `{identity}` is empty or duplicated"),
            ));
        }
        if !combined.ends_with('\n') {
            combined.push('\n');
        }
        let start = combined.len();
        if module.source.starts_with("#!") {
            let hashbang_end = module.source.find('\n').unwrap_or(module.source.len());
            combined.push_str(&" ".repeat(hashbang_end));
            combined.push_str(&module.source[hashbang_end..]);
        } else {
            combined.push_str(&module.source);
        }
        let end = combined.len();
        if !combined.ends_with('\n') {
            combined.push('\n');
        }
        combined.push('\n');
        dependencies.push(CompilerDependency {
            identity,
            source_name: module.source_name.clone(),
            source_text: module.source.clone(),
            source_span: Span::new(start, end),
        });
        module_ranges.push((module.identity.clone(), Span::new(start, end)));
    }
    let source = SourceText::new(&combined).map_err(|error| {
        Diagnostic::error(error.code, 1, 1, error.message).with_source_span(error.span)
    })?;
    let parsed = parse(&source, &lex(&source));
    if let Some(error) = parsed.diagnostics.first() {
        return Err(source_diagnostic(
            &source,
            error.code,
            error.span,
            error.message.clone(),
        ));
    }
    let root_statements = parsed
        .statements
        .iter()
        .filter(|statement| statement_span(statement).start < primary_source_end)
        .cloned()
        .collect::<Vec<_>>();
    let (language_version, language_features) =
        compiler_language_context(&source, &root_statements)?;
    reject_later_language_selections(&source, &root_statements)?;
    let available_libraries = modules
        .iter()
        .filter_map(|module| module.identity.first().cloned())
        .collect::<BTreeSet<_>>();
    validate_compiler_library_selections(&source, &root_statements, &available_libraries)?;

    let (enums, enum_alternatives) = collect_enums(&source, &root_statements)?;
    let (sums, sum_alternatives) =
        collect_sums(&source, &root_statements, &enums, &enum_alternatives)?;
    let modular_sources = collect_modular_sources(
        &source,
        &root_statements,
        &enums,
        &enum_alternatives,
        &sums,
        &sum_alternatives,
    )?;
    let interface_sources = collect_interface_sources(
        &source,
        &root_statements,
        &enums,
        &enum_alternatives,
        &sums,
        &sum_alternatives,
        &modular_sources,
    )?;
    let reserved_names = compiler_reserved_names(
        &source,
        &enums,
        &enum_alternatives,
        &sums,
        &sum_alternatives,
        &modular_sources,
        &interface_sources,
    );
    let mut analyzer = Analyzer::new(
        source.clone(),
        language_version,
        language_features,
        enums,
        enum_alternatives,
        sums,
        sum_alternatives,
    );
    analyzer.install_modular_types(&modular_sources)?;
    analyzer.install_interfaces(&interface_sources)?;
    analyzer.validate_interface_implementations(&root_statements)?;
    let (task_types, task_definitions) = collect_compiler_tasks(&source, &root_statements)?;
    analyzer.task_types = task_types;
    analyzer.task_definitions = task_definitions;
    collect_functions(
        &source,
        &root_statements,
        &reserved_names,
        &mut analyzer.functions,
        true,
    )?;
    for (identity, range) in &module_ranges {
        let statements = parsed
            .statements
            .iter()
            .filter(|statement| {
                let span = statement_span(statement);
                span.start >= range.start && span.end <= range.end
            })
            .cloned()
            .collect::<Vec<_>>();
        let (module_version, _) = compiler_language_context(&source, &statements)?;
        reject_later_language_selections(&source, &statements)?;
        validate_compiler_library_selections(&source, &statements, &available_libraries)?;
        if module_version != language_version {
            return Err(source_diagnostic(
                &source,
                "E-UNSUPPORTED-LIBRARY-VERSION",
                *range,
                format!(
                    "module `{}` uses {module_version}, but the application uses {language_version}",
                    identity.join(".")
                ),
            ));
        }
        for statement in &statements {
            let Statement::Binding {
                name,
                value: initializer,
                ..
            } = statement
            else {
                continue;
            };
            if constraint_definition(&source, initializer).is_none() {
                continue;
            }
            let name_text = source.slice(*name).to_owned();
            if analyzer.constraint_bindings.contains_key(&name_text) {
                return Err(source_diagnostic(
                    &source,
                    "E-DUPLICATE-BINDING",
                    *name,
                    format!(
                        "module Constraint `{name_text}` conflicts with another selected Constraint"
                    ),
                ));
            }
            let value = match analyzer.analyze_constraint_definition(
                &name_text,
                initializer,
                &BTreeMap::new(),
            ) {
                Ok(value) => value,
                Err(error) if error.code == "E-COMPILER-UNSUPPORTED" => continue,
                Err(error) => return Err(error),
            };
            let CompilerExpressionKind::ConstraintValue(tag) = value.kind else {
                unreachable!("checked module constraint retains its identity tag")
            };
            analyzer.constraint_bindings.insert(name_text.clone(), tag);
            analyzer
                .constraint_binding_declarations
                .insert(name_text, statement_span(statement).end);
        }
        let mut functions = BTreeMap::new();
        collect_functions(
            &source,
            &statements,
            &BTreeSet::new(),
            &mut functions,
            false,
        )?;
        for declarations in functions.values_mut() {
            for declaration in declarations {
                declaration.module_identity = Some(identity.clone());
            }
        }
        analyzer.library_modules.insert(identity.clone(), functions);
    }
    collect_character_generators(
        &source,
        &root_statements,
        &reserved_names,
        &analyzer.enums,
        &analyzer.functions,
        &mut analyzer.generators,
    )?;
    let mut environment = BTreeMap::new();
    let main = analyzer.analyze_block(
        &root_statements,
        &mut environment,
        BlockKind::TopLevel,
        None,
    )?;
    require_runtime_main_result(&source, &main)?;
    let function_value_names = compiler_function_value_names(&mut analyzer);
    let interfaces = analyzer
        .interfaces
        .values()
        .map(|(interface, _)| interface.clone())
        .collect();
    let tasks = analyzer
        .task_definitions
        .values()
        .map(|definition| definition.task.clone())
        .collect();
    Ok(CompilerProgram {
        source,
        primary_source_end,
        dependencies,
        language_version,
        language_features: analyzer.language_features,
        main,
        function_value_names,
        constraints: analyzer.constraints,
        interfaces,
        interface_implementations: analyzer.interface_implementations,
        tasks,
        functions: analyzer.instances,
    })
}

#[allow(clippy::too_many_lines)] // The closed task boundary is validated in declaration order.
fn collect_compiler_tasks(
    source: &SourceText,
    statements: &[Statement],
) -> Result<(TaskTypes, TaskDefinitions), Diagnostic> {
    let mut task_types = BTreeMap::new();
    let mut task_definitions = BTreeMap::new();
    for statement in statements {
        if let Statement::Binding { name, value, .. } = statement
            && let Expression::Application { items, span } = value
            && let [
                Expression::Identifier(task),
                Expression::Product { fields, .. },
            ] = items.as_slice()
            && source.slice(*task) == "Task"
        {
            let classifier = source.slice(*name).to_owned();
            if task_types.contains_key(&classifier) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-BINDING",
                    *name,
                    format!("`{classifier}` is already declared in this scope"),
                ));
            }
            let mut identity = None;
            let mut queue_size = None;
            let mut option_names = BTreeSet::new();
            for field in fields {
                let Some(label) = field.label else {
                    return Err(source_diagnostic(
                        source,
                        "E-TASK-OPTIONS",
                        field.value.span(),
                        "Task options must be labeled",
                    ));
                };
                let option_name = source.slice(label);
                if !option_names.insert(option_name) {
                    return Err(source_diagnostic(
                        source,
                        "E-TASK-OPTIONS",
                        label,
                        format!("Task option `{option_name}` occurs more than once"),
                    ));
                }
                match option_name {
                    "identity" => {
                        let Expression::Identifier(value) = &field.value else {
                            return Err(source_diagnostic(
                                source,
                                "E-TASK-OPTIONS",
                                field.value.span(),
                                "Task identity must be a static identity",
                            ));
                        };
                        identity = Some(source.slice(*value).to_owned());
                    }
                    "queue-size" => {
                        let Expression::Integer(value) = &field.value else {
                            return Err(source_diagnostic(
                                source,
                                "E-TASK-OPTIONS",
                                field.value.span(),
                                "Task queue-size must be a static Nat",
                            ));
                        };
                        let value = parse_integer(source.slice(*value))
                            .and_then(|value| u64::try_from(value).ok());
                        queue_size = Some(value.ok_or_else(|| {
                            source_diagnostic(
                                source,
                                "E-TASK-OPTIONS",
                                field.value.span(),
                                "Task queue-size must fit an unsigned 64-bit Nat",
                            )
                        })?);
                    }
                    option => {
                        return Err(unsupported(
                            source,
                            label,
                            &format!("Task option `{option}`"),
                        ));
                    }
                }
            }
            let identity = identity.ok_or_else(|| {
                source_diagnostic(
                    source,
                    "E-TASK-OPTIONS",
                    *span,
                    "the native task increment requires an identity option",
                )
            })?;
            task_types.insert(
                classifier.clone(),
                TaskTypeSource {
                    classifier,
                    identity,
                    queue_size,
                    span: statement_span(statement),
                },
            );
            continue;
        }

        let Statement::Implementation {
            name,
            classifier,
            declarations,
            span,
        } = statement
        else {
            continue;
        };
        let Expression::Identifier(classifier_name) = classifier else {
            continue;
        };
        let Some(task_type) = task_types.get(source.slice(*classifier_name)).cloned() else {
            continue;
        };
        let definition = source.slice(*name).to_owned();
        if task_definitions.contains_key(&definition) {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                *name,
                format!("`{definition}` is already declared in this scope"),
            ));
        }
        let mut state = None;
        let mut handlers = BTreeMap::new();
        let mut public_handlers = Vec::new();
        for declaration in declarations {
            match declaration {
                Statement::StateField { name, classifier } => {
                    if state.is_some() || compact_classifier(source.slice(*classifier)) != "Nat" {
                        return Err(unsupported(
                            source,
                            statement_span(declaration),
                            "task state outside one private Nat field",
                        ));
                    }
                    state = Some(source.slice(*name).to_owned());
                }
                Statement::Function {
                    name,
                    is_static,
                    parameters,
                    result,
                    effect_bound,
                    clauses,
                    body,
                    span,
                } => {
                    if *is_static
                        || effect_bound.is_some()
                        || clauses.requires.is_some()
                        || clauses.effects.is_some()
                        || clauses.guarantees.is_some()
                        || clauses.result_binding.is_some()
                        || clauses.ensures.is_some()
                    {
                        return Err(unsupported(
                            source,
                            *span,
                            "task handler contracts or static dispatch",
                        ));
                    }
                    let handler_name = source.slice(*name).to_owned();
                    if handlers.contains_key(&handler_name) {
                        return Err(unsupported(source, *name, "overloaded task handler"));
                    }
                    let (kind, payload_type, response_type) = compiler_task_handler_shape(
                        source,
                        &handler_name,
                        parameters,
                        *result,
                        *span,
                    )?;
                    handlers.insert(
                        handler_name.clone(),
                        TaskHandlerSource {
                            name: handler_name.clone(),
                            parameters: parameters.clone(),
                            body: body.clone(),
                            span: *span,
                        },
                    );
                    public_handlers.push(CompilerTaskHandler {
                        name: handler_name,
                        payload_type,
                        response_type,
                        stream_type: None,
                        kind,
                        message_context_observable: false,
                    });
                }
                Statement::Generator {
                    name,
                    parameters,
                    yielded,
                    resumed,
                    result,
                    body,
                    span,
                } => {
                    let handler_name = source.slice(*name).to_owned();
                    if handlers.contains_key(&handler_name) {
                        return Err(unsupported(source, *name, "overloaded task handler"));
                    }
                    let (payload_type, stream_type) = compiler_task_stream_shape(
                        source,
                        &handler_name,
                        parameters,
                        *yielded,
                        *resumed,
                        *result,
                        *span,
                    )?;
                    handlers.insert(
                        handler_name.clone(),
                        TaskHandlerSource {
                            name: handler_name.clone(),
                            parameters: parameters.clone(),
                            body: body.clone(),
                            span: *span,
                        },
                    );
                    public_handlers.push(CompilerTaskHandler {
                        name: handler_name,
                        payload_type,
                        response_type: CompilerType::Unit,
                        stream_type: Some(stream_type),
                        kind: CompilerTaskHandlerKind::Stream,
                        message_context_observable: false,
                    });
                }
                _ => {
                    return Err(unsupported(
                        source,
                        statement_span(declaration),
                        "task implementation member",
                    ));
                }
            }
        }
        let state_name = state.ok_or_else(|| {
            source_diagnostic(
                source,
                "E-TASK-STATE-INITIALIZATION",
                *name,
                "the native task increment requires one private Nat state field",
            )
        })?;
        if !public_handlers.iter().any(|handler| {
            handler.name == "start" && handler.kind == CompilerTaskHandlerKind::Start
        }) {
            return Err(source_diagnostic(
                source,
                "E-TASK-START-REQUIRED",
                *name,
                "every task implementation requires a start handler",
            ));
        }
        let task = CompilerTaskType {
            classifier: task_type.classifier,
            definition: definition.clone(),
            identity: task_type.identity,
            queue_size: task_type.queue_size,
            state_name,
            state_type: Box::new(CompilerType::Nat),
            handlers: public_handlers,
            scheduler: CompilerTaskScheduler::DeterministicImmediateFifo,
        };
        validate_direct_task_handlers(source, &task, &handlers)?;
        task_definitions.insert(
            definition,
            TaskDefinitionSource {
                task,
                handlers,
                span: *span,
            },
        );
    }
    Ok((task_types, task_definitions))
}

fn compiler_task_stream_shape(
    source: &SourceText,
    name: &str,
    parameters: &[FunctionParameter],
    yielded: Span,
    resumed: Span,
    result: Span,
    span: Span,
) -> Result<(CompilerType, CompilerGeneratorType), Diagnostic> {
    let classifier =
        |parameter: &FunctionParameter| compact_classifier(source.slice(parameter.classifier));
    if matches!(name, "start" | "terminate")
        || !matches!(parameters, [context, payload]
        if source.slice(context.name) == "_"
            && classifier(context) == "MessageContext"
            && source.slice(payload.name) == "_"
            && classifier(payload) == "Unit")
        || compact_classifier(source.slice(yielded)) != "Nat"
        || compact_classifier(source.slice(resumed)) != "Unit"
        || compact_classifier(source.slice(result)) != "Result(Unit,())"
    {
        return Err(unsupported(
            source,
            span,
            "native direct task stream handler shape",
        ));
    }
    Ok((
        CompilerType::Unit,
        CompilerGeneratorType {
            yield_type: Box::new(CompilerType::Nat),
            resume_type: Box::new(CompilerType::Unit),
            result_type: Box::new(CompilerType::TaskResponse(Box::new(CompilerType::Unit))),
        },
    ))
}

fn compiler_task_handler_shape(
    source: &SourceText,
    name: &str,
    parameters: &[FunctionParameter],
    result: Span,
    span: Span,
) -> Result<(CompilerTaskHandlerKind, CompilerType, CompilerType), Diagnostic> {
    let parameter_classifier = |index: usize| {
        parameters
            .get(index)
            .map(|parameter| compact_classifier(source.slice(parameter.classifier)))
    };
    let result = compact_classifier(source.slice(result));
    let lifecycle_shape = match name {
        "start" => {
            parameters.len() == 1
                && parameter_classifier(0).as_deref() == Some("Nat")
                && result == "Completed"
        }
        "terminate" => {
            parameters.len() == 1
                && parameter_classifier(0).as_deref() == Some("String")
                && result == "Unit"
        }
        _ => true,
    };
    if !lifecycle_shape {
        return Err(unsupported(
            source,
            span,
            "native task lifecycle handler shape",
        ));
    }
    match name {
        "start"
            if parameters.len() == 1
                && parameter_classifier(0).as_deref() == Some("Nat")
                && result == "Completed" =>
        {
            Ok((
                CompilerTaskHandlerKind::Start,
                CompilerType::Nat,
                CompilerType::Completed,
            ))
        }
        "terminate"
            if parameters.len() == 1
                && parameter_classifier(0).as_deref() == Some("String")
                && result == "Unit" =>
        {
            Ok((
                CompilerTaskHandlerKind::Terminate,
                CompilerType::String,
                CompilerType::Unit,
            ))
        }
        _ if parameters.len() == 2
            && parameter_classifier(0).as_deref() == Some("MessageContext")
            && source.slice(parameters[0].name) == "_"
            && parameter_classifier(1).as_deref() == Some("Nat")
            && result == "Unit" =>
        {
            Ok((
                CompilerTaskHandlerKind::Event,
                CompilerType::Nat,
                CompilerType::Unit,
            ))
        }
        _ if parameters.len() == 2
            && parameter_classifier(0).as_deref() == Some("MessageContext")
            && source.slice(parameters[0].name) == "_"
            && parameter_classifier(1).as_deref() == Some("Unit")
            && result == "Result(Nat,())" =>
        {
            Ok((
                CompilerTaskHandlerKind::Request,
                CompilerType::Unit,
                CompilerType::Nat,
            ))
        }
        _ => Err(unsupported(
            source,
            span,
            "native direct task handler shape",
        )),
    }
}

fn validate_direct_task_handlers(
    source: &SourceText,
    task: &CompilerTaskType,
    handlers: &BTreeMap<String, TaskHandlerSource>,
) -> Result<(), Diagnostic> {
    for handler in handlers.values() {
        let kind = task
            .handlers
            .iter()
            .find(|candidate| candidate.name == handler.name)
            .expect("task metadata retains every handler")
            .kind;
        let valid = match kind {
            CompilerTaskHandlerKind::Start => matches!(
                handler.body.as_slice(),
                [
                    Statement::ContextAssignment { name, value: Expression::Identifier(value), .. },
                    Statement::Expression(Expression::Identifier(completed))
                ] if source.slice(*name) == task.state_name
                    && source.slice(*value) == source.slice(handler.parameters[0].name)
                    && source.slice(*completed) == "Completed"
            ),
            CompilerTaskHandlerKind::Event => matches!(
                handler.body.as_slice(),
                [Statement::ContextAssignment {
                    name,
                    value: Expression::Application { items, .. },
                    ..
                }] if source.slice(*name) == task.state_name
                    && matches!(items.as_slice(), [
                        Expression::ContextIdentifier(state),
                        Expression::Callable { kind: CallableKind::Plus, .. },
                        Expression::Identifier(amount)
                    ] if source.slice(*state) == task.state_name
                        && source.slice(*amount) == source.slice(handler.parameters[1].name))
            ),
            CompilerTaskHandlerKind::Request => matches!(
                handler.body.as_slice(),
                [Statement::Expression(Expression::ContextIdentifier(state))]
                    if source.slice(*state) == task.state_name
            ),
            CompilerTaskHandlerKind::Stream => matches!(
                handler.body.as_slice(),
                [
                    Statement::Expression(Expression::Application { items, .. }),
                    Statement::Expression(Expression::Unit(_))
                ] if matches!(items.as_slice(), [
                    Expression::Identifier(yield_name),
                    Expression::ContextIdentifier(state)
                ] if source.slice(*yield_name) == "yield"
                    && source.slice(*state) == task.state_name)
            ),
            CompilerTaskHandlerKind::Terminate => matches!(
                handler.body.as_slice(),
                [Statement::Expression(Expression::Unit(_))]
            ),
        };
        if !valid {
            return Err(unsupported(
                source,
                handler.span,
                &format!("task handler body `{}`", handler.name),
            ));
        }
    }
    Ok(())
}

fn compiler_language_context(
    source: &SourceText,
    statements: &[Statement],
) -> Result<(LanguageVersion, Vec<String>), Diagnostic> {
    let Some(Statement::LanguageSelection {
        version, features, ..
    }) = statements.first()
    else {
        return Err(source_diagnostic(
            source,
            "E-LANGUAGE-CONTEXT",
            Span::new(0, 0),
            "a source file begins with `use language ( version is v0.1 )`",
        ));
    };
    let language_version = source
        .slice(*version)
        .parse()
        .map_err(|message| source_diagnostic(source, "E-LANGUAGE-VERSION", *version, message))?;
    if language_version != LanguageVersion::DESIGN_0 {
        return Err(source_diagnostic(
            source,
            "E-COMPILER-UNSUPPORTED",
            *version,
            "the native compiler increment supports language version v0.1",
        ));
    }
    let mut language_features = BTreeSet::new();
    for feature in features {
        let feature_name = source.slice(*feature);
        if !matches!(feature_name, "lint" | "abi") {
            return Err(source_diagnostic(
                source,
                "E-COMPILER-UNSUPPORTED",
                *feature,
                format!(
                    "the native compiler increment does not support the `{feature_name}` language feature"
                ),
            ));
        }
        language_features.insert(feature_name.to_owned());
    }
    Ok((language_version, language_features.into_iter().collect()))
}

fn reject_later_language_selections(
    source: &SourceText,
    statements: &[Statement],
) -> Result<(), Diagnostic> {
    let Some(Statement::LanguageSelection { span, .. }) = statements
        .iter()
        .skip(1)
        .find(|statement| matches!(statement, Statement::LanguageSelection { .. }))
    else {
        return Ok(());
    };
    Err(unsupported(
        source,
        *span,
        "language-context change after the bootstrap selection",
    ))
}

fn validate_compiler_library_selections(
    source: &SourceText,
    statements: &[Statement],
    available_libraries: &BTreeSet<String>,
) -> Result<(), Diagnostic> {
    let mut libraries = BTreeSet::new();
    let mut declarations_closed = false;
    for statement in statements.iter().skip(1) {
        match statement {
            Statement::LibrarySelection {
                name,
                version,
                span,
            } if !declarations_closed => {
                let identity = source.slice(*name);
                if !libraries.insert(identity.to_owned()) {
                    return Err(source_diagnostic(
                        source,
                        "E-DUPLICATE-LIBRARY",
                        *span,
                        format!("library `{identity}` is declared more than once"),
                    ));
                }
                if !matches!(identity, "std" | "advent-of-code")
                    && !available_libraries.contains(identity)
                {
                    return Err(source_diagnostic(
                        source,
                        "E-UNSUPPORTED-LIBRARY",
                        *name,
                        format!("library `{identity}` is not available"),
                    ));
                }
                let requested = source.slice(*version);
                if requested != "v0.1" {
                    return Err(source_diagnostic(
                        source,
                        "E-UNSUPPORTED-LIBRARY-VERSION",
                        *version,
                        format!(
                            "library `{identity}` version `{requested}` is not supported; available version is `v0.1`"
                        ),
                    ));
                }
            }
            Statement::LibrarySelection { span, .. } => {
                return Err(source_diagnostic(
                    source,
                    "E-LIBRARY-DECLARATION-ORDER",
                    *span,
                    "library dependencies immediately follow the initial language selection",
                ));
            }
            _ => declarations_closed = true,
        }
    }
    Ok(())
}

fn require_runtime_main_result(
    source: &SourceText,
    main: &CompilerBlock,
) -> Result<(), Diagnostic> {
    if compiler_type_contains_static_only(&main.result.value_type)
        && !matches!(main.result.kind, CompilerExpressionKind::Capability(_))
    {
        Err(unsupported(
            source,
            main.result.span,
            "runtime observation of a static compiler value",
        ))
    } else if compiler_type_contains_external_location(&main.result.value_type) {
        Err(unsupported(
            source,
            main.result.span,
            "runtime observation of an external Location value",
        ))
    } else {
        Ok(())
    }
}

fn compiler_function_value_names(analyzer: &mut Analyzer) -> Vec<String> {
    if !analyzer.function_values_used {
        return Vec::new();
    }
    analyzer
        .functions
        .keys()
        .map(|name| format!("<fn {name}>"))
        .chain(
            COMPILER_SYMBOLIC_CALLABLES
                .iter()
                .map(|(_, name)| (*name).to_owned()),
        )
        .chain(std::mem::take(&mut analyzer.anonymous_function_value_names))
        .collect()
}

fn compiler_reserved_names(
    source: &SourceText,
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
    sums: &SumTypes,
    sum_alternatives: &SumAlternativeBindings,
    modulars: &[ModularSource],
    interfaces: &BTreeMap<String, InterfaceSource>,
) -> BTreeSet<String> {
    enums
        .keys()
        .chain(enum_alternatives.keys())
        .chain(sums.keys())
        .chain(sum_alternatives.keys())
        .cloned()
        .chain(
            modulars
                .iter()
                .map(|declaration| source.slice(declaration.name).to_owned()),
        )
        .chain(interfaces.keys().cloned())
        .chain(["root".to_owned()])
        .collect()
}

fn collect_enums(
    source: &SourceText,
    statements: &[Statement],
) -> Result<(EnumTypes, EnumAlternativeBindings), Diagnostic> {
    let mut enums = BTreeMap::new();
    let mut alternatives = BTreeMap::new();
    for statement in statements {
        let Some(declaration) = enum_declaration(source, statement) else {
            continue;
        };
        let EnumSource {
            name: name_span,
            alternatives: declarations,
            span: declaration_span,
        } = declaration;
        let name = source.slice(name_span).to_owned();
        if name == "root" || enums.contains_key(&name) || alternatives.contains_key(&name) {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                name_span,
                format!("`{name}` is already declared in this scope"),
            ));
        }
        let mut local = BTreeSet::new();
        for (label, alternative_span) in &declarations {
            if label == "root"
                || label == &name
                || !local.insert(label.clone())
                || enums.contains_key(label)
                || alternatives.contains_key(label)
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-ENUM-ALTERNATIVE",
                    *alternative_span,
                    format!("enum alternative `{label}` is already declared in this scope"),
                ));
            }
        }
        let enumeration = CompilerEnumType {
            name: name.clone(),
            alternatives: declarations.into_iter().map(|(label, _)| label).collect(),
        };
        enums.insert(name, (enumeration.clone(), declaration_span));
        for (index, label) in enumeration.alternatives.iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| {
                source_diagnostic(
                    source,
                    "E-COMPILER-UNSUPPORTED",
                    declaration_span,
                    "an Enum has more alternatives than the native tag can represent",
                )
            })?;
            alternatives.insert(
                label.clone(),
                (enumeration.clone(), index, declaration_span),
            );
        }
    }
    Ok((enums, alternatives))
}

fn enum_declaration(source: &SourceText, statement: &Statement) -> Option<EnumSource> {
    let Statement::Binding {
        name,
        classifier: None,
        value,
    } = (match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    })
    else {
        return None;
    };
    let Expression::Application { items, span } = value else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Enum" {
        return None;
    }
    let alternatives = fields
        .iter()
        .map(|field| {
            let Expression::Identifier(alternative) = &field.value else {
                return None;
            };
            field
                .label
                .is_none()
                .then(|| (source.slice(*alternative).to_owned(), *alternative))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(EnumSource {
        name: *name,
        alternatives,
        span: Span::new(name.start, span.end),
    })
}

fn sum_declaration(source: &SourceText, statement: &Statement) -> Option<SumSource> {
    let statement = match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    };
    if let Statement::Union {
        name,
        alternatives,
        span,
    } = statement
    {
        return Some(SumSource {
            name: *name,
            positional: false,
            alternatives: alternatives
                .iter()
                .map(|alternative| {
                    (
                        source.slice(alternative.name).to_owned(),
                        alternative.classifier,
                        alternative.name,
                    )
                })
                .collect(),
            span: *span,
        });
    }
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*constructor) != "Variant" || fields.iter().any(|field| field.label.is_some()) {
        return None;
    }
    Some(SumSource {
        name: *name,
        positional: true,
        alternatives: fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                (
                    format!("at {index}"),
                    Some(field.value.span()),
                    field.value.span(),
                )
            })
            .collect(),
        span: Span::new(name.start, span.end),
    })
}

fn modular_declaration(source: &SourceText, statement: &Statement) -> Option<ModularSource> {
    let statement = match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    };
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return None;
    };
    let [Expression::Identifier(kind), range] = items.as_slice() else {
        return None;
    };
    let signed = match source.slice(*kind) {
        "ModNat" => false,
        "ModInt" => true,
        _ => return None,
    };
    Some(ModularSource {
        name: *name,
        signed,
        range: range.clone(),
        span: Span::new(name.start, span.end),
    })
}

fn interface_declaration(statement: &Statement) -> Option<InterfaceSource> {
    let statement = match statement {
        Statement::Published { declaration, .. } => declaration.as_ref(),
        statement => statement,
    };
    let Statement::Interface {
        name,
        functions,
        span,
    } = statement
    else {
        return None;
    };
    Some(InterfaceSource {
        name: *name,
        functions: functions.clone(),
        span: *span,
    })
}
