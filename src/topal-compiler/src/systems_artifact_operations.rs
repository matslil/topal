fn encode_operations(
    encoder: &mut RootEncoder,
    operations: &[CompilerSystemsOperation],
) -> Result<(), CompileError> {
    for operation in operations {
        match operation {
            CompilerSystemsOperation::DescribeBootMemory { .. } => {
                encoder.describe_boot_memory()?;
            }
            CompilerSystemsOperation::CreateFrameAllocator { .. } => {
                encoder.create_frame_allocator()?;
            }
            CompilerSystemsOperation::AllocatePhysicalFrames { request, .. } => {
                encoder.allocate_physical_frames(request.frame_count, request.alignment_frames)?;
            }
            CompilerSystemsOperation::ReserveStaticImageHierarchy { .. } => {
                encoder.reserve_static_image_hierarchy()?;
            }
            CompilerSystemsOperation::ReleasePhysicalFrames => {
                encoder.release_physical_frames()?;
            }
            CompilerSystemsOperation::PrepareStaticImageConstruction { .. } => {
                encoder.prepare_static_image_construction()?;
            }
            CompilerSystemsOperation::ZeroStaticImageConstructionPayload => {
                encoder.zero_static_image_payload()?;
            }
            CompilerSystemsOperation::ZeroStaticImageStackBacking => {
                encoder.zero_static_image_stack_backing()?;
            }
            CompilerSystemsOperation::PublishStaticImageStackLeaf => {
                encoder.publish_static_image_stack_leaf()?;
            }
            CompilerSystemsOperation::InitializeStaticImageStackEntry => {
                encoder.initialize_static_image_stack_entry()?;
            }
            CompilerSystemsOperation::AddStaticImageStackPageSize => {
                encoder.add_static_image_stack_pagesz()?;
            }
            CompilerSystemsOperation::AddStaticImageStackEntryFact => {
                encoder.add_static_image_stack_entry_fact()?;
            }
            CompilerSystemsOperation::AddStaticImageStackProgramHeaders => {
                encoder.add_static_image_stack_program_headers()?;
            }
            CompilerSystemsOperation::AddStaticImageStackExecutableName => {
                encoder.add_static_image_stack_executable_name()?;
            }
            CompilerSystemsOperation::AddStaticImageStackCredentials => {
                encoder.add_static_image_stack_credentials()?;
            }
            CompilerSystemsOperation::AddStaticImageStackSecureMode => {
                encoder.add_static_image_stack_secure_mode()?;
            }
            CompilerSystemsOperation::AddStaticImageStackClockTickRate => {
                encoder.add_static_image_stack_clock_tick_rate()?;
            }
            CompilerSystemsOperation::AddStaticImageStackPlatform => {
                encoder.add_static_image_stack_platform()?;
            }
            CompilerSystemsOperation::AddStaticImageStackHardwareCapabilities => {
                encoder.add_static_image_stack_hardware_capabilities()?;
            }
            CompilerSystemsOperation::AddStaticImageStackLoadMetadata => {
                encoder.add_static_image_stack_load_metadata()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageStackRandomSlot => {
                encoder.authorize_static_image_stack_random_slot()?;
            }
            CompilerSystemsOperation::RetainStaticImagePreparedProcess => {
                encoder.retain_static_image_prepared_process()?;
            }
            CompilerSystemsOperation::QualifyStaticImageRetainedDisjointWork => {
                encoder.qualify_static_image_retained_disjoint_work()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageEntropyHandoff => {
                encoder.authorize_static_image_entropy_handoff()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageEntropyAcquisition => {
                encoder.authorize_static_image_entropy_acquisition()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyDevice => {
                encoder.qualify_static_image_entropy_device()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyTransport => {
                encoder.qualify_static_image_entropy_transport()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyNegotiation => {
                encoder.qualify_static_image_entropy_negotiation()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyQueue => {
                encoder.qualify_static_image_entropy_queue()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyBusMaster => {
                encoder.qualify_static_image_entropy_bus_master()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyDmaBuffers => {
                encoder.qualify_static_image_entropy_dma_buffers()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropySplitRing => {
                encoder.qualify_static_image_entropy_split_ring()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyDmaPreparation => {
                encoder.qualify_static_image_entropy_dma_preparation()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyQueueAddresses => {
                encoder.qualify_static_image_entropy_queue_addresses()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyQueueActivation => {
                encoder.qualify_static_image_entropy_queue_activation()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyRequestCompletion => {
                encoder.qualify_static_image_entropy_request_completion()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropySampleCapture => {
                encoder.qualify_static_image_entropy_sample_capture()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropySampleQuality => {
                encoder.qualify_static_image_entropy_sample_quality()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyPoolLifecycle => {
                encoder.qualify_static_image_entropy_pool_lifecycle()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle => {
                encoder.qualify_static_image_entropy_generator_lifecycle()?;
            }
            CompilerSystemsOperation::DeliverStaticImageEntropyInitialRandom => {
                encoder.deliver_static_image_entropy_initial_random()?;
            }
            CompilerSystemsOperation::EnterStaticImageProcess => {
                encoder.enter_static_image_process()?;
            }
            CompilerSystemsOperation::RestoreStaticImageProcessEntry => {
                encoder.restore_static_image_process_entry()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntropyInitialRandom => {
                encoder.revoke_static_image_entropy_initial_random()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntropyAcquisition => {
                encoder.revoke_static_image_entropy_acquisition()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntropyHandoff => {
                encoder.revoke_static_image_entropy_handoff()?;
            }
            CompilerSystemsOperation::RestoreStaticImagePreparedProcess => {
                encoder.restore_static_image_prepared_process()?;
            }
            CompilerSystemsOperation::RevokeStaticImageStackRandomSlot => {
                encoder.revoke_static_image_stack_random_slot()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackLoadMetadata => {
                encoder.remove_static_image_stack_load_metadata()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackHardwareCapabilities => {
                encoder.remove_static_image_stack_hardware_capabilities()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackPlatform => {
                encoder.remove_static_image_stack_platform()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackClockTickRate => {
                encoder.remove_static_image_stack_clock_tick_rate()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackSecureMode => {
                encoder.remove_static_image_stack_secure_mode()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackCredentials => {
                encoder.remove_static_image_stack_credentials()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackExecutableName => {
                encoder.remove_static_image_stack_executable_name()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackProgramHeaders => {
                encoder.remove_static_image_stack_program_headers()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackEntryFact => {
                encoder.remove_static_image_stack_entry_fact()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackPageSize => {
                encoder.remove_static_image_stack_pagesz()?;
            }
            CompilerSystemsOperation::ClearStaticImageStackEntry => {
                encoder.clear_static_image_stack_entry()?;
            }
            CompilerSystemsOperation::RevokeStaticImageStackLeaf => {
                encoder.revoke_static_image_stack_leaf()?;
            }
            CompilerSystemsOperation::CopyStaticImageConstructionPayload => {
                encoder.copy_static_image_payload()?;
            }
            CompilerSystemsOperation::SealStaticImageConstructionPermissions => {
                encoder.seal_static_image_permissions()?;
            }
            CompilerSystemsOperation::AttachStaticImageAddressSpace => {
                encoder.attach_static_image_address_space()?;
            }
            CompilerSystemsOperation::ActivateStaticImageAddressSpace => {
                encoder.activate_static_image_address_space()?;
            }
            CompilerSystemsOperation::RestoreStaticImageAddressSpace => {
                encoder.restore_static_image_address_space()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageEntry => {
                encoder.authorize_static_image_entry()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageStackSlot => {
                encoder.authorize_static_image_stack_slot()?;
            }
            CompilerSystemsOperation::RevokeStaticImageStackSlot => {
                encoder.revoke_static_image_stack_slot()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntry => {
                encoder.revoke_static_image_entry()?;
            }
            CompilerSystemsOperation::DetachStaticImageAddressSpace => {
                encoder.detach_static_image_address_space()?;
            }
            CompilerSystemsOperation::ReclaimStaticImageConstructionHierarchy => {
                encoder.reclaim_static_image_hierarchy()?;
            }
            CompilerSystemsOperation::RevokeStaticImageConstructionMetadata => {
                encoder.revoke_static_image_construction()?;
            }
            CompilerSystemsOperation::ReclaimStaticImageConstructionPayload => {
                encoder.reclaim_static_image_payload()?;
            }
            CompilerSystemsOperation::MapKernelFrames { request, .. } => {
                encoder.map_kernel_frames(*request)?;
            }
            CompilerSystemsOperation::KernelMappingStoreByte {
                offset_bytes,
                value,
            } => encoder.store_kernel_mapping_byte(*offset_bytes, *value)?,
            CompilerSystemsOperation::KernelMappingLoadByteEquals {
                offset_bytes,
                expected,
                ..
            } => encoder.load_kernel_mapping_byte_equals(*offset_bytes, *expected)?,
            CompilerSystemsOperation::UnmapKernelFrames => encoder.unmap_kernel_frames()?,
            CompilerSystemsOperation::BeginTranslationUpdate { request, .. } => {
                encoder.begin_translation(*request)?;
            }
            CompilerSystemsOperation::CommitTranslationUpdate { .. } => {
                encoder.commit_translation()?;
            }
            CompilerSystemsOperation::ActivateTranslationSpace { .. } => {
                encoder.activate_translation()?;
            }
            CompilerSystemsOperation::BeginTranslationEdit { kind, .. } => {
                encoder.begin_translation_edit(*kind)?;
            }
            CompilerSystemsOperation::MapTranslationFrames { request, .. } => {
                encoder.map_translation_frames(*request)?;
            }
            CompilerSystemsOperation::UnmapTranslationMapping => {
                encoder.unmap_translation_mapping()?;
            }
            CompilerSystemsOperation::CommitTranslationEdit { kind, .. } => {
                encoder.commit_translation_edit(*kind)?;
            }
            CompilerSystemsOperation::EnterCritical { domain, .. } => {
                encoder.enter_critical(*domain)?;
            }
            CompilerSystemsOperation::RestoreCritical { domain } => {
                encoder.restore_critical(*domain)?;
            }
            CompilerSystemsOperation::ConsoleWrite { text } => encoder.console_write(text),
            CompilerSystemsOperation::DebugBreak => encoder.call(ProviderSymbol::DebugBreak),
            CompilerSystemsOperation::BootstrapAllocate { request } => {
                encoder.allocate_bootstrap_region(*request)?;
            }
            CompilerSystemsOperation::BootstrapStoreByte {
                offset_bytes,
                value,
            } => encoder.store_bootstrap_byte(*offset_bytes, *value)?,
            CompilerSystemsOperation::ApplyBootstrapRegionLibrary => {
                encoder.apply_region_library()?;
            }
            CompilerSystemsOperation::BootstrapLoadByteEquals {
                offset_bytes,
                expected,
                ..
            } => encoder.load_bootstrap_byte_equals(*offset_bytes, *expected)?,
            CompilerSystemsOperation::AtomicWordCreate { request } => {
                encoder.create_atomic_word(*request)?;
            }
            CompilerSystemsOperation::AtomicCompareExchangeEquals {
                expected,
                desired,
                success_order,
                failure_order,
                ..
            } => encoder.compare_exchange_atomic_word(
                *expected,
                *desired,
                *success_order,
                *failure_order,
            )?,
            CompilerSystemsOperation::AtomicLoadEquals {
                order, expected, ..
            } => encoder.load_atomic_word_equals(*order, *expected)?,
            CompilerSystemsOperation::AtomicWordEnd => encoder.end_atomic_word()?,
            CompilerSystemsOperation::BootstrapRelease => encoder.release_bootstrap_region()?,
            CompilerSystemsOperation::SendLocalNotification => encoder.send_local_notification()?,
            CompilerSystemsOperation::WaitLocalNotification => encoder.wait_local_notification()?,
            CompilerSystemsOperation::CompleteLocalNotification => {
                encoder.complete_local_notification()?;
            }
            CompilerSystemsOperation::ObserveMonotonicClock => {
                encoder.observe_monotonic_clock()?;
            }
            CompilerSystemsOperation::ObservePciConfigurationDword { request } => {
                encoder.observe_pci_configuration_dword(*request)?;
            }
            CompilerSystemsOperation::UpdatePciConfigurationWord { request } => {
                encoder.update_pci_configuration_word(*request)?;
            }
            CompilerSystemsOperation::CreateDmaBuffer { request } => {
                encoder.create_dma_buffer(*request)?;
            }
            CompilerSystemsOperation::BindDmaBufferReference { request } => {
                encoder.bind_dma_buffer_reference(*request)?;
            }
            CompilerSystemsOperation::StoreDmaBufferInteger { request } => {
                encoder.store_dma_buffer_integer(*request)?;
            }
            CompilerSystemsOperation::PrepareDmaBufferForDevice { request } => {
                encoder.prepare_dma_buffer_for_device(*request)?;
            }
            CompilerSystemsOperation::CompleteDmaBufferForCpu { request } => {
                encoder.complete_dma_buffer_for_cpu(*request)?;
            }
            CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request } => {
                encoder.load_dma_buffer_integer(*request)?;
            }
            CompilerSystemsOperation::CaptureEntropySample { request } => {
                encoder.capture_entropy_sample(*request)?;
            }
            CompilerSystemsOperation::AssessEntropySampleQuality { request } => {
                encoder.assess_entropy_sample_quality(*request)?;
            }
            CompilerSystemsOperation::CreateEntropyPool { request } => {
                encoder.create_entropy_pool(*request)?;
            }
            CompilerSystemsOperation::MixEntropyPoolSample { request } => {
                encoder.mix_entropy_pool_sample(*request)?;
            }
            CompilerSystemsOperation::CreditEntropyPoolContribution { request } => {
                encoder.credit_entropy_pool_contribution(*request)?;
            }
            CompilerSystemsOperation::InitializeEntropyGenerator { request } => {
                encoder.initialize_entropy_generator(*request)?;
            }
            CompilerSystemsOperation::ExtractEntropyGenerator { request } => {
                encoder.extract_entropy_generator(*request)?;
            }
            CompilerSystemsOperation::DeliverInitialAuxvRandom { request } => {
                encoder.deliver_initial_auxv_random(*request)?;
            }
            CompilerSystemsOperation::ReserveInitialUserRandom { request } => {
                encoder.reserve_initial_user_random(*request)?;
            }
            CompilerSystemsOperation::BindReusableUserRandom { request } => {
                encoder.bind_reusable_user_random(*request)?;
            }
            CompilerSystemsOperation::ExpandSmallUserRandomRequests { request } => {
                encoder.expand_small_user_random_requests(*request)?;
            }
            CompilerSystemsOperation::ExpandLargeUserRandomRequests { request } => {
                encoder.expand_large_user_random_requests(*request)?;
            }
            CompilerSystemsOperation::PublishPciRegionDmaReference { request } => {
                encoder.publish_pci_region_dma_reference(*request)?;
            }
            CompilerSystemsOperation::AccessPciRegionRegister { request } => {
                encoder.access_pci_region_register(*request)?;
            }
            CompilerSystemsOperation::ConstructDeadline {
                duration_nanoseconds,
            } => encoder.construct_deadline(*duration_nanoseconds)?,
            CompilerSystemsOperation::ArmDeadline => encoder.arm_deadline()?,
            CompilerSystemsOperation::WaitDeadline => encoder.wait_deadline()?,
            CompilerSystemsOperation::CompleteDeadline => encoder.complete_deadline()?,
            CompilerSystemsOperation::CreateKernelContext => encoder.create_kernel_context()?,
            CompilerSystemsOperation::CreateKernelCompletion => {
                encoder.create_kernel_completion()?;
            }
            CompilerSystemsOperation::CreateRunnableQueue { capacity } => {
                encoder.create_runnable_queue(*capacity)?;
            }
            CompilerSystemsOperation::EnqueueRunnableContext => {
                encoder.enqueue_runnable_context()?;
            }
            CompilerSystemsOperation::DequeueRunnableContext => {
                encoder.dequeue_runnable_context()?;
            }
            CompilerSystemsOperation::TransferKernelContextUntilDeadline => {
                encoder.transfer_kernel_context_until_deadline()?;
            }
            CompilerSystemsOperation::TransferKernelContextUntilCompletion => {
                encoder.transfer_kernel_context_until_completion()?;
            }
            CompilerSystemsOperation::TakePreemptedKernelContext => {
                encoder.take_preempted_kernel_context()?;
            }
            CompilerSystemsOperation::TransferKernelContext => encoder.transfer_kernel_context()?,
            CompilerSystemsOperation::AwaitDeadlinePreemption => {
                encoder.await_deadline_preemption()?;
            }
            CompilerSystemsOperation::AwaitKernelCompletion => {
                encoder.await_kernel_completion()?;
            }
            CompilerSystemsOperation::SignalBlockedKernelCompletion => {
                encoder.signal_blocked_kernel_completion()?;
            }
            CompilerSystemsOperation::ReclaimKernelContext => encoder.reclaim_kernel_context()?,
            CompilerSystemsOperation::ConsumeKernelCompletion => {
                encoder.consume_kernel_completion()?;
            }
            CompilerSystemsOperation::ConsumeEmptyRunnableQueue => {
                encoder.consume_empty_runnable_queue()?;
            }
            CompilerSystemsOperation::PrepareUserAddressSpace => {
                encoder.prepare_user_address_space()?;
            }
            CompilerSystemsOperation::VerifyUserAddressSpaceMapping => {
                encoder.verify_user_address_space_mapping()?;
            }
            CompilerSystemsOperation::MapUserAddressSpace => {
                encoder.map_user_address_space()?;
            }
            CompilerSystemsOperation::StoreUserMappingByte => {
                encoder.store_user_mapping_byte()?;
            }
            CompilerSystemsOperation::InstallUserMappingEntryTrampoline => {
                encoder.install_user_mapping_entry_trampoline()?;
            }
            CompilerSystemsOperation::ProtectUserMappingReadOnly => {
                encoder.protect_user_mapping_read_only()?;
            }
            CompilerSystemsOperation::LoadUserMappingByte => {
                encoder.load_user_mapping_byte()?;
            }
            CompilerSystemsOperation::VerifyUserMappingEntryTrampoline => {
                encoder.verify_user_mapping_entry_trampoline()?;
            }
            CompilerSystemsOperation::PermitUserMappingExecute => {
                encoder.permit_user_mapping_execute()?;
            }
            CompilerSystemsOperation::SealUserMappingImmutableData => {
                encoder.seal_user_mapping_immutable_data()?;
            }
            CompilerSystemsOperation::PermitUserMappingWritableData => {
                encoder.permit_user_mapping_writable_data()?;
            }
            CompilerSystemsOperation::ReserveAnonymousUserMapping => {
                encoder.reserve_anonymous_user_mapping()?;
            }
            CompilerSystemsOperation::ReleaseAnonymousUserMapping => {
                encoder.release_anonymous_user_mapping()?;
            }
            CompilerSystemsOperation::UnmapUserAddressSpace => {
                encoder.unmap_user_address_space()?;
            }
            CompilerSystemsOperation::PrepareUserProbe { process_identity } => {
                encoder.prepare_user_probe(*process_identity)?;
            }
            CompilerSystemsOperation::EnterUserContext => encoder.enter_user_context()?,
            CompilerSystemsOperation::CompleteUserExecution => {
                encoder.complete_user_execution()?;
            }
            CompilerSystemsOperation::ReclaimUserAddressSpace => {
                encoder.reclaim_user_address_space()?;
            }
            CompilerSystemsOperation::CopyInitialUnameToUser
            | CompilerSystemsOperation::CopyInitialTimespecFromUser => {
                return Err(CompileError::Tool(
                    "x86 user transfer lowers only inside the sealed user-syscall entry".into(),
                ));
            }
        }
        if encoder.frame_policy.is_some() {
            encoder.frame_after_operation(operation)?;
        }
    }
    encoder.complete()
}

fn undefined_provider_symbol(object: &mut Object<'_>, symbol: ProviderSymbol) -> SymbolId {
    object.add_symbol(Symbol {
        name: symbol.name().as_bytes().to_vec(),
        value: 0,
        size: 0,
        kind: symbol.kind(),
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Undefined,
        flags: SymbolFlags::None,
    })
}

fn append_root(
    object: &mut Object<'_>,
    section: SectionId,
    name: &str,
    encoded: &RootEncoder,
    provider_symbols: ProviderSymbols,
) -> Result<SymbolId, CompileError> {
    let start = object.append_section_data(section, &encoded.bytes, 16);
    let root_symbol = object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value: start,
        size: encoded.bytes.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    for (offset, name, addend, call) in &encoded.frame_relocations {
        let symbol = object.symbol_id(name.as_bytes()).unwrap_or_else(|| {
            object.add_symbol(Symbol {
                name: name.as_bytes().to_vec(),
                value: 0,
                size: 0,
                kind: if *call {
                    SymbolKind::Text
                } else {
                    SymbolKind::Data
                },
                scope: SymbolScope::Linkage,
                weak: false,
                section: SymbolSection::Undefined,
                flags: SymbolFlags::None,
            })
        });
        object
            .add_relocation(
                section,
                Relocation {
                    offset: start + offset,
                    symbol,
                    addend: if *call {
                        -4
                    } else {
                        i64::try_from(*addend).map_err(|_| {
                            CompileError::Tool("root token relocation overflow".into())
                        })? - 4
                    },
                    flags: RelocationFlags::Generic {
                        kind: RelocationKind::Relative,
                        encoding: if *call {
                            RelocationEncoding::X86Branch
                        } else {
                            RelocationEncoding::X86RipRelative
                        },
                        size: 32,
                    },
                },
            )
            .map_err(|e| CompileError::Tool(e.to_string()))?;
    }
    if !encoded.source_library_relocations.is_empty() {
        let dependency = object.add_symbol(Symbol {
            name: encoded.source_library_export.as_bytes().to_vec(),
            value: 0,
            size: 0,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Undefined,
            flags: SymbolFlags::None,
        });
        for offset in &encoded.source_library_relocations {
            object
                .add_relocation(
                    section,
                    Relocation {
                        offset: start + offset,
                        symbol: dependency,
                        addend: -4,
                        flags: RelocationFlags::Generic {
                            kind: RelocationKind::Relative,
                            encoding: RelocationEncoding::X86RipRelative,
                            size: 32,
                        },
                    },
                )
                .map_err(|error| {
                    CompileError::Tool(format!("source-library relocation: {error}"))
                })?;
        }
    }
    for (offset, dependency) in &encoded.library_relocations {
        let (name, kind) = match dependency {
            RootLibraryDependency::RegionApply => (
                crate::systems_provider_object::root_region::APPLY,
                SymbolKind::Text,
            ),
            RootLibraryDependency::RegionFinalize => (
                crate::systems_provider_object::root_region::FINALIZE,
                SymbolKind::Text,
            ),
            RootLibraryDependency::Code => (
                crate::systems_numeric_library::EXPORT_SYMBOL,
                SymbolKind::Text,
            ),
            RootLibraryDependency::ImageCode => (
                crate::systems_numeric_library::NumericPolicy::Elf64Header.native_export(),
                SymbolKind::Text,
            ),
            RootLibraryDependency::DispatchCode => (
                encoded
                    .dispatch_policy
                    .ok_or_else(|| {
                        CompileError::Tool("dispatch relocation requires selected policy".into())
                    })?
                    .native_export(),
                SymbolKind::Text,
            ),
            RootLibraryDependency::ProcessIdentityCode => (
                crate::systems_numeric_library::NumericPolicy::ProcessIdentity.native_export(),
                SymbolKind::Text,
            ),
            RootLibraryDependency::RetainedIdentityCode => (
                crate::systems_numeric_library::NumericPolicy::RetainedIdentity.native_export(),
                SymbolKind::Text,
            ),
            RootLibraryDependency::StructuredIdentityCode => (
                crate::systems_numeric_library::NumericPolicy::StructuredIdentity.native_export(),
                SymbolKind::Text,
            ),
            RootLibraryDependency::RetainedState => (
                crate::systems_numeric_library::STATE_SYMBOL,
                SymbolKind::Data,
            ),
            RootLibraryDependency::Arena => ("topal_x86_systems_numeric_arena", SymbolKind::Data),
        };
        let symbol = object.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: 0,
            size: 0,
            kind,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Undefined,
            flags: SymbolFlags::None,
        });
        object
            .add_relocation(
                section,
                Relocation {
                    offset: start + offset,
                    symbol,
                    addend: -4,
                    flags: RelocationFlags::Generic {
                        kind: RelocationKind::Relative,
                        encoding: RelocationEncoding::X86RipRelative,
                        size: 32,
                    },
                },
            )
            .map_err(|e| CompileError::Tool(e.to_string()))?;
    }
    for relocation in &encoded.relocations {
        let symbol = match relocation.target {
            ProviderSymbol::DescribeBootMemory => provider_symbols.describe_boot_memory,
            ProviderSymbol::AllocatePhysicalFrames => provider_symbols.allocate_physical_frames,
            ProviderSymbol::PrepareStaticImage => provider_symbols.prepare_static_image,
            ProviderSymbol::ZeroStaticImagePayload => provider_symbols.zero_static_image_payload,
            ProviderSymbol::ZeroStaticImageStackBacking => {
                provider_symbols.zero_static_image_stack_backing
            }
            ProviderSymbol::PublishStaticImageStackLeaf => {
                provider_symbols.publish_static_image_stack_leaf
            }
            ProviderSymbol::InitializeStaticImageStackEntry => {
                provider_symbols.initialize_static_image_stack_entry
            }
            ProviderSymbol::AddStaticImageStackPageSize => {
                provider_symbols.add_static_image_stack_pagesz
            }
            ProviderSymbol::AddStaticImageStackEntryFact => {
                provider_symbols.add_static_image_stack_entry_fact
            }
            ProviderSymbol::AddStaticImageStackProgramHeaders => {
                provider_symbols.add_static_image_stack_program_headers
            }
            ProviderSymbol::AddStaticImageStackExecutableName => {
                provider_symbols.add_static_image_stack_executable_name
            }
            ProviderSymbol::AddStaticImageStackCredentials => {
                provider_symbols.add_static_image_stack_credentials
            }
            ProviderSymbol::AddStaticImageStackSecureMode => {
                provider_symbols.add_static_image_stack_secure_mode
            }
            ProviderSymbol::AddStaticImageStackClockTickRate => {
                provider_symbols.add_static_image_stack_clock_tick_rate
            }
            ProviderSymbol::AddStaticImageStackPlatform => {
                provider_symbols.add_static_image_stack_platform
            }
            ProviderSymbol::AddStaticImageStackHardwareCapabilities => {
                provider_symbols.add_static_image_stack_hardware_capabilities
            }
            ProviderSymbol::AddStaticImageStackLoadMetadata => {
                provider_symbols.add_static_image_stack_load_metadata
            }
            ProviderSymbol::AuthorizeStaticImageStackRandomSlot => {
                provider_symbols.authorize_static_image_stack_random_slot
            }
            ProviderSymbol::RetainStaticImagePreparedProcess => {
                provider_symbols.retain_static_image_prepared_process
            }
            ProviderSymbol::QualifyStaticImageRetainedDisjointWork => {
                provider_symbols.qualify_static_image_retained_disjoint_work
            }
            ProviderSymbol::AuthorizeStaticImageEntropyHandoff => {
                provider_symbols.authorize_static_image_entropy_handoff
            }
            ProviderSymbol::AuthorizeStaticImageEntropyAcquisition => {
                provider_symbols.authorize_static_image_entropy_acquisition
            }
            ProviderSymbol::QualifyStaticImageEntropyDevice => {
                provider_symbols.qualify_static_image_entropy_device
            }
            ProviderSymbol::QualifyStaticImageEntropyTransport => {
                provider_symbols.qualify_static_image_entropy_transport
            }
            ProviderSymbol::QualifyStaticImageEntropyNegotiation => {
                provider_symbols.qualify_static_image_entropy_negotiation
            }
            ProviderSymbol::QualifyStaticImageEntropyQueue => {
                provider_symbols.qualify_static_image_entropy_queue
            }
            ProviderSymbol::QualifyStaticImageEntropyBusMaster => {
                provider_symbols.qualify_static_image_entropy_bus_master
            }
            ProviderSymbol::QualifyStaticImageEntropyDmaBuffers => {
                provider_symbols.qualify_static_image_entropy_dma_buffers
            }
            ProviderSymbol::QualifyStaticImageEntropySplitRing => {
                provider_symbols.qualify_static_image_entropy_split_ring
            }
            ProviderSymbol::QualifyStaticImageEntropyDmaPreparation => {
                provider_symbols.qualify_static_image_entropy_dma_preparation
            }
            ProviderSymbol::QualifyStaticImageEntropyQueueAddresses => {
                provider_symbols.qualify_static_image_entropy_queue_addresses
            }
            ProviderSymbol::QualifyStaticImageEntropyQueueActivation => {
                provider_symbols.qualify_static_image_entropy_queue_activation
            }
            ProviderSymbol::QualifyStaticImageEntropyRequestCompletion => {
                provider_symbols.qualify_static_image_entropy_request_completion
            }
            ProviderSymbol::QualifyStaticImageEntropySampleCapture => {
                provider_symbols.qualify_static_image_entropy_sample_capture
            }
            ProviderSymbol::QualifyStaticImageEntropySampleQuality => {
                provider_symbols.qualify_static_image_entropy_sample_quality
            }
            ProviderSymbol::QualifyStaticImageEntropyPoolLifecycle => {
                provider_symbols.qualify_static_image_entropy_pool_lifecycle
            }
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle => {
                provider_symbols.qualify_static_image_entropy_generator_lifecycle
            }
            ProviderSymbol::DeliverStaticImageEntropyInitialRandom => {
                provider_symbols.deliver_static_image_entropy_initial_random
            }
            ProviderSymbol::EnterStaticImageProcess => provider_symbols.enter_static_image_process,
            ProviderSymbol::RestoreStaticImageProcessEntry => {
                provider_symbols.restore_static_image_process_entry
            }
            ProviderSymbol::RevokeStaticImageEntropyInitialRandom => {
                provider_symbols.revoke_static_image_entropy_initial_random
            }
            ProviderSymbol::RevokeStaticImageEntropyAcquisition => {
                provider_symbols.revoke_static_image_entropy_acquisition
            }
            ProviderSymbol::RevokeStaticImageEntropyHandoff => {
                provider_symbols.revoke_static_image_entropy_handoff
            }
            ProviderSymbol::RestoreStaticImagePreparedProcess => {
                provider_symbols.restore_static_image_prepared_process
            }
            ProviderSymbol::RevokeStaticImageStackRandomSlot => {
                provider_symbols.revoke_static_image_stack_random_slot
            }
            ProviderSymbol::RemoveStaticImageStackLoadMetadata => {
                provider_symbols.remove_static_image_stack_load_metadata
            }
            ProviderSymbol::RemoveStaticImageStackHardwareCapabilities => {
                provider_symbols.remove_static_image_stack_hardware_capabilities
            }
            ProviderSymbol::RemoveStaticImageStackPlatform => {
                provider_symbols.remove_static_image_stack_platform
            }
            ProviderSymbol::RemoveStaticImageStackClockTickRate => {
                provider_symbols.remove_static_image_stack_clock_tick_rate
            }
            ProviderSymbol::RemoveStaticImageStackSecureMode => {
                provider_symbols.remove_static_image_stack_secure_mode
            }
            ProviderSymbol::RemoveStaticImageStackCredentials => {
                provider_symbols.remove_static_image_stack_credentials
            }
            ProviderSymbol::RemoveStaticImageStackExecutableName => {
                provider_symbols.remove_static_image_stack_executable_name
            }
            ProviderSymbol::RemoveStaticImageStackProgramHeaders => {
                provider_symbols.remove_static_image_stack_program_headers
            }
            ProviderSymbol::RemoveStaticImageStackEntryFact => {
                provider_symbols.remove_static_image_stack_entry_fact
            }
            ProviderSymbol::RemoveStaticImageStackPageSize => {
                provider_symbols.remove_static_image_stack_pagesz
            }
            ProviderSymbol::ClearStaticImageStackEntry => {
                provider_symbols.clear_static_image_stack_entry
            }
            ProviderSymbol::RevokeStaticImageStackLeaf => {
                provider_symbols.revoke_static_image_stack_leaf
            }
            ProviderSymbol::CopyStaticImagePayload => provider_symbols.copy_static_image_payload,
            ProviderSymbol::SealStaticImagePermissions => {
                provider_symbols.seal_static_image_permissions
            }
            ProviderSymbol::AttachStaticImageAddressSpace => {
                provider_symbols.attach_static_image_address_space
            }
            ProviderSymbol::ActivateStaticImageAddressSpace => {
                provider_symbols.activate_static_image_address_space
            }
            ProviderSymbol::RestoreStaticImageAddressSpace => {
                provider_symbols.restore_static_image_address_space
            }
            ProviderSymbol::AuthorizeStaticImageEntry => {
                provider_symbols.authorize_static_image_entry
            }
            ProviderSymbol::AuthorizeStaticImageStackSlot => {
                provider_symbols.authorize_static_image_stack_slot
            }
            ProviderSymbol::RevokeStaticImageStackSlot => {
                provider_symbols.revoke_static_image_stack_slot
            }
            ProviderSymbol::RevokeStaticImageEntry => provider_symbols.revoke_static_image_entry,
            ProviderSymbol::DetachStaticImageAddressSpace => {
                provider_symbols.detach_static_image_address_space
            }
            ProviderSymbol::RevokeStaticImage => provider_symbols.revoke_static_image,
            ProviderSymbol::BeginTranslation => provider_symbols.begin_translation,
            ProviderSymbol::CommitTranslation => provider_symbols.commit_translation,
            ProviderSymbol::ActivateTranslation => provider_symbols.activate_translation,
            ProviderSymbol::BeginTranslationEdit => provider_symbols.begin_translation_edit,
            ProviderSymbol::MapTranslationFrames => provider_symbols.map_translation_frames,
            ProviderSymbol::UnmapTranslationMapping => provider_symbols.unmap_translation_mapping,
            ProviderSymbol::CommitTranslationEdit => provider_symbols.commit_translation_edit,
            ProviderSymbol::EnterCritical => provider_symbols.enter_critical,
            ProviderSymbol::RestoreCritical => provider_symbols.restore_critical,
            ProviderSymbol::AtomicWordCreate => provider_symbols.atomic_word_create,
            ProviderSymbol::AtomicWordCompareExchange => {
                provider_symbols.atomic_word_compare_exchange
            }
            ProviderSymbol::AtomicWordLoad => provider_symbols.atomic_word_load,
            ProviderSymbol::SendLocalNotification => provider_symbols.send_local_notification,
            ProviderSymbol::WaitLocalNotification => provider_symbols.wait_local_notification,
            ProviderSymbol::CompleteLocalNotification => {
                provider_symbols.complete_local_notification
            }
            ProviderSymbol::ObserveMonotonicClock => provider_symbols.observe_monotonic_clock,
            ProviderSymbol::ObservePciConfigurationDword => {
                provider_symbols.observe_pci_configuration_dword
            }
            ProviderSymbol::UpdatePciConfigurationWord => {
                provider_symbols.update_pci_configuration_word
            }
            ProviderSymbol::CreateDmaBuffer => provider_symbols.create_dma_buffer,
            ProviderSymbol::BindDmaBufferReference => provider_symbols.bind_dma_buffer_reference,
            ProviderSymbol::StoreDmaBufferInteger => provider_symbols.store_dma_buffer_integer,
            ProviderSymbol::PrepareDmaBufferForDevice => {
                provider_symbols.prepare_dma_buffer_for_device
            }
            ProviderSymbol::CompleteDmaBufferForCpu => provider_symbols.complete_dma_buffer_for_cpu,
            ProviderSymbol::LoadDmaBufferInteger => provider_symbols.load_dma_buffer_integer,
            ProviderSymbol::CaptureEntropySample => provider_symbols.capture_entropy_sample,
            ProviderSymbol::MixEntropyPoolSample => provider_symbols.mix_entropy_pool_sample,
            ProviderSymbol::InitializeEntropyGenerator => {
                provider_symbols.initialize_entropy_generator
            }
            ProviderSymbol::ExtractEntropyGenerator => provider_symbols.extract_entropy_generator,
            ProviderSymbol::DeliverInitialAuxvRandom => {
                provider_symbols.deliver_initial_auxv_random
            }
            ProviderSymbol::ReserveInitialUserRandom => {
                provider_symbols.reserve_initial_user_random
            }
            ProviderSymbol::BindReusableUserRandom => provider_symbols.bind_reusable_user_random,
            ProviderSymbol::PublishPciRegionDmaReference => {
                provider_symbols.publish_pci_region_dma_reference
            }
            ProviderSymbol::DmaVirtqueueControl => provider_symbols.dma_virtqueue_control,
            ProviderSymbol::DmaEntropyPayload => provider_symbols.dma_entropy_payload,
            ProviderSymbol::RawEntropySample => provider_symbols.raw_entropy_sample,
            ProviderSymbol::EntropyPoolState => provider_symbols.entropy_pool_state,
            ProviderSymbol::EntropyGeneratorKey => provider_symbols.entropy_generator_key,
            ProviderSymbol::EntropyOutput => provider_symbols.entropy_output,
            ProviderSymbol::EntropyUserRandomService => {
                provider_symbols.entropy_user_random_service
            }
            ProviderSymbol::StaticProcessBrkReserve => provider_symbols.static_process_brk_reserve,
            ProviderSymbol::StaticProcessExecSnapshot => {
                provider_symbols.static_process_exec_snapshot
            }
            ProviderSymbol::StaticProcessDescriptorTable => {
                provider_symbols.static_process_descriptor_table
            }
            ProviderSymbol::ReadPciRegionRegister => provider_symbols.read_pci_region_register,
            ProviderSymbol::ReadPciRegionRegisterEventually => {
                provider_symbols.read_pci_region_register_eventually
            }
            ProviderSymbol::WritePciRegionRegister => provider_symbols.write_pci_region_register,
            ProviderSymbol::ConstructDeadline => provider_symbols.construct_deadline,
            ProviderSymbol::ArmDeadline => provider_symbols.arm_deadline,
            ProviderSymbol::WaitDeadline => provider_symbols.wait_deadline,
            ProviderSymbol::CompleteDeadline => provider_symbols.complete_deadline,
            ProviderSymbol::CreateKernelContext => provider_symbols.create_kernel_context,
            ProviderSymbol::TransferKernelContext => provider_symbols.transfer_kernel_context,
            ProviderSymbol::AwaitDeadlinePreemption => provider_symbols.await_deadline_preemption,
            ProviderSymbol::PreemptCurrentKernelContext => {
                provider_symbols.preempt_current_kernel_context
            }
            ProviderSymbol::RetireKernelContext => provider_symbols.retire_kernel_context,
            ProviderSymbol::ReclaimKernelContext => provider_symbols.reclaim_kernel_context,
            ProviderSymbol::PrepareUserAddressSpace => provider_symbols.prepare_user_address_space,
            ProviderSymbol::VerifyUserAddressSpaceMapping => {
                provider_symbols.verify_user_address_space_mapping
            }
            ProviderSymbol::MapUserAddressSpace => provider_symbols.map_user_address_space,
            ProviderSymbol::StoreUserMappingByte => provider_symbols.store_user_mapping_byte,
            ProviderSymbol::InstallUserMappingEntryTrampoline => {
                provider_symbols.install_user_mapping_entry_trampoline
            }
            ProviderSymbol::ProtectUserMappingReadOnly => {
                provider_symbols.protect_user_mapping_read_only
            }
            ProviderSymbol::LoadUserMappingByte => provider_symbols.load_user_mapping_byte,
            ProviderSymbol::VerifyUserMappingEntryTrampoline => {
                provider_symbols.verify_user_mapping_entry_trampoline
            }
            ProviderSymbol::PermitUserMappingExecute => {
                provider_symbols.permit_user_mapping_execute
            }
            ProviderSymbol::SealUserMappingImmutableData => {
                provider_symbols.seal_user_mapping_immutable_data
            }
            ProviderSymbol::PermitUserMappingWritableData => {
                provider_symbols.permit_user_mapping_writable_data
            }
            ProviderSymbol::ReserveAnonymousUserMapping => {
                provider_symbols.reserve_anonymous_user_mapping
            }
            ProviderSymbol::ReleaseAnonymousUserMapping => {
                provider_symbols.release_anonymous_user_mapping
            }
            ProviderSymbol::UnmapUserAddressSpace => provider_symbols.unmap_user_address_space,
            ProviderSymbol::PrepareUserProbe => provider_symbols.prepare_user_probe,
            ProviderSymbol::EnterUserContext => provider_symbols.enter_user_context,
            ProviderSymbol::CompleteUserBacking => provider_symbols.complete_user_backing,
            ProviderSymbol::ReclaimUserAddressSpace => provider_symbols.reclaim_user_address_space,
            ProviderSymbol::UserSyscall => provider_symbols.user_syscall,
            ProviderSymbol::KernelThreadEntry => {
                provider_symbols.kernel_thread_entry.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 bootstrap root references an unplaced kernel-thread entry".into(),
                    )
                })?
            }
            ProviderSymbol::CompletionThreadEntry => {
                provider_symbols.completion_thread_entry.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 bootstrap root references an unplaced completion-thread entry".into(),
                    )
                })?
            }
            ProviderSymbol::TerminalThreadEntry => {
                provider_symbols.terminal_thread_entry.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 bootstrap root references an unplaced terminal-thread entry".into(),
                    )
                })?
            }
            ProviderSymbol::Uart16550Write => provider_symbols.uart16550_write,
            ProviderSymbol::DebugBreak => provider_symbols.debug_break,
            ProviderSymbol::InterruptReturn => provider_symbols.interrupt_return,
            ProviderSymbol::Fatal => provider_symbols.fatal,
            ProviderSymbol::BootstrapStorage => {
                provider_symbols.bootstrap_storage.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 root lowering emitted a storage relocation without storage use".into(),
                    )
                })?
            }
        };
        object
            .add_relocation(
                section,
                Relocation {
                    offset: start + relocation.offset,
                    symbol,
                    addend: relocation.addend,
                    flags: RelocationFlags::Generic {
                        kind: relocation.kind,
                        encoding: relocation.encoding,
                        size: 32,
                    },
                },
            )
            .map_err(|error| {
                CompileError::Tool(format!("cannot encode systems root relocation: {error}"))
            })?;
    }
    Ok(root_symbol)
}
