//! TK-DEC-254: owned parent parking and a distinct serialized static child.
//! All offsets are private ABI/backing details, never source-visible addresses.
use super::file_descriptions::{DESCRIPTIONS, TABLE_CONTEXT};
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};
#[path = "systems_process_pair.rs"]
mod pair;

pub(in super::super) const STORAGE_BYTES: u64 = 0x3e000;
const HEADER: i32 = 0x2000;
const CONTEXT: i32 = 0x2100;
const SIMD: i32 = 0x2800;
const TABLE: i32 = 0x3000;
const LOW_LEAVES: i32 = 0x5000;
const HIGH_LEAVES: i32 = 0x6000;
const DATA: i32 = 0x7000;
const HEAP: i32 = 0x19000;
const STACK: i32 = 0x3b000;

fn copy(e: &mut E<'_>, source: super::R, from: i32, destination: super::R, to: i32, bytes: u64) {
    for verify in [false, true] {
        e.mov(Si, source);
        e.add(Si, from);
        e.mov(Di, destination);
        e.add(Di, to);
        e.imm(C, bytes);
        e.0.bytes(if verify {
            &[0xfc, 0xf3, 0xa6]
        } else {
            &[0xfc, 0xf3, 0xa4]
        });
        if verify {
            e.0.jump_if(0x85, "fatal");
        }
    }
}
fn erase(e: &mut E<'_>, from: i32, bytes: u64) {
    for verify in [false, true] {
        e.mov(Di, R15);
        e.add(Di, from);
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

fn read_msr(e: &mut E<'_>, msr: u32) {
    e.imm(C, u64::from(msr));
    e.0.bytes(&[0x0f, 0x32, 0x48, 0xc1, 0xe2, 0x20]);
    e.rr(0x09, A, D);
}
pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "child-dispatch-done");
    e.gs(0x8b, R11, 104);
    e.cmp(R11, 11);
    e.0.jump_if(0x82, "child-dispatch-done");
    e.cmp(R11, 15);
    e.0.jump_if(0x87, "child-dispatch-done");
    for (number, label) in [
        (57, "child-fork"),
        (56, "child-clone"),
        (61, "child-wait"),
        (218, "child-register-tid"),
        (273, "child-register-robust"),
        (158, "child-thread-local"),
        (24, "pair-yield"),
    ] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.0.bind("child-dispatch-done")?;
    Ok(())
}

