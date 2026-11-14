// TOPAL-SYSTEMS-BOOT-PARTICIPANT-SOURCE-001, invalid internal model states only.
fn participant_init() -> SystemsOperation {
    SystemsOperation::InitializeBootParticipants {
        entry: BootParticipantEntry::InitialProviderSelected,
        failure_message: "failed".into(),
    }
}

fn participant_window() -> BootstrapAuthorityState {
    let mut state = BootstrapAuthorityState {
        allocator_created: true,
        ..Default::default()
    };
    for operation in [
        SystemsOperation::PrepareKernelImageTables {
            failure_message: "failed".into(),
        },
        SystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_IMAGE_TABLES_PREPARED".into(),
        },
        SystemsOperation::RetainKernelImageTables,
        SystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_IMAGE_TABLES_RETAINED".into(),
        },
    ] {
        state.observe(0, &operation).unwrap();
    }
    state
}

#[test]
fn participant_model_requires_the_initial_retained_unborrowed_window() {
    let mut absent = BootstrapAuthorityState {
        allocator_created: true,
        ..Default::default()
    };
    assert!(absent.observe(0, &participant_init()).is_err());
    for case in 0..6 {
        let mut state = participant_window();
        match case {
            0 => state.allocator_created = false,
            1 => state.image_tables_retained = false,
            2 => state.image_retention_observed = false,
            3 => {
                state.memory_ownership =
                    BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST)
            }
            4 => state.translation = BootstrapTranslationState::ReplacementActive,
            5 => {
                state
                    .observe(
                        0,
                        &SystemsOperation::ConsoleWrite {
                            text: "ordinary".into(),
                        },
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            state.observe(0, &participant_init()).is_err(),
            "case {case}"
        );
    }
}

#[test]
fn participant_model_requires_exact_consumption_then_observation() {
    let mut state = participant_window();
    assert!(
        state
            .observe(
                0,
                &SystemsOperation::RetainBootParticipantsForKernelLifetime
            )
            .is_err()
    );
    state.observe(0, &participant_init()).unwrap();
    assert_eq!(
        state.complete_boot_participants().unwrap_err().code,
        "E-SYSTEMS-BOOT-PARTICIPANTS-LIVE"
    );
    assert!(state.observe(0, &participant_init()).is_err());
    assert!(
        state
            .observe(
                0,
                &SystemsOperation::ConsoleWrite {
                    text: "TOPAL_BOOT_PARTICIPANTS_ARRIVED".into()
                }
            )
            .is_err()
    );
    assert!(
        state
            .observe(0, &SystemsOperation::DiscardRetainedKernelImageTables)
            .is_err()
    );
    state
        .observe(
            0,
            &SystemsOperation::RetainBootParticipantsForKernelLifetime,
        )
        .unwrap();
    assert!(state.complete_boot_participants().is_err());
    assert!(
        state
            .observe(
                0,
                &SystemsOperation::ConsoleWrite {
                    text: "ordinary".into()
                }
            )
            .is_err()
    );
    state
        .observe(
            0,
            &SystemsOperation::ConsoleWrite {
                text: "TOPAL_BOOT_PARTICIPANTS_ARRIVED".into(),
            },
        )
        .unwrap();
    state.complete_boot_participants().unwrap();
    assert!(state.boot_participants == BootParticipantRetentionState::Retained);
    assert!(state.observe(0, &participant_init()).is_err());
    assert!(
        state
            .observe(
                0,
                &SystemsOperation::RetainBootParticipantsForKernelLifetime
            )
            .is_err()
    );
}

#[test]
fn participant_model_cannot_gain_arrival_from_a_marker_or_regained_allocator() {
    let mut state = participant_window();
    state
        .observe(
            0,
            &SystemsOperation::ConsoleWrite {
                text: "TOPAL_BOOT_PARTICIPANTS_ARRIVED".into(),
            },
        )
        .unwrap();
    assert!(state.boot_participants == BootParticipantRetentionState::Unselected);
    assert!(state.observe(0, &participant_init()).is_err());
    let mut state = participant_window();
    state
        .observe(
            0,
            &SystemsOperation::AllocatePhysicalFrames {
                request: INITIAL_SINGLE_FRAME_REQUEST,
                failure_message: "failed".into(),
            },
        )
        .unwrap();
    state
        .observe(0, &SystemsOperation::ReleasePhysicalFrames)
        .unwrap();
    assert!(state.memory_ownership == BootstrapMemoryOwnership::None);
    assert!(state.observe(0, &participant_init()).is_err());
}

#[test]
fn participant_operations_cannot_enter_other_special_entry_contexts() {
    for operation in [
        participant_init(),
        SystemsOperation::RetainBootParticipantsForKernelLifetime,
    ] {
        let mut program = program(SystemsDisposition::Resume);
        program.debug_break.handler.operations.push(operation);
        program.debug_break.handler.effects = program
            .debug_break
            .handler
            .operations
            .iter()
            .map(|op| op.semantic_identity().to_owned())
            .chain(std::iter::once(SYSTEMS_RESUME_DEBUG_BREAK.into()))
            .collect();
        program.debug_break.handler.effects.sort();
        program.debug_break.handler.effects.dedup();
        assert_eq!(
            validate_systems_program(&program).unwrap_err().code,
            "E-SYSTEMS-OPERATION"
        );
    }
}
