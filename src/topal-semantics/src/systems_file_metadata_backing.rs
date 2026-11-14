//! Canonical owned bytes shared by normal observations and opaque region loans.
//! TOPAL-SYSTEMS-NAMESPACE-{OWNERSHIP,ACTION}-001.
use super::*;

/// Read-only derived observation. It owns no metadata or allocation authority.
pub(super) struct PageRecord<'a> {
    pub(super) owner: Option<u64>,
    pub(super) index: u64,
    pub(super) bytes: &'a Vec<u8>,
}

impl State {
    pub(super) fn word(&self, offset: usize) -> u64 {
        u64::from_le_bytes(self.metadata[offset..offset + 8].try_into().unwrap())
    }
    pub(super) fn put_word(&mut self, offset: usize, value: u64) {
        self.metadata[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
    pub(super) fn next_identity(&self) -> u64 {
        self.word(0)
    }
    pub(super) fn set_next_identity(&mut self, value: u64) {
        self.put_word(0, value);
    }
    pub(super) fn object(&self, slot: usize) -> Option<Object> {
        let base = 256 + slot * 128;
        (self.word(base) == 1).then(|| Object {
            identity: self.word(base + 8),
            size: self.word(base + 16),
            inode: SystemsInodeMetadata {
                bytes: self.metadata[base + 24..base + 56].try_into().unwrap(),
            },
            link_count: self.word(base + 56),
            may_link_unlinked: self.word(base + 64) == 1,
        })
    }
    pub(super) fn objects(&self) -> [Option<Object>; OBJECTS] {
        std::array::from_fn(|slot| self.object(slot))
    }
    pub(super) fn set_object(&mut self, slot: usize, object: Option<Object>) {
        let base = 256 + slot * 128;
        self.metadata[base..base + 128].fill(0);
        if let Some(object) = object {
            self.put_word(base, 1);
            self.put_word(base + 8, object.identity);
            self.put_word(base + 16, object.size);
            self.metadata[base + 24..base + 56].copy_from_slice(&object.inode.bytes);
            self.put_word(base + 56, object.link_count);
            self.put_word(base + 64, u64::from(object.may_link_unlinked));
        }
    }
    // Only direct portable model transitions use this helper. An opaque source
    // loan always stores directly into metadata; completion never copies a
    // decoded observation or expected-state snapshot back into this backing.
    pub(super) fn update_object(&mut self, slot: usize, update: impl FnOnce(&mut Object)) {
        let mut value = self.object(slot).expect("validated live object");
        update(&mut value);
        self.set_object(slot, Some(value));
    }
    #[cfg(test)]
    pub(super) fn object_inode_bytes_mut(&mut self, slot: usize) -> &mut [u8; 32] {
        let base = 256 + slot * 128 + 24;
        (&mut self.metadata[base..base + 32]).try_into().unwrap()
    }
    pub(super) fn name(&self, slot: usize) -> Option<NameEntry> {
        let identity = self.word(64 + slot * 16);
        let length = self.word(72 + slot * 16).min(256) as usize;
        let base = 1536 + slot * 256;
        (identity != 0).then(|| NameEntry {
            identity,
            name: self.metadata[base..base + length].to_vec(),
        })
    }
    pub(super) fn names(&self) -> [Option<NameEntry>; NAMES] {
        std::array::from_fn(|slot| self.name(slot))
    }
    pub(super) fn set_name(&mut self, slot: usize, entry: Option<NameEntry>) {
        self.metadata[1536 + slot * 256..1536 + (slot + 1) * 256].fill(0);
        self.metadata[64 + slot * 16..80 + slot * 16].fill(0);
        if let Some(entry) = entry {
            assert!(entry.name.len() <= 255);
            self.put_word(64 + slot * 16, entry.identity);
            self.put_word(72 + slot * 16, entry.name.len() as u64);
            let start = 1536 + slot * 256;
            self.metadata[start..start + entry.name.len()].copy_from_slice(&entry.name);
        }
    }
    pub(super) fn update_name(&mut self, slot: usize, update: impl FnOnce(&mut NameEntry)) {
        let mut value = self.name(slot).expect("validated live entry");
        update(&mut value);
        self.set_name(slot, Some(value));
    }
    pub(super) fn directory_inode(&self) -> SystemsInodeMetadata {
        SystemsInodeMetadata {
            bytes: self.metadata[32..64].try_into().unwrap(),
        }
    }
    pub(super) fn update_directory_inode(
        &mut self,
        update: impl FnOnce(&mut SystemsInodeMetadata),
    ) {
        let mut value = self.directory_inode();
        update(&mut value);
        self.metadata[32..64].copy_from_slice(&value.bytes);
    }
    pub(super) fn page_allocation(&self) -> [u8; PAGES] {
        self.metadata[24..32].try_into().unwrap()
    }
    pub(super) fn page_allocation_mut(&mut self) -> &mut [u8; PAGES] {
        (&mut self.metadata[24..32]).try_into().unwrap()
    }
    pub(super) fn page_owner(&self, slot: usize) -> Option<u64> {
        let base = 1024 + slot * 32;
        let identity = self.word(base);
        (identity != 0).then_some(identity)
    }
    pub(super) fn page_index(&self, slot: usize) -> u64 {
        self.word(1032 + slot * 32)
    }
    pub(super) fn associate_page(&mut self, slot: usize, owner: u64, index: u64) {
        let base = 1024 + slot * 32;
        self.metadata[base..base + 32].fill(0);
        self.put_word(base, owner);
        self.put_word(base + 8, index);
    }
    pub(super) fn erase_page(&mut self, slot: usize) {
        self.pages[slot].bytes.fill(0);
        self.metadata[1024 + slot * 32..1056 + slot * 32].fill(0);
        self.page_allocation_mut()[slot] = 0;
    }
    pub(super) fn page_records(&self) -> impl Iterator<Item = PageRecord<'_>> {
        self.pages
            .iter()
            .enumerate()
            .map(|(slot, page)| PageRecord {
                owner: self.page_owner(slot),
                index: self.page_index(slot),
                bytes: &page.bytes,
            })
    }
    #[cfg(test)]
    pub(super) fn page_snapshot(&self) -> Vec<(Option<u64>, u64, Vec<u8>)> {
        self.page_records()
            .map(|p| (p.owner, p.index, p.bytes.clone()))
            .collect()
    }
    pub(super) fn metadata_layout_consistent(&self) -> bool {
        let zero = |range: std::ops::Range<usize>| self.metadata[range].iter().all(|&b| b == 0);
        self.next_identity() >= 7
            && [8..24, 128..256, 768..1024, 1280..1536, 2560..4096]
                .into_iter()
                .all(zero)
            && self.word(32) & !0o7777 == 0o40000
            && (0..OBJECTS).all(|slot| {
                let base = 256 + slot * 128;
                match self.word(base) {
                    0 => zero(base..base + 128),
                    1 => {
                        self.word(base + 16) <= MAX_OFFSET
                            && self.word(base + 24) & !0o7777 == 0o100000
                            && self.word(base + 64) <= 1
                            && zero(base + 72..base + 128)
                    }
                    _ => false,
                }
            })
            && (0..NAMES).all(|slot| {
                let identity = self.word(64 + slot * 16);
                let length = self.word(72 + slot * 16);
                let start = 1536 + slot * 256;
                if identity == 0 {
                    length == 0 && zero(start..start + 256)
                } else {
                    (1..=255).contains(&length) && zero(start + length as usize..start + 256)
                }
            })
            && (0..PAGES).all(|slot| {
                let base = 1024 + slot * 32;
                self.word(base) == 0 && zero(base..base + 32)
                    || self.word(base) >= 7 && zero(base + 16..base + 32)
            })
    }
}
