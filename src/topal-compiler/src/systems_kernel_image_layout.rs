//! Compiler-derived image subdivision, never live root or publication authority.
//!
//! TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001 / OWNERS-001. Fresh source reconstruction
//! owns the input classification. The result sizes an image subdivision bank;
//! it neither maps holes nor qualifies boot stacks, MMIO, user tables or CPUs.
use crate::systems_frame_publication::{self as frame, Binding};
use crate::{CompileError, LlvmTools};
use object::{Object, ObjectSection};
use serde::Serialize;
use std::collections::BTreeSet;

const PAGE: u64 = 4096;
const BLOCK: u64 = 2 * 1024 * 1024;
const LIMIT: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemsKernelImageRole {
    Code,
    Immutable,
    Mutable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemsKernelImageSpan {
    pub start: u64,
    pub end: u64,
    pub role: SystemsKernelImageRole,
}

/// A descriptive compiler product. This type is deliberately not deserializable
/// and is not accepted by any owner, native activation or publication interface.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SystemsKernelImageLayout {
    pub schema: String,
    pub kernel_sha256: String,
    pub minimum_granule: u64,
    pub large_granule: u64,
    pub address_limit: u64,
    pub spans: Vec<SystemsKernelImageSpan>,
    /// Physical 2 MiB regions requiring image subdivisions. Holes are still
    /// unspecified, rather than implicitly writable or executable mappings.
    pub subdivided_regions: Vec<u64>,
    /// Exact bytes for these additional 512-entry leaf tables only. This is not
    /// the complete root/context allocation or an allocator ownership witness.
    pub image_leaf_storage_bytes: u64,
}

fn invalid(message: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-KERNEL-IMAGE-LAYOUT: {message}"))
}

/// Rebuild every source/native contribution before deriving image subdivisions.
/// This opt-in inspection never changes existing artifact or boot contracts.
///
/// # Errors
/// Rejects incomplete/stale reconstruction, unsupported allocated storage and
/// incompatible page-rounded roles. A valid ELF or rehashed manifest alone is
/// insufficient. The current private process-MM producer is not qualified here.
pub fn plan_x86_64_systems_frame_kernel_image(
    kernel: &[u8],
    binding: &Binding,
    tools: &LlvmTools,
) -> Result<SystemsKernelImageLayout, CompileError> {
    if binding.schema != "topal.frame-root-reconstruction.x86_64/1"
        || binding.llvm_version != tools.version
    {
        return Err(invalid("unqualified reconstruction schema/toolchain"));
    }
    let scratch = crate::systems_tail_publication::Scratch::new()?;
    let mut fresh = frame::rebuild(&binding.root_source, &binding.library, tools, &scratch.0)?;
    // Always require the complete association, including when a candidate has
    // erased all library snapshots. Standalone component binding is not enough.
    fresh.complete_libraries(&binding.other_sources, tools, &scratch.0)?;
    fresh.bind(kernel)?;
    if fresh.record != *binding {
        return Err(invalid(
            "candidate differs from complete fresh reconstruction",
        ));
    }
    from_reconstructed(kernel, &fresh.record.linked, &fresh.record.linker_padding)
}

