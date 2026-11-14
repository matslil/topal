//! Versioned pure finite policy contracts, independent of native Enum order.
use super::{CompileError, invalid};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinitePolicy {
    BootOpen,
    FileOpen,
}
impl FinitePolicy {
    pub(crate) fn from_interface(name: &str) -> Result<Self, CompileError> {
        match name {
            "boot-open-decision-v1" => Ok(Self::BootOpen),
            "file-open-decision-v1" => Ok(Self::FileOpen),
            _ => Err(invalid("unqualified finite policy interface")),
        }
    }
    pub(crate) const fn arguments(self) -> usize {
        match self {
            Self::BootOpen => 3,
            Self::FileOpen => 8,
        }
    }
    pub(crate) const fn export(self) -> &'static str {
        match self {
            Self::BootOpen => super::EXPORT_SYMBOL,
            Self::FileOpen => "topal_kernel_file_open_policy_v1",
        }
    }
    pub(crate) const fn classifier(self) -> &'static str {
        match self {
            Self::BootOpen => "BootOpenDecision",
            Self::FileOpen => "FileOpenDecision",
        }
    }
    pub(crate) const fn choices(self) -> &'static [&'static str] {
        match self {
            Self::BootOpen => &super::CHOICES,
            Self::FileOpen => &[
                "OpenDescription",
                "NotDirectory",
                "DirectoryWrite",
                "ReadOnlyView",
                "Missing",
                "AlreadyExists",
                "CreateDescription",
                "TruncateDescription",
            ],
        }
    }
}
