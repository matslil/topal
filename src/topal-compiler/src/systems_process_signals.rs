//! Process-owned native registrations. No signal delivery is implied.
//! TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-{ACTION,MASK}-001; TK-DEC-241.

use super::file_descriptions::{STAGING, TABLE_CONTEXT, wipe_staging};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};

const ACTIONS: i32 = 1792;
const BLOCKED: i32 = 32;
const MASK: u64 = !0x40100_u64;

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    // Preserve raw syscall arguments until the selected service captures them.
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "process-signals-dispatch-end");
    e.cmp(A, 13);
    e.0.jump_if(0x84, "process-signal-action");
    e.cmp(A, 14);
    e.0.jump_if(0x84, "process-signal-mask");
    e.0.bind("process-signals-dispatch-end")?;
    Ok(())
}

fn prepare(e: &mut E<'_>) {
    e.gs(0x8b, R12, TABLE_CONTEXT);
    e.0.bytes(&[0x41, 0x89, 0xff]); // native int signal/how -> r15d
    e.mov(R14, D); // optional old-value destination
    wipe_staging(e);
    e.gs(0x8b, A, 336); // saved native sigsetsize
    e.cmp(A, 8);
    e.0.jump_if(0x85, "process-signal-einval");
    e.gs(0x8b, R8, 320); // saved native input pointer
}

fn copy_input(e: &mut E<'_>, size: u64) {
    e.imm(R9, size);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "process-signal-efault");
    e.mov(Si, R8);
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.imm(C, size);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
}

fn publish(e: &mut E<'_>, size: u64) {
    e.mov(R8, R14);
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "process-signal-success");
    e.imm(R9, size);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "process-signal-efault");
    e.mov(Di, R14);
    e.mov(Si, R12);
    e.add(Si, STAGING + 64);
    e.imm(C, size);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.0.jump("process-signal-success");
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("process-signal-action")?;
    prepare(&mut e);
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "process-signal-action-number");
    copy_input(&mut e, 32);
    e.0.bind("process-signal-action-number")?;
    e.cmp(R15, 1);
    e.0.jump_if(0x82, "process-signal-einval");
    e.cmp(R15, 64);
    e.0.jump_if(0x87, "process-signal-einval");
    e.mov(R13, R15);
    e.add(R13, -1);
    e.0.bytes(&[0x49, 0xc1, 0xe5, 5]);
    e.rr(0x01, R13, R12);
    e.add(R13, ACTIONS);
    e.gs(0x8b, A, 320);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "process-signal-action-capture");
    e.cmp(R15, 9);
    e.0.jump_if(0x84, "process-signal-einval");
    e.cmp(R15, 19);
    e.0.jump_if(0x84, "process-signal-einval");
    e.0.bind("process-signal-action-capture")?;
    for offset in (0..32).step_by(8) {
        e.load(A, R13, offset);
        e.store(R12, STAGING + 64 + offset, A);
    }
    e.gs(0x8b, A, 320);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "process-signal-action-publish");
    e.load(A, R12, STAGING + 8);
    e.imm(R11, 0xdc00_0807);
    e.rr(0x21, A, R11);
    e.store(R12, STAGING + 8, A);
    e.load(A, R12, STAGING + 24);
    e.imm(R11, MASK);
    e.rr(0x21, A, R11);
    e.store(R12, STAGING + 24, A);
    for offset in (0..32).step_by(8) {
        e.load(A, R12, STAGING + offset);
        e.store(R13, offset, A);
    }
    e.0.bind("process-signal-action-publish")?;
    publish(&mut e, 32);

    e.0.bind("process-signal-mask")?;
    prepare(&mut e);
    e.load(A, R12, BLOCKED);
    e.store(R12, STAGING + 64, A);
    e.cmp(R8, 0);
    e.0.jump_if(0x84, "process-signal-mask-publish");
    copy_input(&mut e, 8);
    e.load(A, R12, STAGING);
    e.imm(R11, MASK);
    e.rr(0x21, A, R11);
    e.cmp(R15, 0);
    e.0.jump_if(0x84, "process-signal-mask-block");
    e.cmp(R15, 1);
    e.0.jump_if(0x84, "process-signal-mask-unblock");
    e.cmp(R15, 2);
    e.0.jump_if(0x85, "process-signal-einval");
    e.0.jump("process-signal-mask-commit");
    e.0.bind("process-signal-mask-block")?;
    e.load(R11, R12, BLOCKED);
    e.rr(0x09, A, R11);
    e.0.jump("process-signal-mask-commit");
    e.0.bind("process-signal-mask-unblock")?;
    e.imm(R11, u64::MAX);
    e.rr(0x31, A, R11); // ~new
    e.load(R11, R12, BLOCKED);
    e.rr(0x21, A, R11);
    e.0.bind("process-signal-mask-commit")?;
    e.store(R12, BLOCKED, A);
    e.0.bind("process-signal-mask-publish")?;
    publish(&mut e, 8);
    for (label, result) in [
        ("process-signal-success", 0),
        ("process-signal-einval", -22_i64),
        ("process-signal-efault", -14),
    ] {
        e.0.bind(label)?;
        wipe_staging(&mut e);
        e.imm(A, result as u64);
        emit_static_process_sysret(e.0);
    }

    // Import the prelude's validated representation, then consume its authority.
    e.0.bind("process-signals-import")?;
    e.gs(0x8b, A, 417);
    e.arith(4, A, 255);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    let quit = ACTIONS + 2 * 32;
    e.imm(A, 1);
    e.store(R12, quit, A);
    e.imm(A, 0x0400_0000);
    e.store(R12, quit + 8, A);
    e.gs(0x8b, A, 416);
    e.arith(4, A, 255);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "process-signals-import-external");
    e.imm(A, 0x004a_6580);
    e.0.jump("process-signals-import-restorer");
    e.0.bind("process-signals-import-external")?;
    e.cmp(A, 2);
    e.0.jump_if(0x85, "fatal");
    e.imm(A, 0x004a_9740);
    e.0.bind("process-signals-import-restorer")?;
    e.store(R12, quit + 16, A);
    e.imm(A, 0xffff_fffe_7fff_ffff & MASK);
    e.store(R12, quit + 24, A);
    for (offset, context) in [(0, 168), (8, 176), (16, 224), (24, 232)] {
        e.gs(0x8b, A, context);
        if offset == 24 {
            e.imm(R11, MASK);
            e.rr(0x21, A, R11);
        }
        e.store(R12, ACTIONS + 16 * 32 + offset, A);
        e.imm(A, 0);
        e.gs(0x89, A, context);
    }
    e.0.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    e.0.bytes(&417_u32.to_le_bytes());
    e.0.bytes(&[0, 0xc3]);

    // Tail-called by exec filtering: retain SIG_IGN alone, clear all metadata.
    e.0.bind("process-signals-exec-reset")?;
    e.mov(R13, R12);
    e.add(R13, ACTIONS);
    e.imm(R15, 64);
    e.0.bind("process-signals-exec-action")?;
    e.load(A, R13, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "process-signals-exec-ignored");
    e.imm(A, 0);
    e.store(R13, 0, A);
    e.0.bind("process-signals-exec-ignored")?;
    e.imm(A, 0);
    for offset in [8, 16, 24] {
        e.store(R13, offset, A);
    }
    e.add(R13, 32);
    e.add(R15, -1);
    e.cmp(R15, 0);
    e.0.jump_if(0x85, "process-signals-exec-action");
    e.0.bytes(&[0xc3]);
    Ok(())
}
