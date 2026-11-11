//! Unpublished owning bootstrap gateway. Root/publication binding is separate.
use super::*;
#[path = "systems_frame_gateway_memory.rs"]
mod memory;

pub(crate) const CALL_GATE: &str = "topal_x86_frame_gateway_v1";
pub(crate) const ABORT_GATE: &str = "topal_x86_frame_gateway_abort_v1";
pub(crate) const RECOVERY_GATE: &str = "topal_x86_frame_gateway_recovery_v1";
const DISPATCH: &str = "topal_frame_kernel_dispatch_v1";
const CONTEXT: &str = "topal_x86_frame_context_v1";
const HISTORY: &str = "topal_x86_frame_history_v1";
pub(crate) const RETIRED: u64 = 4032;
const MIN_SP: i32 = 0x109040;
const MAX_SP: i32 = 0x17fff8;
const DOMAINS: [(&str, u64); 7] = [
    ("topal_x86_frame_private_v1", 65528),
    ("topal_x86_frame_stack_v1", 4096),
    ("topal_x86_frame_facts_v1", 24),
    ("topal_x86_frame_output_v1", 56),
    ("topal_frame_allocator_active_v1", 8),
    ("topal_x86_tail_boundary_active_v1", 8),
    ("topal_x86_frame_details_v1", 32),
];

fn halt(code: &mut Code, label: &str) -> Result<(), CompileError> {
    code.encoder.bind(label)?;
    code.encoder.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    Ok(())
}
fn check_zero(code: &mut Code, symbol: &'static str, bytes: u64, fail: &str) {
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, symbol);
    e.mov(Di, R11);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, fail);
}
fn wipe(code: &mut Code, symbol: &'static str, bytes: u64, fail: &str) {
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, symbol);
    erase(&mut e, R11, bytes, fail);
}
fn poison(code: &mut Code, syscalls: bool) {
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, CONTEXT);
    e.imm(A, 3);
    e.store(R11, 0, A);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.imm(A, 0);
    e.store(R11, 24, A);
    e.imm(A, 1);
    e.store(R11, 32, A);
    if syscalls {
        address(&mut e, &mut code.references, R11, memory::STATE);
        e.imm(A, 3);
        e.store(R11, 0, A);
    }
}
fn scratch(code: &mut Code, fail: &str, retired: u64) {
    let mut e = E(&mut code.encoder);
    e.mov(R11, R12);
    e.add(R11, -(retired as i32));
    erase(&mut e, R11, retired, fail);
}
fn finish(code: &mut Code, normal: bool, fail: &str) {
    saved_rows(code, 7);
    pop_caller(code);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R11, _Sp);
        e.add(R11, -48);
        erase(&mut e, R11, 48, fail);
        address(&mut e, &mut code.references, R11, RECOVERY_GATE);
        if normal {
            e.load(A, R11, 32);
        } else {
            e.imm(A, 0);
        }
        e.imm(C, 0);
        for offset in (0..48).step_by(8) {
            e.store(R11, offset, C);
        }
        e.0.bind("recovery_cleared").unwrap();
        for offset in (0..48).step_by(8) {
            e.mem(0x81, Di, R11, offset);
            e.0.bytes(&0_i32.to_le_bytes());
            e.0.jump_if(0x85, fail);
        }
        for reg in [C, D, Si, Di, R8, R9, R10, R11] {
            e.imm(reg, 0);
        }
    }
}

