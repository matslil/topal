/// Closed semantic operations, not source instruction or address primitives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerKernelRegionOperation {
    Count,
    Load,
    Store,
    Window { write: bool },
}

enum KernelRegionSelection<'a> {
    Entry(&'a str),
    Modules(&'a [CompilerKernelRegionModuleExport]),
}

#[derive(Clone, Debug)]
pub struct CompilerKernelRegionModuleExport {
    pub module: Vec<String>,
    pub name: String,
    pub parameters: Vec<CompilerType>,
    pub result: CompilerType,
}

#[cfg(test)]
#[path = "kernel_region_tests.rs"]
mod kernel_region_tests;

/// Typed source entry selection. This supplies no live/native region authority.
/// The systems publisher must additionally qualify the complete object/runtime.
pub fn analyze_kernel_region_library(
    text: &str,
    modules: &[CompilerSourceModule],
    entry: &str,
) -> Result<CompilerProgram, Diagnostic> {
    let selection = KernelRegionSelection::Entry(entry);
    let program = analyze_for_compiler_options(text, modules, &[entry.into()], Some(&selection))?;
    if !crate::published_function_names(text)
        .iter()
        .any(|name| canonically_equal(name, entry))
    {
        return Err(Diagnostic::error(
            "E-SYSTEMS-REGION-SOURCE",
            1,
            1,
            "region entry must name a published ordinary function",
        ));
    }
    kernel_region_function_effects(&program)?;
    Ok(program)
}

/// Check reachable public module signatures without caller declarations or
/// fake source values. Symbolic typed inputs carry no live resource authority.
pub fn analyze_kernel_region_module_exports(
    modules: &[CompilerSourceModule],
    exports: &[CompilerKernelRegionModuleExport],
) -> Result<CompilerProgram, Diagnostic> {
    let mut source = "use language (version is v0.1, features is (systems))\n".to_owned();
    let libraries = modules
        .iter()
        .filter_map(|module| module.identity.first())
        .collect::<BTreeSet<_>>();
    for library in libraries {
        source.push_str(&format!("use library {library} (version is v0.1)\n"));
    }
    for export in exports {
        let module = modules
            .iter()
            .find(|module| module.identity == export.module)
            .ok_or_else(|| {
                Diagnostic::error(
                    "E-SYSTEMS-REGION-SOURCE",
                    1,
                    1,
                    "isolated export has no frozen module",
                )
            })?;
        if !crate::published_function_names(&module.source)
            .iter()
            .any(|name| canonically_equal(name, &export.name))
        {
            return Err(Diagnostic::error(
                "E-SYSTEMS-REGION-SOURCE",
                1,
                1,
                "isolated export is private or absent",
            ));
        }
    }
    let selection = KernelRegionSelection::Modules(exports);
    let names = exports
        .iter()
        .map(|export| format!("{}.{}", export.module.join("."), export.name))
        .collect::<Vec<_>>();
    let program = analyze_for_compiler_options(&source, modules, &names, Some(&selection))?;
    if !exports.is_empty() {
        kernel_region_function_effects(&program)?;
    }
    Ok(program)
}

fn region_module_declaration(statement: &Statement) -> bool {
    match statement {
        Statement::LanguageSelection { .. }
        | Statement::LibrarySelection { .. }
        | Statement::Function { .. } => true,
        Statement::Published { declaration, .. } => {
            matches!(declaration.as_ref(), Statement::Function { .. })
        }
        _ => false,
    }
}

impl Analyzer {
    fn instantiate_kernel_region_module(
        &mut self,
        export: &CompilerKernelRegionModuleExport,
    ) -> Result<(), Diagnostic> {
        let declarations = self
            .library_modules
            .get(&export.module)
            .and_then(|functions| functions.get(&export.name))
            .cloned()
            .ok_or_else(|| {
                unsupported(
                    &self.source,
                    Span::new(0, 0),
                    "missing isolated module declaration",
                )
            })?;
        for mut declaration in declarations {
            // Ordinary packaged calls lower to their declared fields. Preserve
            // those exact types when independently instantiating the signature.
            let mut parameters = Vec::new();
            for parameter in &declaration.parameters {
                if parameter.qualifier.is_some() || parameter.default.is_some() {
                    return Err(unsupported(
                        &self.source,
                        parameter.name,
                        "qualified/defaulted isolated region input",
                    ));
                }
                if parameter.fields.is_empty() {
                    parameters.push(parameter.clone());
                } else {
                    if parameter.fields.iter().any(|field| {
                        !field.fields.is_empty()
                            || field.qualifier.is_some()
                            || field.default.is_some()
                    }) {
                        return Err(unsupported(
                            &self.source,
                            parameter.name,
                            "nested/qualified/defaulted isolated region field",
                        ));
                    }
                    parameters.extend(parameter.fields.iter().cloned());
                }
            }
            declaration.parameters = parameters;
            if declaration.parameters.len() != export.parameters.len() {
                continue;
            }
            let mut substitutions = self.classifier_substitutions.clone();
            let mut matched = true;
            for (parameter, expected) in declaration.parameters.iter().zip(&export.parameters) {
                if self
                    .specialize_classifier(parameter.classifier, expected, &mut substitutions)?
                    .as_ref()
                    != Some(expected)
                {
                    matched = false;
                    break;
                }
            }
            if !matched {
                continue;
            }
            let arguments = declaration
                .parameters
                .iter()
                .zip(&export.parameters)
                .enumerate()
                .map(|(index, (parameter, ty))| CompilerExpression {
                    kind: CompilerExpressionKind::Local(format!("<declared-module-input-{index}>")),
                    value_type: ty.clone(),
                    int_range: None,
                    rational_value: None,
                    span: parameter.name,
                })
                .collect::<Vec<_>>();
            let count = arguments.len();
            let metadata = CompilerCallMetadata {
                callable_arguments: vec![None; count],
                aggregate_arguments: vec![StaticValueFacts::default(); count],
                callable_captures: Vec::new(),
                aggregate_captures: Vec::new(),
                scope_arguments: vec![None; count],
                scope_captures: Vec::new(),
                lexical_captures: Vec::new(),
                context_captures: Vec::new(),
            };
            let previous = std::mem::replace(&mut self.classifier_substitutions, substitutions);
            let name = format!("{}.{}", export.module.join("."), export.name);
            let result = self.finish_selected_call(
                &name,
                &declaration,
                arguments,
                &metadata,
                declaration.span,
            );
            self.classifier_substitutions = previous;
            let result = result?;
            if result.value_type != export.result {
                return Err(unsupported(
                    &self.source,
                    declaration.span,
                    "isolated module result mismatch",
                ));
            }
            return Ok(());
        }
        Err(unsupported(
            &self.source,
            Span::new(0, 0),
            "isolated module declared signature mismatch",
        ))
    }
    fn kernel_region_vocabulary(&self, span: Span) -> bool {
        self.kernel_region_sources
            .iter()
            .any(|range| range.start <= span.start && span.end <= range.end)
    }

    fn instantiate_kernel_region_entry(&mut self, entry: &str) -> Result<(), Diagnostic> {
        let matches = self
            .functions
            .iter()
            .filter(|(name, _)| canonically_equal(name, entry))
            .collect::<Vec<_>>();
        let [(name, declarations)] = matches.as_slice() else {
            return Err(unsupported(
                &self.source,
                Span::new(0, 0),
                "ambiguous or missing region entry",
            ));
        };
        let [declaration] = declarations.as_slice() else {
            return Err(unsupported(
                &self.source,
                Span::new(0, 0),
                "overloaded region entry",
            ));
        };
        let name = (*name).clone();
        let declaration = declaration.clone();
        if declaration.is_static
            || declaration.parameters.len() != 1
            || self.parse_classifier(declaration.parameters[0].classifier)?
                != CompilerType::KernelByteRegion
            || self.parse_classifier(declaration.result)? != CompilerType::Unit
        {
            return Err(unsupported(
                &self.source,
                declaration.span,
                "region entry requires (KernelByteRegion) -> Unit",
            ));
        }
        let argument = CompilerExpression {
            kind: CompilerExpressionKind::Local("<qualified-region-input>".into()),
            value_type: CompilerType::KernelByteRegion,
            int_range: None,
            rational_value: None,
            span: declaration.parameters[0].name,
        };
        let metadata = CompilerCallMetadata {
            callable_arguments: vec![None],
            aggregate_arguments: vec![StaticValueFacts::default()],
            callable_captures: Vec::new(),
            aggregate_captures: Vec::new(),
            scope_arguments: vec![None],
            scope_captures: Vec::new(),
            lexical_captures: Vec::new(),
            context_captures: Vec::new(),
        };
        self.finish_selected_call(
            &name,
            &declaration,
            vec![argument],
            &metadata,
            declaration.span,
        )?;
        Ok(())
    }

    fn analyze_kernel_region_member(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let [
            receiver,
            Expression::Identifier(byte),
            Expression::Identifier(member),
            rest @ ..,
        ] = items
        else {
            return Ok(None);
        };
        let Expression::Identifier(name) = receiver else {
            return Ok(None);
        };
        if self.source.slice(*byte) != "byte"
            || !environment
                .get(self.source.slice(*name))
                .is_some_and(|facts| facts.value_type == CompilerType::KernelByteRegion)
        {
            return Ok(None);
        }
        if !self.kernel_region_vocabulary(span) {
            return Err(unsupported(
                &self.source,
                span,
                "kernel-region member without systems vocabulary",
            ));
        }
        let region = Box::new(self.analyze_expression(receiver, environment)?);
        let member = self.source.slice(*member).to_owned();
        let (operation, names, result) = match member.as_str() {
            "count" => (
                CompilerKernelRegionOperation::Count,
                &[][..],
                CompilerType::Nat,
            ),
            "load" => (
                CompilerKernelRegionOperation::Load,
                &["offset-bytes"][..],
                CompilerType::Optional(Box::new(CompilerType::Nat)),
            ),
            "store" => (
                CompilerKernelRegionOperation::Store,
                &["offset-bytes", "value"][..],
                CompilerType::Boolean,
            ),
            "window" => (
                CompilerKernelRegionOperation::Window { write: false },
                &["offset-bytes", "byte-count", "rights"][..],
                CompilerType::Optional(Box::new(CompilerType::KernelByteRegion)),
            ),
            _ => {
                return Err(unsupported(
                    &self.source,
                    span,
                    "unknown kernel-region byte member",
                ));
            }
        };
        let fields = if names.is_empty() {
            if !rest.is_empty() {
                return Err(unsupported(&self.source, span, "byte count arguments"));
            }
            &[][..]
        } else {
            let [Expression::Product { fields, .. }] = rest else {
                return Err(unsupported(
                    &self.source,
                    span,
                    "kernel-region member requires named facts",
                ));
            };
            fields.as_slice()
        };
        if fields.len() != names.len() || fields.iter().any(|field| field.label.is_none()) {
            return Err(unsupported(
                &self.source,
                span,
                "kernel-region member fact set",
            ));
        }
        let mut arguments = Vec::new();
        let mut argument_roles = Vec::new();
        let mut selected = operation;
        for expected in names {
            let matching = fields
                .iter()
                .filter(|field| self.source.slice(field.label.unwrap()) == *expected)
                .collect::<Vec<_>>();
            let [_field] = matching.as_slice() else {
                return Err(unsupported(
                    &self.source,
                    span,
                    "missing or duplicate kernel-region fact",
                ));
            };
        }
        // Named-product fields retain ordinary source evaluation order. The
        // canonical native slots are assigned only after each value is checked.
        for field in fields {
            let label = self.source.slice(field.label.unwrap()).to_owned();
            if label == "rights" {
                let Expression::Identifier(rights) = &field.value else {
                    return Err(unsupported(
                        &self.source,
                        field.value.span(),
                        "static kernel-region rights",
                    ));
                };
                selected = CompilerKernelRegionOperation::Window {
                    write: match self.source.slice(*rights) {
                        "read-only" => false,
                        "read-write" => true,
                        _ => {
                            return Err(unsupported(&self.source, *rights, "kernel-region rights"));
                        }
                    },
                };
                continue;
            }
            let value = self.analyze_expression(&field.value, environment)?;
            let value = adapt_function_call_argument(&CompilerType::Nat, &value)
                .or_else(|| self.adapt_proven_nonnegative_nat_argument(&CompilerType::Nat, &value))
                .ok_or_else(|| {
                    unsupported(
                        &self.source,
                        field.value.span(),
                        "kernel-region fact must be Nat",
                    )
                })?;
            arguments.push(value);
            argument_roles.push(
                names
                    .iter()
                    .position(|name| *name == label.as_str())
                    .expect("validated fact"),
            );
        }
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::KernelRegion {
                operation: selected,
                region,
                arguments,
                argument_roles,
            },
            value_type: result,
            int_range: (selected == CompilerKernelRegionOperation::Count).then(|| IntRange {
                lower: BigInt::from(0),
                upper: BigInt::from(65536),
            }),
            rational_value: None,
            span,
        }))
    }
}
