impl BootstrapAuthorityState {
    fn observe_user_execution(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::PrepareUserAddressSpace => {
                if self.kernel_context != BootstrapKernelContextState::QueueConsumed
                    || self.kernel_completion != BootstrapKernelCompletionState::Consumed
                    || self.user_execution != BootstrapUserExecutionState::Fresh
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::Frames(INITIAL_MULTI_FRAME_REQUEST)
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user-address-space preparation requires completed kernel-context cleanup, its exact three-frame backing, and no live critical scope",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
                self.user_execution =
                    BootstrapUserExecutionState::AddressSpacePrepared(INITIAL_MULTI_FRAME_REQUEST);
            }
            SystemsOperation::PrepareUserProbe { process_identity } => {
                let BootstrapUserExecutionState::MappingWritableData(backing, 2) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user probe preparation requires both finalized image mappings resident",
                    ));
                };
                if *process_identity != 1
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.initial_boot_resource_validated
                    || !self.anonymous_user_mapping_reserved
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user probe preparation requires PID one, the validated boot resource, the ready resident image, no source-owned extent, and no live critical scope",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::PreparedWithResidentImage(backing);
            }
            SystemsOperation::ReserveAnonymousUserMapping => {
                let BootstrapUserExecutionState::MappingWritableData(backing, 2) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous reservation requires both finalized image mappings",
                    ));
                };
                if self.memory_ownership
                    != BootstrapMemoryOwnership::Frames(POST_RANDOM_ANONYMOUS_EXTENT_REQUEST)
                    || self.anonymous_user_mapping_reserved
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous reservation requires the exact source-owned growth extent and no prior reservation",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
                self.anonymous_user_mapping_reserved = true;
                self.user_execution = BootstrapUserExecutionState::MappingWritableData(backing, 2);
            }
            SystemsOperation::VerifyUserAddressSpaceMapping => {
                let BootstrapUserExecutionState::AddressSpacePrepared(backing) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping verification requires the exact unverified prepared address space",
                    ));
                };
                if self.memory_ownership
                    != BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST)
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping verification requires one disjoint source-owned frame and no live critical scope",
                    ));
                }
                self.user_execution = BootstrapUserExecutionState::AddressSpaceVerified(backing);
            }
            SystemsOperation::MapUserAddressSpace => {
                let (backing, generation) = match self.user_execution {
                    BootstrapUserExecutionState::AddressSpacePrepared(backing) => (backing, 1),
                    BootstrapUserExecutionState::MappingExecutable(backing, 1) => (backing, 2),
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user mapping publication requires the prepared address space at its next exact generation",
                        ));
                    }
                };
                let mapping_request = if generation == 1 {
                    INITIAL_TWO_FRAME_REQUEST
                } else {
                    INITIAL_SINGLE_FRAME_REQUEST
                };
                if self.memory_ownership != BootstrapMemoryOwnership::Frames(mapping_request)
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping publication requires one disjoint source-owned frame and no live critical scope",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::UserMapping(mapping_request);
                self.user_execution = BootstrapUserExecutionState::MappingLive(backing, generation);
            }
            SystemsOperation::StoreUserMappingByte => {
                let BootstrapUserExecutionState::MappingLive(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping store requires the exact live mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping store requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingStored(backing, generation);
            }
            SystemsOperation::InstallUserMappingEntryTrampoline => {
                let BootstrapUserExecutionState::MappingLive(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline installation requires the exact live mapping",
                    ));
                };
                if generation != 1
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_TWO_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline installation requires the first owned disjoint mapping",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingEntryInstalled(backing, generation);
            }
            SystemsOperation::ProtectUserMappingReadOnly => {
                let (backing, generation) = match self.user_execution {
                    BootstrapUserExecutionState::MappingStored(backing, generation)
                    | BootstrapUserExecutionState::MappingEntryInstalled(backing, generation) => {
                        (backing, generation)
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user mapping protection requires the exact populated writable mapping",
                        ));
                    }
                };
                if self.memory_ownership
                    != BootstrapMemoryOwnership::UserMapping(if generation == 1 {
                        INITIAL_TWO_FRAME_REQUEST
                    } else {
                        INITIAL_SINGLE_FRAME_REQUEST
                    })
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping protection requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingProtected(backing, generation);
            }
            SystemsOperation::LoadUserMappingByte => {
                let BootstrapUserExecutionState::MappingProtected(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping load requires the exact protected mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping load requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingVerified(backing, generation);
            }
            SystemsOperation::VerifyUserMappingEntryTrampoline => {
                let BootstrapUserExecutionState::MappingProtected(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline verification requires the exact protected mapping",
                    ));
                };
                if generation != 1
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_TWO_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline verification requires the first owned disjoint mapping",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingVerified(backing, generation);
            }
            SystemsOperation::PermitUserMappingExecute => {
                let BootstrapUserExecutionState::MappingVerified(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping execute promotion requires the exact verified mapping",
                    ));
                };
                if generation != 1
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_TWO_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping execute promotion requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingExecutable(backing, generation);
                self.memory_ownership = BootstrapMemoryOwnership::None;
            }
            SystemsOperation::SealUserMappingImmutableData => {
                let BootstrapUserExecutionState::MappingVerified(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping data sealing requires the exact verified mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping data sealing requires the second generation and its owned frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingImmutableData(backing, generation);
                self.memory_ownership = BootstrapMemoryOwnership::None;
            }
            SystemsOperation::PermitUserMappingWritableData => {
                let BootstrapUserExecutionState::MappingVerified(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping writable-data promotion requires the exact verified mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping writable-data promotion requires the second generation and its owned frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingWritableData(backing, generation);
                self.memory_ownership = BootstrapMemoryOwnership::None;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_IMAGE_READY" => {
                if self.user_execution
                    != BootstrapUserExecutionState::MappingWritableData(
                        INITIAL_MULTI_FRAME_REQUEST,
                        2,
                    )
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.resident_user_image_ready
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user image readiness requires both finalized role mappings to remain resident exactly once",
                    ));
                }
                self.resident_user_image_ready = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_BOOT_RESOURCE_OK" => {
                if self.user_execution
                    != BootstrapUserExecutionState::MappingWritableData(
                        INITIAL_MULTI_FRAME_REQUEST,
                        2,
                    )
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || self.initial_boot_resource_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "boot-resource witness requires one validated opaque resource with both finalized role mappings resident",
                    ));
                }
                self.initial_boot_resource_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.initial_boot_resource_validated
                    || self.multipage_user_executable_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "multipage-executable witness requires exact second-page user return with both finalized mappings resident",
                    ));
                }
                self.multipage_user_executable_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_READ_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.initial_boot_resource_validated
                    || !self.multipage_user_executable_completed
                    || self.constructed_user_read_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed read witness requires exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_read_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_CLOSE_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_read_completed
                    || self.constructed_user_close_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-close witness requires the completed constructed read and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_close_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_DUP_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_close_completed
                    || self.constructed_user_dup_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-duplication witness requires the close witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_dup_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_FCNTL_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_dup_completed
                    || self.constructed_user_fcntl_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-metadata witness requires the duplication witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_fcntl_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_FSTAT_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_fcntl_completed
                    || self.constructed_user_fstat_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-stat witness requires the fcntl witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_fstat_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_TCGETS_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_fstat_completed
                    || self.constructed_user_tcgets_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "terminal-attributes witness requires the fstat witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_tcgets_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_WINSIZE_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_tcgets_completed
                    || self.constructed_user_winsize_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "console-geometry witness requires the TCGETS witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_winsize_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_LSEEK_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_winsize_completed
                    || self.constructed_user_lseek_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "console-seek witness requires the window-size witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_lseek_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_read_completed
                    || !self.constructed_user_close_completed
                    || !self.constructed_user_dup_completed
                    || !self.constructed_user_fcntl_completed
                    || !self.constructed_user_fstat_completed
                    || !self.constructed_user_tcgets_completed
                    || !self.constructed_user_winsize_completed
                    || !self.constructed_user_lseek_completed
                    || self.constructed_user_image_entered
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image entry witness requires exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_entered = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_STACK_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_entered
                    || self.constructed_user_initial_stack_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image stack witness requires exact returned entry with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_stack_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_stack_validated
                    || self.constructed_user_initial_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image auxiliary-vector witness requires exact returned initial-stack validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_auxv_validated
                    || self.constructed_user_initial_entry_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image entry auxiliary-vector witness requires exact returned page-size validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_entry_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_entry_auxv_validated
                    || self.constructed_user_initial_execfn_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image executable-name auxiliary-vector witness requires exact returned entry-fact validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_execfn_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_execfn_auxv_validated
                    || self.constructed_user_initial_credential_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image credential auxiliary-vector witness requires exact returned executable-name validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_credential_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_IMAGE_ELF_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_credential_auxv_validated
                    || self.constructed_user_elf_identity_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image ELF witness requires exact returned credential validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_elf_identity_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_elf_identity_validated
                    || self.external_user_elf_loaded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "external ELF-load witness requires the validated resident ELF identity exactly once",
                    ));
                }
                self.external_user_elf_loaded = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.external_user_elf_loaded
                    || self.external_user_elf_copy_verified
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "external ELF-copy witness requires the loaded resource payload exactly once",
                    ));
                }
                self.external_user_elf_copy_verified = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.external_user_elf_copy_verified
                    || self.runtime_user_code_extent_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "runtime executable-extent witness requires the verified external copy exactly once",
                    ));
                }
                self.runtime_user_code_extent_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.runtime_user_code_extent_validated
                    || self.runtime_user_entry_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "runtime ELF-entry witness requires the validated executable extent exactly once",
                    ));
                }
                self.runtime_user_entry_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.runtime_user_entry_validated
                    || self.runtime_user_data_extent_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "runtime data/BSS-extent witness requires the validated ELF entry exactly once",
                    ));
                }
                self.runtime_user_data_extent_validated = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_IMAGE_BSS_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.external_user_elf_copy_verified
                    || !self.runtime_user_code_extent_validated
                    || !self.runtime_user_data_extent_validated
                    || self.external_user_bss_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "external ELF data/BSS witness requires the verified resource copy exactly once",
                    ));
                }
                self.external_user_bss_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_DATA_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_elf_identity_validated
                    || !self.external_user_elf_loaded
                    || !self.external_user_elf_copy_verified
                    || !self.runtime_user_code_extent_validated
                    || !self.runtime_user_data_extent_validated
                    || !self.external_user_bss_validated
                    || self.constructed_user_image_data_observed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image data witness requires exact returned entry with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_data_observed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_data_observed
                    || self.constructed_user_image_copy_to_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image copy-to-user witness requires exact returned data observation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_copy_to_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_copy_to_completed
                    || self.constructed_user_image_copy_from_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image copy-from-user witness requires exact returned output completion with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_copy_from_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_copy_from_completed
                    || self.constructed_user_image_syscall_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image syscall witness requires exact returned data observation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_syscall_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RETAINED" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_image_entered
                    || !self.constructed_user_initial_stack_validated
                    || !self.constructed_user_initial_auxv_validated
                    || !self.constructed_user_initial_entry_auxv_validated
                    || !self.constructed_user_initial_execfn_auxv_validated
                    || !self.constructed_user_initial_credential_auxv_validated
                    || !self.constructed_user_elf_identity_validated
                    || !self.external_user_elf_loaded
                    || !self.external_user_elf_copy_verified
                    || !self.runtime_user_code_extent_validated
                    || !self.runtime_user_data_extent_validated
                    || !self.external_user_bss_validated
                    || !self.constructed_user_image_data_observed
                    || !self.constructed_user_image_copy_to_completed
                    || !self.constructed_user_image_copy_from_completed
                    || !self.constructed_user_image_syscall_completed
                    || self.resident_user_image_retained
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "retained-image witness requires exact user return with both finalized mappings resident",
                    ));
                }
                self.resident_user_image_retained = true;
            }
            SystemsOperation::UnmapUserAddressSpace => {
                let (backing, generation) = match self.user_execution {
                    BootstrapUserExecutionState::CompletedWithExecutableImage(backing) => {
                        (backing, 1)
                    }
                    BootstrapUserExecutionState::CompletedWithResidentImage(backing) => {
                        (backing, 2)
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user unmap requires exact returned execution and its generation-specific image role",
                        ));
                    }
                };
                if self.memory_ownership != BootstrapMemoryOwnership::None {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user unmap requires its owned disjoint frame",
                    ));
                }
                if generation == 2 && !self.resident_user_image_retained {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "immutable data cannot be unmapped before the post-return retained-image witness",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Frames(if generation == 1 {
                    INITIAL_TWO_FRAME_REQUEST
                } else {
                    INITIAL_SINGLE_FRAME_REQUEST
                });
                self.user_execution = match generation {
                    1 => BootstrapUserExecutionState::CompletedImageRevoked(backing),
                    2 => BootstrapUserExecutionState::CompletedWithExecutableImage(backing),
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user unmap requires one of the two admitted mapping generations",
                        ));
                    }
                };
            }
            SystemsOperation::ReleaseAnonymousUserMapping => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.anonymous_user_mapping_reserved
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous release requires returned user execution and its live reservation",
                    ));
                }
                self.anonymous_user_mapping_reserved = false;
                self.memory_ownership =
                    BootstrapMemoryOwnership::Frames(POST_RANDOM_ANONYMOUS_EXTENT_REQUEST);
            }
            SystemsOperation::EnterUserContext => {
                let BootstrapUserExecutionState::PreparedWithResidentImage(backing) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user entry requires the exact prepared probe with both image mappings resident",
                    ));
                };
                if !self.critical_stack.is_empty()
                    || !self.initial_auxv_random_delivered
                    || !self.user_random_service_bound
                    || !self.user_random_small_requests_enabled
                    || !self.user_random_large_requests_enabled
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user entry requires the exact prepared probe, delivered initial AT_RANDOM object, reusable getrandom service, and restored kernel context",
                    ));
                }
                self.user_random_service_bound = false;
                self.user_random_small_requests_enabled = false;
                self.user_random_large_requests_enabled = false;
                self.user_random_service_completed = true;
                self.initial_user_random_delivered = true;
                self.user_execution =
                    BootstrapUserExecutionState::CompletedWithResidentImage(backing);
            }
            SystemsOperation::CompleteUserExecution => {
                let BootstrapUserExecutionState::CompletedImageRevoked(backing) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user completion requires terminal exit_group restoration and complete image teardown",
                    ));
                };
                self.user_execution = BootstrapUserExecutionState::Revoked(backing);
            }
            SystemsOperation::ReclaimUserAddressSpace => {
                let BootstrapUserExecutionState::Revoked(backing) = self.user_execution else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user-address-space reclaim requires the exact revoked address space",
                    ));
                };
                self.user_execution = BootstrapUserExecutionState::Consumed;
                self.memory_ownership = BootstrapMemoryOwnership::Frames(backing);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_translation(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<(), SystemsModelError> {
        match operation {
            SystemsOperation::BeginTranslationUpdate { request, .. } => {
                if !self.allocator_created
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.translation != BootstrapTranslationState::BootstrapActive
                    || *request != TranslationUpdateRequest::initial_bootstrap_equivalent()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION",
                        "translation begin requires the live allocator, no borrowed extent, the bootstrap translation, and the sealed request",
                    ));
                }
                self.translation = BootstrapTranslationState::Update;
            }
            SystemsOperation::CommitTranslationUpdate { .. } => {
                if self.translation != BootstrapTranslationState::Update {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION",
                        "translation commit requires one live exclusive update",
                    ));
                }
                self.translation = BootstrapTranslationState::InactiveSpace;
            }
            SystemsOperation::ActivateTranslationSpace { .. } => {
                if self.translation != BootstrapTranslationState::InactiveSpace {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION",
                        "translation activation requires one committed inactive space",
                    ));
                }
                self.translation = BootstrapTranslationState::ReplacementActive;
            }
            operation @ (SystemsOperation::BeginTranslationEdit { .. }
            | SystemsOperation::MapTranslationFrames { .. }
            | SystemsOperation::UnmapTranslationMapping
            | SystemsOperation::CommitTranslationEdit { .. }) => {
                self.observe_translation_edit(operation)?;
            }
            _ => unreachable!("caller selects translation operations"),
        }
        Ok(())
    }

    fn observe_translation_edit(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<(), SystemsModelError> {
        match operation {
            SystemsOperation::BeginTranslationEdit { kind, .. } => match kind {
                TranslationEditKind::Map
                    if self.translation == BootstrapTranslationState::ReplacementActive
                        && self.memory_ownership
                            == BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST) =>
                {
                    self.translation = BootstrapTranslationState::MapEdit;
                }
                TranslationEditKind::Unmap
                    if self.translation == BootstrapTranslationState::MappedActive
                        && matches!(
                            self.memory_ownership,
                            BootstrapMemoryOwnership::Mapping(_)
                        ) =>
                {
                    self.translation = BootstrapTranslationState::UnmapEdit;
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit begin requires the matching refined context and owned resource",
                    ));
                }
            },
            SystemsOperation::MapTranslationFrames { request, .. } => {
                let BootstrapMemoryOwnership::Frames(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit map requires the sealed policy, map edit, and live extent",
                    ));
                };
                if self.translation != BootstrapTranslationState::MapEdit
                    || frame_request != INITIAL_SINGLE_FRAME_REQUEST
                    || *request != TranslationMappingRequest::initial_read_write()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit map requires the sealed policy, map edit, and live extent",
                    ));
                }
                self.translation = BootstrapTranslationState::MapStaged;
                self.memory_ownership = BootstrapMemoryOwnership::ProvisionalMapping(frame_request);
            }
            SystemsOperation::UnmapTranslationMapping => {
                let BootstrapMemoryOwnership::Mapping(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit unmap requires the matching edit and live mapping",
                    ));
                };
                if self.translation != BootstrapTranslationState::UnmapEdit {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit unmap requires the matching edit and live mapping",
                    ));
                }
                self.translation = BootstrapTranslationState::UnmapStaged;
                self.memory_ownership = BootstrapMemoryOwnership::ProvisionalFrames(frame_request);
            }
            SystemsOperation::CommitTranslationEdit { kind, .. } => match kind {
                TranslationEditKind::Map
                    if self.translation == BootstrapTranslationState::MapStaged =>
                {
                    let BootstrapMemoryOwnership::ProvisionalMapping(frame_request) =
                        self.memory_ownership
                    else {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-TRANSLATION-EDIT",
                            "translation edit commit requires one complete matching candidate",
                        ));
                    };
                    self.translation = BootstrapTranslationState::MappedActive;
                    self.memory_ownership = BootstrapMemoryOwnership::Mapping(frame_request);
                }
                TranslationEditKind::Unmap
                    if self.translation == BootstrapTranslationState::UnmapStaged =>
                {
                    let BootstrapMemoryOwnership::ProvisionalFrames(frame_request) =
                        self.memory_ownership
                    else {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-TRANSLATION-EDIT",
                            "translation edit commit requires one complete matching candidate",
                        ));
                    };
                    self.translation = BootstrapTranslationState::RemovedActive;
                    self.memory_ownership = BootstrapMemoryOwnership::Frames(frame_request);
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit commit requires one complete matching candidate",
                    ));
                }
            },
            _ => unreachable!("caller selects translation-edit operations"),
        }
        Ok(())
    }

    fn complete(self) -> Result<(), SystemsModelError> {
        if self.image_backing_live || self.image_tables_retained || (self.image_backing_discarded && !self.image_backing_discard_observed) {
            return Err(SystemsModelError::new("E-SYSTEMS-FRAMES-LIVE", "image-backing preparation remains live at disposition"));
        }
        if !self.critical_stack.is_empty() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CRITICAL-LIVE",
                "bootstrap handler consumes its context while critical restoration authority remains live",
            ));
        }
        if !self.memory_described {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-BOOT-MEMORY",
                "the bootstrap handler must consume its entered context through boot-memory description",
            ));
        }
        if !self.allocator_created {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-FRAMES",
                "the bootstrap handler must consume memory description through frame-allocator creation",
            ));
        }
        match self.memory_ownership {
            BootstrapMemoryOwnership::None
                if !self.retained_static_process
                    && !self.entropy_pending_static_process
                    && !self.entropy_acquiring_static_process => {}
            BootstrapMemoryOwnership::None => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-FRAMES-LIVE",
                    "bootstrap handler consumes its context while a retained static process remains live",
                ));
            }
            BootstrapMemoryOwnership::Frames(_)
            | BootstrapMemoryOwnership::StaticElfMappingResources
            | BootstrapMemoryOwnership::StaticElfConstructionLive
            | BootstrapMemoryOwnership::StaticElfConstructionZeroed
            | BootstrapMemoryOwnership::StaticElfConstructionPopulated
            | BootstrapMemoryOwnership::StaticElfConstructionSealed
            | BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy
            | BootstrapMemoryOwnership::StaticElfConstructionAttached
            | BootstrapMemoryOwnership::StaticElfConstructionActive
            | BootstrapMemoryOwnership::StaticElfConstructionRestored
            | BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized
            | BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized
            | BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorizedWithBacking
            | BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed
            | BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished
            | BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized
            | BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized
            | BootstrapMemoryOwnership::StaticElfConstructionEntryRestored
            | BootstrapMemoryOwnership::StaticElfConstructionDetached
            | BootstrapMemoryOwnership::StaticElfConstructionHierarchyReturned
            | BootstrapMemoryOwnership::StaticElfConstructionMetadataReturned
            | BootstrapMemoryOwnership::StaticElfConstructionRevoked => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-FRAMES-LIVE",
                    "bootstrap handler consumes its context while a physical-frame extent remains live",
                ));
            }
            BootstrapMemoryOwnership::Mapping(_) => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-MAPPING-LIVE",
                    "bootstrap handler consumes its context while a kernel mapping remains live",
                ));
            }
            BootstrapMemoryOwnership::UserMapping(_) => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "bootstrap handler consumes its context while a user mapping remains live",
                ));
            }
            BootstrapMemoryOwnership::ProvisionalMapping(_)
            | BootstrapMemoryOwnership::ProvisionalFrames(_) => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-TRANSLATION-EDIT-LIVE",
                    "bootstrap handler consumes its context while a translation edit candidate remains live",
                ));
            }
        }
        if matches!(
            self.translation,
            BootstrapTranslationState::Update
                | BootstrapTranslationState::InactiveSpace
                | BootstrapTranslationState::MapEdit
                | BootstrapTranslationState::MapStaged
                | BootstrapTranslationState::UnmapEdit
                | BootstrapTranslationState::UnmapStaged
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-TRANSLATION-LIVE",
                "bootstrap handler consumes its context while a translation update, inactive space, or edit remains live",
            ));
        }
        if self.local_notification != BootstrapLocalNotificationState::Completed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LOCAL-INTERRUPT-LIVE",
                "bootstrap completion requires one completed local-notification send/wait lifecycle",
            ));
        }
        if self.monotonic_clock_observations != 2 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-MONOTONIC-CLOCK-LIVE",
                "bootstrap completion requires exactly two monotonic-clock observations",
            ));
        }
        if self.pci_configuration_observations
            != PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-PCI-CONFIG-LIVE",
                "bootstrap completion requires the complete sealed virtio-rng PCI capability-discovery sequence",
            ));
        }
        if !self.pci_bus_master_authorized {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-PCI-CONFIG-UPDATE-LIVE",
                "bootstrap completion requires the sealed PCI bus-master authorization",
            ));
        }
        if self.dma_buffers_created != DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-BUFFER-LIVE",
                "bootstrap completion requires both sealed CPU-owned virtio-rng DMA buffers",
            ));
        }
        if self.dma_construction_operations
            != 1 + DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-CONSTRUCT-LIVE",
                "bootstrap completion requires the exact CPU-owned virtio-rng split-ring construction",
            ));
        }
        if self.dma_buffers_prepared != DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-PREPARE-LIVE",
                "bootstrap completion requires the exact payload-then-control DMA device-preparation sequence",
            ));
        }
        if self.dma_references_published
            != PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-PUBLISH-LIVE",
                "bootstrap completion requires all three sealed virtio-rng queue-address publications",
            ));
        }
        if self.pci_region_register_accesses
            != PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-PCI-REGION-LIVE",
                "bootstrap completion requires the complete sealed virtio-rng feature-negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences",
            ));
        }
        if !self.dma_control_completed
            || self.dma_used_ring_reads
                != DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-USED-RING-LIVE",
                "bootstrap completion requires CPU return and exact validation of the sealed virtio-rng used ring",
            ));
        }
        if !self.dma_payload_completed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-PAYLOAD-LIVE",
                "bootstrap completion requires CPU return of the completed virtio-rng payload",
            ));
        }
        if !self.entropy_sample_captured {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-SAMPLE-LIVE",
                "bootstrap completion requires capture of the sealed uncredited virtio-rng sample",
            ));
        }
        if !self.entropy_sample_assessed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-QUALITY-LIVE",
                "bootstrap completion requires assessment of the captured virtio-rng sample under explicit Q35 trust evidence",
            ));
        }
        if !self.entropy_pool_created {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-POOL-LIVE",
                "bootstrap completion requires creation of the sealed empty entropy pool",
            ));
        }
        if !self.entropy_pool_mixed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-MIX-LIVE",
                "bootstrap completion requires the sealed first entropy-pool mix",
            ));
        }
        if !self.entropy_pool_credited {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-CREDIT-LIVE",
                "bootstrap completion requires transfer of the sealed entropy contribution",
            ));
        }
        if !self.entropy_generator_ready {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-GENERATOR-LIVE",
                "bootstrap completion requires the sealed ready entropy generator",
            ));
        }
        if !self.entropy_generator_extracted {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-EXTRACT-LIVE",
                "bootstrap completion requires the sealed first entropy-generator extraction",
            ));
        }
        if self.deadline != BootstrapDeadlineState::Completed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DEADLINE-EVENT-LIVE",
                "bootstrap completion requires one completed deadline arm/wait lifecycle",
            ));
        }
        if self.kernel_context != BootstrapKernelContextState::QueueConsumed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CONTEXT-TRANSFER-LIVE",
                "bootstrap completion requires all worker contexts to be reclaimed and the runnable queue consumed empty",
            ));
        }
        if self.kernel_completion != BootstrapKernelCompletionState::Consumed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CONTEXT-TRANSFER-LIVE",
                "bootstrap completion requires the one-shot completion state to be consumed",
            ));
        }
        if self.user_execution != BootstrapUserExecutionState::Consumed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-USER-CONTEXT-LIVE",
                "bootstrap completion requires one consumed getpid/exit_group user execution",
            ));
        }
        if !self.initial_auxv_random_delivered {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-DELIVER-LIVE",
                "bootstrap completion requires affine delivery of the first entropy output to initial AT_RANDOM",
            ));
        }
        if !self.initial_getrandom_extracted
            || self.initial_user_random_reserved
            || self.user_random_service_bound
            || self.user_random_small_requests_enabled
            || self.user_random_large_requests_enabled
            || !self.user_random_service_completed
            || !self.initial_user_random_delivered
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-RESERVE-LIVE",
                "bootstrap completion requires affine reservation, reusable-service binding, repeated delivery, and teardown of the initial getrandom service",
            ));
        }
        Ok(())
    }
}

