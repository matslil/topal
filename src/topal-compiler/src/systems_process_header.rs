//! Caller-owned immutable input and exclusive copied-byte policy invocation.
use super::R::_Sp as Sp;
use super::numeric::{ARENA_BYTES, ARENA_CONTEXT, erase, zero_readback};
use super::{CompileError, E, R::*, X86FunctionEncoder};

pub(in super::super) const CODE_CONTEXT: u32 = 976;

/// Six native saves and two local words contribute 64 bytes to the complete
/// caller-reserve certificate, in addition to alignment and the actual callee.
pub(in super::super) fn emit_invoke(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.cmp(Sp, 0x108000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(Sp, 0x180000);
    e.0.jump_if(0x87, "fatal");
    e.0.bytes(&[0x9c, 0x58]);
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
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
    e.imm(A, 2);
    e.store(R11, 16, A);

    // The named file is a borrow of the validated immutable archive, separate
    // from any description's current offset or close-on-exec flags.
    e.gs(0x8b, R10, super::super::process_files::TABLE_CONTEXT);
    e.load(A, R10, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R10, 48);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R10, 24);
    e.cmp(A, 64);
    e.0.jump_if(0x82, "fatal");
    e.load(Di, R10, 16);
    e.gs(0x8b, R8, super::super::process_files::ARCHIVE_CONTEXT);
    e.add(
        R8,
        super::super::X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET as i32,
    );
    e.rr(0x39, Di, R8);
    e.0.jump_if(0x85, "fatal");
    e.imm(Si, 64);
    e.imm(D, 62);
    e.imm(C, 16);
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
    e.cmp(A, 1);
    e.0.jump_if(0x87, "fatal");
    e.store(Sp, 0, A);
    e.gs(0x8b, R11, ARENA_CONTEXT);
    e.gs(0x8b, A, 256);
    e.load(R10, R11, 0);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 16);
    e.cmp(A, 2);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 8);
    e.cmp(A, 32);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fatal");
    erase(&mut e, R11, ARENA_BYTES);
    e.load(R8, Sp, 0);
    e.add(Sp, 16);
    e.0.bytes(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    // Erase exactly the original caller's reserve, including popped saves,
    // after restoring live facts. Only a validated Boolean remains in R8.
    e.mov(R11, Sp);
    e.add(R11, -4096);
    erase(&mut e, R11, 4096);
    for register in [A, C, D, Si, Di, R9, R10, R11] {
        e.imm(register, 0);
    }
    e.mov(A, R8);
    e.imm(R8, 0);
    Ok(())
}

pub(in super::super) fn emit_refusal(code: &mut X86FunctionEncoder) {
    let mut e = E(code);
    e.wipe();
    e.imm(A, (-8i64) as u64);
    super::super::emit_static_process_sysret(e.0);
}
