//! Architecture-independent kernel-context transfer protocol.

use crate::{BootstrapRegion, BootstrapStoragePlacement, SystemsModelError};

pub const SYSTEMS_KERNEL_CONTEXT_CREATE: &str = "topal.systems.context.kernel.create/2";
pub const SYSTEMS_KERNEL_CONTEXT_TRANSFER: &str = "topal.systems.context.kernel.transfer/2";
pub const SYSTEMS_KERNEL_CONTEXT_RETIRE: &str =
    "topal.systems.disposition.kernel-context-retire-to-caller/2";
pub const SYSTEMS_KERNEL_CONTEXT_RECLAIM: &str = "topal.systems.context.kernel.reclaim/1";
pub const SYSTEMS_KERNEL_THREAD_ENTRY: &str = "topal.systems.entry.resumed.kernel-thread/1";

pub const INITIAL_PROCESSOR_IDENTITY: &str = "topal.systems.processor.initial/1";
pub const INITIAL_ADDRESS_SPACE_IDENTITY: &str = "topal.systems.address-space.initial-kernel/1";
pub const INITIAL_KERNEL_THREAD_ENTRY_IDENTITY: &str =
    "topal.systems.entry.resumed.kernel-thread.initial/1";
pub const COOPERATIVE_KERNEL_THREAD_ENTRY_IDENTITY: &str =
    "topal.systems.entry.resumed.kernel-thread.cooperative/1";
