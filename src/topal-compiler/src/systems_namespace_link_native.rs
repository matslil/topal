//! Full old lookup, then destination creation, write/mount and inode eligibility.
use super::*;

pub(super) fn emit(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("namespace-transaction-link-source")?;
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "namespace-link-empty-source");
    e.0.call("namespace-walk"); // MODE0, including final/trailing lookup
    e.cmp(R15, 7);
    e.0.jump_if(0x82, "namespace-link-nonowned-source");
    e.store(R12, SOURCE_ENTRY, R11);
    e.0.call("file-object-by-identity");
    e.0.jump("namespace-link-owned-source");
    e.0.bind("namespace-link-empty-source")?;
    e.mov(A, Bp);
    e.arith(4, A, 0x1000);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-enoent");
    e.0.bytes(&[0x89, 0xdb]);
    e.0.bytes(&[0x81, 0xfb, 0x9c, 0xff, 0xff, 0xff]);
    e.0.jump_if(0x84, "namespace-link-empty-cwd");
    e.mov(Di, B);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "namespace-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, files::KIND);
    e.0.jump_if(0x84, "namespace-link-file-description");
    e.cmp(A, 4);
    e.0.jump_if(0x84, "namespace-link-directory-description");
    e.cmp(A, 3);
    e.0.jump_if(0x85, "namespace-enosys"); // other inode kinds remain unqualified
    e.imm(R15, 5);
    e.0.jump("namespace-link-nonowned-source");
    e.0.bind("namespace-link-empty-cwd")?;
    e.load(R15, R12, CWD);
    e.0.jump("namespace-link-nonowned-source");
    e.0.bind("namespace-link-directory-description")?;
    e.load(R15, R14, 32);
    e.0.jump("namespace-link-nonowned-source");
    e.0.bind("namespace-link-file-description")?;
    e.imm(A, 4);
    e.store(R12, SOURCE_ENTRY, A);
    e.0.call("file-object-lookup");
    e.0.bind("namespace-link-owned-source")?;
    save_source_object(e);
    e.imm(A, 6);
    e.store(R12, OLD_PARENT, A);
    e.0.jump("namespace-transaction-new-path");
    e.0.bind("namespace-link-nonowned-source")?;
    e.store(R12, SOURCE_ID, R15);
    e.imm(A, 3);
    e.cmp(R15, 6);
    e.0.jump_if(0x85, "namespace-link-source-mount");
    e.imm(A, 6);
    e.0.bind("namespace-link-source-mount")?;
    e.store(R12, OLD_PARENT, A);
    e.0.jump("namespace-transaction-new-path");

    e.0.bind("namespace-transaction-link-target")?;
    e.mov(B, R15); // actual destination parent, including root-only disposition
    new_component(e);
    e.load(A, R12, FINAL);
    e.0.call("namespace-transaction-final-kind");
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-eexist");
    e.0.call("namespace-transaction-final-lookup");
    e.cmp(R15, 0);
    e.0.jump_if(0x85, "namespace-eexist");
    e.load(A, R12, TRAILING);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-enoent");
    e.cmp(B, 6);
    e.0.jump_if(0x85, "namespace-erofs");
    e.load(A, R12, OLD_PARENT);
    e.cmp(A, 6);
    e.0.jump_if(0x85, "namespace-exdev");
    e.load(R15, R12, SOURCE_ID);
    e.cmp(R15, 7);
    e.0.jump_if(0x82, "namespace-eperm"); // same-mount directory cannot be linked
    e.0.call("file-object-by-identity");
    e.load(A, R10, 56);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-link-eligible");
    e.load(A, R10, 64);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "namespace-enoent");
    e.0.bind("namespace-link-eligible")?;
    e.0.call("file-entry-free");
    e.0.jump_if(0x82, "namespace-enospc");
    e.mov(R15, R11);
    e.imm(R14, 4);
    e.imm(R10, 0);
    e.imm(Bp, 0);
    plan_source(e);
    committed(e);
    Ok(())
}
