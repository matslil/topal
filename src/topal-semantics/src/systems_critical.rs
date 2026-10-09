//! Architecture-independent affine critical-scope lifecycle.

use std::fmt;

pub const SYSTEMS_CRITICAL_ENTER: &str = "topal.systems.critical.enter/1";
pub const SYSTEMS_CRITICAL_RESTORE: &str = "topal.systems.critical.restore/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CriticalDomain {
    LocalMaskableInterrupts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalInterruptState {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CriticalScopeError {
    EmptyProcessorIdentity,
    WrongProcessor,
    OutOfOrderRestore,
}

impl fmt::Display for CriticalScopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyProcessorIdentity => "empty-processor-identity",
            Self::WrongProcessor => "wrong-processor",
            Self::OutOfOrderRestore => "out-of-order-restore",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CriticalRestoreToken {
    processor_identity: String,
    domain: CriticalDomain,
    nesting_identity: u64,
    prior_state: LocalInterruptState,
}

/// An affine model of the current processor's local interrupt state.
#[derive(Debug, Eq, PartialEq)]
pub struct ProcessorInterruptContext {
    processor_identity: String,
    local_interrupt_state: LocalInterruptState,
    next_nesting_identity: u64,
}

/// A consumed processor context refined by one or more critical entries.
#[derive(Debug, Eq, PartialEq)]
pub struct LocalInterruptCriticalContext {
    processor_identity: String,
    next_nesting_identity: u64,
    restoration: Vec<CriticalRestoreToken>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CriticalRestoration {
    Processor(ProcessorInterruptContext),
    Critical(LocalInterruptCriticalContext),
}

impl ProcessorInterruptContext {
    /// Construct one modeled current-processor context.
    ///
    /// # Errors
    ///
    /// Returns a stable error for an empty processor identity.
    pub fn new(
        processor_identity: impl Into<String>,
        local_interrupt_state: LocalInterruptState,
    ) -> Result<Self, CriticalScopeError> {
        let processor_identity = processor_identity.into();
        if processor_identity.is_empty() {
            return Err(CriticalScopeError::EmptyProcessorIdentity);
        }
        Ok(Self {
            processor_identity,
            local_interrupt_state,
            next_nesting_identity: 1,
        })
    }

    #[must_use]
    pub fn processor_identity(&self) -> &str {
        &self.processor_identity
    }

    #[must_use]
    pub const fn local_interrupt_state(&self) -> LocalInterruptState {
        self.local_interrupt_state
    }

    #[must_use]
    pub fn enter_local_maskable_interrupts(self) -> LocalInterruptCriticalContext {
        let token = CriticalRestoreToken {
            processor_identity: self.processor_identity.clone(),
            domain: CriticalDomain::LocalMaskableInterrupts,
            nesting_identity: self.next_nesting_identity,
            prior_state: self.local_interrupt_state,
        };
        LocalInterruptCriticalContext {
            processor_identity: self.processor_identity,
            next_nesting_identity: self.next_nesting_identity + 1,
            restoration: vec![token],
        }
    }
}

impl LocalInterruptCriticalContext {
    #[must_use]
    pub fn processor_identity(&self) -> &str {
        &self.processor_identity
    }

    #[must_use]
    pub fn nesting_depth(&self) -> usize {
        self.restoration.len()
    }

    #[must_use]
    pub fn enter_nested(mut self) -> Self {
        self.restoration.push(CriticalRestoreToken {
            processor_identity: self.processor_identity.clone(),
            domain: CriticalDomain::LocalMaskableInterrupts,
            nesting_identity: self.next_nesting_identity,
            prior_state: LocalInterruptState::Disabled,
        });
        self.next_nesting_identity += 1;
        self
    }

    /// Consume the innermost restoration authority on the owning processor.
    ///
    /// # Errors
    ///
    /// Returns a stable error for processor mismatch or malformed nesting.
    pub fn restore(
        mut self,
        processor_identity: &str,
    ) -> Result<CriticalRestoration, CriticalScopeError> {
        if self.processor_identity != processor_identity {
            return Err(CriticalScopeError::WrongProcessor);
        }
        let token = self
            .restoration
            .pop()
            .ok_or(CriticalScopeError::OutOfOrderRestore)?;
        if token.processor_identity != processor_identity
            || token.domain != CriticalDomain::LocalMaskableInterrupts
        {
            return Err(CriticalScopeError::OutOfOrderRestore);
        }
        if self.restoration.is_empty() {
            return Ok(CriticalRestoration::Processor(ProcessorInterruptContext {
                processor_identity: self.processor_identity,
                local_interrupt_state: token.prior_state,
                next_nesting_identity: self.next_nesting_identity,
            }));
        }
        if token.prior_state != LocalInterruptState::Disabled {
            return Err(CriticalScopeError::OutOfOrderRestore);
        }
        Ok(CriticalRestoration::Critical(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_exact_prior_local_interrupt_state() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-CRITICAL-001.
        for prior in [LocalInterruptState::Enabled, LocalInterruptState::Disabled] {
            let context = ProcessorInterruptContext::new("cpu-0", prior).unwrap();
            let critical = context.enter_local_maskable_interrupts();
            assert_eq!(critical.nesting_depth(), 1);
            let CriticalRestoration::Processor(restored) = critical.restore("cpu-0").unwrap()
            else {
                panic!("outer restore must return the processor context")
            };
            assert_eq!(restored.local_interrupt_state(), prior);
        }
    }

    #[test]
    fn nested_scopes_restore_lifo_and_cannot_cross_processors() {
        // TOPAL-SYSTEMS-CRITICAL-001.
        let critical = ProcessorInterruptContext::new("cpu-0", LocalInterruptState::Enabled)
            .unwrap()
            .enter_local_maskable_interrupts()
            .enter_nested();
        assert_eq!(critical.nesting_depth(), 2);
        let CriticalRestoration::Critical(outer) = critical.restore("cpu-0").unwrap() else {
            panic!("inner restore must return the outer critical context")
        };
        assert_eq!(outer.nesting_depth(), 1);
        assert_eq!(
            outer.restore("cpu-1").unwrap_err(),
            CriticalScopeError::WrongProcessor
        );
    }
}
