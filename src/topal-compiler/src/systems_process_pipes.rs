//! TK-DEC-256: shared FIFO backing outside private process snapshots.
use super::file_descriptions::{
    CHILD_DESCRIPTIONS, DESCRIPTIONS, STAGING, TABLE_CONTEXT, wipe_staging,
};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};
#[path = "systems_pipe_create.rs"]
mod create;
#[path = "systems_pipe_transfer.rs"]
mod transfer;
#[path = "systems_pipe_wait.rs"]
mod wait;

pub(super) const BASE: i32 = 0x3e000;
pub(super) const BUFFER: i32 = 0x3f000;
pub(super) const BYTES: u64 = 0x5000;
const RECORD: i32 = 128;
const COUNTER: i32 = BASE + 512;
const HEADER: i32 = 0x2000;
const CONTEXT: i32 = 0x2100;

fn buffer(e: &mut E<'_>, object: super::R) {
    e.mov(R11, object);
    e.rr(0x29, R11, R12);
    e.add(R11, -BASE);
    e.0.bytes(&[0x49, 0xc1, 0xeb, 7, 0x49, 0xc1, 0xe3, 12]);
    e.rr(0x01, R11, R12);
    e.add(R11, BUFFER);
}
fn erase(e: &mut E<'_>, base: super::R, bytes: u64) {
    for verify in [false, true] {
        e.mov(Di, base);
        e.imm(A, 0);
        e.imm(C, bytes / 8);
        e.0.bytes(if verify {
            &[0xfc, 0xf3, 0x48, 0xaf]
        } else {
            &[0xfc, 0xf3, 0x48, 0xab]
        });
        if verify {
            e.0.jump_if(0x85, "fatal");
        }
    }
}

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "pipe-dispatch-done");
    e.gs(0x8b, R11, 104);
    e.cmp(R11, 11);
    e.0.jump_if(0x82, "pipe-dispatch-done");
    for (number, label) in [(22, "pipe-create"), (293, "pipe-create2")] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.0.bind("pipe-dispatch-done")?;
    Ok(())
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    create::emit(e.0)?;
    transfer::emit(e.0)?;
    wait::emit(e.0)?;
    // R14 is a checked pipe description; return its exact owned object in R10.
    // Preserve user-transfer arguments, slot/alias pointers and release counters.
    e.0.bind("pipe-object-lookup")?;
    e.load(R11, R14, 32);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fatal");
    e.mov(R10, R12);
    e.add(R10, BASE);
    e.imm(C, 4);
    e.0.bind("pipe-object-next")?;
    e.load(A, R10, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "pipe-object-advance");
    e.load(A, R10, 8);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x84, "pipe-object-found");
    e.0.bind("pipe-object-advance")?;
    e.add(R10, RECORD);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, "pipe-object-next");
    e.0.jump("fatal");
    e.0.bind("pipe-object-found")?;
    e.load(A, R10, 32);
    e.cmp(A, 4096);
    e.0.jump_if(0x83, "fatal");
    e.load(A, R10, 40);
    e.cmp(A, 4096);
    e.0.jump_if(0x87, "fatal");
    e.mov(A, R14);
    e.rr(0x29, A, R12);
    e.cmp(A, CHILD_DESCRIPTIONS);
    e.0.jump_if(0x82, "pipe-object-primary-description");
    e.add(A, -CHILD_DESCRIPTIONS);
    e.0.bytes(&[0x48, 0xc1, 0xe8, 6]);
    e.add(A, 17);
    e.0.jump("pipe-object-description-ready");
    e.0.bind("pipe-object-primary-description")?;
    e.add(A, -DESCRIPTIONS);
    e.0.bytes(&[0x48, 0xc1, 0xe8, 6]);
    e.add(A, 1);
    e.0.bind("pipe-object-description-ready")?;
    e.load(R11, R14, 0);
    e.cmp(R11, 5);
    e.0.jump_if(0x84, "pipe-object-reader");
    e.cmp(R11, 6);
    e.0.jump_if(0x85, "fatal");
    e.load(C, R10, 24);
    e.0.jump("pipe-object-endpoint-ready");
    e.0.bind("pipe-object-reader")?;
    e.load(C, R10, 16);
    e.0.bind("pipe-object-endpoint-ready")?;
    e.rr(0x39, A, C);
    e.0.jump_if(0x85, "fatal");
    e.0.bytes(&[0xc3]);

    e.0.bind("pipe-stat")?;
    e.0.call("pipe-object-lookup");
    e.imm(A, 2);
    e.store(R12, STAGING, A); // anonymous device, distinct from boot root
    e.load(A, R10, 8);
    e.store(R12, STAGING + 8, A);
    e.imm(A, 0x1180);
    e.store(R12, STAGING + 24, A);
    for (source, target) in [(72, 72), (80, 88), (88, 104)] {
        e.load(A, R10, source);
        e.store(R12, STAGING + target, A);
    }
    e.0.jump("fd-stat-publish");
    e.0.bind("pipe-capacity")?;
    e.load(A, R14, 0);
    e.cmp(A, 5);
    e.0.jump_if(0x84, "pipe-capacity-valid");
    e.cmp(A, 6);
    e.0.jump_if(0x85, "fd-ebadf");
    e.0.bind("pipe-capacity-valid")?;
    e.0.call("pipe-object-lookup");
    e.imm(A, 4096);
    emit_static_process_sysret(e.0);
    e.0.bind("pipe-ioctl")?;
    e.0.bytes(&[0x89, 0xf6]);
    e.cmp(Si, 0x541b);
    e.0.jump_if(0x85, "fd-enotty");
    e.mov(R8, D);
    e.imm(R9, 4);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "pipe-efault");
    e.0.call("pipe-object-lookup");
    e.load(A, R10, 40);
    e.0.bytes(&[0x41, 0x89, 0x00]);
    e.imm(A, 0);
    emit_static_process_sysret(e.0);

    e.0.bind("pipe-last-endpoint")?;
    e.0.bytes(&[0x41, 0x55]); // preserve caller's slot pointer
    e.0.call("pipe-object-lookup");
    e.mov(R13, R10);
    e.load(A, R14, 0);
    e.cmp(A, 5);
    e.0.jump_if(0x84, "pipe-last-reader");
    e.imm(A, 0);
    e.store(R13, 24, A);
    e.0.jump("pipe-last-peer-published");
    e.0.bind("pipe-last-reader")?;
    e.imm(A, 0);
    e.store(R13, 16, A);
    e.0.bind("pipe-last-peer-published")?;
    e.0.call("pipe-notify");
    e.load(A, R13, 16);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "pipe-last-return");
    e.load(A, R13, 24);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "pipe-last-return");
    e.load(A, R13, 8);
    for offset in [HEADER + 104, HEADER + 120] {
        e.load(R11, R12, offset);
        e.rr(0x39, A, R11);
        e.0.jump_if(0x84, "fatal");
    }
    buffer(&mut e, R13);
    e.mov(R10, R11);
    erase(&mut e, R10, 4096);
    erase(&mut e, R13, RECORD as u64);
    e.0.bind("pipe-last-return")?;
    e.0.bytes(&[0x41, 0x5d, 0xc3]);

    e.0.bind("pipe-terminal-release")?;
    for (offset, bytes) in [(BASE, 512), (BASE + 520, 4096 - 520), (BUFFER, 4 * 4096)] {
        e.mov(Di, R12);
        e.add(Di, offset);
        e.imm(A, 0);
        e.imm(C, bytes / 8);
        e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
        e.0.jump_if(0x85, "fatal");
    }
    e.mov(R10, R12);
    e.add(R10, BASE);
    erase(&mut e, R10, BYTES);
    e.0.bytes(&[0xc3]);
    for (label, errno) in [
        ("pipe-enfile", 23),
        ("pipe-emfile", 24),
        ("pipe-efault", 14),
        ("pipe-einval", 22),
        ("pipe-enosys", 38),
        ("pipe-eio", 5),
        ("pipe-eagain", 11),
        ("pipe-epipe", 32),
    ] {
        e.0.bind(label)?;
        wipe_staging(&mut e);
        e.imm(A, (-errno as i64) as u64);
        emit_static_process_sysret(e.0);
    }
    Ok(())
}
