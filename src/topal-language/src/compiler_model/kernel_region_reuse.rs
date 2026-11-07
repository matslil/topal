/// Reuse checked same-declaration bodies, preserving small qualified closures.
/// No semantic type/effect/value/capture is discarded; only integer-range metadata
/// unused by lowering is removed from retained definitions before requalification.
fn coalesce_kernel_region_helpers(program: &mut CompilerProgram) -> Result<(), Diagnostic> {
    if program.functions.len() <= 64 {
        return Ok(());
    }
    // Check every original body, capture, effect and call before any merging.
    kernel_region_analysis_with_budget(program, None)?;
    fn block(block: &mut CompilerBlock, aliases: &BTreeMap<String, String>) {
        for statement in &mut block.statements {
            expression(
                match statement {
                    CompilerStatement::Binding(binding) => &mut binding.value,
                    CompilerStatement::Discard(value) => value,
                },
                aliases,
            );
        }
        expression(&mut block.result, aliases);
    }
    fn expression(value: &mut CompilerExpression, aliases: &BTreeMap<String, String>) {
        use CompilerExpressionKind as K;
        value.int_range = None;
        match &mut value.kind {
            K::Unit | K::Boolean(_) | K::Int(_) | K::Local(_) | K::OptionalNone | K::ListEmpty => {}
            K::IntToNat(input)
            | K::Absolute(input)
            | K::Not(input)
            | K::OptionalSome(input)
            | K::ContainerEntryCount(input)
            | K::ContainerEmpty(input) => expression(input, aliases),
            K::KernelRegion {
                region, arguments, ..
            } => {
                expression(region, aliases);
                for argument in arguments {
                    expression(argument, aliases);
                }
            }
            K::Block(body) => block(body, aliases),
            K::Call { symbol, arguments } => {
                if let Some(retained) = aliases.get(symbol) {
                    *symbol = retained.clone();
                }
                for argument in arguments {
                    expression(argument, aliases);
                }
            }
            K::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                expression(subject, aliases);
                expression(when_true, aliases);
                expression(when_false, aliases);
            }
            K::OptionalDecision {
                subject,
                some_action,
                none_action,
                ..
            } => {
                expression(subject, aliases);
                expression(some_action, aliases);
                expression(none_action, aliases);
            }
            K::Binary { left, right, .. }
            | K::ListEntry {
                value: left,
                remaining: right,
            } => {
                expression(left, aliases);
                expression(right, aliases);
            }
            K::Record(fields) => {
                for (_, field) in fields {
                    expression(field, aliases);
                }
            }
            K::RecordField { record, .. } | K::TupleField { tuple: record, .. } => {
                expression(record, aliases)
            }
            K::RecordReconstruct { base, replacements } => {
                expression(base, aliases);
                for (_, value) in replacements {
                    expression(value, aliases);
                }
            }
            K::Tuple(fields) => {
                for field in fields {
                    expression(field, aliases);
                }
            }
            K::ContainerCollect {
                source,
                kind: CompilerContainerKind::Array,
                ..
            } => expression(source, aliases),
            K::ArrayAt { array, .. } => expression(array, aliases),
            _ => unreachable!("original closed region graph was validated"),
        }
    }
    let mut retained: Vec<CompilerFunction> = Vec::new();
    let mut aliases = BTreeMap::new();
    for mut function in std::mem::take(&mut program.functions) {
        // Instances arrive in callee-before-caller order; every equivalent
        // child reference is canonical before comparing a parent body.
        block(&mut function.body, &aliases);
        for parameter in &mut function.parameters {
            parameter.int_range = None;
        }
        let original_symbol = std::mem::take(&mut function.symbol);
        let equivalent = retained.iter().find(|candidate| {
            let mut identity = (*candidate).clone();
            identity.symbol.clear();
            identity == function
        });
        if let Some(existing) = equivalent {
            aliases.insert(original_symbol, existing.symbol.clone());
        } else {
            function.symbol = original_symbol.clone();
            aliases.insert(original_symbol.clone(), original_symbol);
            retained.push(function);
        }
    }
    block(&mut program.main, &aliases);
    program.functions = retained;
    kernel_region_analysis(program)?;
    Ok(())
}

#[cfg(test)]
#[path = "kernel_region_reuse_tests.rs"]
mod kernel_region_reuse_tests;
