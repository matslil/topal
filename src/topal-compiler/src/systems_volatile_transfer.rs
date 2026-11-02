//! Native byte/size realization; TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001.
use super::super::R::_Bp as Bp;
use super::*;

fn clipped(e: &mut E<'_>, prefix: &str, remaining: super::super::R) -> Result<(), CompileError> {
    e.mov(Bp, R9);
    e.cmp(Bp, 256);
    let quantum = format!("{prefix}-quantum");
    let page = format!("{prefix}-page");
    let end = format!("{prefix}-end");
    e.0.jump_if(0x86, &quantum);
    e.imm(Bp, 256);
    e.0.bind(&quantum)?;
    e.mov(A, R15);
    e.arith(4, A, 4095);
    e.imm(C, 4096);
    e.rr(0x29, C, A);
    e.rr(0x39, Bp, C);
    e.0.jump_if(0x86, &page);
    e.mov(Bp, C);
    e.0.bind(&page)?;
    e.rr(0x39, Bp, remaining);
    e.0.jump_if(0x86, &end);
    e.mov(Bp, remaining);
    e.0.bind(&end)?;
    Ok(())
}
fn emit_read(e: &mut E<'_>, positional: bool, source_position: bool) -> Result<(), CompileError> {
    let prefix = if positional {
        "file-pread"
    } else {
        "file-read"
    };
    e.0.bind(prefix)?;
    e.load(A, R14, 16);
    e.arith(4, A, 3);
    e.cmp(A, 1);
    e.0.jump_if(0x84, "file-ebadf");
    e.mov(R8, Si);
    e.mov(R9, D);
    e.0.call("fd-native-range");
    e.0.jump_if(0x82, "file-efault");
    e.0.call("file-object-lookup");
    e.mov(B, R10);
    if positional {
        e.gs(0x8b, R15, 336);
    } else {
        e.load(R15, R14, 24);
    }
    verify_native_range(e, R15, R9, "file-einval");
    e.cmp(R9, 0);
    e.0.jump_if(0x84, "file-zero");
    e.load(R13, R10, 16);
    e.rr(0x39, R15, R13);
    e.0.jump_if(0x83, "file-zero");
    e.rr(0x29, R13, R15);
    clipped(e, prefix, R13)?;
    e.mov(R9, Bp);
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "file-efault");
    wipe_staging(e);
    e.mov(R10, B);
    e.0.call("file-page-lookup");
    e.0.jump_if(0x82, &format!("{prefix}-staged"));
    payload(e, Si, C);
    e.mov(A, R15);
    e.arith(4, A, 4095);
    e.rr(0x01, Si, A);
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.mov(C, Bp);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.0.bind(&format!("{prefix}-staged"))?;
    e.mov(Di, R8);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.mov(C, Bp);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    if source_position && !positional {
        super::super::super::file_region::emit_position_update(e.0, Bp);
    } else {
        e.rr(0x01, R15, Bp);
        if !positional {
            e.store(R14, 24, R15);
        }
    }
    wipe_staging(e);
    e.mov(A, Bp);
    emit_static_process_sysret(e.0);

    Ok(())
}

