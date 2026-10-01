//! Ordered native compilation application service.

use crate::{CompileError, CompileOptions, NativeArtifactMetadata, backend, frontend, toolchain};

/// Run source through discovery, checking, lowering, verification, and atomic
/// artifact publication.
pub(crate) fn compile(
    source: &str,
    options: &CompileOptions,
) -> Result<NativeArtifactMetadata, CompileError> {
    let plan = crate::OptimizationPlan::resolve(&options.target, &options.optimization)?;
    if let Some(destination) = &options.optimization.explain {
        plan.publish_explanation(destination)?;
    }
    let checked = frontend::check(
        source,
        &options.library_root,
        options.standard_library.as_deref(),
    )?;
    let llvm = backend::lower(checked.program(), &options.source_name);
    toolchain::materialize(
        checked.program(),
        checked.static_archives(),
        checked.shared_objects(),
        checked.foreign_dependencies(),
        &llvm,
        options,
        &plan,
    )
}
