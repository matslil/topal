const INITIAL_TWO_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 2,
    alignment_frames: 1,
};

const INITIAL_MULTI_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 3,
    alignment_frames: 1,
};

const STATIC_ELF_HIERARCHY_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 5,
    alignment_frames: 1,
};

const POST_RANDOM_ANONYMOUS_EXTENT_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 34,
    alignment_frames: 1,
};

const STATIC_ELF_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 503,
    alignment_frames: 1,
};

#[derive(Default, Eq, PartialEq)]
enum BootstrapTranslationState {
    #[default]
    BootstrapActive,
    Update,
    InactiveSpace,
    ReplacementActive,
    MapEdit,
    MapStaged,
    MappedActive,
    UnmapEdit,
    UnmapStaged,
    RemovedActive,
}

impl BootstrapAuthorityState {
    #[allow(clippy::too_many_lines)] // The sealed bootstrap transition order is audited together.
    fn observe(
        &mut self,
        index: usize,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        if self.image_backing_live {
            let marker = if self.image_tables { "TOPAL_KERNEL_IMAGE_TABLES_PREPARED" }
                         else { "TOPAL_KERNEL_IMAGE_BACKING_RESERVED" };
            match operation {
                SystemsOperation::ConsoleWrite { text }
                    if text == marker && !self.image_backing_observed => {
                        self.image_backing_observed = true;
                        return Ok(true);
                    }
                SystemsOperation::DiscardKernelImageBacking if self.image_backing_observed && !self.image_tables => {}
                SystemsOperation::DiscardKernelImageTables if self.image_backing_observed && self.image_tables => {}
                _ => return Err(SystemsModelError::new("E-SYSTEMS-KERNEL-IMAGE", "image backing requires one reservation observation then consuming discard before other boot work")),
            }
        } else if self.image_backing_discarded && !self.image_backing_discard_observed {
            let marker = if self.image_tables { "TOPAL_KERNEL_IMAGE_TABLES_DISCARDED" }
                         else { "TOPAL_KERNEL_IMAGE_BACKING_DISCARDED" };
            if let SystemsOperation::ConsoleWrite { text } = operation
                && text == marker
            {
                self.image_backing_discard_observed = true;
                return Ok(true);
            }
            return Err(SystemsModelError::new("E-SYSTEMS-KERNEL-IMAGE", "image backing requires its discard observation before continuing"));
        }
        if self.observe_critical(operation)? {
            return Ok(true);
        }
        if self.observe_local_notification(operation)? {
            return Ok(true);
        }
        if self.observe_deadline(operation)? {
            return Ok(true);
        }
        if self.observe_kernel_context(operation)? {
            return Ok(true);
        }
        if self.observe_user_execution(operation)? {
            return Ok(true);
        }
        match operation {
            SystemsOperation::DescribeBootMemory { .. } => {
                if index != 0 || self.memory_described {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-BOOT-MEMORY",
                        "boot memory must be described exactly once as the first bootstrap operation",
                    ));
                }
                self.memory_described = true;
            }
            SystemsOperation::CreateFrameAllocator { .. } => {
                if index != 1 || !self.memory_described || self.allocator_created {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "the frame allocator must consume memory description exactly once as the second bootstrap operation",
                    ));
                }
                self.allocator_created = true;
            }
            SystemsOperation::AllocatePhysicalFrames { request, .. } => {
                if !self.allocator_created {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "physical-frame allocation requires one allocator and an admitted static ownership state",
                    ));
                }
                self.memory_ownership = match (self.memory_ownership, *request) {
                    (
                        BootstrapMemoryOwnership::None,
                        request @ (INITIAL_SINGLE_FRAME_REQUEST
                        | INITIAL_TWO_FRAME_REQUEST
                        | INITIAL_MULTI_FRAME_REQUEST
                        | POST_RANDOM_ANONYMOUS_EXTENT_REQUEST
                        | STATIC_ELF_FRAME_REQUEST),
                    ) => BootstrapMemoryOwnership::Frames(request),
                    (
                        BootstrapMemoryOwnership::Frames(STATIC_ELF_FRAME_REQUEST),
                        INITIAL_SINGLE_FRAME_REQUEST,
                    ) => BootstrapMemoryOwnership::StaticElfMappingResources,
                    (
                        BootstrapMemoryOwnership::StaticElfConstructionSealed,
                        STATIC_ELF_HIERARCHY_FRAME_REQUEST,
                    ) => BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy,
                    (
                        BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized,
                        INITIAL_TWO_FRAME_REQUEST,
                    ) => {
                        BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorizedWithBacking
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-FRAMES",
                            "physical-frame allocation requires an admitted static extent shape and exact nested ownership",
                        ));
                    }
                };
            }
            SystemsOperation::ReserveKernelImageBacking { .. } | SystemsOperation::PrepareKernelImageTables { .. } => {
                if !self.allocator_created || self.image_backing_live || self.image_backing_discarded
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.translation != BootstrapTranslationState::BootstrapActive
                {
                    return Err(SystemsModelError::new("E-SYSTEMS-KERNEL-IMAGE", "image backing requires the unborrowed initial allocator and one current-image preparation"));
                }
                self.image_backing_live = true;
                self.image_tables = matches!(operation, SystemsOperation::PrepareKernelImageTables { .. });
            }
            SystemsOperation::DiscardKernelImageBacking | SystemsOperation::DiscardKernelImageTables => {
                if !self.image_backing_live || self.image_tables != matches!(operation, SystemsOperation::DiscardKernelImageTables) {
                    return Err(SystemsModelError::new("E-SYSTEMS-KERNEL-IMAGE", "image-backing discard requires its exact live preparation"));
                }
                self.image_backing_live = false;
                self.image_backing_discarded = true;
            }
            SystemsOperation::ReserveStaticImageHierarchy { .. } => {
                if !self.allocator_created
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::StaticElfConstructionSealed
                    || !matches!(
                        self.translation,
                        BootstrapTranslationState::BootstrapActive
                            | BootstrapTranslationState::ReplacementActive
                            | BootstrapTranslationState::RemovedActive
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "provider-selected hierarchy backing requires the exact sealed construction, allocator and available translation context",
                    ));
                }
                // The initial provider has one qualified bootstrap-equivalent
                // topology. Keep purpose in the transition rather than treating
                // a source integer as backing authority.
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy;
            }
            SystemsOperation::ReleasePhysicalFrames => {
                if matches!(
                    self.memory_ownership,
                    BootstrapMemoryOwnership::UserMapping(_)
                ) {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "physical-frame release is invalid while a user mapping owns the extent",
                    ));
                }
                self.memory_ownership = match self.memory_ownership {
                    BootstrapMemoryOwnership::StaticElfMappingResources => {
                        BootstrapMemoryOwnership::Frames(STATIC_ELF_FRAME_REQUEST)
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionMetadataReturned => {
                        BootstrapMemoryOwnership::StaticElfConstructionRevoked
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy => {
                        BootstrapMemoryOwnership::StaticElfConstructionSealed
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed => {
                        BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionHierarchyReturned => {
                        BootstrapMemoryOwnership::StaticElfConstructionSealed
                    }
                    BootstrapMemoryOwnership::Frames(_) => BootstrapMemoryOwnership::None,
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-FRAMES",
                            "physical-frame release requires the next exact live extent",
                        ));
                    }
                };
            }
            SystemsOperation::PrepareStaticImageConstruction { .. } => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfMappingResources {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image construction requires the exact live payload and metadata resources",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionLive;
            }
            SystemsOperation::ZeroStaticImageConstructionPayload => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionLive {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image payload zeroing requires one live uninitialized construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionZeroed;
            }
            SystemsOperation::CopyStaticImageConstructionPayload => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionZeroed {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image payload copy requires one completely zeroed construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionPopulated;
            }
            SystemsOperation::SealStaticImageConstructionPermissions => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionPopulated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image permission sealing requires one populated construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionSealed;
            }
            SystemsOperation::AttachStaticImageAddressSpace => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space attachment requires one sealed construction and its live five-frame hierarchy",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionAttached;
            }
            SystemsOperation::ActivateStaticImageAddressSpace => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionAttached
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space activation requires one fresh attached construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionActive;
            }
            SystemsOperation::RestoreStaticImageAddressSpace => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionActive {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space restoration requires one active construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionRestored;
            }
            SystemsOperation::AuthorizeStaticImageEntry => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionRestored
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entry authorization requires one restored attached construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized;
            }
            SystemsOperation::RevokeStaticImageEntry => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entry revocation requires one authorized construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionEntryRestored;
            }
            SystemsOperation::AuthorizeStaticImageStackSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-slot authorization requires one entry-authorized construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized;
            }
            SystemsOperation::ZeroStaticImageStackBacking => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorizedWithBacking
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-backing zeroing requires one slot-authorized construction with live backing",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed;
            }
            SystemsOperation::PublishStaticImageStackLeaf => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-leaf publication requires one zeroed stack backing",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished;
            }
            SystemsOperation::InitializeStaticImageStackEntry => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-entry initialization requires one published stack leaf",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized;
            }
            SystemsOperation::AddStaticImageStackPageSize => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack page-size addition requires one initialized stack entry",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded;
            }
            SystemsOperation::AddStaticImageStackEntryFact => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack entry-fact addition requires one live page-size fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded;
            }
            SystemsOperation::AddStaticImageStackProgramHeaders => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack program-header addition requires one live entry fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded;
            }
            SystemsOperation::AddStaticImageStackExecutableName => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack executable-name addition requires one live program-header group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded;
            }
            SystemsOperation::AddStaticImageStackCredentials => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack credential addition requires one live executable-name fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded;
            }
            SystemsOperation::AddStaticImageStackSecureMode => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack secure-mode addition requires one live credential group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded;
            }
            SystemsOperation::AddStaticImageStackClockTickRate => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack clock-tick-rate addition requires one live security fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded;
            }
            SystemsOperation::AddStaticImageStackPlatform => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack platform addition requires one live clock-tick-rate fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded;
            }
            SystemsOperation::RemoveStaticImageStackPlatform => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack platform removal requires one live platform fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded;
            }
            SystemsOperation::AddStaticImageStackHardwareCapabilities => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack hardware-capability addition requires one live platform fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded;
            }
            SystemsOperation::AddStaticImageStackLoadMetadata => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack load-metadata addition requires one live hardware-capability group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded;
            }
            SystemsOperation::AuthorizeStaticImageStackRandomSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack random-slot authorization requires one live load-metadata group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized;
            }
            SystemsOperation::RetainStaticImagePreparedProcess => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized
                    || self.retained_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image prepared-process retention requires one live random-slot-authorized construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
                self.retained_static_process = true;
            }
            SystemsOperation::QualifyStaticImageRetainedDisjointWork => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.retained_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image retained disjoint work requires the exact retained process and closed allocator channel",
                    ));
                }
            }
            SystemsOperation::AuthorizeStaticImageEntropyHandoff => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.retained_static_process
                    || self.entropy_pending_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy handoff authorization requires the exact retained process",
                    ));
                }
                self.retained_static_process = false;
                self.entropy_pending_static_process = true;
            }
            SystemsOperation::AuthorizeStaticImageEntropyAcquisition => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.entropy_pending_static_process
                    || self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy acquisition authorization requires the exact entropy-pending process",
                    ));
                }
                self.entropy_pending_static_process = false;
                self.entropy_acquiring_static_process = true;
            }
            SystemsOperation::QualifyStaticImageEntropyDevice => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-device qualification requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyTransport => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-transport qualification requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyNegotiation => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-transport negotiation requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyQueue => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-queue discovery requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyBusMaster => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy bus-master qualification requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyDmaBuffers => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy DMA-buffer coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropySplitRing => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy split-ring coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyDmaPreparation => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy DMA-preparation coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyQueueAddresses => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy queue-address coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyQueueActivation => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy queue-activation coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyRequestCompletion => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy request-completion coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropySampleCapture => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-sample capture coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropySampleQuality => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-sample quality coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyPoolLifecycle => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-pool lifecycle coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-generator lifecycle coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::DeliverStaticImageEntropyInitialRandom => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                    || self.randomized_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image initial-random delivery requires the exact acquisition owner",
                    ));
                }
                self.entropy_acquiring_static_process = false;
                self.randomized_static_process = true;
            }
            SystemsOperation::EnterStaticImageProcess => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.randomized_static_process
                    || self.completed_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image process entry requires the exact randomized owner",
                    ));
                }
                self.randomized_static_process = false;
                self.completed_static_process = true;
            }
            SystemsOperation::RestoreStaticImageProcessEntry => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.randomized_static_process
                    || !self.completed_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image process-entry restoration requires the exact completed owner",
                    ));
                }
                self.completed_static_process = false;
                self.randomized_static_process = true;
            }
            SystemsOperation::RevokeStaticImageEntropyInitialRandom => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || self.entropy_acquiring_static_process
                    || !self.randomized_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image initial-random revocation requires the exact randomized owner",
                    ));
                }
                self.randomized_static_process = false;
                self.entropy_acquiring_static_process = true;
            }
            SystemsOperation::RevokeStaticImageEntropyAcquisition => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy acquisition revocation requires the exact acquisition owner",
                    ));
                }
                self.entropy_acquiring_static_process = false;
                self.entropy_pending_static_process = true;
            }
            SystemsOperation::RevokeStaticImageEntropyHandoff => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.retained_static_process
                    || !self.entropy_pending_static_process
                    || self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy handoff revocation requires the exact entropy-pending process",
                    ));
                }
                self.entropy_pending_static_process = false;
                self.retained_static_process = true;
            }
            SystemsOperation::RestoreStaticImagePreparedProcess => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.retained_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image prepared-process restoration requires the exact retained process and closed allocator channel",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized;
                self.retained_static_process = false;
            }
            SystemsOperation::RevokeStaticImageStackRandomSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack random-slot revocation requires one live random-slot authority",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded;
            }
            SystemsOperation::RemoveStaticImageStackLoadMetadata => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack load-metadata removal requires one live load-metadata group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded;
            }
            SystemsOperation::RemoveStaticImageStackHardwareCapabilities => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack hardware-capability removal requires one live capability group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded;
            }
            SystemsOperation::RemoveStaticImageStackClockTickRate => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack clock-tick-rate removal requires one live accounting-rate fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded;
            }
            SystemsOperation::RemoveStaticImageStackSecureMode => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack secure-mode removal requires one live security fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded;
            }
            SystemsOperation::RemoveStaticImageStackCredentials => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack credential removal requires one live credential group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded;
            }
            SystemsOperation::RemoveStaticImageStackExecutableName => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack executable-name removal requires one live executable-name fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded;
            }
            SystemsOperation::RemoveStaticImageStackProgramHeaders => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack program-header removal requires one live program-header group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded;
            }
            SystemsOperation::RemoveStaticImageStackEntryFact => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack entry-fact removal requires one live entry fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded;
            }
            SystemsOperation::RemoveStaticImageStackPageSize => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack page-size removal requires one live page-size fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized;
            }
            SystemsOperation::ClearStaticImageStackEntry => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-entry clearing requires one initialized stack entry",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished;
            }
            SystemsOperation::RevokeStaticImageStackLeaf => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-leaf revocation requires one published stack leaf",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed;
            }
            SystemsOperation::RevokeStaticImageStackSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-slot revocation requires one slot-authorized construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized;
            }
            SystemsOperation::DetachStaticImageAddressSpace => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionEntryRestored
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space detachment requires one attached inactive construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionDetached;
            }
            SystemsOperation::ReclaimStaticImageConstructionHierarchy => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionDetached
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image hierarchy reclaim requires one detached construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionHierarchyReturned;
            }
            SystemsOperation::RevokeStaticImageConstructionMetadata => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionSealed {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image revocation requires one sealed initial-leaf construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionMetadataReturned;
            }
            SystemsOperation::ReclaimStaticImageConstructionPayload => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionRevoked {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image payload reclaim requires complete revocation and metadata release",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Frames(STATIC_ELF_FRAME_REQUEST);
            }
            SystemsOperation::MapKernelFrames { request, .. } => {
                let BootstrapMemoryOwnership::Frames(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping requires one live single-frame extent and the sealed mapping policy",
                    ));
                };
                if frame_request != INITIAL_SINGLE_FRAME_REQUEST
                    || *request != KernelMappingRequest::initial_read_write()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping requires one live single-frame extent and the sealed mapping policy",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Mapping(frame_request);
            }
            SystemsOperation::KernelMappingStoreByte { offset_bytes, .. }
            | SystemsOperation::KernelMappingLoadByteEquals { offset_bytes, .. } => {
                if !matches!(self.memory_ownership, BootstrapMemoryOwnership::Mapping(_))
                    || *offset_bytes >= 4096
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping byte access requires one live mapping and an in-bounds offset",
                    ));
                }
            }
            SystemsOperation::UnmapKernelFrames => {
                let BootstrapMemoryOwnership::Mapping(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel unmap requires one live mapping",
                    ));
                };
                self.memory_ownership = BootstrapMemoryOwnership::Frames(frame_request);
            }
            operation @ (SystemsOperation::BeginTranslationUpdate { .. }
            | SystemsOperation::CommitTranslationUpdate { .. }
            | SystemsOperation::ActivateTranslationSpace { .. }
            | SystemsOperation::BeginTranslationEdit { .. }
            | SystemsOperation::MapTranslationFrames { .. }
            | SystemsOperation::UnmapTranslationMapping
            | SystemsOperation::CommitTranslationEdit { .. }) => {
                self.observe_translation(operation)?;
            }
            SystemsOperation::ObserveMonotonicClock => {
                if self.local_notification != BootstrapLocalNotificationState::Completed
                    || self.monotonic_clock_observations >= 2
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MONOTONIC-CLOCK",
                        "monotonic-clock observation requires the resumed context and admits exactly two initial observations",
                    ));
                }
                self.monotonic_clock_observations += 1;
            }
            SystemsOperation::ObservePciConfigurationDword { request } => {
                let expected = PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                    .get(self.pci_configuration_observations);
                if !self.allocator_created
                    || !self.critical_stack.is_empty()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG",
                        "the initial PCI configuration observations require the exact sealed Q35 virtio-rng capability-discovery sequence and a restored bootstrap context",
                    ));
                }
                self.pci_configuration_observations += 1;
            }
            SystemsOperation::AccessPciRegionRegister { request } => {
                let negotiation_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len();
                let discovery_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len();
                let activation_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len();
                let notification_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len();
                let expected = if self.pci_region_register_accesses < negotiation_len {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        .get(self.pci_region_register_accesses)
                } else if self.pci_region_register_accesses < negotiation_len + discovery_len {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                        .get(self.pci_region_register_accesses - negotiation_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len
                {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                        .get(self.pci_region_register_accesses - negotiation_len - discovery_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len + notification_len
                {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len,
                    )
                } else {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len
                            - notification_len,
                    )
                };
                if self.pci_configuration_observations
                    != PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
                    || !self.critical_stack.is_empty()
                    || (self.pci_region_register_accesses >= negotiation_len + discovery_len
                        && self.dma_references_published
                            != PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                                .len())
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-PCI-REGION",
                        "PCI region access requires the completed capability discovery and exact sealed virtio-rng negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences",
                    ));
                }
                self.pci_region_register_accesses += 1;
            }
            SystemsOperation::UpdatePciConfigurationWord { request } => {
                if self.pci_region_register_accesses
                    != PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                    || self.pci_bus_master_authorized
                    || !self.critical_stack.is_empty()
                    || !request.is_well_formed()
                    || *request
                        != PciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG-UPDATE",
                        "PCI configuration update requires complete queue discovery and the exact sealed bus-master command transition",
                    ));
                }
                self.pci_bus_master_authorized = true;
            }
            SystemsOperation::CreateDmaBuffer { request } => {
                let expected =
                    DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.get(self.dma_buffers_created);
                if !self.pci_bus_master_authorized
                    || !self.critical_stack.is_empty()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-BUFFER",
                        "DMA-buffer creation requires bus-master authorization and the exact sealed CPU-owned virtio-rng buffer sequence",
                    ));
                }
                self.dma_buffers_created += 1;
            }
            SystemsOperation::BindDmaBufferReference { request } => {
                if self.dma_buffers_created != DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
                    || self.dma_construction_operations != 0
                    || !request.is_well_formed()
                    || *request != DmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-CONSTRUCT",
                        "DMA reference binding requires both CPU-owned buffers and the exact sealed virtio-rng descriptor reference",
                    ));
                }
                self.dma_construction_operations = 1;
            }
            SystemsOperation::StoreDmaBufferInteger { request } => {
                let expected_index = self.dma_construction_operations.checked_sub(1);
                let expected = expected_index.and_then(|index| {
                    DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.get(index)
                });
                if self.dma_buffers_created != DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-CONSTRUCT",
                        "DMA integer store requires the exact sealed CPU-owned virtio-rng split-ring construction sequence",
                    ));
                }
                self.dma_construction_operations += 1;
            }
            SystemsOperation::PrepareDmaBufferForDevice { request } => {
                let expected = DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG
                    .get(self.dma_buffers_prepared);
                if self.dma_construction_operations
                    != 1 + DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-PREPARE",
                        "DMA device preparation requires complete split-ring construction and the exact sealed payload-then-control ownership sequence",
                    ));
                }
                self.dma_buffers_prepared += 1;
            }
            SystemsOperation::CompleteDmaBufferForCpu { request } => {
                let required_region_accesses =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.len();
                let completes_control = !self.dma_control_completed
                    && *request == DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1]
                    && self.dma_used_ring_reads == 0;
                let completes_payload = self.dma_control_completed
                    && !self.dma_payload_completed
                    && self.dma_used_ring_reads
                        == DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len()
                    && *request == DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
                if self.pci_region_register_accesses != required_region_accesses
                    || !request.is_well_formed()
                    || (!completes_control && !completes_payload)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-COMPLETE",
                        "DMA completion requires the exact sealed control-then-payload ownership sequence around virtio-rng used-ring validation",
                    ));
                }
                if completes_control {
                    self.dma_control_completed = true;
                } else {
                    self.dma_payload_completed = true;
                }
            }
            SystemsOperation::LoadDmaBufferIntegerEquals { request } => {
                let expected = DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                    .get(self.dma_used_ring_reads);
                if !self.dma_control_completed
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-USED-RING",
                        "DMA integer read requires CPU-owned control memory and the exact sealed virtio-rng used-ring validation sequence",
                    ));
                }
                self.dma_used_ring_reads += 1;
            }
            SystemsOperation::CaptureEntropySample { request } => {
                if !self.dma_payload_completed
                    || self.entropy_sample_captured
                    || !request.is_well_formed()
                    || *request != EntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-SAMPLE",
                        "entropy-sample capture requires the exact sealed uncredited virtio-rng payload range after CPU completion",
                    ));
                }
                self.entropy_sample_captured = true;
            }
            SystemsOperation::AssessEntropySampleQuality { request } => {
                if !self.entropy_sample_captured
                    || self.entropy_sample_assessed
                    || !request.is_well_formed()
                    || *request != EntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-QUALITY",
                        "entropy-sample quality assessment requires the exact sealed Q35 trust evidence and full-quality credit after capture",
                    ));
                }
                self.entropy_sample_assessed = true;
            }
            SystemsOperation::CreateEntropyPool { request } => {
                if !self.entropy_sample_assessed
                    || self.entropy_pool_created
                    || !request.is_well_formed()
                    || *request != EntropyPoolRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-POOL",
                        "entropy-pool creation requires the exact sealed empty 256-bit pool after sample assessment",
                    ));
                }
                self.entropy_pool_created = true;
            }
            SystemsOperation::MixEntropyPoolSample { request } => {
                if !self.entropy_sample_assessed
                    || !self.entropy_pool_created
                    || self.entropy_pool_mixed
                    || !request.is_well_formed()
                    || *request != EntropyPoolMixRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-MIX",
                        "entropy-pool mixing requires the exact sealed assessed sample, empty pool, conditioner, and contribution",
                    ));
                }
                self.entropy_pool_mixed = true;
            }
            SystemsOperation::CreditEntropyPoolContribution { request } => {
                if !self.entropy_pool_mixed
                    || self.entropy_pool_credited
                    || !request.is_well_formed()
                    || *request != EntropyPoolCreditRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-CREDIT",
                        "entropy-pool credit transfer requires the exact sealed mixed pool and live first contribution",
                    ));
                }
                self.entropy_pool_credited = true;
            }
            SystemsOperation::InitializeEntropyGenerator { request } => {
                if !self.entropy_pool_credited
                    || self.entropy_generator_ready
                    || !request.is_well_formed()
                    || *request != EntropyGeneratorRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-GENERATOR",
                        "entropy-generator initialization requires the exact sealed credited pool and generation-one ChaCha20 request",
                    ));
                }
                self.entropy_generator_ready = true;
            }
            SystemsOperation::ExtractEntropyGenerator { request } => {
                if !self.entropy_generator_ready || !request.is_well_formed() {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-EXTRACT",
                        "entropy-generator extraction requires the next exact sealed ready ChaCha20 generation",
                    ));
                }
                if *request == EntropyGeneratorExtractRequest::INITIAL_Q35
                    && !self.entropy_generator_extracted
                {
                    self.entropy_generator_extracted = true;
                } else if *request == EntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM
                    && self.entropy_generator_extracted
                    && self.initial_auxv_random_delivered
                    && !self.initial_getrandom_extracted
                {
                    self.initial_getrandom_extracted = true;
                } else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-EXTRACT",
                        "entropy-generator extraction requires the next exact sealed ready ChaCha20 generation",
                    ));
                }
            }
            SystemsOperation::DeliverInitialAuxvRandom { request } => {
                if !self.entropy_generator_extracted
                    || self.initial_auxv_random_delivered
                    || !request.is_well_formed()
                    || *request != EntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-DELIVER",
                        "initial AT_RANDOM delivery requires the exact live first output and prepared resident initial process before entry",
                    ));
                }
                self.initial_auxv_random_delivered = true;
            }
            SystemsOperation::ReserveInitialUserRandom { request } => {
                if !self.initial_getrandom_extracted
                    || self.initial_user_random_reserved
                    || !request.is_well_formed()
                    || *request != EntropyOutputReservationRequest::INITIAL_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-RESERVE",
                        "initial getrandom reservation requires the exact live second output and prepared resident initial process before entry",
                    ));
                }
                self.initial_user_random_reserved = true;
            }
            SystemsOperation::BindReusableUserRandom { request } => {
                if !self.initial_user_random_reserved
                    || self.user_random_service_bound
                    || self.user_random_service_completed
                    || !request.is_well_formed()
                    || *request != EntropyUserRandomServiceRequest::INITIAL_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-USER-SERVICE",
                        "reusable getrandom binding requires the generation-three generator, live initial reservation, and prepared resident initial process before entry",
                    ));
                }
                self.initial_user_random_reserved = false;
                self.user_random_service_bound = true;
            }
            SystemsOperation::ExpandSmallUserRandomRequests { request } => {
                if !self.user_random_service_bound
                    || self.user_random_small_requests_enabled
                    || self.user_random_service_completed
                    || !request.is_well_formed()
                    || *request != EntropyUserRandomSmallRequest::INITIAL_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-USER-SMALL-REQUEST",
                        "small-request getrandom refinement requires the exact live reusable service and prepared resident initial process before entry",
                    ));
                }
                self.user_random_small_requests_enabled = true;
            }
            SystemsOperation::ExpandLargeUserRandomRequests { request } => {
                if !self.user_random_service_bound
                    || !self.user_random_small_requests_enabled
                    || self.user_random_large_requests_enabled
                    || self.user_random_service_completed
                    || !request.is_well_formed()
                    || *request != EntropyUserRandomSmallRequest::LARGE_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-USER-LARGE-REQUEST",
                        "large-request getrandom refinement requires the exact live small-request service and prepared resident initial process before entry",
                    ));
                }
                self.user_random_large_requests_enabled = true;
            }
            SystemsOperation::PublishPciRegionDmaReference { request } => {
                let expected = PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                    .get(self.dma_references_published);
                if self.dma_buffers_prepared
                    != DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-PUBLISH",
                        "PCI-region DMA-reference publication requires both device-owned buffers and the exact sealed descriptor-driver-device register sequence",
                    ));
                }
                self.dma_references_published += 1;
            }
            _ if !self.allocator_created => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-FRAMES",
                    "bootstrap operations require a frame-allocator context",
                ));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_local_notification(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::SendLocalNotification => {
                if !self.allocator_created
                    || self.local_notification != BootstrapLocalNotificationState::Fresh
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-LOCAL-INTERRUPT",
                        "local notification send requires one restored processor context and no pending event",
                    ));
                }
                self.local_notification = BootstrapLocalNotificationState::Pending;
            }
            SystemsOperation::WaitLocalNotification => {
                if self.local_notification != BootstrapLocalNotificationState::Pending {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-LOCAL-INTERRUPT",
                        "local notification wait requires the matching affine pending session",
                    ));
                }
                self.local_notification = BootstrapLocalNotificationState::Completed;
            }
            SystemsOperation::CompleteLocalNotification => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local notification completion is admitted only by its external-interrupt entry",
                ));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_deadline(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::ConstructDeadline {
                duration_nanoseconds,
            } => {
                if self.monotonic_clock_observations != 2
                    || self.deadline != BootstrapDeadlineState::Fresh
                    || *duration_nanoseconds != 1_000_000
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "the initial deadline must derive once from the second observation and exact duration 1[ms]",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Constructed;
            }
            SystemsOperation::ArmDeadline => {
                if self.deadline != BootstrapDeadlineState::Constructed
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "deadline arm requires the restored context and one fresh same-clock deadline",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Armed;
            }
            SystemsOperation::WaitDeadline => {
                if self.deadline != BootstrapDeadlineState::Armed {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "deadline wait requires the matching affine armed event",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Completed;
            }
            SystemsOperation::CompleteDeadline => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline completion is admitted only by its external-interrupt entry",
                ));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_critical(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        if !self.critical_stack.is_empty()
            && !matches!(
                operation,
                SystemsOperation::ConsoleWrite { .. }
                    | SystemsOperation::EnterCritical { .. }
                    | SystemsOperation::RestoreCritical { .. }
            )
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CRITICAL",
                "a local-interrupt critical context admits only nonblocking scoped operations and matching restoration",
            ));
        }
        match operation {
            SystemsOperation::EnterCritical { domain, .. } => {
                if !self.allocator_created || *domain != CriticalDomain::LocalMaskableInterrupts {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CRITICAL",
                        "critical entry requires the live processor context and the local-maskable-interrupts domain",
                    ));
                }
                self.next_critical_identity += 1;
                self.critical_stack
                    .push((*domain, self.next_critical_identity));
            }
            SystemsOperation::RestoreCritical { domain } => {
                let Some((current_domain, _)) = self.critical_stack.pop() else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CRITICAL",
                        "critical restore requires live affine restoration authority",
                    ));
                };
                if current_domain != *domain {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CRITICAL",
                        "critical scopes must restore their matching domain in last-in-first-out order",
                    ));
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_kernel_context(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::CreateKernelCompletion => {
                if self.deadline != BootstrapDeadlineState::Constructed
                    || self.kernel_completion != BootstrapKernelCompletionState::Fresh
                    || self.kernel_context != BootstrapKernelContextState::Fresh
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "one-shot completion creation requires the restored dispatcher before worker creation",
                    ));
                }
                self.kernel_completion = BootstrapKernelCompletionState::Created;
            }
            SystemsOperation::CreateKernelContext => {
                if self.deadline != BootstrapDeadlineState::Constructed
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context creation requires one constructed deadline and restored processor context",
                    ));
                }
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::Fresh => {
                        BootstrapKernelContextState::PreemptibleCreated
                    }
                    BootstrapKernelContextState::PreemptibleCreated => {
                        BootstrapKernelContextState::CompletionCreated
                    }
                    BootstrapKernelContextState::CompletionCreated => {
                        BootstrapKernelContextState::AllSuspended
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "the completion profile creates exactly three contexts before dispatch",
                        ));
                    }
                };
            }
            SystemsOperation::TransferKernelContext => {
                if !self.critical_stack.is_empty() {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context transfer requires no live critical scope",
                    ));
                }
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::TerminalDequeued => {
                        BootstrapKernelContextState::TerminalCompleted
                    }
                    BootstrapKernelContextState::PreemptibleDequeuedAgain => {
                        BootstrapKernelContextState::PreemptibleCompleted
                    }
                    BootstrapKernelContextState::CompletionDequeuedAgain => {
                        BootstrapKernelContextState::CompletionCompleted
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "ordinary kernel-context transfers must follow terminal, preemptible, then awakened-completion redispatch order",
                        ));
                    }
                };
            }
            SystemsOperation::TransferKernelContextUntilDeadline => {
                if !self.critical_stack.is_empty()
                    || self.deadline != BootstrapDeadlineState::Constructed
                    || self.kernel_context != BootstrapKernelContextState::PreemptibleDequeued
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "deadline-preemptible transfer requires the first FIFO context, constructed deadline, and no critical scope",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Completed;
                self.kernel_context = BootstrapKernelContextState::PreemptedOutcome;
            }
            SystemsOperation::TakePreemptedKernelContext => {
                if self.kernel_context != BootstrapKernelContextState::PreemptedOutcome
                    || self.deadline != BootstrapDeadlineState::Completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "taking a preempted context requires the matching completed deadline transfer",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::PreemptedTaken;
            }
            SystemsOperation::TransferKernelContextUntilCompletion => {
                if !self.critical_stack.is_empty()
                    || self.kernel_context != BootstrapKernelContextState::CompletionDequeued
                    || self.kernel_completion != BootstrapKernelCompletionState::Created
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "completion-bound transfer requires the selected fresh block-once worker and restored dispatcher",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::CompletionBlocked;
                self.kernel_completion = BootstrapKernelCompletionState::Blocked;
            }
            SystemsOperation::SignalBlockedKernelCompletion => {
                if self.kernel_context != BootstrapKernelContextState::TerminalReclaimed
                    || self.kernel_completion != BootstrapKernelCompletionState::Blocked
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "completion signal requires the matching blocked worker after terminal reclamation",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::CompletionSignaled;
                self.kernel_completion = BootstrapKernelCompletionState::Signaled;
            }
            SystemsOperation::CreateRunnableQueue { capacity } => {
                if self.kernel_context != BootstrapKernelContextState::AllSuspended
                    || *capacity != 3
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "the initial runnable queue requires all three suspended contexts and capacity three",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::QueueCreated;
            }
            SystemsOperation::EnqueueRunnableContext => {
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::QueueCreated => {
                        BootstrapKernelContextState::PreemptibleEnqueued
                    }
                    BootstrapKernelContextState::PreemptibleEnqueued => {
                        BootstrapKernelContextState::CompletionEnqueued
                    }
                    BootstrapKernelContextState::CompletionEnqueued => {
                        BootstrapKernelContextState::AllEnqueued
                    }
                    BootstrapKernelContextState::PreemptedTaken => {
                        BootstrapKernelContextState::PreemptedEnqueued
                    }
                    BootstrapKernelContextState::CompletionSignaled => {
                        BootstrapKernelContextState::CompletionEnqueuedAgain
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "runnable enqueue must preserve preemptible, completion, terminal, preemptible, completion FIFO order",
                        ));
                    }
                };
            }
            SystemsOperation::DequeueRunnableContext => {
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::AllEnqueued => {
                        BootstrapKernelContextState::PreemptibleDequeued
                    }
                    BootstrapKernelContextState::PreemptedEnqueued => {
                        BootstrapKernelContextState::CompletionDequeued
                    }
                    BootstrapKernelContextState::CompletionBlocked => {
                        BootstrapKernelContextState::TerminalDequeued
                    }
                    BootstrapKernelContextState::CompletionEnqueuedAgain => {
                        BootstrapKernelContextState::PreemptibleDequeuedAgain
                    }
                    BootstrapKernelContextState::PreemptibleReclaimed => {
                        BootstrapKernelContextState::CompletionDequeuedAgain
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "runnable dequeue must select preemptible, completion, terminal, preemptible, completion FIFO order",
                        ));
                    }
                };
            }
            SystemsOperation::ReclaimKernelContext => {
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::TerminalCompleted => {
                        BootstrapKernelContextState::TerminalReclaimed
                    }
                    BootstrapKernelContextState::PreemptibleCompleted => {
                        BootstrapKernelContextState::PreemptibleReclaimed
                    }
                    BootstrapKernelContextState::CompletionCompleted => {
                        BootstrapKernelContextState::CompletionReclaimed
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "kernel-context reclaim requires the matching completed transfer",
                        ));
                    }
                };
            }
            SystemsOperation::ConsumeEmptyRunnableQueue => {
                if self.kernel_context != BootstrapKernelContextState::CompletionReclaimed
                    || self.kernel_completion != BootstrapKernelCompletionState::Consumed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "runnable queue consumption requires an empty queue, three reclaimed contexts, and consumed completion",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::QueueConsumed;
            }
            SystemsOperation::ConsumeKernelCompletion => {
                if self.kernel_context != BootstrapKernelContextState::CompletionReclaimed
                    || self.kernel_completion != BootstrapKernelCompletionState::Signaled
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "one-shot completion consumption requires its awakened worker to be reclaimed",
                    ));
                }
                self.kernel_completion = BootstrapKernelCompletionState::Consumed;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
