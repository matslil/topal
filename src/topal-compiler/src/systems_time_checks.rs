//! Independent executed native time/interval boundaries; TK-DEC-251.
use super::checks::Witness;
pub(in super::super) const ENTRY: u32 = 0x4e0000;
pub(in super::super) const TEMPLATE: u32 = 0x4e8000;
const BUF: u64 = 0x7fe400;
const INTERVAL: u64 = BUF + 64;
const REMAIN: u64 = BUF + 96;

fn copy(w: &mut Witness, source: u64, destination: u64, count: u64) {
    for (prefix, value) in [
        (&[0x48, 0xbe][..], source),
        (&[0x48, 0xbf][..], destination),
        (&[0x48, 0xb9][..], count),
    ] {
        w.imm(prefix, value);
    }
    w.0.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
}
fn normalized(w: &mut Witness, address: u64) {
    w.imm(&[0x48, 0xa1], address);
    w.0.extend_from_slice(&[0x48, 0x85, 0xc0, 0x78, 0xfe]);
    w.imm(&[0x48, 0xa1], address + 8);
    w.0.extend_from_slice(&[0x48, 0x3d, 0x00, 0xca, 0x9a, 0x3b, 0x73, 0xfe]);
}
fn nanoseconds(w: &mut Witness, address: u64) {
    w.imm(&[0x48, 0xa1], address);
    w.0.extend_from_slice(&[0x48, 0x69, 0xc0, 0x00, 0xca, 0x9a, 0x3b]);
    w.imm(&[0x48, 0xbe], address + 8);
    w.0.extend_from_slice(&[0x48, 0x03, 0x06]);
}

pub(in super::super) fn code() -> Vec<u8> {
    let mut w = Witness::default();
    w.0.extend_from_slice(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    copy(&mut w, 0x7ffe00, 0x7feb00, 512);
    w.call(229, &[0x1_0000_0001, BUF], 0);
    w.memory(BUF, 0);
    w.memory(BUF + 8, 10);
    w.call(229, &[1, 0], 0);
    for number in [228, 229] {
        w.call(number, &[0, u64::MAX], -22);
        for address in [0x619000, u64::MAX, u64::from(ENTRY), 0x7ffff1] {
            w.store(0x7ffff8, 0xdecafbad);
            w.call(number, &[1, address], -14);
            w.memory(0x7ffff8, 0xdecafbad);
        }
        w.call(number, &[1, 0x7ffff0], 0);
        normalized(&mut w, 0x7ffff0);
    }
    w.call(228, &[1, 0], -14);
    w.store(REMAIN, 0xdecafbad);
    w.store(REMAIN + 8, 0x51ab);
    for address in [0, u64::MAX, 0x619000, 0x7ffff1] {
        w.call(35, &[address, REMAIN], -14);
        w.call(230, &[1, 0, address, REMAIN], -14);
    }
    for (seconds, nanos) in [(u64::MAX, 0), (0, u64::MAX), (0, 1_000_000_000)] {
        w.store(INTERVAL, seconds);
        w.store(INTERVAL + 8, nanos);
        w.call(35, &[INTERVAL, REMAIN], -22);
        w.call(230, &[0, 0, INTERVAL, REMAIN], -22);
    }
    // Read-only input and an input ending exactly at the mapping boundary.
    w.call(35, &[u64::from(TEMPLATE), u64::MAX], 0);
    w.store(0x7ffff0, 0);
    w.store(0x7ffff8, 1);
    w.call(230, &[1, 0, 0x7ffff0, u64::MAX], 0);
    for (index, nanos) in [0_u64, 1, 1_000_001, 2_000_000].into_iter().enumerate() {
        w.store(INTERVAL, 0);
        w.store(INTERVAL + 8, nanos);
        w.call(228, &[0x1_0000_0001, BUF], 0);
        normalized(&mut w, BUF);
        nanoseconds(&mut w, BUF);
        w.0.extend_from_slice(&[0x48, 0x89, 0xc3]);
        if index % 2 == 0 {
            w.call(35, &[INTERVAL, REMAIN], 0);
        } else {
            w.call(230, &[0x1_0000_0001, 0x1_0000_0000, INTERVAL, REMAIN], 0);
        }
        w.call(228, &[1, BUF + 16], 0);
        normalized(&mut w, BUF + 16);
        nanoseconds(&mut w, BUF + 16);
        w.0.extend_from_slice(&[0x48, 0x29, 0xd8, 0x72, 0xfe]);
        w.imm(&[0x49, 0xbb], nanos);
        w.0.extend_from_slice(&[0x4c, 0x39, 0xd8, 0x72, 0xfe]);
        w.memory(REMAIN, 0xdecafbad);
        w.memory(REMAIN + 8, 0x51ab);
    }
    copy(&mut w, 0x7feb00, 0x7ffe00, 512);
    w.imm(&[0x48, 0xbf], 0x7feb00);
    w.imm(&[0x48, 0xb9], 512 / 8);
    w.0.extend_from_slice(&[0x31, 0xc0, 0xfc, 0xf3, 0x48, 0xab]);
    w.0.extend_from_slice(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    let bytes = w.finish();
    assert!(bytes.len() < (TEMPLATE - ENTRY) as usize);
    bytes
}
