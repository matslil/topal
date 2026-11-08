#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedSystemsProviderObject {
    pub plan: X86SystemsProviderPlan,
    pub bytes: Vec<u8>,
}

/// Generate one relocatable ELF64 provider object from a checked systems root.
///
/// # Errors
///
/// Returns a tool error when provider planning fails or the ELF writer cannot
/// encode the closed object.
pub fn generate_x86_64_systems_provider_object(
    program: &CompilerSystemsProgram,
) -> Result<GeneratedSystemsProviderObject, CompileError> {
    let plan = plan_x86_64_systems_provider(program)?;
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    object.add_file_symbol(b"topal-generated-x86-systems-provider".to_vec());

    let text = object.add_section(
        Vec::new(),
        X86_SYSTEMS_PROVIDER_TEXT_SECTION.as_bytes().to_vec(),
        SectionKind::Text,
    );
    append_function(
        &mut object,
        text,
        "topal_x86_systems_uart16550_write",
        &[
            X86Instruction::MoveUartLineStatusPortToDx,
            X86Instruction::InputByteFromPort,
            X86Instruction::TestTransmitterHoldingRegisterEmpty,
            X86Instruction::RetryWhileTransmitterBusy,
            X86Instruction::MoveUartDataPortToDx,
            X86Instruction::MoveByteArgumentToAl,
            X86Instruction::OutputByteToPort,
            X86Instruction::Return,
        ],
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_REGION_READ_SYMBOL,
        &pci_region_configuration_gateway(PciRegionGatewayOperation::ReadEquals)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL,
        &pci_region_configuration_gateway(PciRegionGatewayOperation::ReadEventuallyEquals)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL,
        &pci_region_configuration_gateway(PciRegionGatewayOperation::Write)?,
    );
    append_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL,
        &[
            X86Instruction::MovePciAddressPortToDx,
            X86Instruction::MoveDwordSelectorToEax,
            X86Instruction::OutputDwordToPort,
            X86Instruction::MovePciDataPortToDx,
            X86Instruction::InputDwordFromPort,
            X86Instruction::CompareDwordWithExpected,
            X86Instruction::SetEqualAl,
            X86Instruction::ZeroExtendAlToEax,
            X86Instruction::Return,
        ],
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL,
        &pci_configuration_update_word()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL,
        &dma_buffer_create()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL,
        &dma_buffer_bind_reference()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL,
        &dma_buffer_store_integer()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL,
        &dma_buffer_prepare_device()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL,
        &dma_buffer_prepare_device()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL,
        &dma_buffer_load_integer()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL,
        &entropy_capture_dma_sample()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL,
        &entropy_mix_blake2s_pool_sample()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_GENERATOR_INITIALIZE_SYMBOL,
        &entropy_initialize_chacha20_generator()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_GENERATOR_EXTRACT_SYMBOL,
        &entropy_extract_chacha20_generator()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM_SYMBOL,
        &entropy_deliver_initial_auxv_random()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM_SYMBOL,
        &entropy_reserve_initial_user_random()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL,
        &entropy_bind_reusable_user_random()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL,
        &pci_region_write_dma_reference()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
        &boot_memory_validator()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL,
        &physical_frame_selector()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL,
        &static_image_prepare_initial_leaves()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL,
        &static_image_zero_payload()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL,
        &static_image_zero_stack_backing()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL,
        &static_image_publish_stack_leaf()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_LEAF_SYMBOL,
        &static_image_revoke_stack_leaf()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL,
        &static_image_initialize_stack_entry()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_CLEAR_STACK_ENTRY_SYMBOL,
        &static_image_clear_stack_entry()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL,
        &static_image_add_stack_pagesz()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL,
        &static_image_add_stack_entry_fact()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PROGRAM_HEADERS_SYMBOL,
        &static_image_add_stack_program_headers()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_EXECUTABLE_NAME_SYMBOL,
        &static_image_add_stack_executable_name()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CREDENTIALS_SYMBOL,
        &static_image_add_stack_credentials()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_SECURE_MODE_SYMBOL,
        &static_image_add_stack_secure_mode()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CLOCK_TICK_RATE_SYMBOL,
        &static_image_add_stack_clock_tick_rate()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PLATFORM_SYMBOL,
        &static_image_add_stack_platform()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_HARDWARE_CAPABILITIES_SYMBOL,
        &static_image_add_stack_hardware_capabilities()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_LOAD_METADATA_SYMBOL,
        &static_image_add_stack_load_metadata()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_LOAD_METADATA_SYMBOL,
        &static_image_remove_stack_load_metadata()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_RANDOM_SLOT_SYMBOL,
        &static_image_authorize_stack_random_slot()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_RETAIN_PREPARED_PROCESS_SYMBOL,
        &static_image_retain_prepared_process()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_RETAINED_DISJOINT_WORK_SYMBOL,
        &static_image_qualify_retained_disjoint_work()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_HANDOFF_SYMBOL,
        &static_image_authorize_entropy_handoff()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_ACQUISITION_SYMBOL,
        &static_image_authorize_entropy_acquisition()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DEVICE_SYMBOL,
        &static_image_qualify_entropy_device()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_TRANSPORT_SYMBOL,
        &static_image_qualify_entropy_transport()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_NEGOTIATION_SYMBOL,
        &static_image_qualify_entropy_negotiation()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_SYMBOL,
        &static_image_qualify_entropy_queue()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_BUS_MASTER_SYMBOL,
        &static_image_qualify_entropy_bus_master()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_BUFFERS_SYMBOL,
        &static_image_qualify_entropy_dma_buffers()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SPLIT_RING_SYMBOL,
        &static_image_qualify_entropy_split_ring()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_PREPARATION_SYMBOL,
        &static_image_qualify_entropy_dma_preparation()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ADDRESSES_SYMBOL,
        &static_image_qualify_entropy_queue_addresses()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ACTIVATION_SYMBOL,
        &static_image_qualify_entropy_queue_activation()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_REQUEST_COMPLETION_SYMBOL,
        &static_image_qualify_entropy_request_completion()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_CAPTURE_SYMBOL,
        &static_image_qualify_entropy_sample_capture()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_QUALITY_SYMBOL,
        &static_image_qualify_entropy_sample_quality()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_POOL_LIFECYCLE_SYMBOL,
        &static_image_qualify_entropy_pool_lifecycle()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE_SYMBOL,
        &static_image_qualify_entropy_generator_lifecycle()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_DELIVER_ENTROPY_INITIAL_RANDOM_SYMBOL,
        &static_image_deliver_entropy_initial_random()?,
    );
    if program.inode_mode_library.is_some() {
        inode_region::append(&mut object, text, program.dispatch_library.as_ref().is_some_and(|library|
            library.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6
        ))?;
    }
    if program.namespace_library.is_some() {
        namespace_region::append(&mut object, text, program.dispatch_library.as_ref().is_some_and(|library|
            library.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6
        ))?;
    }
    if program.ram_page_pool_library.is_some() {
        ram_page_region::append(&mut object, text)?;
    }
    if let Some(requirement) = &program.file_position_library {
        let profile = file_region::FileRegionProfile::from_interface(requirement.interface)?;
        file_region::append(&mut object, text, profile)?;
    }
    append_linkable_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
        &static_image_process_enter(
            program.kernel_library.is_some(),
            program.kernel_library.as_ref().is_some_and(|r| r.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::FileOpenDecisionV1),
            program.time_library.is_some(),
            program.image_library.is_some(),
            program.dispatch_library.is_some(),
            program.dispatch_library.as_ref().map(|r| crate::systems_numeric_library::NumericPolicy::from_interface(r.interface)).transpose()?.unwrap_or(crate::systems_numeric_library::NumericPolicy::LinuxDispatch),
            program.process_identity_library.is_some(),
            crate::systems_numeric_library::uses_retained_identity(program),
            crate::systems_numeric_library::uses_structured_identity(program),
            program.file_position_library.is_some(),
            program.ram_page_pool_library.is_some(),
            program.inode_mode_library.is_some(),
            program.namespace_library.is_some(),
        )?,
    )?;
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY_SYMBOL,
        &static_image_process_restore_entry()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_INITIAL_RANDOM_SYMBOL,
        &static_image_revoke_entropy_initial_random()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_ACQUISITION_SYMBOL,
        &static_image_revoke_entropy_acquisition()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_HANDOFF_SYMBOL,
        &static_image_revoke_entropy_handoff()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_RESTORE_PREPARED_PROCESS_SYMBOL,
        &static_image_restore_prepared_process()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_RANDOM_SLOT_SYMBOL,
        &static_image_revoke_stack_random_slot()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_HARDWARE_CAPABILITIES_SYMBOL,
        &static_image_remove_stack_hardware_capabilities()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PLATFORM_SYMBOL,
        &static_image_remove_stack_platform()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CLOCK_TICK_RATE_SYMBOL,
        &static_image_remove_stack_clock_tick_rate()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_SECURE_MODE_SYMBOL,
        &static_image_remove_stack_secure_mode()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CREDENTIALS_SYMBOL,
        &static_image_remove_stack_credentials()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_EXECUTABLE_NAME_SYMBOL,
        &static_image_remove_stack_executable_name()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PROGRAM_HEADERS_SYMBOL,
        &static_image_remove_stack_program_headers()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_ENTRY_FACT_SYMBOL,
        &static_image_remove_stack_entry_fact()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PAGESZ_SYMBOL,
        &static_image_remove_stack_pagesz()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_COPY_PAYLOAD_SYMBOL,
        &static_image_copy_payload()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_SEAL_PERMISSIONS_SYMBOL,
        &static_image_seal_permissions()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ATTACH_ADDRESS_SPACE_SYMBOL,
        &static_image_attach_address_space()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ACTIVATE_ADDRESS_SPACE_SYMBOL,
        &static_image_activate_address_space()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_RESTORE_ADDRESS_SPACE_SYMBOL,
        &static_image_restore_address_space()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTRY_SYMBOL,
        &static_image_entry_authority(false)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_SLOT_SYMBOL,
        &static_image_stack_slot_authority(false)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_SLOT_SYMBOL,
        &static_image_stack_slot_authority(true)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTRY_SYMBOL,
        &static_image_entry_authority(true)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_DETACH_ADDRESS_SPACE_SYMBOL,
        &static_image_detach_address_space()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL,
        &static_image_revoke_initial_leaves()?,
    );
    append_translation_functions(&mut object, text)?;
    append_critical_functions(&mut object, text);
    append_atomic_functions(&mut object, text);
    append_local_notification_functions(&mut object, text);
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
        &monotonic_clock_now()?,
    );
    append_deadline_functions(&mut object, text)?;
    append_context_functions(&mut object, text)?;
    append_user_functions(&mut object, text)?;
    append_function(
        &mut object,
        text,
        "topal_x86_systems_debug_break",
        &[X86Instruction::Breakpoint, X86Instruction::Return],
    );
    append_function(
        &mut object,
        text,
        "topal_x86_systems_interrupt_return",
        &[X86Instruction::InterruptReturn],
    );
    append_function(
        &mut object,
        text,
        "topal_x86_systems_fatal",
        &[
            X86Instruction::DisableInterrupts,
            X86Instruction::Halt,
            X86Instruction::JumpBackToHalt,
        ],
    );

    if program.time_library.is_some()
        || program.image_library.is_some()
        || program.dispatch_library.is_some()
        || program.process_identity_library.is_some()
    {
        append_encoded_function(
            &mut object,
            text,
            process_runtime::numeric::ALLOCATE_SYMBOL,
            &process_runtime::numeric::allocator()?,
        );
        append_encoded_function(
            &mut object,
            text,
            process_runtime::numeric::ABORT_SYMBOL,
            &[0xfa, 0xf4, 0xeb, 0xfd],
        );
        let arena = object.add_section(
            Vec::new(),
            b".bss.topal.numeric".to_vec(),
            SectionKind::UninitializedData,
        );
        let offset = object.append_section_bss(arena, process_runtime::numeric::ARENA_BYTES, 4096);
        object.add_symbol(Symbol {
            name: process_runtime::numeric::ARENA_SYMBOL.as_bytes().to_vec(),
            value: offset,
            size: process_runtime::numeric::ARENA_BYTES,
            kind: SymbolKind::Data,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(arena),
            flags: SymbolFlags::None,
        });
    }
    if crate::systems_numeric_library::uses_retained_identity(program) {
        let state = object.add_section(
            Vec::new(),
            b".bss.topal.numeric-state".to_vec(),
            SectionKind::UninitializedData,
        );
        let offset = object.append_section_bss(state, 4096, 4096);
        object.add_symbol(Symbol {
            name: crate::systems_numeric_library::STATE_SYMBOL
                .as_bytes()
                .to_vec(),
            value: offset,
            size: 4096,
            kind: SymbolKind::Data,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(state),
            flags: SymbolFlags::None,
        });
    }
    let storage = object.add_section(
        Vec::new(),
        X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION.as_bytes().to_vec(),
        SectionKind::UninitializedData,
    );
    let storage_offset = object.append_section_bss(
        storage,
        plan.bootstrap_placement.capacity_bytes,
        plan.bootstrap_placement.alignment_bytes,
    );
    object.add_symbol(Symbol {
        name: b"topal_bootstrap_storage".to_vec(),
        value: storage_offset,
        size: plan.bootstrap_placement.capacity_bytes,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(storage),
        flags: SymbolFlags::None,
    });
    if let Some(requirement) = &program.region_library
        && requirement.interface != topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionTailActionV1 {
        let framed = requirement.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1;
        root_region::append(
            &mut object,
            text,
            plan.bootstrap_placement.capacity_bytes,
            framed,
        )?;
    }
    let dma_storage = object.add_section(
        Vec::new(),
        X86_SYSTEMS_DMA_STORAGE_SECTION.as_bytes().to_vec(),
        SectionKind::UninitializedData,
    );
    let control_offset = object.append_section_bss(dma_storage, 4096, 4096);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL.as_bytes().to_vec(),
        value: control_offset,
        size: 4096,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(dma_storage),
        flags: SymbolFlags::None,
    });
    let payload_offset = object.append_section_bss(dma_storage, 4096, 4096);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL.as_bytes().to_vec(),
        value: payload_offset,
        size: 4096,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(dma_storage),
        flags: SymbolFlags::None,
    });

    let entropy_storage = object.add_section(
        Vec::new(),
        X86_SYSTEMS_ENTROPY_STORAGE_SECTION.as_bytes().to_vec(),
        SectionKind::UninitializedData,
    );
    let sample_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL.as_bytes().to_vec(),
        value: sample_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let pool_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL.as_bytes().to_vec(),
        value: pool_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let generator_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL.as_bytes().to_vec(),
        value: generator_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let output_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL.as_bytes().to_vec(),
        value: output_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let user_random_service_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL
            .as_bytes()
            .to_vec(),
        value: user_random_service_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });

    let process_storage = object.add_section(
        Vec::new(),
        X86_SYSTEMS_STATIC_PROCESS_STORAGE_SECTION
            .as_bytes()
            .to_vec(),
        SectionKind::UninitializedData,
    );
    let brk_reserve_size = 34 * 4096;
    let brk_reserve_offset = object.append_section_bss(process_storage, brk_reserve_size, 4096);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_STATIC_PROCESS_BRK_RESERVE_SYMBOL
            .as_bytes()
            .to_vec(),
        value: brk_reserve_offset,
        size: brk_reserve_size,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(process_storage),
        flags: SymbolFlags::None,
    });
    let exec_snapshot_offset = object.append_section_bss(process_storage, 0xb000, 16);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_STATIC_PROCESS_EXEC_SNAPSHOT_SYMBOL
            .as_bytes()
            .to_vec(),
        value: exec_snapshot_offset,
        size: 0xb000,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(process_storage),
        flags: SymbolFlags::None,
    });

    let descriptor_table_offset = object.append_section_bss(
        process_storage,
        process_runtime::children::STORAGE_BYTES,
        4096,
    );
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL
            .as_bytes()
            .to_vec(),
        value: descriptor_table_offset,
        size: process_runtime::children::STORAGE_BYTES,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(process_storage),
        flags: SymbolFlags::None,
    });

    let note = object.add_section(
        Vec::new(),
        X86_SYSTEMS_PROVIDER_NOTE_SECTION.as_bytes().to_vec(),
        SectionKind::Note,
    );
    let note_description = provider_note(&plan);
    object.append_section_data(note, &elf_note(note_description.as_bytes())?, 4);

    let bytes = object.write().map_err(|error| {
        CompileError::Tool(format!("cannot encode systems provider ELF: {error}"))
    })?;
    Ok(GeneratedSystemsProviderObject { plan, bytes })
}

