fn model_atomic_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<Option<bool>, SystemsModelError> {
    match operation {
        SystemsOperation::AtomicWordCreate { request } => {
            let owned_region = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic word creation requires one live ordinary region",
                )
            })?;
            *atomic = Some(AtomicWordLocation::create(storage, owned_region, *request)?);
            transitions.push(SystemsTransition::CreateAtomicWord { request: *request });
        }
        SystemsOperation::AtomicCompareExchangeEquals {
            expected,
            desired,
            success_order,
            failure_order,
            failure_message,
        } => {
            let outcome = atomic
                .as_mut()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "compare/exchange requires one live atomic word",
                    )
                })?
                .compare_exchange(storage, *expected, *desired, *success_order, *failure_order)?;
            let (observed, exchanged) = match outcome {
                AtomicCompareExchangeResult::Exchanged { previous } => (previous, true),
                AtomicCompareExchangeResult::Observed { actual } => (actual, false),
            };
            transitions.push(SystemsTransition::CompareExchangeAtomicWord {
                expected: *expected,
                desired: *desired,
                observed,
                exchanged,
                success_order: *success_order,
                failure_order: *failure_order,
            });
            if !exchanged {
                transitions.push(SystemsTransition::Fatal {
                    message: failure_message.clone(),
                });
                return Ok(Some(true));
            }
        }
        SystemsOperation::AtomicLoadEquals {
            order,
            expected,
            failure_message,
        } => {
            let value = atomic
                .as_ref()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "atomic load requires one live atomic word",
                    )
                })?
                .load(storage, *order)?;
            transitions.push(SystemsTransition::LoadAtomicWord {
                value,
                order: *order,
            });
            if value != *expected {
                transitions.push(SystemsTransition::Fatal {
                    message: failure_message.clone(),
                });
                return Ok(Some(true));
            }
        }
        SystemsOperation::AtomicWordEnd => {
            let location = atomic.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic end requires one live atomic word",
                )
            })?;
            *region = Some(location.end());
            transitions.push(SystemsTransition::EndAtomicWord);
        }
        _ => return Ok(None),
    }
    Ok(Some(false))
}

