//! TOPAL-SYSTEMS-NAMED-FILE-{OWNERSHIP,PATH,OBSERVATION}-001.
use super::*;
use crate::{SystemsFileOpenFacts, SystemsFileOpenPlan};

#[derive(Clone, Copy, Debug)]
pub struct SystemsNamedFileOpen {
    pub rights: SystemsFileRights,
    pub create: bool,
    pub exclusive: bool,
    pub truncate: bool,
    pub append: bool,
    pub mode: u32,
    pub umask: SystemsFileUmask,
}
/// Copied semantic names/identities, never an object or directory capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsNamedDirectoryEntry {
    pub cookie: u64,
    pub identity: u64,
    pub name: Vec<u8>,
    pub directory: bool,
    pub native_x86_64_bytes: usize,
}
fn valid_name(name: &[u8]) -> Result<(), SystemsModelError> {
    if name.len() > 255 {
        return Err(error("ENAMETOOLONG"));
    }
    if name.is_empty() || name == b"." || name == b".." || name.contains(&0) || name.contains(&b'/')
    {
        return Err(error("invalid copied regular component"));
    }
    Ok(())
}
impl SystemsVolatileFilePool {
    fn named_object(&self, name: &[u8]) -> Option<usize> {
        self.state
            .objects
            .iter()
            .position(|o| o.as_ref().is_some_and(|o| o.name.as_deref() == Some(name)))
    }
    pub fn open_named(
        &mut self,
        owner: &str,
        name: &[u8],
        request: SystemsNamedFileOpen,
        seconds: u64,
    ) -> Result<SystemsVolatileFile, SystemsModelError> {
        self.owned(owner)?;
        // Native slot selection precedes final name disposition. References
        // represent the aggregate bounded descriptor owners, not object links.
        let (ticket, next_ticket) = self.ticket()?;
        valid_name(name)?;
        let existing = self.named_object(name);
        let plan = SystemsFileOpenFacts {
            write: request.rights != SystemsFileRights::Read,
            directory: false,
            directory_required: false,
            mutable: true,
            exists: existing.is_some(),
            create: request.create,
            exclusive: request.exclusive,
            truncate: request.truncate,
        }
        .plan();
        match plan {
            SystemsFileOpenPlan::Missing => return Err(error("ENOENT")),
            SystemsFileOpenPlan::AlreadyExists => return Err(error("EEXIST")),
            SystemsFileOpenPlan::OpenDescription
            | SystemsFileOpenPlan::CreateDescription
            | SystemsFileOpenPlan::TruncateDescription => {}
            _ => return Err(error("incompatible owned regular-open plan")),
        }
        let description = self.state.next_description;
        let next_description = description
            .checked_add(1)
            .ok_or_else(|| error("description identity exhausted"))?;
        let object = match existing {
            Some(index) => index,
            None => self
                .state
                .objects
                .iter()
                .position(Option::is_none)
                .ok_or_else(|| error("ENOSPC"))?,
        };
        // Every fallible check precedes namespace, data or counter publication.
        let created = if existing.is_none() {
            let identity = self.state.next_identity;
            Some((
                identity,
                identity
                    .checked_add(1)
                    .ok_or_else(|| error("object identity exhausted"))?,
            ))
        } else {
            None
        };
        if let Some((identity, next)) = created {
            self.state.objects[object] = Some(Object {
                identity,
                name: Some(name.to_vec()),
                size: 0,
                mode: request.umask.apply(request.mode),
                times: [seconds; 3],
            });
            self.state.next_identity = next;
            self.state.directory_times = [seconds; 2];
        } else if plan == SystemsFileOpenPlan::TruncateDescription {
            let o = self.state.objects[object].as_mut().unwrap();
            let identity = o.identity;
            o.size = 0;
            o.times[1] = seconds;
            o.times[2] = seconds;
            for (slot, page) in self
                .state
                .pages
                .iter_mut()
                .enumerate()
                .filter(|(_, p)| p.owner == Some(identity))
            {
                page.erase();
                self.state.page_allocation[slot] = 0;
            }
        }
        self.state.descriptions.insert(
            description,
            Description {
                object,
                rights: request.rights,
                append: request.append,
                position: [0; 8],
                references: 1,
            },
        );
        self.state.references.insert(ticket, description);
        self.state.next_description = next_description;
        self.state.next_ticket = next_ticket;
        Ok(SystemsVolatileFile {
            pool: self.identity.clone(),
            ticket,
        })
    }
    pub fn metadata_by_name(
        &self,
        owner: &str,
        name: &[u8],
    ) -> Result<SystemsVolatileFileMetadata, SystemsModelError> {
        self.owned(owner)?;
        valid_name(name)?;
        let index = self.named_object(name).ok_or_else(|| error("ENOENT"))?;
        Ok(self.object_metadata(index, 0))
    }
    pub fn unlink(
        &mut self,
        owner: &str,
        name: &[u8],
        seconds: u64,
    ) -> Result<(), SystemsModelError> {
        self.owned(owner)?;
        valid_name(name)?;
        let index = self.named_object(name).ok_or_else(|| error("ENOENT"))?;
        let object = self.state.objects[index].as_mut().unwrap();
        object.name = None;
        object.times[2] = seconds;
        self.state.directory_times = [seconds; 2];
        self.reclaim_unlinked(index);
        Ok(())
    }
    pub fn directory_times(&self, owner: &str) -> Result<[u64; 2], SystemsModelError> {
        self.owned(owner)?;
        Ok(self.state.directory_times)
    }
    pub fn directory_entries(
        &self,
        owner: &str,
        cookie: u64,
        count: u32,
        complete_output: bool,
    ) -> Result<(Vec<SystemsNamedDirectoryEntry>, u64), SystemsModelError> {
        self.owned(owner)?;
        if cookie > 6 {
            return Err(error("invalid qualified directory cookie"));
        }
        let budget = (count as usize).min(512);
        let (mut current, mut bytes, mut entries) = (cookie, 0, vec![]);
        while current < 6 {
            let candidate = match current {
                0 => Some((6, b".".to_vec(), true)),
                1 => Some((3, b"..".to_vec(), true)),
                _ => self.state.objects[(current - 2) as usize]
                    .as_ref()
                    .and_then(|o| o.name.as_ref().map(|n| (o.identity, n.clone(), false))),
            };
            if let Some((identity, name, directory)) = candidate {
                let length = (19 + name.len() + 1).div_ceil(8) * 8;
                if length > budget - bytes {
                    if entries.is_empty() {
                        return Err(error("EINVAL"));
                    }
                    break;
                }
                bytes += length;
                entries.push(SystemsNamedDirectoryEntry {
                    cookie: current + 1,
                    identity,
                    name,
                    directory,
                    native_x86_64_bytes: length,
                });
            }
            current += 1;
        }
        if bytes > 0 && !complete_output {
            return Err(error("EFAULT"));
        }
        Ok((entries, current))
    }
    pub(super) fn reclaim_unlinked(&mut self, object: usize) {
        if self.state.objects[object].as_ref().unwrap().name.is_some()
            || self.state.descriptions.values().any(|d| d.object == object)
        {
            return;
        }
        let identity = self.state.objects[object].as_ref().unwrap().identity;
        for (slot, page) in self
            .state
            .pages
            .iter_mut()
            .enumerate()
            .filter(|(_, p)| p.owner == Some(identity))
        {
            page.erase();
            self.state.page_allocation[slot] = 0;
        }
        self.state.objects[object] = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ADMITTED: SystemsFileUserAdmission = SystemsFileUserAdmission {
        numerical_range: true,
        complete_prefix: true,
    };
    fn request() -> SystemsNamedFileOpen {
        SystemsNamedFileOpen {
            rights: SystemsFileRights::ReadWrite,
            create: true,
            exclusive: false,
            truncate: false,
            append: false,
            mode: 0o666,
            umask: SystemsFileUmask::default(),
        }
    }
    fn create(p: &mut SystemsVolatileFilePool, name: &[u8]) -> SystemsVolatileFile {
        p.open_named("owner", name, request(), 100).unwrap()
    }
    fn rewind(p: &mut SystemsVolatileFilePool, f: &SystemsVolatileFile) {
        p.seek("owner", f, 0, SystemsFileSeek::Set).unwrap();
    }
    #[test]
    fn independent_positions_and_aliases_share_only_object_bytes_and_allocation() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let a = create(&mut p, b"alpha");
        assert_eq!(p.write("owner", &a, b"contents", ADMITTED, 101).unwrap(), 8);
        let first = p.metadata("owner", &a).unwrap();
        let b = p
            .open_named(
                "owner",
                b"alpha",
                SystemsNamedFileOpen {
                    create: false,
                    ..request()
                },
                102,
            )
            .unwrap();
        assert_eq!(p.metadata("owner", &b).unwrap().identity, first.identity);
        assert_eq!(p.metadata("owner", &b).unwrap().allocated_bytes, PAGE);
        assert_eq!(p.metadata("owner", &b).unwrap().position, 0);
        assert_eq!(p.metadata("owner", &a).unwrap().position, 8);
        assert_eq!(p.read("owner", &b, 3, ADMITTED).unwrap(), b"con");
        let alias = p.alias("owner", &b).unwrap();
        assert_eq!(p.read("owner", &alias, 5, ADMITTED).unwrap(), b"tents");
        assert_eq!(p.metadata("owner", &b).unwrap().position, 8);
        rewind(&mut p, &b);
        assert_eq!(p.write("owner", &a, b"!", ADMITTED, 103).unwrap(), 1);
        assert_eq!(p.read("owner", &b, 9, ADMITTED).unwrap(), b"contents!");
        p.close("owner", &a).unwrap();
        p.close("owner", &alias).unwrap();
        p.close("owner", &b).unwrap();
        assert_eq!(p.metadata_by_name("owner", b"alpha").unwrap().size, 9);
        assert!(p.state.descriptions.is_empty());
        assert_eq!(p.directory_times("owner").unwrap(), [100, 100]);
    }
    #[test]
    fn unlink_retains_independent_descriptions_and_recreation_has_fresh_identity() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let a = create(&mut p, b"same");
        p.write("owner", &a, b"old bytes", ADMITTED, 101).unwrap();
        let b = p
            .open_named(
                "owner",
                b"same",
                SystemsNamedFileOpen {
                    create: false,
                    ..request()
                },
                102,
            )
            .unwrap();
        let old = p.metadata("owner", &a).unwrap().identity;
        p.unlink("owner", b"same", 103).unwrap();
        assert!(p.metadata_by_name("owner", b"same").is_err());
        assert_eq!(p.metadata("owner", &a).unwrap().links, 0);
        let replacement = create(&mut p, b"same");
        assert!(p.metadata("owner", &replacement).unwrap().identity > old);
        p.write("owner", &replacement, b"new", ADMITTED, 104)
            .unwrap();
        p.close("owner", &a).unwrap();
        assert_eq!(p.read("owner", &b, 9, ADMITTED).unwrap(), b"old bytes");
        assert!(p.state.pages.iter().any(|page| page.owner == Some(old)));
        p.close("owner", &b).unwrap();
        assert!(p.state.pages.iter().all(|page| page.owner != Some(old)));
        assert_eq!(p.metadata_by_name("owner", b"same").unwrap().links, 1);
        rewind(&mut p, &replacement);
        assert_eq!(p.read("owner", &replacement, 9, ADMITTED).unwrap(), b"new");
    }
    #[test]
    fn truncate_open_uses_owner_write_authority_without_granting_descriptor_write() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let a = create(&mut p, b"data");
        p.write("owner", &a, b"erase contents", ADMITTED, 101)
            .unwrap();
        let readonly = p
            .open_named(
                "owner",
                b"data",
                SystemsNamedFileOpen {
                    rights: SystemsFileRights::Read,
                    create: false,
                    truncate: true,
                    mode: 0,
                    ..request()
                },
                102,
            )
            .unwrap();
        let metadata = p.metadata("owner", &a).unwrap();
        assert_eq!(
            (metadata.size, metadata.position, metadata.allocated_bytes),
            (0, 14, 0)
        );
        assert_eq!(metadata.mode, 0o644);
        assert_eq!(
            (metadata.modified_seconds, metadata.changed_seconds),
            (102, 102)
        );
        let before = p.state.clone();
        assert!(
            p.write("owner", &readonly, b"denied", ADMITTED, 103)
                .is_err()
        );
        assert_eq!(p.state, before);
        assert_eq!(p.read("owner", &readonly, 10, ADMITTED).unwrap(), b"");
        p.open_named(
            "owner",
            b"data",
            SystemsNamedFileOpen {
                truncate: true,
                ..request()
            },
            104,
        )
        .unwrap();
        assert_eq!(p.metadata("owner", &a).unwrap().modified_seconds, 104);
        assert_eq!(p.directory_times("owner").unwrap(), [100, 100]);
    }
    #[test]
    fn linked_and_open_unlinked_objects_share_capacity_and_errors_preserve_the_store() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let a = create(&mut p, b"a");
        p.write("owner", &a, b"a", ADMITTED, 101).unwrap();
        let alias = p.alias("owner", &a).unwrap();
        for name in [b"b", b"c", b"d"] {
            let f = create(&mut p, name);
            p.close("owner", &f).unwrap();
        }
        p.unlink("owner", b"a", 102).unwrap();
        let before = p.state.clone();
        assert!(
            p.open_named("owner", b"e", request(), 103)
                .unwrap_err()
                .to_string()
                .contains("ENOSPC")
        );
        assert_eq!(p.state, before);
        p.close("owner", &a).unwrap();
        assert!(p.open_named("owner", b"e", request(), 103).is_err());
        p.close("owner", &alias).unwrap();
        let e = create(&mut p, b"e");
        let before = p.state.clone();
        for (name, open, reason) in [
            (
                b"e".as_slice(),
                SystemsNamedFileOpen {
                    exclusive: true,
                    ..request()
                },
                "EEXIST",
            ),
            (
                b"missing",
                SystemsNamedFileOpen {
                    create: false,
                    ..request()
                },
                "ENOENT",
            ),
        ] {
            assert!(
                p.open_named("owner", name, open, 999)
                    .unwrap_err()
                    .to_string()
                    .contains(reason)
            );
            assert_eq!(p.state, before);
        }
        assert!(p.unlink("owner", b"missing", 999).is_err());
        assert_eq!(p.state, before);
        p.close("owner", &e).unwrap();
        p.unlink("owner", b"b", 105).unwrap();
        let f = create(&mut p, b"fresh");
        assert_eq!(p.metadata("owner", &f).unwrap().allocated_bytes, 0);
        p.terminate("owner").unwrap();
        assert!(
            p.state
                .pages
                .iter()
                .all(|page| page.owner.is_none() && page.bytes.iter().all(|&b| b == 0))
        );
        assert!(p.state.objects.iter().all(Option::is_none));
        assert!(p.state.descriptions.is_empty() && p.state.references.is_empty());
        assert_eq!(p.state.directory_times, [0, 0]);
    }
    #[test]
    fn exact_long_raw_names_directory_cookies_and_rejected_copies_are_independent() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let mut long = vec![0xff; 255];
        long[10] = 0x80;
        let a = create(&mut p, &long);
        p.close("owner", &a).unwrap();
        let initial = p.state.clone();
        let (dots, cursor) = p.directory_entries("owner", 0, 48, true).unwrap();
        assert_eq!(
            dots.iter().map(|e| e.name.as_slice()).collect::<Vec<_>>(),
            [b".".as_slice(), b".."]
        );
        assert_eq!(cursor, 2);
        assert!(
            p.directory_entries("owner", cursor, 279, true)
                .unwrap_err()
                .to_string()
                .contains("EINVAL")
        );
        assert_eq!(p.state, initial);
        assert!(
            p.directory_entries("owner", cursor, 280, false)
                .unwrap_err()
                .to_string()
                .contains("EFAULT")
        );
        assert_eq!(p.state, initial);
        let (entries, end) = p.directory_entries("owner", cursor, 280, true).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, long);
        assert_eq!(entries[0].native_x86_64_bytes, 280);
        assert_eq!(end, 6);
        assert_eq!(
            p.directory_entries("owner", end, 0, false).unwrap(),
            (vec![], 6)
        );
        p.unlink("owner", &long, 101).unwrap();
        let (dots, end) = p.directory_entries("owner", 0, 512, true).unwrap();
        assert_eq!((dots.len(), end), (2, 6));
        let before = p.state.clone();
        for invalid in [
            vec![],
            b".".to_vec(),
            b"..".to_vec(),
            b"a/b".to_vec(),
            b"a\0b".to_vec(),
            vec![b'x'; 256],
        ] {
            assert!(p.open_named("owner", &invalid, request(), 999).is_err());
            assert_eq!(p.state, before);
        }
    }
}
