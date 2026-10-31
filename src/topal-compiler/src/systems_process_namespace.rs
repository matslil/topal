//! Immutable object selection and independent descriptions (TK-DEC-242).
//! Native pointers exist only inside this closed x86 provider implementation.
use super::R::_Bp as Bp;
use super::file_descriptions::{DESCRIPTIONS, SLOTS, STAGING, TABLE_CONTEXT, wipe_staging};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};

const PATH: i32 = 4096;
const CWD: i32 = 40;

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "namespace-dispatch-end");
    for (number, label) in [
        (2, "namespace-open"),
        (257, "namespace-openat"),
        (262, "namespace-stat-at"),
        (217, "namespace-getdents"),
        (79, "namespace-getcwd"),
        (80, "namespace-chdir"),
        (81, "namespace-fchdir"),
    ] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.0.bind("namespace-dispatch-end")?;
    Ok(())
}

fn table(e: &mut E<'_>) {
    e.gs(0x8b, R12, TABLE_CONTEXT);
}
fn byte(e: &mut E<'_>, dst: super::R, base: super::R, offset: i32) {
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
fn wipe_name(e: &mut E<'_>) {
    e.mov(Di, R12);
    e.add(Di, PATH);
    e.imm(A, 0);
    e.imm(C, 512);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    e.mov(Di, R12);
    e.add(Di, PATH);
    e.imm(C, 512);
    e.0.bytes(&[0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
}
fn finish(e: &mut E<'_>, value: i64) {
    wipe_name(e);
    e.imm(A, value as u64);
    emit_static_process_sysret(e.0);
}

fn emit_copy_and_walk(e: &mut E<'_>) -> Result<(), CompileError> {
    // R8 candidate; B dirfd and BP flags survive. R13 is copied length.
    e.0.bind("namespace-copy-name")?;
    table(e);
    wipe_name(e);
    e.imm(R13, 0);
    e.mov(Si, R12);
    e.add(Si, PATH);
    e.0.bind("namespace-copy-byte")?;
    e.imm(R9, 1);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "namespace-efault");
    byte(e, A, R8, 0);
    e.mem(0x88, A, Si, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-copy-complete");
    e.add(R13, 1);
    e.cmp(R13, 4096);
    e.0.jump_if(0x84, "namespace-enametoolong");
    e.add(R8, 1);
    e.0.jump_if(0x82, "namespace-efault");
    e.add(Si, 1);
    e.0.jump("namespace-copy-byte");
    e.0.bind("namespace-copy-complete")?;
    e.0.bytes(&[0xc3]);

    // Return R15 object. Only private copied bytes are traversed below.
    e.0.bind("namespace-walk")?;
    e.mov(R13, R12);
    e.add(R13, PATH);
    byte(e, A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-enoent");
    e.cmp(A, 47);
    e.0.jump_if(0x84, "namespace-walk-root");
    e.0.bytes(&[0x89, 0xdb]); // native int dirfd
    e.0.bytes(&[0x81, 0xfb, 0x9c, 0xff, 0xff, 0xff]);
    e.0.jump_if(0x84, "namespace-walk-cwd");
    e.mov(Di, B);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "namespace-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "namespace-enotdir");
    e.load(R15, R14, 32);
    e.mov(R13, R12);
    e.add(R13, PATH);
    e.0.jump("namespace-component");
    e.0.bind("namespace-walk-cwd")?;
    e.load(R15, R12, CWD);
    e.0.jump("namespace-component");
    e.0.bind("namespace-walk-root")?;
    e.imm(R15, 3);
    e.0.bind("namespace-component")?;
    byte(e, A, R13, 0);
    e.cmp(A, 47);
    e.0.jump_if(0x85, "namespace-component-start");
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "namespace-enotdir");
    e.add(R13, 1);
    e.0.jump("namespace-component");
    e.0.bind("namespace-component-start")?;
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-walk-complete");
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "namespace-enotdir");
    e.mov(Si, R13);
    e.imm(D, 0);
    e.0.bind("namespace-component-byte")?;
    byte(e, A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-component-end");
    e.cmp(A, 47);
    e.0.jump_if(0x84, "namespace-component-end");
    e.add(D, 1);
    e.cmp(D, 255);
    e.0.jump_if(0x87, "namespace-enametoolong");
    e.add(R13, 1);
    e.0.jump("namespace-component-byte");
    e.0.bind("namespace-component-end")?;
    e.cmp(D, 1);
    e.0.jump_if(0x85, "namespace-check-parent");
    byte(e, A, Si, 0);
    e.cmp(A, 46);
    e.0.jump_if(0x84, "namespace-component");
    e.0.bind("namespace-check-parent")?;
    e.cmp(D, 2);
    e.0.jump_if(0x85, "namespace-check-named");
    byte(e, A, Si, 0);
    e.cmp(A, 46);
    e.0.jump_if(0x85, "namespace-check-named");
    byte(e, A, Si, 1);
    e.cmp(A, 46);
    e.0.jump_if(0x85, "namespace-check-named");
    e.imm(R15, 3);
    e.0.jump("namespace-component");
    e.0.bind("namespace-check-named")?;
    e.cmp(R15, 3);
    e.0.jump_if(0x85, "namespace-check-busybox");
    e.cmp(D, 3);
    e.0.jump_if(0x85, "namespace-enoent");
    for (offset, ch) in b"bin".iter().enumerate() {
        byte(e, A, Si, offset as i32);
        e.cmp(A, i32::from(*ch));
        e.0.jump_if(0x85, "namespace-enoent");
    }
    e.imm(R15, 4);
    e.0.jump("namespace-component");
    e.0.bind("namespace-check-busybox")?;
    e.cmp(R15, 4);
    e.0.jump_if(0x85, "fatal");
    e.cmp(D, 7);
    e.0.jump_if(0x85, "namespace-enoent");
    for (offset, ch) in b"busybox".iter().enumerate() {
        byte(e, A, Si, offset as i32);
        e.cmp(A, i32::from(*ch));
        e.0.jump_if(0x85, "namespace-enoent");
    }
    e.imm(R15, 5);
    e.0.jump("namespace-component");
    e.0.bind("namespace-walk-complete")?;
    e.0.bytes(&[0xc3]);
    Ok(())
}

fn emit_open(e: &mut E<'_>, source_bound: bool) -> Result<(), CompileError> {
    e.0.bind("namespace-open")?;
    e.mov(R8, Di);
    e.mov(Bp, Si);
    e.imm(B, 0xffff_ff9c);
    e.0.jump("namespace-open-common");
    e.0.bind("namespace-openat")?;
    e.mov(R8, Si);
    e.mov(Bp, D);
    e.mov(B, Di);
    e.0.bind("namespace-open-common")?;
    // Native flags request access; no writable authority follows from them.
    e.0.bytes(&[0x89, 0xed]);
    e.mov(A, Bp);
    e.arith(4, A, !0x000b_8903);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.mov(A, Bp);
    e.arith(4, A, 3);
    e.cmp(A, 3); // nonstandard access-mode-three remains unqualified
    e.0.jump_if(0x84, "fatal");
    e.0.call("namespace-copy-name");
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "namespace-enoent");
    e.imm(R14, 0);
    e.mov(R10, R12);
    e.add(R10, SLOTS);
    e.0.bind("namespace-free-slot")?;
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-slot-ready");
    e.add(R14, 1);
    e.cmp(R14, 16);
    e.0.jump_if(0x84, "namespace-emfile");
    e.add(R10, 16);
    e.0.jump("namespace-free-slot");
    e.0.bind("namespace-slot-ready")?;
    e.0.bytes(&[0x41, 0x56]);
    e.0.call("namespace-walk");
    e.0.bytes(&[0x41, 0x5e]);
    if source_bound {
        super::policy::emit_open_decision(e)?;
    } else {
        e.mov(A, Bp);
        e.arith(4, A, 0x10000);
        e.cmp(A, 0);
        e.0.jump_if(0x84, "namespace-open-type-ready");
        e.cmp(R15, 5);
        e.0.jump_if(0x84, "namespace-enotdir");
        e.0.bind("namespace-open-type-ready")?;
        e.mov(A, Bp);
        e.arith(4, A, 3);
        e.cmp(A, 0);
        e.0.jump_if(0x84, "namespace-open-access-ready");
        e.cmp(R15, 5);
        e.0.jump_if(0x85, "namespace-eisdir");
        // The retained member belongs to the prepared immutable boot-root view.
        // Lookup/type errors precede this filesystem-level access denial.
        e.0.jump("namespace-erofs");
        e.0.bind("namespace-open-access-ready")?;
    }
    // Two sixteen-slot processes can retain at most thirty-two descriptions.
    // A free active slot therefore implies a free cell across BOTH banks.
    e.mov(R10, R12);
    e.add(R10, DESCRIPTIONS);
    e.imm(R11, 0);
    e.0.bind("namespace-free-description")?;
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-description-ready");
    e.add(R11, 1);
    e.cmp(R11, 32);
    e.0.jump_if(0x84, "fatal");
    e.add(R10, 64);
    e.cmp(R11, 16);
    e.0.jump_if(0x85, "namespace-free-description");
    e.mov(R10, R12);
    e.add(R10, super::file_descriptions::CHILD_DESCRIPTIONS);
    e.0.jump("namespace-free-description");
    e.0.bind("namespace-description-ready")?;
    e.imm(A, 4);
    e.cmp(R15, 5);
    e.0.jump_if(0x85, "namespace-description-kind");
    e.imm(A, 3);
    e.0.bind("namespace-description-kind")?;
    e.store(R10, 0, A);
    e.imm(A, 1);
    e.store(R10, 8, A);
    e.store(R10, 32, R15);
    e.mov(A, Bp);
    e.arith(4, A, !0x80100);
    e.arith(1, A, 0x8000);
    e.store(R10, 16, A);
    e.mov(R13, R14);
    e.0.bytes(&[0x49, 0xc1, 0xe5, 4]);
    e.rr(0x01, R13, R12);
    e.add(R13, SLOTS);
    e.add(R11, 1);
    e.store(R13, 0, R11);
    e.mov(A, Bp);
    e.0.bytes(&[0x48, 0xc1, 0xe8, 19]);
    e.arith(4, A, 1);
    e.store(R13, 8, A);
    wipe_name(e);
    e.mov(A, R14);
    emit_static_process_sysret(e.0);
    Ok(())
}

fn emit_metadata(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-stat-at")?;
    e.mov(B, Di);
    e.mov(R8, Si);
    e.mov(Bp, R10);
    e.0.bytes(&[0x89, 0xed]);
    table(e);
    // AT_EMPTY_PATH admits a null pointer, and turns a copied empty name into
    // descriptor observation before the vfs_statx flag checks for nonnegative fd.
    e.mov(A, Bp);
    e.arith(4, A, 0x1000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-stat-copy");
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "namespace-stat-empty");
    e.0.bind("namespace-stat-copy")?;
    e.0.call("namespace-copy-name");
    e.cmp(R13, 0);
    e.0.jump_if(0x85, "namespace-stat-flags");
    e.mov(A, Bp);
    e.arith(4, A, 0x1000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-enoent");
    e.0.bind("namespace-stat-empty")?;
    e.0.bytes(&[0x89, 0xdb]);
    e.cmp(B, 0x7fffffff);
    e.0.jump_if(0x87, "namespace-stat-empty-cwd");
    wipe_name(e);
    e.mov(Di, B);
    e.gs(0x8b, Si, 328);
    e.0.jump("fd-stat");
    e.0.bind("namespace-stat-empty-cwd")?;
    e.0.bytes(&[0x81, 0xfb, 0x9c, 0xff, 0xff, 0xff]);
    e.0.jump_if(0x85, "namespace-ebadf");
    e.mov(A, Bp);
    e.arith(4, A, !0x7900);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-einval");
    e.load(R15, R12, CWD);
    e.0.jump("namespace-stat-object");
    e.0.bind("namespace-stat-flags")?;
    e.mov(A, Bp);
    e.arith(4, A, !0x7900);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-einval");
    e.0.call("namespace-walk");
    e.0.bind("namespace-stat-object")?;
    wipe_name(e);
    e.gs(0x8b, R8, 328);
    e.imm(R9, 144);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "namespace-efault");
    wipe_staging(e);
    e.0.call("namespace-stat-value");
    e.0.jump("fd-stat-publish-to-r8");

    // R15 is immutable object identity; staging was cleared by the caller.
    e.0.bind("namespace-stat-value")?;
    e.imm(A, 1);
    e.store(R12, STAGING, A);
    e.store(R12, STAGING + 16, A);
    e.store(R12, STAGING + 8, R15);
    e.imm(A, 4096);
    e.store(R12, STAGING + 56, A);
    e.cmp(R15, 5);
    e.0.jump_if(0x85, "namespace-stat-directory");
    for (offset, value) in [(24, 0o100755), (48, 2_024_544), (64, 3955)] {
        e.imm(A, value);
        e.store(R12, STAGING + offset, A);
    }
    e.0.bytes(&[0xc3]);
    e.0.bind("namespace-stat-directory")?;
    e.imm(A, 0o40755);
    e.store(R12, STAGING + 24, A);
    e.imm(A, 2);
    e.cmp(R15, 4);
    e.0.jump_if(0x84, "namespace-stat-directory-links");
    e.cmp(R15, 3);
    e.0.jump_if(0x85, "fatal");
    e.imm(A, 3);
    e.0.bind("namespace-stat-directory-links")?;
    e.store(R12, STAGING + 16, A);
    e.0.bytes(&[0xc3]);
    Ok(())
}

fn emit_cwd(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-chdir")?;
    e.mov(R8, Di);
    e.imm(B, 0xffff_ff9c);
    e.0.call("namespace-copy-name");
    e.0.call("namespace-walk");
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "namespace-enotdir");
    e.store(R12, CWD, R15);
    finish(e, 0);
    e.0.bind("namespace-fchdir")?;
    table(e);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "namespace-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "namespace-enotdir");
    e.load(A, R14, 32);
    e.store(R12, CWD, A);
    finish(e, 0);
    e.0.bind("namespace-getcwd")?;
    table(e);
    e.mov(R8, Di);
    e.load(A, R12, CWD);
    e.imm(R15, 2);
    e.cmp(A, 3);
    e.0.jump_if(0x84, "namespace-cwd-length");
    e.cmp(A, 4);
    e.0.jump_if(0x85, "fatal");
    e.imm(R15, 5);
    e.0.bind("namespace-cwd-length")?;
    e.rr(0x39, Si, R15);
    e.0.jump_if(0x82, "namespace-erange");
    e.mov(R9, R15);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "namespace-efault");
    wipe_staging(e);
    e.imm(A, u64::from_le_bytes(*b"/bin\0\0\0\0"));
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "namespace-cwd-bytes");
    e.imm(A, 47);
    e.0.bind("namespace-cwd-bytes")?;
    e.store(R12, STAGING, A);
    e.mov(Di, R8);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.mov(C, R15);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    wipe_staging(e);
    wipe_name(e);
    e.mov(A, R15);
    emit_static_process_sysret(e.0);
    Ok(())
}

