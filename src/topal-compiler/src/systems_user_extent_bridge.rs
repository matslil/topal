//! Private C-ABI bridge for the unpublished full-root experiment.
use super::root_region::{Code, address, append_code, branch};
use super::x86_function::{E, R::*};
use super::*;

// Owning helper return + RBP/five volatile saves/alignment + the bridge's
// return/alignment/gateway-return words. The inner gateway window is derived
// after reserving these bytes within the unchanged 4096-byte caller bound.
pub(crate) const CALLER_OUTER_BYTES: u64 = 8 + 8 + 40 + 8 + 24;
pub(crate) const CALLER_RETIRED_BYTES: u64 = (4096 - 48 - CALLER_OUTER_BYTES) & !15;

pub(crate) fn caller_object() -> Result<Vec<u8>, CompileError> {
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let section = object.add_section(
        Vec::new(),
        b".text.topal.user.extent.callers".to_vec(),
        SectionKind::Text,
    );
    let frame = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    object.add_section(Vec::new(), b".note.GNU-stack".to_vec(), SectionKind::Other);
    for (name, return_name, direction) in [
        (
            "topal_x86_user_extent_readable_v1",
            "topal_x86_user_extent_readable_return_v1",
            0,
        ),
        (
            "topal_x86_user_extent_writable_v1",
            "topal_x86_user_extent_writable_return_v1",
            1,
        ),
    ] {
        let mut code = Code::default();
        code.encoder.bytes(&[0x55]); // saved RBP
        code.unwind
            .push((code.encoder.bytes.len() as u32, vec![0x0e, 16, 0x86, 2]));
        E(&mut code.encoder).mov(_Bp, _Sp);
        code.unwind
            .push((code.encoder.bytes.len() as u32, vec![0x0c, 6, 16]));
        // Preserve exactly the additional volatile registers promised by the
        // existing internal helper ABI. RBP supplies an alignment-independent
        // CFA and the position of their saved words.
        code.encoder
            .bytes(&[0x52, 0x56, 0x57, 0x41, 0x50, 0x41, 0x51]);
        let mut e = E(&mut code.encoder);
        e.arith(4, _Sp, -16);
        e.mov(Di, R8);
        e.mov(Si, R9);
        e.imm(D, direction);
        branch(
            &mut e,
            &mut code.references,
            "topal_x86_user_extent_call_v1",
            true,
        );
        let bridge_return = code.encoder.bytes.len() as u64;
        let mut e = E(&mut code.encoder);
        e.imm(C, 0);
        e.store(_Sp, -8, C); // completed bridge return word
        e.store(_Bp, -48, C); // optional alignment word, or the same return word
        e.mov(_Sp, _Bp);
        e.add(_Sp, -40);
        for pop in [
            &[0x41, 0x59][..],
            &[0x41, 0x58][..],
            &[0x5f][..],
            &[0x5e][..],
            &[0x5a][..],
        ] {
            e.0.bytes(pop);
            e.store(_Sp, -8, C);
        }
        e.0.bytes(&[0x5d]);
        code.unwind
            .push((code.encoder.bytes.len() as u32, vec![0x0c, 7, 8, 0x08, 6]));
        let mut e = E(&mut code.encoder);
        e.store(_Sp, -8, C);
        e.cmp(A, 1);
        e.0.bytes(&[0xf5]); // CMC: CF=0 exactly for native status zero
        let point = code.encoder.bytes.len() as u64;
        code.encoder.bytes(&[0xc3]);
        append_code(&mut object, section, frame, name, code)?;
        let start = object
            .symbol(object.symbol_id(name.as_bytes()).unwrap())
            .value;
        object.add_symbol(Symbol {
            name: return_name.as_bytes().to_vec(),
            value: start + point,
            size: 0,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(section),
            flags: SymbolFlags::None,
        });
        object.add_symbol(Symbol {
            name: format!("{}_bridge_return_v1", name.trim_end_matches("_v1")).into_bytes(),
            value: start + bridge_return,
            size: 0,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(section),
            flags: SymbolFlags::None,
        });
    }
    append_caller_abort(&mut object, section, frame)?;
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}

