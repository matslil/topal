#[allow(clippy::too_many_lines)] // Exhaustive transition detail remains beside its canonical trace spelling.
pub(crate) fn semantic_trace(
    program: &CompilerSystemsProgram,
) -> Result<Vec<String>, CompileError> {
    let transitions = model_systems_transitions(program)
        .map_err(|error| CompileError::Tool(format!("invalid systems transition model: {error}")))?
        .into_iter()
        .map(|transition| {
            let identity = transition.semantic_identity();
            match transition {
                CompilerSystemsTransition::BorrowBootstrapRegionForLibrary { generation } => format!("{identity};generation={generation}"),
                CompilerSystemsTransition::CheckDynamicBootstrapByte { offset_bytes, expected } => format!("{identity};runtime-guard;offset={offset_bytes};expected={expected}"),
                CompilerSystemsTransition::RequireRegionLibraryReclamation => format!("{identity};reclamation-obligation"),
                CompilerSystemsTransition::ResumeBootstrapRegionAfterLibrary { require_true } => if require_true {
                    format!("{identity};resume-owner-obligation;runtime-guard;required-source-boolean=true;instance=root-region")
                } else { format!("{identity};resume-owner-obligation") },
                CompilerSystemsTransition::ProvisionBootstrapStorage {
                    capacity_bytes,
                    alignment_bytes,
                } => format!("{identity}:capacity={capacity_bytes}:alignment={alignment_bytes}"),
                CompilerSystemsTransition::ConsoleWrite { text } => {
                    format!("{identity}:sha256:{}", sha256(text.as_bytes()))
                }
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity,
                    request,
                } => format!(
                    "{identity}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:offset={}:expected={:#010x}",
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.register_offset_bytes,
                    request.expected_value
                ),
                CompilerSystemsTransition::UpdatePciConfigurationWord {
                    transaction_identity,
                    request,
                } => format!(
                    "{identity}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:offset={}:mask={:#06x}:expected={:#06x}:desired={:#06x}",
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.register_offset_bytes,
                    request.mask,
                    request.expected_masked_value,
                    request.desired_masked_value
                ),
                CompilerSystemsTransition::CreateDmaBuffer { request } => {
                    let direction = match request.direction {
                        CompilerDmaDirection::Bidirectional => "bidirectional",
                        CompilerDmaDirection::DeviceToMemory => "device-to-memory",
                    };
                    let role = match request.role {
                        CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
                        CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
                    };
                    format!(
                        "{identity}:buffer={}:device={}:domain={}:bytes={}:alignment={}:direction={direction}:visibility=provider-managed:role={role}",
                        request.buffer_identity,
                        request.device_identity,
                        request.translation_domain_identity,
                        request.byte_count,
                        request.alignment_bytes,
                    )
                }
                CompilerSystemsTransition::BindDmaBufferReference { request } => format!(
                    "{identity}:destination-buffer={}:destination-offset={}:referenced-buffer={}:referenced-offset={}:width={}",
                    request.destination_buffer_identity,
                    request.destination_offset_bytes,
                    request.referenced_buffer_identity,
                    request.referenced_offset_bytes,
                    request.width_bytes,
                ),
                CompilerSystemsTransition::StoreDmaBufferInteger { request } => format!(
                    "{identity}:buffer={}:offset={}:width={}:value={}",
                    request.buffer_identity,
                    request.offset_bytes,
                    request.width_bytes,
                    request.value,
                ),
                CompilerSystemsTransition::PrepareDmaBufferForDevice { request } => {
                    let direction = match request.direction {
                        CompilerDmaDirection::Bidirectional => "bidirectional",
                        CompilerDmaDirection::DeviceToMemory => "device-to-memory",
                    };
                    let role = match request.role {
                        CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
                        CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
                    };
                    format!(
                        "{identity}:buffer={}:device={}:domain={}:direction={direction}:visibility=provider-managed:role={role}",
                        request.buffer_identity,
                        request.device_identity,
                        request.translation_domain_identity,
                    )
                }
                CompilerSystemsTransition::CompleteDmaBufferForCpu { request } => {
                    let direction = match request.direction {
                        CompilerDmaDirection::Bidirectional => "bidirectional",
                        CompilerDmaDirection::DeviceToMemory => "device-to-memory",
                    };
                    let role = match request.role {
                        CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
                        CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
                    };
                    format!(
                        "{identity}:buffer={}:device={}:domain={}:direction={direction}:visibility=provider-managed:role={role}",
                        request.buffer_identity,
                        request.device_identity,
                        request.translation_domain_identity,
                    )
                }
                CompilerSystemsTransition::LoadDmaBufferIntegerEquals { request } => format!(
                    "{identity}:buffer={}:offset={}:width={}:expected={}",
                    request.buffer_identity,
                    request.offset_bytes,
                    request.width_bytes,
                    request.expected_value,
                ),
                CompilerSystemsTransition::CaptureEntropySample { request } => format!(
                    "{identity}:sample={}:buffer={}:offset={}:bytes={}:device={}:domain={}:direction=device-to-memory:visibility=provider-managed:role=entropy-payload:credited-entropy-bits={}",
                    request.sample_identity,
                    request.buffer_identity,
                    request.offset_bytes,
                    request.byte_count,
                    request.device_identity,
                    request.translation_domain_identity,
                    request.credited_entropy_bits,
                ),
                CompilerSystemsTransition::AssessEntropySampleQuality { request } => format!(
                    "{identity}:sample={}:source={}:trust-evidence={}:bytes={}:quality-parts-per-1024={}:credited-entropy-bits={}",
                    request.sample_identity,
                    request.source_identity,
                    request.trust_evidence_identity,
                    request.byte_count,
                    request.quality_parts_per_1024,
                    request.credited_entropy_bits,
                ),
                CompilerSystemsTransition::CreateEntropyPool { request } => format!(
                    "{identity}:pool={}:credit-capacity-bits={}:initial-credited-entropy-bits={}",
                    request.pool_identity,
                    request.credit_capacity_bits,
                    request.initial_credited_entropy_bits,
                ),
                CompilerSystemsTransition::MixEntropyPoolSample { request } => format!(
                    "{identity}:pool={}:sample={}:conditioner={}:contribution={}:bytes={}:contribution-entropy-bits={}",
                    request.pool_identity,
                    request.sample_identity,
                    request.conditioner_identity,
                    request.contribution_identity,
                    request.byte_count,
                    request.contribution_entropy_bits,
                ),
                CompilerSystemsTransition::CreditEntropyPoolContribution { request } => format!(
                    "{identity}:pool={}:contribution={}:conditioner-origin={}:prior-credited-entropy-bits={}:contribution-entropy-bits={}:credit-capacity-bits={}:resulting-credited-entropy-bits={}",
                    request.pool_identity,
                    request.contribution_identity,
                    request.conditioner_origin_identity,
                    request.prior_credited_entropy_bits,
                    request.contribution_entropy_bits,
                    request.credit_capacity_bits,
                    request.resulting_credited_entropy_bits,
                ),
                CompilerSystemsTransition::InitializeEntropyGenerator { request } => format!(
                    "{identity}:pool={}:generator={}:algorithm={}:generation={}:key-bytes={}:required-entropy-bits={}:observed-pool-entropy-bits={}",
                    request.pool_identity,
                    request.generator_identity,
                    request.algorithm_identity,
                    request.generation,
                    request.key_byte_count,
                    request.required_entropy_bits,
                    request.observed_pool_entropy_bits,
                ),
                CompilerSystemsTransition::ExtractEntropyGenerator { request } => format!(
                    "{identity}:generator={}:algorithm={}:prior-generation={}:resulting-generation={}:output={}:block-bytes={}:output-offset={}:output-bytes={}:counter={}:nonce={}",
                    request.generator_identity,
                    request.algorithm_identity,
                    request.prior_generation,
                    request.resulting_generation,
                    request.output_identity,
                    request.block_byte_count,
                    request.output_offset_bytes,
                    request.output_byte_count,
                    request.counter,
                    request.nonce_identity,
                ),
                CompilerSystemsTransition::DeliverInitialAuxvRandom { request } => format!(
                    "{identity}:output={}:source-bytes={}:selected-offset={}:delivered-bytes={}:discarded-offset={}:discarded-bytes={}:address-space={}:interface={}:auxiliary-tag={}:single-use={}",
                    request.output_identity,
                    request.source_byte_count,
                    request.selected_offset_bytes,
                    request.delivered_byte_count,
                    request.discarded_offset_bytes,
                    request.discarded_byte_count,
                    request.address_space_identity,
                    request.interface_identity,
                    request.auxiliary_tag,
                    request.single_use,
                ),
                CompilerSystemsTransition::ReserveInitialUserRandom { request } => format!(
                    "{identity}:output={}:source-bytes={}:selected-offset={}:maximum-delivery-bytes={}:discarded-offset={}:discarded-bytes={}:process={}:interface={}:service={}:supported-flags={}:incompatible-flags={}:single-use={}",
                    request.output_identity,
                    request.source_byte_count,
                    request.selected_offset_bytes,
                    request.maximum_delivery_byte_count,
                    request.discarded_offset_bytes,
                    request.discarded_byte_count,
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.supported_flag_mask,
                    request.incompatible_flag_mask,
                    request.single_use,
                ),
                CompilerSystemsTransition::BindReusableUserRandom { request } => format!(
                    "{identity}:generator={}:generation={}:initial-output={}:block-bytes={}:output-bytes={}:maximum-delivery-bytes={}:process={}:interface={}:service={}:supported-flags={}:incompatible-flags={}:reserved-first-output={}:reusable={}",
                    request.generator_identity,
                    request.generation,
                    request.initial_output_identity,
                    request.block_byte_count,
                    request.output_byte_count,
                    request.maximum_delivery_byte_count,
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.supported_flag_mask,
                    request.incompatible_flag_mask,
                    request.reserved_first_output,
                    request.reusable,
                ),
                CompilerSystemsTransition::ExpandSmallUserRandomRequests { request } => format!(
                    "{identity}:process={}:interface={}:service={}:algorithm={}:output-bytes={}:maximum-output-count={}:maximum-delivery-bytes={}:complete-request={}",
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.algorithm_identity,
                    request.output_byte_count,
                    request.maximum_output_count,
                    request.maximum_delivery_byte_count,
                    request.complete_request,
                ),
                CompilerSystemsTransition::ExpandLargeUserRandomRequests { request } => format!(
                    "{identity}:process={}:interface={}:service={}:algorithm={}:output-bytes={}:maximum-output-count={}:maximum-delivery-bytes={}:complete-request={}",
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.algorithm_identity,
                    request.output_byte_count,
                    request.maximum_output_count,
                    request.maximum_delivery_byte_count,
                    request.complete_request,
                ),
                CompilerSystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity,
                    request,
                } => format!(
                    "{identity}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:access-capability-offset={}:bar={}:region-offset={}:region-length={}:register-offset={}:width={}:buffer={}:buffer-offset={}:device-identity={}:domain={}",
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.access_capability_offset_bytes,
                    request.bar,
                    request.region_offset_bytes,
                    request.region_length_bytes,
                    request.register_offset_bytes,
                    request.width_bytes,
                    request.buffer_identity,
                    request.buffer_offset_bytes,
                    request.device_identity,
                    request.translation_domain_identity,
                ),
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity,
                    request,
                } => format!(
                    "{}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:access-capability-offset={}:bar={}:region-offset={}:region-length={}:register-offset={}:width={}:value={:#010x}",
                    identity,
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.access_capability_offset_bytes,
                    request.bar,
                    request.region_offset_bytes,
                    request.region_length_bytes,
                    request.register_offset_bytes,
                    request.width_bytes,
                    request.value
                ),
                CompilerSystemsTransition::Fatal { message } => {
                    format!("{identity}:sha256:{}", sha256(message.as_bytes()))
                }
                CompilerSystemsTransition::AllocateBootstrapRegion { request } => format!(
                    "{identity}:bytes={}:alignment={}:placement={}",
                    request.byte_count,
                    request.alignment_bytes,
                    request.placement.semantic_identity()
                ),
                CompilerSystemsTransition::AllocatePhysicalFrames { request } => format!(
                    "{identity}:frames={}:alignment={}",
                    request.frame_count, request.alignment_frames
                ),
                CompilerSystemsTransition::MapKernelFrames { .. } => {
                    format!("{identity}:rights=read-write:execution=denied:memory-kind=normal")
                }
                CompilerSystemsTransition::MapTranslationFrames { .. } => format!(
                    "{identity}:rights=read-write:execution=denied:memory-kind=normal:placement=provider-selected"
                ),
                CompilerSystemsTransition::BeginTranslationUpdate { .. } => {
                    format!(
                        "{identity}:template=bootstrap-equivalent:page-policy=provider-selected"
                    )
                }
                CompilerSystemsTransition::BeginTranslationEdit { kind }
                | CompilerSystemsTransition::CommitTranslationEdit { kind } => format!(
                    "{identity}:kind={}",
                    match kind {
                        CompilerTranslationEditKind::Map => "map",
                        CompilerTranslationEditKind::Unmap => "unmap",
                    }
                ),
                CompilerSystemsTransition::EnterCritical {
                    domain,
                    nesting_identity,
                }
                | CompilerSystemsTransition::RestoreCritical {
                    domain,
                    nesting_identity,
                } => format!(
                    "{identity}:domain={}:nesting={nesting_identity}",
                    match domain {
                        topal_language::CompilerCriticalDomain::LocalMaskableInterrupts => {
                            "local-maskable-interrupts"
                        }
                    }
                ),
                CompilerSystemsTransition::CreateAtomicWord { request } => format!(
                    "{identity}:offset={}:initial={}:domain=cpu-shared",
                    request.offset_bytes, request.initial_value
                ),
                CompilerSystemsTransition::CompareExchangeAtomicWord {
                    expected,
                    desired,
                    observed,
                    exchanged,
                    success_order,
                    failure_order,
                } => format!(
                    "{identity}:expected={expected}:desired={desired}:observed={observed}:exchanged={exchanged}:success-order={success_order:?}:failure-order={failure_order:?}"
                ),
                CompilerSystemsTransition::LoadAtomicWord { value, order } => {
                    format!("{identity}:value={value}:order={order:?}")
                }
                CompilerSystemsTransition::ConstructDeadline {
                    source_observation_identity,
                    duration_nanoseconds,
                } => format!(
                    "{identity}:source-observation={source_observation_identity}:duration-nanoseconds={duration_nanoseconds}"
                ),
                CompilerSystemsTransition::CreateKernelContext { context_identity }
                | CompilerSystemsTransition::EnterKernelThread { context_identity }
                | CompilerSystemsTransition::ReclaimKernelContext { context_identity } => {
                    format!("{identity}:context={context_identity}")
                }
                CompilerSystemsTransition::CreateKernelCompletion {
                    completion_identity,
                }
                | CompilerSystemsTransition::ConsumeKernelCompletion {
                    completion_identity,
                } => format!("{identity}:completion={completion_identity}"),
                CompilerSystemsTransition::CreateRunnableQueue { capacity } => {
                    format!("{identity}:capacity={capacity}")
                }
                CompilerSystemsTransition::EnqueueRunnableContext { context_identity }
                | CompilerSystemsTransition::DequeueRunnableContext { context_identity } => {
                    format!("{identity}:context={context_identity}")
                }
                CompilerSystemsTransition::TransferKernelContext {
                    context_identity,
                    caller_identity,
                }
                | CompilerSystemsTransition::RetireKernelContextToCaller {
                    context_identity,
                    caller_identity,
                } => format!(
                    "{identity}:context={context_identity}:caller={caller_identity}"
                ),
                CompilerSystemsTransition::TransferKernelContextUntilDeadline {
                    context_identity,
                    caller_identity,
                    event_identity,
                }
                | CompilerSystemsTransition::PreemptKernelContext {
                    context_identity,
                    caller_identity,
                    event_identity,
                }
                | CompilerSystemsTransition::RestorePreemptedKernelContext {
                    context_identity,
                    caller_identity,
                    event_identity,
                } => format!(
                    "{identity}:context={context_identity}:caller={caller_identity}:event={event_identity}"
                ),
                CompilerSystemsTransition::TransferKernelContextUntilCompletion {
                    context_identity,
                    caller_identity,
                    completion_identity,
                }
                | CompilerSystemsTransition::BlockKernelContext {
                    context_identity,
                    caller_identity,
                    completion_identity,
                }
                | CompilerSystemsTransition::SignalBlockedKernelCompletion {
                    context_identity,
                    caller_identity,
                    completion_identity,
                }
                | CompilerSystemsTransition::RestoreBlockedKernelContext {
                    context_identity,
                    caller_identity,
                    completion_identity,
                } => format!(
                    "{identity}:context={context_identity}:caller={caller_identity}:completion={completion_identity}"
                ),
                CompilerSystemsTransition::AwaitDeadlinePreemption { context_identity } => {
                    format!("{identity}:context={context_identity}")
                }
                CompilerSystemsTransition::AwaitKernelCompletion {
                    context_identity,
                    completion_identity,
                } => format!(
                    "{identity}:context={context_identity}:completion={completion_identity}"
                ),
                CompilerSystemsTransition::PrepareUserAddressSpace {
                    address_space_identity,
                }
                | CompilerSystemsTransition::VerifyUserAddressSpaceMapping {
                    address_space_identity,
                }
                | CompilerSystemsTransition::ReclaimUserAddressSpace {
                    address_space_identity,
                } => format!("{identity}:address-space={address_space_identity}"),
                CompilerSystemsTransition::MapUserAddressSpace {
                    address_space_identity,
                    mapping_identity,
                }
                | CompilerSystemsTransition::UnmapUserAddressSpace {
                    address_space_identity,
                    mapping_identity,
                } => format!(
                    "{identity}:address-space={address_space_identity}:mapping={mapping_identity}"
                ),
                CompilerSystemsTransition::ReserveAnonymousUserMapping {
                    address_space_identity,
                    reservation_identity,
                }
                | CompilerSystemsTransition::ReleaseAnonymousUserMapping {
                    address_space_identity,
                    reservation_identity,
                } => format!(
                    "{identity}:address-space={address_space_identity}:reservation={reservation_identity}"
                ),
                CompilerSystemsTransition::StoreUserMappingByte { mapping_identity }
                | CompilerSystemsTransition::InstallUserMappingEntryTrampoline {
                    mapping_identity,
                }
                | CompilerSystemsTransition::ProtectUserMappingReadOnly { mapping_identity }
                | CompilerSystemsTransition::LoadUserMappingByte { mapping_identity }
                | CompilerSystemsTransition::VerifyUserMappingEntryTrampoline {
                    mapping_identity,
                } => {
                    format!("{identity}:mapping={mapping_identity}")
                }
                CompilerSystemsTransition::PermitUserMappingExecute { mapping_identity }
                | CompilerSystemsTransition::SealUserMappingImmutableData { mapping_identity }
                | CompilerSystemsTransition::PermitUserMappingWritableData {
                    mapping_identity,
                } => {
                    format!("{identity}:mapping={mapping_identity}")
                }
                CompilerSystemsTransition::PrepareUserProbe {
                    process_identity,
                    probe_identity,
                } => format!(
                    "{identity}:process={process_identity}:probe={probe_identity}"
                ),
                CompilerSystemsTransition::EnterUserExecution {
                    process_identity,
                    kernel_continuation_identity,
                } => format!(
                    "{identity}:process={process_identity}:continuation={kernel_continuation_identity}"
                ),
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity,
                    number,
                    arguments,
                } => format!(
                    "{identity}:entry={entry_identity}:number={number}:arguments={arguments:?}"
                ),
                CompilerSystemsTransition::BeginUserTransfer {
                    entry_identity,
                    transfer_identity,
                    candidate,
                    extent,
                } => format!(
                    "{identity}:entry={entry_identity}:transfer={transfer_identity}:candidate={candidate}:extent={extent}"
                ),
                CompilerSystemsTransition::CopyToUserComplete {
                    transfer_identity,
                    copied,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied={copied}"
                ),
                CompilerSystemsTransition::CopyToUserFault {
                    transfer_identity,
                    copied_prefix,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied-prefix={copied_prefix}"
                ),
                CompilerSystemsTransition::CopyFromUserComplete {
                    transfer_identity,
                    copied,
                    seconds,
                    nanoseconds,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied={copied}:seconds={seconds}:nanoseconds={nanoseconds}"
                ),
                CompilerSystemsTransition::CopyFromUserFault {
                    transfer_identity,
                    copied_prefix,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied-prefix={copied_prefix}"
                ),
                CompilerSystemsTransition::EndUserTransfer {
                    entry_identity,
                    transfer_identity,
                } => format!(
                    "{identity}:entry={entry_identity}:transfer={transfer_identity}"
                ),
                CompilerSystemsTransition::ReturnToUser {
                    entry_identity,
                    result,
                } => format!("{identity}:entry={entry_identity}:result={result}"),
                CompilerSystemsTransition::TerminateUser {
                    entry_identity,
                    status,
                } => format!("{identity}:entry={entry_identity}:status={status}"),
                CompilerSystemsTransition::ResumeUserKernelContinuation {
                    kernel_continuation_identity,
                    status,
                } => format!(
                    "{identity}:continuation={kernel_continuation_identity}:status={status}"
                ),
                CompilerSystemsTransition::CompleteUserExecution { process_identity } => {
                    format!("{identity}:process={process_identity}")
                }
                CompilerSystemsTransition::TakePreemptedKernelContext {
                    context_identity,
                    event_identity,
                } => format!(
                    "{identity}:context={context_identity}:event={event_identity}"
                ),
                CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity }
                | CompilerSystemsTransition::ResumePreemptionDispatcher { caller_identity }
                | CompilerSystemsTransition::ResumeCompletionDispatcher { caller_identity } => {
                    format!("{identity}:caller={caller_identity}")
                }
                CompilerSystemsTransition::StoreBootstrapByte {
                    offset_bytes,
                    value,
                }
                | CompilerSystemsTransition::LoadBootstrapByte {
                    offset_bytes,
                    value,
                }
                | CompilerSystemsTransition::StoreKernelMappingByte {
                    offset_bytes,
                    value,
                }
                | CompilerSystemsTransition::LoadKernelMappingByte {
                    offset_bytes,
                    value,
                } => format!("{identity}:offset={offset_bytes}:value={value}"),
                CompilerSystemsTransition::EnterBootstrap
                | CompilerSystemsTransition::DescribeBootMemory
                | CompilerSystemsTransition::CreateFrameAllocator
                | CompilerSystemsTransition::ReleasePhysicalFrames
                | CompilerSystemsTransition::PrepareStaticImageConstruction
                | CompilerSystemsTransition::ZeroStaticImageConstructionPayload
                | CompilerSystemsTransition::ZeroStaticImageStackBacking
                | CompilerSystemsTransition::PublishStaticImageStackLeaf
                | CompilerSystemsTransition::InitializeStaticImageStackEntry
                | CompilerSystemsTransition::AddStaticImageStackPageSize
                | CompilerSystemsTransition::AddStaticImageStackEntryFact
                | CompilerSystemsTransition::AddStaticImageStackProgramHeaders
                | CompilerSystemsTransition::AddStaticImageStackExecutableName
                | CompilerSystemsTransition::AddStaticImageStackCredentials
                | CompilerSystemsTransition::AddStaticImageStackSecureMode
                | CompilerSystemsTransition::AddStaticImageStackClockTickRate
                | CompilerSystemsTransition::AddStaticImageStackPlatform
                | CompilerSystemsTransition::AddStaticImageStackHardwareCapabilities
                | CompilerSystemsTransition::AddStaticImageStackLoadMetadata
                | CompilerSystemsTransition::AuthorizeStaticImageStackRandomSlot
                | CompilerSystemsTransition::RetainStaticImagePreparedProcess
                | CompilerSystemsTransition::QualifyStaticImageRetainedDisjointWork
                | CompilerSystemsTransition::AuthorizeStaticImageEntropyHandoff
                | CompilerSystemsTransition::AuthorizeStaticImageEntropyAcquisition
                | CompilerSystemsTransition::QualifyStaticImageEntropyDevice
                | CompilerSystemsTransition::QualifyStaticImageEntropyTransport
                | CompilerSystemsTransition::QualifyStaticImageEntropyNegotiation
                | CompilerSystemsTransition::QualifyStaticImageEntropyQueue
                | CompilerSystemsTransition::QualifyStaticImageEntropyBusMaster
                | CompilerSystemsTransition::QualifyStaticImageEntropyDmaBuffers
                | CompilerSystemsTransition::QualifyStaticImageEntropySplitRing
                | CompilerSystemsTransition::QualifyStaticImageEntropyDmaPreparation
                | CompilerSystemsTransition::QualifyStaticImageEntropyQueueAddresses
                | CompilerSystemsTransition::QualifyStaticImageEntropyQueueActivation
                | CompilerSystemsTransition::QualifyStaticImageEntropyRequestCompletion
                | CompilerSystemsTransition::QualifyStaticImageEntropySampleCapture
                | CompilerSystemsTransition::QualifyStaticImageEntropySampleQuality
                | CompilerSystemsTransition::QualifyStaticImageEntropyPoolLifecycle
                | CompilerSystemsTransition::QualifyStaticImageEntropyGeneratorLifecycle
                | CompilerSystemsTransition::DeliverStaticImageEntropyInitialRandom
                | CompilerSystemsTransition::EnterStaticImageProcess
                | CompilerSystemsTransition::RestoreStaticImageProcessEntry
                | CompilerSystemsTransition::RevokeStaticImageEntropyInitialRandom
                | CompilerSystemsTransition::RevokeStaticImageEntropyAcquisition
                | CompilerSystemsTransition::RevokeStaticImageEntropyHandoff
                | CompilerSystemsTransition::RestoreStaticImagePreparedProcess
                | CompilerSystemsTransition::RevokeStaticImageStackRandomSlot
                | CompilerSystemsTransition::RemoveStaticImageStackLoadMetadata
                | CompilerSystemsTransition::RemoveStaticImageStackHardwareCapabilities
                | CompilerSystemsTransition::RemoveStaticImageStackPlatform
                | CompilerSystemsTransition::RemoveStaticImageStackClockTickRate
                | CompilerSystemsTransition::RemoveStaticImageStackSecureMode
                | CompilerSystemsTransition::RemoveStaticImageStackCredentials
                | CompilerSystemsTransition::RemoveStaticImageStackExecutableName
                | CompilerSystemsTransition::RemoveStaticImageStackProgramHeaders
                | CompilerSystemsTransition::RemoveStaticImageStackEntryFact
                | CompilerSystemsTransition::RemoveStaticImageStackPageSize
                | CompilerSystemsTransition::ClearStaticImageStackEntry
                | CompilerSystemsTransition::RevokeStaticImageStackLeaf
                | CompilerSystemsTransition::CopyStaticImageConstructionPayload
                | CompilerSystemsTransition::SealStaticImageConstructionPermissions
                | CompilerSystemsTransition::AttachStaticImageAddressSpace
                | CompilerSystemsTransition::ActivateStaticImageAddressSpace
                | CompilerSystemsTransition::RestoreStaticImageAddressSpace
                | CompilerSystemsTransition::AuthorizeStaticImageEntry
                | CompilerSystemsTransition::AuthorizeStaticImageStackSlot
                | CompilerSystemsTransition::RevokeStaticImageStackSlot
                | CompilerSystemsTransition::RevokeStaticImageEntry
                | CompilerSystemsTransition::DetachStaticImageAddressSpace
                | CompilerSystemsTransition::ReclaimStaticImageConstructionHierarchy
                | CompilerSystemsTransition::RevokeStaticImageConstructionMetadata
                | CompilerSystemsTransition::ReclaimStaticImageConstructionPayload
                | CompilerSystemsTransition::UnmapKernelFrames
                | CompilerSystemsTransition::CommitTranslationUpdate
                | CompilerSystemsTransition::ActivateTranslationSpace
                | CompilerSystemsTransition::UnmapTranslationMapping
                | CompilerSystemsTransition::ObserveDebugBreak
                | CompilerSystemsTransition::EnterDebugBreak
                | CompilerSystemsTransition::ResumeDebugBreak
                | CompilerSystemsTransition::SendLocalNotification { .. }
                | CompilerSystemsTransition::BeginLocalNotificationWait { .. }
                | CompilerSystemsTransition::ObserveLocalNotification { .. }
                | CompilerSystemsTransition::EnterLocalNotificationInterrupt { .. }
                | CompilerSystemsTransition::CompleteLocalNotificationInterrupt { .. }
                | CompilerSystemsTransition::ResumeLocalNotificationInterrupt { .. }
                | CompilerSystemsTransition::EndLocalNotificationWait { .. }
                | CompilerSystemsTransition::ObserveMonotonicClock { .. }
                | CompilerSystemsTransition::ArmDeadline { .. }
                | CompilerSystemsTransition::BeginDeadlineWait { .. }
                | CompilerSystemsTransition::ObserveDeadline { .. }
                | CompilerSystemsTransition::EnterDeadlineInterrupt { .. }
                | CompilerSystemsTransition::CompleteDeadlineInterrupt { .. }
                | CompilerSystemsTransition::ResumeDeadlineInterrupt { .. }
                | CompilerSystemsTransition::EndDeadlineWait { .. }
                | CompilerSystemsTransition::EndAtomicWord
                | CompilerSystemsTransition::ConsumeEmptyRunnableQueue
                | CompilerSystemsTransition::ReleaseBootstrapRegion => identity.into(),
            }
        })
        .collect();
    Ok(transitions)
}

fn allocate_stage(parent: &Path) -> Result<PathBuf, CompileError> {
    for _ in 0..100 {
        let sequence = NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
        let stage = parent.join(format!(
            ".topal-systems-artifact-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&stage) {
            Ok(()) => return Ok(stage),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(io_error("create systems artifact stage", &stage)(error)),
        }
    }
    Err(CompileError::Io(
        "cannot allocate a unique systems artifact stage".into(),
    ))
}

fn run_tool(command: &mut Command) -> Result<(), CompileError> {
    let display = format!("{command:?}");
    let output = command.output().map_err(|error| {
        CompileError::Tool(format!(
            "cannot execute systems artifact tool {display}: {error}"
        ))
    })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(CompileError::Tool(format!(
            "systems artifact tool failed ({display}):\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn io_error<'a>(
    action: &'static str,
    path: &'a Path,
) -> impl FnOnce(std::io::Error) -> CompileError + 'a {
    move |error| CompileError::Io(format!("cannot {action} {}: {error}", path.display()))
}
