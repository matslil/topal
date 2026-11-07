//! TK-DEC-281; direct portable transitions, not source/native execution.
//! TOPAL-SYSTEMS-NAMESPACE-{OWNERSHIP,TRANSACTION}-001.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsFileRename {
    Replace,
    NoReplace,
    Exchange,
}

impl SystemsFileRename {
    /// Linux unsigned-int adaptation belongs to the personality, not core Nat.
    pub fn from_native(flags: u64) -> Result<Self, SystemsModelError> {
        let flags = flags as u32;
        if flags & !7 != 0 || flags & 2 != 0 && flags & 5 != 0 {
            return Err(error("EINVAL"));
        }
        match flags {
            0 => Ok(Self::Replace),
            1 => Ok(Self::NoReplace),
            2 => Ok(Self::Exchange),
            _ => Err(error("unqualified whiteout operation")),
        }
    }
}

impl SystemsVolatileFilePool {
    /// Model O_TMPFILE|O_EXCL's permanent refusal of initial link publication.
    /// Existing `create` models an anonymous temporary inode eligible to link.
    pub fn create_unlinkable(
        &mut self,
        owner: &str,
        rights: SystemsFileRights,
        mode: u32,
        umask: SystemsFileUmask,
        append: bool,
        seconds: u64,
    ) -> Result<SystemsVolatileFile, SystemsModelError> {
        let file = self.create(owner, rights, mode, umask, append, seconds)?;
        let description = self.state.references[&file.ticket];
        let object = self.state.descriptions[&description].object;
        self.state
            .update_object(object, |inode| inode.may_link_unlinked = false);
        Ok(file)
    }

    pub fn hard_link(
        &mut self,
        owner: &str,
        source: &[u8],
        destination: &[u8],
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        valid_name(source)?;
        let object = self.named_object(source).ok_or_else(|| error("ENOENT"))?;
        self.link_object(object, destination, seconds)
    }

    /// Already admitted empty-path lookup; the reference still proves actual
    /// owner/lifetime. Prepared credentials are not supplied by a numeric inode.
    pub fn hard_link_by_file(
        &mut self,
        owner: &str,
        source: &SystemsVolatileFile,
        destination: &[u8],
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        let description = self.description(owner, source)?;
        let object = self.state.descriptions[&description].object;
        self.link_object(object, destination, seconds)
    }

    fn link_object(
        &mut self,
        object: usize,
        destination: &[u8],
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        valid_name(destination)?;
        if self.named_entry(destination).is_some() {
            return Err(error("EEXIST"));
        }
        let inode = self.state.object(object).unwrap();
        if self.links(inode.identity) == 0 && !inode.may_link_unlinked {
            return Err(error("ENOENT"));
        }
        let slot = self
            .state
            .names()
            .iter()
            .position(Option::is_none)
            .ok_or_else(|| error("ENOSPC"))?;
        let identity = self.state.object(object).unwrap().identity;
        self.state.set_name(
            slot,
            Some(NameEntry {
                identity,
                name: destination.to_vec(),
            }),
        );
        self.state.update_object(object, |inode| {
            inode.link_count += 1;
            inode.may_link_unlinked = false;
            inode.inode.changed(seconds);
        });
        self.state
            .update_directory_inode(|inode| inode.modified(seconds));
        Ok(())
    }

    pub fn rename(
        &mut self,
        owner: &str,
        source: &[u8],
        destination: &[u8],
        operation: SystemsFileRename,
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        valid_name(source)?;
        valid_name(destination)?;
        let old = self.named_entry(source).ok_or_else(|| error("ENOENT"))?;
        let new = self.named_entry(destination);
        if operation == SystemsFileRename::NoReplace && new.is_some() {
            return Err(error("EEXIST"));
        }
        if operation == SystemsFileRename::Exchange && new.is_none() {
            return Err(error("ENOENT"));
        }
        let old_identity = self.state.name(old).unwrap().identity;
        let new_identity = new.map(|index| self.state.name(index).unwrap().identity);
        // Linux vfs_rename's same-inode return precedes timestamp mutation.
        if new_identity == Some(old_identity) {
            return Ok(());
        }
        let source_object = self.named_object(source).unwrap();
        let victim = self.named_object(destination);
        if operation == SystemsFileRename::Exchange {
            let new = new.unwrap();
            self.state
                .update_name(old, |entry| entry.identity = new_identity.unwrap());
            self.state
                .update_name(new, |entry| entry.identity = old_identity);
        } else if let Some(new) = new {
            self.state
                .update_name(new, |entry| entry.identity = old_identity);
            self.state.set_name(old, None);
            self.state
                .update_object(victim.unwrap(), |object| object.link_count -= 1);
        } else {
            self.state
                .update_name(old, |entry| entry.name = destination.to_vec());
        }
        self.state
            .update_object(source_object, |object| object.inode.changed(seconds));
        if let Some(object) = victim {
            self.state
                .update_object(object, |object| object.inode.changed(seconds));
            self.reclaim_unlinked(object);
        }
        self.state
            .update_directory_inode(|inode| inode.modified(seconds));
        Ok(())
    }
}

#[cfg(test)]
#[path = "systems_namespace_transaction_tests.rs"]
mod tests;
