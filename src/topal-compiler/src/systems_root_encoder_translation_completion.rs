impl RootEncoder {
    fn revoke_static_image_entropy_acquisition(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_acquisition(false)
    }

    fn remove_static_image_stack_program_headers(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_program_headers(false)
    }

    fn remove_static_image_stack_entry_fact(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackEntryFactAdded
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_ENTRY removal requires one entry-fact-qualified stack entry under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::RemoveStaticImageStackEntryFact)?;
        self.static_image = RootStaticImageState::StackPageSizeAdded;
        Ok(())
    }

    fn remove_static_image_stack_pagesz(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackPageSizeAdded
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_PAGESZ removal requires one extended stack entry under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::RemoveStaticImageStackPageSize)?;
        self.static_image = RootStaticImageState::StackEntryInitialized;
        Ok(())
    }

    fn revoke_static_image_stack_leaf(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackLeafPublished
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-leaf revocation requires one published leaf under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageStackLeaf)?;
        self.static_image = RootStaticImageState::StackBackingZeroed;
        Ok(())
    }

    fn revoke_static_image_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntryAuthorized
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entry revocation requires one authorized construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageEntry)?;
        self.static_image = RootStaticImageState::EntryRestored;
        Ok(())
    }

    fn reclaim_static_image_hierarchy(&mut self) -> Result<(), CompileError> {
        if self.static_image != RootStaticImageState::Detached
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image hierarchy reclaim requires one detached construction".into(),
            ));
        }
        self.physical_frame_count = Some(5);
        self.static_image = RootStaticImageState::HierarchyReturned;
        Ok(())
    }

    fn reclaim_static_image_payload(&mut self) -> Result<(), CompileError> {
        if self.static_image != RootStaticImageState::MetadataReturned
            || self.physical_frame_count != Some(503)
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image payload reclaim requires revoked leaves and released metadata"
                    .into(),
            ));
        }
        self.static_image = RootStaticImageState::PayloadReturned;
        Ok(())
    }

    fn begin_translation(
        &mut self,
        request: CompilerTranslationUpdateRequest,
    ) -> Result<(), CompileError> {
        if !self.frame_allocator_created
            || self.physical_frame_count.is_some()
            || self.kernel_mapping_live
            || self.translation != RootTranslationState::BootstrapActive
            || request != CompilerTranslationUpdateRequest::initial_bootstrap_equivalent()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the live allocator, no borrowed frames, and the sealed translation request".into(),
            ));
        }
        if self.frame_policy.is_some() {
            self.frame_internal_allocate(5, "bootstrap-translation",
                crate::systems_frame_ownership::Purpose::BootstrapHierarchy)?;
        } else {
            self.call(ProviderSymbol::BeginTranslation);
        }
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xc4]); // retain root in r12
        self.translation = RootTranslationState::Update;
        Ok(())
    }

    fn commit_translation(&mut self) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::Update {
            return Err(CompileError::Tool(
                "x86 root lowering encountered translation commit without an update".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // rdi = r12
        self.call_checked_bool(ProviderSymbol::CommitTranslation)?;
        self.translation = RootTranslationState::InactiveSpace;
        Ok(())
    }

    fn activate_translation(&mut self) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::InactiveSpace {
            return Err(CompileError::Tool(
                "x86 root lowering encountered translation activation without an inactive space"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // rdi = r12
        self.call_checked_bool(ProviderSymbol::ActivateTranslation)?;
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xe7]); // retain active root in r15
        self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]); // consume inactive-space token
        self.translation = RootTranslationState::ReplacementActive;
        Ok(())
    }

    fn begin_translation_edit(
        &mut self,
        kind: CompilerTranslationEditKind,
    ) -> Result<(), CompileError> {
        match kind {
            CompilerTranslationEditKind::Map
                if self.translation == RootTranslationState::ReplacementActive
                    && self.physical_frame_count == Some(1)
                    && !self.kernel_mapping_live =>
            {
                self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]); // payload in rdi
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // boot params in rsi
                self.bytes.extend_from_slice(&[0x31, 0xd2]); // map kind
                if self.frame_policy.is_some() {
                    self.frame_internal_allocate(2, "translation-edit",
                        crate::systems_frame_ownership::Purpose::TranslationEdit)?;
                } else {
                    self.call(ProviderSymbol::BeginTranslationEdit);
                }
                self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
                self.jump_to_fatal_if(0x84)?;
                self.bytes.extend_from_slice(&[0x49, 0x89, 0xc4]); // metadata in r12
                self.translation = RootTranslationState::MapEdit;
            }
            CompilerTranslationEditKind::Unmap
                if self.translation == RootTranslationState::MappedActive
                    && self.kernel_mapping_live
                    && self.physical_frame_count.is_none() =>
            {
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]); // active root
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf6]); // mapping
                self.bytes
                    .extend_from_slice(&[0xba, 0x01, 0x00, 0x00, 0x00]);
                self.call_checked_bool(ProviderSymbol::BeginTranslationEdit)?;
                self.translation = RootTranslationState::UnmapEdit;
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root lowering encountered translation edit begin without matching active authority".into(),
                ));
            }
        }
        Ok(())
    }

    fn map_translation_frames(
        &mut self,
        request: CompilerTranslationMappingRequest,
    ) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::MapEdit
            || self.physical_frame_count != Some(1)
            || self.kernel_mapping_live
            || request != CompilerTranslationMappingRequest::initial_read_write()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires a map edit, live frame, and sealed active mapping policy".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // metadata PD
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // payload frame
        self.call_checked_bool(ProviderSymbol::MapTranslationFrames)?;
        self.physical_frame_count = None;
        self.translation = RootTranslationState::MapStaged;
        Ok(())
    }

    fn unmap_translation_mapping(&mut self) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::UnmapEdit
            || !self.kernel_mapping_live
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires an unmap edit and its live mapping".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf6]);
        self.call_checked_bool(ProviderSymbol::UnmapTranslationMapping)?;
        self.kernel_mapping_live = false;
        self.translation = RootTranslationState::UnmapStaged;
        Ok(())
    }

    fn commit_translation_edit(
        &mut self,
        kind: CompilerTranslationEditKind,
    ) -> Result<(), CompileError> {
        match kind {
            CompilerTranslationEditKind::Map
                if self.translation == RootTranslationState::MapStaged =>
            {
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]);
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
                self.bytes.extend_from_slice(&[0x31, 0xd2]);
                self.call(ProviderSymbol::CommitTranslationEdit);
                self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
                self.jump_to_fatal_if(0x84)?;
                self.bytes.extend_from_slice(&[0x49, 0x89, 0xc6]); // opaque mapping
                self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]);
                self.kernel_mapping_live = true;
                self.translation = RootTranslationState::MappedActive;
            }
            CompilerTranslationEditKind::Unmap
                if self.translation == RootTranslationState::UnmapStaged =>
            {
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]);
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf6]);
                self.bytes
                    .extend_from_slice(&[0xba, 0x01, 0x00, 0x00, 0x00]);
                self.call_checked_bool(ProviderSymbol::CommitTranslationEdit)?;
                self.bytes.extend_from_slice(&[0x4d, 0x31, 0xf6]);
                self.physical_frame_count = Some(1);
                self.translation = RootTranslationState::RemovedActive;
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root lowering encountered translation edit commit without a matching candidate".into(),
                ));
            }
        }
        Ok(())
    }

    fn enter_critical(&mut self, domain: CompilerCriticalDomain) -> Result<(), CompileError> {
        if domain != CompilerCriticalDomain::LocalMaskableInterrupts
            || self.critical == RootCriticalState::Live
            || self.physical_frame_count.is_some()
            || self.kernel_mapping_live
            || self.translation != RootTranslationState::RemovedActive
            || self.static_image != RootStaticImageState::PayloadReturned
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the restored active context and one non-nested local-interrupt critical scope".into(),
            ));
        }
        self.call(ProviderSymbol::EnterCritical);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]); // validate opaque token
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xc4]); // retain token in r12
        self.critical = RootCriticalState::Live;
        Ok(())
    }

    fn restore_critical(&mut self, domain: CompilerCriticalDomain) -> Result<(), CompileError> {
        if domain != CompilerCriticalDomain::LocalMaskableInterrupts
            || self.critical != RootCriticalState::Live
        {
            return Err(CompileError::Tool(
                "x86 root lowering encountered critical restoration without matching affine authority".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // opaque token in rdi
        self.call_checked_bool(ProviderSymbol::RestoreCritical)?;
        self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]); // consume token
        self.critical = RootCriticalState::Restored;
        Ok(())
    }

    fn complete(&self) -> Result<(), CompileError> {
        if self.image_backing_live {
            return Err(CompileError::Tool("root ended with live image-backing preparation".into()));
        }
        if let Some(frames) = &self.frame_policy {
            if frames.current.is_some()
                || frames.payload.is_some()
                || frames
                    .owners
                    .keys()
                    .any(|key| *key != "bootstrap-translation")
            {
                return Err(CompileError::Tool(format!(
                    "root lowering ended with unretired affine frame handles: {:?}",
                    frames.owners
                )));
            }
        }
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live atomic word".into(),
            ));
        }
        if self.bootstrap_region_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live bootstrap region".into(),
            ));
        }
        if self.physical_frame_count.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live physical-frame extent".into(),
            ));
        }
        if self.kernel_mapping_live {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live kernel mapping".into(),
            ));
        }
        if self.critical == RootCriticalState::Live {
            return Err(CompileError::Tool(
                "x86 root lowering ended with live critical restoration authority".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.pci_configuration_observations
                != CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete sealed PCI capability-discovery sequence".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.pci_region_register_accesses
                != CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                        .len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                        .len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                        .len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete sealed virtio-rng feature-negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.pci_bus_master_authorized
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its sealed PCI bus-master authorization".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_buffers_created != CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without both sealed CPU-owned virtio-rng DMA buffers"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_construction_operations
                != 1 + CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete CPU-owned virtio-rng split ring"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_buffers_prepared
                != CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete payload-then-control DMA preparation sequence"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_references_published
                != CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                    .len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without all three sealed virtio-rng queue-address publications"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && (!self.dma_control_completed
                || self.dma_used_ring_reads
                    != CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len())
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without CPU return and exact validation of the sealed virtio-rng used ring".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.dma_payload_completed {
            return Err(CompileError::Tool(
                "x86 root lowering ended without CPU return of the completed virtio-rng payload"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_sample_captured {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed uncredited virtio-rng sample capture"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_sample_assessed {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed trusted-host virtio-rng sample assessment"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_pool_created {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed empty entropy pool".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_pool_mixed {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed first entropy-pool mix".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_pool_credited {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed first entropy-credit transfer".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_generator_ready {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed generation-one entropy generator"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && !self.entropy_generator_extracted
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed first entropy-generator extraction"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && !self.initial_auxv_random_delivered
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without affine initial AT_RANDOM delivery".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && (!self.initial_getrandom_extracted
                || self.initial_user_random_reserved
                || !self.user_random_service_bound
                || !self.user_random_small_requests_enabled
                || !self.user_random_large_requests_enabled)
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed reusable getrandom service".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.local_notification != RootLocalNotificationState::Completed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without completing its local-notification lifecycle"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && self.monotonic_observations != 2
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without exactly two monotonic-clock observations".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.deadline != RootDeadlineState::Completed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without completing its deadline lifecycle".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.kernel_context != RootKernelContextState::QueueConsumed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without reclaiming all contexts and consuming its runnable queue"
                    .into(),
            ));
        }
        if self.context_stack_offsets != [None, None, None] {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live cooperative context stack".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.completion != RootCompletionState::Consumed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live one-shot completion".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.user_execution != RootUserExecutionState::Consumed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live initial user execution".into(),
            ));
        }
        if matches!(
            self.translation,
            RootTranslationState::Update
                | RootTranslationState::InactiveSpace
                | RootTranslationState::MapEdit
                | RootTranslationState::MapStaged
                | RootTranslationState::UnmapEdit
                | RootTranslationState::UnmapStaged
        ) {
            return Err(CompileError::Tool(
                "x86 root lowering ended with an incomplete translation lifecycle".into(),
            ));
        }
        Ok(())
    }

    fn call_checked_bool(&mut self, target: ProviderSymbol) -> Result<(), CompileError> {
        self.call(target);
        self.bytes.extend_from_slice(&[0x84, 0xc0]);
        self.jump_to_fatal_if(0x84)
    }

    fn jump_to_fatal_if(&mut self, condition: u8) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x0f, condition]);
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target: ProviderSymbol::Fatal,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
        Ok(())
    }

    fn console_write(&mut self, text: &str) {
        for byte in text.as_bytes() {
            self.bytes.push(0xbf);
            self.bytes
                .extend_from_slice(&u32::from(*byte).to_le_bytes());
            self.call(ProviderSymbol::Uart16550Write);
        }
    }

    fn call(&mut self, target: ProviderSymbol) {
        self.bytes.push(0xe8);
        self.relocations.push(PendingRelocation {
            offset: self.bytes.len() as u64,
            target,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn jump(&mut self, target: ProviderSymbol) {
        self.bytes.push(0xe9);
        self.relocations.push(PendingRelocation {
            offset: self.bytes.len() as u64,
            target,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn push(&mut self, register: SavedRegister) {
        let (rex, opcode) = register_encoding(register, 0x50);
        if let Some(rex) = rex {
            self.bytes.push(rex);
        }
        self.bytes.push(opcode);
    }

    fn pop(&mut self, register: SavedRegister) {
        let (rex, opcode) = register_encoding(register, 0x58);
        if let Some(rex) = rex {
            self.bytes.push(rex);
        }
        self.bytes.push(opcode);
    }

    /// A completed event cannot grant a second affine source entry. A raw
    /// controller notification can remain queued while its first interrupt
    /// frame is suspended, so acknowledge it before source dispatch.
    fn filter_completed_deadline_notification(&mut self) -> Result<(), CompileError> {
        self.load_bootstrap_storage_address(24)?;
        self.bytes
            .extend_from_slice(&[0x80, 0x7f, 0x28, 3, 0x0f, 0x85]);
        let branch = self.bytes.len();
        self.bytes.extend_from_slice(&[0; 4]);
        self.bytes.extend_from_slice(&[0x48, 0xb8]);
        self.bytes.extend_from_slice(&0xfee0_00b0_u64.to_le_bytes());
        self.bytes.extend_from_slice(&[0xc7, 0, 0, 0, 0, 0]);
        for register in SAVED_REGISTERS.into_iter().rev() {
            self.pop(register);
        }
        self.jump(ProviderSymbol::InterruptReturn);
        let distance = i32::try_from(self.bytes.len() - branch - 4)
            .map_err(|_| CompileError::Tool("deadline filter branch exceeds i32".into()))?;
        self.bytes[branch..branch + 4].copy_from_slice(&distance.to_le_bytes());
        Ok(())
    }

    fn allocate_bootstrap_region(
        &mut self,
        request: CompilerBootstrapStorageRequest,
    ) -> Result<(), CompileError> {
        if self.bootstrap_region_offset.is_some()
            || request.placement != CompilerBootstrapStoragePlacement::BootstrapReclaimable
            || !matches!(
                (request.byte_count, request.alignment_bytes),
                (64, 8) | (16_384, 16)
            )
        {
            return Err(CompileError::Tool(
                "x86 root lowering encountered an overlapping or unsealed bootstrap region".into(),
            ));
        }
        let mask = request.alignment_bytes - 1;
        let mut offset = self
            .bootstrap_next_offset
            .checked_add(mask)
            .ok_or_else(|| CompileError::Tool("bootstrap placement overflows".into()))?
            & !mask;
        // The low page belongs to provider event/context control. No ordinary
        // region (including the 64-byte source witness) may expose those bytes.
        offset = offset.max(4096);
        self.bootstrap_next_offset = offset
            .checked_add(request.byte_count)
            .ok_or_else(|| CompileError::Tool("bootstrap placement overflows".into()))?;
        if self.bootstrap_next_offset > self.bootstrap_capacity {
            return Err(CompileError::Tool("x86 bootstrap placement exceeds the declared pool after private-prefix reservation".into()));
        }
        self.bootstrap_region_offset = Some(offset);
        self.bootstrap_region_bytes = Some(request.byte_count);
        Ok(())
    }

    fn store_bootstrap_byte(&mut self, offset: u64, value: u8) -> Result<(), CompileError> {
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering rejected plain access while an atomic word is live".into(),
            ));
        }
        let storage_offset = self.bootstrap_storage_offset(offset)?;
        self.bytes.extend_from_slice(&[0xc6, 0x05]);
        self.rip_relative_storage(storage_offset, 1)?;
        self.bytes.push(value);
        Ok(())
    }

    fn call_region_dependency(&mut self, dependency: RootLibraryDependency) {
        self.bytes.push(0xe8);
        self.library_relocations
            .push((self.bytes.len() as u64, dependency));
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn apply_region_library(&mut self) -> Result<(), CompileError> {
        if !self.region_library
            || self.atomic_word_offset.is_some()
            || self.bootstrap_region_bytes != Some(64)
        {
            return Err(CompileError::Tool(
                "E-SYSTEMS-REGION-ROOT: invocation requires the selected plain owner".into(),
            ));
        }
        let offset = self.bootstrap_storage_offset(0)?;
        self.load_bootstrap_storage_address(offset)?;
        self.bytes.extend_from_slice(&[0x48, 0xbe]);
        self.bytes.extend_from_slice(&64_u64.to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0xba]);
        self.bytes.extend_from_slice(
            &offset
                .checked_add(1)
                .ok_or_else(|| CompileError::Tool("root region owner identity overflow".into()))?
                .to_le_bytes(),
        );
        self.call_region_dependency(RootLibraryDependency::RegionApply);
        Ok(())
    }

    fn load_bootstrap_byte_equals(
        &mut self,
        offset: u64,
        expected: u8,
    ) -> Result<(), CompileError> {
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering rejected plain access while an atomic word is live".into(),
            ));
        }
        let storage_offset = self.bootstrap_storage_offset(offset)?;
        self.bytes.extend_from_slice(&[0x0f, 0xb6, 0x05]);
        self.rip_relative_storage(storage_offset, 0)?;
        self.bytes.extend_from_slice(&[0x3c, expected, 0x0f, 0x85]);
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target: ProviderSymbol::Fatal,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
        Ok(())
    }

    fn release_bootstrap_region(&mut self) -> Result<(), CompileError> {
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering rejected region release while an atomic word is live".into(),
            ));
        }
        if self.bootstrap_region_offset.take().is_none() {
            return Err(CompileError::Tool(
                "x86 root lowering encountered release without a live bootstrap region".into(),
            ));
        }
        self.bootstrap_region_bytes = None;
        Ok(())
    }
}