fn validate_atomic_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::AtomicWordCreate { request } => {
            if atomic.is_some() || *request != AtomicWordRequest::initial() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC",
                    "the initial slice admits one live atomic word with the sealed request",
                ));
            }
            let owned_region = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic word creation requires one exclusively owned ordinary region",
                )
            })?;
            *atomic = Some(AtomicWordLocation::create(storage, owned_region, *request)?);
        }
        SystemsOperation::AtomicCompareExchangeEquals {
            expected,
            desired,
            success_order,
            failure_order,
            ..
        } => {
            if (*expected, *desired, *success_order, *failure_order)
                != (41, 42, AtomicOrder::AcquireRelease, AtomicOrder::Acquire)
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC",
                    "the initial compare/exchange requires expected 41, desired 42, acquire-release success, and acquire failure",
                ));
            }
            atomic
                .as_mut()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "compare/exchange requires one live atomic word",
                    )
                })?
                .compare_exchange(storage, *expected, *desired, *success_order, *failure_order)?;
        }
        SystemsOperation::AtomicLoadEquals { order, .. } => {
            if *order != AtomicOrder::Acquire {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-ORDER",
                    "the initial atomic load requires acquire order",
                ));
            }
            atomic
                .as_ref()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "atomic load requires one live atomic word",
                    )
                })?
                .load(storage, *order)?;
        }
        SystemsOperation::AtomicWordEnd => {
            let location = atomic.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic end requires one live atomic word",
                )
            })?;
            if region.replace(location.end()).is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic end cannot overwrite ordinary region ownership",
                ));
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn validate_bootstrap_storage_operations(
    program: &SystemsProgram,
) -> Result<(), SystemsModelError> {
    let mut storage = BootstrapStorageState::new(
        program.bootstrap_storage.clone(),
        "systems-program-validation",
    )?;
    let mut region: Option<BootstrapRegion> = None;
    let mut context_stacks: Vec<BootstrapRegion> = Vec::new();
    let mut atomic: Option<AtomicWordLocation> = None;
    let mut authority = BootstrapAuthorityState::default();
    let mut owner_had_atomic = false;
    for (index, operation) in program.bootstrap.handler.operations.iter().enumerate() {
        match operation {
            SystemsOperation::BootstrapAllocate { .. } => owner_had_atomic = false,
            SystemsOperation::AtomicWordCreate { .. } => owner_had_atomic = true,
            SystemsOperation::ApplyBootstrapRegionLibrary if owner_had_atomic => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-REGION-ROOT",
                    "initial source loan must precede atomic derivation",
                ));
            }
            _ => {}
        }
        let authority_operation = authority.observe(index, operation)?;
        if validate_kernel_context_storage_operation(
            operation,
            &mut storage,
            &mut region,
            &mut context_stacks,
            &authority.kernel_context,
        )? {
            continue;
        }
        if authority_operation {
            continue;
        }
        if validate_atomic_operation(operation, &mut storage, &mut region, &mut atomic)? {
            continue;
        }
        validate_bootstrap_storage_operation(operation, &mut storage, &mut region)?;
    }
    authority.complete()?;
    validate_bootstrap_owned_completion(region.as_ref(), atomic.as_ref(), &context_stacks)
}

