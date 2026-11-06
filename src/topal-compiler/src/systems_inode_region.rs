//! TK-DEC-280: actual inode loan, never a copied candidate/native mutation.
use super::CompileError;
use super::root_region::{Code, address, branch, clear_temporaries, erase, zero_readback};
use super::x86_function::{E, R, R::*};
use crate::systems_region_library::{INODE_FRAME_ABORT, INODE_FRAME_ALLOCATE, INODE_FRAME_EXPORT};
use object::write::{Object, SectionId, Symbol, SymbolSection};
use object::{SectionKind, SymbolFlags, SymbolKind, SymbolScope};

#[path = "systems_inode_region_inspection.rs"]
mod inspection;
pub use inspection::SystemsInodeRegionBinding;
pub(crate) use inspection::inspect_linked;

pub(crate) const APPLY: &str = "topal_x86_systems_inode_mode_apply_v1";
pub(crate) const FINALIZE: &str = "topal_x86_systems_inode_mode_finalize_v1";
pub(crate) const ARENA: &str = "topal_x86_systems_inode_mode_arena";
pub(crate) const CONTROL: &str = "topal_x86_systems_inode_mode_control";
pub(crate) const HISTORY: &str = "topal_x86_systems_inode_mode_history";
pub(super) const ARENA_BYTES: u64 = 65536;
pub(super) const RETIRED_STACK_BYTES: u64 = 4096;
pub(super) const LIVE_FRAME_BYTES: u64 = 8 + 14 * 8 + 64;
pub(super) const CALL_ALIGNMENT_BYTES: u64 = 64;
const DIRECTORY: i32 = super::process_runtime::volatile_files::BASE + 32;
const OBJECTS: i32 = super::process_runtime::volatile_files::OBJECTS;
const PIPES: i32 = super::process_runtime::pipes::BASE;
const SAVES: [(R, u8); 13] = [
    (B, 3),
    (R12, 12),
    (R13, 13),
    (R14, 14),
    (R15, 15),
    (Di, 5),
    (Si, 4),
    (D, 1),
    (C, 2),
    (R8, 8),
    (R9, 9),
    (R10, 10),
    (R11, 11),
];

