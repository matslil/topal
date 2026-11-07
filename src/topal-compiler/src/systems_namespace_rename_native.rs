//! Ordered two-parent rename admission; no persistent stores here.
use super::*;

pub(super) fn emit(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-transaction-rename-target")?;
    // Root and bin share a readonly mount; tmp is a distinct mutable mount.
    e.load(B, R12, OLD_PARENT);
    e.cmp(B, 6);
    e.0.jump_if(0x84, "namespace-rename-old-mutable");
    e.cmp(R15, 6);
    e.0.jump_if(0x84, "namespace-exdev");
    e.0.jump("namespace-rename-mounts-ready");
    e.0.bind("namespace-rename-old-mutable")?;
    e.cmp(R15, 6);
    e.0.jump_if(0x85, "namespace-exdev");
    e.0.bind("namespace-rename-mounts-ready")?;
    e.mov(Si, R12);
    e.add(Si, OLD_NAME);
    e.load(D, R12, OLD_LENGTH);
    e.load(A, R12, OLD_FINAL);
    e.0.call("namespace-transaction-final-kind");
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-ebusy");
    new_component(e);
    e.load(A, R12, FINAL);
    e.0.call("namespace-transaction-final-kind");
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-rename-final-kinds-ready");
    e.mov(A, Bp);
    e.arith(4, A, 1);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-eexist");
    e.0.jump("namespace-ebusy");
    e.0.bind("namespace-rename-final-kinds-ready")?;
    e.cmp(B, 6);
    e.0.jump_if(0x85, "namespace-erofs");
    e.load(D, R12, OLD_LENGTH);
    e.cmp(D, 255);
    e.0.jump_if(0x87, "namespace-enametoolong");
    e.mov(Si, R12);
    e.add(Si, OLD_NAME);
    e.0.call("file-entry-by-name");
    e.0.jump_if(0x82, "namespace-enoent");
    e.store(R12, SOURCE_ENTRY, R11);
    e.0.call("file-object-by-identity");
    save_source_object(e);
    new_component(e);
    e.imm(R15, 6);
    e.0.call("namespace-transaction-final-lookup");
    e.mov(R13, R11); // target entry survives checked object lookup
    e.cmp(R15, 0);
    e.0.jump_if(0x84, "namespace-rename-target-missing");
    e.mov(A, Bp);
    e.arith(4, A, 1);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-eexist");
    trailing(e, "namespace-rename-existing-trailing-ready")?;
    e.mem(0x3b, R15, R12, SOURCE_ID);
    e.0.jump_if(0x84, "namespace-transaction-success");
    e.0.call("file-object-by-identity");
    e.mov(R14, R10);
    e.rr(0x29, R14, R12);
    e.add(R14, -files::OBJECTS);
    files::shift(e, R14, false, 7);
    e.load(R10, R10, 8); // victim identity, not a writable record address
    e.mov(R15, R13); // target entry index
    e.cmp(Bp, 2);
    e.0.jump_if(0x84, "namespace-rename-exchange");
    e.imm(Bp, 2);
    e.0.jump("namespace-rename-plan");
    e.0.bind("namespace-rename-exchange")?;
    e.imm(Bp, 3);
    e.0.jump("namespace-rename-plan");
    e.0.bind("namespace-rename-target-missing")?;
    e.cmp(Bp, 2);
    e.0.jump_if(0x84, "namespace-enoent");
    trailing(e, "namespace-rename-missing-trailing-ready")?;
    e.imm(R14, 4);
    e.imm(R10, 0);
    e.load(R15, R12, SOURCE_ENTRY); // a move reuses its slot at full capacity
    e.imm(Bp, 1);
    e.0.bind("namespace-rename-plan")?;
    plan_source(e);
    committed(e);
    Ok(())
}

fn trailing(e: &mut E<'_>, ready: &str) -> Result<(), CompileError> {
    // Both final lookups (including NOREPLACE/EXCHANGE) precede these checks.
    for offset in [OLD_TRAILING, TRAILING] {
        e.load(A, R12, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "namespace-enotdir");
    }
    e.0.bind(ready)?;
    Ok(())
}
