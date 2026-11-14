//! Private selected adapter storage; no processor startup or public authority.
use super::*;

pub(crate) const SETUP: usize = 8192;
pub(crate) const REVISION: &str =
    "topal.boot-adapter.linux-x86-protocol-2.15-q35-startup-storage/1";
pub(crate) const OFFSET: usize = 4096;
pub(crate) const SEED: u64 = u64::from_le_bytes(*b"TOPAL-ST");
pub(crate) const STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;
pub(crate) const MASK: u64 = 0xa55a_3cc3_f00f_9669;

pub(crate) fn page() -> Vec<u8> {
    (0..512u64)
        .flat_map(|i| SEED.wrapping_add(i.wrapping_mul(STRIDE)).to_le_bytes())
        .collect()
}

pub(crate) fn fixture(kernel: &[u8]) -> Result<Vec<u8>, CompileError> {
    let original = unpublished_component_fixture(kernel)?;
    let mut image = original[..SETUP_BYTES].to_vec();
    image.resize(SETUP, 0);
    image[0x1f1] = 15;
    image[OFFSET..SETUP].copy_from_slice(&page());
    image.extend_from_slice(&original[SETUP_BYTES..]);
    Ok(image)
}

pub(crate) fn contract() -> serde_json::Value {
    serde_json::json!({"schema":"topal.boot-startup-storage.x86_64/1",
        "setup_bytes":SETUP,"setup_sectors":15,"page_offset":OFFSET,"page_bytes":4096,
        "seed":SEED,"stride":STRIDE,"probe_mask":MASK,"page_sha256":sha256(&page()),
        "owner":"boot-adapter-setup-instance","lifetime":"kernel","reclamation":false,
        "processor_startup":false,"participant_arrival":false})
}

pub(crate) fn llvm() -> String {
    include_str!("runtime/kernel_boot_startup.ll")
        .replace("{{STARTUP_SEED}}", &SEED.to_string())
        .replace("{{STARTUP_STRIDE}}", &STRIDE.to_string())
        .replace("{{STARTUP_MASK}}", &MASK.to_string())
}

#[cfg(test)]
#[path = "systems_boot_startup_tests.rs"]
mod tests;
