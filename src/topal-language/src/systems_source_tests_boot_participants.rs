#[path = "../../../linux-kernel/tools/user-memory/boot_participant_source.rs"]
mod boot_participant_source;

#[test]
fn boot_participant_source_preserves_typed_entry_and_affine_domain_retention() {
    // TOPAL-SYSTEMS-BOOT-PARTICIPANT-SOURCE-001.
    let source = boot_participant_source::with_boot_participants(SOURCE);
    let program = analyze_systems_for_compiler(
        &source,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    validate_systems_program(&program).unwrap();
    let renamed = source
        .replacen("Ok arrivals then", "Ok arrived-domain then", 1)
        .replacen(
            "retain arrivals for-kernel-lifetime",
            "retain arrived-domain for-kernel-lifetime",
            1,
        );
    assert_eq!(
        analyze_systems_for_compiler(
            &renamed,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu()
        )
        .unwrap(),
        program
    );
    let operations = &program.bootstrap.handler.operations;
    let start = operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::InitializeBootParticipants { .. }
            )
        })
        .unwrap();
    assert!(
        matches!(&operations[start],CompilerSystemsOperation::InitializeBootParticipants {
        entry:CompilerBootParticipantEntry::InitialProviderSelected,failure_message
    } if failure_message=="boot participant initialization failed")
    );
    assert!(matches!(
        operations[start + 1],
        CompilerSystemsOperation::RetainBootParticipantsForKernelLifetime
    ));
    assert!(
        matches!(&operations[start+2],CompilerSystemsOperation::ConsoleWrite {text} if text=="TOPAL_BOOT_PARTICIPANTS_ARRIVED")
    );
    assert!(matches!(
        operations[start + 3],
        CompilerSystemsOperation::BeginTranslationUpdate { .. }
    ));
    let transitions = model_systems_transitions(&program).unwrap();
    assert_eq!(
        transitions
            .iter()
            .filter(|t| matches!(
                t,
                CompilerSystemsTransition::RequireBootParticipantArrivals {
                    entry: CompilerBootParticipantEntry::InitialProviderSelected
                }
            ))
            .count(),
        1
    );
    assert_eq!(
        transitions
            .iter()
            .filter(|t| matches!(t, CompilerSystemsTransition::RetainBootParticipantDomain))
            .count(),
        1
    );
    assert_eq!(
        CompilerBootParticipantEntry::InitialProviderSelected.semantic_identity(),
        SYSTEMS_BOOT_PARTICIPANT_ENTRY
    );
    assert_eq!(
        operations[start].semantic_identity(),
        SYSTEMS_BOOT_PARTICIPANTS_INITIALIZE
    );
    assert_eq!(
        operations[start + 1].semantic_identity(),
        SYSTEMS_BOOT_PARTICIPANTS_RETAIN
    );
}

#[test]
fn boot_participant_source_rejects_native_policy_fields_and_ambiguous_resources() {
    let source = boot_participant_source::with_boot_participants(SOURCE);
    for (old, new) in [
        ("entry is provider-selected", "entry is 1"),
        ("entry is provider-selected", "entry is arbitrary-handler"),
        (
            "entry is provider-selected",
            "entry is provider-selected, cpu-count is 2",
        ),
        (
            "entry is provider-selected",
            "entry is provider-selected, entry is provider-selected",
        ),
        ("entry is provider-selected", "address is 4096"),
        (
            "memory boot participants initialize",
            "other boot participants initialize",
        ),
        ("Ok arrivals then", "Ok memory then"),
        (
            "retain arrivals for-kernel-lifetime",
            "retain unrelated for-kernel-lifetime",
        ),
        (
            "memory boot participants retain",
            "other boot participants retain",
        ),
        (
            "retain arrivals for-kernel-lifetime",
            "retain arrivals until-next-root",
        ),
        (
            "retain arrivals for-kernel-lifetime",
            "release arrivals for-kernel-lifetime",
        ),
        (
            "failure fatal \"boot participant initialization failed\"",
            "memory fatal \"boot participant initialization failed\"",
        ),
    ] {
        assert!(source.contains(old));
        assert!(
            analyze_systems_for_compiler(
                &source.replacen(old, new, 1),
                &CompilerSystemsTargetSelection::initial_x86_64_qemu()
            )
            .is_err(),
            "{old} => {new}"
        );
    }
}

