impl Analyzer {
    fn adapt_structural_equality(
        &self,
        mut left: CompilerExpression,
        mut right: CompilerExpression,
        span: Span,
    ) -> Result<(CompilerExpression, CompilerExpression, CompilerType), Diagnostic> {
        if is_exact_comparable(&left.value_type) && is_exact_comparable(&right.value_type) {
            left = forget_nat_evidence(left);
            right = forget_nat_evidence(right);
            let rational_domain = matches!(
                left.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            ) || matches!(
                right.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            );
            if rational_domain
                && (matches!(
                    left.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ) || matches!(
                    right.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ))
            {
                return Err(self.no_structural_comparison(
                    span,
                    "cross-domain infinity conversion is not defined",
                ));
            }
            let value_type = if rational_domain {
                left = into_rational(left);
                right = into_rational(right);
                CompilerType::Rational
            } else {
                CompilerType::Int
            };
            return Ok((left, right, value_type));
        }
        if left.value_type == CompilerType::Character && right.value_type == CompilerType::String {
            left = forget_character_evidence(left);
        } else if left.value_type == CompilerType::String
            && right.value_type == CompilerType::Character
        {
            right = forget_character_evidence(right);
        }
        if left.value_type != right.value_type {
            if let Some(adapted) = adapt_function_call_argument(&left.value_type, &right) {
                right = adapted;
            } else if let Some(adapted) = adapt_function_call_argument(&right.value_type, &left) {
                left = adapted;
            }
        }
        if left.value_type == right.value_type {
            if compiler_equality_supported(&left.value_type) {
                let value_type = left.value_type.clone();
                return Ok((left, right, value_type));
            }
            return Err(
                self.no_structural_comparison(span, "corresponding fields have no common equality")
            );
        }

        match (left.value_type.clone(), right.value_type.clone()) {
            (CompilerType::Tuple(left_types), CompilerType::Tuple(right_types))
                if left_types.len() == right_types.len() =>
            {
                let (
                    CompilerExpressionKind::Tuple(left_fields),
                    CompilerExpressionKind::Tuple(right_fields),
                ) = (&mut left.kind, &mut right.kind)
                else {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "structural equality conversion on an opaque tuple",
                    ));
                };
                let mut field_types = Vec::with_capacity(left_fields.len());
                for index in 0..left_fields.len() {
                    let (left_field, right_field, field_type) = self.adapt_structural_equality(
                        left_fields[index].clone(),
                        right_fields[index].clone(),
                        span,
                    )?;
                    left_fields[index] = left_field;
                    right_fields[index] = right_field;
                    field_types.push(field_type);
                }
                let value_type = CompilerType::Tuple(field_types);
                left.value_type = value_type.clone();
                right.value_type = value_type.clone();
                Ok((left, right, value_type))
            }
            (CompilerType::Record(left_types), CompilerType::Record(right_types)) => {
                self.adapt_record_equality(left, right, left_types, &right_types, span)
            }
            _ => {
                Err(self
                    .no_structural_comparison(span, "corresponding fields have no common equality"))
            }
        }
    }

    fn adapt_record_equality(
        &self,
        mut left: CompilerExpression,
        mut right: CompilerExpression,
        left_types: Vec<(String, CompilerType)>,
        right_types: &[(String, CompilerType)],
        span: Span,
    ) -> Result<(CompilerExpression, CompilerExpression, CompilerType), Diagnostic> {
        let left_labels = left_types
            .iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>();
        let right_labels = right_types
            .iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>();
        if left_labels != right_labels {
            return Err(self.no_structural_comparison(span, "record shapes differ"));
        }
        let (
            CompilerExpressionKind::Record(left_fields),
            CompilerExpressionKind::Record(right_fields),
        ) = (&mut left.kind, &mut right.kind)
        else {
            return Err(unsupported(
                &self.source,
                span,
                "structural equality conversion on an opaque record",
            ));
        };
        let mut field_types = Vec::with_capacity(left_types.len());
        for (label, _) in left_types {
            let left_index = left_fields
                .iter()
                .position(|(name, _)| name == &label)
                .expect("checked left Record retains its type labels");
            let right_index = right_fields
                .iter()
                .position(|(name, _)| name == &label)
                .expect("checked right Record retains its type labels");
            let (left_field, right_field, field_type) = self.adapt_structural_equality(
                left_fields[left_index].1.clone(),
                right_fields[right_index].1.clone(),
                span,
            )?;
            left_fields[left_index].1 = left_field;
            right_fields[right_index].1 = right_field;
            field_types.push((label, field_type));
        }
        let value_type = CompilerType::Record(field_types);
        left.value_type = value_type.clone();
        right.value_type = value_type.clone();
        Ok((left, right, value_type))
    }

    fn adapt_tuple_ordering(
        &self,
        mut left: CompilerExpression,
        mut right: CompilerExpression,
        span: Span,
    ) -> Result<(CompilerExpression, CompilerExpression, CompilerType), Diagnostic> {
        if is_exact_comparable(&left.value_type) && is_exact_comparable(&right.value_type) {
            left = forget_nat_evidence(left);
            right = forget_nat_evidence(right);
            let rational_domain = matches!(
                left.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            ) || matches!(
                right.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            );
            if rational_domain
                && (matches!(
                    left.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ) || matches!(
                    right.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ))
            {
                return Err(self.no_structural_comparison(
                    span,
                    "cross-domain infinity conversion is not defined",
                ));
            }
            let value_type = if rational_domain {
                left = into_rational(left);
                right = into_rational(right);
                CompilerType::Rational
            } else {
                CompilerType::Int
            };
            return Ok((left, right, value_type));
        }
        if left.value_type == right.value_type {
            if compiler_ordering_supported(&left.value_type) {
                let value_type = left.value_type.clone();
                return Ok((left, right, value_type));
            }
            return Err(self.no_structural_comparison(
                span,
                "corresponding fields have no common total order",
            ));
        }
        let (CompilerType::Tuple(left_types), CompilerType::Tuple(right_types)) =
            (left.value_type.clone(), right.value_type.clone())
        else {
            return Err(self.no_structural_comparison(
                span,
                "corresponding fields have no common total order",
            ));
        };
        if left_types.len() != right_types.len() {
            return Err(self.no_structural_comparison(span, "tuple arities differ"));
        }
        let (
            CompilerExpressionKind::Tuple(left_fields),
            CompilerExpressionKind::Tuple(right_fields),
        ) = (&mut left.kind, &mut right.kind)
        else {
            return Err(unsupported(
                &self.source,
                span,
                "structural ordering conversion on an opaque tuple",
            ));
        };
        let mut field_types = Vec::with_capacity(left_fields.len());
        for index in 0..left_fields.len() {
            let (left_field, right_field, field_type) = self.adapt_tuple_ordering(
                left_fields[index].clone(),
                right_fields[index].clone(),
                span,
            )?;
            left_fields[index] = left_field;
            right_fields[index] = right_field;
            field_types.push(field_type);
        }
        let value_type = CompilerType::Tuple(field_types);
        left.value_type = value_type.clone();
        right.value_type = value_type.clone();
        Ok((left, right, value_type))
    }

    fn no_structural_comparison(&self, span: Span, reason: &str) -> Diagnostic {
        source_diagnostic(
            &self.source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            format!("no structural comparison applies: {reason}"),
        )
    }

    fn finish_power(
        &self,
        left: CompilerExpression,
        right: CompilerExpression,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_exact_numeric(&self.source, left.span, &left.value_type)?;
        let exponent_is_nat = right.value_type == CompilerType::Nat;
        if !exponent_is_nat {
            require_type(
                &self.source,
                right.span,
                &CompilerType::Int,
                &right.value_type,
            )?;
        }
        let exponent = exact_int(&right);
        if left.value_type == CompilerType::Int && !exponent_is_nat {
            let Some(ref exponent) = exponent else {
                return Err(unsupported(
                    &self.source,
                    right.span,
                    "Int power with an exponent not proven to satisfy Nat",
                ));
            };
            if exponent < &BigInt::from(0) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-TYPE-MISMATCH",
                    right.span,
                    "an Int exponent must satisfy Nat",
                ));
            }
        }
        let can_fail = left.value_type == CompilerType::Rational
            && !is_proven_nonzero_numeric(&left)
            && !exponent_is_nat
            && exponent
                .as_ref()
                .is_none_or(|value| value < &BigInt::from(0));
        if can_fail && is_proven_zero_numeric(&left) && compiler_expression_is_closed(&left) {
            return Err(division_by_zero(&self.source, left.span));
        }
        if can_fail {
            let error_span = left.span;
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalPower,
                left,
                right,
                CompilerType::Rational,
                span,
                error_span,
            ));
        }
        let result_type = left.value_type.clone();
        Ok(Self::finish_binary(
            CompilerBinary::Power,
            left,
            right,
            result_type,
            span,
        ))
    }

    fn finish_modular_binary(
        &self,
        operation: CompilerBinary,
        left: CompilerExpression,
        right: CompilerExpression,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_same_type(&self.source, span, &left.value_type, &right.value_type)?;
        let CompilerType::Modular(modular) = &left.value_type else {
            unreachable!("same-type modular operation has two modular operands")
        };
        let (value_type, int_range) = match operation {
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply => {
                let exact = exact_int(&left)
                    .zip(exact_int(&right))
                    .map(|(left, right)| {
                        let value = match operation {
                            CompilerBinary::Add => left + right,
                            CompilerBinary::Subtract => left - right,
                            CompilerBinary::Multiply => left * right,
                            _ => unreachable!("selected modular arithmetic operation"),
                        };
                        IntRange::exact(reduce_modular(value, modular))
                    });
                (
                    left.value_type.clone(),
                    exact.or_else(|| Some(modular_range(modular))),
                )
            }
            CompilerBinary::Equal
            | CompilerBinary::NotEqual
            | CompilerBinary::Less
            | CompilerBinary::Greater
            | CompilerBinary::LessEqual
            | CompilerBinary::GreaterEqual => (CompilerType::Boolean, None),
            CompilerBinary::Compare => (CompilerType::Comparison, None),
            _ => {
                return Err(unsupported(
                    &self.source,
                    span,
                    "operation over modular values",
                ));
            }
        };
        let mut result = Self::finish_binary(operation, left, right, value_type, span);
        result.int_range = int_range;
        Ok(result)
    }

    fn finish_binary(
        operation: CompilerBinary,
        left: CompilerExpression,
        right: CompilerExpression,
        value_type: CompilerType,
        span: Span,
    ) -> CompilerExpression {
        let int_range = match (&value_type, operation) {
            (CompilerType::Int | CompilerType::Nat, CompilerBinary::Add) => {
                combine_ranges(&left, &right, |a, b| IntRange {
                    lower: &a.lower + &b.lower,
                    upper: &a.upper + &b.upper,
                })
            }
            (CompilerType::Int | CompilerType::Nat, CompilerBinary::Subtract) => {
                combine_ranges(&left, &right, |a, b| IntRange {
                    lower: &a.lower - &b.upper,
                    upper: &a.upper - &b.lower,
                })
            }
            (CompilerType::Int | CompilerType::Nat, CompilerBinary::Multiply) => {
                combine_ranges(&left, &right, multiply_range)
            }
            (CompilerType::Int, CompilerBinary::Modulo) => modulo_range(&left, &right),
            (CompilerType::Int, CompilerBinary::Power) => power_range(&left, &right),
            _ => None,
        };
        let rational_value = (value_type == CompilerType::Rational)
            .then(|| exact_rational_binary(operation, &left, &right))
            .flatten();
        CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation,
                left: Box::new(left),
                right: Box::new(right),
            },
            value_type,
            int_range,
            rational_value,
            span,
        }
    }

    fn finish_fallible_binary(
        operation: CompilerFallible,
        left: CompilerExpression,
        right: CompilerExpression,
        success_type: CompilerType,
        span: Span,
        error_span: Span,
    ) -> CompilerExpression {
        CompilerExpression {
            kind: CompilerExpressionKind::Fallible {
                operation,
                left: Box::new(left),
                right: Box::new(right),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(success_type)),
            int_range: None,
            rational_value: None,
            span,
        }
    }

    fn finish_validation(
        operation: CompilerValidation,
        value: CompilerExpression,
        success_type: CompilerType,
        span: Span,
        error_span: Span,
    ) -> CompilerExpression {
        CompilerExpression {
            kind: CompilerExpressionKind::Validate {
                operation,
                value: Box::new(value),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(success_type)),
            int_range: None,
            rational_value: None,
            span,
        }
    }

    fn analyze_library_application(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let selected = self
            .library_modules
            .iter()
            .filter_map(|(identity, functions)| {
                if items.len() <= identity.len()
                    || !identity.iter().zip(items).all(|(component, item)| {
                        matches!(item, Expression::Identifier(name)
                            if self.source.slice(*name) == component)
                    })
                {
                    return None;
                }
                let Expression::Identifier(member) = &items[identity.len()] else {
                    return None;
                };
                let member_name = self.source.slice(*member);
                let declarations = functions.get(member_name)?;
                let published = declarations
                    .iter()
                    .filter(|declaration| declaration.published)
                    .cloned()
                    .collect::<Vec<_>>();
                (!published.is_empty()).then(|| {
                    (
                        identity.len(),
                        format!("{}.{}", identity.join("."), member_name),
                        published,
                    )
                })
            })
            .max_by_key(|(module_length, _, _)| *module_length);
        let Some((module_length, function_name, declarations)) = selected else {
            return Ok(None);
        };
        let function_index = module_length;
        if items.len() == function_index + 1 {
            self.function_values_used = true;
            let tag_index = self
                .functions
                .len()
                .checked_add(COMPILER_SYMBOLIC_CALLABLES.len())
                .and_then(|value| value.checked_add(self.anonymous_function_value_names.len()))
                .ok_or_else(|| unsupported(&self.source, span, "native Function value tag"))?;
            let tag = u32::try_from(tag_index)
                .map_err(|_| unsupported(&self.source, span, "native Function value tag"))?;
            self.anonymous_function_value_names
                .push(format!("<fn {function_name}>"));
            self.anonymous_callables.insert(
                tag,
                CompilerCallableFacts::Named {
                    name: function_name,
                    declarations,
                    captures: Vec::new(),
                },
            );
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::FunctionValue(tag),
                value_type: CompilerType::Function,
                int_range: None,
                rational_value: None,
                span,
            }));
        }
        self.analyze_resolved_call_from(
            items,
            span,
            environment,
            function_index,
            &function_name,
            &declarations,
            &[],
            false,
        )
        .map(Some)
    }

    fn analyze_call(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let generator = items.iter().enumerate().find_map(|(index, item)| {
            let Expression::Identifier(name) = item else {
                return None;
            };
            let name = self.source.slice(*name);
            (!environment.contains_key(name) && self.generators.contains_key(name))
                .then_some((index, name.to_owned()))
        });
        if let Some((generator_index, generator_name)) = generator {
            return self.analyze_custom_generator_call(
                items,
                span,
                environment,
                generator_index,
                &generator_name,
            );
        }
        if let Some(module_identity) = self.active_library_module.clone()
            && let Some(functions) = self.library_modules.get(&module_identity)
        {
            let selected = items.iter().enumerate().find_map(|(index, item)| {
                let Expression::Identifier(name) = item else {
                    return None;
                };
                let name = self.source.slice(*name);
                (!environment.contains_key(name) && functions.contains_key(name))
                    .then_some((index, name.to_owned()))
            });
            if let Some((function_index, function_name)) = selected {
                let declarations = functions
                    .get(&function_name)
                    .expect("selected library function exists")
                    .clone();
                let canonical = format!("{}.{}", module_identity.join("."), function_name);
                return self.analyze_resolved_call_from(
                    items,
                    span,
                    environment,
                    function_index,
                    &canonical,
                    &declarations,
                    &[],
                    false,
                );
            }
        }
        if let Some(Expression::Identifier(name)) = items.first()
            && let Some(callable) = self
                .root_callable_bindings
                .get(self.source.slice(*name))
                .cloned()
        {
            return self.analyze_callable_application(callable, items, span, environment);
        }
        let function = items.iter().enumerate().find_map(|(index, item)| {
            let Expression::Identifier(name) = item else {
                return None;
            };
            let name = self.source.slice(*name);
            (!environment.contains_key(name) && self.functions.contains_key(name))
                .then_some((index, name.to_owned()))
        });
        let Some((function_index, function_name)) = function else {
            return Err(unsupported(&self.source, span, "application"));
        };
        self.analyze_resolved_call(items, span, environment, function_index, &function_name)
    }

    fn analyze_custom_generator_argument(
        &mut self,
        argument: &Expression,
        initial_type: &CompilerType,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if initial_type
            == &CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
                CompilerType::Int,
                CompilerType::String,
            ])))
            && let Some(value) = exact_optional_int_string_none_literal(&self.source, argument)
        {
            return Ok(value);
        }
        if let Some(enumeration) = recursive_nominal_enumeration(initial_type)
            && let Some(value) =
                exact_recursive_nominal_literal(&self.source, argument, enumeration, "First")
        {
            return Ok(value);
        }
        self.analyze_expression(argument, environment)
    }

    fn analyze_custom_generator_call(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        generator_index: usize,
        generator_name: &str,
    ) -> Result<CompilerExpression, Diagnostic> {
        let declarations = self
            .generators
            .get(generator_name)
            .expect("selected custom generator overload set exists")
            .clone();
        self.analyze_custom_generator_call_from(
            items,
            span,
            environment,
            generator_index,
            generator_name,
            &declarations,
            "root",
        )
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Value and Character generator instantiation paths stay adjacent.
    fn analyze_custom_generator_call_from(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        generator_index: usize,
        generator_name: &str,
        declarations: &[GeneratorSource],
        declaration_namespace: &str,
    ) -> Result<CompilerExpression, Diagnostic> {
        let argument_sources = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (index != generator_index).then_some(item))
            .collect::<Vec<_>>();
        let arguments = if let [declaration] = declarations
            && declaration.additional_initial_parameters.is_empty()
            && let [argument] = argument_sources.as_slice()
        {
            vec![self.analyze_custom_generator_argument(
                argument,
                &declaration.initial_parameter.value_type,
                environment,
            )?]
        } else {
            argument_sources
                .iter()
                .map(|argument| self.analyze_expression(argument, environment))
                .collect::<Result<Vec<_>, _>>()?
        };
        let flattened_arguments = flattened_product_arguments(&argument_sources, &arguments);
        let is_overload_set = declarations.len() > 1;
        let mut selected = None;
        for declaration in declarations {
            let parameter_count = 1 + declaration.additional_initial_parameters.len();
            let candidate_arguments = if parameter_count > 1
                && let Some(flattened) = &flattened_arguments
            {
                flattened.as_slice()
            } else {
                arguments.as_slice()
            };
            if candidate_arguments.len() != parameter_count {
                continue;
            }
            let mut initials = Vec::with_capacity(parameter_count);
            for (parameter, argument) in std::iter::once(&declaration.initial_parameter)
                .chain(&declaration.additional_initial_parameters)
                .zip(candidate_arguments)
            {
                match adapt_custom_generator_initial(
                    &self.source,
                    parameter,
                    argument,
                    generator_name,
                    span,
                ) {
                    Ok(initial) => initials.push(initial),
                    Err(diagnostic) if diagnostic.code == "E-NO-APPLICABLE-GENERATOR-OVERLOAD" => {
                        initials.clear();
                        break;
                    }
                    Err(diagnostic) => return Err(diagnostic),
                }
            }
            if initials.len() == parameter_count {
                selected = Some((declaration.clone(), initials));
                break;
            }
        }
        let Some((declaration, mut initials)) = selected else {
            let actual = arguments
                .iter()
                .map(|argument| argument.value_type.name())
                .collect::<Vec<_>>()
                .join(", ");
            let available = declarations
                .iter()
                .map(|declaration| {
                    let inputs = std::iter::once(&declaration.initial_parameter)
                        .chain(&declaration.additional_initial_parameters)
                        .map(|parameter| parameter.value_type.name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({inputs})")
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(source_diagnostic(
                &self.source,
                "E-NO-APPLICABLE-GENERATOR-OVERLOAD",
                span,
                format!(
                    "no `{generator_name}` generator overload accepts ({actual}); available input signatures: {available}"
                ),
            ));
        };
        let admitted_initials = if declaration.additional_initial_parameters.is_empty() {
            matches!(initials.as_slice(), [initial]
                if exact_int(initial) == Some(BigInt::from(7)))
        } else {
            matches!(initials.as_slice(), [value, suffix]
                if exact_int(value) == Some(BigInt::from(7))
                    && exact_string(suffix).as_deref() == Some("item"))
        };
        if is_overload_set && !admitted_initials {
            return Err(unsupported(
                &self.source,
                span,
                "custom generator overload input outside exact 7 or (7, \"item\")",
            ));
        }
        let admitted_function_boundary =
            self.in_function && exact_value_boundary_generator_source(&self.source, &declaration);
        if self.in_function
            && (!declaration.prefix.statements.is_empty() || declaration.value_yields.is_some())
            && !admitted_function_boundary
        {
            return Err(unsupported(
                &self.source,
                span,
                "custom generator with an independent value direction outside a direct root construction",
            ));
        }
        let initial = initials.remove(0);
        let additional_initials = initials;
        if let Some(yields) = declaration.value_yields {
            let continuations = declaration.value_continuations;
            let explicit_return = declaration.explicit_return;
            let mut result = declaration.result;
            if let Some(mut function) = declaration.local_function {
                let symbol = self.reserve_function_symbol(&function.source_name);
                let CompilerExpressionKind::Call {
                    symbol: result_symbol,
                    ..
                } = &mut result.kind
                else {
                    unreachable!("checked generator-local function retains its final call")
                };
                result_symbol.clone_from(&symbol);
                function.symbol = symbol;
                self.instances.push(function);
            }
            let yield_type = std::iter::once(&declaration.initial_parameter)
                .chain(&declaration.additional_initial_parameters)
                .nth(declaration.yield_parameter)
                .expect("checked generator yield parameter exists")
                .value_type
                .clone();
            let value_type = CompilerType::Generator(CompilerGeneratorType {
                yield_type: Box::new(yield_type),
                resume_type: Box::new(CompilerType::Unit),
                result_type: Box::new(result.value_type.clone()),
            });
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::CustomValueGenerator {
                    declaration: self.source.slice(declaration.name).to_owned(),
                    declaration_namespace: declaration_namespace.to_owned(),
                    declaration_span: declaration.span,
                    initial_parameter: Box::new(declaration.initial_parameter),
                    initial: Box::new(initial),
                    additional_initial_parameters: declaration.additional_initial_parameters,
                    additional_initials,
                    prefix: Box::new(declaration.prefix),
                    yields,
                    continuations,
                    explicit_return,
                    result: Box::new(result),
                },
                value_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        let characters = if let Some(characters) = declaration.literal_characters {
            characters
        } else {
            let character = Self::known_string_value(&initial, environment).ok_or_else(|| {
                unsupported(
                    &self.source,
                    initial.span,
                    "dynamic Character custom generator input",
                )
            })?;
            vec![character; declaration.yield_count]
        };
        let locals = declaration.local.into_iter().collect();
        let mut close_handler = declaration.close_handler;
        if let Some(mut function) = declaration.local_function {
            let symbol = self.reserve_function_symbol(&function.source_name);
            let handler = close_handler
                .as_mut()
                .expect("checked Character generator-local function has a close handler");
            for rule in &mut handler.error_codes {
                if let CompilerExpressionKind::Call {
                    symbol: call_symbol,
                    ..
                } = &mut rule.action.kind
                {
                    call_symbol.clone_from(&symbol);
                }
            }
            for action in [&mut handler.error_action, &mut handler.ok_action] {
                if let CompilerExpressionKind::Call {
                    symbol: call_symbol,
                    ..
                } = &mut action.kind
                {
                    call_symbol.clone_from(&symbol);
                }
            }
            function.symbol = symbol;
            self.instances.push(function);
        }
        let result = declaration.result;
        let value_type = character_generator_type(result.value_type.clone());
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::CustomCharacterGenerator {
                declaration: self.source.slice(declaration.name).to_owned(),
                declaration_namespace: declaration_namespace.to_owned(),
                declaration_span: declaration.span,
                initial_parameter: Box::new(declaration.initial_parameter),
                initial: Box::new(initial),
                prefix: Box::new(declaration.prefix),
                characters,
                locals,
                close_handler,
                result: Box::new(result),
            },
            value_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_resolved_call(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        function_index: usize,
        function_name: &str,
    ) -> Result<CompilerExpression, Diagnostic> {
        let declarations = self
            .functions
            .get(function_name)
            .expect("selected overload set exists")
            .clone();
        self.analyze_resolved_call_from(
            items,
            span,
            environment,
            function_index,
            function_name,
            &declarations,
            &[],
            false,
        )
    }
}
