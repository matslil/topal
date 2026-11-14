impl RootEncoder {
    fn erase_entropy_sample_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::RawEntropySample)
    }

    fn erase_entropy_pool_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::EntropyPoolState)
    }

    fn erase_entropy_generator_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::EntropyGeneratorKey)
    }

    fn erase_entropy_output_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::EntropyOutput)
    }

    fn erase_entropy_storage(&mut self, symbol: ProviderSymbol) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(symbol, 0)?;
        self.bytes
            .extend_from_slice(&[0x31, 0xc0, 0x48, 0x89, 0x07]);
        for offset in [8_u8, 16, 24] {
            self.bytes.extend_from_slice(&[0x48, 0x89, 0x47, offset]);
        }
        self.bytes.extend_from_slice(&[0x48, 0x83, 0x3f, 0]);
        self.jump_to_fatal_if(0x85)?;
        for offset in [8_u8, 16, 24] {
            self.bytes.extend_from_slice(&[0x48, 0x83, 0x7f, offset, 0]);
            self.jump_to_fatal_if(0x85)?;
        }
        Ok(())
    }

    fn emit_unaccounted_pci_region_dma_reference(
        &mut self,
        request: CompilerPciRegionDmaReferenceRequest,
    ) -> Result<(), CompileError> {
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
        self.call_checked_bool(ProviderSymbol::PublishPciRegionDmaReference)
    }

    fn emit_unaccounted_pci_configuration_word_update(
        &mut self,
        request: CompilerPciConfigurationWordUpdateRequest,
    ) -> Result<(), CompileError> {
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.register_offset_bytes & 0xfc);
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&u32::from(request.mask).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.expected_masked_value).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&u32::from(request.desired_masked_value).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&u32::from(request.register_offset_bytes & 0x02).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::UpdatePciConfigurationWord)
    }

    fn emit_unaccounted_dma_buffer_create(
        &mut self,
        request: CompilerDmaBufferRequest,
    ) -> Result<(), CompileError> {
        let packed_metadata = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => 0x01_u32,
            CompilerDmaBufferRole::EntropyPayload => 0x12_u32,
        };
        self.bytes.push(0xbf);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.byte_count as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.alignment_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb9]);
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CreateDmaBuffer)
    }

    fn emit_unaccounted_dma_buffer_bind_reference(
        &mut self,
        request: CompilerDmaBufferReferenceRequest,
    ) -> Result<(), CompileError> {
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
        self.call_checked_bool(ProviderSymbol::BindDmaBufferReference)
    }

    fn emit_unaccounted_dma_buffer_integer_store(
        &mut self,
        request: CompilerDmaBufferIntegerStoreRequest,
    ) -> Result<(), CompileError> {
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
        self.call_checked_bool(ProviderSymbol::StoreDmaBufferInteger)
    }

    fn emit_unaccounted_dma_buffer_integer_load(
        &mut self,
        request: CompilerDmaBufferIntegerLoadRequest,
    ) -> Result<(), CompileError> {
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
        self.call_checked_bool(ProviderSymbol::LoadDmaBufferInteger)
    }

    fn emit_unaccounted_dma_buffer_ownership(
        &mut self,
        request: CompilerDmaBufferPreparationRequest,
        provider: ProviderSymbol,
    ) -> Result<(), CompileError> {
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
        self.call_checked_bool(provider)
    }

    fn emit_unaccounted_entropy_sample_capture(
        &mut self,
        request: CompilerEntropySampleCaptureRequest,
    ) -> Result<(), CompileError> {
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
        self.call_checked_bool(ProviderSymbol::CaptureEntropySample)
    }

    fn emit_unaccounted_entropy_sample_quality(
        &mut self,
        request: CompilerEntropySampleQualityRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed()
            || request != CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy quality requires the exact target-independent Q35 trust transition".into(),
            ));
        }
        Ok(())
    }

    fn emit_unaccounted_entropy_pool_create(
        &mut self,
        request: CompilerEntropyPoolRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyPoolRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-pool creation requires the exact target-independent empty pool".into(),
            ));
        }
        Ok(())
    }

    fn emit_unaccounted_entropy_pool_mix(
        &mut self,
        request: CompilerEntropyPoolMixRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyPoolMixRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-pool mixing requires the exact first BLAKE2s contribution".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::RawEntropySample, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.call_checked_bool(ProviderSymbol::MixEntropyPoolSample)
    }

    fn emit_unaccounted_entropy_pool_credit(
        &mut self,
        request: CompilerEntropyPoolCreditRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyPoolCreditRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-pool credit requires the exact target-independent first contribution".into(),
            ));
        }
        Ok(())
    }

    fn emit_unaccounted_entropy_generator_initialize(
        &mut self,
        request: CompilerEntropyGeneratorRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyGeneratorRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-generator initialization requires the exact first ChaCha20 generation".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.call_checked_bool(ProviderSymbol::InitializeEntropyGenerator)
    }

    fn emit_unaccounted_entropy_generator_extract(
        &mut self,
        request: CompilerEntropyGeneratorExtractRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed()
            || request != CompilerEntropyGeneratorExtractRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy-generator extraction requires the exact first ChaCha20 output".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::ExtractEntropyGenerator)
    }

    fn emit_unaccounted_pci_region_register_access(
        &mut self,
        request: CompilerPciRegionRegisterRequest,
    ) -> Result<(), CompileError> {
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.access_capability_offset_bytes);
        let target_offset = request
            .region_offset_bytes
            .checked_add(request.register_offset_bytes)
            .ok_or_else(|| CompileError::Tool("PCI region target offset overflowed".into()))?;
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&u32::from(request.bar).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes.extend_from_slice(&target_offset.to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes.extend_from_slice(&request.value.to_le_bytes());
        let symbol = match request.direction {
            CompilerPciRegionRegisterDirection::ReadEquals => ProviderSymbol::ReadPciRegionRegister,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                ProviderSymbol::ReadPciRegionRegisterEventually
            }
            CompilerPciRegionRegisterDirection::Write => ProviderSymbol::WritePciRegionRegister,
        };
        self.call_checked_bool(symbol)
    }
}
