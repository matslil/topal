//! Actual captured syscall/cleanup context, before claiming an invocation lease.
use super::*;

pub(super) const STATE: &str = "topal_x86_frame_mm_state_v1";
const EPOCH: &str = "topal_x86_frame_mm_epoch_v1";

pub(super) fn entry(code: &mut Code, user_extents: bool) -> Result<(), CompileError> {
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, "topal_bootstrap_storage");
    e.add(R11, 512);
    e.cmp(Di, 4);
    e.0.jump_if(0x86, "mm_boot_entry");
    if user_extents {
        e.cmp(Di, 8);
        e.0.jump_if(0x84, "mm_extent_entry");
    }
    e.cmp(Di, 7);
    e.0.jump_if(0x87, "unsafe");
    // These operations take the captured ABI request, no caller pointers.
    for r in [D, C, R8] {
        e.cmp(r, 0);
        e.0.jump_if(0x85, "unsafe");
    }
    e.load(A, _Sp, 0);
    e.cmp(Di, 5);
    e.0.jump_if(0x84, "mm_brk_entry");
    e.cmp(Si, 0);
    e.0.jump_if(0x85, "unsafe");
    e.cmp(Di, 6);
    e.0.jump_if(0x84, "mm_cleanup_entry");
    address(
        &mut e,
        &mut code.references,
        R10,
        "topal_x86_frame_mm_exec_return_v1",
    );
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "unsafe");
    e.imm(R9, 59);
    e.0.jump("mm_syscall_context");
    e.0.bind("mm_brk_entry")?;
    address(
        &mut e,
        &mut code.references,
        R10,
        "topal_x86_frame_mm_brk_return_v1",
    );
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "unsafe");
    e.load(A, R11, 312);
    e.rr(0x39, A, Si);
    e.0.jump_if(0x85, "unsafe");
    e.imm(R9, 12);
    e.0.bind("mm_syscall_context")?;
    e.load(A, R11, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "unsafe");
    e.load(A, R11, 256);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "unsafe");
    e.load(R10, R11, 24);
    e.imm(C, 0xc000_0101);
    e.0.jump("mm_current_hardware");
    e.0.bind("mm_cleanup_entry")?;
    address(
        &mut e,
        &mut code.references,
        R10,
        "topal_x86_frame_mm_cleanup_return_v1",
    );
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "unsafe");
    e.load(A, R11, 0);
    e.cmp(A, 2);
    e.0.jump_if(0x85, "unsafe");
    e.load(R10, R11, 16);
    e.imm(C, 0xc000_0102);
    e.imm(R9, 0);
    e.0.bind("mm_current_hardware")?;
    e.0.bytes(&[0x0f, 0x20, 0xd8]); // actual CR3
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "unsafe");
    e.0.bytes(&[0x0f, 0x32, 0x48, 0xc1, 0xe2, 32]); // rdmsr; shl rdx,32
    e.rr(0x09, A, D);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "unsafe");
    e.load(A, R11, 8);
    e.mov(R10, _Sp);
    e.add(R10, 16);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "unsafe");
    address(&mut e, &mut code.references, R11, EPOCH);
    e.load(A, R11, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "unsafe");
    e.load(A, R11, 8);
    e.cmp(Di, 6);
    e.0.jump_if(0x85, "mm_request_number");
    e.cmp(A, 60);
    e.0.jump_if(0x84, "mm_context_ready");
    e.cmp(A, 231);
    e.0.jump_if(0x85, "unsafe");
    e.0.jump("mm_context_ready");
    e.0.bind("mm_request_number")?;
    e.rr(0x39, A, R9);
    e.0.jump_if(0x85, "unsafe");
    e.0.bind("mm_context_ready")?;
    e.imm(D, 0);
    e.imm(C, 0);
    e.imm(R9, 0);
    e.0.jump("mm_entry_checked");
    e.0.bind("mm_boot_entry")?;
    e.load(A, R11, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "unsafe");
    e.0.bind("mm_entry_checked")?;
    if user_extents {
        e.0.jump("mm_extent_entry_done");
        extent_entry(code)?;
        code.encoder.bind("mm_extent_entry_done")?;
    }
    Ok(())
}

fn extent_entry(code: &mut Code) -> Result<(), CompileError> {
    let mut e = E(&mut code.encoder);
    e.0.bind("mm_extent_entry")?;
    e.cmp(R8, 0);
    e.0.jump_if(0x85, "unsafe");
    e.cmp(C, 1);
    e.0.jump_if(0x87, "unsafe");
    // Save the scalar length/direction in volatile registers before RDMSR.
    // No stack store or cleanup authority is needed for pre-entry refusal.
    e.mov(R9, D);
    e.mov(R8, C);
    e.load(A, R11, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "unsafe");
    e.load(A, R11, 256);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "unsafe");
    e.load(R10, R11, 24);
    e.0.bytes(&[0x0f, 0x20, 0xd8]);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "unsafe");
    e.imm(C, 0xc000_0101);
    e.0.bytes(&[0x0f, 0x32, 0x48, 0xc1, 0xe2, 32]);
    e.rr(0x09, A, D);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "unsafe");
    e.load(A, R11, 8);
    e.cmp(A, MAX_SP + 8);
    e.0.jump_if(0x87, "unsafe");
    e.mov(R10, _Sp);
    e.add(R10, 16);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x82, "unsafe");
    e.load(A, _Sp, 0);
    address(
        &mut e,
        &mut code.references,
        R10,
        "topal_x86_user_extent_return_v1",
    );
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "unsafe");
    address(&mut e, &mut code.references, R10, EPOCH);
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "unsafe");
    e.mov(D, R9);
    e.mov(C, R8);
    e.imm(R8, 0);
    e.imm(R9, 0);
    e.imm(R10, 0);
    e.0.jump("mm_extent_entry_done");
    Ok(())
}
pub(super) fn capture(code: &mut Code) -> Result<(), CompileError> {
    let mut e = E(&mut code.encoder);
    e.cmp(R13, 5);
    e.0.jump_if(0x82, "mm_epoch_captured");
    address(&mut e, &mut code.references, R10, EPOCH);
    e.load(A, R10, 0);
    e.store(R11, 40, A);
    e.0.bind("mm_epoch_captured")?;
    Ok(())
}
pub(super) fn result(code: &mut Code, user_extents: bool) -> Result<(), CompileError> {
    let mut e = E(&mut code.encoder);
    if user_extents {
        e.cmp(C, 8);
        e.0.jump_if(0x85, "mm_extent_result_other");
        e.cmp(A, 4);
        e.0.jump_if(0x87, "failed");
        e.0.jump("result_checked");
        e.0.bind("mm_extent_result_other")?;
    }
    e.cmp(C, 5);
    e.0.jump_if(0x85, "mm_result_checked");
    e.cmp(A, 0x5f7000);
    e.0.jump_if(0x82, "failed");
    e.cmp(A, 0x7fe000);
    e.0.jump_if(0x87, "failed");
    e.0.jump("result_checked");
    e.0.bind("mm_result_checked")?;
    Ok(())
}
