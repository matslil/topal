use super::*;

const SOURCE: &str = "use language (version is v0.1, features is (systems))
copy-byte is fn ((region : KernelByteRegion, source : Nat, destination : Nat)) -> Boolean
  region byte load (offset-bytes is source)
    Some value then region byte store (offset-bytes is destination, value is value)
    None then false
pub apply is fn (region : KernelByteRegion) -> Unit
  observed is region byte count
  selected : Optional KernelByteRegion is region byte window (offset-bytes is 4, byte-count is 4, rights is read-write)
  selected
    Some window then {
      copied : Boolean is copy-byte (window, 0, 1)
      ()
    }
    None then ()
";

#[test]
fn region_entry_uses_declared_inputs_without_an_executable_source_client() {
    // TOPAL-SYSTEMS-REGION-SOURCE-ENTRY-001, -MEMBERS-001, -CLOSURE-001.
    let program = analyze_kernel_region_library(SOURCE, &[], "apply").unwrap();
    assert_eq!(program.primary_source_end, SOURCE.len());
    assert_eq!(program.source.slice(Span::new(0, SOURCE.len())), SOURCE);
    assert!(program.main.statements.is_empty());
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Unit
    ));
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "apply")
        .unwrap();
    assert_eq!(entry.parameters.len(), 1);
    assert_eq!(
        entry.parameters[0].value_type,
        CompilerType::KernelByteRegion
    );
    assert_eq!(entry.result_type, CompilerType::Unit);
    let effects = kernel_region_function_effects(&program).unwrap();
    assert_eq!(effects.len(), 2);
    assert!(effects.iter().all(|effect| effect.read && effect.write));
}

#[test]
fn systems_vocabulary_is_scoped_to_qualified_source_units() {
    // TOPAL-SYSTEMS-REGION-SOURCE-ENTRY-001.
    assert!(analyze_for_compiler(SOURCE).is_err());
    assert!(
        analyze_kernel_region_library(&SOURCE.replace(", features is (systems)", ""), &[], "apply")
            .is_err()
    );
    assert!(analyze_kernel_region_library(SOURCE, &[], "copy-byte").is_err());
    let wrong = SOURCE.replace("pub apply", "apply");
    assert!(analyze_kernel_region_library(&wrong, &[], "apply").is_err());
}

#[test]
fn region_effects_propagate_through_imported_ordinary_helpers() {
    // TOPAL-SYSTEMS-REGION-SOURCE-CLOSURE-001.
    let source = "use language (version is v0.1, features is (systems))
use library support (version is v0.1)
pub apply is fn (region : KernelByteRegion) -> Unit
  copied is support bytes copy-byte region
  ()
";
    let helper = "use language (version is v0.1, features is (systems))
pub copy-byte is fn (region : KernelByteRegion) -> Boolean
  region byte load (offset-bytes is 0)
    Some value then region byte store (offset-bytes is 1, value is value)
    None then false
";
    let module = CompilerSourceModule {
        identity: vec!["support".into(), "bytes".into()],
        source_name: "support/bytes.t".into(),
        source: helper.into(),
    };
    let program =
        analyze_kernel_region_library(source, std::slice::from_ref(&module), "apply").unwrap();
    assert!(
        program
            .functions
            .iter()
            .any(|f| f.module_identity.is_some())
    );
    assert!(
        kernel_region_function_effects(&program)
            .unwrap()
            .iter()
            .all(|effect| effect.read && effect.write)
    );
    let mut missing_feature = module.clone();
    missing_feature.source = helper.replace(", features is (systems)", "");
    assert!(analyze_kernel_region_library(source, &[missing_feature], "apply").is_err());
    let mut initialized = module;
    initialized.source.push_str("initialized is 1\n");
    assert!(analyze_kernel_region_library(source, &[initialized], "apply").is_err());
}

