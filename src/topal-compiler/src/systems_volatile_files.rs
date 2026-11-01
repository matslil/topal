//! TK-DEC-265: supervisor-owned sparse RAM filesystem, outside process snapshots.
//! TOPAL-SYSTEMS-VOLATILE-FILE-OWNERSHIP-001.
use super::R::_Sp as Sp;
use super::file_descriptions::{STAGING, TABLE_CONTEXT, wipe_staging};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};
#[path = "systems_volatile_create.rs"]
mod create;
#[path = "systems_volatile_transfer.rs"]
mod transfer;

pub(super) const BASE: i32 = 0x43000;
pub(super) const BYTES: u64 = 0x9000;
pub(super) const OBJECTS: i32 = BASE + 256;
pub(super) const PAGES: i32 = BASE + 1024;
pub(super) const BUFFER: i32 = BASE + 4096;
// Private process bytes after the transfer staging and before signal actions.
// In particular, table+32 already belongs to the blocked signal mask.
pub(super) const UMASK: i32 = STAGING + 256;
pub(super) const KIND: i32 = 7;
// One call to file-clock, twelve saved GPRs and its RTC observation call.
const MAXIMUM_HELPER_STACK: u64 = 8 + 12 * 8 + 8;

const _: () = {
    assert!(UMASK >= STAGING + 256);
    assert!(UMASK + 8 <= super::signals::ACTIONS);
    assert!(UMASK >= super::signals::BLOCKED + 8);
    assert!(BASE as u64 >= super::pipes::BASE as u64 + super::pipes::BYTES);
    assert!(OBJECTS + 4 * 128 <= PAGES);
    assert!(PAGES + 8 * 32 <= BUFFER);
    assert!(BUFFER as u64 + 8 * 4096 == BASE as u64 + BYTES);
};

pub(in super::super) fn profile() -> crate::SystemsVolatileFileProfile {
    crate::SystemsVolatileFileProfile {
        schema: "topal.volatile-file.x86_64/1".into(),
        process_storage_symbol: super::super::X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL
            .into(),
        process_storage_bytes: super::children::STORAGE_BYTES,
        filesystem_offset: BASE as u64,
        filesystem_bytes: BYTES,
        object_offset: (OBJECTS - BASE) as u64,
        object_count: 4,
        object_bytes: 128,
        page_metadata_offset: (PAGES - BASE) as u64,
        page_metadata_bytes: 32,
        resident_pages: 8,
        page_bytes: 4096,
        payload_offset: (BUFFER - BASE) as u64,
        private_umask_offset: UMASK as u64,
        initial_umask: 0o022,
        transfer_staging_offset: STAGING as u64,
        transfer_quantum: 256,
        maximum_file_offset: i64::MAX as u64,
        maximum_native_helper_stack_bytes: MAXIMUM_HELPER_STACK,
        timestamp_policy: "realtime-seconds-noatime".into(),
    }
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
fn shift(e: &mut E<'_>, register: super::R, left: bool, count: u8) {
    let r = register as u8;
    e.0.bytes(&[
        0x48 | (r >> 3),
        0xc1,
        (if left { 0xe0 } else { 0xe8 }) | (r & 7),
        count,
    ]);
}
/// Linux rw_verify_area checks the full request before clipping/EOF/append.
pub(super) fn verify_native_range(
    e: &mut E<'_>,
    position: super::R,
    count: super::R,
    failure: &str,
) {
    e.cmp(count, 0);
    e.0.jump_if(0x8c, failure);
    e.mov(A, position);
    e.rr(0x01, A, count);
    e.0.jump_if(0x80, failure);
}
fn payload(e: &mut E<'_>, destination: super::R, index: super::R) {
    e.mov(destination, index);
    shift(e, destination, true, 12);
    e.rr(0x01, destination, R12);
    e.add(destination, BUFFER);
}

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "file-dispatch-done");
    for (number, label) in [(77, "file-truncate"), (95, "file-umask")] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.0.bind("file-dispatch-done")?;
    Ok(())
}

pub(in super::super) fn emit_initialize(code: &mut X86FunctionEncoder) {
    let mut e = E(code);
    e.imm(A, 7);
    e.store(R12, BASE, A);
    e.imm(A, 0o022);
    e.store(R12, UMASK, A);
}

