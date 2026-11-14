// TOPAL-SYSTEMS-REGION-LIBRARY-BINDING-001 and
// TOPAL-SYSTEMS-NAMESPACE-ACTION-001; systems code sharing preserves core meaning.
use super::*;

fn source(repetitions: usize) -> String {
    let mut text = "use language (version is v0.1, features is (systems))\nwrite-one is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean\n  region byte store (offset-bytes is offset, value is value)\npub apply is fn ((region : KernelByteRegion, first : Nat, second : Nat)) -> Boolean\n".to_owned();
    for offset in 0..repetitions {
        text.push_str(&format!(
            "  written-{offset} is write-one (region, {offset}, second)\n"
        ));
    }
    text.push_str("  true\n");
    text
}

fn expanded(text: &str) -> CompilerProgram {
    let selection = KernelRegionSelection::Entry(
        "apply",
        CompilerKernelRegionProfile::FramedAction { scalar_count: 2 },
    );
    analyze_for_compiler_options(text, &[], &["apply".into()], Some(&selection)).unwrap()
}

fn calls(program: &CompilerProgram) -> Vec<(String, Vec<CompilerExpression>)> {
    named_calls(program, "apply")
}

fn named_calls(program: &CompilerProgram, name: &str) -> Vec<(String, Vec<CompilerExpression>)> {
    fn normalize(value: &mut CompilerExpression) {
        value.int_range = None;
        if let CompilerExpressionKind::IntToNat(input) = &mut value.kind {
            normalize(input);
        }
    }
    program
        .functions
        .iter()
        .find(|f| f.source_name == name)
        .unwrap()
        .body
        .statements
        .iter()
        .map(|statement| {
            let CompilerStatement::Binding(binding) = statement else {
                panic!("expected source binding")
            };
            let CompilerExpressionKind::Call { symbol, arguments } = &binding.value.kind else {
                panic!("expected actual call")
            };
            let mut arguments = arguments.clone();
            // Analysis-only ranges are intentionally not a runtime observation.
            for argument in &mut arguments {
                normalize(argument);
            }
            (symbol.clone(), arguments)
        })
        .collect()
}

#[test]
fn small_region_closures_retain_every_existing_representation_bit() {
    let mut program = expanded(&source(4));
    let before = program.clone();
    assert!(program.functions.len() <= 64);
    coalesce_kernel_region_helpers(&mut program).unwrap();
    assert_eq!(program, before);
}

#[test]
fn equivalent_checked_helpers_share_code_but_preserve_each_ordered_call_and_value() {
    // TOPAL-SYSTEMS-NAMESPACE-ACTION-001, TOPAL-SYSTEMS-REGION-SOURCE-CLOSURE-001.
    let mut program = expanded(&source(80));
    assert_eq!(program.functions.len(), 81);
    let before = calls(&program);
    assert_eq!(
        before.iter().map(|(s, _)| s).collect::<BTreeSet<_>>().len(),
        80
    );
    coalesce_kernel_region_helpers(&mut program).unwrap();
    assert_eq!(program.functions.len(), 2);
    let after = calls(&program);
    assert_eq!(after.len(), 80);
    assert_eq!(
        after.iter().map(|(s, _)| s).collect::<BTreeSet<_>>().len(),
        1
    );
    for (index, ((_, before), (_, after))) in before.into_iter().zip(after).enumerate() {
        assert_eq!(before, after, "call {index}");
    }
    let effects = kernel_region_function_effects(&program).unwrap();
    assert!(effects.iter().all(|effect| effect.write && !effect.read));
}

