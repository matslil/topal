//! Packaging adapter for the initial Topal kernel toolchain-gate image.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use topal_compiler::{
    SYSTEMS_INITIAL_USER_IMAGE_FILE, SYSTEMS_KERNEL_FILE, SYSTEMS_PROVENANCE_FILE,
    SYSTEMS_STATIC_USER_IMAGE_FILE, SystemsArtifactProvenance, publish_x86_64_linux_boot_image,
};

fn main() {
    if let Err(error) = run(env::args_os()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(mut arguments: impl Iterator<Item = OsString>) -> Result<(), Box<dyn Error>> {
    let program_name = arguments
        .next()
        .unwrap_or_else(|| OsString::from("topal-kernel-toolchain-gate-builder"));
    let linked = arguments.next().ok_or_else(|| usage(&program_name))?;
    let destination = arguments.next().ok_or_else(|| usage(&program_name))?;
    if arguments.next().is_some() {
        return Err(usage(&program_name).into());
    }
    publish(&PathBuf::from(linked), &PathBuf::from(destination))
}

fn usage(program_name: &OsString) -> String {
    format!(
        "usage: {} LINKED-ARTIFACT-DIRECTORY BOOT-DIRECTORY",
        Path::new(program_name).display()
    )
}

fn publish(linked: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    if destination.exists() {
        return Err(format!(
            "boot-adapter destination already exists: {}",
            destination.display()
        )
        .into());
    }
    let kernel_path = linked.join(SYSTEMS_KERNEL_FILE);
    let initial_user_image_path = linked.join(SYSTEMS_INITIAL_USER_IMAGE_FILE);
    let static_user_image_path = linked.join(SYSTEMS_STATIC_USER_IMAGE_FILE);
    let provenance_path = linked.join(SYSTEMS_PROVENANCE_FILE);
    let kernel = fs::read(&kernel_path)?;
    let initial_user_image = fs::read(&initial_user_image_path)?;
    let static_user_image = fs::read(&static_user_image_path)?;
    let provenance: SystemsArtifactProvenance =
        serde_json::from_slice(&fs::read(&provenance_path)?)?;
    let boot = publish_x86_64_linux_boot_image(
        &kernel,
        &provenance,
        &initial_user_image,
        &static_user_image,
        destination,
    )?;
    println!("{}", boot.image.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use topal_compiler::{
        LlvmTools, X86_BOOT_IMAGE_FILE, X86_BOOT_PROVENANCE_FILE, X86_INITIAL_BOOT_RESOURCE_FILE,
        publish_x86_64_systems_artifact_with_static_user_image,
        qualified_x86_64_static_user_image_fixture,
    };
    use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

    use super::*;

    static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn packages_an_independently_published_systems_artifact() {
        // TOPAL-COMP-SYSTEMS-ARTIFACT-001, TOPAL-COMP-SYSTEMS-TEST-001.
        let parent = env::temp_dir().join(format!(
            "topal-toolchain-gate-builder-{}-{}",
            std::process::id(),
            NEXT_TEST.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&parent).unwrap();
        let program = analyze_systems_for_compiler(
            include_str!("../../../kernel/arch/x86_64/toolchain-gate.t"),
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let tools = LlvmTools::discover(None).unwrap();
        let linked = parent.join("linked");
        publish_x86_64_systems_artifact_with_static_user_image(
            &program,
            &tools,
            &qualified_x86_64_static_user_image_fixture(),
            &linked,
        )
        .unwrap();
        let boot = parent.join("boot");
        publish(&linked, &boot).unwrap();
        assert!(boot.join(X86_BOOT_IMAGE_FILE).is_file());
        assert!(boot.join(X86_BOOT_PROVENANCE_FILE).is_file());
        assert!(boot.join(X86_INITIAL_BOOT_RESOURCE_FILE).is_file());
        let initial_user_image = linked.join(SYSTEMS_INITIAL_USER_IMAGE_FILE);
        let mut tampered = fs::read(&initial_user_image).unwrap();
        tampered[0] ^= 0xff;
        fs::write(&initial_user_image, &tampered).unwrap();
        let rejected = parent.join("tampered-boot");
        assert!(publish(&linked, &rejected).is_err());
        assert!(!rejected.exists());
        tampered[0] ^= 0xff;
        fs::write(&initial_user_image, tampered).unwrap();
        let static_user_image = linked.join(SYSTEMS_STATIC_USER_IMAGE_FILE);
        let mut tampered = fs::read(&static_user_image).unwrap();
        tampered[0] ^= 0xff;
        fs::write(&static_user_image, tampered).unwrap();
        let rejected = parent.join("tampered-static-boot");
        assert!(publish(&linked, &rejected).is_err());
        assert!(!rejected.exists());
        fs::remove_dir_all(parent).unwrap();
    }
}
