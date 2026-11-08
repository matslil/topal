//! Checked graph/range to portable owner-session constraints; no native grant.
use crate::{CompileError, analyze_systems_tail_storage};
use std::collections::{BTreeMap, BTreeSet};
use topal_language::compiler::CompilerProgram;
use topal_semantics::{
    RegionLibraryBinding, TailScalarBound, TailScalarKind, TailSessionBinding, TailSessionLimits,
};

fn invalid(reason: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-TAIL-MODEL: {reason}"))
}

/// Rebuild an owned traversal's reference constraints from compiler IR.
///
/// The result neither authenticates externally constructed IR as source nor
/// binds executable native code, kernel storage/context, hooks or erasure.
///
/// # Errors
/// Returns a diagnostic for unsupported source/ranges, a missing invariant
/// region, unrepresentable identity costs or invalid reference constraints.
pub fn inspect_systems_tail_session_binding(
    program: &CompilerProgram,
    entry: &str,
) -> Result<TailSessionBinding, CompileError> {
    let storage = analyze_systems_tail_storage(program, entry)?;
    let plan = &storage.source_plan;
    if storage.region_parameter.is_none() {
        return Err(invalid("owned session requires one invariant region"));
    }
    let functions: BTreeMap<_, _> = plan
        .functions
        .iter()
        .map(|f| {
            (
                f.symbol.clone(),
                f.callees.iter().cloned().collect::<BTreeSet<_>>(),
            )
        })
        .collect();
    let region_functions = plan
        .functions
        .iter()
        .filter(|f| {
            f.parameter_classifiers
                .iter()
                .any(|c| c == "KernelByteRegion")
        })
        .map(|f| f.symbol.clone())
        .collect();
    let library = RegionLibraryBinding::new(
        plan.checked_graph_sha256.clone(),
        entry.into(),
        functions,
        plan.maximum_one_step_function_depth,
        plan.maximum_one_step_function_depth,
    )
    .map_err(|e| invalid(format!("{e:?}")))?;
    let roots = storage
        .values
        .iter()
        .map(|value| {
            Ok(match value.classifier.as_str() {
                "Nat" => TailScalarBound::Nat {
                    maximum: value.upper.parse().map_err(invalid)?,
                },
                "Int" => TailScalarBound::Int {
                    minimum: value.lower.parse().map_err(invalid)?,
                    maximum: value.upper.parse().map_err(invalid)?,
                },
                "Boolean" => TailScalarBound::Boolean,
                _ => return Err(invalid("unsupported carried classifier")),
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let measure = storage
        .values
        .iter()
        .position(|v| v.parameter == plan.measure_parameter)
        .ok_or_else(|| invalid("missing scalar measure"))?;
    let entry_function = plan
        .functions
        .iter()
        .find(|f| f.symbol == entry)
        .ok_or_else(|| invalid("missing source entry"))?;
    let result = match entry_function.result_classifier.as_str() {
        "Nat" => TailScalarKind::Nat,
        "Int" => TailScalarKind::Int,
        "Boolean" => TailScalarKind::Boolean,
        "Unit" => TailScalarKind::Unit,
        _ => return Err(invalid("unsupported result classifier")),
    };
    let limits = TailSessionLimits {
        base_bound: plan.base_bound.parse().map_err(invalid)?,
        decrements: plan
            .back_edges
            .iter()
            .map(|e| e.decrement.parse().map_err(invalid))
            .collect::<Result<_, _>>()?,
        frames_per_step: storage.source_frames_per_step.parse().map_err(invalid)?,
        loans_per_step: storage
            .source_region_loans_per_step
            .parse()
            .map_err(invalid)?,
        work_bytes: storage.source_step_bytes.try_into().map_err(invalid)?,
    };
    // Source constants are not native readonly grants. The model may allocate
    // operation results; exact executable readonly roots remain separately bound.
    let binding = TailSessionBinding::new(
        library,
        region_functions,
        roots,
        measure,
        result,
        Vec::new(),
        limits,
    )
    .map_err(|e| invalid(format!("{e:?}")))?;
    if binding.bank_bytes() as u64 != storage.bank_bytes {
        return Err(invalid("canonical model bank differs from analyzed layout"));
    }
    Ok(binding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use topal_semantics::*;
    #[test]
    fn actual_clear_source_binds_ranges_helper_scopes_and_owned_state() {
        // TOPAL-SYSTEMS-TAIL-SESSION-001. Source inspection plus reference
        // transitions, NOT a claim that the model executes the compiled body.
        let source = include_str!("../../../linux-kernel/kernel/algorithms/clear-prefix.t");
        let program = topal_language::compiler::analyze_kernel_region_tail_source(
            source,
            &[],
            "clear-prefix",
            3,
        )
        .unwrap();
        let entry = &program
            .functions
            .iter()
            .find(|f| f.source_name == "clear-prefix")
            .unwrap()
            .symbol;
        let binding = inspect_systems_tail_session_binding(&program, entry).unwrap();
        assert_eq!(binding.bank_bytes(), 80);
        let mut storage = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 16,
                alignment_bytes: 8,
            },
            "source-model-owner",
        )
        .unwrap();
        let region = storage
            .allocate(BootstrapStorageRequest {
                byte_count: 16,
                alignment_bytes: 8,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            })
            .unwrap();
        storage.store_byte(&region, 5, 0x7b).unwrap();
        let nat = |n: u64| TailScalar::Nat(n.into());
        let mut session = TailSession::begin(
            &mut storage,
            &region,
            &binding,
            RegionLibraryAccess::ReadWrite,
            &[nat(1), nat(5), nat(1)],
        )
        .unwrap();
        session.start_step().unwrap();
        let root = session.root_view().unwrap();
        let helper = |name: &str| {
            program
                .functions
                .iter()
                .find(|f| f.source_name == name)
                .unwrap()
                .symbol
                .as_str()
        };
        let next = session
            .enter_helper(helper("next-status"), Some(&root))
            .unwrap()
            .unwrap();
        let store = session
            .enter_helper(helper("store-status"), Some(&next))
            .unwrap()
            .unwrap();
        session.store_byte(&store, 5, 0).unwrap();
        session.return_helper().unwrap();
        session.return_helper().unwrap();
        for (i, value) in [nat(0), nat(6), nat(1)].iter().enumerate() {
            let value = session.allocate(value).unwrap();
            session.stage(i, &value).unwrap();
        }
        session.returned(true).unwrap();
        session.checkpoint().unwrap();
        assert_eq!(session.identity_counters(), (5, 5));
        session.start_step().unwrap();
        let result = session.allocate(&TailScalar::Boolean(true)).unwrap();
        session.stage_result(Some(&result)).unwrap();
        session.returned(false).unwrap();
        session.checkpoint().unwrap();
        assert_eq!(
            session
                .complete(RegionActionReclamation {
                    arena_erased: true,
                    control_erased: true,
                    retired_stack_erased: true,
                    bindings_revoked: true,
                    caller_restored: true
                })
                .unwrap(),
            TailScalar::Boolean(true)
        );
        assert!(session.private_buffers_reclaimed());
        drop(session);
        assert_eq!(storage.load_byte(&region, 5).unwrap(), 0);
    }
    #[test]
    fn ownerless_source_and_changed_graph_do_not_receive_owned_bindings() {
        let source = "use language (version is v0.1)\nrepeat is fn (n : Nat) -> Nat : Decreases n\n  n\n    <= 0 then 0\n    otherwise repeat(n - 1)\nrepeat(2)\n";
        let program = topal_language::analyze_for_compiler(source).unwrap();
        let entry = &program
            .functions
            .iter()
            .find(|f| f.source_name == "repeat")
            .unwrap()
            .symbol;
        assert!(
            inspect_systems_tail_session_binding(&program, entry)
                .unwrap_err()
                .to_string()
                .contains("invariant region")
        );
        assert!(inspect_systems_tail_session_binding(&program, "missing").is_err());
    }
}
