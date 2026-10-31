//! Independent executed observations of prepared process and system views.
use super::checks::Witness;
pub(in super::super) const ENTRY: u32 = 0x4d0000;
pub(in super::super) const TEMPLATE: u32 = 0x4d8000;
const BUFFER: u64 = 0x7fe400;

pub(in super::super) fn system_bytes() -> Vec<u8> {
    let mut bytes = vec![0; 390];
    for (index, field) in [
        "Linux",
        "topal",
        "7.2.9-topal",
        "#1 Topal",
        "x86_64",
        "(none)",
    ]
    .into_iter()
    .enumerate()
    {
        bytes[index * 65..index * 65 + field.len()].copy_from_slice(field.as_bytes());
    }
    bytes
}

fn compare(w: &mut Witness, address: u64) {
    w.imm(&[0x48, 0xbe], address);
    w.imm(&[0x48, 0xbf], u64::from(TEMPLATE));
    w.imm(&[0x48, 0xb9], 390);
    w.0.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
}

pub(in super::super) fn code() -> Vec<u8> {
    let mut w = Witness::default();
    // Keep the caller's upper-stack ABI payloads while exercising an exact
    // end-of-mapping destination. This is user-owned fixture scratch.
    w.imm(&[0x48, 0xbe], 0x7ffe00);
    w.imm(&[0x48, 0xbf], 0x7fe800);
    w.imm(&[0x48, 0xb9], 512);
    w.0.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
    // Unknown selectors ignore all arguments and preserve user state. These
    // run before the first exec and after every maximum-environment generation.
    w.store(BUFFER, 0xdecafbad);
    for number in [u64::MAX, 0x7fff_ffff, 0x2000, 0x1_0000_2000, 0x4000_0027] {
        w.imm(&[0x49, 0xb8], u64::MAX - 7);
        w.imm(&[0x49, 0xb9], 0x619000);
        w.call(number, &[u64::MAX, 0x619000, 0x800000, u64::MAX - 1], -38);
        for (bytes, value) in [
            ([0x48, 0x89, 0xf8], u64::MAX),
            ([0x48, 0x89, 0xf0], 0x619000),
            ([0x48, 0x89, 0xd0], 0x800000),
            ([0x4c, 0x89, 0xd0], u64::MAX - 1),
            ([0x4c, 0x89, 0xc0], u64::MAX - 7),
            ([0x4c, 0x89, 0xc8], 0x619000),
        ] {
            w.0.extend_from_slice(&bytes);
            w.expect(value);
        }
        w.memory(BUFFER, 0xdecafbad);
        w.call(0x1_0000_0027, &[], 1); // upper bits must not hide getpid
    }
    for (number, expected) in [(186, 1), (110, 0), (39, 1), (39, 1), (110, 0), (186, 1)] {
        w.call(number, &[u64::MAX, 0x619000, 0, 17], expected);
    }
    for address in [BUFFER, 0x7ff000, 0x800000 - 390] {
        w.call(63, &[address, u64::MAX], 0);
        compare(&mut w, address);
    }
    w.store(0x7ffff8, 0xdecafbad);
    for address in [
        0,
        u64::MAX - 100,
        0x7ffff8,
        0x619000,
        0x4d0000,
        0x800000 - 389,
    ] {
        w.call(63, &[address], -14);
        w.memory(0x7ffff8, 0xdecafbad);
    }
    w.call(63, &[BUFFER], 0);
    compare(&mut w, BUFFER);
    w.imm(&[0x48, 0xbe], 0x7fe800);
    w.imm(&[0x48, 0xbf], 0x7ffe00);
    w.imm(&[0x48, 0xb9], 512);
    w.0.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
    let code = w.finish();
    assert!(code.len() < (TEMPLATE - ENTRY) as usize);
    code
}
