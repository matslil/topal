const fn register_encoding(register: SavedRegister, base: u8) -> (Option<u8>, u8) {
    let index = match register {
        SavedRegister::Rax => 0,
        SavedRegister::Rcx => 1,
        SavedRegister::Rdx => 2,
        SavedRegister::Rbx => 3,
        SavedRegister::Rbp => 5,
        SavedRegister::Rsi => 6,
        SavedRegister::Rdi => 7,
        SavedRegister::R8 => 8,
        SavedRegister::R9 => 9,
        SavedRegister::R10 => 10,
        SavedRegister::R11 => 11,
        SavedRegister::R12 => 12,
        SavedRegister::R13 => 13,
        SavedRegister::R14 => 14,
        SavedRegister::R15 => 15,
    };
    if index < 8 {
        (None, base + index)
    } else {
        (Some(0x41), base + index - 8)
    }
}

#[allow(clippy::too_many_lines)] // Every typed provider dependency is constructed in one closed root boundary.
pub(crate) fn generate_root_object(
    program: &CompilerSystemsProgram,
) -> Result<Vec<u8>, CompileError> {
    generate_x86_64_systems_provider_object(program)?;
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    object.add_file_symbol(b"topal-generated-x86-systems-root".to_vec());
    let text = object.add_section(
        Vec::new(),
        X86_SYSTEMS_ROOT_TEXT_SECTION.as_bytes().to_vec(),
        SectionKind::Text,
    );
    let uses_bootstrap_storage = program
        .bootstrap
        .handler
        .operations
        .iter()
        .any(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::BootstrapAllocate { .. }
                    | CompilerSystemsOperation::BootstrapStoreByte { .. }
                    | CompilerSystemsOperation::BootstrapLoadByteEquals { .. }
                    | CompilerSystemsOperation::AtomicWordCreate { .. }
                    | CompilerSystemsOperation::AtomicCompareExchangeEquals { .. }
                    | CompilerSystemsOperation::AtomicLoadEquals { .. }
                    | CompilerSystemsOperation::AtomicWordEnd
                    | CompilerSystemsOperation::SendLocalNotification
                    | CompilerSystemsOperation::WaitLocalNotification
                    | CompilerSystemsOperation::ObserveMonotonicClock
                    | CompilerSystemsOperation::ConstructDeadline { .. }
                    | CompilerSystemsOperation::ArmDeadline
                    | CompilerSystemsOperation::WaitDeadline
                    | CompilerSystemsOperation::BootstrapRelease
                    | CompilerSystemsOperation::PrepareUserAddressSpace
                    | CompilerSystemsOperation::MapUserAddressSpace
                    | CompilerSystemsOperation::StoreUserMappingByte
                    | CompilerSystemsOperation::InstallUserMappingEntryTrampoline
                    | CompilerSystemsOperation::ProtectUserMappingReadOnly
                    | CompilerSystemsOperation::LoadUserMappingByte
                    | CompilerSystemsOperation::VerifyUserMappingEntryTrampoline
                    | CompilerSystemsOperation::PermitUserMappingExecute
                    | CompilerSystemsOperation::SealUserMappingImmutableData
                    | CompilerSystemsOperation::PermitUserMappingWritableData
                    | CompilerSystemsOperation::UnmapUserAddressSpace
                    | CompilerSystemsOperation::PrepareUserProbe { .. }
                    | CompilerSystemsOperation::EnterUserContext
                    | CompilerSystemsOperation::CompleteUserExecution
            )
        })
        || program
            .local_notification
            .handler
            .operations
            .iter()
            .any(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CompleteLocalNotification
                )
            })
        || program
            .deadline_notification
            .handler
            .operations
            .iter()
            .any(|operation| matches!(operation, CompilerSystemsOperation::CompleteDeadline));
    let mut symbols = ProviderSymbols {
        describe_boot_memory: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DescribeBootMemory,
        ),
        allocate_physical_frames: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AllocatePhysicalFrames,
        ),
        prepare_static_image: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareStaticImage,
        ),
        zero_static_image_payload: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ZeroStaticImagePayload,
        ),
        zero_static_image_stack_backing: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ZeroStaticImageStackBacking,
        ),
        publish_static_image_stack_leaf: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PublishStaticImageStackLeaf,
        ),
        initialize_static_image_stack_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::InitializeStaticImageStackEntry,
        ),
        add_static_image_stack_pagesz: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackPageSize,
        ),
        add_static_image_stack_entry_fact: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackEntryFact,
        ),
        add_static_image_stack_program_headers: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackProgramHeaders,
        ),
        add_static_image_stack_executable_name: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackExecutableName,
        ),
        add_static_image_stack_credentials: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackCredentials,
        ),
        add_static_image_stack_secure_mode: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackSecureMode,
        ),
        add_static_image_stack_clock_tick_rate: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackClockTickRate,
        ),
        add_static_image_stack_platform: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackPlatform,
        ),
        add_static_image_stack_hardware_capabilities: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackHardwareCapabilities,
        ),
        add_static_image_stack_load_metadata: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackLoadMetadata,
        ),
        authorize_static_image_stack_random_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageStackRandomSlot,
        ),
        retain_static_image_prepared_process: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RetainStaticImagePreparedProcess,
        ),
        qualify_static_image_retained_disjoint_work: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageRetainedDisjointWork,
        ),
        authorize_static_image_entropy_handoff: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageEntropyHandoff,
        ),
        authorize_static_image_entropy_acquisition: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageEntropyAcquisition,
        ),
        qualify_static_image_entropy_device: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyDevice,
        ),
        qualify_static_image_entropy_transport: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyTransport,
        ),
        qualify_static_image_entropy_negotiation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyNegotiation,
        ),
        qualify_static_image_entropy_queue: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyQueue,
        ),
        qualify_static_image_entropy_bus_master: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyBusMaster,
        ),
        qualify_static_image_entropy_dma_buffers: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyDmaBuffers,
        ),
        qualify_static_image_entropy_split_ring: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropySplitRing,
        ),
        qualify_static_image_entropy_dma_preparation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyDmaPreparation,
        ),
        qualify_static_image_entropy_queue_addresses: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyQueueAddresses,
        ),
        qualify_static_image_entropy_queue_activation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyQueueActivation,
        ),
        qualify_static_image_entropy_request_completion: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyRequestCompletion,
        ),
        qualify_static_image_entropy_sample_capture: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropySampleCapture,
        ),
        qualify_static_image_entropy_sample_quality: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropySampleQuality,
        ),
        qualify_static_image_entropy_pool_lifecycle: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyPoolLifecycle,
        ),
        qualify_static_image_entropy_generator_lifecycle: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle,
        ),
        deliver_static_image_entropy_initial_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DeliverStaticImageEntropyInitialRandom,
        ),
        enter_static_image_process: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EnterStaticImageProcess,
        ),
        restore_static_image_process_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RestoreStaticImageProcessEntry,
        ),
        revoke_static_image_entropy_initial_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntropyInitialRandom,
        ),
        revoke_static_image_entropy_acquisition: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntropyAcquisition,
        ),
        revoke_static_image_entropy_handoff: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntropyHandoff,
        ),
        restore_static_image_prepared_process: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RestoreStaticImagePreparedProcess,
        ),
        revoke_static_image_stack_random_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageStackRandomSlot,
        ),
        remove_static_image_stack_load_metadata: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackLoadMetadata,
        ),
        remove_static_image_stack_hardware_capabilities: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackHardwareCapabilities,
        ),
        remove_static_image_stack_platform: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackPlatform,
        ),
        remove_static_image_stack_clock_tick_rate: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackClockTickRate,
        ),
        remove_static_image_stack_secure_mode: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackSecureMode,
        ),
        remove_static_image_stack_credentials: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackCredentials,
        ),
        remove_static_image_stack_executable_name: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackExecutableName,
        ),
        remove_static_image_stack_program_headers: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackProgramHeaders,
        ),
        remove_static_image_stack_entry_fact: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackEntryFact,
        ),
        remove_static_image_stack_pagesz: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackPageSize,
        ),
        clear_static_image_stack_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ClearStaticImageStackEntry,
        ),
        revoke_static_image_stack_leaf: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageStackLeaf,
        ),
        copy_static_image_payload: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CopyStaticImagePayload,
        ),
        seal_static_image_permissions: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::SealStaticImagePermissions,
        ),
        attach_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AttachStaticImageAddressSpace,
        ),
        activate_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ActivateStaticImageAddressSpace,
        ),
        restore_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RestoreStaticImageAddressSpace,
        ),
        authorize_static_image_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageEntry,
        ),
        authorize_static_image_stack_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageStackSlot,
        ),
        revoke_static_image_stack_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageStackSlot,
        ),
        revoke_static_image_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntry,
        ),
        detach_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DetachStaticImageAddressSpace,
        ),
        revoke_static_image: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImage,
        ),
        begin_translation: undefined_provider_symbol(&mut object, ProviderSymbol::BeginTranslation),
        commit_translation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CommitTranslation,
        ),
        activate_translation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ActivateTranslation,
        ),
        begin_translation_edit: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::BeginTranslationEdit,
        ),
        map_translation_frames: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::MapTranslationFrames,
        ),
        unmap_translation_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::UnmapTranslationMapping,
        ),
        commit_translation_edit: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CommitTranslationEdit,
        ),
        enter_critical: undefined_provider_symbol(&mut object, ProviderSymbol::EnterCritical),
        restore_critical: undefined_provider_symbol(&mut object, ProviderSymbol::RestoreCritical),
        atomic_word_create: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AtomicWordCreate,
        ),
        atomic_word_compare_exchange: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AtomicWordCompareExchange,
        ),
        atomic_word_load: undefined_provider_symbol(&mut object, ProviderSymbol::AtomicWordLoad),
        send_local_notification: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::SendLocalNotification,
        ),
        wait_local_notification: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::WaitLocalNotification,
        ),
        complete_local_notification: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CompleteLocalNotification,
        ),
        observe_monotonic_clock: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ObserveMonotonicClock,
        ),
        observe_pci_configuration_dword: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ObservePciConfigurationDword,
        ),
        update_pci_configuration_word: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::UpdatePciConfigurationWord,
        ),
        create_dma_buffer: undefined_provider_symbol(&mut object, ProviderSymbol::CreateDmaBuffer),
        bind_dma_buffer_reference: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::BindDmaBufferReference,
        ),
        store_dma_buffer_integer: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StoreDmaBufferInteger,
        ),
        prepare_dma_buffer_for_device: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareDmaBufferForDevice,
        ),
        complete_dma_buffer_for_cpu: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CompleteDmaBufferForCpu,
        ),
        load_dma_buffer_integer: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::LoadDmaBufferInteger,
        ),
        capture_entropy_sample: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CaptureEntropySample,
        ),
        mix_entropy_pool_sample: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::MixEntropyPoolSample,
        ),
        initialize_entropy_generator: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::InitializeEntropyGenerator,
        ),
        extract_entropy_generator: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ExtractEntropyGenerator,
        ),
        deliver_initial_auxv_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DeliverInitialAuxvRandom,
        ),
        reserve_initial_user_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReserveInitialUserRandom,
        ),
        bind_reusable_user_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::BindReusableUserRandom,
        ),
        publish_pci_region_dma_reference: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PublishPciRegionDmaReference,
        ),
        dma_virtqueue_control: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DmaVirtqueueControl,
        ),
        dma_entropy_payload: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DmaEntropyPayload,
        ),
        raw_entropy_sample: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RawEntropySample,
        ),
        entropy_pool_state: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EntropyPoolState,
        ),
        entropy_generator_key: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EntropyGeneratorKey,
        ),
        entropy_output: undefined_provider_symbol(&mut object, ProviderSymbol::EntropyOutput),
        entropy_user_random_service: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EntropyUserRandomService,
        ),
        static_process_brk_reserve: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StaticProcessBrkReserve,
        ),
        static_process_exec_snapshot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StaticProcessExecSnapshot,
        ),
        static_process_descriptor_table: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StaticProcessDescriptorTable,
        ),
        read_pci_region_register: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReadPciRegionRegister,
        ),
        read_pci_region_register_eventually: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReadPciRegionRegisterEventually,
        ),
        write_pci_region_register: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::WritePciRegionRegister,
        ),
        construct_deadline: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ConstructDeadline,
        ),
        arm_deadline: undefined_provider_symbol(&mut object, ProviderSymbol::ArmDeadline),
        wait_deadline: undefined_provider_symbol(&mut object, ProviderSymbol::WaitDeadline),
        complete_deadline: undefined_provider_symbol(&mut object, ProviderSymbol::CompleteDeadline),
        create_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CreateKernelContext,
        ),
        transfer_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::TransferKernelContext,
        ),
        await_deadline_preemption: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AwaitDeadlinePreemption,
        ),
        preempt_current_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PreemptCurrentKernelContext,
        ),
        retire_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RetireKernelContext,
        ),
        reclaim_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReclaimKernelContext,
        ),
        prepare_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareUserAddressSpace,
        ),
        verify_user_address_space_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::VerifyUserAddressSpaceMapping,
        ),
        map_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::MapUserAddressSpace,
        ),
        store_user_mapping_byte: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StoreUserMappingByte,
        ),
        install_user_mapping_entry_trampoline: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::InstallUserMappingEntryTrampoline,
        ),
        protect_user_mapping_read_only: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ProtectUserMappingReadOnly,
        ),
        load_user_mapping_byte: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::LoadUserMappingByte,
        ),
        verify_user_mapping_entry_trampoline: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::VerifyUserMappingEntryTrampoline,
        ),
        permit_user_mapping_execute: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PermitUserMappingExecute,
        ),
        seal_user_mapping_immutable_data: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::SealUserMappingImmutableData,
        ),
        permit_user_mapping_writable_data: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PermitUserMappingWritableData,
        ),
        reserve_anonymous_user_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReserveAnonymousUserMapping,
        ),
        release_anonymous_user_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReleaseAnonymousUserMapping,
        ),
        unmap_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::UnmapUserAddressSpace,
        ),
        prepare_user_probe: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareUserProbe,
        ),
        complete_user_backing: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CompleteUserBacking,
        ),
        reclaim_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReclaimUserAddressSpace,
        ),
        enter_user_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EnterUserContext,
        ),
        user_syscall: undefined_provider_symbol(&mut object, ProviderSymbol::UserSyscall),
        kernel_thread_entry: None,
        completion_thread_entry: None,
        terminal_thread_entry: None,
        uart16550_write: undefined_provider_symbol(&mut object, ProviderSymbol::Uart16550Write),
        debug_break: undefined_provider_symbol(&mut object, ProviderSymbol::DebugBreak),
        interrupt_return: undefined_provider_symbol(&mut object, ProviderSymbol::InterruptReturn),
        fatal: undefined_provider_symbol(&mut object, ProviderSymbol::Fatal),
        bootstrap_storage: uses_bootstrap_storage
            .then(|| undefined_provider_symbol(&mut object, ProviderSymbol::BootstrapStorage)),
    };

    let mut kernel_thread = RootEncoder {
        role: RootRole::CooperativeKernelThreadHandler,
        ..RootEncoder::default()
    };
    encode_operations(
        &mut kernel_thread,
        &program.kernel_thread.handler.operations,
    )?;
    match program.kernel_thread.handler.disposition {
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            kernel_thread.retire_kernel_context()?;
        }
        CompilerSystemsDisposition::Fatal { .. } => kernel_thread.jump(ProviderSymbol::Fatal),
        CompilerSystemsDisposition::Resume => {
            return Err(CompileError::Tool(
                "x86 kernel-thread root cannot resume as an interrupt entry".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 kernel-thread root cannot preempt itself as a deadline entry".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 kernel-thread root cannot dispatch a user syscall".into(),
            ));
        }
    }
    let kernel_thread_entry = append_root(
        &mut object,
        text,
        X86_SYSTEMS_KERNEL_THREAD_ENTRY,
        &kernel_thread,
        symbols,
    )?;
    symbols.kernel_thread_entry = Some(kernel_thread_entry);

    let mut completion_thread = RootEncoder {
        role: RootRole::CompletionKernelThreadHandler,
        ..RootEncoder::default()
    };
    encode_operations(
        &mut completion_thread,
        &program.completion_thread.handler.operations,
    )?;
    match program.completion_thread.handler.disposition {
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            completion_thread.retire_kernel_context()?;
        }
        CompilerSystemsDisposition::Fatal { .. } => {
            completion_thread.jump(ProviderSymbol::Fatal);
        }
        CompilerSystemsDisposition::Resume
        | CompilerSystemsDisposition::PreemptKernelContext
        | CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 completion-thread root requires retirement disposition".into(),
            ));
        }
    }
    let completion_thread_entry = append_root(
        &mut object,
        text,
        X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
        &completion_thread,
        symbols,
    )?;
    symbols.completion_thread_entry = Some(completion_thread_entry);

    let mut terminal_thread = RootEncoder {
        role: RootRole::TerminalKernelThreadHandler,
        ..RootEncoder::default()
    };
    encode_operations(
        &mut terminal_thread,
        &program.terminal_thread.handler.operations,
    )?;
    match program.terminal_thread.handler.disposition {
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            terminal_thread.retire_kernel_context()?;
        }
        CompilerSystemsDisposition::Fatal { .. } => terminal_thread.jump(ProviderSymbol::Fatal),
        CompilerSystemsDisposition::Resume => {
            return Err(CompileError::Tool(
                "x86 terminal-thread root cannot resume as an interrupt entry".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 terminal-thread root cannot preempt a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 terminal-thread root cannot dispatch a user syscall".into(),
            ));
        }
    }
    let terminal_thread_entry = append_root(
        &mut object,
        text,
        X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
        &terminal_thread,
        symbols,
    )?;
    symbols.terminal_thread_entry = Some(terminal_thread_entry);

    if program.user_syscall.handler.operations
        != [
            CompilerSystemsOperation::CopyInitialUnameToUser,
            CompilerSystemsOperation::CopyInitialTimespecFromUser,
        ]
        || program.user_syscall.handler.disposition
            != (CompilerSystemsDisposition::DispatchInitialUserSyscall {
                getpid_result: 1,
                uname_sysname: "Linux".into(),
                uname_nodename: "topal".into(),
                uname_release: "7.2.9-topal".into(),
                uname_version: "#1 Topal".into(),
                uname_machine: "x86_64".into(),
                uname_domainname: "(none)".into(),
                nanosleep_seconds: 0,
                nanosleep_nanoseconds: 0,
                nanosleep_remainder_candidate: 0,
                clock_gettime_result: 0,
                clock_getres_result: 0,
                write_descriptor: 1,
                write_maximum_bytes: 256,
                read_descriptor: 0,
                read_maximum_bytes: 256,
                initial_descriptor_count: 3,
                bad_descriptor_error: 9,
                too_many_open_files_error: 24,
                console_input_description: 1,
                console_output_description: 2,
                console_error_description: 3,
                invalid_argument_error: 22,
                fcntl_getfd_command: 1,
                fcntl_setfd_command: 2,
                fcntl_getfl_command: 3,
                close_on_exec_flag: 1,
                read_only_status: 0,
                write_only_status: 1,
                stat_layout_bytes: 144,
                console_device: 0,
                console_inode: 1,
                console_mode: 8576,
                console_special_device: 17408,
                console_block_size: 4096,
                terminal_layout_bytes: 36,
                terminal_get_attributes_request: 21505,
                terminal_control_flags: 2239,
                terminal_minimum_input: 1,
                not_a_terminal_error: 25,
                window_size_layout_bytes: 8,
                window_size_query_request: 21523,
                seek_maximum_whence: 4,
                illegal_seek_error: 29,
                brk_data_page_bytes: 4096,
                getppid_result: 0,
                gettid_result: 1,
                getuid_result: 0,
                geteuid_result: 0,
                getgid_result: 0,
                getegid_result: 0,
                arch_set_fs_result: 0,
                arch_get_fs_result: 0,
                set_tid_address_result: 1,
                set_robust_list_result: 0,
                getrlimit_result: 0,
                prlimit64_result: 0,
                rt_sigprocmask_result: 0,
                anonymous_page_bytes: 4096,
                madvise_result: 0,
                mprotect_result: 0,
                munmap_result: 0,
                exit_group_status: 0,
                unsupported_syscall_error: 38,
                sched_yield_result: 0,
            })
    {
        return Err(CompileError::Tool(
            "x86 initial user-syscall entry requires the sealed implemented-call dispatch and unsupported-syscall fallback"
                .into(),
        ));
    }

    let mut bootstrap = RootEncoder {
        bootstrap_capacity: program.bootstrap_storage.capacity_bytes,
        region_library: program.region_library.is_some(),
        role: RootRole::BootstrapWithLocalNotification,
        source_bound_library: program.kernel_library.is_some(),
        source_library_export: program
            .kernel_library
            .as_ref()
            .map(|r| {
                crate::systems_library::finite::FinitePolicy::from_interface(r.interface.as_str())
                    .map(|p| p.export())
            })
            .transpose()?
            .unwrap_or(crate::systems_library::EXPORT_SYMBOL),
        numeric_library: program.time_library.is_some(),
        image_library: program.image_library.is_some(),
        dispatch_library: program.dispatch_library.is_some(),
        dispatch_policy: program
            .dispatch_library
            .as_ref()
            .map(|r| crate::systems_numeric_library::NumericPolicy::from_interface(r.interface))
            .transpose()?,
        process_identity_library: program.process_identity_library.is_some(),
        retained_identity: crate::systems_numeric_library::uses_retained_identity(program),
        structured_identity: crate::systems_numeric_library::uses_structured_identity(program),
        ..RootEncoder::default()
    };
    encode_operations(&mut bootstrap, &program.bootstrap.handler.operations)?;
    if bootstrap.region_library {
        bootstrap.call_region_dependency(RootLibraryDependency::RegionFinalize);
    }
    bootstrap.jump(ProviderSymbol::Fatal);
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_KERNEL_ENTRY,
        &bootstrap,
        symbols,
    )?;

    let mut deadline = RootEncoder {
        role: RootRole::DeadlineHandler,
        ..RootEncoder::default()
    };
    for register in SAVED_REGISTERS {
        deadline.push(register);
    }
    deadline.filter_completed_deadline_notification()?;
    encode_operations(
        &mut deadline,
        &program.deadline_notification.handler.operations,
    )?;
    match program.deadline_notification.handler.disposition {
        CompilerSystemsDisposition::Resume => {
            for register in SAVED_REGISTERS.into_iter().rev() {
                deadline.pop(register);
            }
            deadline.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::Fatal { .. } => deadline.jump(ProviderSymbol::Fatal),
        CompilerSystemsDisposition::PreemptKernelContext => {
            deadline.preempt_current_kernel_context()?;
            for register in SAVED_REGISTERS.into_iter().rev() {
                deadline.pop(register);
            }
            deadline.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            return Err(CompileError::Tool(
                "x86 deadline entry cannot retire a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 deadline entry cannot dispatch a user syscall".into(),
            ));
        }
    }
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_DEADLINE_ENTRY,
        &deadline,
        symbols,
    )?;

    let mut local_notification = RootEncoder {
        role: RootRole::LocalNotificationHandler,
        ..RootEncoder::default()
    };
    for register in SAVED_REGISTERS {
        local_notification.push(register);
    }
    encode_operations(
        &mut local_notification,
        &program.local_notification.handler.operations,
    )?;
    match program.local_notification.handler.disposition {
        CompilerSystemsDisposition::Resume => {
            for register in SAVED_REGISTERS.into_iter().rev() {
                local_notification.pop(register);
            }
            local_notification.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::Fatal { .. } => {
            local_notification.jump(ProviderSymbol::Fatal);
        }
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            return Err(CompileError::Tool(
                "x86 local-notification entry cannot retire a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 local-notification entry cannot preempt a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 local-notification entry cannot dispatch a user syscall".into(),
            ));
        }
    }
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
        &local_notification,
        symbols,
    )?;

    let mut debug_break = RootEncoder::default();
    for register in SAVED_REGISTERS {
        debug_break.push(register);
    }
    encode_operations(&mut debug_break, &program.debug_break.handler.operations)?;
    match program.debug_break.handler.disposition {
        CompilerSystemsDisposition::Resume => {
            for register in SAVED_REGISTERS.into_iter().rev() {
                debug_break.pop(register);
            }
            debug_break.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::Fatal { .. } => debug_break.jump(ProviderSymbol::Fatal),
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            return Err(CompileError::Tool(
                "x86 debug-break entry cannot retire a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 debug-break entry cannot preempt a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 debug-break entry cannot dispatch a user syscall".into(),
            ));
        }
    }
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        &debug_break,
        symbols,
    )?;
    object
        .write()
        .map_err(|error| CompileError::Tool(format!("cannot encode systems root ELF: {error}")))
}
