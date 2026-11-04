//! TK-DEC-279: exact field effects on actual loaned bytes; no native evidence.
use super::*;
use num_bigint::BigUint;

/// Encoding data, independent of CPU word size and carrying no region authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionLibraryByteOrder {
    LittleEndian,
    BigEndian,
}

/// A statically selected systems format, never an address or a live grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionLibraryUnsignedFormat {
    byte_count: u8,
    byte_order: RegionLibraryByteOrder,
}

impl RegionLibraryUnsignedFormat {
    pub fn new(
        byte_count: u8,
        byte_order: RegionLibraryByteOrder,
    ) -> Result<Self, RegionLibraryError> {
        if !(1..=8).contains(&byte_count) {
            return Err(RegionLibraryError::Binding);
        }
        Ok(Self {
            byte_count,
            byte_order,
        })
    }

    pub const fn byte_count(self) -> u8 {
        self.byte_count
    }

    pub const fn byte_order(self) -> RegionLibraryByteOrder {
        self.byte_order
    }

    fn shift(self, index: u8) -> u32 {
        u32::from(match self.byte_order {
            RegionLibraryByteOrder::LittleEndian => index,
            RegionLibraryByteOrder::BigEndian => self.byte_count - 1 - index,
        }) * 8
    }
}

impl RegionLibraryInvocation<'_> {
    /// Preflight the whole contained field without observing any owner payload.
    fn unsigned_offset(
        &self,
        window: Window,
        offset: &BigUint,
        format: RegionLibraryUnsignedFormat,
    ) -> Result<u64, RegionLibraryError> {
        if offset.bits() > 64 {
            return Err(RegionLibraryError::Bounds);
        }
        let offset = offset.to_u64_digits().first().copied().unwrap_or(0);
        let width = u64::from(format.byte_count);
        if offset
            .checked_add(width)
            .is_none_or(|end| end > window.count)
        {
            return Err(RegionLibraryError::Bounds);
        }
        let absolute = window
            .start
            .checked_add(offset)
            .ok_or(RegionLibraryError::Bounds)?;
        if absolute
            .checked_add(width)
            .is_none_or(|end| end > self.storage.count())
        {
            return Err(RegionLibraryError::Bounds);
        }
        Ok(absolute)
    }

    /// Models exact Nat decoding through an existing live view. Bounds errors
    /// correspond to source None; authority errors retain their existing law.
    pub fn load_unsigned(
        &mut self,
        view: &RegionLibraryView,
        offset: &BigUint,
        format: RegionLibraryUnsignedFormat,
    ) -> Result<BigUint, RegionLibraryError> {
        let window = self.window(view)?;
        let start = self.unsigned_offset(window, offset, format)?;
        let mut value = 0_u64;
        for index in 0..format.byte_count {
            let byte = self.storage.load(start + u64::from(index))?;
            value |= u64::from(byte) << format.shift(index);
        }
        Ok(BigUint::from(value))
    }

    /// Models a complete encoding after whole-span/rights/value admission.
    /// Refusal observes no payload and leaves the complete invocation unchanged.
    pub fn store_unsigned(
        &mut self,
        view: &RegionLibraryView,
        offset: &BigUint,
        format: RegionLibraryUnsignedFormat,
        value: &BigUint,
    ) -> Result<(), RegionLibraryError> {
        let window = self.window(view)?;
        if window.access != RegionLibraryAccess::ReadWrite {
            return Err(RegionLibraryError::ReadOnly);
        }
        let start = self.unsigned_offset(window, offset, format)?;
        if value.bits() > u64::from(format.byte_count) * 8 {
            return Err(RegionLibraryError::UnsignedValue);
        }
        // Conversion happens only after proving that the entire Nat fits.
        let value = value.to_u64_digits().first().copied().unwrap_or(0);
        for index in 0..format.byte_count {
            let byte = ((value >> format.shift(index)) & 255) as u8;
            self.storage.store(start + u64::from(index), byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "systems_region_unsigned_tests.rs"]
mod tests;
