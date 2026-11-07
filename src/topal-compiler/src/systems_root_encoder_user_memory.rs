impl RootEncoder {
    fn prepare_user_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::QueueConsumed
            || self.completion != RootCompletionState::Consumed
            || self.user_execution != RootUserExecutionState::Fresh
            || self.physical_frame_count != Some(3)
            || self.translation != RootTranslationState::RemovedActive
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 user-address-space preparation requires completed kernel cleanup, exact three-frame backing, and the restored replacement translation".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xda]); // backing base in rdx
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::PrepareUserAddressSpace)?;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
        self.physical_frame_count = None;
        self.user_execution = RootUserExecutionState::AddressSpacePrepared;
        Ok(())
    }

    fn prepare_user_probe(&mut self, process_identity: u64) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::AnonymousUserMappingReserved
            || self.physical_frame_count.is_some()
            || process_identity != 1
        {
            return Err(CompileError::Tool(
                "x86 user probe preparation requires PID one, both finalized image mappings, and no source-owned extent".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]); // syscall entry in RSI
        self.rip_relative_symbol(ProviderSymbol::UserSyscall, 0)?;
        self.call_checked_bool(ProviderSymbol::PrepareUserProbe)?;
        self.user_execution = RootUserExecutionState::PreparedWithResidentImage;
        Ok(())
    }

    fn verify_user_address_space_mapping(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::AddressSpacePrepared
            || self.physical_frame_count != Some(2)
        {
            return Err(CompileError::Tool(
                "x86 user mapping verification requires the prepared address space and its two-frame code extent".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xda]); // disjoint frame base in rdx
        self.call_checked_bool(ProviderSymbol::VerifyUserAddressSpaceMapping)?;
        self.user_execution = RootUserExecutionState::AddressSpaceVerified;
        Ok(())
    }

    fn map_user_address_space(&mut self) -> Result<(), CompileError> {
        let generation = match self.user_execution {
            RootUserExecutionState::AddressSpacePrepared => 1,
            RootUserExecutionState::UserMappingExecutable(1) => 2,
            _ => 0,
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || generation == 0
            || self.physical_frame_count != Some(if generation == 1 { 2 } else { 1 })
        {
            return Err(CompileError::Tool(
                "x86 user map requires the prepared address space and exact role extent".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xda]);
        self.call_checked_bool(ProviderSymbol::MapUserAddressSpace)?;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
        self.physical_frame_count = None;
        self.user_execution = RootUserExecutionState::UserMappingLive(generation);
        Ok(())
    }

    fn store_user_mapping_byte(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingLive(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping store requires the exact live mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::StoreUserMappingByte)?;
        let RootUserExecutionState::UserMappingLive(generation) = self.user_execution else {
            unreachable!("live mapping checked above");
        };
        self.user_execution = RootUserExecutionState::UserMappingStored(generation);
        Ok(())
    }

    fn install_user_mapping_entry_trampoline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingLive(1)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 entry-trampoline installation requires the first exact live mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::InstallUserMappingEntryTrampoline)?;
        self.user_execution = RootUserExecutionState::UserMappingEntryInstalled(1);
        Ok(())
    }

    fn protect_user_mapping_read_only(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || !matches!(
                self.user_execution,
                RootUserExecutionState::UserMappingStored(_)
                    | RootUserExecutionState::UserMappingEntryInstalled(_)
            )
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping protection requires the exact stored writable mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::ProtectUserMappingReadOnly)?;
        let generation = match self.user_execution {
            RootUserExecutionState::UserMappingStored(generation)
            | RootUserExecutionState::UserMappingEntryInstalled(generation) => generation,
            _ => unreachable!("populated mapping checked above"),
        };
        self.user_execution = RootUserExecutionState::UserMappingProtected(generation);
        Ok(())
    }

    fn load_user_mapping_byte(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingProtected(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping load requires the exact protected mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::LoadUserMappingByte)?;
        let RootUserExecutionState::UserMappingProtected(generation) = self.user_execution else {
            unreachable!("protected mapping checked above");
        };
        self.user_execution = RootUserExecutionState::UserMappingVerified(generation);
        Ok(())
    }

    fn verify_user_mapping_entry_trampoline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingProtected(1)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 entry-trampoline verification requires the first exact protected mapping"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::VerifyUserMappingEntryTrampoline)?;
        self.user_execution = RootUserExecutionState::UserMappingVerified(1);
        Ok(())
    }

    fn permit_user_mapping_execute(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingVerified(1)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping execute promotion requires the exact verified mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::PermitUserMappingExecute)?;
        let RootUserExecutionState::UserMappingVerified(generation) = self.user_execution else {
            unreachable!("verified mapping checked above");
        };
        self.user_execution = RootUserExecutionState::UserMappingExecutable(generation);
        Ok(())
    }

    fn seal_user_mapping_immutable_data(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingVerified(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 immutable-data seal requires the exact second verified mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::SealUserMappingImmutableData)?;
        self.user_execution = RootUserExecutionState::UserMappingImmutableData(2);
        Ok(())
    }

    fn permit_user_mapping_writable_data(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingVerified(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 writable-data promotion requires the exact second verified mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::PermitUserMappingWritableData)?;
        self.user_execution = RootUserExecutionState::UserMappingWritableData(2);
        Ok(())
    }

    fn reserve_anonymous_user_mapping(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingWritableData(2)
            || self.physical_frame_count != Some(34)
        {
            return Err(CompileError::Tool(
                "x86 anonymous reservation requires both finalized image mappings and one source-owned frame"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // anonymous frame in rsi
        self.call_checked_bool(ProviderSymbol::ReserveAnonymousUserMapping)?;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
        self.physical_frame_count = None;
        self.user_execution = RootUserExecutionState::AnonymousUserMappingReserved;
        Ok(())
    }

    fn release_anonymous_user_mapping(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::CompletedWithResidentImage
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 anonymous release requires returned user execution and its live reservation"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call(ProviderSymbol::ReleaseAnonymousUserMapping);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]);
        self.physical_frame_count = Some(34);
        self.user_execution = RootUserExecutionState::CompletedWithResidentImageAnonymousReleased;
        Ok(())
    }

    fn unmap_user_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || !matches!(
                self.user_execution,
                RootUserExecutionState::CompletedWithExecutableImage
                    | RootUserExecutionState::CompletedWithResidentImageAnonymousReleased
            )
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user unmap requires returned execution with the exact finalized code or data mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call(ProviderSymbol::UnmapUserAddressSpace);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]);
        let generation = match self.user_execution {
            RootUserExecutionState::CompletedWithExecutableImage => 1,
            RootUserExecutionState::CompletedWithResidentImageAnonymousReleased => 2,
            _ => unreachable!("finalized mapping checked above"),
        };
        self.physical_frame_count = Some(if generation == 1 { 2 } else { 1 });
        self.user_execution = match generation {
            1 => RootUserExecutionState::CompletedImageRevoked,
            2 => RootUserExecutionState::CompletedWithExecutableImage,
            _ => unreachable!("only two mapping generations are constructed"),
        };
        Ok(())
    }

    fn enter_user_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserRandomServiceEntryReady
            || !self.user_random_service_bound
            || !self.user_random_small_requests_enabled
            || !self.user_random_large_requests_enabled
        {
            return Err(CompileError::Tool(
                "x86 user entry requires the randomized and reusable-getrandom-bound entry-ready context with both resident image mappings"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::EnterUserContext)?;
        self.user_execution = RootUserExecutionState::CompletedWithResidentImage;
        Ok(())
    }

    fn complete_user_execution(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::CompletedImageRevoked
        {
            return Err(CompileError::Tool(
                "x86 user completion requires terminal exit_group restoration and image teardown"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::CompleteUserBacking)?;
        self.user_execution = RootUserExecutionState::Revoked;
        Ok(())
    }

    fn reclaim_user_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::Revoked
        {
            return Err(CompileError::Tool(
                "x86 user-address-space reclaim requires the exact revoked space".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call(ProviderSymbol::ReclaimUserAddressSpace);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]);
        self.physical_frame_count = Some(3);
        self.user_execution = RootUserExecutionState::Consumed;
        Ok(())
    }

    fn load_bootstrap_storage_address(&mut self, storage_offset: u64) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]); // lea rdi, [rip+disp32]
        self.rip_relative_storage(storage_offset, 0)
    }

    fn bootstrap_storage_offset(&self, offset: u64) -> Result<u64, CompileError> {
        self.bootstrap_region_offset
            .ok_or_else(|| {
                CompileError::Tool(
                    "x86 root lowering encountered byte access without a live region".into(),
                )
            })?
            .checked_add(offset)
            .ok_or_else(|| CompileError::Tool("bootstrap storage offset overflows".into()))
    }

    fn rip_relative_storage(
        &mut self,
        storage_offset: u64,
        trailing_bytes: i64,
    ) -> Result<(), CompileError> {
        let storage_offset = i64::try_from(storage_offset).map_err(|_| {
            CompileError::Tool("bootstrap storage offset exceeds x86 relocation range".into())
        })?;
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target: ProviderSymbol::BootstrapStorage,
            addend: storage_offset - 4 - trailing_bytes,
            kind: RelocationKind::Relative,
            encoding: RelocationEncoding::X86RipRelative,
        });
        self.bytes.extend_from_slice(&[0; 4]);
        Ok(())
    }

    fn rip_relative_symbol(
        &mut self,
        target: ProviderSymbol,
        trailing_bytes: i64,
    ) -> Result<(), CompileError> {
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target,
            addend: -4 - trailing_bytes,
            kind: RelocationKind::Relative,
            encoding: RelocationEncoding::X86RipRelative,
        });
        self.bytes.extend_from_slice(&[0; 4]);
        Ok(())
    }
}
