//! Architecture-independent one-shot deadline-event protocol.

use crate::{ClockIdentity, Deadline, Duration, Instant, SystemsModelError};

pub const SYSTEMS_DEADLINE_AFTER: &str = "topal.systems.time.deadline.after/2";
pub const SYSTEMS_DEADLINE_ARM: &str = "topal.systems.time.deadline.arm/2";
pub const SYSTEMS_DEADLINE_WAIT: &str = "topal.systems.time.deadline.wait/2";
pub const SYSTEMS_DEADLINE_COMPLETE: &str = "topal.systems.time.deadline.complete/1";
pub const SYSTEMS_RESUME_DEADLINE: &str = "topal.systems.disposition.resume-deadline/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeadlineEventDelivery {
    pub event_identity: u64,
    pub scheduled: Instant,
    pub observed: Instant,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ArmedDeadline {
    event_identity: u64,
    scheduled: Instant,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeadlineInterruptContext {
    delivery: DeadlineEventDelivery,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompletedDeadlineInterrupt {
    delivery: DeadlineEventDelivery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsDeadlineProtocol {
    clock: ClockIdentity,
    next_event_identity: u64,
}

impl SystemsDeadlineProtocol {
    #[must_use]
    pub const fn new(clock: ClockIdentity) -> Self {
        Self {
            clock,
            next_event_identity: 1,
        }
    }

    /// Construct one absolute same-clock deadline without observing the clock.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a wrong clock, zero duration, or range
    /// overflow.
    pub fn deadline_after(
        &self,
        instant: Instant,
        duration: Duration,
    ) -> Result<Deadline, SystemsModelError> {
        if instant.clock != self.clock {
            return Err(deadline_error(
                "a deadline must retain the protocol's exact clock identity",
            ));
        }
        if duration.0 == 0 {
            return Err(deadline_error(
                "the initial deadline duration must be positive",
            ));
        }
        instant
            .checked_add(duration)
            .map(|instant| Deadline { instant })
            .map_err(deadline_error)
    }

    /// Consume one absolute deadline into a fresh affine event.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for a wrong clock or exhausted event
    /// identity. Whether it is already expired affects provider delivery, not
    /// the stored scheduled instant.
    pub fn arm(&mut self, deadline: Deadline) -> Result<ArmedDeadline, SystemsModelError> {
        if deadline.instant.clock != self.clock {
            return Err(deadline_error(
                "a deadline event cannot arm a deadline from another clock",
            ));
        }
        let event_identity = self.next_event_identity;
        self.next_event_identity = event_identity
            .checked_add(1)
            .ok_or_else(|| deadline_error("deadline-event identity exhausted"))?;
        Ok(ArmedDeadline {
            event_identity,
            scheduled: deadline.instant,
        })
    }

    /// Deliver one armed event at a newly accepted same-clock instant.
    ///
    /// # Errors
    ///
    /// Delivery from another clock or before the scheduled instant is
    /// rejected. Equality represents immediate delivery of an expired
    /// deadline; later instants preserve lateness.
    pub fn deliver(
        &self,
        armed: ArmedDeadline,
        observed: Instant,
    ) -> Result<DeadlineInterruptContext, SystemsModelError> {
        if observed.clock != self.clock {
            return Err(deadline_error(
                "deadline delivery must observe the scheduled clock",
            ));
        }
        if observed.ticks < armed.scheduled.ticks {
            return Err(deadline_error(
                "deadline delivery cannot precede its scheduled instant",
            ));
        }
        Ok(DeadlineInterruptContext {
            delivery: DeadlineEventDelivery {
                event_identity: armed.event_identity,
                scheduled: armed.scheduled,
                observed,
            },
        })
    }

    #[must_use]
    pub fn complete(&self, context: DeadlineInterruptContext) -> CompletedDeadlineInterrupt {
        CompletedDeadlineInterrupt {
            delivery: context.delivery,
        }
    }

    #[must_use]
    pub fn resume(&self, completed: CompletedDeadlineInterrupt) -> DeadlineEventDelivery {
        completed.delivery
    }
}

fn deadline_error(message: impl Into<String>) -> SystemsModelError {
    SystemsModelError::new("E-SYSTEMS-DEADLINE-EVENT", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOCK: ClockIdentity = ClockIdentity(1);

    fn instant(ticks: u128) -> Instant {
        Instant {
            clock: CLOCK,
            ticks,
        }
    }

    #[test]
    fn retains_absolute_schedule_and_late_delivery() {
        // TOPAL-SYSTEMS-DEADLINE-EVENT-001.
        let mut protocol = SystemsDeadlineProtocol::new(CLOCK);
        let deadline = protocol.deadline_after(instant(10), Duration(5)).unwrap();
        let armed = protocol.arm(deadline).unwrap();
        let context = protocol.deliver(armed, instant(19)).unwrap();
        let delivery = protocol.resume(protocol.complete(context));

        assert_eq!(delivery.event_identity, 1);
        assert_eq!(delivery.scheduled, instant(15));
        assert_eq!(delivery.observed, instant(19));
    }

    #[test]
    fn already_expired_deadline_keeps_original_schedule() {
        // TOPAL-SYSTEMS-DEADLINE-EVENT-001.
        let mut protocol = SystemsDeadlineProtocol::new(CLOCK);
        let deadline = protocol.deadline_after(instant(10), Duration(5)).unwrap();
        let armed = protocol.arm(deadline).unwrap();
        let delivery =
            protocol.resume(protocol.complete(protocol.deliver(armed, instant(30)).unwrap()));

        assert_eq!(delivery.scheduled, instant(15));
        assert_eq!(delivery.observed, instant(30));
    }

    #[test]
    fn rejects_wrong_clock_early_delivery_and_overflow() {
        // TOPAL-SYSTEMS-DEADLINE-EVENT-001.
        let mut protocol = SystemsDeadlineProtocol::new(CLOCK);
        let wrong = Instant {
            clock: ClockIdentity(2),
            ticks: 10,
        };
        assert_eq!(
            protocol
                .deadline_after(wrong, Duration(1))
                .unwrap_err()
                .code,
            "E-SYSTEMS-DEADLINE-EVENT"
        );
        assert_eq!(
            protocol
                .deadline_after(instant(u128::MAX), Duration(1))
                .unwrap_err()
                .code,
            "E-SYSTEMS-DEADLINE-EVENT"
        );

        let deadline = protocol.deadline_after(instant(10), Duration(5)).unwrap();
        let armed = protocol.arm(deadline).unwrap();
        assert_eq!(
            protocol.deliver(armed, instant(14)).unwrap_err().code,
            "E-SYSTEMS-DEADLINE-EVENT"
        );
    }
}
