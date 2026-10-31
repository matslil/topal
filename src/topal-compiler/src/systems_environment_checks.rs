//! Executed environment-boundary requests and protected startup payloads.
use super::checks::Witness;
pub(in super::super) const PRE_ENTRY: u32 = 0x4c4000;
pub(in super::super) const POST_ENTRY: u32 = 0x4c8000;
pub(in super::super) const TEMPLATE: u32 = 0x4cc000;
const SOURCE: u64 = 0x607000;
const VECTOR: u64 = 0x607400;
const BAD: u64 = 0x608000;
const BAD_VECTOR: u64 = 0x608800;
const ENTRY: u64 = 0x7ff800;
const ENVIRONMENT: u64 = 0x7ffb20;

fn entries() -> Vec<Vec<u8>> {
    let mut entries = vec![b"TOPAL_TOKEN=alpha".to_vec(), b"TOPAL_TOKEN=beta".to_vec()];
    entries.extend((0..13).map(|_| Vec::new()));
    let mut last = b"LONG=".to_vec();
    last.extend_from_slice(&[b'x'; 970]);
    entries.push(last);
    entries
}
pub(in super::super) fn bytes() -> Vec<u8> {
    let bytes: Vec<_> = entries()
        .into_iter()
        .flat_map(|mut value| {
            value.push(0);
            value
        })
        .collect();
    assert_eq!(bytes.len(), 1024);
    bytes
}
fn fill(w: &mut Witness, address: u64, count: u64, value: u8) {
    w.imm(&[0x48, 0xbf], address);
    w.imm(&[0x48, 0xb9], count);
    w.imm(&[0x48, 0xb8], u64::from(value));
    w.0.extend_from_slice(&[0xfc, 0xf3, 0xaa]);
}
fn exec_error(w: &mut Witness, environment: u64, errno: i64) {
    w.call(59, &[0x5f9000, 0x5f9300, environment], -errno);
    w.call(8, &[4, 0, 1], 7);
    w.call(72, &[10, 1, 0], 1);
}
pub(in super::super) fn pre_code(mode: &str) -> Vec<u8> {
    let mut w = Witness::default();
    w.imm(&[0x48, 0xb8], u64::from(super::observation_checks::ENTRY));
    w.0.extend_from_slice(&[0xff, 0xd0]);
    for pointer in [u64::MAX - 3, 0x7ffffc] {
        exec_error(&mut w, pointer, 14);
    }
    w.store(BAD, u64::from_le_bytes(*b"A=one\0\0\0"));
    w.store(BAD_VECTOR, BAD);
    w.store(BAD_VECTOR + 8, 0x619000);
    w.store(BAD_VECTOR + 16, 0);
    exec_error(&mut w, BAD_VECTOR, 14);
    w.memory(BAD, u64::from_le_bytes(*b"A=one\0\0\0"));
    for i in 0..16 {
        w.store(BAD_VECTOR + i * 8, BAD);
    }
    w.store(BAD_VECTOR + 128, u64::MAX);
    exec_error(&mut w, BAD_VECTOR, 7);
    fill(&mut w, BAD, 1024, b'x');
    w.store(BAD + 1024, 0);
    w.store(BAD_VECTOR, BAD);
    w.store(BAD_VECTOR + 8, 0);
    exec_error(&mut w, BAD_VECTOR, 7);
    fill(&mut w, BAD, 512, b'a');
    w.store(BAD + 512, 0);
    fill(&mut w, BAD + 0x300, 512, b'b');
    w.store(BAD + 0x500, 0);
    w.store(BAD_VECTOR, BAD);
    w.store(BAD_VECTOR + 8, BAD + 0x300);
    w.store(BAD_VECTOR + 16, 0);
    exec_error(&mut w, BAD_VECTOR, 7);
    w.memory(BAD, u64::from_le_bytes([b'a'; 8]));
    if mode == "maximum" {
        for (i, chunk) in bytes().chunks_exact(8).enumerate() {
            w.store(
                SOURCE + i as u64 * 8,
                u64::from_le_bytes(chunk.try_into().unwrap()),
            );
        }
        let mut offset = 0;
        for (i, value) in entries().into_iter().enumerate() {
            w.store(VECTOR + i as u64 * 8, SOURCE + offset);
            offset += value.len() as u64 + 1;
        }
        w.store(VECTOR + 128, 0);
    } else {
        assert!(matches!(mode, "empty" | "null"));
        w.store(VECTOR, 0);
    }
    let code = w.finish();
    assert!(code.len() < (POST_ENTRY - PRE_ENTRY) as usize);
    code
}
fn payloads(w: &mut Witness, count: u64) {
    let auxiliary = ENTRY + 80 + (count + 1) * 8;
    w.memory(ENTRY + 80 + count * 8, 0);
    w.memory(auxiliary, 6);
    w.memory(auxiliary + 8, 4096);
    w.memory(auxiliary + 5 * 16, 31);
    w.memory(auxiliary + 5 * 16 + 8, 0x7fff40);
    w.memory(auxiliary + 12 * 16 + 8, 0x7fff20);
    w.memory(auxiliary + 17 * 16 + 8, 0x7fff30);
    w.memory(auxiliary + 18 * 16, 0);
    w.memory(auxiliary + 18 * 16 + 8, 0);
    w.memory(0x7fff40, u64::from_le_bytes(*b"/bin/bus"));
    w.memory(0x7fff48, u64::from_le_bytes(*b"ybox\0\0\0\0"));
    w.memory(0x7fff20, u64::from_le_bytes(*b"x86_64\0\0"));
    w.imm(&[0x48, 0xbe], 0x7ffa20);
    w.imm(&[0x48, 0xbf], 0x4ab000);
    w.imm(&[0x48, 0xb9], 256);
    w.0.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
    if count != 0 {
        w.imm(&[0x48, 0xbe], ENVIRONMENT);
        w.imm(&[0x48, 0xbf], u64::from(TEMPLATE));
        w.imm(&[0x48, 0xb9], 1024);
        w.0.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
    }
}
pub(in super::super) fn post_code(mode: &str) -> Vec<u8> {
    let mut w = Witness::default();
    w.imm(&[0x48, 0xb8], u64::from(super::observation_checks::ENTRY));
    w.0.extend_from_slice(&[0xff, 0xd0]);
    let count = if mode == "maximum" { 16 } else { 0 };
    if count != 0 {
        // A caller-owned nonzero byte transports progress across destroyed
        // images. The kernel never interprets it or controls this user loop.
        w.0.extend_from_slice(&[0x44, 0x0f, 0xb6, 0x3c, 0x25]);
        w.0.extend_from_slice(&((ENVIRONMENT + 1022) as u32).to_le_bytes());
        w.0.extend_from_slice(&[0x49, 0x83, 0xff, b'x', 0x73, 0x02, 0xeb, 0xfe]);
        w.0.extend_from_slice(&[0x49, 0x83, 0xff, b'z', 0x76, 0x02, 0xeb, 0xfe]);
        w.0.extend_from_slice(&[0xc6, 0x04, 0x25]);
        w.0.extend_from_slice(&((ENVIRONMENT + 1022) as u32).to_le_bytes());
        w.0.push(b'x');
        let mut offset = 0;
        for (i, value) in entries().into_iter().enumerate() {
            w.memory(ENTRY + 80 + i as u64 * 8, ENVIRONMENT + offset);
            offset += value.len() as u64 + 1;
        }
    }
    payloads(&mut w, count);
    // The current user CALL return stays above the written interval; payloads
    // above initial RSP must survive this ordinary 4KiB downward stack use.
    w.0.extend_from_slice(&[
        0x48, 0x81, 0xec, 0, 0x10, 0, 0, 0x48, 0x89, 0xe7, 0xb9, 0, 0x10, 0, 0, 0xb0, 0x5a, 0xfc,
        0xf3, 0xaa, 0x48, 0x81, 0xc4, 0, 0x10, 0, 0,
    ]);
    payloads(&mut w, count);
    if count != 0 {
        repeated_generation(&mut w);
    }
    w.finish()
}