fn emit_lookup(e: &mut E<'_>) -> Result<(), CompileError> {
    // Sampling uses shared staging before a transfer has copied any payload.
    // Preserve all service state; return seconds in A and carry after unwinding.
    e.0.bind("file-clock")?;
    e.cmp(Sp, 0x108000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(Sp, 0x180000);
    e.0.jump_if(0x87, "fatal");
    e.0.bytes(&[0x9c, 0x58]);
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.0.bytes(&[
        0x53, 0x55, 0x57, 0x56, 0x52, 0x41, 0x50, 0x41, 0x51, 0x41, 0x52, 0x41, 0x53, 0x41, 0x55,
        0x41, 0x56, 0x41, 0x57,
    ]);
    e.0.call("realtime-observe");
    e.0.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5b, 0x41, 0x5a, 0x41, 0x59, 0x41, 0x58, 0x5a,
        0x5e, 0x5f, 0x5d, 0x5b,
    ]);
    e.0.bytes(&[0xc3]);
    // Checked R14 description -> owned R10 object. No user access. Preserve
    // transfer arguments, release counters, R12/R13/R14 and callee-save state.
    e.0.bind("file-object-lookup")?;
    e.load(A, R14, 0);
    e.cmp(A, KIND);
    e.0.jump_if(0x85, "fatal");
    e.mov(R10, R12);
    e.add(R10, OBJECTS);
    e.imm(C, 4);
    e.0.bind("file-object-next")?;
    e.load(A, R10, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "file-object-advance");
    e.load(A, R14, 32);
    e.mem(0x3b, A, R10, 8);
    e.0.jump_if(0x84, "file-object-found");
    e.0.bind("file-object-advance")?;
    e.add(R10, 128);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, "file-object-next");
    e.0.jump("fatal");
    e.0.bind("file-object-found")?;
    e.load(A, R10, 8);
    e.cmp(A, 7);
    e.0.jump_if(0x82, "fatal");
    e.load(A, R10, 16);
    e.cmp(A, 0);
    e.0.jump_if(0x8c, "fatal");
    e.load(A, R14, 24);
    e.cmp(A, 0);
    e.0.jump_if(0x8c, "fatal");
    e.0.bytes(&[0xc3]);
    // Object R10 and position R15 -> R11 association and C physical slot.
    // Only A/C/R11/flags clobbered; CF denotes absence, never fabricated data.
    e.0.bind("file-page-lookup")?;
    e.mov(R11, R12);
    e.add(R11, PAGES);
    e.imm(C, 0);
    e.0.bind("file-page-next")?;
    e.load(A, R11, 0);
    e.mem(0x3b, A, R10, 8);
    e.0.jump_if(0x85, "file-page-advance");
    e.mov(A, R15);
    shift(e, A, false, 12);
    e.mem(0x3b, A, R11, 8);
    e.0.jump_if(0x84, "file-page-found");
    e.0.bind("file-page-advance")?;
    e.add(R11, 32);
    e.add(C, 1);
    e.cmp(C, 8);
    e.0.jump_if(0x82, "file-page-next");
    e.imm(R11, 0);
    e.0.bytes(&[0xf9, 0xc3]);
    e.0.bind("file-page-found")?;
    e.0.bytes(&[0xf8, 0xc3]);
    Ok(())
}

fn emit_lifetime(e: &mut E<'_>) -> Result<(), CompileError> {
    // Last-description drop preserves slot/ref-loop state, including B/R8.
    e.0.bind("file-last-description")?;
    e.0.bytes(&[0x41, 0x50, 0x41, 0x51, 0x41, 0x52, 0x41, 0x53, 0x41, 0x57]);
    e.0.call("file-object-lookup");
    e.imm(R8, 0);
    e.mov(R9, R12);
    e.add(R9, PAGES);
    e.0.bind("file-release-page")?;
    e.load(A, R9, 0);
    e.mem(0x3b, A, R10, 8);
    e.0.jump_if(0x85, "file-release-page-next");
    payload(e, R11, R8);
    erase(e, R11, 4096);
    erase(e, R9, 32);
    e.0.bind("file-release-page-next")?;
    e.add(R9, 32);
    e.add(R8, 1);
    e.cmp(R8, 8);
    e.0.jump_if(0x82, "file-release-page");
    erase(e, R10, 128);
    e.0.bytes(&[
        0x41, 0x5f, 0x41, 0x5b, 0x41, 0x5a, 0x41, 0x59, 0x41, 0x58, 0xc3,
    ]);
    e.0.bind("file-terminal-release")?;
    e.mov(Di, R12);
    e.add(Di, BASE + 8);
    e.imm(A, 0);
    e.imm(C, (BYTES - 8) / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
    e.mov(R11, R12);
    e.add(R11, BASE);
    erase(e, R11, BYTES);
    e.0.bytes(&[0xc3]);
    Ok(())
}

pub(in super::super) fn emit_services(
    code: &mut X86FunctionEncoder,
    file_open_plan: bool,
) -> Result<(), CompileError> {
    let mut e = E(code);
    emit_lookup(&mut e)?;
    emit_lifetime(&mut e)?;
    transfer::emit(e.0)?;
    create::emit(e.0, file_open_plan)?;
    e.0.bind("file-umask")?;
    e.gs(0x8b, R12, TABLE_CONTEXT);
    e.load(R15, R12, UMASK);
    e.mov(A, Di);
    e.arith(4, A, 0o777);
    e.store(R12, UMASK, A);
    e.mov(A, R15);
    emit_static_process_sysret(e.0);
    for (name, errno) in [
        ("file-ebadf", 9),
        ("file-einval", 22),
        ("file-enxio", 6),
        ("file-enospc", 28),
        ("file-efbig", 27),
        ("file-efault", 14),
        ("file-eio", 5),
    ] {
        e.0.bind(name)?;
        e.gs(0x8b, R12, TABLE_CONTEXT);
        wipe_staging(&mut e);
        e.imm(A, (-errno as i64) as u64);
        emit_static_process_sysret(e.0);
    }
    Ok(())
}
