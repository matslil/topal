//! Real filesystem map loan and checked association publication (TK-DEC-278).
//! TOPAL-SYSTEMS-REGION-PAGE-{OWNER,ALLOCATE,RELEASE}-001.
use super::root_region::{Code, address, branch, clear_temporaries, erase, zero_readback};
use super::x86_function::{E, R, R::*};
use super::{CompileError, X86FunctionEncoder};
use object::write::{Object, SectionId, Symbol, SymbolSection};
use object::{SectionKind, SymbolFlags, SymbolKind, SymbolScope};

#[path = "systems_ram_page_region_inspection.rs"]
mod inspection;
pub use inspection::SystemsRamPageRegionBinding;
pub(crate) use inspection::inspect_linked;

pub(crate) const APPLY: &str = "topal_x86_systems_ram_page_apply_v1";
pub(crate) const FINALIZE: &str = "topal_x86_systems_ram_page_finalize_v1";
pub(crate) const ARENA: &str = "topal_x86_systems_ram_page_arena";
pub(crate) const CONTROL: &str = "topal_x86_systems_ram_page_control";
pub(crate) const HISTORY: &str = "topal_x86_systems_ram_page_history";
pub(super) const ARENA_BYTES: u64 = 65536;
pub(super) const RETIRED_STACK_BYTES: u64 = 4096;
pub(super) const LIVE_FRAME_BYTES: u64 = 8 + 14 * 8 + 64;
// Both the caller alignment save and the inner alignment/call reserve fit here.
pub(super) const CALL_ALIGNMENT_BYTES: u64 = 64;
const MAP: i32 = super::process_runtime::volatile_files::BASE + 24;
const PAGES: i32 = super::process_runtime::volatile_files::PAGES;
const OBJECTS: i32 = super::process_runtime::volatile_files::OBJECTS;
const BUFFER: i32 = super::process_runtime::volatile_files::BUFFER;
const _: () = {
    assert!(ARENA_BYTES == super::file_region::ARENA_BYTES);
    assert!(RETIRED_STACK_BYTES == super::file_region::RETIRED_STACK_BYTES);
};

// Bytes are read without widening the eight-byte owner map or its before-image.
fn byte(e: &mut E<'_>, dst: R, base: R, offset: i32) {
    e.0.bytes(&[
        0x40 | (((dst as u8) >> 3) << 2) | ((base as u8) >> 3),
        0x0f,
        0xb6,
        0x80 | (((dst as u8) & 7) << 3) | ((base as u8) & 7),
    ]);
    if (base as u8) & 7 == 4 {
        e.0.bytes(&[0x24]);
    }
    e.0.bytes(&offset.to_le_bytes());
}
fn shift(e: &mut E<'_>, reg: R, left: bool, amount: u8) {
    let r = reg as u8;
    e.0.bytes(&[
        0x48 | (r >> 3),
        0xc1,
        (if left { 0xe0 } else { 0xe8 }) | (r & 7),
        amount,
    ]);
}
fn slot_pointer(e: &mut E<'_>, dst: R, slot: R, offset: i32, scale: u8) {
    e.mov(dst, slot);
    shift(e, dst, true, scale);
    e.rr(0x01, dst, R13);
    e.add(dst, offset);
}

// These save/restore ordinary caller state, outside the source retired reserve.
// DWARF uses the unchanged RBP-based CFA throughout the nested ordinary call.
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

