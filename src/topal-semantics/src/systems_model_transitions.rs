fn model_user_execution_transition(
    operation: &SystemsOperation,
    handler: &SystemsHandler,
    protocol: &mut SystemsUserExecutionProtocol,
    prepared: &mut Option<PreparedUserAddressSpace>,
    mapping: &mut Option<UserMapping>,
    resident_code: &mut Option<UserMapping>,
    anonymous_reservation: &mut Option<ReservedAnonymousUserMapping>,
    suspended: &mut Option<SuspendedUserContext>,
    completed: &mut Option<CompletedUserExecution>,
    revoked: &mut Option<RevokedUserAddressSpace>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::PrepareUserAddressSpace => {
            if prepared.is_some() || suspended.is_some() || completed.is_some() || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user model admits exactly one live address-space lifecycle",
                ));
            }
            *prepared = Some(protocol.prepare_address_space(INITIAL_MULTI_FRAME_REQUEST)?);
            transitions.push(SystemsTransition::PrepareUserAddressSpace {
                address_space_identity: 1,
            });
        }
        SystemsOperation::VerifyUserAddressSpaceMapping => {
            if prepared.is_none() || suspended.is_some() || completed.is_some() || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user mapping model requires one prepared address space",
                ));
            }
            transitions.push(SystemsTransition::VerifyUserAddressSpaceMapping {
                address_space_identity: 1,
            });
        }
        SystemsOperation::MapUserAddressSpace => {
            if prepared.is_none()
                || mapping.is_some()
                || (resident_code.is_some()
                    && resident_code.as_ref().map(UserMapping::identity) != Some(1))
                || suspended.is_some()
                || completed.is_some()
                || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user mapping model requires one prepared address space",
                ));
            }
            let mapping_request = if resident_code.is_none() {
                INITIAL_TWO_FRAME_REQUEST
            } else {
                INITIAL_SINGLE_FRAME_REQUEST
            };
            let fresh_mapping = protocol.map_user_frame(
                prepared.as_ref().expect("checked prepared user space"),
                mapping_request,
            )?;
            let mapping_identity = fresh_mapping.identity();
            *mapping = Some(fresh_mapping);
            transitions.push(SystemsTransition::MapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity,
            });
        }
        SystemsOperation::ReserveAnonymousUserMapping => {
            if prepared.is_none()
                || mapping.is_none()
                || resident_code.is_none()
                || anonymous_reservation.is_some()
                || suspended.is_some()
                || completed.is_some()
                || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "anonymous reservation requires both resident image mappings and one prepared address space",
                ));
            }
            let reservation = protocol.reserve_anonymous_user_frame(
                prepared.as_ref().expect("checked prepared user space"),
                INITIAL_SINGLE_FRAME_REQUEST,
            )?;
            let reservation_identity = reservation.identity();
            *anonymous_reservation = Some(reservation);
            transitions.push(SystemsTransition::ReserveAnonymousUserMapping {
                address_space_identity: 1,
                reservation_identity,
            });
        }
        SystemsOperation::StoreUserMappingByte => {
            *mapping = Some(protocol.store_user_mapping_byte(mapping.take().ok_or_else(
                || {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping store requires one live affine mapping",
                    )
                },
            )?)?);
            let mapping_identity = mapping
                .as_ref()
                .expect("stored mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::StoreUserMappingByte { mapping_identity });
        }
        SystemsOperation::InstallUserMappingEntryTrampoline => {
            *mapping = Some(protocol.install_user_mapping_entry_trampoline(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline installation requires one live affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("entry-installed mapping was just restored")
                .identity();
            transitions
                .push(SystemsTransition::InstallUserMappingEntryTrampoline { mapping_identity });
        }
        SystemsOperation::ProtectUserMappingReadOnly => {
            *mapping = Some(protocol.protect_user_mapping_read_only(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping protection requires one stored affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("protected mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::ProtectUserMappingReadOnly { mapping_identity });
        }
        SystemsOperation::LoadUserMappingByte => {
            *mapping = Some(protocol.load_user_mapping_byte(mapping.take().ok_or_else(
                || {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping load requires one protected affine mapping",
                    )
                },
            )?)?);
            let mapping_identity = mapping
                .as_ref()
                .expect("verified mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::LoadUserMappingByte { mapping_identity });
        }
        SystemsOperation::VerifyUserMappingEntryTrampoline => {
            *mapping = Some(protocol.verify_user_mapping_entry_trampoline(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline verification requires one protected affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("entry-verified mapping was just restored")
                .identity();
            transitions
                .push(SystemsTransition::VerifyUserMappingEntryTrampoline { mapping_identity });
        }
        SystemsOperation::PermitUserMappingExecute => {
            if resident_code.is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user model admits exactly one resident executable-code mapping",
                ));
            }
            *resident_code = Some(protocol.permit_user_mapping_execute(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping execute promotion requires one verified affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = resident_code
                .as_ref()
                .expect("executable mapping was just retained")
                .identity();
            transitions.push(SystemsTransition::PermitUserMappingExecute { mapping_identity });
        }
        SystemsOperation::SealUserMappingImmutableData => {
            *mapping = Some(protocol.seal_user_mapping_immutable_data(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping data sealing requires one verified affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("immutable-data mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::SealUserMappingImmutableData { mapping_identity });
        }
        SystemsOperation::PermitUserMappingWritableData => {
            *mapping = Some(protocol.permit_user_mapping_writable_data(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping writable-data promotion requires one verified affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("writable-data mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::PermitUserMappingWritableData { mapping_identity });
        }
        SystemsOperation::UnmapUserAddressSpace => {
            let mapping = if mapping.is_some() {
                mapping.take().expect("mapping presence checked")
            } else {
                resident_code.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user unmap requires one role-specific affine mapping",
                    )
                })?
            };
            let mapping_identity = mapping.identity();
            let returned = if let Some(address_space) = prepared.as_ref() {
                protocol.unmap_user_frame(address_space, mapping)?
            } else {
                protocol.unmap_completed_user_frame(
                    completed.as_ref().ok_or_else(|| {
                        SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user unmap requires one prepared or exactly returned address space",
                        )
                    })?,
                    mapping,
                )?
            };
            let expected_return = if mapping_identity == 1 {
                INITIAL_TWO_FRAME_REQUEST
            } else {
                INITIAL_SINGLE_FRAME_REQUEST
            };
            if returned != expected_return {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user unmap returned the wrong physical-frame extent",
                ));
            }
            transitions.push(SystemsTransition::UnmapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity,
            });
        }
        SystemsOperation::ReleaseAnonymousUserMapping => {
            let reservation = anonymous_reservation.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "anonymous release requires one live affine reservation",
                )
            })?;
            let reservation_identity = reservation.identity();
            let returned = protocol.release_anonymous_user_frame(
                completed.as_ref().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous release requires completed user execution",
                    )
                })?,
                reservation,
            )?;
            if returned != INITIAL_SINGLE_FRAME_REQUEST {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "anonymous release returned the wrong physical-frame extent",
                ));
            }
            transitions.push(SystemsTransition::ReleaseAnonymousUserMapping {
                address_space_identity: 1,
                reservation_identity,
            });
        }
        SystemsOperation::PrepareUserProbe { process_identity } => {
            if mapping.is_none()
                || resident_code.is_none()
                || anonymous_reservation.is_none()
                || suspended.is_some()
                || completed.is_some()
                || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user probe requires exactly one resident executable-code and immutable-data mapping",
                ));
            }
            *suspended = Some(protocol.prepare_probe(
                *process_identity,
                prepared.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user probe model requires one prepared address space",
                    )
                })?,
            )?);
            transitions.push(SystemsTransition::PrepareUserProbe {
                process_identity: *process_identity,
                probe_identity: 1,
            });
        }
        SystemsOperation::EnterUserContext => {
            let suspended_context = suspended.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user entry model requires one prepared suspended context",
                )
            })?;
            let active = protocol.enter(suspended_context)?;
            transitions.push(SystemsTransition::EnterUserExecution {
                process_identity: 1,
                kernel_continuation_identity: 1,
            });
            let (
                getpid_result,
                uname_value,
                nanosleep_value,
                remainder_candidate,
                exit_group_status,
            ) = match &handler.disposition {
                SystemsDisposition::DispatchInitialUserSyscall {
                    getpid_result,
                    uname_sysname,
                    uname_nodename,
                    uname_release,
                    uname_version,
                    uname_machine,
                    uname_domainname,
                    nanosleep_seconds,
                    nanosleep_nanoseconds,
                    nanosleep_remainder_candidate,
                    clock_gettime_result: _,
                    clock_getres_result: _,
                    write_descriptor: _,
                    write_maximum_bytes: _,
                    read_descriptor: _,
                    read_maximum_bytes: _,
                    initial_descriptor_count: _,
                    bad_descriptor_error: _,
                    too_many_open_files_error: _,
                    console_input_description: _,
                    console_output_description: _,
                    console_error_description: _,
                    invalid_argument_error: _,
                    fcntl_getfd_command: _,
                    fcntl_setfd_command: _,
                    fcntl_getfl_command: _,
                    close_on_exec_flag: _,
                    read_only_status: _,
                    write_only_status: _,
                    stat_layout_bytes: _,
                    console_device: _,
                    console_inode: _,
                    console_mode: _,
                    console_special_device: _,
                    console_block_size: _,
                    terminal_layout_bytes: _,
                    terminal_get_attributes_request: _,
                    terminal_control_flags: _,
                    terminal_minimum_input: _,
                    not_a_terminal_error: _,
                    window_size_layout_bytes: _,
                    window_size_query_request: _,
                    seek_maximum_whence: _,
                    illegal_seek_error: _,
                    brk_data_page_bytes: _,
                    getppid_result: _,
                    gettid_result: _,
                    getuid_result: _,
                    geteuid_result: _,
                    getgid_result: _,
                    getegid_result: _,
                    arch_set_fs_result: _,
                    arch_get_fs_result: _,
                    set_tid_address_result: _,
                    set_robust_list_result: _,
                    getrlimit_result: _,
                    prlimit64_result: _,
                    rt_sigprocmask_result: _,
                    anonymous_page_bytes: _,
                    madvise_result: _,
                    mprotect_result: _,
                    munmap_result: _,
                    exit_group_status,
                    unsupported_syscall_error: _,
                    sched_yield_result: _,
                } => (
                    *getpid_result,
                    LinuxUtsName {
                        sysname: uname_sysname.clone(),
                        nodename: uname_nodename.clone(),
                        release: uname_release.clone(),
                        version: uname_version.clone(),
                        machine: uname_machine.clone(),
                        domainname: uname_domainname.clone(),
                    },
                    LinuxTimespec {
                        seconds: i64::try_from(*nanosleep_seconds).map_err(|_| {
                            SystemsModelError::new(
                                "E-SYSTEMS-USER-TRANSFER",
                                "sealed nanosleep seconds exceed the native signed layout",
                            )
                        })?,
                        nanoseconds: i64::try_from(*nanosleep_nanoseconds).map_err(|_| {
                            SystemsModelError::new(
                                "E-SYSTEMS-USER-TRANSFER",
                                "sealed nanosleep nanoseconds exceed the native signed layout",
                            )
                        })?,
                    },
                    *nanosleep_remainder_candidate,
                    *exit_group_status,
                ),
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user entry model requires the checked initial syscall dispatcher",
                    ));
                }
            };
            let getpid = protocol.enter_syscall(active, crate::LINUX_X86_64_GETPID, [0; 6])?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 1,
                number: crate::LINUX_X86_64_GETPID,
                arguments: [0; 6],
            });
            let active = protocol.return_getpid(getpid, getpid_result)?;
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 1,
                result: getpid_result,
            });
            let uname = protocol.enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 2,
                number: LINUX_X86_64_UNAME,
                arguments: [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            });
            let scope = protocol.begin_uname_transfer(uname)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 2,
                transfer_identity: 1,
                candidate: INITIAL_USER_UNAME_VALID_CANDIDATE,
                extent: LINUX_NEW_UTSNAME_BYTES,
            });
            let outcome = protocol.copy_initial_utsname_to_user(
                scope,
                &uname_value,
                LINUX_NEW_UTSNAME_BYTES,
            )?;
            transitions.push(SystemsTransition::CopyToUserComplete {
                transfer_identity: 1,
                copied: LINUX_NEW_UTSNAME_BYTES,
            });
            let active = protocol.return_uname(outcome, 0)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 2,
                transfer_identity: 1,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 2,
                result: 0,
            });
            let uname = protocol.enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 3,
                number: LINUX_X86_64_UNAME,
                arguments: [INITIAL_USER_UNAME_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            });
            let scope = protocol.begin_uname_transfer(uname)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 3,
                transfer_identity: 2,
                candidate: INITIAL_USER_UNAME_FAULT_CANDIDATE,
                extent: LINUX_NEW_UTSNAME_BYTES,
            });
            let outcome = protocol.copy_initial_utsname_to_user(scope, &uname_value, 256)?;
            transitions.push(SystemsTransition::CopyToUserFault {
                transfer_identity: 2,
                copied_prefix: 256,
            });
            let active = protocol.return_uname(outcome, LINUX_X86_64_EFAULT_RESULT)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 3,
                transfer_identity: 2,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 3,
                result: LINUX_X86_64_EFAULT_RESULT,
            });
            let nanosleep = protocol.enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [
                    INITIAL_USER_TIMESPEC_VALID_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 4,
                number: LINUX_X86_64_NANOSLEEP,
                arguments: [
                    INITIAL_USER_TIMESPEC_VALID_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            });
            let scope = protocol.begin_nanosleep_transfer(nanosleep)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 4,
                transfer_identity: 3,
                candidate: INITIAL_USER_TIMESPEC_VALID_CANDIDATE,
                extent: LINUX_TIMESPEC_BYTES,
            });
            let outcome = protocol.copy_initial_timespec_from_user(
                scope,
                Some(nanosleep_value),
                LINUX_TIMESPEC_BYTES,
            )?;
            transitions.push(SystemsTransition::CopyFromUserComplete {
                transfer_identity: 3,
                copied: LINUX_TIMESPEC_BYTES,
                seconds: nanosleep_value.seconds,
                nanoseconds: nanosleep_value.nanoseconds,
            });
            let active = protocol.return_nanosleep(outcome, 0)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 4,
                transfer_identity: 3,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 4,
                result: 0,
            });
            let nanosleep = protocol.enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [
                    INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 5,
                number: LINUX_X86_64_NANOSLEEP,
                arguments: [
                    INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            });
            let scope = protocol.begin_nanosleep_transfer(nanosleep)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 5,
                transfer_identity: 4,
                candidate: INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
                extent: LINUX_TIMESPEC_BYTES,
            });
            let outcome = protocol.copy_initial_timespec_from_user(scope, None, 8)?;
            transitions.push(SystemsTransition::CopyFromUserFault {
                transfer_identity: 4,
                copied_prefix: 8,
            });
            let active = protocol.return_nanosleep(outcome, LINUX_X86_64_EFAULT_RESULT)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 5,
                transfer_identity: 4,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 5,
                result: LINUX_X86_64_EFAULT_RESULT,
            });
            let exit = protocol.enter_syscall(
                active,
                crate::LINUX_X86_64_EXIT_GROUP,
                [exit_group_status, 0, 0, 0, 0, 0],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 6,
                number: crate::LINUX_X86_64_EXIT_GROUP,
                arguments: [exit_group_status, 0, 0, 0, 0, 0],
            });
            *completed = Some(protocol.terminate_exit_group(exit, exit_group_status)?);
            transitions.push(SystemsTransition::TerminateUser {
                entry_identity: 6,
                status: exit_group_status,
            });
            transitions.push(SystemsTransition::ResumeUserKernelContinuation {
                kernel_continuation_identity: 1,
                status: exit_group_status,
            });
        }
        SystemsOperation::CompleteUserExecution => {
            if mapping.is_some() || resident_code.is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user completion requires both resident image mappings revoked",
                ));
            }
            let summary = protocol.complete(completed.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user completion model requires terminal user execution",
                )
            })?)?;
            transitions.push(SystemsTransition::CompleteUserExecution {
                process_identity: summary.process_identity,
            });
            *revoked = Some(summary.revoked_address_space);
        }
        SystemsOperation::ReclaimUserAddressSpace => {
            let backing = protocol.reclaim_address_space(revoked.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user-address-space reclaim model requires one revoked space",
                )
            })?)?;
            if backing != INITIAL_MULTI_FRAME_REQUEST {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user-address-space reclaim returned the wrong backing shape",
                ));
            }
            transitions.push(SystemsTransition::ReclaimUserAddressSpace {
                address_space_identity: 1,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn model_kernel_context_transition(
    operation: &SystemsOperation,
    preemptible_handler: &SystemsHandler,
    completion_handler: &SystemsHandler,
    terminal_handler: &SystemsHandler,
    deadline_handler: &SystemsHandler,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    context_stacks: &mut Vec<BootstrapRegion>,
    state: &mut BootstrapKernelContextState,
    deadline_state: &mut BootstrapDeadlineState,
    next_deadline_event_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::CreateKernelCompletion => {
            if *state != BootstrapKernelContextState::Fresh {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-completion model creates exactly one completion before workers",
                ));
            }
            transitions.push(SystemsTransition::CreateKernelCompletion {
                completion_identity: 1,
            });
        }
        SystemsOperation::CreateKernelContext => {
            context_stacks.push(region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model requires one live stack region",
                )
            })?);
            let context_identity = context_stacks.len() as u64;
            *state = match *state {
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
                        "kernel-context model creates exactly three worker contexts",
                    ));
                }
            };
            transitions.push(SystemsTransition::CreateKernelContext { context_identity });
        }
        SystemsOperation::CreateRunnableQueue { capacity } => {
            if *state != BootstrapKernelContextState::AllSuspended || *capacity != 3 {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model requires capacity three after creating all workers",
                ));
            }
            *state = BootstrapKernelContextState::QueueCreated;
            transitions.push(SystemsTransition::CreateRunnableQueue {
                capacity: *capacity,
            });
        }
        SystemsOperation::EnqueueRunnableContext => {
            let (context_identity, next_state) = match *state {
                BootstrapKernelContextState::QueueCreated => {
                    (1, BootstrapKernelContextState::PreemptibleEnqueued)
                }
                BootstrapKernelContextState::PreemptibleEnqueued => {
                    (2, BootstrapKernelContextState::CompletionEnqueued)
                }
                BootstrapKernelContextState::CompletionEnqueued => {
                    (3, BootstrapKernelContextState::AllEnqueued)
                }
                BootstrapKernelContextState::PreemptedTaken => {
                    (1, BootstrapKernelContextState::PreemptedEnqueued)
                }
                BootstrapKernelContextState::CompletionSignaled => {
                    (2, BootstrapKernelContextState::CompletionEnqueuedAgain)
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context model enqueue violates runnable FIFO ownership",
                    ));
                }
            };
            *state = next_state;
            transitions.push(SystemsTransition::EnqueueRunnableContext { context_identity });
        }
        SystemsOperation::DequeueRunnableContext => {
            let (context_identity, next_state) = match *state {
                BootstrapKernelContextState::AllEnqueued => {
                    (1, BootstrapKernelContextState::PreemptibleDequeued)
                }
                BootstrapKernelContextState::PreemptedEnqueued => {
                    (2, BootstrapKernelContextState::CompletionDequeued)
                }
                BootstrapKernelContextState::CompletionBlocked => {
                    (3, BootstrapKernelContextState::TerminalDequeued)
                }
                BootstrapKernelContextState::CompletionEnqueuedAgain => {
                    (1, BootstrapKernelContextState::PreemptibleDequeuedAgain)
                }
                BootstrapKernelContextState::PreemptibleReclaimed => {
                    (2, BootstrapKernelContextState::CompletionDequeuedAgain)
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context model dequeue violates runnable FIFO ownership",
                    ));
                }
            };
            *state = next_state;
            transitions.push(SystemsTransition::DequeueRunnableContext { context_identity });
        }
        SystemsOperation::TransferKernelContextUntilDeadline => {
            if *state != BootstrapKernelContextState::PreemptibleDequeued
                || *deadline_state != BootstrapDeadlineState::Constructed
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "deadline-preemptible model transfer requires the first selected context and constructed deadline",
                ));
            }
            let [
                SystemsOperation::ConsoleWrite { text: entered },
                SystemsOperation::AwaitDeadlinePreemption,
                SystemsOperation::ConsoleWrite { .. },
            ] = preemptible_handler.operations.as_slice()
            else {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "preemptible handler must enter, await matching preemption, and resume",
                ));
            };
            if deadline_handler.operations != [SystemsOperation::CompleteDeadline]
                || deadline_handler.disposition != SystemsDisposition::PreemptKernelContext
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline-preemption handler must complete and preempt the bound context",
                ));
            }
            let event_identity = *next_deadline_event_identity;
            *next_deadline_event_identity = event_identity.checked_add(1).ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline-event identity exhausted",
                )
            })?;
            transitions.extend([
                SystemsTransition::ArmDeadline { event_identity },
                SystemsTransition::TransferKernelContextUntilDeadline {
                    context_identity: 1,
                    caller_identity: 1,
                    event_identity,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 1,
                },
                SystemsTransition::ConsoleWrite {
                    text: entered.clone(),
                },
                SystemsTransition::AwaitDeadlinePreemption {
                    context_identity: 1,
                },
                SystemsTransition::ObserveDeadline { event_identity },
                SystemsTransition::EnterDeadlineInterrupt { event_identity },
                SystemsTransition::CompleteDeadlineInterrupt { event_identity },
                SystemsTransition::PreemptKernelContext {
                    event_identity,
                    context_identity: 1,
                    caller_identity: 1,
                },
                SystemsTransition::ResumePreemptionDispatcher { caller_identity: 1 },
            ]);
            *deadline_state = BootstrapDeadlineState::Completed;
            *state = BootstrapKernelContextState::PreemptedOutcome;
        }
        SystemsOperation::TakePreemptedKernelContext => {
            if *state != BootstrapKernelContextState::PreemptedOutcome
                || *deadline_state != BootstrapDeadlineState::Completed
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "taking the preempted model context requires its completed event outcome",
                ));
            }
            transitions.push(SystemsTransition::TakePreemptedKernelContext {
                context_identity: 1,
                event_identity: next_deadline_event_identity.saturating_sub(1),
            });
            *state = BootstrapKernelContextState::PreemptedTaken;
        }
        SystemsOperation::TransferKernelContextUntilCompletion => {
            if *state != BootstrapKernelContextState::CompletionDequeued {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "completion-bound model transfer requires the selected block-once worker",
                ));
            }
            let [
                SystemsOperation::ConsoleWrite { text: entered },
                SystemsOperation::AwaitKernelCompletion,
                SystemsOperation::ConsoleWrite { .. },
            ] = completion_handler.operations.as_slice()
            else {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "completion worker must enter, await its one-shot completion, and resume",
                ));
            };
            transitions.extend([
                SystemsTransition::TransferKernelContextUntilCompletion {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 2,
                },
                SystemsTransition::ConsoleWrite {
                    text: entered.clone(),
                },
                SystemsTransition::AwaitKernelCompletion {
                    context_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::BlockKernelContext {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::ResumeCompletionDispatcher { caller_identity: 2 },
            ]);
            *state = BootstrapKernelContextState::CompletionBlocked;
        }
        SystemsOperation::SignalBlockedKernelCompletion => {
            if *state != BootstrapKernelContextState::TerminalReclaimed {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "completion signal requires the blocked worker after terminal reclamation",
                ));
            }
            transitions.push(SystemsTransition::SignalBlockedKernelCompletion {
                context_identity: 2,
                caller_identity: 2,
                completion_identity: 1,
            });
            *state = BootstrapKernelContextState::CompletionSignaled;
        }
        SystemsOperation::TransferKernelContext => match *state {
            BootstrapKernelContextState::TerminalDequeued => {
                let [SystemsOperation::ConsoleWrite { text }] =
                    terminal_handler.operations.as_slice()
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "terminal handler must write its entry marker before retirement",
                    ));
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 3,
                        caller_identity: 3,
                    },
                    SystemsTransition::EnterKernelThread {
                        context_identity: 3,
                    },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 3,
                        caller_identity: 3,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 3 },
                ]);
                *state = BootstrapKernelContextState::TerminalCompleted;
            }
            BootstrapKernelContextState::PreemptibleDequeuedAgain => {
                let [_, _, SystemsOperation::ConsoleWrite { text }] =
                    preemptible_handler.operations.as_slice()
                else {
                    unreachable!("validated preemptible handler shape")
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 1,
                        caller_identity: 4,
                    },
                    SystemsTransition::RestorePreemptedKernelContext {
                        context_identity: 1,
                        caller_identity: 4,
                        event_identity: next_deadline_event_identity.saturating_sub(1),
                    },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 1,
                        caller_identity: 4,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 4 },
                ]);
                *state = BootstrapKernelContextState::PreemptibleCompleted;
            }
            BootstrapKernelContextState::CompletionDequeuedAgain => {
                let [_, _, SystemsOperation::ConsoleWrite { text }] =
                    completion_handler.operations.as_slice()
                else {
                    unreachable!("validated completion handler shape")
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 2,
                        caller_identity: 5,
                    },
                    SystemsTransition::RestoreBlockedKernelContext {
                        context_identity: 2,
                        caller_identity: 5,
                        completion_identity: 1,
                    },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 2,
                        caller_identity: 5,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 5 },
                ]);
                *state = BootstrapKernelContextState::CompletionCompleted;
            }
            _ => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model requires preemptible, completion, terminal, preemptible, completion FIFO order",
                ));
            }
        },
        SystemsOperation::ReclaimKernelContext => {
            let (stack_index, context_identity, next_state) = match *state {
                BootstrapKernelContextState::TerminalCompleted => {
                    (2, 3, BootstrapKernelContextState::TerminalReclaimed)
                }
                BootstrapKernelContextState::PreemptibleCompleted => {
                    (0, 1, BootstrapKernelContextState::PreemptibleReclaimed)
                }
                BootstrapKernelContextState::CompletionCompleted => {
                    (0, 2, BootstrapKernelContextState::CompletionReclaimed)
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context reclaim requires one completed worker stack",
                    ));
                }
            };
            if context_stacks.len() <= stack_index {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context reclaim lost its owned worker stack",
                ));
            }
            let stack = context_stacks.remove(stack_index);
            storage.release(stack)?;
            *state = next_state;
            transitions.push(SystemsTransition::ReclaimKernelContext { context_identity });
        }
        SystemsOperation::ConsumeEmptyRunnableQueue => {
            if *state != BootstrapKernelContextState::CompletionReclaimed {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model consumes only an empty queue after all three reclamations",
                ));
            }
            *state = BootstrapKernelContextState::QueueConsumed;
            transitions.push(SystemsTransition::ConsumeEmptyRunnableQueue);
        }
        SystemsOperation::ConsumeKernelCompletion => {
            if *state != BootstrapKernelContextState::CompletionReclaimed {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-completion model consumes completion only after awakened worker reclamation",
                ));
            }
            transitions.push(SystemsTransition::ConsumeKernelCompletion {
                completion_identity: 1,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_deadline_transition(
    operation: &SystemsOperation,
    handler: &SystemsHandler,
    state: &mut BootstrapDeadlineState,
    next_event_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::ConstructDeadline {
            duration_nanoseconds,
        } => {
            *state = BootstrapDeadlineState::Constructed;
            transitions.push(SystemsTransition::ConstructDeadline {
                source_observation_identity: 2,
                duration_nanoseconds: *duration_nanoseconds,
            });
        }
        SystemsOperation::ArmDeadline => {
            let event_identity = *next_event_identity;
            *next_event_identity = event_identity.checked_add(1).ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline-event identity exhausted",
                )
            })?;
            *state = BootstrapDeadlineState::Armed;
            transitions.push(SystemsTransition::ArmDeadline { event_identity });
        }
        SystemsOperation::WaitDeadline => {
            let event_identity = (*next_event_identity).saturating_sub(1);
            transitions.extend([
                SystemsTransition::BeginDeadlineWait { event_identity },
                SystemsTransition::ObserveDeadline { event_identity },
                SystemsTransition::EnterDeadlineInterrupt { event_identity },
            ]);
            for handler_operation in &handler.operations {
                if !matches!(handler_operation, SystemsOperation::CompleteDeadline) {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "deadline handler contains an operation outside its completion protocol",
                    ));
                }
                transitions.push(SystemsTransition::CompleteDeadlineInterrupt { event_identity });
            }
            if handler.disposition != SystemsDisposition::Resume {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline handler must resume after completion",
                ));
            }
            transitions.extend([
                SystemsTransition::ResumeDeadlineInterrupt { event_identity },
                SystemsTransition::EndDeadlineWait { event_identity },
            ]);
            *state = BootstrapDeadlineState::Completed;
        }
        SystemsOperation::CompleteDeadline => {
            unreachable!("deadline completion is modeled inside the matching wait")
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_monotonic_clock_transition(
    operation: &SystemsOperation,
    next_observation_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    if !matches!(operation, SystemsOperation::ObserveMonotonicClock) {
        return Ok(false);
    }
    let observation_identity = *next_observation_identity;
    *next_observation_identity = observation_identity.checked_add(1).ok_or_else(|| {
        SystemsModelError::new(
            "E-SYSTEMS-MONOTONIC-CLOCK",
            "monotonic-clock observation identity exhausted",
        )
    })?;
    transitions.push(SystemsTransition::ObserveMonotonicClock {
        observation_identity,
    });
    Ok(true)
}

fn model_bootstrap_operation(
    operation: &SystemsOperation,
    debug_break_handler: &SystemsHandler,
    require_region_true: bool,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
    next_pci_transaction_identity: &mut u64,
    next_pci_region_transaction_identity: &mut u64,
    next_pci_update_transaction_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::DescribeBootMemory { .. } => {
            transitions.push(SystemsTransition::DescribeBootMemory);
        }
        SystemsOperation::CreateFrameAllocator { .. } => {
            transitions.push(SystemsTransition::CreateFrameAllocator);
        }
        SystemsOperation::AllocatePhysicalFrames { request, .. } => {
            transitions.push(SystemsTransition::AllocatePhysicalFrames { request: *request });
        }
        SystemsOperation::ReleasePhysicalFrames => {
            transitions.push(SystemsTransition::ReleasePhysicalFrames);
        }
        SystemsOperation::PrepareStaticImageConstruction { .. } => {
            transitions.push(SystemsTransition::PrepareStaticImageConstruction);
        }
        SystemsOperation::ZeroStaticImageConstructionPayload => {
            transitions.push(SystemsTransition::ZeroStaticImageConstructionPayload);
        }
        SystemsOperation::ZeroStaticImageStackBacking => {
            transitions.push(SystemsTransition::ZeroStaticImageStackBacking);
        }
        SystemsOperation::PublishStaticImageStackLeaf => {
            transitions.push(SystemsTransition::PublishStaticImageStackLeaf);
        }
        SystemsOperation::InitializeStaticImageStackEntry => {
            transitions.push(SystemsTransition::InitializeStaticImageStackEntry);
        }
        SystemsOperation::AddStaticImageStackPageSize => {
            transitions.push(SystemsTransition::AddStaticImageStackPageSize);
        }
        SystemsOperation::AddStaticImageStackEntryFact => {
            transitions.push(SystemsTransition::AddStaticImageStackEntryFact);
        }
        SystemsOperation::AddStaticImageStackProgramHeaders => {
            transitions.push(SystemsTransition::AddStaticImageStackProgramHeaders);
        }
        SystemsOperation::AddStaticImageStackExecutableName => {
            transitions.push(SystemsTransition::AddStaticImageStackExecutableName);
        }
        SystemsOperation::AddStaticImageStackCredentials => {
            transitions.push(SystemsTransition::AddStaticImageStackCredentials);
        }
        SystemsOperation::AddStaticImageStackSecureMode => {
            transitions.push(SystemsTransition::AddStaticImageStackSecureMode);
        }
        SystemsOperation::AddStaticImageStackClockTickRate => {
            transitions.push(SystemsTransition::AddStaticImageStackClockTickRate);
        }
        SystemsOperation::AddStaticImageStackPlatform => {
            transitions.push(SystemsTransition::AddStaticImageStackPlatform);
        }
        SystemsOperation::AddStaticImageStackHardwareCapabilities => {
            transitions.push(SystemsTransition::AddStaticImageStackHardwareCapabilities);
        }
        SystemsOperation::AddStaticImageStackLoadMetadata => {
            transitions.push(SystemsTransition::AddStaticImageStackLoadMetadata);
        }
        SystemsOperation::AuthorizeStaticImageStackRandomSlot => {
            transitions.push(SystemsTransition::AuthorizeStaticImageStackRandomSlot);
        }
        SystemsOperation::RetainStaticImagePreparedProcess => {
            transitions.push(SystemsTransition::RetainStaticImagePreparedProcess);
        }
        SystemsOperation::QualifyStaticImageRetainedDisjointWork => {
            transitions.push(SystemsTransition::QualifyStaticImageRetainedDisjointWork);
        }
        SystemsOperation::AuthorizeStaticImageEntropyHandoff => {
            transitions.push(SystemsTransition::AuthorizeStaticImageEntropyHandoff);
        }
        SystemsOperation::AuthorizeStaticImageEntropyAcquisition => {
            transitions.push(SystemsTransition::AuthorizeStaticImageEntropyAcquisition);
        }
        SystemsOperation::QualifyStaticImageEntropyDevice => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyDevice);
        }
        SystemsOperation::QualifyStaticImageEntropyTransport => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyTransport);
        }
        SystemsOperation::QualifyStaticImageEntropyNegotiation => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyNegotiation);
        }
        SystemsOperation::QualifyStaticImageEntropyQueue => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyQueue);
        }
        SystemsOperation::QualifyStaticImageEntropyBusMaster => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyBusMaster);
        }
        SystemsOperation::QualifyStaticImageEntropyDmaBuffers => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyDmaBuffers);
        }
        SystemsOperation::QualifyStaticImageEntropySplitRing => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropySplitRing);
        }
        SystemsOperation::QualifyStaticImageEntropyDmaPreparation => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyDmaPreparation);
        }
        SystemsOperation::QualifyStaticImageEntropyQueueAddresses => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyQueueAddresses);
        }
        SystemsOperation::QualifyStaticImageEntropyQueueActivation => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyQueueActivation);
        }
        SystemsOperation::QualifyStaticImageEntropyRequestCompletion => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyRequestCompletion);
        }
        SystemsOperation::QualifyStaticImageEntropySampleCapture => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropySampleCapture);
        }
        SystemsOperation::QualifyStaticImageEntropySampleQuality => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropySampleQuality);
        }
        SystemsOperation::QualifyStaticImageEntropyPoolLifecycle => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyPoolLifecycle);
        }
        SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyGeneratorLifecycle);
        }
        SystemsOperation::DeliverStaticImageEntropyInitialRandom => {
            transitions.push(SystemsTransition::DeliverStaticImageEntropyInitialRandom);
        }
        SystemsOperation::EnterStaticImageProcess => {
            transitions.push(SystemsTransition::EnterStaticImageProcess);
        }
        SystemsOperation::RestoreStaticImageProcessEntry => {
            transitions.push(SystemsTransition::RestoreStaticImageProcessEntry);
        }
        SystemsOperation::RevokeStaticImageEntropyInitialRandom => {
            transitions.push(SystemsTransition::RevokeStaticImageEntropyInitialRandom);
        }
        SystemsOperation::RevokeStaticImageEntropyAcquisition => {
            transitions.push(SystemsTransition::RevokeStaticImageEntropyAcquisition);
        }
        SystemsOperation::RevokeStaticImageEntropyHandoff => {
            transitions.push(SystemsTransition::RevokeStaticImageEntropyHandoff);
        }
        SystemsOperation::RestoreStaticImagePreparedProcess => {
            transitions.push(SystemsTransition::RestoreStaticImagePreparedProcess);
        }
        SystemsOperation::RevokeStaticImageStackRandomSlot => {
            transitions.push(SystemsTransition::RevokeStaticImageStackRandomSlot);
        }
        SystemsOperation::RemoveStaticImageStackLoadMetadata => {
            transitions.push(SystemsTransition::RemoveStaticImageStackLoadMetadata);
        }
        SystemsOperation::RemoveStaticImageStackHardwareCapabilities => {
            transitions.push(SystemsTransition::RemoveStaticImageStackHardwareCapabilities);
        }
        SystemsOperation::RemoveStaticImageStackPlatform => {
            transitions.push(SystemsTransition::RemoveStaticImageStackPlatform);
        }
        SystemsOperation::RemoveStaticImageStackClockTickRate => {
            transitions.push(SystemsTransition::RemoveStaticImageStackClockTickRate);
        }
        SystemsOperation::RemoveStaticImageStackSecureMode => {
            transitions.push(SystemsTransition::RemoveStaticImageStackSecureMode);
        }
        SystemsOperation::RemoveStaticImageStackCredentials => {
            transitions.push(SystemsTransition::RemoveStaticImageStackCredentials);
        }
        SystemsOperation::RemoveStaticImageStackExecutableName => {
            transitions.push(SystemsTransition::RemoveStaticImageStackExecutableName);
        }
        SystemsOperation::RemoveStaticImageStackProgramHeaders => {
            transitions.push(SystemsTransition::RemoveStaticImageStackProgramHeaders);
        }
        SystemsOperation::RemoveStaticImageStackEntryFact => {
            transitions.push(SystemsTransition::RemoveStaticImageStackEntryFact);
        }
        SystemsOperation::RemoveStaticImageStackPageSize => {
            transitions.push(SystemsTransition::RemoveStaticImageStackPageSize);
        }
        SystemsOperation::ClearStaticImageStackEntry => {
            transitions.push(SystemsTransition::ClearStaticImageStackEntry);
        }
        SystemsOperation::RevokeStaticImageStackLeaf => {
            transitions.push(SystemsTransition::RevokeStaticImageStackLeaf);
        }
        SystemsOperation::CopyStaticImageConstructionPayload => {
            transitions.push(SystemsTransition::CopyStaticImageConstructionPayload);
        }
        SystemsOperation::SealStaticImageConstructionPermissions => {
            transitions.push(SystemsTransition::SealStaticImageConstructionPermissions);
        }
        SystemsOperation::AttachStaticImageAddressSpace => {
            transitions.push(SystemsTransition::AttachStaticImageAddressSpace);
        }
        SystemsOperation::ActivateStaticImageAddressSpace => {
            transitions.push(SystemsTransition::ActivateStaticImageAddressSpace);
        }
        SystemsOperation::RestoreStaticImageAddressSpace => {
            transitions.push(SystemsTransition::RestoreStaticImageAddressSpace);
        }
        SystemsOperation::AuthorizeStaticImageEntry => {
            transitions.push(SystemsTransition::AuthorizeStaticImageEntry);
        }
        SystemsOperation::AuthorizeStaticImageStackSlot => {
            transitions.push(SystemsTransition::AuthorizeStaticImageStackSlot);
        }
        SystemsOperation::RevokeStaticImageStackSlot => {
            transitions.push(SystemsTransition::RevokeStaticImageStackSlot);
        }
        SystemsOperation::RevokeStaticImageEntry => {
            transitions.push(SystemsTransition::RevokeStaticImageEntry);
        }
        SystemsOperation::DetachStaticImageAddressSpace => {
            transitions.push(SystemsTransition::DetachStaticImageAddressSpace);
        }
        SystemsOperation::ReclaimStaticImageConstructionHierarchy => {
            transitions.push(SystemsTransition::ReclaimStaticImageConstructionHierarchy);
        }
        SystemsOperation::RevokeStaticImageConstructionMetadata => {
            transitions.push(SystemsTransition::RevokeStaticImageConstructionMetadata);
        }
        SystemsOperation::ReclaimStaticImageConstructionPayload => {
            transitions.push(SystemsTransition::ReclaimStaticImageConstructionPayload);
        }
        SystemsOperation::MapKernelFrames { request, .. } => {
            transitions.push(SystemsTransition::MapKernelFrames { request: *request });
        }
        SystemsOperation::KernelMappingStoreByte {
            offset_bytes,
            value,
        } => transitions.push(SystemsTransition::StoreKernelMappingByte {
            offset_bytes: *offset_bytes,
            value: *value,
        }),
        SystemsOperation::KernelMappingLoadByteEquals {
            offset_bytes,
            expected,
            ..
        } => transitions.push(SystemsTransition::LoadKernelMappingByte {
            offset_bytes: *offset_bytes,
            value: *expected,
        }),
        SystemsOperation::UnmapKernelFrames => {
            transitions.push(SystemsTransition::UnmapKernelFrames);
        }
        SystemsOperation::BeginTranslationUpdate { request, .. } => {
            transitions.push(SystemsTransition::BeginTranslationUpdate { request: *request });
        }
        SystemsOperation::CommitTranslationUpdate { .. } => {
            transitions.push(SystemsTransition::CommitTranslationUpdate);
        }
        SystemsOperation::ActivateTranslationSpace { .. } => {
            transitions.push(SystemsTransition::ActivateTranslationSpace);
        }
        SystemsOperation::BeginTranslationEdit { kind, .. } => {
            transitions.push(SystemsTransition::BeginTranslationEdit { kind: *kind });
        }
        SystemsOperation::MapTranslationFrames { request, .. } => {
            transitions.push(SystemsTransition::MapTranslationFrames { request: *request });
        }
        SystemsOperation::UnmapTranslationMapping => {
            transitions.push(SystemsTransition::UnmapTranslationMapping);
        }
        SystemsOperation::CommitTranslationEdit { kind, .. } => {
            transitions.push(SystemsTransition::CommitTranslationEdit { kind: *kind });
        }
        SystemsOperation::ConsoleWrite { text } => {
            transitions.push(SystemsTransition::ConsoleWrite { text: text.clone() });
        }
        SystemsOperation::ObservePciConfigurationDword { request } => {
            let transaction_identity = *next_pci_transaction_identity;
            *next_pci_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG",
                        "PCI configuration transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::ObservePciConfigurationDword {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::UpdatePciConfigurationWord { request } => {
            let transaction_identity = *next_pci_update_transaction_identity;
            *next_pci_update_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG-UPDATE",
                        "PCI configuration update transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::UpdatePciConfigurationWord {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::CreateDmaBuffer { request } => {
            transitions.push(SystemsTransition::CreateDmaBuffer { request: *request });
        }
        SystemsOperation::BindDmaBufferReference { request } => {
            transitions.push(SystemsTransition::BindDmaBufferReference { request: *request });
        }
        SystemsOperation::StoreDmaBufferInteger { request } => {
            transitions.push(SystemsTransition::StoreDmaBufferInteger { request: *request });
        }
        SystemsOperation::PrepareDmaBufferForDevice { request } => {
            transitions.push(SystemsTransition::PrepareDmaBufferForDevice { request: *request });
        }
        SystemsOperation::CompleteDmaBufferForCpu { request } => {
            transitions.push(SystemsTransition::CompleteDmaBufferForCpu { request: *request });
        }
        SystemsOperation::LoadDmaBufferIntegerEquals { request } => {
            transitions.push(SystemsTransition::LoadDmaBufferIntegerEquals { request: *request });
        }
        SystemsOperation::CaptureEntropySample { request } => {
            transitions.push(SystemsTransition::CaptureEntropySample { request: *request });
        }
        SystemsOperation::AssessEntropySampleQuality { request } => {
            transitions.push(SystemsTransition::AssessEntropySampleQuality { request: *request });
        }
        SystemsOperation::CreateEntropyPool { request } => {
            transitions.push(SystemsTransition::CreateEntropyPool { request: *request });
        }
        SystemsOperation::MixEntropyPoolSample { request } => {
            transitions.push(SystemsTransition::MixEntropyPoolSample { request: *request });
        }
        SystemsOperation::CreditEntropyPoolContribution { request } => {
            transitions
                .push(SystemsTransition::CreditEntropyPoolContribution { request: *request });
        }
        SystemsOperation::InitializeEntropyGenerator { request } => {
            transitions.push(SystemsTransition::InitializeEntropyGenerator { request: *request });
        }
        SystemsOperation::ExtractEntropyGenerator { request } => {
            transitions.push(SystemsTransition::ExtractEntropyGenerator { request: *request });
        }
        SystemsOperation::DeliverInitialAuxvRandom { request } => {
            transitions.push(SystemsTransition::DeliverInitialAuxvRandom { request: *request });
        }
        SystemsOperation::ReserveInitialUserRandom { request } => {
            transitions.push(SystemsTransition::ReserveInitialUserRandom { request: *request });
        }
        SystemsOperation::BindReusableUserRandom { request } => {
            transitions.push(SystemsTransition::BindReusableUserRandom { request: *request });
        }
        SystemsOperation::ExpandSmallUserRandomRequests { request } => {
            transitions
                .push(SystemsTransition::ExpandSmallUserRandomRequests { request: *request });
        }
        SystemsOperation::ExpandLargeUserRandomRequests { request } => {
            transitions
                .push(SystemsTransition::ExpandLargeUserRandomRequests { request: *request });
        }
        SystemsOperation::PublishPciRegionDmaReference { request } => {
            let transaction_identity = *next_pci_region_transaction_identity;
            *next_pci_region_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-DMA-PUBLISH",
                        "PCI-region DMA-reference transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::PublishPciRegionDmaReference {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::AccessPciRegionRegister { request } => {
            let transaction_identity = *next_pci_region_transaction_identity;
            *next_pci_region_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-PCI-REGION",
                        "PCI region transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::AccessPciRegionRegister {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::DebugBreak => {
            return Ok(model_debug_break(debug_break_handler, transitions));
        }
        _ => {
            return model_bootstrap_storage_operation(
                operation,
                require_region_true,
                storage,
                region,
                atomic,
                transitions,
            );
        }
    }
    Ok(false)
}

