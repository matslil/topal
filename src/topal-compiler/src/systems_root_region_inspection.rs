//! Exact native templates, owned storage and combined source/caller bounds.
use super::*;
use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRootRegionStorage {
    pub symbol: String,
    pub address: u64,
    pub bytes: u64,
    pub alignment: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRootRegionCode {
    pub symbol: String,
    pub address: u64,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRootRegionBinding {
    pub schema: String,
    pub interface: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub instance: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub scalar_count: Option<u8>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub live_frame_bytes: Option<u64>,
    pub maximum_source_stack_bytes: u64,
    pub caller_alignment_bytes: u64,
    pub combined_stack_bytes: u64,
    pub retired_stack_reserve_bytes: u64,
    pub imported_hook_stack_bytes: u64,
    pub storage: Vec<SystemsRootRegionStorage>,
    pub code: Vec<SystemsRootRegionCode>,
    pub source_code: Vec<SystemsRootRegionCode>,
    pub source_data: Vec<crate::SystemsRegionReadOnlyBinding>,
}
fn invalid(message: impl Into<String>) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-REGION-ROOT: {}", message.into()))
}
fn range(address: u64, bytes: u64) -> Result<(u64, u64), CompileError> {
    let end = address
        .checked_add(bytes)
        .ok_or_else(|| invalid("linked extent overflow"))?;
    if bytes == 0
        || address < 0x200000
        || end > 0x40000000
        || (address < 0x800000 && end > 0x400000)
    {
        return Err(invalid(
            "linked region extent is outside supervisor kernel storage",
        ));
    }
    Ok((address, end))
}

pub(crate) fn inspect_linked(
    bytes: &[u8],
    library: &crate::SystemsRegionLibraryArtifact,
    storage_bytes: u64,
) -> Result<SystemsRootRegionBinding, CompileError> {
    let framed = match (library.schema.as_str(), library.instance.as_deref()) {
        ("topal.kernel-region-source-object.x86_64/3", None) => false,
        (
            "topal.kernel-region-frame-source-object.x86_64/2",
            Some(crate::systems_region_library::ROOT_INSTANCE),
        ) => true,
        _ => return Err(invalid("incompatible root source instance/context")),
    };
    let symbols = library.native_symbols()?;
    let (export, allocate, abort_hook) = native_names(framed);
    let combined_stack_bytes = library
        .maximum_stack_bytes
        .checked_add(CALL_ALIGNMENT_BYTES)
        .and_then(|n| n.checked_add(if framed { LIVE_FRAME_BYTES } else { 0 }))
        .ok_or_else(|| invalid("combined source/caller stack overflow"))?;
    if library.scalar_count != framed.then_some(0)
        || library.allocation.adapter_family_sha256.as_deref()
            != if framed {
                Some("c33b5beb0f1869341bf24493e867f5d3101a578a12962b525c2613c1e09e3bbe")
            } else {
                None
            }
        || library.target != "x86_64-unknown-none"
        || library.data_layout != crate::X86_SYSTEMS_DATA_LAYOUT
        || library.native_export != export
        || library.declared_entry
            != if framed {
                "(KernelByteRegion) -> Boolean"
            } else {
                "(KernelByteRegion) -> Unit"
            }
        || library.imported_hooks != [allocate, abort_hook]
        || library.imported_hook_stack_bytes != 8
        || library.stack_limit_bytes != 4096
        || combined_stack_bytes > RETIRED_STACK_BYTES
        || library.allocation.numeric.maximum_bytes > ARENA_BYTES
        || library.allocation.numeric.limit_bytes != ARENA_BYTES
        || library.allocation.maximum_view_nodes > 256
        || library.allocation.control_bytes != 64
        || library.allocation.view_bytes != 64
    {
        return Err(invalid(
            "region source and root runtime certificate disagree",
        ));
    }
    let file = object::File::parse(bytes).map_err(|error| invalid(error.to_string()))?;
    if file.kind() != object::ObjectKind::Executable
        || file.architecture() != object::Architecture::X86_64
    {
        return Err(invalid("region root requires a linked x86-64 executable"));
    }
    let source_data = crate::inspect_systems_region_read_only_data(bytes, library)?;
    for data in &source_data {
        range(data.address, data.bytes)?;
    }
    let allocation = &library.allocation.numeric;
    let digest = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
    if library.machine_profile != "x86_64-owned-region-gpr-only/1"
        || library.native_abi
            != if framed {
                "topal.kernel-region-frame-control-view64-u64frame-bool.x86_64-sysv/1"
            } else {
                "topal.kernel-region-control-view64.x86_64-sysv/1"
            }
        || library.runtime_sha256 != digest(include_bytes!("runtime/kernel_regions.ll"))
        || library.platform_hooks_sha256
            != digest(include_bytes!("runtime/kernel_region_platform.ll"))
        || library.allocation.region_runtime_sha256 != library.runtime_sha256
        || library.allocation.adapter_sha256
            != if framed {
                crate::codegen::region_frame_adapter_sha256(0)
            } else {
                crate::codegen::region_adapter_sha256()
            }
        || allocation.certificate
            != if framed {
                "topal.kernel-region-frame-allocation/1"
            } else {
                "topal.kernel-region-allocation/1"
            }
        || allocation.runtime_sha256 != digest(include_bytes!("runtime/value_core.ll"))
        || allocation.collection_runtime_sha256
            != digest(include_bytes!("runtime/fundamental_containers.ll"))
        || allocation.collection_layout_sha256
            != digest(include_bytes!("runtime/list_int_layout.ll"))
        || allocation.bridge_bytes != 64
        || allocation.header_bytes != 32
        || allocation
            .source_evaluation_bytes
            .checked_add(64)
            .and_then(|n| n.checked_add(32))
            != Some(allocation.maximum_bytes)
        || allocation.maximum_operand_bits > 256
        || library.allocation.maximum_view_nodes == 0
    {
        return Err(invalid("inconsistent region allocation certificate"));
    }
    let mut owned = Vec::new();
    let mut source_code = Vec::new();
    let mut seen = BTreeSet::new();
    let mut costs = BTreeMap::from([(allocate.to_owned(), 8_u64), (abort_hook.to_owned(), 8_u64)]);
    for function in &library.functions {
        if !seen.insert(function.symbol.as_str()) {
            return Err(invalid("duplicate region function"));
        }
        let mut nested = 0;
        let mut callees = BTreeSet::new();
        for callee in &function.callees {
            if !callees.insert(callee) {
                return Err(invalid("duplicate region callee"));
            }
            nested = nested.max(
                *costs
                    .get(callee)
                    .ok_or_else(|| invalid("incomplete or cyclic region call certificate"))?,
            );
        }
        let cost = function
            .frame_bytes
            .checked_add(8)
            .and_then(|n| n.checked_add(nested))
            .ok_or_else(|| invalid("region call bound overflow"))?;
        if cost != function.maximum_call_path_bytes
            || costs.insert(function.symbol.clone(), cost).is_some()
        {
            return Err(invalid("inconsistent region call bound"));
        }
        let found = file
            .symbols()
            .filter(|symbol| {
                symbol.is_definition() && symbol.name().ok() == Some(function.symbol.as_str())
            })
            .collect::<Vec<_>>();
        if found.len() != 1 {
            return Err(invalid(
                "missing or ambiguous linked ordinary source/runtime function",
            ));
        }
        let symbol = &found[0];
        if symbol.kind() != SymbolKind::Text {
            return Err(invalid("invalid linked source code"));
        }
        range(symbol.address(), symbol.size())?;
        let section = file
            .section_by_index(
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("source code has no section"))?,
            )
            .map_err(|error| invalid(error.to_string()))?;
        if !matches!(section.flags(), object::SectionFlags::Elf { sh_flags, .. } if sh_flags.0 & (7 | object::elf::SHF_TLS.0) == 6)
        {
            return Err(invalid("source code is not immutable executable text"));
        }
        let offset = usize::try_from(
            symbol
                .address()
                .checked_sub(section.address())
                .ok_or_else(|| invalid("source code outside its section"))?,
        )
        .map_err(|_| invalid("source code offset overflow"))?;
        let count =
            usize::try_from(symbol.size()).map_err(|_| invalid("source code size overflow"))?;
        let end = offset
            .checked_add(count)
            .ok_or_else(|| invalid("source code extent overflow"))?;
        let data = section.data().map_err(|error| invalid(error.to_string()))?;
        let code = data
            .get(offset..end)
            .ok_or_else(|| invalid("truncated linked source code"))?;
        // Bind the relocated bytes, not just symbol presence or the unlinked
        // object digest. Boot inspection compares this compiler-owned binding.
        source_code.push(SystemsRootRegionCode {
            symbol: function.symbol.clone(),
            address: symbol.address(),
            bytes: symbol.size(),
            sha256: digest(code),
        });
    }
    let linked_names = file
        .symbols()
        .filter(|s| s.is_definition() && s.kind() == SymbolKind::Text)
        .filter_map(|s| s.name().ok())
        .filter(|name| {
            *name == export
                || name.starts_with(&symbols.prefix)
                    && (framed || !name.starts_with(crate::systems_region_library::ACTION_PREFIX))
                || !framed && name.starts_with("topal.kernel.region.")
        })
        .collect::<Vec<_>>();
    if linked_names.len() != seen.len() || linked_names.iter().any(|name| !seen.contains(name)) {
        return Err(invalid("extra or missing root source/runtime code"));
    }
    if costs.get(export).and_then(|cost| cost.checked_add(8)) != Some(library.maximum_stack_bytes) {
        return Err(invalid("inconsistent complete region stack certificate"));
    }
    let bootstrap = file
        .symbol_by_name("topal_bootstrap_storage")
        .ok_or_else(|| invalid("missing bootstrap owner"))?;
    if bootstrap.size() != storage_bytes {
        return Err(invalid("bootstrap owner extent mismatch"));
    }
    let bootstrap_range = range(bootstrap.address(), storage_bytes)?;
    let mut extents = vec![bootstrap_range];
    for (name, count, alignment) in [
        (ARENA, ARENA_BYTES, 4096),
        (CONTROL, 64, 8),
        (HISTORY, 8, 8),
    ] {
        let symbol = file
            .symbol_by_name(name)
            .ok_or_else(|| invalid(format!("missing root storage {name}")))?;
        let extent = range(symbol.address(), count)?;
        if symbol.kind() != SymbolKind::Data
            || !symbol.is_definition()
            || symbol.size() != count
            || symbol.address() % alignment != 0
            || extents
                .iter()
                .any(|(start, end)| extent.0 < *end && *start < extent.1)
        {
            return Err(invalid("overlapping or mismatched root storage"));
        }
        let section = file
            .section_by_index(
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("root storage has no section"))?,
            )
            .map_err(|error| invalid(error.to_string()))?;
        if section.kind() != SectionKind::UninitializedData {
            return Err(invalid("root storage is not zero-initialized owned BSS"));
        }
        if !matches!(section.flags(), object::SectionFlags::Elf { sh_flags, .. } if sh_flags.0 & (7 | object::elf::SHF_TLS.0) == 3)
            || extent.0 < section.address()
            || extent.1
                > section
                    .address()
                    .checked_add(section.size())
                    .ok_or_else(|| invalid("root BSS extent overflow"))?
        {
            return Err(invalid(
                "root storage is not contained in allocated writable BSS",
            ));
        }
        for other in file.symbols().filter(|s| {
            s.is_definition()
                && s.kind() == SymbolKind::Data
                && s.size() > 0
                && s.name().ok() != Some(name)
        }) {
            let end = other
                .address()
                .checked_add(other.size())
                .ok_or_else(|| invalid("other owned extent overflow"))?;
            if extent.0 < end && other.address() < extent.1 {
                return Err(invalid(
                    "root invocation storage aliases another allocation",
                ));
            }
        }
        extents.push(extent);
        owned.push(SystemsRootRegionStorage {
            symbol: name.into(),
            address: symbol.address(),
            bytes: count,
            alignment,
        });
    }
    let mut records = Vec::new();
    let abort = Code {
        encoder: X86FunctionEncoder {
            bytes: vec![0xfa, 0xf4, 0xeb, 0xfd],
            ..X86FunctionEncoder::default()
        },
        references: Vec::new(),
        unwind: Vec::new(),
    };
    for (name, code) in [
        (APPLY, apply(storage_bytes, framed)?),
        (allocate, allocator(framed)?),
        (abort_hook, abort),
        (FINALIZE, finalize(framed)?),
    ] {
        let symbol = file
            .symbol_by_name(name)
            .ok_or_else(|| invalid(format!("missing native root function {name}")))?;
        let Code {
            encoder,
            references,
            ..
        } = code;
        let mut expected = encoder.finish()?;
        let extent = range(symbol.address(), symbol.size())?;
        if symbol.kind() != SymbolKind::Text
            || !symbol.is_definition()
            || symbol.size() != expected.len() as u64
            || extents
                .iter()
                .any(|(start, end)| extent.0 < *end && *start < extent.1)
        {
            return Err(invalid("root function size, kind or placement mismatch"));
        }
        for (offset, target, relative) in references {
            let target = file
                .symbol_by_name(target)
                .ok_or_else(|| invalid("missing linked root dependency"))?;
            if !target.is_definition() {
                return Err(invalid("undefined root dependency"));
            }
            if relative {
                let next = symbol
                    .address()
                    .checked_add(offset)
                    .and_then(|value| value.checked_add(4))
                    .ok_or_else(|| invalid("root call-site overflow"))?;
                let delta = i128::from(target.address()) - i128::from(next);
                let delta = i32::try_from(delta)
                    .map_err(|_| invalid("linked root branch is out of range"))?;
                expected[offset as usize..offset as usize + 4]
                    .copy_from_slice(&delta.to_le_bytes());
            } else {
                expected[offset as usize..offset as usize + 8]
                    .copy_from_slice(&target.address().to_le_bytes());
            }
        }
        let section = file
            .section_by_index(
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("root code has no section"))?,
            )
            .map_err(|error| invalid(error.to_string()))?;
        if !matches!(section.flags(), object::SectionFlags::Elf { sh_flags, .. } if sh_flags.0 & (7 | object::elf::SHF_TLS.0) == 6)
        {
            return Err(invalid("native root code is not immutable allocated text"));
        }
        let data = section.data().map_err(|error| invalid(error.to_string()))?;
        let offset = usize::try_from(
            symbol
                .address()
                .checked_sub(section.address())
                .ok_or_else(|| invalid("root code outside section"))?,
        )
        .map_err(|_| invalid("root code offset overflow"))?;
        let actual = data
            .get(offset..offset + expected.len())
            .ok_or_else(|| invalid("truncated root code"))?;
        if actual != expected {
            return Err(invalid(format!(
                "linked native root template differs: {name}"
            )));
        }
        records.push(SystemsRootRegionCode {
            symbol: name.into(),
            address: symbol.address(),
            bytes: symbol.size(),
            sha256: format!("{:x}", Sha256::digest(actual)),
        });
    }
    Ok(SystemsRootRegionBinding {
        schema: if framed {
            "topal.root-region-invocation.x86_64/3"
        } else {
            "topal.root-region-invocation.x86_64/2"
        }
        .into(),
        interface: if framed {
            "borrowed-byte-region-instance-action-v1"
        } else {
            "borrowed-byte-region-v1"
        }
        .into(),
        instance: symbols.instance.clone(),
        scalar_count: framed.then_some(0),
        live_frame_bytes: framed.then_some(LIVE_FRAME_BYTES),
        maximum_source_stack_bytes: library.maximum_stack_bytes,
        caller_alignment_bytes: CALL_ALIGNMENT_BYTES,
        combined_stack_bytes,
        retired_stack_reserve_bytes: RETIRED_STACK_BYTES,
        imported_hook_stack_bytes: 8,
        storage: owned,
        code: records,
        source_code,
        source_data,
    })
}
