impl RootEncoder {
    fn describe_boot_memory(&mut self) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xf5]); // retain boot params in r13
        self.call_checked_bool(ProviderSymbol::DescribeBootMemory)
    }

    fn create_frame_allocator(&mut self) -> Result<(), CompileError> {
        if self.frame_allocator_created {
            return Err(CompileError::Tool(
                "x86 root lowering encountered duplicate frame-allocator creation".into(),
            ));
        }
        if self.frame_policy.is_some() {
            self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // actual retained handoff
            self.frame_call(0, None, 0, None, false)?;
        }
        self.frame_allocator_created = true;
        Ok(())
    }

    fn reserve_static_image_hierarchy(&mut self) -> Result<(), CompileError> {
        // This provider supports only the qualified bootstrap-equivalent
        // topology. A future image-protected provider must derive a different
        // requirement from its actual image/owner binding, not widen this path.
        if self.static_image != RootStaticImageState::Sealed
            || !matches!(
                self.translation,
                RootTranslationState::BootstrapActive
                    | RootTranslationState::ReplacementActive
                    | RootTranslationState::RemovedActive
            )
        {
            return Err(CompileError::Tool(
                "provider-selected hierarchy backing requires a sealed construction and qualified translation context".into(),
            ));
        }
        self.allocate_physical_frames(5, 1)
    }

    fn allocate_physical_frames(
        &mut self,
        frame_count: u64,
        alignment_frames: u64,
    ) -> Result<(), CompileError> {
        let nested_static_elf_metadata = self.physical_frame_count == Some(503)
            && frame_count == 1
            && !self.retained_static_elf_payload;
        let nested_static_elf_hierarchy = self.static_image == RootStaticImageState::Sealed
            && self.physical_frame_count.is_none()
            && frame_count == 5
            && !self.retained_static_elf_payload;
        let nested_static_elf_stack_backing = self.static_image
            == RootStaticImageState::StackSlotAuthorized
            && self.physical_frame_count.is_none()
            && frame_count == 2
            && !self.retained_static_elf_payload;
        let retained_static_process_work = self.static_image
            == RootStaticImageState::PreparedProcessRetained
            && self.physical_frame_count.is_none()
            && frame_count == 1
            && !self.retained_static_elf_payload;
        let post_random_anonymous_extent = self.user_execution
            == RootUserExecutionState::UserMappingWritableData(2)
            && self.physical_frame_count.is_none()
            && frame_count == 34;
        if !self.frame_allocator_created
            || (self.physical_frame_count.is_some() && !nested_static_elf_metadata)
            || self.kernel_mapping_live
            || !matches!(frame_count, 1..=3 | 5 | 34 | 503)
            || alignment_frames != 1
            || (frame_count == 5 && !nested_static_elf_hierarchy)
            || (self.static_image == RootStaticImageState::Sealed && !nested_static_elf_hierarchy)
            || (self.static_image == RootStaticImageState::StackSlotAuthorized
                && !nested_static_elf_stack_backing)
            || (frame_count == 34 && !post_random_anonymous_extent)
            || (self.user_execution == RootUserExecutionState::AddressSpacePrepared
                && frame_count != 2)
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires one live allocator and an admitted static frame-extent request".into(),
            ));
        }
        let resident_code = self.user_execution == RootUserExecutionState::UserMappingExecutable(1);
        if retained_static_process_work {
            self.bytes.extend_from_slice(&[0x48, 0x83, 0xec, 0x10]);
            self.bytes.extend_from_slice(&[0x48, 0x89, 0x1c, 0x24]);
            self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf1]);
            self.bytes.extend_from_slice(&[0x4c, 0x8d, 0x83]);
            self.bytes.extend_from_slice(&0x1000_u32.to_le_bytes());
        } else if nested_static_elf_stack_backing {
            self.bytes.extend_from_slice(&[0x48, 0x89, 0xdd]); // retain hierarchy in rbp
            self.bytes
                .extend_from_slice(&[0x48, 0x8d, 0xbb, 0x00, 0x50, 0x00, 0x00]); // floor after hierarchy
            self.bytes.extend_from_slice(&[0x31, 0xc9]); // no separate exclusion
            self.bytes.extend_from_slice(&[0x45, 0x31, 0xc0]);
        } else if nested_static_elf_hierarchy {
            self.bytes.extend_from_slice(&[0x49, 0x89, 0xdc]); // retain leaf metadata in r12
            self.bytes
                .extend_from_slice(&[0x48, 0x8d, 0xbb, 0x00, 0x10, 0x00, 0x00]); // floor after leaf table
            self.bytes.extend_from_slice(&[0x31, 0xc9]); // no separate exclusion
            self.bytes.extend_from_slice(&[0x45, 0x31, 0xc0]);
        } else if nested_static_elf_metadata {
            self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]); // payload base in rcx
            self.bytes.extend_from_slice(&[0x4c, 0x8d, 0x83]); // payload end in r8
            self.bytes.extend_from_slice(&0x001f_7000_u32.to_le_bytes());
            self.bytes.extend_from_slice(&[0x49, 0x89, 0xde]); // retain payload in r14
        } else if matches!(
            self.user_execution,
            RootUserExecutionState::AddressSpacePrepared
                | RootUserExecutionState::UserMappingExecutable(1)
        ) {
            self.load_bootstrap_storage_address(176)?;
            self.bytes.extend_from_slice(&[0x48, 0x8b, 0x4f, 0x68]); // excluded backing base in rcx
            self.bytes.extend_from_slice(&[0x4c, 0x8d, 0x81]); // excluded backing end in r8
            self.bytes.extend_from_slice(&0x0000_3000_u32.to_le_bytes());
        } else {
            self.bytes.extend_from_slice(&[0x31, 0xc9]); // no excluded extent
            self.bytes.extend_from_slice(&[0x45, 0x31, 0xc0]); // no excluded end
        }
        if nested_static_elf_hierarchy || nested_static_elf_stack_backing {
            // The exact monotonic candidate and empty exclusion were prepared above.
        } else if resident_code {
            self.bytes.extend_from_slice(&[0x48, 0x8b, 0x7f, 0x78]); // code frame base
            self.bytes
                .extend_from_slice(&[0x48, 0x81, 0xc7, 0x00, 0x20, 0x00, 0x00]); // after code extent
        } else {
            match self.translation {
                RootTranslationState::BootstrapActive => {
                    self.bytes
                        .extend_from_slice(&[0xbf, 0x00, 0x00, 0x00, 0x01]);
                }
                RootTranslationState::ReplacementActive | RootTranslationState::RemovedActive => {
                    self.bytes
                        .extend_from_slice(&[0x49, 0x8d, 0xbf, 0x00, 0x50, 0x00, 0x00]);
                }
                _ => {
                    return Err(CompileError::Tool(
                    "x86 root lowering encountered frame allocation in an unavailable translation state".into(),
                ));
                }
            }
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // rsi = retained boot params
        let frame_count = u32::try_from(frame_count)
            .map_err(|_| CompileError::Tool("physical frame count exceeds u32".into()))?;
        self.bytes.push(0xba); // edx = frame count
        self.bytes.extend_from_slice(&frame_count.to_le_bytes());
        if self.frame_policy.is_some() {
            self.frame_allocate_source(u64::from(frame_count), alignment_frames)?;
        } else {
            self.call(ProviderSymbol::AllocatePhysicalFrames);
        }
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]); // test rax, rax
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]); // retain physical base in rbx
        if nested_static_elf_stack_backing {
            // The provider carries the upper-page anchor so all existing ABI
            // entry offsets remain relative to the page containing RSP. The
            // complete affine extent starts one base page below this anchor.
            self.bytes
                .extend_from_slice(&[0x48, 0x81, 0xc3, 0x00, 0x10, 0x00, 0x00]);
        }
        self.physical_frame_count = Some(u64::from(frame_count));
        if nested_static_elf_metadata {
            self.retained_static_elf_payload = true;
        }
        Ok(())
    }

    fn map_kernel_frames(
        &mut self,
        request: CompilerKernelMappingRequest,
    ) -> Result<(), CompileError> {
        if self.physical_frame_count != Some(1)
            || self.kernel_mapping_live
            || request != CompilerKernelMappingRequest::initial_read_write()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires one live frame extent and the sealed mapping policy"
                    .into(),
            ));
        }
        self.physical_frame_count = None;
        self.kernel_mapping_live = true;
        Ok(())
    }

    fn store_kernel_mapping_byte(
        &mut self,
        offset_bytes: u64,
        value: u8,
    ) -> Result<(), CompileError> {
        if !self.kernel_mapping_live || offset_bytes >= 4096 {
            return Err(CompileError::Tool(
                "x86 root lowering requires an in-bounds live kernel mapping store".into(),
            ));
        }
        let offset = u32::try_from(offset_bytes)
            .map_err(|_| CompileError::Tool("kernel mapping offset exceeds u32".into()))?;
        if self.translation == RootTranslationState::MappedActive {
            self.bytes.extend_from_slice(&[0x41, 0xc6, 0x86]); // [r14+disp32]
        } else {
            self.bytes.extend_from_slice(&[0xc6, 0x83]); // [rbx+disp32]
        }
        self.bytes.extend_from_slice(&offset.to_le_bytes());
        self.bytes.push(value);
        Ok(())
    }

    fn load_kernel_mapping_byte_equals(
        &mut self,
        offset_bytes: u64,
        expected: u8,
    ) -> Result<(), CompileError> {
        if !self.kernel_mapping_live || offset_bytes >= 4096 {
            return Err(CompileError::Tool(
                "x86 root lowering requires an in-bounds live kernel mapping load".into(),
            ));
        }
        let offset = u32::try_from(offset_bytes)
            .map_err(|_| CompileError::Tool("kernel mapping offset exceeds u32".into()))?;
        if self.translation == RootTranslationState::MappedActive {
            self.bytes.extend_from_slice(&[0x41, 0x0f, 0xb6, 0x86]); // [r14+disp32]
        } else {
            self.bytes.extend_from_slice(&[0x0f, 0xb6, 0x83]); // [rbx+disp32]
        }
        self.bytes.extend_from_slice(&offset.to_le_bytes());
        self.bytes.extend_from_slice(&[0x3c, expected]); // cmp al, imm8
        self.jump_to_fatal_if(0x85)
    }

    fn unmap_kernel_frames(&mut self) -> Result<(), CompileError> {
        if !self.kernel_mapping_live || self.physical_frame_count.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering encountered unmap without a live kernel mapping".into(),
            ));
        }
        self.kernel_mapping_live = false;
        self.physical_frame_count = Some(1);
        Ok(())
    }

    fn release_physical_frames(&mut self) -> Result<(), CompileError> {
        let Some(frame_count) = self.physical_frame_count else {
            return Err(CompileError::Tool(
                "x86 root lowering encountered release without a live physical-frame extent".into(),
            ));
        };
        if self.frame_policy.is_some() {
            self.frame_release_source()?;
        }
        if matches!(
            self.static_image,
            RootStaticImageState::Sealed | RootStaticImageState::HierarchyReturned
        ) && frame_count == 5
        {
            self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe3]); // restore leaf metadata
            self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]); // clear hierarchy identity
            self.bytes.extend_from_slice(&[0x31, 0xd2]);
            self.physical_frame_count = None;
            self.static_image = RootStaticImageState::Sealed;
            return Ok(());
        }
        if self.static_image == RootStaticImageState::StackBackingZeroed && frame_count == 2 {
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]); // clear backing identity
            self.bytes.extend_from_slice(&[0x48, 0x89, 0xeb]); // restore hierarchy identity
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xed]); // clear retained hierarchy copy
            self.bytes.extend_from_slice(&[0x31, 0xd2]);
            self.physical_frame_count = None;
            self.static_image = RootStaticImageState::StackSlotAuthorized;
            return Ok(());
        }
        if self.static_image == RootStaticImageState::PreparedProcessRetained && frame_count == 1 {
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
            self.bytes.extend_from_slice(&[0x48, 0x8b, 0x1c, 0x24]);
            self.bytes.extend_from_slice(&[0x48, 0x83, 0xc4, 0x10]);
            self.bytes.extend_from_slice(&[0x31, 0xd2]);
            self.physical_frame_count = None;
            return Ok(());
        }
        if self.retained_static_elf_payload {
            if frame_count != 1 {
                return Err(CompileError::Tool(
                    "x86 root lowering requires metadata release before its retained static ELF payload".into(),
                ));
            }
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]); // clear metadata base
            self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf3]); // restore payload base
            self.bytes.extend_from_slice(&[0x4d, 0x31, 0xf6]); // clear retained payload register
            self.bytes.extend_from_slice(&[0x31, 0xd2]); // clear transient frame count
            self.physical_frame_count = Some(503);
            self.retained_static_elf_payload = false;
            return Ok(());
        }
        self.physical_frame_count = None;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]); // clear retained physical base
        self.bytes.extend_from_slice(&[0x31, 0xd2]); // clear transient frame count
        Ok(())
    }

    fn prepare_static_image_construction(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.translation != RootTranslationState::RemovedActive
            || self.static_image != RootStaticImageState::Fresh
            || self.physical_frame_count != Some(1)
            || !self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image preparation requires the exact live payload and metadata extents"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.call_checked_bool(ProviderSymbol::PrepareStaticImage)?;
        self.physical_frame_count = None;
        self.retained_static_elf_payload = false;
        self.static_image = RootStaticImageState::Live;
        Ok(())
    }

    fn revoke_static_image_construction(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Sealed
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image revocation requires one live inactive construction".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.call_checked_bool(ProviderSymbol::RevokeStaticImage)?;
        self.physical_frame_count = Some(1);
        self.retained_static_elf_payload = true;
        self.static_image = RootStaticImageState::MetadataReturned;
        Ok(())
    }

    fn zero_static_image_payload(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Live
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image payload zeroing requires one live uninitialized construction"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.call_checked_bool(ProviderSymbol::ZeroStaticImagePayload)?;
        self.static_image = RootStaticImageState::Zeroed;
        Ok(())
    }

    fn copy_static_image_payload(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Zeroed
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image payload copy requires one completely zeroed construction".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe9]); // boot params
        self.call_checked_bool(ProviderSymbol::CopyStaticImagePayload)?;
        self.static_image = RootStaticImageState::Populated;
        Ok(())
    }

    fn seal_static_image_permissions(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Populated
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image permission sealing requires one populated construction".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe9]); // boot params
        self.call_checked_bool(ProviderSymbol::SealStaticImagePermissions)?;
        self.static_image = RootStaticImageState::Sealed;
        Ok(())
    }

    fn attach_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Sealed
            || self.physical_frame_count != Some(5)
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space attachment requires one sealed construction and its live hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]); // leaf metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]); // hierarchy base
        self.call_checked_bool(ProviderSymbol::AttachStaticImageAddressSpace)?;
        self.physical_frame_count = None;
        self.static_image = RootStaticImageState::Attached;
        Ok(())
    }

    fn detach_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntryRestored
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space detachment requires one attached inactive construction"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::DetachStaticImageAddressSpace)?;
        self.static_image = RootStaticImageState::Detached;
        Ok(())
    }

    fn activate_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Attached
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space activation requires one fresh attached construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]); // static root
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]); // bootstrap root
        self.call_checked_bool(ProviderSymbol::ActivateStaticImageAddressSpace)?;
        self.static_image = RootStaticImageState::Active;
        Ok(())
    }

    fn restore_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Active
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space restoration requires one active construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::RestoreStaticImageAddressSpace)?;
        self.static_image = RootStaticImageState::Restored;
        Ok(())
    }

    fn authorize_static_image_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Restored
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entry authorization requires one restored attached construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::AuthorizeStaticImageEntry)?;
        self.static_image = RootStaticImageState::EntryAuthorized;
        Ok(())
    }

    fn authorize_static_image_stack_slot(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntryAuthorized
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-slot authorization requires one entry-authorized construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::AuthorizeStaticImageStackSlot)?;
        self.static_image = RootStaticImageState::StackSlotAuthorized;
        Ok(())
    }

    fn revoke_static_image_stack_slot(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackSlotAuthorized
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-slot revocation requires one slot-authorized construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageStackSlot)?;
        self.static_image = RootStaticImageState::EntryAuthorized;
        Ok(())
    }

    fn zero_static_image_stack_backing(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackSlotAuthorized
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-backing zeroing requires one slot-authorized construction and its live two-frame backing"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]); // opaque backing base
        self.call_checked_bool(ProviderSymbol::ZeroStaticImageStackBacking)?;
        self.static_image = RootStaticImageState::StackBackingZeroed;
        Ok(())
    }

    fn publish_static_image_stack_leaf(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackBackingZeroed
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-leaf publication requires one zeroed backing under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]); // leaf metadata
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]); // hierarchy
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]); // bootstrap root
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]); // backing
        self.call_checked_bool(ProviderSymbol::PublishStaticImageStackLeaf)?;
        self.static_image = RootStaticImageState::StackLeafPublished;
        Ok(())
    }

    fn initialize_static_image_stack_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackLeafPublished
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-entry initialization requires one published stack leaf under the inactive qualified hierarchy"
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
        self.call_checked_bool(ProviderSymbol::InitializeStaticImageStackEntry)?;
        self.static_image = RootStaticImageState::StackEntryInitialized;
        Ok(())
    }

    fn clear_static_image_stack_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackEntryInitialized
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-entry clearing requires one initialized stack entry under the inactive qualified hierarchy"
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
        self.call_checked_bool(ProviderSymbol::ClearStaticImageStackEntry)?;
        self.static_image = RootStaticImageState::StackLeafPublished;
        Ok(())
    }

    fn add_static_image_stack_pagesz(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackEntryInitialized
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_PAGESZ addition requires one initialized stack entry under the inactive qualified hierarchy"
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
        self.call_checked_bool(ProviderSymbol::AddStaticImageStackPageSize)?;
        self.static_image = RootStaticImageState::StackPageSizeAdded;
        Ok(())
    }

    fn add_static_image_stack_entry_fact(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackPageSizeAdded
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_ENTRY addition requires one page-size-qualified stack entry under the inactive qualified hierarchy"
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
        self.call_checked_bool(ProviderSymbol::AddStaticImageStackEntryFact)?;
        self.static_image = RootStaticImageState::StackEntryFactAdded;
        Ok(())
    }

    fn transition_static_image_stack_program_headers(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackEntryFactAdded
        } else {
            RootStaticImageState::StackProgramHeadersAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image program-header transition requires its exact qualified stack owner under the inactive hierarchy"
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
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackProgramHeaders
        } else {
            ProviderSymbol::RemoveStaticImageStackProgramHeaders
        })?;
        self.static_image = if add {
            RootStaticImageState::StackProgramHeadersAdded
        } else {
            RootStaticImageState::StackEntryFactAdded
        };
        Ok(())
    }

    fn add_static_image_stack_program_headers(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_program_headers(true)
    }

    fn transition_static_image_stack_executable_name(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackProgramHeadersAdded
        } else {
            RootStaticImageState::StackExecutableNameAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image executable-name transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackExecutableName
        } else {
            ProviderSymbol::RemoveStaticImageStackExecutableName
        })?;
        self.static_image = if add {
            RootStaticImageState::StackExecutableNameAdded
        } else {
            RootStaticImageState::StackProgramHeadersAdded
        };
        Ok(())
    }

    fn add_static_image_stack_executable_name(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_executable_name(true)
    }
    fn remove_static_image_stack_executable_name(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_executable_name(false)
    }
}
