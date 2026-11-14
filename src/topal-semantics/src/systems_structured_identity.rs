//! Independent semantic history; numeric equality confers no owner authority.
use crate::{SystemsModelError, SystemsProcessIdentityFacts};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemsStructuredIdentityFacts {
    pub next: u64,
    pub creations: u64,
    pub wrapped: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemsStructuredIdentityTransition {
    pub allocated: u64,
    pub state: SystemsStructuredIdentityFacts,
}
impl SystemsStructuredIdentityFacts {
    pub const INITIAL: Self = Self {
        next: 0,
        creations: 0,
        wrapped: false,
    };

    pub fn validate_published(self) -> Result<(), SystemsModelError> {
        if !(2..=i32::MAX as u64).contains(&self.next) || self.creations == 0 {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn validate_invocable(self) -> Result<(), SystemsModelError> {
        if self != Self::INITIAL {
            self.validate_published()?;
        }
        if self.creations == u64::MAX {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn transition(self) -> Result<SystemsStructuredIdentityTransition, SystemsModelError> {
        self.validate_invocable()?;
        let creations = self.creations.checked_add(1).ok_or_else(invalid)?;
        let selected = SystemsProcessIdentityFacts { next: self.next }.transition()?;
        Ok(SystemsStructuredIdentityTransition {
            allocated: selected.allocated,
            state: Self {
                next: selected.next,
                creations,
                wrapped: self.wrapped || selected.allocated == i32::MAX as u64,
            },
        })
    }
}
fn invalid() -> SystemsModelError {
    SystemsModelError::new(
        "E-SYSTEMS-LIBRARY",
        "invalid or exhausted structured process identity history",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structured_history_retains_full_count_and_sticky_wrap_value_law() {
        // TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001
        let mut state = SystemsStructuredIdentityFacts::INITIAL;
        for expected in 2..65536 {
            let transition = state.transition().unwrap();
            assert_eq!(transition.allocated, expected);
            assert_eq!(transition.state.creations, expected - 1);
            assert!(!transition.state.wrapped);
            transition.state.validate_published().unwrap();
            state = transition.state;
        }
        for count in [
            1,
            u32::MAX as u64 - 1,
            u32::MAX as u64,
            u32::MAX as u64 + 1,
            u64::MAX - 1,
        ] {
            for cursor in [2, i32::MAX as u64 - 1, i32::MAX as u64] {
                for wrapped in [false, true] {
                    let previous = SystemsStructuredIdentityFacts {
                        next: cursor,
                        creations: count,
                        wrapped,
                    };
                    let transition = previous.transition().unwrap();
                    assert_eq!(transition.allocated, cursor);
                    assert_eq!(transition.state.creations, count + 1);
                    assert_eq!(
                        transition.state.wrapped,
                        wrapped || cursor == i32::MAX as u64
                    );
                    assert_eq!(
                        transition.state.next,
                        if cursor == i32::MAX as u64 {
                            2
                        } else {
                            cursor + 1
                        }
                    );
                    transition.state.validate_published().unwrap();
                }
            }
        }
        let wrapped = SystemsStructuredIdentityFacts {
            next: i32::MAX as u64,
            creations: 7,
            wrapped: false,
        }
        .transition()
        .unwrap()
        .state;
        assert_eq!(wrapped.next, 2);
        assert!(wrapped.wrapped && wrapped.transition().unwrap().state.wrapped);
    }
    #[test]
    fn malformed_initial_history_and_exhaustion_never_produce_a_proposal() {
        // TOPAL-SYSTEMS-RETAINED-STRUCTURED-STATE-001; TOPAL-SYSTEMS-STRUCTURED-PROCESS-IDENTITY-001
        for facts in [
            SystemsStructuredIdentityFacts {
                next: 0,
                creations: 0,
                wrapped: true,
            },
            SystemsStructuredIdentityFacts {
                next: 0,
                creations: 1,
                wrapped: false,
            },
            SystemsStructuredIdentityFacts {
                next: 2,
                creations: 0,
                wrapped: false,
            },
            SystemsStructuredIdentityFacts {
                next: 1,
                creations: 1,
                wrapped: false,
            },
            SystemsStructuredIdentityFacts {
                next: 2147483648,
                creations: 1,
                wrapped: false,
            },
            SystemsStructuredIdentityFacts {
                next: u64::MAX,
                creations: 1,
                wrapped: false,
            },
            SystemsStructuredIdentityFacts {
                next: 2,
                creations: u64::MAX,
                wrapped: true,
            },
        ] {
            let before = facts;
            assert!(facts.transition().is_err());
            assert_eq!(facts, before);
        }
        let exhausted = SystemsStructuredIdentityFacts {
            next: 2,
            creations: u64::MAX,
            wrapped: true,
        };
        exhausted.validate_published().unwrap();
        assert!(exhausted.transition().is_err());
    }
}
