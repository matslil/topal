//! Controller return body on the already-retired source stack; no new reserve.
use super::*;

pub(super) const CALL: &str = "topal_x86_tail_checkpoint_call_v1";
pub(super) const BODY: &str = "topal_tail_checkpoint_body_v1";

fn call() -> Result<(Code, u64, u64), CompileError> {
    let mut code = Code::default();
    push_caller(&mut code);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R12, _Sp);
    }
    saved_rows(&mut code, 12);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R13, Si);
        e.mov(R14, Di);
        e.mov(R15, D);
        e.mov(B, C);
    }
    // Independent explicit history/outcome survive admission's volatile work.
    admission(&mut code, "deny", true)?;
    {
        let mut e = E(&mut code.encoder);
        e.cmp(R15, 0);
        e.0.jump_if(0x84, "deny");
        e.mov(A, R15);
        e.arith(4, A, 7);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "deny");
        e.mov(A, R15);
        e.add(A, 40);
        e.rr(0x39, A, R15);
        e.0.jump_if(0x82, "deny");
        e.rr(0x39, A, R8);
        e.0.jump_if(0x86, "history_private_ok");
        e.rr(0x39, R10, R15);
        e.0.jump_if(0x87, "deny");
        e.0.bind("history_private_ok")?;
        e.rr(0x39, A, R9);
        e.0.jump_if(0x86, "history_stack_ok");
        e.rr(0x39, D, R15);
        e.0.jump_if(0x87, "deny");
        e.0.bind("history_stack_ok")?;
        e.store(R14, 0, R12);
        e.imm(A, 1);
        e.store(R14, 8, A);
        e.store(R14, 16, R13);
        e.mov(Di, R14);
        e.add(Di, 64);
        e.store(R14, 24, Di);
        address(&mut e, &mut code.references, A, BODY);
        e.store(R14, 32, A);
        e.store(R14, 40, R15);
        e.mov(_Bp, R15);
        e.imm(A, PRIVATE_BYTES);
        e.store(R14, 48, A);
        e.imm(A, STACK_BYTES);
        e.store(R14, 56, A);
        address(&mut e, &mut code.references, R11, ACTIVE);
        e.store(R11, 0, R14);
        e.mov(Si, B);
        e.mov(_Sp, R13);
        e.add(_Sp, STACK_BYTES as i32);
        branch(&mut e, &mut code.references, BODY, true);
        e.mov(R15, A);
        e.cmp(A, 1);
        e.0.jump_if(0x87, "fail");
        e.rr(0x39, A, B);
        e.0.jump_if(0x85, "fail");
        e.mov(_Sp, R12);
    }
    saved_rows(&mut code, 7);
    {
        let mut e = E(&mut code.encoder);
        erase(&mut e, R13, STACK_BYTES, "fail");
        erase(&mut e, R14, HEADER_BYTES, "fail");
    }
    clear_active(&mut code);
    clear_volatile(&mut E(&mut code.encoder));
    E(&mut code.encoder).mov(A, R15);
    pop_caller(&mut code);
    let normal = code.encoder.bytes.len() as u64;
    code.encoder.bytes(&[0xc3]);
    code.encoder.bind("fail")?;
    saved_rows(&mut code, 12);
    let mut fatal = 0;
    fatal_cleanup(&mut code, &mut fatal, HistoryPin::Retained(_Bp))?;
    code.encoder.bind("deny")?;
    saved_rows(&mut code, 7);
    clear_volatile(&mut E(&mut code.encoder));
    E(&mut code.encoder).imm(A, 2);
    pop_caller(&mut code);
    code.encoder.bytes(&[0xc3]);
    Ok((code, normal, fatal))
}

pub(super) fn append(
    object: &mut Object<'_>,
    text: object::write::SectionId,
    frame: object::write::SectionId,
) -> Result<(), CompileError> {
    let (code, normal, fatal) = call()?;
    append_discontinuous_code(object, text, frame, CALL, code)?;
    let start = object
        .symbol(object.symbol_id(CALL.as_bytes()).unwrap())
        .value;
    marker(
        object,
        text,
        "topal_x86_tail_checkpoint_reclaimed_v1",
        start + normal,
    );
    marker(
        object,
        text,
        "topal_x86_tail_checkpoint_fatal_v1",
        start + fatal,
    );
    Ok(())
}
