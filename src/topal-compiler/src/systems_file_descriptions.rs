//! Native realization of TK-DEC-240's owned descriptor/description table.
//! All pointers in this module are private provider implementation details.

use super::super::{
    X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET, emit_initial_boot_resource_handoff_validation,
    emit_static_process_sysret,
};
use super::R::_Sp as Sp;
use super::{CompileError, E, R::*, X86FunctionEncoder};

pub(in super::super) const TABLE_CONTEXT: u32 = 928;
pub(in super::super) const ARCHIVE_CONTEXT: u32 = 936;
pub(super) const SLOTS: i32 = 64;
pub(super) const DESCRIPTIONS: i32 = 320;
pub(in super::super) const TABLE_BYTES: u64 = 8192;
pub(super) const STAGING: i32 = 1408;
const FILE_SIZE: u64 = 2_024_544;

/// The two new private arguments follow context, heap and pristine snapshot.
/// Entry still has its five-register provider prologue on the kernel stack.
pub(in super::super) fn emit_entry_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.load(A, Sp, 0x48); // tenth System V argument: table backing
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.mov(R8, A);
    e.arith(4, R8, 4095);
    e.cmp(R8, 0);
    e.0.jump_if(0x85, "fail");
    // The active user hierarchy overrides identity mappings in [4MiB,8MiB).
    // Both private table and borrowed archive must retain supervisor identity
    // addresses outside that interval during user-copy service execution.
    e.mov(R8, A);
    e.add(R8, TABLE_BYTES as i32);
    e.0.jump_if(0x82, "fail");
    e.cmp(R8, 0x4000_0000);
    e.0.jump_if(0x87, "fail");
    e.cmp(R8, 0x0040_0000);
    e.0.jump_if(0x86, "fd-table-identity-disjoint");
    e.cmp(A, 0x0080_0000);
    e.0.jump_if(0x82, "fail");
    e.0.bind("fd-table-identity-disjoint")?;
    e.store(R10, TABLE_CONTEXT as i32, A);
    e.mov(Di, A);
    e.imm(A, 0);
    e.imm(C, TABLE_BYTES / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fail");
    e.load(Si, Sp, 0x50); // eleventh argument: retained boot handoff
    e.cmp(Si, 0);
    e.0.jump_if(0x84, "fail");
    e.cmp(Si, 0x00ff_f000);
    e.0.jump_if(0x87, "fail");
    emit_initial_boot_resource_handoff_validation(e.0)?;
    e.cmp(R9, 0x0040_0000);
    e.0.jump_if(0x86, "fd-archive-identity-disjoint");
    e.cmp(R8, 0x0080_0000);
    e.0.jump_if(0x82, "fail");
    e.0.bind("fd-archive-identity-disjoint")?;
    e.store(R10, ARCHIVE_CONTEXT as i32, R8);
    Ok(())
}

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "fd-dispatch-complete");
    for (number, label) in [
        (0, "fd-read"),
        (1, "fd-write"),
        (3, "fd-close"),
        (5, "fd-stat"),
        (8, "fd-seek"),
        (16, "fd-ioctl"),
        (32, "fd-dup"),
        (33, "fd-dup2"),
        (72, "fd-fcntl"),
        (292, "fd-dup3"),
    ] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.cmp(A, 302);
    e.0.jump_if(0x85, "fd-check-getrlimit");
    e.0.bytes(&[0x83, 0xfe, 7]);
    e.0.jump_if(0x84, "fd-prlimit-query");
    e.0.jump("fd-dispatch-complete");
    e.0.bind("fd-check-getrlimit")?;
    e.cmp(A, 97);
    e.0.jump_if(0x85, "fd-dispatch-complete");
    e.0.bytes(&[0x83, 0xff, 7]);
    e.0.jump_if(0x84, "fd-getrlimit-query");
    e.0.bind("fd-dispatch-complete")?;
    Ok(())
}

