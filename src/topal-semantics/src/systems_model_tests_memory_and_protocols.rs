#[test]
fn prepared_user_space_requires_two_mapping_generations_and_released_extent() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-FRAMES-001,
    // TOPAL-SYSTEMS-CONTEXT-006.
    let mut live_at_probe = program(SystemsDisposition::Resume);
    let scratch_release = live_at_probe
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::PrepareUserAddressSpace))
        .unwrap()
        + 3;
    let prepare_probe = live_at_probe
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::PrepareUserProbe { .. }))
        .unwrap();
    live_at_probe
        .bootstrap
        .handler
        .operations
        .swap(scratch_release, prepare_probe);
    assert_eq!(
        validate_systems_program(&live_at_probe).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut third_extent = program(SystemsDisposition::Resume);
    let scratch_allocation = third_extent
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::PrepareUserAddressSpace))
        .unwrap()
        + 2;
    third_extent.bootstrap.handler.operations.insert(
        scratch_allocation,
        SystemsOperation::AllocatePhysicalFrames {
            request: INITIAL_SINGLE_FRAME_REQUEST,
            failure_message: "third frame allocation failed".into(),
        },
    );
    assert_eq!(
        validate_systems_program(&third_extent).unwrap_err().code,
        "E-SYSTEMS-FRAMES"
    );

    let mut missing_unmap = program(SystemsDisposition::Resume);
    let unmap = missing_unmap
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::UnmapUserAddressSpace))
        .unwrap();
    missing_unmap.bootstrap.handler.operations.remove(unmap);
    refresh_bootstrap_effects(&mut missing_unmap);
    assert_eq!(
        validate_systems_program(&missing_unmap).unwrap_err().code,
        "E-SYSTEMS-FRAMES"
    );

    let mut duplicate_store = program(SystemsDisposition::Resume);
    let store = duplicate_store
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::StoreUserMappingByte))
        .unwrap();
    duplicate_store
        .bootstrap
        .handler
        .operations
        .insert(store + 1, SystemsOperation::StoreUserMappingByte);
    assert_eq!(
        validate_systems_program(&duplicate_store).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut missing_protection = program(SystemsDisposition::Resume);
    let protection = missing_protection
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::ProtectUserMappingReadOnly))
        .unwrap();
    missing_protection
        .bootstrap
        .handler
        .operations
        .remove(protection);
    refresh_bootstrap_effects(&mut missing_protection);
    assert_eq!(
        validate_systems_program(&missing_protection)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicate_protection = program(SystemsDisposition::Resume);
    let protection = duplicate_protection
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::ProtectUserMappingReadOnly))
        .unwrap();
    duplicate_protection
        .bootstrap
        .handler
        .operations
        .insert(protection + 1, SystemsOperation::ProtectUserMappingReadOnly);
    assert_eq!(
        validate_systems_program(&duplicate_protection)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut reordered_access = program(SystemsDisposition::Resume);
    let store = reordered_access
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::StoreUserMappingByte))
        .unwrap();
    reordered_access
        .bootstrap
        .handler
        .operations
        .swap(store, store + 1);
    assert_eq!(
        validate_systems_program(&reordered_access)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut code_promoted_to_writable_data = program(SystemsDisposition::Resume);
    let execute = code_promoted_to_writable_data
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::PermitUserMappingExecute))
        .unwrap();
    code_promoted_to_writable_data.bootstrap.handler.operations[execute] =
        SystemsOperation::PermitUserMappingWritableData;
    refresh_bootstrap_effects(&mut code_promoted_to_writable_data);
    assert_eq!(
        validate_systems_program(&code_promoted_to_writable_data)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut data_promoted_to_code = program(SystemsDisposition::Resume);
    let seal = data_promoted_to_code
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::PermitUserMappingWritableData))
        .unwrap();
    data_promoted_to_code.bootstrap.handler.operations[seal] =
        SystemsOperation::PermitUserMappingExecute;
    refresh_bootstrap_effects(&mut data_promoted_to_code);
    assert_eq!(
        validate_systems_program(&data_promoted_to_code)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_image_readiness = program(SystemsDisposition::Resume);
    let readiness = late_image_readiness
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_READY"
            )
        })
        .unwrap();
    late_image_readiness
        .bootstrap
        .handler
        .operations
        .swap(readiness, readiness + 1);
    assert_eq!(
        validate_systems_program(&late_image_readiness)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicate_image_readiness = program(SystemsDisposition::Resume);
    let readiness = duplicate_image_readiness
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_READY"
            )
        })
        .unwrap();
    duplicate_image_readiness
        .bootstrap
        .handler
        .operations
        .insert(
            readiness + 1,
            SystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
            },
        );
    assert_eq!(
        validate_systems_program(&duplicate_image_readiness)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_initial_stack = program(SystemsDisposition::Resume);
    let initial_stack = late_initial_stack
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_STACK_OK"
            )
        })
        .unwrap();
    late_initial_stack
        .bootstrap
        .handler
        .operations
        .swap(initial_stack, initial_stack + 1);
    assert_eq!(
        validate_systems_program(&late_initial_stack)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_initial_auxv = program(SystemsDisposition::Resume);
    let initial_auxv = late_initial_auxv
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_OK"
            )
        })
        .unwrap();
    late_initial_auxv
        .bootstrap
        .handler
        .operations
        .swap(initial_auxv, initial_auxv + 1);
    assert_eq!(
        validate_systems_program(&late_initial_auxv)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_initial_entry_auxv = program(SystemsDisposition::Resume);
    let initial_entry_auxv = late_initial_entry_auxv
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK"
            )
        })
        .unwrap();
    late_initial_entry_auxv
        .bootstrap
        .handler
        .operations
        .swap(initial_entry_auxv, initial_entry_auxv + 1);
    assert_eq!(
        validate_systems_program(&late_initial_entry_auxv)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_initial_execfn_auxv = program(SystemsDisposition::Resume);
    let initial_execfn_auxv = late_initial_execfn_auxv
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK"
            )
        })
        .unwrap();
    late_initial_execfn_auxv
        .bootstrap
        .handler
        .operations
        .swap(initial_execfn_auxv, initial_execfn_auxv + 1);
    assert_eq!(
        validate_systems_program(&late_initial_execfn_auxv)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_initial_credential_auxv = program(SystemsDisposition::Resume);
    let initial_credential_auxv = late_initial_credential_auxv
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK"
            )
        })
        .unwrap();
    late_initial_credential_auxv
        .bootstrap
        .handler
        .operations
        .swap(initial_credential_auxv, initial_credential_auxv + 1);
    assert_eq!(
        validate_systems_program(&late_initial_credential_auxv)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_elf_identity = program(SystemsDisposition::Resume);
    let elf_identity = late_elf_identity
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_ELF_OK"
            )
        })
        .unwrap();
    late_elf_identity
        .bootstrap
        .handler
        .operations
        .swap(elf_identity, elf_identity + 1);
    assert_eq!(
        validate_systems_program(&late_elf_identity)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_image_copy_to = program(SystemsDisposition::Resume);
    let image_copy_to = late_image_copy_to
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK"
            )
        })
        .unwrap();
    late_image_copy_to
        .bootstrap
        .handler
        .operations
        .swap(image_copy_to, image_copy_to + 1);
    assert_eq!(
        validate_systems_program(&late_image_copy_to)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_image_copy_from = program(SystemsDisposition::Resume);
    let image_copy_from = late_image_copy_from
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK"
            )
        })
        .unwrap();
    late_image_copy_from
        .bootstrap
        .handler
        .operations
        .swap(image_copy_from, image_copy_from + 1);
    assert_eq!(
        validate_systems_program(&late_image_copy_from)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_image_syscall = program(SystemsDisposition::Resume);
    let image_syscall = late_image_syscall
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK"
            )
        })
        .unwrap();
    late_image_syscall
        .bootstrap
        .handler
        .operations
        .swap(image_syscall, image_syscall + 1);
    assert_eq!(
        validate_systems_program(&late_image_syscall)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut late_image_retention = program(SystemsDisposition::Resume);
    let retained = late_image_retention
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_RETAINED"
            )
        })
        .unwrap();
    late_image_retention
        .bootstrap
        .handler
        .operations
        .swap(retained, retained + 1);
    assert_eq!(
        validate_systems_program(&late_image_retention)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut completion_before_teardown = program(SystemsDisposition::Resume);
    let completion = completion_before_teardown
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::CompleteUserExecution))
        .unwrap();
    let retained = completion_before_teardown
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_RETAINED"
            )
        })
        .unwrap();
    completion_before_teardown
        .bootstrap
        .handler
        .operations
        .swap(completion, retained + 1);
    assert_eq!(
        validate_systems_program(&completion_before_teardown)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut third_generation = program(SystemsDisposition::Resume);
    let release = third_generation
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter(|(_, operation)| matches!(operation, SystemsOperation::UnmapUserAddressSpace))
        .nth(1)
        .map(|(index, _)| index + 1)
        .unwrap();
    third_generation.bootstrap.handler.operations.splice(
        release..release,
        [
            SystemsOperation::MapUserAddressSpace,
            SystemsOperation::StoreUserMappingByte,
            SystemsOperation::ProtectUserMappingReadOnly,
            SystemsOperation::LoadUserMappingByte,
            SystemsOperation::PermitUserMappingExecute,
            SystemsOperation::UnmapUserAddressSpace,
        ],
    );
    assert_eq!(
        validate_systems_program(&third_generation)
            .unwrap_err()
            .code,
        "E-SYSTEMS-USER-CONTEXT"
    );
}

