//! Private debugger CFI for the native bridge, including its dynamic alignment.
//! DWARF5 §6.4: .debug_frame CIE v4, DWARF32 offsets, x86-64 register numbers.
//! https://dwarfstd.org/doc/DWARF5.pdf
//! These nonallocated tables provide no runtime exception or unwind service.
use super::*;
use object::write::SymbolId;

fn finish_record(record: &mut Vec<u8>) {
    while record.len() % 8 != 0 {
        record.push(0); // DW_CFA_nop
    }
    let length = (record.len() - 4) as u32;
    record[..4].copy_from_slice(&length.to_le_bytes());
}

pub(super) fn append(
    object: &mut Object<'_>,
    section: SectionId,
    function: SymbolId,
    bytes: u64,
    rows: &[(u32, Vec<u8>)],
    discontinuous: bool,
) -> Result<(), CompileError> {
    // length, CIE_id, version, augmentation, address/segment sizes,
    // code alignment=1, data alignment=-8, return-address register=RIP.
    let mut cie = vec![0, 0, 0, 0, 255, 255, 255, 255, 4];
    // GNU 'S' marks a stack discontinuity for target debuggers. GDB otherwise
    // rejects a valid high-private-stack -> low-kernel-stack recovery as an
    // inward older frame. This does not introduce a source signal event or
    // runtime unwinding; all saved-register/CFA rules remain exact.
    if discontinuous {
        cie.push(b'S');
    }
    cie.extend_from_slice(&[0, 8, 0, 1, 0x78, 16]);
    cie.extend_from_slice(&[0x0c, 7, 8, 0x90, 1]); // CFA=RSP+8; RIP=[CFA-8]
    // The bridge preserves all SysV callee-saved registers before saving them.
    for register in [3, 6, 12, 13, 14, 15] {
        cie.extend_from_slice(&[0x08, register]); // same_value
    }
    finish_record(&mut cie);
    let cie_offset = object.append_section_data(section, &cie, 8);
    let mut fde = vec![0; 16]; // length, CIE offset, initial PC
    fde.extend_from_slice(&bytes.to_le_bytes());
    let mut previous = 0;
    for (pc, instructions) in rows {
        if *pc < previous || u64::from(*pc) >= bytes {
            return Err(CompileError::Tool("invalid root-region CFI row".into()));
        }
        fde.push(0x04); // DW_CFA_advance_loc4, code-alignment factor is one
        fde.extend_from_slice(&(pc - previous).to_le_bytes());
        fde.extend_from_slice(instructions);
        previous = *pc;
    }
    finish_record(&mut fde);
    let start = object.append_section_data(section, &fde, 8);
    let cie_symbol = object.section_symbol(section);
    // CIE_pointer is an absolute section offset, not an eh_frame displacement.
    // Relocating it is essential when the linker merges LLVM's debug tables.
    for (offset, symbol, size, addend) in [
        (start + 4, cie_symbol, 32, cie_offset as i64),
        (start + 8, function, 64, 0),
    ] {
        object
            .add_relocation(
                section,
                Relocation {
                    offset,
                    symbol,
                    addend,
                    flags: RelocationFlags::Generic {
                        kind: RelocationKind::Absolute,
                        encoding: RelocationEncoding::Generic,
                        size,
                    },
                },
            )
            .map_err(|error| CompileError::Tool(format!("root region CFI relocation: {error}")))?;
    }
    Ok(())
}
