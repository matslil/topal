//! Shared clock borrowing and relative deadlines; TK-DEC-251.
//! TOPAL-SYSTEMS-STATIC-PROCESS-{MONOTONIC,RELATIVE-SLEEP}-001.
use super::super::monotonic_clock_now;
use super::R::{_Bp as Bp, _Sp as Sp};
use super::file_descriptions::{STAGING, TABLE_CONTEXT, wipe_staging};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};

pub(in super::super) const CLOCK_CONTEXT: u32 = 944;
const NATIVE_MAX: u64 = i64::MAX as u64;
const NATIVE_SEC_MAX: u64 = NATIVE_MAX / 1_000_000_000;
const LAST_TICK: u64 = NATIVE_MAX / 10 + 1;

pub(in super::super) fn emit_entry_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.load(A, Sp, 0x58); // twelfth provider argument, after its five saves
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.mov(R8, A);
    e.arith(4, R8, 7);
    e.cmp(R8, 0);
    e.0.jump_if(0x85, "fail");
    e.mov(R8, A);
    e.add(R8, 24);
    e.0.jump_if(0x82, "fail");
    e.cmp(R8, 0x4000_0000);
    e.0.jump_if(0x87, "fail");
    e.cmp(R8, 0x400000);
    e.0.jump_if(0x86, "process-time-owner-disjoint");
    e.cmp(A, 0x800000);
    e.0.jump_if(0x82, "fail");
    e.0.bind("process-time-owner-disjoint")?;
    // The checked root lends its fresh clock before ordinary source samples.
    // Only its 24-byte observation state is borrowed; deadline state is separate.
    for offset in [0, 8, 16] {
        e.load(R11, A, offset);
        e.cmp(R11, 0);
        e.0.jump_if(0x85, "fail");
    }
    e.store(R10, CLOCK_CONTEXT as i32, A);
    Ok(())
}

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "process-time-dispatch-end");
    e.gs(0x8b, R11, 104);
    e.arith(4, R11, 255);
    e.cmp(R11, 11);
    e.0.jump_if(0x82, "process-time-dispatch-end");
    for (number, label) in [
        (35, "process-time-nanosleep"),
        (228, "process-time-gettime"),
        (229, "process-time-getres"),
        (230, "process-time-clock-nanosleep"),
    ] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.0.bind("process-time-dispatch-end")?;
    Ok(())
}

fn begin(e: &mut E<'_>) {
    e.gs(0x8b, R12, TABLE_CONTEXT);
    wipe_staging(e);
}

