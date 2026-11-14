use super::*;
use topal_semantics::{RegionLibraryByteOrder as Order, RegionLibraryUnsignedFormat as Format};

fn source(body: &str) -> String {
    format!(
        "use language (version is v0.1, features is (systems))\npub apply is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean\n  {body}\n"
    )
}

#[test]
fn unsigned_formats_preserve_static_encoding_and_dynamic_fact_order() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-001, -QUALIFY-001.
    for width in 1..=8 {
        for (spelling, order) in [
            ("little-endian", Order::LittleEndian),
            ("big-endian", Order::BigEndian),
        ] {
            let text = source(&format!(
                "region unsigned store (value is value, byte-order is {spelling}, offset-bytes is offset, byte-count is {width})"
            ));
            let program = analyze_kernel_region_action_library(&text, &[], "apply").unwrap();
            let entry = program
                .functions
                .iter()
                .find(|f| f.source_name == "apply")
                .unwrap();
            let CompilerExpressionKind::KernelRegion {
                operation,
                arguments,
                argument_roles,
                ..
            } = &entry.body.result.kind
            else {
                panic!("missing primitive")
            };
            assert_eq!(
                *operation,
                CompilerKernelRegionOperation::UnsignedStore(Format::new(width, order).unwrap())
            );
            assert_eq!(argument_roles, &[1, 0]);
            assert_eq!(arguments.len(), 2);
            assert!(arguments.iter().all(|v| v.value_type == CompilerType::Nat));
            assert!(kernel_region_unsigned_runtime_required(&program).unwrap());
            let effects = kernel_region_function_effects(&program).unwrap();
            assert!(effects.iter().all(|e| !e.read && e.write));
        }
    }
}

#[test]
fn unsigned_format_facts_reject_missing_duplicate_dynamic_and_out_of_range_values() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-001.
    let valid = "region unsigned store (offset-bytes is offset, byte-count is 8, byte-order is little-endian, value is value)";
    for (old, new) in [
        ("byte-count is 8", "byte-count is 0"),
        ("byte-count is 8", "byte-count is 9"),
        ("byte-count is 8", "byte-count is 257"),
        ("byte-count is 8", "byte-count is 18446744073709551624"),
        ("byte-count is 8", "byte-count is -1"),
        ("byte-count is 8", "byte-count is offset"),
        ("byte-count is 8", "byte-count is (4 + 4)"),
        ("byte-order is little-endian", "byte-order is native-endian"),
        ("byte-order is little-endian", "byte-order is offset"),
        ("byte-order is little-endian", "byte-order is 0"),
        ("byte-count is 8, ", ""),
        ("byte-count is 8", "byte-count is 8, byte-count is 8"),
        ("byte-count is 8", "byte-count is 8, extra is 0"),
        ("value is value", "value is true"),
        ("value is value", "value is -1"),
        ("offset-bytes is offset", "offset-bytes is -1"),
    ] {
        let changed = source(&valid.replace(old, new));
        assert!(
            analyze_kernel_region_action_library(&changed, &[], "apply").is_err(),
            "{changed}"
        );
    }
}

#[test]
fn unsigned_load_is_optional_nat_with_read_effect_through_ordinary_helpers() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-001, -QUALIFY-001.
    let text = "use language (version is v0.1, features is (systems))\nread-field is fn (region : KernelByteRegion) -> Optional Nat\n  region unsigned load (offset-bytes is 0, byte-count is 8, byte-order is big-endian)\npub apply is fn (region : KernelByteRegion) -> Unit\n  result is read-field region\n  ()\n";
    let program = analyze_kernel_region_library(text, &[], "apply").unwrap();
    assert!(
        kernel_region_function_effects(&program)
            .unwrap()
            .iter()
            .all(|e| e.read && !e.write)
    );
    assert!(kernel_region_unsigned_runtime_required(&program).unwrap());
    for marker in ["-> Optional Nat", "-> Unit"] {
        let changed = text.replace(marker, &format!("{marker} : Effects ()"));
        assert!(analyze_kernel_region_library(&changed, &[], "apply").is_err());
    }
    assert!(analyze_for_compiler(text).is_err());
    assert!(
        analyze_kernel_region_library(&text.replace(", features is (systems)", ""), &[], "apply")
            .is_err()
    );
}

#[test]
fn unsigned_facts_remain_arbitrary_nat_and_never_require_a_machine_word_type() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-001.
    for value in [
        "9223372036854775808",
        "18446744073709551615",
        "18446744073709551616",
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
    ] {
        let text = source(&format!(
            "region unsigned store (offset-bytes is {value}, byte-count is 8, byte-order is little-endian, value is {value})"
        ));
        analyze_kernel_region_action_library(&text, &[], "apply").unwrap();
    }
    let text = source("region byte store (offset-bytes is offset, value is value)");
    let program = analyze_kernel_region_action_library(&text, &[], "apply").unwrap();
    assert!(!kernel_region_unsigned_runtime_required(&program).unwrap());
}

#[test]
fn unsigned_runtime_selection_covers_lowered_unselected_branches() {
    // TOPAL-SYSTEMS-REGION-UNSIGNED-QUALIFY-001.
    let text = source(
        "false\n    true then region unsigned store (offset-bytes is offset, byte-count is 3, byte-order is big-endian, value is value)\n    false then true",
    );
    let program = analyze_kernel_region_action_library(&text, &[], "apply").unwrap();
    assert!(kernel_region_unsigned_runtime_required(&program).unwrap());
    assert!(
        kernel_region_function_effects(&program)
            .unwrap()
            .iter()
            .all(|e| e.write)
    );
    assert!(
        analyze_kernel_region_action_library(
            &text.replace("-> Boolean", "-> Boolean : Effects ()"),
            &[],
            "apply"
        )
        .is_err()
    );
}
