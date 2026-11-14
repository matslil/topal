//! Actual canonical metadata loans; no compiled source or native authority proof.
//! TOPAL-SYSTEMS-NAMESPACE-ACTION-001.
use super::*;
use crate::{
    RegionActionReclamation, RegionLibraryBinding, RegionLibraryError, RegionLibraryInvocation,
    RegionLibraryView,
};

/// Already copied semantic components or an actual live model reference.
/// None of these names, flags or numeric slots create storage authority.
#[derive(Debug, Clone, Copy)]
pub enum SystemsNamespaceRequest<'a> {
    Link {
        source: &'a [u8],
        destination: &'a [u8],
    },
    LinkByFile {
        source: &'a SystemsVolatileFile,
        destination: &'a [u8],
    },
    Rename {
        source: &'a [u8],
        destination: &'a [u8],
        operation: SystemsFileRename,
    },
}

#[derive(Debug)]
struct Plan {
    operation: u64,
    seconds: u64,
    source_object: usize,
    source_entry: Option<usize>,
    target_object: Option<usize>,
    target_entry: usize,
    source_identity: u64,
    target_identity: u64,
    source_links: u64,
    name: Vec<u8>,
}

fn word(bytes: &[u8; PAGE], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn put(bytes: &mut [u8; PAGE], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

impl Plan {
    fn input(&self) -> [u8; 512] {
        let mut input = [0; 512];
        let fields = [
            self.source_object as u64,
            self.source_entry.map_or(4, |slot| slot as u64),
            self.target_object.map_or(4, |slot| slot as u64),
            self.target_entry as u64,
            self.name.len() as u64,
            self.source_identity,
            self.target_identity,
            self.source_links,
        ];
        for (index, value) in fields.into_iter().enumerate() {
            input[index * 8..index * 8 + 8].copy_from_slice(&value.to_le_bytes());
        }
        input[128..128 + self.name.len()].copy_from_slice(&self.name);
        input
    }
    /// A comparison-only oracle. This snapshot is never published or copied
    /// back into the actual owner, including after opaque source failure.
    fn expected(&self, before: &[u8; PAGE], input: &[u8; 512]) -> Box<[u8; PAGE]> {
        let mut expected = Box::new(*before);
        expected[3072..3584].copy_from_slice(input);
        let object = 256 + self.source_object * 128;
        let target_entry = 64 + self.target_entry * 16;
        match self.operation {
            0 => {
                let name = 1536 + self.target_entry * 256;
                expected[name..name + 256].copy_from_slice(&input[128..384]);
                put(&mut expected, target_entry, self.source_identity);
                put(&mut expected, target_entry + 8, self.name.len() as u64);
                put(&mut expected, object + 56, self.source_links + 1);
                put(&mut expected, object + 64, 0);
            }
            1 => {
                let slot = self.source_entry.unwrap();
                let name = 1536 + slot * 256;
                expected[name..name + 256].copy_from_slice(&input[128..384]);
                put(&mut expected, 72 + slot * 16, self.name.len() as u64);
            }
            2 => {
                let slot = self.source_entry.unwrap();
                expected[64 + slot * 16..80 + slot * 16].fill(0);
                expected[1536 + slot * 256..1536 + (slot + 1) * 256].fill(0);
                put(&mut expected, target_entry, self.source_identity);
                let victim = 256 + self.target_object.unwrap() * 128;
                let remaining = word(before, victim + 56) - 1;
                put(&mut expected, victim + 56, remaining);
                put(&mut expected, victim + 48, self.seconds);
            }
            3 => {
                put(
                    &mut expected,
                    64 + self.source_entry.unwrap() * 16,
                    self.target_identity,
                );
                put(&mut expected, target_entry, self.source_identity);
                put(
                    &mut expected,
                    256 + self.target_object.unwrap() * 128 + 48,
                    self.seconds,
                );
            }
            _ => unreachable!("private admitted namespace operation"),
        }
        put(&mut expected, object + 48, self.seconds);
        put(&mut expected, 48, self.seconds);
        put(&mut expected, 56, self.seconds);
        expected
    }
}

#[derive(Debug)]
pub(crate) struct NamespaceActionStorage<'a> {
    state: &'a mut State,
    expected: Box<[u8; PAGE]>,
    inputs: (u64, u64),
    victim: Option<usize>,
    owner: String,
    identity: Arc<()>,
}

impl SystemsVolatileFilePool {
    /// Checked model admission, not a Linux path/credential adapter. Refusals
    /// are region errors rather than errno; direct personality laws stay in
    /// link/rename. None means an exact successful no-op with no issued action.
    pub fn borrow_namespace_action_after_admission<'a>(
        &'a mut self,
        owner: &str,
        request: SystemsNamespaceRequest<'_>,
        seconds: u64,
        binding: &'a RegionLibraryBinding,
    ) -> Result<Option<(RegionLibraryInvocation<'a>, RegionLibraryView)>, RegionLibraryError> {
        self.owned(owner).map_err(|_| RegionLibraryError::Owner)?;
        let Some(plan) = self.namespace_plan(owner, request, seconds)? else {
            return Ok(None);
        };
        let generation = self
            .state
            .namespace_actions
            .generation
            .checked_add(1)
            .ok_or(RegionLibraryError::IdentityExhausted)?;
        let input = plan.input();
        let expected = plan.expected(&self.state.metadata, &input);
        // All recoverable checks, including generation exhaustion and clean
        // backing validation, precede staging and publication of the loan.
        self.state.metadata[3072..3584].copy_from_slice(&input);
        self.state.namespace_actions.generation = generation;
        self.state.namespace_actions.active = true;
        let storage = NamespaceActionStorage {
            state: &mut self.state,
            expected,
            inputs: (plan.operation, seconds),
            victim: (plan.operation == 2)
                .then_some(plan.target_object)
                .flatten(),
            owner: self.owner.clone(),
            identity: self.identity.clone(),
        };
        Ok(Some(RegionLibraryInvocation::begin_namespace(
            storage, binding, generation,
        )))
    }

    fn namespace_plan(
        &self,
        owner: &str,
        request: SystemsNamespaceRequest<'_>,
        seconds: u64,
    ) -> Result<Option<Plan>, RegionLibraryError> {
        let checked_name = |name: &[u8]| valid_name(name).map_err(|_| RegionLibraryError::Owner);
        let (source_object, source_entry, destination, rename) = match request {
            SystemsNamespaceRequest::Link {
                source,
                destination,
            }
            | SystemsNamespaceRequest::Rename {
                source,
                destination,
                ..
            } => {
                checked_name(source)?;
                let entry = self.named_entry(source).ok_or(RegionLibraryError::Owner)?;
                let object = self.named_object(source).ok_or(RegionLibraryError::Owner)?;
                let rename = if let SystemsNamespaceRequest::Rename { operation, .. } = request {
                    Some(operation)
                } else {
                    None
                };
                (object, Some(entry), destination, rename)
            }
            SystemsNamespaceRequest::LinkByFile {
                source,
                destination,
            } => {
                let description = self
                    .description(owner, source)
                    .map_err(|_| RegionLibraryError::Owner)?;
                (
                    self.state.descriptions[&description].object,
                    None,
                    destination,
                    None,
                )
            }
        };
        checked_name(destination)?;
        let inode = self
            .state
            .object(source_object)
            .ok_or(RegionLibraryError::Owner)?;
        let new = self.named_entry(destination);
        let target_object = self.named_object(destination);
        let target_identity =
            target_object.map_or(0, |index| self.state.object(index).unwrap().identity);
        let (operation, target_entry) = if let Some(rename) = rename {
            if rename == SystemsFileRename::NoReplace && new.is_some()
                || rename == SystemsFileRename::Exchange && new.is_none()
            {
                return Err(RegionLibraryError::Owner);
            }
            if new.is_some() && inode.identity == target_identity {
                return Ok(None);
            }
            (
                if rename == SystemsFileRename::Exchange {
                    3
                } else if new.is_some() {
                    2
                } else {
                    1
                },
                new.unwrap_or(source_entry.unwrap()),
            )
        } else {
            if new.is_some() || inode.link_count == 0 && !inode.may_link_unlinked {
                return Err(RegionLibraryError::Owner);
            }
            (
                0,
                self.state
                    .names()
                    .iter()
                    .position(Option::is_none)
                    .ok_or(RegionLibraryError::Capacity)?,
            )
        };
        Ok(Some(Plan {
            operation,
            seconds,
            source_object,
            source_entry,
            target_object,
            target_entry,
            source_identity: inode.identity,
            target_identity,
            source_links: inode.link_count,
            name: destination.to_vec(),
        }))
    }
}

