//! Shared private x86 register/memory encoding; carries no process authority.
use super::X86FunctionEncoder;

#[derive(Clone, Copy)]
#[repr(u8)]
pub(super) enum R {
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

pub(super) struct E<'a>(pub(super) &'a mut X86FunctionEncoder);
impl E<'_> {
    #[cfg_attr(test, track_caller)]
    pub(super) fn rr(&mut self, op: u8, dst: R, src: R) {
        self.0.bytes(&[
            0x48 | (((src as u8) >> 3) << 2) | ((dst as u8) >> 3),
            op,
            0xc0 | (((src as u8) & 7) << 3) | ((dst as u8) & 7),
        ]);
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn mov(&mut self, dst: R, src: R) {
        self.rr(0x89, dst, src);
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn imm(&mut self, dst: R, value: u64) {
        self.0
            .bytes(&[0x48 | ((dst as u8) >> 3), 0xb8 | ((dst as u8) & 7)]);
        self.0.bytes(&value.to_le_bytes());
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn arith(&mut self, op: u8, dst: R, value: i32) {
        self.0.bytes(&[
            0x48 | ((dst as u8) >> 3),
            0x81,
            0xc0 | (op << 3) | ((dst as u8) & 7),
        ]);
        self.0.bytes(&value.to_le_bytes());
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn add(&mut self, dst: R, n: i32) {
        self.arith(0, dst, n);
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn cmp(&mut self, dst: R, n: i32) {
        self.arith(7, dst, n);
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn mem(&mut self, op: u8, reg: R, base: R, offset: i32) {
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
    #[cfg_attr(test, track_caller)]
    pub(super) fn load(&mut self, dst: R, base: R, offset: i32) {
        self.mem(0x8b, dst, base, offset);
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn store(&mut self, base: R, offset: i32, src: R) {
        self.mem(0x89, src, base, offset);
    }
    #[cfg_attr(test, track_caller)]
    pub(super) fn gs(&mut self, op: u8, reg: R, offset: u32) {
        self.0.bytes(&[
            0x65,
            0x48 | (((reg as u8) >> 3) << 2),
            op,
            0x04 | (((reg as u8) & 7) << 3),
            0x25,
        ]);
        self.0.bytes(&offset.to_le_bytes());
    }
}
