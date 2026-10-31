//! Actual RTC-backed epoch seconds; TOPAL-SYSTEMS-STATIC-PROCESS-REALTIME-SECONDS-001.
use super::file_descriptions::{STAGING, TABLE_CONTEXT, wipe_staging};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "realtime-dispatch-end");
    e.cmp(A, 201);
    e.0.jump_if(0x84, "realtime-seconds");
    e.0.bind("realtime-dispatch-end")?;
    Ok(())
}

fn rtc(e: &mut E<'_>, index: u8) {
    // The pinned RTC index port reads 0xff, not a saved index/NMI state.
    // The platform owns this indexed device exclusively; no date/control write.
    e.0.bytes(&[0xb0, index, 0xe6, 0x70, 0xe4, 0x71, 0x0f, 0xb6, 0xc0]);
}
fn multiply(e: &mut E<'_>, register: super::R, factor: i32) {
    let r = register as u8;
    e.0.bytes(&[
        0x48 | ((r >> 3) << 2) | (r >> 3),
        0x69,
        0xc0 | ((r & 7) << 3) | (r & 7),
    ]);
    e.0.bytes(&factor.to_le_bytes());
}
fn leap(year: u64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("realtime-seconds")?;
    e.gs(0x8b, R12, TABLE_CONTEXT);
    wipe_staging(&mut e);
    e.imm(R15, 1024);
    e.0.bind("realtime-sample-retry")?;
    e.cmp(R15, 0);
    e.0.jump_if(0x84, "realtime-eio");
    e.add(R15, -1);
    rtc(&mut e, 0x0a);
    e.arith(4, A, 128);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "realtime-sample-retry");
    let registers = [0, 2, 4, 7, 8, 9, 0x32, 0x0b];
    for (i, index) in registers.into_iter().enumerate() {
        rtc(&mut e, index);
        e.store(R12, STAGING + i as i32 * 8, A);
    }
    rtc(&mut e, 0x0a);
    e.arith(4, A, 128);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "realtime-sample-retry");
    for (i, index) in registers.into_iter().enumerate() {
        rtc(&mut e, index);
        e.load(R10, R12, STAGING + i as i32 * 8);
        e.rr(0x39, A, R10);
        e.0.jump_if(0x85, "realtime-sample-retry");
    }
    rtc(&mut e, 0x0a);
    e.arith(4, A, 128);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "realtime-sample-retry");
    e.load(B, R12, STAGING + 56);
    e.mov(A, B);
    e.arith(4, A, 0x82);
    e.cmp(A, 2);
    e.0.jump_if(0x85, "realtime-eio"); // require running 24-hour representation
    e.mov(A, B);
    e.arith(4, A, 4);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "realtime-decoded");
    e.mov(R13, R12);
    e.add(R13, STAGING);
    e.imm(R14, 7);
    e.0.bind("realtime-decode-bcd")?;
    e.load(A, R13, 0);
    e.mov(R10, A);
    e.arith(4, R10, 15);
    e.cmp(R10, 9);
    e.0.jump_if(0x87, "realtime-eio");
    e.0.bytes(&[0x48, 0xc1, 0xe8, 4]);
    e.cmp(A, 9);
    e.0.jump_if(0x87, "realtime-eio");
    multiply(&mut e, A, 10);
    e.rr(0x01, A, R10);
    e.store(R13, 0, A);
    e.add(R13, 8);
    e.add(R14, -1);
    e.cmp(R14, 0);
    e.0.jump_if(0x85, "realtime-decode-bcd");
    e.0.bind("realtime-decoded")?;
    for (offset, min, max) in [
        (0, 0, 59),
        (8, 0, 59),
        (16, 0, 23),
        (24, 1, 31),
        (32, 1, 12),
        (40, 0, 99),
        (48, 19, 23),
    ] {
        e.load(A, R12, STAGING + offset);
        e.cmp(A, min);
        e.0.jump_if(0x82, "realtime-eio");
        e.cmp(A, max);
        e.0.jump_if(0x87, "realtime-eio");
    }
    e.load(A, R12, STAGING + 48);
    multiply(&mut e, A, 100);
    e.load(B, R12, STAGING + 40);
    e.rr(0x01, A, B);
    e.cmp(A, 1970);
    e.0.jump_if(0x82, "realtime-eio");
    e.cmp(A, 2261);
    e.0.jump_if(0x87, "realtime-eio");
    e.add(A, -1970);
    e.0.bytes(&[0x48, 0xc1, 0xe0, 4]);
    e.0.rip_lea(&[0x4c, 0x8d, 0x2d], "realtime-year-table");
    e.rr(0x01, R13, A);
    e.load(R15, R13, 0);
    e.load(R14, R13, 8);
    e.load(B, R12, STAGING + 32);
    e.add(B, -1);
    e.0.bytes(&[0x48, 0xc1, 0xe3, 4]);
    e.0.rip_lea(&[0x4c, 0x8d, 0x2d], "realtime-month-table");
    e.rr(0x01, R13, B);
    e.load(A, R13, 0);
    e.load(C, R13, 8);
    e.cmp(R14, 0);
    e.0.jump_if(0x84, "realtime-month-ready");
    e.load(B, R12, STAGING + 32);
    e.cmp(B, 2);
    e.0.jump_if(0x82, "realtime-month-ready");
    e.0.jump_if(0x84, "realtime-february");
    e.add(A, 1);
    e.0.jump("realtime-month-ready");
    e.0.bind("realtime-february")?;
    e.add(C, 1);
    e.0.bind("realtime-month-ready")?;
    e.load(B, R12, STAGING + 24);
    e.rr(0x39, B, C);
    e.0.jump_if(0x87, "realtime-eio");
    e.add(B, -1);
    e.rr(0x01, R15, A);
    e.rr(0x01, R15, B);
    multiply(&mut e, R15, 86400);
    for (offset, factor) in [(16, 3600), (8, 60), (0, 1)] {
        e.load(A, R12, STAGING + offset);
        multiply(&mut e, A, factor);
        e.rr(0x01, R15, A);
    }
    // Native time samples before attempting its optional user publication.
    e.gs(0x8b, R8, 312);
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "realtime-success");
    e.imm(R9, 8);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "realtime-efault");
    e.store(R8, 0, R15);
    e.0.bind("realtime-success")?;
    wipe_staging(&mut e);
    e.mov(A, R15);
    emit_static_process_sysret(e.0);
    for (label, errno) in [("realtime-eio", 5), ("realtime-efault", 14)] {
        e.0.bind(label)?;
        wipe_staging(&mut e);
        e.imm(A, (-errno as i64) as u64);
        emit_static_process_sysret(e.0);
    }
    // Calendar constants are private immutable provider data, never a chosen
    // timestamp. The runtime RTC year/month select their corresponding entries.
    e.0.bind("realtime-year-table")?;
    let mut days = 0_u64;
    for year in 1970..2262 {
        e.0.bytes(&days.to_le_bytes());
        e.0.bytes(&u64::from(leap(year)).to_le_bytes());
        days += if leap(year) { 366 } else { 365 };
    }
    e.0.bind("realtime-month-table")?;
    let mut days = 0_u64;
    for length in [31_u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31] {
        e.0.bytes(&days.to_le_bytes());
        e.0.bytes(&length.to_le_bytes());
        days += length;
    }
    Ok(())
}
