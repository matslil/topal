//! Actual model backing, never native authority or an executed source body.
use super::*;
use std::sync::Arc;

/// Actual model pool identity; equal human-readable labels create no grant.
#[derive(Clone, Debug)]
pub(super) struct ControllerOwner(pub(super) Arc<()>);
impl PartialEq for ControllerOwner {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ControllerOwner {}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct ControllerActionState {
    pub(crate) generation: u64,
    pub(crate) active: bool,
    pub(crate) poisoned: bool,
}

/// Explicit model obligations. These booleans are not physical erasure evidence.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RegionActionReclamation {
    pub arena_erased: bool,
    pub control_erased: bool,
    pub retired_stack_erased: bool,
    pub bindings_revoked: bool,
    pub caller_restored: bool,
}
impl RegionActionReclamation {
    pub(crate) fn complete(self) -> bool {
        self.arena_erased
            && self.control_erased
            && self.retired_stack_erased
            && self.bindings_revoked
            && self.caller_restored
    }
}

#[derive(Debug)]
pub(super) enum Storage<'a> {
    Bootstrap {
        storage: &'a mut BootstrapStorageState,
        region: &'a BootstrapRegion,
    },
    Position {
        field: &'a mut [u8; 8],
        domain: &'a mut ControllerActionState,
        owner: (String, u64),
        scope: ControllerOwner,
        resource: ResourceIdentity,
        inputs: (u64, u64),
        expected: u64,
    },
    PagePool(crate::systems_volatile_files::page_actions::PageActionStorage<'a>),
    Inode {
        field: &'a mut [u8; 32],
        domain: &'a mut ControllerActionState,
        owner: (String, u64),
        scope: ControllerOwner,
        resource: ResourceIdentity,
        inputs: (u64, u64),
        expected: [u8; 32],
    },
}
impl Storage<'_> {
    pub(super) fn owner(&self) -> (String, u64) {
        match self {
            Self::Bootstrap { region, .. } => region.library_owner_identity(),
            Self::Position { owner, .. } => owner.clone(),
            Self::Inode { owner, .. } => owner.clone(),
            Self::PagePool(storage) => storage.owner(),
        }
    }
    pub(super) fn scope(&self) -> Option<ControllerOwner> {
        match self {
            Self::Bootstrap { .. } => None,
            Self::Position { scope, .. } => Some(scope.clone()),
            Self::Inode { scope, .. } => Some(scope.clone()),
            Self::PagePool(storage) => Some(ControllerOwner(storage.identity())),
        }
    }
    pub(super) fn count(&self) -> u64 {
        match self {
            Self::Bootstrap { region, .. } => region.byte_count(),
            Self::Position { .. } => 8,
            Self::PagePool(_) => 8,
            Self::Inode { .. } => 32,
        }
    }
    pub(super) fn resource(&self) -> ResourceIdentity {
        match self {
            Self::Bootstrap { region, .. } => {
                let (execution, allocation) = region.library_owner_identity();
                ResourceIdentity(QualifiedName(vec![
                    "systems".into(),
                    "borrowed-region".into(),
                    execution,
                    allocation.to_string(),
                ]))
            }
            Self::Position { resource, .. } => resource.clone(),
            Self::Inode { resource, .. } => resource.clone(),
            Self::PagePool(storage) => ResourceIdentity(QualifiedName(vec![
                "systems".into(),
                "file-controller".into(),
                storage.controller().into(),
            ])),
        }
    }
    pub(super) fn load(&mut self, offset: u64) -> Result<u8, RegionLibraryError> {
        match self {
            Self::Bootstrap { storage, region } => storage
                .load_byte(region, offset)
                .map_err(|_| RegionLibraryError::Owner),
            Self::Position { field, domain, .. } => {
                if !domain.active || domain.poisoned {
                    return Err(RegionLibraryError::Owner);
                }
                field
                    .get(offset as usize)
                    .copied()
                    .ok_or(RegionLibraryError::Bounds)
            }
            Self::PagePool(storage) => storage.load(offset),
            Self::Inode { field, domain, .. } => {
                if !domain.active || domain.poisoned {
                    return Err(RegionLibraryError::Owner);
                }
                let offset = usize::try_from(offset).map_err(|_| RegionLibraryError::Bounds)?;
                field.get(offset).copied().ok_or(RegionLibraryError::Bounds)
            }
        }
    }
    pub(super) fn store(&mut self, offset: u64, value: u8) -> Result<(), RegionLibraryError> {
        match self {
            Self::Bootstrap { storage, region } => storage
                .store_byte(region, offset, value)
                .map_err(|_| RegionLibraryError::Owner),
            Self::Position { field, domain, .. } => {
                if !domain.active || domain.poisoned {
                    return Err(RegionLibraryError::Owner);
                }
                *field
                    .get_mut(offset as usize)
                    .ok_or(RegionLibraryError::Bounds)? = value;
                Ok(())
            }
            Self::PagePool(storage) => storage.store(offset, value),
            Self::Inode { field, domain, .. } => {
                if !domain.active || domain.poisoned {
                    return Err(RegionLibraryError::Owner);
                }
                let offset = usize::try_from(offset).map_err(|_| RegionLibraryError::Bounds)?;
                *field.get_mut(offset).ok_or(RegionLibraryError::Bounds)? = value;
                Ok(())
            }
        }
    }
    pub(super) fn finish_position(
        &mut self,
        result: bool,
        proof: RegionActionReclamation,
    ) -> Result<(), RegionLibraryError> {
        let Self::Position {
            field,
            domain,
            expected,
            ..
        } = self
        else {
            return Err(RegionLibraryError::Binding);
        };
        if !proof.complete() || !result || u64::from_le_bytes(**field) != *expected {
            domain.poisoned = true;
            return Err(RegionLibraryError::Invariant);
        }
        domain.active = false;
        Ok(())
    }
    pub(super) fn abandon(&mut self) {
        if let Self::PagePool(storage) = self {
            storage.abandon();
        }
        if let Self::Position { domain, .. } = self {
            if domain.active {
                domain.poisoned = true;
            }
        }
        if let Self::Inode { domain, .. } = self {
            if domain.active {
                domain.poisoned = true;
            }
        }
    }
    pub(super) fn is_position(&self) -> bool {
        matches!(self, Self::Position { .. })
    }
    pub(super) fn is_controller(&self) -> bool {
        matches!(
            self,
            Self::Position { .. } | Self::PagePool(_) | Self::Inode { .. }
        )
    }
}
