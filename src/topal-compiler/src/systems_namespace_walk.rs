//! Copied-name lookup with separate final-component facts (TK-DEC-267).
use super::super::volatile_files::{self as files, OBJECTS};
use super::*;
#[cfg(test)]
#[path = "systems_namespace_walk_component.rs"]
mod component;

#[cfg(test)]
pub(in super::super::super) fn component_object(
    namespace_entries: bool,
) -> Result<Vec<u8>, CompileError> {
    component::object(namespace_entries)
}

fn reject_final_dot(e: &mut E<'_>, label: &str) -> Result<(), CompileError> {
    e.load(A, R12, MODE);
    e.cmp(A, 2);
    e.0.jump_if(0x85, label);
    e.load(A, R12, FINAL);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "namespace-eisdir");
    e.0.bind(label)?;
    Ok(())
}
fn regular_component(e: &mut E<'_>) {
    e.cmp(R15, 5);
    e.0.jump_if(0x84, "namespace-enotdir");
    e.cmp(R15, 7);
    e.0.jump_if(0x83, "namespace-enotdir");
}
fn component_facts(e: &mut E<'_>) -> Result<(), CompileError> {
    e.mov(A, Si);
    e.rr(0x29, A, R12);
    e.add(A, -PATH);
    e.store(R12, COMPONENT, A);
    e.store(R12, LENGTH, D);
    e.store(R12, PARENT, R15);
    e.mov(R8, R13);
    e.imm(R9, 1);
    e.imm(R10, 0);
    e.0.bind("namespace-final-scan")?;
    byte(e, A, R8, 0);
    e.cmp(A, 47);
    e.0.jump_if(0x85, "namespace-final-scanned");
    e.imm(R10, 1);
    e.add(R8, 1);
    e.0.jump("namespace-final-scan");
    e.0.bind("namespace-final-scanned")?;
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-final-flags-ready");
    e.imm(R9, 0);
    e.0.bind("namespace-final-flags-ready")?;
    e.store(R12, FINAL, R9);
    e.store(R12, TRAILING, R10);
    Ok(())
}
fn normal_component(e: &mut E<'_>) -> Result<(), CompileError> {
    e.load(A, R12, FINAL);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "namespace-normal-length");
    e.load(A, R12, MODE);
    e.cmp(A, 2);
    e.0.jump_if(0x85, "namespace-normal-open");
    // Linux unlink checks the parent mount before final length/child lookup.
    e.cmp(R15, 6);
    e.0.jump_if(0x85, "namespace-erofs");
    e.0.jump("namespace-normal-length");
    e.0.bind("namespace-normal-open")?;
    e.cmp(A, 1);
    e.0.jump_if(0x85, "namespace-normal-length");
    e.mov(A, Bp);
    e.arith(4, A, 0x40);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-normal-length");
    e.load(A, R12, TRAILING);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "namespace-eisdir");
    e.0.bind("namespace-normal-length")?;
    e.cmp(D, 255);
    e.0.jump_if(0x87, "namespace-enametoolong");
    Ok(())
}
fn lookup_temporary_name(e: &mut E<'_>, namespace_entries: bool) -> Result<(), CompileError> {
    e.0.bind("namespace-tmp-lookup")?;
    if namespace_entries {
        e.0.call("file-entry-by-name");
        e.0.jump_if(0x82, "namespace-missing-component");
        e.0.jump("namespace-component");
        return Ok(());
    }
    e.mov(R10, R12);
    e.add(R10, OBJECTS);
    e.imm(R11, 0);
    e.0.bind("namespace-tmp-name-next")?;
    for (offset, value) in [(0, 1), (56, 1)] {
        e.load(A, R10, offset);
        e.cmp(A, value);
        e.0.jump_if(0x85, "namespace-tmp-name-advance");
    }
    e.mem(0x3b, D, R10, 72);
    e.0.jump_if(0x85, "namespace-tmp-name-advance");
    files::name_pointer(e, R8, R10);
    e.mov(Di, Si);
    e.mov(C, D);
    e.0.bind("namespace-tmp-name-byte")?;
    byte(e, A, R8, 0);
    byte(e, R9, Di, 0);
    e.rr(0x39, A, R9);
    e.0.jump_if(0x85, "namespace-tmp-name-advance");
    e.add(R8, 1);
    e.add(Di, 1);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, "namespace-tmp-name-byte");
    e.load(R15, R10, 8);
    e.0.jump("namespace-component");
    e.0.bind("namespace-tmp-name-advance")?;
    e.add(R10, 128);
    e.add(R11, 1);
    e.cmp(R11, 4);
    e.0.jump_if(0x82, "namespace-tmp-name-next");
    e.0.jump("namespace-missing-component");
    Ok(())
}

