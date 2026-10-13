//! Architecture-independent ownership model for opaque kernel mappings.

use std::collections::BTreeMap;
use std::fmt;

use crate::{FrameAllocatorContext, PhysicalFrameError, PhysicalFrameExtent};

pub const SYSTEMS_KERNEL_MAP: &str = "topal.systems.mapping.kernel.map/1";
pub const SYSTEMS_KERNEL_UNMAP: &str = "topal.systems.mapping.kernel.unmap/1";
pub const SYSTEMS_KERNEL_MAPPING_STORE_BYTE: &str = "topal.systems.mapping.kernel.store-byte/1";
pub const SYSTEMS_KERNEL_MAPPING_LOAD_BYTE: &str = "topal.systems.mapping.kernel.load-byte/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelMappingRights {
    ReadOnly,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelExecutionPolicy {
    Denied,
    Allowed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelMemoryKind {
    Normal,
    Device,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelMappingRequest {
    pub rights: KernelMappingRights,
    pub execution: KernelExecutionPolicy,
    pub memory_kind: KernelMemoryKind,
}

impl KernelMappingRequest {
    #[must_use]
    pub const fn initial_read_write() -> Self {
        Self {
            rights: KernelMappingRights::ReadWrite,
            execution: KernelExecutionPolicy::Denied,
            memory_kind: KernelMemoryKind::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelMappingError {
    UnsupportedPolicy,
    WrongAllocator,
    UnknownExtent,
    ExtentAlreadyMapped,
    SizeOverflow,
    OutOfBounds,
    ReadOnly,
    UninitializedByte,
}

impl fmt::Display for KernelMappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPolicy => "unsupported-policy",
            Self::WrongAllocator => "wrong-allocator",
            Self::UnknownExtent => "unknown-extent",
            Self::ExtentAlreadyMapped => "extent-already-mapped",
            Self::SizeOverflow => "size-overflow",
            Self::OutOfBounds => "out-of-bounds",
            Self::ReadOnly => "read-only",
            Self::UninitializedByte => "uninitialized-byte",
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct KernelMapping {
    provider_identity: String,
    model_virtual_identity: String,
    request: KernelMappingRequest,
    byte_count: u64,
    frames: PhysicalFrameExtent,
    contents: BTreeMap<u64, u8>,
}

impl FrameAllocatorContext {
    /// Consume one owned frame extent into a provider-qualified kernel mapping.
    ///
    /// # Errors
    ///
    /// Returns a stable mapping error for unsupported policy, provenance,
    /// duplicate mapping, or size overflow.
    pub fn map_kernel_frames(
        &mut self,
        frames: PhysicalFrameExtent,
        request: KernelMappingRequest,
        provider_identity: impl Into<String>,
    ) -> Result<KernelMapping, KernelMappingError> {
        if request != KernelMappingRequest::initial_read_write() {
            return Err(KernelMappingError::UnsupportedPolicy);
        }
        let provider_identity = provider_identity.into();
        if provider_identity.is_empty() {
            return Err(KernelMappingError::UnsupportedPolicy);
        }
        let byte_count = frames
            .frame_count
            .checked_mul(self.page_size())
            .ok_or(KernelMappingError::SizeOverflow)?;
        self.begin_mapping(&frames).map_err(mapping_frame_error)?;
        Ok(KernelMapping {
            model_virtual_identity: format!(
                "{provider_identity}/kernel-virtual/{}",
                frames.model_start_frame()
            ),
            provider_identity,
            request,
            byte_count,
            frames,
            contents: BTreeMap::new(),
        })
    }
}

impl KernelMapping {
    #[must_use]
    pub const fn request(&self) -> KernelMappingRequest {
        self.request
    }

    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    #[must_use]
    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    /// Store one ordinary byte through the live mapping.
    ///
    /// # Errors
    ///
    /// Returns `ReadOnly` or `OutOfBounds` when the mapping does not authorize
    /// the requested store.
    pub fn store_byte(&mut self, offset: u64, value: u8) -> Result<(), KernelMappingError> {
        if self.request.rights != KernelMappingRights::ReadWrite {
            return Err(KernelMappingError::ReadOnly);
        }
        if offset >= self.byte_count {
            return Err(KernelMappingError::OutOfBounds);
        }
        self.contents.insert(offset, value);
        Ok(())
    }

    /// Load one initialized ordinary byte through the live mapping.
    ///
    /// # Errors
    ///
    /// Returns a bounds or initialization error without exposing an address.
    pub fn load_byte(&self, offset: u64) -> Result<u8, KernelMappingError> {
        if offset >= self.byte_count {
            return Err(KernelMappingError::OutOfBounds);
        }
        self.contents
            .get(&offset)
            .copied()
            .ok_or(KernelMappingError::UninitializedByte)
    }

    /// Consume this mapping and return its original physical-frame extent.
    ///
    /// # Errors
    ///
    /// Returns a provenance error when the wrong allocator attempts unmap.
    pub fn unmap(
        self,
        allocator: &mut FrameAllocatorContext,
    ) -> Result<PhysicalFrameExtent, KernelMappingError> {
        allocator
            .finish_mapping(&self.frames)
            .map_err(mapping_frame_error)?;
        Ok(self.frames)
    }
}

fn mapping_frame_error(error: PhysicalFrameError) -> KernelMappingError {
    match error {
        PhysicalFrameError::WrongAllocator => KernelMappingError::WrongAllocator,
        PhysicalFrameError::MappedExtent => KernelMappingError::ExtentAlreadyMapped,
        _ => KernelMappingError::UnknownExtent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BootMemoryClass, BootMemoryDescription, BootMemoryProvenance, FrameAllocatorResult,
        MemoryDescribedContext, NormalizedBootMemoryRange, PhysicalFrameRequest,
    };

    fn make_allocator(identity: &str) -> FrameAllocatorContext {
        let described = MemoryDescribedContext {
            handoff_identity: "handoff/1".into(),
            memory: BootMemoryDescription {
                page_size: 4096,
                ranges: vec![NormalizedBootMemoryRange {
                    start: 0x1000,
                    end: 0x5000,
                    class: BootMemoryClass::Allocatable,
                    provenance: vec![BootMemoryProvenance {
                        class: BootMemoryClass::Allocatable,
                        source_kind: "ram".into(),
                        provider: "provider/1".into(),
                    }],
                }],
            },
        };
        let FrameAllocatorResult::Ready(allocator) = described.create_frame_allocator(identity)
        else {
            panic!("valid description must create an allocator")
        };
        allocator
    }

    #[test]
    fn mapping_owns_access_and_returns_the_original_extent() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-MAPPING-001.
        let mut allocator = make_allocator("allocator/1");
        let frames = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 1,
                alignment_frames: 1,
            })
            .unwrap();
        let duplicate = frames.clone();
        let mut mapping = allocator
            .map_kernel_frames(
                frames,
                KernelMappingRequest::initial_read_write(),
                "mapping-provider/1",
            )
            .unwrap();
        assert_eq!(mapping.byte_count(), 4096);
        assert_eq!(
            allocator.release(duplicate),
            Err(PhysicalFrameError::MappedExtent)
        );
        mapping.store_byte(0, 165).unwrap();
        assert_eq!(mapping.load_byte(0), Ok(165));
        let frames = mapping.unmap(&mut allocator).unwrap();
        allocator.release(frames).unwrap();
        allocator.complete().unwrap();
    }

    #[test]
    fn mapping_rejects_policy_bounds_and_wrong_allocator() {
        // TOPAL-SYSTEMS-MAPPING-001, TOPAL-SYSTEMS-QUALIFY-001.
        let mut allocator = make_allocator("allocator/1");
        let frames = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 1,
                alignment_frames: 1,
            })
            .unwrap();
        let invalid = KernelMappingRequest {
            rights: KernelMappingRights::ReadWrite,
            execution: KernelExecutionPolicy::Allowed,
            memory_kind: KernelMemoryKind::Normal,
        };
        let frames = allocator
            .map_kernel_frames(frames, invalid, "mapping-provider/1")
            .unwrap_err();
        assert_eq!(frames, KernelMappingError::UnsupportedPolicy);

        let frames = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 1,
                alignment_frames: 1,
            })
            .unwrap();
        let mut mapping = allocator
            .map_kernel_frames(
                frames,
                KernelMappingRequest::initial_read_write(),
                "mapping-provider/1",
            )
            .unwrap();
        assert_eq!(
            mapping.load_byte(0),
            Err(KernelMappingError::UninitializedByte)
        );
        assert_eq!(
            mapping.store_byte(4096, 1),
            Err(KernelMappingError::OutOfBounds)
        );
        let mut foreign = make_allocator("allocator/2");
        assert_eq!(
            mapping.unmap(&mut foreign),
            Err(KernelMappingError::WrongAllocator)
        );
    }
}