fn table(e: &mut E<'_>) {
    e.gs(0x8b, R12, TABLE_CONTEXT);
}
fn slot(e: &mut E<'_>, index: super::R) {
    e.mov(R13, index);
    e.0.bytes(&[0x49, 0xc1, 0xe5, 4]);
    e.rr(0x01, R13, R12);
    e.add(R13, SLOTS);
}
fn return_value(e: &mut E<'_>) {
    emit_static_process_sysret(e.0);
}
pub(super) fn wipe_staging(e: &mut E<'_>) {
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.imm(A, 0);
    e.imm(C, 32);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.imm(C, 32);
    e.0.bytes(&[0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
}

pub(in super::super) fn emit_initialize(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("fd-table-initialize")?;
    table(&mut e);
    e.load(A, R12, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.gs(0x8b, A, 737);
    e.arith(4, A, 255);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.imm(A, 16);
    e.store(R12, 8, A);
    e.gs(0x8b, A, ARCHIVE_CONTEXT);
    e.add(A, X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET as i32);
    e.store(R12, 16, A);
    e.imm(A, FILE_SIZE);
    e.store(R12, 24, A);
    e.imm(A, 3);
    e.store(R12, 40, A); // cwd object: root
    e.imm(A, 1);
    e.store(R12, 48, A); // borrowed prepared-root identity witness
    e.imm(A, 5);
    e.store(R12, DESCRIPTIONS + 3 * 64 + 32, A);
    for index in 0..4_i32 {
        e.imm(A, index as u64 + 1);
        e.store(R12, SLOTS + 16 * index, A);
        let description = DESCRIPTIONS + 64 * index;
        e.imm(
            A,
            if index == 0 {
                1
            } else if index == 3 {
                3
            } else {
                2
            },
        );
        e.store(R12, description, A); // serial input / console output / file
        e.imm(A, 1);
        e.store(R12, description + 8, A); // reference count
        e.imm(
            A,
            if index == 3 {
                0x8000
            } else if index == 0 {
                0
            } else {
                1
            },
        );
        e.store(R12, description + 16, A); // native open status/access flags
    }
    // The previous single-description owner is consumed by this transfer.
    e.0.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    e.0.bytes(&737_u32.to_le_bytes());
    e.0.bytes(&[0]);
    e.0.call("process-signals-import");
    e.imm(A, 1);
    e.store(R12, 0, A);
    e.0.bytes(&[0xc3]);
    Ok(())
}

fn emit_lookup_and_release(e: &mut E<'_>) -> Result<(), CompileError> {
    // RDI is the native descriptor; return R12 table, R13 slot, R14 description.
    // CF distinguishes invalid/closed descriptors without user-memory access.
    e.0.bind("fd-lookup")?;
    e.0.bytes(&[0x89, 0xff]); // unsigned int fd
    e.cmp(Di, 16);
    e.0.jump_if(0x83, "fd-lookup-invalid");
    table(e);
    slot(e, Di);
    e.load(A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fd-lookup-invalid");
    e.cmp(A, 16);
    e.0.jump_if(0x87, "fatal");
    e.add(A, -1);
    e.0.bytes(&[0x48, 0xc1, 0xe0, 6]);
    e.mov(R14, R12);
    e.add(R14, DESCRIPTIONS);
    e.rr(0x01, R14, A);
    e.load(A, R14, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.load(A, R14, 8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.0.bytes(&[0xf8, 0xc3]);
    e.0.bind("fd-lookup-invalid")?;
    e.0.bytes(&[0xf9, 0xc3]);
    e.0.bind("fd-drop-reference")?;
    e.load(A, R14, 8);
    e.add(A, -1);
    e.store(R14, 8, A);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fd-drop-slot");
    for offset in (0..64).step_by(8) {
        e.store(R14, offset, A);
    }
    e.0.bind("fd-drop-slot")?;
    e.imm(A, 0);
    e.store(R13, 0, A);
    e.store(R13, 8, A);
    e.0.bytes(&[0xc3]);
    e.0.bind("fd-close")?;
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.0.call("fd-drop-reference");
    e.imm(A, 0);
    return_value(e);
    Ok(())
}

fn emit_duplication(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("fd-dup")?;
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.imm(R8, 0);
    e.imm(R9, 0);
    e.0.jump("fd-allocate-alias");
    e.0.bind("fd-fcntl")?;
    e.0.bytes(&[0x89, 0xf6]);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    for (command, label) in [
        (0, "fd-fcntl-dup"),
        (1030, "fd-fcntl-cloexec-dup"),
        (1, "fd-getfd"),
        (2, "fd-setfd"),
        (3, "fd-getfl"),
    ] {
        e.cmp(Si, command);
        e.0.jump_if(0x84, label);
    }
    e.0.jump("fd-einval");
    e.0.bind("fd-getfd")?;
    e.load(A, R13, 8);
    return_value(e);
    e.0.bind("fd-setfd")?;
    e.mov(A, D);
    e.arith(4, A, 1);
    e.store(R13, 8, A);
    e.imm(A, 0);
    return_value(e);
    e.0.bind("fd-getfl")?;
    e.load(A, R14, 16);
    return_value(e);
    e.0.bind("fd-fcntl-dup")?;
    e.imm(R9, 0);
    e.0.jump("fd-fcntl-dup-ready");
    e.0.bind("fd-fcntl-cloexec-dup")?;
    e.imm(R9, 1);
    e.0.bind("fd-fcntl-dup-ready")?;
    e.0.bytes(&[0x41, 0x89, 0xd0]); // native int lower bound -> r8d
    e.cmp(R8, 16);
    e.0.jump_if(0x83, "fd-einval");
    e.0.bind("fd-allocate-alias")?;
    e.mov(R15, R14);
    e.0.bind("fd-find-free-slot")?;
    e.cmp(R8, 16);
    e.0.jump_if(0x83, "fd-emfile");
    slot(e, R8);
    e.load(A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fd-install-alias");
    e.add(R8, 1);
    e.0.jump("fd-find-free-slot");
    e.0.bind("fd-dup2")?;
    e.0.bytes(&[0x89, 0xff, 0x89, 0xf6]);
    e.rr(0x39, Di, Si);
    e.0.jump_if(0x85, "fd-dup2-distinct");
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.mov(A, Di);
    return_value(e);
    e.0.bind("fd-dup2-distinct")?;
    e.imm(R9, 0);
    e.0.jump("fd-replace-alias");
    e.0.bind("fd-dup3")?;
    e.0.bytes(&[0x89, 0xff, 0x89, 0xf6, 0x89, 0xd2]);
    e.mov(A, D);
    e.arith(4, A, !0x80000);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fd-einval");
    e.rr(0x39, Di, Si);
    e.0.jump_if(0x84, "fd-einval");
    e.mov(R9, D);
    e.0.bytes(&[0x49, 0xc1, 0xe9, 19]);
    e.0.bind("fd-replace-alias")?;
    e.cmp(Si, 16);
    e.0.jump_if(0x83, "fd-ebadf");
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.mov(R15, R14);
    e.mov(R8, Si);
    e.mov(Di, R8);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-install-alias");
    e.0.call("fd-drop-reference");
    e.0.bind("fd-install-alias")?;
    e.mov(A, R15);
    e.rr(0x29, A, R12);
    e.add(A, -DESCRIPTIONS);
    e.0.bytes(&[0x48, 0xc1, 0xe8, 6]);
    e.add(A, 1);
    e.store(R13, 0, A);
    e.store(R13, 8, R9);
    e.load(A, R15, 8);
    e.add(A, 1);
    e.store(R15, 8, A);
    e.mov(A, R8);
    return_value(e);
    Ok(())
}

pub(in super::super) fn emit_lifetime_services(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    emit_lookup_and_release(&mut e)?;
    emit_duplication(&mut e)?;
    for (name, errno) in [
        ("fd-ebadf", 9),
        ("fd-einval", 22),
        ("fd-emfile", 24),
        ("fd-efault", 14),
        ("fd-espipe", 29),
        ("fd-enotty", 25),
        ("fd-esrch", 3),
        ("fd-eisdir", 21),
        ("fd-enotdir", 20),
    ] {
        e.0.bind(name)?;
        e.imm(A, (-errno as i64) as u64);
        return_value(&mut e);
    }
    Ok(())
}

pub(in super::super) fn emit_transfer_services(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    // Numerical access_ok-style range admission precedes a zero/EOF result.
    // Live page permissions are checked separately for the bytes transferred.
    e.0.bind("fd-native-range")?;
    e.mov(A, R8);
    e.rr(0x01, A, R9);
    e.0.jump_if(0x82, "fd-native-range-invalid");
    e.imm(R10, 0x0000_8000_0000_0000);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x87, "fd-native-range-invalid");
    e.0.bytes(&[0xf8, 0xc3]);
    e.0.bind("fd-native-range-invalid")?;
    e.0.bytes(&[0xf9, 0xc3]);

    e.0.bind("fd-write")?;
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, 2);
    e.0.jump_if(0x85, "fd-ebadf");
    e.0.jump("runtime-console-payload");

    e.0.bind("fd-read")?;
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.load(B, R14, 0);
    e.cmp(B, 2);
    e.0.jump_if(0x84, "fd-ebadf");
    e.mov(R8, Si);
    e.mov(R9, D);
    e.0.call("fd-native-range");
    e.0.jump_if(0x82, "fd-efault");
    e.cmp(B, 4);
    e.0.jump_if(0x84, "fd-eisdir");
    e.cmp(R9, 0);
    e.0.jump_if(0x84, "fd-read-zero");
    e.cmp(B, 1);
    e.0.jump_if(0x84, "fd-serial-read");
    e.cmp(B, 3);
    e.0.jump_if(0x85, "fd-eisdir");
    e.load(R15, R14, 24);
    e.load(A, R12, 24);
    e.rr(0x39, R15, A);
    e.0.jump_if(0x83, "fd-read-zero");
    e.rr(0x29, A, R15); // remaining bytes
    e.rr(0x39, R9, A);
    e.0.jump_if(0x86, "fd-read-remaining-ready");
    e.mov(R9, A);
    e.0.bind("fd-read-remaining-ready")?;
    e.cmp(R9, 256);
    e.0.jump_if(0x86, "fd-read-count-ready");
    e.imm(R9, 256);
    e.0.bind("fd-read-count-ready")?;
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "fd-efault");
    wipe_staging(&mut e);
    e.load(Si, R12, 16);
    e.rr(0x01, Si, R15);
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.mov(C, R9);
    e.0.bytes(&[0xf3, 0xa4]);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.gs(0x8b, Di, 320);
    e.mov(C, R9);
    e.0.bytes(&[0xf3, 0xa4]);
    e.rr(0x01, R15, R9);
    e.store(R14, 24, R15);
    wipe_staging(&mut e);
    e.mov(A, R9);
    return_value(&mut e);
    e.0.bind("fd-read-zero")?;
    e.imm(A, 0);
    return_value(&mut e);

    e.0.bind("fd-serial-read")?;
    e.cmp(R9, 256);
    e.0.jump_if(0x86, "fd-serial-count-ready");
    e.imm(R9, 256);
    e.0.bind("fd-serial-count-ready")?;
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "fd-efault");
    wipe_staging(&mut e);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.imm(R15, 0);
    e.0.bind("fd-serial-first-byte")?;
    e.0.bytes(&[0x66, 0xba, 0xfd, 3, 0xec, 0xa8, 1]);
    e.0.jump_if(0x84, "fd-serial-first-byte");
    e.0.bind("fd-serial-next-byte")?;
    e.0.bytes(&[0x66, 0xba, 0xf8, 3, 0xec, 0x88, 0x06]);
    e.add(Si, 1);
    e.add(R15, 1);
    e.rr(0x39, R15, R9);
    e.0.jump_if(0x83, "fd-serial-publish");
    e.0.bytes(&[0x66, 0xba, 0xfd, 3, 0xec, 0xa8, 1]);
    e.0.jump_if(0x85, "fd-serial-next-byte");
    e.0.bind("fd-serial-publish")?;
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.gs(0x8b, Di, 320);
    e.mov(C, R15);
    e.0.bytes(&[0xf3, 0xa4]);
    wipe_staging(&mut e);
    e.mov(A, R15);
    return_value(&mut e);

    e.0.bind("fd-seek")?;
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, 3);
    e.0.jump_if(0x84, "fd-seek-kind-ready");
    e.cmp(A, 4);
    e.0.jump_if(0x85, "fd-espipe");
    e.0.bind("fd-seek-kind-ready")?;
    e.0.bytes(&[0x89, 0xd2]);
    e.cmp(D, 0);
    e.0.jump_if(0x84, "fd-seek-set");
    e.cmp(D, 1);
    e.0.jump_if(0x84, "fd-seek-current");
    e.cmp(D, 2);
    e.0.jump_if(0x85, "fd-einval");
    e.load(A, R14, 0);
    e.cmp(A, 4);
    e.0.jump_if(0x84, "fd-seek-set");
    e.load(A, R12, 24);
    e.0.jump("fd-seek-add");
    e.0.bind("fd-seek-set")?;
    e.imm(A, 0);
    e.0.jump("fd-seek-add");
    e.0.bind("fd-seek-current")?;
    e.load(A, R14, 24);
    e.0.bind("fd-seek-add")?;
    e.rr(0x01, A, Si);
    e.0.jump_if(0x80, "fd-einval"); // signed overflow
    e.cmp(A, 0);
    e.0.jump_if(0x8c, "fd-einval");
    e.store(R14, 24, A);
    return_value(&mut e);

    e.0.bind("fd-stat")?;
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.mov(R8, Si);
    e.imm(R9, 144);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "fd-efault");
    wipe_staging(&mut e);
    e.imm(A, 1);
    e.store(R12, STAGING, A);
    e.store(R12, STAGING + 8, A);
    e.store(R12, STAGING + 16, A);
    e.imm(A, 4096);
    e.store(R12, STAGING + 56, A);
    e.load(A, R14, 0);
    e.cmp(A, 3);
    e.0.jump_if(0x84, "fd-stat-file");
    e.cmp(A, 4);
    e.0.jump_if(0x84, "fd-stat-file");
    e.imm(A, 0x2180);
    e.store(R12, STAGING + 24, A);
    e.imm(A, 0x4400);
    e.store(R12, STAGING + 40, A);
    e.0.jump("fd-stat-publish");
    e.0.bind("fd-stat-file")?;
    e.load(R15, R14, 32);
    e.0.call("namespace-stat-value");
    e.0.bind("fd-stat-publish")?;
    e.0.bind("fd-stat-publish-to-r8")?;
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.mov(Di, R8);
    e.imm(C, 144);
    e.0.bytes(&[0xf3, 0xa4]);
    wipe_staging(&mut e);
    e.imm(A, 0);
    return_value(&mut e);

    e.0.bind("fd-ioctl")?;
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fd-ebadf");
    e.0.jump("fd-enotty");

    e.0.bind("fd-prlimit-query")?;
    e.0.bytes(&[0x89, 0xff]);
    e.cmp(Di, 1);
    e.0.jump_if(0x87, "fd-esrch");
    e.cmp(D, 0);
    e.0.jump_if(0x85, "fd-einval");
    e.gs(0x8b, R8, 336);
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "fd-limit-no-output");
    e.0.jump("fd-limit-publish");
    e.0.bind("fd-getrlimit-query")?;
    e.mov(R8, Si);
    e.0.bind("fd-limit-publish")?;
    e.imm(R9, 16);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "fd-efault");
    e.imm(A, 16);
    e.store(R8, 0, A);
    e.store(R8, 8, A);
    e.0.bind("fd-limit-no-output")?;
    e.imm(A, 0);
    return_value(&mut e);
    Ok(())
}