/// DI actual controller, SI actual object, D action, C logical page / backing slot.
/// The owning service supplies this grant after recoverable admission. Preserve
/// every incoming GPR except A; return the actual source-selected backing slot.
pub(super) fn apply() -> Result<Code, CompileError> {
    use crate::systems_region_library::{PAGE_FRAME_ABORT, PAGE_FRAME_EXPORT};
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
    e.store(R12, 0, D);
    e.store(R12, 32, C);
    e.cmp(D, 1);
    e.0.jump_if(0x87, "fail");
    e.cmp(D, 0);
    e.0.jump_if(0x84, "facts-ready");
    e.cmp(C, 8);
    e.0.jump_if(0x83, "fail");
    e.store(R12, 8, C);
    e.0.bind("facts-ready")?;
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
    // Explicit comparisons establish containment before the stride check.
    e.mov(A, R13);
    e.add(A, OBJECTS);
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
    e.load(A, R14, 8);
    e.cmp(A, 7);
    e.0.jump_if(0x82, "fail");
    e.load(A, R14, 16);
    e.cmp(A, 0);
    e.0.jump_if(0x8c, "fail");
    e.load(A, R12, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "request-ready");
    e.load(A, R12, 32);
    e.imm(R11, (i64::MAX as u64) >> 12);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x87, "fail");
    e.0.bind("request-ready")?;
    e.load(A, R13, MAP);
    e.store(R12, 24, A);
    canonical_map(&mut e)?;
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
    // All context, owner, request, map, storage and generation checks precede
    // admitted release erasure. A terminal failure never resumes this owner.
    e.load(A, R12, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "payload-ready");
    e.load(R9, R12, 8);
    slot_pointer(&mut e, R11, R9, BUFFER, 12);
    erase(&mut e, R11, 4096);
    e.0.bind("payload-ready")?;
    address(&mut e, &mut code.references, A, PAGE_FRAME_EXPORT);
    e.store(B, 0, A);
    e.store(B, 8, R13);
    e.mov(A, R13);
    e.add(A, MAP);
    e.store(B, 32, A);
    e.imm(A, 8);
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
    branch(&mut e, &mut code.references, PAGE_FRAME_EXPORT, true);
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
    // Retain only the owner's scalar preparation facts in registers. Erase
    // the complete immutable input/result frame before testing the actual
    // Boolean or inspecting the opaque source delta, including false returns.
    e.load(R15, R12, 16);
    e.load(B, R12, 24);
    e.load(R9, R12, 0);
    e.load(R10, R12, 32);
    erase(&mut e, R12, 64);
    for register in [A, C, D, Si, Di, R8, R11] {
        e.imm(register, 0);
    }
    e.cmp(R15, 1);
    e.0.jump_if(0x85, "fail");
    validate_delta(&mut e)?;
    e.mov(R15, R11);
    slot_pointer(&mut e, R11, R15, PAGES, 5);
    e.cmp(R9, 0);
    e.0.jump_if(0x85, "release-association");
    zero_readback(&mut e, R11, 32);
    slot_pointer(&mut e, R8, R15, BUFFER, 12);
    zero_readback(&mut e, R8, 4096);
    e.load(A, R14, 8);
    e.store(R11, 0, A);
    e.store(R11, 8, R10);
    e.0.jump("association-ready");
    e.0.bind("release-association")?;
    erase(&mut e, R11, 32);
    e.0.bind("association-ready")?;
    clear_temporaries(&mut e);
    e.mov(A, R15);
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
    branch(&mut e, &mut code.references, PAGE_FRAME_ABORT, false);
    Ok(code)
}

fn canonical_map(e: &mut E<'_>) -> Result<(), CompileError> {
    // Count capacity only. This loop never chooses an allocation candidate.
    e.imm(R8, 0);
    for slot in 0..8 {
        let occupied = format!("occupied-{slot}");
        let next = format!("canonical-next-{slot}");
        byte(e, A, R13, MAP + slot);
        e.cmp(A, 1);
        e.0.jump_if(0x87, "fail");
        e.cmp(A, 1);
        e.0.jump_if(0x84, &occupied);
        for offset in [0, 8, 16, 24] {
            e.load(A, R13, PAGES + slot * 32 + offset);
            e.cmp(A, 0);
            e.0.jump_if(0x85, "fail");
        }
        e.mov(R11, R13);
        e.add(R11, BUFFER + slot * 4096);
        zero_readback(e, R11, 4096);
        e.add(R8, 1);
        e.0.jump(&next);
        e.0.bind(&occupied)?;
        e.load(R9, R13, PAGES + slot * 32);
        e.cmp(R9, 7);
        e.0.jump_if(0x82, "fail");
        e.load(R10, R13, PAGES + slot * 32 + 8);
        e.imm(A, (i64::MAX as u64) >> 12);
        e.rr(0x39, R10, A);
        e.0.jump_if(0x87, "fail");
        for offset in [16, 24] {
            e.load(A, R13, PAGES + slot * 32 + offset);
            e.cmp(A, 0);
            e.0.jump_if(0x85, "fail");
        }
        let owner = format!("canonical-owner-{slot}");
        for object in 0..4 {
            let advance = format!("owner-{slot}-{object}");
            e.load(A, R13, OBJECTS + object * 128);
            e.cmp(A, 1);
            e.0.jump_if(0x85, &advance);
            e.load(A, R13, OBJECTS + object * 128 + 8);
            e.rr(0x39, R9, A);
            e.0.jump_if(0x84, &owner);
            e.0.bind(&advance)?;
        }
        e.0.jump("fail");
        e.0.bind(&owner)?;
        for previous in 0..slot {
            let different = format!("unique-{slot}-{previous}");
            e.load(A, R13, PAGES + previous * 32);
            e.rr(0x39, R9, A);
            e.0.jump_if(0x85, &different);
            e.load(A, R13, PAGES + previous * 32 + 8);
            e.rr(0x39, R10, A);
            e.0.jump_if(0x84, "fail");
            e.0.bind(&different)?;
        }
        // A fresh allocation cannot duplicate a resident logical page.
        e.load(A, R12, 0);
        e.cmp(A, 0);
        e.0.jump_if(0x85, &next);
        e.load(A, R14, 8);
        e.rr(0x39, R9, A);
        e.0.jump_if(0x85, &next);
        e.load(A, R12, 32);
        e.rr(0x39, R10, A);
        e.0.jump_if(0x84, "fail");
        e.0.bind(&next)?;
    }
    e.load(A, R12, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "release-admission");
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "fail");
    e.0.jump("map-admitted");
    e.0.bind("release-admission")?;
    e.load(R9, R12, 8);
    e.mov(R11, R13);
    e.add(R11, MAP);
    e.rr(0x01, R11, R9);
    byte(e, A, R11, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    slot_pointer(e, R11, R9, PAGES, 5);
    e.load(A, R11, 0);
    e.mem(0x3b, A, R14, 8);
    e.0.jump_if(0x85, "fail");
    e.0.bind("map-admitted")?;
    Ok(())
}