fn append_deadline_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (X86_SYSTEMS_DEADLINE_AFTER_SYMBOL, deadline_after()?),
        (X86_SYSTEMS_DEADLINE_ARM_SYMBOL, deadline_arm()?),
        (X86_SYSTEMS_DEADLINE_WAIT_SYMBOL, deadline_wait()),
        (X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL, deadline_complete()?),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}

fn append_context_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (X86_SYSTEMS_CONTEXT_CREATE_SYMBOL, context_create()?),
        (X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL, context_transfer()?),
        (
            X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL,
            context_await_deadline_preemption()?,
        ),
        (
            X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL,
            context_preempt_current()?,
        ),
        (X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL, context_retire()?),
        (X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL, context_reclaim()?),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}

fn append_user_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL,
            user_prepare_initial_address_space()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL,
            user_verify_initial_mapping()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL,
            user_map_initial_frame()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
            user_store_initial_mapping_byte()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL,
            user_install_initial_entry_trampoline()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
            user_protect_initial_mapping_read_only()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL,
            user_load_initial_mapping_byte()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL,
            user_verify_initial_entry_trampoline()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
            user_permit_initial_mapping_execute()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL,
            user_seal_initial_mapping_immutable_data()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL,
            user_permit_initial_mapping_writable_data()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
            user_reserve_initial_anonymous_mapping()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
            user_release_initial_anonymous_mapping()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
            user_unmap_initial_frame()?,
        ),
        (
            X86_SYSTEMS_USER_PREPARE_SYMBOL,
            user_prepare_initial_probe()?,
        ),
        (X86_SYSTEMS_USER_ENTER_SYMBOL, user_enter_initial_context()?),
        (
            X86_SYSTEMS_USER_COMPLETE_SYMBOL,
            user_complete_initial_backing()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL,
            user_reclaim_initial_address_space()?,
        ),
        (
            X86_SYSTEMS_USER_SYSCALL_SYMBOL,
            user_dispatch_initial_syscall()?,
        ),
        (
            X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
            user_copy_page_fault_entry()?,
        ),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}
