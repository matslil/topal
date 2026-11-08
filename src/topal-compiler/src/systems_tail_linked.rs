//! Relocate fresh compiler objects against exact linked input contributions.
//! Supplied manifests are not authority: callers must rebuild them from source.
use crate::{CompileError, LlvmTools};
use object::{
    Object, ObjectSection, ObjectSegment, ObjectSymbol, RelocationTarget, SectionFlags, SymbolKind,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

const PREFIX: &str = "topal.tail-inputs.";
// Retention/merging/string properties control input processing. LLD can
// propagate them into a shared output section without changing our input's
// extent or bytes. Exact endpoints, disjoint contributions and relocated bytes
// below prove preservation; actual segment permissions are checked separately.
const LINKER_PROPERTIES: u64 =
    object::elf::SHF_GNU_RETAIN.0 | object::elf::SHF_MERGE.0 | object::elf::SHF_STRINGS.0;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub marker: String,
    pub name: String,
    pub bytes: u64,
    pub alignment: u64,
    pub flags: u64,
    pub zero: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub marker: String,
    pub address: u64,
    pub bytes: u64,
    pub alignment: u64,
    pub flags: u64,
    pub sha256: String,
}
fn invalid(message: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-TAIL-LINK: {message}"))
}
fn flags(section: &object::Section<'_, '_>) -> Result<u64, CompileError> {
    match section.flags() {
        SectionFlags::Elf { sh_flags, .. } => Ok(sh_flags.0),
        _ => Err(invalid("ELF section flags required")),
    }
}
fn parse(bytes: &[u8], kind: object::ObjectKind) -> Result<object::File<'_>, CompileError> {
    let file = object::File::parse(bytes).map_err(invalid)?;
    if file.format() != object::BinaryFormat::Elf
        || !file.is_64()
        || file.kind() != kind
        || file.architecture() != object::Architecture::X86_64
        || !file.is_little_endian()
    {
        return Err(invalid("little-endian x86-64 ELF required"));
    }
    Ok(file)
}
fn run(command: &mut Command) -> Result<(), CompileError> {
    let result = command.output().map_err(invalid)?;
    if !result.status.success() {
        return Err(invalid(String::from_utf8_lossy(&result.stderr)));
    }
    Ok(())
}
fn transformed_name(section: &Section) -> String {
    // Existing root/provider section names are semantic link-map inputs.
    // Endpoint markers identify their contributions without renaming them.
    if section.marker.starts_with("topal.tail-inputs.root.")
        || section.marker.starts_with("topal.tail-inputs.provider.")
    {
        return section.name.clone();
    }
    format!(
        "{}.{}",
        if section.zero {
            ".bss"
        } else if section.flags & 4 != 0 {
            ".text"
        } else if section.flags & 1 != 0 {
            ".data"
        } else {
            ".rodata"
        },
        section.marker
    )
}

