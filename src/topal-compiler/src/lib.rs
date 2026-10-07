//! Correctness-first native compilation for the currently admitted Topal slice.

mod artifact;
mod backend;
mod codegen;
mod frontend;
mod optimization;
mod pipeline;
mod standard_library;
mod systems_artifact;
mod systems_boot_image;
mod systems_pipeline;
mod systems_provider;
mod systems_provider_object;
mod toolchain;

use std::fmt;
use std::path::{Path, PathBuf};

pub use artifact::{
    DigestEntry, ExportEntry, NATIVE_ABI, NATIVE_ARTIFACT_SCHEMA, NativeArtifactMetadata,
    NativeSlice, PLATFORM_ABI,
};
pub use optimization::{
    ExplanationDestination, GENERIC_X86_64_MODEL, LLVM_DEFAULT_PIPELINE,
    OPTIMIZATION_PLAN_REVISION, OptimizationLevel, OptimizationOverride, OptimizationPlan,
    OptimizationRequest, RUNTIME_GLOBAL_DCE, TARGET_REGISTRY_REVISION, TargetSelection,
    optimization_listing, target_listing,
};
pub use standard_library::{
    STANDARD_LIBRARY_ABI, STANDARD_LIBRARY_ENTRY, STANDARD_LIBRARY_SCHEMA, STANDARD_LIBRARY_SONAME,
    StandardLibrarySlice, build_standard_library, standard_library_manifest_path,
};
pub use systems_artifact::{
    PublishedSystemsArtifact, SYSTEMS_DEBUG_FILE, SYSTEMS_KERNEL_FILE, SYSTEMS_MAP_FILE,
    SYSTEMS_PROVENANCE_FILE, SystemsArtifactPlacement, SystemsArtifactProvenance,
    X86_SYSTEMS_ARTIFACT_REVISION, X86_SYSTEMS_DEBUG_BREAK_ENTRY, X86_SYSTEMS_KERNEL_ENTRY,
    X86_SYSTEMS_ROOT_OBJECT_REVISION, X86_SYSTEMS_ROOT_TEXT_SECTION,
    publish_x86_64_systems_artifact,
};
pub use systems_boot_image::{
    GeneratedX86LinuxBootImage, PublishedX86LinuxBootImage, X86_BOOT_IMAGE_FILE,
    X86_BOOT_PROVENANCE_FILE, X86_INITIAL_IDENTITY_LIMIT, X86_INITIAL_STACK_TOP,
    X86_KERNEL_MINIMUM_ADDRESS, X86_LINUX_BOOT_ADAPTER_REVISION, X86_LINUX_BOOT_PROTOCOL,
    X86_LINUX_SETUP_SECTORS, X86_PROTECTED_PAYLOAD_ADDRESS, X86LinuxBootImageProvenance,
    generate_x86_64_linux_boot_image, publish_x86_64_linux_boot_image,
};
pub use systems_provider::{
    SystemsBootstrapPlacementPlan, SystemsProviderOperationPlan, X86_SYSTEMS_DATA_LAYOUT,
    X86_SYSTEMS_PLATFORM_ABI, X86_SYSTEMS_PROVIDER_REVISION, X86SystemsLowering,
    X86SystemsProviderPlan, plan_x86_64_systems_provider,
};
pub use systems_provider_object::{
    GeneratedSystemsProviderObject, X86_SYSTEMS_ALLOCATABLE_FLOOR, X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
    X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION, X86_SYSTEMS_PROVIDER_NOTE_SECTION,
    X86_SYSTEMS_PROVIDER_OBJECT_REVISION, X86_SYSTEMS_PROVIDER_TEXT_SECTION,
    generate_x86_64_systems_provider_object,
};
pub use toolchain::LlvmTools;
use topal_source::Diagnostic;

pub const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
pub const DATA_LAYOUT: &str =
    "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";
pub const LLVM_MAJOR: u32 = 22;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Emit {
    LlvmIr,
    Object,
    Executable,
}

impl Emit {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LlvmIr => "llvm-ir",
            Self::Object => "object",
            Self::Executable => "executable",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub source_name: String,
    pub output: PathBuf,
    pub emit: Emit,
    pub llvm_tools: Option<PathBuf>,
    pub library_root: PathBuf,
    pub target: TargetSelection,
    pub optimization: OptimizationRequest,
    pub standard_library: Option<PathBuf>,
}

#[derive(Debug)]
pub enum CompileError {
    Diagnostic(Diagnostic),
    Tool(String),
    Io(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(diagnostic) => diagnostic.fmt(formatter),
            Self::Tool(message) | Self::Io(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for CompileError {}

/// Compile source and publish the output with its canonical metadata sidecar.
///
/// # Errors
///
/// Returns a source diagnostic, an LLVM tool failure, or an I/O failure. LLVM
/// failures occur before the requested output path is published.
pub fn compile_source(
    source: &str,
    options: &CompileOptions,
) -> Result<NativeArtifactMetadata, CompileError> {
    pipeline::compile(source, options)
}

/// Check and atomically publish a qualified freestanding systems artifact.
///
/// # Errors
///
/// Returns a source diagnostic, target/profile rejection, LLVM tool failure,
/// structural-inspection failure, or I/O failure before partial publication.
pub fn publish_systems_source(
    source: &str,
    options: &CompileOptions,
) -> Result<PublishedSystemsArtifact, CompileError> {
    systems_pipeline::publish(source, options)
}

#[must_use]
pub fn selects_systems_publication(options: &CompileOptions) -> bool {
    systems_pipeline::selected(options)
}

#[must_use]
pub fn metadata_path(output: &Path) -> PathBuf {
    let mut name = output.as_os_str().to_owned();
    name.push(".topal.json");
    PathBuf::from(name)
}
