//! Qualified store lending, prepared-graph validation and last-borrow retirement.
use super::R::_Sp as Sp;
use super::numeric::{erase, zero_readback};
use super::{CompileError, E, R::*, X86FunctionEncoder};
#[path = "systems_retained_validation.rs"]
mod graph;

pub(in super::super) const STORE_CONTEXT: u32 = 1000;
pub(in super::super) const STORE_BYTES: u64 = 4096;

pub(in super::super) fn emit_entry_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.load(R8, Sp, 0x90); // Explicit nineteenth root argument; never ambient lookup.
    e.cmp(R8, 0x200000);
    e.0.jump_if(0x82, "fail");
    e.mov(A, R8);
    e.arith(4, A, 4095);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.mov(A, R8);
    e.add(A, 4096);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, 0x40000000);
    e.0.jump_if(0x87, "fail");
    e.cmp(A, 0x400000);
    e.0.jump_if(0x86, "retained-entry-user-disjoint");
    e.cmp(R8, 0x800000);
    e.0.jump_if(0x82, "fail");
    e.0.bind("retained-entry-user-disjoint")?;
    e.load(R9, R10, super::numeric::ARENA_CONTEXT as i32);
    e.rr(0x39, A, R9);
    e.0.jump_if(0x86, "retained-entry-arena-disjoint");
    e.add(R9, 65536);
    e.0.jump_if(0x82, "fail");
    e.rr(0x39, R8, R9);
    e.0.jump_if(0x82, "fail");
    e.0.bind("retained-entry-arena-disjoint")?;
    e.load(R9, R10, super::identity::CODE_CONTEXT as i32);
    e.cmp(R9, 0);
    e.0.jump_if(0x84, "fail");
    // Entry validation returns a normal false before ownership is published.
    e.mov(Di, R8);
    e.imm(A, 0);
    e.imm(C, 512);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fail");
    e.store(R10, STORE_CONTEXT as i32, R8);
    e.store(R8, 0, R10);
    e.store(R8, 8, R9);
    Ok(())
}

/// The invocation's six callee saves are already live. Scalars are in its two
/// stack result slots. Validate a complete independent graph before cleanup.
pub(in super::super) fn emit_prepared_validation(
    code: &mut X86FunctionEncoder,
    structured: bool,
) -> Result<(), CompileError> {
    let layout = crate::systems_retained_layout::RetainedValueLayout::identity(structured)?;
    let mut e = E(code);
    e.gs(0x8b, R12, STORE_CONTEXT);
    e.cmp(R12, 0);
    e.0.jump_if(0x84, "fatal");
    e.gs(0x8b, R10, 256);
    e.load(A, R12, 0);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.gs(0x8b, R10, super::identity::CODE_CONTEXT);
    e.load(A, R12, 8);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R12, 40);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R12, 32);
    e.cmp(A, -1);
    e.0.jump_if(0x84, "fatal");
    e.load(R8, R12, 24);
    e.cmp(R8, 1);
    e.0.jump_if(0x87, "fatal");
    e.load(A, R12, 16);
    e.cmp(A, 1);
    e.0.jump_if(0x87, "fatal");
    e.cmp(A, 0);
    e.0.jump_if(0x85, "retained-prepared-initialized");
    e.cmp(R8, 0);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R12, 32);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.0.jump("retained-prepared-version-valid");
    e.0.bind("retained-prepared-initialized")?;
    e.load(A, R12, 32);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.0.bind("retained-prepared-version-valid")?;
    e.arith(6, R8, 1); // XOR selects the one inactive bank.
    e.load(A, R12, 48);
    e.rr(0x39, A, R8);
    e.0.jump_if(0x85, "fatal");
    e.mov(R13, R12);
    e.add(R13, 64);
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "retained-prepared-bank-ready");
    e.add(R13, 2016);
    e.0.bind("retained-prepared-bank-ready")?;
    graph::emit(&mut e, &layout)?;
    e.load(D, Sp, 8);
    e.load(A, R12, 56);
    e.rr(0x39, A, D);
    e.0.jump_if(0x85, "fatal");
    e.mov(R11, R13);
    e.add(R11, layout.graph_bytes as i32);
    zero_readback(&mut e, R11, 2016 - layout.graph_bytes);
    Ok(())
}

/// The owner has consumed both numeric results into its construction state.
/// Preserve all live native callee facts, publish the value and retire its old
/// graph without creating another stack frame or leaving source pointers live.
pub(in super::super) fn emit_commit(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R8, STORE_CONTEXT);
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "fatal");
    e.gs(0x8b, R10, 256);
    e.load(A, R8, 0);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.gs(0x8b, R10, super::identity::CODE_CONTEXT);
    e.load(A, R8, 8);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R8, 40);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(R9, R8, 24);
    e.cmp(R9, 1);
    e.0.jump_if(0x87, "fatal");
    e.mov(R10, R9);
    e.arith(6, R10, 1);
    e.load(A, R8, 48);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R8, 32);
    e.add(A, 1);
    e.0.jump_if(0x82, "fatal");
    e.store(R8, 32, A);
    e.store(R8, 24, R10);
    e.imm(A, 1);
    e.store(R8, 16, A);
    e.imm(A, 0);
    for offset in [40, 48, 56] {
        e.store(R8, offset, A);
    }
    e.mov(R11, R8);
    e.add(R11, 64);
    e.cmp(R9, 0);
    e.0.jump_if(0x84, "retained-retire-bank-ready");
    e.add(R11, 2016);
    e.0.bind("retained-retire-bank-ready")?;
    erase(&mut e, R11, 2016);
    for register in [A, C, D, Si, Di, R8, R9, R10, R11] {
        e.imm(register, 0);
    }
    Ok(())
}

/// All published/prepared graphs belong to this whole region. Termination
/// consumes them without dereferencing a source pointer or restoring history.
pub(in super::super) fn emit_release(code: &mut X86FunctionEncoder) {
    let mut e = E(code);
    e.load(R11, R10, STORE_CONTEXT as i32);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fatal");
    e.load(A, R11, 0);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 8);
    e.load(R8, R10, super::identity::CODE_CONTEXT as i32);
    e.rr(0x39, A, R8);
    e.0.jump_if(0x85, "fatal");
    erase(&mut e, R11, STORE_BYTES);
    for register in [A, C, D, Si, Di, R8, R9, R11] {
        e.imm(register, 0);
    }
}
