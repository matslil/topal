//! Native I2C platform adapters.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TargetAddress(u16);

impl TargetAddress {
    /// Constructs a 7-bit target address.
    /// # Errors
    /// Rejects reserved/out-of-range values outside `0x08..=0x77`.
    pub fn seven_bit(value: u16) -> Result<Self, I2cFailure> {
        (0x08..=0x77)
            .contains(&value)
            .then_some(Self(value))
            .ok_or(I2cFailure::InvalidAddress)
    }
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum I2cFailure {
    InvalidAddress,
    NegativeAcknowledgement,
    TransferLimit,
    ArbitrationLost,
    Removed,
    Native(String),
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::{I2cFailure, TargetAddress};
    use i2cdev::core::{I2CMessage, I2CTransfer};
    use i2cdev::linux::{LinuxI2CDevice, LinuxI2CMessage};
    use std::path::Path;

    #[derive(Debug)]
    pub struct LinuxI2cDevice(LinuxI2CDevice);
    impl LinuxI2cDevice {
        /// Opens a broker-approved `i2c-dev` path for one target.
        /// # Errors
        /// Preserves Linux binding provenance in `Native`.
        pub fn open(path: &Path, target: TargetAddress) -> Result<Self, I2cFailure> {
            LinuxI2CDevice::new(path, target.get())
                .map(Self)
                .map_err(|error| I2cFailure::Native(error.to_string()))
        }
        /// Performs an atomic register-address write/read using `I2C_RDWR`.
        /// # Errors
        /// Returns the Linux binding's diagnostic provenance.
        pub fn register_read(
            &mut self,
            register: &[u8],
            output: &mut [u8],
        ) -> Result<(), I2cFailure> {
            let mut messages = [
                LinuxI2CMessage::write(register),
                LinuxI2CMessage::read(output),
            ];
            self.0
                .transfer(&mut messages)
                .map(|_| ())
                .map_err(|error| I2cFailure::Native(error.to_string()))
        }
    }
}
