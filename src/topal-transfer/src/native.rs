//! Native resource ownership and platform support manifests.

use std::fs::File;
use std::io;
use std::net::UdpSocket;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePlatform {
    Linux,
    Windows,
    MacOs,
    Android,
    Ios,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupportManifest {
    pub abi_revision: u16,
    pub platform: NativePlatform,
    pub positioned_file_read: bool,
    pub loopback_datagram: bool,
    pub raw_i2c: bool,
}

#[must_use]
pub const fn support_manifest() -> SupportManifest {
    let platform = if cfg!(target_os = "linux") {
        NativePlatform::Linux
    } else if cfg!(target_os = "windows") {
        NativePlatform::Windows
    } else if cfg!(target_os = "macos") {
        NativePlatform::MacOs
    } else if cfg!(target_os = "android") {
        NativePlatform::Android
    } else if cfg!(target_os = "ios") {
        NativePlatform::Ios
    } else {
        NativePlatform::Other
    };
    SupportManifest {
        abi_revision: crate::host::HOST_ABI_REVISION,
        platform,
        positioned_file_read: !matches!(platform, NativePlatform::Other),
        loopback_datagram: !matches!(platform, NativePlatform::Other),
        raw_i2c: matches!(platform, NativePlatform::Linux),
    }
}

/// Backend-private ownership of an embedding-supplied file capability.
#[derive(Debug)]
pub struct NativeFile(File);

impl NativeFile {
    #[must_use]
    pub const fn inject(file: File) -> Self {
        Self(file)
    }
    /// Reads without changing shared file position.
    /// # Errors
    /// Returns the native I/O failure as diagnostic provenance.
    #[cfg(unix)]
    pub fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize> {
        std::os::unix::fs::FileExt::read_at(&self.0, buffer, offset)
    }
    /// Reads without changing shared file position.
    /// # Errors
    /// Returns the native I/O failure as diagnostic provenance.
    #[cfg(windows)]
    pub fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize> {
        std::os::windows::fs::FileExt::seek_read(&self.0, buffer, offset)
    }
}

/// Backend-private ownership of an embedding-supplied datagram socket.
#[derive(Debug)]
pub struct NativeDatagram(UdpSocket);

impl NativeDatagram {
    #[must_use]
    pub const fn inject(socket: UdpSocket) -> Self {
        Self(socket)
    }
    /// Sends one message to the socket's connected peer.
    /// # Errors
    /// Returns native I/O provenance without exposing the socket itself.
    pub fn send(&self, message: &[u8]) -> io::Result<usize> {
        self.0.send(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifest_never_claims_mobile_i2c() {
        let manifest = support_manifest();
        if matches!(
            manifest.platform,
            NativePlatform::Android | NativePlatform::Ios
        ) {
            assert!(!manifest.raw_i2c);
        }
        assert_eq!(manifest.abi_revision, 1);
    }
}
