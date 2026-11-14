//! Actual shared inode byte loans; no source execution or physical erasure proof.
//! TOPAL-SYSTEMS-INODE-MODE-ACTION-001.
use super::*;
use std::sync::Arc;

/// Which already validated controller lends this block. Not a numeric grant.
#[derive(Clone, Copy)]
pub(crate) enum InodeController {
    Filesystem,
    Pipe,
}

impl<'a> RegionLibraryInvocation<'a> {
    pub(crate) fn begin_inode(
        field: &'a mut [u8; 32],
        domain: &'a mut ControllerActionState,
        controller: (&str, Arc<()>, InodeController),
        kind: u16,
        inputs: (u64, u64),
        binding: &'a RegionLibraryBinding,
    ) -> Result<(Self, RegionLibraryView), RegionLibraryError> {
        if domain.active || domain.poisoned {
            return Err(RegionLibraryError::Owner);
        }
        if inputs.0 > 0o7777 {
            return Err(RegionLibraryError::Bounds);
        }
        let mode = u64::from_le_bytes(field[..8].try_into().unwrap());
        if mode & !0o7777 != u64::from(kind)
            || !matches!(
                (controller.2, kind),
                (InodeController::Filesystem, 0o100000 | 0o40000)
                    | (InodeController::Pipe, 0o10000)
            )
        {
            return Err(RegionLibraryError::Owner);
        }
        let generation = domain
            .generation
            .checked_add(1)
            .ok_or(RegionLibraryError::IdentityExhausted)?;
        let mut expected = *field;
        expected[..8].copy_from_slice(&(u64::from(kind) | inputs.0).to_le_bytes());
        expected[24..].copy_from_slice(&inputs.1.to_le_bytes());
        let (label, identity, controller_kind) = controller;
        let (owner_class, resource_class) = match controller_kind {
            InodeController::Filesystem => (2, "file-controller"),
            InodeController::Pipe => (3, "pipe-controller"),
        };
        // Complete admission before publishing the invocation generation.
        domain.generation = generation;
        domain.active = true;
        Ok(Self::from_storage(
            Storage::Inode {
                field,
                domain,
                owner: (label.into(), owner_class),
                scope: owner::ControllerOwner(identity),
                resource: ResourceIdentity(QualifiedName(vec![
                    "systems".into(),
                    resource_class.into(),
                    label.into(),
                ])),
                inputs,
                expected,
            },
            binding,
            generation,
            RegionLibraryAccess::ReadWrite,
        ))
    }

    /// Copied admitted facts, without predicting or executing an opaque body.
    pub fn inode_mode_action_inputs(&self) -> Result<(u64, u64), RegionLibraryError> {
        self.active()?;
        match &self.storage {
            Storage::Inode { inputs, .. } => Ok(*inputs),
            _ => Err(RegionLibraryError::Binding),
        }
    }

    pub fn complete_inode_mode_action(
        &mut self,
        result: bool,
        proof: RegionActionReclamation,
    ) -> Result<(), RegionLibraryError> {
        self.active()?;
        if self.frames.len() != 1 {
            return Err(RegionLibraryError::NotRoot);
        }
        let Storage::Inode {
            field,
            domain,
            expected,
            ..
        } = &mut self.storage
        else {
            return Err(RegionLibraryError::Binding);
        };
        let valid = result && proof.complete() && **field == *expected;
        if valid {
            domain.active = false;
        } else {
            domain.poisoned = true;
        }
        self.revoke();
        if valid {
            Ok(())
        } else {
            Err(RegionLibraryError::Invariant)
        }
    }
}