#[test]
fn declaration_owner_opcode_and_literal_differences_never_merge() {
    let mut program = expanded(&source(80));
    program.functions[1].module_identity = Some(vec!["different-owner".into()]);
    program.functions[2]
        .source_name
        .push_str("-different-declaration");
    let CompilerExpressionKind::KernelRegion { arguments, .. } =
        &mut program.functions[3].body.result.kind
    else {
        panic!("expected byte store")
    };
    arguments[0].kind = CompilerExpressionKind::Int(BigInt::from(7));
    let CompilerExpressionKind::KernelRegion { operation, .. } =
        &mut program.functions[4].body.result.kind
    else {
        panic!("expected byte store")
    };
    *operation = CompilerKernelRegionOperation::UnsignedStore(
        topal_semantics::RegionLibraryUnsignedFormat::new(
            8,
            topal_semantics::RegionLibraryByteOrder::LittleEndian,
        )
        .unwrap(),
    );
    coalesce_kernel_region_helpers(&mut program).unwrap();
    assert_eq!(program.functions.len(), 6);
    assert_eq!(
        calls(&program)
            .iter()
            .map(|(s, _)| s)
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );
}

#[test]
fn original_foreign_capture_and_unresolved_graph_refusals_precede_reuse() {
    for fault in 0..3 {
        let mut program = expanded(&source(80));
        match fault {
            0 => {
                program.functions[1].foreign = Some(CompilerCFunction {
                    library_identity: "foreign".into(),
                    external_symbol: "external".into(),
                    parameters: vec![],
                    result: CompilerCValue::Void,
                })
            }
            1 => program.functions[1].parameters[0].source_visible = false,
            _ => {
                program.functions[1].body.result.kind = CompilerExpressionKind::Call {
                    symbol: "unresolved".into(),
                    arguments: vec![],
                }
            }
        }
        assert!(
            coalesce_kernel_region_helpers(&mut program).is_err(),
            "fault {fault}"
        );
    }
}

fn tail_source(repetitions: usize) -> String {
    let mut text = source(repetitions).replace(
        "pub apply is fn ((region : KernelByteRegion, first : Nat, second : Nat)) -> Boolean",
        "work is fn ((region : KernelByteRegion, first : Nat, second : Nat)) -> Nat",
    );
    text.truncate(text.len() - "  true\n".len());
    text.push_str("  first\npub traverse is fn ((region : KernelByteRegion, remaining : Nat, unused : Nat, status : Nat)) -> Nat : Decreases remaining\n  remaining\n    <= 0 then status\n    otherwise traverse(region, remaining - 1, unused, work(region, unused, status))\n");
    text
}

fn expanded_tail(text: &str) -> CompilerProgram {
    let selection = KernelRegionSelection::TailSource("traverse", 3, KernelRegionTailResult::Nat);
    analyze_for_compiler_options(text, &[], &["traverse".into()], Some(&selection)).unwrap()
}

#[test]
fn tail_sharing_retains_small_closures_and_the_exact_recursive_entry() {
    // TOPAL-SYSTEMS-TAIL-HELPER-REUSE-001.
    let mut small = expanded_tail(&tail_source(2));
    let before = small.clone();
    coalesce_kernel_region_tail_helpers(&mut small, "traverse").unwrap();
    assert_eq!(small, before);
    let mut large = expanded_tail(&tail_source(80));
    assert!(large.functions.len() > 64);
    let before = named_calls(&large, "work");
    let root = large
        .functions
        .iter()
        .find(|f| f.source_name == "traverse")
        .unwrap()
        .symbol
        .clone();
    coalesce_kernel_region_tail_helpers(&mut large, "traverse").unwrap();
    assert_eq!(large.functions.len(), 3);
    let after = named_calls(&large, "work");
    assert_eq!(before.len(), 80);
    assert_eq!(
        after.iter().map(|(s, _)| s).collect::<BTreeSet<_>>().len(),
        1
    );
    for ((_, before), (_, after)) in before.into_iter().zip(after) {
        assert_eq!(before, after);
    }
    let entry = large
        .functions
        .iter()
        .find(|f| f.source_name == "traverse")
        .unwrap();
    assert_eq!(entry.symbol, root);
    let CompilerExpressionKind::OrderedComparisonDecision { otherwise, .. } =
        &entry.body.result.kind
    else {
        panic!("countdown")
    };
    let CompilerExpressionKind::Call { symbol, arguments } = &otherwise.kind else {
        panic!("self edge")
    };
    assert_eq!(symbol, &root);
    assert_eq!(arguments.len(), 4);
    // Code sharing never qualifies the graph under the old acyclic profile.
    assert!(kernel_region_function_effects(&large).is_err());
}