#[test]
fn local_notification_requires_matching_wait_completion_and_resume() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-LOCAL-INTERRUPT-001.
    let mut live = program(SystemsDisposition::Resume);
    remove_context_lifecycle(&mut live);
    live.bootstrap
        .handler
        .operations
        .retain(|operation| !matches!(operation, SystemsOperation::WaitLocalNotification));
    live.bootstrap
        .handler
        .effects
        .retain(|effect| effect != SYSTEMS_LOCAL_NOTIFICATION_WAIT);
    live.bootstrap
        .handler
        .operations
        .retain(|operation| !matches!(operation, SystemsOperation::ObserveMonotonicClock));
    live.bootstrap
        .handler
        .effects
        .retain(|effect| effect != SYSTEMS_MONOTONIC_CLOCK_NOW);
    live.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            SystemsOperation::ConstructDeadline { .. }
                | SystemsOperation::ArmDeadline
                | SystemsOperation::WaitDeadline
        )
    });
    live.bootstrap.handler.effects.retain(|effect| {
        !matches!(
            effect.as_str(),
            SYSTEMS_DEADLINE_AFTER | SYSTEMS_DEADLINE_ARM | SYSTEMS_DEADLINE_WAIT
        )
    });
    assert_eq!(
        validate_systems_program(&live).unwrap_err().code,
        "E-SYSTEMS-LOCAL-INTERRUPT-LIVE"
    );

    let mut duplicate = program(SystemsDisposition::Resume);
    duplicate
        .bootstrap
        .handler
        .operations
        .insert(5, SystemsOperation::SendLocalNotification);
    assert_eq!(
        validate_systems_program(&duplicate).unwrap_err().code,
        "E-SYSTEMS-LOCAL-INTERRUPT"
    );

    let mut incomplete = program(SystemsDisposition::Resume);
    incomplete.local_notification.handler.operations.clear();
    incomplete.local_notification.handler.effects = vec![SYSTEMS_RESUME_LOCAL_NOTIFICATION.into()];
    assert_eq!(
        validate_systems_program(&incomplete).unwrap_err().code,
        "E-SYSTEMS-LOCAL-INTERRUPT"
    );
}

