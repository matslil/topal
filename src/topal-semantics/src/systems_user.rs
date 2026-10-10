//! Architecture-independent affine user-execution and syscall-entry model.

use crate::SystemsModelError;

pub const SYSTEMS_USER_CONTEXT_PREPARE: &str = "topal.systems.context.user.probe.prepare/1";
pub const SYSTEMS_USER_CONTEXT_ENTER: &str = "topal.systems.context.user.enter/1";
pub const SYSTEMS_USER_SYSCALL_ENTRY: &str = "topal.systems.entry.user-syscall/1";
pub const SYSTEMS_USER_SYSCALL_DISPATCH: &str =
    "topal.systems.disposition.user-syscall.dispatch-initial/1";
pub const SYSTEMS_USER_SYSCALL_RETURN: &str = "topal.systems.disposition.user-syscall.return/1";
pub const SYSTEMS_USER_TERMINATE: &str = "topal.systems.disposition.user-syscall.terminate/1";
pub const SYSTEMS_USER_CONTEXT_COMPLETE: &str = "topal.systems.context.user.complete/1";

pub const INITIAL_USER_ADDRESS_SPACE_IDENTITY: &str = "topal.systems.address-space.initial-user/1";
pub const INITIAL_USER_THREAD_IDENTITY: &str = "topal.systems.thread.initial-user/1";
pub const LINUX_X86_64_ABI_IDENTITY: &str = "topal.systems.abi.linux-x86_64-7.2.9/1";
pub const LINUX_X86_64_GETPID: u64 = 39;
pub const LINUX_X86_64_EXIT_GROUP: u64 = 231;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UserProbeStage {
    AwaitingGetpid,
    ReturnedFromGetpid,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ValidatedUserState {
    provider_identity: String,
    address_space_identity: String,
    abi_identity: String,
    probe_identity: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SuspendedUserContext {
    validated: ValidatedUserState,
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
    address_space_identity: String,
    thread_identity: String,
    abi_identity: String,
    process_identity: u64,
    kernel_continuation_identity: u64,
    terminal_entry_identity: u64,
    status: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserExecutionCompletion {
    pub process_identity: u64,
    pub kernel_continuation_identity: u64,
    pub terminal_entry_identity: u64,
    pub status: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsUserExecutionProtocol {
    provider_identity: String,
    processor_identity: String,
    address_space_identity: String,
    thread_identity: String,
    abi_identity: String,
    next_probe_identity: u64,
    next_kernel_continuation_identity: u64,
    next_entry_identity: u64,
}

impl SystemsUserExecutionProtocol {
    #[must_use]
    pub fn initial_linux_x86_64() -> Self {
        Self {
            provider_identity: "topal.systems.user-provider.initial/1".into(),
            processor_identity: crate::INITIAL_PROCESSOR_IDENTITY.into(),
            address_space_identity: INITIAL_USER_ADDRESS_SPACE_IDENTITY.into(),
            thread_identity: INITIAL_USER_THREAD_IDENTITY.into(),
            abi_identity: LINUX_X86_64_ABI_IDENTITY.into(),
            next_probe_identity: 1,
            next_kernel_continuation_identity: 1,
            next_entry_identity: 1,
        }
    }

    /// Create one validated qualification-only user continuation.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless the initial source-owned PID is one.
    pub fn prepare_probe(
        &mut self,
        process_identity: u64,
    ) -> Result<SuspendedUserContext, SystemsModelError> {
        if process_identity != 1 {
            return Err(user_error(
                "the initial user probe requires source-owned process identity one",
            ));
        }
        let probe_identity = self.next_probe_identity;
        self.next_probe_identity = probe_identity
            .checked_add(1)
            .ok_or_else(|| user_error("user-probe identity exhausted"))?;
        Ok(SuspendedUserContext {
            validated: ValidatedUserState {
                provider_identity: self.provider_identity.clone(),
                address_space_identity: self.address_space_identity.clone(),
                abi_identity: self.abi_identity.clone(),
                probe_identity,
            },
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
                number == LINUX_X86_64_EXIT_GROUP && arguments[0] == 0 && arguments[1..] == [0; 5]
            }
        };
        if !valid {
            return Err(user_error(
                "user syscall does not match the sealed getpid then exit_group protocol",
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
            || entry.active.suspended.stage != UserProbeStage::ReturnedFromGetpid
            || status != entry.arguments[0]
        {
            return Err(user_error(
                "exit_group termination requires the exact returned user continuation and status",
            ));
        }
        Ok(CompletedUserExecution {
            provider_identity: entry.active.suspended.validated.provider_identity,
            processor_identity: entry.active.suspended.processor_identity,
            address_space_identity: entry.active.suspended.validated.address_space_identity,
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
            || completed.address_space_identity != self.address_space_identity
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
        })
    }

    fn validate_suspended(
        &self,
        suspended: &SuspendedUserContext,
    ) -> Result<(), SystemsModelError> {
        if suspended.validated.provider_identity != self.provider_identity
            || suspended.processor_identity != self.processor_identity
            || suspended.validated.address_space_identity != self.address_space_identity
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
}

fn user_error(message: impl Into<String>) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-USER-CONTEXT", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_getpid_return_and_exit_group_termination() {
        // TOPAL-SYSTEMS-CONTEXT-006.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        let suspended = protocol.prepare_probe(1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        assert_eq!(getpid.number(), 39);
        assert_eq!(getpid.arguments(), [0; 6]);
        let active = protocol.return_getpid(getpid, 1).unwrap();
        let exit = protocol
            .enter_syscall(active, LINUX_X86_64_EXIT_GROUP, [0; 6])
            .unwrap();
        let completed = protocol.terminate_exit_group(exit, 0).unwrap();
        let summary = protocol.complete(completed).unwrap();
        assert_eq!(summary.process_identity, 1);
        assert_eq!(summary.kernel_continuation_identity, 1);
        assert_eq!(summary.terminal_entry_identity, 2);
        assert_eq!(summary.status, 0);
    }

    #[test]
    fn rejects_wrong_order_result_status_and_identity() {
        // TOPAL-SYSTEMS-CONTEXT-006 negative conformance.
        let mut protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
        assert_eq!(
            protocol.prepare_probe(2).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = protocol.prepare_probe(1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        assert_eq!(
            protocol
                .enter_syscall(active, LINUX_X86_64_EXIT_GROUP, [0; 6])
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = protocol.prepare_probe(1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        assert_eq!(
            protocol.return_getpid(getpid, 2).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = protocol.prepare_probe(1).unwrap();
        let active = protocol.enter(suspended).unwrap();
        let getpid = protocol
            .enter_syscall(active, LINUX_X86_64_GETPID, [0; 6])
            .unwrap();
        let active = protocol.return_getpid(getpid, 1).unwrap();
        let exit = protocol
            .enter_syscall(active, LINUX_X86_64_EXIT_GROUP, [0; 6])
            .unwrap();
        assert_eq!(
            protocol.terminate_exit_group(exit, 1).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let suspended = protocol.prepare_probe(1).unwrap();
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
}
