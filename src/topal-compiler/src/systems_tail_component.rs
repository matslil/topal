//! Unqualified source-step components; no kernel artifact grants this record authority.
use crate::{CompileError, SystemsTailStorageBound};
use serde::Serialize;
use sha2::{Digest, Sha256};
use topal_language::compiler::{CompilerProgram, CompilerType};

#[cfg(test)]
#[path = "systems_tail_component_tests.rs"]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailComponentFunction {
    pub identity: u64,
    pub symbol: String,
    pub has_region: bool,
    pub callees: Vec<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailReadonlyRoot {
    pub symbol: String,
    pub bytes: u64,
}
/// Diagnostic/native-test input only. It does not attest external IR as source,
/// qualify a bridge or make a component acceptable to the kernel publisher.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsTailStepComponent {
    pub schema: String,
    pub qualification: String,
    pub storage: SystemsTailStorageBound,
    pub functions: Vec<SystemsTailComponentFunction>,
    pub entry_identity: u64,
    pub readonly_roots: Vec<SystemsTailReadonlyRoot>,
    pub region_runtime_sha256: String,
    pub llvm_sha256: String,
    pub llvm: String,
}

pub fn emit_systems_tail_step_component(
    program: &CompilerProgram,
    source_name: &str,
    entry: &str,
) -> Result<SystemsTailStepComponent, CompileError> {
    let storage = crate::analyze_systems_tail_storage(program, entry)?;
    let identity = |symbol: &str| {
        program
            .functions
            .iter()
            .position(|f| f.symbol == symbol)
            .unwrap() as u64
            + 1
    };
    let functions = storage
        .source_plan
        .functions
        .iter()
        .map(|function| SystemsTailComponentFunction {
            identity: identity(&function.symbol),
            symbol: function.symbol.clone(),
            has_region: program
                .functions
                .iter()
                .find(|f| f.symbol == function.symbol)
                .unwrap()
                .parameters
                .iter()
                .any(|p| p.value_type == CompilerType::KernelByteRegion),
            callees: function
                .callees
                .iter()
                .map(|symbol| identity(symbol))
                .collect(),
        })
        .collect();
    let module = crate::codegen::emit_tail_step_module(
        program,
        source_name,
        entry,
        storage.unsigned_runtime_sha256.is_some(),
    )?;
    Ok(SystemsTailStepComponent {
        schema: "topal.tail-source-step-component.x86_64/1".into(),
        qualification: "source step component only; bridge, actual layout, stack/register reclamation and kernel execution unqualified".into(),
        storage, functions, entry_identity: identity(entry), readonly_roots: module.readonly_roots,
        region_runtime_sha256: module.region_runtime_sha256,
        llvm_sha256: format!("{:x}", Sha256::digest(module.llvm.as_bytes())), llvm: module.llvm,
    })
}
