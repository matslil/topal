//! Owned FIFO and readiness model; numeric observations grant no authority.
use crate::SystemsModelError;
use std::collections::BTreeMap;
use std::sync::Arc;

const CAPACITY: usize = 4096;
const OBJECTS: usize = 4;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Direction {
    Read,
    Write,
}

#[derive(Debug)]
pub struct SystemsPipeEndpoint {
    pool: Arc<()>,
    ticket: u64,
    generation: u64,
    direction: Direction,
}

#[derive(Debug)]
pub struct SystemsPipeWait {
    pool: Arc<()>,
    ticket: u64,
    generation: u64,
}

#[derive(Debug)]
pub enum SystemsPipeTransfer<T> {
    Complete(T),
    Blocked(SystemsPipeWait),
}

#[derive(Debug, Eq, PartialEq)]
pub struct SystemsPipeMetadata {
    pub generation: u64,
    pub capacity: usize,
    pub buffered: usize,
    pub readers: usize,
    pub writers: usize,
    pub created_seconds: u64,
    pub accessed_seconds: u64,
    pub modified_seconds: u64,
    pub nonblocking: bool,
}

#[derive(Debug)]
struct Pipe {
    generation: u64,
    bytes: Vec<u8>,
    head: usize,
    count: usize,
    references: [usize; 2],
    nonblocking: bool,
    times: [u64; 3],
}
#[derive(Debug)]
struct Wait {
    generation: u64,
    endpoint: u64,
    direction: Direction,
    needed: usize,
    ready: bool,
}

#[derive(Debug)]
pub struct SystemsPipePool {
    owner: String,
    identity: Arc<()>,
    objects: Vec<Option<Pipe>>,
    endpoints: BTreeMap<u64, (u64, Direction)>,
    waits: BTreeMap<u64, Wait>,
    next_generation: u64,
    next_endpoint: u64,
    next_wait: u64,
    live: bool,
}

fn error(message: &str) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-PIPE", message)
}

