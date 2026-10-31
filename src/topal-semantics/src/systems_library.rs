//! Static source-library obligations; no code or numeric authority is modeled.
use crate::SystemsModelError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsSourceLibraryInterface {
    BootOpenDecisionV1,
}
impl SystemsSourceLibraryInterface {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BootOpenDecisionV1 => "boot-open-decision-v1",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsSourceLibraryRequirement {
    pub source: String,
    pub export: String,
    pub interface: SystemsSourceLibraryInterface,
    pub stack_limit_bytes: u64,
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
            || self.stack_limit_bytes != 1024
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "source library requires a contained relative .t unit, structural public export and 1024-byte stack reserve",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_import_obligations_without_granting_code_authority() {
        // TOPAL-SYSTEMS-SOURCE-LIBRARY-001, TOPAL-SYSTEMS-KERNEL-POLICY-BINDING-001
        let valid = SystemsSourceLibraryRequirement {
            source: "policy/boot-open-access.t".into(),
            export: "boot-open-decision".into(),
            interface: SystemsSourceLibraryInterface::BootOpenDecisionV1,
            stack_limit_bytes: 1024,
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
