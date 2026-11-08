//! Experimental target-owned stack/recovery boundary, not a publication grant.
use super::CompileError;
use super::root_region::{Code, address, append_code, append_discontinuous_code, branch};
use super::x86_function::{E, R, R::*};
use object::write::{Object, Symbol, SymbolSection};
use object::{
    Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};

pub(crate) const CALL: &str = "topal_x86_tail_boundary_call_v1";
pub(crate) const ABORT: &str = "topal_x86_tail_boundary_abort_v1";
pub(crate) const FAIL: &str = "topal_x86_tail_boundary_fail_v1";
pub(crate) const ACTIVE: &str = "topal_x86_tail_boundary_active_v1";
const ENTRY: &str = "topal_tail_step_component_v1";
const FATAL: &str = "topal_tail_boundary_fatal_v1";
pub(crate) const PRIVATE_BYTES: u64 = 65528;
pub(crate) const HEADER_BYTES: u64 = 64;
pub(crate) const STACK_BYTES: u64 = 4096;
#[path = "systems_tail_checkpoint_boundary.rs"]
mod checkpoint;
#[path = "systems_tail_completion_boundary.rs"]
mod completion;
#[path = "systems_tail_kernel.rs"]
pub(crate) mod kernel;

fn row(code: &mut Code, instructions: Vec<u8>) {
    code.unwind
        .push((code.encoder.bytes.len() as u32, instructions));
}
fn saved_rows(code: &mut Code, register: u8) {
    let mut instructions = vec![0x0c, register, 56];
    for (reg, offset) in [(6, 2), (3, 3), (12, 4), (13, 5), (14, 6), (15, 7)] {
        instructions.extend([0x80 | reg, offset]);
    }
    row(code, instructions);
}
fn push_caller(code: &mut Code) {
    for (index, (bytes, reg)) in [
        (&[0x55][..], 6),
        (&[0x53], 3),
        (&[0x41, 0x54], 12),
        (&[0x41, 0x55], 13),
        (&[0x41, 0x56], 14),
        (&[0x41, 0x57], 15),
    ]
    .into_iter()
    .enumerate()
    {
        code.encoder.bytes(bytes);
        row(
            code,
            vec![0x0e, (16 + 8 * index) as u8, 0x80 | reg, (2 + index) as u8],
        );
    }
}
fn pop_caller(code: &mut Code) {
    for (index, (bytes, reg)) in [
        (&[0x41, 0x5f][..], 15),
        (&[0x41, 0x5e], 14),
        (&[0x41, 0x5d], 13),
        (&[0x41, 0x5c], 12),
        (&[0x5b], 3),
        (&[0x5d], 6),
    ]
    .into_iter()
    .enumerate()
    {
        code.encoder.bytes(bytes);
        row(code, vec![0x0e, (48 - 8 * index) as u8, 0x08, reg]);
    }
}
fn clear_volatile(e: &mut E<'_>) {
    for reg in [A, C, D, Si, Di, R8, R9, R10, R11] {
        e.imm(reg, 0);
    }
}
fn erase(e: &mut E<'_>, base: R, bytes: u64, failure: &str) {
    e.mov(Di, base);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    e.mov(Di, base);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, failure);
}
fn clear_active(code: &mut Code) {
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, ACTIVE);
    e.imm(A, 0);
    e.store(R11, 0, A);
}
enum HistoryPin {
    Absent,
    Header,
    Retained(R),
}
fn fatal_cleanup(
    code: &mut Code,
    marker: &mut u64,
    history: HistoryPin,
) -> Result<(), CompileError> {
    fatal_cleanup_named(code, marker, history, "")
}
fn fatal_cleanup_named(
    code: &mut Code,
    marker: &mut u64,
    history: HistoryPin,
    prefix: &str,
) -> Result<(), CompileError> {
    E(&mut code.encoder).mov(_Sp, R12);
    saved_rows(code, 7);
    if !matches!(history, HistoryPin::Absent) {
        let mut e = E(&mut code.encoder);
        match history {
            HistoryPin::Header => e.load(R11, R14, 40),
            HistoryPin::Retained(register) => e.mov(R11, register),
            HistoryPin::Absent => unreachable!(),
        }
        e.imm(A, 0);
        e.store(R11, 24, A);
        e.imm(A, 1);
        e.store(R11, 32, A);
    }
    {
        let private = format!("{prefix}retire_private");
        let end = format!("{prefix}hard_fail");
        let mut e = E(&mut code.encoder);
        erase(&mut e, R13, STACK_BYTES, &private);
        e.0.bind(&private)?;
        erase(&mut e, R14, PRIVATE_BYTES, &end);
        e.0.bind(&end)?;
    }
    clear_active(code);
    clear_volatile(&mut E(&mut code.encoder));
    pop_caller(code);
    *marker = code.encoder.bytes.len() as u64;
    E(&mut code.encoder).add(_Sp, -8);
    row(code, vec![0x0e, 16]);
    let mut e = E(&mut code.encoder);
    branch(&mut e, &mut code.references, FATAL, true);
    e.0.bytes(&[0x0f, 0x0b]); // A wrongly returning fatal implementation cannot resume the owner.
    Ok(())
}