fn validate_kernel_context_storage_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    context_stacks: &mut Vec<BootstrapRegion>,
    state: &BootstrapKernelContextState,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::CreateKernelCompletion => {}
        SystemsOperation::CreateKernelContext => {
            let stack = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context creation requires one live exclusive stack region",
                )
            })?;
            if stack.byte_count() != 16_384
                || stack.alignment_bytes() != 16
                || stack.placement() != crate::BootstrapStoragePlacement::BootstrapReclaimable
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "the initial kernel context requires one 16 KiB, 16-byte-aligned bootstrap-reclaimable stack",
                ));
            }
            context_stacks.push(stack);
        }
        SystemsOperation::TransferKernelContext
        | SystemsOperation::TransferKernelContextUntilDeadline
        | SystemsOperation::TransferKernelContextUntilCompletion => {
            if context_stacks.is_empty() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context transfer requires one live suspended context stack",
                ));
            }
        }
        SystemsOperation::CreateRunnableQueue { .. }
        | SystemsOperation::EnqueueRunnableContext
        | SystemsOperation::DequeueRunnableContext
        | SystemsOperation::TakePreemptedKernelContext
        | SystemsOperation::SignalBlockedKernelCompletion
        | SystemsOperation::ConsumeKernelCompletion
        | SystemsOperation::ConsumeEmptyRunnableQueue => {}
        SystemsOperation::ReclaimKernelContext => {
            let index = match state {
                BootstrapKernelContextState::TerminalReclaimed => 2,
                BootstrapKernelContextState::PreemptibleReclaimed
                | BootstrapKernelContextState::CompletionReclaimed => 0,
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context reclaim requires one completed context stack",
                    ));
                }
            };
            if context_stacks.len() <= index {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context reclaim requires its owned context stack",
                ));
            }
            storage.release(context_stacks.remove(index))?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn validate_bootstrap_storage_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
) -> Result<(), SystemsModelError> {
    match operation {
        SystemsOperation::BootstrapAllocate { request } => {
            if region.is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "the initial executable slice admits one live bootstrap region",
                ));
            }
            *region = Some(storage.allocate(*request).map_err(|code| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-REQUEST",
                    format!("static bootstrap allocation cannot succeed: {code:?}"),
                )
            })?);
        }
        SystemsOperation::BootstrapStoreByte {
            offset_bytes,
            value,
        } => storage.store_byte(
            region.as_ref().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "bootstrap byte store requires one live region",
                )
            })?,
            *offset_bytes,
            *value,
        )?,
        SystemsOperation::BootstrapLoadByteEquals { offset_bytes, .. } => {
            let _ = storage.known_byte(
                region.as_ref().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-STORAGE-LIFETIME",
                        "bootstrap byte load requires one live region",
                    )
                })?,
                *offset_bytes,
            )?;
        }
        SystemsOperation::ApplyBootstrapRegionLibrary => {
            let owner = required_region(region.as_ref(), "library apply")?;
            if owner.byte_count() != 64 {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-REGION-ROOT",
                    "root region library requires the 64-byte plain owner",
                ));
            }
            storage.opaque_library_effect(owner)?;
        }
        SystemsOperation::BootstrapRelease => {
            let released = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "bootstrap release requires one live region",
                )
            })?;
            storage.release(released)?;
        }
        SystemsOperation::ConsoleWrite { .. } | SystemsOperation::DebugBreak => {}
        SystemsOperation::PrepareUserAddressSpace
        | SystemsOperation::VerifyUserAddressSpaceMapping
        | SystemsOperation::MapUserAddressSpace
        | SystemsOperation::ReserveAnonymousUserMapping
        | SystemsOperation::StoreUserMappingByte
        | SystemsOperation::InstallUserMappingEntryTrampoline
        | SystemsOperation::ProtectUserMappingReadOnly
        | SystemsOperation::PermitUserMappingExecute
        | SystemsOperation::SealUserMappingImmutableData
        | SystemsOperation::PermitUserMappingWritableData
        | SystemsOperation::LoadUserMappingByte
        | SystemsOperation::VerifyUserMappingEntryTrampoline
        | SystemsOperation::UnmapUserAddressSpace
        | SystemsOperation::ReleaseAnonymousUserMapping
        | SystemsOperation::PrepareUserProbe { .. }
        | SystemsOperation::EnterUserContext
        | SystemsOperation::CompleteUserExecution
        | SystemsOperation::ReclaimUserAddressSpace => {}
        _ => unreachable!("authority or atomic operations continue above"),
    }
    Ok(())
}

