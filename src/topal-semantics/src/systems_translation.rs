//! Architecture-independent affine translation-space lifecycle.

use std::fmt;

use crate::{
    FrameAllocatorContext, KernelMapping, KernelMappingError, KernelMappingRequest,
    PhysicalFrameError, PhysicalFrameExtent, PhysicalFrameRequest,
};

pub const SYSTEMS_TRANSLATION_BEGIN: &str = "topal.systems.translation.begin/1";
pub const SYSTEMS_TRANSLATION_COMMIT: &str = "topal.systems.translation.commit/1";
pub const SYSTEMS_TRANSLATION_ACTIVATE: &str = "topal.systems.translation.activate/1";
pub const SYSTEMS_TRANSLATION_EDIT_BEGIN: &str = "topal.systems.translation.edit.begin/1";
pub const SYSTEMS_TRANSLATION_EDIT_MAP: &str = "topal.systems.translation.edit.map-kernel/1";
pub const SYSTEMS_TRANSLATION_EDIT_UNMAP: &str = "topal.systems.translation.edit.unmap-kernel/1";
pub const SYSTEMS_TRANSLATION_EDIT_COMMIT: &str = "topal.systems.translation.edit.commit/1";

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
    WrongEditKind,
    WrongMapping,
}

impl fmt::Display for TranslationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedRequest => "unsupported-request",
            Self::InvalidProvider => "invalid-provider",
            Self::BackingUnavailable => "backing-unavailable",
            Self::WrongProvider => "wrong-provider",
            Self::WrongEditKind => "wrong-edit-kind",
            Self::WrongMapping => "wrong-mapping",
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
    edit_backing: Vec<PhysicalFrameExtent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationEditKind {
    Map,
    Unmap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationPlacement {
    ProviderSelected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranslationMappingRequest {
    pub mapping: KernelMappingRequest,
    pub placement: TranslationPlacement,
}

impl TranslationMappingRequest {
    #[must_use]
    pub const fn initial_read_write() -> Self {
        Self {
            mapping: KernelMappingRequest::initial_read_write(),
            placement: TranslationPlacement::ProviderSelected,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TranslationEdit {
    active: ActiveTranslation,
    kind: TranslationEditKind,
    provider_identity: String,
    metadata_backing: Option<PhysicalFrameExtent>,
    staged: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProvisionalKernelMapping {
    mapping: KernelMapping,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProvisionalFrameExtent {
    frames: PhysicalFrameExtent,
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
            edit_backing: Vec::new(),
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
    pub fn backing_frame_count(&self) -> u64 {
        self.backing.frame_count
            + self
                .edit_backing
                .iter()
                .map(|extent| extent.frame_count)
                .sum::<u64>()
    }

    /// Consume the active context into an exclusive mapping edit.
    ///
    /// # Errors
    ///
    /// Returns a stable error for provider mismatch or unavailable hierarchy
    /// backing. Failure consumes the active context into the fatal-only path.
    pub fn begin_map_edit(
        self,
        allocator: &mut FrameAllocatorContext,
        provider_identity: &str,
    ) -> Result<TranslationEdit, TranslationError> {
        if self.provider_identity != provider_identity {
            return Err(TranslationError::WrongProvider);
        }
        let metadata_backing = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 2,
                alignment_frames: 1,
            })
            .map_err(translation_frame_error)?;
        Ok(TranslationEdit {
            active: self,
            kind: TranslationEditKind::Map,
            provider_identity: provider_identity.into(),
            metadata_backing: Some(metadata_backing),
            staged: false,
        })
    }

    /// Consume the active context into an exclusive unmapping edit.
    ///
    /// # Errors
    ///
    /// Returns `WrongProvider` when the active space and provider differ.
    pub fn begin_unmap_edit(
        self,
        provider_identity: &str,
    ) -> Result<TranslationEdit, TranslationError> {
        if self.provider_identity != provider_identity {
            return Err(TranslationError::WrongProvider);
        }
        Ok(TranslationEdit {
            active: self,
            kind: TranslationEditKind::Unmap,
            provider_identity: provider_identity.into(),
            metadata_backing: None,
            staged: false,
        })
    }
}

impl TranslationEdit {
    #[must_use]
    pub const fn kind(&self) -> TranslationEditKind {
        self.kind
    }

    /// Consume frames into an inaccessible provisional mapping.
    ///
    /// # Errors
    ///
    /// Rejects the wrong edit kind, policy, provider, or frame provenance.
    pub fn map_frames(
        mut self,
        allocator: &mut FrameAllocatorContext,
        frames: PhysicalFrameExtent,
        request: TranslationMappingRequest,
    ) -> Result<(Self, ProvisionalKernelMapping), TranslationError> {
        if self.kind != TranslationEditKind::Map || self.staged {
            return Err(TranslationError::WrongEditKind);
        }
        if request != TranslationMappingRequest::initial_read_write() {
            return Err(TranslationError::UnsupportedRequest);
        }
        let mapping = allocator
            .map_kernel_frames(frames, request.mapping, self.provider_identity.clone())
            .map_err(translation_mapping_error)?;
        self.staged = true;
        Ok((self, ProvisionalKernelMapping { mapping }))
    }

    /// Publish a staged mapping and return the refined active context.
    ///
    /// # Errors
    ///
    /// Rejects incomplete, mismatched, or wrong-provider commits.
    pub fn commit_map(
        mut self,
        provisional: ProvisionalKernelMapping,
        provider_identity: &str,
    ) -> Result<(ActiveTranslation, KernelMapping), TranslationError> {
        if self.kind != TranslationEditKind::Map || !self.staged {
            return Err(TranslationError::WrongEditKind);
        }
        if self.provider_identity != provider_identity
            || provisional.mapping.provider_identity() != provider_identity
        {
            return Err(TranslationError::WrongProvider);
        }
        self.active.edit_backing.push(
            self.metadata_backing
                .take()
                .ok_or(TranslationError::BackingUnavailable)?,
        );
        Ok((self.active, provisional.mapping))
    }

    /// Consume a live mapping into an unavailable provisional frame extent.
    ///
    /// # Errors
    ///
    /// Rejects the wrong edit kind, provider, or allocator provenance.
    pub fn unmap(
        mut self,
        mapping: KernelMapping,
        allocator: &mut FrameAllocatorContext,
    ) -> Result<(Self, ProvisionalFrameExtent), TranslationError> {
        if self.kind != TranslationEditKind::Unmap || self.staged {
            return Err(TranslationError::WrongEditKind);
        }
        if mapping.provider_identity() != self.provider_identity {
            return Err(TranslationError::WrongMapping);
        }
        let frames = mapping
            .unmap(allocator)
            .map_err(translation_mapping_error)?;
        self.staged = true;
        Ok((self, ProvisionalFrameExtent { frames }))
    }

    /// Complete removal/invalidation and restore releasable frame ownership.
    ///
    /// # Errors
    ///
    /// Rejects incomplete or wrong-provider commits.
    pub fn commit_unmap(
        self,
        provisional: ProvisionalFrameExtent,
        provider_identity: &str,
    ) -> Result<(ActiveTranslation, PhysicalFrameExtent), TranslationError> {
        if self.kind != TranslationEditKind::Unmap || !self.staged {
            return Err(TranslationError::WrongEditKind);
        }
        if self.provider_identity != provider_identity {
            return Err(TranslationError::WrongProvider);
        }
        Ok((self.active, provisional.frames))
    }
}

fn translation_frame_error(_error: PhysicalFrameError) -> TranslationError {
    TranslationError::BackingUnavailable
}

fn translation_mapping_error(error: KernelMappingError) -> TranslationError {
    match error {
        KernelMappingError::UnsupportedPolicy => TranslationError::UnsupportedRequest,
        _ => TranslationError::WrongMapping,
    }
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

    #[test]
    fn active_edit_publishes_mapping_and_invalidates_before_frame_return() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-MAPPING-001.
        let mut allocator = allocator();
        let active = allocator
            .begin_translation_update(
                TranslationUpdateRequest::initial_bootstrap_equivalent(),
                "x86-provider/1",
            )
            .unwrap()
            .commit()
            .activate("x86-provider/1")
            .unwrap();
        let frames = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 1,
                alignment_frames: 1,
            })
            .unwrap();
        let edit = active
            .begin_map_edit(&mut allocator, "x86-provider/1")
            .unwrap();
        assert_eq!(edit.kind(), TranslationEditKind::Map);
        let (edit, provisional) = edit
            .map_frames(
                &mut allocator,
                frames,
                TranslationMappingRequest::initial_read_write(),
            )
            .unwrap();
        let (active, mut mapping) = edit.commit_map(provisional, "x86-provider/1").unwrap();
        assert_eq!(active.backing_frame_count(), 5);
        mapping.store_byte(0, 60).unwrap();
        assert_eq!(mapping.load_byte(0), Ok(60));

        let edit = active.begin_unmap_edit("x86-provider/1").unwrap();
        let (edit, provisional_frames) = edit.unmap(mapping, &mut allocator).unwrap();
        let (_active, frames) = edit
            .commit_unmap(provisional_frames, "x86-provider/1")
            .unwrap();
        allocator.release(frames).unwrap();
        assert_eq!(allocator.complete(), Err(PhysicalFrameError::LiveExtents));
    }

    #[test]
    fn active_edit_rejects_wrong_provider_kind_and_policy() {
        // TOPAL-SYSTEMS-MAPPING-001, TOPAL-SYSTEMS-QUALIFY-001.
        let mut allocator = allocator();
        let active = allocator
            .begin_translation_update(
                TranslationUpdateRequest::initial_bootstrap_equivalent(),
                "x86-provider/1",
            )
            .unwrap()
            .commit()
            .activate("x86-provider/1")
            .unwrap();
        assert_eq!(
            active.begin_unmap_edit("other-provider"),
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
        let frames = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 1,
                alignment_frames: 1,
            })
            .unwrap();
        let edit = active.begin_unmap_edit("x86-provider/1").unwrap();
        assert_eq!(
            edit.map_frames(
                &mut allocator,
                frames,
                TranslationMappingRequest::initial_read_write(),
            ),
            Err(TranslationError::WrongEditKind)
        );
    }
}
