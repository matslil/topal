//! Whole-invocation return on the existing sequential private stack.
use super::*;

pub(super) const CALL: &str = "topal_x86_tail_complete_call_v1";
pub(super) const BODY: &str = "topal_tail_complete_body_v1";
pub(super) const OUTPUT_BYTES: u64 = 56;

fn apart(
    e: &mut E<'_>,
    base: R,
    end: R,
    other: R,
    other_end: R,
    label: &str,
) -> Result<(), CompileError> {
    e.rr(0x39, end, other);
    e.0.jump_if(0x86, label);
    e.rr(0x39, other_end, base);
    e.0.jump_if(0x87, "deny");
    e.0.bind(label)?;
    Ok(())
}
fn checked_end(e: &mut E<'_>, base: R, bytes: i32) {
    e.cmp(base, 0);
    e.0.jump_if(0x84, "deny");
    e.mov(A, base);
    e.arith(4, A, 7);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "deny");
    e.mov(A, base);
    e.add(A, bytes);
    e.rr(0x39, A, base);
    e.0.jump_if(0x82, "deny");
}

/// The caller supplies output/history facts; neither pin comes from context.
fn call() -> Result<(Code, u64, u64), CompileError> {
    let mut code = Code::default();
    push_caller(&mut code);
    E(&mut code.encoder).mov(R12, _Sp);
    saved_rows(&mut code, 12);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R13, Si);
        e.mov(R14, Di);
        e.mov(R15, D);
        e.mov(B, C);
    }
    admission(&mut code, "deny", true)?;
    {
        let mut e = E(&mut code.encoder);
        checked_end(&mut e, R15, 40);
        apart(&mut e, R15, A, R8, R10, "history_private_ok")?;
        apart(&mut e, R15, A, R9, D, "history_stack_ok")?;
        e.mov(R11, A);
        checked_end(&mut e, B, OUTPUT_BYTES as i32);
        apart(&mut e, B, A, R8, R10, "output_private_ok")?;
        apart(&mut e, B, A, R9, D, "output_stack_ok")?;
        apart(&mut e, B, A, R15, R11, "output_history_ok")?;
        e.mov(_Bp, R15);
        e.store(R14, 0, R12);
        e.imm(A, 1);
        e.store(R14, 8, A);
        e.store(R14, 16, R13);
        e.mov(Di, R14);
        e.add(Di, HEADER_BYTES as i32);
        e.store(R14, 24, Di);
        address(&mut e, &mut code.references, A, BODY);
        e.store(R14, 32, A);
        e.store(R14, 40, _Bp);
        // Role-specific output pins; source/checkpoint use extent constants here.
        e.store(R14, 48, B);
        e.imm(A, OUTPUT_BYTES);
        e.store(R14, 56, A);
        address(&mut e, &mut code.references, R11, ACTIVE);
        e.store(R11, 0, R14);
        e.mov(Si, B);
        e.mov(_Sp, R13);
        e.add(_Sp, STACK_BYTES as i32);
        branch(&mut e, &mut code.references, BODY, true);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
        e.mov(_Sp, R12);
    }
    saved_rows(&mut code, 7);
    {
        let mut e = E(&mut code.encoder);
        erase(&mut e, R13, STACK_BYTES, "fail");
        erase(&mut e, R14, PRIVATE_BYTES, "fail");
    }
    clear_active(&mut code);
    {
        let mut e = E(&mut code.encoder);
        e.imm(A, 0);
        e.store(_Bp, 24, A);
        clear_volatile(&mut e);
    }
    pop_caller(&mut code);
    let normal = code.encoder.bytes.len() as u64;
    code.encoder.bytes(&[0xc3]);
    code.encoder.bind("fail")?;
    saved_rows(&mut code, 12);
    let mut fatal = 0;
    failure(&mut code, &mut fatal)?;
    code.encoder.bind("deny")?;
    saved_rows(&mut code, 7);
    clear_volatile(&mut E(&mut code.encoder));
    E(&mut code.encoder).imm(A, 2);
    pop_caller(&mut code);
    code.encoder.bytes(&[0xc3]);
    Ok((code, normal, fatal))
}

fn failure(code: &mut Code, marker: &mut u64) -> Result<(), CompileError> {
    E(&mut code.encoder).mov(_Sp, R12);
    saved_rows(code, 7);
    {
        let mut e = E(&mut code.encoder);
        e.imm(A, 0);
        e.store(_Bp, 24, A);
        e.imm(A, 1);
        e.store(_Bp, 32, A);
        erase(&mut e, B, OUTPUT_BYTES, "output_retired");
        e.0.bind("output_retired")?;
    }
    fatal_cleanup_named(code, marker, HistoryPin::Retained(_Bp), "complete_")
}

pub(super) fn abort_cleanup(code: &mut Code, marker: &mut u64) -> Result<(), CompileError> {
    // The generic recovery entry has already pivoted to the owning caller and
    // installed its saved-register unwind row before replacing body registers.
    {
        let mut e = E(&mut code.encoder);
        e.load(B, R14, 48);
        e.load(_Bp, R14, 40);
    }
    failure(code, marker)
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
        "topal_x86_tail_complete_reclaimed_v1",
        start + normal,
    );
    marker(
        object,
        text,
        "topal_x86_tail_complete_fatal_v1",
        start + fatal,
    );
    Ok(())
}