pub(super) fn emit(
    e: &mut E<'_>,
    named: bool,
    namespace_entries: bool,
) -> Result<(), CompileError> {
    emit_copy(e, named)?;
    emit_walk(e, named, namespace_entries)
}

fn emit_copy(e: &mut E<'_>, named: bool) -> Result<(), CompileError> {
    // R8 candidate; B dirfd and BP flags survive. R13 is copied length.
    e.0.bind("namespace-copy-name")?;
    table(e);
    wipe_name(e);
    if named {
        e.imm(A, 0);
        for offset in (COMPONENT..=PLAN).step_by(8) {
            e.store(R12, offset, A);
        }
    }
    e.imm(R13, 0);
    e.mov(Si, R12);
    e.add(Si, PATH);
    e.0.bind("namespace-copy-byte")?;
    e.imm(R9, 1);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "namespace-efault");
    byte(e, A, R8, 0);
    e.mem(0x88, A, Si, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-copy-complete");
    e.add(R13, 1);
    e.cmp(R13, 4096);
    e.0.jump_if(0x84, "namespace-enametoolong");
    e.add(R8, 1);
    e.0.jump_if(0x82, "namespace-efault");
    e.add(Si, 1);
    e.0.jump("namespace-copy-byte");
    e.0.bind("namespace-copy-complete")?;
    e.0.bytes(&[0xc3]);
    Ok(())
}

