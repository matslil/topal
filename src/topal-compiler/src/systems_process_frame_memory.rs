//! Captured ABI requests forwarded to the combined owning memory gateway.
use super::x86_function::{E, R::*};
use super::{CompileError, X86FunctionEncoder, emit_static_process_sysret};
const GATE: &str = "topal_x86_frame_gateway_v1";

pub(super) fn capture(code: &mut X86FunctionEncoder) {
    // The original user registers have been captured and the trusted kernel
    // continuation stack installed. No untrusted stack is ever used here.
    code.bytes(&[0x89, 0xc0]);
    code.external_address(&[0x4c, 0x8d, 0x1d], "topal_x86_frame_mm_epoch_v1");
    let mut e = E(code);
    e.load(R10, R11, 0);
    e.cmp(R10, -1);
    e.0.jump_if(0x84, "fatal");
    e.add(R10, 1);
    e.store(R11, 0, R10);
    e.store(R11, 8, A);
}
pub(super) fn call(
    code: &mut X86FunctionEncoder,
    op: u64,
    label: &str,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.imm(Di, op);
    e.imm(D, 0);
    e.imm(C, 0);
    e.imm(R8, 0);
    e.0.bytes(&[0x48, 0x83, 0xec, 8, 0x48, 0xc7, 0x04, 0x24, 0, 0, 0, 0]);
    e.0.external_call(GATE);
    e.0.bind(label)?;
    e.0.bytes(&[0x48, 0xc7, 0x04, 0x24, 0, 0, 0, 0, 0x48, 0x83, 0xc4, 8]);
    Ok(())
}
pub(super) fn brk(
    code: &mut X86FunctionEncoder,
    user_extent_probe: bool,
) -> Result<(), CompileError> {
    E(code).mov(Si, Di);
    call(code, 5, "frame-mm-brk-return")?;
    if user_extent_probe {
        // Unpublished experiment only. Preserve the actual ABI result, run the
        // native observation after the MM owner has returned, then physically
        // retire the completed probe's caller storage before resumption.
        code.bytes(&[0x50]);
        code.external_call("topal_x86_user_extent_probe_v1");
        let mut e = E(code);
        e.mov(Di, _Sp);
        e.add(Di, -4096);
        e.imm(A, 0);
        e.imm(C, 512);
        e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab, 0x58]);
        e.imm(C, 0);
        e.store(_Sp, -8, C);
    }
    let mut e = E(code);
    // Preserve the predecessor startup witness without using it to choose a
    // count, an allocation policy, a range, or whether the request is legal.
    e.gs(0x8b, R9, 104);
    e.gs(0x8b, R10, 312);
    e.cmp(R10, 0);
    e.0.jump_if(0x84, "frame-mm-brk-query");
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "frame-mm-brk-sysret");
    for (phase, label) in [
        (1_u8, "frame-mm-started"),
        (6, "frame-mm-grown"),
        (7, "frame-mm-aligned"),
    ] {
        e.cmp(R9, phase as i32);
        e.0.jump_if(0x84, label);
    }
    e.0.jump("frame-mm-brk-sysret");
    e.0.bind("frame-mm-brk-query")?;
    e.cmp(R9, 0);
    e.0.jump_if(0x85, "frame-mm-brk-sysret");
    e.0.bytes(&[0x65, 0xc6, 0x04, 0x25, 104, 0, 0, 0, 1]);
    e.0.jump("frame-mm-brk-sysret");
    for (next, label, witness) in [
        (2_u8, "frame-mm-started", 0x5f7d80),
        (7, "frame-mm-grown", 0x618d80),
        (8, "frame-mm-aligned", 0x619000),
    ] {
        e.0.bind(label)?;
        if e.0.owned_user_extents {
            // An ordinary resize during startup does not consume a different
            // historical observation. It remains a legal MM request above.
            e.cmp(R10, witness);
            e.0.jump_if(0x85, "frame-mm-brk-sysret");
        }
        e.0.bytes(&[0x65, 0xc6, 0x04, 0x25, 104, 0, 0, 0, next]);
        e.0.jump("frame-mm-brk-sysret");
    }
    e.0.bind("frame-mm-brk-sysret")?;
    emit_static_process_sysret(e.0);
    Ok(())
}