fn model_bootstrap_storage_operation(
    operation: &SystemsOperation,
    require_region_true: bool,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    if let Some(terminal) = model_atomic_operation(operation, storage, region, atomic, transitions)?
    {
        return Ok(terminal);
    }
    match operation {
        SystemsOperation::DescribeBootMemory { .. } => {
            unreachable!("boot-memory refinement is modeled by the caller")
        }
        SystemsOperation::CreateFrameAllocator { .. }
        | SystemsOperation::AllocatePhysicalFrames { .. }
        | SystemsOperation::ReserveStaticImageHierarchy { .. }
        | SystemsOperation::ReleasePhysicalFrames
        | SystemsOperation::PrepareStaticImageConstruction { .. }
        | SystemsOperation::ZeroStaticImageConstructionPayload
        | SystemsOperation::ZeroStaticImageStackBacking
        | SystemsOperation::PublishStaticImageStackLeaf
        | SystemsOperation::InitializeStaticImageStackEntry
        | SystemsOperation::AddStaticImageStackPageSize
        | SystemsOperation::AddStaticImageStackEntryFact
        | SystemsOperation::AddStaticImageStackProgramHeaders
        | SystemsOperation::AddStaticImageStackExecutableName
        | SystemsOperation::AddStaticImageStackCredentials
        | SystemsOperation::AddStaticImageStackSecureMode
        | SystemsOperation::AddStaticImageStackClockTickRate
        | SystemsOperation::AddStaticImageStackPlatform
        | SystemsOperation::AddStaticImageStackHardwareCapabilities
        | SystemsOperation::AddStaticImageStackLoadMetadata
        | SystemsOperation::AuthorizeStaticImageStackRandomSlot
        | SystemsOperation::RetainStaticImagePreparedProcess
        | SystemsOperation::QualifyStaticImageRetainedDisjointWork
        | SystemsOperation::AuthorizeStaticImageEntropyHandoff
        | SystemsOperation::AuthorizeStaticImageEntropyAcquisition
        | SystemsOperation::QualifyStaticImageEntropyDevice
        | SystemsOperation::QualifyStaticImageEntropyTransport
        | SystemsOperation::QualifyStaticImageEntropyNegotiation
        | SystemsOperation::QualifyStaticImageEntropyQueue
        | SystemsOperation::QualifyStaticImageEntropyBusMaster
        | SystemsOperation::QualifyStaticImageEntropyDmaBuffers
        | SystemsOperation::QualifyStaticImageEntropySplitRing
        | SystemsOperation::QualifyStaticImageEntropyDmaPreparation
        | SystemsOperation::QualifyStaticImageEntropyQueueAddresses
        | SystemsOperation::QualifyStaticImageEntropyQueueActivation
        | SystemsOperation::QualifyStaticImageEntropyRequestCompletion
        | SystemsOperation::QualifyStaticImageEntropySampleCapture
        | SystemsOperation::QualifyStaticImageEntropySampleQuality
        | SystemsOperation::QualifyStaticImageEntropyPoolLifecycle
        | SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle
        | SystemsOperation::DeliverStaticImageEntropyInitialRandom
        | SystemsOperation::EnterStaticImageProcess
        | SystemsOperation::RestoreStaticImageProcessEntry
        | SystemsOperation::RevokeStaticImageEntropyInitialRandom
        | SystemsOperation::RevokeStaticImageEntropyAcquisition
        | SystemsOperation::RevokeStaticImageEntropyHandoff
        | SystemsOperation::RestoreStaticImagePreparedProcess
        | SystemsOperation::RevokeStaticImageStackRandomSlot
        | SystemsOperation::RemoveStaticImageStackLoadMetadata
        | SystemsOperation::RemoveStaticImageStackHardwareCapabilities
        | SystemsOperation::RemoveStaticImageStackPlatform
        | SystemsOperation::RemoveStaticImageStackClockTickRate
        | SystemsOperation::RemoveStaticImageStackSecureMode
        | SystemsOperation::RemoveStaticImageStackCredentials
        | SystemsOperation::RemoveStaticImageStackExecutableName
        | SystemsOperation::RemoveStaticImageStackProgramHeaders
        | SystemsOperation::RemoveStaticImageStackEntryFact
        | SystemsOperation::RemoveStaticImageStackPageSize
        | SystemsOperation::ClearStaticImageStackEntry
        | SystemsOperation::RevokeStaticImageStackLeaf
        | SystemsOperation::CopyStaticImageConstructionPayload
        | SystemsOperation::SealStaticImageConstructionPermissions
        | SystemsOperation::AttachStaticImageAddressSpace
        | SystemsOperation::ActivateStaticImageAddressSpace
        | SystemsOperation::RestoreStaticImageAddressSpace
        | SystemsOperation::AuthorizeStaticImageEntry
        | SystemsOperation::AuthorizeStaticImageStackSlot
        | SystemsOperation::RevokeStaticImageStackSlot
        | SystemsOperation::RevokeStaticImageEntry
        | SystemsOperation::DetachStaticImageAddressSpace
        | SystemsOperation::ReclaimStaticImageConstructionHierarchy
        | SystemsOperation::RevokeStaticImageConstructionMetadata
        | SystemsOperation::ReclaimStaticImageConstructionPayload
        | SystemsOperation::MapKernelFrames { .. }
        | SystemsOperation::KernelMappingStoreByte { .. }
        | SystemsOperation::KernelMappingLoadByteEquals { .. }
        | SystemsOperation::UnmapKernelFrames => {
            unreachable!("physical-frame and mapping operations are modeled by the caller")
        }
        SystemsOperation::BeginTranslationUpdate { .. }
        | SystemsOperation::CommitTranslationUpdate { .. }
        | SystemsOperation::ActivateTranslationSpace { .. }
        | SystemsOperation::BeginTranslationEdit { .. }
        | SystemsOperation::MapTranslationFrames { .. }
        | SystemsOperation::UnmapTranslationMapping
        | SystemsOperation::CommitTranslationEdit { .. } => {
            unreachable!("translation operations are modeled by the caller")
        }
        SystemsOperation::EnterCritical { .. } | SystemsOperation::RestoreCritical { .. } => {
            unreachable!("critical operations are modeled by the caller")
        }
        SystemsOperation::SendLocalNotification
        | SystemsOperation::WaitLocalNotification
        | SystemsOperation::CompleteLocalNotification => {
            unreachable!("local-notification operations are modeled by the caller")
        }
        SystemsOperation::ObserveMonotonicClock => {
            unreachable!("monotonic-clock observations are modeled by the caller")
        }
        SystemsOperation::ObservePciConfigurationDword { .. }
        | SystemsOperation::UpdatePciConfigurationWord { .. }
        | SystemsOperation::CreateDmaBuffer { .. }
        | SystemsOperation::BindDmaBufferReference { .. }
        | SystemsOperation::StoreDmaBufferInteger { .. }
        | SystemsOperation::PrepareDmaBufferForDevice { .. }
        | SystemsOperation::CompleteDmaBufferForCpu { .. }
        | SystemsOperation::LoadDmaBufferIntegerEquals { .. }
        | SystemsOperation::CaptureEntropySample { .. }
        | SystemsOperation::AssessEntropySampleQuality { .. }
        | SystemsOperation::CreateEntropyPool { .. }
        | SystemsOperation::MixEntropyPoolSample { .. }
        | SystemsOperation::CreditEntropyPoolContribution { .. }
        | SystemsOperation::InitializeEntropyGenerator { .. }
        | SystemsOperation::ExtractEntropyGenerator { .. }
        | SystemsOperation::DeliverInitialAuxvRandom { .. }
        | SystemsOperation::ReserveInitialUserRandom { .. }
        | SystemsOperation::BindReusableUserRandom { .. }
        | SystemsOperation::ExpandSmallUserRandomRequests { .. }
        | SystemsOperation::ExpandLargeUserRandomRequests { .. }
        | SystemsOperation::PublishPciRegionDmaReference { .. }
        | SystemsOperation::AccessPciRegionRegister { .. } => {
            unreachable!("PCI configuration and region accesses are modeled by the caller")
        }
        SystemsOperation::ConstructDeadline { .. }
        | SystemsOperation::ArmDeadline
        | SystemsOperation::WaitDeadline
        | SystemsOperation::CompleteDeadline => {
            unreachable!("deadline-event operations are modeled by the caller")
        }
        SystemsOperation::CreateKernelContext
        | SystemsOperation::CreateKernelCompletion
        | SystemsOperation::CreateRunnableQueue { .. }
        | SystemsOperation::EnqueueRunnableContext
        | SystemsOperation::DequeueRunnableContext
        | SystemsOperation::TransferKernelContextUntilDeadline
        | SystemsOperation::TransferKernelContextUntilCompletion
        | SystemsOperation::TakePreemptedKernelContext
        | SystemsOperation::TransferKernelContext
        | SystemsOperation::AwaitDeadlinePreemption
        | SystemsOperation::AwaitKernelCompletion
        | SystemsOperation::SignalBlockedKernelCompletion
        | SystemsOperation::ReclaimKernelContext
        | SystemsOperation::ConsumeKernelCompletion
        | SystemsOperation::ConsumeEmptyRunnableQueue => {
            unreachable!("kernel-context operations are modeled by the caller")
        }
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
        | SystemsOperation::ReclaimUserAddressSpace
        | SystemsOperation::CopyInitialUnameToUser
        | SystemsOperation::CopyInitialTimespecFromUser => {
            unreachable!("user-execution operations are modeled by the caller")
        }
        SystemsOperation::BootstrapAllocate { request } => {
            *region = Some(storage.allocate(*request).map_err(|code| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-REQUEST",
                    format!("checked bootstrap allocation failed during modeling: {code:?}"),
                )
            })?);
            transitions.push(SystemsTransition::AllocateBootstrapRegion { request: *request });
        }
        SystemsOperation::BootstrapStoreByte {
            offset_bytes,
            value,
        } => {
            storage.store_byte(
                required_region(region.as_ref(), "store")?,
                *offset_bytes,
                *value,
            )?;
            transitions.push(SystemsTransition::StoreBootstrapByte {
                offset_bytes: *offset_bytes,
                value: *value,
            });
        }
        SystemsOperation::ApplyBootstrapRegionLibrary => {
            let generation = storage
                .opaque_library_effect(required_region(region.as_ref(), "library apply")?)?;
            transitions.extend([
                SystemsTransition::BorrowBootstrapRegionForLibrary { generation },
                SystemsTransition::RequireRegionLibraryReclamation,
                SystemsTransition::ResumeBootstrapRegionAfterLibrary {
                    require_true: require_region_true,
                },
            ]);
        }
        SystemsOperation::BootstrapLoadByteEquals {
            offset_bytes,
            expected,
            failure_message,
        } => {
            let owner = required_region(region.as_ref(), "load")?;
            if storage.known_byte(owner, *offset_bytes)?.is_none() {
                transitions.push(SystemsTransition::CheckDynamicBootstrapByte {
                    offset_bytes: *offset_bytes,
                    expected: *expected,
                });
                return Ok(false);
            }
            let value = storage.load_byte(owner, *offset_bytes)?;
            transitions.push(SystemsTransition::LoadBootstrapByte {
                offset_bytes: *offset_bytes,
                value,
            });
            if value != *expected {
                transitions.push(SystemsTransition::Fatal {
                    message: failure_message.clone(),
                });
                return Ok(true);
            }
        }
        SystemsOperation::BootstrapRelease => {
            storage.release(region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "bootstrap release requires one live region",
                )
            })?)?;
            transitions.push(SystemsTransition::ReleaseBootstrapRegion);
        }
        SystemsOperation::ConsoleWrite { .. } | SystemsOperation::DebugBreak => {
            unreachable!("non-storage operations are modeled by the caller")
        }
        SystemsOperation::AtomicWordCreate { .. }
        | SystemsOperation::AtomicCompareExchangeEquals { .. }
        | SystemsOperation::AtomicLoadEquals { .. }
        | SystemsOperation::AtomicWordEnd => {
            unreachable!("atomic operations are modeled above")
        }
    }
    Ok(false)
}

fn required_region<'a>(
    region: Option<&'a BootstrapRegion>,
    operation: &str,
) -> Result<&'a BootstrapRegion, SystemsModelError> {
    region.ok_or_else(|| {
        SystemsModelError::new(
            "E-SYSTEMS-STORAGE-LIFETIME",
            format!("bootstrap byte {operation} requires one live region"),
        )
    })
}
