//! Functional conformance tests for TOPAL-SYN-SOURCE-001,
//! TOPAL-SYN-NUM-001, TOPAL-SYN-GRAMMAR-001, TOPAL-SYN-BIND-001,
//! TOPAL-NUM-LITERAL-001, TOPAL-NUM-ADD-001, and TOPAL-INTP-MODE-001 through
//! TOPAL-INTP-MODE-003.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

include!("cli/run.rs");
include!("cli/every_mode_returns_from_direct_named_constraint_arguments.rs");
include!("cli/all_modes_execute_exact_rational_natural_exponentiation.rs");
include!("cli/closed_invalid_list_boundary_has_source_help.rs");
include!("cli/every_mode_preserves_character_list_values.rs");
