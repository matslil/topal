impl RootEncoder {
    fn qualify_static_image_entropy_device(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-device qualification requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyDevice)?;
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&0x8000_2800_u32.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes.extend_from_slice(&0x1044_1af4_u32.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::ObservePciConfigurationDword)
    }

    fn qualify_static_image_entropy_transport(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-transport qualification requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyTransport)?;
        for request in CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
            .iter()
            .skip(1)
        {
            let selector = 0x8000_0000
                | (u32::from(request.bus) << 16)
                | (u32::from(request.device) << 11)
                | (u32::from(request.function) << 8)
                | u32::from(request.register_offset_bytes);
            self.bytes.push(0xbf);
            self.bytes.extend_from_slice(&selector.to_le_bytes());
            self.bytes.push(0xbe);
            self.bytes
                .extend_from_slice(&request.expected_value.to_le_bytes());
            self.call_checked_bool(ProviderSymbol::ObservePciConfigurationDword)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_negotiation(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-transport negotiation requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyNegotiation)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_queue(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy-queue discovery requires the exact acquisition owner"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyQueue)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_bus_master(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy bus-master qualification requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyBusMaster)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        let enable = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        self.emit_unaccounted_pci_configuration_word_update(enable)?;
        self.emit_unaccounted_pci_configuration_word_update(
            CompilerPciConfigurationWordUpdateRequest {
                expected_masked_value: enable.desired_masked_value,
                desired_masked_value: enable.expected_masked_value,
                ..enable
            },
        )?;
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_dma_buffers(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy DMA-buffer coexistence requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyDmaBuffers)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        let enable = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        self.emit_unaccounted_pci_configuration_word_update(enable)?;
        for request in CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG {
            self.emit_unaccounted_dma_buffer_create(request)?;
        }
        self.emit_unaccounted_pci_configuration_word_update(
            CompilerPciConfigurationWordUpdateRequest {
                expected_masked_value: enable.desired_masked_value,
                desired_masked_value: enable.expected_masked_value,
                ..enable
            },
        )?;
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_split_ring(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropySplitRing,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_dma_preparation(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyDmaPreparation,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_queue_addresses(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyQueueAddresses,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_queue_activation(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyQueueActivation,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_request_completion(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyRequestCompletion,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_sample_capture(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropySampleCapture,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_sample_quality(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropySampleQuality,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_pool_lifecycle(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyPoolLifecycle,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_generator_lifecycle(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        )
    }

    fn deliver_static_image_entropy_initial_random(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::DeliverStaticImageEntropyInitialRandom,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )?;
        self.static_image = RootStaticImageState::RandomizedProcess;
        Ok(())
    }

    fn revoke_static_image_entropy_initial_random(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::RandomizedProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image initial-random revocation requires the exact randomized owner"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageEntropyInitialRandom)?;
        self.static_image = RootStaticImageState::EntropyAcquiringProcess;
        Ok(())
    }

    fn enter_static_image_process(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::RandomizedProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
            || self.monotonic_observations != 0
            || self.deadline != RootDeadlineState::Fresh
        {
            return Err(CompileError::Tool(
                "x86 static-image process entry requires the exact randomized owner".into(),
            ));
        }
        // Retain ABI stack alignment with one pad above the twelfth argument.
        // The process borrows the root's existing clock; it owns no new timeline.
        if self.process_identity_library {
            // Nineteen owns the selected store, or is a zero scalar-only pad.
            if self.retained_identity {
                self.bytes.extend_from_slice(&[0x48, 0x8d, 0x05]);
                self.library_relocations.push((
                    self.bytes.len() as u64,
                    RootLibraryDependency::RetainedState,
                ));
                self.bytes.extend_from_slice(&[0; 4]);
                self.bytes.push(0x50);
            } else {
                self.bytes.extend_from_slice(&[0x6a, 0]);
            }
            self.bytes.extend_from_slice(&[0x48, 0x8d, 0x05]);
            self.library_relocations.push((
                self.bytes.len() as u64,
                if self.structured_identity {
                    RootLibraryDependency::StructuredIdentityCode
                } else if self.retained_identity {
                    RootLibraryDependency::RetainedIdentityCode
                } else {
                    RootLibraryDependency::ProcessIdentityCode
                },
            ));
            self.bytes.extend_from_slice(&[0; 4]);
            self.bytes.push(0x50);
        }
        if self.image_library || self.dispatch_library || self.process_identity_library {
            if self.dispatch_library {
                // The seventeenth code argument replaces the previous pad.
                self.bytes.extend_from_slice(&[0x48, 0x8d, 0x05]);
                self.library_relocations
                    .push((self.bytes.len() as u64, RootLibraryDependency::DispatchCode));
                self.bytes.extend_from_slice(&[0; 4]);
                self.bytes.push(0x50);
            } else {
                self.bytes.extend_from_slice(&[0x6a, 0]);
            }
            if self.image_library {
                self.bytes.extend_from_slice(&[0x48, 0x8d, 0x05]);
                self.library_relocations
                    .push((self.bytes.len() as u64, RootLibraryDependency::ImageCode));
                self.bytes.extend_from_slice(&[0; 4]);
                self.bytes.push(0x50);
            } else {
                self.bytes.extend_from_slice(&[0x6a, 0]);
            }
        }
        if self.numeric_library
            || self.image_library
            || self.dispatch_library
            || self.process_identity_library
        {
            for dependency in [RootLibraryDependency::Arena, RootLibraryDependency::Code] {
                if matches!(dependency, RootLibraryDependency::Code) && !self.numeric_library {
                    self.bytes.extend_from_slice(&[0x6a, 0]);
                    continue;
                }
                self.bytes.extend_from_slice(&[0x48, 0x8d, 0x05]);
                self.library_relocations
                    .push((self.bytes.len() as u64, dependency));
                self.bytes.extend_from_slice(&[0; 4]);
                self.bytes.push(0x50);
            }
        }
        if self.source_bound_library {
            self.bytes.extend_from_slice(&[0x48, 0x8d, 0x05]);
            self.source_library_relocations
                .push(self.bytes.len() as u64);
            self.bytes.extend_from_slice(&[0; 4]);
            self.bytes.push(0x50); // thirteenth argument: artifact-owned pure code
        } else if self.numeric_library
            || self.image_library
            || self.dispatch_library
            || self.process_identity_library
        {
            self.bytes.extend_from_slice(&[0x6a, 0]); // no open policy binding
        } else {
            self.bytes.extend_from_slice(&[0x48, 0x83, 0xec, 8]);
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x05]);
        self.rip_relative_storage(24, 0)?;
        self.bytes.push(0x50); // twelfth argument: borrowed clock view
        self.bytes
            .extend_from_slice(&[0x41, 0x55, 0x48, 0x8d, 0x05]);
        // Eleventh argument: retained boot handoff; tenth: private descriptor table.
        self.rip_relative_symbol(ProviderSymbol::StaticProcessDescriptorTable, 0)?;
        self.bytes.push(0x50);
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, // payload
            0x4c, 0x89, 0xe6, // leaf metadata
            0xba, 0xf7, 0x01, 0, 0, // 503 leaves
            0x48, 0x89, 0xe9, // static root
            0x4d, 0x89, 0xf8, // bootstrap root
            0x49, 0x89, 0xd9, // stack backing
            0x48, 0x8d, 0x05, // private exec snapshot
        ]);
        self.rip_relative_symbol(ProviderSymbol::StaticProcessExecSnapshot, 0)?;
        self.bytes.extend_from_slice(&[
            0x50, // ninth System V argument
            0x48, 0x8d, 0x05, // private brk reserve
        ]);
        self.rip_relative_symbol(ProviderSymbol::StaticProcessBrkReserve, 0)?;
        self.bytes.extend_from_slice(&[
            0x50, // eighth System V argument
            0x48, 0x8d, 0x05, // private context
        ]);
        self.rip_relative_storage(512, 0)?;
        self.bytes.push(0x50); // seventh System V argument
        self.call_checked_bool(ProviderSymbol::EnterStaticImageProcess)?;
        self.bytes.extend_from_slice(&[
            0x48,
            0x83,
            0xc4,
            if self.process_identity_library {
                0x68
            } else if self.image_library || self.dispatch_library {
                0x58
            } else if self.numeric_library {
                0x48
            } else {
                0x38
            },
        ]);
        self.static_image = RootStaticImageState::CompletedProcess;
        Ok(())
    }

    fn restore_static_image_process_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::CompletedProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image process-entry restoration requires the exact completed owner"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(512)?;
        self.call_checked_bool(ProviderSymbol::RestoreStaticImageProcessEntry)?;
        self.static_image = RootStaticImageState::RandomizedProcess;
        Ok(())
    }

    fn qualify_static_image_entropy_ring_transaction(
        &mut self,
        provider: ProviderSymbol,
        prepare: bool,
        publish: bool,
        activate: bool,
        complete_request: bool,
        capture_sample: bool,
        assess_sample: bool,
        pool_lifecycle: bool,
        generator_lifecycle: bool,
        deliver_initial_random: bool,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy ring transaction requires the exact acquisition owner"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if deliver_initial_random {
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle
        } else {
            provider
        })?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        let enable = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        self.emit_unaccounted_pci_configuration_word_update(enable)?;
        for request in CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG {
            self.emit_unaccounted_dma_buffer_create(request)?;
        }
        self.emit_unaccounted_dma_buffer_bind_reference(
            CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR,
        )?;
        for request in CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING {
            self.emit_unaccounted_dma_buffer_integer_store(request)?;
        }
        if activate && !complete_request {
            self.emit_unaccounted_dma_buffer_integer_store(CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                value: 0,
            })?;
            self.emit_unaccounted_dma_buffer_integer_load(CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                expected_value: 0,
            })?;
        }
        if prepare {
            for request in CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG {
                self.emit_unaccounted_dma_buffer_ownership(
                    request,
                    ProviderSymbol::PrepareDmaBufferForDevice,
                )?;
            }
        }
        if publish {
            for request in
                CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
            {
                self.emit_unaccounted_pci_region_dma_reference(request)?;
            }
        }
        if activate {
            for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
            {
                self.emit_unaccounted_pci_region_register_access(request)?;
            }
            if complete_request {
                for request in
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
                for request in
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
                self.emit_unaccounted_dma_buffer_ownership(
                    CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                    ProviderSymbol::CompleteDmaBufferForCpu,
                )?;
                for request in CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                {
                    self.emit_unaccounted_dma_buffer_integer_load(request)?;
                }
                self.emit_unaccounted_dma_buffer_ownership(
                    CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                    ProviderSymbol::CompleteDmaBufferForCpu,
                )?;
                if capture_sample {
                    self.emit_unaccounted_entropy_sample_capture(
                        CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
                    )?;
                }
                if assess_sample {
                    self.emit_unaccounted_entropy_sample_quality(
                        CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
                    )?;
                }
                if pool_lifecycle {
                    self.emit_unaccounted_entropy_pool_create(
                        CompilerEntropyPoolRequest::INITIAL_Q35,
                    )?;
                    self.emit_unaccounted_entropy_pool_mix(
                        CompilerEntropyPoolMixRequest::INITIAL_Q35,
                    )?;
                    self.emit_unaccounted_entropy_pool_credit(
                        CompilerEntropyPoolCreditRequest::INITIAL_Q35,
                    )?;
                }
                if generator_lifecycle {
                    self.emit_unaccounted_entropy_generator_initialize(
                        CompilerEntropyGeneratorRequest::INITIAL_Q35,
                    )?;
                    self.emit_unaccounted_entropy_generator_extract(
                        CompilerEntropyGeneratorExtractRequest::INITIAL_Q35,
                    )?;
                }
                if deliver_initial_random {
                    self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]);
                    self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
                    self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
                    self.bytes.extend_from_slice(&[0x48, 0x8d, 0x15]);
                    self.rip_relative_storage(512, 0)?;
                    self.bytes.extend_from_slice(&[0x48, 0x8d, 0x0d]);
                    self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
                    self.call_checked_bool(provider)?;
                }
            }
            for request in [
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::Write,
                    0,
                ),
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::ReadEquals,
                    0,
                ),
                CompilerPciRegionRegisterRequest::initial(
                    0x1c,
                    2,
                    CompilerPciRegionRegisterDirection::ReadEquals,
                    0,
                ),
            ] {
                self.emit_unaccounted_pci_region_register_access(request)?;
            }
            for register_offset_bytes in [32, 40, 48] {
                for request in [
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes + 4,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                ] {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
            }
        } else if publish {
            for register_offset_bytes in [48, 40, 32] {
                for request in [
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes + 4,
                        4,
                        CompilerPciRegionRegisterDirection::Write,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes,
                        4,
                        CompilerPciRegionRegisterDirection::Write,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes + 4,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                ] {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
            }
        }
        if prepare && !complete_request {
            for request in [
                CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
            ] {
                self.emit_unaccounted_dma_buffer_ownership(
                    request,
                    ProviderSymbol::CompleteDmaBufferForCpu,
                )?;
            }
        }
        if complete_request {
            for offset_bytes in [0, 8, 16, 24] {
                self.emit_unaccounted_dma_buffer_integer_store(
                    CompilerDmaBufferIntegerStoreRequest {
                        buffer_identity: 2,
                        offset_bytes,
                        width_bytes: 8,
                        value: 0,
                    },
                )?;
            }
            for offset_bytes in [0, 8, 16, 24] {
                self.emit_unaccounted_dma_buffer_integer_load(
                    CompilerDmaBufferIntegerLoadRequest {
                        buffer_identity: 2,
                        offset_bytes,
                        width_bytes: 8,
                        expected_value: 0,
                    },
                )?;
            }
        }
        for request in [
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 156,
                width_bytes: 4,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 152,
                width_bytes: 4,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 150,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 148,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 132,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 12,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 8,
                width_bytes: 4,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 0,
                width_bytes: 8,
                value: 0,
            },
        ] {
            if complete_request || request.offset_bytes < 148 {
                self.emit_unaccounted_dma_buffer_integer_store(request)?;
            }
        }
        for request in [
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 156,
                width_bytes: 4,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 152,
                width_bytes: 4,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 150,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 148,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 132,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 12,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 8,
                width_bytes: 4,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 0,
                width_bytes: 8,
                expected_value: 0,
            },
        ] {
            if complete_request || request.offset_bytes < 148 {
                self.emit_unaccounted_dma_buffer_integer_load(request)?;
            }
        }
        if capture_sample {
            self.erase_entropy_sample_storage()?;
        }
        if pool_lifecycle {
            self.erase_entropy_pool_storage()?;
        }
        if generator_lifecycle {
            self.erase_entropy_generator_storage()?;
            self.erase_entropy_output_storage()?;
        }
        self.emit_unaccounted_pci_configuration_word_update(
            CompilerPciConfigurationWordUpdateRequest {
                expected_masked_value: enable.desired_masked_value,
                desired_masked_value: enable.expected_masked_value,
                ..enable
            },
        )?;
        if !activate {
            for request in [
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::Write,
                    0,
                ),
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::ReadEquals,
                    0,
                ),
            ] {
                self.emit_unaccounted_pci_region_register_access(request)?;
            }
        }
        Ok(())
    }
}
