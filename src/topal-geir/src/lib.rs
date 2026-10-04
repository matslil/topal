//! Canonical, validated Generic Export Intermediate Representation (GEIR).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use sha2::{Digest, Sha256};
use topal_semantics::LanguageVersion;
use topal_source::is_nfc;

include!("lib/model_validation_and_encoding.rs");
include!("lib/decoding_and_instantiation.rs");
#[cfg(test)]
mod tests {
    include!("lib/tests/identity.rs");
}
