//! Independent user execution of TK-DEC-254 boundaries and parent restoration.
use super::checks::Witness;
pub(in super::super) const ENTRY: u32 = 0x4f0000;
const BUF: u64 = 0x7fe400;
const STATUS: u64 = BUF + 16;
const BASE: u64 = BUF + 32;
const PRIVATE: u64 = 0x5f2000;

fn syscall(w: &mut Witness, number: u64, args: &[u64]) {
    for (prefix, value) in [
        &[0x48, 0xbf][..],
        &[0x48, 0xbe],
        &[0x48, 0xba],
        &[0x49, 0xba],
    ]
    .into_iter()
    .zip(args)
    {
        w.imm(prefix, *value);
    }
    w.imm(&[0x48, 0xb8], number);
    w.0.extend_from_slice(&[0x0f, 0x05]);
}
fn branch_zero(w: &mut Witness) -> usize {
    w.0.extend_from_slice(&[0x48, 0x85, 0xc0, 0x0f, 0x84]);
    let offset = w.0.len();
    w.0.extend_from_slice(&[0; 4]);
    offset
}
fn bind(w: &mut Witness, offset: usize) {
    let relative = i32::try_from(w.0.len() as i64 - offset as i64 - 4).unwrap();
    w.0[offset..offset + 4].copy_from_slice(&relative.to_le_bytes());
}
fn jump(w: &mut Witness) -> usize {
    w.0.push(0xe9);
    let offset = w.0.len();
    w.0.extend_from_slice(&[0; 4]);
    offset
}