fn emit_write(e: &mut E<'_>, positional: bool, source_position: bool) -> Result<(), CompileError> {
    let prefix = if positional {
        "file-pwrite"
    } else {
        "file-write"
    };
    e.0.bind(prefix)?;
    e.load(A, R14, 16);
    e.arith(4, A, 3);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "file-ebadf");
    e.mov(R8, Si);
    e.mov(R9, D);
    e.0.call("fd-native-range");
    e.0.jump_if(0x82, "file-efault");
    e.0.call("file-object-lookup");
    e.mov(B, R10);
    if positional {
        e.gs(0x8b, R15, 336);
    } else {
        e.load(R15, R14, 24);
    }
    verify_native_range(e, R15, R9, "file-einval");
    e.cmp(R9, 0);
    e.0.jump_if(0x84, "file-zero");
    e.load(A, R14, 16);
    e.arith(4, A, 0x400);
    e.cmp(A, 0);
    e.0.jump_if(0x84, &format!("{prefix}-position"));
    e.load(R15, R10, 16);
    e.0.bind(&format!("{prefix}-position"))?;
    e.imm(R13, i64::MAX as u64);
    e.rr(0x39, R15, R13);
    e.0.jump_if(0x84, "file-efbig");
    e.rr(0x29, R13, R15);
    clipped(e, prefix, R13)?;
    e.mov(R9, Bp);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "file-efault");
    e.mov(R10, B);
    e.0.call("file-page-lookup");
    e.0.jump_if(0x83, &format!("{prefix}-page-ready"));
    e.mov(R11, R12);
    e.add(R11, PAGES);
    e.imm(R9, 0);
    e.0.bind(&format!("{prefix}-free-page"))?;
    e.load(A, R11, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, &format!("{prefix}-free-ready"));
    e.add(R11, 32);
    e.add(R9, 1);
    e.cmp(R9, 8);
    e.0.jump_if(0x82, &format!("{prefix}-free-page"));
    e.0.jump("file-enospc");
    e.0.bind(&format!("{prefix}-free-ready"))?;
    e.mov(C, R9);
    e.0.bind(&format!("{prefix}-page-ready"))?;
    payload(e, Di, C);
    e.mov(Si, R11);
    // A free association's payload is clean before it receives an identity.
    e.load(A, Si, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x85, &format!("{prefix}-page-checked"));
    e.0.bytes(&[0x57]);
    e.imm(A, 0);
    e.imm(C, 4096 / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf, 0x5f]);
    e.0.jump_if(0x85, "fatal");
    for offset in [8, 16, 24] {
        e.load(A, Si, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fatal");
    }
    e.0.bind(&format!("{prefix}-page-checked"))?;
    e.0.call("file-clock");
    e.0.jump_if(0x82, "file-eio");
    e.mov(R13, A);
    // The complete input prefix was admitted. Its private copy precedes any
    // page identity, payload, logical-size or shared-position publication.
    e.0.bytes(&[0x57]);
    e.mov(R11, Si);
    e.mov(Si, R8);
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.mov(C, Bp);
    e.0.bytes(&[0xfc, 0xf3, 0xa4, 0x5f]);
    e.load(A, B, 8);
    e.store(R11, 0, A);
    e.mov(A, R15);
    shift(e, A, false, 12);
    e.store(R11, 8, A);
    e.mov(A, R15);
    e.arith(4, A, 4095);
    e.rr(0x01, Di, A);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    e.mov(C, Bp);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    if source_position && !positional {
        super::super::super::file_region::emit_position_update(e.0, Bp);
    } else {
        e.rr(0x01, R15, Bp);
        if !positional {
            e.store(R14, 24, R15);
        }
    }
    e.load(A, B, 16);
    e.rr(0x39, R15, A);
    e.0.jump_if(0x86, &format!("{prefix}-size-ready"));
    e.store(B, 16, R15);
    e.0.bind(&format!("{prefix}-size-ready"))?;
    e.store(B, 40, R13);
    e.store(B, 48, R13);
    wipe_staging(e);
    e.mov(A, Bp);
    emit_static_process_sysret(e.0);
    Ok(())
}

pub(super) fn emit(
    code: &mut X86FunctionEncoder,
    source_position: bool,
) -> Result<(), CompileError> {
    let mut e = E(code);
    emit_read(&mut e, false, source_position)?;
    emit_read(&mut e, true, source_position)?;
    emit_write(&mut e, false, source_position)?;
    emit_write(&mut e, true, source_position)?;
    e.0.bind("file-zero")?;
    e.imm(A, 0);
    emit_static_process_sysret(e.0);
    emit_seek(&mut e)?;
    emit_truncate(&mut e)?;
    emit_stat(&mut e)?;
    Ok(())
}

fn emit_seek(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("file-seek")?;
    e.0.call("file-object-lookup");
    e.mov(B, R10);
    e.cmp(D, 2);
    e.0.jump_if(0x87, "file-seek-extents");
    e.imm(A, 0);
    e.cmp(D, 0);
    e.0.jump_if(0x84, "file-seek-add");
    e.cmp(D, 1);
    e.0.jump_if(0x85, "file-seek-end");
    e.load(A, R14, 24);
    e.0.jump("file-seek-add");
    e.0.bind("file-seek-end")?;
    e.load(A, R10, 16);
    e.0.bind("file-seek-add")?;
    e.rr(0x01, A, Si);
    e.0.jump_if(0x80, "file-einval");
    e.cmp(A, 0);
    e.0.jump_if(0x8c, "file-einval");
    e.mov(R15, A);
    e.0.jump("file-seek-publish");
    e.0.bind("file-seek-extents")?;
    e.cmp(Si, 0);
    e.0.jump_if(0x8c, "file-enxio");
    e.load(A, R10, 16);
    e.rr(0x39, Si, A);
    e.0.jump_if(0x83, "file-enxio");
    e.mov(R15, Si);
    e.0.call("file-page-lookup");
    e.0.jump_if(0x82, "file-seek-absent");
    e.cmp(D, 3);
    e.0.jump_if(0x84, "file-seek-publish");
    e.0.bind("file-seek-next-hole")?;
    e.arith(4, R15, -4096);
    e.add(R15, 4096);
    e.load(A, R10, 16);
    e.rr(0x39, R15, A);
    e.0.jump_if(0x83, "file-seek-hole-eof");
    e.0.call("file-page-lookup");
    e.0.jump_if(0x83, "file-seek-next-hole");
    e.0.jump("file-seek-publish");
    e.0.bind("file-seek-hole-eof")?;
    e.mov(R15, A);
    e.0.jump("file-seek-publish");
    e.0.bind("file-seek-absent")?;
    e.cmp(D, 4);
    e.0.jump_if(0x84, "file-seek-publish");
    e.mov(R9, R15);
    e.load(R15, R10, 16);
    e.mov(R11, R12);
    e.add(R11, PAGES);
    e.imm(C, 8);
    e.0.bind("file-seek-data-next")?;
    e.load(A, R11, 0);
    e.mem(0x3b, A, R10, 8);
    e.0.jump_if(0x85, "file-seek-data-advance");
    e.load(A, R11, 8);
    shift(e, A, true, 12);
    e.rr(0x39, A, R9);
    e.0.jump_if(0x82, "file-seek-data-advance");
    e.rr(0x39, A, R15);
    e.0.jump_if(0x83, "file-seek-data-advance");
    e.mov(R15, A);
    e.0.bind("file-seek-data-advance")?;
    e.add(R11, 32);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, "file-seek-data-next");
    e.load(A, R10, 16);
    e.rr(0x39, R15, A);
    e.0.jump_if(0x84, "file-enxio");
    e.0.bind("file-seek-publish")?;
    e.store(R14, 24, R15);
    e.mov(A, R15);
    emit_static_process_sysret(e.0);
    Ok(())
}

