//! TK-DEC-280: ordinary inode state, not an object capability or native proof.
//! TOPAL-SYSTEMS-INODE-MODE-STATE-001.

/// An owned observation of four explicitly little-endian metadata fields.
/// Copying this value grants no access to the inode from which it was observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsInodeMetadata {
    pub(crate) bytes: [u8; 32],
}

impl SystemsInodeMetadata {
    pub(crate) fn new(kind: u16, permissions: u16, seconds: u64) -> Self {
        let mut value = Self { bytes: [0; 32] };
        value.set(0, u64::from(kind | (permissions & 0o7777)));
        for field in [8, 16, 24] {
            value.set(field, seconds);
        }
        value
    }

    fn get(&self, offset: usize) -> u64 {
        u64::from_le_bytes(self.bytes[offset..offset + 8].try_into().unwrap())
    }

    fn set(&mut self, offset: usize, value: u64) {
        self.bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn encoded(&self) -> &[u8; 32] {
        &self.bytes
    }

    pub fn mode(&self) -> u16 {
        self.get(0) as u16
    }

    pub fn permissions(&self) -> u16 {
        self.mode() & 0o7777
    }

    pub fn accessed_seconds(&self) -> u64 {
        self.get(8)
    }

    pub fn modified_seconds(&self) -> u64 {
        self.get(16)
    }

    pub fn changed_seconds(&self) -> u64 {
        self.get(24)
    }

    /// Linux personality adaptation. The current prepared credential owns the
    /// inode and can retain set-ID bits. Core Nat conversion is not involved.
    pub(crate) fn chmod(&mut self, native_mode: u64, seconds: u64) {
        let permissions = native_mode as u16 & 0o7777;
        self.set(0, u64::from((self.mode() & !0o7777) | permissions));
        self.set(24, seconds);
    }

    pub(crate) fn accessed(&mut self, seconds: u64) {
        self.set(8, seconds);
    }

    pub(crate) fn changed(&mut self, seconds: u64) {
        self.set(24, seconds);
    }

    pub(crate) fn modified(&mut self, seconds: u64) {
        self.set(16, seconds);
        self.changed(seconds);
    }

    #[cfg(test)]
    pub(crate) fn erase(&mut self) {
        self.bytes.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_permission_pattern_preserves_type_and_unrelated_fields() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001. Independent native-byte oracle.
        for kind in [0o100000, 0o40000, 0o10000] {
            let mut inode = SystemsInodeMetadata::new(kind, 0o6754, 0x0102030405060708);
            inode.accessed(0x1112131415161718);
            inode.modified(0x2122232425262728);
            for permissions in 0..=0o7777_u16 {
                let before = inode.clone();
                let native = 0xfedcba9876540000_u64
                    | (u64::from(permissions & 15) << 12)
                    | u64::from(permissions);
                let seconds = u64::MAX - u64::from(permissions);
                inode.chmod(native, seconds);
                let mut expected = *before.encoded();
                expected[..8].copy_from_slice(&u64::from(kind | permissions).to_le_bytes());
                expected[24..].copy_from_slice(&seconds.to_le_bytes());
                assert_eq!(inode.encoded(), &expected);
                assert_eq!(inode.mode(), kind | permissions);
                assert_eq!(inode.permissions(), permissions);
                assert_eq!(inode.accessed_seconds(), before.accessed_seconds());
                assert_eq!(inode.modified_seconds(), before.modified_seconds());
                assert_eq!(inode.changed_seconds(), seconds);
            }
        }
    }

    #[test]
    fn repeated_equal_mode_still_updates_ctime_and_erasure_clears_all_fields() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut inode = SystemsInodeMetadata::new(0o100000, 0o640, 7);
        inode.chmod(0o640, 8);
        assert_eq!(inode.mode(), 0o100640);
        assert_eq!(inode.changed_seconds(), 8);
        assert_eq!(inode.accessed_seconds(), 7);
        assert_eq!(inode.modified_seconds(), 7);
        inode.chmod(u64::MAX, 9);
        assert_eq!(inode.mode(), 0o107777);
        inode.erase();
        assert_eq!(inode.encoded(), &[0; 32]);
    }
}