#[test]
fn monotonic_clock_requires_two_post_interrupt_observations() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-MONOTONIC-CLOCK-001.
    let mut missing = program(SystemsDisposition::Resume);
    remove_context_lifecycle(&mut missing);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            SystemsOperation::ConstructDeadline { .. }
                | SystemsOperation::ArmDeadline
                | SystemsOperation::WaitDeadline
        )
    });
    let observation = missing
        .bootstrap
        .handler
        .operations
        .iter()
        .rposition(|operation| matches!(operation, SystemsOperation::ObserveMonotonicClock))
        .unwrap();
    missing.bootstrap.handler.operations.remove(observation);
    refresh_bootstrap_effects(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-MONOTONIC-CLOCK-LIVE"
    );

    let mut early = program(SystemsDisposition::Resume);
    let observation_index = early
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::ObserveMonotonicClock))
        .unwrap();
    let observation = early.bootstrap.handler.operations.remove(observation_index);
    early.bootstrap.handler.operations.insert(4, observation);
    assert_eq!(
        validate_systems_program(&early).unwrap_err().code,
        "E-SYSTEMS-MONOTONIC-CLOCK"
    );

    let mut extra = program(SystemsDisposition::Resume);
    extra
        .bootstrap
        .handler
        .operations
        .push(SystemsOperation::ObserveMonotonicClock);
    assert_eq!(
        validate_systems_program(&extra).unwrap_err().code,
        "E-SYSTEMS-MONOTONIC-CLOCK"
    );
}

#[test]
fn deadline_requires_exact_construction_preemption_and_completion() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-DEADLINE-EVENT-001,
    // TOPAL-SYSTEMS-CONTEXT-004.
    let mut wrong_duration = program(SystemsDisposition::Resume);
    wrong_duration.bootstrap.handler.operations[8] = SystemsOperation::ConstructDeadline {
        duration_nanoseconds: 2_000_000,
    };
    assert_eq!(
        validate_systems_program(&wrong_duration).unwrap_err().code,
        "E-SYSTEMS-DEADLINE-EVENT"
    );

    let mut missing_preemption = program(SystemsDisposition::Resume);
    remove_context_lifecycle(&mut missing_preemption);
    assert_eq!(
        validate_systems_program(&missing_preemption)
            .unwrap_err()
            .code,
        "E-SYSTEMS-DEADLINE-EVENT-LIVE"
    );

    let mut incomplete = program(SystemsDisposition::Resume);
    incomplete.deadline_notification.handler.operations.clear();
    incomplete.deadline_notification.handler.effects =
        vec![SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT.into()];
    assert_eq!(
        validate_systems_program(&incomplete).unwrap_err().code,
        "E-SYSTEMS-DEADLINE-EVENT"
    );
}

#[test]
fn kernel_context_requires_create_transfer_retirement_and_reclaim() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-CONTEXT-001.
    let mut missing_reclaim = program(SystemsDisposition::Resume);
    let final_reclaim = missing_reclaim
        .bootstrap
        .handler
        .operations
        .iter()
        .rposition(|operation| matches!(operation, SystemsOperation::ReclaimKernelContext))
        .unwrap();
    missing_reclaim
        .bootstrap
        .handler
        .operations
        .remove(final_reclaim);
    missing_reclaim
        .bootstrap
        .handler
        .operations
        .retain(|operation| {
            !matches!(
                operation,
                SystemsOperation::ConsoleWrite { text }
                    if text == "TOPAL_KERNEL_USER_IMAGE_READY"
                        || text == "TOPAL_KERNEL_BOOT_RESOURCE_OK"
                        || text == "TOPAL_KERNEL_USER_READ_OK"
                        || text == "TOPAL_KERNEL_USER_CLOSE_OK"
                        || text == "TOPAL_KERNEL_USER_DUP_OK"
                        || text == "TOPAL_KERNEL_USER_FCNTL_OK"
                        || text == "TOPAL_KERNEL_USER_FSTAT_OK"
                        || text == "TOPAL_KERNEL_USER_TCGETS_OK"
                        || text == "TOPAL_KERNEL_USER_WINSIZE_OK"
                        || text == "TOPAL_KERNEL_USER_LSEEK_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_STACK_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_ELF_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_BSS_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_DATA_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RETAINED"
            )
        });
    missing_reclaim
        .bootstrap
        .handler
        .operations
        .retain(|operation| {
            !matches!(
                operation,
                SystemsOperation::ExtractEntropyGenerator { request }
                    if *request == EntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM
            ) && !matches!(
                operation,
                SystemsOperation::ConsumeKernelCompletion
                    | SystemsOperation::ConsumeEmptyRunnableQueue
                    | SystemsOperation::PrepareUserAddressSpace
                    | SystemsOperation::VerifyUserAddressSpaceMapping
                    | SystemsOperation::MapUserAddressSpace
                    | SystemsOperation::ReserveAnonymousUserMapping
                    | SystemsOperation::StoreUserMappingByte
                    | SystemsOperation::InstallUserMappingEntryTrampoline
                    | SystemsOperation::ProtectUserMappingReadOnly
                    | SystemsOperation::PermitUserMappingExecute
                    | SystemsOperation::SealUserMappingImmutableData
                    | SystemsOperation::PermitUserMappingWritableData
                    | SystemsOperation::LoadUserMappingByte
                    | SystemsOperation::VerifyUserMappingEntryTrampoline
                    | SystemsOperation::UnmapUserAddressSpace
                    | SystemsOperation::ReleaseAnonymousUserMapping
                    | SystemsOperation::PrepareUserProbe { .. }
                    | SystemsOperation::DeliverInitialAuxvRandom { .. }
                    | SystemsOperation::ReserveInitialUserRandom { .. }
                    | SystemsOperation::BindReusableUserRandom { .. }
                    | SystemsOperation::ExpandSmallUserRandomRequests { .. }
                    | SystemsOperation::ExpandLargeUserRandomRequests { .. }
                    | SystemsOperation::EnterUserContext
                    | SystemsOperation::CompleteUserExecution
                    | SystemsOperation::ReclaimUserAddressSpace
                    | SystemsOperation::AllocatePhysicalFrames { .. }
                    | SystemsOperation::ReleasePhysicalFrames
            )
        });
    refresh_bootstrap_effects(&mut missing_reclaim);
    assert_eq!(
        validate_systems_program(&missing_reclaim).unwrap_err().code,
        "E-SYSTEMS-CONTEXT-TRANSFER-LIVE"
    );

    let mut transfer_before_create = program(SystemsDisposition::Resume);
    let create = transfer_before_create
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::CreateKernelContext))
        .unwrap();
    let transfer = transfer_before_create
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, SystemsOperation::TransferKernelContext))
        .or_else(|| {
            transfer_before_create
                .bootstrap
                .handler
                .operations
                .iter()
                .position(|operation| {
                    matches!(
                        operation,
                        SystemsOperation::TransferKernelContextUntilDeadline
                    )
                })
        })
        .unwrap();
    transfer_before_create
        .bootstrap
        .handler
        .operations
        .swap(create, transfer);
    assert_eq!(
        validate_systems_program(&transfer_before_create)
            .unwrap_err()
            .code,
        "E-SYSTEMS-CONTEXT-TRANSFER"
    );

    let mut wrong_handler = program(SystemsDisposition::Resume);
    wrong_handler.kernel_thread.handler.operations[0] = SystemsOperation::ConsoleWrite {
        text: "wrong".into(),
    };
    wrong_handler.kernel_thread.handler.effects = vec![
        SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION.into(),
        SYSTEMS_CONSOLE_WRITE.into(),
        SYSTEMS_KERNEL_CONTEXT_RETIRE.into(),
    ];
    assert_eq!(
        validate_systems_program(&wrong_handler).unwrap_err().code,
        "E-SYSTEMS-CONTEXT-TRANSFER"
    );
}

