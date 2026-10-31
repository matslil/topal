//! Executed user-mode syscall witnesses, independent of provider implementation.
//! Covers file-description and signal-registration rules TK-DEC-240/241.
pub(in super::super) const ENTRY: u32 = 0x004a_c000;
pub(in super::super) const PRE_EXEC: u32 = 0x004b_2000;
const BUF: u64 = 0x7fe400;
const STAT: u64 = 0x7fe600;
const ACTION: u64 = 0x7fe900;
const OLD: u64 = 0x7fe940;
const MASK: u64 = !0x40100_u64;

#[derive(Default)]
struct Witness(Vec<u8>);
impl Witness {
    fn imm(&mut self, prefix: &[u8], value: u64) {
        self.0.extend_from_slice(prefix);
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn store(&mut self, address: u64, value: u64) {
        self.imm(&[0x48, 0xb8], value);
        self.imm(&[0x48, 0xa3], address);
    }
    fn expect(&mut self, value: u64) {
        self.imm(&[0x49, 0xbb], value);
        self.0.extend_from_slice(&[0x4c, 0x39, 0xd8, 0x75, 0xfe]);
    }
    fn memory(&mut self, address: u64, value: u64) {
        self.imm(&[0x48, 0xa1], address);
        self.expect(value);
    }
    fn call(&mut self, number: u64, args: &[u64], expected: i64) {
        for (prefix, value) in [
            &[0x48, 0xbf][..],
            &[0x48, 0xbe],
            &[0x48, 0xba],
            &[0x49, 0xba],
        ]
        .into_iter()
        .zip(args)
        {
            self.imm(prefix, *value);
        }
        self.imm(&[0x49, 0xbc], 0x51ab);
        self.imm(&[0x48, 0xb8], number);
        self.0.extend_from_slice(&[0x0f, 0x05]);
        self.expect(expected as u64);
        self.0
            .extend_from_slice(&[0x49, 0x81, 0xfc, 0xab, 0x51, 0, 0, 0x75, 0xfe]);
    }
    fn action(&mut self, handler: u64) {
        for (offset, value) in [
            (0, handler),
            (8, u64::MAX),
            (16, 0xdecafbad),
            (24, u64::MAX),
        ] {
            self.store(ACTION + offset, value);
        }
    }
    fn query_action(&mut self, signal: u64, handler: u64, flags: u64, restorer: u64, mask: u64) {
        self.call(13, &[signal, 0, OLD, 8], 0);
        for (offset, value) in [(0, handler), (8, flags), (16, restorer), (24, mask)] {
            self.memory(OLD + offset, value);
        }
    }
    fn finish(mut self) -> Vec<u8> {
        // Erase all scratch without touching the user CALL return address.
        self.imm(&[0x48, 0xbf], BUF);
        self.imm(&[0x48, 0xb9], 0x600 / 8);
        self.0
            .extend_from_slice(&[0x31, 0xc0, 0xfc, 0xf3, 0x48, 0xab, 0xc3]);
        self.0
    }
}

pub(in super::super) fn pre_exec() -> Vec<u8> {
    let mut w = Witness::default();
    w.call(72, &[3, 1030, 10], 10);
    w.call(32, &[3], 4);
    w.call(8, &[4, 7, 0], 7);
    w.action(0x12345000);
    w.call(13, &[10, ACTION, 0, 8], 0);
    w.action(1);
    w.call(13, &[12, ACTION, 0, 8], 0);
    w.store(ACTION, 64);
    w.call(14, &[2, ACTION, 0, 8], 0);
    w.call(59, &[0, 0x5f9300, 0x5f90a0], -14);
    w.call(8, &[3, 0, 1], 7);
    w.call(72, &[10, 1, 0], 1);
    w.query_action(10, 0x12345000, 0xdc000807, 0xdecafbad, MASK);
    w.query_action(12, 1, 0xdc000807, 0xdecafbad, MASK);
    w.call(14, &[99, 0, OLD, 8], 0);
    w.memory(OLD, 64);
    w.finish()
}

pub(in super::super) fn code() -> Vec<u8> {
    let mut w = Witness::default();
    // Failed exec preserved all state; successful exec filters only CLOEXEC,
    // caught dispositions and metadata, retaining ignored handlers/mask/offsets.
    w.call(72, &[10, 1, 0], -9);
    w.call(8, &[4, 0, 1], 7);
    w.call(3, &[4], 0);
    w.call(8, &[3, 0, 0], 0);
    w.query_action(10, 0, 0, 0, 0);
    w.query_action(12, 1, 0, 0, 0);
    w.call(14, &[99, 0, OLD, 8], 0);
    w.memory(OLD, 64);
    w.store(ACTION, 0);
    w.call(14, &[2, ACTION, 0, 8], 0);
    // Native resource/metadata values and complete-copy rejection.
    w.call(97, &[7, STAT], 0);
    w.memory(STAT, 16);
    w.memory(STAT + 8, 16);
    w.call(5, &[3, STAT], 0);
    for (offset, value) in [
        (8, 5),
        (24, 0o100755),
        (48, 2_024_544),
        (56, 4096),
        (64, 3955),
    ] {
        w.memory(STAT + offset, value);
    }
    w.call(5, &[99, u64::MAX], -9);
    w.call(5, &[3, 0x7ffff0], -14);
    w.call(0, &[3, BUF, 4], 4);
    w.0.extend_from_slice(&[0x81, 0x3c, 0x25]);
    w.0.extend_from_slice(&(BUF as u32).to_le_bytes());
    w.0.extend_from_slice(&[0x7f, b'E', b'L', b'F', 0x75, 0xfe]);
    w.call(32, &[3], 4);
    w.call(0, &[4, BUF, 1], 1);
    w.0.extend_from_slice(&[0x80, 0x3c, 0x25]);
    w.0.extend_from_slice(&(BUF as u32).to_le_bytes());
    w.0.extend_from_slice(&[2, 0x75, 0xfe]);
    w.call(8, &[3, 0, 1], 5);
    w.call(72, &[4, 2, 1], 0);
    w.call(72, &[3, 1, 0], 0);
    w.call(72, &[4, 1, 0], 1);
    w.call(72, &[4, 3, 0], 0x8000);
    w.call(33, &[99, 3], -9);
    w.call(8, &[3, 0, 1], 5);
    w.call(33, &[4, 4], 4);
    w.call(72, &[4, 1, 0], 1);
    for (source, target, flags, errno) in [(4, 4, 0, -22), (99, 3, 1, -22), (3, 16, 0, -9)] {
        w.call(292, &[source, target, flags], errno);
    }
    w.call(72, &[3, 0, 16], -22);
    w.call(0, &[3, 0x7ffffe, 4], -14);
    w.call(0, &[3, u64::MAX, 4], -14);
    w.call(8, &[3, 0, 1], 5);
    w.call(0, &[3, 0, 0], 0);
    w.call(0, &[3, u64::MAX, 0], -14);
    w.call(8, &[3, u64::MAX, 0], -22);
    w.call(8, &[3, i64::MAX as u64, 0], i64::MAX);
    w.call(8, &[3, 1, 1], -22);
    w.call(0, &[3, BUF, 1], 0);
    w.call(8, &[3, (-2_i64) as u64, 2], 2_024_542);
    w.call(0, &[3, BUF, 256], 2);
    w.call(0, &[3, BUF, 256], 0);
    // A mutable loaded byte must never become immutable file backing.
    w.0.extend_from_slice(&[0x0f, 0xb6, 0x04, 0x25, 0, 0xd0, 0x5e, 0]);
    w.imm(&[0x48, 0xa3], 0x7fe800);
    w.0.extend_from_slice(&[0xc6, 0x04, 0x25, 0, 0xd0, 0x5e, 0, 0xa5]);
    w.call(8, &[3, 0x1ed000, 0], 0x1ed000);
    w.call(0, &[3, BUF, 1], 1);
    w.0.extend_from_slice(&[0x0f, 0xb6, 0x04, 0x25]);
    w.0.extend_from_slice(&(BUF as u32).to_le_bytes());
    w.0.extend_from_slice(&[0x3a, 0x04, 0x25, 0, 0xe8, 0x7f, 0, 0x75, 0xfe]);
    w.imm(&[0x48, 0xa1], 0x7fe800);
    w.0.extend_from_slice(&[0xa2, 0, 0xd0, 0x5e, 0, 0, 0, 0, 0]);
    for fd in 5..16 {
        w.call(32, &[3], fd);
    }
    w.call(32, &[3], -24);
    w.call(3, &[7], 0);
    w.call(72, &[3, 0, 4], 7);
    for fd in 4..16 {
        w.call(3, &[fd], 0);
    }
    w.call(33, &[3, 2], 2);
    w.call(8, &[2, 0, 0], 0);
    w.call(0, &[2, BUF, 4], 4);
    w.call(33, &[1, 2], 2);
    w.call(32, &[1], 4);
    w.call(1, &[4, 0x4ab30e, 14], 14);
    w.call(3, &[4], 0);
    w.call(1, &[4, u64::MAX, 1], -9);
    signal_checks(&mut w);
    let code = w.finish();
    assert!(code.len() < (PRE_EXEC - ENTRY) as usize);
    code
}

fn signal_checks(w: &mut Witness) {
    // Size precedes access; input precedes signal/how validation. Native int
    // arguments ignore their upper bits. Queries ignore a missing selector.
    w.call(13, &[99, u64::MAX, 0, 7], -22);
    w.call(13, &[99, u64::MAX, 0, 8], -14);
    w.call(13, &[9, 0, OLD, 8], 0);
    w.action(0xfeedface);
    w.call(13, &[9, ACTION, 0, 8], -22);
    w.call(13, &[19, ACTION, 0, 8], -22);
    w.call(13, &[0, ACTION, 0, 8], -22);
    w.call(13, &[65, ACTION, 0, 8], -22);
    w.call(13, &[0x10000000a, ACTION, ACTION, 8], 0);
    for offset in (0..32).step_by(8) {
        w.memory(ACTION + offset, 0);
    }
    w.query_action(10, 0xfeedface, 0xdc000807, 0xdecafbad, MASK);
    w.action(1);
    w.call(13, &[10, ACTION, 0x7ffff0, 8], -14);
    w.query_action(10, 1, 0xdc000807, 0xdecafbad, MASK);
    w.call(13, &[10, u64::MAX, 0, 8], -14);
    w.query_action(10, 1, 0xdc000807, 0xdecafbad, MASK);
    w.call(14, &[99, u64::MAX, 0, 7], -22);
    w.call(14, &[99, u64::MAX, 0, 8], -14);
    w.store(ACTION, u64::MAX);
    w.call(14, &[99, ACTION, 0, 8], -22);
    w.call(14, &[2, ACTION, ACTION, 8], 0);
    w.memory(ACTION, 0);
    w.call(14, &[99, 0, OLD, 8], 0);
    w.memory(OLD, MASK);
    w.store(ACTION, 2);
    w.call(14, &[1, ACTION, OLD, 8], 0);
    w.memory(OLD, MASK);
    w.call(14, &[99, 0, OLD, 8], 0);
    w.memory(OLD, MASK & !2);
    w.call(14, &[0, ACTION, 0, 8], 0);
    w.store(ACTION, 32);
    w.call(14, &[2, ACTION, 0x7ffffc, 8], -14);
    w.call(14, &[99, 0, OLD, 8], 0);
    w.memory(OLD, 32);
    // Retain nonempty state: terminal page erasure must reclaim it as well.
}
