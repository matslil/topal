//! Semantic selections from untrusted normalized facts; no handler encoding or authority.
// TK-DEC-261; TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsStaticLinuxOperation {
    Read,
    Write,
    Open,
    Close,
    FileStatus,
    Poll,
    Seek,
    ProtectMemory,
    ProgramBreak,
    SignalAction,
    SignalMask,
    IoControl,
    Pipe,
    Yield,
    Duplicate,
    DuplicateTo,
    NanoSleep,
    ProcessId,
    Clone,
    Fork,
    Execute,
    Exit,
    Wait,
    KernelIdentity,
    FileControl,
    TruncateFile,
    FileCreationMask,
    WorkingDirectory,
    ChangeDirectory,
    ChangeDirectoryByFile,
    ResourceLimit,
    UserId,
    GroupId,
    SetUserId,
    SetGroupId,
    EffectiveUserId,
    EffectiveGroupId,
    ParentId,
    ProcessControl,
    ThreadLocal,
    ThreadId,
    RealtimeSeconds,
    DirectoryEntries,
    RegisterThreadId,
    ClockTime,
    ClockResolution,
    ClockSleep,
    ExitGroup,
    OpenAt,
    FileStatusAt,
    ReadLinkAt,
    RegisterRobustList,
    DuplicateToFlags,
    PipeFlags,
    ResourceLimits,
    RandomBytes,
    RestartableSequences,
    Unsupported,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extended_dispatch_adds_only_two_selectors_and_preserves_full_width_rejection() {
        // TOPAL-SYSTEMS-LINUX-DISPATCH-V2-001
        for selector in 0..65536 {
            let expected = match selector {
                77 => SystemsStaticLinuxOperation::TruncateFile,
                95 => SystemsStaticLinuxOperation::FileCreationMask,
                _ => SystemsStaticLinuxOperation::from_selector(selector),
            };
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector_v2(selector),
                expected
            );
        }
        for selector in [77, 95] {
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector(selector),
                SystemsStaticLinuxOperation::Unsupported
            );
            for high in [1u64 << 32, 1u64 << 63] {
                assert_eq!(
                    SystemsStaticLinuxOperation::from_selector_v2(high | selector),
                    SystemsStaticLinuxOperation::Unsupported
                );
            }
        }
    }
    #[test]
    fn normalized_selector_model_matches_normative_mapping_and_preserves_wide_facts() {
        // TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001
        let specification = include_str!("../../../spec/systems-profile.md");
        let rule = specification
            .split_once("### TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001")
            .unwrap()
            .1;
        let table = rule
            .split_once("```text\n")
            .unwrap()
            .1
            .split_once("```")
            .unwrap()
            .0;
        let expected = table
            .split(';')
            .map(str::trim)
            .filter(|row| !row.is_empty())
            .map(|row| {
                let mut words = row.split_whitespace();
                let number = words.next().unwrap().parse::<u64>().unwrap();
                let name = words.next().unwrap();
                assert!(words.next().is_none());
                (number, name)
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(expected.len(), 55);
        for selector in 0..65536 {
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector(selector).as_str(),
                *expected.get(&selector).unwrap_or(&"Unsupported"),
                "selector={selector}"
            );
        }
        for selector in expected.keys() {
            for high in [1_u64 << 32, 1_u64 << 63] {
                let raw = selector | high;
                assert_eq!(
                    SystemsStaticLinuxOperation::from_selector(raw),
                    SystemsStaticLinuxOperation::Unsupported
                );
                assert_eq!(
                    SystemsStaticLinuxOperation::from_selector(raw & 0xffff_ffff).as_str(),
                    expected[selector]
                );
            }
        }
        assert_eq!(
            SystemsStaticLinuxOperation::from_selector(u64::MAX),
            SystemsStaticLinuxOperation::Unsupported
        );
    }
}
impl SystemsStaticLinuxOperation {
    #[must_use]
    pub const fn from_selector_v2(selector: u64) -> Self {
        match selector {
            77 => Self::TruncateFile,
            95 => Self::FileCreationMask,
            _ => Self::from_selector(selector),
        }
    }
    #[must_use]
    pub const fn from_selector(selector: u64) -> Self {
        match selector {
            0 => Self::Read,
            1 => Self::Write,
            2 => Self::Open,
            3 => Self::Close,
            5 => Self::FileStatus,
            7 => Self::Poll,
            8 => Self::Seek,
            10 => Self::ProtectMemory,
            12 => Self::ProgramBreak,
            13 => Self::SignalAction,
            14 => Self::SignalMask,
            16 => Self::IoControl,
            22 => Self::Pipe,
            24 => Self::Yield,
            32 => Self::Duplicate,
            33 => Self::DuplicateTo,
            35 => Self::NanoSleep,
            39 => Self::ProcessId,
            56 => Self::Clone,
            57 => Self::Fork,
            59 => Self::Execute,
            60 => Self::Exit,
            61 => Self::Wait,
            63 => Self::KernelIdentity,
            72 => Self::FileControl,
            79 => Self::WorkingDirectory,
            80 => Self::ChangeDirectory,
            81 => Self::ChangeDirectoryByFile,
            97 => Self::ResourceLimit,
            102 => Self::UserId,
            104 => Self::GroupId,
            105 => Self::SetUserId,
            106 => Self::SetGroupId,
            107 => Self::EffectiveUserId,
            108 => Self::EffectiveGroupId,
            110 => Self::ParentId,
            157 => Self::ProcessControl,
            158 => Self::ThreadLocal,
            186 => Self::ThreadId,
            201 => Self::RealtimeSeconds,
            217 => Self::DirectoryEntries,
            218 => Self::RegisterThreadId,
            228 => Self::ClockTime,
            229 => Self::ClockResolution,
            230 => Self::ClockSleep,
            231 => Self::ExitGroup,
            257 => Self::OpenAt,
            262 => Self::FileStatusAt,
            267 => Self::ReadLinkAt,
            273 => Self::RegisterRobustList,
            292 => Self::DuplicateToFlags,
            293 => Self::PipeFlags,
            302 => Self::ResourceLimits,
            318 => Self::RandomBytes,
            334 => Self::RestartableSequences,
            _ => Self::Unsupported,
        }
    }
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "Read",
            Self::Write => "Write",
            Self::Open => "Open",
            Self::Close => "Close",
            Self::FileStatus => "FileStatus",
            Self::Poll => "Poll",
            Self::Seek => "Seek",
            Self::ProtectMemory => "ProtectMemory",
            Self::ProgramBreak => "ProgramBreak",
            Self::SignalAction => "SignalAction",
            Self::SignalMask => "SignalMask",
            Self::IoControl => "IoControl",
            Self::Pipe => "Pipe",
            Self::Yield => "Yield",
            Self::Duplicate => "Duplicate",
            Self::DuplicateTo => "DuplicateTo",
            Self::NanoSleep => "NanoSleep",
            Self::ProcessId => "ProcessId",
            Self::Clone => "Clone",
            Self::Fork => "Fork",
            Self::Execute => "Execute",
            Self::Exit => "Exit",
            Self::Wait => "Wait",
            Self::KernelIdentity => "KernelIdentity",
            Self::FileControl => "FileControl",
            Self::TruncateFile => "TruncateFile",
            Self::FileCreationMask => "FileCreationMask",
            Self::WorkingDirectory => "WorkingDirectory",
            Self::ChangeDirectory => "ChangeDirectory",
            Self::ChangeDirectoryByFile => "ChangeDirectoryByFile",
            Self::ResourceLimit => "ResourceLimit",
            Self::UserId => "UserId",
            Self::GroupId => "GroupId",
            Self::SetUserId => "SetUserId",
            Self::SetGroupId => "SetGroupId",
            Self::EffectiveUserId => "EffectiveUserId",
            Self::EffectiveGroupId => "EffectiveGroupId",
            Self::ParentId => "ParentId",
            Self::ProcessControl => "ProcessControl",
            Self::ThreadLocal => "ThreadLocal",
            Self::ThreadId => "ThreadId",
            Self::RealtimeSeconds => "RealtimeSeconds",
            Self::DirectoryEntries => "DirectoryEntries",
            Self::RegisterThreadId => "RegisterThreadId",
            Self::ClockTime => "ClockTime",
            Self::ClockResolution => "ClockResolution",
            Self::ClockSleep => "ClockSleep",
            Self::ExitGroup => "ExitGroup",
            Self::OpenAt => "OpenAt",
            Self::FileStatusAt => "FileStatusAt",
            Self::ReadLinkAt => "ReadLinkAt",
            Self::RegisterRobustList => "RegisterRobustList",
            Self::DuplicateToFlags => "DuplicateToFlags",
            Self::PipeFlags => "PipeFlags",
            Self::ResourceLimits => "ResourceLimits",
            Self::RandomBytes => "RandomBytes",
            Self::RestartableSequences => "RestartableSequences",
            Self::Unsupported => "Unsupported",
        }
    }
}