#[test]
fn models_checked_bootstrap_region_contents_and_release() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STORAGE-001.
    let mut program = program(SystemsDisposition::Resume);
    program.bootstrap.handler.operations.extend([
        SystemsOperation::BootstrapAllocate {
            request: BootstrapStorageRequest {
                byte_count: 64,
                alignment_bytes: 8,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            },
        },
        SystemsOperation::BootstrapStoreByte {
            offset_bytes: 7,
            value: 90,
        },
        SystemsOperation::BootstrapLoadByteEquals {
            offset_bytes: 7,
            expected: 90,
            failure_message: "mismatch".into(),
        },
        SystemsOperation::ConsoleWrite {
            text: "memory ok".into(),
        },
        SystemsOperation::BootstrapRelease,
    ]);
    refresh_bootstrap_effects(&mut program);
    let transitions = model_systems_transitions(&program).unwrap();
    assert!(
        transitions.contains(&SystemsTransition::StoreBootstrapByte {
            offset_bytes: 7,
            value: 90,
        })
    );
    assert!(transitions.contains(&SystemsTransition::LoadBootstrapByte {
        offset_bytes: 7,
        value: 90,
    }));
    assert!(transitions.contains(&SystemsTransition::ReleaseBootstrapRegion));

    let release = program.bootstrap.handler.operations.pop().unwrap();
    program
        .bootstrap
        .handler
        .effects
        .retain(|effect| effect != SYSTEMS_BOOTSTRAP_STORAGE_RELEASE);
    assert_eq!(
        validate_systems_program(&program).unwrap_err().code,
        "E-SYSTEMS-STORAGE-LIVE"
    );
    program.bootstrap.handler.operations.push(release);
    program
        .bootstrap
        .handler
        .effects
        .push(SYSTEMS_BOOTSTRAP_STORAGE_RELEASE.into());
    program.bootstrap.handler.effects.sort();
    let store = program
        .bootstrap
        .handler
        .operations
        .iter_mut()
        .find(|operation| matches!(operation, SystemsOperation::BootstrapStoreByte { .. }))
        .expect("test program contains a bootstrap store");
    let SystemsOperation::BootstrapStoreByte { offset_bytes, .. } = store else {
        unreachable!("matching operation was selected")
    };
    *offset_bytes = 64;
    assert_eq!(
        validate_systems_program(&program).unwrap_err().code,
        "E-SYSTEMS-STORAGE-BOUNDS"
    );
}

#[test]
fn atomic_location_excludes_plain_region_ownership_until_end() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-ATOMIC-001,
    // TOPAL-SYSTEMS-STORAGE-001.
    let mut live = program(SystemsDisposition::Resume);
    live.bootstrap.handler.operations.extend([
        SystemsOperation::BootstrapAllocate {
            request: BootstrapStorageRequest {
                byte_count: 64,
                alignment_bytes: 8,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            },
        },
        SystemsOperation::AtomicWordCreate {
            request: AtomicWordRequest::initial(),
        },
    ]);
    live.bootstrap.handler.effects.extend([
        SYSTEMS_ATOMIC_WORD_CREATE.into(),
        SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE.into(),
    ]);
    live.bootstrap.handler.effects.sort();
    live.bootstrap.handler.effects.dedup();
    assert_eq!(
        validate_systems_program(&live).unwrap_err().code,
        "E-SYSTEMS-ATOMIC-LIVE"
    );

    live.bootstrap
        .handler
        .operations
        .push(SystemsOperation::BootstrapRelease);
    live.bootstrap
        .handler
        .effects
        .push(SYSTEMS_BOOTSTRAP_STORAGE_RELEASE.into());
    live.bootstrap.handler.effects.sort();
    live.bootstrap.handler.effects.dedup();
    assert_eq!(
        validate_systems_program(&live).unwrap_err().code,
        "E-SYSTEMS-STORAGE-LIFETIME"
    );
}

