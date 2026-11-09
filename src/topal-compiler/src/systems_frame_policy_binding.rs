//! Immutable source obligation for the actual allocator's kernel binding.
//! A checked source/component is not native caller or publication authority.
use crate::{CompileError, SystemsSourceLibraryInput, SystemsTailOwnerComponent};
use serde::Serialize;
use topal_language::compiler::{
    CompilerSystemsProgram, CompilerSystemsSourceLibraryInterface as Interface,
    CompilerSystemsTargetSelection, analyze_systems_for_compiler,
};

#[cfg(test)]
#[path = "systems_frame_policy_binding_tests.rs"]
mod tests;

#[derive(Debug, Serialize)]
pub(crate) struct SourceObligation {
    pub schema: String,
    pub qualification: String,
    pub root_source: String,
    pub root_sha256: String,
    pub library: crate::systems_tail_publication::Source,
    pub component_sha256: String,
    pub controller_sha256: String,
    pub bootstrap_target_sha256: String,
}

pub(crate) struct Checked {
    pub program: CompilerSystemsProgram,
    pub component: SystemsTailOwnerComponent,
    pub source: SourceObligation,
    pub controller: String,
    pub bootstrap_target: String,
}

fn invalid(message: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-FRAME-POLICY-BINDING: {message}"))
}

impl Checked {
    // Until the actual target owner/refinement/caller is rebuilt, neither this
    // source obligation nor the component may select the old native allocator.
    pub(crate) fn native_binding_required(&self) -> CompileError {
        invalid(format!(
            "complete native allocator ownership/zeroing/caller binding is required for {} (root {}, {} checked source functions, {} controller bytes, {} bootstrap target bytes)",
            self.program
                .frame_allocator_library
                .as_ref()
                .unwrap()
                .source,
            self.source.root_sha256,
            self.component
                .runtime
                .source_step
                .storage
                .source_plan
                .functions
                .len(),
            self.controller.len(),
            self.bootstrap_target.len(),
        ))
    }
}

pub(crate) fn check(
    root: &str,
    input: &SystemsSourceLibraryInput,
) -> Result<Checked, CompileError> {
    if root.len() > 1024 * 1024
        || input.source_name.is_empty()
        || input.source_name.len() > 4096
        || input.source_name.chars().any(char::is_control)
    {
        return Err(invalid("unqualified root/source snapshot"));
    }
    let program =
        analyze_systems_for_compiler(root, &CompilerSystemsTargetSelection::initial_x86_64_qemu())
            .map_err(CompileError::Diagnostic)?;
    let requirement = program
        .frame_allocator_library
        .as_ref()
        .ok_or_else(|| invalid("missing frame-allocator-library declaration"))?;
    requirement.validate().map_err(invalid)?;
    if requirement.interface != Interface::PhysicalFramePolicyTailV1
        || requirement.source != input.source
    {
        return Err(invalid("source and allocator slot disagree"));
    }
    topal_semantics::model_systems_transitions(&program).map_err(invalid)?;
    let modules = crate::systems_library::modules::validate(input)?;
    let body = topal_language::compiler::analyze_kernel_region_tail_nat_source(
        &input.text,
        &modules,
        &requirement.export,
        3,
    )
    .map_err(|e| CompileError::Tool(e.render(&input.source_name)))?;
    crate::systems_region_library::check_tail_region_isolation(input, &body, &modules)?;
    let entries = body
        .functions
        .iter()
        .filter(|function| {
            function.module_identity.is_none()
                && topal_source::canonically_equal(&function.source_name, &requirement.export)
        })
        .collect::<Vec<_>>();
    let [entry] = entries.as_slice() else {
        return Err(invalid("ambiguous allocator source entry"));
    };
    let component =
        crate::emit_systems_tail_nat_owner_component(&body, &input.source_name, &entry.symbol)?;
    let controller = crate::systems_frame_controller::llvm()?;
    let bootstrap_target = crate::systems_frame_target::llvm(&component)?;
    let source = SourceObligation {
        schema: "topal.frame-policy-source-obligation/3".into(),
        qualification: "checked root/import/source and standalone component only; actual kernel allocator ownership/caller/publication unqualified".into(),
        root_source: root.into(),
        root_sha256: crate::artifact::sha256(root.as_bytes()),
        library: input.into(),
        component_sha256: crate::artifact::sha256(&serde_json::to_vec(&component).map_err(invalid)?),
        controller_sha256: crate::artifact::sha256(controller.as_bytes()),
        bootstrap_target_sha256: crate::artifact::sha256(bootstrap_target.as_bytes()),
    };
    Ok(Checked {
        program,
        component,
        source,
        controller,
        bootstrap_target,
    })
}
