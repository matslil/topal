//! Independent sparse-file laws. No modeled number or buffer is native authority.
//! TOPAL-SYSTEMS-VOLATILE-FILE-{OWNERSHIP,TRANSFER,PERSONALITY}-001.
use crate::{SystemsInodeMetadata, SystemsModelError};
use std::collections::BTreeMap;
use std::sync::Arc;
#[path = "systems_named_files.rs"]
mod named;
pub use named::*;
#[path = "systems_namespace_transactions.rs"]
mod namespace_transactions;
pub use namespace_transactions::SystemsFileRename;
#[path = "systems_inode_file_modes.rs"]
mod inode_modes;
#[path = "systems_positional_files.rs"]
mod positional;
#[path = "systems_file_region_actions.rs"]
mod region_actions;
pub use region_actions::SystemsPositionTransfer;
#[path = "systems_ram_page_actions.rs"]
pub(crate) mod page_actions;
pub use page_actions::SystemsRamPageAction;
#[path = "systems_namespace_region_actions.rs"]
pub(crate) mod namespace_actions;
pub use namespace_actions::SystemsNamespaceRequest;

#[path = "systems_file_metadata_backing.rs"]
mod metadata_backing;

const PAGE: usize = 4096;
const PAGES: usize = 8;
const OBJECTS: usize = 4;
const NAMES: usize = 4;
const QUANTUM: usize = 256;
const MAX_OFFSET: u64 = i64::MAX as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemsFileRights {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Copy)]
pub enum SystemsFileSeek {
    Set,
    Current,
    End,
    Data,
    Hole,
}

#[derive(Debug, Clone, Copy)]
pub struct SystemsFileUserAdmission {
    pub numerical_range: bool,
    pub complete_prefix: bool,
}

/// An explicit model reference; its private pool/ticket cannot be forged by a PID.
#[derive(Debug)]
pub struct SystemsVolatileFile {
    pool: Arc<()>,
    ticket: u64,
}

