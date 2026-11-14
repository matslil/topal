//! TOPAL-SYSTEMS-FILE-OPEN-PLAN-001: copied facts carry no storage authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemsFileOpenFacts {
    pub write: bool,
    pub directory: bool,
    pub directory_required: bool,
    pub mutable: bool,
    pub exists: bool,
    pub create: bool,
    pub exclusive: bool,
    pub truncate: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsFileOpenPlan {
    OpenDescription,
    NotDirectory,
    DirectoryWrite,
    ReadOnlyView,
    Missing,
    AlreadyExists,
    CreateDescription,
    TruncateDescription,
}
impl SystemsFileOpenFacts {
    #[must_use]
    pub fn plan(self) -> SystemsFileOpenPlan {
        use SystemsFileOpenPlan::*;
        match self {
            Self {
                exists: false,
                create: false,
                ..
            } => Missing,
            Self {
                exists: false,
                mutable: false,
                ..
            } => ReadOnlyView,
            Self {
                exists: false,
                directory_required: true,
                ..
            } => NotDirectory,
            Self { exists: false, .. } => CreateDescription,
            Self {
                create: true,
                exclusive: true,
                ..
            } => AlreadyExists,
            Self {
                directory: false,
                directory_required: true,
                ..
            } => NotDirectory,
            Self {
                directory: true,
                write,
                create,
                truncate,
                ..
            } if write || create || truncate => DirectoryWrite,
            Self {
                directory: false,
                mutable: false,
                write,
                truncate,
                ..
            } if write || truncate => ReadOnlyView,
            Self {
                directory: false,
                truncate: true,
                ..
            } => TruncateDescription,
            _ => OpenDescription,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_error_precedence_and_truncate_access_are_distinct() {
        use SystemsFileOpenPlan::*;
        let mut f = SystemsFileOpenFacts {
            write: false,
            directory: false,
            directory_required: false,
            mutable: false,
            exists: true,
            create: false,
            exclusive: false,
            truncate: false,
        };
        assert_eq!(f.plan(), OpenDescription);
        f.exclusive = true; // O_EXCL alone has no disposition effect.
        assert_eq!(f.plan(), OpenDescription);
        f.create = true;
        assert_eq!(f.plan(), AlreadyExists); // before EROFS and type checks.
        f.directory_required = true;
        assert_eq!(f.plan(), AlreadyExists);
        f.exclusive = false;
        assert_eq!(f.plan(), NotDirectory);
        f.directory = true;
        assert_eq!(f.plan(), DirectoryWrite); // even a read-only O_CREAT.
        f.create = false;
        assert_eq!(f.plan(), OpenDescription);
        f.truncate = true;
        assert_eq!(f.plan(), DirectoryWrite);
        f.directory = false;
        f.directory_required = false;
        assert_eq!(f.plan(), ReadOnlyView);
        f.mutable = true;
        assert_eq!(f.plan(), TruncateDescription); // no write access requested.
        f.exists = false;
        assert_eq!(f.plan(), Missing);
        f.create = true;
        assert_eq!(f.plan(), CreateDescription);
        f.directory_required = true;
        assert_eq!(f.plan(), NotDirectory);
        f.mutable = false;
        assert_eq!(f.plan(), ReadOnlyView);
    }
    #[test]
    fn extended_interface_keeps_the_stack_only_import_obligation() {
        use crate::{SystemsSourceLibraryInterface as I, SystemsSourceLibraryRequirement as Q};
        let mut q = Q {
            source: "policy/file-open-access.t".into(),
            export: "file-open-decision".into(),
            interface: I::FileOpenDecisionV1,
            stack_limit_bytes: 1024,
            runtime_limit_bytes: None,
            state_limit_bytes: None,
        };
        assert_eq!(q.interface.as_str(), "file-open-decision-v1");
        q.validate().unwrap();
        q.runtime_limit_bytes = Some(65536);
        assert!(q.validate().is_err());
        q.runtime_limit_bytes = None;
        q.state_limit_bytes = Some(4096);
        assert!(q.validate().is_err());
    }
}
