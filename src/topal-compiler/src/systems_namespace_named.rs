//! Source-open plans and checked named-object publication (TK-DEC-267).
use super::super::volatile_files as files;
use super::*;

pub(super) fn wipe_directory(e: &mut E<'_>) {
    e.mov(R11, R12);
    e.add(R11, files::DIRECTORY_STAGING);
    files::erase(e, R11, 512);
}
pub(super) fn emit_open_plan(e: &mut E<'_>) -> Result<(), CompileError> {
    e.imm(Di, 0);
    // Named bit adaptation is private; facts are derived only from owned views.
    e.mov(A, Bp);
    e.arith(4, A, 3);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "named-plan-read");
    e.arith(1, Di, 1);
    e.0.bind("named-plan-read")?;
    e.cmp(R15, 0);
    e.0.jump_if(0x84, "named-plan-missing");
    e.arith(1, Di, 16);
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "named-plan-regular");
    e.cmp(R15, 7);
    e.0.jump_if(0x83, "named-plan-mutable");
    e.arith(1, Di, 2);
    e.cmp(R15, 6);
    e.0.jump_if(0x85, "named-plan-regular");
    e.0.jump("named-plan-mutable");
    e.0.bind("named-plan-missing")?;
    e.load(A, R12, PARENT);
    e.cmp(A, 6);
    e.0.jump_if(0x85, "named-plan-regular");
    e.0.bind("named-plan-mutable")?;
    e.arith(1, Di, 8);
    e.0.bind("named-plan-regular")?;
    for (native, bit, label) in [
        (0x10000, 4, "named-plan-directory"),
        (0x40, 32, "named-plan-create"),
        (0x80, 64, "named-plan-exclusive"),
        (0x200, 128, "named-plan-truncate"),
    ] {
        e.mov(A, Bp);
        e.arith(4, A, native);
        e.cmp(A, 0);
        e.0.jump_if(0x84, label);
        e.arith(1, Di, bit);
        e.0.bind(label)?;
    }
    super::super::policy::emit_call(e)?;
    for (choice, label) in [
        (1, "namespace-enotdir"),
        (2, "namespace-eisdir"),
        (3, "namespace-erofs"),
        (4, "namespace-enoent"),
        (5, "namespace-eexist"),
    ] {
        e.cmp(A, choice);
        e.0.jump_if(0x84, label);
    }
    e.cmp(A, 6);
    e.0.jump_if(0x84, "file-create-named");
    e.cmp(A, 7);
    e.0.jump_if(0x84, "named-plan-truncate-owner");
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.cmp(R15, 0);
    e.0.jump_if(0x84, "fatal");
    e.mov(A, Bp);
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.mov(A, Bp);
    e.arith(4, A, 0x10000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "named-plan-kind-ready");
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "fatal");
    e.cmp(R15, 7);
    e.0.jump_if(0x83, "fatal");
    e.0.bind("named-plan-kind-ready")?;
    e.cmp(R15, 7);
    e.0.jump_if(0x83, "namespace-open-linked");
    e.mov(A, Bp);
    e.arith(4, A, 3);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.0.jump("named-plan-existing-ready");
    e.0.bind("named-plan-truncate-owner")?;
    e.cmp(R15, 7);
    e.0.jump_if(0x82, "fatal");
    e.mov(A, Bp);
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.mov(A, Bp);
    e.arith(4, A, 0x10000);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.0.jump("namespace-open-linked");
    e.0.bind("named-plan-existing-ready")?;
    Ok(())
}
fn reopen(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-open-linked")?;
    e.mov(R10, R12);
    e.add(R10, DESCRIPTIONS);
    e.imm(B, 0);
    e.0.bind("named-description-next")?;
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "named-description-ready");
    e.add(B, 1);
    e.cmp(B, 32);
    e.0.jump_if(0x84, "fatal");
    e.add(R10, 64);
    e.cmp(B, 16);
    e.0.jump_if(0x85, "named-description-next");
    e.mov(R10, R12);
    e.add(R10, super::super::file_descriptions::CHILD_DESCRIPTIONS);
    e.0.jump("named-description-next");
    e.0.bind("named-description-ready")?;
    e.mov(Di, R10);
    e.imm(A, 0);
    e.imm(C, 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
    e.mov(R13, R10);
    e.0.call("file-object-by-identity");
    e.load(A, R10, 56);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.mov(A, Bp);
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "named-open-publish");
    // RTC clears private transfer staging, but retains every owned GPR here.
    e.0.call("file-clock");
    e.0.jump_if(0x82, "namespace-eio");
    e.mov(R15, A);
    e.0.call("file-release-pages");
    e.imm(A, 0);
    e.store(R10, 16, A);
    e.store(R10, 40, R15);
    e.store(R10, 48, R15);
    e.0.bind("named-open-publish")?;
    e.load(R15, R10, 8);
    e.imm(A, files::KIND as u64);
    e.store(R13, 0, A);
    e.imm(A, 1);
    e.store(R13, 8, A);
    e.store(R13, 32, R15);
    e.mov(A, Bp);
    e.arith(4, A, !(0x80000 | 0x100 | 0x40 | 0x80 | 0x200));
    e.arith(1, A, 0x8000);
    e.store(R13, 16, A);
    e.mov(R10, R14);
    files::shift(e, R10, true, 4);
    e.rr(0x01, R10, R12);
    e.add(R10, SLOTS);
    e.add(B, 1);
    e.store(R10, 0, B);
    e.mov(A, Bp);
    files::shift(e, A, false, 19);
    e.arith(4, A, 1);
    e.store(R10, 8, A);
    wipe_staging(e);
    wipe_name(e);
    e.mov(A, R14);
    emit_static_process_sysret(e.0);
    Ok(())
}
fn unlink(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-unlink")?;
    e.mov(R8, Di);
    e.imm(B, 0xffff_ff9c);
    e.imm(Bp, 0);
    e.0.jump("named-unlink-copy");
    e.0.bind("namespace-unlinkat")?;
    e.mov(B, Di);
    e.mov(R8, Si);
    e.mov(Bp, D);
    e.0.bytes(&[0x89, 0xed]);
    e.mov(A, Bp);
    e.arith(4, A, !0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-einval");
    e.cmp(Bp, 0);
    e.0.jump_if(0x85, "namespace-enosys");
    e.0.bind("named-unlink-copy")?;
    e.0.call("namespace-copy-name");
    e.imm(A, 2);
    e.store(R12, MODE, A);
    e.0.call("namespace-walk");
    e.cmp(R15, 7);
    e.0.jump_if(0x82, "namespace-eisdir");
    e.0.call("file-object-by-identity");
    e.load(A, R10, 56);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.0.call("file-clock");
    e.0.jump_if(0x82, "namespace-eio");
    e.mov(R13, A);
    e.imm(A, 0);
    e.store(R10, 56, A);
    e.store(R10, 72, A);
    files::name_pointer(e, R11, R10);
    files::erase(e, R11, 256);
    e.store(R10, 48, R13);
    e.store(R12, files::BASE + 8, R13);
    e.store(R12, files::BASE + 16, R13);
    e.0.call("file-live-description");
    e.0.jump_if(0x83, "named-unlink-retained");
    e.0.call("file-destroy-object");
    e.0.bind("named-unlink-retained")?;
    wipe_staging(e);
    finish(e, 0);
    Ok(())
}
fn directory(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("named-directory")?;
    e.0.bytes(&[0x89, 0xd5]); // Linux unsigned count admission.
    e.cmp(Bp, 512);
    e.0.jump_if(0x86, "named-directory-budget");
    e.imm(Bp, 512);
    e.0.bind("named-directory-budget")?;
    e.load(R13, R14, 24);
    e.imm(R15, 0);
    wipe_staging(e);
    wipe_directory(e);
    e.0.bind("named-directory-next")?;
    e.cmp(R13, 6);
    e.0.jump_if(0x83, "named-directory-publish");
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "named-directory-dot");
    e.cmp(R13, 1);
    e.0.jump_if(0x84, "named-directory-parent");
    e.mov(R10, R13);
    e.add(R10, -2);
    files::shift(e, R10, true, 7);
    e.rr(0x01, R10, R12);
    e.add(R10, files::OBJECTS);
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "named-directory-skip");
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R10, 56);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "named-directory-skip");
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(D, R10, 72);
    e.cmp(D, 1);
    e.0.jump_if(0x82, "fatal");
    e.cmp(D, 255);
    e.0.jump_if(0x87, "fatal");
    files::name_pointer(e, Si, R10);
    e.load(B, R10, 8);
    e.imm(R8, 8);
    e.0.jump("named-directory-record");
    e.0.bind("named-directory-dot")?;
    e.imm(B, 6);
    e.imm(D, 1);
    e.imm(R8, 4);
    e.0.rip_lea(&[0x48, 0x8d, 0x35], "named-directory-dot-data");
    e.0.jump("named-directory-record");
    e.0.bind("named-directory-parent")?;
    e.imm(B, 3);
    e.imm(D, 2);
    e.imm(R8, 4);
    e.0.rip_lea(&[0x48, 0x8d, 0x35], "named-directory-parent-data");
    e.0.bind("named-directory-record")?;
    e.mov(R9, D);
    e.add(R9, 27);
    e.arith(4, R9, -8);
    e.mov(A, R15);
    e.rr(0x01, A, R9);
    e.rr(0x39, A, Bp);
    e.0.jump_if(0x87, "named-directory-no-fit");
    e.mov(R11, R12);
    e.add(R11, files::DIRECTORY_STAGING);
    e.rr(0x01, R11, R15);
    e.store(R11, 0, B);
    e.mov(A, R13);
    e.add(A, 1);
    e.store(R11, 8, A);
    e.store(R11, 16, R9);
    e.mem(0x88, R8, R11, 18);
    e.mov(Di, R11);
    e.add(Di, 19);
    e.mov(C, D);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.rr(0x01, R15, R9);
    e.0.bind("named-directory-skip")?;
    e.add(R13, 1);
    e.0.jump("named-directory-next");
    e.0.bind("named-directory-no-fit")?;
    e.cmp(R15, 0);
    e.0.jump_if(0x84, "namespace-einval");
    e.0.bind("named-directory-publish")?;
    e.cmp(R15, 0);
    e.0.jump_if(0x84, "named-directory-commit");
    e.gs(0x8b, R8, 320);
    e.mov(R9, R15);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "namespace-efault");
    e.mov(Di, R8);
    e.mov(Si, R12);
    e.add(Si, files::DIRECTORY_STAGING);
    e.mov(C, R15);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.0.bind("named-directory-commit")?;
    e.store(R14, 24, R13);
    wipe_directory(e);
    wipe_staging(e);
    wipe_name(e);
    e.mov(A, R15);
    emit_static_process_sysret(e.0);
    e.0.bind("named-directory-dot-data")?;
    e.0.bytes(b".\0");
    e.0.bind("named-directory-parent-data")?;
    e.0.bytes(b"..\0");
    Ok(())
}
pub(super) fn emit_services(e: &mut E<'_>) -> Result<(), CompileError> {
    reopen(e)?;
    unlink(e)?;
    directory(e)?;
    Ok(())
}
