//! Owned static-process pair; modeled bytes/IDs grant no machine authority.
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
pub enum SystemsChildWaitResult {
    Reaped { pid: u32, status: u32 },
    NoExitedChild,
    Suspended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Child {
    Vacant,
    Running {
        pid: u32,
        inactive: SystemsChildPrivateState,
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
    active_pid: u32,
    parent_waiting: bool,
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
            active_pid: 1,
            parent_waiting: false,
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
        self.child = Child::Running {
            pid,
            inactive: parent,
        };
        self.active_pid = pid;
        Ok(pid)
    }
    pub fn identity(&self) -> (u32, u32) {
        if self.next_pid == 0 {
            return (0, 0);
        }
        if self.active_pid != 1 {
            (self.active_pid, 1)
        } else {
            (1, 0)
        }
    }
    pub fn current(&self) -> &SystemsChildPrivateState {
        &self.active
    }
    /// A completed yield either selects the exact owned counterpart or stays
    /// local when no counterpart is runnable. Shared state is never swapped.
    pub fn yield_current(&mut self, owner: &str) -> Result<bool, SystemsModelError> {
        self.owned(owner)?;
        let Child::Running { pid, inactive } = &mut self.child else {
            return Ok(false);
        };
        if self.active_pid != 1 && self.parent_waiting {
            return Ok(false);
        }
        std::mem::swap(&mut self.active, inactive);
        self.active_pid = if self.active_pid == 1 { *pid } else { 1 };
        Ok(true)
    }
    /// The caller has admitted a blocking live-child wait. Bind before the
    /// handoff; the parent cannot be selected again by an ordinary yield.
    pub fn park_parent_wait(
        &mut self,
        owner: &str,
        selector: i32,
    ) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        let Child::Running { pid, .. } = self.child else {
            return Err(error("ECHILD"));
        };
        if self.active_pid != 1 || ![-1, 0, pid as i32].contains(&selector) {
            return Err(error("ECHILD"));
        }
        if self.parent_waiting {
            return Err(error("parent already waiting"));
        }
        self.parent_waiting = true;
        self.yield_current(owner)?;
        Ok(())
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
        if self.active_pid == 1 {
            return Err(error("parent cannot exit as child"));
        }
        self.drop_slots();
        let Child::Running { pid, inactive } = std::mem::replace(&mut self.child, Child::Vacant)
        else {
            unreachable!("the active child was checked before releasing its slots");
        };
        self.active = inactive;
        self.active_pid = 1;
        self.parent_waiting = false;
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
    ) -> Result<SystemsChildWaitResult, SystemsModelError> {
        self.owned(owner)?;
        if options > 1 {
            return Err(error("EINVAL"));
        }
        if !rusage_null {
            return Err(error("ENOSYS"));
        }
        if self.active_pid != 1 {
            return Err(error("ECHILD"));
        }
        if let Child::Running { pid, .. } = self.child {
            if ![-1, 0, pid as i32].contains(&selector) {
                return Err(error("ECHILD"));
            }
            if options == 1 {
                return Ok(SystemsChildWaitResult::NoExitedChild);
            }
            self.park_parent_wait(owner, selector)?;
            return Ok(SystemsChildWaitResult::Suspended);
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
        Ok(SystemsChildWaitResult::Reaped {
            pid,
            status: u32::from(status) << 8,
        })
    }
    pub fn terminate_parent(&mut self, owner: &str) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        if self.active_pid != 1 {
            return Err(error("parent is parked"));
        }
        if let Child::Running { inactive, .. } = &mut self.child {
            for (id, _) in inactive.slots.values() {
                let count = self.references.get_mut(id).unwrap();
                *count -= 1;
                if *count == 0 {
                    self.references.remove(id);
                    self.description_positions.remove(id);
                }
            }
            inactive.image.fill(0);
            inactive.stack.fill(0);
            inactive.registers.fill(0);
            inactive.extended_state.fill(0);
            inactive.slots.clear();
            inactive.tls = 0;
            inactive.cwd = 0;
            inactive.clear_tid = 0;
            inactive.robust_head = 0;
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
        self.active_pid = 0;
        self.parent_waiting = false;
        self.entropy_generation = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pair() -> SystemsSerializedChildren {
        SystemsSerializedChildren::new(
            "root",
            SystemsChildPrivateState {
                image: vec![1; 24],
                stack: vec![2; 16],
                registers: vec![3; 16],
                extended_state: vec![4; 512],
                tls: 5,
                cwd: 3,
                slots: BTreeMap::from([(3, (7, false))]),
                clear_tid: 8,
                robust_head: 9,
            },
        )
        .unwrap()
    }
    #[test]
    fn cooperative_handoff_preserves_both_private_generations_and_shared_history() {
        // TOPAL-SYSTEMS-STATIC-PROCESS-PAIR-001
        let mut pool = pair();
        let parent = pool.current().clone();
        assert!(!pool.yield_current("root").unwrap());
        pool.create("root").unwrap();
        pool.current_mut("root").unwrap().image.fill(10);
        pool.current_mut("root").unwrap().tls = 11;
        let child = pool.current().clone();
        pool.advance_description("root", 3, 13).unwrap();
        pool.advance_entropy("root").unwrap();
        for _ in 0..4 {
            assert!(pool.yield_current("root").unwrap());
            assert_eq!(pool.identity(), (1, 0));
            assert_eq!(pool.current(), &parent);
            assert_eq!(
                pool.wait("root", -1, 1, false, true).unwrap(),
                SystemsChildWaitResult::NoExitedChild
            );
            assert_eq!(pool.position(7), Some(13));
            assert_eq!(pool.entropy_generation, 1);
            assert!(pool.yield_current("root").unwrap());
            assert_eq!(pool.identity(), (2, 1));
            assert_eq!(pool.current(), &child);
        }
        assert!(pool.yield_current("root").unwrap());
        let before = pool.clone();
        assert!(pool.park_parent_wait("other", -1).is_err());
        assert!(pool.park_parent_wait("root", 3).is_err());
        assert_eq!(pool, before);
        assert_eq!(
            pool.wait("root", -1, 0, false, true).unwrap(),
            SystemsChildWaitResult::Suspended
        );
        assert_eq!(pool.identity(), (2, 1));
        assert!(!pool.yield_current("root").unwrap());
        assert!(pool.wait("root", -1, 1, true, true).is_err());
        pool.exit_child("root", 9).unwrap();
        assert_eq!(pool.current(), &parent);
        assert!(!pool.parent_waiting);
        let before = pool.clone();
        assert!(pool.wait("root", -1, 0, false, true).is_err());
        assert_eq!(pool, before);
        assert_eq!(
            pool.wait("root", -1, 0, true, true).unwrap(),
            SystemsChildWaitResult::Reaped {
                pid: 2,
                status: 9 << 8
            }
        );
        assert!(pool.wait("root", -1, 0, true, true).is_err());
        pool.terminate_parent("root").unwrap();
        assert!(pool.references.is_empty());
        assert_eq!(pool.identity(), (0, 0));
    }
    #[test]
    fn parent_termination_releases_inactive_live_child_references() {
        // TOPAL-SYSTEMS-STATIC-PROCESS-PAIR-001
        let mut pool = pair();
        pool.create("root").unwrap();
        assert_eq!(pool.references[&7], 2);
        pool.yield_current("root").unwrap();
        let before = pool.clone();
        assert!(pool.exit_child("root", 0).is_err());
        assert_eq!(pool, before);
        pool.terminate_parent("root").unwrap();
        assert!(pool.references.is_empty());
        assert!(pool.description_positions.is_empty());
        assert_eq!(pool.child, Child::Vacant);
        assert!(pool.yield_current("root").is_err());
    }
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
        assert_eq!(
            pool.wait("root", -1, 1, true, true).unwrap(),
            SystemsChildWaitResult::Reaped {
                pid: 2,
                status: 7 << 8
            }
        );
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