fn admission(code: &mut Code, refusal: &str, clean_header: bool) -> Result<(), CompileError> {
    {
        let mut e = E(&mut code.encoder);
        e.mov(R8, Di);
        e.mov(R9, Si);
        address(&mut e, &mut code.references, R11, ACTIVE);
        e.load(A, R11, 0);
        e.cmp(A, 0);
        e.0.jump_if(0x85, refusal);
        for base in [R8, R9] {
            e.cmp(base, 0);
            e.0.jump_if(0x84, refusal);
            e.mov(A, base);
            e.arith(4, A, 15);
            e.cmp(A, 0);
            e.0.jump_if(0x85, refusal);
        }
        e.mov(R10, R8);
        e.add(R10, PRIVATE_BYTES as i32);
        e.rr(0x39, R10, R8);
        e.0.jump_if(0x82, refusal);
        e.mov(D, R9);
        e.add(D, STACK_BYTES as i32);
        e.rr(0x39, D, R9);
        e.0.jump_if(0x82, refusal);
        e.rr(0x39, R10, R9);
        e.0.jump_if(0x86, "geometry");
        e.rr(0x39, D, R8);
        e.0.jump_if(0x87, refusal);
        e.0.bind("geometry")?;
        if clean_header {
            e.mov(Di, R8);
            e.imm(A, 0);
            e.imm(C, HEADER_BYTES / 8);
            e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
            e.0.jump_if(0x85, refusal);
        }
    }
    Ok(())
}