fn emit_walk(e: &mut E<'_>, named: bool, namespace_entries: bool) -> Result<(), CompileError> {
    // Return R15 object. Only private copied bytes are traversed below.
    e.0.bind("namespace-walk")?;
    e.mov(R13, R12);
    e.add(R13, PATH);
    byte(e, A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-enoent");
    e.cmp(A, 47);
    e.0.jump_if(0x84, "namespace-walk-root");
    e.0.bytes(&[0x89, 0xdb]); // native int dirfd
    e.0.bytes(&[0x81, 0xfb, 0x9c, 0xff, 0xff, 0xff]);
    e.0.jump_if(0x84, "namespace-walk-cwd");
    e.mov(Di, B);
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "namespace-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "namespace-enotdir");
    e.load(R15, R14, 32);
    e.mov(R13, R12);
    e.add(R13, PATH);
    e.0.jump("namespace-component");
    e.0.bind("namespace-walk-cwd")?;
    e.load(R15, R12, CWD);
    e.0.jump("namespace-component");
    e.0.bind("namespace-walk-root")?;
    e.imm(R15, 3);
    e.0.bind("namespace-component")?;
    byte(e, A, R13, 0);
    e.cmp(A, 47);
    e.0.jump_if(0x85, "namespace-component-start");
    if named {
        regular_component(e);
    } else {
        e.cmp(R15, 5);
        e.0.jump_if(0x84, "namespace-enotdir");
    }

    e.add(R13, 1);
    e.0.jump("namespace-component");
    e.0.bind("namespace-component-start")?;
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-walk-complete");
    if named {
        regular_component(e);
    } else {
        e.cmp(R15, 5);
        e.0.jump_if(0x84, "namespace-enotdir");
    }

    e.mov(Si, R13);
    e.imm(D, 0);
    e.0.bind("namespace-component-byte")?;
    byte(e, A, R13, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "namespace-component-end");
    e.cmp(A, 47);
    e.0.jump_if(0x84, "namespace-component-end");
    e.add(D, 1);
    if !named {
        e.cmp(D, 255);
        e.0.jump_if(0x87, "namespace-enametoolong");
    }

    e.add(R13, 1);
    e.0.jump("namespace-component-byte");
    e.0.bind("namespace-component-end")?;
    if named {
        component_facts(e)?;
        if namespace_entries {
            // Rename resolves both parents before deciding the final name.
            // Keep dot, trailing-slash, length and child lookup facts intact;
            // the two-path adapter applies their independently qualified order.
            e.load(A, R12, MODE);
            e.cmp(A, 3);
            e.0.jump_if(0x85, "namespace-final-parent-ready");
            e.load(A, R12, FINAL);
            e.cmp(A, 1);
            e.0.jump_if(0x85, "namespace-final-parent-ready");
            e.0.bytes(&[0xc3]);
            e.0.bind("namespace-final-parent-ready")?;
        }
    }
    e.cmp(D, 1);
    e.0.jump_if(0x85, "namespace-check-parent");
    byte(e, A, Si, 0);
    e.cmp(A, 46);
    e.0.jump_if(0x85, "namespace-check-parent");
    if named {
        reject_final_dot(e, "namespace-dot-open-ready")?;
    }
    e.0.jump("namespace-component");
    e.0.bind("namespace-check-parent")?;
    e.cmp(D, 2);
    e.0.jump_if(0x85, "namespace-check-named");
    byte(e, A, Si, 0);
    e.cmp(A, 46);
    e.0.jump_if(0x85, "namespace-check-named");
    byte(e, A, Si, 1);
    e.cmp(A, 46);
    e.0.jump_if(0x85, "namespace-check-named");
    if named {
        reject_final_dot(e, "namespace-parent-open-ready")?;
    }
    e.imm(R15, 3);
    e.0.jump("namespace-component");
    e.0.bind("namespace-check-named")?;
    if named {
        normal_component(e)?;
    }
    e.cmp(R15, 3);
    e.0.jump_if(0x85, "namespace-check-busybox");
    e.cmp(D, 3);
    e.0.jump_if(
        0x85,
        if named {
            "namespace-missing-component"
        } else {
            "namespace-enoent"
        },
    );
    byte(e, A, Si, 0);
    e.cmp(A, i32::from(b'b'));
    e.0.jump_if(0x85, "namespace-check-tmp");
    for (offset, ch) in b"bin".iter().enumerate() {
        byte(e, A, Si, offset as i32);
        e.cmp(A, i32::from(*ch));
        e.0.jump_if(
            0x85,
            if named {
                "namespace-missing-component"
            } else {
                "namespace-enoent"
            },
        );
    }
    e.imm(R15, 4);
    e.0.jump("namespace-component");
    e.0.bind("namespace-check-tmp")?;
    for (offset, ch) in b"tmp".iter().enumerate() {
        byte(e, A, Si, offset as i32);
        e.cmp(A, i32::from(*ch));
        e.0.jump_if(
            0x85,
            if named {
                "namespace-missing-component"
            } else {
                "namespace-enoent"
            },
        );
    }
    e.imm(R15, 6);
    e.0.jump("namespace-component");
    e.0.bind("namespace-check-busybox")?;
    e.cmp(R15, 6);
    e.0.jump_if(
        0x84,
        if named {
            "namespace-tmp-lookup"
        } else {
            "namespace-enoent"
        },
    );
    e.cmp(R15, 4);
    e.0.jump_if(0x85, "fatal");
    e.cmp(D, 7);
    e.0.jump_if(
        0x85,
        if named {
            "namespace-missing-component"
        } else {
            "namespace-enoent"
        },
    );
    for (offset, ch) in b"busybox".iter().enumerate() {
        byte(e, A, Si, offset as i32);
        e.cmp(A, i32::from(*ch));
        e.0.jump_if(
            0x85,
            if named {
                "namespace-missing-component"
            } else {
                "namespace-enoent"
            },
        );
    }
    e.imm(R15, 5);
    e.0.jump("namespace-component");
    e.0.bind("namespace-walk-complete")?;
    if named {
        e.load(A, R12, MODE);
        e.cmp(A, 2);
        e.0.jump_if(0x85, "namespace-walk-return");
        e.load(A, R12, FINAL);
        e.cmp(A, 0);
        e.0.jump_if(0x84, "namespace-eisdir");
        e.0.bind("namespace-walk-return")?;
    }
    e.0.bytes(&[0xc3]);
    if named {
        e.0.bind("namespace-missing-component")?;
        e.load(A, R12, MODE);
        e.cmp(A, 1);
        e.0.jump_if(0x85, "namespace-enoent");
        e.load(A, R12, FINAL);
        e.cmp(A, 1);
        e.0.jump_if(0x85, "namespace-enoent");
        e.imm(R15, 0);
        e.0.bytes(&[0xc3]);
        lookup_temporary_name(e, namespace_entries)?;
    }
    Ok(())
}
