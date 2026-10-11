//! Architecture-independent affine user-execution and syscall-entry model.

use crate::{PhysicalFrameRequest, SystemsModelError};

pub const SYSTEMS_USER_CONTEXT_PREPARE: &str = "topal.systems.context.user.probe.prepare/1";
pub const SYSTEMS_USER_ADDRESS_SPACE_PREPARE: &str =
    "topal.systems.memory.user-address-space.prepare/1";
pub const SYSTEMS_USER_ADDRESS_SPACE_RECLAIM: &str =
    "topal.systems.memory.user-address-space.reclaim/1";
pub const SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING: &str =
    "topal.systems.memory.user-address-space.verify-mapping/1";
pub const SYSTEMS_USER_ADDRESS_SPACE_MAP: &str = "topal.systems.memory.user-address-space.map/1";
pub const SYSTEMS_USER_MAPPING_STORE_BYTE: &str = "topal.systems.memory.user-mapping.store-byte/1";
pub const SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY: &str =
    "topal.systems.memory.user-mapping.protect-read-only/1";
pub const SYSTEMS_USER_MAPPING_LOAD_BYTE: &str = "topal.systems.memory.user-mapping.load-byte/1";
pub const SYSTEMS_USER_MAPPING_PERMIT_EXECUTE: &str =
    "topal.systems.memory.user-mapping.permit-execute/1";
pub const SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA: &str =
    "topal.systems.memory.user-mapping.seal-immutable-data/1";
pub const SYSTEMS_USER_ADDRESS_SPACE_UNMAP: &str =
    "topal.systems.memory.user-address-space.unmap/1";
pub const SYSTEMS_USER_CONTEXT_ENTER: &str = "topal.systems.context.user.enter/1";
pub const SYSTEMS_USER_SYSCALL_ENTRY: &str = "topal.systems.entry.user-syscall/1";
pub const SYSTEMS_USER_SYSCALL_DISPATCH: &str =
    "topal.systems.disposition.user-syscall.dispatch-initial/1";
pub const SYSTEMS_USER_SYSCALL_RETURN: &str = "topal.systems.disposition.user-syscall.return/1";
pub const SYSTEMS_USER_TERMINATE: &str = "topal.systems.disposition.user-syscall.terminate/1";
pub const SYSTEMS_USER_CONTEXT_COMPLETE: &str = "topal.systems.context.user.complete/1";
pub const SYSTEMS_USER_TRANSFER_BEGIN: &str = "topal.systems.user-transfer.begin/1";
pub const SYSTEMS_USER_COPY_TO: &str = "topal.systems.user-transfer.copy-to/1";
pub const SYSTEMS_USER_COPY_FROM: &str = "topal.systems.user-transfer.copy-from/1";
pub const SYSTEMS_USER_TRANSFER_END: &str = "topal.systems.user-transfer.end/1";

pub const INITIAL_USER_ADDRESS_SPACE_IDENTITY: &str = "topal.systems.address-space.initial-user/1";
pub const INITIAL_USER_THREAD_IDENTITY: &str = "topal.systems.thread.initial-user/1";
pub const LINUX_X86_64_ABI_IDENTITY: &str = "topal.systems.abi.linux-x86_64-7.2.9/1";
pub const LINUX_X86_64_GETPID: u64 = 39;
pub const LINUX_X86_64_NANOSLEEP: u64 = 35;
pub const LINUX_X86_64_UNAME: u64 = 63;
pub const LINUX_X86_64_EXIT_GROUP: u64 = 231;
pub const LINUX_X86_64_EFAULT_RESULT: u64 = u64::MAX - 13;
pub const INITIAL_USER_UNAME_VALID_CANDIDATE: u64 = 0x0060_1e00;
pub const INITIAL_USER_UNAME_FAULT_CANDIDATE: u64 = 0x0060_1f00;
pub const INITIAL_USER_TIMESPEC_VALID_CANDIDATE: u64 = 0x0060_1ff0;
pub const INITIAL_USER_TIMESPEC_FAULT_CANDIDATE: u64 = 0x0060_1ff8;
pub const LINUX_TIMESPEC_BYTES: u64 = 16;
pub const LINUX_NEW_UTSNAME_FIELD_BYTES: u64 = 65;
pub const LINUX_NEW_UTSNAME_BYTES: u64 = 6 * LINUX_NEW_UTSNAME_FIELD_BYTES;

const INITIAL_USER_BACKING_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 3,
    alignment_frames: 1,
};

