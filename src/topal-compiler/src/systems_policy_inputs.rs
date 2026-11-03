//! Complete compiler-owned policy sections, including relocated anonymous data.
use crate::{CompileError, LlvmTools};
use object::{Object, ObjectSection, ObjectSymbol, RelocationTarget, SectionFlags, SymbolKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

const PREFIX: &str = "topal.policy-inputs.";
const SCHEMA: &str = "topal.policy-linked-inputs.x86_64/1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsPolicySection {
    pub marker: String,
    pub source_section: String,
    pub bytes: u64,
    pub alignment: u64,
    pub executable: bool,
    pub sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsPolicyLinkedSection {
    pub marker: String,
    pub address: u64,
    pub bytes: u64,
    pub alignment: u64,
    pub executable: bool,
    pub sha256: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsPolicyLinkedInputs {
    pub schema: String,
    pub object_sha256: String,
    pub manifest_sha256: String,
    pub sections: Vec<SystemsPolicyLinkedSection>,
}
fn invalid(message: impl Into<String>) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-POLICY-INPUTS: {}", message.into()))
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn flags(section: &object::Section<'_, '_>) -> Result<u64, CompileError> {
    match section.flags() {
        SectionFlags::Elf { sh_flags, .. } => Ok(sh_flags.0),
        _ => Err(invalid("policy section requires ELF flags")),
    }
}
fn section_name(record: &SystemsPolicySection) -> String {
    format!(
        "{}.{}",
        if record.executable {
            ".text"
        } else {
            ".rodata"
        },
        record.marker
    )
}
fn relocations(
    file: &object::File<'_>,
    index: object::SectionIndex,
    original_names: &BTreeMap<String, String>,
) -> Result<Vec<String>, CompileError> {
    let section = file
        .section_by_index(index)
        .map_err(|e| invalid(e.to_string()))?;
    let original = |name: &str| {
        original_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.into())
    };
    section
        .relocations()
        .map(|(offset, r)| {
            let target = match r.target() {
                RelocationTarget::Symbol(index) => {
                    let symbol = file
                        .symbol_by_index(index)
                        .map_err(|e| invalid(e.to_string()))?;
                    let name = if symbol.kind() == SymbolKind::Section {
                        let index = symbol
                            .section_index()
                            .ok_or_else(|| invalid("unplaced relocation target"))?;
                        original(
                            file.section_by_index(index)
                                .map_err(|e| invalid(e.to_string()))?
                                .name()
                                .map_err(|e| invalid(e.to_string()))?,
                        )
                    } else {
                        symbol.name().map_err(|e| invalid(e.to_string()))?.into()
                    };
                    format!(
                        "symbol:{name}:{}:{:?}:{}",
                        symbol.address(),
                        symbol.kind(),
                        symbol.is_global()
                    )
                }
                RelocationTarget::Section(index) => original(
                    file.section_by_index(index)
                        .map_err(|e| invalid(e.to_string()))?
                        .name()
                        .map_err(|e| invalid(e.to_string()))?,
                ),
                RelocationTarget::Absolute => "absolute".into(),
                _ => return Err(invalid("unsupported policy relocation target")),
            };
            Ok(format!(
                "{offset}:{:?}:{:?}:{}:{}:{:?}:{target}",
                r.kind(),
                r.encoding(),
                r.size(),
                r.addend(),
                r.flags()
            ))
        })
        .collect()
}

pub(crate) fn prepare(
    tools: &LlvmTools,
    path: &Path,
    slot: &str,
) -> Result<Vec<SystemsPolicySection>, CompileError> {
    let bytes = fs::read(path).map_err(|e| invalid(e.to_string()))?;
    let file = object::File::parse(bytes.as_slice()).map_err(|e| invalid(e.to_string()))?;
    if file.format() != object::BinaryFormat::Elf
        || file.architecture() != object::Architecture::X86_64
        || file.kind() != object::ObjectKind::Relocatable
        || !file.is_little_endian()
    {
        return Err(invalid(
            "policy inputs require a little-endian x86-64 object",
        ));
    }
    if file
        .symbols()
        .any(|s| s.name().is_ok_and(|n| n.starts_with(PREFIX)))
    {
        return Err(invalid("reserved policy marker namespace already present"));
    }
    let mut records = Vec::new();
    let mut originals = BTreeSet::new();
    let mut command = Command::new(tools.tool_path("llvm-objcopy"));
    for section in file.sections() {
        let f = flags(&section)?;
        if f & object::elf::SHF_TLS.0 != 0 {
            return Err(invalid("TLS policy inputs are forbidden"));
        }
        if f & 2 == 0 || section.size() == 0 {
            continue;
        }
        let executable = f & 4 != 0;
        if f & 1 != 0
            || f & !(2 | 4 | 16 | 32 | 0x200000) != 0
            || (executable && f & (16 | 32) != 0)
            || !matches!(
                (executable, section.kind()),
                (true, object::SectionKind::Text)
                    | (
                        false,
                        object::SectionKind::ReadOnlyData | object::SectionKind::ReadOnlyString
                    )
            )
        {
            return Err(invalid("unsupported allocated policy section"));
        }
        let name = section.name().map_err(|e| invalid(e.to_string()))?;
        if !originals.insert(name.to_owned())
            || section.align() == 0
            || !section.align().is_power_of_two()
        {
            return Err(invalid("ambiguous policy section or alignment"));
        }
        let data = section.data().map_err(|e| invalid(e.to_string()))?;
        if data.len() as u64 != section.size() {
            return Err(invalid("truncated policy object section"));
        }
        let record = SystemsPolicySection {
            marker: format!("{PREFIX}{slot}.{}", records.len()),
            source_section: name.into(),
            bytes: section.size(),
            alignment: section.align(),
            executable,
            sha256: digest(data),
        };
        if !executable {
            command.arg(format!(
                "--set-section-flags={name}=alloc,load,readonly,data,contents"
            ));
        }
        command
            .arg(format!("--rename-section={name}={}", section_name(&record)))
            .arg(format!(
                "--add-symbol={}.begin={name}:0,global,hidden",
                record.marker
            ))
            .arg(format!(
                "--add-symbol={}.end={name}:{},global,hidden",
                record.marker, record.bytes
            ));
        records.push(record);
    }
    if !records.iter().any(|r| r.executable) {
        return Err(invalid("policy object has no code"));
    }
    let transformed = path.with_extension("qualified.o");
    let output = command
        .arg(path)
        .arg(&transformed)
        .output()
        .map_err(|e| invalid(e.to_string()))?;
    if !output.status.success() {
        return Err(invalid(String::from_utf8_lossy(&output.stderr)));
    }
    // LLVM 22 appends added local symbols after existing globals without a
    // valid local/global partition. Add hidden globals, then localize existing
    // symbols in a separate pass, which rebuilds the correct ELF symbol order.
    let localized = path.with_extension("localized.o");
    let mut command = Command::new(tools.tool_path("llvm-objcopy"));
    for record in &records {
        for suffix in ["begin", "end"] {
            command.arg(format!("--localize-symbol={}.{suffix}", record.marker));
        }
    }
    let output = command
        .arg(&transformed)
        .arg(&localized)
        .output()
        .map_err(|e| invalid(e.to_string()))?;
    if !output.status.success() {
        return Err(invalid(String::from_utf8_lossy(&output.stderr)));
    }
    fs::rename(&localized, &transformed).map_err(|e| invalid(e.to_string()))?;
    let changed = fs::read(&transformed).map_err(|e| invalid(e.to_string()))?;
    let actual = object::File::parse(changed.as_slice()).map_err(|e| invalid(e.to_string()))?;
    let names: BTreeMap<_, _> = records
        .iter()
        .map(|r| (section_name(r), r.source_section.clone()))
        .collect();
    let allocated = actual
        .sections()
        .filter(|s| s.size() > 0 && flags(s).is_ok_and(|f| f & 2 != 0))
        .count();
    if allocated != records.len() {
        return Err(invalid("transformed policy section set changed"));
    }
    for record in &records {
        let original = file
            .section_by_name(&record.source_section)
            .ok_or_else(|| invalid("original section missing"))?;
        let section = actual
            .section_by_name(&section_name(record))
            .ok_or_else(|| invalid("transformed section missing"))?;
        if section.size() != record.bytes
            || section.align() != record.alignment
            || flags(&section)? & (7 | object::elf::SHF_TLS.0 | 16 | 32)
                != if record.executable { 6 } else { 2 }
            || digest(section.data().map_err(|e| invalid(e.to_string()))?) != record.sha256
            || relocations(&file, original.index(), &BTreeMap::new())?
                != relocations(&actual, section.index(), &names)?
        {
            return Err(invalid(
                "policy transformation changed payload, placement or relocations",
            ));
        }
        for (suffix, address) in [("begin", 0), ("end", record.bytes)] {
            let name = format!("{}.{suffix}", record.marker);
            let found = actual
                .symbols()
                .filter(|s| s.name().ok() == Some(name.as_str()))
                .collect::<Vec<_>>();
            let [symbol] = found.as_slice() else {
                return Err(invalid("ambiguous policy object marker"));
            };
            if !symbol.is_definition()
                || symbol.is_global()
                || symbol.kind() != SymbolKind::Unknown
                || symbol.size() != 0
                || symbol.section_index() != Some(section.index())
                || symbol.address() != address
            {
                return Err(invalid("incorrect policy object marker endpoint"));
            }
        }
    }
    fs::rename(&transformed, path).map_err(|e| invalid(e.to_string()))?;
    Ok(records)
}

pub(crate) fn inspect(
    bytes: &[u8],
    slot: &str,
    object_sha256: &str,
    records: &[SystemsPolicySection],
) -> Result<SystemsPolicyLinkedInputs, CompileError> {
    let file = object::File::parse(bytes).map_err(|e| invalid(e.to_string()))?;
    if file.format() != object::BinaryFormat::Elf
        || file.architecture() != object::Architecture::X86_64
        || file.kind() != object::ObjectKind::Executable
        || !file.is_little_endian()
        || records.is_empty()
        || !records.iter().any(|r| r.executable)
        || object_sha256.len() != 64
        || !object_sha256.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(invalid("incomplete linked policy object identity"));
    }
    let prefix = format!("{PREFIX}{slot}.");
    let mut expected = BTreeSet::new();
    let mut sections = Vec::new();
    let mut extents = Vec::new();
    for (index, record) in records.iter().enumerate() {
        if record.marker != format!("{prefix}{index}")
            || record.bytes == 0
            || record.alignment == 0
            || !record.alignment.is_power_of_two()
            || record.source_section.is_empty()
            || record.sha256.len() != 64
            || !record.sha256.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err(invalid("invalid or unordered policy section manifest"));
        }
        let mut endpoints = Vec::new();
        for suffix in ["begin", "end"] {
            let name = format!("{}.{suffix}", record.marker);
            expected.insert(name.clone());
            let found = file
                .symbols()
                .filter(|s| s.name().ok() == Some(name.as_str()))
                .collect::<Vec<_>>();
            let [symbol] = found.as_slice() else {
                return Err(invalid("missing or ambiguous linked policy marker"));
            };
            if !symbol.is_definition()
                || symbol.is_global()
                || symbol.kind() != SymbolKind::Unknown
                || symbol.size() != 0
            {
                return Err(invalid("invalid linked policy marker identity"));
            }
            endpoints.push((
                symbol.address(),
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("unplaced policy marker"))?,
            ));
        }
        let (start, owner) = endpoints[0];
        let (end, end_owner) = endpoints[1];
        if owner != end_owner
            || end.checked_sub(start) != Some(record.bytes)
            || start % record.alignment != 0
            || start < 0x200000
            || end > 0x40000000
            || (start < 0x800000 && end > 0x400000)
            || extents.iter().any(|(a, b)| start < *b && end > *a)
        {
            return Err(invalid(
                "linked policy section extent/alignment/ownership mismatch",
            ));
        }
        let section = file
            .section_by_index(owner)
            .map_err(|e| invalid(e.to_string()))?;
        if flags(&section)? & (7 | object::elf::SHF_TLS.0) != if record.executable { 6 } else { 2 }
            || section.align() < record.alignment
        {
            return Err(invalid("linked policy section role or alignment changed"));
        }
        let offset = usize::try_from(
            start
                .checked_sub(section.address())
                .ok_or_else(|| invalid("policy section outside storage"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let count = usize::try_from(record.bytes).map_err(|e| invalid(e.to_string()))?;
        let end_offset = offset
            .checked_add(count)
            .ok_or_else(|| invalid("policy section range overflow"))?;
        let payload = section.data().map_err(|e| invalid(e.to_string()))?;
        let payload = payload
            .get(offset..end_offset)
            .ok_or_else(|| invalid("truncated linked policy section"))?;
        sections.push(SystemsPolicyLinkedSection {
            marker: record.marker.clone(),
            address: start,
            bytes: record.bytes,
            alignment: record.alignment,
            executable: record.executable,
            sha256: digest(payload),
        });
        extents.push((start, end));
    }
    let mut actual = BTreeSet::new();
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|e| invalid(e.to_string()))?;
        if name.starts_with(&prefix) && (!symbol.is_definition() || !actual.insert(name.to_owned()))
        {
            return Err(invalid("ambiguous policy marker namespace"));
        }
    }
    if actual != expected {
        return Err(invalid("extra or missing policy marker binding"));
    }
    Ok(SystemsPolicyLinkedInputs {
        schema: SCHEMA.into(),
        object_sha256: object_sha256.into(),
        manifest_sha256: digest(&serde_json::to_vec(records).map_err(|e| invalid(e.to_string()))?),
        sections,
    })
}

