//! Private command input for the bounded first-image shell.
//! TOPAL-SYSTEMS-USER-EXTENT-001.

use super::{
    CompileError, E, R::*, UserExtentLength, X86FunctionEncoder, emit_owned_user_output,
    emit_static_process_sysret,
};

// This range ends at the stack snapshot, after all transient exec staging.
pub(super) const INPUT_OFFSET: i32 = 0x9e00;
const INPUT_SIZE: i32 = 0x200;
const SERIAL_CAPACITY: i32 = 256;
const SEALED_CURSOR: i32 = SERIAL_CAPACITY;
const SERIAL_LENGTH: i32 = SEALED_CURSOR + 8;
const EXIT_PREFIX: i32 = SERIAL_LENGTH + 8;
const EXIT_LINE: &[u8] = b"exit\n";
const EXIT_MISMATCH: u64 = EXIT_LINE.len() as u64 + 1;
const RETURN: &str = "owned-shell-read-return";

pub(in super::super) struct SealedCommand {
    label: &'static str,
    data: &'static [u8],
}

pub(in super::super) const ECHO: SealedCommand = SealedCommand {
    label: "owned-shell-echo-data",
    data: b"echo topal\n",
};
pub(in super::super) const CD: SealedCommand = SealedCommand {
    label: "owned-shell-cd-data",
    data: b"cd /\n",
};
pub(in super::super) const DIRECTORY_TESTS: SealedCommand = SealedCommand {
    label: "owned-shell-directory-tests-data",
    data: b"echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; \
        test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; \
        exec 3</bin/busybox; echo busybox-open; IFS= read -r -n 4 -u 3 _ && echo busybox-read; \
        exec 3<&-; echo busybox-close\n",
};
pub(in super::super) const PWD: SealedCommand = SealedCommand {
    label: "owned-shell-pwd-data",
    data: b"exec 3</bin/busybox;pwd\n",
};

impl E<'_> {
    fn shell_input_request(&mut self) {
        self.gs(0x8b, R15, 328);
        self.imm(A, 0);
        self.cmp(R15, 0);
        self.0.jump_if(0x84, RETURN);
        self.gs(0x8b, R12, 904);
        self.add(R12, INPUT_OFFSET);
    }

    fn shell_input_admission(&mut self) {
        // Limit the caller's count (R15) to the available bytes (R14).
        self.rr(0x39, R15, R14);
        self.0.bytes(&[0x4d, 0x0f, 0x47, 0xfe]); // cmova r15,r14
        self.gs(0x8b, Di, 320);
        emit_owned_user_output(
            self.0,
            Di,
            UserExtentLength::Register(R15),
            "owned-user-buffer-efault",
        );
    }

    fn shell_input_copy(&mut self) {
        self.mov(C, R15);
        self.0.bytes(&[0xfc, 0xf3, 0xa4]); // cld; rep movsb
    }
}

pub(in super::super) fn emit_read(code: &mut X86FunctionEncoder, command: SealedCommand) {
    let mut e = E(code);
    e.shell_input_request();
    e.load(R13, R12, SEALED_CURSOR);
    e.imm(R14, command.data.len() as u64);
    e.rr(0x39, R13, R14);
    e.0.jump_if(0x83, "fatal");
    e.rr(0x29, R14, R13); // available = total - cursor
    e.shell_input_admission();
    e.0.rip_lea(&[0x48, 0x8d, 0x35], command.label);
    e.rr(0x01, Si, R13);
    e.shell_input_copy();

    e.mov(A, R15);
    e.rr(0x01, R13, R15);
    e.imm(B, 0);
    e.rr(0x39, R15, R14);
    e.0.bytes(&[0x4c, 0x0f, 0x44, 0xeb]); // cmove r13,rbx
    e.store(R12, SEALED_CURSOR, R13);
    // The entry commits its completed-publication state on fallthrough.
    e.0.jump_if(0x85, RETURN);
}

