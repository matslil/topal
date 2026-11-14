//! Invocation-owned positions; TOPAL-SYSTEMS-POSITIONAL-FILE-TRANSFER-001.
use super::*;

pub(super) fn emit(e: &mut E<'_>) -> Result<(), CompileError> {
    for (label, sparse, writing) in [
        ("fd-pread", "file-pread", false),
        ("fd-pwrite", "file-pwrite", true),
    ] {
        e.0.bind(label)?;
        // The source dispatch bridge restored the original fourth argument.
        // Linux checks signed position before attempting descriptor lookup.
        e.cmp(R10, 0);
        e.0.jump_if(0x8c, "fd-einval");
        e.0.call("fd-lookup");
        e.0.jump_if(0x82, "fd-ebadf");
        e.load(B, R14, 0);
        e.cmp(B, super::super::volatile_files::KIND);
        e.0.jump_if(0x84, sparse);
        for kind in [3, 4] {
            e.cmp(B, kind);
            e.0.jump_if(
                0x84,
                if writing {
                    "fd-ebadf"
                } else {
                    "fd-positional-read"
                },
            );
        }
        for kind in [1, 2, 5, 6] {
            e.cmp(B, kind);
            e.0.jump_if(0x84, "fd-espipe");
        }
        e.0.jump("fatal");
    }
    e.0.bind("fd-positional-read")?;
    e.mov(R8, Si);
    e.mov(R9, D);
    e.0.call("fd-native-range");
    e.0.jump_if(0x82, "fd-efault");
    // Captured argument bits survive numerical-range scratch-register use.
    e.gs(0x8b, R15, 336);
    super::super::volatile_files::verify_native_range(e, R15, R9, "fd-einval");
    e.cmp(B, 4);
    e.0.jump_if(0x84, "fd-eisdir");
    e.cmp(R9, 0);
    e.0.jump_if(0x84, "fd-read-zero");
    e.load(A, R12, 24);
    e.rr(0x39, R15, A);
    e.0.jump_if(0x83, "fd-read-zero");
    e.rr(0x29, A, R15);
    e.rr(0x39, R9, A);
    e.0.jump_if(0x86, "fd-pread-remaining-ready");
    e.mov(R9, A);
    e.0.bind("fd-pread-remaining-ready")?;
    e.cmp(R9, 256);
    e.0.jump_if(0x86, "fd-pread-count-ready");
    e.imm(R9, 256);
    e.0.bind("fd-pread-count-ready")?;
    e.0.call("runtime-writable-extent");
    e.0.jump_if(0x82, "fd-efault");
    wipe_staging(e);
    e.load(Si, R12, 16);
    e.rr(0x01, Si, R15);
    e.mov(Di, R12);
    e.add(Di, STAGING);
    e.mov(C, R9);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.mov(Si, R12);
    e.add(Si, STAGING);
    if e.0.owned_user_extents {
        e.mov(Di, R8);
    } else {
        e.gs(0x8b, Di, 320);
    }
    e.mov(C, R9);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    wipe_staging(e);
    e.mov(A, R9);
    return_value(e);
    Ok(())
}
