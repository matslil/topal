//! Target-specific lowering boundary.

use topal_language::compiler::CompilerProgram;

use crate::codegen;

/// One complete LLVM module produced by the qualified native backend.
///
/// The constructor is private so toolchain publication can only receive LLVM
/// text produced from a checked program through this backend.
pub(crate) struct LlvmModule {
    text: String,
}

impl LlvmModule {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }
}

/// Lower a checked program to the compiler's qualified LLVM representation.
pub(crate) fn lower(program: &CompilerProgram, source_name: &str) -> LlvmModule {
    LlvmModule {
        text: codegen::emit_llvm(program, source_name),
    }
}
