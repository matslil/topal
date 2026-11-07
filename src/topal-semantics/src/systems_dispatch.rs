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
    ReadAt,
    WriteAt,
    Unlink,
    UnlinkAt,
    TruncateFile,
    FileCreationMask,
    ChangeMode,
    ChangeModeByFile,
    ChangeModeAt,
    ChangeModeAtFlags,
    Rename,
    Link,
    RenameAt,
    LinkAt,
    RenameAtFlags,
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
    pub const fn from_selector_v6(selector: u64) -> Self {
        match selector {
            82 => Self::Rename,
            86 => Self::Link,
            264 => Self::RenameAt,
            265 => Self::LinkAt,
            316 => Self::RenameAtFlags,
            _ => Self::from_selector_v5(selector),
        }
    }
    #[must_use]
    pub const fn from_selector_v5(selector: u64) -> Self {
        match selector {
            90 => Self::ChangeMode,
            91 => Self::ChangeModeByFile,
            268 => Self::ChangeModeAt,
            452 => Self::ChangeModeAtFlags,
            _ => Self::from_selector_v4(selector),
        }
    }
    #[must_use]
    pub const fn from_selector_v4(selector: u64) -> Self {
        match selector {
            17 => Self::ReadAt,
            18 => Self::WriteAt,
            _ => Self::from_selector_v3(selector),
        }
    }
    pub const fn from_selector_v3(selector: u64) -> Self {
        match selector {
            87 => Self::Unlink,
            263 => Self::UnlinkAt,
            _ => Self::from_selector_v2(selector),
        }
    }
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
            Self::ReadAt => "ReadAt",
            Self::WriteAt => "WriteAt",
            Self::Unlink => "Unlink",
            Self::UnlinkAt => "UnlinkAt",
            Self::TruncateFile => "TruncateFile",
            Self::FileCreationMask => "FileCreationMask",
            Self::ChangeMode => "ChangeMode",
            Self::ChangeModeByFile => "ChangeModeByFile",
            Self::ChangeModeAt => "ChangeModeAt",
            Self::ChangeModeAtFlags => "ChangeModeAtFlags",
            Self::Rename => "Rename",
            Self::Link => "Link",
            Self::RenameAt => "RenameAt",
            Self::LinkAt => "LinkAt",
            Self::RenameAtFlags => "RenameAtFlags",
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

#[cfg(test)]
mod named_file_tests {
    use super::*;
    #[test]
    fn v3_name_removal_does_not_change_either_frozen_selector_set() {
        for selector in 0..65536 {
            let expected = match selector {
                87 => SystemsStaticLinuxOperation::Unlink,
                263 => SystemsStaticLinuxOperation::UnlinkAt,
                _ => SystemsStaticLinuxOperation::from_selector_v2(selector),
            };
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector_v3(selector),
                expected
            );
        }
        for selector in [87, 263] {
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector(selector),
                SystemsStaticLinuxOperation::Unsupported
            );
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector_v2(selector),
                SystemsStaticLinuxOperation::Unsupported
            );
            for high in [1_u64 << 32, 1_u64 << 63] {
                assert_eq!(
                    SystemsStaticLinuxOperation::from_selector_v3(high | selector),
                    SystemsStaticLinuxOperation::Unsupported
                );
            }
        }
    }
}

#[cfg(test)]
mod positional_tests {
    use super::*;
    #[test]
    fn v4_only_adds_positional_operations_and_preserves_all_frozen_sets() {
        for selector in 0..65536 {
            let expected = match selector {
                17 => SystemsStaticLinuxOperation::ReadAt,
                18 => SystemsStaticLinuxOperation::WriteAt,
                _ => SystemsStaticLinuxOperation::from_selector_v3(selector),
            };
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector_v4(selector),
                expected
            );
        }
        for selector in [17, 18] {
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector(selector),
                SystemsStaticLinuxOperation::Unsupported
            );
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector_v2(selector),
                SystemsStaticLinuxOperation::Unsupported
            );
            assert_eq!(
                SystemsStaticLinuxOperation::from_selector_v3(selector),
                SystemsStaticLinuxOperation::Unsupported
            );
            for high in [1_u64 << 32, 1_u64 << 63] {
                assert_eq!(
                    SystemsStaticLinuxOperation::from_selector_v4(high | selector),
                    SystemsStaticLinuxOperation::Unsupported
                );
            }
        }
    }
}

#[cfg(test)]
mod inode_tests {
    use super::SystemsStaticLinuxOperation as Op;

    #[test]
    fn v5_adds_only_the_four_mode_calls_and_preserves_full_native_selectors() {
        // TOPAL-SYSTEMS-LINUX-DISPATCH-V5-001.
        for selector in 0..65536 {
            let expected = match selector {
                90 => Op::ChangeMode,
                91 => Op::ChangeModeByFile,
                268 => Op::ChangeModeAt,
                452 => Op::ChangeModeAtFlags,
                _ => Op::from_selector_v4(selector),
            };
            assert_eq!(Op::from_selector_v5(selector), expected);
        }
        for selector in [90, 91, 268, 452] {
            for old in [
                Op::from_selector,
                Op::from_selector_v2,
                Op::from_selector_v3,
                Op::from_selector_v4,
            ] {
                assert_eq!(old(selector), Op::Unsupported);
            }
            for high in [1_u64 << 32, 1_u64 << 63] {
                assert_eq!(Op::from_selector_v5(high | selector), Op::Unsupported);
            }
        }
        assert_eq!(Op::from_selector_v5(u64::MAX), Op::Unsupported);
    }
}

#[cfg(test)]
mod namespace_tests {
    use super::SystemsStaticLinuxOperation as Op;

    #[test]
    fn v6_adds_exactly_five_namespace_choices_and_preserves_all_frozen_versions() {
        // TOPAL-SYSTEMS-NAMESPACE-ABI-001. This is selector meaning, not an
        // inode/mount grant or proof of any Linux handler's argument behavior.
        for selector in 0..65536 {
            let expected = match selector {
                82 => Op::Rename,
                86 => Op::Link,
                264 => Op::RenameAt,
                265 => Op::LinkAt,
                316 => Op::RenameAtFlags,
                _ => Op::from_selector_v5(selector),
            };
            assert_eq!(Op::from_selector_v6(selector), expected);
        }
        for selector in [82, 86, 264, 265, 316] {
            for old in [
                Op::from_selector,
                Op::from_selector_v2,
                Op::from_selector_v3,
                Op::from_selector_v4,
                Op::from_selector_v5,
            ] {
                assert_eq!(old(selector), Op::Unsupported);
            }
        }
        for selector in 0..65536 {
            for high in [1_u64 << 32, 1_u64 << 63] {
                assert_eq!(Op::from_selector_v6(high | selector), Op::Unsupported);
            }
        }
        assert_eq!(Op::from_selector_v6(u64::MAX), Op::Unsupported);
        assert_eq!(Op::RenameAtFlags.as_str(), "RenameAtFlags");
    }
}
