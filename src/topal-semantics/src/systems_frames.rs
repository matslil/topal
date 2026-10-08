//! Architecture-independent affine physical-frame allocation.

use std::collections::BTreeMap;
use std::fmt;

use crate::{BootMemoryClass, BootMemoryProvenance, MemoryDescribedContext};

pub const SYSTEMS_FRAME_ALLOCATOR_CREATE: &str = "topal.systems.memory.create-frame-allocator/1";
pub const SYSTEMS_FRAMES_ALLOCATE: &str = "topal.systems.frames.allocate/1";
pub const SYSTEMS_FRAMES_RELEASE: &str = "topal.systems.frames.release/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalFrameRequest {
    pub frame_count: u64,
    pub alignment_frames: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalFrameExtent {
    allocator_identity: String,
    extent_identity: u64,
    model_start_frame: u64,
    pub frame_count: u64,
    pub alignment_frames: u64,
    pub provenance: Vec<BootMemoryProvenance>,
}

impl PhysicalFrameExtent {
    /// Return the mathematical frame index used only by the reference model.
    ///
    /// This value is not a machine address and grants no access authority.
    #[must_use]
    pub const fn model_start_frame(&self) -> u64 {
        self.model_start_frame
    }

    #[must_use]
    pub fn allocator_identity(&self) -> &str {
        &self.allocator_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameAllocatorFailureContext {
    pub handoff_identity: String,
    pub error: PhysicalFrameError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrameAllocatorResult {
    Ready(FrameAllocatorContext),
    Failure(FrameAllocatorFailureContext),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalFrameError {
    InvalidDescription,
    InvalidRequest,
    Exhausted,
    WrongAllocator,
    UnknownExtent,
    LiveExtents,
    MappedExtent,
}

impl fmt::Display for PhysicalFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDescription => "invalid-description",
            Self::InvalidRequest => "invalid-request",
            Self::Exhausted => "exhausted",
            Self::WrongAllocator => "wrong-allocator",
            Self::UnknownExtent => "unknown-extent",
            Self::LiveExtents => "live-extents",
            Self::MappedExtent => "mapped-extent",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FreeFrameRun {
    start_frame: u64,
    end_frame: u64,
    provenance: Vec<BootMemoryProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LiveFrameExtent {
    start_frame: u64,
    end_frame: u64,
    alignment_frames: u64,
    provenance: Vec<BootMemoryProvenance>,
    mapped: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameAllocatorContext {
    pub handoff_identity: String,
    allocator_identity: String,
    page_size: u64,
    free: Vec<FreeFrameRun>,
    live: BTreeMap<u64, LiveFrameExtent>,
    next_extent_identity: u64,
}

impl MemoryDescribedContext {
    #[must_use]
    pub fn create_frame_allocator(
        self,
        allocator_identity: impl Into<String>,
    ) -> FrameAllocatorResult {
        let allocator_identity = allocator_identity.into();
        if self.memory.page_size == 0 || !self.memory.page_size.is_power_of_two() {
            return FrameAllocatorResult::Failure(FrameAllocatorFailureContext {
                handoff_identity: self.handoff_identity,
                error: PhysicalFrameError::InvalidDescription,
            });
        }
        let mut free = self
            .memory
            .ranges
            .iter()
            .filter(|range| range.class == BootMemoryClass::Allocatable)
            .map(|range| FreeFrameRun {
                start_frame: range.start / self.memory.page_size,
                end_frame: range.end / self.memory.page_size,
                provenance: range.provenance.clone(),
            })
            .filter(|run| run.start_frame < run.end_frame)
            .collect::<Vec<_>>();
        free.sort_by_key(|run| run.start_frame);
        if allocator_identity.is_empty() || free.is_empty() {
            return FrameAllocatorResult::Failure(FrameAllocatorFailureContext {
                handoff_identity: self.handoff_identity,
                error: PhysicalFrameError::InvalidDescription,
            });
        }
        FrameAllocatorResult::Ready(FrameAllocatorContext {
            handoff_identity: self.handoff_identity,
            allocator_identity,
            page_size: self.memory.page_size,
            free,
            live: BTreeMap::new(),
            next_extent_identity: 0,
        })
    }
}

impl FrameAllocatorContext {
    #[must_use]
    pub fn allocator_identity(&self) -> &str {
        &self.allocator_identity
    }

    #[must_use]
    pub const fn page_size(&self) -> u64 {
        self.page_size
    }

    /// Allocate the lowest aligned extent satisfying the request.
    ///
    /// # Errors
    ///
    /// Returns an explicit malformed-request or exhaustion error without
    /// invalidating the allocator.
    pub fn allocate(
        &mut self,
        request: PhysicalFrameRequest,
    ) -> Result<PhysicalFrameExtent, PhysicalFrameError> {
        if request.frame_count == 0
            || request.alignment_frames == 0
            || !request.alignment_frames.is_power_of_two()
        {
            return Err(PhysicalFrameError::InvalidRequest);
        }
        for index in 0..self.free.len() {
            let run = &self.free[index];
            let Some(aligned_start) = align_up(run.start_frame, request.alignment_frames) else {
                return Err(PhysicalFrameError::Exhausted);
            };
            let Some(end_frame) = aligned_start.checked_add(request.frame_count) else {
                return Err(PhysicalFrameError::Exhausted);
            };
            if end_frame > run.end_frame {
                continue;
            }
            let selected = self.free.remove(index);
            if selected.start_frame < aligned_start {
                self.free.push(FreeFrameRun {
                    start_frame: selected.start_frame,
                    end_frame: aligned_start,
                    provenance: selected.provenance.clone(),
                });
            }
            if end_frame < selected.end_frame {
                self.free.push(FreeFrameRun {
                    start_frame: end_frame,
                    end_frame: selected.end_frame,
                    provenance: selected.provenance.clone(),
                });
            }
            self.free.sort_by_key(|free| free.start_frame);
            let extent_identity = self.next_extent_identity;
            self.next_extent_identity = self
                .next_extent_identity
                .checked_add(1)
                .ok_or(PhysicalFrameError::Exhausted)?;
            self.live.insert(
                extent_identity,
                LiveFrameExtent {
                    start_frame: aligned_start,
                    end_frame,
                    alignment_frames: request.alignment_frames,
                    provenance: selected.provenance.clone(),
                    mapped: false,
                },
            );
            return Ok(PhysicalFrameExtent {
                allocator_identity: self.allocator_identity.clone(),
                extent_identity,
                model_start_frame: aligned_start,
                frame_count: request.frame_count,
                alignment_frames: request.alignment_frames,
                provenance: selected.provenance,
            });
        }
        Err(PhysicalFrameError::Exhausted)
    }

    /// Consume and release an extent to its originating allocator.
    ///
    /// # Errors
    ///
    /// Returns a provenance error for a foreign, duplicated, or unknown
    /// extent.
    #[allow(clippy::needless_pass_by_value)] // Release consumes the affine extent token.
    pub fn release(&mut self, extent: PhysicalFrameExtent) -> Result<(), PhysicalFrameError> {
        if extent.allocator_identity != self.allocator_identity {
            return Err(PhysicalFrameError::WrongAllocator);
        }
        let live = self
            .live
            .get(&extent.extent_identity)
            .ok_or(PhysicalFrameError::UnknownExtent)?;
        if live.mapped {
            return Err(PhysicalFrameError::MappedExtent);
        }
        if live.start_frame != extent.model_start_frame
            || live.end_frame != extent.model_start_frame + extent.frame_count
            || live.alignment_frames != extent.alignment_frames
            || live.provenance != extent.provenance
        {
            return Err(PhysicalFrameError::UnknownExtent);
        }
        let Some(live) = self.live.remove(&extent.extent_identity) else {
            return Err(PhysicalFrameError::UnknownExtent);
        };
        self.free.push(FreeFrameRun {
            start_frame: live.start_frame,
            end_frame: live.end_frame,
            provenance: live.provenance,
        });
        self.free.sort_by_key(|run| run.start_frame);
        coalesce_free_runs(&mut self.free);
        Ok(())
    }

    /// Complete the allocator phase only when no extent remains live.
    ///
    /// # Errors
    ///
    /// Returns `LiveExtents` when affine ownership is not discharged.
    pub fn complete(self) -> Result<(), PhysicalFrameError> {
        if self.live.is_empty() {
            Ok(())
        } else {
            Err(PhysicalFrameError::LiveExtents)
        }
    }

    pub(crate) fn begin_mapping(
        &mut self,
        extent: &PhysicalFrameExtent,
    ) -> Result<(), PhysicalFrameError> {
        if extent.allocator_identity != self.allocator_identity {
            return Err(PhysicalFrameError::WrongAllocator);
        }
        let live = self
            .live
            .get_mut(&extent.extent_identity)
            .ok_or(PhysicalFrameError::UnknownExtent)?;
        if live.mapped {
            return Err(PhysicalFrameError::MappedExtent);
        }
        if live.start_frame != extent.model_start_frame
            || live.end_frame != extent.model_start_frame + extent.frame_count
            || live.alignment_frames != extent.alignment_frames
            || live.provenance != extent.provenance
        {
            return Err(PhysicalFrameError::UnknownExtent);
        }
        live.mapped = true;
        Ok(())
    }

    pub(crate) fn finish_mapping(
        &mut self,
        extent: &PhysicalFrameExtent,
    ) -> Result<(), PhysicalFrameError> {
        if extent.allocator_identity != self.allocator_identity {
            return Err(PhysicalFrameError::WrongAllocator);
        }
        let live = self
            .live
            .get_mut(&extent.extent_identity)
            .ok_or(PhysicalFrameError::UnknownExtent)?;
        if !live.mapped {
            return Err(PhysicalFrameError::UnknownExtent);
        }
        live.mapped = false;
        Ok(())
    }
}

fn align_up(value: u64, alignment: u64) -> Option<u64> {
    value
        .checked_add(alignment - 1)
        .map(|sum| sum & !(alignment - 1))
}

fn coalesce_free_runs(free: &mut Vec<FreeFrameRun>) {
    let mut merged: Vec<FreeFrameRun> = Vec::with_capacity(free.len());
    for run in free.drain(..) {
        if let Some(previous) = merged.last_mut()
            && previous.end_frame == run.start_frame
            && previous.provenance == run.provenance
        {
            previous.end_frame = run.end_frame;
        } else {
            merged.push(run);
        }
    }
    *free = merged;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BootMemoryDescription, BootMemoryProvenance, MemoryDescribedContext,
        NormalizedBootMemoryRange,
    };

    fn described(start: u64, end: u64, identity: &str) -> MemoryDescribedContext {
        MemoryDescribedContext {
            handoff_identity: "handoff/1".into(),
            memory: BootMemoryDescription {
                page_size: 4096,
                ranges: vec![NormalizedBootMemoryRange {
                    start,
                    end,
                    class: BootMemoryClass::Allocatable,
                    provenance: vec![BootMemoryProvenance {
                        class: BootMemoryClass::Allocatable,
                        source_kind: "ram".into(),
                        provider: identity.into(),
                    }],
                }],
            },
        }
    }

    #[test]
    fn allocates_aligned_nonoverlapping_extents_and_reuses_release() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-FRAMES-001.
        let FrameAllocatorResult::Ready(mut allocator) =
            described(0x1000, 0x11_000, "provider/1").create_frame_allocator("allocator/1")
        else {
            panic!("valid description must create an allocator")
        };
        let first = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 2,
                alignment_frames: 4,
            })
            .unwrap();
        let duplicate = first.clone();
        let second = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 2,
                alignment_frames: 4,
            })
            .unwrap();
        assert_eq!(first.model_start_frame(), 4);
        assert_eq!(second.model_start_frame(), 8);
        allocator.release(first).unwrap();
        assert_eq!(
            allocator.release(duplicate),
            Err(PhysicalFrameError::UnknownExtent)
        );
        allocator.release(second).unwrap();
        allocator.complete().unwrap();
    }

