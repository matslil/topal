//! Architecture-independent boot-memory normalization and context refinement.

use std::fmt;

pub const SYSTEMS_BOOT_MEMORY_DESCRIBE: &str = "topal.systems.boot.describe-memory/1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BootMemoryClass {
    Allocatable,
    Reclaimable,
    Persistent,
    Reserved,
    Unusable,
    Unknown,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BootMemoryProvenance {
    pub class: BootMemoryClass,
    pub source_kind: String,
    pub provider: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootMemorySourceRange {
    pub start: u64,
    pub length: u64,
    pub provenance: BootMemoryProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootMemoryReservation {
    pub start: u64,
    pub length: u64,
    pub owner: String,
    pub reclaimable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedBootMemoryRange {
    pub start: u64,
    pub end: u64,
    pub class: BootMemoryClass,
    pub provenance: Vec<BootMemoryProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootMemoryDescription {
    pub page_size: u64,
    pub ranges: Vec<NormalizedBootMemoryRange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnteredBootstrapContext {
    handoff_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryDescribedContext {
    pub handoff_identity: String,
    pub memory: BootMemoryDescription,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootMemoryFailureContext {
    pub handoff_identity: String,
    pub error: BootMemoryError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootMemoryResult {
    Described(MemoryDescribedContext),
    Failure(BootMemoryFailureContext),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootMemoryError {
    InvalidPageSize,
    MalformedRange,
    RangeOverflow,
    UnsupportedHandoff,
    NoAllocatableMemory,
}

impl fmt::Display for BootMemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPageSize => "invalid-page-size",
            Self::MalformedRange => "malformed-range",
            Self::RangeOverflow => "range-overflow",
            Self::UnsupportedHandoff => "unsupported-handoff",
            Self::NoAllocatableMemory => "no-allocatable-memory",
        })
    }
}

impl EnteredBootstrapContext {
    #[must_use]
    pub fn new(handoff_identity: impl Into<String>) -> Self {
        Self {
            handoff_identity: handoff_identity.into(),
        }
    }

    #[must_use]
    pub fn describe_memory(
        self,
        source: &[BootMemorySourceRange],
        reservations: &[BootMemoryReservation],
        page_size: u64,
        handoff_supported: bool,
    ) -> BootMemoryResult {
        let result = if handoff_supported {
            normalize_boot_memory(source, reservations, page_size)
        } else {
            Err(BootMemoryError::UnsupportedHandoff)
        };
        match result {
            Ok(memory) => BootMemoryResult::Described(MemoryDescribedContext {
                handoff_identity: self.handoff_identity,
                memory,
            }),
            Err(error) => BootMemoryResult::Failure(BootMemoryFailureContext {
                handoff_identity: self.handoff_identity,
                error,
            }),
        }
    }
}

/// Normalize provider-supplied ranges and reservations without granting
/// machine-address authority.
///
/// # Errors
///
/// Returns a sealed boot-memory error for an invalid page size, malformed or
/// overflowing range, or a result with no complete allocatable page.
pub fn normalize_boot_memory(
    source: &[BootMemorySourceRange],
    reservations: &[BootMemoryReservation],
    page_size: u64,
) -> Result<BootMemoryDescription, BootMemoryError> {
    if page_size == 0 || !page_size.is_power_of_two() {
        return Err(BootMemoryError::InvalidPageSize);
    }

    let mut claims = Vec::with_capacity(source.len() + reservations.len());
    for range in source {
        claims.push(Claim::from_source(range)?);
    }
    for reservation in reservations {
        claims.push(Claim::from_reservation(reservation)?);
    }
    if claims.is_empty() {
        return Err(BootMemoryError::NoAllocatableMemory);
    }

    let mut boundaries = claims
        .iter()
        .flat_map(|claim| [claim.start, claim.end])
        .collect::<Vec<_>>();
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut ranges = Vec::new();
    for endpoints in boundaries.windows(2) {
        let start = endpoints[0];
        let end = endpoints[1];
        if start == end {
            continue;
        }
        let covering = claims
            .iter()
            .filter(|claim| claim.start <= start && end <= claim.end)
            .collect::<Vec<_>>();
        if covering.is_empty() {
            continue;
        }
        let mut provenance = covering
            .iter()
            .map(|claim| claim.provenance.clone())
            .collect::<Vec<_>>();
        provenance.sort();
        provenance.dedup();
        let class = effective_class(&covering);
        push_page_qualified(&mut ranges, start, end, class, provenance, page_size)?;
    }
    merge_adjacent(&mut ranges);
    if !ranges
        .iter()
        .any(|range| range.class == BootMemoryClass::Allocatable)
    {
        return Err(BootMemoryError::NoAllocatableMemory);
    }
    Ok(BootMemoryDescription { page_size, ranges })
}

#[derive(Clone)]
struct Claim {
    start: u64,
    end: u64,
    provenance: BootMemoryProvenance,
}

impl Claim {
    fn from_source(range: &BootMemorySourceRange) -> Result<Self, BootMemoryError> {
        Self::new(range.start, range.length, range.provenance.clone())
    }

    fn from_reservation(reservation: &BootMemoryReservation) -> Result<Self, BootMemoryError> {
        Self::new(
            reservation.start,
            reservation.length,
            BootMemoryProvenance {
                class: if reservation.reclaimable {
                    BootMemoryClass::Reclaimable
                } else {
                    BootMemoryClass::Reserved
                },
                source_kind: format!("reservation:{}", reservation.owner),
                provider: "topal.systems.boot-reservation/1".into(),
            },
        )
    }

    fn new(
        start: u64,
        length: u64,
        provenance: BootMemoryProvenance,
    ) -> Result<Self, BootMemoryError> {
        if length == 0 {
            return Err(BootMemoryError::MalformedRange);
        }
        let end = start
            .checked_add(length)
            .ok_or(BootMemoryError::RangeOverflow)?;
        Ok(Self {
            start,
            end,
            provenance,
        })
    }
}

fn effective_class(claims: &[&Claim]) -> BootMemoryClass {
    if claims
        .iter()
        .any(|claim| claim.provenance.class == BootMemoryClass::Unusable)
    {
        return BootMemoryClass::Unusable;
    }
    if claims
        .iter()
        .all(|claim| claim.provenance.class == BootMemoryClass::Allocatable)
    {
        return BootMemoryClass::Allocatable;
    }
    let first = claims[0].provenance.class;
    if claims.iter().all(|claim| claim.provenance.class == first) {
        first
    } else {
        BootMemoryClass::Reserved
    }
}

fn push_page_qualified(
    ranges: &mut Vec<NormalizedBootMemoryRange>,
    start: u64,
    end: u64,
    class: BootMemoryClass,
    provenance: Vec<BootMemoryProvenance>,
    page_size: u64,
) -> Result<(), BootMemoryError> {
    if class != BootMemoryClass::Allocatable {
        ranges.push(NormalizedBootMemoryRange {
            start,
            end,
            class,
            provenance,
        });
        return Ok(());
    }
    let aligned_start = start
        .checked_add(page_size - 1)
        .ok_or(BootMemoryError::RangeOverflow)?
        & !(page_size - 1);
    let aligned_end = end & !(page_size - 1);
    if aligned_start >= aligned_end {
        ranges.push(NormalizedBootMemoryRange {
            start,
            end,
            class: BootMemoryClass::Reserved,
            provenance,
        });
        return Ok(());
    }
    if start < aligned_start {
        ranges.push(NormalizedBootMemoryRange {
            start,
            end: aligned_start,
            class: BootMemoryClass::Reserved,
            provenance: provenance.clone(),
        });
    }
    ranges.push(NormalizedBootMemoryRange {
        start: aligned_start,
        end: aligned_end,
        class,
        provenance: provenance.clone(),
    });
    if aligned_end < end {
        ranges.push(NormalizedBootMemoryRange {
            start: aligned_end,
            end,
            class: BootMemoryClass::Reserved,
            provenance,
        });
    }
    Ok(())
}

fn merge_adjacent(ranges: &mut Vec<NormalizedBootMemoryRange>) {
    let mut merged: Vec<NormalizedBootMemoryRange> = Vec::with_capacity(ranges.len());
    for range in ranges.drain(..) {
        if let Some(previous) = merged.last_mut()
            && previous.end == range.start
            && previous.class == range.class
            && previous.provenance == range.provenance
        {
            previous.end = range.end;
        } else {
            merged.push(range);
        }
    }
    *ranges = merged;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(
        start: u64,
        length: u64,
        class: BootMemoryClass,
        kind: &str,
    ) -> BootMemorySourceRange {
        BootMemorySourceRange {
            start,
            length,
            provenance: BootMemoryProvenance {
                class,
                source_kind: kind.into(),
                provider: "test-provider/1".into(),
            },
        }
    }

    #[test]
    fn normalizes_overlaps_reservations_and_page_edges_conservatively() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-BOOT-MEMORY-001.
        let result = normalize_boot_memory(
            &[
                source(0x1003, 0x8ffd, BootMemoryClass::Allocatable, "ram"),
                source(0x4000, 0x1000, BootMemoryClass::Unknown, "future"),
            ],
            &[BootMemoryReservation {
                start: 0x7000,
                length: 0x1000,
                owner: "kernel".into(),
                reclaimable: false,
            }],
            4096,
        )
        .unwrap();
        assert!(result.ranges.iter().any(|range| {
            range.start == 0x2000
                && range.end == 0x4000
                && range.class == BootMemoryClass::Allocatable
        }));
        assert!(result.ranges.iter().any(|range| {
            range.start == 0x4000
                && range.end == 0x5000
                && range.class == BootMemoryClass::Reserved
                && range.provenance.len() == 2
        }));
        assert!(result.ranges.iter().any(|range| {
            range.start == 0x7000 && range.end == 0x8000 && range.class == BootMemoryClass::Reserved
        }));
        assert!(result.ranges.iter().any(|range| {
            range.start == 0x1003 && range.end == 0x2000 && range.class == BootMemoryClass::Reserved
        }));
    }

    #[test]
    fn consumes_entered_context_into_success_or_fatal_only_failure() {
        // TOPAL-SYSTEMS-BOOT-MEMORY-001.
        let described = EnteredBootstrapContext::new("handoff/1").describe_memory(
            &[source(0x2000, 0x2000, BootMemoryClass::Allocatable, "ram")],
            &[],
            4096,
            true,
        );
        assert!(matches!(described, BootMemoryResult::Described(_)));

        let failed =
            EnteredBootstrapContext::new("handoff/1").describe_memory(&[], &[], 4096, false);
        assert!(matches!(
            failed,
            BootMemoryResult::Failure(BootMemoryFailureContext {
                error: BootMemoryError::UnsupportedHandoff,
                ..
            })
        ));
    }

    #[test]
    fn rejects_malformed_overflowing_or_all_reserved_input() {
        // TOPAL-SYSTEMS-BOOT-MEMORY-001, TOPAL-SYSTEMS-QUALIFY-001.
        assert_eq!(
            normalize_boot_memory(
                &[source(0, 0, BootMemoryClass::Allocatable, "ram")],
                &[],
                4096,
            ),
            Err(BootMemoryError::MalformedRange)
        );
        assert_eq!(
            normalize_boot_memory(
                &[source(u64::MAX, 2, BootMemoryClass::Allocatable, "ram",)],
                &[],
                4096,
            ),
            Err(BootMemoryError::RangeOverflow)
        );
        assert_eq!(
            normalize_boot_memory(
                &[source(0, 0x4000, BootMemoryClass::Reserved, "reserved")],
                &[],
                4096,
            ),
            Err(BootMemoryError::NoAllocatableMemory)
        );
    }
}
