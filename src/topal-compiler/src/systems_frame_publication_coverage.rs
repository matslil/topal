//! A complete reconstruction cannot leave unowned allocated code or state.
use super::*;
use object::{ObjectSection, ObjectSegment, SectionFlags, SectionKind};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Padding {
    pub kind: String,
    pub address: u64,
    pub bytes: u64,
}

pub(super) fn validate(
    kernel: &[u8],
    placements: &[Placement],
) -> Result<Vec<Padding>, CompileError> {
    let file = object::File::parse(kernel).map_err(invalid)?;
    let mut padding = Vec::new();
    for section in file.sections() {
        let SectionFlags::Elf { sh_flags, .. } = section.flags() else {
            return Err(invalid("ELF allocation flags required"));
        };
        if sh_flags.0 & 2 == 0 || section.size() == 0 {
            continue;
        }
        let end = section
            .address()
            .checked_add(section.size())
            .ok_or_else(|| invalid("section extent overflow"))?;
        if !section.align().is_power_of_two() {
            return Err(invalid("invalid allocated section alignment"));
        }
        let mut contributions = placements
            .iter()
            .filter(|p| section.address() <= p.address && p.address < end)
            .collect::<Vec<_>>();
        contributions.sort_by_key(|p| p.address);
        if contributions.is_empty() {
            // LLD can retain a page-tail reservation after relaxing every GOT
            // entry. This is linker-owned zero padding, not a source state
            // allowance: require the sole empty GOT and its exact page tail.
            let got = file
                .sections()
                .filter(|s| s.name().ok() == Some(".got"))
                .collect::<Vec<_>>();
            if section.name().ok() == Some(".relro_padding")
                && sh_flags.0 == 3
                && section.kind() == SectionKind::UninitializedData
                && section.size() < 4096
                && end % 4096 == 0
                && got.len() == 1
                && got[0].size() == 0
                && got[0].address() == section.address()
            {
                for load in file.segments() {
                    let bytes = load.data().map_err(invalid)?;
                    let a = section.address().max(load.address());
                    let b = end.min(load.address() + bytes.len() as u64);
                    if a < b
                        && bytes[(a - load.address()) as usize..(b - load.address()) as usize]
                            .iter()
                            .any(|v| *v != 0)
                    {
                        return Err(invalid("initialized RELRO padding"));
                    }
                }
                padding.push(Padding {
                    kind: "lld-empty-got-page-tail/1".into(),
                    address: section.address(),
                    bytes: section.size(),
                });
                continue;
            }
            return Err(invalid(format!(
                "unowned allocated kernel section {}",
                section.name().unwrap_or("?")
            )));
        }
        let zero = section.kind() == SectionKind::UninitializedData;
        let data = section.data().map_err(invalid)?;
        let mut cursor = section.address();
        for part in contributions {
            let aligned = cursor
                .checked_add(part.alignment - 1)
                .map(|n| n & !(part.alignment - 1))
                .ok_or_else(|| invalid("alignment overflow"))?;
            if aligned != part.address
                || part.address.checked_add(part.bytes).is_none_or(|n| n > end)
            {
                return Err(invalid(
                    "unowned allocation between reconstructed contributions",
                ));
            }
            if zero {
                for load in file.segments() {
                    let bytes = load.data().map_err(invalid)?;
                    let a = cursor.max(load.address());
                    let b = part.address.min(load.address() + bytes.len() as u64);
                    if a < b
                        && bytes[(a - load.address()) as usize..(b - load.address()) as usize]
                            .iter()
                            .any(|v| *v != 0)
                    {
                        return Err(invalid("initialized data shadows zero-storage alignment"));
                    }
                }
            } else {
                let a = usize::try_from(cursor - section.address()).map_err(invalid)?;
                let b = usize::try_from(part.address - section.address()).map_err(invalid)?;
                let gap = data
                    .get(a..b)
                    .ok_or_else(|| invalid("alignment outside section data"))?;
                // LLD uses zero data padding and INT3 executable padding.
                // Only the minimal required alignment gap may remain unowned.
                if gap
                    .iter()
                    .any(|v| *v != 0 && !(sh_flags.0 & 4 != 0 && *v == 0xcc))
                {
                    return Err(invalid("noncanonical alignment padding"));
                }
            }
            cursor = part.address + part.bytes;
        }
        if cursor != end {
            return Err(invalid("unowned allocated section suffix"));
        }
    }
    for load in file.segments() {
        let end = load
            .address()
            .checked_add(load.size())
            .ok_or_else(|| invalid("load extent overflow"))?;
        let mut sections = file
            .sections()
            .filter(|s| {
                matches!(s.flags(),SectionFlags::Elf{sh_flags,..}
            if sh_flags.0&2!=0)
                    && s.size() > 0
                    && load.address() <= s.address()
                    && s.address() < end
            })
            .collect::<Vec<_>>();
        sections.sort_by_key(|s| s.address());
        if sections.is_empty() {
            let phoff = u64::from_le_bytes(
                kernel
                    .get(32..40)
                    .ok_or_else(|| invalid("truncated ELF header"))?
                    .try_into()
                    .unwrap(),
            );
            let eh = u16::from_le_bytes(kernel[52..54].try_into().unwrap()) as u64;
            let phsize = u16::from_le_bytes(kernel[54..56].try_into().unwrap()) as u64;
            let phnum = u16::from_le_bytes(kernel[56..58].try_into().unwrap()) as u64;
            let bytes = 64 + phsize * phnum;
            if phoff == 64
                && eh == 64
                && phsize == 56
                && bytes <= 4096
                && load.address() == 0x200000
                && load.size() == bytes
                && load.file_range() == (0, bytes)
                && matches!(load.flags(),object::SegmentFlags::Elf{p_flags,p_type}
                    if p_flags.0==4 && p_type==object::elf::PT_LOAD)
                && load.data().map_err(invalid)? == &kernel[..bytes as usize]
            {
                padding.push(Padding {
                    kind: "lld-readonly-elf-header-load/1".into(),
                    address: load.address(),
                    bytes,
                });
                continue;
            }
            return Err(invalid("unowned load without allocated input sections"));
        }
        if sections[0].address() != load.address() {
            return Err(invalid("unowned loaded prefix"));
        }
        let mut cursor = load.address();
        let data = load.data().map_err(invalid)?;
        if data.len() as u64 > load.size() {
            return Err(invalid("initialized bytes exceed actual load extent"));
        }
        for section in sections {
            let alignment = section.align();
            let start = cursor
                .checked_add(alignment - 1)
                .map(|n| n & !(alignment - 1))
                .ok_or_else(|| invalid("load alignment overflow"))?;
            if section.address() != start {
                return Err(invalid("unowned inter-section load allocation"));
            }
            let a = (cursor - load.address()).min(data.len() as u64) as usize;
            let b = (start - load.address()).min(data.len() as u64) as usize;
            if data[a..b].iter().any(|v| *v != 0) {
                return Err(invalid("nonzero inter-section load padding"));
            }
            cursor = start
                .checked_add(section.size())
                .ok_or_else(|| invalid("section overflow"))?;
        }
        if cursor != end {
            return Err(invalid("unowned loaded suffix"));
        }
    }
    Ok(padding)
}