fn validate_bootstrap_owned_completion(
    region: Option<&BootstrapRegion>,
    atomic: Option<&AtomicWordLocation>,
    context_stacks: &[BootstrapRegion],
) -> Result<(), SystemsModelError> {
    if region.is_some() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-STORAGE-LIVE",
            "bootstrap handler consumes its context while a region remains live",
        ));
    }
    if atomic.is_some() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-ATOMIC-LIVE",
            "bootstrap handler consumes its context while an atomic location remains live",
        ));
    }
    if !context_stacks.is_empty() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-CONTEXT-TRANSFER-LIVE",
            "bootstrap handler consumes its context while a suspended or completed kernel context remains live",
        ));
    }
    Ok(())
}

/// Execute the deterministic abstract transition model for one checked root.
///
/// # Errors
///
/// Returns a stable systems-model diagnostic when the program is outside the
/// sealed initial profile.
pub fn model_systems_transitions(
    program: &SystemsProgram,
) -> Result<Vec<SystemsTransition>, SystemsModelError> {
    validate_systems_program(program)?;
    let mut transitions = initial_systems_transitions(program);
    let mut storage = BootstrapStorageState::new(
        program.bootstrap_storage.clone(),
        "systems-transition-model",
    )?;
    let mut region: Option<BootstrapRegion> = None;
    let mut atomic: Option<AtomicWordLocation> = None;
    let mut context_stacks: Vec<BootstrapRegion> = Vec::new();
    let mut critical_stack = Vec::new();
    let mut next_critical_identity = 1_u64;
    let mut local_notification =
        LocalNotificationProtocol::new("initial-local-notification-source");
    let mut pending_local_notification = None;
    let mut next_clock_observation_identity = 1_u64;
    let mut next_pci_transaction_identity = 1_u64;
    let mut next_pci_region_transaction_identity = 1_u64;
    let mut next_pci_update_transaction_identity = 1_u64;
    let mut deadline_state = BootstrapDeadlineState::Fresh;
    let mut next_deadline_event_identity = 1_u64;
    let mut kernel_context_state = BootstrapKernelContextState::Fresh;
    let mut user_protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
    let mut prepared_user_space: Option<PreparedUserAddressSpace> = None;
    let mut user_mapping: Option<UserMapping> = None;
    let mut resident_code_mapping: Option<UserMapping> = None;
    let mut anonymous_reservation: Option<ReservedAnonymousUserMapping> = None;
    let mut suspended_user: Option<SuspendedUserContext> = None;
    let mut completed_user: Option<CompletedUserExecution> = None;
    let mut revoked_user_space: Option<RevokedUserAddressSpace> = None;
    for operation in &program.bootstrap.handler.operations {
        if model_critical_transition(
            operation,
            &mut critical_stack,
            &mut next_critical_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_local_notification_transition(
            operation,
            &program.local_notification.handler,
            &mut local_notification,
            &mut pending_local_notification,
            &mut transitions,
        )? {
            continue;
        }
        if model_monotonic_clock_transition(
            operation,
            &mut next_clock_observation_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_deadline_transition(
            operation,
            &program.deadline_notification.handler,
            &mut deadline_state,
            &mut next_deadline_event_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_kernel_context_transition(
            operation,
            &program.kernel_thread.handler,
            &program.completion_thread.handler,
            &program.terminal_thread.handler,
            &program.deadline_notification.handler,
            &mut storage,
            &mut region,
            &mut context_stacks,
            &mut kernel_context_state,
            &mut deadline_state,
            &mut next_deadline_event_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_user_execution_transition(
            operation,
            &program.user_syscall.handler,
            &mut user_protocol,
            &mut prepared_user_space,
            &mut user_mapping,
            &mut resident_code_mapping,
            &mut anonymous_reservation,
            &mut suspended_user,
            &mut completed_user,
            &mut revoked_user_space,
            &mut transitions,
        )? {
            continue;
        }
        if model_bootstrap_operation(
            operation,
            &program.debug_break.handler,
            program.region_library.as_ref().is_some_and(|library| {
                matches!(
                    library.interface,
                    crate::SystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
                        | crate::SystemsSourceLibraryInterface::BorrowedByteRegionTailActionV1
                )
            }),
            &mut storage,
            &mut region,
            &mut atomic,
            &mut next_pci_transaction_identity,
            &mut next_pci_region_transaction_identity,
            &mut next_pci_update_transaction_identity,
            &mut transitions,
        )? {
            return Ok(transitions);
        }
    }
    model_bootstrap_disposition(&program.bootstrap.handler.disposition, &mut transitions);
    Ok(transitions)
}
