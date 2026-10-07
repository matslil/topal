//! Ordinary compiler application service for qualified systems artifacts.

use topal_language::compiler::{
    CompilerSystemsTargetSelection, INITIAL_SYSTEMS_BOARD, INITIAL_SYSTEMS_TARGET,
    analyze_systems_for_compiler,
};

use crate::{
    CompileError, CompileOptions, Emit, LlvmTools, OptimizationRequest, PublishedSystemsArtifact,
    publish_x86_64_systems_artifact,
};

pub(crate) fn selected(options: &CompileOptions) -> bool {
    options.target.target.as_deref() == Some(INITIAL_SYSTEMS_TARGET)
}

pub(crate) fn publish(
    source: &str,
    options: &CompileOptions,
) -> Result<PublishedSystemsArtifact, CompileError> {
    validate_options(options)?;
    let program = analyze_systems_for_compiler(
        source,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .map_err(CompileError::Diagnostic)?;
    let tools = LlvmTools::discover(options.llvm_tools.as_deref())?;
    publish_x86_64_systems_artifact(&program, &tools, &options.output)
}

fn validate_options(options: &CompileOptions) -> Result<(), CompileError> {
    if options.emit != Emit::Executable {
        return Err(CompileError::Tool(
            "the qualified systems target publishes only its canonical executable artifact set"
                .into(),
        ));
    }
    if options.target.board.as_deref() != Some(INITIAL_SYSTEMS_BOARD)
        || options.target.cpu.as_deref().unwrap_or("generic") != "generic"
        || options.target.model.is_some()
    {
        return Err(CompileError::Tool(format!(
            "systems target `{INITIAL_SYSTEMS_TARGET}` requires --cpu generic and --board {INITIAL_SYSTEMS_BOARD} without a custom target model"
        )));
    }
    if options.optimization != OptimizationRequest::default() {
        return Err(CompileError::Tool(
            "the qualified systems target admits only the default unoptimized publication profile"
                .into(),
        ));
    }
    if options.standard_library.is_some() {
        return Err(CompileError::Tool(
            "the qualified systems target has no hosted standard-library dependency".into(),
        ));
    }
    Ok(())
}