/// R15 names the expanded process backing; validate each owned identity alias.
fn backing(e: &mut E<'_>, index: usize) -> Result<(), CompileError> {
    e.imm(A, 0x000f_ffff_ffff_f000);
    e.rr(0x21, R11, A);
    e.cmp(R11, 0x200000);
    e.0.jump_if(0x82, "fatal");
    e.mov(A, R11);
    e.add(A, if index == 0 { 0x12000 } else { 4096 });
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 0x40000000);
    e.0.jump_if(0x87, "fatal");
    let label = [
        "child-data-disjoint",
        "child-stack-low-disjoint",
        "child-stack-high-disjoint",
    ][index];
    e.cmp(A, 0x400000);
    e.0.jump_if(0x86, label);
    e.cmp(R11, 0x800000);
    e.0.jump_if(0x82, "fatal");
    e.0.bind(label)?;
    Ok(())
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("child-register-tid")?;
    e.gs(0x89, Di, 152);
    e.gs(0x8b, R12, TABLE_CONTEXT);
    e.load(A, R12, 56);
    emit_static_process_sysret(e.0);
    e.0.bind("child-register-robust")?;
    e.cmp(Si, 24);
    e.0.jump_if(0x85, "child-einval");
    e.gs(0x89, Di, 160);
    e.imm(A, 0);
    emit_static_process_sysret(e.0);
    e.0.bind("child-thread-local")?;
    e.0.bytes(&[0x89, 0xff]); // native int operation selector
    e.cmp(Di, 0x1002);
    e.0.jump_if(0x84, "child-set-fs");
    e.cmp(Di, 0x1003);
    e.0.jump_if(0x85, "child-einval");
    e.mov(R8, Si);
    e.imm(R9, 8);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "child-efault");
    read_msr(&mut e, 0xc0000100);
    e.store(R8, 0, A);
    e.imm(A, 0);
    emit_static_process_sysret(e.0);
    e.0.bind("child-set-fs")?;
    e.imm(A, 0x0000_8000_0000_0000);
    e.rr(0x39, Si, A);
    e.0.jump_if(0x83, "child-eperm");
    e.gs(0x89, Si, 128);
    e.mov(A, Si);
    e.mov(D, A);
    e.0.bytes(&[0x48, 0xc1, 0xea, 0x20]);
    e.imm(C, 0xc0000100);
    e.0.bytes(&[0x0f, 0x30]);
    e.imm(A, 0);
    emit_static_process_sysret(e.0);

    e.0.bind("child-clone")?;
    e.mov(R8, Di);
    e.0.bytes(&[0x45, 0x89, 0xc0]); // lower_32_bits(flags)
    e.mov(A, R8);
    e.arith(4, A, !0x01200000);
    e.cmp(A, 17);
    e.0.jump_if(0x85, "child-enosys");
    e.cmp(Si, 0);
    e.0.jump_if(0x85, "child-enosys");
    e.gs(0x8b, R9, 336);
    e.0.jump("child-create");
    e.0.bind("child-fork")?;
    e.imm(R8, 17);
    e.imm(R9, 0);
    e.0.bind("child-create")?;
    e.gs(0x8b, R15, TABLE_CONTEXT);
    e.load(A, R15, HEADER);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "child-eagain");
    e.load(A, R15, 56);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.gs(0x8b, A, 296);
    e.cmp(A, 0x401000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 0x58c285);
    e.0.jump_if(0x83, "fatal");
    e.gs(0x8b, A, 408);
    e.cmp(A, 0x7fe000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 0x800000);
    e.0.jump_if(0x87, "fatal");
    // No abandoned snapshot or authority may be reused.
    e.mov(Di, R15);
    e.add(Di, CONTEXT);
    e.imm(A, 0);
    e.imm(
        C,
        (super::file_descriptions::CHILD_DESCRIPTIONS as u64 - CONTEXT as u64) / 8,
    );
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
    e.store(R15, HEADER + 32, R8);
    e.store(R15, HEADER + 40, R9);
    e.load(R14, R15, HEADER + 8);
    e.cmp(R14, 0);
    e.0.jump_if(0x85, "child-pid-ready");
    e.imm(R14, 2);
    e.0.bind("child-pid-ready")?;
    e.cmp(R14, 2);
    e.0.jump_if(0x82, "fatal");
    e.cmp(R14, i32::MAX);
    e.0.jump_if(0x87, "fatal");
    e.store(R15, HEADER + 16, R14);
    e.store(R15, HEADER + 88, R14); // inactive parent's pending fork result
    e.mov(A, R14);
    e.add(A, 1);
    e.cmp(R14, i32::MAX);
    e.0.jump_if(0x85, "child-pid-next");
    e.imm(A, 2);
    e.0.bind("child-pid-next")?;
    e.store(R15, HEADER + 8, A);
    e.gs(0x8b, R11, 256);
    copy(&mut e, R11, 0, R15, CONTEXT, 1024);
    // The entropy service is shared history, never a restorable old key.
    for offset in [184, 192, 200, 208, 216, 264] {
        e.imm(A, 0);
        e.store(R15, CONTEXT + offset, A);
    }
    e.0.bytes(&[0x49, 0x0f, 0xae, 0x87]);
    e.0.bytes(&SIMD.to_le_bytes()); // fxsave64
    read_msr(&mut e, 0xc0000100);
    e.store(R15, HEADER + 72, A);
    copy(&mut e, R15, 0, R15, TABLE, 8192);
    e.gs(0x8b, R11, 112);
    copy(&mut e, R11, 0, R15, LOW_LEAVES, 4096);
    e.gs(0x8b, R11, 24);
    copy(&mut e, R11, 0x4000, R15, HIGH_LEAVES, 4096);
    // The exact writable ELF role is eighteen contiguous owned frames.
    e.gs(0x8b, R11, 112);
    e.load(R11, R11, 0xf28);
    backing(&mut e, 0)?;
    e.store(R15, HEADER + 48, R11);
    copy(&mut e, R11, 0, R15, DATA, 0x12000);
    e.gs(0x8b, R11, 120);
    copy(&mut e, R11, 0, R15, HEAP, 0x22000);
    for (index, leaf, target) in [(1, 0x4ff0, STACK), (2, 0x4ff8, STACK + 4096)] {
        e.gs(0x8b, R11, 24);
        e.load(R11, R11, leaf);
        backing(&mut e, index)?;
        e.store(R15, HEADER + 48 + index as i32 * 8, R11);
        copy(&mut e, R11, 0, R15, target, 4096);
    }
    // Each inactive parent slot still owns a reference to its shared description.
    e.imm(R8, 0);
    e.0.bind("child-retain-slot")?;
    e.mov(R13, R8);
    e.0.bytes(&[0x49, 0xc1, 0xe5, 4]);
    e.rr(0x01, R13, R15);
    e.add(R13, 64);
    e.load(A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "child-retain-next");
    e.cmp(A, 32);
    e.0.jump_if(0x87, "fatal");
    e.mov(R11, R15);
    e.add(R11, DESCRIPTIONS);
    e.cmp(A, 16);
    e.0.jump_if(0x86, "child-retain-description-base");
    e.add(
        R11,
        super::file_descriptions::CHILD_DESCRIPTIONS - DESCRIPTIONS - 16 * 64,
    );
    e.0.bind("child-retain-description-base")?;
    e.add(A, -1);
    e.0.bytes(&[0x48, 0xc1, 0xe0, 6]);
    e.rr(0x01, A, R11);
    e.load(R11, A, 0);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fatal");
    e.load(R11, A, 8);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fatal");
    e.add(R11, 1);
    e.cmp(R11, 32);
    e.0.jump_if(0x87, "fatal");
    e.store(A, 8, R11);
    e.0.bind("child-retain-next")?;
    e.add(R8, 1);
    e.cmp(R8, 16);
    e.0.jump_if(0x82, "child-retain-slot");
    e.store(R15, 56, R14);
    e.imm(A, 1);
    e.store(R15, HEADER, A);
    e.imm(A, 0);
    e.gs(0x89, A, 152);
    e.gs(0x89, A, 160);
    e.load(A, R15, HEADER + 32);
    e.arith(4, A, 0x200000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "child-clear-registration-ready");
    e.load(A, R15, HEADER + 40);
    e.gs(0x89, A, 152);
    e.0.bind("child-clear-registration-ready")?;
    e.load(A, R15, HEADER + 32);
    e.arith(4, A, 0x1000000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "child-created");
    e.load(R8, R15, HEADER + 40);
    e.imm(R9, 4);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "child-created");
    e.0.bytes(&[0x45, 0x89, 0x30]); // [r8]=r14d
    e.0.bind("child-created")?;
    e.imm(A, 0);
    emit_static_process_sysret(e.0);

    e.0.bind("child-exit")?;
    e.gs(0x8b, R15, TABLE_CONTEXT);
    e.load(A, R15, HEADER);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R15, HEADER + 16);
    e.load(R11, R15, 56);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "fatal");
    e.mov(A, Di);
    e.arith(4, A, 255);
    e.store(R15, HEADER + 24, A);
    e.0.call("child-release-slots");
    e.gs(0x8b, R15, TABLE_CONTEXT);
    e.load(R14, R15, HEADER + 88); // parent's pending fork/yield/wait retry
    // Evolved entropy state/accounting is shared with the root and must survive.
    for offset in [184_u32, 192, 200, 208, 216, 264] {
        e.gs(0x8b, A, offset);
        e.store(R15, CONTEXT + offset as i32, A);
    }
    for (saved, target, length) in [
        (DATA, 48, 0x12000),
        (STACK, 56, 4096),
        (STACK + 4096, 64, 4096),
    ] {
        e.load(R11, R15, HEADER + target);
        copy(&mut e, R15, saved, R11, 0, length);
    }
    e.gs(0x8b, R11, 120);
    copy(&mut e, R15, HEAP, R11, 0, 0x22000);
    e.gs(0x8b, R11, 112);
    copy(&mut e, R15, LOW_LEAVES, R11, 0, 4096);
    e.gs(0x8b, R11, 24);
    copy(&mut e, R15, HIGH_LEAVES, R11, 0x4000, 4096);
    e.gs(0x8b, A, 24);
    e.0.bytes(&[0x0f, 0x22, 0xd8]); // restore permissions, invalidate translations
    // Shared descriptions retain child position/reference changes; restore only
    // the parent's private header, slots, signals, cwd and staging.
    copy(&mut e, R15, TABLE, R15, 0, 320);
    copy(&mut e, R15, TABLE + 1344, R15, 1344, 8192 - 1344);
    e.gs(0x8b, R11, 256);
    copy(&mut e, R15, CONTEXT, R11, 0, 1024);
    e.load(A, R15, HEADER + 72);
    e.mov(D, A);
    e.0.bytes(&[0x48, 0xc1, 0xea, 0x20]);
    e.imm(C, 0xc0000100);
    e.0.bytes(&[0x0f, 0x30]);
    e.0.bytes(&[0x49, 0x0f, 0xae, 0x8f]);
    e.0.bytes(&SIMD.to_le_bytes()); // fxrstor64
    erase(
        &mut e,
        HEADER + 32,
        super::file_descriptions::CHILD_DESCRIPTIONS as u64 - (HEADER + 32) as u64,
    );
    e.imm(A, 2);
    e.store(R15, HEADER, A);
    e.mov(A, R14);
    emit_static_process_sysret(e.0);

    e.0.bind("child-wait")?;
    e.0.bytes(&[0x89, 0xd2, 0x48, 0x63, 0xff]); // native int options and signed selector
    e.cmp(D, 1);
    e.0.jump_if(0x87, "child-einval");
    e.gs(0x8b, A, 336);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "child-enosys");
    e.gs(0x8b, R15, TABLE_CONTEXT);
    e.load(A, R15, 56);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "child-echild");
    e.load(A, R15, HEADER);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "child-echild");
    e.cmp(A, 2);
    e.0.jump_if(0x87, "fatal");
    for selector in [-1, 0] {
        e.cmp(Di, selector);
        e.0.jump_if(0x84, "child-wait-matched");
    }
    e.load(A, R15, HEADER + 16);
    e.rr(0x39, Di, A);
    e.0.jump_if(0x85, "child-echild");
    e.0.bind("child-wait-matched")?;
    e.load(A, R15, HEADER);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "pair-wait-live");
    e.cmp(Si, 0);
    e.0.jump_if(0x84, "child-wait-published");
    e.mov(R8, Si);
    e.imm(R9, 4);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "child-efault");
    e.load(A, R15, HEADER + 24);
    e.0.bytes(&[0x48, 0xc1, 0xe0, 8, 0x41, 0x89, 0x00]);
    e.0.bind("child-wait-published")?;
    e.load(R14, R15, HEADER + 16);
    for offset in [HEADER, HEADER + 16, HEADER + 24] {
        e.imm(A, 0);
        e.store(R15, offset, A);
    }
    e.mov(A, R14);
    emit_static_process_sysret(e.0);
    for (label, errno) in [
        ("child-eagain", 11),
        ("child-echild", 10),
        ("child-einval", 22),
        ("child-enosys", 38),
        ("child-efault", 14),
        ("child-eperm", 1),
    ] {
        e.0.bind(label)?;
        e.imm(A, (-errno as i64) as u64);
        emit_static_process_sysret(e.0);
    }
    e.0.bind("child-parent-release")?;
    e.gs(0x8b, R15, TABLE_CONTEXT);
    e.load(A, R15, HEADER);
    e.cmp(A, 2);
    e.0.jump_if(0x87, "fatal");
    e.load(R11, R15, 56);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "fatal");
    e.cmp(A, 1);
    e.0.jump_if(0x85, "pair-parent-no-inactive-child");
    e.0.call("pair-release-inactive-slots");
    e.0.bind("pair-parent-no-inactive-child")?;
    erase(
        &mut e,
        HEADER,
        super::file_descriptions::CHILD_DESCRIPTIONS as u64 - HEADER as u64,
    );
    e.0.bytes(&[0xc3]);
    pair::emit_services(e.0)?;
    Ok(())
}

pub(in super::super) fn emit_exit_dispatch(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, HEADER);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "pair-exit-parent");
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 56);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "child-exit");
    e.0.bind("pair-exit-parent")?;
    Ok(())
}