pub(in super::super) fn code() -> Vec<u8> {
    // TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-001, TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-WAIT-001,
    // TOPAL-SYSTEMS-STATIC-PROCESS-THREAD-REGISTRATION-001
    let mut w = Witness::default();
    w.0.extend_from_slice(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    w.imm(&[0x48, 0xbf], BUF + 0x400);
    w.0.extend_from_slice(&[0x48, 0x0f, 0xae, 0x07]); // fxsave64
    w.imm(&[0x48, 0xa1], PRIVATE);
    w.imm(&[0x48, 0xa3], BUF + 0x100);
    syscall(&mut w, 8, &[3, 0, 1]);
    w.imm(&[0x48, 0xa3], BUF + 0x108);
    syscall(&mut w, 79, &[BUF + 0x180, 64]);
    w.0.extend_from_slice(&[0x48, 0x85, 0xc0, 0x78, 0xfe]);
    w.call(61, &[u64::MAX, u64::MAX, 0, 0], -10);
    w.call(56, &[0x111, 0, u64::MAX, u64::MAX], -38);
    w.call(56, &[17, 1, u64::MAX, u64::MAX], -38);
    w.call(218, &[u64::MAX], 1);
    w.call(273, &[u64::MAX, 24], 0);
    w.call(273, &[0, 23], -22);
    w.call(158, &[0x1_00001003, BASE], 0);
    w.memory(BASE, 0x5f7400);
    w.call(158, &[0x1002, u64::MAX], -1);
    w.call(158, &[0x1003, u64::MAX], -14);
    w.call(158, &[0x1003, BASE], 0);
    w.memory(BASE, 0x5f7400);
    w.store(PRIVATE, 0x12345678);
    w.store(BUF, 0x87654321);
    w.call(8, &[3, 0, 0], 0);
    w.imm(&[0x48, 0xb8], 0xabcdef1234567890);
    w.0.extend_from_slice(&[0x66, 0x4c, 0x0f, 0x6e, 0xf8]);
    syscall(&mut w, 57, &[u64::MAX, u64::MAX, u64::MAX, u64::MAX]);
    let child = branch_zero(&mut w);
    // Parent's original syscall continuation, memory, TLS and XMM state.
    w.0.extend_from_slice(&[0x48, 0x83, 0xf8, 1, 0x76, 0xfe, 0x48, 0x89, 0xc5]); // rbp=PID
    w.memory(PRIVATE, 0x12345678);
    w.memory(BUF, 0x87654321);
    w.0.extend_from_slice(&[0x66, 0x4c, 0x0f, 0x7e, 0xf8]);
    w.expect(0xabcdef1234567890);
    w.call(158, &[0x1003, BASE], 0);
    w.memory(BASE, 0x5f7400);
    w.call(39, &[], 1);
    w.call(110, &[], 0);
    w.call(72, &[3, 1], 0);
    w.call(8, &[3, 0, 1], 3);
    syscall(&mut w, 79, &[BUF + 64, 64]);
    w.0.extend_from_slice(&[0x48, 0x85, 0xc0, 0x78, 0xfe]);
    w.imm(&[0x48, 0xbe], BUF + 64);
    w.imm(&[0x48, 0xbf], BUF + 0x180);
    w.imm(&[0x48, 0xb9], 64);
    w.0.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
    w.call(57, &[], -11); // unreaped relationship retains the slot
    for (selector, output, options, rusage, expected) in [
        (1, u64::MAX, 0, 0, -10),
        (u64::MAX, u64::MAX, 0, 0, -14),
        (u64::MAX, STATUS, 2, 0, -22),
        (u64::MAX, STATUS, 0, 1, -38),
    ] {
        w.call(61, &[selector, output, options, rusage], expected);
    }
    w.store(STATUS, 0xdecafbad00000000);
    syscall(&mut w, 61, &[u64::MAX, STATUS, 1, 0]);
    w.0.extend_from_slice(&[0x48, 0x39, 0xe8, 0x75, 0xfe]); // exact retained PID
    w.memory(STATUS, 0xdecafbad00000700);
    w.call(61, &[u64::MAX, STATUS, 0, 0], -10);
    let after = jump(&mut w);
    bind(&mut w, child);
    syscall(&mut w, 39, &[]);
    w.0.extend_from_slice(&[0x48, 0x83, 0xf8, 1, 0x76, 0xfe]);
    w.call(110, &[], 1);
    w.call(57, &[], -11);
    w.call(61, &[u64::MAX, 0, 0, 0], -10);
    w.store(PRIVATE, 0xbad);
    w.store(BUF, 0xbad);
    w.store(BUF + 64, u64::from_le_bytes(*b"/\0\0\0\0\0\0\0"));
    w.call(80, &[BUF + 64], 0);
    w.call(8, &[3, 3, 1], 3);
    w.call(72, &[3, 2, 1], 0);
    w.call(3, &[3], 0);
    w.call(158, &[0x1_00001002, 0x5f7500], 0);
    w.call(158, &[0x1003, BASE], 0);
    w.memory(BASE, 0x5f7500);
    w.call(273, &[0, 24], 0);
    w.0.extend_from_slice(&[0x66, 0x45, 0x0f, 0xef, 0xff]);
    syscall(&mut w, 60, &[0x107]);
    w.0.extend_from_slice(&[0x0f, 0x0b]);
    bind(&mut w, after);

    // A second generation uses clone's lower-32-bit flags and inaccessible TID
    // publication: the child still executes and exits, rather than faulting.
    syscall(&mut w, 56, &[0x1_01200011, 0, u64::MAX, u64::MAX]);
    let child = branch_zero(&mut w);
    w.0.extend_from_slice(&[0x48, 0x39, 0xe8, 0x76, 0xfe, 0x48, 0x89, 0xc5]);
    syscall(&mut w, 61, &[0, 0, 0, 0]);
    w.0.extend_from_slice(&[0x48, 0x39, 0xe8, 0x75, 0xfe]);
    w.call(61, &[u64::MAX, 0, 1, 0], -10);
    let after = jump(&mut w);
    bind(&mut w, child);
    w.call(110, &[], 1);
    syscall(&mut w, 231, &[0]);
    w.0.extend_from_slice(&[0x0f, 0x0b]);
    bind(&mut w, after);

    w.store(BUF + 128, 0xfacefeeddecafbad);
    syscall(&mut w, 56, &[0x01200011, 0, u64::MAX, BUF + 128]);
    let child = branch_zero(&mut w);
    w.0.extend_from_slice(&[0x48, 0x39, 0xe8, 0x76, 0xfe, 0x48, 0x89, 0xc5]);
    w.memory(BUF + 128, 0xfacefeeddecafbad); // child-only publication
    w.0.extend_from_slice(&[0x48, 0x89, 0xef]); // native selector is the actual returned PID
    w.imm(&[0x48, 0xbe], 0);
    w.imm(&[0x48, 0xba], 0);
    w.imm(&[0x49, 0xba], 0);
    w.imm(&[0x48, 0xb8], 61);
    w.0.extend_from_slice(&[0x0f, 0x05, 0x48, 0x39, 0xe8, 0x75, 0xfe]);
    let after = jump(&mut w);
    bind(&mut w, child);
    syscall(&mut w, 186, &[]);
    w.0.extend_from_slice(&[0x48, 0x89, 0xc3]);
    w.imm(&[0x48, 0xbe], BUF + 128);
    w.0.extend_from_slice(&[0x8b, 0x06, 0x48, 0x39, 0xd8, 0x75, 0xfe]);
    syscall(&mut w, 60, &[0]);
    w.0.extend_from_slice(&[0x0f, 0x0b]);
    bind(&mut w, after);

    // Park sixteen DISTINCT parent descriptions, then make the child close its
    // inherited slots and open sixteen independent descriptions. This reaches
    // the second global bank while preserving every parked parent reference.
    w.store(BUF + 0x200, u64::from_le_bytes(*b"/bin/bus"));
    w.store(BUF + 0x208, u64::from_le_bytes(*b"ybox\0\0\0\0"));
    w.call(3, &[4], 0);
    for fd in 4..16 {
        w.call(257, &[(-100_i64) as u64, BUF + 0x200, 0, 0], fd);
    }
    syscall(&mut w, 57, &[]);
    let child = branch_zero(&mut w);
    w.0.extend_from_slice(&[0x48, 0x39, 0xe8, 0x76, 0xfe, 0x48, 0x89, 0xc5]);
    syscall(&mut w, 61, &[u64::MAX, 0, 0, 0]);
    w.0.extend_from_slice(&[0x48, 0x39, 0xe8, 0x75, 0xfe]);
    w.call(8, &[3, 0, 1], 3);
    for fd in 5..16 {
        w.call(3, &[fd], 0);
    }
    w.call(3, &[4], 0);
    w.call(33, &[3, 4], 4);
    let after = jump(&mut w);
    bind(&mut w, child);
    for fd in 0..16 {
        w.call(3, &[fd], 0);
    }
    for fd in 0..16 {
        w.call(257, &[(-100_i64) as u64, BUF + 0x200, 0, 0], fd);
    }
    w.call(8, &[15, 4, 0], 4);
    w.call(3, &[3], 0);
    w.call(32, &[0], 3);
    w.call(8, &[0, 6, 0], 6);
    w.call(8, &[3, 0, 1], 6);
    w.call(292, &[15, 3, 0x80000], 3);
    w.call(72, &[3, 1], 1);
    w.call(72, &[15, 1], 0);
    w.call(8, &[3, 0, 1], 4);
    syscall(&mut w, 60, &[0]);
    w.0.extend_from_slice(&[0x0f, 0x0b]);
    bind(&mut w, after);

    w.imm(&[0x48, 0xa1], BUF + 0x100);
    w.imm(&[0x48, 0xa3], PRIVATE);
    w.imm(&[0x48, 0xa1], BUF + 0x108);
    w.0.extend_from_slice(&[0x48, 0x89, 0xc6]);
    w.imm(&[0x48, 0xbf], 3);
    w.imm(&[0x48, 0xba], 0);
    w.imm(&[0x48, 0xb8], 8);
    w.0.extend_from_slice(&[0x0f, 0x05]);
    w.imm(&[0x48, 0xbf], BUF + 0x400);
    w.0.extend_from_slice(&[0x48, 0x0f, 0xae, 0x0f]); // fxrstor64
    w.0.extend_from_slice(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    let bytes = w.finish();
    assert!(bytes.len() < 0x8000);
    bytes
}
