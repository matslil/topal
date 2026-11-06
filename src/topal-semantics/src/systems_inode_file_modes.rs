//! Shared inode permission laws; no compiled-source or Linux syscall claim.
//! TOPAL-SYSTEMS-INODE-MODE-STATE-001.
use super::*;
use crate::systems_region_library::InodeController;
use crate::{RegionLibraryBinding, RegionLibraryError, RegionLibraryInvocation, RegionLibraryView};

impl SystemsVolatileFilePool {
    /// Lend the actual inode after model owner/reference and request admission.
    /// This neither executes a source body nor proves physical reclamation.
    pub fn borrow_inode_mode_after_admission<'a>(
        &'a mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        permissions: u64,
        seconds: u64,
        binding: &'a RegionLibraryBinding,
    ) -> Result<(RegionLibraryInvocation<'a>, RegionLibraryView), RegionLibraryError> {
        let description = self
            .description(owner, file)
            .map_err(|_| RegionLibraryError::Owner)?;
        let object = self.state.descriptions[&description].object;
        self.borrow_object_inode_mode(object, permissions, seconds, binding)
    }

    pub fn borrow_named_inode_mode_after_admission<'a>(
        &'a mut self,
        owner: &str,
        name: &[u8],
        permissions: u64,
        seconds: u64,
        binding: &'a RegionLibraryBinding,
    ) -> Result<(RegionLibraryInvocation<'a>, RegionLibraryView), RegionLibraryError> {
        self.owned(owner).map_err(|_| RegionLibraryError::Owner)?;
        named::valid_name(name).map_err(|_| RegionLibraryError::Owner)?;
        let object = self.named_object(name).ok_or(RegionLibraryError::Owner)?;
        self.borrow_object_inode_mode(object, permissions, seconds, binding)
    }

    fn borrow_object_inode_mode<'a>(
        &'a mut self,
        object: usize,
        permissions: u64,
        seconds: u64,
        binding: &'a RegionLibraryBinding,
    ) -> Result<(RegionLibraryInvocation<'a>, RegionLibraryView), RegionLibraryError> {
        RegionLibraryInvocation::begin_inode(
            &mut self.state.objects[object]
                .as_mut()
                .ok_or(RegionLibraryError::Owner)?
                .inode,
            &mut self.state.inode_actions,
            (
                &self.owner,
                self.identity.clone(),
                InodeController::Filesystem,
            ),
            0o100000,
            (permissions, seconds),
            binding,
        )
    }

    pub fn borrow_directory_mode_after_admission<'a>(
        &'a mut self,
        owner: &str,
        permissions: u64,
        seconds: u64,
        binding: &'a RegionLibraryBinding,
    ) -> Result<(RegionLibraryInvocation<'a>, RegionLibraryView), RegionLibraryError> {
        self.owned(owner).map_err(|_| RegionLibraryError::Owner)?;
        RegionLibraryInvocation::begin_inode(
            &mut self.state.directory_inode,
            &mut self.state.inode_actions,
            (
                &self.owner,
                self.identity.clone(),
                InodeController::Filesystem,
            ),
            0o40000,
            (permissions, seconds),
            binding,
        )
    }

    /// Model the admitted prepared-root operation on a live description.
    /// Description read/write rights do not authorize or prohibit chmod.
    pub fn chmod(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        native_mode: u64,
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        let description = self.description(owner, file)?;
        let object = self.state.descriptions[&description].object;
        self.state.objects[object]
            .as_mut()
            .unwrap()
            .inode
            .chmod(native_mode, seconds);
        Ok(())
    }

    pub fn chmod_by_name(
        &mut self,
        owner: &str,
        name: &[u8],
        native_mode: u64,
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        named::valid_name(name)?;
        let object = self.named_object(name).ok_or_else(|| error("ENOENT"))?;
        self.state.objects[object]
            .as_mut()
            .unwrap()
            .inode
            .chmod(native_mode, seconds);
        Ok(())
    }

    pub fn chmod_directory(
        &mut self,
        owner: &str,
        native_mode: u64,
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        self.state.directory_inode.chmod(native_mode, seconds);
        Ok(())
    }

    pub fn directory_metadata(
        &self,
        owner: &str,
    ) -> Result<SystemsInodeMetadata, SystemsModelError> {
        self.owned(owner)?;
        Ok(self.state.directory_inode.clone())
    }
}

