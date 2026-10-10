//! Architecture-independent monotonic-clock observation protocol.

use crate::{ClockIdentity, Instant, MonotonicClock, SystemsModelError};

pub const SYSTEMS_MONOTONIC_CLOCK_NOW: &str = "topal.systems.time.monotonic.now/1";
pub const INITIAL_MONOTONIC_CLOCK_IDENTITY: u64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonotonicClockObservation {
    pub observation_identity: u64,
    pub instant: Instant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsMonotonicClock {
    clock: MonotonicClock,
    next_observation_identity: u64,
}

impl SystemsMonotonicClock {
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            clock: MonotonicClock::new(ClockIdentity(INITIAL_MONOTONIC_CLOCK_IDENTITY)),
            next_observation_identity: 1,
        }
    }

    /// Accept one provider observation and assign its source-order identity.
    ///
    /// # Errors
    ///
    /// Returns a stable systems diagnostic when the provider value decreases
    /// or the observation identity is exhausted.
    pub fn observe(
        &mut self,
        provider_ticks: u128,
    ) -> Result<MonotonicClockObservation, SystemsModelError> {
        let instant = self
            .clock
            .observe(provider_ticks)
            .map_err(|message| SystemsModelError::new("E-SYSTEMS-MONOTONIC-CLOCK", message))?;
        let observation_identity = self.next_observation_identity;
        self.next_observation_identity = observation_identity.checked_add(1).ok_or_else(|| {
            SystemsModelError::new(
                "E-SYSTEMS-MONOTONIC-CLOCK",
                "monotonic-clock observation identity exhausted",
            )
        })?;
        Ok(MonotonicClockObservation {
            observation_identity,
            instant,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_ordered_same_clock_observations_and_accepts_equality() {
        let mut clock = SystemsMonotonicClock::initial();
        let first = clock.observe(41).unwrap();
        let second = clock.observe(41).unwrap();
        let third = clock.observe(42).unwrap();

        assert_eq!(first.observation_identity, 1);
        assert_eq!(second.observation_identity, 2);
        assert_eq!(third.observation_identity, 3);
        assert_eq!(first.instant.clock, second.instant.clock);
        assert_eq!(second.instant.ticks, 41);
        assert_eq!(third.instant.ticks, 42);
    }

    #[test]
    fn rejects_regression_without_accepting_an_observation() {
        let mut clock = SystemsMonotonicClock::initial();
        assert_eq!(clock.observe(9).unwrap().observation_identity, 1);
        let error = clock.observe(8).unwrap_err();
        assert_eq!(error.code, "E-SYSTEMS-MONOTONIC-CLOCK");
        assert_eq!(clock.observe(10).unwrap().observation_identity, 2);
    }
}