#[test]
fn false_noeffects_and_resource_escapes_are_rejected() {
    // TOPAL-SYSTEMS-REGION-SOURCE-CLOSURE-001.
    for signature in ["-> Unit", "-> Boolean"] {
        let source = SOURCE.replace(signature, &format!("{signature} : Effects ()"));
        let error = analyze_kernel_region_library(&source, &[], "apply").unwrap_err();
        assert!(error.message.contains("byte effects exceed"), "{error:?}");
    }
    let escape = "use language (version is v0.1, features is (systems))
identity is fn (region : KernelByteRegion) -> KernelByteRegion
  region
pub apply is fn (region : KernelByteRegion) -> Unit
  escaped is identity region
  ()
";
    assert!(analyze_kernel_region_library(escape, &[], "apply").is_err());
    assert!(
        analyze_kernel_region_library(
            &SOURCE.replace(
                "observed is region byte count",
                "forged is KernelByteRegion 0"
            ),
            &[],
            "apply"
        )
        .is_err()
    );
}

#[test]
fn metadata_observation_can_remain_noeffects() {
    // TOPAL-SYSTEMS-REGION-SOURCE-CLOSURE-001.
    let source = "use language (version is v0.1, features is (systems))
extent is fn (region : KernelByteRegion) -> Nat : Effects ()
  region byte count
pub apply is fn (region : KernelByteRegion) -> Unit : Effects ()
  count is extent region
  ()
";
    let program = analyze_kernel_region_library(source, &[], "apply").unwrap();
    assert!(
        kernel_region_function_effects(&program)
            .unwrap()
            .iter()
            .all(|effect| !effect.read && !effect.write)
    );
}

#[test]
fn region_members_reject_incomplete_facts_and_non_nat_values() {
    // TOPAL-SYSTEMS-REGION-SOURCE-MEMBERS-001.
    for replacement in [
        "region byte window (offset-bytes is 4, byte-count is 4, rights is foreign)",
        "region byte window (offset-bytes is 4, byte-count is 4)",
        "region byte window (offset-bytes is 4, byte-count is -1, rights is read-write)",
        "region byte window (offset-bytes is true, byte-count is 4, rights is read-write)",
        "region byte window (offset-bytes is 4, byte-count is 4, rights is read-write, extra is 0)",
    ] {
        let source = SOURCE.replace(
            "region byte window (offset-bytes is 4, byte-count is 4, rights is read-write)",
            replacement,
        );
        assert!(
            analyze_kernel_region_library(&source, &[], "apply").is_err(),
            "{replacement}"
        );
    }
}

#[test]
fn isolated_module_exports_use_symbolic_declared_inputs_and_no_caller_bindings() {
    // TOPAL-SYSTEMS-REGION-SOURCE-ENTRY-001, -CLOSURE-001.
    let text = include_str!("../../../../linux-kernel/kernel/support/bytes.t");
    let module = CompilerSourceModule {
        identity: vec!["support".into(), "bytes".into()],
        source_name: "support/bytes.t".into(),
        source: text.into(),
    };
    let export = CompilerKernelRegionModuleExport {
        module: module.identity.clone(),
        name: "copy-byte".into(),
        parameters: vec![
            CompilerType::KernelByteRegion,
            CompilerType::Nat,
            CompilerType::Nat,
        ],
        result: CompilerType::Boolean,
    };
    let program = analyze_kernel_region_module_exports(
        std::slice::from_ref(&module),
        std::slice::from_ref(&export),
    )
    .unwrap();
    assert!(program.main.statements.is_empty());
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Unit
    ));
    assert_eq!(
        program.functions[0]
            .parameters
            .iter()
            .map(|parameter| &parameter.value_type)
            .collect::<Vec<_>>(),
        export.parameters.iter().collect::<Vec<_>>()
    );
    assert!(
        program.functions[0]
            .parameters
            .iter()
            .all(|parameter| parameter.int_range.is_none())
    );
    let mut wrong = export.clone();
    wrong.parameters[1] = CompilerType::Boolean;
    assert!(analyze_kernel_region_module_exports(std::slice::from_ref(&module), &[wrong]).is_err());
    let mut private = module.clone();
    private.source = text.replace("pub copy-byte", "copy-byte");
    assert!(analyze_kernel_region_module_exports(&[private], &[export]).is_err());
}

