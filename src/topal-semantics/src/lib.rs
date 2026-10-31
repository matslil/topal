//! Shared, deterministic semantic identities for every Topal source tool.

mod architecture;
mod assurance;
pub mod introspection;
mod layout_and_information;
mod portable_runtime;
mod systems;
mod systems_atomic;
mod systems_boot_memory;
mod systems_clock;
mod systems_context;
mod systems_critical;
mod systems_deadline;
mod systems_frames;
mod systems_interrupt;
mod systems_library;
mod systems_mapping;
mod systems_numeric_runtime;
mod systems_storage;
mod systems_translation;
mod systems_user;
pub mod tracing;

pub use architecture::*;
pub use assurance::*;
pub use layout_and_information::*;
pub use portable_runtime::*;
pub use systems::*;
pub use systems_atomic::*;
pub use systems_boot_memory::*;
pub use systems_clock::*;
pub use systems_context::*;
pub use systems_critical::*;
pub use systems_deadline::*;
pub use systems_frames::*;
pub use systems_interrupt::*;
pub use systems_library::*;
pub use systems_mapping::*;
pub use systems_numeric_runtime::*;
pub use systems_storage::*;
pub use systems_translation::*;
pub use systems_user::*;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

include!("lib/semantic_model.rs");
include!("lib/resource_tracker.rs");
#[cfg(test)]
mod tests {
    include!("lib/tests/declaration.rs");
}
mod systems_children;
pub use systems_children::*;
