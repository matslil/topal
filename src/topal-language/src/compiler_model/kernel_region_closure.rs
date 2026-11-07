/// Inferred effects of one source specialization on the entry owner's resource.
/// All admitted views originate in that owner; window aliases never split it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerKernelRegionEffects {
    pub symbol: String,
    pub read: bool,
    pub write: bool,
}

fn region_source_type(ty: &CompilerType, parameter: bool) -> bool {
    match ty {
        CompilerType::Unit | CompilerType::Boolean | CompilerType::Int | CompilerType::Nat => true,
        CompilerType::KernelByteRegion => parameter,
        CompilerType::Optional(payload) => {
            matches!(payload.as_ref(), CompilerType::Int | CompilerType::Nat)
        }
        CompilerType::Record(fields) => fields
            .iter()
            .all(|(_, field)| region_source_type(field, false)),
        CompilerType::Tuple(fields) => fields.iter().all(|field| region_source_type(field, false)),
        CompilerType::List(element) => element.as_ref() == &CompilerType::Int,
        CompilerType::Array { element, .. } => element.as_ref() == &CompilerType::Int,
        _ => false,
    }
}

/// Validate the closed source profile and infer owner effects through calls.
/// This establishes no native calling, storage or allocation evidence.
pub fn kernel_region_function_effects(
    program: &CompilerProgram,
) -> Result<Vec<CompilerKernelRegionEffects>, Diagnostic> {
    kernel_region_analysis(program).map(|(effects, _)| effects)
}

/// Runtime input selection is separate from the source effect row. It is
/// derived from every lowered specialization, including unselected branches.
pub fn kernel_region_unsigned_runtime_required(
    program: &CompilerProgram,
) -> Result<bool, Diagnostic> {
    kernel_region_analysis(program).map(|(_, required)| required)
}

fn kernel_region_analysis(
    program: &CompilerProgram,
) -> Result<(Vec<CompilerKernelRegionEffects>, bool), Diagnostic> {
    kernel_region_analysis_with_budget(program, Some(64))
}

