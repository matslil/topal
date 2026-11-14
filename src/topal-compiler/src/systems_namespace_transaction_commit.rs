//! Complete scalar handoff, actual source call and post-loan victim retirement.
use super::*;

pub(super) fn emit(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-transaction-commit")?;
    // All recoverable admission, including RTC, is complete. D is seconds;
    // the eight plan words remain in registers, without a live plan array.
    for (index, register) in [B, R13, R14, R15, R8, R9, R10, R11].into_iter().enumerate() {
        e.store(R12, OLD_NAME + index as i32 * 8, register);
    }
    for (offset, bytes) in [(64, 64), (128, 256), (384, 128)] {
        e.mov(Di, R12);
        e.add(Di, OLD_NAME + offset);
        files::erase(e, Di, bytes);
    }
    e.mov(Di, R12);
    e.add(Di, OLD_NAME + 128);
    e.mov(C, R8);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    // PATH and transfer facts are not part of the source view or fatal recovery
    // record. Retire them before the grant, including on later source abort.
    wipe_name(e);
    wipe_staging(e);
    e.mov(C, D);
    e.mov(D, Bp);
    e.mov(Di, R12);
    e.mov(Si, R12);
    e.add(Si, files::BASE);
    e.0.external_call(super::super::super::super::namespace_region::APPLY);
    clear_return(e);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    // APPLY has revoked the loan and erased its complete temporary domain.
    // Reclaim only a replacement victim with neither names nor descriptions.
    e.cmp(R14, 4);
    e.0.jump_if(0x84, "namespace-commit-return");
    e.mov(R10, R14);
    files::shift(e, R10, true, 7);
    e.rr(0x01, R10, R12);
    e.add(R10, files::OBJECTS);
    e.load(A, R10, 56);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-commit-return");
    e.0.call("file-live-description");
    e.0.jump_if(0x83, "namespace-commit-inner-return");
    e.0.call("file-destroy-object");
    e.0.bind("namespace-commit-inner-return")?;
    clear_return(e);
    e.0.bind("namespace-commit-return")?;
    e.0.bytes(&[0xc3]);

    e.0.bind("namespace-transaction-clock-failed")?;
    retire_clock_stack(e);
    e.0.jump("namespace-eio");
    emit_final_helpers(e)
}

pub(super) fn emit_final_helpers(e: &mut E<'_>) -> Result<(), CompileError> {
    // A=FINAL, SI copied raw component, D full length. Return A1 for root,
    // dot/dot-dot, A0 for a normal name; preserve parent and flags registers.
    e.0.bind("namespace-transaction-final-kind")?;
    e.cmp(A, 1);
    e.0.jump_if(0x85, "namespace-final-kind-special");
    e.cmp(D, 2);
    e.0.jump_if(0x87, "namespace-final-kind-normal");
    byte(e, A, Si, 0);
    e.cmp(A, 46);
    e.0.jump_if(0x85, "namespace-final-kind-normal");
    e.cmp(D, 1);
    e.0.jump_if(0x84, "namespace-final-kind-special");
    byte(e, A, Si, 1);
    e.cmp(A, 46);
    e.0.jump_if(0x85, "namespace-final-kind-normal");
    e.0.bind("namespace-final-kind-special")?;
    e.imm(A, 1);
    e.0.bytes(&[0xc3]);
    e.0.bind("namespace-final-kind-normal")?;
    e.imm(A, 0);
    e.0.bytes(&[0xc3]);

    // Final lookup without traversing a trailing slash or creating anything.
    // R15 actual parent, SI/D raw name/length. Return R15 id or0, R11 entry/4.
    e.0.bind("namespace-transaction-final-lookup")?;
    e.cmp(D, 255);
    e.0.jump_if(0x87, "namespace-enametoolong");
    e.cmp(R15, 6);
    e.0.jump_if(0x84, "namespace-final-lookup-mutable");
    for (parent, bytes, identity, label) in [
        (3, b"bin".as_slice(), 4, "bin"),
        (3, b"tmp".as_slice(), 6, "tmp"),
        (4, b"busybox".as_slice(), 5, "busybox"),
    ] {
        let next = format!("namespace-final-lookup-after-{label}");
        e.cmp(R15, parent);
        e.0.jump_if(0x85, &next);
        e.cmp(D, bytes.len() as i32);
        e.0.jump_if(0x85, &next);
        for (offset, value) in bytes.iter().enumerate() {
            byte(e, A, Si, offset as i32);
            e.cmp(A, i32::from(*value));
            e.0.jump_if(0x85, &next);
        }
        e.imm(R15, identity);
        e.imm(R11, 4);
        e.0.bytes(&[0xc3]);
        e.0.bind(&next)?;
    }
    e.imm(R15, 0);
    e.imm(R11, 4);
    e.0.bytes(&[0xc3]);
    e.0.bind("namespace-final-lookup-mutable")?;
    e.0.call("file-entry-by-name");
    e.0.bytes(&[0xc3]);
    Ok(())
}