pub(in super::super) fn emit_serial_read(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let mut e = E(code);
    e.shell_input_request();
    e.load(R14, R12, SERIAL_LENGTH);
    e.cmp(R14, SERIAL_CAPACITY);
    e.0.jump_if(0x87, "fatal");
    e.cmp(R14, 0);
    e.0.jump_if(0x85, "owned-shell-serial-copy");

    // Receive one bounded fragment when the queue is empty. The matcher
    // classifies the received line. Its result is live only after delivery.
    e.imm(R14, SERIAL_CAPACITY as u64);
    e.rr(0x39, R15, R14);
    e.0.bytes(&[0x4d, 0x0f, 0x47, 0xfe]); // cmova r15,r14
    e.imm(R14, 0);
    e.load(R13, R12, EXIT_PREFIX);
    e.0.rip_lea(&[0x48, 0x8d, 0x1d], "owned-shell-exit-line");
    e.mov(Si, R12);
    e.0.bind("owned-shell-serial-receive")?;
    e.0.bytes(&[0x66, 0xba, 0xfd, 3, 0xec, 0xa8, 1]);
    e.0.jump_if(0x84, "owned-shell-serial-receive");
    e.0.bytes(&[0x66, 0xba, 0xf8, 3, 0xec, 0x88, 0x06]);
    e.cmp(R13, EXIT_LINE.len() as i32);
    e.0.jump_if(0x83, "owned-shell-serial-mismatch");
    e.0.bytes(&[0x42, 0x3a, 0x04, 0x2b]); // cmp al,[rbx+r13]
    e.0.jump_if(0x85, "owned-shell-serial-mismatch");
    e.add(R13, 1);
    e.0.jump("owned-shell-serial-matched");
    e.0.bind("owned-shell-serial-mismatch")?;
    e.imm(R13, EXIT_MISMATCH);
    e.0.bind("owned-shell-serial-matched")?;
    e.add(Si, 1);
    e.add(R14, 1);
    e.0.bytes(&[0x3c, 10]);
    e.0.jump_if(0x84, "owned-shell-serial-received");
    e.rr(0x39, R14, R15);
    e.0.jump_if(0x82, "owned-shell-serial-receive");
    e.0.bind("owned-shell-serial-received")?;
    e.store(R12, SERIAL_LENGTH, R14);
    e.store(R12, EXIT_PREFIX, R13);

    e.0.bind("owned-shell-serial-copy")?;
    e.shell_input_admission();
    e.mov(Si, R12);
    e.shell_input_copy();
    e.imm(R13, 0);
    e.0.bytes(&[0x80, 0x7e, 0xff, 10, 0x41, 0x0f, 0x94, 0xc5]); // last byte is newline
    // Move the remaining suffix down and erase the vacated bytes.
    e.rr(0x29, R14, R15);
    e.mov(Di, R12);
    e.mov(C, R14);
    e.0.bytes(&[0xf3, 0xa4]);
    e.mov(C, R15);
    e.imm(A, 0);
    e.0.bytes(&[0xf3, 0xaa]);
    e.store(R12, SERIAL_LENGTH, R14);
    e.mov(A, R15);
    e.cmp(R13, 0);
    e.0.jump_if(0x84, RETURN);
    Ok(())
}

pub(in super::super) fn emit_require_exit(code: &mut X86FunctionEncoder) {
    let mut e = E(code);
    e.gs(0x8b, R8, 904);
    e.load(A, R8, INPUT_OFFSET + EXIT_PREFIX);
    e.cmp(A, EXIT_LINE.len() as i32);
    e.0.jump_if(0x85, "fatal");
}

pub(in super::super) fn emit_retire(code: &mut X86FunctionEncoder) {
    let mut e = E(code);
    e.gs(0x8b, R8, 904);
    e.add(R8, INPUT_OFFSET);
    e.mov(Di, R8);
    e.imm(A, 0);
    e.imm(C, INPUT_SIZE as u64 / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    e.mov(Di, R8);
    e.imm(C, INPUT_SIZE as u64 / 8);
    e.0.bytes(&[0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fatal");
}

pub(in super::super) fn emit_data_and_return(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    code.bind(RETURN)?;
    emit_static_process_sysret(code);
    for command in [ECHO, CD, DIRECTORY_TESTS, PWD] {
        code.bind(command.label)?;
        code.bytes(command.data);
    }
    code.bind("owned-shell-exit-line")?;
    code.bytes(EXIT_LINE);
    Ok(())
}
