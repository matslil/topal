//! Native realization of the bounded process byte-transfer services.
//!
//! Specification: TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-{ARGUMENTS,STACK}-001,
//! TOPAL-SYSTEMS-STATIC-PROCESS-CONSOLE-WRITE-001. All addresses below belong
//! to the retained ABI provider, rather than the portable process model.

use super::{CompileError, X86FunctionEncoder, emit_static_process_sysret};

#[path = "systems_process_credentials.rs"]
pub(super) mod credentials;
#[path = "systems_file_descriptions.rs"]
pub(super) mod file_descriptions;
#[path = "systems_process_namespace.rs"]
pub(super) mod namespace;
#[path = "systems_process_observations.rs"]
pub(super) mod observations;
#[path = "systems_process_realtime.rs"]
pub(super) mod realtime;
#[path = "systems_process_signals.rs"]
pub(super) mod signals;

// The pristine segment ends at 0x9728; the rollback stack starts at 0xa000.
// Strings [0,256), pointer offsets [256,320), argc at 320, size at 328.
const STAGING: i32 = 0x9800;
const STAGING_SIZE: i32 = 0x800;
#[path = "systems_exec_environment.rs"]
pub(super) mod environment;
pub(super) const ENTRY: u64 = 0x007f_f800;
pub(super) const ARGUMENT_BASE: u64 = 0x007f_fa20;
pub(super) const ENVIRONMENT_BASE: u64 = 0x007f_fb20;
pub(super) const PLATFORM_BASE: u64 = 0x007f_ff20;
pub(super) const RANDOM_BASE: u64 = 0x007f_ff30;
pub(super) const EXECUTION_NAME: u64 = 0x007f_ff40;
pub(super) const AUX_CONTEXT: u32 = 920;

#[derive(Clone, Copy)]
#[repr(u8)]
enum R {
    A,
    C,
    D,
    B,
    _Sp,
    _Bp,
    Si,
    Di,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}
use R::*;

struct E<'a>(&'a mut X86FunctionEncoder);
impl E<'_> {
    fn rr(&mut self, op: u8, dst: R, src: R) {
        self.0.bytes(&[
            0x48 | (((src as u8) >> 3) << 2) | ((dst as u8) >> 3),
            op,
            0xc0 | (((src as u8) & 7) << 3) | ((dst as u8) & 7),
        ]);
    }
    fn mov(&mut self, dst: R, src: R) {
        self.rr(0x89, dst, src);
    }
    fn imm(&mut self, dst: R, value: u64) {
        self.0
            .bytes(&[0x48 | ((dst as u8) >> 3), 0xb8 | ((dst as u8) & 7)]);
        self.0.bytes(&value.to_le_bytes());
    }
    fn arith(&mut self, op: u8, dst: R, value: i32) {
        self.0.bytes(&[
            0x48 | ((dst as u8) >> 3),
            0x81,
            0xc0 | (op << 3) | ((dst as u8) & 7),
        ]);
        self.0.bytes(&value.to_le_bytes());
    }
    fn add(&mut self, dst: R, n: i32) {
        self.arith(0, dst, n);
    }
    fn cmp(&mut self, dst: R, n: i32) {
        self.arith(7, dst, n);
    }
    fn mem(&mut self, op: u8, reg: R, base: R, offset: i32) {
        self.0.bytes(&[
            0x48 | (((reg as u8) >> 3) << 2) | ((base as u8) >> 3),
            op,
            0x80 | (((reg as u8) & 7) << 3) | ((base as u8) & 7),
        ]);
        if (base as u8) & 7 == 4 {
            self.0.bytes(&[0x24]);
        }
        self.0.bytes(&offset.to_le_bytes());
    }
    fn load(&mut self, dst: R, base: R, offset: i32) {
        self.mem(0x8b, dst, base, offset);
    }
    fn store(&mut self, base: R, offset: i32, src: R) {
        self.mem(0x89, src, base, offset);
    }
    fn gs(&mut self, op: u8, reg: R, offset: u32) {
        self.0.bytes(&[
            0x65,
            0x48 | (((reg as u8) >> 3) << 2),
            op,
            0x04 | (((reg as u8) & 7) << 3),
            0x25,
        ]);
        self.0.bytes(&offset.to_le_bytes());
    }
    fn staging(&mut self) {
        self.gs(0x8b, R12, 904);
        self.add(R12, STAGING);
    }
    fn wipe(&mut self) {
        self.staging();
        self.mov(Di, R12);
        self.imm(A, 0);
        self.imm(C, STAGING_SIZE as u64 / 8);
        self.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
        self.mov(Di, R12);
        self.imm(C, STAGING_SIZE as u64 / 8);
        self.0.bytes(&[0xf3, 0x48, 0xaf]);
        self.0.jump_if(0x85, "fatal");
    }
}

