//! Ordered native compilation application service.

use crate::{CompileError, CompileOptions, NativeArtifactMetadata, backend, frontend, toolchain};

/// Run source through discovery, checking, lowering, verification, and atomic
/// artifact publication.
pub(crate) fn compile(
    source: &str,
    options: &CompileOptions,
) -> Result<NativeArtifactMetadata, CompileError> {
    let checked = frontend::check(source, &options.library_root)?;
    let llvm = backend::lower(checked.program(), &options.source_name);
    toolchain::materialize(checked.program(), &llvm, options)
}
