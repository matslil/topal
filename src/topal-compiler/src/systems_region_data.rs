//! Immutable region inputs are bound independently of relocated instructions.
use super::{CompileError, digest, invalid};
use object::{Object, ObjectSection, ObjectSymbol, SectionFlags, SymbolKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRegionReadOnlyData {
    pub symbol: String,
    pub bytes: u64,
    pub alignment: u64,
    pub sha256: String,
}

pub(super) fn object_data(
    bytes: &[u8],
    prefix: &str,
    allow_empty: bool,
) -> Result<Vec<SystemsRegionReadOnlyData>, CompileError> {
    let file = object::File::parse(bytes).map_err(|e| invalid(e.to_string()))?;
    let mut records = Vec::new();
    let mut names = BTreeSet::new();
    for symbol in file
        .symbols()
        .filter(|s| s.is_definition() && s.kind() == SymbolKind::Data)
    {
        let name = symbol.name().map_err(|e| invalid(e.to_string()))?;
        let section = file
            .section_by_index(
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("unplaced immutable data"))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if !matches!(section.flags(), SectionFlags::Elf { sh_flags, .. } if sh_flags.0 & (7 | object::elf::SHF_TLS.0) == 2)
            || !owned_name(name, prefix)
            || symbol.is_global()
            || !names.insert(name.to_owned())
            || symbol.size() == 0
            || section.align() < 8
            || symbol.address() % 8 != 0
            || section.relocations().next().is_some()
        {
            return Err(invalid(
                "unqualified region immutable data identity/placement/relocation",
            ));
        }
        let start = usize::try_from(
            symbol
                .address()
                .checked_sub(section.address())
                .ok_or_else(|| invalid("immutable data outside its section"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let end = start
            .checked_add(usize::try_from(symbol.size()).map_err(|e| invalid(e.to_string()))?)
            .ok_or_else(|| invalid("immutable data extent overflow"))?;
        let data = section.data().map_err(|e| invalid(e.to_string()))?;
        let data = data
            .get(start..end)
            .ok_or_else(|| invalid("truncated immutable source data"))?;
        records.push(SystemsRegionReadOnlyData {
            symbol: name.into(),
            bytes: symbol.size(),
            alignment: 8,
            sha256: digest(data),
        });
    }
    if records.is_empty() && !allow_empty {
        return Err(invalid("action object omits named immutable data"));
    }
    // Integer data need no dynamic native pointer relocation. Refuse hidden
    // allocated constants instead of leaving them outside the source binding.
    for section in file
        .sections()
        .filter(|s| matches!(s.flags(), SectionFlags::Elf { sh_flags, .. } if sh_flags.0 & (7 | object::elf::SHF_TLS.0) == 2))
    {
        let data = section.data().map_err(|e| invalid(e.to_string()))?;
        let mut covered = vec![false; data.len()];
        for symbol in file.symbols().filter(|s| {
            s.is_definition()
                && s.kind() == SymbolKind::Data
                && s.section_index() == Some(section.index())
        }) {
            let start = usize::try_from(
                symbol
                    .address()
                    .checked_sub(section.address())
                    .ok_or_else(|| invalid("immutable data coverage outside section"))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
            let end = start
                .checked_add(usize::try_from(symbol.size()).map_err(|e| invalid(e.to_string()))?)
                .ok_or_else(|| invalid("immutable data coverage overflow"))?;
            covered
                .get_mut(start..end)
                .ok_or_else(|| invalid("immutable data coverage outside section"))?
                .fill(true);
        }
        if data
            .iter()
            .zip(covered)
            .any(|(byte, covered)| !covered && *byte != 0)
        {
            return Err(invalid("region object contains unbound immutable data"));
        }
    }
    records.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Ok(records)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRegionReadOnlyBinding {
    pub symbol: String,
    pub address: u64,
    pub bytes: u64,
    pub alignment: u64,
    pub sha256: String,
}

/// Inspect immutable region data in a linked x86-64 executable. This supplies
/// no field owner, callable authority, stack or physical reclamation evidence.
pub fn inspect_systems_region_action_data(
    bytes: &[u8],
    library: &super::SystemsRegionLibraryArtifact,
) -> Result<Vec<SystemsRegionReadOnlyBinding>, CompileError> {
    if library.schema != "topal.kernel-region-action-source-object.x86_64/1"
        || library.native_export != super::ACTION_EXPORT
        || library.read_only_data.is_empty()
    {
        return Err(invalid(
            "missing or incompatible immutable region data certificate",
        ));
    }
    inspect_systems_region_read_only_data(bytes, library)
}

fn owned_name(name: &str, prefix: &str) -> bool {
    name.starts_with(prefix) && (prefix != super::PREFIX || !name.starts_with(super::ACTION_PREFIX))
}

/// Inspect complete immutable Unit/action/framed inputs. This does not grant a live
/// region or qualify native callers, bounds, owner lifetimes or reclamation.
pub fn inspect_systems_region_read_only_data(
    bytes: &[u8],
    library: &super::SystemsRegionLibraryArtifact,
) -> Result<Vec<SystemsRegionReadOnlyBinding>, CompileError> {
    use object::{Architecture, BinaryFormat, ObjectKind};
    let symbols = library.native_symbols()?;
    let prefix = symbols.prefix.as_str();
    if library.schema == "topal.kernel-region-action-source-object.x86_64/1"
        && library.read_only_data.is_empty()
    {
        return Err(invalid("incompatible immutable region data certificate"));
    }
    if library.scalar_count.is_none() && library.allocation.adapter_family_sha256.is_some() {
        return Err(invalid(
            "legacy region data cannot carry framed adapter metadata",
        ));
    }
    let file = object::File::parse(bytes).map_err(|e| invalid(e.to_string()))?;
    if file.format() != BinaryFormat::Elf
        || file.architecture() != Architecture::X86_64
        || file.kind() != ObjectKind::Executable
        || !file.is_little_endian()
    {
        return Err(invalid(
            "immutable region data requires a linked x86-64 executable",
        ));
    }
    let mut names = BTreeSet::new();
    let mut bindings = Vec::new();
    let mut previous = None;
    for expected in &library.read_only_data {
        if !owned_name(&expected.symbol, prefix)
            || !names.insert(expected.symbol.as_str())
            || previous.is_some_and(|name| name >= expected.symbol.as_str())
            || expected.alignment != 8
            || expected.bytes == 0
        {
            return Err(invalid(
                "invalid or unordered immutable region data identity",
            ));
        }
        previous = Some(expected.symbol.as_str());
        let matches = file
            .symbols()
            .filter(|symbol| {
                symbol.is_definition() && symbol.name().ok() == Some(expected.symbol.as_str())
            })
            .collect::<Vec<_>>();
        let [symbol] = matches.as_slice() else {
            return Err(invalid("missing or ambiguous linked immutable region data"));
        };
        if symbol.kind() != SymbolKind::Data
            || symbol.is_global()
            || symbol.size() != expected.bytes
            || symbol.address() == 0
            || symbol.address() % expected.alignment != 0
            || symbol.address().checked_add(symbol.size()).is_none()
        {
            return Err(invalid(
                "linked immutable region data identity/extent/alignment mismatch",
            ));
        }
        let section = file
            .section_by_index(
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("unplaced linked immutable region data"))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if !matches!(section.flags(), SectionFlags::Elf { sh_flags, .. } if sh_flags.0 & (7 | object::elf::SHF_TLS.0) == 2)
        {
            return Err(invalid("region data is not immutable allocated storage"));
        }
        let start = usize::try_from(
            symbol
                .address()
                .checked_sub(section.address())
                .ok_or_else(|| invalid("linked immutable region data outside section"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let end = start
            .checked_add(usize::try_from(symbol.size()).map_err(|e| invalid(e.to_string()))?)
            .ok_or_else(|| invalid("linked immutable region data extent overflow"))?;
        let data = section.data().map_err(|e| invalid(e.to_string()))?;
        let data = data
            .get(start..end)
            .ok_or_else(|| invalid("truncated linked immutable region data"))?;
        if digest(data) != expected.sha256 {
            return Err(invalid("linked immutable region data bytes changed"));
        }
        bindings.push(SystemsRegionReadOnlyBinding {
            symbol: expected.symbol.clone(),
            address: symbol.address(),
            bytes: expected.bytes,
            alignment: expected.alignment,
            sha256: expected.sha256.clone(),
        });
    }
    let mut actual = BTreeSet::new();
    for symbol in file
        .symbols()
        .filter(|s| s.is_definition() && s.kind() == SymbolKind::Data)
    {
        let name = symbol.name().map_err(|e| invalid(e.to_string()))?;
        if owned_name(name, prefix) && !actual.insert(name) {
            return Err(invalid("ambiguous immutable region data namespace"));
        }
    }
    if actual != names {
        return Err(invalid("extra or missing immutable region data binding"));
    }
    Ok(bindings)
}
