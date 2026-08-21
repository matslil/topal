//! Validated views with exact mutation dependencies.

use crate::region::{RegionFailure, Span};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationFailure {
    Incomplete,
    Malformed,
    Unsupported,
    Exhausted,
    Invalidated,
}

#[derive(Debug)]
pub struct MutableRegion {
    bytes: Vec<u8>,
    versions: Vec<u64>,
}

impl MutableRegion {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        let versions = vec![0; bytes.len()];
        Self { bytes, versions }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    /// Replaces exactly one checked span and invalidates overlapping evidence.
    /// # Errors
    /// Returns `OutOfBounds` unless the replacement exactly fits `span`.
    pub fn replace(&mut self, span: Span, replacement: &[u8]) -> Result<(), RegionFailure> {
        if replacement.len() != span.length() {
            return Err(RegionFailure::OutOfBounds);
        }
        let range = span.start()
            ..span
                .start()
                .checked_add(span.length())
                .ok_or(RegionFailure::OutOfBounds)?;
        let destination = self
            .bytes
            .get_mut(range.clone())
            .ok_or(RegionFailure::OutOfBounds)?;
        destination.copy_from_slice(replacement);
        self.versions[range]
            .iter_mut()
            .for_each(|version| *version = version.wrapping_add(1));
        Ok(())
    }
    /// Validates a span with a format-specific predicate.
    /// # Errors
    /// Returns a parser failure or `Incomplete` for a bad span.
    pub fn validate(
        &self,
        span: Span,
        predicate: impl FnOnce(&[u8]) -> Result<(), ValidationFailure>,
    ) -> Result<ValidatedView, ValidationFailure> {
        let range = span.start()
            ..span
                .start()
                .checked_add(span.length())
                .ok_or(ValidationFailure::Incomplete)?;
        let bytes = self
            .bytes
            .get(range.clone())
            .ok_or(ValidationFailure::Incomplete)?;
        predicate(bytes)?;
        Ok(ValidatedView {
            span,
            evidence: self.versions[range].to_vec(),
        })
    }
    /// Borrows bytes only while validation evidence remains current.
    /// # Errors
    /// Returns `Invalidated` after any overlapping mutation.
    pub fn bytes(&self, view: &ValidatedView) -> Result<&[u8], ValidationFailure> {
        let range = view.span.start()..view.span.start() + view.span.length();
        if self.versions.get(range.clone()) != Some(view.evidence.as_slice()) {
            return Err(ValidationFailure::Invalidated);
        }
        self.bytes.get(range).ok_or(ValidationFailure::Incomplete)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedView {
    span: Span,
    evidence: Vec<u64>,
}

impl ValidatedView {
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutation_invalidates_only_overlapping_evidence() {
        let mut region = MutableRegion::new(vec![1, 2, 3, 4]);
        let left = region
            .validate(Span::new(0, 2, 4).unwrap(), |_| Ok(()))
            .unwrap();
        let right = region
            .validate(Span::new(2, 2, 4).unwrap(), |_| Ok(()))
            .unwrap();
        region.replace(Span::new(0, 1, 4).unwrap(), &[9]).unwrap();
        assert_eq!(region.bytes(&left), Err(ValidationFailure::Invalidated));
        assert_eq!(region.bytes(&right), Ok(&[3, 4][..]));
    }
}
