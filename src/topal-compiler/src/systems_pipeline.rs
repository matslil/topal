//! Ordinary compiler application service for qualified systems artifacts.

use topal_language::compiler::{
    CompilerSystemsTargetSelection, INITIAL_SYSTEMS_BOARD, INITIAL_SYSTEMS_TARGET,
    analyze_systems_for_compiler,
};

use crate::{
    CompileError, CompileOptions, Emit, LlvmTools, OptimizationRequest, PublishedSystemsArtifact,
    publish_x86_64_systems_artifact, publish_x86_64_systems_artifact_with_initial_user_image,
    publish_x86_64_systems_artifact_with_static_user_image,
    publish_x86_64_systems_artifact_with_user_images,
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
    let initial_user_image = options
        .initial_user_image
        .as_ref()
        .map(|path| {
            std::fs::read(path).map_err(|error| {
                CompileError::Io(format!(
                    "cannot read initial user image {}: {error}",
                    path.display()
                ))
            })
        })
        .transpose()?;
    let static_user_image = options
        .static_user_image
        .as_ref()
        .map(|path| {
            std::fs::read(path).map_err(|error| {
                CompileError::Io(format!(
                    "cannot read static user image {}: {error}",
                    path.display()
                ))
            })
        })
        .transpose()?;
    if program.kernel_library.is_some() || program.time_library.is_some() {
        let source = program
            .kernel_library
            .as_ref()
            .map(|r| crate::systems_library::load_source_library(r, &options.library_root))
            .transpose()?;
        let time = program
            .time_library
            .as_ref()
            .map(|r| crate::systems_library::load_source_library(r, &options.library_root))
            .transpose()?;
        return crate::publish_x86_64_systems_artifact_with_libraries(
            &program,
            &tools,
            initial_user_image.as_deref(),
            static_user_image.as_deref(),
            source.as_ref(),
            time.as_ref(),
            &options.output,
        );
    }
    match (initial_user_image.as_deref(), static_user_image.as_deref()) {
        (Some(initial), Some(static_image)) => publish_x86_64_systems_artifact_with_user_images(
            &program,
            &tools,
            initial,
            Some(static_image),
            &options.output,
        ),
        (Some(initial), None) => publish_x86_64_systems_artifact_with_initial_user_image(
            &program,
            &tools,
            initial,
            &options.output,
        ),
        (None, Some(static_image)) => publish_x86_64_systems_artifact_with_static_user_image(
            &program,
            &tools,
            static_image,
            &options.output,
        ),
        (None, None) => publish_x86_64_systems_artifact(&program, &tools, &options.output),
    }
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