/// Attach endpoints to every allocated contribution without changing payload.
pub(crate) fn prepare(
    tools: &LlvmTools,
    path: &Path,
    role: &str,
) -> Result<Vec<Section>, CompileError> {
    if ![
        "root",
        "provider",
        "source",
        "runtime",
        "owner",
        "boundary",
        "kernel-owner",
    ]
    .contains(&role)
    {
        return Err(invalid("unqualified object role"));
    }
    let raw = fs::read(path).map_err(invalid)?;
    let file = parse(&raw, object::ObjectKind::Relocatable)?;
    if file
        .symbols()
        .any(|s| s.name().is_ok_and(|n| n.starts_with(PREFIX)))
    {
        return Err(invalid("reserved input markers already present"));
    }
    let mut records = Vec::new();
    let mut names = BTreeSet::new();
    let mut copy = Command::new(tools.tool_path("llvm-objcopy"));
    for section in file.sections() {
        let f = flags(&section)?;
        if f & object::elf::SHF_TLS.0 != 0 {
            return Err(invalid("TLS is unqualified"));
        }
        if f & 2 == 0 || section.size() == 0 {
            continue;
        }
        let name = section.name().map_err(invalid)?;
        let zero = section.kind() == object::SectionKind::UninitializedData;
        if !names.insert(name.to_owned())
            || records.len() >= 4096
            || section.size() > 64 * 1024 * 1024
            || section.align() == 0
            || !section.align().is_power_of_two()
            || f & !(7 | 16 | 32 | 0x200000) != 0
            || f & 5 == 5
            || (zero && f != 3)
            || !matches!(
                section.kind(),
                object::SectionKind::Text
                    | object::SectionKind::Data
                    | object::SectionKind::ReadOnlyData
                    | object::SectionKind::ReadOnlyString
                    | object::SectionKind::UninitializedData
            )
            || (["source", "runtime", "owner"].contains(&role) && f & 1 != 0)
        {
            return Err(invalid(format!(
                "unqualified allocated section {role}:{name}"
            )));
        }
        let record = Section {
            marker: format!("{PREFIX}{role}.{}", records.len()),
            name: name.into(),
            bytes: section.size(),
            alignment: section.align(),
            flags: f & !(16 | 32),
            zero,
        };
        if f & (16 | 32) != 0 {
            if f & 5 != 0 {
                return Err(invalid("mergeable writable/executable input"));
            }
            copy.arg(format!(
                "--set-section-flags={name}=alloc,load,readonly,data,contents"
            ));
        }
        copy.arg(format!(
            "--rename-section={name}={}",
            transformed_name(&record)
        ));
        for (suffix, offset) in [("begin", 0), ("end", record.bytes)] {
            copy.arg(format!(
                "--add-symbol={}.{suffix}={name}:{offset},global,hidden",
                record.marker
            ));
        }
        records.push(record);
    }
    if records.is_empty() {
        return Err(invalid("object has no allocated input"));
    }
    let changed = path.with_extension("marked.o");
    run(copy.arg(path).arg(&changed))?;
    let localized = path.with_extension("localized.o");
    let mut copy = Command::new(tools.tool_path("llvm-objcopy"));
    for record in &records {
        for suffix in ["begin", "end"] {
            copy.arg(format!("--localize-symbol={}.{suffix}", record.marker));
        }
    }
    run(copy.arg(&changed).arg(&localized))?;
    let transformed = fs::read(&localized).map_err(invalid)?;
    let actual = parse(&transformed, object::ObjectKind::Relocatable)?;
    let mut allocated = 0;
    for section in actual.sections() {
        if flags(&section)? & 2 != 0 && section.size() > 0 {
            allocated += 1;
        }
    }
    if allocated != records.len() {
        return Err(invalid("allocated section set changed"));
    }
    for record in &records {
        let old = file
            .section_by_name(&record.name)
            .ok_or_else(|| invalid("missing original section"))?;
        let new = actual
            .section_by_name(&transformed_name(record))
            .ok_or_else(|| invalid("missing transformed section"))?;
        if new.size() != record.bytes
            || new.align() != record.alignment
            || flags(&new)? != record.flags
            || (new.kind() == object::SectionKind::UninitializedData) != record.zero
            || old.data().map_err(invalid)? != new.data().map_err(invalid)?
        {
            return Err(invalid(
                "marker transformation changed input payload/layout",
            ));
        }
        // Rename/localization must not change actual relocation semantics.
        let describe = |f: &object::File<'_>,
                        s: &object::Section<'_, '_>|
         -> Result<Vec<String>, CompileError> {
            s.relocations()
                .map(|(offset, r)| {
                    let target = match r.target() {
                        RelocationTarget::Symbol(i) => {
                            let symbol = f.symbol_by_index(i).map_err(invalid)?;
                            let name = if symbol.kind() == SymbolKind::Section {
                                f.section_by_index(
                                    symbol
                                        .section_index()
                                        .ok_or_else(|| invalid("unplaced section symbol"))?,
                                )
                                .map_err(invalid)?
                                .name()
                                .map_err(invalid)?
                                .to_owned()
                            } else {
                                symbol.name().map_err(invalid)?.into()
                            };
                            let name = records
                                .iter()
                                .find(|v| transformed_name(v) == name)
                                .map_or(name.clone(), |v| v.name.clone());
                            format!(
                                "{name}:{}:{:?}:{}:{}",
                                symbol.address(),
                                symbol.kind(),
                                symbol.is_global(),
                                symbol.is_weak()
                            )
                        }
                        _ => return Err(invalid("unsupported input relocation target")),
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
        };
        if describe(&file, &old)? != describe(&actual, &new)? {
            return Err(invalid("marker transformation changed relocations"));
        }
    }
    fs::rename(localized, path).map_err(invalid)?;
    Ok(records)
}

fn unique<'a>(
    file: &'a object::File<'a>,
    name: &str,
    global: Option<bool>,
) -> Result<object::Symbol<'a, 'a>, CompileError> {
    let symbols = file
        .symbols()
        .filter(|s| {
            s.name().ok() == Some(name)
                && s.is_definition()
                && global.is_none_or(|global| s.is_global() == global)
        })
        .collect::<Vec<_>>();
    match symbols.as_slice() {
        [_] => Ok(symbols.into_iter().next().unwrap()),
        _ => Err(invalid(format!("missing or ambiguous symbol {name}"))),
    }
}
fn range(start: u64, bytes: u64) -> Result<u64, CompileError> {
    let end = start
        .checked_add(bytes)
        .ok_or_else(|| invalid("extent overflow"))?;
    if bytes == 0 || start < 0x200000 || end > 0x40000000 || (start < 0x800000 && end > 0x400000) {
        return Err(invalid("input outside supervisor kernel domain"));
    }
    Ok(end)
}

fn loaded(
    file: &object::File<'_>,
    start: u64,
    record: &Section,
    expected: &[u8],
) -> Result<(), CompileError> {
    let limit = start
        .checked_add(record.bytes)
        .ok_or_else(|| invalid("load extent overflow"))?;
    let segments = file
        .segments()
        .filter(|s| {
            s.address() <= start
                && s.address()
                    .checked_add(s.size())
                    .is_some_and(|end| end >= limit)
        })
        .collect::<Vec<_>>();
    let [segment] = segments.as_slice() else {
        return Err(invalid("input needs exactly one load owner"));
    };
    let want =
        4 | if record.flags & 1 != 0 { 2 } else { 0 } | if record.flags & 4 != 0 { 1 } else { 0 };
    if !matches!(segment.flags(),object::SegmentFlags::Elf {p_flags,p_type} if p_flags.0==want && p_type==object::elf::PT_LOAD)
    {
        return Err(invalid("load permissions disagree with input"));
    }
    let data = segment.data().map_err(invalid)?;
    let offset = usize::try_from(start - segment.address()).map_err(invalid)?;
    if record.zero {
        let extent = usize::try_from(record.bytes).map_err(invalid)?;
        let end = offset
            .checked_add(extent)
            .ok_or_else(|| invalid("zero-load extent overflow"))?
            .min(data.len());
        if offset < end && data[offset..end].iter().any(|b| *b != 0) {
            return Err(invalid("nonzero initialized load shadows zero storage"));
        }
    } else if data.get(
        offset
            ..offset
                .checked_add(expected.len())
                .ok_or_else(|| invalid("load extent overflow"))?,
    ) != Some(expected)
    {
        return Err(invalid("load bytes disagree with reconstructed input"));
    }
    Ok(())
}

/// Compare complete relocated sections using only fresh objects/section records.
/// The returned placements are observations to compare with candidate metadata.
pub(crate) fn inspect(
    kernel: &[u8],
    objects: &[(&[u8], &[Section])],
) -> Result<Vec<Placement>, CompileError> {
    let linked = parse(kernel, object::ObjectKind::Executable)?;
    let mut placements = Vec::new();
    let mut markers = BTreeSet::new();
    let mut occupied = Vec::new();
    for (raw, records) in objects {
        let file = parse(raw, object::ObjectKind::Relocatable)?;
        let mut bases = BTreeMap::new();
        for record in *records {
            if !markers.insert(format!("{}.begin", record.marker))
                || !markers.insert(format!("{}.end", record.marker))
            {
                return Err(invalid("duplicate section contribution"));
            }
            let begin = unique(&linked, &format!("{}.begin", record.marker), Some(false))?;
            let end = unique(&linked, &format!("{}.end", record.marker), Some(false))?;
            let start = begin.address();
            let limit = range(start, record.bytes)?;
            if end.address() != limit
                || begin.section_index() != end.section_index()
                || begin.size() != 0
                || end.size() != 0
                || start % record.alignment != 0
                || begin.kind() != SymbolKind::Unknown
                || end.kind() != SymbolKind::Unknown
                || occupied.iter().any(|(a, b)| start < *b && limit > *a)
            {
                return Err(invalid("section endpoint/ownership mismatch"));
            }
            let section = linked
                .section_by_index(
                    begin
                        .section_index()
                        .ok_or_else(|| invalid("unplaced input"))?,
                )
                .map_err(invalid)?;
            if flags(&section)? & !LINKER_PROPERTIES != record.flags & !LINKER_PROPERTIES
                || section.align() < record.alignment
                || (section.kind() == object::SectionKind::UninitializedData) != record.zero
            {
                return Err(invalid(format!(
                    "linked section permissions/layout changed for {}: flags {} expected {}, alignment {} expected {}, zero {} expected {}",
                    record.marker,
                    flags(&section)?,
                    record.flags,
                    section.align(),
                    record.alignment,
                    section.kind() == object::SectionKind::UninitializedData,
                    record.zero
                )));
            }
            let original = file
                .section_by_name(&transformed_name(record))
                .ok_or_else(|| invalid("fresh input section absent"))?;
            if original.size() != record.bytes
                || original.align() != record.alignment
                || flags(&original)? != record.flags
            {
                return Err(invalid("fresh input manifest mismatch"));
            }
            bases.insert(original.index().0, start);
            occupied.push((start, limit));
        }
        for symbol in file
            .symbols()
            .filter(|s| s.is_definition() && s.is_global())
        {
            let Some(base) = symbol.section_index().and_then(|index| bases.get(&index.0)) else {
                continue;
            };
            let expected = base
                .checked_add(symbol.address())
                .ok_or_else(|| invalid("symbol address overflow"))?;
            let actual = unique(&linked, symbol.name().map_err(invalid)?, None)?;
            if actual.address() != expected
                || actual.size() != symbol.size()
                || actual.kind() != symbol.kind()
                || actual.is_weak() != symbol.is_weak()
            {
                return Err(invalid("defined symbol changed ownership/identity"));
            }
        }
        for record in *records {
            let section = file
                .section_by_name(&transformed_name(record))
                .ok_or_else(|| invalid("fresh section missing"))?;
            let start = bases[&section.index().0];
            let mut expected = section.data().map_err(invalid)?.to_vec();
            for (offset, relocation) in section.relocations() {
                if record.zero {
                    return Err(invalid("relocated zero storage"));
                }
                let target = match relocation.target() {
                    RelocationTarget::Symbol(index) => {
                        let symbol = file.symbol_by_index(index).map_err(invalid)?;
                        if symbol.section_index().is_some() {
                            let base = symbol
                                .section_index()
                                .and_then(|index| bases.get(&index.0))
                                .ok_or_else(|| {
                                    invalid("reference to nonallocated or absolute local input")
                                })?;
                            base.checked_add(symbol.address())
                                .ok_or_else(|| invalid("local relocation overflow"))?
                        } else if symbol.is_undefined() && symbol.is_global() && !symbol.is_weak() {
                            unique(&linked, symbol.name().map_err(invalid)?, None)?.address()
                        } else {
                            return Err(invalid(format!(
                                "unqualified relocation symbol {} {:?}",
                                symbol.name().unwrap_or("?"),
                                symbol.kind()
                            )));
                        }
                    }
                    _ => return Err(invalid("unqualified linked relocation target")),
                };
                let value = i128::from(target) + i128::from(relocation.addend());
                let bytes = match (relocation.kind(), relocation.encoding(), relocation.size()) {
                    (object::RelocationKind::Absolute, object::RelocationEncoding::Generic, 64) => {
                        u64::try_from(value)
                            .map_err(invalid)?
                            .to_le_bytes()
                            .to_vec()
                    }
                    (object::RelocationKind::Absolute, object::RelocationEncoding::Generic, 32) => {
                        u32::try_from(value)
                            .map_err(invalid)?
                            .to_le_bytes()
                            .to_vec()
                    }
                    (
                        object::RelocationKind::Absolute,
                        object::RelocationEncoding::X86Signed,
                        32,
                    ) => i32::try_from(value)
                        .map_err(invalid)?
                        .to_le_bytes()
                        .to_vec(),
                    (
                        object::RelocationKind::Relative | object::RelocationKind::PltRelative,
                        object::RelocationEncoding::Generic | object::RelocationEncoding::X86Branch,
                        32,
                    ) => {
                        let place = start
                            .checked_add(offset)
                            .ok_or_else(|| invalid("relocation place overflow"))?;
                        i32::try_from(value - i128::from(place))
                            .map_err(invalid)?
                            .to_le_bytes()
                            .to_vec()
                    }
                    _ => {
                        return Err(invalid(format!(
                            "unsupported relocation {:?}/{:?}/{}",
                            relocation.kind(),
                            relocation.encoding(),
                            relocation.size()
                        )));
                    }
                };
                if relocation.has_implicit_addend() {
                    return Err(invalid("implicit relocation addend"));
                }
                let offset = usize::try_from(offset).map_err(invalid)?;
                let end = offset
                    .checked_add(bytes.len())
                    .ok_or_else(|| invalid("relocation extent overflow"))?;
                expected
                    .get_mut(offset..end)
                    .ok_or_else(|| invalid("relocation outside input"))?
                    .copy_from_slice(&bytes);
            }
            let marker = unique(&linked, &format!("{}.begin", record.marker), Some(false))?;
            let actual = linked
                .section_by_index(
                    marker
                        .section_index()
                        .ok_or_else(|| invalid("unplaced section"))?,
                )
                .map_err(invalid)?;
            let digest = if record.zero {
                if !expected.is_empty() {
                    return Err(invalid("initialized zero-storage input"));
                }
                crate::artifact::sha256(&[])
            } else {
                let offset = usize::try_from(
                    start
                        .checked_sub(actual.address())
                        .ok_or_else(|| invalid("input outside linked section"))?,
                )
                .map_err(invalid)?;
                let end = offset
                    .checked_add(expected.len())
                    .ok_or_else(|| invalid("linked input extent overflow"))?;
                let actual = actual.data().map_err(invalid)?;
                if actual.get(offset..end) != Some(expected.as_slice()) {
                    return Err(invalid(format!(
                        "relocated bytes differ for {}",
                        record.marker
                    )));
                }
                crate::artifact::sha256(&expected)
            };
            loaded(&linked, start, record, &expected)?;
            placements.push(Placement {
                marker: record.marker.clone(),
                address: start,
                bytes: record.bytes,
                alignment: record.alignment,
                flags: record.flags,
                sha256: digest,
            });
        }
    }
    let actual = linked
        .symbols()
        .filter_map(|s| s.name().ok())
        .filter(|n| n.starts_with(PREFIX))
        .collect::<BTreeSet<_>>();
    if actual != markers.iter().map(String::as_str).collect() {
        return Err(invalid("orphan or missing input marker namespace"));
    }
    Ok(placements)
}