fn terminal_caller(code: &mut Code, retired: u64, label: &str) -> Result<(), CompileError> {
    if retired == RETIRED {
        return Ok(());
    }
    let mut e = E(&mut code.encoder);
    e.load(A, _Sp, 0);
    address(
        &mut e,
        &mut code.references,
        R10,
        "topal_x86_user_extent_return_v1",
    );
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, label);
    branch(
        &mut e,
        &mut code.references,
        "topal_x86_user_extent_caller_abort_v1",
        false,
    );
    e.0.bind(label)?;
    clear_volatile(&mut e);
    Ok(())
}
fn late(code: &mut Code, syscalls: bool, retired: u64) -> Result<(), CompileError> {
    code.encoder.bind("late_failure")?;
    poison(code, syscalls);
    // Original callee registers are already restored and their copies erased.
    // Never reload those slots on a late failure.
    wipe(code, "topal_x86_frame_details_v1", 32, "late_halt");
    {
        let mut e = E(&mut code.encoder);
        e.mov(R11, _Sp);
        e.add(R11, -((retired + 48) as i32));
        erase(&mut e, R11, retired + 48, "late_halt");
        address(&mut e, &mut code.references, R11, RECOVERY_GATE);
        e.imm(C, 0);
        for offset in (0..48).step_by(8) {
            e.store(R11, offset, C);
        }
        for offset in (0..48).step_by(8) {
            e.mem(0x81, Di, R11, offset);
            e.0.bytes(&0_i32.to_le_bytes());
            e.0.jump_if(0x85, "late_halt");
        }
        clear_volatile(&mut e);
    }
    terminal_caller(code, retired, "late_caller_not_owned")?;
    halt(code, "late_halt")
}
fn call(
    syscalls: bool,
    user_extents: bool,
    retired: u64,
) -> Result<(Code, u64, u64), CompileError> {
    let mut code = Code::default();
    {
        let mut e = E(&mut code.encoder);
        e.cmp(_Sp, MIN_SP);
        e.0.jump_if(0x82, "unsafe");
        e.cmp(_Sp, MAX_SP);
        e.0.jump_if(0x87, "unsafe");
        e.mov(A, _Sp);
        e.arith(4, A, 15);
        e.cmp(A, 8);
        e.0.jump_if(0x85, "unsafe");
        e.imm(A, 0);
        e.0.bytes(&[0x8c, 0xc8]);
        e.arith(4, A, 3);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "unsafe");
        e.0.bytes(&[0x9c]);
    }
    row(&mut code, vec![0x0e, 16]);
    code.encoder.bytes(&[0x58]);
    row(&mut code, vec![0x0e, 8]);
    {
        let mut e = E(&mut code.encoder);
        e.arith(4, A, 0x600);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "unsafe"); // IF=DF=0
    }
    if syscalls {
        memory::entry(&mut code, user_extents, retired != RETIRED)?;
    }
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, R11, RECOVERY_GATE);
        for offset in (0..48).step_by(8) {
            e.load(A, R11, offset);
            e.cmp(A, 0);
            e.0.jump_if(0x85, "unsafe");
        }
    }
    push_caller(&mut code);
    E(&mut code.encoder).mov(R12, _Sp);
    saved_rows(&mut code, 12);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R13, Di);
        e.mov(R14, Si);
        e.mov(R15, D);
        e.mov(B, C);
        e.mov(_Bp, R8);
        address(&mut e, &mut code.references, R11, RECOVERY_GATE);
        e.store(R11, 0, R12);
        e.imm(A, 1);
        e.store(R11, 8, A);
        e.store(R11, 16, R13);
    }
    if syscalls {
        memory::capture(&mut code)?;
    }
    {
        let mut e = E(&mut code.encoder);
        e.mov(_Sp, R12);
        e.arith(4, _Sp, -16); // at most8 alignment bytes
        e.mov(Di, R13);
        e.mov(Si, R14);
        e.mov(D, R15);
        e.mov(C, B);
        e.mov(R8, _Bp);
        branch(&mut e, &mut code.references, DISPATCH, true);
        e.0.bind("dispatched")?;
        e.mov(_Sp, R12);
        address(&mut e, &mut code.references, R11, RECOVERY_GATE);
        e.store(R11, 32, A);
        e.load(C, R11, 8);
        e.cmp(C, 1);
        e.0.jump_if(0x85, "failed");
        e.load(C, R11, 0);
        e.rr(0x39, C, R12);
        e.0.jump_if(0x85, "failed");
        e.load(C, R11, 16);
        e.cmp(C, 0);
        e.0.jump_if(0x84, "constructor_result");
    }
    if syscalls {
        memory::result(&mut code, user_extents)?;
    }
    {
        let mut e = E(&mut code.encoder);
        e.cmp(A, 1);
        e.0.jump_if(0x87, "failed");
        e.0.jump("result_checked");
        e.0.bind("constructor_result")?;
        e.cmp(A, 0);
        e.0.jump_if(0x84, "result_checked");
        address(&mut e, &mut code.references, R11, CONTEXT);
        e.rr(0x39, A, R11);
        e.0.jump_if(0x85, "failed");
        e.0.bind("result_checked")?;
    }
    for (name, bytes) in DOMAINS.into_iter().take(6) {
        check_zero(&mut code, name, bytes, "failed");
    }
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, R11, HISTORY);
        e.load(A, R11, 24);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "failed");
        e.load(A, R11, 32);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "failed");
        address(&mut e, &mut code.references, R11, CONTEXT);
        e.load(A, R11, 0);
        e.cmp(A, 2);
        e.0.jump_if(0x84, "failed");
    }
    scratch(&mut code, "failed", retired);
    finish(&mut code, true, "late_failure");
    let normal = code.encoder.bytes.len() as u64;
    code.encoder.bytes(&[0xc3]);
    late(&mut code, syscalls, retired)?;
    code.encoder.bind("failed")?;
    saved_rows(&mut code, 12);
    branch(
        &mut E(&mut code.encoder),
        &mut code.references,
        ABORT_GATE,
        false,
    );
    row(&mut code, vec![0x0c, 7, 8]);
    let refused = code.encoder.bytes.len() as u64;
    halt(&mut code, "unsafe")?;
    Ok((code, normal, refused))
}
fn abort(syscalls: bool, retired: u64) -> Result<(Code, u64), CompileError> {
    let mut code = Code::default();
    row(&mut code, vec![0x07, 16]);
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, R11, RECOVERY_GATE);
        e.load(A, R11, 8);
        e.cmp(A, 1);
        e.0.jump_if(0x85, "unsafe");
        e.load(R12, R11, 0);
        e.cmp(R12, MIN_SP - 48);
        e.0.jump_if(0x82, "unsafe");
        e.cmp(R12, MAX_SP - 48);
        e.0.jump_if(0x87, "unsafe");
        e.mov(A, R12);
        e.arith(4, A, 15);
        e.cmp(A, 8);
        e.0.jump_if(0x85, "unsafe");
        e.mov(_Sp, R12);
    }
    saved_rows(&mut code, 12);
    row(&mut code, vec![0x90, 1]);
    poison(&mut code, syscalls);
    for (i, (name, bytes)) in DOMAINS.into_iter().enumerate() {
        let next = format!("retired_{i}");
        wipe(&mut code, name, bytes, &next);
        code.encoder.bind(&next)?;
    }
    scratch(&mut code, "scratch_erased", retired);
    code.encoder.bind("scratch_erased")?;
    finish(&mut code, false, "late_failure");
    terminal_caller(&mut code, retired, "fatal_caller_not_owned")?;
    let reclaimed = code.encoder.bytes.len() as u64;
    halt(&mut code, "reclaimed")?;
    late(&mut code, syscalls, retired)?;
    halt(&mut code, "unsafe")?;
    Ok((code, reclaimed))
}
pub(crate) fn component_object(complete_body_bound: u64) -> Result<Vec<u8>, CompileError> {
    component_selected(complete_body_bound, false, false, RETIRED)
}
pub(crate) fn memory_component_object(complete_body_bound: u64) -> Result<Vec<u8>, CompileError> {
    component_selected(complete_body_bound, true, false, RETIRED)
}
pub(crate) fn user_extent_component_object(
    complete_body_bound: u64,
) -> Result<Vec<u8>, CompileError> {
    component_selected(complete_body_bound, true, true, RETIRED)
}