fn model_local_notification_transition(
    operation: &SystemsOperation,
    handler: &SystemsHandler,
    protocol: &mut LocalNotificationProtocol,
    pending: &mut Option<u64>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::SendLocalNotification => {
            let event_identity = protocol.send(LocalInterruptMaskState::Disabled)?;
            if pending.replace(event_identity).is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification model encountered a second pending event",
                ));
            }
            transitions.push(SystemsTransition::SendLocalNotification { event_identity });
        }
        SystemsOperation::WaitLocalNotification => {
            let event_identity = pending.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification model wait lost its pending event",
                )
            })?;
            transitions.push(SystemsTransition::BeginLocalNotificationWait { event_identity });
            protocol.enter(event_identity)?;
            transitions.push(SystemsTransition::ObserveLocalNotification { event_identity });
            transitions.push(SystemsTransition::EnterLocalNotificationInterrupt { event_identity });
            for handler_operation in &handler.operations {
                if !matches!(
                    handler_operation,
                    SystemsOperation::CompleteLocalNotification
                ) {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-LOCAL-INTERRUPT",
                        "local-notification model encountered an unsupported handler operation",
                    ));
                }
                protocol.complete(event_identity)?;
                transitions
                    .push(SystemsTransition::CompleteLocalNotificationInterrupt { event_identity });
            }
            if handler.disposition != SystemsDisposition::Resume {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification model requires resume after completion",
                ));
            }
            protocol.resume(event_identity)?;
            transitions
                .push(SystemsTransition::ResumeLocalNotificationInterrupt { event_identity });
            let restored = protocol.finish_wait(event_identity)?;
            if restored != LocalInterruptMaskState::Disabled {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification wait did not restore the sealed initial mask state",
                ));
            }
            transitions.push(SystemsTransition::EndLocalNotificationWait { event_identity });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_bootstrap_disposition(
    disposition: &SystemsDisposition,
    transitions: &mut Vec<SystemsTransition>,
) {
    if let SystemsDisposition::Fatal { message } = disposition {
        transitions.push(SystemsTransition::Fatal {
            message: message.clone(),
        });
    }
}