#[test]
fn tail_original_foreign_capture_unresolved_effect_and_helper_cycles_precede_sharing() {
    // The sole ignored graph edge belongs to the entry; all helper edges count.
    for fault in 0..5 {
        let mut program = expanded_tail(&tail_source(80));
        let root = program
            .functions
            .iter()
            .find(|f| f.source_name == "traverse")
            .unwrap()
            .symbol
            .clone();
        match fault {
            0 => {
                program.functions[0].foreign = Some(CompilerCFunction {
                    library_identity: "foreign".into(),
                    external_symbol: "external".into(),
                    parameters: vec![],
                    result: CompilerCValue::Void,
                })
            }
            1 => program.functions[0].parameters[0].source_visible = false,
            2 => {
                program.functions[0].body.result.kind = CompilerExpressionKind::Call {
                    symbol: "unresolved".into(),
                    arguments: vec![],
                }
            }
            3 => {
                // An explicit empty row cannot hide transitive byte writes.
                program
                    .functions
                    .iter_mut()
                    .find(|f| f.symbol == root)
                    .unwrap()
                    .declared_effects = Some(CompilerEffectRow { identities: vec![] });
            }
            _ => {
                program.functions[0].body.result.kind = CompilerExpressionKind::Call {
                    symbol: root,
                    arguments: vec![],
                }
            }
        }
        let before = program.clone();
        assert!(
            coalesce_kernel_region_tail_helpers(&mut program, "traverse").is_err(),
            "fault {fault}"
        );
        assert_eq!(program, before, "original validation must precede edits");
    }
}

#[test]
fn tail_sharing_retains_declaration_identity_and_the_existing_function_budget() {
    let mut program = expanded_tail(&tail_source(80));
    for (index, function) in program.functions.iter_mut().enumerate() {
        if function.source_name == "write-one" {
            function.module_identity = Some(vec![format!("distinct-{index}")]);
        }
    }
    assert!(coalesce_kernel_region_tail_helpers(&mut program, "traverse").is_err());
    assert!(program.functions.len() > 64);
}

#[test]
fn ordinary_region_helper_composition_does_not_retain_collection_dispatch_frames() {
    let mut text = "use language (version is v0.1, features is (systems))\ncompare is fn (value : Nat) -> Boolean\n  value = 7\n".to_owned();
    let mut previous = "compare".to_owned();
    for level in 0..24 {
        let next = format!("forward-{level}");
        text.push_str(&format!(
            "{next} is fn (value : Nat) -> Boolean\n  {previous} value\n"
        ));
        previous = next;
    }
    text.push_str(&format!("pub apply is fn ((region : KernelByteRegion, first : Nat, second : Nat)) -> Boolean\n  {previous} first\n"));
    let program = analyze_kernel_region_framed_action_library(&text, &[], "apply", 2).unwrap();
    assert_eq!(program.functions.len(), 26);
    assert!(
        kernel_region_function_effects(&program)
            .unwrap()
            .iter()
            .all(|effect| !effect.read && !effect.write)
    );
}

#[test]
fn sharing_does_not_relax_the_sixty_four_distinct_function_budget() {
    let mut text = "use language (version is v0.1, features is (systems))\n".to_owned();
    for index in 0..80 {
        text.push_str(&format!("write-{index} is fn ((region : KernelByteRegion, value : Nat)) -> Boolean\n  region byte store (offset-bytes is {index}, value is value)\n"));
    }
    text.push_str(
        "pub apply is fn ((region : KernelByteRegion, first : Nat, second : Nat)) -> Boolean\n",
    );
    for index in 0..80 {
        text.push_str(&format!(
            "  written-{index} is write-{index} (region, second)\n"
        ));
    }
    text.push_str("  true\n");
    let mut program = expanded(&text);
    assert_eq!(program.functions.len(), 81);
    let error = coalesce_kernel_region_helpers(&mut program).unwrap_err();
    assert!(
        error
            .message
            .contains("region-library function identity budget")
    );
}