pub const TERMINAL_KERNEL_THREAD_ENTRY_IDENTITY: &str =
    "topal.systems.entry.resumed.kernel-thread.terminal/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelContextEntryProtocol {
    CooperativeOnce,
    Terminal,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SuspendedKernelContext {
    context_identity: u64,
    processor_identity: String,
    address_space_identity: String,
    entry_identity: String,
    entry_protocol: KernelContextEntryProtocol,
    activation_count: u8,
    stack: BootstrapRegion,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ActiveKernelContextTransfer {
    context_identity: u64,
    processor_identity: String,
    address_space_identity: String,
    entry_identity: String,
    entry_protocol: KernelContextEntryProtocol,
    stack: BootstrapRegion,
    caller_identity: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompletedKernelContextTransfer {
    context_identity: u64,
    processor_identity: String,
    address_space_identity: String,
    entry_identity: String,
    entry_protocol: KernelContextEntryProtocol,
    stack: BootstrapRegion,
    caller_identity: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelContextCompletion {
    pub context_identity: u64,
    pub caller_identity: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum KernelContextTransferOutcome {
    Suspended(SuspendedKernelContext),
    Retired(CompletedKernelContextTransfer),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsKernelContextProtocol {
    provider_identity: String,
    processor_identity: String,
    address_space_identity: String,
    entry_identity: String,
    next_context_identity: u64,
    next_caller_identity: u64,
}

impl SystemsKernelContextProtocol {
    #[must_use]
    pub fn initial() -> Self {
        Self {
            provider_identity: "topal.systems.context-provider.initial/1".into(),
            processor_identity: INITIAL_PROCESSOR_IDENTITY.into(),
            address_space_identity: INITIAL_ADDRESS_SPACE_IDENTITY.into(),
            entry_identity: INITIAL_KERNEL_THREAD_ENTRY_IDENTITY.into(),
            next_context_identity: 1,
            next_caller_identity: 1,
        }
    }

    /// Consume one checked ordinary region into an initial suspended context.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic unless the region has the sealed stack
    /// size, alignment, and placement.
    pub fn create(
        &mut self,
        stack: BootstrapRegion,
    ) -> Result<SuspendedKernelContext, SystemsModelError> {
        self.create_with_protocol(
            stack,
            INITIAL_KERNEL_THREAD_ENTRY_IDENTITY,
            KernelContextEntryProtocol::Terminal,
        )
    }

    /// Consume one checked region into a context with a closed entry protocol.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for the wrong stack contract or an entry
    /// identity which does not match its admitted protocol.
    pub fn create_with_protocol(
        &mut self,
        stack: BootstrapRegion,
        entry_identity: &str,
        entry_protocol: KernelContextEntryProtocol,
    ) -> Result<SuspendedKernelContext, SystemsModelError> {
        if stack.byte_count() != 16_384
            || stack.alignment_bytes() != 16
            || stack.placement() != BootstrapStoragePlacement::BootstrapReclaimable
        {
            return Err(context_error(
                "the initial kernel context requires one 16 KiB, 16-byte-aligned bootstrap-reclaimable stack",
            ));
        }
        let context_identity = self.next_context_identity;
        self.next_context_identity = context_identity
            .checked_add(1)
            .ok_or_else(|| context_error("kernel-context identity exhausted"))?;
        Ok(SuspendedKernelContext {
            context_identity,
            processor_identity: self.processor_identity.clone(),
            address_space_identity: self.address_space_identity.clone(),
            entry_identity: entry_identity.into(),
            entry_protocol,
            activation_count: 0,
            stack,
        })
    }

    /// Suspend the caller and select one matching suspended continuation.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a context from another processor,
    /// address space, entry, or provider protocol.
    pub fn transfer(
        &mut self,
        suspended: SuspendedKernelContext,
    ) -> Result<ActiveKernelContextTransfer, SystemsModelError> {
        self.validate_identity(
            &suspended.processor_identity,
            &suspended.address_space_identity,
            &suspended.entry_identity,
        )?;
        let caller_identity = self.next_caller_identity;
        self.next_caller_identity = caller_identity
            .checked_add(1)
            .ok_or_else(|| context_error("suspended-caller identity exhausted"))?;
        Ok(ActiveKernelContextTransfer {
            context_identity: suspended.context_identity,
            processor_identity: suspended.processor_identity,
            address_space_identity: suspended.address_space_identity,
            entry_identity: suspended.entry_identity,
            entry_protocol: suspended.entry_protocol,
            stack: suspended.stack,
            caller_identity,
        })
    }

    /// Retire the running worker and resume exactly its suspended caller.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for mismatched protocol identity or a
    /// worker with live obligations.
    pub fn retire_to_caller(
        &self,
        active: ActiveKernelContextTransfer,
        live_obligations: usize,
    ) -> Result<CompletedKernelContextTransfer, SystemsModelError> {
        self.validate_identity(
            &active.processor_identity,
            &active.address_space_identity,
            &active.entry_identity,
        )?;
        if live_obligations != 0 {
            return Err(context_error(
                "kernel-thread retirement requires a transfer-safe point with no live obligations",
            ));
        }
        Ok(CompletedKernelContextTransfer {
            context_identity: active.context_identity,
            processor_identity: active.processor_identity,
            address_space_identity: active.address_space_identity,
            entry_identity: active.entry_identity,
            entry_protocol: active.entry_protocol,
            stack: active.stack,
            caller_identity: active.caller_identity,
        })
    }

    /// Run one statically qualified context until it hands back or retires.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for an identity mismatch, an unsupported
    /// entry protocol state, or retirement with live obligations.
    pub fn dispatch(
        &mut self,
        mut suspended: SuspendedKernelContext,
        live_obligations: usize,
    ) -> Result<KernelContextTransferOutcome, SystemsModelError> {
        self.validate_identity(
            &suspended.processor_identity,
            &suspended.address_space_identity,
            &suspended.entry_identity,
        )?;
        let caller_identity = self.next_caller_identity;
        self.next_caller_identity = caller_identity
            .checked_add(1)
            .ok_or_else(|| context_error("suspended-caller identity exhausted"))?;
        match (suspended.entry_protocol, suspended.activation_count) {
            (KernelContextEntryProtocol::CooperativeOnce, 0) => {
                suspended.activation_count = 1;
                Ok(KernelContextTransferOutcome::Suspended(suspended))
            }
            (KernelContextEntryProtocol::CooperativeOnce, 1)
            | (KernelContextEntryProtocol::Terminal, 0) => {
                if live_obligations != 0 {
                    return Err(context_error(
                        "kernel-thread retirement requires a transfer-safe point with no live obligations",
                    ));
                }
                Ok(KernelContextTransferOutcome::Retired(
                    CompletedKernelContextTransfer {
                        context_identity: suspended.context_identity,
                        processor_identity: suspended.processor_identity,
                        address_space_identity: suspended.address_space_identity,
                        entry_identity: suspended.entry_identity,
                        entry_protocol: suspended.entry_protocol,
                        stack: suspended.stack,
                        caller_identity,
                    },
                ))
            }
            (KernelContextEntryProtocol::CooperativeOnce, _) => Err(context_error(
                "cooperative kernel context may hand back exactly once before retirement",
            )),
            (KernelContextEntryProtocol::Terminal, _) => Err(context_error(
                "terminal kernel context cannot be selected after retirement",
            )),
        }
    }

    /// Consume a completed transfer and return its stack for pool reclamation.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a completion from another protocol.
    pub fn reclaim(
        &self,
        completed: CompletedKernelContextTransfer,
    ) -> Result<(KernelContextCompletion, BootstrapRegion), SystemsModelError> {
        self.validate_identity(
            &completed.processor_identity,
            &completed.address_space_identity,
            &completed.entry_identity,
        )?;
        Ok((
            KernelContextCompletion {
                context_identity: completed.context_identity,
                caller_identity: completed.caller_identity,
            },
            completed.stack,
        ))
    }

    fn validate_identity(
        &self,
        processor_identity: &str,
        address_space_identity: &str,
        entry_identity: &str,
    ) -> Result<(), SystemsModelError> {
        let valid_entry = entry_identity == self.entry_identity
            || entry_identity == COOPERATIVE_KERNEL_THREAD_ENTRY_IDENTITY
            || entry_identity == TERMINAL_KERNEL_THREAD_ENTRY_IDENTITY;
        if self.provider_identity != "topal.systems.context-provider.initial/1"
            || processor_identity != self.processor_identity
            || address_space_identity != self.address_space_identity
            || !valid_entry
        {
            return Err(context_error(
                "kernel-context transfer requires matching provider, processor, address space, and entry identities",
            ));
        }
        Ok(())
    }
}

fn context_error(message: impl Into<String>) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-CONTEXT-TRANSFER", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BootstrapStorageDescriptor, BootstrapStorageRequest, BootstrapStorageState};

    fn stack(storage: &mut BootstrapStorageState) -> BootstrapRegion {
        storage
            .allocate(BootstrapStorageRequest {
                byte_count: 16_384,
                alignment_bytes: 16,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            })
            .unwrap()
    }

    #[test]
    fn transfers_retires_resumes_and_reclaims_one_stack() {
        // TOPAL-SYSTEMS-CONTEXT-001.
        let mut storage = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 65_536,
                alignment_bytes: 4096,
            },
            "context-test",
        )
        .unwrap();
        let mut protocol = SystemsKernelContextProtocol::initial();
        let suspended = protocol.create(stack(&mut storage)).unwrap();
        let active = protocol.transfer(suspended).unwrap();
        let completed = protocol.retire_to_caller(active, 0).unwrap();
        let (completion, stack) = protocol.reclaim(completed).unwrap();

        assert_eq!(completion.context_identity, 1);
        assert_eq!(completion.caller_identity, 1);
        storage.release(stack).unwrap();
        storage.complete_bootstrap().unwrap();
    }

    #[test]
    fn rejects_wrong_stack_and_retirement_with_live_obligations() {
        // TOPAL-SYSTEMS-CONTEXT-001.
        let mut storage = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 65_536,
                alignment_bytes: 4096,
            },
            "context-negative",
        )
        .unwrap();
        let short = storage
            .allocate(BootstrapStorageRequest {
                byte_count: 4096,
                alignment_bytes: 16,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            })
            .unwrap();
        let mut protocol = SystemsKernelContextProtocol::initial();
        assert_eq!(
            protocol.create(short).unwrap_err().code,
            "E-SYSTEMS-CONTEXT-TRANSFER"
        );

        let suspended = protocol.create(stack(&mut storage)).unwrap();
        let active = protocol.transfer(suspended).unwrap();
        assert_eq!(
            protocol.retire_to_caller(active, 1).unwrap_err().code,
            "E-SYSTEMS-CONTEXT-TRANSFER"
        );
    }

    #[test]
    fn dispatches_two_contexts_in_source_selected_fifo_order() {
        // TOPAL-SYSTEMS-CONTEXT-002.
        let mut storage = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 65_536,
                alignment_bytes: 4096,
            },
            "cooperative-context-test",
        )
        .unwrap();
        let mut protocol = SystemsKernelContextProtocol::initial();
        let cooperative = protocol
            .create_with_protocol(
                stack(&mut storage),
                COOPERATIVE_KERNEL_THREAD_ENTRY_IDENTITY,
                KernelContextEntryProtocol::CooperativeOnce,
            )
            .unwrap();
        let terminal = protocol
            .create_with_protocol(
                stack(&mut storage),
                TERMINAL_KERNEL_THREAD_ENTRY_IDENTITY,
                KernelContextEntryProtocol::Terminal,
            )
            .unwrap();

        let KernelContextTransferOutcome::Suspended(cooperative) =
            protocol.dispatch(cooperative, 0).unwrap()
        else {
            panic!("cooperative worker must hand back on first selection")
        };
        let KernelContextTransferOutcome::Retired(terminal) =
            protocol.dispatch(terminal, 0).unwrap()
        else {
            panic!("terminal worker must retire on first selection")
        };
        let (terminal_completion, terminal_stack) = protocol.reclaim(terminal).unwrap();
        assert_eq!(terminal_completion.context_identity, 2);
        storage.release(terminal_stack).unwrap();

        let KernelContextTransferOutcome::Retired(cooperative) =
            protocol.dispatch(cooperative, 0).unwrap()
        else {
            panic!("cooperative worker must retire on second selection")
        };
        let (cooperative_completion, cooperative_stack) = protocol.reclaim(cooperative).unwrap();
        assert_eq!(cooperative_completion.context_identity, 1);
        storage.release(cooperative_stack).unwrap();
        storage.complete_bootstrap().unwrap();
    }

    #[test]
    fn rejects_wrong_cooperative_outcome_and_live_retirement_obligations() {
        // TOPAL-SYSTEMS-CONTEXT-002.
        let mut storage = BootstrapStorageState::new(
            BootstrapStorageDescriptor {
                capacity_bytes: 65_536,
                alignment_bytes: 4096,
            },
            "cooperative-context-negative",
        )
        .unwrap();
        let mut protocol = SystemsKernelContextProtocol::initial();
        let cooperative = protocol
            .create_with_protocol(
                stack(&mut storage),
                COOPERATIVE_KERNEL_THREAD_ENTRY_IDENTITY,
                KernelContextEntryProtocol::CooperativeOnce,
            )
            .unwrap();
        let KernelContextTransferOutcome::Suspended(cooperative) =
            protocol.dispatch(cooperative, 0).unwrap()
        else {
            panic!("cooperative worker must hand back")
        };
        assert_eq!(
            protocol.dispatch(cooperative, 1).unwrap_err().code,
            "E-SYSTEMS-CONTEXT-TRANSFER"
        );
    }
}