#[test]
fn action_selection_preserves_declared_nat_inputs_and_persistent_owner_effects() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001; this supplies no live owner grant.
    let text = include_str!("../../../../linux-kernel/kernel/policy/file-position.t");
    let helper = include_str!("../../../../linux-kernel/kernel/support/word-bytes.t");
    let module = CompilerSourceModule {
        identity: vec!["support".into(), "word-bytes".into()],
        source_name: "support/word-bytes.t".into(),
        source: helper.into(),
    };
    let program = analyze_kernel_region_action_library(text, &[module], "advance").unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "advance")
        .unwrap();
    assert_eq!(
        entry
            .parameters
            .iter()
            .map(|p| p.value_type.clone())
            .collect::<Vec<_>>(),
        [
            CompilerType::KernelByteRegion,
            CompilerType::Nat,
            CompilerType::Nat
        ]
    );
    assert_eq!(entry.result_type, CompilerType::Boolean);
    assert!(program.main.statements.is_empty());
    assert!(matches!(
        program.main.result.kind,
        CompilerExpressionKind::Unit
    ));
    assert_eq!(program.primary_source_end, text.len());
    assert!(
        kernel_region_function_effects(&program)
            .unwrap()
            .iter()
            .find(|e| e.symbol == entry.symbol)
            .unwrap()
            .write
    );
    assert!(analyze_for_compiler(text).is_err());
    assert!(analyze_kernel_region_library(text, &[], "advance").is_err());
}

#[test]
fn action_context_rejects_wrong_shapes_private_entries_and_false_effects() {
    // TOPAL-SYSTEMS-REGION-ACTION-SOURCE-001.
    let source = "use language (version is v0.1, features is (systems))\npub apply is fn ((region : KernelByteRegion, a : Nat, b : Nat)) -> Boolean\n  region byte store (offset-bytes is a, value is b)\n";
    analyze_kernel_region_action_library(source, &[], "apply").unwrap();
    for (old, new) in [
        ("a : Nat", "a : Int"),
        ("b : Nat", "b : Boolean"),
        ("pub apply", "apply"),
        ("-> Boolean", "-> Boolean : Effects ()"),
        (", features is (systems)", ""),
    ] {
        assert!(
            analyze_kernel_region_action_library(&source.replace(old, new), &[], "apply").is_err(),
            "{new}"
        );
    }
    assert!(analyze_kernel_region_action_library(SOURCE, &[], "apply").is_err());
}

#[test]
fn framed_selection_matches_exact_declared_count_without_changing_core_operands() {
    // TOPAL-SYSTEMS-REGION-FRAME-SOURCE-001.
    for count in [0, 1, 2, 3, 16] {
        let mut parameters = "region : KernelByteRegion".to_owned();
        for i in 0..count {
            parameters.push_str(&format!(", fact{i} : Nat"));
        }
        let source = format!(
            "use language (version is v0.1, features is (systems))\npub apply is fn (({parameters})) -> Boolean\n  true\n"
        );
        let program =
            analyze_kernel_region_framed_action_library(&source, &[], "apply", count).unwrap();
        let entry = program
            .functions
            .iter()
            .find(|f| f.source_name == "apply")
            .unwrap();
        assert_eq!(entry.parameters.len(), usize::from(count) + 1);
        assert_eq!(
            entry.parameters[0].value_type,
            CompilerType::KernelByteRegion
        );
        assert!(
            entry.parameters[1..]
                .iter()
                .all(|p| p.value_type == CompilerType::Nat)
        );
        assert_eq!(entry.result_type, CompilerType::Boolean);
        for expected in [17, count + 1] {
            assert!(
                analyze_kernel_region_framed_action_library(&source, &[], "apply", expected)
                    .is_err()
            );
        }
        for (old, new) in [
            ("pub apply", "apply"),
            ("-> Boolean", "-> Nat"),
            (", features is (systems)", ""),
            ("region : KernelByteRegion", "region : Nat"),
        ] {
            assert!(
                analyze_kernel_region_framed_action_library(
                    &source.replace(old, new),
                    &[],
                    "apply",
                    count
                )
                .is_err()
            );
        }
        if count > 0 {
            assert!(
                analyze_kernel_region_framed_action_library(
                    &source.replace("fact0 : Nat", "fact0 : Int"),
                    &[],
                    "apply",
                    count
                )
                .is_err()
            );
        }
        assert_eq!(
            analyze_kernel_region_action_library(&source, &[], "apply").is_ok(),
            count == 2
        );
    }
    let flat = "use language (version is v0.1, features is (systems))\npub apply is fn (region : KernelByteRegion, a : Nat, b : Nat) -> Boolean\n  true\n";
    assert!(analyze_kernel_region_framed_action_library(flat, &[], "apply", 2).is_err());
}