#[cfg(test)]
#[path = "systems_inode_file_action_tests.rs"]
mod action_tests;

#[cfg(test)]
mod tests {
    use super::*;
    const OK: SystemsFileUserAdmission = SystemsFileUserAdmission {
        numerical_range: true,
        complete_prefix: true,
    };

    fn open(
        pool: &mut SystemsVolatileFilePool,
        name: &[u8],
        rights: SystemsFileRights,
    ) -> SystemsVolatileFile {
        pool.open_named(
            "owner",
            name,
            SystemsNamedFileOpen {
                rights,
                create: true,
                exclusive: false,
                truncate: false,
                append: false,
                mode: 0o666,
                umask: SystemsFileUmask::default(),
            },
            100,
        )
        .unwrap()
    }

    #[test]
    fn readonly_independent_and_inherited_descriptions_observe_one_inode_without_rights_changes() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
        let writer = open(&mut pool, b"file", SystemsFileRights::ReadWrite);
        pool.write("owner", &writer, b"payload", OK, 101).unwrap();
        let reader = open(&mut pool, b"file", SystemsFileRights::Read);
        let inherited = pool.alias("owner", &reader).unwrap();
        let before = pool.state.clone();
        pool.chmod("owner", &reader, 0xfedcba9876540ed1, 102)
            .unwrap();
        let mut expected = before;
        let inode = &mut expected.objects[0].as_mut().unwrap().inode.bytes;
        // The full state oracle below changes only the two specified words.
        inode[..8].copy_from_slice(&0o107321_u64.to_le_bytes());
        inode[24..].copy_from_slice(&102_u64.to_le_bytes());
        assert_eq!(pool.state, expected);
        for file in [&writer, &reader, &inherited] {
            let metadata = pool.metadata("owner", file).unwrap();
            assert_eq!(metadata.mode, 0o7321);
            assert_eq!(metadata.accessed_seconds, 100);
            assert_eq!(metadata.modified_seconds, 101);
            assert_eq!(metadata.changed_seconds, 102);
        }
        assert!(pool.write("owner", &inherited, b"denied", OK, 999).is_err());
        pool.chmod("owner", &inherited, 0, 103).unwrap();
        assert_eq!(pool.write("owner", &writer, b"!", OK, 104).unwrap(), 1);
        assert_eq!(pool.read("owner", &reader, 20, OK).unwrap(), b"payload!");
        assert_eq!(pool.metadata("owner", &inherited).unwrap().position, 8);
        assert_eq!(pool.metadata("owner", &writer).unwrap().mode, 0);
    }

    #[test]
    fn unlinked_old_inode_and_recreated_spelling_keep_independent_modes_and_lifetimes() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
        let old = open(&mut pool, b"same", SystemsFileRights::ReadWrite);
        let old_identity = pool.metadata("owner", &old).unwrap().identity;
        pool.write("owner", &old, b"old", OK, 101).unwrap();
        pool.unlink("owner", b"same", 102).unwrap();
        let fresh = open(&mut pool, b"same", SystemsFileRights::ReadWrite);
        let before = pool.metadata("owner", &fresh).unwrap();
        assert_ne!(before.identity, old_identity);
        pool.chmod("owner", &old, 0o6701, 103).unwrap();
        assert_eq!(pool.metadata("owner", &fresh).unwrap(), before);
        assert_eq!(pool.metadata("owner", &old).unwrap().links, 0);
        assert_eq!(pool.metadata("owner", &old).unwrap().mode, 0o6701);
        pool.chmod_by_name("owner", b"same", 0o4222, 104).unwrap();
        assert_eq!(pool.metadata("owner", &old).unwrap().mode, 0o6701);
        assert_eq!(pool.metadata("owner", &fresh).unwrap().mode, 0o4222);
        pool.close("owner", &old).unwrap();
        assert!(pool.chmod("owner", &old, 0o7777, 999).is_err());
        let reused = open(&mut pool, b"new", SystemsFileRights::ReadWrite);
        assert_ne!(
            pool.metadata("owner", &reused).unwrap().identity,
            old_identity
        );
        assert_eq!(pool.metadata("owner", &reused).unwrap().mode, 0o644);
        assert_eq!(pool.metadata("owner", &fresh).unwrap().mode, 0o4222);
        pool.close("owner", &fresh).unwrap();
        pool.chmod_by_name("owner", b"same", 0o1234, 105).unwrap();
        assert_eq!(
            pool.metadata_by_name("owner", b"same").unwrap().mode,
            0o1234
        );
    }

    #[test]
    fn directory_mode_survives_namespace_mutation_and_chmod_does_not_change_entries_or_mtime() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
        let file = open(&mut pool, b"entry", SystemsFileRights::ReadWrite);
        let entries = pool.directory_entries("owner", 0, 512, true).unwrap();
        let before = pool.directory_metadata("owner").unwrap();
        assert_eq!(before.mode(), 0o41777);
        pool.chmod_directory("owner", 0o4750, 110).unwrap();
        let after = pool.directory_metadata("owner").unwrap();
        assert_eq!(after.mode(), 0o44750);
        assert_eq!(after.accessed_seconds(), before.accessed_seconds());
        assert_eq!(after.modified_seconds(), before.modified_seconds());
        assert_eq!(after.changed_seconds(), 110);
        assert_eq!(
            pool.directory_entries("owner", 0, 512, true).unwrap(),
            entries
        );
        pool.unlink("owner", b"entry", 111).unwrap();
        let after = pool.directory_metadata("owner").unwrap();
        assert_eq!(after.mode(), 0o44750);
        assert_eq!(after.modified_seconds(), 111);
        assert_eq!(after.changed_seconds(), 111);
        pool.close("owner", &file).unwrap();
        pool.terminate("owner").unwrap();
        assert_eq!(pool.state.directory_inode.encoded(), &[0; 32]);
        assert!(pool.chmod_directory("owner", 0o777, 999).is_err());
        assert!(pool.chmod_by_name("owner", b"entry", 0o777, 999).is_err());
    }

    #[test]
    fn failed_owner_reference_name_and_controller_admissions_preserve_complete_state() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut pool = SystemsVolatileFilePool::new("owner").unwrap();
        let file = open(&mut pool, b"file", SystemsFileRights::ReadWrite);
        let mut foreign_pool = SystemsVolatileFilePool::new("owner").unwrap();
        let foreign = open(&mut foreign_pool, b"file", SystemsFileRights::ReadWrite);
        let before = pool.state.clone();
        assert!(pool.chmod("other", &file, 0, 999).is_err());
        assert!(pool.chmod("owner", &foreign, 0, 999).is_err());
        assert!(pool.chmod_directory("other", 0, 999).is_err());
        for name in [
            &b"missing"[..],
            b"",
            b".",
            b"..",
            b"a/b",
            b"a\0b",
            &[b'x'; 256],
        ] {
            assert!(pool.chmod_by_name("owner", name, 0, 999).is_err());
        }
        assert_eq!(pool.state, before);
        for domain in [false, true] {
            if domain {
                pool.state.page_actions.poisoned = true;
            } else {
                pool.state.position_actions.poisoned = true;
            }
            let before = pool.state.clone();
            assert!(pool.chmod("owner", &file, 0, 999).is_err());
            assert!(pool.chmod_by_name("owner", b"file", 0, 999).is_err());
            assert!(pool.chmod_directory("owner", 0, 999).is_err());
            assert_eq!(pool.state, before);
            pool.state.page_actions.poisoned = false;
            pool.state.position_actions.poisoned = false;
        }
        pool.close("owner", &file).unwrap();
        let before = pool.state.clone();
        assert!(pool.chmod("owner", &file, 0, 999).is_err());
        assert_eq!(pool.state, before);
    }
}