fn call(history_offset: Option<u64>) -> Result<(Code, u64, u64), CompileError> {
    let mut code = Code::default();
    admission(&mut code, "deny", true)?;
    if let Some(offset) = history_offset {
        let mut e = E(&mut code.encoder);
        e.load(R10, R8, (HEADER_BYTES + offset) as i32);
        e.cmp(R10, 0);
        e.0.jump_if(0x84, "deny");
        e.mov(A, R10);
        e.arith(4, A, 7);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "deny");
    }
    push_caller(&mut code);
    E(&mut code.encoder).mov(R12, _Sp);
    // Switch the CFA base before the first instruction that pivots RSP.
    saved_rows(&mut code, 12);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R13, R9);
        e.mov(R14, R8);
        e.store(R14, 0, R12);
        e.imm(A, 1);
        e.store(R14, 8, A);
        e.store(R14, 16, R13);
        e.mov(Di, R14);
        e.add(Di, HEADER_BYTES as i32);
        e.store(R14, 24, Di);
        address(&mut e, &mut code.references, A, ENTRY);
        e.store(R14, 32, A);
        if history_offset.is_some() {
            e.store(R14, 40, R10);
            // Returned source restores this register before target cleanup.
            // Header erasure must not destroy the late-failure history pin.
            e.mov(_Bp, R10);
        }
        e.imm(A, PRIVATE_BYTES);
        e.store(R14, 48, A);
        e.imm(A, STACK_BYTES);
        e.store(R14, 56, A);
        address(&mut e, &mut code.references, R11, ACTIVE);
        e.store(R11, 0, R14);
        e.mov(_Sp, R13);
        e.add(_Sp, STACK_BYTES as i32);
    }
    {
        let mut e = E(&mut code.encoder);
        branch(&mut e, &mut code.references, ENTRY, true);
        e.mov(R15, A);
        e.cmp(A, 1);
        e.0.jump_if(0x87, "fail");
        e.mov(_Sp, R12);
    }
    saved_rows(&mut code, 7);
    {
        let mut e = E(&mut code.encoder);
        erase(&mut e, R13, STACK_BYTES, "fail");
        erase(&mut e, R14, HEADER_BYTES, "fail");
    }
    clear_active(&mut code);
    clear_volatile(&mut E(&mut code.encoder));
    E(&mut code.encoder).mov(A, R15);
    pop_caller(&mut code);
    let normal = code.encoder.bytes.len() as u64;
    code.encoder.bytes(&[0xc3]);
    code.encoder.bind("fail")?;
    saved_rows(&mut code, 12);
    let mut fatal = 0;
    fatal_cleanup(
        &mut code,
        &mut fatal,
        if history_offset.is_some() {
            HistoryPin::Retained(_Bp)
        } else {
            HistoryPin::Absent
        },
    )?;
    code.encoder.bind("deny")?;
    let mut reset = vec![0x0c, 7, 8];
    for reg in [3, 6, 12, 13, 14, 15] {
        reset.extend([0x08, reg]);
    }
    row(&mut code, reset);
    let mut e = E(&mut code.encoder);
    clear_volatile(&mut e);
    e.imm(A, 2);
    e.0.bytes(&[0xc3]);
    Ok((code, normal, fatal))
}
fn abort(pinned_history: bool) -> Result<(Code, u64, Option<u64>), CompileError> {
    let mut code = Code::default();
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, R14, ACTIVE);
    }
    // The source caller's R14 is unavailable once recovery overwrites it.
    // Only the later pivot makes the owning caller's saved registers reachable.
    row(&mut code, vec![0x07, 14]); // DW_CFA_undefined
    {
        let mut e = E(&mut code.encoder);
        e.load(R14, R14, 0);
        e.cmp(R14, 0);
        e.0.jump_if(0x84, "inactive");
        e.load(R12, R14, 0);
    }
    row(&mut code, vec![0x07, 12]);
    {
        let mut e = E(&mut code.encoder);
        e.load(R13, R14, 16);
    }
    row(&mut code, vec![0x07, 13]);
    let mut completion_fatal = None;
    if pinned_history {
        E(&mut code.encoder).mov(_Sp, R12);
        saved_rows(&mut code, 7);
        {
            let mut e = E(&mut code.encoder);
            e.load(A, R14, 32);
            address(&mut e, &mut code.references, R11, completion::BODY);
            e.rr(0x39, A, R11);
            e.0.jump_if(0x85, "ordinary_abort");
        }
        let mut marker = 0;
        completion::abort_cleanup(&mut code, &mut marker)?;
        completion_fatal = Some(marker);
        code.encoder.bind("ordinary_abort")?;
        saved_rows(&mut code, 7);
    }
    let mut fatal = 0;
    fatal_cleanup(
        &mut code,
        &mut fatal,
        if pinned_history {
            HistoryPin::Header
        } else {
            HistoryPin::Absent
        },
    )?;
    code.encoder.bind("inactive")?;
    // This edge bypasses the R12/R13 loads; R14 remains unavailable.
    row(&mut code, vec![0x0c, 7, 8, 0x08, 12, 0x08, 13, 0x07, 14]);
    E(&mut code.encoder).add(_Sp, -8);
    row(&mut code, vec![0x0e, 16]);
    let mut e = E(&mut code.encoder);
    branch(&mut e, &mut code.references, FATAL, true);
    e.0.bytes(&[0x0f, 0x0b]);
    Ok((code, fatal, completion_fatal))
}

