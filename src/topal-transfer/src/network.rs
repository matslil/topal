//! Typed IP identities and validated packet headers.

use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpAddress {
    V4(Ipv4Addr),
    V6 { address: Ipv6Addr, scope: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpFailure {
    InvalidPrefix,
    Incomplete,
    WrongVersion,
    Malformed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpPrefix {
    pub address: IpAddress,
    length: u8,
}

impl IpPrefix {
    /// Constructs a family-checked prefix.
    /// # Errors
    /// Returns `InvalidPrefix` above 32 bits for IPv4 or 128 for IPv6.
    pub fn new(address: IpAddress, length: u8) -> Result<Self, IpFailure> {
        let maximum = if matches!(address, IpAddress::V4(_)) {
            32
        } else {
            128
        };
        (length <= maximum)
            .then_some(Self { address, length })
            .ok_or(IpFailure::InvalidPrefix)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Header {
    pub header_length: u8,
    pub total_length: u16,
    pub protocol: u8,
}

impl Ipv4Header {
    /// Validates the fixed and declared IPv4 header bounds.
    /// # Errors
    /// Distinguishes incomplete, wrong-version, and malformed input.
    pub fn parse(bytes: &[u8]) -> Result<Self, IpFailure> {
        if bytes.len() < 20 {
            return Err(IpFailure::Incomplete);
        }
        if bytes[0] >> 4 != 4 {
            return Err(IpFailure::WrongVersion);
        }
        let header_length = (bytes[0] & 0x0f) * 4;
        let total_length = u16::from_be_bytes([bytes[2], bytes[3]]);
        if header_length < 20
            || total_length < u16::from(header_length)
            || usize::from(total_length) > bytes.len()
        {
            return Err(IpFailure::Malformed);
        }
        Ok(Self {
            header_length,
            total_length,
            protocol: bytes[9],
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv6Header {
    pub payload_length: u16,
    pub next_header: u8,
    pub hop_limit: u8,
}

impl Ipv6Header {
    /// Validates an IPv6 base header and declared payload bound.
    /// # Errors
    /// Distinguishes incomplete, wrong-version, and malformed input.
    pub fn parse(bytes: &[u8]) -> Result<Self, IpFailure> {
        if bytes.len() < 40 {
            return Err(IpFailure::Incomplete);
        }
        if bytes[0] >> 4 != 6 {
            return Err(IpFailure::WrongVersion);
        }
        let payload_length = u16::from_be_bytes([bytes[4], bytes[5]]);
        if 40 + usize::from(payload_length) > bytes.len() {
            return Err(IpFailure::Malformed);
        }
        Ok(Self {
            payload_length,
            next_header: bytes[6],
            hop_limit: bytes[7],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn families_remain_distinct() {
        assert!(IpPrefix::new(IpAddress::V4(Ipv4Addr::LOCALHOST), 33).is_err());
        assert!(
            IpPrefix::new(
                IpAddress::V6 {
                    address: Ipv6Addr::LOCALHOST,
                    scope: 2
                },
                128
            )
            .is_ok()
        );
        let mut v4 = vec![0; 20];
        v4[0] = 0x45;
        v4[2..4].copy_from_slice(&20u16.to_be_bytes());
        assert_eq!(Ipv4Header::parse(&v4).unwrap().header_length, 20);
        assert_eq!(Ipv6Header::parse(&v4), Err(IpFailure::Incomplete));
    }
}
