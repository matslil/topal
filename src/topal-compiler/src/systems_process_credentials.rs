//! Readonly prepared-root witness; no mutable user auxiliary data is authority.
//! TOPAL-SYSTEMS-STATIC-PROCESS-ROOT-CREDENTIALS-001; TK-DEC-244.
use super::file_descriptions::TABLE_CONTEXT;
use super::{CompileError, E, R::*, X86FunctionEncoder, emit_static_process_sysret};

pub(in super::super) fn emit_dispatch(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R11, TABLE_CONTEXT);
    e.load(R11, R11, 0);
    e.cmp(R11, 1);
    e.0.jump_if(0x85, "root-credentials-dispatch-end");
    // Initial image identity preparation retains its predecessor adapter.
    e.gs(0x8b, R11, 104);
    e.arith(4, R11, 255);
    e.cmp(R11, 11);
    e.0.jump_if(0x82, "root-credentials-dispatch-end");
    for (number, label) in [
        (102, "root-credentials-query"),
        (104, "root-credentials-getgid"),
        (107, "root-credentials-query"),
        (108, "root-credentials-query"),
        (105, "root-credentials-setuid"),
        (106, "root-credentials-setgid"),
    ] {
        e.cmp(A, number);
        e.0.jump_if(0x84, label);
    }
    e.0.bind("root-credentials-dispatch-end")?;
    Ok(())
}

pub(in super::super) fn emit_services(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("root-credentials-getgid")?;
    e.imm(R9, 13);
    e.0.jump("root-credentials-observe");
    e.0.bind("root-credentials-setuid")?;
    e.imm(R9, 15);
    e.0.jump("root-credentials-select");
    e.0.bind("root-credentials-setgid")?;
    e.imm(R9, 14);
    e.0.bind("root-credentials-select")?;
    e.0.bytes(&[0x89, 0xff]);
    e.cmp(Di, 0);
    e.0.jump_if(0x85, "fatal");
    e.0.bind("root-credentials-observe")?;
    // Record only the credential call actually observed. The predecessor's
    // final command phase (32) must remain available for image commitment.
    e.cmp(R11, 15);
    e.0.jump_if(0x87, "root-credentials-query");
    e.0.bytes(&[0x65, 0x44, 0x88, 0x0c, 0x25]);
    e.0.bytes(&104_u32.to_le_bytes());
    e.0.bind("root-credentials-query")?;
    e.gs(0x8b, R12, TABLE_CONTEXT);
    e.load(A, R12, 48);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.imm(A, 0);
    emit_static_process_sysret(e.0);
    Ok(())
}
