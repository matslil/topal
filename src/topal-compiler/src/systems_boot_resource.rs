//! Deterministic initial boot resource for the x86-64 kernel gate.

use crate::CompileError;

pub const X86_INITIAL_BOOT_RESOURCE_REVISION: &str = "topal.boot-resource.x86_64-initial-newc/2";
pub const X86_INITIAL_BOOT_RESOURCE_FILE: &str = "initramfs.cpio";
pub const X86_INITIAL_BOOT_RESOURCE_MEMBER: &str = "init";
pub const X86_STATIC_BOOT_RESOURCE_MEMBER: &str = "static-user.elf";
pub(crate) const X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET: u32 = 116;
pub(crate) const X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET: u32 = 12_532;

const NEWC_MAGIC: &[u8; 6] = b"070701";

pub(crate) fn generate_initial_boot_resource(
    image: &[u8],
    static_image: &[u8],
) -> Result<Vec<u8>, CompileError> {
    if image.is_empty() || static_image.is_empty() {
        return Err(CompileError::Tool(
            "initial boot resource requires nonempty init and static images".into(),
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
    if archive.len()
        != (X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + image.len()).next_multiple_of(4)
    {
        return Err(CompileError::Tool(
            "initial boot resource payload offset changed unexpectedly".into(),
        ));
    }
    let static_image_size = u32::try_from(static_image.len())
        .map_err(|_| CompileError::Tool("static boot image exceeds newc limits".into()))?;
    append_newc_entry(
        &mut archive,
        2,
        0o100755,
        X86_STATIC_BOOT_RESOURCE_MEMBER,
        static_image,
        static_image_size,
    )?;
    if archive.len()
        != (X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + static_image.len())
            .next_multiple_of(4)
    {
        return Err(CompileError::Tool(
            "static boot resource payload offset changed unexpectedly".into(),
        ));
    }
    append_newc_entry(&mut archive, 3, 0, "TRAILER!!!", &[], 0)?;
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
    use crate::systems_artifact::qualified_x86_64_static_user_image_fixture;

    #[test]
    fn publishes_two_deterministic_newc_members() {
        let mut initial = vec![0_u8; 12_288];
        initial[..9].copy_from_slice(b"elf-image");
        let static_image = qualified_x86_64_static_user_image_fixture();
        let first = generate_initial_boot_resource(&initial, &static_image).unwrap();
        let second = generate_initial_boot_resource(&initial, &static_image).unwrap();
        assert_eq!(first, second);
        assert!(first.starts_with(NEWC_MAGIC));
        assert!(first.windows(5).any(|window| window == b"init\0"));
        assert!(first.windows(9).any(|window| window == b"elf-image"));
        assert!(
            first
                .windows(16)
                .any(|window| window == b"static-user.elf\0")
        );
        assert_eq!(
            &first[X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                ..X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + static_image.len()],
            static_image
        );
        assert_eq!(
            &first[X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + 9],
            b"elf-image"
        );
        assert!(first.windows(11).any(|window| window == b"TRAILER!!!\0"));
        assert_eq!(first.len() % 4, 0);
        assert_eq!(first.len(), 2_037_200);
    }

    #[test]
    fn rejects_an_empty_member() {
        assert!(generate_initial_boot_resource(&[], b"static").is_err());
        assert!(generate_initial_boot_resource(b"init", &[]).is_err());
    }
}
