//! Bounded-copy nested packet inspection and offload equivalence.

use crate::network::{IpFailure, Ipv4Header};
use crate::region::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Offload {
    Software,
    SimulatedHardware,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallFailure {
    IncompleteFrame,
    UnsupportedProtocol,
    InvalidPacket,
    HopLimitExpired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForwardTrace {
    pub ethernet: Span,
    pub ipv4: Span,
    pub payload: Span,
    pub checksum_offload: Offload,
    pub payload_copies: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Forwarded {
    pub bytes: Vec<u8>,
    pub trace: ForwardTrace,
}

/// Inspects Ethernet and IPv4 layers, decrements TTL, and forwards the same payload storage.
/// # Errors
/// Rejects truncated, unsupported, malformed, or expired packets before forwarding.
pub fn forward_ipv4(mut frame: Vec<u8>, offload: Offload) -> Result<Forwarded, FirewallFailure> {
    if frame.len() < 14 {
        return Err(FirewallFailure::IncompleteFrame);
    }
    if frame[12..14] != [0x08, 0x00] {
        return Err(FirewallFailure::UnsupportedProtocol);
    }
    let header = Ipv4Header::parse(&frame[14..]).map_err(map_ip)?;
    let header_len = usize::from(header.header_length);
    let total = usize::from(header.total_length);
    if frame[22] <= 1 {
        return Err(FirewallFailure::HopLimitExpired);
    }
    frame[22] -= 1;
    frame[24] = 0;
    frame[25] = 0;
    let checksum = ipv4_checksum(&frame[14..14 + header_len]);
    frame[24..26].copy_from_slice(&checksum.to_be_bytes());
    Ok(Forwarded {
        bytes: frame,
        trace: ForwardTrace {
            ethernet: Span::new(0, 14, 14 + total).map_err(|_| FirewallFailure::IncompleteFrame)?,
            ipv4: Span::new(14, header_len, 14 + total)
                .map_err(|_| FirewallFailure::InvalidPacket)?,
            payload: Span::new(14 + header_len, total - header_len, 14 + total)
                .map_err(|_| FirewallFailure::InvalidPacket)?,
            checksum_offload: offload,
            payload_copies: 0,
        },
    })
}

fn map_ip(failure: IpFailure) -> FirewallFailure {
    match failure {
        IpFailure::Incomplete => FirewallFailure::IncompleteFrame,
        IpFailure::WrongVersion => FirewallFailure::UnsupportedProtocol,
        IpFailure::InvalidPrefix | IpFailure::Malformed => FirewallFailure::InvalidPacket,
    }
}

#[must_use]
pub fn ipv4_checksum(header: &[u8]) -> u16 {
    let mut sum = 0u32;
    for chunk in header.chunks(2) {
        let word = u16::from_be_bytes([chunk[0], chunk.get(1).copied().unwrap_or(0)]);
        sum += u32::from(word);
        while sum > 0xffff {
            sum = (sum & 0xffff) + (sum >> 16);
        }
    }
    let Ok(folded) = u16::try_from(sum) else {
        return 0;
    };
    !folded
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> Vec<u8> {
        let mut f = vec![0; 14 + 24];
        f[12..14].copy_from_slice(&[8, 0]);
        f[14] = 0x45;
        f[16..18].copy_from_slice(&24u16.to_be_bytes());
        f[22] = 2;
        f[23] = 6;
        f[34..].copy_from_slice(b"data");
        f
    }
    #[test]
    fn software_and_offload_are_semantically_equivalent() {
        let software = forward_ipv4(frame(), Offload::Software).unwrap();
        let hardware = forward_ipv4(frame(), Offload::SimulatedHardware).unwrap();
        assert_eq!(software.bytes, hardware.bytes);
        assert_eq!(software.trace.payload_copies, 0);
        assert_eq!(software.trace.payload, hardware.trace.payload);
        assert_eq!(software.bytes[22], 1);
    }
}
