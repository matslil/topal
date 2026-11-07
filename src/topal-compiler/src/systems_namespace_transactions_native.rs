//! TK-DEC-281: native admission; only the ordinary source publishes namespace.
//! TOPAL-SYSTEMS-NAMESPACE-{ABI,ACTION,TRANSACTION}-001.
use super::super::volatile_files as files;
use super::*;

#[cfg(test)]
#[path = "systems_namespace_final_component.rs"]
mod component;
#[path = "systems_namespace_link_native.rs"]
mod link;
#[path = "systems_namespace_transaction_commit.rs"]
mod publication;

#[cfg(test)]
pub(in super::super::super) fn final_component_object() -> Result<Vec<u8>, CompileError> {
    component::object()
}
#[path = "systems_namespace_rename_native.rs"]
mod rename;

// Pre-admission only; the source input replaces and retires this representation.
const OLD_NAME: i32 = files::DIRECTORY_STAGING;
const OLD_PARENT: i32 = OLD_NAME + 256;
const OLD_LENGTH: i32 = OLD_NAME + 264;
const OLD_TRAILING: i32 = OLD_NAME + 272;
const OLD_FINAL: i32 = OLD_NAME + 280;
const SOURCE_OBJECT: i32 = OLD_NAME + 288;
const SOURCE_ID: i32 = OLD_NAME + 296;
const SOURCE_ENTRY: i32 = OLD_NAME + 304;
const SOURCE_LINKS: i32 = OLD_NAME + 312;
// Copy/walk clear facts192..240 but preserve these captured argument words.
const NEW_POINTER: i32 = STAGING;
const NEW_DIRECTORY: i32 = STAGING + 8;
const LINK_OPERATION: i32 = STAGING + 16;

pub(super) fn emit(e: &mut E<'_>) -> Result<(), CompileError> {
    for (label, link, at, flags) in [
        ("namespace-rename", false, false, false),
        ("namespace-link", true, false, false),
        ("namespace-renameat", false, true, false),
        ("namespace-linkat", true, true, true),
        ("namespace-renameat2", false, true, true),
    ] {
        e.0.bind(label)?;
        if flags {
            e.mov(Bp, R8);
        } else {
            e.imm(Bp, 0);
        }
        if at {
            e.mov(R8, Si);
            e.mov(B, Di);
            e.mov(R9, R10);
            e.mov(R10, D);
        } else {
            e.mov(R8, Di);
            e.imm(B, 0xffff_ff9c);
            e.mov(R9, Si);
            e.imm(R10, 0xffff_ff9c);
        }
        e.imm(R15, u64::from(link));
        e.0.jump("namespace-transaction-captured");
    }
    e.0.bind("namespace-transaction-captured")?;
    table(e);
    e.0.bytes(&[0x89, 0xed]); // native unsigned/int flags: low32 only
    e.store(R12, NEW_POINTER, R9);
    e.store(R12, NEW_DIRECTORY, R10);
    e.store(R12, LINK_OPERATION, R15);
    e.cmp(R15, 1);
    e.0.jump_if(0x84, "namespace-transaction-link-flags");
    e.mov(A, Bp);
    e.arith(4, A, !7);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-einval");
    e.mov(A, Bp);
    e.arith(4, A, 2);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-transaction-rename-flags-ready");
    e.mov(A, Bp);
    e.arith(4, A, 5);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-einval");
    e.0.bind("namespace-transaction-rename-flags-ready")?;
    e.mov(A, Bp);
    e.arith(4, A, 4);
    e.cmp(A, 0);
    // Whiteout device publication remains explicitly unqualified, never an
    // ordinary successful rename. Invalid EXCHANGE combinations took priority.
    e.0.jump_if(0x85, "namespace-enosys");
    e.0.jump("namespace-transaction-flags-ready");
    e.0.bind("namespace-transaction-link-flags")?;
    e.mov(A, Bp);
    e.arith(4, A, !0x1400);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "namespace-einval");
    e.0.bind("namespace-transaction-flags-ready")?;
    named::wipe_directory(e);
    e.0.call("namespace-copy-name");
    e.load(A, R12, LINK_OPERATION);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "namespace-transaction-link-source");
    e.imm(A, 3);
    e.store(R12, MODE, A);
    e.0.call("namespace-walk");
    e.store(R12, OLD_PARENT, R15);
    for (from, to) in [
        (LENGTH, OLD_LENGTH),
        (TRAILING, OLD_TRAILING),
        (FINAL, OLD_FINAL),
    ] {
        e.load(A, R12, from);
        e.store(R12, to, A);
    }
    e.load(D, R12, LENGTH);
    e.cmp(D, 255);
    e.0.jump_if(0x87, "namespace-transaction-new-path");
    e.load(Si, R12, COMPONENT);
    e.rr(0x01, Si, R12);
    e.add(Si, PATH);
    e.mov(Di, R12);
    e.add(Di, OLD_NAME);
    e.mov(C, D);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    // A long final name is retained as a length fact, not copied past256 or
    // rejected before the second parent/mount/write ordering has been applied.
    e.0.bind("namespace-transaction-new-path")?;
    e.load(R8, R12, NEW_POINTER);
    e.load(B, R12, NEW_DIRECTORY);
    e.0.call("namespace-copy-name");
    e.imm(A, 3);
    e.store(R12, MODE, A);
    e.0.call("namespace-walk");
    e.load(A, R12, LINK_OPERATION);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "namespace-transaction-link-target");
    e.0.jump("namespace-transaction-rename-target");

    rename::emit(e)?;
    link::emit(e)?;
    publication::emit(e)?;
    e.0.bind("namespace-transaction-success")?;
    retire_admission(e);
    named::wipe_directory(e);
    finish(e, 0);
    Ok(())
}

