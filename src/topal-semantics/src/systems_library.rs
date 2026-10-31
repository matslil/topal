//! Static source-library obligations; no code or numeric authority is modeled.
use crate::SystemsModelError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsSourceLibraryInterface {
    BootOpenDecisionV1,
    RelativeSleepDeadlineV1,
}
impl SystemsSourceLibraryInterface {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BootOpenDecisionV1 => "boot-open-decision-v1",
            Self::RelativeSleepDeadlineV1 => "relative-sleep-deadline-v1",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsSourceLibraryRequirement {
    pub source: String,
    pub export: String,
    pub interface: SystemsSourceLibraryInterface,
    pub stack_limit_bytes: u64,
    pub runtime_limit_bytes: Option<u64>,
}
impl SystemsSourceLibraryRequirement {
    /// Check portable import metadata; source semantics remain a shared frontend
    /// obligation at publication, rather than a machine-code reference model.
    pub fn validate(&self) -> Result<(), SystemsModelError> {
        if self.source.is_empty()
            || !self.source.ends_with(".t")
            || self.source.contains('\\')
            || self.source.chars().any(char::is_control)
            || self
                .source
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            || self.export.is_empty()
            || self
                .export
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
            || !match self.interface {
                SystemsSourceLibraryInterface::BootOpenDecisionV1 => {
                    self.stack_limit_bytes == 1024 && self.runtime_limit_bytes.is_none()
                }
                SystemsSourceLibraryInterface::RelativeSleepDeadlineV1 => {
                    self.stack_limit_bytes == 4096 && self.runtime_limit_bytes == Some(65536)
                }
            }
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "source library requires a contained relative .t unit, structural public export and the selected interface's exact stack/runtime reserves",
            ));
        }
        Ok(())
    }
}

/// Independent arithmetic model; these numeric facts carry no clock authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemsSleepDeadlineFacts {
    pub seconds: u64,
    pub nanoseconds: u32,
    pub raw_ticks: u64,
    pub wrapped: bool,
}
impl SystemsSleepDeadlineFacts {
    /// Compute the source policy over the complete admitted native input range.
    pub fn deadline(self) -> Result<u64, SystemsModelError> {
        // All expressions fit u128: raw ticks have 64 bits and the duration is
        // at most signed-native MAX. This model does not mirror native shifts.
        let maximum = i64::MAX as u128;
        if u128::from(self.seconds) > maximum || self.nanoseconds >= 1_000_000_000 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "invalid relative-sleep facts",
            ));
        }
        let duration = if self.seconds >= 9_223_372_036 {
            maximum
        } else {
            u128::from(self.seconds) * 1_000_000_000 + u128::from(self.nanoseconds)
        };
        let absolute = if self.wrapped {
            maximum
        } else {
            (u128::from(self.raw_ticks) * 10 + duration).min(maximum)
        };
        Ok(absolute.div_ceil(10) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_numeric_deadline_boundaries_without_machine_authority() {
        // TOPAL-SYSTEMS-NUMERIC-RUNTIME-001, TOPAL-SYSTEMS-SLEEP-DEADLINE-LIBRARY-001
        let limit = (i64::MAX as u64).div_ceil(10);
        for (seconds, nanoseconds, raw_ticks, wrapped, expected) in [
            (0, 0, 0, false, 0),
            (0, 1, 0, false, 1),
            (0, 10, 17, false, 18),
            (0, 11, 17, false, 19),
            (0, 1, u64::MAX, false, limit),
            (9_223_372_036, 0, 0, false, limit),
            (9_223_372_035, 0, 300_000_000, false, limit),
            (0, 0, 0, true, limit),
            (i64::MAX as u64, 999_999_999, u64::MAX, true, limit),
        ] {
            assert_eq!(
                SystemsSleepDeadlineFacts {
                    seconds,
                    nanoseconds,
                    raw_ticks,
                    wrapped
                }
                .deadline()
                .unwrap(),
                expected
            );
        }
        assert!(
            SystemsSleepDeadlineFacts {
                seconds: u64::MAX,
                nanoseconds: 0,
                raw_ticks: 0,
                wrapped: false
            }
            .deadline()
            .is_err()
        );
        assert!(
            SystemsSleepDeadlineFacts {
                seconds: 0,
                nanoseconds: 1_000_000_000,
                raw_ticks: 0,
                wrapped: false
            }
            .deadline()
            .is_err()
        );
        let valid = SystemsSourceLibraryRequirement {
            source: "policy/relative-sleep-deadline.t".into(),
            export: "sleep-deadline".into(),
            interface: SystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
        };
        valid.validate().unwrap();
        for runtime in [None, Some(0), Some(65535), Some(65537)] {
            let mut wrong = valid.clone();
            wrong.runtime_limit_bytes = runtime;
            assert!(wrong.validate().is_err());
        }
    }

    #[test]
    fn retains_import_obligations_without_granting_code_authority() {
        // TOPAL-SYSTEMS-SOURCE-LIBRARY-001, TOPAL-SYSTEMS-KERNEL-POLICY-BINDING-001
        let valid = SystemsSourceLibraryRequirement {
            source: "policy/boot-open-access.t".into(),
            export: "boot-open-decision".into(),
            interface: SystemsSourceLibraryInterface::BootOpenDecisionV1,
            stack_limit_bytes: 1024,
            runtime_limit_bytes: None,
        };
        valid.validate().unwrap();
        assert_eq!(valid.interface.as_str(), "boot-open-decision-v1");
        for path in [
            "",
            "/policy.t",
            "../policy.t",
            "a/../policy.t",
            "a//policy.t",
            "a/./policy.t",
            "policy.ll",
            "a\\policy.t",
            "bad\0.t",
        ] {
            let mut wrong = valid.clone();
            wrong.source = path.into();
            assert_eq!(wrong.validate().unwrap_err().code, "E-SYSTEMS-LIBRARY");
        }
        for limit in [0, 1023, 1025, u64::MAX] {
            let mut wrong = valid.clone();
            wrong.stack_limit_bytes = limit;
            assert!(wrong.validate().is_err());
        }
        for export in ["", "two names", "bad\nname", "bad\0name"] {
            let mut wrong = valid.clone();
            wrong.export = export.into();
            assert!(wrong.validate().is_err());
        }
    }
}
