//! Instance labels own link namespaces, never live region authority.
use super::*;

pub(crate) const INSTANCE_PREFIX: &str = "topal.region-instance.";
pub(crate) const ROOT_INSTANCE: &str = "root-region";
pub(crate) const FILE_INSTANCE: &str = "file-position";
pub(crate) const PAGE_INSTANCE: &str = "ram-page-pool";
pub(crate) const INODE_INSTANCE: &str = "inode-metadata";
pub(crate) const INODE_FRAME_PREFIX: &str = "topal.region-instance.inode-metadata.";
pub(crate) const INODE_FRAME_EXPORT: &str = "topal_kernel_region_frame_inode_metadata_apply_v1";
pub(crate) const INODE_FRAME_ALLOCATE: &str =
    "topal_kernel_region_frame_inode_metadata_allocate_v1";
pub(crate) const INODE_FRAME_ABORT: &str = "topal_kernel_region_frame_inode_metadata_abort_v1";
pub(crate) const PAGE_FRAME_PREFIX: &str = "topal.region-instance.ram-page-pool.";
pub(crate) const PAGE_FRAME_EXPORT: &str = "topal_kernel_region_frame_ram_page_pool_apply_v1";
pub(crate) const PAGE_FRAME_ALLOCATE: &str = "topal_kernel_region_frame_ram_page_pool_allocate_v1";
pub(crate) const PAGE_FRAME_ABORT: &str = "topal_kernel_region_frame_ram_page_pool_abort_v1";
pub(crate) const ROOT_FRAME_PREFIX: &str = "topal.region-instance.root-region.";
pub(crate) const FILE_FRAME_PREFIX: &str = "topal.region-instance.file-position.";
pub(crate) const ROOT_FRAME_EXPORT: &str = "topal_kernel_region_frame_root_region_apply_v1";
pub(crate) const ROOT_FRAME_ALLOCATE: &str = "topal_kernel_region_frame_root_region_allocate_v1";
pub(crate) const ROOT_FRAME_ABORT: &str = "topal_kernel_region_frame_root_region_abort_v1";
pub(crate) const FILE_FRAME_EXPORT: &str = "topal_kernel_region_frame_file_position_apply_v1";
pub(crate) const FILE_FRAME_ALLOCATE: &str = "topal_kernel_region_frame_file_position_allocate_v1";
pub(crate) const FILE_FRAME_ABORT: &str = "topal_kernel_region_frame_file_position_abort_v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionNativeSymbols {
    pub(crate) instance: Option<String>,
    pub(crate) export: String,
    pub(crate) allocate: String,
    pub(crate) abort: String,
    pub(crate) prefix: String,
}

impl RegionNativeSymbols {
    pub(crate) fn for_profile(
        profile: CompilerKernelRegionProfile,
        instance: Option<&str>,
    ) -> Result<Self, CompileError> {
        if let Some(label) = instance {
            if !matches!(profile, CompilerKernelRegionProfile::FramedAction { .. })
                || label.is_empty()
                || label.len() > 48
                || !label.as_bytes()[0].is_ascii_lowercase()
                || label.ends_with('-')
                || label.contains("--")
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                return Err(invalid("invalid framed source instance identity"));
            }
            let stem = label.replace('-', "_");
            return Ok(Self {
                instance: Some(label.into()),
                export: format!("topal_kernel_region_frame_{stem}_apply_v1"),
                allocate: format!("topal_kernel_region_frame_{stem}_allocate_v1"),
                abort: format!("topal_kernel_region_frame_{stem}_abort_v1"),
                prefix: format!("{INSTANCE_PREFIX}{label}."),
            });
        }
        let (export, allocate, abort, prefix) = match profile {
            CompilerKernelRegionProfile::Unit => (EXPORT, ALLOCATE, ABORT, PREFIX),
            CompilerKernelRegionProfile::Action => {
                (ACTION_EXPORT, ACTION_ALLOCATE, ACTION_ABORT, ACTION_PREFIX)
            }
            CompilerKernelRegionProfile::FramedAction { .. } => {
                (FRAME_EXPORT, FRAME_ALLOCATE, FRAME_ABORT, FRAME_PREFIX)
            }
        };
        Ok(Self {
            instance: None,
            export: export.into(),
            allocate: allocate.into(),
            abort: abort.into(),
            prefix: prefix.into(),
        })
    }
}

