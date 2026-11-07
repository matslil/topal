//! TOPAL-SYSTEMS-POSITIONAL-FILE-{TRANSFER,OBSERVATION}-001.
use super::*;

impl SystemsVolatileFilePool {
    pub fn read_at(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        count: u64,
        position: i64,
        admitted: SystemsFileUserAdmission,
    ) -> Result<Vec<u8>, SystemsModelError> {
        self.owned(owner)?;
        if position < 0 {
            return Err(error("EINVAL"));
        }
        self.read_position(owner, file, count, admitted, Some(position as u64))
    }
    pub fn write_at(
        &mut self,
        owner: &str,
        file: &SystemsVolatileFile,
        bytes: &[u8],
        position: i64,
        admitted: SystemsFileUserAdmission,
        seconds: u64,
    ) -> Result<usize, SystemsModelError> {
        self.owned(owner)?;
        if position < 0 {
            return Err(error("EINVAL"));
        }
        self.write_position(owner, file, bytes, admitted, seconds, Some(position as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const OK: SystemsFileUserAdmission = SystemsFileUserAdmission {
        numerical_range: true,
        complete_prefix: true,
    };
    fn create(p: &mut SystemsVolatileFilePool, name: &[u8], append: bool) -> SystemsVolatileFile {
        p.open_named(
            "owner",
            name,
            SystemsNamedFileOpen {
                rights: SystemsFileRights::ReadWrite,
                create: true,
                exclusive: false,
                truncate: false,
                append,
                mode: 0o600,
                umask: SystemsFileUmask::default(),
            },
            100,
        )
        .unwrap()
    }
    fn at(p: &SystemsVolatileFilePool, file: &SystemsVolatileFile) -> u64 {
        p.metadata("owner", file).unwrap().position
    }
    #[test]
    fn positional_bytes_share_objects_but_never_publish_alias_or_independent_positions() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let f = create(&mut p, b"file", false);
        p.write("owner", &f, b"abcdefgh", OK, 101).unwrap();
        let alias = p.alias("owner", &f).unwrap();
        let other = create(&mut p, b"file", false);
        p.seek("owner", &other, 2, SystemsFileSeek::Set).unwrap();
        assert_eq!(p.read_at("owner", &f, 3, 2, OK).unwrap(), b"cde");
        assert_eq!(p.write_at("owner", &alias, b"XY", 1, OK, 102).unwrap(), 2);
        assert_eq!(p.read_at("owner", &other, 8, 0, OK).unwrap(), b"aXYdefgh");
        assert_eq!((at(&p, &f), at(&p, &alias), at(&p, &other)), (8, 8, 2));
        p.seek("owner", &f, i64::MAX, SystemsFileSeek::Set).unwrap();
        assert_eq!(p.read_at("owner", &alias, 8, 0, OK).unwrap(), b"aXYdefgh");
        assert_eq!(p.write_at("owner", &other, b"!", 3, OK, 103).unwrap(), 1);
        assert_eq!(at(&p, &alias), MAX_OFFSET);
        assert_eq!(at(&p, &other), 2);
    }
    #[test]
    fn append_uses_eof_after_supplied_interval_checks_without_advancing_any_position() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let f = create(&mut p, b"file", false);
        p.write("owner", &f, b"prefix", OK, 101).unwrap();
        let append = create(&mut p, b"file", true);
        assert_eq!(
            p.write_at("owner", &append, b"suffix", 0, OK, 102).unwrap(),
            6
        );
        assert_eq!(p.read_at("owner", &f, 12, 0, OK).unwrap(), b"prefixsuffix");
        assert_eq!((at(&p, &f), at(&p, &append)), (6, 0));
        let before = p.state.clone();
        assert!(
            p.write_at("owner", &append, b"XX", i64::MAX - 1, OK, 999)
                .unwrap_err()
                .to_string()
                .contains("EINVAL")
        );
        assert_eq!(p.state, before);
        assert_eq!(
            p.write_at("owner", &append, b"", i64::MAX, OK, 999)
                .unwrap(),
            0
        );
        assert_eq!(p.state, before);
    }
    #[test]
    fn sparse_unlinked_bytes_and_final_independent_description_reclamation_preserve_positions() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let f = create(&mut p, b"file", false);
        let other = create(&mut p, b"file", false);
        p.write("owner", &f, b"old", OK, 101).unwrap();
        p.unlink("owner", b"file", 102).unwrap();
        let sparse = (1_i64 << 48) + 13;
        assert_eq!(
            p.write_at("owner", &other, b"AB", sparse, OK, 103).unwrap(),
            2
        );
        assert_eq!(
            p.read_at("owner", &f, 8, sparse - 6, OK).unwrap(),
            b"\0\0\0\0\0\0AB"
        );
        assert_eq!((at(&p, &f), at(&p, &other)), (3, 0));
        assert_eq!(p.metadata("owner", &f).unwrap().allocated_bytes, 2 * PAGE);
        p.close("owner", &f).unwrap();
        assert_eq!(p.read_at("owner", &other, 3, 0, OK).unwrap(), b"old");
        p.close("owner", &other).unwrap();
        assert!(p.state.objects().iter().all(Option::is_none));
        assert!(
            p.state
                .page_records()
                .all(|page| page.owner.is_none() && page.bytes.iter().all(|&b| b == 0))
        );
    }
    #[test]
    fn zero_eof_rejected_copies_and_full_intervals_preserve_the_complete_store() {
        let mut p = SystemsVolatileFilePool::new("owner").unwrap();
        let f = create(&mut p, b"file", false);
        p.write("owner", &f, b"contents", OK, 101).unwrap();
        let before = p.state.clone();
        for admission in [
            SystemsFileUserAdmission {
                numerical_range: false,
                ..OK
            },
            SystemsFileUserAdmission {
                complete_prefix: false,
                ..OK
            },
        ] {
            assert!(p.read_at("owner", &f, 1, 0, admission).is_err());
            assert!(p.write_at("owner", &f, b"!", 0, admission, 999).is_err());
            assert_eq!(p.state, before);
        }
        assert!(p.read_at("owner", &f, 0, -1, OK).is_err());
        assert!(p.write_at("owner", &f, b"", -1, OK, 999).is_err());
        assert!(p.read_at("owner", &f, 1, i64::MAX, OK).is_err());
        assert!(p.write_at("owner", &f, b"!", i64::MAX, OK, 999).is_err());
        assert_eq!(p.read_at("owner", &f, 0, i64::MAX, OK).unwrap(), b"");
        assert_eq!(p.write_at("owner", &f, b"", i64::MAX, OK, 999).unwrap(), 0);
        assert_eq!(p.read_at("owner", &f, 1, 100, OK).unwrap(), b"");
        assert_eq!(p.state, before);
        assert_eq!(
            p.write_at("owner", &f, &[0xa7; 300], 4093, OK, 102)
                .unwrap(),
            3
        );
        assert_eq!(p.read_at("owner", &f, 300, 4093, OK).unwrap(), [0xa7; 3]);
        assert_eq!(at(&p, &f), 8);
    }
}
