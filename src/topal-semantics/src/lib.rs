//! Shared, deterministic semantic identities for every Topal source tool.

mod architecture;
mod assurance;
pub mod introspection;
mod layout_and_information;
mod portable_runtime;
mod systems;
mod systems_boot_memory;
mod systems_critical;
mod systems_frames;
mod systems_mapping;
mod systems_storage;
mod systems_translation;
pub mod tracing;

pub use architecture::*;
pub use assurance::*;
pub use layout_and_information::*;
pub use portable_runtime::*;
pub use systems::*;
pub use systems_boot_memory::*;
pub use systems_critical::*;
pub use systems_frames::*;
pub use systems_mapping::*;
pub use systems_storage::*;
pub use systems_translation::*;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

include!("lib/semantic_model.rs");
include!("lib/resource_tracker.rs");
#[cfg(test)]
mod tests {
    include!("lib/tests/declaration.rs");
}