pub(crate) fn validate(
    bytes: &[u8],
    slot: &str,
    object_sha256: &str,
    records: &[SystemsPolicySection],
    binding: Option<&SystemsPolicyLinkedInputs>,
) -> Result<(), CompileError> {
    let actual = inspect(bytes, slot, object_sha256, records)?;
    if binding != Some(&actual) {
        return Err(invalid(
            "linked policy section bytes or complete binding changed",
        ));
    }
    Ok(())
}

pub(crate) fn validate_slots(bytes: &[u8], slots: &[&str]) -> Result<(), CompileError> {
    let file = object::File::parse(bytes).map_err(|e| invalid(e.to_string()))?;
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|e| invalid(e.to_string()))?;
        if name.starts_with(PREFIX)
            && !slots
                .iter()
                .any(|slot| name.starts_with(&format!("{PREFIX}{slot}.")))
        {
            return Err(invalid("policy markers exist without a selected slot"));
        }
    }
    Ok(())
}

pub(crate) fn validate_object_output(
    outputs: &[crate::DigestEntry],
    slot: &str,
    digest: &str,
) -> Result<(), CompileError> {
    let name = format!("{slot}.o");
    let found = outputs
        .iter()
        .filter(|o| o.identity == name)
        .collect::<Vec<_>>();
    let [object] = found.as_slice() else {
        return Err(invalid("missing or duplicate policy object output"));
    };
    if object.sha256 != digest {
        return Err(invalid(
            "policy object digest disagrees with published output",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_object_inputs_allow_empty_data_and_refuse_tls_writable_and_reserved_inputs() {
        // TOPAL-SYSTEMS-POLICY-LINKED-INPUTS-001: actual object transformation.
        let tools = LlvmTools::discover(None).unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "topal-policy-object-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        for (name, global, diagnostic) in [
            ("empty", "", None),
            (
                "writable",
                "@state = global i64 1\n",
                Some("unsupported allocated policy section"),
            ),
            (
                "tls",
                "@state = thread_local global i64 1\n",
                Some("TLS policy inputs are forbidden"),
            ),
        ] {
            let source = root.join(format!("{name}.ll"));
            let object = root.join(format!("{name}.o"));
            fs::write(
                &source,
                format!("{global}define i64 @example() nounwind {{ ret i64 0 }}\n"),
            )
            .unwrap();
            let output = Command::new(tools.tool_path("llc"))
                .args(["-filetype=obj", "-mtriple=x86_64-unknown-none"])
                .arg(&source)
                .arg("-o")
                .arg(&object)
                .output()
                .unwrap();
            assert!(output.status.success());
            let result = prepare(&tools, &object, "kernel-library");
            if let Some(diagnostic) = diagnostic {
                let error = result.unwrap_err();
                assert!(error.to_string().contains(diagnostic), "{error}");
            } else {
                let records = result.unwrap();
                assert!(!records.is_empty());
                assert!(records.iter().all(|s| s.executable));
                let error = prepare(&tools, &object, "kernel-library").unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("reserved policy marker namespace already present")
                );
            }
        }
    }
}
