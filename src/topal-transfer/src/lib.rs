//! Native host adapters for Topal data transfers.
//!
//! Portable transfer semantics live exclusively in ordinary Topal source
//! below `library/std`. This crate owns only mechanisms which cross an OS or
//! device boundary and does not define a second standard-library API.

pub mod i2c;
pub mod native;

/// ABI revision shared by native adapters and their Topal host contract.
pub const HOST_ABI_REVISION: u16 = 1;
