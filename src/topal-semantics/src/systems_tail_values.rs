//! Complete canonical values in owned reference buffers; not native authority.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TailScalarKind {
    Nat,
    Int,
    Boolean,
    Unit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TailScalar {
    Nat(BigUint),
    Int(BigInt),
    Boolean(bool),
    Unit,
}
impl TailScalar {
    pub fn kind(&self) -> TailScalarKind {
        match self {
            Self::Nat(_) => TailScalarKind::Nat,
            Self::Int(_) => TailScalarKind::Int,
            Self::Boolean(_) => TailScalarKind::Boolean,
            Self::Unit => TailScalarKind::Unit,
        }
    }
    pub(super) fn initial_word(&self) -> bool {
        match self {
            Self::Nat(n) => n.bits() <= 64,
            Self::Int(n) => n.bits() <= 64,
            Self::Boolean(_) => true,
            Self::Unit => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TailScalarBound {
    Nat { maximum: BigUint },
    Int { minimum: BigInt, maximum: BigInt },
    Boolean,
}
impl TailScalarBound {
    pub(super) fn valid(&self) -> bool {
        match self {
            Self::Nat { maximum } => maximum.bits() <= 256,
            Self::Int { minimum, maximum } => {
                minimum <= maximum && minimum.bits().max(maximum.bits()) <= 256
            }
            Self::Boolean => true,
        }
    }
    pub(super) fn contains(&self, value: &TailScalar) -> bool {
        match (self, value) {
            (Self::Nat { maximum }, TailScalar::Nat(n)) => n <= maximum,
            (Self::Int { minimum, maximum }, TailScalar::Int(n)) => minimum <= n && n <= maximum,
            (Self::Boolean, TailScalar::Boolean(_)) => true,
            _ => false,
        }
    }
    pub(super) fn kind(&self) -> TailScalarKind {
        match self {
            Self::Nat { .. } => TailScalarKind::Nat,
            Self::Int { .. } => TailScalarKind::Int,
            Self::Boolean => TailScalarKind::Boolean,
        }
    }
    pub(super) fn bytes(&self) -> usize {
        let bits = match self {
            Self::Nat { maximum } => maximum.bits(),
            Self::Int { minimum, maximum } => minimum.bits().max(maximum.bits()),
            Self::Boolean => return 8,
        }
        .max(1);
        (16 + 4 * bits.div_ceil(32) as usize).div_ceil(8) * 8
    }
}

pub(super) fn encode(value: &TailScalar) -> Result<Vec<u8>, RegionLibraryError> {
    let number = match value {
        TailScalar::Nat(n) => BigInt::from(n.clone()),
        TailScalar::Int(n) => n.clone(),
        TailScalar::Boolean(value) => return Ok(u64::from(*value).to_le_bytes().to_vec()),
        TailScalar::Unit => return Ok(Vec::new()),
    };
    if number.bits() > 256 {
        return Err(RegionLibraryError::Capacity);
    }
    let (sign, limbs) = number.to_u32_digits();
    let mut bytes = vec![0; (16 + 4 * limbs.len()).div_ceil(8) * 8];
    bytes[..8].copy_from_slice(&u64::from(sign == Sign::Minus).to_le_bytes());
    bytes[8..16].copy_from_slice(&(limbs.len() as u64).to_le_bytes());
    for (index, limb) in limbs.into_iter().enumerate() {
        bytes[16 + index * 4..20 + index * 4].copy_from_slice(&limb.to_le_bytes());
    }
    Ok(bytes)
}

pub(super) fn decode(kind: TailScalarKind, bytes: &[u8]) -> Result<TailScalar, RegionLibraryError> {
    let word = |offset: usize| -> Result<u64, RegionLibraryError> {
        bytes
            .get(offset..offset + 8)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(RegionLibraryError::Invariant)
    };
    if kind == TailScalarKind::Unit {
        return if bytes.iter().all(|b| *b == 0) {
            Ok(TailScalar::Unit)
        } else {
            Err(RegionLibraryError::Invariant)
        };
    }
    if kind == TailScalarKind::Boolean {
        let n = word(0)?;
        return if n <= 1 && bytes[8..].iter().all(|b| *b == 0) {
            Ok(TailScalar::Boolean(n == 1))
        } else {
            Err(RegionLibraryError::Invariant)
        };
    }
    let sign = word(0)?;
    let count = word(8)?;
    if sign > 1
        || count > 8
        || (kind == TailScalarKind::Nat && sign != 0)
        || (count == 0 && sign != 0)
    {
        return Err(RegionLibraryError::Invariant);
    }
    let end = 16 + count as usize * 4;
    let raw = bytes.get(16..end).ok_or(RegionLibraryError::Invariant)?;
    if bytes[end..].iter().any(|b| *b != 0) {
        return Err(RegionLibraryError::Invariant);
    }
    let limbs: Vec<_> = raw
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    if limbs.last() == Some(&0) {
        return Err(RegionLibraryError::Invariant);
    }
    let magnitude = BigUint::new(limbs);
    Ok(if kind == TailScalarKind::Nat {
        TailScalar::Nat(magnitude)
    } else {
        TailScalar::Int(BigInt::from_biguint(
            if sign == 1 { Sign::Minus } else { Sign::Plus },
            magnitude,
        ))
    })
}