#[test]
fn models_affine_physical_frame_allocation_and_release() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-FRAMES-001.
    let mut program = program(SystemsDisposition::Resume);
    let request = PhysicalFrameRequest {
        frame_count: 3,
        alignment_frames: 1,
    };
    program.bootstrap.handler.operations.splice(
        2..2,
        [
            SystemsOperation::AllocatePhysicalFrames {
                request,
                failure_message: "frame allocation failed".into(),
            },
            SystemsOperation::ReleasePhysicalFrames,
        ],
    );
    program.bootstrap.handler.effects.extend([
        SYSTEMS_FRAMES_ALLOCATE.into(),
        SYSTEMS_FRAMES_RELEASE.into(),
    ]);
    program.bootstrap.handler.effects.sort();
    program.bootstrap.handler.effects.dedup();
    let transitions = model_systems_transitions(&program).unwrap();
    assert!(transitions.contains(&SystemsTransition::AllocatePhysicalFrames { request }));
    assert!(transitions.contains(&SystemsTransition::ReleasePhysicalFrames));

    let mut unsupported = program.clone();
    let SystemsOperation::AllocatePhysicalFrames {
        request: unsupported_request,
        ..
    } = &mut unsupported.bootstrap.handler.operations[2]
    else {
        unreachable!("test inserted the allocation at index two")
    };
    unsupported_request.frame_count = 4;
    assert_eq!(
        validate_systems_program(&unsupported).unwrap_err().code,
        "E-SYSTEMS-FRAMES"
    );

    let mut mapped = program.clone();
    mapped.bootstrap.handler.operations.insert(
        3,
        SystemsOperation::MapKernelFrames {
            request: KernelMappingRequest::initial_read_write(),
            failure_message: "mapping failed".into(),
        },
    );
    refresh_bootstrap_effects(&mut mapped);
    assert_eq!(
        validate_systems_program(&mapped).unwrap_err().code,
        "E-SYSTEMS-MAPPING"
    );

    program.bootstrap.handler.operations.drain(2..4);
    program
        .bootstrap
        .handler
        .operations
        .push(SystemsOperation::AllocatePhysicalFrames {
            request,
            failure_message: "late frame allocation failed".into(),
        });
    refresh_bootstrap_effects(&mut program);
    assert_eq!(
        validate_systems_program(&program).unwrap_err().code,
        "E-SYSTEMS-FRAMES-LIVE"
    );
}

#[test]
fn models_exact_static_elf_mapping_resource_ownership() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STATIC-ELF-MAPPING-RESOURCES-001.
    let mut program = program(SystemsDisposition::Resume);
    let operations = [
        SystemsOperation::AllocatePhysicalFrames {
            request: STATIC_ELF_FRAME_REQUEST,
            failure_message: "payload allocation failed".into(),
        },
        SystemsOperation::AllocatePhysicalFrames {
            request: INITIAL_SINGLE_FRAME_REQUEST,
            failure_message: "metadata allocation failed".into(),
        },
        SystemsOperation::ReleasePhysicalFrames,
        SystemsOperation::ReleasePhysicalFrames,
    ];
    program
        .bootstrap
        .handler
        .operations
        .splice(2..2, operations.clone());
    refresh_bootstrap_effects(&mut program);
    assert!(validate_systems_program(&program).is_ok());

    let mut wrong_metadata = program.clone();
    let SystemsOperation::AllocatePhysicalFrames { request, .. } =
        &mut wrong_metadata.bootstrap.handler.operations[3]
    else {
        unreachable!("test inserted metadata allocation at index three")
    };
    *request = INITIAL_TWO_FRAME_REQUEST;
    assert_eq!(
        validate_systems_program(&wrong_metadata).unwrap_err().code,
        "E-SYSTEMS-FRAMES"
    );

    let mut incomplete = program;
    incomplete.bootstrap.handler.operations.remove(5);
    refresh_bootstrap_effects(&mut incomplete);
    assert_eq!(
        validate_systems_program(&incomplete).unwrap_err().code,
        "E-SYSTEMS-FRAMES"
    );
}

