//! TK-DEC-255 / TOPAL-SYSTEMS-STATIC-PROCESS-PAIR-001.
//! Exchange owned private continuations only at completed syscall boundaries.
use super::super::file_descriptions::{CHILD_DESCRIPTIONS, SLOTS};
use super::*;

/// Exchange qwords and independently read back both destinations. Source and
/// destination are disjoint owned supervisor aliases; no user PTE is followed.
fn exchange(
    e: &mut E<'_>,
    source: super::super::R,
    from: i32,
    destination: super::super::R,
    to: i32,
    bytes: u64,
    label: &'static str,
) -> Result<(), CompileError> {
    e.mov(Si, source);
    e.add(Si, from);
    e.mov(Di, destination);
    e.add(Di, to);
    e.imm(C, bytes / 8);
    e.0.bind(label)?;
    e.load(A, Si, 0);
    e.load(D, Di, 0);
    e.store(Si, 0, D);
    e.store(Di, 0, A);
    e.load(R8, Si, 0);
    e.rr(0x39, R8, D);
    e.0.jump_if(0x85, "fatal");
    e.load(R9, Di, 0);
    e.rr(0x39, R9, A);
    e.0.jump_if(0x85, "fatal");
    e.add(Si, 8);
    e.add(Di, 8);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, label);
    Ok(())
}