impl SystemsRegionLibraryArtifact {
    pub(crate) fn native_symbols(&self) -> Result<RegionNativeSymbols, CompileError> {
        use CompilerKernelRegionProfile as Profile;
        let (profile, instance) = match (
            self.schema.as_str(),
            self.scalar_count,
            self.instance.as_deref(),
        ) {
            (
                "topal.kernel-region-source-object.x86_64/3"
                | "topal.kernel-region-source-object.x86_64/4",
                None,
                None,
            ) => (Profile::Unit, None),
            (
                "topal.kernel-region-action-source-object.x86_64/1"
                | "topal.kernel-region-action-source-object.x86_64/2",
                None,
                None,
            ) => (Profile::Action, None),
            (
                "topal.kernel-region-frame-source-object.x86_64/1"
                | "topal.kernel-region-frame-source-object.x86_64/3",
                Some(count),
                None,
            ) if count <= topal_language::compiler::MAX_KERNEL_REGION_FRAME_SCALARS => (
                Profile::FramedAction {
                    scalar_count: count,
                },
                None,
            ),
            (
                "topal.kernel-region-frame-source-object.x86_64/2"
                | "topal.kernel-region-frame-source-object.x86_64/4",
                Some(count),
                Some(label),
            ) if count <= topal_language::compiler::MAX_KERNEL_REGION_FRAME_SCALARS => (
                Profile::FramedAction {
                    scalar_count: count,
                },
                Some(label),
            ),
            _ => return Err(invalid("incompatible region instance/profile certificate")),
        };
        let runtime = self.allocation.unsigned_runtime_sha256.as_deref();
        if self.schema != source_object_schema(profile, instance.is_some(), runtime.is_some())
            || runtime.is_some_and(|digest| digest != UNSIGNED_RUNTIME_SHA256)
        {
            return Err(invalid("unsigned runtime/profile certificate mismatch"));
        }
        let symbols = RegionNativeSymbols::for_profile(profile, instance)?;
        let unsigned_prefix = if profile == Profile::Unit {
            "topal.kernel.region.unsigned.".to_owned()
        } else {
            format!("{}primitive.unsigned.", symbols.prefix)
        };
        let unsigned_code = self
            .functions
            .iter()
            .any(|function| function.symbol.starts_with(&unsigned_prefix));
        // Couple the schema/digest to actual retained helper inputs as well.
        // Removing the digest together with a schema downgrade cannot turn
        // unsigned code into a byte-only certificate, and an empty upgrade
        // cannot attach orphan unsigned evidence to byte-only code.
        if unsigned_code != runtime.is_some() {
            return Err(invalid("unsigned runtime/code presence mismatch"));
        }
        if self.native_export != symbols.export
            || self.imported_hooks != [symbols.allocate.clone(), symbols.abort.clone()]
        {
            return Err(invalid("region instance entry/hook identity mismatch"));
        }
        Ok(symbols)
    }
}

/// Complete namespace registration is checked only after both owning caller
/// bindings; labels/metadata alone do not qualify a live native context.
pub(crate) fn validate_instance_namespaces(
    kernel: &[u8],
    libraries: &[Option<&SystemsRegionLibraryArtifact>],
) -> Result<(), CompileError> {
    let expected = libraries
        .iter()
        .flatten()
        .map(|library| library.native_symbols())
        .collect::<Result<Vec<_>, _>>()?;
    let file = object::File::parse(kernel).map_err(|e| invalid(e.to_string()))?;
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|e| invalid(e.to_string()))?;
        if name.starts_with(INSTANCE_PREFIX)
            && !expected
                .iter()
                .any(|n| n.instance.is_some() && name.starts_with(&n.prefix))
        {
            return Err(invalid("unselected region instance namespace"));
        }
        if name.starts_with("topal_kernel_region_frame_")
            && !expected
                .iter()
                .any(|n| name == n.export || name == n.allocate || name == n.abort)
        {
            return Err(invalid("unselected framed native entry/hook"));
        }
    }
    Ok(())
}