fn check_output(e: &mut E<'_>) {
    e.mov(R8, B);
    e.imm(R9, 16);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "process-time-efault");
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("process-time-gettime")?;
    e.0.bytes(&[0x89, 0xff]); // native clockid_t
    e.cmp(Di, 1);
    e.0.jump_if(0x85, "process-time-einval");
    e.mov(B, Si);
    begin(&mut e);
    check_output(&mut e);
    e.0.call("process-time-sample");
    // Convert extended ticks / (100 million ticks per second). Guard DIV and
    // the signed native seconds range rather than truncate the clock epoch.
    e.cmp(D, 100_000_000);
    e.0.jump_if(0x83, "fatal");
    e.imm(R10, 100_000_000);
    e.0.bytes(&[0x49, 0xf7, 0xf2]);
    e.cmp(A, 0);
    e.0.jump_if(0x8c, "fatal");
    e.0.bytes(&[0x48, 0x6b, 0xd2, 10]);
    e.0.jump("process-time-publish");

    e.0.bind("process-time-getres")?;
    e.0.bytes(&[0x89, 0xff]);
    e.cmp(Di, 1);
    e.0.jump_if(0x85, "process-time-einval");
    e.cmp(Si, 0);
    e.0.jump_if(0x84, "process-time-success");
    e.mov(B, Si);
    begin(&mut e);
    check_output(&mut e);
    e.imm(A, 0);
    e.imm(D, 10);
    e.0.bind("process-time-publish")?;
    e.store(R12, STAGING, A);
    e.store(R12, STAGING + 8, D);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.mov(Di, B);
    e.imm(C, 2);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa5]);
    e.0.jump("process-time-success");

    e.0.bind("process-time-clock-nanosleep")?;
    e.0.bytes(&[0x89, 0xff, 0x89, 0xf6]);
    e.cmp(Di, 1); // only the two qualified relative interval clock views
    e.0.jump_if(0x87, "fatal");
    e.cmp(Si, 0);
    e.0.jump_if(0x85, "fatal");
    e.mov(B, D);
    e.0.jump("process-time-copy-interval");
    e.0.bind("process-time-nanosleep")?;
    e.mov(B, Di);
    e.0.bind("process-time-copy-interval")?;
    begin(&mut e);
    e.mov(R8, B);
    e.imm(R9, 16);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "process-time-efault");
    e.mov(Si, B);
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.imm(C, 2);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa5]);
    e.load(B, R12, STAGING);
    e.load(Bp, R12, STAGING + 8);
    e.cmp(B, 0);
    e.0.jump_if(0x8c, "process-time-einval");
    e.cmp(Bp, 1_000_000_000);
    e.0.jump_if(0x83, "process-time-einval");
    wipe_staging(&mut e);
    e.imm(R13, NATIVE_MAX);
    e.imm(A, NATIVE_SEC_MAX);
    e.rr(0x39, B, A);
    e.0.jump_if(0x83, "process-time-duration-ready");
    e.mov(A, B);
    e.imm(R10, 1_000_000_000);
    e.0.bytes(&[0x49, 0xf7, 0xe2]); // unsigned mul: RDX:RAX
    e.rr(0x01, A, Bp);
    e.mov(R13, A);
    e.0.bind("process-time-duration-ready")?;
    e.0.call("process-time-sample");
    e.imm(R14, LAST_TICK);
    e.cmp(D, 0);
    e.0.jump_if(0x85, "process-time-target-ready");
    e.imm(R10, NATIVE_MAX / 10);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x87, "process-time-target-ready");
    e.imm(R10, 10);
    e.0.bytes(&[0x49, 0xf7, 0xe2]);
    e.rr(0x01, A, R13);
    e.0.jump_if(0x82, "process-time-target-ready");
    e.cmp(A, 0);
    e.0.jump_if(0x8c, "process-time-target-ready");
    e.add(A, 9); // ceil native deadline to source resolution
    e.imm(D, 0);
    e.0.bytes(&[0x49, 0xf7, 0xf2]);
    e.mov(R14, A);
    e.0.bind("process-time-target-ready")?;
    e.0.bytes(&[0xf3, 0x90]); // PAUSE; a controlled timing gate observes this boundary
    e.0.call("process-time-sample");
    e.cmp(D, 0);
    e.0.jump_if(0x85, "process-time-success");
    e.rr(0x39, A, R14);
    e.0.jump_if(0x82, "process-time-target-ready");

    e.0.bind("process-time-success")?;
    begin(&mut e);
    e.imm(A, 0);
    emit_static_process_sysret(e.0);
    e.0.bind("process-time-efault")?;
    begin(&mut e);
    e.imm(A, (-14_i64) as u64);
    emit_static_process_sysret(e.0);
    e.0.bind("process-time-einval")?;
    begin(&mut e);
    e.imm(A, (-22_i64) as u64);
    emit_static_process_sysret(e.0);

    e.0.bind("process-time-sample")?;
    e.gs(0x8b, Di, CLOCK_CONTEXT);
    e.0.call("process-time-clock-source");
    e.cmp(C, 1);
    e.0.jump_if(0x85, "fatal");
    e.0.bytes(&[0xc3]);
    e.0.bind("process-time-clock-source")?;
    e.0.bytes(&monotonic_clock_now()?);
    Ok(())
}