const INITIAL_USER_MAPPING_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 1,
    alignment_frames: 1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UserProbeStage {
    AwaitingGetpid,
    ReturnedFromGetpid,
    ReturnedFromValidUname,
    ReturnedFromFaultingUname,
    ReturnedFromValidNanosleep,
    ReturnedFromFaultingNanosleep,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ValidatedUserState {
    provider_identity: String,
    address_space_identity: u64,
    abi_identity: String,
    probe_identity: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PreparedUserAddressSpace {
    provider_identity: String,
    address_space_identity: u64,
    backing: PhysicalFrameRequest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UserMappingStage {
    Live,
    Stored,
    Protected,
    Verified,
    Executable,
    ImmutableData,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UserMapping {
    provider_identity: String,
    address_space_identity: u64,
    mapping_identity: u64,
    backing: PhysicalFrameRequest,
    stage: UserMappingStage,
}

impl UserMapping {
    /// Return the opaque identity used only for semantic trace correlation.
    #[must_use]
    pub const fn identity(&self) -> u64 {
        self.mapping_identity
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RevokedUserAddressSpace {
    provider_identity: String,
    address_space_identity: u64,
    backing: PhysicalFrameRequest,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SuspendedUserContext {
    validated: ValidatedUserState,
    address_space: PreparedUserAddressSpace,
    processor_identity: String,
    thread_identity: String,
    process_identity: u64,
    stage: UserProbeStage,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ActiveUserExecution {
    suspended: SuspendedUserContext,
    kernel_continuation_identity: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UserSyscallContext {
    active: ActiveUserExecution,
    entry_identity: u64,
    number: u64,
    arguments: [u64; 6],
}

#[derive(Debug, Eq, PartialEq)]
pub struct UserTransferScope {
    entry: UserSyscallContext,
    transfer_identity: u64,
    candidate: u64,
    extent: u64,
    direction: UserTransferDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UserTransferDirection {
    ToUser,
    FromUser,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UserTransferOutcome {
    Complete {
        scope: UserTransferScope,
        copied: u64,
    },
    Fault {
        scope: UserTransferScope,
        copied_prefix: u64,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum UserInputTransferOutcome {
    Complete {
        scope: UserTransferScope,
        copied: u64,
        value: LinuxTimespec,
    },
    Fault {
        scope: UserTransferScope,
        copied_prefix: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinuxTimespec {
    pub seconds: i64,
    pub nanoseconds: i64,
}

impl LinuxTimespec {
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            seconds: 0,
            nanoseconds: 0,
        }
    }

    fn validate(self) -> Result<(), SystemsModelError> {
        if self.seconds < 0 || !(0..1_000_000_000).contains(&self.nanoseconds) {
            return Err(user_transfer_error(
                "native Linux timespec requires nonnegative seconds and nanoseconds below one billion",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxUtsName {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String,
    pub domainname: String,
}

impl LinuxUtsName {
    #[must_use]
    pub fn initial() -> Self {
        Self {
            sysname: "Linux".into(),
            nodename: "topal".into(),
            release: "7.2.9-topal".into(),
            version: "#1 Topal".into(),
            machine: "x86_64".into(),
            domainname: "(none)".into(),
        }
    }

    fn validate(&self) -> Result<(), SystemsModelError> {
        if self != &Self::initial()
            || [
                &self.sysname,
                &self.nodename,
                &self.release,
                &self.version,
                &self.machine,
                &self.domainname,
            ]
            .iter()
            .any(|field| field.len() >= LINUX_NEW_UTSNAME_FIELD_BYTES as usize)
        {
            return Err(user_transfer_error(
                "initial uname transfer requires the exact source-owned NUL-terminated field values",
            ));
        }
        Ok(())
    }
}

impl UserSyscallContext {
    #[must_use]
    pub fn number(&self) -> u64 {
        self.number
    }

    #[must_use]
    pub fn arguments(&self) -> [u64; 6] {
        self.arguments
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompletedUserExecution {
    provider_identity: String,
    processor_identity: String,
    address_space: PreparedUserAddressSpace,
    thread_identity: String,
    abi_identity: String,
    process_identity: u64,
    kernel_continuation_identity: u64,
    terminal_entry_identity: u64,
    status: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UserExecutionCompletion {
    pub process_identity: u64,
    pub kernel_continuation_identity: u64,
    pub terminal_entry_identity: u64,
    pub status: u64,
    pub revoked_address_space: RevokedUserAddressSpace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsUserExecutionProtocol {
    provider_identity: String,
    processor_identity: String,
    thread_identity: String,
    abi_identity: String,
    next_probe_identity: u64,
    next_address_space_identity: u64,
    next_mapping_identity: u64,
    next_kernel_continuation_identity: u64,
    next_entry_identity: u64,
    next_transfer_identity: u64,
}

impl SystemsUserExecutionProtocol {
    #[must_use]
    pub fn initial_linux_x86_64() -> Self {
        Self {
            provider_identity: "topal.systems.user-provider.initial/1".into(),
            processor_identity: crate::INITIAL_PROCESSOR_IDENTITY.into(),
            thread_identity: INITIAL_USER_THREAD_IDENTITY.into(),
            abi_identity: LINUX_X86_64_ABI_IDENTITY.into(),
            next_probe_identity: 1,
            next_address_space_identity: 1,
            next_mapping_identity: 1,
            next_kernel_continuation_identity: 1,
            next_entry_identity: 1,
            next_transfer_identity: 1,
        }
    }

    /// Prepare one opaque qualification-only user address space.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless backing has the sealed three-frame
    /// shape.
    pub fn prepare_address_space(
        &mut self,
        backing: PhysicalFrameRequest,
    ) -> Result<PreparedUserAddressSpace, SystemsModelError> {
        if backing != INITIAL_USER_BACKING_REQUEST {
            return Err(user_error(
                "the initial user address space requires exact three-frame backing",
            ));
        }
        let address_space_identity = self.next_address_space_identity;
        self.next_address_space_identity = address_space_identity
            .checked_add(1)
            .ok_or_else(|| user_error("user-address-space identity exhausted"))?;
        Ok(PreparedUserAddressSpace {
            provider_identity: self.provider_identity.clone(),
            address_space_identity,
            backing,
        })
    }

    /// Consume one disjoint frame into the sealed user mapping.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for the wrong provider, backing shape, or
    /// exhausted mapping identity.
    pub fn map_user_frame(
        &mut self,
        address_space: &PreparedUserAddressSpace,
        backing: PhysicalFrameRequest,
    ) -> Result<UserMapping, SystemsModelError> {
        if address_space.provider_identity != self.provider_identity
            || backing != INITIAL_USER_MAPPING_REQUEST
        {
            return Err(user_error(
                "the initial user mapping requires its prepared address space and exact one-frame backing",
            ));
        }
        let mapping_identity = self.next_mapping_identity;
        self.next_mapping_identity = mapping_identity
            .checked_add(1)
            .ok_or_else(|| user_error("user-mapping identity exhausted"))?;
        Ok(UserMapping {
            provider_identity: self.provider_identity.clone(),
            address_space_identity: address_space.address_space_identity,
            mapping_identity,
            backing,
            stage: UserMappingStage::Live,
        })
    }

    /// Consume a live mapping into its fixed-byte stored refinement.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless the mapping is the exact live
    /// provider-owned refinement.
    pub fn store_user_mapping_byte(
        &self,
        mut mapping: UserMapping,
    ) -> Result<UserMapping, SystemsModelError> {
        self.validate_user_mapping(&mapping, UserMappingStage::Live)?;
        mapping.stage = UserMappingStage::Stored;
        Ok(mapping)
    }

    /// Consume a stored writable mapping into its read-only refinement.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless the mapping is the exact stored
    /// provider-owned refinement.
    pub fn protect_user_mapping_read_only(
        &self,
        mut mapping: UserMapping,
    ) -> Result<UserMapping, SystemsModelError> {
        self.validate_user_mapping(&mapping, UserMappingStage::Stored)?;
        mapping.stage = UserMappingStage::Protected;
        Ok(mapping)
    }

    /// Consume a protected mapping into its fixed-byte verified refinement.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless the mapping is the exact protected
    /// provider-owned refinement.
    pub fn load_user_mapping_byte(
        &self,
        mut mapping: UserMapping,
    ) -> Result<UserMapping, SystemsModelError> {
        self.validate_user_mapping(&mapping, UserMappingStage::Protected)?;
        mapping.stage = UserMappingStage::Verified;
        Ok(mapping)
    }

    /// Consume a verified read-only mapping into its executable refinement.
    pub fn permit_user_mapping_execute(
        &self,
        mut mapping: UserMapping,
    ) -> Result<UserMapping, SystemsModelError> {
        self.validate_user_mapping(&mapping, UserMappingStage::Verified)?;
        if mapping.mapping_identity != 1 {
            return Err(user_error(
                "only the first bounded mapping generation has executable-code role",
            ));
        }
        mapping.stage = UserMappingStage::Executable;
        Ok(mapping)
    }

    /// Consume the second verified mapping into its immutable-data refinement.
    pub fn seal_user_mapping_immutable_data(
        &self,
        mut mapping: UserMapping,
    ) -> Result<UserMapping, SystemsModelError> {
        self.validate_user_mapping(&mapping, UserMappingStage::Verified)?;
        if mapping.mapping_identity != 2 {
            return Err(user_error(
                "only the second bounded mapping generation has immutable-data role",
            ));
        }
        mapping.stage = UserMappingStage::ImmutableData;
        Ok(mapping)
    }

    /// Consume a verified mapping and return its exact original frame extent.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a wrong address space, provider, or
    /// mapping refinement.
    pub fn unmap_user_frame(
        &self,
        address_space: &PreparedUserAddressSpace,
        mapping: UserMapping,
    ) -> Result<PhysicalFrameRequest, SystemsModelError> {
        if mapping.stage != UserMappingStage::Executable
            && mapping.stage != UserMappingStage::ImmutableData
        {
            return Err(user_error(
                "user unmap requires the exact role-classified affine refinement",
            ));
        }
        if mapping.provider_identity != self.provider_identity
            || mapping.backing != INITIAL_USER_MAPPING_REQUEST
        {
            return Err(user_error(
                "user mapping operation requires the exact preceding affine refinement",
            ));
        }
        if address_space.provider_identity != self.provider_identity
            || mapping.address_space_identity != address_space.address_space_identity
        {
            return Err(user_error(
                "user unmap requires the mapping's exact prepared address space",
            ));
        }
        Ok(mapping.backing)
    }

    fn validate_user_mapping(
        &self,
        mapping: &UserMapping,
        expected: UserMappingStage,
    ) -> Result<(), SystemsModelError> {
        if mapping.provider_identity != self.provider_identity
            || mapping.backing != INITIAL_USER_MAPPING_REQUEST
            || mapping.stage != expected
        {
            return Err(user_error(
                "user mapping operation requires the exact preceding affine refinement",
            ));
        }
        Ok(())
    }

    /// Create one validated qualification-only user continuation.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless the initial source-owned PID is one.
    pub fn prepare_probe(
        &mut self,
        process_identity: u64,
        address_space: PreparedUserAddressSpace,
    ) -> Result<SuspendedUserContext, SystemsModelError> {
        if process_identity != 1 || address_space.provider_identity != self.provider_identity {
            return Err(user_error(
                "the initial user probe requires source-owned process identity one and its prepared address space",
            ));
        }
        let probe_identity = self.next_probe_identity;
        self.next_probe_identity = probe_identity
            .checked_add(1)
            .ok_or_else(|| user_error("user-probe identity exhausted"))?;
        Ok(SuspendedUserContext {
            validated: ValidatedUserState {
                provider_identity: self.provider_identity.clone(),
                address_space_identity: address_space.address_space_identity,
                abi_identity: self.abi_identity.clone(),
                probe_identity,
            },
            address_space,
            processor_identity: self.processor_identity.clone(),
            thread_identity: self.thread_identity.clone(),
            process_identity,
            stage: UserProbeStage::AwaitingGetpid,
        })
    }

    /// Register the exact kernel continuation and enter user execution.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a context from another provider,
    /// processor, address space, thread, or ABI.
    pub fn enter(
        &mut self,
        suspended: SuspendedUserContext,
    ) -> Result<ActiveUserExecution, SystemsModelError> {
        self.validate_suspended(&suspended)?;
        let kernel_continuation_identity = self.next_kernel_continuation_identity;
        self.next_kernel_continuation_identity = kernel_continuation_identity
            .checked_add(1)
            .ok_or_else(|| user_error("user kernel-continuation identity exhausted"))?;
        Ok(ActiveUserExecution {
            suspended,
            kernel_continuation_identity,
        })
    }

    /// Capture one ABI request as an affine typed syscall entry.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless the request is the next operation in
    /// the sealed getpid/exit_group qualification protocol.
    pub fn enter_syscall(
        &mut self,
        active: ActiveUserExecution,
        number: u64,
        arguments: [u64; 6],
    ) -> Result<UserSyscallContext, SystemsModelError> {
        self.validate_suspended(&active.suspended)?;
        let valid = match active.suspended.stage {
            UserProbeStage::AwaitingGetpid => number == LINUX_X86_64_GETPID && arguments == [0; 6],
            UserProbeStage::ReturnedFromGetpid => {
                number == LINUX_X86_64_UNAME
                    && arguments[0] == INITIAL_USER_UNAME_VALID_CANDIDATE
                    && arguments[1..] == [0; 5]
            }
            UserProbeStage::ReturnedFromValidUname => {
                number == LINUX_X86_64_UNAME
                    && arguments[0] == INITIAL_USER_UNAME_FAULT_CANDIDATE
                    && arguments[1..] == [0; 5]
            }
            UserProbeStage::ReturnedFromFaultingUname => {
                number == LINUX_X86_64_NANOSLEEP
                    && arguments[0] == INITIAL_USER_TIMESPEC_VALID_CANDIDATE
                    && arguments[1..] == [0; 5]
            }
            UserProbeStage::ReturnedFromValidNanosleep => {
                number == LINUX_X86_64_NANOSLEEP
                    && arguments[0] == INITIAL_USER_TIMESPEC_FAULT_CANDIDATE
                    && arguments[1..] == [0; 5]
            }
            UserProbeStage::ReturnedFromFaultingNanosleep => {
                number == LINUX_X86_64_EXIT_GROUP && arguments[0] == 0 && arguments[1..] == [0; 5]
            }
        };
        if !valid {
            return Err(user_error(
                "user syscall does not match the sealed getpid, uname success/fault, nanosleep success/fault, then exit_group protocol",
            ));
        }
        let entry_identity = self.next_entry_identity;
        self.next_entry_identity = entry_identity
            .checked_add(1)
            .ok_or_else(|| user_error("user-syscall entry identity exhausted"))?;
        Ok(UserSyscallContext {
            active,
            entry_identity,
            number,
            arguments,
        })
    }

    /// Consume one uname entry into its exact affine user-transfer scope.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for the wrong syscall, candidate, extent,
    /// identity, or overflowed user range.
    pub fn begin_uname_transfer(
        &mut self,
        entry: UserSyscallContext,
    ) -> Result<UserTransferScope, SystemsModelError> {
        self.validate_entry(&entry)?;
        if entry.number != LINUX_X86_64_UNAME
            || !matches!(
                entry.active.suspended.stage,
                UserProbeStage::ReturnedFromGetpid | UserProbeStage::ReturnedFromValidUname
            )
        {
            return Err(user_transfer_error(
                "user transfer requires the exact live uname syscall entry",
            ));
        }
        let candidate = entry.arguments[0];
        let extent = LINUX_NEW_UTSNAME_BYTES;
        candidate.checked_add(extent - 1).ok_or_else(|| {
            user_transfer_error("user transfer candidate range must not overflow")
        })?;
        let transfer_identity = self.next_transfer_identity;
        self.next_transfer_identity = transfer_identity
            .checked_add(1)
            .ok_or_else(|| user_transfer_error("user-transfer identity exhausted"))?;
        Ok(UserTransferScope {
            entry,
            transfer_identity,
            candidate,
            extent,
            direction: UserTransferDirection::ToUser,
        })
    }

    /// Consume one nanosleep entry into an input-direction transfer scope.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for the wrong syscall, candidate, extent,
    /// identity, remainder candidate, or overflowed user range.
    pub fn begin_nanosleep_transfer(
        &mut self,
        entry: UserSyscallContext,
    ) -> Result<UserTransferScope, SystemsModelError> {
        self.validate_entry(&entry)?;
        if entry.number != LINUX_X86_64_NANOSLEEP
            || entry.arguments[1] != 0
            || !matches!(
                entry.active.suspended.stage,
                UserProbeStage::ReturnedFromFaultingUname
                    | UserProbeStage::ReturnedFromValidNanosleep
            )
        {
            return Err(user_transfer_error(
                "user input transfer requires the exact live nanosleep syscall entry and null remainder candidate",
            ));
        }
        let candidate = entry.arguments[0];
        let extent = LINUX_TIMESPEC_BYTES;
        candidate.checked_add(extent - 1).ok_or_else(|| {
            user_transfer_error("user input transfer candidate range must not overflow")
        })?;
        let transfer_identity = self.next_transfer_identity;
        self.next_transfer_identity = transfer_identity
            .checked_add(1)
            .ok_or_else(|| user_transfer_error("user-transfer identity exhausted"))?;
        Ok(UserTransferScope {
            entry,
            transfer_identity,
            candidate,
            extent,
            direction: UserTransferDirection::FromUser,
        })
    }

    /// Copy the source-owned uname bytes through one fault-contained scope.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a wrong scope, source value, or
    /// provider-reported writable prefix.
    pub fn copy_initial_utsname_to_user(
        &self,
        scope: UserTransferScope,
        value: &LinuxUtsName,
        writable_prefix: u64,
    ) -> Result<UserTransferOutcome, SystemsModelError> {
        self.validate_transfer_scope(
            &scope,
            UserTransferDirection::ToUser,
            LINUX_X86_64_UNAME,
            LINUX_NEW_UTSNAME_BYTES,
        )?;
        value.validate()?;
        if writable_prefix >= scope.extent {
            return Ok(UserTransferOutcome::Complete {
                copied: scope.extent,
                scope,
            });
        }
        if scope.candidate != INITIAL_USER_UNAME_FAULT_CANDIDATE || writable_prefix != 256 {
            return Err(user_transfer_error(
                "initial uname fault requires the exact guard-page prefix",
            ));
        }
        Ok(UserTransferOutcome::Fault {
            scope,
            copied_prefix: writable_prefix,
        })
    }

    /// Copy and validate one native timespec without exposing partial staging.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a wrong scope, decoded value, or
    /// provider-reported readable prefix.
    pub fn copy_initial_timespec_from_user(
        &self,
        scope: UserTransferScope,
        decoded: Option<LinuxTimespec>,
        readable_prefix: u64,
    ) -> Result<UserInputTransferOutcome, SystemsModelError> {
        self.validate_transfer_scope(
            &scope,
            UserTransferDirection::FromUser,
            LINUX_X86_64_NANOSLEEP,
            LINUX_TIMESPEC_BYTES,
        )?;
        if readable_prefix >= scope.extent {
            let value = decoded.ok_or_else(|| {
                user_transfer_error("complete timespec transfer requires one decoded value")
            })?;
            value.validate()?;
            if value != LinuxTimespec::zero() {
                return Err(user_transfer_error(
                    "initial nanosleep requires the sealed zero-duration timespec",
                ));
            }
            return Ok(UserInputTransferOutcome::Complete {
                copied: scope.extent,
                value,
                scope,
            });
        }
        if decoded.is_some()
            || scope.candidate != INITIAL_USER_TIMESPEC_FAULT_CANDIDATE
            || readable_prefix != 8
        {
            return Err(user_transfer_error(
                "faulting timespec transfer exposes no value and requires the exact guard-page prefix",
            ));
        }
        Ok(UserInputTransferOutcome::Fault {
            scope,
            copied_prefix: readable_prefix,
        })
    }

    /// Reunite the refined transfer scope with its syscall entry and return
    /// the exact Linux result to user execution.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for an outcome/result mismatch or wrong
    /// probe stage.
    pub fn return_uname(
        &self,
        outcome: UserTransferOutcome,
        result: u64,
    ) -> Result<ActiveUserExecution, SystemsModelError> {
        let (mut scope, required_result, complete) = match outcome {
            UserTransferOutcome::Complete { scope, copied } => {
                if copied != LINUX_NEW_UTSNAME_BYTES {
                    return Err(user_transfer_error(
                        "complete uname transfer requires the entire layout extent",
                    ));
                }
                (scope, 0, true)
            }
            UserTransferOutcome::Fault {
                scope,
                copied_prefix,
            } => {
                if copied_prefix >= LINUX_NEW_UTSNAME_BYTES {
                    return Err(user_transfer_error(
                        "faulting uname transfer requires a proper copied prefix",
                    ));
                }
                (scope, LINUX_X86_64_EFAULT_RESULT, false)
            }
        };
        self.validate_transfer_scope(
            &scope,
            UserTransferDirection::ToUser,
            LINUX_X86_64_UNAME,
            LINUX_NEW_UTSNAME_BYTES,
        )?;
        if result != required_result {
            return Err(user_transfer_error(
                "uname return result must match its complete or fault outcome",
            ));
        }
        scope.entry.active.suspended.stage = match (scope.entry.active.suspended.stage, complete) {
            (UserProbeStage::ReturnedFromGetpid, true) => UserProbeStage::ReturnedFromValidUname,
            (UserProbeStage::ReturnedFromValidUname, false) => {
                UserProbeStage::ReturnedFromFaultingUname
            }
            _ => {
                return Err(user_transfer_error(
                    "uname transfer outcome does not match the sealed probe stage",
                ));
            }
        };
        Ok(scope.entry.active)
    }

    /// Reunite a completed or faulting timespec input with its nanosleep entry.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for an outcome/result/value mismatch or
    /// wrong probe stage.
    pub fn return_nanosleep(
        &self,
        outcome: UserInputTransferOutcome,
        result: u64,
    ) -> Result<ActiveUserExecution, SystemsModelError> {
        let (mut scope, required_result, complete) = match outcome {
            UserInputTransferOutcome::Complete {
                scope,
                copied,
                value,
            } => {
                if copied != LINUX_TIMESPEC_BYTES || value != LinuxTimespec::zero() {
                    return Err(user_transfer_error(
                        "complete nanosleep transfer requires the entire zero-duration layout",
                    ));
                }
                (scope, 0, true)
            }
            UserInputTransferOutcome::Fault {
                scope,
                copied_prefix,
            } => {
                if copied_prefix >= LINUX_TIMESPEC_BYTES {
                    return Err(user_transfer_error(
                        "faulting nanosleep transfer requires a proper copied prefix",
                    ));
                }
                (scope, LINUX_X86_64_EFAULT_RESULT, false)
            }
        };
        self.validate_transfer_scope(
            &scope,
            UserTransferDirection::FromUser,
            LINUX_X86_64_NANOSLEEP,
            LINUX_TIMESPEC_BYTES,
        )?;
        if result != required_result {
            return Err(user_transfer_error(
                "nanosleep return result must match its complete or fault outcome",
            ));
        }
        scope.entry.active.suspended.stage = match (scope.entry.active.suspended.stage, complete) {
            (UserProbeStage::ReturnedFromFaultingUname, true) => {
                UserProbeStage::ReturnedFromValidNanosleep
            }
            (UserProbeStage::ReturnedFromValidNanosleep, false) => {
                UserProbeStage::ReturnedFromFaultingNanosleep
            }
            _ => {
                return Err(user_transfer_error(
                    "nanosleep transfer outcome does not match the sealed probe stage",
                ));
            }
        };
        Ok(scope.entry.active)
    }

    /// Encode the source-owned PID and resume the exact captured continuation.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for the wrong syscall, result, or identity.
    pub fn return_getpid(
        &self,
        mut entry: UserSyscallContext,
        result: u64,
    ) -> Result<ActiveUserExecution, SystemsModelError> {
        self.validate_entry(&entry)?;
        if entry.number != LINUX_X86_64_GETPID
            || entry.active.suspended.stage != UserProbeStage::AwaitingGetpid
            || result != entry.active.suspended.process_identity
        {
            return Err(user_error(
                "getpid return requires the source-owned PID and exact captured continuation",
            ));
        }
        entry.active.suspended.stage = UserProbeStage::ReturnedFromGetpid;
        Ok(entry.active)
    }

    /// Consume exit_group entry and restore the registered kernel continuation.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for the wrong syscall, status, or identity.
    pub fn terminate_exit_group(
        &self,
        entry: UserSyscallContext,
        status: u64,
    ) -> Result<CompletedUserExecution, SystemsModelError> {
        self.validate_entry(&entry)?;
        if entry.number != LINUX_X86_64_EXIT_GROUP
            || entry.active.suspended.stage != UserProbeStage::ReturnedFromFaultingNanosleep
            || status != entry.arguments[0]
        {
            return Err(user_error(
                "exit_group termination requires the exact returned user continuation and status",
            ));
        }
        Ok(CompletedUserExecution {
            provider_identity: entry.active.suspended.validated.provider_identity,
            processor_identity: entry.active.suspended.processor_identity,
            address_space: entry.active.suspended.address_space,
            thread_identity: entry.active.suspended.thread_identity,
            abi_identity: entry.active.suspended.validated.abi_identity,
            process_identity: entry.active.suspended.process_identity,
            kernel_continuation_identity: entry.active.kernel_continuation_identity,
            terminal_entry_identity: entry.entry_identity,
            status,
        })
    }

    /// Consume terminal user execution and expose its semantic completion.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a completion from another protocol.
    pub fn complete(
        &self,
        completed: CompletedUserExecution,
    ) -> Result<UserExecutionCompletion, SystemsModelError> {
        if completed.provider_identity != self.provider_identity
            || completed.processor_identity != self.processor_identity
            || completed.address_space.provider_identity != self.provider_identity
            || completed.address_space.address_space_identity == 0
            || completed.thread_identity != self.thread_identity
            || completed.abi_identity != self.abi_identity
        {
            return Err(user_error(
                "user completion requires matching provider, processor, address-space, thread, and ABI identities",
            ));
        }
        Ok(UserExecutionCompletion {
            process_identity: completed.process_identity,
            kernel_continuation_identity: completed.kernel_continuation_identity,
            terminal_entry_identity: completed.terminal_entry_identity,
            status: completed.status,
            revoked_address_space: RevokedUserAddressSpace {
                provider_identity: completed.address_space.provider_identity,
                address_space_identity: completed.address_space.address_space_identity,
                backing: completed.address_space.backing,
            },
        })
    }

    /// Reclaim exact backing from one revoked user address space.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a space from another provider.
    pub fn reclaim_address_space(
        &self,
        revoked: RevokedUserAddressSpace,
    ) -> Result<PhysicalFrameRequest, SystemsModelError> {
        if revoked.provider_identity != self.provider_identity
            || revoked.address_space_identity == 0
        {
            return Err(user_error(
                "user-address-space reclaim requires matching provider and live identity",
            ));
        }
        Ok(revoked.backing)
    }

    fn validate_suspended(
        &self,
        suspended: &SuspendedUserContext,
    ) -> Result<(), SystemsModelError> {
        if suspended.validated.provider_identity != self.provider_identity
            || suspended.processor_identity != self.processor_identity
            || suspended.validated.address_space_identity
                != suspended.address_space.address_space_identity
            || suspended.address_space.provider_identity != self.provider_identity
            || suspended.thread_identity != self.thread_identity
            || suspended.validated.abi_identity != self.abi_identity
            || suspended.validated.probe_identity == 0
            || suspended.process_identity != 1
        {
            return Err(user_error(
                "user execution requires matching provider, processor, address-space, thread, ABI, probe, and process identities",
            ));
        }
        Ok(())
    }

    fn validate_entry(&self, entry: &UserSyscallContext) -> Result<(), SystemsModelError> {
        self.validate_suspended(&entry.active.suspended)?;
        if entry.entry_identity == 0 || entry.active.kernel_continuation_identity == 0 {
            return Err(user_error(
                "user syscall requires a fresh entry and registered kernel continuation",
            ));
        }
        Ok(())
    }

    fn validate_transfer_scope(
        &self,
        scope: &UserTransferScope,
        direction: UserTransferDirection,
        number: u64,
        extent: u64,
    ) -> Result<(), SystemsModelError> {
        self.validate_entry(&scope.entry)?;
        if scope.transfer_identity == 0
            || scope.entry.number != number
            || scope.candidate != scope.entry.arguments[0]
            || scope.extent != extent
            || scope.direction != direction
        {
            return Err(user_transfer_error(
                "user transfer scope must retain its exact syscall entry, candidate, extent, and identity",
            ));
        }
        Ok(())
    }
}

fn user_error(message: impl Into<String>) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-USER-CONTEXT", message)
}

fn user_transfer_error(message: impl Into<String>) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-USER-TRANSFER", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepared_probe(
        protocol: &mut SystemsUserExecutionProtocol,
        process_identity: u64,
    ) -> Result<SuspendedUserContext, SystemsModelError> {
        let address_space = protocol.prepare_address_space(INITIAL_USER_BACKING_REQUEST)?;
        protocol.prepare_probe(process_identity, address_space)
    }

    fn active_after_uname_fault(
        protocol: &mut SystemsUserExecutionProtocol,
    ) -> ActiveUserExecution {
        let suspended = prepared_probe(protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        let active = protocol.return_getpid(getpid, 1).unwrap();
        let uname = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_uname_transfer(uname).unwrap();
        let outcome = protocol
            .copy_initial_utsname_to_user(scope, &LinuxUtsName::initial(), LINUX_NEW_UTSNAME_BYTES)
            .unwrap();
        let active = protocol.return_uname(outcome, 0).unwrap();
        let uname = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_uname_transfer(uname).unwrap();
        let outcome = protocol
            .copy_initial_utsname_to_user(scope, &LinuxUtsName::initial(), 256)
            .unwrap();
        protocol
            .return_uname(outcome, LINUX_X86_64_EFAULT_RESULT)
            .unwrap()
    }

    #[test]
    fn retains_disjoint_code_and_data_mappings_until_role_specific_unmap() {
        // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-MAPPING-001.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let address_space = protocol
            .prepare_address_space(INITIAL_USER_BACKING_REQUEST)
            .unwrap();
        let mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        let first_identity = mapping.identity();
        let mapping = protocol.store_user_mapping_byte(mapping).unwrap();
        let mapping = protocol.protect_user_mapping_read_only(mapping).unwrap();
        let mapping = protocol.load_user_mapping_byte(mapping).unwrap();
        let mapping = protocol.permit_user_mapping_execute(mapping).unwrap();
        let code_mapping = mapping;
        let data_mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        assert_ne!(data_mapping.identity(), first_identity);
        let data_mapping = protocol.store_user_mapping_byte(data_mapping).unwrap();
        let data_mapping = protocol
            .protect_user_mapping_read_only(data_mapping)
            .unwrap();
        let data_mapping = protocol.load_user_mapping_byte(data_mapping).unwrap();
        let data_mapping = protocol
            .seal_user_mapping_immutable_data(data_mapping)
            .unwrap();
        assert_eq!(
            protocol
                .unmap_user_frame(&address_space, data_mapping)
                .unwrap(),
            INITIAL_USER_MAPPING_REQUEST
        );
        assert_eq!(
            protocol
                .unmap_user_frame(&address_space, code_mapping)
                .unwrap(),
            INITIAL_USER_MAPPING_REQUEST
        );
    }

    #[test]
    fn rejects_wrong_user_mapping_shape_order_and_unverified_unmap() {
        // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-MAPPING-001.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let address_space = protocol
            .prepare_address_space(INITIAL_USER_BACKING_REQUEST)
            .unwrap();
        assert_eq!(
            protocol
                .map_user_frame(&address_space, INITIAL_USER_BACKING_REQUEST)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        assert_eq!(
            protocol.load_user_mapping_byte(mapping).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let address_space = protocol
            .prepare_address_space(INITIAL_USER_BACKING_REQUEST)
            .unwrap();
        let mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        let mapping = protocol.store_user_mapping_byte(mapping).unwrap();
        assert_eq!(
            protocol.load_user_mapping_byte(mapping).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let address_space = protocol
            .prepare_address_space(INITIAL_USER_BACKING_REQUEST)
            .unwrap();
        let mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        assert_eq!(
            protocol
                .unmap_user_frame(&address_space, mapping)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );
    }

    #[test]
    fn rejects_cross_role_user_mapping_finalization() {
        // TOPAL-SYSTEMS-MAPPING-001, TK-DEC-044.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let address_space = protocol
            .prepare_address_space(INITIAL_USER_BACKING_REQUEST)
            .unwrap();
        let mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        let mapping = protocol.store_user_mapping_byte(mapping).unwrap();
        let mapping = protocol.protect_user_mapping_read_only(mapping).unwrap();
        let mapping = protocol.load_user_mapping_byte(mapping).unwrap();
        assert_eq!(
            protocol
                .seal_user_mapping_immutable_data(mapping)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let address_space = protocol
            .prepare_address_space(INITIAL_USER_BACKING_REQUEST)
            .unwrap();
        let mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        let mapping = protocol.store_user_mapping_byte(mapping).unwrap();
        let mapping = protocol.protect_user_mapping_read_only(mapping).unwrap();
        let mapping = protocol.load_user_mapping_byte(mapping).unwrap();
        let mapping = protocol.permit_user_mapping_execute(mapping).unwrap();
        protocol.unmap_user_frame(&address_space, mapping).unwrap();
        let mapping = protocol
            .map_user_frame(&address_space, INITIAL_USER_MAPPING_REQUEST)
            .unwrap();
        let mapping = protocol.store_user_mapping_byte(mapping).unwrap();
        let mapping = protocol.protect_user_mapping_read_only(mapping).unwrap();
        let mapping = protocol.load_user_mapping_byte(mapping).unwrap();
        assert_eq!(
            protocol
                .permit_user_mapping_execute(mapping)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );
    }

    #[test]
    fn completes_getpid_uname_timespec_transfers_and_exit_group_termination() {
        // TOPAL-SYSTEMS-CONTEXT-006, TOPAL-SYSTEMS-RECOVERY-001.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        assert_eq!(getpid.number(), 39);
        assert_eq!(getpid.arguments(), [0; 6]);
        let active = protocol.return_getpid(getpid, 1).unwrap();
        let uname = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_uname_transfer(uname).unwrap();
        let outcome = protocol
            .copy_initial_utsname_to_user(scope, &LinuxUtsName::initial(), LINUX_NEW_UTSNAME_BYTES)
            .unwrap();
        let active = protocol.return_uname(outcome, 0).unwrap();
        let uname = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_uname_transfer(uname).unwrap();
        let outcome = protocol
            .copy_initial_utsname_to_user(scope, &LinuxUtsName::initial(), 256)
            .unwrap();
        let active = protocol
            .return_uname(outcome, LINUX_X86_64_EFAULT_RESULT)
            .unwrap();
        let nanosleep = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [INITIAL_USER_TIMESPEC_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_nanosleep_transfer(nanosleep).unwrap();
        let outcome = protocol
            .copy_initial_timespec_from_user(
                scope,
                Some(LinuxTimespec::zero()),
                LINUX_TIMESPEC_BYTES,
            )
            .unwrap();
        let active = protocol.return_nanosleep(outcome, 0).unwrap();
        let nanosleep = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [INITIAL_USER_TIMESPEC_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_nanosleep_transfer(nanosleep).unwrap();
        let outcome = protocol
            .copy_initial_timespec_from_user(scope, None, 8)
            .unwrap();
        let active = protocol
            .return_nanosleep(outcome, LINUX_X86_64_EFAULT_RESULT)
            .unwrap();
        let exit = protocol
            .enter_syscall(active, LINUX_X86_64_EXIT_GROUP, [0; 6])
            .unwrap();
        let completed = protocol.terminate_exit_group(exit, 0).unwrap();
        let summary = protocol.complete(completed).unwrap();
        assert_eq!(summary.process_identity, 1);
        assert_eq!(summary.kernel_continuation_identity, 1);
        assert_eq!(summary.terminal_entry_identity, 6);
        assert_eq!(summary.status, 0);
        assert_eq!(
            protocol
                .reclaim_address_space(summary.revoked_address_space)
                .unwrap(),
            INITIAL_USER_BACKING_REQUEST
        );
    }

    #[test]
    fn rejects_wrong_order_result_status_and_identity() {
        // TOPAL-SYSTEMS-CONTEXT-006 negative conformance.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        assert_eq!(
            protocol
                .prepare_address_space(PhysicalFrameRequest {
                    frame_count: 1,
                    alignment_frames: 1,
                })
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );
        assert_eq!(
            prepared_probe(&mut protocol, 2).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        assert_eq!(
            protocol
                .enter_syscall(active, LINUX_X86_64_EXIT_GROUP, [0; 6])
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        assert_eq!(
            protocol.return_getpid(getpid, 2).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        let active = protocol.return_getpid(getpid, 1).unwrap();
        assert_eq!(
            protocol
                .enter_syscall(active, LINUX_X86_64_UNAME, [0; 6])
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        let active = protocol.return_getpid(getpid, 1).unwrap();
        let uname = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_uname_transfer(uname).unwrap();
        let outcome = protocol
            .copy_initial_utsname_to_user(scope, &LinuxUtsName::initial(), LINUX_NEW_UTSNAME_BYTES)
            .unwrap();
        assert_eq!(
            protocol.return_uname(outcome, 1).unwrap_err().code,
            "E-SYSTEMS-USER-TRANSFER"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        let active = protocol.return_getpid(getpid, 1).unwrap();
        let uname = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_uname_transfer(uname).unwrap();
        assert_eq!(
            protocol
                .copy_initial_utsname_to_user(scope, &LinuxUtsName::initial(), 1)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-TRANSFER"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        let active = protocol.return_getpid(getpid, 1).unwrap();
        let uname = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_uname_transfer(uname).unwrap();
        let mut wrong_value = LinuxUtsName::initial();
        wrong_value.machine = "aarch64".into();
        assert_eq!(
            protocol
                .copy_initial_utsname_to_user(scope, &wrong_value, LINUX_NEW_UTSNAME_BYTES)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-TRANSFER"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        let active = protocol.return_getpid(getpid, 1).unwrap();
        assert_eq!(
            protocol
                .enter_syscall(active, LINUX_X86_64_EXIT_GROUP, [0; 6])
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = prepared_probe(&mut protocol, 1).unwrap();
        let mut active = protocol.enter(suspended).unwrap();
        active.suspended.thread_identity = "topal.systems.thread.other/1".into();
        assert_eq!(
            protocol
                .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );
    }

    #[test]
    fn rejects_partial_or_invalid_timespec_values() {
        // TOPAL-SYSTEMS-RECOVERY-001 negative conformance.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let active = active_after_uname_fault(&mut protocol);
        assert_eq!(
            protocol
                .enter_syscall(
                    active,
                    LINUX_X86_64_NANOSLEEP,
                    [INITIAL_USER_TIMESPEC_VALID_CANDIDATE, 1, 0, 0, 0, 0],
                )
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let active = active_after_uname_fault(&mut protocol);
        let nanosleep = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [INITIAL_USER_TIMESPEC_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_nanosleep_transfer(nanosleep).unwrap();
        assert_eq!(
            protocol
                .copy_initial_timespec_from_user(scope, None, LINUX_TIMESPEC_BYTES)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-TRANSFER"
        );

        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let active = active_after_uname_fault(&mut protocol);
        let nanosleep = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [INITIAL_USER_TIMESPEC_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_nanosleep_transfer(nanosleep).unwrap();
        assert_eq!(
            protocol
                .copy_initial_timespec_from_user(
                    scope,
                    Some(LinuxTimespec {
                        seconds: -1,
                        nanoseconds: 0,
                    }),
                    LINUX_TIMESPEC_BYTES,
                )
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-TRANSFER"
        );

        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let active = active_after_uname_fault(&mut protocol);
        let nanosleep = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [INITIAL_USER_TIMESPEC_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_nanosleep_transfer(nanosleep).unwrap();
        let outcome = protocol
            .copy_initial_timespec_from_user(
                scope,
                Some(LinuxTimespec::zero()),
                LINUX_TIMESPEC_BYTES,
            )
            .unwrap();
        let active = protocol.return_nanosleep(outcome, 0).unwrap();
        let nanosleep = protocol
            .enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [INITIAL_USER_TIMESPEC_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            )
            .unwrap();
        let scope = protocol.begin_nanosleep_transfer(nanosleep).unwrap();
        assert_eq!(
            protocol
                .copy_initial_timespec_from_user(scope, Some(LinuxTimespec::zero()), 8)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-TRANSFER"
        );
    }
}