/// Failure/cancellation on the owning controller stack between source steps.
/// The trusted caller supplies the same still-owned extents; the inactive cell
/// cannot recover them. No source context/header contents are trusted here.
fn controller_fail() -> Result<(Code, u64), CompileError> {
    let mut code = Code::default();
    admission(&mut code, "invalid", false)?;
    push_caller(&mut code);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R12, _Sp);
        e.mov(R13, R9);
        e.mov(R14, R8);
    }
    let mut fatal = 0;
    fatal_cleanup(&mut code, &mut fatal, HistoryPin::Absent)?;
    code.encoder.bind("invalid")?;
    let mut reset = vec![0x0c, 7, 8];
    for reg in [3, 6, 12, 13, 14, 15] {
        reset.extend([0x08, reg]);
    }
    row(&mut code, reset);
    E(&mut code.encoder).add(_Sp, -8);
    row(&mut code, vec![0x0e, 16]);
    let mut e = E(&mut code.encoder);
    // Invalid controller protocol supplies no extent authority. Do not erase
    // arbitrary addresses or return to an owner whose cleanup is unproven.
    branch(&mut e, &mut code.references, FATAL, true);
    e.0.bytes(&[0x0f, 0x0b]);
    Ok((code, fatal))
}

pub(crate) fn component_object() -> Result<Vec<u8>, CompileError> {
    object_with_history(None)
}
pub(crate) fn runtime_object(history_offset: u64) -> Result<Vec<u8>, CompileError> {
    if history_offset % 8 != 0 || history_offset > PRIVATE_BYTES - HEADER_BYTES - 8 {
        return Err(CompileError::Tool(
            "invalid tail runtime history layout".into(),
        ));
    }
    object_with_history(Some(history_offset))
}
fn object_with_history(history_offset: Option<u64>) -> Result<Vec<u8>, CompileError> {
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = object.add_section(
        Vec::new(),
        b".text.topal.tail.boundary".to_vec(),
        SectionKind::Text,
    );
    let frame = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    object.add_section(Vec::new(), b".note.GNU-stack".to_vec(), SectionKind::Other);
    let data = object.add_section(
        Vec::new(),
        b".bss.topal.tail.boundary".to_vec(),
        SectionKind::UninitializedData,
    );
    let offset = object.append_section_bss(data, 8, 8);
    object.add_symbol(Symbol {
        name: ACTIVE.as_bytes().to_vec(),
        value: offset,
        size: 8,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(data),
        flags: SymbolFlags::None,
    });
    let (call, normal, fatal) = call(history_offset)?;
    append_discontinuous_code(&mut object, text, frame, CALL, call)?;
    let start = object
        .symbol(object.symbol_id(CALL.as_bytes()).unwrap())
        .value;
    for (name, offset) in [
        ("topal_x86_tail_boundary_return_reclaimed_v1", normal),
        ("topal_x86_tail_boundary_return_fatal_v1", fatal),
    ] {
        marker(&mut object, text, name, start + offset);
    }
    let (abort, fatal, completion_fatal) = abort(history_offset.is_some())?;
    append_discontinuous_code(&mut object, text, frame, ABORT, abort)?;
    let start = object
        .symbol(object.symbol_id(ABORT.as_bytes()).unwrap())
        .value;
    marker(
        &mut object,
        text,
        "topal_x86_tail_boundary_abort_reclaimed_v1",
        start + fatal,
    );
    if let Some(offset) = completion_fatal {
        marker(
            &mut object,
            text,
            "topal_x86_tail_complete_abort_reclaimed_v1",
            start + offset,
        );
    }
    let (failure, fatal) = controller_fail()?;
    append_code(&mut object, text, frame, FAIL, failure)?;
    let start = object
        .symbol(object.symbol_id(FAIL.as_bytes()).unwrap())
        .value;
    marker(
        &mut object,
        text,
        "topal_x86_tail_boundary_controller_reclaimed_v1",
        start + fatal,
    );
    if history_offset.is_some() {
        checkpoint::append(&mut object, text, frame)?;
        completion::append(&mut object, text, frame)?;
    }
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}
fn marker(object: &mut Object<'_>, text: object::write::SectionId, name: &str, value: u64) {
    object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value,
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });
}