fn emit_truncate(e: &mut E<'_>) -> Result<(), CompileError> {
    e.0.bind("file-truncate")?;
    e.cmp(Si, 0);
    e.0.jump_if(0x8c, "file-einval");
    e.0.call("fd-lookup");
    e.0.jump_if(0x82, "file-ebadf");
    e.load(A, R14, 0);
    e.cmp(A, KIND);
    e.0.jump_if(0x85, "file-einval");
    e.load(A, R14, 16);
    e.arith(4, A, 3);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "file-einval");
    e.0.call("file-object-lookup");
    e.mov(B, R10);
    e.mov(R15, Si);
    e.load(A, R10, 16);
    e.rr(0x39, R15, A);
    e.0.jump_if(0x84, "file-zero");
    e.0.call("file-clock");
    e.0.jump_if(0x82, "file-eio");
    e.mov(R13, A);
    e.load(A, B, 16);
    e.rr(0x39, R15, A);
    e.0.jump_if(0x87, "file-truncate-publish");
    e.mov(R11, R12);
    e.add(R11, PAGES);
    e.imm(R9, 0);
    e.0.bind("file-truncate-page")?;
    e.load(A, R11, 0);
    e.mem(0x3b, A, B, 8);
    e.0.jump_if(0x85, "file-truncate-next");
    e.load(A, R11, 8);
    shift(e, A, true, 12);
    e.rr(0x39, A, R15);
    e.0.jump_if(0x83, "file-truncate-release-page");
    e.mov(R10, R15);
    e.rr(0x29, R10, A);
    e.cmp(R10, 4096);
    e.0.jump_if(0x83, "file-truncate-next");
    payload(e, R8, R9);
    e.rr(0x01, R8, R10);
    for verify in [false, true] {
        e.mov(Di, R8);
        e.imm(C, 4096);
        e.rr(0x29, C, R10);
        e.imm(A, 0);
        e.0.bytes(if verify {
            &[0xfc, 0xf3, 0xae]
        } else {
            &[0xfc, 0xf3, 0xaa]
        });
        if verify {
            e.0.jump_if(0x85, "fatal");
        }
    }
    e.0.jump("file-truncate-next");
    e.0.bind("file-truncate-release-page")?;
    payload(e, R8, R9);
    erase(e, R8, 4096);
    erase(e, R11, 32);
    e.0.bind("file-truncate-next")?;
    e.add(R11, 32);
    e.add(R9, 1);
    e.cmp(R9, 8);
    e.0.jump_if(0x82, "file-truncate-page");
    e.0.bind("file-truncate-publish")?;
    e.store(B, 16, R15);
    e.store(B, 40, R13);
    e.store(B, 48, R13);
    e.0.jump("file-zero");
    Ok(())
}

fn emit_stat(e: &mut E<'_>) -> Result<(), CompileError> {
    // The shared fstat path already admitted output and cleared native staging.
    e.0.bind("file-stat")?;
    e.0.call("file-object-lookup");
    e.0.bind("file-stat-object")?;
    e.imm(A, 4096);
    e.store(R12, STAGING + 56, A);
    e.imm(A, 2);
    e.store(R12, STAGING, A);
    e.load(A, R10, 8);
    e.store(R12, STAGING + 8, A);
    e.load(A, R10, 56);
    e.store(R12, STAGING + 16, A);
    e.load(A, R10, 24);
    e.arith(1, A, 0o100000);
    e.store(R12, STAGING + 24, A);
    e.load(A, R10, 16);
    e.store(R12, STAGING + 48, A);
    for (source, target) in [(32, 72), (40, 88), (48, 104)] {
        e.load(A, R10, source);
        e.store(R12, STAGING + target, A);
    }
    e.imm(R15, 0);
    e.mov(R11, R12);
    e.add(R11, PAGES);
    e.imm(C, 8);
    e.0.bind("file-stat-page")?;
    e.load(A, R11, 0);
    e.mem(0x3b, A, R10, 8);
    e.0.jump_if(0x85, "file-stat-page-next");
    e.add(R15, 8);
    e.0.bind("file-stat-page-next")?;
    e.add(R11, 32);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, "file-stat-page");
    e.store(R12, STAGING + 64, R15);
    e.0.jump("fd-stat-publish-to-r8");
    Ok(())
}
