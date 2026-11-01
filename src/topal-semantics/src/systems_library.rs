//! Static source-library obligations; no code or numeric authority is modeled.
use crate::SystemsModelError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SystemsSourceLibraryInterface {
    BootOpenDecisionV1,
    RelativeSleepDeadlineV1,
    StaticElf64HeaderV1,
    StaticLinuxDispatchV1,
    ProcessIdentityTransitionV1,
    RetainedProcessIdentityV1,
    StructuredProcessIdentityV1,
}
impl SystemsSourceLibraryInterface {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BootOpenDecisionV1 => "boot-open-decision-v1",
            Self::RelativeSleepDeadlineV1 => "relative-sleep-deadline-v1",
            Self::StaticElf64HeaderV1 => "static-elf64-header-v1",
            Self::StaticLinuxDispatchV1 => "static-linux-dispatch-v1",
            Self::ProcessIdentityTransitionV1 => "process-identity-transition-v1",
            Self::RetainedProcessIdentityV1 => "retained-process-identity-v1",
            Self::StructuredProcessIdentityV1 => "structured-process-identity-v1",
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
    pub state_limit_bytes: Option<u64>,
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
                    self.stack_limit_bytes == 1024
                        && self.runtime_limit_bytes.is_none()
                        && self.state_limit_bytes.is_none()
                }
                SystemsSourceLibraryInterface::RelativeSleepDeadlineV1
                | SystemsSourceLibraryInterface::StaticElf64HeaderV1
                | SystemsSourceLibraryInterface::StaticLinuxDispatchV1
                | SystemsSourceLibraryInterface::ProcessIdentityTransitionV1 => {
                    self.stack_limit_bytes == 4096
                        && self.runtime_limit_bytes == Some(65536)
                        && self.state_limit_bytes.is_none()
                }
                SystemsSourceLibraryInterface::RetainedProcessIdentityV1
                | SystemsSourceLibraryInterface::StructuredProcessIdentityV1 => {
                    self.stack_limit_bytes == 4096
                        && self.runtime_limit_bytes == Some(65536)
                        && self.state_limit_bytes == Some(4096)
                }
            }
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "source library requires a contained relative .t unit, structural public export and the selected interface's exact stack/runtime/state reserves",
            ));
        }
        Ok(())
    }
}

/// Copied protocol facts, never a file, addressed storage or mapping grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsElf64HeaderFacts {
    pub bytes: [u8; 64],
    pub machine: u16,
    pub maximum_program_headers: u16,
}
impl SystemsElf64HeaderFacts {
    pub fn admitted(&self) -> Result<bool, SystemsModelError> {
        if self.machine == 0 || self.maximum_program_headers == 0 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "invalid ELF64 header facts",
            ));
        }
        let word = |offset| u16::from_le_bytes(self.bytes[offset..offset + 2].try_into().unwrap());
        let version = u32::from_le_bytes(self.bytes[20..24].try_into().unwrap());
        Ok(self.bytes[..7] == [127, 69, 76, 70, 2, 1, 1]
            && word(16) == 2
            && word(18) == self.machine
            && version == 1
            && word(52) == 64
            && word(54) == 56
            && (1..=self.maximum_program_headers).contains(&word(56)))
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
    fn copied_elf64_header_facts_preserve_exact_admission_and_unexamined_bytes() {
        // TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001
        let mut header = SystemsElf64HeaderFacts {
            bytes: [0; 64],
            machine: 62,
            maximum_program_headers: 16,
        };
        header.bytes[..7].copy_from_slice(&[127, 69, 76, 70, 2, 1, 1]);
        for (offset, value) in [(16, 2u16), (18, 62), (52, 64), (54, 56), (56, 4)] {
            header.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
        header.bytes[20..24].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(header.admitted(), Ok(true));
        for position in 0..64 {
            for value in 0..=255 {
                let mut changed = header.clone();
                changed.bytes[position] = value;
                let expected = match position {
                    0..=6 | 16..=23 | 52..=55 => value == header.bytes[position],
                    56 => (1..=16).contains(&value),
                    57 => value == 0,
                    _ => true,
                };
                assert_eq!(
                    changed.admitted(),
                    Ok(expected),
                    "position={position} value={value}"
                );
            }
        }
        let mut other = header.clone();
        other.machine = u16::MAX;
        other.maximum_program_headers = u16::MAX;
        other.bytes[18..20].copy_from_slice(&u16::MAX.to_le_bytes());
        other.bytes[56..58].copy_from_slice(&u16::MAX.to_le_bytes());
        assert_eq!(other.admitted(), Ok(true));
        other.bytes[56..58].copy_from_slice(&0u16.to_le_bytes());
        assert_eq!(other.admitted(), Ok(false));
        for (machine, maximum) in [(0, 16), (62, 0)] {
            let mut invalid = header.clone();
            invalid.machine = machine;
            invalid.maximum_program_headers = maximum;
            assert!(invalid.admitted().is_err());
        }
        let requirement = SystemsSourceLibraryRequirement {
            source: "policy/header.t".into(),
            export: "header-admitted?".into(),
            interface: SystemsSourceLibraryInterface::StaticElf64HeaderV1,
            stack_limit_bytes: 4096,
            runtime_limit_bytes: Some(65536),
            state_limit_bytes: None,
        };
        requirement.validate().unwrap();
        assert_eq!(requirement.interface.as_str(), "static-elf64-header-v1");
    }

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
            state_limit_bytes: None,
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
            state_limit_bytes: None,
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
