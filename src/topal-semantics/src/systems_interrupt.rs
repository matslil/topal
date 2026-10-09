//! Architecture-independent local-notification interrupt protocol.

use crate::SystemsModelError;

pub const SYSTEMS_LOCAL_NOTIFICATION_SEND: &str =
    "topal.systems.interrupt.local-notification.send/1";
pub const SYSTEMS_LOCAL_NOTIFICATION_WAIT: &str =
    "topal.systems.interrupt.local-notification.wait/1";
pub const SYSTEMS_LOCAL_NOTIFICATION_COMPLETE: &str =
    "topal.systems.interrupt.local-notification.complete/1";
pub const SYSTEMS_RESUME_LOCAL_NOTIFICATION: &str =
    "topal.systems.disposition.resume-local-notification/1";
pub const SYSTEMS_ENTRY_LOCAL_NOTIFICATION: &str =
    "topal.systems.entry.external.local-notification/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalInterruptMaskState {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LocalNotificationState {
    Idle,
    Pending {
        event_identity: u64,
        prior_mask: LocalInterruptMaskState,
    },
    Entered {
        event_identity: u64,
        prior_mask: LocalInterruptMaskState,
    },
    Completed {
        event_identity: u64,
        prior_mask: LocalInterruptMaskState,
    },
    Resumed {
        event_identity: u64,
        prior_mask: LocalInterruptMaskState,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalNotificationProtocol {
    source_identity: String,
    next_event_identity: u64,
    state: LocalNotificationState,
}

impl LocalNotificationProtocol {
    #[must_use]
    pub fn new(source_identity: impl Into<String>) -> Self {
        Self {
            source_identity: source_identity.into(),
            next_event_identity: 1,
            state: LocalNotificationState::Idle,
        }
    }

    #[must_use]
    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    /// Consume the current processor continuation into one pending event.
    ///
    /// # Errors
    ///
    /// Rejects an empty source identity, event-identity exhaustion, or a
    /// second send while another event is live.
    pub fn send(&mut self, prior_mask: LocalInterruptMaskState) -> Result<u64, SystemsModelError> {
        if self.source_identity.is_empty() {
            return Err(interrupt_error(
                "a local-notification source requires an identity",
            ));
        }
        if self.state != LocalNotificationState::Idle {
            return Err(interrupt_error(
                "local notification send requires an idle source",
            ));
        }
        let event_identity = self.next_event_identity;
        self.next_event_identity = self
            .next_event_identity
            .checked_add(1)
            .ok_or_else(|| interrupt_error("local-notification event identity overflowed"))?;
        self.state = LocalNotificationState::Pending {
            event_identity,
            prior_mask,
        };
        Ok(event_identity)
    }

    /// Bind the provider observation and typed entry to the pending event.
    ///
    /// # Errors
    ///
    /// Rejects an event not matching the sole pending session.
    pub fn enter(&mut self, event_identity: u64) -> Result<(), SystemsModelError> {
        let LocalNotificationState::Pending {
            event_identity: pending,
            prior_mask,
        } = self.state
        else {
            return Err(interrupt_error(
                "local-notification entry requires one pending event",
            ));
        };
        if event_identity != pending {
            return Err(interrupt_error(
                "local-notification entry does not match the pending event",
            ));
        }
        self.state = LocalNotificationState::Entered {
            event_identity,
            prior_mask,
        };
        Ok(())
    }

    /// Consume the entry context and discharge its completion obligation.
    ///
    /// # Errors
    ///
    /// Rejects completion outside the matching entered event.
    pub fn complete(&mut self, event_identity: u64) -> Result<(), SystemsModelError> {
        let LocalNotificationState::Entered {
            event_identity: entered,
            prior_mask,
        } = self.state
        else {
            return Err(interrupt_error(
                "local-notification completion requires one entered event",
            ));
        };
        if event_identity != entered {
            return Err(interrupt_error(
                "local-notification completion does not match the entered event",
            ));
        }
        self.state = LocalNotificationState::Completed {
            event_identity,
            prior_mask,
        };
        Ok(())
    }

    /// Resume the interrupted continuation after completion.
    ///
    /// # Errors
    ///
    /// Rejects resume before completion or for a different event.
    pub fn resume(&mut self, event_identity: u64) -> Result<(), SystemsModelError> {
        let LocalNotificationState::Completed {
            event_identity: completed,
            prior_mask,
        } = self.state
        else {
            return Err(interrupt_error(
                "local-notification resume requires completed entry authority",
            ));
        };
        if event_identity != completed {
            return Err(interrupt_error(
                "local-notification resume does not match the completed event",
            ));
        }
        self.state = LocalNotificationState::Resumed {
            event_identity,
            prior_mask,
        };
        Ok(())
    }

    /// Finish the wait and return the exact prior interrupt-mask state.
    ///
    /// # Errors
    ///
    /// Rejects wait completion before matching entry completion and resume.
    pub fn finish_wait(
        &mut self,
        event_identity: u64,
    ) -> Result<LocalInterruptMaskState, SystemsModelError> {
        let LocalNotificationState::Resumed {
            event_identity: resumed,
            prior_mask,
        } = self.state
        else {
            return Err(interrupt_error(
                "local-notification wait requires a resumed matching event",
            ));
        };
        if event_identity != resumed {
            return Err(interrupt_error(
                "local-notification wait does not match the resumed event",
            ));
        }
        self.state = LocalNotificationState::Idle;
        Ok(prior_mask)
    }

    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.state == LocalNotificationState::Idle
    }
}

fn interrupt_error(message: impl Into<String>) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-LOCAL-INTERRUPT", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_one_event_and_restores_exact_prior_mask() {
        for prior in [
            LocalInterruptMaskState::Enabled,
            LocalInterruptMaskState::Disabled,
        ] {
            let mut protocol = LocalNotificationProtocol::new("cpu0-local-notification");
            let event = protocol.send(prior).unwrap();
            assert_eq!(event, 1);
            protocol.enter(event).unwrap();
            protocol.complete(event).unwrap();
            protocol.resume(event).unwrap();
            assert_eq!(protocol.finish_wait(event).unwrap(), prior);
            assert!(protocol.is_idle());
        }
    }

    #[test]
    fn rejects_duplicate_unmatched_and_out_of_order_transitions() {
        let mut protocol = LocalNotificationProtocol::new("cpu0-local-notification");
        let event = protocol.send(LocalInterruptMaskState::Disabled).unwrap();
        assert!(protocol.send(LocalInterruptMaskState::Enabled).is_err());
        assert!(protocol.enter(event + 1).is_err());
        protocol.enter(event).unwrap();
        assert!(protocol.resume(event).is_err());
        protocol.complete(event).unwrap();
        assert!(protocol.complete(event).is_err());
        protocol.resume(event).unwrap();
        assert!(protocol.finish_wait(event + 1).is_err());
        assert_eq!(
            protocol.finish_wait(event).unwrap(),
            LocalInterruptMaskState::Disabled
        );
    }
}
