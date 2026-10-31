//! Artifact-owned pure Topal policy calls; TK-DEC-252.
use super::R::{_Bp as Bp, _Sp as Sp};
use super::{CompileError, E, R::*, X86FunctionEncoder};

pub(in super::super) const POLICY_CONTEXT: u32 = 952;

pub(in super::super) fn emit_entry_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.load(A, Sp, 0x60); // thirteenth argument after five provider saves
    e.cmp(A, 0x200000);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, 0x40000000);
    e.0.jump_if(0x83, "fail");
    e.cmp(A, 0x400000);
    e.0.jump_if(0x82, "policy-entry-disjoint");
    e.cmp(A, 0x800000);
    e.0.jump_if(0x82, "fail");
    e.0.bind("policy-entry-disjoint")?;
    e.store(R10, POLICY_CONTEXT as i32, A);
    Ok(())
}

pub(super) fn emit_open_decision(e: &mut E<'_>) -> Result<(), CompileError> {
    // Pack only three finite facts. No object, flags word or pointer crosses.
    e.imm(Di, 0);
    e.mov(A, Bp);
    e.arith(4, A, 3);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "policy-open-read");
    e.arith(1, Di, 1);
    e.0.bind("policy-open-read")?;
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "policy-open-regular");
    e.arith(1, Di, 2);
    e.0.bind("policy-open-regular")?;
    e.mov(A, Bp);
    e.arith(4, A, 0x10000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "policy-open-facts");
    e.arith(1, Di, 4);
    e.0.bind("policy-open-facts")?;

    // Reserve the proved complete call path on the supervisor bootstrap stack.
    e.cmp(Sp, 0x107400);
    e.0.jump_if(0x82, "fatal");
    e.cmp(Sp, 0x180000);
    e.0.jump_if(0x87, "fatal");
    e.gs(0x8b, A, POLICY_CONTEXT);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.mov(R11, Sp);
    e.arith(4, Sp, -16);
    e.add(Sp, -16);
    e.store(Sp, 0, R11);
    e.0.bytes(&[0xff, 0xd0]); // private C ABI call; R12/R14/R15/BP retained
    e.load(R11, Sp, 0);
    e.imm(D, 0);
    e.store(Sp, 0, D);
    e.mov(Sp, R11);
    for (choice, label) in [
        (1, "namespace-enotdir"),
        (2, "namespace-eisdir"),
        (3, "namespace-erofs"),
    ] {
        e.cmp(A, choice);
        e.0.jump_if(0x84, label);
    }
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    // Pure policy cannot grant authority for a writable description.
    e.mov(A, Bp);
    e.arith(4, A, 3);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    Ok(())
}