    #[test]
    fn rejects_malformed_exhausted_and_foreign_requests_without_losing_state() {
        // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-QUALIFY-001.
        let FrameAllocatorResult::Ready(mut allocator) =
            described(0x1000, 0x5000, "provider/1").create_frame_allocator("allocator/1")
        else {
            panic!("valid description must create an allocator")
        };
        assert_eq!(
            allocator.allocate(PhysicalFrameRequest {
                frame_count: 0,
                alignment_frames: 1,
            }),
            Err(PhysicalFrameError::InvalidRequest)
        );
        assert_eq!(
            allocator.allocate(PhysicalFrameRequest {
                frame_count: 5,
                alignment_frames: 1,
            }),
            Err(PhysicalFrameError::Exhausted)
        );
        let FrameAllocatorResult::Ready(mut foreign) =
            described(0x8000, 0xc000, "provider/2").create_frame_allocator("allocator/2")
        else {
            panic!("valid description must create an allocator")
        };
        let extent = allocator
            .allocate(PhysicalFrameRequest {
                frame_count: 1,
                alignment_frames: 1,
            })
            .unwrap();
        assert_eq!(
            foreign.release(extent.clone()),
            Err(PhysicalFrameError::WrongAllocator)
        );
        assert_eq!(
            allocator.clone().complete(),
            Err(PhysicalFrameError::LiveExtents)
        );
        allocator.release(extent).unwrap();
        allocator.complete().unwrap();
    }

    #[test]
    fn consumes_invalid_or_empty_descriptions_into_fatal_only_failure() {
        // TOPAL-SYSTEMS-FRAMES-001.
        let invalid = described(0x1000, 0x5000, "provider/1").create_frame_allocator("");
        assert!(matches!(
            invalid,
            FrameAllocatorResult::Failure(FrameAllocatorFailureContext {
                error: PhysicalFrameError::InvalidDescription,
                ..
            })
        ));
        let empty = MemoryDescribedContext {
            handoff_identity: "handoff/1".into(),
            memory: BootMemoryDescription {
                page_size: 4096,
                ranges: Vec::new(),
            },
        }
        .create_frame_allocator("allocator/1");
        assert!(matches!(empty, FrameAllocatorResult::Failure(_)));
    }
}