/// Check R8..R8+R9 against the live mappings, including each leaf's present
/// and user bits. This also catches heap holes before/after brk publication.
/// Preserves every register except RAX/RCX/R10/R11; CF reports failure.
pub(super) fn emit_extent_subroutine(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("runtime-readable-extent")?;
    e.imm(C, 5);
    e.0.jump("runtime-extent-check");
    e.0.bind("runtime-writable-extent")?;
    e.imm(C, 7);
    e.0.bind("runtime-extent-check")?;
    e.mov(A, R8);
    e.rr(0x01, A, R9);
    e.0.jump_if(0x82, "runtime-extent-invalid");
    e.cmp(R9, 0);
    e.0.jump_if(0x84, "runtime-extent-valid");
    e.cmp(R8, 0x400000);
    e.0.jump_if(0x82, "runtime-extent-invalid");
    e.cmp(A, 0x619000);
    e.0.jump_if(0x86, "runtime-extent-pages");
    e.cmp(R8, 0x7fe000);
    e.0.jump_if(0x82, "runtime-extent-invalid");
    e.cmp(A, 0x800000);
    e.0.jump_if(0x87, "runtime-extent-invalid");
    e.0.bind("runtime-extent-pages")?;
    e.mov(R10, R8);
    e.arith(4, R10, -4096);
    e.add(A, -1);
    e.arith(4, A, -4096);
    e.0.bind("runtime-extent-page")?;
    // The retained root owns the low leaf and a separate high leaf table.
    e.gs(0x8b, R11, 112);
    e.cmp(R10, 0x600000);
    e.0.jump_if(0x82, "runtime-extent-low-leaf");
    e.gs(0x8b, R11, 24);
    e.add(R11, 0x4000);
    e.0.bind("runtime-extent-low-leaf")?;
    e.0.bytes(&[0x41, 0x52]); // preserve page cursor while indexing leaf
    e.0.bytes(&[0x49, 0xc1, 0xea, 9]); // r10 = page index * 8
    e.arith(4, R10, 0xff8);
    e.rr(0x01, R11, R10);
    e.load(R11, R11, 0);
    e.0.bytes(&[0x41, 0x5a]);
    e.rr(0x21, R11, C);
    e.rr(0x39, R11, C);
    e.0.jump_if(0x85, "runtime-extent-invalid");
    e.rr(0x39, R10, A);
    e.0.jump_if(0x84, "runtime-extent-valid");
    e.add(R10, 4096);
    e.0.jump("runtime-extent-page");
    e.0.bind("runtime-extent-valid")?;
    e.0.bytes(&[0xf8, 0xc3]);
    e.0.bind("runtime-extent-invalid")?;
    e.0.bytes(&[0xf9, 0xc3]);
    Ok(())
}

/// Copy first, decide second, commit in the caller. No user loads precede a
/// complete checked byte/qword extent. No retained process state is changed.
pub(super) fn emit_exec_arguments(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.wipe();
    e.gs(0x8b, B, 312); // pathname
    e.imm(R15, 0);
    e.0.bind("runtime-exec-path-byte")?;
    e.cmp(R15, 256);
    e.0.jump_if(0x83, "runtime-exec-enametoolong");
    e.mov(R8, B);
    e.imm(R9, 1);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.bytes(&[0x8a, 0x03, 0x43, 0x88, 0x04, 0x3c]); // al=[rbx]; [r12+r15]=al
    e.add(R15, 1);
    e.0.bytes(&[0x84, 0xc0]);
    e.0.jump_if(0x84, "runtime-exec-path-copied");
    e.add(B, 1);
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.jump("runtime-exec-path-byte");
    e.0.bind("runtime-exec-path-copied")?;
    for (i, byte) in b"/bin/busybox\0".iter().enumerate() {
        e.0.bytes(&[0x41, 0x80, 0xbc, 0x24]);
        e.0.bytes(&(i as u32).to_le_bytes());
        e.0.bytes(&[*byte]);
        e.0.jump_if(0x85, "runtime-exec-enoent");
    }
    e.wipe();
    e.gs(0x8b, R13, 320);
    e.imm(R14, 0);
    e.imm(R15, 0);
    e.0.bind("runtime-exec-vector-entry")?;
    e.mov(R8, R13);
    e.imm(R9, 8);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.load(B, R13, 0);
    e.cmp(B, 0);
    e.0.jump_if(0x84, "runtime-exec-vector-copied");
    e.cmp(R14, 8);
    e.0.jump_if(0x83, "runtime-exec-e2big");
    // Save offset without retaining any old user pointer in the aggregate.
    e.mov(Di, R14);
    e.0.bytes(&[0x48, 0xc1, 0xe7, 3]);
    e.rr(0x01, Di, R12);
    e.store(Di, 256, R15);
    e.0.bind("runtime-exec-argument-byte")?;
    e.cmp(R15, 256);
    e.0.jump_if(0x83, "runtime-exec-e2big");
    e.mov(R8, B);
    e.imm(R9, 1);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.bytes(&[0x8a, 0x03, 0x43, 0x88, 0x04, 0x3c]);
    e.add(R15, 1);
    e.0.bytes(&[0x84, 0xc0]);
    e.0.jump_if(0x84, "runtime-exec-argument-copied");
    e.add(B, 1);
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.jump("runtime-exec-argument-byte");
    e.0.bind("runtime-exec-argument-copied")?;
    e.add(R14, 1);
    e.add(R13, 8);
    e.0.jump_if(0x82, "runtime-exec-efault");
    e.0.jump("runtime-exec-vector-entry");
    e.0.bind("runtime-exec-vector-copied")?;
    e.cmp(R14, 0);
    e.0.jump_if(0x84, "runtime-exec-einval");
    e.store(R12, 320, R14);
    e.store(R12, 328, R15);
    e.0.call("runtime-exec-copy-environment");
    e.0.jump("runtime-exec-arguments-ready");
    for (label, errno) in [
        ("runtime-exec-efault", 14),
        ("runtime-exec-enametoolong", 36),
        ("runtime-exec-enoent", 2),
        ("runtime-exec-e2big", 7),
        ("runtime-exec-einval", 22),
    ] {
        e.0.bind(label)?;
        e.wipe();
        e.imm(A, (-errno as i64) as u64);
        emit_static_process_sysret(e.0);
    }
    e.0.bind("runtime-exec-arguments-ready")?;
    Ok(())
}

