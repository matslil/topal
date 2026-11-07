//! Linux argument/lookup/mount/clock admission before the ordinary inode action.
use super::R::_Bp as Bp;
use super::file_descriptions::{TABLE_CONTEXT, wipe_staging};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    for (number, label) in [
        (90, "inode-chmod"),
        (91, "inode-fchmod"),
        (268, "inode-fchmodat"),
        (452, "inode-fchmodat2"),
    ] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    Ok(())
}

fn finish(e: &mut E<'_>, value: i64) {
    wipe_staging(e);
    super::namespace::wipe_name(e);
    e.imm(A, value as u64);
    emit_static_process_sysret(e.0);
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("inode-chmod")?;
    e.mov(R8, Di);
    e.mov(Bp, Si);
    e.imm(B, 0xffff_ff9c);
    e.imm(R15, 0);
    e.0.jump("inode-path-common");
    e.0.bind("inode-fchmodat")?;
    e.mov(R8, Si);
    e.mov(Bp, D);
    e.mov(B, Di);
    e.imm(R15, 0); // the three-argument ABI does not inspect R10
    e.0.jump("inode-path-common");
    e.0.bind("inode-fchmodat2")?;
    e.mov(R8, Si);
    e.mov(Bp, D);
    e.mov(B, Di);
    e.mov(R15, R10);
    e.0.bytes(&[0x45, 0x89, 0xff]); // flags: unsigned int, before flag admission
    e.mov(A, R15);
    e.arith(4, A, !0x1100);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "inode-einval");
    e.0.bind("inode-path-common")?;
    e.arith(4, Bp, 0o7777); // umode_t then S_IALLUGO, no creation umask
    e.gs(0x8b, R12, TABLE_CONTEXT);
    wipe_staging(&mut e);
    e.0.call("namespace-copy-name");
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "inode-empty-path");
    e.imm(A, 0);
    e.store(R12, super::namespace::MODE, A); // lookup, not create/unlink
    e.0.call("namespace-walk");
    e.0.jump("inode-object");
    e.0.bind("inode-empty-path")?;
    e.mov(A, R15);
    e.arith(4, A, 0x1000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "inode-enoent");
    e.0.bytes(&[0x89, 0xdb]); // native int dirfd
    e.0.bytes(&[0x81, 0xfb, 0x9c, 0xff, 0xff, 0xff]);
    e.0.jump_if(0x84, "inode-empty-cwd");
    e.mov(Di, B);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "inode-ebadf");
    e.0.jump("inode-description");
    e.0.bind("inode-empty-cwd")?;
    e.load(R15, R12, 40);
    e.0.jump("inode-object");

    e.0.bind("inode-fchmod")?;
    e.gs(0x8b, R12, TABLE_CONTEXT);
    e.mov(Bp, Si);
    e.arith(4, Bp, 0o7777);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "inode-ebadf");
    e.0.bind("inode-description")?;
    e.load(A, R14, 0);
    e.cmp(A, 7);
    e.0.jump_if(0x84, "inode-file-description");
    e.cmp(A, 4);
    e.0.jump_if(0x84, "inode-directory-description");
    e.cmp(A, 5);
    e.0.jump_if(0x84, "inode-pipe-description");
    e.cmp(A, 6);
    e.0.jump_if(0x84, "inode-pipe-description");
    e.cmp(A, 3);
    e.0.jump_if(0x84, "inode-erofs");
    e.0.jump("inode-enosys"); // prepared console inode mutation remains unqualified
    e.0.bind("inode-directory-description")?;
    e.load(R15, R14, 32);
    e.0.jump("inode-object");
    e.0.bind("inode-file-description")?;
    e.0.call("file-object-lookup");
    e.imm(D, 0);
    e.0.jump("inode-admitted");
    e.0.bind("inode-pipe-description")?;
    e.0.call("pipe-object-lookup");
    e.imm(D, 2);
    e.0.jump("inode-admitted");

    e.0.bind("inode-object")?;
    e.cmp(R15, 6);
    e.0.jump_if(0x84, "inode-directory");
    e.cmp(R15, 7);
    e.0.jump_if(0x82, "inode-erofs");
    e.0.call("file-object-by-identity");
    e.imm(D, 0);
    e.0.jump("inode-admitted");
    e.0.bind("inode-directory")?;
    e.mov(R10, R12);
    e.add(R10, super::volatile_files::BASE + 32);
    e.imm(D, 1);
    e.0.bind("inode-admitted")?;
    // All recoverable lookup/access checks precede the clock and source grant.
    // The checked clock preserves kind, actual record and adapted permissions.
    e.0.call("file-clock");
    e.0.jump_if(0x82, "inode-eio");
    e.mov(R8, A);
    e.mov(Di, R12);
    e.mov(Si, R10);
    e.mov(C, Bp);
    e.0.external_call(super::super::inode_region::APPLY);
    finish(&mut e, 0);
    for (label, errno) in [
        ("inode-einval", 22),
        ("inode-enoent", 2),
        ("inode-ebadf", 9),
        ("inode-erofs", 30),
        ("inode-enosys", 38),
        ("inode-eio", 5),
    ] {
        e.0.bind(label)?;
        e.gs(0x8b, R12, TABLE_CONTEXT);
        finish(&mut e, -errno);
    }
    Ok(())
}
