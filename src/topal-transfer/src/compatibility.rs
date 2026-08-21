//! Compatibility rules for public packages and the semantic host ABI.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Revision {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityFailure {
    DifferentMajor,
    MissingRequiredRevision,
}

/// Negotiates a required contract against an implementation revision.
/// # Errors
/// Rejects different major revisions and older implementation revisions.
pub const fn negotiate(
    required: Revision,
    provided: Revision,
) -> Result<Revision, CompatibilityFailure> {
    if required.major != provided.major {
        return Err(CompatibilityFailure::DifferentMajor);
    }
    if provided.minor < required.minor {
        return Err(CompatibilityFailure::MissingRequiredRevision);
    }
    Ok(required)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compatibility_never_silently_crosses_major_versions() {
        assert_eq!(
            negotiate(
                Revision { major: 1, minor: 1 },
                Revision { major: 1, minor: 2 }
            ),
            Ok(Revision { major: 1, minor: 1 })
        );
        assert_eq!(
            negotiate(
                Revision { major: 1, minor: 0 },
                Revision { major: 2, minor: 0 }
            ),
            Err(CompatibilityFailure::DifferentMajor)
        );
    }
}