#[test]
fn models_static_elf_initial_leaf_construction_and_revocation() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STATIC-ELF-LEAVES-001,
    // TOPAL-SYSTEMS-STATIC-ELF-ZERO-001, TOPAL-SYSTEMS-STATIC-ELF-COPY-001.
    let mut program = program(SystemsDisposition::Resume);
    program.bootstrap.handler.operations.splice(
        2..2,
        [
            SystemsOperation::AllocatePhysicalFrames {
                request: STATIC_ELF_FRAME_REQUEST,
                failure_message: "payload allocation failed".into(),
            },
            SystemsOperation::AllocatePhysicalFrames {
                request: INITIAL_SINGLE_FRAME_REQUEST,
                failure_message: "metadata allocation failed".into(),
            },
            SystemsOperation::PrepareStaticImageConstruction {
                failure_message: "leaf construction failed".into(),
            },
            SystemsOperation::ZeroStaticImageConstructionPayload,
            SystemsOperation::CopyStaticImageConstructionPayload,
            SystemsOperation::SealStaticImageConstructionPermissions,
            SystemsOperation::RevokeStaticImageConstructionMetadata,
            SystemsOperation::ReleasePhysicalFrames,
            SystemsOperation::ReclaimStaticImageConstructionPayload,
            SystemsOperation::ReleasePhysicalFrames,
        ],
    );
    refresh_bootstrap_effects(&mut program);
    let transitions = model_systems_transitions(&program).unwrap();
    assert!(transitions.contains(&SystemsTransition::PrepareStaticImageConstruction));
    assert!(transitions.contains(&SystemsTransition::ZeroStaticImageConstructionPayload));
    assert!(transitions.contains(&SystemsTransition::CopyStaticImageConstructionPayload));
    assert!(transitions.contains(&SystemsTransition::SealStaticImageConstructionPermissions));
    assert!(transitions.contains(&SystemsTransition::RevokeStaticImageConstructionMetadata));
    assert!(transitions.contains(&SystemsTransition::ReclaimStaticImageConstructionPayload));

    let mut reclaim_before_metadata_release = program.clone();
    reclaim_before_metadata_release
        .bootstrap
        .handler
        .operations
        .swap(9, 10);
    assert_eq!(
        validate_systems_program(&reclaim_before_metadata_release)
            .unwrap_err()
            .code,
        "E-SYSTEMS-STATIC-IMAGE"
    );

    let mut revoke_before_zero = program.clone();
    revoke_before_zero.bootstrap.handler.operations.remove(5);
    refresh_bootstrap_effects(&mut revoke_before_zero);
    assert_eq!(
        validate_systems_program(&revoke_before_zero)
            .unwrap_err()
            .code,
        "E-SYSTEMS-STATIC-IMAGE"
    );

    let mut revoke_before_copy = program.clone();
    revoke_before_copy.bootstrap.handler.operations.remove(6);
    refresh_bootstrap_effects(&mut revoke_before_copy);
    assert_eq!(
        validate_systems_program(&revoke_before_copy)
            .unwrap_err()
            .code,
        "E-SYSTEMS-STATIC-IMAGE"
    );

    let mut revoke_before_seal = program.clone();
    revoke_before_seal.bootstrap.handler.operations.remove(7);
    refresh_bootstrap_effects(&mut revoke_before_seal);
    assert_eq!(
        validate_systems_program(&revoke_before_seal)
            .unwrap_err()
            .code,
        "E-SYSTEMS-STATIC-IMAGE"
    );

    let mut live = program;
    live.bootstrap.handler.operations.truncate(5);
    refresh_bootstrap_effects(&mut live);
    assert_eq!(
        validate_systems_program(&live).unwrap_err().code,
        "E-SYSTEMS-FRAMES-LIVE"
    );
}

