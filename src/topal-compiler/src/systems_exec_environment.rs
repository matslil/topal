//! Owned envp bytes and above-entry payloads; TK-DEC-245.
use super::{
    ARGUMENT_BASE, AUX_CONTEXT, CompileError, E, ENTRY, ENVIRONMENT_BASE, EXECUTION_NAME,
    PLATFORM_BASE, R::*, RANDOM_BASE, X86FunctionEncoder,
};
const ENV_COUNT: i32 = 336;
const ENV_SIZE: i32 = 344;
const ENV_OFFSETS: i32 = 352;
const ENV_BYTES: i32 = 480;

pub(in super::super) fn emit_copy(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("runtime-exec-copy-environment")?;
    e.staging();
    e.gs(0x8b, R13, 328);
    e.imm(R14, 0);
    e.imm(R15, 0);
    e.cmp(R13, 0);
    e.0.jump_if(0x84, "runtime-environment-copied");
    e.0.bind("runtime-environment-entry")?;
    e.mov(R8, R13);
    e.imm(R9, 8);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.load(B, R13, 0);
    e.cmp(B, 0);
    e.0.jump_if(0x84, "runtime-environment-copied");
    e.cmp(R14, 16);
    e.0.jump_if(0x83, "runtime-exec-e2big");
    e.mov(Di, R14);
    e.0.bytes(&[0x48, 0xc1, 0xe7, 3]);
    e.rr(0x01, Di, R12);
    e.store(Di, ENV_OFFSETS, R15);
    e.0.bind("runtime-environment-byte")?;
    e.cmp(R15, 1024);
    e.0.jump_if(0x83, "runtime-exec-e2big");
    e.mov(R8, B);
    e.imm(R9, 1);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.bytes(&[0x8a, 0x03, 0x43, 0x88, 0x84, 0x3c]);
    e.0.bytes(&ENV_BYTES.to_le_bytes());
    e.add(R15, 1);
    e.0.bytes(&[0x84, 0xc0]);
    e.0.jump_if(0x84, "runtime-environment-string");
    e.add(B, 1);
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.jump("runtime-environment-byte");
    e.0.bind("runtime-environment-string")?;
    e.add(R14, 1);
    e.add(R13, 8);
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.jump("runtime-environment-entry");
    e.0.bind("runtime-environment-copied")?;
    e.store(R12, ENV_COUNT, R14);
    e.store(R12, ENV_SIZE, R15);
    e.0.bytes(&[0xc3]);
    Ok(())
}

pub(super) fn emit_stack(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.staging();
    e.load(R14, R12, 320);
    e.load(C, R12, 328);
    e.mov(Si, R12);
    e.imm(Di, ARGUMENT_BASE);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.load(C, R12, ENV_SIZE);
    e.mov(Si, R12);
    e.add(Si, ENV_BYTES);
    e.imm(Di, ENVIRONMENT_BASE);
    e.0.bytes(&[0xf3, 0xa4]);
    // The admitted executable name is independent of caller-provided argv[0].
    for (offset, chunk) in b"/bin/busybox\0\0\0\0".chunks_exact(8).enumerate() {
        e.imm(A, u64::from_le_bytes(chunk.try_into().unwrap()));
        e.imm(Di, EXECUTION_NAME + offset as u64 * 8);
        e.store(Di, 0, A);
    }
    e.imm(Di, ENTRY);
    e.store(Di, 0, R14);
    e.add(Di, 8);
    e.mov(B, R12);
    e.add(B, 256);
    e.mov(C, R14);
    e.0.bind("runtime-exec-stack-argv")?;
    e.load(A, B, 0);
    e.add(A, ARGUMENT_BASE as i32);
    e.store(Di, 0, A);
    e.add(B, 8);
    e.add(Di, 8);
    e.add(C, -1);
    e.0.jump_if(0x85, "runtime-exec-stack-argv");
    e.add(Di, 8); // explicit zero argv terminator from fresh cleared stack
    e.load(C, R12, ENV_COUNT);
    e.cmp(C, 0);
    e.0.jump_if(0x84, "runtime-exec-stack-env-done");
    e.mov(B, R12);
    e.add(B, ENV_OFFSETS);
    e.0.bind("runtime-exec-stack-env")?;
    e.load(A, B, 0);
    e.add(A, ENVIRONMENT_BASE as i32);
    e.store(Di, 0, A);
    e.add(B, 8);
    e.add(Di, 8);
    e.add(C, -1);
    e.0.jump_if(0x85, "runtime-exec-stack-env");
    e.0.bind("runtime-exec-stack-env-done")?;
    e.add(Di, 8);
    e.gs(0x89, Di, AUX_CONTEXT);
    for (tag, value) in [
        (6, 4096),
        (9, 0x4a6580),
        (3, 0x400040),
        (4, 56),
        (5, 11),
        (31, EXECUTION_NAME),
        (11, 0),
        (12, 0),
        (13, 0),
        (14, 0),
        (23, 0),
        (17, 100),
        (15, PLATFORM_BASE),
        (16, 0x178bfbfd),
        (26, 0),
        (7, 0),
        (8, 0),
        (25, RANDOM_BASE),
        (0, 0),
    ] {
        e.imm(A, tag);
        e.store(Di, 0, A);
        e.imm(A, value);
        e.store(Di, 8, A);
        e.add(Di, 16);
    }
    // Eight args/sixteen env entries plus nineteen aux pairs end below fa20.
    e.cmp(Di, ARGUMENT_BASE as i32);
    e.0.jump_if(0x87, "fatal");
    e.wipe();
    Ok(())
}