fn initial_systems_transitions(program: &SystemsProgram) -> Vec<SystemsTransition> {
    vec![
        SystemsTransition::ProvisionBootstrapStorage {
            capacity_bytes: program.bootstrap_storage.capacity_bytes,
            alignment_bytes: program.bootstrap_storage.alignment_bytes,
        },
        SystemsTransition::EnterBootstrap,
    ]
}

fn model_critical_transition(
    operation: &SystemsOperation,
    critical_stack: &mut Vec<(CriticalDomain, u64)>,
    next_nesting_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::EnterCritical { domain, .. } => {
            let nesting_identity = *next_nesting_identity;
            *next_nesting_identity += 1;
            critical_stack.push((*domain, nesting_identity));
            transitions.push(SystemsTransition::EnterCritical {
                domain: *domain,
                nesting_identity,
            });
        }
        SystemsOperation::RestoreCritical { domain } => {
            let Some((entered_domain, nesting_identity)) = critical_stack.pop() else {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CRITICAL",
                    "validated critical restoration lost its matching entry",
                ));
            };
            if entered_domain != *domain {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CRITICAL",
                    "validated critical restoration changed its domain",
                ));
            }
            transitions.push(SystemsTransition::RestoreCritical {
                domain: *domain,
                nesting_identity,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_debug_break(handler: &SystemsHandler, transitions: &mut Vec<SystemsTransition>) -> bool {
    transitions.push(SystemsTransition::ObserveDebugBreak);
    transitions.push(SystemsTransition::EnterDebugBreak);
    for operation in &handler.operations {
        if let SystemsOperation::ConsoleWrite { text } = operation {
            transitions.push(SystemsTransition::ConsoleWrite { text: text.clone() });
        }
    }
    match &handler.disposition {
        SystemsDisposition::Resume => {
            transitions.push(SystemsTransition::ResumeDebugBreak);
            false
        }
        SystemsDisposition::Fatal { message } => {
            transitions.push(SystemsTransition::Fatal {
                message: message.clone(),
            });
            true
        }
        SystemsDisposition::RetireKernelContextToCaller => {
            unreachable!("debug-break validation rejects kernel-thread retirement")
        }
        SystemsDisposition::PreemptKernelContext => {
            unreachable!("debug-break validation rejects kernel-context preemption")
        }
        SystemsDisposition::DispatchInitialUserSyscall { .. } => {
            unreachable!("debug-break validation rejects user-syscall dispatch")
        }
    }
}
