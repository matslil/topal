//! Architecture-independent atomic-location model.

use crate::{BootstrapRegion, BootstrapStorageState, SystemsModelError};

pub const SYSTEMS_ATOMIC_WORD_CREATE: &str = "topal.systems.atomic.word.create/1";
pub const SYSTEMS_ATOMIC_COMPARE_EXCHANGE: &str = "topal.systems.atomic.word.compare-exchange/1";
pub const SYSTEMS_ATOMIC_LOAD: &str = "topal.systems.atomic.word.load/1";
pub const SYSTEMS_ATOMIC_END: &str = "topal.systems.atomic.word.end/1";
pub const INITIAL_ATOMIC_WORD_BYTES: u64 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicDomain {
    CpuShared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicOrder {
    AtomicOnly,
    Acquire,
    Release,
    AcquireRelease,
    Sequential,
}

impl AtomicOrder {
    const fn admits_load(self) -> bool {
        matches!(self, Self::AtomicOnly | Self::Acquire | Self::Sequential)
    }

    const fn admits_failure(self) -> bool {
        matches!(self, Self::AtomicOnly | Self::Acquire)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AtomicWordRequest {
    pub offset_bytes: u64,
    pub initial_value: u64,
    pub domain: AtomicDomain,
}

impl AtomicWordRequest {
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            offset_bytes: 8,
            initial_value: 41,
            domain: AtomicDomain::CpuShared,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicCompareExchangeResult {
    Exchanged { previous: u64 },
    Observed { actual: u64 },
}

#[derive(Debug)]
pub struct AtomicWordLocation {
    region: BootstrapRegion,
    request: AtomicWordRequest,
    value: u64,
    modification_order: Vec<u64>,
}

impl AtomicWordLocation {
    /// Consume an ordinary region into one aligned CPU-shared atomic word.
    ///
    /// # Errors
    ///
    /// Returns a stable atomic diagnostic if the word is out of bounds,
    /// misaligned, belongs to dead storage, or names an unsupported domain.
    pub fn create(
        storage: &mut BootstrapStorageState,
        region: BootstrapRegion,
        request: AtomicWordRequest,
    ) -> Result<Self, SystemsModelError> {
        if request.domain != AtomicDomain::CpuShared {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-DOMAIN",
                "the initial atomic word requires the cpu-shared domain",
            ));
        }
        let absolute_offset = region
            .offset_bytes()
            .checked_add(request.offset_bytes)
            .ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-BOUNDS",
                    "atomic word offset overflows the region extent",
                )
            })?;
        let end = request
            .offset_bytes
            .checked_add(INITIAL_ATOMIC_WORD_BYTES)
            .ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-BOUNDS",
                    "atomic word extent overflows the region extent",
                )
            })?;
        if end > region.byte_count() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-BOUNDS",
                "atomic word must be wholly contained in its ordinary region",
            ));
        }
        if absolute_offset % INITIAL_ATOMIC_WORD_BYTES != 0 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-ALIGNMENT",
                "atomic word must be naturally aligned to the provider word width",
            ));
        }
        storage.atomic_store_word(&region, request.offset_bytes, request.initial_value)?;
        Ok(Self {
            region,
            request,
            value: request.initial_value,
            modification_order: vec![request.initial_value],
        })
    }

    #[must_use]
    pub const fn request(&self) -> AtomicWordRequest {
        self.request
    }

    #[must_use]
    pub const fn value(&self) -> u64 {
        self.value
    }

    #[must_use]
    pub fn modification_order(&self) -> &[u64] {
        &self.modification_order
    }

    /// Perform one indivisible compare/exchange in the location modification
    /// order.
    ///
    /// # Errors
    ///
    /// Returns a stable ordering diagnostic for an invalid failure order.
    pub fn compare_exchange(
        &mut self,
        storage: &mut BootstrapStorageState,
        expected: u64,
        desired: u64,
        _success_order: AtomicOrder,
        failure_order: AtomicOrder,
    ) -> Result<AtomicCompareExchangeResult, SystemsModelError> {
        if !failure_order.admits_failure() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-ORDER",
                "compare/exchange failure order cannot release",
            ));
        }
        let previous = self.value;
        if previous == expected {
            self.value = desired;
            self.modification_order.push(desired);
            storage.atomic_store_word(&self.region, self.request.offset_bytes, desired)?;
            Ok(AtomicCompareExchangeResult::Exchanged { previous })
        } else {
            Ok(AtomicCompareExchangeResult::Observed { actual: previous })
        }
    }

    /// Observe the current value with one admitted load order.
    ///
    /// # Errors
    ///
    /// Returns a stable ordering diagnostic when a load is assigned a release
    /// order or storage no longer retains the location contents.
    pub fn load(
        &self,
        storage: &BootstrapStorageState,
        order: AtomicOrder,
    ) -> Result<u64, SystemsModelError> {
        if !order.admits_load() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-ORDER",
                "atomic load order cannot release",
            ));
        }
        let stored = storage.atomic_load_word(&self.region, self.request.offset_bytes)?;
        if stored != self.value {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ATOMIC-STATE",
                "atomic location state disagrees with its owned storage",
            ));
        }
        Ok(stored)
    }

    #[must_use]
    pub fn end(self) -> BootstrapRegion {
        self.region
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BootstrapStorageDescriptor, BootstrapStoragePlacement, BootstrapStorageRequest};

    fn storage_and_region() -> (BootstrapStorageState, BootstrapRegion) {
        let mut storage = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 4096,
                alignment_bytes: 4096,
            },
            "atomic-test",
        )
        .unwrap();
        let region = storage
            .allocate(BootstrapStorageRequest {
                byte_count: 64,
                alignment_bytes: 8,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            })
            .unwrap();
        (storage, region)
    }

    #[test]
    fn compare_exchange_records_one_modification_and_returns_region() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-ATOMIC-001,
        // TOPAL-SYSTEMS-ORDER-001.
        let (mut storage, region) = storage_and_region();
        let mut atomic =
            AtomicWordLocation::create(&mut storage, region, AtomicWordRequest::initial()).unwrap();
        assert_eq!(
            atomic
                .compare_exchange(
                    &mut storage,
                    41,
                    42,
                    AtomicOrder::AcquireRelease,
                    AtomicOrder::Acquire,
                )
                .unwrap(),
            AtomicCompareExchangeResult::Exchanged { previous: 41 }
        );
        assert_eq!(atomic.load(&storage, AtomicOrder::Acquire).unwrap(), 42);
        assert_eq!(atomic.modification_order(), [41, 42]);
        let region = atomic.end();
        assert_eq!(storage.atomic_load_word(&region, 8).unwrap(), 42);
        storage.release(region).unwrap();
    }

    #[test]
    fn failed_compare_exchange_observes_without_modifying() {
        // TOPAL-SYSTEMS-ATOMIC-001, TOPAL-SYSTEMS-ORDER-001.
        let (mut storage, region) = storage_and_region();
        let mut atomic =
            AtomicWordLocation::create(&mut storage, region, AtomicWordRequest::initial()).unwrap();
        assert_eq!(
            atomic
                .compare_exchange(
                    &mut storage,
                    7,
                    42,
                    AtomicOrder::AcquireRelease,
                    AtomicOrder::Acquire,
                )
                .unwrap(),
            AtomicCompareExchangeResult::Observed { actual: 41 }
        );
        assert_eq!(atomic.modification_order(), [41]);
    }

    #[test]
    fn rejects_out_of_bounds_alignment_and_release_failure_order() {
        // TOPAL-SYSTEMS-ATOMIC-001, TOPAL-SYSTEMS-ORDER-001.
        let (mut storage, region) = storage_and_region();
        assert_eq!(
            AtomicWordLocation::create(
                &mut storage,
                region,
                AtomicWordRequest {
                    offset_bytes: 60,
                    ..AtomicWordRequest::initial()
                },
            )
            .unwrap_err()
            .code,
            "E-SYSTEMS-ATOMIC-BOUNDS"
        );

        let (mut storage, region) = storage_and_region();
        let mut atomic =
            AtomicWordLocation::create(&mut storage, region, AtomicWordRequest::initial()).unwrap();
        assert_eq!(
            atomic
                .compare_exchange(
                    &mut storage,
                    41,
                    42,
                    AtomicOrder::AcquireRelease,
                    AtomicOrder::Release,
                )
                .unwrap_err()
                .code,
            "E-SYSTEMS-ATOMIC-ORDER"
        );
    }
}
