//! Private numeric arena lending; source sees only ordinary numeric facts.
use super::R::_Sp as Sp;
use super::{CompileError, E, R::*, X86FunctionEncoder};

pub(in super::super) const CODE_CONTEXT: u32 = 960;
pub(in super::super) const ARENA_CONTEXT: u32 = 968;
pub(in super::super) const ARENA_BYTES: u64 = 65536;
pub(in super::super) const ARENA_SYMBOL: &str = "topal_x86_systems_numeric_arena";
pub(in super::super) const ALLOCATE_SYMBOL: &str = "topal_kernel_numeric_allocate_v1";
pub(in super::super) const ABORT_SYMBOL: &str = "topal_kernel_numeric_abort_v1";

fn zero_readback(e: &mut E<'_>, base: super::R, bytes: u64) {
    e.mov(Di, base);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
}
fn erase(e: &mut E<'_>, base: super::R, bytes: u64) {
    e.mov(Di, base);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    zero_readback(e, base, bytes);
}
pub(in super::super) fn emit_entry_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.load(A, Sp, 0x68); // fourteenth argument after five provider saves
    e.cmp(A, 0x200000);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, 0x40000000);
    e.0.jump_if(0x83, "fail");
    e.cmp(A, 0x400000);
    e.0.jump_if(0x82, "numeric-code-disjoint");
    e.cmp(A, 0x800000);
    e.0.jump_if(0x82, "fail");
    e.0.bind("numeric-code-disjoint")?;
    e.store(R10, CODE_CONTEXT as i32, A);
    e.load(R8, Sp, 0x70); // fifteenth argument: artifact-owned temporary arena
    e.cmp(R8, 0x200000);
    e.0.jump_if(0x82, "fail");
    e.mov(A, R8);
    e.arith(4, A, 4095);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.mov(A, R8);
    e.add(A, ARENA_BYTES as i32);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, 0x40000000);
    e.0.jump_if(0x87, "fail");
    e.cmp(A, 0x400000);
    e.0.jump_if(0x86, "numeric-arena-disjoint");
    e.cmp(R8, 0x800000);
    e.0.jump_if(0x82, "fail");
    e.0.bind("numeric-arena-disjoint")?;
    e.mov(Di, R8);
    e.imm(A, 0);
    e.imm(C, ARENA_BYTES / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fail");
    e.store(R10, ARENA_CONTEXT as i32, R8);
    Ok(())
}

/// B/BP interval, R13 raw sample, R15 wrapped Boolean; return target in R14.
pub(in super::super) fn emit_invoke(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.cmp(Sp, 0x108000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(Sp, 0x180000);
    e.0.jump_if(0x87, "fatal");
    e.0.bytes(&[0x9c, 0x58]); // PUSHFQ/POP RAX, outside the library call path
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
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
    e.imm(A, 1);
    e.store(R11, 16, A); // exclusive active invocation
    e.mov(Di, B);
    e.mov(Si, super::R::_Bp);
    e.mov(D, R13);
    e.mov(C, R15);
    e.gs(0x8b, A, CODE_CONTEXT);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.mov(R11, Sp);
    e.arith(4, Sp, -16);
    e.add(Sp, -16);
    e.store(Sp, 0, R11);
    e.0.bytes(&[0xff, 0xd0]); // artifact-owned private C ABI bridge
    e.load(R11, Sp, 0);
    e.mov(Sp, R11);
    e.mov(R14, A);
    e.imm(R10, 922337203685477581);
    e.rr(0x39, R14, R10);
    e.0.jump_if(0x87, "fatal");
    e.gs(0x8b, R11, ARENA_CONTEXT);
    e.gs(0x8b, A, 256);
    e.load(R10, R11, 0);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 16);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 8);
    e.cmp(A, 32);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fatal");
    erase(&mut e, R11, ARENA_BYTES);
    // Retired helper frames can contain private numeric pointers. The complete
    // caller-owned extra stack reserve is erased without touching live frames.
    e.mov(R11, Sp);
    e.add(R11, -4096);
    erase(&mut e, R11, 4096);
    for register in [A, C, D, Si, Di, R8, R9, R10, R11] {
        e.imm(register, 0);
    }
    Ok(())
}

pub(in super::super) fn emit_terminal_readback(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.load(R11, R10, ARENA_CONTEXT as i32);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fatal");
    zero_readback(&mut e, R11, ARENA_BYTES);
    Ok(())
}

/// This native C ABI leaf uses no stack beyond its caller's return address.
pub(in super::super) fn allocator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    let mut e = E(&mut code);
    e.gs(0x8b, R11, ARENA_CONTEXT);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fail");
    e.gs(0x8b, R10, 256);
    e.load(R8, R11, 0);
    e.rr(0x39, R8, R10);
    e.0.jump_if(0x85, "fail");
    e.load(R8, R11, 16);
    e.cmp(R8, 1);
    e.0.jump_if(0x85, "fail");
    e.mov(A, Di);
    e.add(A, 7);
    e.0.jump_if(0x82, "fail");
    e.arith(4, A, -8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.load(R9, R11, 8);
    e.cmp(R9, 32);
    e.0.jump_if(0x82, "fail");
    e.cmp(R9, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fail");
    e.mov(R8, R9);
    e.rr(0x01, R8, A);
    e.0.jump_if(0x82, "fail");
    e.cmp(R8, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fail");
    e.store(R11, 8, R8);
    e.load(C, R11, 24);
    e.add(C, 1);
    e.0.jump_if(0x82, "fail");
    e.cmp(C, 8192);
    e.0.jump_if(0x87, "fail");
    e.store(R11, 24, C);
    e.mov(A, R11);
    e.rr(0x01, A, R9);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    e.0.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    code.finish()
}