pub(in super::super) fn emit_reclamation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("fd-exec-filter")?;
    e.imm(B, 1);
    e.0.jump("fd-release-slots");
    e.0.bind("fd-terminal-release")?;
    e.imm(B, 0);
    e.0.bind("fd-release-slots")?;
    table(&mut e);
    e.imm(R8, 0);
    e.0.bind("fd-release-slot")?;
    slot(&mut e, R8);
    e.load(A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fd-release-next");
    e.cmp(B, 0);
    e.0.jump_if(0x84, "fd-release-live-slot");
    e.load(A, R13, 8);
    e.arith(4, A, 1);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fd-release-next");
    e.0.bind("fd-release-live-slot")?;
    e.mov(Di, R8);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "fatal");
    e.0.call("fd-drop-reference");
    e.0.bind("fd-release-next")?;
    e.add(R8, 1);
    e.cmp(R8, 16);
    e.0.jump_if(0x82, "fd-release-slot");
    e.cmp(B, 0);
    e.0.jump_if(0x85, "process-signals-exec-reset");
    // All slot and description state must be gone before erasing the header.
    e.mov(Di, R12);
    e.add(Di, SLOTS);
    e.imm(A, 0);
    e.imm(C, ((DESCRIPTIONS + 16 * 64 - SLOTS) / 8) as u64);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
    e.mov(Di, R12);
    e.imm(C, TABLE_BYTES / 8);
    e.0.bytes(&[0xf3, 0x48, 0xab]);
    e.mov(Di, R12);
    e.imm(C, TABLE_BYTES / 8);
    e.0.bytes(&[0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
    e.0.bind("fd-release-return")?;
    e.0.bytes(&[0xc3]);
    Ok(())
}