#[cfg(test)]
pub(crate) fn owned_user_extent_component_object(
    complete_body_bound: u64,
) -> Result<Vec<u8>, CompileError> {
    use crate::systems_provider_object::user_extent_bridge::{
        CALLER_OUTER_BYTES, CALLER_RETIRED_BYTES,
    };
    if complete_body_bound
        .checked_add(64 + CALLER_OUTER_BYTES)
        .is_none_or(|n| n > 4096)
        || CALLER_RETIRED_BYTES + 48 + CALLER_OUTER_BYTES > 4096
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-USER-EXTENT: complete caller exceeds its reserve".into(),
        ));
    }
    component_selected(complete_body_bound, true, true, CALLER_RETIRED_BYTES)
}
fn component_selected(
    complete_body_bound: u64,
    syscalls: bool,
    user_extents: bool,
    retired: u64,
) -> Result<Vec<u8>, CompileError> {
    if retired == 0
        || retired % 16 != 0
        || retired > RETIRED
        || complete_body_bound == 0
        || complete_body_bound % 8 != 0
        || complete_body_bound
            .checked_add(16)
            .is_none_or(|n| n > retired)
        || complete_body_bound.checked_add(72).is_none_or(|n| n > 4096)
        || (syscalls && complete_body_bound.checked_add(80).is_none_or(|n| n > 4096))
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-FRAME-POLICY-BINDING: incomplete owning gateway stack bound".into(),
        ));
    }
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = object.add_section(
        Vec::new(),
        b".text.topal.frame.gateway".to_vec(),
        SectionKind::Text,
    );
    let frame = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    object.add_section(Vec::new(), b".note.GNU-stack".to_vec(), SectionKind::Other);
    let bss = object.add_section(
        Vec::new(),
        b".bss.topal.frame.gateway".to_vec(),
        SectionKind::UninitializedData,
    );
    let value = object.append_section_bss(bss, 48, 16);
    object.add_symbol(Symbol {
        name: RECOVERY_GATE.as_bytes().to_vec(),
        value,
        size: 48,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(bss),
        flags: SymbolFlags::None,
    });
    // Define the terminal function before the call gate references it. The
    // shared writer otherwise creates a redundant undefined forward symbol.
    let (code, point) = abort(syscalls, retired)?;
    append_code(&mut object, text, frame, ABORT_GATE, code)?;
    let start = object
        .symbol(object.symbol_id(ABORT_GATE.as_bytes()).unwrap())
        .value;
    marker(
        &mut object,
        text,
        "topal_x86_frame_gateway_fatal_reclaimed_v1",
        start + point,
    );
    let (code, normal, refused) = call(syscalls, user_extents, retired)?;
    let checking = code.encoder.labels["recovery_cleared"] as u64;
    let late = code.encoder.labels["late_halt"] as u64;
    let dispatched = code.encoder.labels["dispatched"] as u64;
    append_code(&mut object, text, frame, CALL_GATE, code)?;
    let start = object
        .symbol(object.symbol_id(CALL_GATE.as_bytes()).unwrap())
        .value;
    for (name, offset) in [
        ("topal_x86_frame_gateway_return_reclaimed_v1", normal),
        ("topal_x86_frame_gateway_refused_v1", refused),
        ("topal_x86_frame_gateway_recovery_check_v1", checking),
        ("topal_x86_frame_gateway_late_reclaimed_v1", late),
        ("topal_x86_frame_gateway_dispatched_v1", dispatched),
    ] {
        marker(&mut object, text, name, start + offset);
    }
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}