#[test]
fn models_static_elf_stack_entry_lifecycle() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STATIC-ELF-STACK-LEAF-001,
    // TOPAL-SYSTEMS-STATIC-ELF-STACK-ENTRY-001.
    let mut program = program(SystemsDisposition::Resume);
    program.bootstrap.handler.operations.splice(
        2..2,
        [
            SystemsOperation::AllocatePhysicalFrames {
                request: STATIC_ELF_FRAME_REQUEST,
                failure_message: "payload allocation failed".into(),
            },
            SystemsOperation::AllocatePhysicalFrames {
                request: INITIAL_SINGLE_FRAME_REQUEST,
                failure_message: "metadata allocation failed".into(),
            },
            SystemsOperation::PrepareStaticImageConstruction {
                failure_message: "leaf construction failed".into(),
            },
            SystemsOperation::ZeroStaticImageConstructionPayload,
            SystemsOperation::CopyStaticImageConstructionPayload,
            SystemsOperation::SealStaticImageConstructionPermissions,
            SystemsOperation::AllocatePhysicalFrames {
                request: STATIC_ELF_HIERARCHY_FRAME_REQUEST,
                failure_message: "hierarchy allocation failed".into(),
            },
            SystemsOperation::AttachStaticImageAddressSpace,
            SystemsOperation::ActivateStaticImageAddressSpace,
            SystemsOperation::RestoreStaticImageAddressSpace,
            SystemsOperation::AuthorizeStaticImageEntry,
            SystemsOperation::AuthorizeStaticImageStackSlot,
            SystemsOperation::AllocatePhysicalFrames {
                request: INITIAL_TWO_FRAME_REQUEST,
                failure_message: "stack backing allocation failed".into(),
            },
            SystemsOperation::ZeroStaticImageStackBacking,
            SystemsOperation::PublishStaticImageStackLeaf,
            SystemsOperation::InitializeStaticImageStackEntry,
            SystemsOperation::AddStaticImageStackPageSize,
            SystemsOperation::AddStaticImageStackEntryFact,
            SystemsOperation::AddStaticImageStackProgramHeaders,
            SystemsOperation::AddStaticImageStackExecutableName,
            SystemsOperation::AddStaticImageStackCredentials,
            SystemsOperation::AddStaticImageStackSecureMode,
            SystemsOperation::AddStaticImageStackClockTickRate,
            SystemsOperation::AddStaticImageStackPlatform,
            SystemsOperation::AddStaticImageStackHardwareCapabilities,
            SystemsOperation::AddStaticImageStackLoadMetadata,
            SystemsOperation::AuthorizeStaticImageStackRandomSlot,
            SystemsOperation::RetainStaticImagePreparedProcess,
            SystemsOperation::QualifyStaticImageRetainedDisjointWork,
            SystemsOperation::AuthorizeStaticImageEntropyHandoff,
            SystemsOperation::AuthorizeStaticImageEntropyAcquisition,
            SystemsOperation::QualifyStaticImageEntropyDevice,
            SystemsOperation::QualifyStaticImageEntropyTransport,
            SystemsOperation::QualifyStaticImageEntropyNegotiation,
            SystemsOperation::QualifyStaticImageEntropyQueue,
            SystemsOperation::QualifyStaticImageEntropyBusMaster,
            SystemsOperation::QualifyStaticImageEntropyDmaBuffers,
            SystemsOperation::QualifyStaticImageEntropySplitRing,
            SystemsOperation::QualifyStaticImageEntropyDmaPreparation,
            SystemsOperation::QualifyStaticImageEntropyQueueAddresses,
            SystemsOperation::QualifyStaticImageEntropyQueueActivation,
            SystemsOperation::QualifyStaticImageEntropyRequestCompletion,
            SystemsOperation::QualifyStaticImageEntropySampleCapture,
            SystemsOperation::QualifyStaticImageEntropySampleQuality,
            SystemsOperation::QualifyStaticImageEntropyPoolLifecycle,
            SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle,
            SystemsOperation::DeliverStaticImageEntropyInitialRandom,
            SystemsOperation::EnterStaticImageProcess,
            SystemsOperation::RestoreStaticImageProcessEntry,
            SystemsOperation::RevokeStaticImageEntropyInitialRandom,
            SystemsOperation::RevokeStaticImageEntropyAcquisition,
            SystemsOperation::RevokeStaticImageEntropyHandoff,
            SystemsOperation::RestoreStaticImagePreparedProcess,
            SystemsOperation::RevokeStaticImageStackRandomSlot,
            SystemsOperation::RemoveStaticImageStackLoadMetadata,
            SystemsOperation::RemoveStaticImageStackHardwareCapabilities,
            SystemsOperation::RemoveStaticImageStackPlatform,
            SystemsOperation::RemoveStaticImageStackClockTickRate,
            SystemsOperation::RemoveStaticImageStackSecureMode,
            SystemsOperation::RemoveStaticImageStackCredentials,
            SystemsOperation::RemoveStaticImageStackExecutableName,
            SystemsOperation::RemoveStaticImageStackProgramHeaders,
            SystemsOperation::RemoveStaticImageStackEntryFact,
            SystemsOperation::RemoveStaticImageStackPageSize,
            SystemsOperation::ClearStaticImageStackEntry,
            SystemsOperation::RevokeStaticImageStackLeaf,
            SystemsOperation::ReleasePhysicalFrames,
            SystemsOperation::RevokeStaticImageStackSlot,
            SystemsOperation::RevokeStaticImageEntry,
            SystemsOperation::DetachStaticImageAddressSpace,
            SystemsOperation::ReclaimStaticImageConstructionHierarchy,
            SystemsOperation::ReleasePhysicalFrames,
            SystemsOperation::RevokeStaticImageConstructionMetadata,
            SystemsOperation::ReleasePhysicalFrames,
            SystemsOperation::ReclaimStaticImageConstructionPayload,
            SystemsOperation::ReleasePhysicalFrames,
        ],
    );
    refresh_bootstrap_effects(&mut program);
    let transitions = model_systems_transitions(&program).unwrap();
    assert!(transitions.contains(&SystemsTransition::PublishStaticImageStackLeaf));
    assert!(transitions.contains(&SystemsTransition::InitializeStaticImageStackEntry));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackPageSize));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackEntryFact));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackProgramHeaders));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackExecutableName));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackCredentials));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackSecureMode));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackClockTickRate));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackPlatform));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackHardwareCapabilities));
    assert!(transitions.contains(&SystemsTransition::AddStaticImageStackLoadMetadata));
    assert!(transitions.contains(&SystemsTransition::AuthorizeStaticImageStackRandomSlot));
    assert!(transitions.contains(&SystemsTransition::RetainStaticImagePreparedProcess));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageRetainedDisjointWork));
    assert!(transitions.contains(&SystemsTransition::AuthorizeStaticImageEntropyHandoff));
    assert!(transitions.contains(&SystemsTransition::AuthorizeStaticImageEntropyAcquisition));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyDevice));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyTransport));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyNegotiation));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyQueue));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyBusMaster));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyDmaBuffers));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropySplitRing));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyDmaPreparation));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyQueueAddresses));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyQueueActivation));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyRequestCompletion));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropySampleCapture));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropySampleQuality));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyPoolLifecycle));
    assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyGeneratorLifecycle));
    assert!(transitions.contains(&SystemsTransition::DeliverStaticImageEntropyInitialRandom));
    assert!(transitions.contains(&SystemsTransition::EnterStaticImageProcess));
    assert!(transitions.contains(&SystemsTransition::RestoreStaticImageProcessEntry));
    assert!(transitions.contains(&SystemsTransition::RevokeStaticImageEntropyInitialRandom));
    assert!(transitions.contains(&SystemsTransition::RevokeStaticImageEntropyAcquisition));
    assert!(transitions.contains(&SystemsTransition::RevokeStaticImageEntropyHandoff));
    assert!(transitions.contains(&SystemsTransition::RestoreStaticImagePreparedProcess));
    assert!(transitions.contains(&SystemsTransition::RevokeStaticImageStackRandomSlot));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackLoadMetadata));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackHardwareCapabilities));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackPlatform));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackClockTickRate));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackSecureMode));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackCredentials));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackExecutableName));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackProgramHeaders));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackEntryFact));
    assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackPageSize));
    assert!(transitions.contains(&SystemsTransition::ClearStaticImageStackEntry));
    assert!(transitions.contains(&SystemsTransition::RevokeStaticImageStackLeaf));

    let mut release_while_initialized = program.clone();
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    release_while_initialized
        .bootstrap
        .handler
        .operations
        .remove(19);
    refresh_bootstrap_effects(&mut release_while_initialized);
    assert_eq!(
        validate_systems_program(&release_while_initialized)
            .unwrap_err()
            .code,
        "E-SYSTEMS-FRAMES"
    );

    let mut duplicate_publication = program.clone();
    duplicate_publication
        .bootstrap
        .handler
        .operations
        .insert(17, SystemsOperation::PublishStaticImageStackLeaf);
    refresh_bootstrap_effects(&mut duplicate_publication);
    assert_eq!(
        validate_systems_program(&duplicate_publication)
            .unwrap_err()
            .code,
        "E-SYSTEMS-STATIC-IMAGE"
    );

    let mut duplicate_initialization = program.clone();
    duplicate_initialization
        .bootstrap
        .handler
        .operations
        .insert(18, SystemsOperation::InitializeStaticImageStackEntry);
    refresh_bootstrap_effects(&mut duplicate_initialization);
    assert_eq!(
        validate_systems_program(&duplicate_initialization)
            .unwrap_err()
            .code,
        "E-SYSTEMS-STATIC-IMAGE"
    );

    let mut remove_load_metadata_before_addition = program;
    remove_load_metadata_before_addition
        .bootstrap
        .handler
        .operations
        .swap(27, 28);
    assert_eq!(
        validate_systems_program(&remove_load_metadata_before_addition)
            .unwrap_err()
            .code,
        "E-SYSTEMS-STATIC-IMAGE"
    );
}

