#[path = "../../../linux-kernel/tools/user-memory/boot_participant_source.rs"]
mod boot_participant_source;

#[test]
fn boot_participant_emission_cannot_ignore_an_unqualified_operation() {
    // TOPAL-SYSTEMS-BOOT-PARTICIPANT-SOURCE-001, direct internal emitter boundary.
    for operation in [
        CompilerSystemsOperation::InitializeBootParticipants {
            entry: topal_language::compiler::CompilerBootParticipantEntry::InitialProviderSelected,
            failure_message: "failed".into(),
        },
        CompilerSystemsOperation::RetainBootParticipantsForKernelLifetime,
    ] {
        let mut encoder = RootEncoder::default();
        let error = encode_operations(&mut encoder, &[operation])
            .unwrap_err()
            .to_string();
        assert!(error.contains("E-SYSTEMS-BOOT-PARTICIPANTS"), "{error}");
        assert!(encoder.bytes.is_empty());
        assert!(encoder.relocations.is_empty());
    }
}

#[test]
fn boot_participant_trace_records_a_requirement_without_predicting_cpu_arrival() {
    let source = boot_participant_source::with_boot_participants(SOURCE);
    let program = analyze_systems_for_compiler(
        &source,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let trace = semantic_trace(&program).unwrap();
    let arrival = trace
        .iter()
        .find(|row| row.starts_with("topal.systems.boot-participants.initialize/1:"))
        .unwrap();
    assert_eq!(
        arrival,
        "topal.systems.boot-participants.initialize/1:entry=topal.systems.entry.boot-participant/1:requires-native-arrival:no-predicted-participants"
    );
    let retain = trace
        .iter()
        .find(|row| row.starts_with("topal.systems.boot-participants.retain-kernel-lifetime/1:"))
        .unwrap();
    assert!(retain.ends_with(":whole-entry-owners:kernel-lifetime:no-reclamation:no-activation"));
}
