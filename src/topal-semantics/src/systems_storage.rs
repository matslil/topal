//! Architecture-independent bounded bootstrap-storage model.

use std::collections::BTreeMap;

use crate::SystemsModelError;

pub const SYSTEMS_BOOTSTRAP_STORAGE_PROVISION: &str = "topal.systems.storage.bootstrap.provision/1";
pub const SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE: &str = "topal.systems.storage.bootstrap.allocate/1";
pub const SYSTEMS_BOOTSTRAP_STORAGE_RELEASE: &str = "topal.systems.storage.bootstrap.release/1";
pub const SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE: &str = "topal.systems.storage.bootstrap.complete/1";
pub const SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE: &str =
    "topal.systems.storage.bootstrap-region.store-byte/1";
pub const SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE: &str =
    "topal.systems.storage.bootstrap-region.load-byte/1";
pub const SYSTEMS_BOOTSTRAP_STORAGE_INVALID_REQUEST: &str =
    "topal.systems.storage.error.invalid-request/1";
pub const SYSTEMS_BOOTSTRAP_STORAGE_EXHAUSTED: &str = "topal.systems.storage.error.exhausted/1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapStorageDescriptor {
    pub capacity_bytes: u64,
    pub alignment_bytes: u64,
}

impl BootstrapStorageDescriptor {
    /// Validate the artifact-provided storage bound.
    ///
    /// # Errors
    ///
    /// Returns `E-SYSTEMS-STORAGE` when capacity or alignment is zero, the
    /// alignment is not a power of two, or alignment exceeds capacity.
    pub fn validate(&self) -> Result<(), SystemsModelError> {
        if self.capacity_bytes == 0
            || self.alignment_bytes == 0
            || !self.alignment_bytes.is_power_of_two()
            || self.alignment_bytes > self.capacity_bytes
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE",
                "bounded bootstrap storage requires positive capacity and power-of-two alignment not exceeding capacity",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapStoragePlacement {
    BootstrapReclaimable,
    PageAlignedTable,
}

impl BootstrapStoragePlacement {
    #[must_use]
    pub const fn semantic_identity(self) -> &'static str {
        match self {
            Self::BootstrapReclaimable => "topal.systems.placement.bootstrap-reclaimable/1",
            Self::PageAlignedTable => "topal.systems.placement.page-aligned-table/1",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootstrapStorageRequest {
    pub byte_count: u64,
    pub alignment_bytes: u64,
    pub placement: BootstrapStoragePlacement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapStorageErrorCode {
    InvalidRequest,
    Exhausted,
}

impl BootstrapStorageErrorCode {
    #[must_use]
    pub const fn semantic_identity(self) -> &'static str {
        match self {
            Self::InvalidRequest => SYSTEMS_BOOTSTRAP_STORAGE_INVALID_REQUEST,
            Self::Exhausted => SYSTEMS_BOOTSTRAP_STORAGE_EXHAUSTED,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootstrapStorageTransition {
    Provisioned {
        capacity_bytes: u64,
        alignment_bytes: u64,
    },
    Allocated {
        allocation_id: u64,
        offset_bytes: u64,
        request: BootstrapStorageRequest,
    },
    AllocationFailed {
        request: BootstrapStorageRequest,
        code: BootstrapStorageErrorCode,
    },
    Released {
        allocation_id: u64,
    },
    StoredByte {
        allocation_id: u64,
        offset_bytes: u64,
        value: u8,
    },
    LoadedByte {
        allocation_id: u64,
        offset_bytes: u64,
        value: u8,
    },
    Completed,
}

impl BootstrapStorageTransition {
    #[must_use]
    pub const fn semantic_identity(&self) -> &'static str {
        match self {
            Self::Provisioned { .. } => SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
            Self::Allocated { .. } => SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            Self::AllocationFailed { code, .. } => code.semantic_identity(),
            Self::Released { .. } => SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            Self::StoredByte { .. } => SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            Self::LoadedByte { .. } => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            Self::Completed => SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BootstrapRegion {
    execution_identity: String,
    allocation_id: u64,
    offset_bytes: u64,
    byte_count: u64,
    alignment_bytes: u64,
    placement: BootstrapStoragePlacement,
}

impl BootstrapRegion {
    #[must_use]
    pub const fn offset_bytes(&self) -> u64 {
        self.offset_bytes
    }

    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    #[must_use]
    pub const fn alignment_bytes(&self) -> u64 {
        self.alignment_bytes
    }

    #[must_use]
    pub const fn placement(&self) -> BootstrapStoragePlacement {
        self.placement
    }
}

#[derive(Debug)]
pub struct BootstrapStorageState {
    descriptor: BootstrapStorageDescriptor,
    execution_identity: String,
    next_offset: u64,
    next_allocation_id: u64,
    live_regions: BTreeMap<u64, (u64, u64)>,
    contents: BTreeMap<(u64, u64), u8>,
    transitions: Vec<BootstrapStorageTransition>,
    complete: bool,
}

impl BootstrapStorageState {
    /// Construct one live monotonic pool from checked artifact evidence.
    ///
    /// # Errors
    ///
    /// Returns `E-SYSTEMS-STORAGE` when the descriptor is invalid.
    pub fn new(
        descriptor: BootstrapStorageDescriptor,
        execution_identity: impl Into<String>,
    ) -> Result<Self, SystemsModelError> {
        descriptor.validate()?;
        let execution_identity = execution_identity.into();
        if execution_identity.is_empty() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-PROVENANCE",
                "bootstrap storage requires a nonempty execution identity",
            ));
        }
        Ok(Self {
            transitions: vec![BootstrapStorageTransition::Provisioned {
                capacity_bytes: descriptor.capacity_bytes,
                alignment_bytes: descriptor.alignment_bytes,
            }],
            descriptor,
            execution_identity,
            next_offset: 0,
            next_allocation_id: 0,
            live_regions: BTreeMap::new(),
            contents: BTreeMap::new(),
            complete: false,
        })
    }

    #[must_use]
    pub const fn descriptor(&self) -> &BootstrapStorageDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn execution_identity(&self) -> &str {
        &self.execution_identity
    }

    #[must_use]
    pub const fn consumed_bytes(&self) -> u64 {
        self.next_offset
    }

    #[must_use]
    pub fn live_region_count(&self) -> usize {
        self.live_regions.len()
    }

    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    #[must_use]
    pub fn transitions(&self) -> &[BootstrapStorageTransition] {
        &self.transitions
    }

    /// Allocate one affine region without reusing released bytes.
    ///
    /// # Errors
    ///
    /// Returns `InvalidRequest` for a zero-sized, wrongly aligned, overflowing,
    /// or post-bootstrap request. Returns `Exhausted` when the aligned request
    /// does not fit the remaining bounded capacity.
    pub fn allocate(
        &mut self,
        request: BootstrapStorageRequest,
    ) -> Result<BootstrapRegion, BootstrapStorageErrorCode> {
        if self.complete
            || request.byte_count == 0
            || request.alignment_bytes == 0
            || !request.alignment_bytes.is_power_of_two()
            || request.alignment_bytes > self.descriptor.alignment_bytes
        {
            return self.allocation_failure(request, BootstrapStorageErrorCode::InvalidRequest);
        }
        let remainder = self.next_offset % request.alignment_bytes;
        let padding = if remainder == 0 {
            0
        } else {
            request.alignment_bytes - remainder
        };
        let Some(offset_bytes) = self.next_offset.checked_add(padding) else {
            return self.allocation_failure(request, BootstrapStorageErrorCode::InvalidRequest);
        };
        let Some(end) = offset_bytes.checked_add(request.byte_count) else {
            return self.allocation_failure(request, BootstrapStorageErrorCode::InvalidRequest);
        };
        if end > self.descriptor.capacity_bytes {
            return self.allocation_failure(request, BootstrapStorageErrorCode::Exhausted);
        }
        let allocation_id = self.next_allocation_id;
        let Some(next_allocation_id) = self.next_allocation_id.checked_add(1) else {
            return self.allocation_failure(request, BootstrapStorageErrorCode::InvalidRequest);
        };
        self.next_allocation_id = next_allocation_id;
        self.next_offset = end;
        self.live_regions
            .insert(allocation_id, (offset_bytes, request.byte_count));
        self.transitions
            .push(BootstrapStorageTransition::Allocated {
                allocation_id,
                offset_bytes,
                request,
            });
        Ok(BootstrapRegion {
            execution_identity: self.execution_identity.clone(),
            allocation_id,
            offset_bytes,
            byte_count: request.byte_count,
            alignment_bytes: request.alignment_bytes,
            placement: request.placement,
        })
    }

    fn allocation_failure(
        &mut self,
        request: BootstrapStorageRequest,
        code: BootstrapStorageErrorCode,
    ) -> Result<BootstrapRegion, BootstrapStorageErrorCode> {
        self.transitions
            .push(BootstrapStorageTransition::AllocationFailed { request, code });
        Err(code)
    }

    /// Store one byte through a borrowed live bootstrap region.
    ///
    /// # Errors
    ///
    /// Returns a stable storage diagnostic when the region belongs to another
    /// execution, is no longer live, or the offset is outside its byte count.
    pub fn store_byte(
        &mut self,
        region: &BootstrapRegion,
        offset_bytes: u64,
        value: u8,
    ) -> Result<(), SystemsModelError> {
        let allocation_id = self.validate_byte_access(region, offset_bytes)?;
        self.contents.insert((allocation_id, offset_bytes), value);
        self.transitions
            .push(BootstrapStorageTransition::StoredByte {
                allocation_id,
                offset_bytes,
                value,
            });
        Ok(())
    }

    /// Load one byte through a borrowed live bootstrap region.
    ///
    /// Unwritten bytes read as zero because the artifact-provided pool is
    /// zero-initialized before bootstrap entry.
    ///
    /// # Errors
    ///
    /// Returns a stable storage diagnostic when the region belongs to another
    /// execution, is no longer live, or the offset is outside its byte count.
    pub fn load_byte(
        &mut self,
        region: &BootstrapRegion,
        offset_bytes: u64,
    ) -> Result<u8, SystemsModelError> {
        let allocation_id = self.validate_byte_access(region, offset_bytes)?;
        let value = self
            .contents
            .get(&(allocation_id, offset_bytes))
            .copied()
            .unwrap_or(0);
        self.transitions
            .push(BootstrapStorageTransition::LoadedByte {
                allocation_id,
                offset_bytes,
                value,
            });
        Ok(value)
    }

    fn validate_byte_access(
        &self,
        region: &BootstrapRegion,
        offset_bytes: u64,
    ) -> Result<u64, SystemsModelError> {
        if region.execution_identity != self.execution_identity {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-PROVENANCE",
                "bootstrap region belongs to another execution",
            ));
        }
        if self.live_regions.get(&region.allocation_id)
            != Some(&(region.offset_bytes, region.byte_count))
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-REGION",
                "bootstrap region is not live in this storage pool",
            ));
        }
        if offset_bytes >= region.byte_count {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-BOUNDS",
                format!(
                    "bootstrap byte offset {offset_bytes} is outside region size {}",
                    region.byte_count
                ),
            ));
        }
        Ok(region.allocation_id)
    }

    pub(crate) fn atomic_store_word(
        &mut self,
        region: &BootstrapRegion,
        offset_bytes: u64,
        value: u64,
    ) -> Result<(), SystemsModelError> {
        let allocation_id = self.validate_word_access(region, offset_bytes)?;
        for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.contents
                .insert((allocation_id, offset_bytes + index as u64), byte);
        }
        Ok(())
    }

    pub(crate) fn atomic_load_word(
        &self,
        region: &BootstrapRegion,
        offset_bytes: u64,
    ) -> Result<u64, SystemsModelError> {
        let allocation_id = self.validate_word_access(region, offset_bytes)?;
        let mut bytes = [0_u8; 8];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = self
                .contents
                .get(&(allocation_id, offset_bytes + index as u64))
                .copied()
                .unwrap_or(0);
        }
        Ok(u64::from_le_bytes(bytes))
    }

    fn validate_word_access(
        &self,
        region: &BootstrapRegion,
        offset_bytes: u64,
    ) -> Result<u64, SystemsModelError> {
        let end = offset_bytes.checked_add(8).ok_or_else(|| {
            SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-BOUNDS",
                "atomic word extent overflows its region",
            )
        })?;
        if end > region.byte_count {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-BOUNDS",
                "atomic word must be wholly contained in its region",
            ));
        }
        self.validate_byte_access(region, offset_bytes)
    }

    /// Consume one region while retaining monotonic pool consumption.
    ///
    /// # Errors
    ///
    /// Returns `E-SYSTEMS-STORAGE-REGION` when the region is not live in this
    /// pool.
    pub fn release(&mut self, region: BootstrapRegion) -> Result<(), SystemsModelError> {
        let BootstrapRegion {
            execution_identity,
            allocation_id,
            offset_bytes,
            byte_count,
            alignment_bytes: _,
            placement: _,
        } = region;
        if execution_identity != self.execution_identity {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-PROVENANCE",
                "bootstrap region belongs to another execution",
            ));
        }
        let expected = (offset_bytes, byte_count);
        if self.live_regions.remove(&allocation_id) != Some(expected) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-REGION",
                "bootstrap region is not live in this storage pool",
            ));
        }
        self.contents
            .retain(|(owner, _), _| *owner != allocation_id);
        self.transitions
            .push(BootstrapStorageTransition::Released { allocation_id });
        Ok(())
    }

    /// Complete bootstrap and make the complete pool reclaimable.
    ///
    /// # Errors
    ///
    /// Returns `E-SYSTEMS-STORAGE-LIVE` while a region remains live or
    /// `E-SYSTEMS-STORAGE-PHASE` after bootstrap has already completed.
    pub fn complete_bootstrap(&mut self) -> Result<(), SystemsModelError> {
        if self.complete {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-PHASE",
                "bootstrap storage has already completed",
            ));
        }
        if !self.live_regions.is_empty() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-STORAGE-LIVE",
                "bootstrap cannot complete while a storage region remains live",
            ));
        }
        self.complete = true;
        self.transitions.push(BootstrapStorageTransition::Completed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage() -> BootstrapStorageState {
        BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 16_384,
                alignment_bytes: 4096,
            },
            "test-execution",
        )
        .unwrap()
    }

    fn request(byte_count: u64, alignment_bytes: u64) -> BootstrapStorageRequest {
        BootstrapStorageRequest {
            byte_count,
            alignment_bytes,
            placement: BootstrapStoragePlacement::BootstrapReclaimable,
        }
    }

    #[test]
    fn allocation_is_aligned_bounded_and_monotonic() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STORAGE-001.
        let mut storage = storage();
        let first = storage.allocate(request(17, 16)).unwrap();
        let second = storage.allocate(request(4096, 4096)).unwrap();
        assert_eq!(first.offset_bytes(), 0);
        assert_eq!(second.offset_bytes(), 4096);
        assert_eq!(storage.consumed_bytes(), 8192);
        storage.release(first).unwrap();
        let third = storage.allocate(request(16, 16)).unwrap();
        assert_eq!(third.offset_bytes(), 8192);
        assert_eq!(storage.live_region_count(), 2);
        assert_eq!(
            storage
                .transitions()
                .iter()
                .map(BootstrapStorageTransition::semantic_identity)
                .collect::<Vec<_>>(),
            [
                SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
                SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            ]
        );
    }

    #[test]
    fn malformed_and_exhausted_requests_are_distinct() {
        // TOPAL-SYSTEMS-STORAGE-001.
        let mut storage = storage();
        assert_eq!(
            storage.allocate(request(0, 16)).unwrap_err(),
            BootstrapStorageErrorCode::InvalidRequest
        );
        assert_eq!(
            storage.allocate(request(1, 8192)).unwrap_err(),
            BootstrapStorageErrorCode::InvalidRequest
        );
        assert_eq!(
            storage.allocate(request(16_385, 4096)).unwrap_err(),
            BootstrapStorageErrorCode::Exhausted
        );
        assert_eq!(storage.consumed_bytes(), 0);
        assert_eq!(
            storage
                .transitions()
                .iter()
                .map(BootstrapStorageTransition::semantic_identity)
                .collect::<Vec<_>>(),
            [
                SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
                SYSTEMS_BOOTSTRAP_STORAGE_INVALID_REQUEST,
                SYSTEMS_BOOTSTRAP_STORAGE_INVALID_REQUEST,
                SYSTEMS_BOOTSTRAP_STORAGE_EXHAUSTED,
            ]
        );
    }

    #[test]
    fn byte_access_retains_contents_and_checks_bounds_and_provenance() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STORAGE-001.
        let mut storage = storage();
        let region = storage.allocate(request(8, 8)).unwrap();
        assert_eq!(storage.load_byte(&region, 0).unwrap(), 0);
        storage.store_byte(&region, 3, 90).unwrap();
        assert_eq!(storage.load_byte(&region, 3).unwrap(), 90);
        assert_eq!(
            storage.load_byte(&region, 8).unwrap_err().code,
            "E-SYSTEMS-STORAGE-BOUNDS"
        );

        let mut other = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 16_384,
                alignment_bytes: 4096,
            },
            "other-execution",
        )
        .unwrap();
        assert_eq!(
            other.store_byte(&region, 0, 1).unwrap_err().code,
            "E-SYSTEMS-STORAGE-PROVENANCE"
        );
        storage.release(region).unwrap();
        assert_eq!(
            storage
                .transitions()
                .iter()
                .map(BootstrapStorageTransition::semantic_identity)
                .collect::<Vec<_>>(),
            [
                SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
                SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
                SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
                SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
                SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
                SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            ]
        );
    }

    #[test]
    fn completion_requires_released_regions_and_closes_the_pool() {
        // TOPAL-SYSTEMS-AUTHORITY-001, TOPAL-SYSTEMS-STORAGE-001.
        let mut storage = storage();
        let region = storage.allocate(request(4096, 4096)).unwrap();
        assert_eq!(
            storage.complete_bootstrap().unwrap_err().code,
            "E-SYSTEMS-STORAGE-LIVE"
        );
        storage.release(region).unwrap();
        storage.complete_bootstrap().unwrap();
        assert!(storage.is_complete());
        assert_eq!(
            storage.transitions().last(),
            Some(&BootstrapStorageTransition::Completed)
        );
        assert_eq!(
            storage.allocate(request(16, 16)).unwrap_err(),
            BootstrapStorageErrorCode::InvalidRequest
        );
    }

    #[test]
    fn descriptor_validation_fails_closed() {
        for descriptor in [
            BootstrapStorageDescriptor {
                capacity_bytes: 0,
                alignment_bytes: 4096,
            },
            BootstrapStorageDescriptor {
                capacity_bytes: 4096,
                alignment_bytes: 0,
            },
            BootstrapStorageDescriptor {
                capacity_bytes: 4096,
                alignment_bytes: 3,
            },
            BootstrapStorageDescriptor {
                capacity_bytes: 1024,
                alignment_bytes: 4096,
            },
        ] {
            assert_eq!(descriptor.validate().unwrap_err().code, "E-SYSTEMS-STORAGE");
        }
    }

    #[test]
    fn region_provenance_prevents_cross_execution_release() {
        // TOPAL-SYSTEMS-AUTHORITY-001, TOPAL-SYSTEMS-STORAGE-001.
        let descriptor = BootstrapStorageDescriptor {
            capacity_bytes: 16_384,
            alignment_bytes: 4096,
        };
        let mut first = BootstrapStorageState::new(descriptor.clone(), "first").unwrap();
        let mut second = BootstrapStorageState::new(descriptor, "second").unwrap();
        let region = first.allocate(request(4096, 4096)).unwrap();
        assert_eq!(
            second.release(region).unwrap_err().code,
            "E-SYSTEMS-STORAGE-PROVENANCE"
        );
        assert_eq!(first.live_region_count(), 1);
        assert_eq!(second.live_region_count(), 0);
    }

    #[test]
    fn storage_session_requires_execution_provenance() {
        // TOPAL-SYSTEMS-AUTHORITY-001.
        assert_eq!(
            BootstrapStorageState::new(
                BootstrapStorageDescriptor {
                    capacity_bytes: 4096,
                    alignment_bytes: 4096,
                },
                "",
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-STORAGE-PROVENANCE"
        );
    }
}
