//! Immutable prepared process/system observations; TK-DEC-247.
use super::file_descriptions::TABLE_CONTEXT;
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};
pub(super) const IDENTITY: i32 = 56;

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "process-observations-dispatch-end");
    e.gs(0x8b, R11, 104);
    e.arith(4, R11, 255);
    e.cmp(R11, 11);
    e.0.jump_if(0x82, "process-observations-dispatch-end");
    for (number, label) in [
        (39, "process-observations-self"),
        (186, "process-observations-self"),
        (110, "process-observations-parent"),
        (63, "process-observations-uname"),
    ] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.0.bind("process-observations-dispatch-end")?;
    Ok(())
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("process-observations-parent")?;
    e.imm(R9, 1);
    e.0.jump("process-observations-identity");
    e.0.bind("process-observations-self")?;
    e.imm(R9, 0);
    e.0.bind("process-observations-identity")?;
    e.gs(0x8b, R12, TABLE_CONTEXT);
    e.load(A, R12, IDENTITY);
    // The imported prepared initial process has one thread and no parent
    // visible in its qualified view. The witness is never mutable user data.
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.rr(0x29, A, R9);
    emit_static_process_sysret(e.0);

    e.0.bind("process-observations-uname")?;
    e.mov(R8, Di);
    e.imm(R9, 390);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "process-observations-efault");
    e.0.rip_lea(&[0x48, 0x8d, 0x35], "static-process-shell-uts-data");
    e.imm(C, 390);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.imm(A, 0);
    emit_static_process_sysret(e.0);
    e.0.bind("process-observations-efault")?;
    e.imm(A, (-14_i64) as u64);
    emit_static_process_sysret(e.0);
    Ok(())
}