#[test]
fn critical_scope_requires_lifo_restore_before_blocking_or_completion() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-CRITICAL-001.
    let mut blocked = program(SystemsDisposition::Resume);
    blocked.bootstrap.handler.operations.insert(
        2,
        SystemsOperation::EnterCritical {
            domain: CriticalDomain::LocalMaskableInterrupts,
            failure_message: "critical entry failed".into(),
        },
    );
    blocked
        .bootstrap
        .handler
        .effects
        .push(SYSTEMS_CRITICAL_ENTER.into());
    blocked.bootstrap.handler.effects.sort();
    assert_eq!(
        validate_systems_program(&blocked).unwrap_err().code,
        "E-SYSTEMS-CRITICAL"
    );

    blocked.bootstrap.handler.operations.insert(
        3,
        SystemsOperation::RestoreCritical {
            domain: CriticalDomain::LocalMaskableInterrupts,
        },
    );
    blocked
        .bootstrap
        .handler
        .effects
        .push(SYSTEMS_CRITICAL_RESTORE.into());
    blocked.bootstrap.handler.effects.sort();
    assert!(validate_systems_program(&blocked).is_ok());

    let mut live = program(SystemsDisposition::Resume);
    live.bootstrap.handler.operations.truncate(2);
    live.bootstrap
        .handler
        .operations
        .push(SystemsOperation::EnterCritical {
            domain: CriticalDomain::LocalMaskableInterrupts,
            failure_message: "critical entry failed".into(),
        });
    live.bootstrap.handler.effects = vec![
        SYSTEMS_BOOT_MEMORY_DESCRIBE.into(),
        SYSTEMS_CRITICAL_ENTER.into(),
        SYSTEMS_FATAL.into(),
        SYSTEMS_FRAME_ALLOCATOR_CREATE.into(),
    ];
    assert_eq!(
        validate_systems_program(&live).unwrap_err().code,
        "E-SYSTEMS-CRITICAL-LIVE"
    );

    let mut restore_without_entry = program(SystemsDisposition::Resume);
    restore_without_entry.bootstrap.handler.operations.insert(
        2,
        SystemsOperation::RestoreCritical {
            domain: CriticalDomain::LocalMaskableInterrupts,
        },
    );
    restore_without_entry
        .bootstrap
        .handler
        .effects
        .push(SYSTEMS_CRITICAL_RESTORE.into());
    restore_without_entry.bootstrap.handler.effects.sort();
    assert_eq!(
        validate_systems_program(&restore_without_entry)
            .unwrap_err()
            .code,
        "E-SYSTEMS-CRITICAL"
    );
}

#[test]
fn fatal_debug_break_cannot_resume_bootstrap() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-DISPOSITION-001.
    let transitions = model_systems_transitions(&program(SystemsDisposition::Fatal {
        message: "fault".into(),
    }))
    .unwrap();
    assert_eq!(
        transitions.last(),
        Some(&SystemsTransition::Fatal {
            message: "fault".into(),
        })
    );
    assert!(!transitions.contains(&SystemsTransition::Fatal {
        message: "done".into(),
    }));
}

#[test]
fn rejects_invalid_context_operations_and_effect_claims() {
    // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-AUTHORITY-001.
    let mut invalid_context = program(SystemsDisposition::Resume);
    invalid_context
        .debug_break
        .handler
        .operations
        .push(SystemsOperation::DebugBreak);
    assert_eq!(
        model_systems_transitions(&invalid_context)
            .unwrap_err()
            .code,
        "E-SYSTEMS-OPERATION"
    );

    let mut invalid_effects = program(SystemsDisposition::Resume);
    invalid_effects.bootstrap.handler.effects.clear();
    assert_eq!(
        model_systems_transitions(&invalid_effects)
            .unwrap_err()
            .code,
        "E-SYSTEMS-EFFECTS"
    );
}

#[test]
fn rejects_wrong_target_and_bootstrap_resume() {
    // TOPAL-SYSTEMS-QUALIFY-001, TOPAL-SYSTEMS-DISPOSITION-001.
    let mut wrong_target = program(SystemsDisposition::Resume);
    wrong_target.target.board = "another-board".into();
    assert_eq!(
        validate_systems_program(&wrong_target).unwrap_err().code,
        "E-SYSTEMS-TARGET"
    );

    let mut invalid_disposition = program(SystemsDisposition::Resume);
    invalid_disposition.bootstrap.handler.disposition = SystemsDisposition::Resume;
    invalid_disposition.bootstrap.handler.effects = vec![
        SYSTEMS_CONSOLE_WRITE.into(),
        SYSTEMS_DEBUG_BREAK.into(),
        SYSTEMS_RESUME_DEBUG_BREAK.into(),
    ];
    assert_eq!(
        validate_systems_program(&invalid_disposition)
            .unwrap_err()
            .code,
        "E-SYSTEMS-DISPOSITION"
    );
}
#[test]
fn rejects_invalid_bootstrap_storage_descriptor() {
    // TOPAL-SYSTEMS-STORAGE-001.
    let mut invalid = program(SystemsDisposition::Resume);
    invalid.bootstrap_storage.alignment_bytes = 3;
    assert_eq!(
        validate_systems_program(&invalid).unwrap_err().code,
        "E-SYSTEMS-STORAGE"
    );
}