/// R10 checked live object. Only stable indexes/identities are staged; the
/// source later checks them again against actual owned bytes before stores.
fn save_source_object(e: &mut E<'_>) {
    e.mov(A, R10);
    e.rr(0x29, A, R12);
    e.add(A, -files::OBJECTS);
    files::shift(e, A, false, 7);
    e.store(R12, SOURCE_OBJECT, A);
    e.load(A, R10, 8);
    e.store(R12, SOURCE_ID, A);
    e.load(A, R10, 56);
    e.store(R12, SOURCE_LINKS, A);
}

fn new_component(e: &mut E<'_>) {
    e.load(Si, R12, COMPONENT);
    e.rr(0x01, Si, R12);
    e.add(Si, PATH);
    e.load(D, R12, LENGTH);
}

fn plan_source(e: &mut E<'_>) {
    e.load(B, R12, SOURCE_OBJECT);
    e.load(R13, R12, SOURCE_ENTRY);
    e.load(R8, R12, LENGTH);
    e.load(R9, R12, SOURCE_ID);
    e.load(R11, R12, SOURCE_LINKS);
    e.load(Si, R12, COMPONENT);
    e.rr(0x01, Si, R12);
    e.add(Si, PATH);
}

fn committed(e: &mut E<'_>) {
    e.0.call("file-clock");
    e.0.jump_if(0x82, "namespace-transaction-clock-failed");
    e.mov(D, A);
    retire_clock_stack(e);
    e.0.call("namespace-transaction-commit");
    // The commit helper has returned; retire its now-completed return word.
    clear_return(e);
    e.0.jump("namespace-transaction-success");
}

fn retire_clock_stack(e: &mut E<'_>) {
    // No syscall handler frame is below current SP. Preserve the scalar plan
    // and D seconds while retiring the completed clock and lookup call tree.
    e.mov(Di, super::super::R::_Sp);
    e.add(Di, -4096);
    e.imm(A, 0);
    e.imm(C, 512);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    e.mov(Di, super::super::R::_Sp);
    e.add(Di, -4096);
    e.imm(C, 512);
    e.0.bytes(&[0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
}

pub(super) fn retire_admission(e: &mut E<'_>) {
    // Earlier helpers can reach the common errno exit without returning.
    // Restore only the trusted captured kernel stack, never a candidate from
    // pathname/user metadata, before retiring the whole abandoned call area.
    e.gs(0x8b, super::super::R::_Sp, 8);
    e.cmp(super::super::R::_Sp, 0x108000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(super::super::R::_Sp, 0x180000);
    e.0.jump_if(0x87, "fatal");
    e.mov(A, super::super::R::_Sp);
    e.arith(4, A, 7);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    retire_clock_stack(e);
}

fn clear_return(e: &mut E<'_>) {
    e.mem(0xc7, A, super::super::R::_Sp, -8);
    e.0.bytes(&[0; 4]);
    e.mem(0x81, Di, super::super::R::_Sp, -8);
    e.0.bytes(&[0; 4]);
    e.0.jump_if(0x85, "fatal");
}