fn append_caller_abort(
    object: &mut Object<'_>,
    section: object::write::SectionId,
    frame: object::write::SectionId,
) -> Result<(), CompileError> {
    let mut code = Code::default();
    code.unwind.push((0, vec![0x07, 16])); // nonreturning caller cleanup
    let mut e = E(&mut code.encoder);
    // The owning gateway has restored its exact entry SP and callee registers.
    // Verify the emitted helper layout again before following its saved RBP.
    e.mov(R11, _Sp);
    e.mov(R10, _Bp);
    e.rr(0x29, R10, _Sp);
    e.cmp(R10, 64);
    e.0.jump_if(0x84, "caller_shape");
    e.cmp(R10, 72);
    e.0.jump_if(0x85, "caller_halt");
    e.0.bind("caller_shape")?;
    e.mov(R10, _Bp);
    e.add(R10, 16);
    e.cmp(R10, 0x180000);
    e.0.jump_if(0x87, "caller_halt");
    e.cmp(R11, 0x109040);
    e.0.jump_if(0x82, "caller_halt");
    e.load(A, R11, 16);
    address(
        &mut e,
        &mut code.references,
        R10,
        "topal_x86_user_extent_readable_bridge_return_v1",
    );
    e.rr(0x39, A, R10);
    e.0.jump_if(0x84, "caller_site");
    address(
        &mut e,
        &mut code.references,
        R10,
        "topal_x86_user_extent_writable_bridge_return_v1",
    );
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "caller_halt");
    e.0.bind("caller_site")?;
    address(&mut e, &mut code.references, R10, "topal_bootstrap_storage");
    e.load(A, R10, 520);
    e.mov(R10, _Bp);
    e.add(R10, 16);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x82, "caller_halt");
    e.load(R10, _Bp, 0);
    e.mov(_Sp, _Bp);
    e.add(_Sp, 16);
    e.mov(_Bp, R10);
    // Retire only the 80/88 bytes belonging to this completed adapter; its
    // inner gateway already retired the derived 3952+48-byte window.
    for opcode in [0xab, 0xaf] {
        e.mov(Di, R11);
        e.mov(C, _Sp);
        e.rr(0x29, C, R11);
        e.0.bytes(&[0x48, 0xc1, 0xe9, 3]);
        e.imm(A, 0);
        e.0.bytes(&[0xfc, 0xf3, 0x48, opcode]);
        if opcode == 0xaf {
            e.0.jump_if(0x85, "caller_halt");
        }
    }
    for reg in [A, C, D, Si, Di, R8, R9, R10, R11] {
        e.imm(reg, 0);
    }
    let reclaimed = e.0.bytes.len() as u64;
    e.0.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    e.0.bind("caller_halt")?;
    e.0.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    append_code(
        object,
        section,
        frame,
        "topal_x86_user_extent_caller_abort_v1",
        code,
    )?;
    let start = object
        .symbol(
            object
                .symbol_id(b"topal_x86_user_extent_caller_abort_v1")
                .unwrap(),
        )
        .value;
    object.add_symbol(Symbol {
        name: b"topal_x86_user_extent_caller_reclaimed_v1".to_vec(),
        value: start + reclaimed,
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    Ok(())
}

pub(crate) fn object() -> Result<Vec<u8>, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.mov(C, D);
    e.mov(D, Si);
    e.mov(Si, Di);
    e.imm(Di, 8);
    e.imm(R8, 0);
    e.0.bytes(&[0x6a, 0]);
    code.unwind
        .push((code.encoder.bytes.len() as u32, vec![0x0e, 16]));
    branch(
        &mut E(&mut code.encoder),
        &mut code.references,
        "topal_x86_frame_gateway_v1",
        true,
    );
    let point = code.encoder.bytes.len() as u64;
    let mut e = E(&mut code.encoder);
    e.imm(C, 0);
    e.store(_Sp, -8, C);
    e.store(_Sp, 0, C);
    e.add(_Sp, 8);
    code.unwind
        .push((code.encoder.bytes.len() as u32, vec![0x0e, 8]));
    code.encoder.bytes(&[0xc3]);
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let section = object.add_section(
        Vec::new(),
        b".text.topal.user.extent.bridge".to_vec(),
        SectionKind::Text,
    );
    let frame = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    object.add_section(Vec::new(), b".note.GNU-stack".to_vec(), SectionKind::Other);
    append_code(
        &mut object,
        section,
        frame,
        "topal_x86_user_extent_call_v1",
        code,
    )?;
    let start = object
        .symbol(object.symbol_id(b"topal_x86_user_extent_call_v1").unwrap())
        .value;
    object.add_symbol(Symbol {
        name: b"topal_x86_user_extent_return_v1".to_vec(),
        value: start + point,
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}