#[test]
fn boot_participant_source_rejects_missing_duplicate_interleaved_and_late_consumers() {
    let source = boot_participant_source::with_boot_participants(SOURCE);
    let retain = "memory boot participants retain arrivals for-kernel-lifetime";
    let initialize = "memory boot participants initialize (entry is provider-selected)";
    for (old, new) in [
        (retain.to_owned(), String::new()),
        (retain.to_owned(), format!("{retain}\n {retain}")),
        (
            retain.to_owned(),
            format!("memory console write \"interleaving\"\n {retain}"),
        ),
        (
            initialize.to_owned(),
            format!("memory console write \"late initialization\"\n {initialize}"),
        ),
        (
            "TOPAL_BOOT_PARTICIPANTS_ARRIVED".into(),
            "TOPAL_KERNEL_IMAGE_TABLES_RETAINED".into(),
        ),
        (
            "memory console write \"TOPAL_KERNEL_IMAGE_TABLES_RETAINED\"".into(),
            String::new(),
        ),
        (
            "memory kernel image retain tables image-backing during".into(),
            "memory kernel image discard tables image-backing during".into(),
        ),
    ] {
        assert!(source.contains(&old));
        assert!(
            analyze_systems_for_compiler(
                &source.replacen(&old, &new, 1),
                &CompilerSystemsTargetSelection::initial_x86_64_qemu()
            )
            .is_err(),
            "{old} => {new}"
        );
    }
}

#[test]
fn boot_participant_semantics_independently_rejects_forged_success_ir() {
    let source = boot_participant_source::with_boot_participants(SOURCE);
    let baseline = analyze_systems_for_compiler(
        &source,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let start = baseline
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|op| {
            matches!(
                op,
                CompilerSystemsOperation::InitializeBootParticipants { .. }
            )
        })
        .unwrap();
    for case in 0..6 {
        let mut program = baseline.clone();
        let operations = &mut program.bootstrap.handler.operations;
        match case {
            0 => {
                operations.remove(start + 1);
            }
            1 => {
                operations.remove(start + 2);
            }
            2 => {
                operations.insert(start, operations[start].clone());
            }
            3 => {
                operations.insert(
                    start + 2,
                    CompilerSystemsOperation::RetainBootParticipantsForKernelLifetime,
                );
            }
            4 => {
                operations.insert(
                    start,
                    CompilerSystemsOperation::ConsoleWrite {
                        text: "ordinary".into(),
                    },
                );
            }
            5 => {
                operations.remove(start);
            }
            _ => unreachable!(),
        }
        program.bootstrap.handler.effects = operations
            .iter()
            .map(|op| op.semantic_identity().to_owned())
            .chain(std::iter::once(
                program
                    .bootstrap
                    .handler
                    .disposition
                    .semantic_identity(program.bootstrap.handler.context)
                    .to_owned(),
            ))
            .collect();
        program.bootstrap.handler.effects.sort();
        program.bootstrap.handler.effects.dedup();
        assert!(validate_systems_program(&program).is_err(), "case {case}");
        assert!(model_systems_transitions(&program).is_err(), "case {case}");
    }
}

#[test]
fn boot_participant_initialization_remains_explicit_in_existing_profiles() {
    let program = analyze_systems_for_compiler(
        SOURCE,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    assert!(
        !program
            .bootstrap
            .handler
            .operations
            .iter()
            .any(|op| matches!(
                op,
                CompilerSystemsOperation::InitializeBootParticipants { .. }
                    | CompilerSystemsOperation::RetainBootParticipantsForKernelLifetime
            ))
    );
    assert!(
        !model_systems_transitions(&program)
            .unwrap()
            .iter()
            .any(|t| matches!(
                t,
                CompilerSystemsTransition::RequireBootParticipantArrivals { .. }
                    | CompilerSystemsTransition::RetainBootParticipantDomain
            ))
    );
}