fn record(e: &mut E<'_>, offset: i32) {
    e.mov(A, R13);
    e.add(A, offset);
    e.rr(0x39, R14, A);
    e.0.jump_if(0x82, "fail");
    e.mov(R11, R14);
    e.rr(0x29, R11, A);
    e.cmp(R11, 4 * 128);
    e.0.jump_if(0x83, "fail");
    e.arith(4, R11, 127);
    e.cmp(R11, 0);
    e.0.jump_if(0x85, "fail");
    e.load(A, R14, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
}

fn pipe_description(e: &mut E<'_>, offset: i32, kind: u64) -> Result<(), CompileError> {
    let absent = format!("pipe-description-absent-{offset}");
    let child = format!("pipe-description-child-{offset}");
    let ready = format!("pipe-description-ready-{offset}");
    e.load(R9, R14, offset);
    e.cmp(R9, 0);
    e.0.jump_if(0x84, &absent);
    e.cmp(R9, 32);
    e.0.jump_if(0x87, "fail");
    e.cmp(R9, 16);
    e.0.jump_if(0x87, &child);
    e.add(R9, -1);
    e.mov(R10, R13);
    e.add(R10, super::process_runtime::file_descriptions::DESCRIPTIONS);
    e.0.jump(&ready);
    e.0.bind(&child)?;
    e.add(R9, -17);
    e.mov(R10, R13);
    e.add(
        R10,
        super::process_runtime::file_descriptions::CHILD_DESCRIPTIONS,
    );
    e.0.bind(&ready)?;
    e.0.bytes(&[0x49, 0xc1, 0xe1, 6]); // checked description ordinal, stride 64
    e.rr(0x01, R10, R9);
    e.load(A, R10, 0);
    e.cmp(A, kind as i32);
    e.0.jump_if(0x85, "fail");
    e.load(A, R10, 8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.load(A, R10, 32);
    e.mem(0x3b, A, R14, 8);
    e.0.jump_if(0x85, "fail");
    e.0.bind(&absent)?;
    Ok(())
}

/// DI actual controller, SI owned record (or directory block), D kind 0/1/2,
/// C admitted permissions and R8 admitted clock seconds. Preserve all GPRs
/// except A. The source, not this guard, performs the metadata writes.
pub(super) fn apply() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.cmp(_Sp, 0x108000);
    e.0.jump_if(0x82, "fail");
    e.cmp(_Sp, 0x180000);
    e.0.jump_if(0x87, "fail");
    e.0.bytes(&[0x9c]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0e, 16]));
    e.0.bytes(&[0x58]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0e, 8]));
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.0.bytes(&[0x55]);
    code.unwind
        .push((e.0.bytes.len() as u32, vec![0x0e, 16, 0x86, 2]));
    e.0.bytes(&[0x48, 0x89, 0xe5]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0d, 6]));
    for (index, (register, dwarf)) in SAVES.iter().enumerate() {
        let number = *register as u8;
        if number >= 8 {
            e.0.bytes(&[0x41]);
        }
        e.0.bytes(&[0x50 | (number & 7)]);
        code.unwind
            .push((e.0.bytes.len() as u32, vec![0x80 | dwarf, index as u8 + 3]));
    }
    e.add(_Sp, -64);
    e.mov(R12, _Sp);
    e.mov(R13, Di);
    e.mov(R14, Si);
    e.imm(A, 0);
    for offset in (0..64).step_by(8) {
        e.store(R12, offset, A);
    }
    e.store(R12, 0, C);
    e.store(R12, 8, R8);
    e.store(R12, 24, D);
    e.cmp(C, 4095);
    e.0.jump_if(0x87, "fail");
    e.cmp(D, 2);
    e.0.jump_if(0x87, "fail");
    e.gs(
        0x8b,
        R11,
        super::process_runtime::file_descriptions::TABLE_CONTEXT,
    );
    e.rr(0x39, R13, R11);
    e.0.jump_if(0x85, "fail");
    address(
        &mut e,
        &mut code.references,
        R11,
        super::X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL,
    );
    e.rr(0x39, R13, R11);
    e.0.jump_if(0x85, "fail");
    e.load(A, R13, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    e.cmp(D, 1);
    e.0.jump_if(0x84, "directory");
    e.cmp(D, 2);
    e.0.jump_if(0x84, "pipe");
    record(&mut e, OBJECTS);
    e.load(A, R14, 8);
    e.cmp(A, 7);
    e.0.jump_if(0x82, "fail");
    e.load(A, R14, 56);
    e.cmp(A, 1);
    e.0.jump_if(0x87, "fail");
    e.add(R14, 24);
    e.imm(D, 0o100000);
    e.0.jump("owner-ready");
    e.0.bind("directory")?;
    e.mov(A, R13);
    e.add(A, DIRECTORY);
    e.rr(0x39, R14, A);
    e.0.jump_if(0x85, "fail");
    e.imm(D, 0o40000);
    e.0.jump("owner-ready");
    e.0.bind("pipe")?;
    record(&mut e, PIPES);
    e.load(A, R14, 8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.load(A, R14, 16);
    e.load(R11, R14, 24);
    e.rr(0x09, A, R11);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    pipe_description(&mut e, 16, 5)?;
    pipe_description(&mut e, 24, 6)?;
    e.add(R14, 64);
    e.imm(D, 0o10000);
    e.0.bind("owner-ready")?;
    e.load(A, R14, 0);
    e.arith(4, A, !0o7777);
    e.rr(0x39, A, D);
    e.0.jump_if(0x85, "fail");
    for offset in (0..32).step_by(8) {
        e.load(A, R14, offset);
        e.store(R12, 32 + offset, A);
    }
    address(&mut e, &mut code.references, B, CONTROL);
    zero_readback(&mut e, B, 64);
    address(&mut e, &mut code.references, R11, ARENA);
    zero_readback(&mut e, R11, ARENA_BYTES);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.load(A, R11, 0);
    e.mov(R15, A);
    e.add(R15, 1);
    e.0.jump_if(0x82, "fail");
    e.store(B, 16, A);
    address(&mut e, &mut code.references, A, INODE_FRAME_EXPORT);
    e.store(B, 0, A);
    e.store(B, 8, R13);
    e.store(B, 32, R14);
    e.imm(A, 32);
    e.store(B, 40, A);
    e.imm(A, 1);
    e.store(B, 48, A);
    address(&mut e, &mut code.references, R11, ARENA);
    e.store(R11, 0, B);
    e.imm(A, 32);
    e.store(R11, 8, A);
    e.store(R11, 16, R15);
    e.mov(Di, B);
    e.mov(Si, R13);
    e.mov(D, R15);
    e.mov(C, R12);
    e.imm(R8, 2);
    e.arith(4, _Sp, -16);
    e.add(_Sp, -16);
    branch(&mut e, &mut code.references, INODE_FRAME_EXPORT, true);
    e.mov(_Sp, R12);
    e.store(R12, 16, A);
    for offset in [0, 8, 24, 32, 40, 48, 56] {
        e.load(A, B, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
    }
    e.load(A, B, 16);
    e.rr(0x39, A, R15);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, R11, ARENA);
    e.load(A, R11, 0);
    e.rr(0x39, A, B);
    e.0.jump_if(0x85, "fail");
    e.load(A, R11, 16);
    e.rr(0x39, A, R15);
    e.0.jump_if(0x85, "fail");
    e.load(A, R11, 8);
    e.cmp(A, 32);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fail");
    erase(&mut e, R11, ARENA_BYTES);
    erase(&mut e, B, 64);
    e.mov(R11, R12);
    e.add(R11, -(RETIRED_STACK_BYTES as i32));
    erase(&mut e, R11, RETIRED_STACK_BYTES);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.store(R11, 0, R15);
    // Retain only admitted owner facts, then erase the entire source frame
    // before accepting the opaque result or inspecting owner bytes.
    e.load(R15, R12, 16);
    e.load(B, R12, 32);
    e.arith(4, B, !0o7777);
    e.load(C, R12, 0);
    e.rr(0x09, B, C);
    e.load(R9, R12, 40);
    e.load(R10, R12, 48);
    e.load(R8, R12, 8);
    erase(&mut e, R12, 64);
    for register in [A, C, D, Si, Di, R11] {
        e.imm(register, 0);
    }
    e.cmp(R15, 1);
    e.0.jump_if(0x85, "fail");
    for (offset, expected) in [(0, B), (8, R9), (16, R10), (24, R8)] {
        e.load(A, R14, offset);
        e.rr(0x39, A, expected);
        e.0.jump_if(0x85, "fail");
    }
    clear_temporaries(&mut e);
    e.imm(A, 1);
    e.add(_Sp, 64);
    for (register, dwarf) in SAVES.iter().rev() {
        let number = *register as u8;
        if number >= 8 {
            e.0.bytes(&[0x41]);
        }
        e.0.bytes(&[0x58 | (number & 7)]);
        code.unwind
            .push((e.0.bytes.len() as u32, vec![0x08, *dwarf]));
    }
    e.0.bytes(&[0x5d]);
    code.unwind
        .push((e.0.bytes.len() as u32, vec![0x0c, 7, 8, 0x08, 6]));
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    code.unwind.push((e.0.bytes.len() as u32, vec![0x07, 16]));
    branch(&mut e, &mut code.references, INODE_FRAME_ABORT, false);
    Ok(code)
}

pub(super) fn allocator() -> Result<Code, CompileError> {
    super::root_region::allocator_for(CONTROL, ARENA, INODE_FRAME_EXPORT, INODE_FRAME_ABORT, true)
}
pub(super) fn finalize() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, ARENA);
    zero_readback(&mut e, R11, ARENA_BYTES);
    address(&mut e, &mut code.references, R11, CONTROL);
    zero_readback(&mut e, R11, 64);
    address(&mut e, &mut code.references, R11, HISTORY);
    erase(&mut e, R11, 8);
    clear_temporaries(&mut e);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    branch(&mut e, &mut code.references, INODE_FRAME_ABORT, false);
    Ok(code)
}
pub(super) fn append(object: &mut Object<'_>, text: SectionId) -> Result<(), CompileError> {
    for (name, bytes, alignment) in [
        (ARENA, ARENA_BYTES, 4096),
        (CONTROL, 64, 8),
        (HISTORY, 8, 8),
    ] {
        let section = object.add_section(
            Vec::new(),
            format!(".bss.{name}").into_bytes(),
            SectionKind::UninitializedData,
        );
        let offset = object.append_section_bss(section, bytes, alignment);
        object.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: offset,
            size: bytes,
            kind: SymbolKind::Data,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(section),
            flags: SymbolFlags::None,
        });
    }
    super::append_encoded_function(object, text, INODE_FRAME_ABORT, &[0xfa, 0xf4, 0xeb, 0xfd]);
    let debug = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    for (name, code) in [
        (APPLY, apply()?),
        (INODE_FRAME_ALLOCATE, allocator()?),
        (FINALIZE, finalize()?),
    ] {
        super::root_region::append_code(object, text, debug, name, code)?;
    }
    Ok(())
}