/// Construct this generation's vectors and protected above-entry payloads.
pub(super) fn emit_exec_stack(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    environment::emit_stack(code)
}

/// The first command hands over to a process lifetime; later image generations
/// validate their own completed startup, not the consumed shell command.
pub(super) fn emit_exec_generation_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, A, 896);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "runtime-exec-first-generation");
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.gs(0x8b, A, 104);
    e.cmp(A, 11);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 15);
    e.0.jump_if(0x87, "fatal");
    e.0.jump("runtime-exec-generation-ready");
    e.0.bind("runtime-exec-first-generation")?;
    // These are predecessor qualification facts, not a later-generation token.
    e.gs(0x8b, A, 104);
    e.cmp(A, 32);
    e.0.jump_if(0x85, "fatal");
    e.0.bytes(&[0x65, 0x80, 0x3c, 0x25]);
    e.0.bytes(&418_u32.to_le_bytes());
    e.0.bytes(&[6]);
    e.0.jump_if(0x85, "fatal");
    e.0.bind("runtime-exec-generation-ready")?;
    Ok(())
}

pub(super) fn emit_exec_commit_witness(code: &mut X86FunctionEncoder) {
    let mut e = E(code);
    for byte in b"\nTOPAL_KERNEL_PROCESS_EXEC_COMMITTED\n" {
        e.imm(A, u64::from(*byte));
        e.0.call("runtime-uart-byte");
    }
}

pub(super) fn emit_stack_output_extent(code: &mut X86FunctionEncoder, register_r10: bool) {
    let mut e = E(code);
    let reg = if register_r10 { R10 } else { Si };
    e.cmp(reg, 0x7fe000);
    e.0.jump_if(0x82, "fatal");
    e.mov(A, reg);
    e.add(A, 16);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 0x800000);
    e.0.jump_if(0x87, "fatal");
    // The complete bounded stack must still belong to this active generation
    // and both retained leaves must remain writable, user-accessible and NX.
    e.gs(0x8b, R9, 24);
    e.0.bytes(&[0x0f, 0x20, 0xd8]);
    e.arith(4, A, -4096);
    e.rr(0x39, A, R9);
    e.0.jump_if(0x85, "fatal");
    e.imm(R8, 0x8000_0000_0000_0007);
    for offset in [0x4ff0, 0x4ff8] {
        e.load(A, R9, offset);
        e.rr(0x21, A, R8);
        e.rr(0x39, A, R8);
        e.0.jump_if(0x85, "fatal");
    }
}

/// Select the base used by the existing credential readback offsets.
pub(super) fn emit_credential_base(
    code: &mut X86FunctionEncoder,
    ready: &'static str,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, A, 896);
    e.cmp(A, 1);
    e.0.jump_if(0x85, ready);
    e.gs(0x8b, D, AUX_CONTEXT);
    e.add(D, 96 - 0xf18);
    e.0.bind(ready)?;
    Ok(())
}

