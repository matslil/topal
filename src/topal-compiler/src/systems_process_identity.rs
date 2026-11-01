//! Pure identity-state proposal; the process owner commits only after cleanup.
use super::R::_Sp as Sp;
use super::numeric::{ARENA_BYTES, ARENA_CONTEXT, erase, zero_readback};
use super::{CompileError, E, R::*, X86FunctionEncoder};

pub(in super::super) const CODE_CONTEXT: u32 = 992;

pub(in super::super) fn emit_invoke(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.load(R8, R15, 0x2008); // Immutable copied cursor; owner state stays untouched.
    e.cmp(Sp, 0x108000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(Sp, 0x180000);
    e.0.jump_if(0x87, "fatal");
    e.0.bytes(&[0x9c, 0x58]);
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    // Six callee saves and two scalar result slots are part of the stack proof.
    e.0.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    e.add(Sp, -16);
    e.gs(0x8b, R11, ARENA_CONTEXT);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fatal");
    zero_readback(&mut e, R11, ARENA_BYTES);
    e.gs(0x8b, A, 256);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.store(R11, 0, A);
    e.imm(A, 32);
    e.store(R11, 8, A);
    e.imm(A, 4);
    e.store(R11, 16, A);
    e.mov(Di, R8);
    e.gs(0x8b, A, CODE_CONTEXT);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.mov(R11, Sp);
    e.arith(4, Sp, -16);
    e.add(Sp, -16);
    e.store(Sp, 0, R11);
    e.0.bytes(&[0xff, 0xd0]);
    e.load(R11, Sp, 0);
    e.mov(Sp, R11);
    for register in [A, D] {
        e.cmp(register, 2);
        e.0.jump_if(0x82, "fatal");
        e.cmp(register, i32::MAX);
        e.0.jump_if(0x87, "fatal");
    }
    e.store(Sp, 0, A);
    e.store(Sp, 8, D);
    e.gs(0x8b, R11, ARENA_CONTEXT);
    e.gs(0x8b, A, 256);
    e.load(R10, R11, 0);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 16);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 8);
    e.cmp(A, 32);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fatal");
    erase(&mut e, R11, ARENA_BYTES);
    e.load(R8, Sp, 0);
    e.load(R9, Sp, 8);
    e.add(Sp, 16);
    e.0.bytes(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    e.mov(R11, Sp);
    e.add(R11, -4096);
    erase(&mut e, R11, 4096);
    for register in [A, C, D, Si, Di, R10, R11] {
        e.imm(register, 0);
    }
    e.mov(A, R8);
    e.mov(D, R9);
    e.imm(R8, 0);
    e.imm(R9, 0);
    Ok(())
}
