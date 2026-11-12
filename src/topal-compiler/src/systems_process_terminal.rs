//! TOPAL-SYSTEMS-PROCESS-MM-001: terminate the actual imported outer owner.
//! This path is entered by a jump at the captured syscall stack boundary.
use super::file_descriptions::TABLE_CONTEXT;
use super::{CompileError, E, R::*, X86FunctionEncoder};

pub(super) fn emit(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("process-mm-parent-exit")?;
    e.gs(0x8b, R15, TABLE_CONTEXT);
    for (offset, expected) in [(0, 1), (56, 1)] {
        e.load(A, R15, offset);
        e.cmp(A, expected);
        e.0.jump_if(0x85, "fatal");
    }
    e.load(A, R15, 0x2000);
    e.cmp(A, 2);
    e.0.jump_if(0x87, "fatal");
    cancel_wait(&mut e)?;
    // Waits may no longer reference backing or descriptors after cancellation.
    // Clear them before either resource family can be released, and read back.
    for offset in [0x2060, 0x2078] {
        e.imm(A, 0);
        e.store(R15, offset, A);
        e.load(R11, R15, offset);
        e.cmp(R11, 0);
        e.0.jump_if(0x85, "fatal");
    }
    e.imm(Si, 0);
    super::super::frame_memory::call(e.0, 13, "process-mm-cancel-return")?;
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.gs(0x8b, Di, 312);
    super::emit_exit_witness(e.0);
    // Inactive-slot destruction takes R12 as the shared description-store
    // owner. SYSCALL preserved the user's R12; obtain the trusted table here.
    e.gs(0x8b, R12, TABLE_CONTEXT);
    e.0.call("child-parent-release");
    e.0.call("fd-terminal-release");
    super::shell_input::emit_retire(e.0);
    e.0.bind("process-mm-parent-resources-retired")?;
    // Initialization transferred the prelude dispositions into the owned table.
    // Its destructors above consumed current registrations; these aliases must
    // already be empty. Do not erase live authority to satisfy cleanup.
    for offset in [168, 176, 224, 232] {
        e.gs(0x8b, A, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fatal");
    }
    for offset in [417_u32, 737] {
        e.0.bytes(&[0x65, 0x80, 0x3c, 0x25]);
        e.0.bytes(&offset.to_le_bytes());
        e.0.bytes(&[0]);
        e.0.jump_if(0x85, "fatal");
    }
    // Retire diagnostic staging only after the real owners are consumed.
    // The fstat/startup transcript occupies 616..623; 472..615 is distinct
    // transfer staging and remains subject to the existing zero readback.
    e.imm(A, 0);
    e.gs(0x89, A, 616);
    for offset in (736..=888).step_by(8) {
        e.imm(A, 0);
        e.gs(0x89, A, offset);
    }
    e.0.bytes(&[0x65, 0xc6, 0x04, 0x25]);
    e.0.bytes(&418_u32.to_le_bytes());
    e.0.bytes(&[0]);
    // Shared teardown verifies quiescence, erases entropy and registrations,
    // detaches the context, then releases active MM and reads back all storage.
    e.0.jump("static-process-terminal-phase-ready");
    Ok(())
}

/// Resolve the parked native FD and actual FIFO before consuming a wait.
/// A saved condition/object integer cannot manufacture an endpoint owner.
fn cancel_wait(e: &mut E<'_>) -> Result<(), CompileError> {
    e.load(A, R15, 0x2060);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "process-mm-cancel-no-wait");
    e.cmp(A, 2);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 3);
    e.0.jump_if(0x87, "fatal");
    e.mov(R9, A);
    e.load(A, R15, 0x2000);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.mov(R12, R15);
    e.load(A, R15, 0x2100 + 312);
    e.0.bytes(&[0x89, 0xc0]); // retained native int FD
    e.cmp(A, 16);
    e.0.jump_if(0x83, "fatal");
    e.0.bytes(&[0x48, 0xc1, 0xe0, 4]);
    e.mov(R13, R15);
    e.add(R13, 0x3040);
    e.rr(0x01, R13, A);
    e.load(A, R13, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 32);
    e.0.jump_if(0x87, "fatal");
    e.mov(R14, R15);
    e.add(R14, super::file_descriptions::DESCRIPTIONS);
    e.cmp(A, 16);
    e.0.jump_if(0x86, "process-mm-cancel-description-base");
    e.add(
        R14,
        super::file_descriptions::CHILD_DESCRIPTIONS
            - super::file_descriptions::DESCRIPTIONS
            - 16 * 64,
    );
    e.0.bind("process-mm-cancel-description-base")?;
    e.add(A, -1);
    e.0.bytes(&[0x48, 0xc1, 0xe0, 6]);
    e.rr(0x01, R14, A);
    e.load(A, R14, 0);
    e.add(R9, 3);
    e.rr(0x39, A, R9);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R14, 8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.0.call("pipe-object-lookup");
    e.load(A, R10, 8);
    e.load(R11, R15, 0x2078);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "fatal");
    e.0.jump("process-mm-cancel-wait-validated");
    e.0.bind("process-mm-cancel-no-wait")?;
    e.load(A, R15, 0x2078);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.0.bind("process-mm-cancel-wait-validated")?;
    Ok(())
}