impl SystemsPipePool {
    pub fn new(owner: &str) -> Result<Self, SystemsModelError> {
        if owner.is_empty() {
            return Err(error("missing pipe owner"));
        }
        Ok(Self {
            owner: owner.into(),
            identity: Arc::new(()),
            objects: (0..OBJECTS).map(|_| None).collect(),
            endpoints: BTreeMap::new(),
            waits: BTreeMap::new(),
            next_generation: 1,
            next_endpoint: 1,
            next_wait: 1,
            live: true,
        })
    }
    fn owned(&self, owner: &str) -> Result<(), SystemsModelError> {
        if !self.live || self.owner != owner {
            return Err(error("wrong or released pipe owner"));
        }
        Ok(())
    }
    fn endpoint(
        &self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
    ) -> Result<usize, SystemsModelError> {
        self.owned(owner)?;
        if !Arc::ptr_eq(&self.identity, &endpoint.pool)
            || self.endpoints.get(&endpoint.ticket)
                != Some(&(endpoint.generation, endpoint.direction))
        {
            return Err(error("stale or wrong-pool endpoint"));
        }
        self.objects
            .iter()
            .position(|object| {
                object
                    .as_ref()
                    .is_some_and(|p| p.generation == endpoint.generation)
            })
            .ok_or_else(|| error("endpoint without FIFO owner"))
    }
    pub fn create(
        &mut self,
        owner: &str,
        nonblocking: bool,
        output_writable: bool,
        observed_seconds: u64,
    ) -> Result<(SystemsPipeEndpoint, SystemsPipeEndpoint), SystemsModelError> {
        self.owned(owner)?;
        let index = self
            .objects
            .iter()
            .position(Option::is_none)
            .ok_or_else(|| error("ENFILE"))?;
        if self.endpoints.len() > 30 {
            return Err(error("EMFILE"));
        }
        let next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| error("generation exhausted"))?;
        let next_endpoint = self
            .next_endpoint
            .checked_add(2)
            .ok_or_else(|| error("endpoint exhausted"))?;
        if !output_writable {
            return Err(error("EFAULT"));
        }
        let generation = self.next_generation;
        let read = SystemsPipeEndpoint {
            pool: self.identity.clone(),
            ticket: self.next_endpoint,
            generation,
            direction: Direction::Read,
        };
        let write = SystemsPipeEndpoint {
            pool: self.identity.clone(),
            ticket: self.next_endpoint + 1,
            generation,
            direction: Direction::Write,
        };
        self.objects[index] = Some(Pipe {
            generation,
            bytes: vec![0; CAPACITY],
            head: 0,
            count: 0,
            references: [1, 1],
            nonblocking,
            times: [observed_seconds; 3],
        });
        self.endpoints
            .insert(read.ticket, (generation, Direction::Read));
        self.endpoints
            .insert(write.ticket, (generation, Direction::Write));
        self.next_generation = next_generation;
        self.next_endpoint = next_endpoint;
        Ok((read, write))
    }
    pub fn alias(
        &mut self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
    ) -> Result<SystemsPipeEndpoint, SystemsModelError> {
        let index = self.endpoint(owner, endpoint)?;
        if self.endpoints.len() == 32 {
            return Err(error("EMFILE"));
        }
        let next = self
            .next_endpoint
            .checked_add(1)
            .ok_or_else(|| error("endpoint exhausted"))?;
        let alias = SystemsPipeEndpoint {
            pool: self.identity.clone(),
            ticket: self.next_endpoint,
            generation: endpoint.generation,
            direction: endpoint.direction,
        };
        self.objects[index].as_mut().unwrap().references[endpoint.direction as usize] += 1;
        self.endpoints
            .insert(alias.ticket, (alias.generation, alias.direction));
        self.next_endpoint = next;
        Ok(alias)
    }
    pub fn metadata(
        &self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
    ) -> Result<SystemsPipeMetadata, SystemsModelError> {
        let index = self.endpoint(owner, endpoint)?;
        let p = self.objects[index].as_ref().unwrap();
        Ok(SystemsPipeMetadata {
            generation: p.generation,
            capacity: CAPACITY,
            buffered: p.count,
            readers: p.references[0],
            writers: p.references[1],
            created_seconds: p.times[0],
            accessed_seconds: p.times[1],
            modified_seconds: p.times[2],
            nonblocking: p.nonblocking,
        })
    }
    fn block(
        &mut self,
        endpoint: &SystemsPipeEndpoint,
        needed: usize,
    ) -> Result<SystemsPipeWait, SystemsModelError> {
        if self.waits.len() >= 2 || self.waits.values().any(|w| w.endpoint == endpoint.ticket) {
            return Err(error(
                "wait owner already suspended or pair capacity exhausted",
            ));
        }
        let next = self
            .next_wait
            .checked_add(1)
            .ok_or_else(|| error("wait identity exhausted"))?;
        let ticket = self.next_wait;
        self.waits.insert(
            ticket,
            Wait {
                generation: endpoint.generation,
                endpoint: endpoint.ticket,
                direction: endpoint.direction,
                needed,
                ready: false,
            },
        );
        self.next_wait = next;
        Ok(SystemsPipeWait {
            pool: self.identity.clone(),
            ticket,
            generation: endpoint.generation,
        })
    }
    fn notify(&mut self, index: usize) {
        let p = self.objects[index].as_ref().unwrap();
        for w in self
            .waits
            .values_mut()
            .filter(|w| w.generation == p.generation)
        {
            let ready = match w.direction {
                Direction::Read => p.count > 0 || p.references[1] == 0,
                Direction::Write => {
                    p.references[0] == 0
                        || if w.needed > CAPACITY {
                            p.count < CAPACITY
                        } else {
                            CAPACITY - p.count >= w.needed
                        }
                }
            };
            w.ready |= ready;
        }
    }
    pub fn read(
        &mut self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
        count: usize,
        output_writable: bool,
        observed_seconds: u64,
    ) -> Result<SystemsPipeTransfer<Vec<u8>>, SystemsModelError> {
        let index = self.endpoint(owner, endpoint)?;
        if endpoint.direction != Direction::Read {
            return Err(error("EBADF"));
        }
        let p = self.objects[index].as_ref().unwrap();
        if count == 0 || (p.count == 0 && p.references[1] == 0) {
            return Ok(SystemsPipeTransfer::Complete(vec![]));
        }
        if p.count == 0 {
            if p.nonblocking {
                return Err(error("EAGAIN"));
            }
            return self.block(endpoint, 1).map(SystemsPipeTransfer::Blocked);
        }
        let size = count.min(p.count);
        if !output_writable {
            return Err(error("EFAULT"));
        }
        let p = self.objects[index].as_mut().unwrap();
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(p.bytes[p.head]);
            p.bytes[p.head] = 0;
            p.head = (p.head + 1) % CAPACITY;
        }
        p.count -= size;
        p.times[1] = observed_seconds;
        self.notify(index);
        Ok(SystemsPipeTransfer::Complete(result))
    }
    pub fn write(
        &mut self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
        bytes: &[u8],
        input_readable: bool,
        sigpipe_ignored: bool,
        observed_seconds: u64,
    ) -> Result<SystemsPipeTransfer<usize>, SystemsModelError> {
        let index = self.endpoint(owner, endpoint)?;
        if endpoint.direction != Direction::Write {
            return Err(error("EBADF"));
        }
        if bytes.is_empty() {
            return Ok(SystemsPipeTransfer::Complete(0));
        }
        let p = self.objects[index].as_ref().unwrap();
        if p.references[0] == 0 {
            return Err(error(if sigpipe_ignored { "EPIPE" } else { "ENOSYS" }));
        }
        let free = CAPACITY - p.count;
        if free == 0 || (bytes.len() <= CAPACITY && free < bytes.len()) {
            if p.nonblocking {
                return Err(error("EAGAIN"));
            }
            return self
                .block(endpoint, bytes.len())
                .map(SystemsPipeTransfer::Blocked);
        }
        if !input_readable {
            return Err(error("EFAULT"));
        }
        let size = bytes.len().min(free);
        let p = self.objects[index].as_mut().unwrap();
        for (offset, value) in bytes[..size].iter().enumerate() {
            p.bytes[(p.head + p.count + offset) % CAPACITY] = *value;
        }
        p.count += size;
        p.times[2] = observed_seconds;
        self.notify(index);
        Ok(SystemsPipeTransfer::Complete(size))
    }
    fn wait(&self, owner: &str, wait: &SystemsPipeWait) -> Result<&Wait, SystemsModelError> {
        self.owned(owner)?;
        if !Arc::ptr_eq(&self.identity, &wait.pool) {
            return Err(error("wrong-pool wait"));
        }
        self.waits
            .get(&wait.ticket)
            .filter(|w| w.generation == wait.generation)
            .ok_or_else(|| error("stale wait identity"))
    }
    pub fn take_ready(
        &mut self,
        owner: &str,
        wait: &SystemsPipeWait,
    ) -> Result<bool, SystemsModelError> {
        if !self.wait(owner, wait)?.ready {
            return Ok(false);
        }
        self.waits.remove(&wait.ticket);
        Ok(true)
    }
    pub fn cancel_wait(
        &mut self,
        owner: &str,
        wait: &SystemsPipeWait,
    ) -> Result<(), SystemsModelError> {
        self.wait(owner, wait)?;
        self.waits.remove(&wait.ticket);
        Ok(())
    }
    pub fn close(
        &mut self,
        owner: &str,
        endpoint: &SystemsPipeEndpoint,
    ) -> Result<(), SystemsModelError> {
        let index = self.endpoint(owner, endpoint)?;
        if self.waits.values().any(|w| w.endpoint == endpoint.ticket) {
            return Err(error("wait still retains endpoint"));
        }
        self.endpoints.remove(&endpoint.ticket);
        let p = self.objects[index].as_mut().unwrap();
        p.references[endpoint.direction as usize] -= 1;
        self.notify(index);
        if self.objects[index].as_ref().unwrap().references == [0, 0] {
            assert!(
                !self
                    .waits
                    .values()
                    .any(|w| w.generation == endpoint.generation)
            );
            self.objects[index].as_mut().unwrap().bytes.fill(0);
            self.objects[index] = None;
        }
        Ok(())
    }
    pub fn terminate(&mut self, owner: &str) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        self.waits.clear();
        self.endpoints.clear();
        for p in self.objects.iter_mut().flatten() {
            p.bytes.fill(0);
        }
        self.objects.iter_mut().for_each(|p| *p = None);
        self.next_generation = 0;
        self.next_endpoint = 0;
        self.next_wait = 0;
        self.live = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn complete<T>(transfer: SystemsPipeTransfer<T>) -> T {
        match transfer {
            SystemsPipeTransfer::Complete(value) => value,
            _ => panic!("unexpected suspension"),
        }
    }
    fn blocked<T>(transfer: SystemsPipeTransfer<T>) -> SystemsPipeWait {
        match transfer {
            SystemsPipeTransfer::Blocked(wait) => wait,
            _ => panic!("missing suspension"),
        }
    }
    #[test]
    fn atomic_wrap_faults_and_nonblocking_boundaries_preserve_fifo_bytes() {
        // TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-001, TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-TRANSFER-001
        let mut p = SystemsPipePool::new("root").unwrap();
        assert!(p.create("root", true, false, 100).is_err());
        assert_eq!(p.next_generation, 1);
        let (read, write) = p.create("root", true, true, 100).unwrap();
        assert!(p.read("root", &write, 1, true, 101).is_err());
        assert!(p.read("root", &read, 1, true, 101).is_err());
        let bytes: Vec<_> = (0..3000).map(|n| (n % 251) as u8).collect();
        assert_eq!(
            complete(p.write("root", &write, &bytes, true, false, 102).unwrap()),
            3000
        );
        let before = p.metadata("root", &read).unwrap();
        assert!(
            p.write("root", &write, &[7; 2000], true, false, 103)
                .is_err()
        );
        assert!(p.read("root", &read, 2000, false, 103).is_err());
        assert_eq!(p.metadata("root", &read).unwrap(), before);
        assert_eq!(
            complete(p.read("root", &read, 2500, true, 104).unwrap()),
            bytes[..2500]
        );
        assert!(
            p.objects[0].as_ref().unwrap().bytes[..2500]
                .iter()
                .all(|b| *b == 0)
        );
        assert!(
            p.write("root", &write, &[9; 100], false, false, 105)
                .is_err()
        );
        assert_eq!(
            complete(
                p.write("root", &write, &[9; 4097], true, false, 106)
                    .unwrap()
            ),
            3596
        );
        let mut expected = bytes[2500..].to_vec();
        expected.extend_from_slice(&[9; 3596]);
        assert_eq!(
            complete(p.read("root", &read, 8192, true, 107).unwrap()),
            expected
        );
        assert!(p.objects[0].as_ref().unwrap().bytes.iter().all(|b| *b == 0));
        let alias = p.alias("root", &write).unwrap();
        p.close("root", &write).unwrap();
        assert!(p.read("root", &read, 1, true, 108).is_err());
        p.close("root", &alias).unwrap();
        assert!(complete(p.read("root", &read, 1, false, 109).unwrap()).is_empty());
        p.close("root", &read).unwrap();
        assert!(p.objects.iter().all(Option::is_none));
    }
    #[test]
    fn waits_follow_exact_generation_predicates_peer_lifetime_and_once_only_delivery() {
        // TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-WAIT-001
        let mut p = SystemsPipePool::new("root").unwrap();
        let (read, write) = p.create("root", false, true, 100).unwrap();
        let (other_read, other_write) = p.create("root", false, true, 100).unwrap();
        let waiting = blocked(p.read("root", &read, 1, false, 101).unwrap());
        assert!(!p.take_ready("root", &waiting).unwrap());
        p.write("root", &other_write, b"unrelated", true, false, 101)
            .unwrap();
        assert!(!p.take_ready("root", &waiting).unwrap());
        p.write("root", &write, b"abc", true, false, 102).unwrap();
        assert!(p.take_ready("root", &waiting).unwrap());
        assert!(p.take_ready("root", &waiting).is_err());
        assert!(p.read("root", &read, 3, false, 103).is_err());
        assert_eq!(
            complete(p.read("root", &read, 3, true, 104).unwrap()),
            b"abc"
        );
        p.write("root", &write, &[1; 4096], true, false, 105)
            .unwrap();
        let waiting = blocked(
            p.write("root", &write, &[2; 4096], true, false, 106)
                .unwrap(),
        );
        p.read("root", &read, 4095, true, 107).unwrap();
        assert!(!p.take_ready("root", &waiting).unwrap());
        p.read("root", &read, 1, true, 108).unwrap();
        assert!(p.take_ready("root", &waiting).unwrap());
        p.write("root", &write, &[2; 4096], true, false, 109)
            .unwrap();
        let waiting = blocked(p.write("root", &write, b"x", true, false, 110).unwrap());
        p.close("root", &read).unwrap();
        assert!(p.take_ready("root", &waiting).unwrap());
        assert!(
            p.write("root", &write, b"x", true, false, 111)
                .unwrap_err()
                .to_string()
                .contains("ENOSYS")
        );
        assert!(
            p.write("root", &write, b"x", true, true, 111)
                .unwrap_err()
                .to_string()
                .contains("EPIPE")
        );
        p.close("root", &write).unwrap();
        let (fresh_read, fresh_write) = p.create("root", false, true, 112).unwrap();
        assert_ne!(fresh_read.generation, read.generation);
        assert!(p.metadata("root", &read).is_err());
        assert!(p.take_ready("root", &waiting).is_err());
        let waiting = blocked(p.read("root", &fresh_read, 1, true, 113).unwrap());
        p.close("root", &fresh_write).unwrap();
        assert!(p.take_ready("root", &waiting).unwrap());
        assert!(complete(p.read("root", &fresh_read, 1, false, 114).unwrap()).is_empty());
        p.close("root", &fresh_read).unwrap();
        p.close("root", &other_read).unwrap();
        p.close("root", &other_write).unwrap();
    }
    #[test]
    fn wrong_pool_capacity_cancellation_and_outer_termination_reject_stale_authority() {
        // TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-001, TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-WAIT-001
        let mut p = SystemsPipePool::new("root").unwrap();
        let mut other = SystemsPipePool::new("root").unwrap();
        let mut handles = vec![];
        for _ in 0..OBJECTS {
            handles.push(p.create("root", false, true, 0).unwrap());
        }
        assert!(
            p.create("root", false, true, 0)
                .unwrap_err()
                .to_string()
                .contains("ENFILE")
        );
        assert!(other.close("root", &handles[0].0).is_err());
        assert!(p.close("other", &handles[0].0).is_err());
        let wait = blocked(p.read("root", &handles[0].0, 1, true, 0).unwrap());
        assert!(other.take_ready("root", &wait).is_err());
        assert!(p.close("root", &handles[0].0).is_err());
        p.cancel_wait("root", &wait).unwrap();
        assert!(p.take_ready("root", &wait).is_err());
        let _wait = blocked(p.read("root", &handles[0].0, 1, true, 0).unwrap());
        p.terminate("root").unwrap();
        assert!(p.objects.iter().all(Option::is_none));
        assert!(p.endpoints.is_empty() && p.waits.is_empty());
        assert!(p.metadata("root", &handles[0].0).is_err());
        assert!(p.create("root", false, true, 0).is_err());
    }
}
