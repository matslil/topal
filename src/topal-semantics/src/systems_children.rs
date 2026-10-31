//! Serialized child ownership; modeled bytes/IDs grant no machine authority.
use crate::SystemsModelError;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsChildPrivateState {
    pub image: Vec<u8>,
    pub stack: Vec<u8>,
    pub registers: Vec<u64>,
    pub extended_state: Vec<u8>,
    pub tls: u64,
    pub cwd: u64,
    pub slots: BTreeMap<u8, (u64, bool)>,
    pub clear_tid: u64,
    pub robust_head: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Child {
    Vacant,
    Running {
        pid: u32,
        parent: SystemsChildPrivateState,
    },
    Exited {
        pid: u32,
        status: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsSerializedChildren {
    owner: String,
    next_pid: u32,
    child: Child,
    active: SystemsChildPrivateState,
    // Shared service history deliberately does not roll back with the parent.
    description_positions: BTreeMap<u64, u64>,
    references: BTreeMap<u64, usize>,
    entropy_generation: u64,
}

fn error(message: &str) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-CHILD", message)
}

impl SystemsSerializedChildren {
    pub fn new(owner: &str, parent: SystemsChildPrivateState) -> Result<Self, SystemsModelError> {
        if owner.is_empty() {
            return Err(error("missing process owner"));
        }
        let mut references = BTreeMap::new();
        for (description, _) in parent.slots.values() {
            *references.entry(*description).or_insert(0) += 1;
        }
        Ok(Self {
            owner: owner.into(),
            next_pid: 2,
            child: Child::Vacant,
            active: parent,
            description_positions: references.keys().map(|key| (*key, 0)).collect(),
            references,
            entropy_generation: 0,
        })
    }
    fn owned(&self, owner: &str) -> Result<(), SystemsModelError> {
        if self.owner == owner && self.next_pid != 0 {
            Ok(())
        } else {
            Err(error("wrong or released process owner"))
        }
    }
    pub fn create(&mut self, owner: &str) -> Result<u32, SystemsModelError> {
        self.owned(owner)?;
        if self.child != Child::Vacant {
            return Err(error("child capacity occupied"));
        }
        let pid = self.next_pid;
        self.next_pid = if pid == i32::MAX as u32 { 2 } else { pid + 1 };
        let parent = self.active.clone();
        for (description, _) in parent.slots.values() {
            *self.references.get_mut(description).unwrap() += 1;
        }
        self.active.clear_tid = 0;
        self.active.robust_head = 0;
        self.child = Child::Running { pid, parent };
        Ok(pid)
    }
    pub fn identity(&self) -> (u32, u32) {
        if self.next_pid == 0 {
            return (0, 0);
        }
        if let Child::Running { pid, .. } = self.child {
            (pid, 1)
        } else {
            (1, 0)
        }
    }
    pub fn current(&self) -> &SystemsChildPrivateState {
        &self.active
    }
    #[cfg(test)]
    fn current_mut(
        &mut self,
        owner: &str,
    ) -> Result<&mut SystemsChildPrivateState, SystemsModelError> {
        self.owned(owner)?;
        Ok(&mut self.active)
    }
    pub fn advance_description(
        &mut self,
        owner: &str,
        slot: u8,
        bytes: u64,
    ) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        let (id, _) = self
            .active
            .slots
            .get(&slot)
            .ok_or_else(|| error("missing descriptor"))?;
        let position = self.description_positions.get_mut(id).unwrap();
        *position = position
            .checked_add(bytes)
            .ok_or_else(|| error("position overflow"))?;
        Ok(())
    }
    pub fn position(&self, description: u64) -> Option<u64> {
        self.description_positions.get(&description).copied()
    }
    pub fn advance_entropy(&mut self, owner: &str) -> Result<u64, SystemsModelError> {
        self.owned(owner)?;
        self.entropy_generation = self
            .entropy_generation
            .checked_add(1)
            .ok_or_else(|| error("entropy history overflow"))?;
        Ok(self.entropy_generation)
    }
    fn drop_slots(&mut self) {
        for (id, _) in self.active.slots.values() {
            let count = self.references.get_mut(id).unwrap();
            *count -= 1;
            if *count == 0 {
                self.references.remove(id);
                self.description_positions.remove(id);
            }
        }
        self.active.slots.clear();
    }
    pub fn exit_child(&mut self, owner: &str, status: u64) -> Result<u32, SystemsModelError> {
        self.owned(owner)?;
        let Child::Running { .. } = &self.child else {
            return Err(error("no active child"));
        };
        self.drop_slots();
        let Child::Running { pid, parent } = std::mem::replace(&mut self.child, Child::Vacant)
        else {
            unreachable!("the active child was checked before releasing its slots");
        };
        self.active = parent;
        self.child = Child::Exited {
            pid,
            status: status as u8,
        };
        Ok(pid)
    }
    pub fn wait(
        &mut self,
        owner: &str,
        selector: i32,
        options: u32,
        status_writable: bool,
        rusage_null: bool,
    ) -> Result<(u32, u32), SystemsModelError> {
        self.owned(owner)?;
        if options > 1 {
            return Err(error("EINVAL"));
        }
        if !rusage_null {
            return Err(error("ENOSYS"));
        }
        let Child::Exited { pid, status } = self.child else {
            return Err(error("ECHILD"));
        };
        if ![-1, 0, pid as i32].contains(&selector) {
            return Err(error("ECHILD"));
        }
        if !status_writable {
            return Err(error("EFAULT"));
        }
        self.child = Child::Vacant;
        Ok((pid, u32::from(status) << 8))
    }
    pub fn terminate_parent(&mut self, owner: &str) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        if matches!(self.child, Child::Running { .. }) {
            return Err(error("parent is parked"));
        }
        self.drop_slots();
        self.active.image.fill(0);
        self.active.stack.fill(0);
        self.active.registers.fill(0);
        self.active.extended_state.fill(0);
        self.active.tls = 0;
        self.active.cwd = 0;
        self.active.clear_tid = 0;
        self.active.robust_head = 0;
        self.child = Child::Vacant;
        self.next_pid = 0;
        self.entropy_generation = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_child_restores_parent_but_retains_shared_service_history() {
        // TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-001, TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-WAIT-001
        let parent = SystemsChildPrivateState {
            image: vec![1; 24],
            stack: vec![2; 16],
            registers: vec![3; 16],
            extended_state: vec![4; 512],
            tls: 5,
            cwd: 3,
            slots: BTreeMap::from([(3, (7, false)), (4, (7, true))]),
            clear_tid: 8,
            robust_head: 9,
        };
        let mut pool = SystemsSerializedChildren::new("root", parent.clone()).unwrap();
        assert!(pool.create("other").is_err());
        assert_eq!(pool.create("root").unwrap(), 2);
        let before = pool.clone();
        assert!(pool.create("root").is_err());
        assert_eq!(pool, before);
        assert_eq!(pool.identity(), (2, 1));
        assert_eq!(pool.current().clear_tid, 0);
        let child = pool.current_mut("root").unwrap();
        child.image.fill(10);
        child.stack.fill(11);
        child.registers.fill(12);
        child.extended_state.fill(13);
        child.tls = 14;
        child.cwd = 4;
        child.clear_tid = 15;
        child.robust_head = 16;
        pool.advance_description("root", 4, 100).unwrap();
        pool.advance_entropy("root").unwrap();
        assert!(pool.terminate_parent("root").is_err());
        assert_eq!(pool.exit_child("root", 0x107).unwrap(), 2);
        assert_eq!(pool.current(), &parent);
        assert_eq!(pool.identity(), (1, 0));
        assert_eq!(pool.position(7), Some(100));
        assert_eq!(pool.entropy_generation, 1);
        assert_eq!(pool.references[&7], 2);
        for (selector, options, writable, rusage) in [
            (3, 0, true, true),
            (-1, 2, true, true),
            (-1, 0, false, true),
            (-1, 0, true, false),
        ] {
            let before = pool.clone();
            assert!(
                pool.wait("root", selector, options, writable, rusage)
                    .is_err()
            );
            assert_eq!(pool, before);
        }
        assert!(pool.create("root").is_err());
        assert_eq!(pool.wait("root", -1, 1, true, true).unwrap(), (2, 7 << 8));
        assert!(pool.wait("root", -1, 0, true, true).is_err());
        assert_eq!(pool.create("root").unwrap(), 3);
        pool.exit_child("root", 0).unwrap();
        pool.terminate_parent("root").unwrap();
        assert!(pool.references.is_empty());
        assert!(pool.description_positions.is_empty());
        assert_eq!(pool.active.tls, 0);
        assert!(pool.create("root").is_err());
        assert_eq!(pool.identity(), (0, 0));
        assert!(
            pool.active
                .image
                .iter()
                .chain(&pool.active.stack)
                .chain(&pool.active.extended_state)
                .all(|b| *b == 0)
        );
    }
}