fn repeated_generation(w: &mut Witness) {
    const ACTION: u64 = 0x7fe900;
    const OLD: u64 = 0x7fe940;
    w.call(72, &[10, 1, 0], -9);
    w.call(8, &[4, 0, 1], 7);
    w.store(0x7fe400, 0);
    w.call(79, &[0x7fe400, 64], 5);
    w.memory(0x7fe400, u64::from_le_bytes(*b"/bin\0\0\0\0"));
    for (signal, handler) in [(10, 0), (12, 1)] {
        w.call(13, &[signal, 0, OLD, 8], 0);
        for (offset, value) in [(0, handler), (8, 0), (16, 0), (24, 0)] {
            w.memory(OLD + offset, value);
        }
    }
    w.call(14, &[99, 0, OLD, 8], 0);
    w.memory(OLD, 64);
    // A copied argv prefix followed by a bad env pointer must preserve this
    // generation, and must not emit a successful-commit observation.
    w.call(59, &[0x7fff40, ENTRY + 8, u64::MAX], -14);
    w.call(8, &[4, 0, 1], 7);
    w.0.extend_from_slice(&[0x49, 0x83, 0xff, b'z', 0x0f, 0x84]);
    let skip = w.0.len();
    w.0.extend_from_slice(&[0; 4]);
    w.call(72, &[4, 1030, 10], 10);
    for (offset, value) in [
        (0, 0x12345000),
        (8, u64::MAX),
        (16, 0xdecafbad),
        (24, u64::MAX),
    ] {
        w.store(ACTION + offset, value);
    }
    w.call(13, &[10, ACTION, 0, 8], 0);
    w.0.extend_from_slice(&[0x41, 0xff, 0xc7, 0x44, 0x88, 0x3c, 0x25]);
    w.0.extend_from_slice(&((ENVIRONMENT + 1022) as u32).to_le_bytes());
    for (prefix, value) in [
        (&[0x48, 0xbf][..], 0x7fff40),
        (&[0x48, 0xbe], ENTRY + 8),
        (&[0x48, 0xba], ENTRY + 80),
    ] {
        w.imm(prefix, value);
    }
    w.imm(&[0x48, 0xb8], 59);
    w.0.extend_from_slice(&[0x0f, 0x05, 0x0f, 0x0b]);
    let displacement = i32::try_from(w.0.len() - skip - 4).unwrap();
    w.0[skip..skip + 4].copy_from_slice(&displacement.to_le_bytes());
}
