//! Actual admission/controller mechanisms; kernel caller proofs remain separate.
use crate::{CompileError, SystemsTailRuntimeComponent};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use topal_language::compiler::CompilerProgram;
#[path = "systems_tail_owner_inspection.rs"]
pub(crate) mod inspection;
#[cfg(test)]
#[path = "systems_tail_owner_tests.rs"]
mod tests;

#[derive(Clone, Debug, Serialize)]
pub struct SystemsTailOwnerComponent {
    pub schema: String,
    pub qualification: String,
    pub runtime: SystemsTailRuntimeComponent,
    pub caller_input_frame_bytes: u64,
    pub llvm_sha256: String,
    pub llvm: String,
}

/// Construct admission and the actual three-role controller.
/// This grants no actual kernel ownership, caller retirement or publication.
pub fn emit_systems_tail_owner_component(
    program: &CompilerProgram,
    source_name: &str,
    entry: &str,
) -> Result<SystemsTailOwnerComponent, CompileError> {
    let runtime = crate::emit_systems_tail_runtime_component(program, source_name, entry)?;
    let storage = &runtime.source_step.storage;
    let plan = &storage.source_plan;
    let root = plan
        .functions
        .iter()
        .find(|f| f.symbol == plan.entry_symbol)
        .unwrap();
    if root.parameter_classifiers != ["KernelByteRegion", "Nat", "Nat", "Nat"]
        || root.result_classifier != "Boolean"
        || plan.measure_parameter != 1
    {
        return Err(CompileError::Tool("E-SYSTEMS-TAIL-OWNER: initial owner interface requires region/three Nat facts -> Boolean with first Nat measure".into()));
    }
    let frames: u64 = storage
        .source_frames_per_step
        .parse()
        .map_err(|_| CompileError::Tool("invalid owner frame budget".into()))?;
    let loans: u64 = storage
        .source_region_loans_per_step
        .parse()
        .map_err(|_| CompileError::Tool("invalid owner loan budget".into()))?;
    if frames == 0 || loans == 0 {
        return Err(CompileError::Tool(
            "invalid owner identity multiplicity".into(),
        ));
    }
    let word = |text: &str| -> u64 {
        u64::try_from(text.parse::<num_bigint::BigUint>().unwrap()).unwrap_or(u64::MAX)
    };
    let mut replacements = runtime
        .layout
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect::<BTreeMap<_, _>>();
    replacements.insert("BASE".into(), word(&plan.base_bound).to_string());
    replacements.insert(
        "DECREMENT".into(),
        word(&plan.minimum_decrement).to_string(),
    );
    replacements.insert("FRAMES_PER_STEP".into(), frames.to_string());
    replacements.insert("LOANS_PER_STEP".into(), loans.to_string());
    replacements.insert(
        "ENTRY".into(),
        runtime.source_step.entry_identity.to_string(),
    );
    let mut seeds = String::new();
    for value in &storage.values {
        let index = value.parameter - 1;
        let offset = runtime.layout.fields["BANKS"] + value.bank_offset;
        seeds.push_str(&format!("  %seed.{index} = getelementptr i8, ptr %context, i64 {offset}\n  call void @o.number(ptr %seed.{index}, i64 %input.{index})\n"));
    }
    replacements.insert("SEEDS".into(), seeds);
    let mut llvm = format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n",
        crate::X86_SYSTEMS_DATA_LAYOUT
    );
    llvm.push_str(&geometry());
    llvm.push_str(include_str!("runtime/kernel_tail_owner.ll"));
    for (key, value) in replacements {
        llvm = llvm.replace(&format!("{{{{{key}}}}}"), &value);
    }
    if llvm.contains("{{") {
        return Err(CompileError::Tool("unexpanded owner layout".into()));
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
    Ok(SystemsTailOwnerComponent {schema:"topal.tail-owner-controller.x86_64/1".into(),
        qualification:"native admission/controller mechanisms only; actual mapped ownership, kernel context/caller retirement and publication unqualified".into(),
        runtime, caller_input_frame_bytes:24,llvm_sha256:format!("{:x}",Sha256::digest(llvm.as_bytes())),llvm})
}

fn geometry() -> String {
    use std::fmt::Write;
    let mut text = String::from(
        "@topal_x86_tail_boundary_active_v1 = external global ptr\ndefine internal i1 @o.geometry(ptr %owner, i64 %count, ptr %input, ptr %history, ptr %private, ptr %stack, ptr %output) nounwind noinline {\nentry:\n",
    );
    let domains = [
        ("owner", "%count", 1),
        ("input", "24", 8),
        ("history", "40", 8),
        ("private", "65528", 16),
        ("stack", "4096", 16),
        ("output", "56", 8),
        ("topal_x86_tail_boundary_active_v1", "8", 8),
    ];
    let mut tests = Vec::new();
    for (index, (name, bytes, align)) in domains.iter().enumerate() {
        let prefix = if index == 6 { "@" } else { "%" };
        let _ = writeln!(
            text,
            "  %s{index} = ptrtoint ptr {prefix}{name} to i64\n  %e{index} = add i64 %s{index}, {bytes}\n  %n{index} = icmp ne i64 %s{index}, 0\n  %w{index} = icmp uge i64 %e{index}, %s{index}\n  %a{index} = and i64 %s{index}, {}\n  %b{index} = icmp eq i64 %a{index}, 0",
            align - 1
        );
        tests.extend([
            format!("%n{index}"),
            format!("%w{index}"),
            format!("%b{index}"),
        ]);
        for prior in 0..index {
            let _ = writeln!(
                text,
                "  %l{prior}_{index} = icmp ule i64 %e{prior}, %s{index}\n  %r{prior}_{index} = icmp ule i64 %e{index}, %s{prior}\n  %d{prior}_{index} = or i1 %l{prior}_{index}, %r{prior}_{index}"
            );
            tests.push(format!("%d{prior}_{index}"));
        }
    }
    text.push_str("  %nonempty = icmp ugt i64 %count, 0\n");
    let mut combined = "%nonempty".to_string();
    for (index, test) in tests.iter().enumerate() {
        let _ = writeln!(text, "  %t{index} = and i1 {combined}, {test}");
        combined = format!("%t{index}");
    }
    let _ = writeln!(text, "  ret i1 {combined}\n}}");
    text
}