pub(in super::super) fn emit_services(
    code: &mut X86FunctionEncoder,
    source_bound: bool,
) -> Result<(), CompileError> {
    let mut e = E(code);
    emit_copy_and_walk(&mut e)?;
    emit_open(&mut e, source_bound)?;
    emit_metadata(&mut e)?;
    emit_cwd(&mut e)?;
    // Directory encoding is emitted by its separate, narrowly scoped helper.
    emit_directory(&mut e)?;
    for (name, errno) in [
        ("namespace-efault", 14),
        ("namespace-enoent", 2),
        ("namespace-enametoolong", 36),
        ("namespace-ebadf", 9),
        ("namespace-enotdir", 20),
        ("namespace-eisdir", 21),
        ("namespace-erofs", 30),
        ("namespace-emfile", 24),
        ("namespace-einval", 22),
        ("namespace-erange", 34),
    ] {
        e.0.bind(name)?;
        table(&mut e);
        wipe_staging(&mut e);
        finish(&mut e, -errno);
    }
    Ok(())
}

fn emit_directory(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-getdents")?;
    table(e);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "namespace-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "namespace-enotdir");
    e.0.bytes(&[0x89, 0xd5]); // unsigned native count -> ebp
    e.load(R13, R14, 24);
    e.load(B, R14, 32);
    e.imm(R15, 0);
    wipe_staging(e);
    e.0.bind("namespace-directory-record")?;
    e.cmp(R13, 3);
    e.0.jump_if(0x83, "namespace-directory-publish");
    e.imm(R9, 24);
    e.cmp(R13, 2);
    e.0.jump_if(0x85, "namespace-directory-length");
    e.cmp(B, 4);
    e.0.jump_if(0x85, "namespace-directory-length");
    e.imm(R9, 32);
    e.0.bind("namespace-directory-length")?;
    e.mov(A, R15);
    e.rr(0x01, A, R9);
    e.rr(0x39, A, Bp);
    e.0.jump_if(0x87, "namespace-directory-publish");
    e.mov(R10, R12);
    e.add(R10, STAGING);
    e.rr(0x01, R10, R15);
    e.mov(A, B);
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "namespace-directory-inode");
    e.imm(A, 3);
    e.cmp(R13, 1);
    e.0.jump_if(0x84, "namespace-directory-inode");
    e.mov(A, B);
    e.add(A, 1);
    e.0.bind("namespace-directory-inode")?;
    e.store(R10, 0, A);
    e.mov(A, R13);
    e.add(A, 1);
    e.store(R10, 8, A);
    e.cmp(R13, 0);
    e.0.jump_if(0x85, "namespace-directory-parent");
    e.imm(A, u64::from_le_bytes([24, 0, 4, b'.', 0, 0, 0, 0]));
    e.0.jump("namespace-directory-name");
    e.0.bind("namespace-directory-parent")?;
    e.cmp(R13, 1);
    e.0.jump_if(0x85, "namespace-directory-child");
    e.imm(A, u64::from_le_bytes([24, 0, 4, b'.', b'.', 0, 0, 0]));
    e.0.jump("namespace-directory-name");
    e.0.bind("namespace-directory-child")?;
    e.cmp(B, 4);
    e.0.jump_if(0x84, "namespace-directory-busybox");
    e.imm(A, u64::from_le_bytes([24, 0, 4, b'b', b'i', b'n', 0, 0]));
    e.0.jump("namespace-directory-name");
    e.0.bind("namespace-directory-busybox")?;
    e.imm(A, u64::from_le_bytes([b'o', b'x', 0, 0, 0, 0, 0, 0]));
    e.store(R10, 24, A);
    e.imm(
        A,
        u64::from_le_bytes([32, 0, 8, b'b', b'u', b's', b'y', b'b']),
    );
    e.0.bind("namespace-directory-name")?;
    e.store(R10, 16, A);
    e.rr(0x01, R15, R9);
    e.add(R13, 1);
    e.0.jump("namespace-directory-record");
    e.0.bind("namespace-directory-publish")?;
    e.cmp(R15, 0);
    e.0.jump_if(0x85, "namespace-directory-copy");
    e.cmp(R13, 3);
    e.0.jump_if(0x82, "namespace-einval");
    finish(e, 0);
    e.0.bind("namespace-directory-copy")?;
    e.gs(0x8b, R8, 320);
    e.mov(R9, R15);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "namespace-efault");
    e.mov(Di, R8);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.mov(C, R15);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.store(R14, 24, R13);
    wipe_staging(e);
    wipe_name(e);
    e.mov(A, R15);
    emit_static_process_sysret(e.0);
    Ok(())
}