pub(super) fn emit_console_write(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("runtime-console-write")?;
    // Ordinary libc startup must have reached getuid before application I/O.
    e.gs(0x8b, A, 104);
    e.cmp(A, 11);
    e.0.jump_if(0x82, "fatal");
    e.cmp(A, 15);
    e.0.jump_if(0x87, "fatal");
    e.cmp(Di, 1);
    e.0.jump_if(0x84, "runtime-write-fd-ready");
    e.cmp(Di, 2);
    e.0.jump_if(0x85, "runtime-write-ebadf");
    e.0.bind("runtime-write-fd-ready")?;
    e.0.bind("runtime-console-payload")?;
    e.cmp(D, 256);
    e.0.jump_if(0x87, "runtime-write-einval");
    e.cmp(D, 0);
    e.0.jump_if(0x84, "runtime-write-success");
    e.mov(R8, Si);
    e.mov(R9, D);
    e.0.call("runtime-readable-extent");
    e.0.jump_if(0x82, "runtime-write-efault");
    e.wipe();
    e.gs(0x8b, Si, 320);
    e.gs(0x8b, C, 328);
    e.mov(Di, R12);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.mov(Si, R12);
    e.gs(0x8b, R14, 328);
    e.0.bind("runtime-write-byte")?;
    e.0.bytes(&[0xac]);
    e.0.call("runtime-uart-byte");
    e.add(R14, -1);
    e.0.jump_if(0x85, "runtime-write-byte");
    e.wipe();
    e.0.bind("runtime-write-success")?;
    e.gs(0x8b, A, 328);
    emit_static_process_sysret(e.0);
    for (label, errno) in [
        ("runtime-write-ebadf", 9),
        ("runtime-write-einval", 22),
        ("runtime-write-efault", 14),
    ] {
        e.0.bind(label)?;
        e.imm(A, (-errno as i64) as u64);
        emit_static_process_sysret(e.0);
    }
    e.0.bind("runtime-uart-byte")?;
    e.0.bytes(&[0x52, 0x50]);
    e.0.bind("runtime-uart-ready")?;
    e.0.bytes(&[0x66, 0xba, 0xfd, 3, 0xec, 0xa8, 0x20]);
    e.0.jump_if(0x84, "runtime-uart-ready");
    e.0.bytes(&[0x66, 0xba, 0xf8, 3, 0x58, 0xee, 0x5a, 0xc3]);
    Ok(())
}

pub(super) fn emit_exit_witness(code: &mut X86FunctionEncoder) {
    let mut e = E(code);
    e.mov(R14, Di);
    e.arith(4, R14, 255);
    for byte in b"\nTOPAL_KERNEL_PROCESS_EXIT_STATUS=" {
        e.imm(A, u64::from(*byte));
        e.0.call("runtime-uart-byte");
    }
    // Hexadecimal digits computed from the actual retained low eight bits.
    for shift in [4, 0] {
        e.mov(A, R14);
        if shift != 0 {
            e.0.bytes(&[0x48, 0xc1, 0xe8, shift]);
        }
        e.arith(4, A, 15);
        e.add(A, 0x30);
        // Branch-free adjustment: '0'..'9', then 'a'..'f'.
        e.cmp(A, 0x3a);
        e.imm(R8, 0);
        e.0.bytes(&[0x41, 0x0f, 0x93, 0xc0]);
        e.0.bytes(&[0x4d, 0x6b, 0xc0, 39]);
        e.rr(0x01, A, R8);
        e.0.call("runtime-uart-byte");
    }
    e.imm(A, 10);
    e.0.call("runtime-uart-byte");
    // The status has been observed; the enclosing rollback consumes no status.
    e.imm(Di, 0);
}

pub(super) fn emit_command_input(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.gs(0x8b, R14, 328);
    e.cmp(R14, 256);
    e.0.jump_if(0x86, "runtime-command-count-ready");
    e.imm(R14, 256);
    e.0.bind("runtime-command-count-ready")?;
    e.wipe();
    e.mov(Si, R12);
    e.imm(R15, 0);
    e.cmp(R14, 0);
    e.0.jump_if(0x84, "runtime-command-received");
    e.0.bind("runtime-command-byte")?;
    e.0.bytes(&[0x66, 0xba, 0xfd, 3, 0xec, 0xa8, 1]);
    e.0.jump_if(0x84, "runtime-command-byte");
    e.0.bytes(&[0x66, 0xba, 0xf8, 3, 0xec, 0x88, 0x06]);
    e.add(Si, 1);
    e.add(R15, 1);
    e.0.bytes(&[0x3c, 10]);
    e.0.jump_if(0x84, "runtime-command-received");
    e.rr(0x39, R15, R14);
    e.0.jump_if(0x82, "runtime-command-byte");
    e.0.bind("runtime-command-received")?;
    e.mov(Si, R12);
    e.gs(0x8b, Di, 320);
    e.mov(C, R15);
    e.0.bytes(&[0xfc, 0xf3, 0xa4]);
    e.wipe();
    e.mov(A, R15);
    Ok(())
}