/// Private process state: copying at fork does not share future mask changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemsFileUmask(u16);
impl Default for SystemsFileUmask {
    fn default() -> Self {
        Self(0o022)
    }
}
impl SystemsFileUmask {
    pub fn replace(&mut self, native: u32) -> u16 {
        let old = self.0;
        self.0 = native as u16 & 0o777;
        old
    }
    pub fn apply(&self, native_mode: u32) -> u16 {
        (native_mode as u16 & 0o7777) & !self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemsVolatileFileMetadata {
    pub identity: u64,
    pub links: u64,
    pub size: u64,
    pub position: u64,
    pub allocated_bytes: usize,
    pub mode: u16,
    pub accessed_seconds: u64,
    pub modified_seconds: u64,
    pub changed_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Object {
    identity: u64,
    may_link_unlinked: bool,
    link_count: u64,
    size: u64,
    inode: SystemsInodeMetadata,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct NameEntry {
    identity: u64,
    name: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Description {
    object: usize,
    rights: SystemsFileRights,
    append: bool,
    position: [u8; 8],
    references: usize,
}
impl Description {
    fn position(&self) -> u64 {
        u64::from_le_bytes(self.position)
    }
    fn set_position(&mut self, value: u64) {
        self.position = value.to_le_bytes();
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    bytes: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    metadata: [u8; PAGE],
    pages: Vec<Page>,
    descriptions: BTreeMap<u64, Description>,
    references: BTreeMap<u64, u64>,
    next_description: u64,
    next_ticket: u64,
    live: bool,
    position_actions: crate::systems_region_library::ControllerActionState,
    page_actions: crate::systems_region_library::ControllerActionState,
    inode_actions: crate::systems_region_library::ControllerActionState,
    namespace_actions: crate::systems_region_library::ControllerActionState,
}

#[derive(Debug)]
pub struct SystemsVolatileFilePool {
    owner: String,
    identity: Arc<()>,
    state: State,
}
fn error(message: &str) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-VOLATILE-FILE", message)
}
fn verify_range(position: u64, count: u64) -> Result<(), SystemsModelError> {
    if count > MAX_OFFSET || position > MAX_OFFSET - count {
        return Err(error("EINVAL"));
    }
    Ok(())
}
impl SystemsVolatileFilePool {
    pub fn new(owner: &str) -> Result<Self, SystemsModelError> {
        if owner.is_empty() {
            return Err(error("missing filesystem owner"));
        }
        Ok(Self {
            owner: owner.into(),
            identity: Arc::new(()),
            state: {
                let mut state = State {
                    metadata: [0; PAGE],
                    pages: (0..PAGES)
                        .map(|_| Page {
                            bytes: vec![0; PAGE],
                        })
                        .collect(),
                    descriptions: BTreeMap::new(),
                    references: BTreeMap::new(),
                    next_description: 1,
                    next_ticket: 1,
                    live: true,
                    position_actions: Default::default(),
                    page_actions: Default::default(),
                    inode_actions: Default::default(),
                    namespace_actions: Default::default(),
                };
                state.set_next_identity(7);
                state.metadata[32..64]
                    .copy_from_slice(&SystemsInodeMetadata::new(0o40000, 0o1777, 0).bytes);
                state
            },
        })
    }
    fn owned(&self, owner: &str) -> Result<(), SystemsModelError> {
        if !self.state.live
            || self.owner != owner
            || self.state.position_actions.active
            || self.state.position_actions.poisoned
            || self.state.page_actions.active
            || self.state.page_actions.poisoned
            || self.state.inode_actions.active
            || self.state.inode_actions.poisoned
            || self.state.namespace_actions.active
            || self.state.namespace_actions.poisoned
        {
            return Err(error("wrong or released filesystem owner"));
        }
        if !self.allocation_consistent() {
            return Err(error("inconsistent page allocation owner"));
        }
        if !self.namespace_consistent() {
            return Err(error("inconsistent namespace owner"));
        }
        if !self.descriptions_consistent() {
            return Err(error("inconsistent description owner"));
        }
        Ok(())
    }

    fn descriptions_consistent(&self) -> bool {
        self.state.references.len() <= 32
            && self.state.next_description != 0
            && self.state.next_ticket != 0
            && self.state.references.iter().all(|(&ticket, description)| {
                ticket != 0 && self.state.descriptions.contains_key(description)
            })
            && self
                .state
                .descriptions
                .iter()
                .all(|(&identity, description)| {
                    identity != 0
                        && description.object < OBJECTS
                        && self.state.object(description.object).is_some()
                        && description.position() <= MAX_OFFSET
                        && description.references != 0
                        && self
                            .state
                            .references
                            .values()
                            .filter(|&&value| value == identity)
                            .count()
                            == description.references
                })
    }

    fn namespace_consistent(&self) -> bool {
        let mut names = std::collections::BTreeSet::new();
        let mut identities = std::collections::BTreeSet::new();
        self.state.metadata_layout_consistent()
            && self.state.objects().iter().flatten().all(|object| {
                object.identity >= 7
                    && object.identity < self.state.next_identity()
                    && identities.insert(object.identity)
                    && object.link_count == self.links(object.identity)
            })
            && self.state.names().len() == NAMES
            && self.state.names().iter().flatten().all(|entry| {
                valid_name(&entry.name).is_ok()
                    && names.insert(entry.name.clone())
                    && self.state.objects().iter().flatten().any(|object| {
                        object.identity == entry.identity && !object.may_link_unlinked
                    })
            })
    }

    fn links(&self, identity: u64) -> u64 {
        self.state
            .names()
            .iter()
            .flatten()
            .filter(|entry| entry.identity == identity)
            .count() as u64
    }

    fn allocation_consistent(&self) -> bool {
        let mut associations = std::collections::BTreeSet::new();
        self.state.pages.len() == PAGES
            && self.state.page_records().enumerate().all(|(slot, page)| {
                page.bytes.len() == PAGE
                    && match (self.state.page_allocation()[slot], page.owner) {
                        (0, None) => page.index == 0 && page.bytes.iter().all(|byte| *byte == 0),
                        (1, Some(identity)) => {
                            page.index <= MAX_OFFSET / PAGE as u64
                                && self
                                    .state
                                    .objects()
                                    .iter()
                                    .flatten()
                                    .any(|o| o.identity == identity)
                                && associations.insert((identity, page.index))
                        }
                        _ => false,
                    }
            })
    }
    fn description(
        &self,
        owner: &str,
        file: &SystemsVolatileFile,
    ) -> Result<u64, SystemsModelError> {
        self.owned(owner)?;
        if !Arc::ptr_eq(&self.identity, &file.pool) {
            return Err(error("wrong filesystem reference"));
        }
        self.state
            .references
            .get(&file.ticket)
            .copied()
            .ok_or_else(|| error("stale file reference"))
    }
    fn ticket(&self) -> Result<(u64, u64), SystemsModelError> {
        if self.state.references.len() >= 32 {
            return Err(error("EMFILE"));
        }
        let ticket = self.state.next_ticket;
        Ok((
            ticket,
            ticket
                .checked_add(1)
                .ok_or_else(|| error("reference identity exhausted"))?,
        ))
    }
    pub fn create(
        &mut self,
        owner: &str,
        rights: SystemsFileRights,
        mode: u32,
        umask: SystemsFileUmask,
        append: bool,
        seconds: u64,
    ) -> Result<SystemsVolatileFile, SystemsModelError> {
        self.owned(owner)?;
        if rights == SystemsFileRights::Read {
            return Err(error("EINVAL: temporary creation requires write access"));
        }
        let (ticket, next_ticket) = self.ticket()?;
        let index = self
            .state
            .objects()
            .iter()
            .position(Option::is_none)
            .ok_or_else(|| error("ENOSPC"))?;
        let identity = self.state.next_identity();
        let next_identity = identity
            .checked_add(1)
            .ok_or_else(|| error("object identity exhausted"))?;
        let description = self.state.next_description;
        let next_description = description
            .checked_add(1)
            .ok_or_else(|| error("description identity exhausted"))?;
        self.state.set_object(
            index,
            Some(Object {
                identity,
                may_link_unlinked: true,
                link_count: 0,
                size: 0,
                inode: SystemsInodeMetadata::new(0o100000, umask.apply(mode), seconds),
            }),
        );
        self.state.descriptions.insert(
            description,
            Description {
                object: index,
                rights,
                append,
                position: [0; 8],
                references: 1,
            },
        );
        self.state.references.insert(ticket, description);
        self.state.next_description = next_description;
        self.state.set_next_identity(next_identity);
        self.state.next_ticket = next_ticket;
        Ok(SystemsVolatileFile {
            pool: self.identity.clone(),
            ticket,
        })
    }
    /// Duplication and inherited references retain one shared description.
    pub fn alias(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
    ) -> Result<SystemsVolatileFile, SystemsModelError> {
        let identity = self.description(owner, file)?;
        let (ticket, next) = self.ticket()?;
        self.state.references.insert(ticket, identity);
        self.state
            .descriptions
            .get_mut(&identity)
            .unwrap()
            .references += 1;
        self.state.next_ticket = next;
        Ok(SystemsVolatileFile {
            pool: self.identity.clone(),
            ticket,
        })
    }
    pub fn metadata(
        &self,
        owner: &str,
        file: &SystemsVolatileFile,
    ) -> Result<SystemsVolatileFileMetadata, SystemsModelError> {
        let identity = self.description(owner, file)?;
        let d = &self.state.descriptions[&identity];
        Ok(self.object_metadata(d.object, d.position()))
    }
    fn object_metadata(&self, object: usize, position: u64) -> SystemsVolatileFileMetadata {
        let o = self.state.object(object).unwrap();
        SystemsVolatileFileMetadata {
            identity: o.identity,
            links: self.links(o.identity),
            size: o.size,
            position,
            mode: o.inode.permissions(),
            allocated_bytes: self
                .state
                .page_records()
                .filter(|p| p.owner == Some(o.identity))
                .count()
                * PAGE,
            accessed_seconds: o.inode.accessed_seconds(),
            modified_seconds: o.inode.modified_seconds(),
            changed_seconds: o.inode.changed_seconds(),
        }
    }
    pub fn read(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        count: u64,
        admitted: SystemsFileUserAdmission,
    ) -> Result<Vec<u8>, SystemsModelError> {
        self.read_position(owner, file, count, admitted, None)
    }
    pub(super) fn read_position(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        count: u64,
        admitted: SystemsFileUserAdmission,
        supplied: Option<u64>,
    ) -> Result<Vec<u8>, SystemsModelError> {
        let identity = self.description(owner, file)?;
        let d = &self.state.descriptions[&identity];
        if d.rights == SystemsFileRights::Write {
            return Err(error("EBADF"));
        }
        if !admitted.numerical_range {
            return Err(error("EFAULT"));
        }
        let position = supplied.unwrap_or(d.position());
        verify_range(position, count)?;
        let o = self.state.object(d.object).unwrap();
        if count == 0 || position >= o.size {
            return Ok(vec![]);
        }
        let offset = (position % PAGE as u64) as usize;
        let length = count
            .min(o.size - position)
            .min(QUANTUM as u64)
            .min((PAGE - offset) as u64) as usize;
        if !admitted.complete_prefix {
            return Err(error("EFAULT"));
        }
        let bytes = match self
            .state
            .page_records()
            .find(|p| p.owner == Some(o.identity) && p.index == position / PAGE as u64)
        {
            Some(p) => p.bytes[offset..offset + length].to_vec(),
            None => vec![0; length],
        };
        if supplied.is_none() {
            self.state
                .descriptions
                .get_mut(&identity)
                .unwrap()
                .set_position(position + length as u64);
        }
        Ok(bytes)
    }
    pub fn write(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        bytes: &[u8],
        admitted: SystemsFileUserAdmission,
        seconds: u64,
    ) -> Result<usize, SystemsModelError> {
        self.write_position(owner, file, bytes, admitted, seconds, None)
    }
    pub(super) fn write_position(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        bytes: &[u8],
        admitted: SystemsFileUserAdmission,
        seconds: u64,
        supplied: Option<u64>,
    ) -> Result<usize, SystemsModelError> {
        let identity = self.description(owner, file)?;
        let d = &self.state.descriptions[&identity];
        if d.rights == SystemsFileRights::Read {
            return Err(error("EBADF"));
        }
        if !admitted.numerical_range {
            return Err(error("EFAULT"));
        }
        let supplied_position = supplied.unwrap_or(d.position());
        verify_range(supplied_position, bytes.len() as u64)?;
        if bytes.is_empty() {
            return Ok(0);
        }
        let object = d.object;
        let o = self.state.object(object).unwrap();
        let object_identity = o.identity;
        let position = if d.append { o.size } else { supplied_position };
        if position == MAX_OFFSET {
            return Err(error("EFBIG"));
        }
        let offset = (position % PAGE as u64) as usize;
        let length = ((bytes.len().min(QUANTUM).min(PAGE - offset)) as u64)
            .min(MAX_OFFSET - position) as usize;
        if !admitted.complete_prefix {
            return Err(error("EFAULT"));
        }
        let page_index = position / PAGE as u64;
        let slot = self
            .state
            .page_records()
            .position(|p| p.owner == Some(object_identity) && p.index == page_index)
            .or_else(|| {
                self.state
                    .page_allocation()
                    .iter()
                    .position(|bit| *bit == 0)
            })
            .ok_or_else(|| error("ENOSPC"))?;
        if self.state.page_owner(slot).is_none() {
            debug_assert!(
                self.state.page_index(slot) == 0
                    && self.state.pages[slot].bytes.iter().all(|&b| b == 0)
            );
            self.state.associate_page(slot, object_identity, page_index);
            self.state.page_allocation_mut()[slot] = 1;
        }
        self.state.pages[slot].bytes[offset..offset + length].copy_from_slice(&bytes[..length]);
        let end = position + length as u64;
        self.state.update_object(object, |o| {
            o.size = o.size.max(end);
            o.inode.modified(seconds);
        });
        if supplied.is_none() {
            self.state
                .descriptions
                .get_mut(&identity)
                .unwrap()
                .set_position(end);
        }
        Ok(length)
    }
    pub fn seek(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        offset: i64,
        whence: SystemsFileSeek,
    ) -> Result<u64, SystemsModelError> {
        let identity = self.description(owner, file)?;
        let d = &self.state.descriptions[&identity];
        let object = self.state.object(d.object).unwrap();
        let size = object.size;
        let object_identity = object.identity;
        let position = match whence {
            SystemsFileSeek::Set | SystemsFileSeek::Current | SystemsFileSeek::End => {
                let base = match whence {
                    SystemsFileSeek::Current => d.position(),
                    SystemsFileSeek::End => size,
                    _ => 0,
                };
                let result = base as i128 + offset as i128;
                if !(0..=MAX_OFFSET as i128).contains(&result) {
                    return Err(error("EINVAL"));
                }
                result as u64
            }
            SystemsFileSeek::Data | SystemsFileSeek::Hole => {
                if offset < 0 || offset as u64 >= size {
                    return Err(error("ENXIO"));
                }
                let position = offset as u64;
                let index = position / PAGE as u64;
                let has = |index| {
                    self.state
                        .page_records()
                        .any(|p| p.owner == Some(object_identity) && p.index == index)
                };
                match whence {
                    SystemsFileSeek::Data if has(index) => position,
                    SystemsFileSeek::Data => self
                        .state
                        .page_records()
                        .filter(|p| p.owner == Some(object_identity) && p.index > index)
                        .map(|p| p.index * PAGE as u64)
                        .filter(|&n| n < size)
                        .min()
                        .ok_or_else(|| error("ENXIO"))?,
                    _ if !has(index) => position,
                    _ => {
                        let mut next = index + 1;
                        while has(next) {
                            next += 1;
                        }
                        (next * PAGE as u64).min(size)
                    }
                }
            }
        };
        self.state
            .descriptions
            .get_mut(&identity)
            .unwrap()
            .set_position(position);
        Ok(position)
    }
    pub fn truncate(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        length: i64,
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        let identity = self.description(owner, file)?;
        let d = &self.state.descriptions[&identity];
        if d.rights == SystemsFileRights::Read || length < 0 {
            return Err(error("EINVAL"));
        }
        let object = d.object;
        let o = self.state.object(object).unwrap();
        let old_size = o.size;
        let object_identity = o.identity;
        let length = length as u64;
        if length == old_size {
            return Ok(());
        }
        if length < old_size {
            for slot in 0..PAGES {
                if self.state.page_owner(slot) != Some(object_identity) {
                    continue;
                }
                let start = self.state.page_index(slot) * PAGE as u64;
                if start >= length {
                    self.state.erase_page(slot);
                } else if length - start < PAGE as u64 {
                    self.state.pages[slot].bytes[(length - start) as usize..].fill(0);
                }
            }
        }
        self.state.update_object(object, |o| {
            o.size = length;
            o.inode.modified(seconds);
        });
        Ok(())
    }
    pub fn close(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
    ) -> Result<(), SystemsModelError> {
        let identity = self.description(owner, file)?;
        self.state.references.remove(&file.ticket);
        let d = self.state.descriptions.get_mut(&identity).unwrap();
        d.references -= 1;
        if d.references == 0 {
            let object = d.object;
            self.state.descriptions.remove(&identity);
            self.reclaim_unlinked(object);
        }
        Ok(())
    }
    pub fn terminate(&mut self, owner: &str) -> Result<(), SystemsModelError> {
        // Terminal erasure never resumes a poisoned controller. A live Rust
        // field loan still exclusively borrows this pool and prevents entry.
        if !self.state.live || self.owner != owner {
            return Err(error("wrong or released filesystem owner"));
        }
        for page in &mut self.state.pages {
            page.bytes.fill(0);
        }
        self.state.metadata.fill(0);
        self.state.descriptions.clear();
        self.state.references.clear();
        self.state.next_description = 0;
        self.state.next_ticket = 0;
        self.state.position_actions = Default::default();
        self.state.page_actions = Default::default();
        self.state.inode_actions = Default::default();
        self.state.namespace_actions = Default::default();
        self.state.live = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const OK: SystemsFileUserAdmission = SystemsFileUserAdmission {
        numerical_range: true,
        complete_prefix: true,
    };
    const BAD_PREFIX: SystemsFileUserAdmission = SystemsFileUserAdmission {
        numerical_range: true,
        complete_prefix: false,
    };
    const BAD_RANGE: SystemsFileUserAdmission = SystemsFileUserAdmission {
        numerical_range: false,
        complete_prefix: true,
    };
    fn pool() -> SystemsVolatileFilePool {
        SystemsVolatileFilePool::new("owner").unwrap()
    }
    fn create(p: &mut SystemsVolatileFilePool) -> SystemsVolatileFile {
        p.create(
            "owner",
            SystemsFileRights::ReadWrite,
            0o666,
            SystemsFileUmask::default(),
            false,
            100,
        )
        .unwrap()
    }
    fn metadata(
        p: &SystemsVolatileFilePool,
        f: &SystemsVolatileFile,
    ) -> SystemsVolatileFileMetadata {
        p.metadata("owner", f).unwrap()
    }
    fn seek(p: &mut SystemsVolatileFilePool, f: &SystemsVolatileFile, position: i64) {
        assert_eq!(
            p.seek("owner", f, position, SystemsFileSeek::Set).unwrap(),
            position as u64
        );
    }
    fn write_all(p: &mut SystemsVolatileFilePool, f: &SystemsVolatileFile, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let n = p.write("owner", f, bytes, OK, 101).unwrap();
            assert!(n > 0);
            bytes = &bytes[n..];
        }
    }
    fn read_all(p: &mut SystemsVolatileFilePool, f: &SystemsVolatileFile) -> Vec<u8> {
        let mut bytes = vec![];
        loop {
            let next = p.read("owner", f, QUANTUM as u64, OK).unwrap();
            if next.is_empty() {
                return bytes;
            }
            bytes.extend(next);
        }
    }
    #[test]
    fn sparse_large_offsets_are_independent_of_capacity_and_seek_uses_allocation_extents() {
        // TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001
        let mut p = pool();
        let f = create(&mut p);
        let large = (1_i64 << 48) + 13;
        seek(&mut p, &f, large);
        assert_eq!(p.write("owner", &f, b"abc", OK, 101).unwrap(), 3);
        assert_eq!(metadata(&p, &f).size, large as u64 + 3);
        assert_eq!(metadata(&p, &f).allocated_bytes, PAGE);
        seek(&mut p, &f, large - 5);
        assert_eq!(p.read("owner", &f, 8, OK).unwrap(), b"\0\0\0\0\0abc");
        assert_eq!(p.seek("owner", &f, 0, SystemsFileSeek::Hole).unwrap(), 0);
        assert_eq!(
            p.seek("owner", &f, 0, SystemsFileSeek::Data).unwrap(),
            1_u64 << 48
        );
        assert_eq!(
            p.seek("owner", &f, large, SystemsFileSeek::Data).unwrap(),
            large as u64
        );
        assert_eq!(
            p.seek("owner", &f, large, SystemsFileSeek::Hole).unwrap(),
            large as u64 + 3
        );
        let before = p.state.clone();
        assert!(
            p.seek("owner", &f, large + 3, SystemsFileSeek::Hole)
                .unwrap_err()
                .to_string()
                .contains("ENXIO")
        );
        assert_eq!(p.state, before);
        // A resident all-zero page remains a data extent.
        seek(&mut p, &f, 0);
        p.write("owner", &f, &[0], OK, 102).unwrap();
        assert_eq!(p.seek("owner", &f, 0, SystemsFileSeek::Data).unwrap(), 0);
        assert_eq!(
            p.seek("owner", &f, 0, SystemsFileSeek::Hole).unwrap(),
            PAGE as u64
        );
    }
    #[test]
    fn actual_prefixes_page_boundaries_and_aliases_preserve_complete_bytes() {
        // TOPAL-SYSTEMS-VOLATILE-FILE-{OWNERSHIP,TRANSFER}-001
        let mut p = pool();
        let f = create(&mut p);
        let bytes: Vec<_> = (0..8193).map(|i| (i % 251) as u8).collect();
        write_all(&mut p, &f, &bytes);
        let inherited = p.alias("owner", &f).unwrap();
        seek(&mut p, &inherited, 4093);
        assert_eq!(p.read("owner", &f, 1000, OK).unwrap(), bytes[4093..4096]);
        assert_eq!(metadata(&p, &inherited).position, 4096);
        assert_eq!(
            p.read("owner", &inherited, 1000, OK).unwrap(),
            bytes[4096..4352]
        );
        seek(&mut p, &f, 0);
        assert_eq!(read_all(&mut p, &inherited), bytes);
        assert_eq!(metadata(&p, &f).allocated_bytes, 3 * PAGE);
        // noatime preserves the creation timestamp despite actual reads.
        assert_eq!(metadata(&p, &f).accessed_seconds, 100);
        p.close("owner", &f).unwrap();
        assert!(p.metadata("owner", &f).is_err());
        assert_eq!(metadata(&p, &inherited).size, 8193);
        p.close("owner", &inherited).unwrap();
        assert!(
            p.state
                .page_records()
                .all(|page| page.owner.is_none() && page.bytes.iter().all(|&b| b == 0))
        );
    }
    #[test]
    fn shrink_and_regrowth_erase_removed_suffixes_without_changing_shared_position() {
        // TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001
        let mut p = pool();
        let f = create(&mut p);
        write_all(&mut p, &f, &[0xa7; 8193]);
        let alias = p.alias("owner", &f).unwrap();
        p.truncate("owner", &alias, 17, 105).unwrap();
        assert_eq!(metadata(&p, &f).position, 8193);
        assert_eq!(metadata(&p, &f).allocated_bytes, PAGE);
        p.truncate("owner", &f, 8193, 106).unwrap();
        seek(&mut p, &alias, 0);
        let mut expected = vec![0; 8193];
        expected[..17].fill(0xa7);
        assert_eq!(read_all(&mut p, &f), expected);
        assert_eq!(metadata(&p, &f).allocated_bytes, PAGE);
        let before = p.state.clone();
        p.truncate("owner", &f, 8193, 999).unwrap();
        assert_eq!(p.state, before);
        p.truncate("owner", &f, 0, 107).unwrap();
        assert_eq!(metadata(&p, &alias).position, 8193);
        assert_eq!(metadata(&p, &alias).allocated_bytes, 0);
        assert!(
            p.state
                .page_records()
                .all(|page| page.bytes.iter().all(|&b| b == 0))
        );
    }
    #[test]
    fn shared_pool_exhaustion_allows_overwrites_and_reclaims_only_after_last_reference() {
        // TOPAL-SYSTEMS-VOLATILE-FILE-OWNERSHIP-001
        let mut p = pool();
        let mut files = vec![];
        for _ in 0..OBJECTS {
            files.push(create(&mut p));
        }
        let before = p.state.clone();
        assert!(
            p.create(
                "owner",
                SystemsFileRights::ReadWrite,
                0o600,
                SystemsFileUmask::default(),
                false,
                200
            )
            .unwrap_err()
            .to_string()
            .contains("ENOSPC")
        );
        assert_eq!(p.state, before);
        for f in &files {
            p.write("owner", f, b"first", OK, 201).unwrap();
            seek(&mut p, f, PAGE as i64);
            p.write("owner", f, b"second", OK, 202).unwrap();
        }
        seek(&mut p, &files[0], 2 * PAGE as i64);
        let before = p.state.clone();
        assert!(
            p.write("owner", &files[0], b"full", OK, 203)
                .unwrap_err()
                .to_string()
                .contains("ENOSPC")
        );
        assert_eq!(p.state, before);
        seek(&mut p, &files[0], 0);
        assert_eq!(
            p.write("owner", &files[0], b"overwritten", OK, 204)
                .unwrap(),
            11
        );
        let old_identity = metadata(&p, &files[1]).identity;
        let alias = p.alias("owner", &files[1]).unwrap();
        p.close("owner", &files[1]).unwrap();
        assert!(p.state.page_records().all(|page| page.owner.is_some()));
        p.close("owner", &alias).unwrap();
        assert_eq!(
            p.state
                .page_records()
                .filter(|page| page.owner.is_none())
                .count(),
            2
        );
        let replacement = create(&mut p);
        assert!(metadata(&p, &replacement).identity > old_identity);
        p.truncate("owner", &replacement, 2 * PAGE as i64, 205)
            .unwrap();
        assert_eq!(read_all(&mut p, &replacement), vec![0; 2 * PAGE]);
        assert_eq!(metadata(&p, &replacement).allocated_bytes, 0);
    }
    #[test]
    fn rejected_copy_range_offset_and_length_preserve_the_complete_store() {
        // TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001
        let mut p = pool();
        let f = create(&mut p);
        assert!(p.read("owner", &f, 0, BAD_RANGE).is_err());
        assert!(p.write("owner", &f, &[], BAD_RANGE, 101).is_err());
        let before = p.state.clone();
        assert_eq!(p.read("owner", &f, 12, BAD_PREFIX).unwrap(), b"");
        assert_eq!(p.write("owner", &f, &[], BAD_PREFIX, 101).unwrap(), 0);
        assert_eq!(p.state, before);
        p.write("owner", &f, b"contents", OK, 101).unwrap();
        seek(&mut p, &f, 0);
        let before = p.state.clone();
        for admitted in [BAD_RANGE, BAD_PREFIX] {
            assert!(p.read("owner", &f, 4, admitted).is_err());
            assert_eq!(p.state, before);
            assert!(p.write("owner", &f, b"bad", admitted, 999).is_err());
            assert_eq!(p.state, before);
        }
        assert!(p.truncate("owner", &f, -1, 999).is_err());
        assert_eq!(p.state, before);
        assert!(p.seek("owner", &f, -1, SystemsFileSeek::Set).is_err());
        assert_eq!(p.state, before);
        for whence in [SystemsFileSeek::Data, SystemsFileSeek::Hole] {
            assert!(
                p.seek("owner", &f, -1, whence)
                    .unwrap_err()
                    .to_string()
                    .contains("ENXIO")
            );
            assert_eq!(p.state, before);
        }
        seek(&mut p, &f, i64::MAX - 1);
        let before = p.state.clone();
        assert!(
            p.write("owner", &f, b"two", OK, 102)
                .unwrap_err()
                .to_string()
                .contains("EINVAL")
        );
        assert_eq!(p.state, before);
        assert_eq!(p.write("owner", &f, b"t", OK, 102).unwrap(), 1);
        assert_eq!(metadata(&p, &f).size, MAX_OFFSET);
        let before = p.state.clone();
        assert!(
            p.write("owner", &f, b"x", OK, 103)
                .unwrap_err()
                .to_string()
                .contains("EINVAL")
        );
        assert_eq!(p.state, before);
        assert!(p.seek("owner", &f, 1, SystemsFileSeek::Current).is_err());
        assert_eq!(p.state, before);
        assert_eq!(
            p.seek("owner", &f, -1, SystemsFileSeek::End).unwrap(),
            MAX_OFFSET - 1
        );
        let before = p.state.clone();
        assert!(
            p.read("owner", &f, 999, OK)
                .unwrap_err()
                .to_string()
                .contains("EINVAL")
        );
        assert_eq!(p.state, before);
        assert_eq!(p.read("owner", &f, 1, OK).unwrap(), b"t");
        let appended = p
            .create(
                "owner",
                SystemsFileRights::ReadWrite,
                0o600,
                SystemsFileUmask::default(),
                true,
                100,
            )
            .unwrap();
        p.truncate("owner", &appended, i64::MAX - 1, 101).unwrap();
        assert_eq!(p.write("owner", &appended, b"two", OK, 102).unwrap(), 1);
        seek(&mut p, &appended, 0);
        let before = p.state.clone();
        assert!(
            p.write("owner", &appended, b"x", OK, 103)
                .unwrap_err()
                .to_string()
                .contains("EFBIG")
        );
        assert_eq!(p.state, before);
    }
    #[test]
    fn append_rights_private_umask_and_initial_mode_obey_the_selected_contract() {
        // TOPAL-SYSTEMS-VOLATILE-FILE-PERSONALITY-001
        let mut parent = SystemsFileUmask::default();
        let mut child = parent;
        assert_eq!(child.replace(0xffff_ffc0), 0o022);
        assert_eq!(child.apply(0o7777), 0o7077);
        assert_eq!(parent.apply(0o7777), 0o7755);
        assert_eq!(parent.replace(0), 0o022);
        let mut p = pool();
        assert!(
            p.create("owner", SystemsFileRights::Read, 0o777, parent, false, 100)
                .is_err()
        );
        let f = p
            .create("owner", SystemsFileRights::Write, 0o666, child, true, 100)
            .unwrap();
        assert_eq!(metadata(&p, &f).mode, 0o066);
        assert_eq!(p.write("owner", &f, b"a", OK, 101).unwrap(), 1);
        seek(&mut p, &f, 0);
        assert_eq!(p.write("owner", &f, &[], BAD_PREFIX, 999).unwrap(), 0);
        assert_eq!(metadata(&p, &f).position, 0);
        assert_eq!(p.write("owner", &f, b"b", OK, 102).unwrap(), 1);
        assert_eq!(metadata(&p, &f).position, 2);
        let before = p.state.clone();
        assert!(
            p.read("owner", &f, 0, OK)
                .unwrap_err()
                .to_string()
                .contains("EBADF")
        );
        assert_eq!(p.state, before);
        let identity = p.state.references[&f.ticket];
        let index = p.state.descriptions[&identity].object;
        assert_eq!(
            &p.state
                .page_records()
                .find(|page| page.owner == Some(p.state.object(index).unwrap().identity))
                .unwrap()
                .bytes[..2],
            b"ab"
        );
    }
    #[test]
    fn pool_authority_stale_aliases_identity_exhaustion_and_terminal_cleanup_are_independent() {
        // TOPAL-SYSTEMS-VOLATILE-FILE-OWNERSHIP-001
        assert!(SystemsVolatileFilePool::new("").is_err());
        let mut p = pool();
        let f = create(&mut p);
        let mut other = pool();
        let before = p.state.clone();
        let other_before = other.state.clone();
        assert!(p.alias("wrong", &f).is_err());
        assert_eq!(p.state, before);
        assert!(other.close("owner", &f).is_err());
        assert_eq!(other.state, other_before);
        let mut aliases = vec![];
        for _ in 1..32 {
            aliases.push(p.alias("owner", &f).unwrap());
        }
        let before = p.state.clone();
        assert!(
            p.alias("owner", &f)
                .unwrap_err()
                .to_string()
                .contains("EMFILE")
        );
        assert_eq!(p.state, before);
        p.close("owner", &aliases.pop().unwrap()).unwrap();
        p.state.next_ticket = u64::MAX;
        let before = p.state.clone();
        assert!(p.alias("owner", &f).is_err());
        assert_eq!(p.state, before);
        p.state.next_ticket = 100;
        p.state.set_next_identity(u64::MAX);
        let before = p.state.clone();
        assert!(
            p.create(
                "owner",
                SystemsFileRights::ReadWrite,
                0o600,
                SystemsFileUmask::default(),
                false,
                100
            )
            .is_err()
        );
        assert_eq!(p.state, before);
        p.write("owner", &f, b"erase retained data", OK, 101)
            .unwrap();
        p.terminate("owner").unwrap();
        assert!(p.state.page_records().all(|page| page.owner.is_none()
            && page.index == 0
            && page.bytes.iter().all(|&b| b == 0)));
        assert!(p.state.objects().iter().all(Option::is_none));
        assert!(p.state.descriptions.is_empty() && p.state.references.is_empty());
        assert_eq!((p.state.next_identity(), p.state.next_ticket), (0, 0));
        let before = p.state.clone();
        assert!(p.close("owner", &f).is_err());
        assert!(p.metadata("owner", &aliases[0]).is_err());
        assert!(p.terminate("owner").is_err());
        assert_eq!(p.state, before);
    }
}
