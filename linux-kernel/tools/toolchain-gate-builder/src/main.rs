//! Lab-only publisher for the initial Topal kernel toolchain-gate image.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use topal_compiler::{LlvmTools, publish_x86_64_linux_boot_image, publish_x86_64_systems_artifact};
use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

const SOURCE: &str = include_str!("../../../kernel/arch/x86_64/toolchain-gate.t");

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
    let destination = arguments
        .next()
        .ok_or_else(|| format!("usage: {} DESTINATION", Path::new(&program_name).display()))?;
    if arguments.next().is_some() {
        return Err(format!("usage: {} DESTINATION", Path::new(&program_name).display()).into());
    }
    publish(&PathBuf::from(destination))
}

fn publish(destination: &Path) -> Result<(), Box<dyn Error>> {
    if destination.exists() {
        return Err(format!(
            "toolchain-gate destination already exists: {}",
            destination.display()
        )
        .into());
    }
    fs::create_dir_all(destination)?;
    let result = (|| {
        let program = analyze_systems_for_compiler(
            SOURCE,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )?;
        let tools = LlvmTools::discover(None)?;
        let linked =
            publish_x86_64_systems_artifact(&program, &tools, &destination.join("linked"))?;
        let kernel = fs::read(&linked.kernel)?;
        let boot =
            publish_x86_64_linux_boot_image(&kernel, &linked.record, &destination.join("boot"))?;
        println!("{}", boot.image.display());
        Ok::<(), Box<dyn Error>>(())
    })();
    if result.is_err() {
        fs::remove_dir_all(destination)?;
    }
    result
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use topal_compiler::{X86_BOOT_IMAGE_FILE, X86_BOOT_PROVENANCE_FILE};

    use super::*;

    static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn publishes_the_lab_image_from_the_checked_kernel_source() {
        // TOPAL-COMP-SYSTEMS-ARTIFACT-001, TOPAL-COMP-SYSTEMS-TEST-001.
        let destination = env::temp_dir().join(format!(
            "topal-toolchain-gate-builder-{}-{}",
            std::process::id(),
            NEXT_TEST.fetch_add(1, Ordering::Relaxed)
        ));
        publish(&destination).unwrap();
        assert!(destination.join("linked/kernel.elf").is_file());
        assert!(destination.join("boot").join(X86_BOOT_IMAGE_FILE).is_file());
        assert!(
            destination
                .join("boot")
                .join(X86_BOOT_PROVENANCE_FILE)
                .is_file()
        );
        fs::remove_dir_all(destination).unwrap();
    }
}
