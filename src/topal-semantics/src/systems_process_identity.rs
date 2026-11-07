//! Immutable identity facts carry no process ownership or native authority.
use crate::SystemsModelError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemsProcessIdentityFacts {
    pub next: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemsProcessIdentityTransition {
    pub allocated: u64,
    pub next: u64,
}
impl SystemsProcessIdentityFacts {
    pub fn transition(self) -> Result<SystemsProcessIdentityTransition, SystemsModelError> {
        if self.next == 1 || self.next > i32::MAX as u64 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "invalid copied process identity cursor",
            ));
        }
        let allocated = self.next.max(2);
        let next = if allocated < i32::MAX as u64 {
            allocated + 1
        } else {
            2
        };
        Ok(SystemsProcessIdentityTransition { allocated, next })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_facts_cover_initial_increment_wrap_and_invalid_private_cursors() {
        // TOPAL-SYSTEMS-PROCESS-IDENTITY-LIBRARY-001
        for (input, allocated, next) in [
            (0, 2, 3),
            (2, 2, 3),
            (3, 3, 4),
            (2147483646, 2147483646, 2147483647),
            (2147483647, 2147483647, 2),
        ] {
            assert_eq!(
                SystemsProcessIdentityFacts { next: input }.transition(),
                Ok(SystemsProcessIdentityTransition { allocated, next })
            );
        }
        for next in [1, 2147483648, u64::MAX] {
            assert!(SystemsProcessIdentityFacts { next }.transition().is_err());
        }
        let mut current = 0;
        for expected in 2..65536 {
            let transition = SystemsProcessIdentityFacts { next: current }
                .transition()
                .unwrap();
            assert_eq!(transition.allocated, expected);
            current = transition.next;
        }
    }
}
