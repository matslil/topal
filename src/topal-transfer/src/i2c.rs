//! Atomic I2C transactions and platform adapters.

use std::collections::BTreeMap;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Segment {
    Write(Vec<u8>),
    Read(usize),
}

#[derive(Debug)]
pub struct VirtualBus {
    targets: BTreeMap<TargetAddress, Vec<u8>>,
    limit: usize,
}

impl VirtualBus {
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self {
            targets: BTreeMap::new(),
            limit,
        }
    }
    pub fn attach(&mut self, address: TargetAddress, registers: Vec<u8>) {
        self.targets.insert(address, registers);
    }
    /// Executes all segments as one combined transaction.
    /// # Errors
    /// Reports address, NACK, or transfer-limit failure without partial mutation.
    pub fn transfer(
        &mut self,
        address: TargetAddress,
        segments: &[Segment],
    ) -> Result<Vec<Vec<u8>>, I2cFailure> {
        let total = segments
            .iter()
            .map(|segment| match segment {
                Segment::Write(bytes) => bytes.len(),
                Segment::Read(length) => *length,
            })
            .sum::<usize>();
        if total > self.limit {
            return Err(I2cFailure::TransferLimit);
        }
        let registers = self
            .targets
            .get(&address)
            .ok_or(I2cFailure::NegativeAcknowledgement)?;
        let mut cursor = 0usize;
        let mut reads = Vec::new();
        for segment in segments {
            match segment {
                Segment::Write(bytes) => {
                    if let Some(register) = bytes.first() {
                        cursor = usize::from(*register);
                    }
                }
                Segment::Read(length) => {
                    let end = cursor
                        .checked_add(*length)
                        .ok_or(I2cFailure::TransferLimit)?;
                    reads.push(
                        registers
                            .get(cursor..end)
                            .ok_or(I2cFailure::NegativeAcknowledgement)?
                            .to_vec(),
                    );
                    cursor = end;
                }
            }
        }
        Ok(reads)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combined_sensor_read_and_nack_are_explicit() {
        let address = TargetAddress::seven_bit(0x48).unwrap();
        let mut bus = VirtualBus::new(8);
        bus.attach(address, vec![10, 20, 30]);
        assert_eq!(
            bus.transfer(address, &[Segment::Write(vec![1]), Segment::Read(2)]),
            Ok(vec![vec![20, 30]])
        );
        assert_eq!(
            bus.transfer(TargetAddress::seven_bit(0x49).unwrap(), &[Segment::Read(1)]),
            Err(I2cFailure::NegativeAcknowledgement)
        );
    }
}
