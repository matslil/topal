//! Deterministic initial boot resource for the x86-64 kernel gate.

use crate::CompileError;

pub const X86_INITIAL_BOOT_RESOURCE_REVISION: &str = "topal.boot-resource.x86_64-initial-newc/1";
pub const X86_INITIAL_BOOT_RESOURCE_FILE: &str = "initramfs.cpio";
pub const X86_INITIAL_BOOT_RESOURCE_MEMBER: &str = "init";

const NEWC_MAGIC: &[u8; 6] = b"070701";

pub(crate) fn generate_initial_boot_resource(image: &[u8]) -> Result<Vec<u8>, CompileError> {
    if image.is_empty() {
        return Err(CompileError::Tool(
            "initial boot resource requires a nonempty image".into(),
        ));
    }
    let image_size = u32::try_from(image.len())
        .map_err(|_| CompileError::Tool("initial boot image exceeds newc limits".into()))?;
    let mut archive = Vec::new();
    append_newc_entry(
        &mut archive,
        1,
        0o100755,
        X86_INITIAL_BOOT_RESOURCE_MEMBER,
        image,
        image_size,
    )?;
    append_newc_entry(&mut archive, 2, 0, "TRAILER!!!", &[], 0)?;
    Ok(archive)
}

fn append_newc_entry(
    archive: &mut Vec<u8>,
    inode: u32,
    mode: u32,
    name: &str,
    data: &[u8],
    data_size: u32,
) -> Result<(), CompileError> {
    let name_size = u32::try_from(name.len() + 1)
        .map_err(|_| CompileError::Tool("initial boot resource name exceeds newc limits".into()))?;
    archive.extend_from_slice(NEWC_MAGIC);
    for field in [inode, mode, 0, 0, 1, 0, data_size, 0, 0, 0, 0, name_size, 0] {
        archive.extend_from_slice(format!("{field:08x}").as_bytes());
    }
    archive.extend_from_slice(name.as_bytes());
    archive.push(0);
    pad_four(archive);
    archive.extend_from_slice(data);
    pad_four(archive);
    Ok(())
}

fn pad_four(bytes: &mut Vec<u8>) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishes_one_deterministic_newc_member() {
        let first = generate_initial_boot_resource(b"elf-image").unwrap();
        let second = generate_initial_boot_resource(b"elf-image").unwrap();
        assert_eq!(first, second);
        assert!(first.starts_with(NEWC_MAGIC));
        assert!(first.windows(5).any(|window| window == b"init\0"));
        assert!(first.windows(9).any(|window| window == b"elf-image"));
        assert!(first.windows(11).any(|window| window == b"TRAILER!!!\0"));
        assert_eq!(first.len() % 4, 0);
    }

    #[test]
    fn rejects_an_empty_initial_image() {
        assert!(generate_initial_boot_resource(&[]).is_err());
    }
}
