//! Architecture-independent affine translation-space lifecycle.

use std::fmt;

use crate::{FrameAllocatorContext, PhysicalFrameError, PhysicalFrameExtent, PhysicalFrameRequest};

pub const SYSTEMS_TRANSLATION_BEGIN: &str = "topal.systems.translation.begin/1";
pub const SYSTEMS_TRANSLATION_COMMIT: &str = "topal.systems.translation.commit/1";
pub const SYSTEMS_TRANSLATION_ACTIVATE: &str = "topal.systems.translation.activate/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationTemplate {
    BootstrapEquivalent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationPagePolicy {
    ProviderSelected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranslationUpdateRequest {
    pub template: TranslationTemplate,
    pub page_policy: TranslationPagePolicy,
}

impl TranslationUpdateRequest {
    #[must_use]
    pub const fn initial_bootstrap_equivalent() -> Self {
        Self {
            template: TranslationTemplate::BootstrapEquivalent,
            page_policy: TranslationPagePolicy::ProviderSelected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationError {
    UnsupportedRequest,
    InvalidProvider,
    BackingUnavailable,
    WrongProvider,
}

impl fmt::Display for TranslationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedRequest => "unsupported-request",
            Self::InvalidProvider => "invalid-provider",
            Self::BackingUnavailable => "backing-unavailable",
            Self::WrongProvider => "wrong-provider",
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TranslationUpdate {
    provider_identity: String,
    request: TranslationUpdateRequest,
    backing: PhysicalFrameExtent,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TranslationSpace {
    provider_identity: String,
    request: TranslationUpdateRequest,
    backing: PhysicalFrameExtent,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ActiveTranslation {
    provider_identity: String,
    request: TranslationUpdateRequest,
    backing: PhysicalFrameExtent,
}

impl FrameAllocatorContext {
    /// Reserve provider-owned backing for an exclusive translation update.
    ///
    /// # Errors
    ///
    /// Returns a stable error when the sealed request/provider is invalid or
    /// three contiguous model frames cannot be reserved.
    pub fn begin_translation_update(
        &mut self,
        request: TranslationUpdateRequest,
        provider_identity: impl Into<String>,
    ) -> Result<TranslationUpdate, TranslationError> {
        if request != TranslationUpdateRequest::initial_bootstrap_equivalent() {
            return Err(TranslationError::UnsupportedRequest);
        }
        let provider_identity = provider_identity.into();
        if provider_identity.is_empty() {
            return Err(TranslationError::InvalidProvider);
        }
        let backing = self
            .allocate(PhysicalFrameRequest {
                frame_count: 3,
                alignment_frames: 1,
            })
            .map_err(translation_frame_error)?;
        Ok(TranslationUpdate {
            provider_identity,
            request,
            backing,
        })
    }
}

impl TranslationUpdate {
    #[must_use]
    pub const fn request(&self) -> TranslationUpdateRequest {
        self.request
    }

    #[must_use]
    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    #[must_use]
    pub fn commit(self) -> TranslationSpace {
        TranslationSpace {
            provider_identity: self.provider_identity,
            request: self.request,
            backing: self.backing,
        }
    }
}

impl TranslationSpace {
    /// Activate this committed space through its originating provider.
    ///
    /// # Errors
    ///
    /// Returns `WrongProvider` without manufacturing active authority.
    pub fn activate(self, provider_identity: &str) -> Result<ActiveTranslation, TranslationError> {
        if self.provider_identity != provider_identity {
            return Err(TranslationError::WrongProvider);
        }
        Ok(ActiveTranslation {
            provider_identity: self.provider_identity,
            request: self.request,
            backing: self.backing,
        })
    }
}

impl ActiveTranslation {
    #[must_use]
    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    #[must_use]
    pub const fn request(&self) -> TranslationUpdateRequest {
        self.request
    }

    #[must_use]
    pub const fn backing_frame_count(&self) -> u64 {
        self.backing.frame_count
    }
}

fn translation_frame_error(_error: PhysicalFrameError) -> TranslationError {
    TranslationError::BackingUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BootMemoryClass, BootMemoryDescription, BootMemoryProvenance, FrameAllocatorResult,
        MemoryDescribedContext, NormalizedBootMemoryRange,
    };

    fn allocator() -> FrameAllocatorContext {
        let described = MemoryDescribedContext {
            handoff_identity: "handoff/1".into(),
            memory: BootMemoryDescription {
                page_size: 4096,
                ranges: vec![NormalizedBootMemoryRange {
                    start: 0x10_0000,
                    end: 0x20_0000,
                    class: BootMemoryClass::Allocatable,
                    provenance: vec![BootMemoryProvenance {
                        class: BootMemoryClass::Allocatable,
                        source_kind: "ram".into(),
                        provider: "provider/1".into(),
                    }],
                }],
            },
        };
        let FrameAllocatorResult::Ready(allocator) =
            described.create_frame_allocator("allocator/1")
        else {
            panic!("valid description must create an allocator")
        };
        allocator
    }

    #[test]
    fn update_commit_and_activation_preserve_sealed_request_and_provenance() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-MAPPING-001.
        let mut allocator = allocator();
        let update = allocator
            .begin_translation_update(
                TranslationUpdateRequest::initial_bootstrap_equivalent(),
                "x86-provider/1",
            )
            .unwrap();
        assert_eq!(update.provider_identity(), "x86-provider/1");
        let space = update.commit();
        assert_eq!(
            space.activate("other-provider"),
            Err(TranslationError::WrongProvider)
        );

        let active = allocator
            .begin_translation_update(
                TranslationUpdateRequest::initial_bootstrap_equivalent(),
                "x86-provider/1",
            )
            .unwrap()
            .commit()
            .activate("x86-provider/1")
            .unwrap();
        assert_eq!(active.provider_identity(), "x86-provider/1");
        assert_eq!(active.backing_frame_count(), 3);
        assert_eq!(
            active.request(),
            TranslationUpdateRequest::initial_bootstrap_equivalent()
        );
        assert_eq!(allocator.complete(), Err(PhysicalFrameError::LiveExtents));
    }

    #[test]
    fn update_rejects_invalid_provider_and_insufficient_backing() {
        // TOPAL-SYSTEMS-MAPPING-001, TOPAL-SYSTEMS-QUALIFY-001.
        let mut allocator = allocator();
        assert_eq!(
            allocator.begin_translation_update(
                TranslationUpdateRequest::initial_bootstrap_equivalent(),
                "",
            ),
            Err(TranslationError::InvalidProvider)
        );
        let _ = allocator.allocate(PhysicalFrameRequest {
            frame_count: 254,
            alignment_frames: 1,
        });
        assert_eq!(
            allocator.begin_translation_update(
                TranslationUpdateRequest::initial_bootstrap_equivalent(),
                "x86-provider/1",
            ),
            Err(TranslationError::BackingUnavailable)
        );
    }
}