pub(super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("pair-yield")?;
    e.gs(0x8b, R15, TABLE_CONTEXT);
    e.load(A, R15, HEADER);
    e.cmp(A, 2);
    e.0.jump_if(0x87, "fatal");
    e.cmp(A, 1);
    e.0.jump_if(0x85, "pair-local-zero");
    e.load(A, R15, 56);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "pair-yield-switch");
    e.load(R11, R15, HEADER + 16);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R15, HEADER + 80); // parent waiting for this child
    e.cmp(A, 0);
    e.0.jump_if(0x85, "pair-local-zero");
    e.0.bind("pair-yield-switch")?;
    e.imm(R8, 0);
    e.0.jump("pair-switch");

    e.0.bind("pair-wait-live")?;
    e.cmp(D, 1);
    e.0.jump_if(0x84, "pair-local-zero");
    e.load(A, R15, HEADER + 80);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    // Re-enter the native wait after the exact child exits. No kernel stack
    // continuation or checked user output is retained across the handoff.
    e.gs(0x8b, A, 296);
    e.cmp(A, 0x401002);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 0x58c285);
    e.0.jump_if(0x83, "fatal");
    e.add(A, -2);
    e.gs(0x89, A, 296);
    e.imm(A, 1);
    e.store(R15, HEADER + 80, A);
    e.imm(R8, 61);
    e.0.bind("pair-switch")?;
    // Both private records must still belong to this exact pair and hierarchy.
    e.load(A, R15, TABLE);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R15, 56);
    e.load(R11, R15, HEADER + 16);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "pair-switch-target-identity");
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "fatal");
    e.imm(R11, 1);
    e.0.bind("pair-switch-target-identity")?;
    e.load(A, R15, TABLE + 56);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "fatal");
    for offset in [24, 112, 120, 256, 904, 928, 936, 944, 952, 960, 968] {
        e.gs(0x8b, A, offset);
        e.load(R11, R15, CONTEXT + offset as i32);
        e.rr(0x39, A, R11);
        e.0.jump_if(0x85, "fatal");
    }
    e.load(A, R15, CONTEXT + 296);
    e.cmp(A, 0x401000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 0x58c285);
    e.0.jump_if(0x83, "fatal");
    e.load(A, R15, CONTEXT + 408);
    e.cmp(A, 0x7fe000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 0x800000);
    e.0.jump_if(0x87, "fatal");
    // Swap native return ownership before touching either private snapshot.
    e.load(A, R15, HEADER + 88);
    e.store(R15, HEADER + 88, R8);
    e.store(R15, HEADER + 112, A);

    for (snapshot, pointer, length, label) in [
        (DATA, 48, 0x12000, "pair-exchange-image"),
        (STACK, 56, 4096, "pair-exchange-stack-low"),
        (STACK + 4096, 64, 4096, "pair-exchange-stack-high"),
    ] {
        e.load(R11, R15, HEADER + pointer);
        exchange(&mut e, R11, 0, R15, snapshot, length, label)?;
    }
    e.gs(0x8b, R11, 120);
    exchange(&mut e, R11, 0, R15, HEAP, 0x22000, "pair-exchange-heap")?;
    e.gs(0x8b, R11, 112);
    exchange(
        &mut e,
        R11,
        0,
        R15,
        LOW_LEAVES,
        4096,
        "pair-exchange-low-leaves",
    )?;
    e.gs(0x8b, R11, 24);
    exchange(
        &mut e,
        R11,
        0x4000,
        R15,
        HIGH_LEAVES,
        4096,
        "pair-exchange-high-leaves",
    )?;
    // Shared description cells are not part of either private table snapshot.
    exchange(&mut e, R15, 0, R15, TABLE, 320, "pair-exchange-slots")?;
    exchange(
        &mut e,
        R15,
        1344,
        R15,
        TABLE + 1344,
        8192 - 1344,
        "pair-exchange-private-table",
    )?;
    e.gs(0x8b, R11, 256);
    // Exclude shared generator history, whose inactive representation is zero.
    for (offset, bytes, label) in [
        (0, 184, "pair-exchange-context-head"),
        (224, 40, "pair-exchange-context-middle"),
        (272, 752, "pair-exchange-context-tail"),
    ] {
        exchange(&mut e, R11, offset, R15, CONTEXT + offset, bytes, label)?;
    }
    for offset in [184, 192, 200, 208, 216, 264] {
        e.load(A, R15, CONTEXT + offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fatal");
    }
    // Hardware extended state uses a temporary aligned record, erased before
    // user return. Neither copy loop nor checked provider uses SIMD registers.
    e.0.bytes(&[0x49, 0x0f, 0xae, 0x87]);
    e.0.bytes(&(SIMD + 512).to_le_bytes());
    e.0.bytes(&[0x49, 0x0f, 0xae, 0x8f]);
    e.0.bytes(&SIMD.to_le_bytes());
    copy(&mut e, R15, SIMD + 512, R15, SIMD, 512);
    erase(&mut e, SIMD + 512, 512);
    read_msr(&mut e, 0xc0000100);
    e.mov(R14, A);
    e.load(A, R15, HEADER + 72);
    e.mov(D, A);
    e.0.bytes(&[0x48, 0xc1, 0xea, 0x20]);
    e.imm(C, 0xc0000100);
    e.0.bytes(&[0x0f, 0x30]);
    e.store(R15, HEADER + 72, R14);
    e.gs(0x8b, A, 24);
    e.0.bytes(&[0x0f, 0x22, 0xd8]);
    e.load(A, R15, HEADER + 112);
    e.imm(R11, 0);
    e.store(R15, HEADER + 112, R11);
    emit_static_process_sysret(e.0);
    e.0.bind("pair-local-zero")?;
    e.imm(A, 0);
    emit_static_process_sysret(e.0);

    // Parent outer-owner teardown releases the inactive child's slot references
    // before its private table is erased. The active parent table stays intact.
    e.0.bind("pair-release-inactive-slots")?;
    e.load(A, R15, TABLE);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R15, TABLE + 56);
    e.load(R11, R15, HEADER + 16);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "fatal");
    e.imm(R8, 0);
    e.0.bind("pair-release-inactive-next")?;
    e.mov(R13, R8);
    e.0.bytes(&[0x49, 0xc1, 0xe5, 4]);
    e.rr(0x01, R13, R15);
    e.add(R13, TABLE + SLOTS);
    e.load(A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "pair-release-inactive-advance");
    e.cmp(A, 32);
    e.0.jump_if(0x87, "fatal");
    e.mov(R14, R15);
    e.add(R14, DESCRIPTIONS);
    e.cmp(A, 16);
    e.0.jump_if(0x86, "pair-release-description-base");
    e.add(R14, CHILD_DESCRIPTIONS - DESCRIPTIONS - 16 * 64);
    e.0.bind("pair-release-description-base")?;
    e.add(A, -1);
    e.0.bytes(&[0x48, 0xc1, 0xe0, 6]);
    e.rr(0x01, R14, A);
    e.load(A, R14, 8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.0.call("fd-drop-reference");
    e.0.bind("pair-release-inactive-advance")?;
    e.add(R8, 1);
    e.cmp(R8, 16);
    e.0.jump_if(0x82, "pair-release-inactive-next");
    e.0.bytes(&[0xc3]);
    Ok(())
}
