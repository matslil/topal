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
