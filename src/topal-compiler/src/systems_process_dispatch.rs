//! Source operation selection between captured entry and native handler routing.
use super::R::_Sp as Sp;
use super::numeric::{ARENA_BYTES, ARENA_CONTEXT, erase, zero_readback};
use super::{CompileError, E, R::*, X86FunctionEncoder};

pub(in super::super) const CODE_CONTEXT: u32 = 984;

pub(in super::super) fn emit_invoke(
    code: &mut X86FunctionEncoder,
    extended: bool,
) -> Result<(), CompileError> {
    let mut e = E(code);
    // The target ABI has already normalized A. Its original six argument words
    // and complete user continuation are retained in the captured context.
    e.mov(R8, A);
    e.cmp(Sp, 0x108000);
    e.0.jump_if(0x82, "fatal");
    e.cmp(Sp, 0x180000);
    e.0.jump_if(0x87, "fatal");
    e.0.bytes(&[0x9c, 0x58]);
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    e.0.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    e.add(Sp, -16);
    e.gs(0x8b, R11, ARENA_CONTEXT);
    e.cmp(R11, 0);
    e.0.jump_if(0x84, "fatal");
    zero_readback(&mut e, R11, ARENA_BYTES);
    e.gs(0x8b, A, 256);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.store(R11, 0, A);
    e.imm(A, 32);
    e.store(R11, 8, A);
    e.imm(A, 3);
    e.store(R11, 16, A);
    e.mov(Di, R8);
    e.gs(0x8b, A, CODE_CONTEXT);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fatal");
    e.mov(R11, Sp);
    e.arith(4, Sp, -16);
    e.add(Sp, -16);
    e.store(Sp, 0, R11);
    e.0.bytes(&[0xff, 0xd0]);
    e.load(R11, Sp, 0);
    e.mov(Sp, R11);
    // These are the bridge's named operation encodings. Never route the
    // captured untrusted selector after this point, including unsupported input.
    let policy = if extended {
        crate::systems_numeric_library::NumericPolicy::LinuxDispatchV2
    } else {
        crate::systems_numeric_library::NumericPolicy::LinuxDispatch
    };
    for &(_, operation) in policy.dispatch_choices() {
        e.cmp(
            A,
            if operation == u64::MAX {
                -1
            } else {
                operation as i32
            },
        );
        e.0.jump_if(0x84, "static-process-source-dispatch-result-valid");
    }
    e.0.jump("fatal");
    e.0.bind("static-process-source-dispatch-result-valid")?;
    e.store(Sp, 0, A);
    e.gs(0x8b, R11, ARENA_CONTEXT);
    e.gs(0x8b, A, 256);
    e.load(R10, R11, 0);
    e.rr(0x39, A, R10);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 16);
    e.cmp(A, 3);
    e.0.jump_if(0x85, "fatal");
    e.load(A, R11, 8);
    e.cmp(A, 32);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fatal");
    erase(&mut e, R11, ARENA_BYTES);
    e.load(R8, Sp, 0);
    e.add(Sp, 16);
    e.0.bytes(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    e.mov(R11, Sp);
    e.add(R11, -4096);
    erase(&mut e, R11, 4096);
    for register in [A, C, D, Si, Di, R9, R10, R11] {
        e.imm(register, 0);
    }
    e.mov(A, R8);
    e.imm(R8, 0);
    // Reload captured facts without dereferencing any user candidate. Native
    // argument validation remains solely with the selected qualified handler.
    for (register, offset) in [
        (C, 296),
        (R11, 304),
        (Di, 312),
        (Si, 320),
        (D, 328),
        (R10, 336),
        (R8, 344),
        (R9, 352),
    ] {
        e.gs(0x8b, register, offset);
    }
    Ok(())
}
