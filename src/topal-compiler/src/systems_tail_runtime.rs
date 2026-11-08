//! Provider-owned hooks; source/runtime components are not kernel certificates.
use crate::{CompileError, SystemsTailStepComponent};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use topal_language::compiler::CompilerProgram;
#[path = "systems_tail_runtime_inspection.rs"]
pub(crate) mod inspection;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailRuntimeLayout {
    pub context_bytes: u64,
    pub fields: BTreeMap<String, u64>,
    pub frame_bytes: u64,
    pub history_bytes: u64,
    pub minimum_temporary_bytes: u64,
    pub recovery_header_bytes: u64,
    pub recovery_cell_bytes: u64,
    pub caller_output_frame_bytes: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailRuntimeComponent {
    pub schema: String,
    pub qualification: String,
    pub source_step: SystemsTailStepComponent,
    pub layout: SystemsTailRuntimeLayout,
    pub llvm_sha256: String,
    pub llvm: String,
    pub boundary_sha256: String,
    pub boundary: Vec<u8>,
}
fn invalid(reason: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-TAIL-RUNTIME: {reason}"))
}
fn align(value: u64, granule: u64) -> u64 {
    value.div_ceil(granule) * granule
}

/// Rebuild a source step and its experimental provider-owned hook component.
/// This grants no external-IR source attestation or kernel execution authority.
///
/// # Errors
/// Rejects unsupported source/ranges, oversized actual context, or target output
/// construction failure. Caller admission, full linked stack/context/owner and
/// physical refinement remain independent obligations.
pub fn emit_systems_tail_runtime_component(
    program: &CompilerProgram,
    source_name: &str,
    entry: &str,
) -> Result<SystemsTailRuntimeComponent, CompileError> {
    runtime_for_component(crate::emit_systems_tail_step_component(
        program,
        source_name,
        entry,
    )?)
}
pub(crate) fn runtime_for_component(
    source_step: SystemsTailStepComponent,
) -> Result<SystemsTailRuntimeComponent, CompileError> {
    let storage = &source_step.storage;
    let plan = &storage.source_plan;
    let depth = plan.maximum_one_step_function_depth as u64;
    let parameters = plan
        .functions
        .iter()
        .find(|f| f.symbol == plan.entry_symbol)
        .ok_or_else(|| invalid("missing entry"))?
        .parameter_classifiers
        .len();
    let functions = source_step
        .functions
        .iter()
        .map(|f| f.identity)
        .max()
        .ok_or_else(|| invalid("missing functions"))?
        + 1;
    if depth == 0 || depth > 64 || parameters == 0 || parameters > 17 || functions > 65 {
        return Err(invalid("invalid component limits"));
    }
    let mut fields = BTreeMap::from([("CONTROL".into(), 0), ("FRAMES".into(), 64)]);
    let mut offset = 64 + depth * 88;
    for name in [
        "HISTORY",
        "FRAME_END",
        "LOAN_END",
        "DEPTH_WORD",
        "BANK",
        "PHASE",
        "STAGED",
        "USED",
        "STEPS",
        "STEP_LIMIT",
        "STEP_FRAME_END",
        "STEP_LOAN_END",
    ] {
        fields.insert(name.into(), offset);
        offset += 8;
    }
    offset = align(offset, 16);
    fields.insert("BANKS".into(), offset);
    offset += 2 * storage.bank_bytes;
    fields.insert("WORK".into(), offset);
    // A source with an unreachable recursive arm can require no allocations.
    // Retain one physical granule, included in the actual layout/cost, so every
    // existing native work/provenance representation has a nonempty extent.
    let work_bytes = storage.source_step_bytes.max(8);
    offset += work_bytes;
    fields.insert("STARTS".into(), offset);
    let starts = work_bytes.div_ceil(64);
    offset += starts;
    offset = align(offset, 8);
    fields.insert("RESULT".into(), offset);
    offset += 48;
    fields.insert("RESULT_KIND".into(), offset);
    offset += 8;
    let context_bytes = align(offset, 16);
    let layout = SystemsTailRuntimeLayout {
        context_bytes,
        fields,
        frame_bytes: 88,
        history_bytes: 40,
        minimum_temporary_bytes: context_bytes + 64 + 8,
        recovery_header_bytes: 64,
        recovery_cell_bytes: 8,
        caller_output_frame_bytes: 56,
    };
    if layout.minimum_temporary_bytes > 65536 {
        return Err(invalid("actual context exceeds existing temporary reserve"));
    }
    let mut roots = vec![(4, 0, 0, 0); parameters];
    let mut mask = 0u64;
    for value in &storage.values {
        let kind = match value.classifier.as_str() {
            "Nat" => 1,
            "Int" => 2,
            "Boolean" => 3,
            _ => return Err(invalid("unsupported root")),
        };
        roots[value.parameter] = (
            kind,
            value.bank_offset,
            value.bank_slot_bytes,
            value.maximum_bits,
        );
        mask |= 1 << value.parameter;
    }
    let roots = roots
        .into_iter()
        .map(|(k, o, n, b)| format!("%Root {{ i64 {k}, i64 {o}, i64 {n}, i64 {b} }}"))
        .collect::<Vec<_>>()
        .join(", ");
    let result_classifier = &plan
        .functions
        .iter()
        .find(|f| f.symbol == plan.entry_symbol)
        .unwrap()
        .result_classifier;
    let result_kind = match result_classifier.as_str() {
        "Nat" | "Int" => 2,
        "Boolean" => 3,
        "Unit" => 4,
        _ => return Err(invalid("unsupported result")),
    };
    let mut regions = vec![0; functions as usize];
    let mut calls = vec![0; (functions * functions) as usize];
    for function in &source_step.functions {
        regions[function.identity as usize] = u8::from(function.has_region);
        for callee in &function.callees {
            calls[(function.identity * functions + callee) as usize] = 1;
        }
    }
    let bytes = |values: Vec<u8>| {
        values
            .into_iter()
            .map(|v| format!("i8 {v}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let base: num_bigint::BigUint = plan.base_bound.parse().map_err(invalid)?;
    let base_word = u64::try_from(base).unwrap_or(u64::MAX);
    let mut decrements = BTreeSet::new();
    for edge in &plan.back_edges {
        let value: num_bigint::BigUint = edge.decrement.parse().map_err(invalid)?;
        if let Ok(word) = u64::try_from(value) {
            decrements.insert(word);
        }
    }
    let mut replacements = layout
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect::<BTreeMap<_, _>>();
    for (name, value) in [
        ("PARAMETERS", parameters.to_string()),
        ("FUNCTIONS", functions.to_string()),
        ("CALL_COUNT", (functions * functions).to_string()),
        ("MAX_DEPTH", depth.to_string()),
        ("ENTRY", source_step.entry_identity.to_string()),
        ("BANK_BYTES", storage.bank_bytes.to_string()),
        ("BOTH_BANK_BYTES", (2 * storage.bank_bytes).to_string()),
        ("WORK_BYTES", work_bytes.to_string()),
        ("START_BYTES", starts.to_string()),
        ("ROOT_TABLE", roots),
        ("REGION_TABLE", bytes(regions)),
        ("CALL_TABLE", bytes(calls)),
        ("SCALAR_MASK", mask.to_string()),
        ("RETIRED_HELPER_BYTES", ((depth - 1) * 88).to_string()),
        ("MEASURE_PARAMETER", plan.measure_parameter.to_string()),
        ("BASE_WORD", base_word.to_string()),
        ("DECREMENT_COUNT", decrements.len().to_string()),
        (
            "DECREMENT_TABLE",
            decrements
                .iter()
                .map(|d| format!("i64 {d}"))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("FRAMES_PER_STEP", storage.source_frames_per_step.clone()),
        (
            "LOANS_PER_STEP",
            storage.source_region_loans_per_step.clone(),
        ),
        ("OPERAND_BITS", storage.maximum_operand_bits.to_string()),
        ("EXPECTED_RESULT_KIND", result_kind.to_string()),
        (
            "RESULT_NUMBER_KIND",
            if result_classifier == "Nat" { "1" } else { "2" }.into(),
        ),
    ] {
        replacements.insert(name.into(), value);
    }
    let mut llvm = format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n",
        crate::X86_SYSTEMS_DATA_LAYOUT
    );
    for fragment in [
        include_str!("runtime/kernel_tail_hooks.ll"),
        include_str!("runtime/kernel_tail_numbers.ll"),
        include_str!("runtime/kernel_tail_scopes.ll"),
        include_str!("runtime/kernel_tail_roots.ll"),
        include_str!("runtime/kernel_tail_checkpoint.ll"),
        include_str!("runtime/kernel_tail_completion.ll"),
    ] {
        llvm.push_str(fragment);
    }
    for (key, value) in replacements {
        llvm = llvm.replace(&format!("{{{{{key}}}}}"), &value);
    }
    if llvm.contains("{{") {
        return Err(invalid("unexpanded runtime layout token"));
    }
    llvm = llvm
        .lines()
        .map(|line| {
            if line.starts_with("define ") && line.ends_with(" {") {
                format!(
                    "{} noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{",
                    line.strip_suffix(" {").unwrap()
                )
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let boundary =
        crate::systems_provider_object::tail_boundary::runtime_object(layout.fields["HISTORY"])?;
    Ok(SystemsTailRuntimeComponent {schema:"topal.tail-provider-hooks.x86_64/1".into(),
        qualification:"native source/checkpoint/completion mechanisms only; owning-kernel admission, caller resource bounds/context and kernel execution unqualified".into(),
        layout,llvm_sha256:format!("{:x}",Sha256::digest(llvm.as_bytes())),llvm,
        boundary_sha256:format!("{:x}",Sha256::digest(&boundary)),boundary,source_step})
}
