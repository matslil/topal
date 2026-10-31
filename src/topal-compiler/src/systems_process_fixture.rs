//! Clean ABI witness for the reusable static-process services. This fixture
//! checks transferred values and syscall observations; it is not a shell.

pub(super) const RUNTIME_ENTRY: u32 = 0x004a_a000;
pub(super) const ARGUMENT_BYTES: u32 = 0x004a_b000;
pub(super) const OUTPUT_BYTES: u32 = 0x004a_b200;
#[path = "systems_process_checks.rs"]
pub(super) mod checks;

pub(super) fn arguments() -> Vec<u8> {
    let mut bytes = b"/bin/busybox\0true\0\0".to_vec();
    bytes.extend_from_slice(&[b'x'; 232]);
    bytes.extend_from_slice(&[0; 5]);
    assert_eq!(bytes.len(), 256);
    bytes
}

pub(super) fn output() -> Vec<u8> {
    let mut bytes = b"runtime-clean\n".to_vec();
    bytes.extend_from_slice(&[b'z'; 256]);
    bytes.extend_from_slice(b"runtime-alias\n");
    bytes
}

pub(super) fn code() -> Vec<u8> {
    let mut code = Vec::new();
    // TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-STACK-001: exact maximum-sized
    // aggregate, including empty arguments, entirely reconstructed in the new
    // stack. No pointer may still name an old argument or the private staging.
    for (offset, expected) in [
        (0, 8),
        (8, 0x7fe000),
        (16, 0x7fe00d),
        (24, 0x7fe012),
        (32, 0x7fe013),
        (40, 0x7fe0fc),
        (48, 0x7fe0fd),
        (56, 0x7fe0fe),
        (64, 0x7fe0ff),
        (72, 0),
    ] {
        code.extend_from_slice(&[0x48, 0x81, 0x7c, 0x24, offset]);
        code.extend_from_slice(&u32::to_le_bytes(expected));
        code.extend_from_slice(&[0x75, 0xfe]);
    }
    code.extend_from_slice(&[0xbe, 0, 0xe0, 0x7f, 0, 0xbf]);
    code.extend_from_slice(&ARGUMENT_BYTES.to_le_bytes());
    code.extend_from_slice(&[0xb9, 0, 1, 0, 0, 0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
    code.extend_from_slice(&[0xb8]);
    code.extend_from_slice(&checks::ENTRY.to_le_bytes());
    code.extend_from_slice(&[0xff, 0xd0]);
    // TOPAL-SYSTEMS-STATIC-PROCESS-CONSOLE-WRITE-001: error priority,
    // complete-extent rejection without partial output, empty candidates,
    // both descriptions, and the maximum successful transfer count.
    for (fd, pointer, count, expected) in [
        (0, u64::MAX, 0, -9_i8),
        (1, u64::MAX, 257, -22),
        (1, u64::MAX, 1, -14),
        (1, 0x7fffff, 2, -14),
        (2, u64::MAX, 0, 0),
    ] {
        write_call(&mut code, fd, pointer, count);
        code.extend_from_slice(&[0x48, 0x83, 0xf8, expected as u8, 0x75, 0xfe]);
    }
    write_call(&mut code, 1, u64::from(OUTPUT_BYTES), 14);
    code.extend_from_slice(&[0x48, 0x83, 0xf8, 14, 0x75, 0xfe]);
    write_call(&mut code, 2, u64::from(OUTPUT_BYTES + 14), 256);
    code.extend_from_slice(&[0x48, 0x3d, 0, 1, 0, 0, 0x75, 0xfe]);
    // The observed status is the low eight bits of this actual exit request.
    code.extend_from_slice(&[
        0xbf, 0x23, 1, 0, 0, 0xb8, 60, 0, 0, 0, 0x0f, 0x05, 0x0f, 0x0b,
    ]);
    code
}

fn write_call(code: &mut Vec<u8>, fd: u32, pointer: u64, count: u32) {
    code.push(0xbf);
    code.extend_from_slice(&fd.to_le_bytes());
    code.extend_from_slice(&[0x48, 0xbe]);
    code.extend_from_slice(&pointer.to_le_bytes());
    code.push(0xba);
    code.extend_from_slice(&count.to_le_bytes());
    code.extend_from_slice(&[0xb8, 1, 0, 0, 0, 0x0f, 0x05]);
}
