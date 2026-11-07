//! Fatal recovery never trusts the aborting callee's stack/frame registers.
use super::*;

fn zero_only(e: &mut E<'_>, base: R, bytes: u64) {
    e.mov(Di, base);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
}

/// The V6 owning adapter frame remains above APPLY's recovery frame. It is
/// abandoned on fatal source completion; retire it using only protected entry
/// context, independently of aborting source registers or its live stack.
pub(super) fn admission_halt() -> Result<(Code, u64), CompileError> {
    let mut code = Code::default();
    code.unwind.push((0, vec![0x07, 16]));
    let mut e = E(&mut code.encoder);
    e.0.bytes(&[0xfa]);
    e.gs(0x8b, _Sp, ADMISSION_STACK_CONTEXT);
    e.cmp(_Sp, ADMISSION_STACK_MINIMUM);
    e.0.jump_if(0x82, "clear");
    e.cmp(_Sp, ADMISSION_STACK_MAXIMUM);
    e.0.jump_if(0x87, "clear");
    e.mov(A, _Sp);
    e.arith(4, A, 7);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "clear");
    e.mov(R11, _Sp);
    e.add(R11, -(ADMISSION_RETIRED_STACK_BYTES as i32));
    zero_only(&mut e, R11, ADMISSION_RETIRED_STACK_BYTES);
    e.mov(Di, R11);
    e.imm(C, ADMISSION_RETIRED_STACK_BYTES / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    // A failed readback still reaches the same non-returning halt. Source
    // recovery has already attempted every other owned temporary erasure.
    e.0.bind("clear")?;
    clear_temporaries(&mut e);
    for register in [B, R12, R13, R14, R15, _Bp] {
        e.imm(register, 0);
    }
    let halt_offset = e.0.bytes.len() as u64;
    e.0.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    Ok((code, halt_offset))
}

pub(super) fn abort() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.0.bytes(&[0xfa]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x07, 16]));
    address(&mut e, &mut code.references, R11, RECOVERY);
    e.load(R13, R11, 0);
    // Reject corrupted recovery addresses without writing through them. The
    // saved frame was published by APPLY before issuing the actual source loan.
    e.cmp(R13, 0x108000 - FRAME_DELTA);
    e.0.jump_if(0x82, "no-frame");
    e.cmp(R13, STACK_ENTRY_MAXIMUM - FRAME_DELTA);
    e.0.jump_if(0x87, "no-frame");
    e.mov(A, R13);
    e.arith(4, A, 7);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "frame-ready");
    e.0.bind("no-frame")?;
    e.imm(R13, 0);
    e.0.bind("frame-ready")?;
    // Attempt EVERY erasure before any readback can fail. No calls, pushes or
    // stack references are made while erasing the currently executing reserve.
    for (symbol, count) in [
        (ARENA, ARENA_BYTES),
        (CONTROL, 64),
        (oracle::EXPECTED, oracle::EXPECTED_BYTES),
        (RECOVERY, RECOVERY_BYTES),
    ] {
        address(&mut e, &mut code.references, R11, symbol);
        zero_only(&mut e, R11, count);
    }
    address(
        &mut e,
        &mut code.references,
        R14,
        super::super::X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL,
    );
    e.add(R14, METADATA + 3072);
    zero_only(&mut e, R14, 512);
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "frame-erased");
    e.add(R13, -(RETIRED_STACK_BYTES as i32));
    zero_only(&mut e, R13, RETIRED_STACK_BYTES + LIVE_FRAME_BYTES);
    e.0.bind("frame-erased")?;
    for (symbol, count) in [
        (ARENA, ARENA_BYTES),
        (CONTROL, 64),
        (oracle::EXPECTED, oracle::EXPECTED_BYTES),
        (RECOVERY, RECOVERY_BYTES),
    ] {
        address(&mut e, &mut code.references, R11, symbol);
        zero_readback(&mut e, R11, count);
    }
    zero_readback(&mut e, R14, 512);
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "fail");
    zero_readback(&mut e, R13, RETIRED_STACK_BYTES + LIVE_FRAME_BYTES);
    e.0.bind("fail")?;
    clear_temporaries(&mut e);
    for register in [B, R12, R13, R14, R15, _Bp] {
        e.imm(register, 0);
    }
    branch(&mut e, &mut code.references, HALT, false);
    Ok(code)
}
