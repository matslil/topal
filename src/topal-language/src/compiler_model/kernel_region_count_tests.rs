use super::*;

const ROOT: &str = "use language (version is v0.1, features is (systems))
extent is fn (region : KernelByteRegion) -> Nat : Effects ()
  region byte count
pub inspect is fn ((region : KernelByteRegion, remaining : Nat, expected : Nat, state : Nat)) -> Boolean : Decreases remaining
  remaining
    <= 0 then (extent region) = expected
    otherwise inspect(region, remaining - 1, expected, state)
";

#[test]
fn tail_terminal_rules_retain_all_checked_actions_and_the_original_nat_guard() {
    // TOPAL-SYSTEMS-TAIL-PLAN-001: extra terminal actions cannot recur. The
    // complete original decision remains in checked IR, not the proof slice.
    let text = ROOT.replace(
        "    otherwise inspect",
        "    <= state then (extent region) = expected\n    otherwise inspect",
    );
    let program = analyze_kernel_region_tail_source(&text, &[], "inspect", 3).unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "inspect")
        .unwrap();
    assert!(
        matches!(&entry.body.result.kind,CompilerExpressionKind::OrderedComparisonDecision{rules,..} if rules.len()==2)
    );
    assert!(analyze_kernel_region_framed_action_library(&text, &[], "inspect", 3).is_err());
    let recursive = text.replace(
        "<= state then (extent region) = expected",
        "<= state then inspect(region, remaining, expected, state)",
    );
    assert!(analyze_kernel_region_tail_source(&recursive, &[], "inspect", 3).is_err());
    let missing = text.replace("<= 0 then (extent region) = expected\n", "");
    assert!(analyze_kernel_region_tail_source(&missing, &[], "inspect", 3).is_err());
}

fn assert_count_bounds(program: &CompilerProgram, upper: Option<u64>) {
    let mut count = 0;
    for function in &program.functions {
        let expressions = function
            .body
            .statements
            .iter()
            .filter_map(|statement| {
                if let CompilerStatement::Binding(binding) = statement {
                    Some(&binding.value)
                } else {
                    None
                }
            })
            .chain(std::iter::once(&function.body.result));
        for expression in expressions {
            if matches!(
                expression.kind,
                CompilerExpressionKind::KernelRegion {
                    operation: CompilerKernelRegionOperation::Count,
                    ..
                }
            ) {
                assert_eq!(expression.value_type, CompilerType::Nat);
                if let Some(upper) = upper {
                    let range = expression.int_range.as_ref().unwrap();
                    assert_eq!(range.lower, BigInt::from(0));
                    assert_eq!(range.upper, BigInt::from(upper));
                } else {
                    assert!(expression.int_range.is_none());
                }
                count += 1;
            }
        }
    }
    assert!(count > 0, "fixture must inspect a real Count expression");
}

#[test]
fn tail_count_does_not_inherit_the_legacy_owner_or_arena_cap() {
    // TOPAL-SYSTEMS-TAIL-BOUNDS-001, TOPAL-SYSTEMS-REGION-SOURCE-MEMBERS-001.
    let program = analyze_kernel_region_tail_source(ROOT, &[], "inspect", 3).unwrap();
    assert_count_bounds(&program, None);
    let legacy = "use language (version is v0.1, features is (systems))\npub inspect is fn (region : KernelByteRegion) -> Unit\n  extent is region byte count\n  ()\n";
    let program = analyze_kernel_region_library(legacy, &[], "inspect").unwrap();
    assert_count_bounds(&program, Some(65536));
}

#[test]
fn tail_imported_and_isolated_modules_use_the_same_uncapped_count_meaning() {
    // TOPAL-SYSTEMS-TAIL-BOUNDS-001: neither caller nor module invents an extent.
    let module=CompilerSourceModule {identity:vec!["support".into(),"extent".into()],
        source_name:"support/extent.t".into(),source:"use language (version is v0.1, features is (systems))\npub count is fn (region : KernelByteRegion) -> Nat : Effects ()\n  region byte count\n".into()};
    let root = ROOT
        .replace(
            "extent is fn (region : KernelByteRegion) -> Nat : Effects ()\n  region byte count\n",
            "use library support (version is v0.1)\n",
        )
        .replace("(extent region)", "(support extent count region)");
    let program =
        analyze_kernel_region_tail_source(&root, std::slice::from_ref(&module), "inspect", 3)
            .unwrap();
    assert_count_bounds(&program, None);
    let export = CompilerKernelRegionModuleExport {
        module: module.identity.clone(),
        name: "count".into(),
        parameters: vec![CompilerType::KernelByteRegion],
        result: CompilerType::Nat,
    };
    let program = analyze_kernel_region_tail_module_exports(
        std::slice::from_ref(&module),
        std::slice::from_ref(&export),
    )
    .unwrap();
    assert_count_bounds(&program, None);
    let legacy = analyze_kernel_region_module_exports(
        std::slice::from_ref(&module),
        std::slice::from_ref(&export),
    )
    .unwrap();
    assert_count_bounds(&legacy, Some(65536));
    let mut private = module.clone();
    private.source = private.source.replace("pub count", "count");
    assert!(
        analyze_kernel_region_tail_module_exports(&[private], std::slice::from_ref(&export))
            .is_err()
    );
    let mut initialized = module;
    initialized.source.push_str("initialized is 1\n");
    assert!(analyze_kernel_region_tail_module_exports(&[initialized], &[export]).is_err());
}

