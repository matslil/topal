impl RootEncoder {
    fn transition_static_image_stack_credentials(&mut self, add: bool) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackExecutableNameAdded
        } else {
            RootStaticImageState::StackCredentialsAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image credential transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackCredentials
        } else {
            ProviderSymbol::RemoveStaticImageStackCredentials
        })?;
        self.static_image = if add {
            RootStaticImageState::StackCredentialsAdded
        } else {
            RootStaticImageState::StackExecutableNameAdded
        };
        Ok(())
    }

    fn add_static_image_stack_credentials(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_credentials(true)
    }

    fn remove_static_image_stack_credentials(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_credentials(false)
    }

    fn transition_static_image_stack_secure_mode(&mut self, add: bool) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackCredentialsAdded
        } else {
            RootStaticImageState::StackSecureModeAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image secure-mode transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackSecureMode
        } else {
            ProviderSymbol::RemoveStaticImageStackSecureMode
        })?;
        self.static_image = if add {
            RootStaticImageState::StackSecureModeAdded
        } else {
            RootStaticImageState::StackCredentialsAdded
        };
        Ok(())
    }

    fn add_static_image_stack_secure_mode(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_secure_mode(true)
    }

    fn remove_static_image_stack_secure_mode(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_secure_mode(false)
    }

    fn transition_static_image_stack_clock_tick_rate(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackSecureModeAdded
        } else {
            RootStaticImageState::StackClockTickRateAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image clock-tick-rate transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackClockTickRate
        } else {
            ProviderSymbol::RemoveStaticImageStackClockTickRate
        })?;
        self.static_image = if add {
            RootStaticImageState::StackClockTickRateAdded
        } else {
            RootStaticImageState::StackSecureModeAdded
        };
        Ok(())
    }

    fn add_static_image_stack_clock_tick_rate(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_clock_tick_rate(true)
    }

    fn remove_static_image_stack_clock_tick_rate(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_clock_tick_rate(false)
    }

    fn transition_static_image_stack_platform(&mut self, add: bool) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackClockTickRateAdded
        } else {
            RootStaticImageState::StackPlatformAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image platform transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackPlatform
        } else {
            ProviderSymbol::RemoveStaticImageStackPlatform
        })?;
        self.static_image = if add {
            RootStaticImageState::StackPlatformAdded
        } else {
            RootStaticImageState::StackClockTickRateAdded
        };
        Ok(())
    }

    fn add_static_image_stack_platform(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_platform(true)
    }

    fn remove_static_image_stack_platform(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_platform(false)
    }

    fn transition_static_image_stack_hardware_capabilities(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackPlatformAdded
        } else {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image hardware-capability transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackHardwareCapabilities
        } else {
            ProviderSymbol::RemoveStaticImageStackHardwareCapabilities
        })?;
        self.static_image = if add {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        } else {
            RootStaticImageState::StackPlatformAdded
        };
        Ok(())
    }

    fn add_static_image_stack_hardware_capabilities(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_hardware_capabilities(true)
    }

    fn remove_static_image_stack_hardware_capabilities(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_hardware_capabilities(false)
    }

    fn transition_static_image_stack_load_metadata(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        } else {
            RootStaticImageState::StackLoadMetadataAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image load-metadata transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackLoadMetadata
        } else {
            ProviderSymbol::RemoveStaticImageStackLoadMetadata
        })?;
        self.static_image = if add {
            RootStaticImageState::StackLoadMetadataAdded
        } else {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        };
        Ok(())
    }

    fn add_static_image_stack_load_metadata(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_load_metadata(true)
    }

    fn remove_static_image_stack_load_metadata(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_load_metadata(false)
    }

    fn transition_static_image_stack_random_slot(
        &mut self,
        authorize: bool,
    ) -> Result<(), CompileError> {
        let required = if authorize {
            RootStaticImageState::StackLoadMetadataAdded
        } else {
            RootStaticImageState::StackRandomSlotAuthorized
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(2)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image random-slot transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if authorize {
            ProviderSymbol::AuthorizeStaticImageStackRandomSlot
        } else {
            ProviderSymbol::RevokeStaticImageStackRandomSlot
        })?;
        self.static_image = if authorize {
            RootStaticImageState::StackRandomSlotAuthorized
        } else {
            RootStaticImageState::StackLoadMetadataAdded
        };
        Ok(())
    }

    fn authorize_static_image_stack_random_slot(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_random_slot(true)
    }

    fn revoke_static_image_stack_random_slot(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_random_slot(false)
    }

    fn transition_static_image_prepared_process(
        &mut self,
        retain: bool,
    ) -> Result<(), CompileError> {
        let required = if retain {
            RootStaticImageState::StackRandomSlotAuthorized
        } else {
            RootStaticImageState::PreparedProcessRetained
        };
        let required_frames = if retain { Some(2) } else { None };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != required_frames
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image prepared-process transition requires its exact retained owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if retain {
            ProviderSymbol::RetainStaticImagePreparedProcess
        } else {
            ProviderSymbol::RestoreStaticImagePreparedProcess
        })?;
        self.static_image = if retain {
            RootStaticImageState::PreparedProcessRetained
        } else {
            RootStaticImageState::StackRandomSlotAuthorized
        };
        self.physical_frame_count = if retain { None } else { Some(2) };
        Ok(())
    }

    fn retain_static_image_prepared_process(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_prepared_process(true)
    }

    fn restore_static_image_prepared_process(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_prepared_process(false)
    }

    fn qualify_static_image_retained_disjoint_work(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::PreparedProcessRetained
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image retained disjoint work requires the exact retained owner and closed allocator channel".into()));
        }
        self.allocate_physical_frames(1, 1)?;
        self.release_physical_frames()?;
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageRetainedDisjointWork)
    }

    fn transition_static_image_entropy_handoff(
        &mut self,
        authorize: bool,
    ) -> Result<(), CompileError> {
        let required = if authorize {
            RootStaticImageState::PreparedProcessRetained
        } else {
            RootStaticImageState::EntropyPendingProcess
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-handoff transition requires its exact opaque process owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if authorize {
            ProviderSymbol::AuthorizeStaticImageEntropyHandoff
        } else {
            ProviderSymbol::RevokeStaticImageEntropyHandoff
        })?;
        self.static_image = if authorize {
            RootStaticImageState::EntropyPendingProcess
        } else {
            RootStaticImageState::PreparedProcessRetained
        };
        Ok(())
    }

    fn authorize_static_image_entropy_handoff(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_handoff(true)
    }

    fn revoke_static_image_entropy_handoff(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_handoff(false)
    }

    fn transition_static_image_entropy_acquisition(
        &mut self,
        authorize: bool,
    ) -> Result<(), CompileError> {
        let required = if authorize {
            RootStaticImageState::EntropyPendingProcess
        } else {
            RootStaticImageState::EntropyAcquiringProcess
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-acquisition transition requires its exact opaque process owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if authorize {
            ProviderSymbol::AuthorizeStaticImageEntropyAcquisition
        } else {
            ProviderSymbol::RevokeStaticImageEntropyAcquisition
        })?;
        self.static_image = if authorize {
            RootStaticImageState::EntropyAcquiringProcess
        } else {
            RootStaticImageState::EntropyPendingProcess
        };
        Ok(())
    }

    fn authorize_static_image_entropy_acquisition(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_acquisition(true)
    }
}