impl NamespaceActionStorage<'_> {
    pub(crate) fn owner(&self) -> (String, u64) {
        (self.owner.clone(), 4)
    }
    pub(crate) fn identity(&self) -> Arc<()> {
        self.identity.clone()
    }
    pub(crate) fn controller(&self) -> &str {
        &self.owner
    }
    pub(crate) fn inputs(&self) -> (u64, u64) {
        self.inputs
    }
    fn active(&self) -> Result<(), RegionLibraryError> {
        if self.state.namespace_actions.active && !self.state.namespace_actions.poisoned {
            Ok(())
        } else {
            Err(RegionLibraryError::Owner)
        }
    }
    pub(crate) fn load(&self, offset: u64) -> Result<u8, RegionLibraryError> {
        self.active()?;
        let offset = usize::try_from(offset).map_err(|_| RegionLibraryError::Bounds)?;
        self.state
            .metadata
            .get(offset)
            .copied()
            .ok_or(RegionLibraryError::Bounds)
    }
    pub(crate) fn store(&mut self, offset: u64, value: u8) -> Result<(), RegionLibraryError> {
        self.active()?;
        let offset = usize::try_from(offset).map_err(|_| RegionLibraryError::Bounds)?;
        *self
            .state
            .metadata
            .get_mut(offset)
            .ok_or(RegionLibraryError::Bounds)? = value;
        Ok(())
    }
    pub(crate) fn finish(
        &mut self,
        result: bool,
        proof: RegionActionReclamation,
    ) -> Result<(), RegionLibraryError> {
        let valid = result && proof.complete() && self.state.metadata == *self.expected;
        // Staging is temporary even on fatal failure. Persistent admitted stores
        // remain; the expected snapshot is never a rollback or publication map.
        self.state.metadata[3072..3584].fill(0);
        if !valid {
            self.state.namespace_actions.poisoned = true;
            return Err(RegionLibraryError::Invariant);
        }
        self.state.namespace_actions.active = false;
        if let Some(victim) = self.victim {
            self.state.reclaim_unlinked(victim);
        }
        Ok(())
    }
    pub(crate) fn abandon(&mut self) {
        if self.state.namespace_actions.active {
            self.state.metadata[3072..3584].fill(0);
            self.state.namespace_actions.poisoned = true;
        }
    }
}

#[cfg(test)]
#[path = "systems_namespace_action_tests.rs"]
mod tests;