#[test]
fn a_legacy_count_cap_cannot_supply_tail_nat_subtraction_evidence() {
    // TOPAL-SYSTEMS-TAIL-BOUNDS-001: the old cap would prove a negative Nat safe
    // for an actual larger owner. Ordinary Int subtraction remains available.
    let helper = "distance is fn (region : KernelByteRegion) -> Nat\n  extent is region byte count\n  65536 - extent\n";
    let root = ROOT
        .replace(
            "extent is fn (region : KernelByteRegion) -> Nat : Effects ()\n  region byte count\n",
            helper,
        )
        .replace("(extent region)", "(distance region)");
    assert!(analyze_kernel_region_tail_source(&root, &[], "inspect", 3).is_err());
    let legacy = format!(
        "use language (version is v0.1, features is (systems))\n{helper}pub inspect is fn (region : KernelByteRegion) -> Unit\n  extent is distance region\n  ()\n"
    );
    assert!(analyze_kernel_region_library(&legacy, &[], "inspect").is_ok());
    let integer = root.replace(helper, &helper.replace("-> Nat", "-> Int"));
    analyze_kernel_region_tail_source(&integer, &[], "inspect", 3).unwrap();
    let module = CompilerSourceModule {
        identity: vec!["support".into(), "extent".into()],
        source_name: "support/extent.t".into(),
        source: format!("use language (version is v0.1, features is (systems))\npub {helper}"),
    };
    let export = CompilerKernelRegionModuleExport {
        module: module.identity.clone(),
        name: "distance".into(),
        parameters: vec![CompilerType::KernelByteRegion],
        result: CompilerType::Nat,
    };
    assert!(
        analyze_kernel_region_tail_module_exports(
            std::slice::from_ref(&module),
            std::slice::from_ref(&export)
        )
        .is_err()
    );
    assert!(analyze_kernel_region_module_exports(&[module], &[export]).is_ok());
}

#[test]
fn natural_tail_selection_retains_an_exact_source_only_signature() {
    // TOPAL-SYSTEMS-TAIL-NAT-OWNER-001. This supplies no live owner or word cast.
    let source = "use language (version is v0.1, features is (systems))\npub query is fn ((region : KernelByteRegion, remaining : Nat, offset : Nat, state : Nat)) -> Nat : Decreases remaining\n  remaining\n    <= 0 then offset\n    otherwise query(region, remaining - 1, offset + 1, state)\n";
    let program = analyze_kernel_region_tail_nat_source(source, &[], "query", 3).unwrap();
    let entry = program
        .functions
        .iter()
        .find(|f| f.source_name == "query")
        .unwrap();
    assert_eq!(entry.result_type, CompilerType::Nat);
    assert_eq!(
        entry
            .parameters
            .iter()
            .map(|p| p.value_type.clone())
            .collect::<Vec<_>>(),
        vec![
            CompilerType::KernelByteRegion,
            CompilerType::Nat,
            CompilerType::Nat,
            CompilerType::Nat
        ]
    );
    assert!(entry.parameters.iter().all(|p| p.int_range.is_none()));
    assert!(program.main.statements.is_empty());
    assert!(analyze_kernel_region_tail_source(source, &[], "query", 3).is_err());
    for input in [
        source.replace("pub query", "query"),
        source.replace("-> Nat", "-> Int"),
        source.replace(" : Decreases remaining", ""),
    ] {
        assert!(analyze_kernel_region_tail_nat_source(&input, &[], "query", 3).is_err());
    }
    assert!(analyze_kernel_region_tail_nat_source(source, &[], "query", 2).is_err());
    assert!(analyze_kernel_region_tail_nat_source(source, &[], "query", 17).is_err());
}
