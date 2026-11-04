impl RootEncoder {
    fn create_atomic_word(
        &mut self,
        request: CompilerAtomicWordRequest,
    ) -> Result<(), CompileError> {
        if request != CompilerAtomicWordRequest::initial() || self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering requires one live atomic word with the sealed request".into(),
            ));
        }
        let storage_offset = self.bootstrap_storage_offset(request.offset_bytes)?;
        self.load_bootstrap_storage_address(storage_offset)?;
        self.bytes.push(0xbe); // mov esi, imm32
        let initial_value = u32::try_from(request.initial_value)
            .map_err(|_| CompileError::Tool("atomic initial value exceeds x86 imm32".into()))?;
        self.bytes.extend_from_slice(&initial_value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::AtomicWordCreate)?;
        self.atomic_word_offset = Some(request.offset_bytes);
        Ok(())
    }

    fn compare_exchange_atomic_word(
        &mut self,
        expected: u64,
        desired: u64,
        success_order: CompilerAtomicOrder,
        failure_order: CompilerAtomicOrder,
    ) -> Result<(), CompileError> {
        if self.atomic_word_offset != Some(8)
            || (expected, desired, success_order, failure_order)
                != (
                    41,
                    42,
                    CompilerAtomicOrder::AcquireRelease,
                    CompilerAtomicOrder::Acquire,
                )
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the sealed atomic compare/exchange".into(),
            ));
        }
        self.load_bootstrap_storage_address(self.bootstrap_storage_offset(8)?)?;
        self.bytes.push(0xbe); // mov esi, imm32
        let expected = u32::try_from(expected)
            .map_err(|_| CompileError::Tool("atomic expected value exceeds x86 imm32".into()))?;
        self.bytes.extend_from_slice(&expected.to_le_bytes());
        self.bytes.push(0xba); // mov edx, imm32
        let desired = u32::try_from(desired)
            .map_err(|_| CompileError::Tool("atomic desired value exceeds x86 imm32".into()))?;
        self.bytes.extend_from_slice(&desired.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::AtomicWordCompareExchange)
    }

    fn load_atomic_word_equals(
        &mut self,
        order: CompilerAtomicOrder,
        expected: u64,
    ) -> Result<(), CompileError> {
        if self.atomic_word_offset != Some(8)
            || order != CompilerAtomicOrder::Acquire
            || expected != 42
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the sealed acquire atomic load".into(),
            ));
        }
        self.load_bootstrap_storage_address(self.bootstrap_storage_offset(8)?)?;
        self.call(ProviderSymbol::AtomicWordLoad);
        self.bytes.extend_from_slice(&[0x48, 0x83, 0xf8, 0x2a]); // cmp rax, 42
        self.jump_to_fatal_if(0x85)
    }

    fn end_atomic_word(&mut self) -> Result<(), CompileError> {
        if self.atomic_word_offset.take() != Some(8) {
            return Err(CompileError::Tool(
                "x86 root lowering encountered atomic end without a live location".into(),
            ));
        }
        Ok(())
    }

    fn send_local_notification(&mut self) -> Result<(), CompileError> {
        if self.local_notification != RootLocalNotificationState::Idle
            || self.bootstrap_region_offset.is_some()
            || self.atomic_word_offset.is_some()
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires a restored context and no live bootstrap region before local-notification send".into(),
            ));
        }
        self.load_bootstrap_storage_address(16)?;
        self.call_checked_bool(ProviderSymbol::SendLocalNotification)?;
        self.local_notification = RootLocalNotificationState::Pending;
        Ok(())
    }

    fn wait_local_notification(&mut self) -> Result<(), CompileError> {
        if self.local_notification != RootLocalNotificationState::Pending {
            return Err(CompileError::Tool(
                "x86 root lowering encountered local-notification wait without pending authority"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(16)?;
        self.call_checked_bool(ProviderSymbol::WaitLocalNotification)?;
        self.local_notification = RootLocalNotificationState::Completed;
        Ok(())
    }

    fn complete_local_notification(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::LocalNotificationHandler {
            return Err(CompileError::Tool(
                "x86 local-notification completion is admitted only in its external entry".into(),
            ));
        }
        self.load_bootstrap_storage_address(16)?;
        self.call(ProviderSymbol::CompleteLocalNotification);
        Ok(())
    }

    fn observe_monotonic_clock(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.local_notification != RootLocalNotificationState::Completed
            || self.monotonic_observations >= 2
        {
            return Err(CompileError::Tool(
                "x86 monotonic-clock observation requires the completed local-notification lifecycle and its sealed two-read sequence".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call(ProviderSymbol::ObserveMonotonicClock);
        self.bytes.extend_from_slice(&[0x85, 0xc9]); // test provider status in ecx
        self.jump_to_fatal_if(0x84)?;
        self.monotonic_observations += 1;
        Ok(())
    }

    fn observe_pci_configuration_dword(
        &mut self,
        request: CompilerPciConfigurationDwordRequest,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || !self.frame_allocator_created
            || self.critical != RootCriticalState::Restored
            || CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                .get(self.pci_configuration_observations)
                != Some(&request)
        {
            return Err(CompileError::Tool(
                "x86 PCI configuration observation requires the restored bootstrap context and exact sealed Q35 virtio-rng capability-discovery sequence".into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.register_offset_bytes);
        self.bytes.push(0xbf); // selector in edi
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe); // expected value in esi
        self.bytes
            .extend_from_slice(&request.expected_value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::ObservePciConfigurationDword)?;
        self.pci_configuration_observations += 1;
        Ok(())
    }

    fn update_pci_configuration_word(
        &mut self,
        request: CompilerPciConfigurationWordUpdateRequest,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.pci_region_register_accesses
                != CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
            || self.pci_bus_master_authorized
            || !request.is_well_formed()
            || request
                != CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER
        {
            return Err(CompileError::Tool(
                "x86 PCI configuration update requires complete queue discovery and the exact sealed bus-master command transition".into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.register_offset_bytes & 0xfc);
        self.bytes.push(0xbf); // aligned configuration selector in edi
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe); // mask in esi
        self.bytes
            .extend_from_slice(&u32::from(request.mask).to_le_bytes());
        self.bytes.push(0xba); // expected masked value in edx
        self.bytes
            .extend_from_slice(&u32::from(request.expected_masked_value).to_le_bytes());
        self.bytes.push(0xb9); // desired masked value in ecx
        self.bytes
            .extend_from_slice(&u32::from(request.desired_masked_value).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]); // data-port offset in r8d
        self.bytes
            .extend_from_slice(&u32::from(request.register_offset_bytes & 0x02).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::UpdatePciConfigurationWord)?;
        self.pci_bus_master_authorized = true;
        Ok(())
    }

    fn create_dma_buffer(&mut self, request: CompilerDmaBufferRequest) -> Result<(), CompileError> {
        let expected =
            CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.get(self.dma_buffers_created);
        if self.role != RootRole::BootstrapWithLocalNotification
            || !self.pci_bus_master_authorized
            || !request.is_well_formed()
            || expected != Some(&request)
        {
            return Err(CompileError::Tool(
                "x86 DMA-buffer creation requires bus-master authorization and the exact sealed virtio-rng buffer sequence".into(),
            ));
        }
        let packed_metadata = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => 0x01_u32,
            CompilerDmaBufferRole::EntropyPayload => 0x12_u32,
        };
        self.bytes.push(0xbf); // buffer identity in edi
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xbe); // device identity in esi
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xba); // translation-domain identity in edx
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.push(0xb9); // byte count in ecx
        self.bytes
            .extend_from_slice(&(request.byte_count as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]); // alignment in r8d
        self.bytes
            .extend_from_slice(&(request.alignment_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb9]); // packed checked metadata in r9d
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CreateDmaBuffer)?;
        self.dma_buffers_created += 1;
        Ok(())
    }

    fn bind_dma_buffer_reference(
        &mut self,
        request: CompilerDmaBufferReferenceRequest,
    ) -> Result<(), CompileError> {
        if self.dma_buffers_created != CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
            || self.dma_construction_operations != 0
            || !request.is_well_formed()
            || request != CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR
        {
            return Err(CompileError::Tool(
                "x86 DMA reference binding requires the exact sealed virtio-rng descriptor reference".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.destination_offset_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x15]);
        self.rip_relative_symbol(ProviderSymbol::DmaEntropyPayload, 0)?;
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.referenced_offset_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::BindDmaBufferReference)?;
        self.dma_construction_operations = 1;
        Ok(())
    }

    fn store_dma_buffer_integer(
        &mut self,
        request: CompilerDmaBufferIntegerStoreRequest,
    ) -> Result<(), CompileError> {
        let expected = self
            .dma_construction_operations
            .checked_sub(1)
            .and_then(|index| {
                CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.get(index)
            });
        if expected != Some(&request) || !request.is_well_formed() {
            return Err(CompileError::Tool(
                "x86 DMA integer store requires the exact sealed virtio-rng split-ring sequence"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.offset_bytes as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0xb9]);
        self.bytes.extend_from_slice(&request.value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::StoreDmaBufferInteger)?;
        self.dma_construction_operations += 1;
        Ok(())
    }

    fn prepare_dma_buffer_for_device(
        &mut self,
        request: CompilerDmaBufferPreparationRequest,
    ) -> Result<(), CompileError> {
        let expected = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG
            .get(self.dma_buffers_prepared);
        if self.dma_construction_operations
            != 1 + CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
            || expected != Some(&request)
            || !request.is_well_formed()
        {
            return Err(CompileError::Tool(
                "x86 DMA device preparation requires complete split-ring construction and the exact sealed payload-then-control sequence"
                    .into(),
            ));
        }
        let symbol = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => ProviderSymbol::DmaVirtqueueControl,
            CompilerDmaBufferRole::EntropyPayload => ProviderSymbol::DmaEntropyPayload,
        };
        let packed_metadata = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => 0x01_u32,
            CompilerDmaBufferRole::EntropyPayload => 0x12_u32,
        };
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(symbol, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::PrepareDmaBufferForDevice)?;
        self.dma_buffers_prepared += 1;
        Ok(())
    }

    fn complete_dma_buffer_for_cpu(
        &mut self,
        request: CompilerDmaBufferPreparationRequest,
    ) -> Result<(), CompileError> {
        let required_accesses = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
            .len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.len();
        let completes_control = !self.dma_control_completed
            && request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1]
            && self.dma_used_ring_reads == 0;
        let completes_payload = self.dma_control_completed
            && !self.dma_payload_completed
            && self.dma_used_ring_reads
                == CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len()
            && request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
        if self.pci_region_register_accesses != required_accesses
            || !request.is_well_formed()
            || (!completes_control && !completes_payload)
        {
            return Err(CompileError::Tool(
                "x86 DMA completion requires the exact sealed control-then-payload ownership sequence around used-ring validation".into(),
            ));
        }
        let (symbol, packed_metadata) = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => {
                (ProviderSymbol::DmaVirtqueueControl, 0x01_u32)
            }
            CompilerDmaBufferRole::EntropyPayload => (ProviderSymbol::DmaEntropyPayload, 0x12_u32),
        };
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(symbol, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CompleteDmaBufferForCpu)?;
        if completes_control {
            self.dma_control_completed = true;
        } else {
            self.dma_payload_completed = true;
        }
        Ok(())
    }

    fn load_dma_buffer_integer(
        &mut self,
        request: CompilerDmaBufferIntegerLoadRequest,
    ) -> Result<(), CompileError> {
        let expected = CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
            .get(self.dma_used_ring_reads);
        if !self.dma_control_completed || expected != Some(&request) || !request.is_well_formed() {
            return Err(CompileError::Tool(
                "x86 DMA integer read requires the exact sealed virtio-rng used-ring sequence"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.offset_bytes as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0xb9]);
        self.bytes
            .extend_from_slice(&request.expected_value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::LoadDmaBufferInteger)?;
        self.dma_used_ring_reads += 1;
        Ok(())
    }

    fn capture_entropy_sample(
        &mut self,
        request: CompilerEntropySampleCaptureRequest,
    ) -> Result<(), CompileError> {
        if !self.dma_payload_completed
            || self.entropy_sample_captured
            || !request.is_well_formed()
            || request != CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG
        {
            return Err(CompileError::Tool(
                "x86 entropy-sample capture requires the exact sealed uncredited virtio-rng payload range after CPU completion".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaEntropyPayload, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::RawEntropySample, 0)?;
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.sample_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.byte_count as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb9]);
        self.bytes
            .extend_from_slice(&(request.credited_entropy_bits as u32).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CaptureEntropySample)?;
        self.entropy_sample_captured = true;
        Ok(())
    }

    fn assess_entropy_sample_quality(
        &mut self,
        request: CompilerEntropySampleQualityRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_sample_captured
            || self.entropy_sample_assessed
            || !request.is_well_formed()
            || request != CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
        {
            return Err(CompileError::Tool(
                "x86 entropy-sample quality assessment requires the exact sealed Q35 trust evidence and full-quality credit after capture".into(),
            ));
        }
        self.entropy_sample_assessed = true;
        Ok(())
    }

    fn create_entropy_pool(
        &mut self,
        request: CompilerEntropyPoolRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_sample_assessed
            || self.entropy_pool_created
            || !request.is_well_formed()
            || request != CompilerEntropyPoolRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-pool creation requires the exact sealed empty 256-bit pool after sample assessment".into(),
            ));
        }
        self.entropy_pool_created = true;
        Ok(())
    }

    fn mix_entropy_pool_sample(
        &mut self,
        request: CompilerEntropyPoolMixRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_sample_assessed
            || !self.entropy_pool_created
            || self.entropy_pool_mixed
            || !request.is_well_formed()
            || request != CompilerEntropyPoolMixRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-pool mixing requires the exact sealed BLAKE2s first-generation request after assessment and empty-pool creation".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::RawEntropySample, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.call_checked_bool(ProviderSymbol::MixEntropyPoolSample)?;
        self.entropy_pool_mixed = true;
        Ok(())
    }

    fn credit_entropy_pool_contribution(
        &mut self,
        request: CompilerEntropyPoolCreditRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_pool_mixed
            || self.entropy_pool_credited
            || !request.is_well_formed()
            || request != CompilerEntropyPoolCreditRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-credit transfer requires the exact sealed mixed pool and first contribution".into(),
            ));
        }
        self.entropy_pool_credited = true;
        Ok(())
    }

    fn initialize_entropy_generator(
        &mut self,
        request: CompilerEntropyGeneratorRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_pool_credited
            || self.entropy_generator_ready
            || !request.is_well_formed()
            || request != CompilerEntropyGeneratorRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-generator initialization requires the exact sealed credited pool and generation-one ChaCha20 request".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.call_checked_bool(ProviderSymbol::InitializeEntropyGenerator)?;
        self.entropy_generator_ready = true;
        Ok(())
    }

    fn extract_entropy_generator(
        &mut self,
        request: CompilerEntropyGeneratorExtractRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_generator_ready || !request.is_well_formed() {
            return Err(CompileError::Tool(
                "x86 entropy-generator extraction requires the next exact sealed ready ChaCha20 generation".into(),
            ));
        }
        if request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35 {
            if self.entropy_generator_extracted {
                return Err(CompileError::Tool(
                    "x86 first entropy-generator extraction was already consumed".into(),
                ));
            }
        } else if request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM {
            if !self.entropy_generator_extracted
                || !self.initial_auxv_random_delivered
                || self.initial_getrandom_extracted
            {
                return Err(CompileError::Tool(
                    "x86 second entropy-generator extraction requires consumed first output after initial AT_RANDOM delivery".into(),
                ));
            }
        } else {
            return Err(CompileError::Tool(
                "x86 entropy-generator extraction request is not admitted by the initial profile"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::ExtractEntropyGenerator)?;
        if request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35 {
            self.entropy_generator_extracted = true;
        } else {
            self.initial_getrandom_extracted = true;
        }
        Ok(())
    }

    fn deliver_initial_auxv_random(
        &mut self,
        request: CompilerEntropyOutputDeliveryRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_generator_extracted
            || self.initial_auxv_random_delivered
            || self.user_execution != RootUserExecutionState::PreparedWithResidentImage
            || !request.is_well_formed()
            || request != CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
        {
            return Err(CompileError::Tool(
                "x86 initial AT_RANDOM delivery requires the exact live first output and prepared resident initial process"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::DeliverInitialAuxvRandom)?;
        self.initial_auxv_random_delivered = true;
        self.user_execution = RootUserExecutionState::RandomizedEntryReady;
        Ok(())
    }

    fn reserve_initial_user_random(
        &mut self,
        request: CompilerEntropyOutputReservationRequest,
    ) -> Result<(), CompileError> {
        if !self.initial_getrandom_extracted
            || self.initial_user_random_reserved
            || self.user_execution != RootUserExecutionState::RandomizedEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 initial getrandom reservation requires the exact live second output and randomized entry-ready process"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::ReserveInitialUserRandom)?;
        self.initial_user_random_reserved = true;
        self.user_execution = RootUserExecutionState::UserRandomReservedEntryReady;
        Ok(())
    }

    fn bind_reusable_user_random(
        &mut self,
        request: CompilerEntropyUserRandomServiceRequest,
    ) -> Result<(), CompileError> {
        if !self.initial_user_random_reserved
            || self.user_random_service_bound
            || self.user_execution != RootUserExecutionState::UserRandomReservedEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 reusable getrandom binding requires the exact live generation-three reservation and entry-ready process"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x15]);
        self.rip_relative_symbol(ProviderSymbol::EntropyUserRandomService, 0)?;
        self.call_checked_bool(ProviderSymbol::BindReusableUserRandom)?;
        self.initial_user_random_reserved = false;
        self.user_random_service_bound = true;
        self.user_execution = RootUserExecutionState::UserRandomServiceEntryReady;
        Ok(())
    }

    fn expand_small_user_random_requests(
        &mut self,
        request: CompilerEntropyUserRandomSmallRequest,
    ) -> Result<(), CompileError> {
        if !self.user_random_service_bound
            || self.user_random_small_requests_enabled
            || self.user_execution != RootUserExecutionState::UserRandomServiceEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyUserRandomSmallRequest::INITIAL_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 small-request getrandom refinement requires the exact live reusable service before entry"
                    .into(),
            ));
        }
        self.user_random_small_requests_enabled = true;
        Ok(())
    }

    fn expand_large_user_random_requests(
        &mut self,
        request: CompilerEntropyUserRandomSmallRequest,
    ) -> Result<(), CompileError> {
        if !self.user_random_service_bound
            || !self.user_random_small_requests_enabled
            || self.user_random_large_requests_enabled
            || self.user_execution != RootUserExecutionState::UserRandomServiceEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyUserRandomSmallRequest::LARGE_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 large-request getrandom refinement requires the exact live small-request service before entry"
                    .into(),
            ));
        }
        self.user_random_large_requests_enabled = true;
        Ok(())
    }

    fn publish_pci_region_dma_reference(
        &mut self,
        request: CompilerPciRegionDmaReferenceRequest,
    ) -> Result<(), CompileError> {
        let expected = CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
            .get(self.dma_references_published);
        if self.dma_buffers_prepared
            != CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len()
            || expected != Some(&request)
            || !request.is_well_formed()
        {
            return Err(CompileError::Tool(
                "x86 PCI-region DMA-reference publication requires both device-owned buffers and the exact sealed queue-address sequence"
                    .into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.access_capability_offset_bytes);
        let target_offset = request
            .region_offset_bytes
            .checked_add(request.register_offset_bytes)
            .ok_or_else(|| {
                CompileError::Tool("PCI DMA-reference target offset overflowed".into())
            })?;
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&u32::from(request.bar).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes.extend_from_slice(&target_offset.to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x0d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.buffer_offset_bytes as u32).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::PublishPciRegionDmaReference)?;
        self.dma_references_published += 1;
        Ok(())
    }

    fn access_pci_region_register(
        &mut self,
        request: CompilerPciRegionRegisterRequest,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.pci_configuration_observations
                != CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
            || {
                let negotiation_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len();
                let discovery_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len();
                let activation_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len();
                let notification_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                        .len();
                let expected = if self.pci_region_register_accesses < negotiation_len {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        .get(self.pci_region_register_accesses)
                } else if self.pci_region_register_accesses < negotiation_len + discovery_len {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                        .get(self.pci_region_register_accesses - negotiation_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len
                {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                        .get(self.pci_region_register_accesses - negotiation_len - discovery_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len + notification_len
                {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len,
                    )
                } else {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len
                            - notification_len,
                    )
                };
                expected != Some(&request)
                    || (self.pci_region_register_accesses >= negotiation_len + discovery_len
                        && self.dma_references_published
                            != CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES.len())
            }
        {
            return Err(CompileError::Tool(
                "x86 PCI region access requires completed capability discovery and the exact sealed virtio-rng negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences".into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.access_capability_offset_bytes);
        let target_offset = request
            .region_offset_bytes
            .checked_add(request.register_offset_bytes)
            .ok_or_else(|| CompileError::Tool("PCI region target offset overflowed".into()))?;
        self.bytes.push(0xbf); // configuration access selector in edi
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe); // BAR index in esi
        self.bytes
            .extend_from_slice(&u32::from(request.bar).to_le_bytes());
        self.bytes.push(0xba); // BAR-relative target offset in edx
        self.bytes.extend_from_slice(&target_offset.to_le_bytes());
        self.bytes.push(0xb9); // exact access width in ecx
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]); // value in r8d
        self.bytes.extend_from_slice(&request.value.to_le_bytes());
        let symbol = match request.direction {
            CompilerPciRegionRegisterDirection::ReadEquals => ProviderSymbol::ReadPciRegionRegister,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                ProviderSymbol::ReadPciRegionRegisterEventually
            }
            CompilerPciRegionRegisterDirection::Write => ProviderSymbol::WritePciRegionRegister,
        };
        self.call_checked_bool(symbol)?;
        self.pci_region_register_accesses += 1;
        Ok(())
    }
}