fn kernel_region_analysis_with_budget(
    program: &CompilerProgram,
    maximum_functions: Option<usize>,
) -> Result<(Vec<CompilerKernelRegionEffects>, bool), Diagnostic> {
    let invalid = |span, message| unsupported(&program.source, span, message);
    let functions = program
        .functions
        .iter()
        .map(|f| (f.symbol.clone(), f))
        .collect::<BTreeMap<_, _>>();
    if functions.len() != program.functions.len()
        || functions.is_empty()
        || maximum_functions.is_some_and(|limit| functions.len() > limit)
    {
        return Err(invalid(
            program.main.result.span,
            "region-library function identity budget",
        ));
    }
    fn visit_block(
        block: &CompilerBlock,
        calls: &mut BTreeSet<String>,
        effects: &mut (bool, bool, bool),
    ) -> Result<(), Span> {
        for statement in &block.statements {
            visit(
                match statement {
                    CompilerStatement::Binding(binding) => &binding.value,
                    CompilerStatement::Discard(value) => value,
                },
                calls,
                effects,
            )?;
        }
        visit(&block.result, calls, effects)
    }
    fn visit(
        value: &CompilerExpression,
        calls: &mut BTreeSet<String>,
        effects: &mut (bool, bool, bool),
    ) -> Result<(), Span> {
        use CompilerExpressionKind as K;
        let local_view = matches!(&value.value_type, CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::KernelByteRegion);
        if !region_source_type(&value.value_type, true) && !local_view {
            return Err(value.span);
        }
        match &value.kind {
            K::Unit | K::Boolean(_) | K::Int(_) | K::Local(_) | K::OptionalNone | K::ListEmpty => {
                Ok(())
            }
            K::IntToNat(input)
            | K::Absolute(input)
            | K::Not(input)
            | K::OptionalSome(input)
            | K::ContainerEntryCount(input)
            | K::ContainerEmpty(input) => visit(input, calls, effects),
            K::KernelRegion {
                operation,
                region,
                arguments,
                ..
            } => {
                effects.0 |= matches!(
                    operation,
                    CompilerKernelRegionOperation::Load
                        | CompilerKernelRegionOperation::UnsignedLoad(_)
                );
                effects.1 |= matches!(
                    operation,
                    CompilerKernelRegionOperation::Store
                        | CompilerKernelRegionOperation::UnsignedStore(_)
                );
                effects.2 |= matches!(
                    operation,
                    CompilerKernelRegionOperation::UnsignedLoad(_)
                        | CompilerKernelRegionOperation::UnsignedStore(_)
                );
                visit(region, calls, effects)?;
                for argument in arguments {
                    visit(argument, calls, effects)?;
                }
                Ok(())
            }
            K::Block(block) => visit_block(block, calls, effects),
            K::Call { symbol, arguments } => {
                calls.insert(symbol.clone());
                for argument in arguments {
                    visit(argument, calls, effects)?;
                }
                Ok(())
            }
            K::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                visit(subject, calls, effects)?;
                visit(when_true, calls, effects)?;
                visit(when_false, calls, effects)
            }
            K::OptionalDecision {
                subject,
                some_action,
                none_action,
                ..
            } => {
                visit(subject, calls, effects)?;
                visit(some_action, calls, effects)?;
                visit(none_action, calls, effects)
            }
            K::Binary { left, right, .. }
            | K::ListEntry {
                value: left,
                remaining: right,
            } => {
                visit(left, calls, effects)?;
                visit(right, calls, effects)
            }
            K::Record(fields) => {
                for (_, field) in fields {
                    visit(field, calls, effects)?;
                }
                Ok(())
            }
            K::RecordField { record, .. } | K::TupleField { tuple: record, .. } => {
                visit(record, calls, effects)
            }
            K::RecordReconstruct { base, replacements } => {
                visit(base, calls, effects)?;
                for (_, value) in replacements {
                    visit(value, calls, effects)?;
                }
                Ok(())
            }
            K::Tuple(fields) => {
                for field in fields {
                    visit(field, calls, effects)?;
                }
                Ok(())
            }
            K::ContainerCollect {
                source,
                kind: CompilerContainerKind::Array,
                ..
            } => visit(source, calls, effects),
            K::ArrayAt { array, .. } => visit(array, calls, effects),
            _ => Err(value.span),
        }
    }
    let mut graph = BTreeMap::new();
    for (symbol, function) in &functions {
        if function.foreign.is_some()
            || function.is_static
            || !function.result_captures.is_empty()
            || !function.pattern_identities.is_empty()
            || !region_source_type(&function.result_type, false)
            || function.parameters.iter().any(|p| {
                (!p.source_visible && !p.discarded) || !region_source_type(&p.value_type, true)
            })
        {
            return Err(invalid(
                function.span,
                "region-library result, capture or parameter escape",
            ));
        }
        let mut calls = BTreeSet::new();
        let mut effects = (false, false, false);
        visit_block(&function.body, &mut calls, &mut effects)
            .map_err(|span| invalid(span, "unqualified region-library source value/operation"))?;
        graph.insert(symbol.clone(), (calls, effects));
    }
    fn closure(
        symbol: &str,
        graph: &BTreeMap<String, (BTreeSet<String>, (bool, bool, bool))>,
        active: &mut BTreeSet<String>,
        cache: &mut BTreeMap<String, (bool, bool, bool)>,
    ) -> Result<(bool, bool, bool), ()> {
        if let Some(result) = cache.get(symbol) {
            return Ok(*result);
        }
        if !active.insert(symbol.into()) {
            return Err(());
        }
        let (calls, mut effects) = graph.get(symbol).cloned().ok_or(())?;
        for callee in calls {
            let nested = closure(&callee, graph, active, cache)?;
            effects.0 |= nested.0;
            effects.1 |= nested.1;
            effects.2 |= nested.2;
        }
        active.remove(symbol);
        cache.insert(symbol.into(), effects);
        Ok(effects)
    }
    let mut results = Vec::new();
    let mut unsigned_runtime = false;
    let mut cache = BTreeMap::new();
    for (symbol, function) in functions {
        let (read, write, unsigned) = closure(&symbol, &graph, &mut BTreeSet::new(), &mut cache)
            .map_err(|()| invalid(function.span, "region-library recursive or unresolved call"))?;
        unsigned_runtime |= unsigned;
        if (read || write)
            && function
                .declared_effects
                .as_ref()
                .is_some_and(|row| row.identities.is_empty())
        {
            return Err(invalid(
                function.span,
                "region-library byte effects exceed empty Effects ()",
            ));
        }
        results.push(CompilerKernelRegionEffects {
            symbol,
            read,
            write,
        });
    }
    Ok((results, unsigned_runtime))
}
