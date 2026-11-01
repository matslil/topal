//! Native anonymous-file creation after checked namespace resolution.
use super::super::{
    R::_Bp as Bp,
    file_descriptions::{CHILD_DESCRIPTIONS, DESCRIPTIONS, SLOTS},
    namespace,
};
use super::*;

pub(super) fn emit(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    // R14 is a lowest-free descriptor; R15 is the checked directory identity.
    // BP retains flags and staging+248 retains the copied native mode word.
    e.0.bind("file-create-temporary")?;
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "namespace-enotdir");
    e.cmp(R15, 6);
    e.0.jump_if(0x85, "namespace-erofs");
    e.mov(R13, R12);
    e.add(R13, OBJECTS);
    e.imm(C, 4);
    e.0.bind("file-create-object-next")?;
    e.load(A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "file-create-object-ready");
    e.add(R13, 128);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, "file-create-object-next");
    e.0.jump("namespace-enospc");
    e.0.bind("file-create-object-ready")?;
    e.mov(Di, R13);
    e.imm(A, 0);
    e.imm(C, 128 / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
    e.load(R8, R12, BASE);
    e.cmp(R8, 7);
    e.0.jump_if(0x82, "fatal");
    e.mov(B, R8);
    e.add(B, 1);
    e.0.jump_if(0x82, "namespace-enospc");
    e.mov(R10, R12);
    e.add(R10, DESCRIPTIONS);
    e.imm(R11, 0);
    e.0.bind("file-create-description-next")?;
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "file-create-description-ready");
    e.add(R11, 1);
    e.cmp(R11, 32);
    e.0.jump_if(0x84, "fatal");
    e.add(R10, 64);
    e.cmp(R11, 16);
    e.0.jump_if(0x85, "file-create-description-next");
    e.mov(R10, R12);
    e.add(R10, CHILD_DESCRIPTIONS);
    e.0.jump("file-create-description-next");
    e.0.bind("file-create-description-ready")?;
    e.mov(Di, R10);
    e.imm(A, 0);
    e.imm(C, 64 / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
    e.load(R9, R12, STAGING + 248);
    e.arith(4, R9, 0o7777);
    e.load(A, R12, UMASK);
    e.0.bytes(&[0x48, 0xf7, 0xd0]);
    e.rr(0x21, R9, A);
    // Clock failure is recoverable before any resource has been installed.
    e.0.call("file-clock");
    e.0.jump_if(0x82, "namespace-eio");
    e.mov(R15, A);
    namespace::wipe_name(&mut e);
    wipe_staging(&mut e);
    e.store(R12, BASE, B);
    e.imm(A, 1);
    e.store(R13, 0, A);
    e.store(R13, 8, R8);
    e.store(R13, 24, R9);
    for offset in [32, 40, 48] {
        e.store(R13, offset, R15);
    }
    e.mov(A, Bp);
    shift(&mut e, A, false, 7);
    e.arith(4, A, 1);
    e.store(R13, 64, A);
    e.imm(A, KIND as u64);
    e.store(R10, 0, A);
    e.imm(A, 1);
    e.store(R10, 8, A);
    e.mov(A, Bp);
    e.arith(4, A, !(0x80000 | 0x100 | 0x80 | 0x40 | 0x200));
    e.arith(1, A, 0x8000);
    e.store(R10, 16, A);
    e.store(R10, 32, R8);
    e.mov(Si, R14);
    shift(&mut e, Si, true, 4);
    e.rr(0x01, Si, R12);
    e.add(Si, SLOTS);
    e.add(R11, 1);
    e.store(Si, 0, R11);
    e.mov(A, Bp);
    shift(&mut e, A, false, 19);
    e.arith(4, A, 1);
    e.store(Si, 8, A);
    e.mov(A, R14);
    emit_static_process_sysret(e.0);
    Ok(())
}