fn validate_delta(e: &mut E<'_>) -> Result<(), CompileError> {
    // B before-map, R9 action, R10 admitted index; all source staging is erased.
    e.imm(R11, 8);
    for slot in 0..8 {
        let next = format!("delta-next-{slot}");
        let release = format!("delta-release-{slot}");
        let changed = format!("delta-changed-{slot}");
        byte(e, D, R13, MAP + slot);
        e.mov(R8, B);
        if slot != 0 {
            shift(e, R8, false, (slot * 8) as u8);
        }
        e.arith(4, R8, 255);
        e.rr(0x39, D, R8);
        e.0.jump_if(0x84, &next);
        e.cmp(R11, 8);
        e.0.jump_if(0x85, "fail");
        e.cmp(R9, 1);
        e.0.jump_if(0x84, &release);
        e.cmp(R8, 0);
        e.0.jump_if(0x85, "fail");
        e.cmp(D, 1);
        e.0.jump_if(0x85, "fail");
        e.0.jump(&changed);
        e.0.bind(&release)?;
        e.cmp(R10, slot);
        e.0.jump_if(0x85, "fail");
        e.cmp(R8, 1);
        e.0.jump_if(0x85, "fail");
        e.cmp(D, 0);
        e.0.jump_if(0x85, "fail");
        e.0.bind(&changed)?;
        e.imm(R11, slot as u64);
        e.0.bind(&next)?;
    }
    e.cmp(R11, 8);
    e.0.jump_if(0x83, "fail");
    Ok(())
}

pub(super) fn allocator() -> Result<Code, CompileError> {
    use crate::systems_region_library::{PAGE_FRAME_ABORT, PAGE_FRAME_EXPORT};
    super::root_region::allocator_for(CONTROL, ARENA, PAGE_FRAME_EXPORT, PAGE_FRAME_ABORT, true)
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
    branch(
        &mut e,
        &mut code.references,
        crate::systems_region_library::PAGE_FRAME_ABORT,
        false,
    );
    Ok(code)
}
pub(super) fn append(object: &mut Object<'_>, text: SectionId) -> Result<(), CompileError> {
    use crate::systems_region_library::{PAGE_FRAME_ABORT, PAGE_FRAME_ALLOCATE};
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
    super::append_encoded_function(object, text, PAGE_FRAME_ABORT, &[0xfa, 0xf4, 0xeb, 0xfd]);
    let debug = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    for (name, code) in [
        (APPLY, apply()?),
        (PAGE_FRAME_ALLOCATE, allocator()?),
        (FINALIZE, finalize()?),
    ] {
        super::root_region::append_code(object, text, debug, name, code)?;
    }
    Ok(())
}

/// Service call sites are closed, serialized and nonrecursive. Preserve their
/// volatile registers as well as ordinary callee state across the source call.
pub(super) fn emit_action(code: &mut X86FunctionEncoder, object: R, action: u64, argument: R) {
    // Preserve the service's arguments while installing the native grant. The
    // controller preserves this outer frame's registers; retire it on return.
    let mut e = E(code);
    for register in [Di, Si, D, C] {
        e.0.bytes(&[0x50 | (register as u8)]);
    }
    e.mov(C, argument);
    e.mov(Si, object);
    e.mov(Di, R12);
    e.imm(D, action);
    e.0.external_call(APPLY);
    for register in [C, D, Si, Di] {
        e.0.bytes(&[0x58 | (register as u8)]);
    }
}
