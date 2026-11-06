//! TOPAL-SYSTEMS-INODE-MODE-STATE-001: both endpoints share inode metadata.
use super::*;
use crate::systems_region_library::InodeController;
use crate::{RegionLibraryBinding, RegionLibraryError, RegionLibraryInvocation, RegionLibraryView};

impl SystemsPipePool {
    pub fn borrow_inode_mode_after_admission<'a>(
        &'a mut self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
        permissions: u64,
        seconds: u64,
        binding: &'a RegionLibraryBinding,
    ) -> Result<(RegionLibraryInvocation<'a>, RegionLibraryView), RegionLibraryError> {
        let index = self
            .endpoint(owner, endpoint)
            .map_err(|_| RegionLibraryError::Owner)?;
        RegionLibraryInvocation::begin_inode(
            &mut self.objects[index].as_mut().unwrap().inode,
            &mut self.inode_actions,
            (&self.owner, self.identity.clone(), InodeController::Pipe),
            0o10000,
            (permissions, seconds),
            binding,
        )
    }

    /// An admitted prepared-root metadata operation, independent of direction.
    pub fn chmod(
        &mut self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
        native_mode: u64,
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        let index = self.endpoint(owner, endpoint)?;
        self.objects[index]
            .as_mut()
            .unwrap()
            .inode
            .chmod(native_mode, seconds);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct Snapshot {
        objects: Vec<Option<Pipe>>,
        endpoints: BTreeMap<u64, (u64, Direction)>,
        waits: BTreeMap<u64, Wait>,
        counters: [u64; 3],
        live: bool,
        inode_actions: crate::systems_region_library::ControllerActionState,
    }

    fn snapshot(pool: &SystemsPipePool) -> Snapshot {
        Snapshot {
            objects: pool.objects.clone(),
            endpoints: pool.endpoints.clone(),
            waits: pool.waits.clone(),
            counters: [pool.next_generation, pool.next_endpoint, pool.next_wait],
            live: pool.live,
            inode_actions: pool.inode_actions.clone(),
        }
    }

    #[test]
    fn both_endpoints_and_inherited_aliases_share_mode_without_modifying_fifo_state() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut pool = SystemsPipePool::new("owner").unwrap();
        let (read, write) = pool.create("owner", false, true, 100).unwrap();
        let inherited = pool.alias("owner", &write).unwrap();
        pool.write("owner", &write, b"payload", true, false, 101)
            .unwrap();
        let mut expected = snapshot(&pool);
        pool.chmod("owner", &read, u64::MAX, 102).unwrap();
        let bytes = &mut expected.objects[0].as_mut().unwrap().inode.bytes;
        bytes[..8].copy_from_slice(&0o17777_u64.to_le_bytes());
        bytes[24..].copy_from_slice(&102_u64.to_le_bytes());
        assert_eq!(snapshot(&pool), expected);
        for endpoint in [&read, &write, &inherited] {
            let metadata = pool.metadata("owner", endpoint).unwrap();
            assert_eq!(metadata.mode, 0o7777);
            assert_eq!(metadata.created_seconds, 100);
            assert_eq!(metadata.accessed_seconds, 100);
            assert_eq!(metadata.modified_seconds, 101);
            assert_eq!(metadata.changed_seconds, 102);
        }
        pool.chmod("owner", &inherited, 0, 103).unwrap();
        match pool.read("owner", &read, 7, true, 104).unwrap() {
            SystemsPipeTransfer::Complete(bytes) => assert_eq!(bytes, b"payload"),
            _ => panic!("permission change altered an existing endpoint"),
        }
        assert_eq!(pool.metadata("owner", &read).unwrap().changed_seconds, 103);
        pool.write("owner", &write, b"!", true, false, 105).unwrap();
        let metadata = pool.metadata("owner", &inherited).unwrap();
        assert_eq!(metadata.mode, 0);
        assert_eq!(metadata.modified_seconds, 105);
        assert_eq!(metadata.changed_seconds, 105);
    }

    #[test]
    fn mode_change_does_not_wake_waiters_and_reused_slots_do_not_inherit_metadata() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut pool = SystemsPipePool::new("owner").unwrap();
        let (read, write) = pool.create("owner", false, true, 100).unwrap();
        let wait = match pool.read("owner", &read, 1, false, 101).unwrap() {
            SystemsPipeTransfer::Blocked(wait) => wait,
            _ => panic!("expected a retained reader"),
        };
        let waits = pool.waits.clone();
        pool.chmod("owner", &write, 0o4755, 102).unwrap();
        assert_eq!(pool.waits, waits);
        assert!(!pool.take_ready("owner", &wait).unwrap());
        pool.close("owner", &write).unwrap();
        assert!(pool.take_ready("owner", &wait).unwrap());
        pool.chmod("owner", &read, 0o1234, 103).unwrap();
        pool.close("owner", &read).unwrap();
        assert!(pool.objects.iter().all(Option::is_none));
        let (fresh, fresh_writer) = pool.create("owner", true, true, 104).unwrap();
        assert_ne!(fresh.generation, read.generation);
        let before = snapshot(&pool);
        assert!(pool.chmod("owner", &read, 0, 999).is_err());
        assert!(pool.chmod("owner", &write, 0, 999).is_err());
        assert_eq!(snapshot(&pool), before);
        let metadata = pool.metadata("owner", &fresh).unwrap();
        assert_eq!(metadata.mode, 0o600);
        assert_eq!(metadata.changed_seconds, 104);
        pool.chmod("owner", &fresh_writer, 0o7654, 105).unwrap();
        pool.terminate("owner").unwrap();
        let before = snapshot(&pool);
        assert!(pool.chmod("owner", &fresh, 0, 999).is_err());
        assert_eq!(snapshot(&pool), before);
        assert!(pool.objects.iter().all(Option::is_none));
    }

    #[test]
    fn equal_labels_generation_and_ticket_do_not_authorize_a_foreign_pipe() {
        // TOPAL-SYSTEMS-INODE-MODE-STATE-001.
        let mut first = SystemsPipePool::new("owner").unwrap();
        let mut second = SystemsPipePool::new("owner").unwrap();
        let (read, _) = first.create("owner", false, true, 100).unwrap();
        let (foreign, _) = second.create("owner", false, true, 100).unwrap();
        assert_eq!(
            (read.ticket, read.generation),
            (foreign.ticket, foreign.generation)
        );
        let before = snapshot(&first);
        assert!(first.chmod("other", &read, 0, 999).is_err());
        assert!(first.chmod("owner", &foreign, 0, 999).is_err());
        assert_eq!(snapshot(&first), before);
        assert_eq!(second.metadata("owner", &foreign).unwrap().mode, 0o600);
    }
}

#[cfg(test)]
#[path = "systems_inode_pipe_action_tests.rs"]
mod action_tests;