// Callers must freshly reconstruct complete contributions and allocation/load
// coverage first. This descriptive result is still never live image authority.
pub(crate) fn from_reconstructed(
    kernel: &[u8],
    placements: &[crate::systems_tail_linked::Placement],
    linker_padding: &[frame::LinkerExtent],
) -> Result<SystemsKernelImageLayout, CompileError> {
    use SystemsKernelImageRole::{Code, Immutable, Mutable};
    let file = object::File::parse(kernel).map_err(invalid)?;
    let mut regions = Vec::new();
    for section in file.sections() {
        let object::SectionFlags::Elf { sh_flags, .. } = section.flags() else {
            return Err(invalid("ELF section required"));
        };
        if sh_flags.0 & 2 == 0 || section.size() == 0 {
            continue;
        }
        let end = section
            .address()
            .checked_add(section.size())
            .ok_or_else(|| invalid("allocated extent overflow"))?;
        let inputs = placements
            .iter()
            .filter(|p| section.address() <= p.address && p.address < end)
            .collect::<Vec<_>>();
        let role = if !inputs.is_empty() {
            // bind() has reconstructed all bytes/relocations and required full
            // allocation coverage. Only generated semantic input flags classify
            // source storage; section names from an arbitrary ELF cannot do so.
            let role = |flags: u64| match flags & 7 {
                2 => Ok(Immutable),
                3 => Ok(Mutable),
                6 => Ok(Code),
                _ => Err(invalid("unsupported reconstructed semantic role")),
            };
            let expected = role(inputs[0].flags)?;
            if inputs.iter().any(|p| role(p.flags).ok() != Some(expected))
                || role(sh_flags.0)? != expected
            {
                return Err(invalid("inconsistent input/section roles"));
            }
            expected
        } else if linker_padding.iter().any(|p| {
            p.kind == "lld-empty-got-page-tail/1"
                && p.address == section.address()
                && p.bytes == section.size()
        }) {
            // Fully checked, unused zero reservation, with no relocation slots.
            Immutable
        } else {
            return Err(invalid("allocation lacks reconstructed semantic ownership"));
        };
        regions.push(SystemsKernelImageSpan {
            start: section.address(),
            end,
            role,
        });
    }
    for padding in linker_padding {
        if padding.kind == "lld-readonly-elf-header-load/1" {
            regions.push(SystemsKernelImageSpan {
                start: padding.address,
                end: padding
                    .address
                    .checked_add(padding.bytes)
                    .ok_or_else(|| invalid("header extent overflow"))?,
                role: Immutable,
            });
        } else if padding.kind != "lld-empty-got-page-tail/1" {
            return Err(invalid("unqualified linker-owned extent"));
        }
    }
    let spans = partition(regions)?;
    let mut candidates = BTreeSet::new();
    for span in &spans {
        candidates.insert(span.start / BLOCK * BLOCK);
        candidates.insert((span.end - 1) / BLOCK * BLOCK);
    }
    let subdivided_regions = candidates
        .into_iter()
        .filter(|start| {
            !spans
                .iter()
                .any(|span| span.start <= *start && *start + BLOCK <= span.end)
        })
        .collect::<Vec<_>>();
    Ok(SystemsKernelImageLayout {
        schema: "topal.kernel-image-layout.x86_64/1".into(),
        kernel_sha256: crate::artifact::sha256(kernel),
        minimum_granule: PAGE,
        large_granule: BLOCK,
        address_limit: LIMIT,
        image_leaf_storage_bytes: subdivided_regions.len() as u64 * PAGE,
        subdivided_regions,
        spans,
    })
}

fn partition(
    mut regions: Vec<SystemsKernelImageSpan>,
) -> Result<Vec<SystemsKernelImageSpan>, CompileError> {
    if regions.is_empty() {
        return Err(invalid("empty image"));
    }
    for region in &mut regions {
        if region.start >= region.end || region.end > LIMIT {
            return Err(invalid("unbounded or empty image extent"));
        }
        region.start &= !(PAGE - 1);
        region.end = region
            .end
            .checked_add(PAGE - 1)
            .ok_or_else(|| invalid("page rounding overflow"))?
            & !(PAGE - 1);
        if region.end > LIMIT {
            return Err(invalid("page-rounded image exceeds provider domain"));
        }
    }
    regions.sort_by_key(|r| (r.start, r.end));
    let mut spans: Vec<SystemsKernelImageSpan> = Vec::new();
    for region in regions {
        if let Some(previous) = spans.last_mut() {
            if region.start < previous.end && region.role != previous.role {
                return Err(invalid("incompatible image roles share a minimum granule"));
            }
            if region.start <= previous.end && region.role == previous.role {
                previous.end = previous.end.max(region.end);
                continue;
            }
        }
        spans.push(region);
    }
    Ok(spans)
}

#[cfg(test)]
#[path = "systems_kernel_image_layout_tests.rs"]
mod tests;
