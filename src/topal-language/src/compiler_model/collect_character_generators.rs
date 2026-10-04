#[allow(clippy::too_many_lines)] // Exact declaration and every retained yield stay fail-closed together.
fn collect_character_generators(
    source: &SourceText,
    statements: &[Statement],
    reserved_names: &BTreeSet<String>,
    enums: &EnumTypes,
    functions: &BTreeMap<String, Vec<FunctionSource>>,
    generators: &mut BTreeMap<String, Vec<GeneratorSource>>,
) -> Result<(), Diagnostic> {
    for statement in statements {
        let Statement::Generator {
            name,
            parameters,
            yielded,
            resumed,
            result,
            body,
            span,
        } = statement
        else {
            continue;
        };
        let name_text = source.slice(*name).to_owned();
        if reserved_names.contains(&name_text) || functions.contains_key(&name_text) {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                *name,
                format!("`{name_text}` is already declared in this scope"),
            ));
        }
        if parameters.len() == 2 {
            let generator = exact_int_string_generator_overload(
                source, *name, parameters, *yielded, *resumed, *result, body, *span,
            )?;
            register_generator(source, generators, &name_text, generator)?;
            continue;
        }
        let [parameter] = parameters.as_slice() else {
            return Err(unsupported(
                source,
                *span,
                "custom generator with other than one initial parameter",
            ));
        };
        let initial_classifier = compact_classifier(source.slice(parameter.classifier));
        let initial_type = match initial_classifier.as_str() {
            "Boolean" => CompilerType::Boolean,
            "Character" => CompilerType::Character,
            "Comparison" => CompilerType::Comparison,
            "Int" => CompilerType::Int,
            "ListInt" => int_list_type(),
            "Nat" => CompilerType::Nat,
            "OptionalInt" => CompilerType::Optional(Box::new(CompilerType::Int)),
            "Optional(Int,String)" => CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
                CompilerType::Int,
                CompilerType::String,
            ]))),
            "RangeInt" => CompilerType::Range(Box::new(CompilerType::Int)),
            "Rational" => CompilerType::Rational,
            "Result((Int,String),langarithmeticArithmeticErrorCode)" => {
                nested_result_product_type()
            }
            "Result(Rational,langarithmeticArithmeticErrorCode)" => {
                CompilerType::Result(Box::new(CompilerType::Rational))
            }
            "String" => CompilerType::String,
            "Unit" => CompilerType::Unit,
            "(Int,String)" => {
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
            }
            classifier if classifier == RECURSIVE_NOMINAL_GENERATOR_CLASSIFIER => enums
                .get("Choice")
                .filter(|(_, declaration)| declaration.end <= parameter.classifier.start)
                .map(|(enumeration, _)| recursive_nominal_generator_type(enumeration))
                .ok_or_else(|| {
                    unsupported(
                        source,
                        *span,
                        "recursive nominal custom generator before its exact Choice declaration",
                    )
                })?,
            classifier => enums
                .get(classifier)
                .filter(|(_, declaration)| declaration.end <= parameter.classifier.start)
                .map(|(enumeration, _)| CompilerType::Enum(enumeration.clone()))
                .ok_or_else(|| {
                    unsupported(
                        source,
                        *span,
                        "custom generator outside the admitted Boolean, Character, Comparison, Choice Enum, Int, List Int, Nat, Optional Int, Optional (Int, String), recursive nominal product, Range Int, Rational, Result Rational, Result (Int, String), String, Unit, or (Int, String) initial-input subset",
                    )
                })?,
        };
        let initial_parameter = CompilerParameter {
            name: source.slice(parameter.name).to_owned(),
            discarded: false,
            source_visible: true,
            value_type: initial_type.clone(),
            int_range: None,
            span: parameter.name,
        };
        if generators.get(&name_text).is_some_and(|overloads| {
            overloads.iter().any(|candidate| {
                candidate.additional_initial_parameters.is_empty()
                    && candidate.initial_parameter.value_type == initial_type
            })
        }) {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-GENERATOR-OVERLOAD",
                *span,
                format!("generator overload `{name_text}` has the same input classifiers"),
            ));
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == int_list_type()
            && compact_classifier(source.slice(*yielded)) == "ListInt"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result)) == "ListInt"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_list_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Int
            && source.slice(*yielded) == "Int"
            && source.slice(*resumed) == "Unit"
            && source.slice(*result) == "String"
        {
            let generator =
                exact_unary_int_string_generator_overload(source, *name, parameter, body, *span)?;
            register_generator(source, generators, &name_text, generator)?;
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Boolean
            && source.slice(*yielded) == "Boolean"
            && source.slice(*resumed) == "Unit"
            && matches!(source.slice(*result), "Boolean" | "String")
        {
            let (exact_body, local_function) = if body
                .first()
                .and_then(|statement| enum_declaration(source, statement))
                .is_some()
            {
                if enums.contains_key("Choice") {
                    return Err(unsupported(
                        source,
                        *span,
                        "generator-local Choice beside a root Choice declaration",
                    ));
                }
                let (body, function) =
                    exact_boolean_local_function_generator_body(source, parameter, body, *span)?;
                (body, Some(function))
            } else {
                (
                    exact_boolean_value_generator_body(source, parameter, *result, body, *span)?,
                    None,
                )
            };
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_body;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && let Some(enumeration) = recursive_nominal_enumeration(&initial_type)
            && compact_classifier(source.slice(*yielded)) == RECURSIVE_NOMINAL_GENERATOR_CLASSIFIER
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result)) == RECURSIVE_NOMINAL_GENERATOR_CLASSIFIER
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_recursive_nominal_generator_body(
                source,
                parameter,
                enumeration,
                body,
                *span,
            )?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Comparison
            && source.slice(*yielded) == "Comparison"
            && source.slice(*resumed) == "Unit"
            && source.slice(*result) == "Comparison"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_comparison_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Result(Box::new(CompilerType::Rational))
            && compact_classifier(source.slice(*yielded))
                == "Result(Rational,langarithmeticArithmeticErrorCode)"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result))
                == "Result(Rational,langarithmeticArithmeticErrorCode)"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_result_rational_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
            && compact_classifier(source.slice(*yielded)) == "(Int,String)"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result)) == "(Int,String)"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_int_string_product_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && let CompilerType::Enum(enumeration) = &initial_type
            && enumeration.name == "Choice"
            && enumeration.alternatives == ["First", "Second"]
            && source.slice(*yielded) == "Choice"
            && source.slice(*resumed) == "Unit"
            && source.slice(*result) == "Choice"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_enum_value_generator_body(source, parameter, enumeration, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Nat
            && source.slice(*yielded) == "Nat"
            && source.slice(*resumed) == "Unit"
            && source.slice(*result) == "Nat"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_nat_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Range(Box::new(CompilerType::Int))
            && compact_classifier(source.slice(*yielded)) == "RangeInt"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result)) == "RangeInt"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_int_range_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == nested_optional_product_type()
            && compact_classifier(source.slice(*yielded)) == "Optional(Int,String)"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result))
                == "Result((Int,String),langarithmeticArithmeticErrorCode)"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_nested_result_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type
                == CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
                    CompilerType::Int,
                    CompilerType::String,
                ])))
            && compact_classifier(source.slice(*yielded)) == "Optional(Int,String)"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result)) == "Optional(Int,String)"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_nested_optional_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == nested_result_product_type()
            && compact_classifier(source.slice(*yielded))
                == "Result((Int,String),langarithmeticArithmeticErrorCode)"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result))
                == "Result((Int,String),langarithmeticArithmeticErrorCode)"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_nested_result_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Optional(Box::new(CompilerType::Int))
            && compact_classifier(source.slice(*yielded)) == "OptionalInt"
            && source.slice(*resumed) == "Unit"
            && compact_classifier(source.slice(*result)) == "OptionalInt"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_optional_int_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Unit
            && source.slice(*yielded) == "Unit"
            && source.slice(*resumed) == "Unit"
            && source.slice(*result) == "Unit"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_unit_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Rational
            && source.slice(*yielded) == "Rational"
            && source.slice(*resumed) == "Unit"
            && source.slice(*result) == "Rational"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_rational_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::Int
            && source.slice(*yielded) == "Int"
            && source.slice(*resumed) == "Unit"
            && source.slice(*result) == "Int"
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_int_value_generator_body(source, parameter, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if parameter.fields.is_empty()
            && parameter.default.is_none()
            && parameter.qualifier.is_none()
            && source.slice(parameter.name) != "_"
            && initial_type == CompilerType::String
            && source.slice(*yielded) == "String"
            && source.slice(*resumed) == "Unit"
            && matches!(source.slice(*result), "Unit" | "String")
        {
            let ExactValueGeneratorBody {
                yields: value_yields,
                continuations: value_continuations,
                explicit_return,
                result: final_value,
            } = exact_string_value_generator_body(source, parameter, *result, body, *span)?;
            let yield_count = value_yields.len();
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix: CompilerBlock {
                        statements: Vec::new(),
                        result: unit_expression(*result),
                    },
                    literal_characters: None,
                    value_yields: Some(value_yields),
                    value_continuations,
                    explicit_return,
                    yield_count,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: final_value,
                });
            continue;
        }
        if !parameter.fields.is_empty()
            || parameter.default.is_some()
            || parameter.qualifier.is_some()
            || source.slice(*yielded) != "Character"
            || source.slice(*resumed) != "Unit"
            || !matches!(
                (&initial_type, source.slice(*result)),
                (CompilerType::Character, "Unit" | "Character") | (CompilerType::String, "Unit")
            )
        {
            return Err(unsupported(
                source,
                *span,
                "custom generator outside the Character-yield/Unit-resume/admitted initial/result subset",
            ));
        }
        if initial_type == CompilerType::String {
            let (prefix, literal_characters) =
                exact_string_input_generator_body(source, parameter, body, *span)?;
            generators
                .entry(name_text)
                .or_default()
                .push(GeneratorSource {
                    name: *name,
                    span: *span,
                    initial_parameter,
                    additional_initial_parameters: Vec::new(),
                    yield_parameter: 0,
                    prefix,
                    literal_characters: Some(literal_characters),
                    value_yields: None,
                    value_continuations: Vec::new(),
                    explicit_return: None,
                    yield_count: 1,
                    local: None,
                    local_function: None,
                    close_handler: None,
                    result: unit_expression(*result),
                });
            continue;
        }
        if source.slice(*result) == "Unit" {
            let close = if body
                .first()
                .and_then(|statement| enum_declaration(source, statement))
                .is_some()
            {
                if enums.contains_key("CloseChoice") {
                    return Err(unsupported(
                        source,
                        *span,
                        "generator-local CloseChoice beside a root CloseChoice declaration",
                    ));
                }
                let (handler, function) = exact_character_generator_local_close_handler(
                    source,
                    parameter.name,
                    body,
                    *span,
                )?;
                Some((handler, Some(function)))
            } else {
                exact_character_generator_close_handler(source, parameter.name, body)
                    .map(|handler| (handler, None))
            };
            if let Some((close_handler, local_function)) = close {
                generators
                    .entry(name_text)
                    .or_default()
                    .push(GeneratorSource {
                        name: *name,
                        span: *span,
                        initial_parameter,
                        additional_initial_parameters: Vec::new(),
                        yield_parameter: 0,
                        prefix: CompilerBlock {
                            statements: Vec::new(),
                            result: unit_expression(*result),
                        },
                        literal_characters: None,
                        value_yields: None,
                        value_continuations: Vec::new(),
                        explicit_return: None,
                        yield_count: 1,
                        local: None,
                        local_function,
                        close_handler: Some(close_handler),
                        result: unit_expression(*result),
                    });
                continue;
            }
        }
        let Some((final_statement, yields)) = body.split_last() else {
            return Err(unsupported(
                source,
                *span,
                "custom generator body without an admitted final value",
            ));
        };
        let mut local = None;
        let mut yielded_name = parameter.name;
        let mut yield_count = 0;
        for body_statement in yields {
            if let Statement::Binding {
                name: local_name,
                classifier,
                value: Expression::Identifier(local_value),
            } = body_statement
            {
                if local.is_some()
                    || classifier.is_none_or(|classifier| source.slice(classifier) != "Character")
                    || source.slice(*local_value) != source.slice(parameter.name)
                    || source.slice(*local_name) == "_"
                    || source.slice(*local_name) == source.slice(parameter.name)
                {
                    return Err(unsupported(
                        source,
                        *span,
                        "custom generator local binding outside one distinct Character alias of its initial parameter",
                    ));
                }
                local = Some(CompilerGeneratorLocal {
                    parameter: CompilerParameter {
                        name: source.slice(*local_name).to_owned(),
                        discarded: false,
                        source_visible: true,
                        value_type: CompilerType::Character,
                        int_range: None,
                        span: *local_name,
                    },
                    activation_after_resumptions: yield_count,
                });
                yielded_name = *local_name;
                continue;
            }

            if let Statement::Binding {
                name: local_name,
                classifier,
                value:
                    Expression::Application {
                        items: yield_items, ..
                    },
            } = body_statement
            {
                let [
                    Expression::Identifier(yield_operation),
                    Expression::Identifier(yielded_value),
                ] = yield_items.as_slice()
                else {
                    return Err(unsupported(
                        source,
                        *span,
                        "custom generator resume binding outside one exact Character yield",
                    ));
                };
                if local.is_some()
                    || yield_count != 0
                    || classifier.is_some_and(|classifier| source.slice(classifier) != "Unit")
                    || source.slice(*local_name) == "_"
                    || source.slice(*local_name) == source.slice(parameter.name)
                    || source.slice(*yield_operation) != "yield"
                    || source.slice(*yielded_value) != source.slice(yielded_name)
                {
                    return Err(unsupported(
                        source,
                        *span,
                        "custom generator resume binding outside one exact Character yield",
                    ));
                }
                yield_count += 1;
                local = Some(CompilerGeneratorLocal {
                    parameter: CompilerParameter {
                        name: source.slice(*local_name).to_owned(),
                        discarded: false,
                        source_visible: true,
                        value_type: CompilerType::Unit,
                        int_range: None,
                        span: *local_name,
                    },
                    activation_after_resumptions: yield_count,
                });
                continue;
            }

            let Statement::Discard {
                value:
                    Expression::Application {
                        items: yield_items, ..
                    },
                ..
            } = body_statement
            else {
                return Err(unsupported(
                    source,
                    *span,
                    "custom generator body outside admitted Character aliases and discarded yields",
                ));
            };
            let [
                Expression::Identifier(yield_operation),
                Expression::Identifier(yielded_value),
            ] = yield_items.as_slice()
            else {
                return Err(unsupported(
                    source,
                    *span,
                    "custom generator yield expression",
                ));
            };
            if source.slice(*yield_operation) != "yield"
                || source.slice(*yielded_value) != source.slice(yielded_name)
            {
                return Err(unsupported(
                    source,
                    *span,
                    "custom generator body outside yielding its active Character value",
                ));
            }
            yield_count += 1;
        }
        if local.as_ref().is_some_and(|local| {
            local.parameter.value_type == CompilerType::Character
                && local.activation_after_resumptions == yield_count
        }) {
            return Err(unsupported(
                source,
                *span,
                "custom generator local binding without a following yield",
            ));
        }
        let final_value = match (source.slice(*result), final_statement) {
            ("Unit", Statement::Expression(Expression::Unit(span)))
                if local
                    .as_ref()
                    .is_none_or(|local| local.parameter.value_type == CompilerType::Character) =>
            {
                unit_expression(*span)
            }
            ("Unit", Statement::Expression(Expression::Identifier(final_name)))
                if local.as_ref().is_some_and(|local| {
                    local.parameter.value_type == CompilerType::Unit
                        && local.parameter.name == source.slice(*final_name)
                        && local.activation_after_resumptions == 1
                        && yield_count == 1
                }) =>
            {
                unit_expression(*final_name)
            }
            ("Character", Statement::Expression(Expression::String(literal))) => {
                let value = parse_string(source.slice(*literal)).ok_or_else(|| {
                    source_diagnostic(
                        source,
                        "E-STRING-LITERAL",
                        *literal,
                        "invalid string literal delimiter",
                    )
                })?;
                let count = character_count(value);
                if count != 1 {
                    return Err(source_diagnostic(
                        source,
                        "E-CHARACTER-CLASSIFIER",
                        *literal,
                        format!(
                            "Character requires exactly one user-perceived character, but this String contains {count}"
                        ),
                    ));
                }
                CompilerExpression {
                    kind: CompilerExpressionKind::String(value.to_owned()),
                    value_type: CompilerType::Character,
                    int_range: None,
                    rational_value: None,
                    span: *literal,
                }
            }
            _ => {
                return Err(unsupported(
                    source,
                    *span,
                    "custom generator final value outside exact Unit, bound Unit resume, or Character literal",
                ));
            }
        };
        if final_value.value_type == CompilerType::Character
            && (local.is_some() || yield_count != 1)
        {
            return Err(unsupported(
                source,
                *span,
                "Character-final custom generator outside one direct initial-parameter yield",
            ));
        }
        generators
            .entry(name_text)
            .or_default()
            .push(GeneratorSource {
                name: *name,
                span: *span,
                initial_parameter,
                additional_initial_parameters: Vec::new(),
                yield_parameter: 0,
                prefix: CompilerBlock {
                    statements: Vec::new(),
                    result: unit_expression(*result),
                },
                literal_characters: None,
                value_yields: None,
                value_continuations: Vec::new(),
                explicit_return: None,
                yield_count,
                local,
                local_function: None,
                close_handler: None,
                result: final_value,
            });
    }
    Ok(())
}

