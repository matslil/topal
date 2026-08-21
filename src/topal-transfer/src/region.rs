//! Owned regions, checked spans, and scatter/gather descriptions.

use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionFailure {
    OutOfBounds,
    Misaligned,
    Overlap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    start: usize,
    length: usize,
}

impl Span {
    /// Constructs a checked span.
    /// # Errors
    /// Returns `OutOfBounds` on overflow or when the span exceeds `bound`.
    pub fn new(start: usize, length: usize, bound: usize) -> Result<Self, RegionFailure> {
        (start.checked_add(length).is_some_and(|end| end <= bound))
            .then_some(Self { start, length })
            .ok_or(RegionFailure::OutOfBounds)
    }
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }
    #[must_use]
    pub const fn length(self) -> usize {
        self.length
    }
    fn range(self) -> Range<usize> {
        self.start..self.start + self.length
    }
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        self.start < other.start + other.length && other.start < self.start + self.length
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Region(Arc<[u8]>);

impl Region {
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into().into())
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Borrows a checked immutable span.
    /// # Errors
    /// Returns `OutOfBounds` when `span` does not belong to this region.
    pub fn get(&self, span: Span) -> Result<&[u8], RegionFailure> {
        self.0.get(span.range()).ok_or(RegionFailure::OutOfBounds)
    }
    /// Validates alignment for a host substitution.
    /// # Errors
    /// Returns `Misaligned` for zero alignment or a non-aligned start.
    pub fn require_alignment(span: Span, alignment: usize) -> Result<(), RegionFailure> {
        (alignment != 0 && span.start.is_multiple_of(alignment))
            .then_some(())
            .ok_or(RegionFailure::Misaligned)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScatterGather(Vec<Span>);

impl ScatterGather {
    /// Constructs a checked descriptor without copying payload bytes.
    /// # Errors
    /// Returns `OutOfBounds` if a constituent span is invalid.
    pub fn new(spans: Vec<Span>, bound: usize) -> Result<Self, RegionFailure> {
        spans
            .iter()
            .try_for_each(|span| Span::new(span.start, span.length, bound).map(|_| ()))?;
        Ok(Self(spans))
    }
    #[must_use]
    pub fn spans(&self) -> impl ExactSizeIterator<Item = Span> + '_ {
        self.0.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spans_are_checked_and_zero_copy() {
        let region = Region::from_bytes(vec![1, 2, 3, 4]);
        let span = Span::new(1, 2, region.len()).unwrap();
        assert_eq!(region.get(span), Ok(&[2, 3][..]));
        assert_eq!(
            Span::new(usize::MAX, 2, region.len()),
            Err(RegionFailure::OutOfBounds)
        );
        assert_eq!(
            Region::require_alignment(span, 2),
            Err(RegionFailure::Misaligned)
        );
    }
}