fn compiler_declared_effect_row(
    source: &SourceText,
    effect_bound: Option<Span>,
) -> Result<Option<CompilerEffectRow>, Diagnostic> {
    let Some(effect_bound) = effect_bound else {
        return Ok(None);
    };
    let text = source.slice(effect_bound);
    if compact_classifier(text) == "Effects()" {
        return Ok(Some(CompilerEffectRow {
            identities: Vec::new(),
        }));
    }
    if explicit_single_measure(text).is_some() || explicit_absolute_measure(text).is_some() {
        return Ok(None);
    }
    Err(unsupported(
        source,
        effect_bound,
        "nonempty or polymorphic function effect bound",
    ))
}

fn same_function_input_header(
    source: &SourceText,
    left: &FunctionSource,
    right: &FunctionSource,
) -> bool {
    left.is_static == right.is_static
        && left.parameters.len() == right.parameters.len()
        && left
            .parameters
            .iter()
            .zip(&right.parameters)
            .all(|(left, right)| {
                compact_classifier(source.slice(left.classifier))
                    == compact_classifier(source.slice(right.classifier))
            })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BlockKind {
    TopLevel,
    Function,
    Lexical,
}

struct AnalyzedBlock {
    block: CompilerBlock,
    returns_from_function: bool,
}
