fn audit_provider_user_completion(
    file: &object::File<'_>,
    data: &[u8],
    entry_payload: &InitialUserEntryPayload,
) {
    let user_syscall = symbol_bytes(&file, X86_SYSTEMS_USER_SYSCALL_SYMBOL);
    assert!(
        user_syscall
            .windows(3)
            .any(|bytes| bytes == [0x83, 0xf8, 0x09])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x0a])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x0b])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x1c])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x18])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x3d, 0xe4, 0, 0, 0])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x3d, 0xe5, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x48, 0xc7, 0x44, 0x24, 0x08, 0x0a, 0, 0, 0])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xda, 0xff, 0xff, 0xff])
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == 0x0060_6000_u64.to_le_bytes())
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes())
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == 0x8000_0000_0000_0005_u64.to_le_bytes())
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0xc7, 0x02, 0, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0, 0, 0]),
        "the userspace adapter must retain bounded VM lifecycle, clock dispatch, and unsupported-syscall failure"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x18, 0, 0, 0, 0x0f, 0x05])
            .count(),
        1,
        "constructed code must prove one successful sole-runnable yield"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0xe4, 0, 0, 0, 0x0f, 0x05])
            .count(),
        2,
        "constructed code must prove two monotonic clock observations"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0xe5, 0, 0, 0, 0x0f, 0x05])
            .count(),
        2,
        "constructed code must prove exact and null monotonic clock-resolution queries"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0xff, 0xff, 0xff, 0x7f, 0x0f, 0x05])
            .count(),
        1,
        "constructed code must prove one recoverable unsupported-syscall result"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x1c, 0, 0, 0, 0x0f, 0x05])
            .count(),
        2,
        "constructed code must prove inaccessible discard and rejected advice"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x0a, 0, 0, 0, 0x0f, 0x05])
            .count(),
        4,
        "constructed code must prove read, none, read-write, and rejected executable protection"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x09, 0, 0, 0, 0x0f, 0x05])
            .count(),
        3,
        "constructed code must prove first map, active-map rejection, and remap"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x0b, 0, 0, 0, 0x0f, 0x05])
            .count(),
        2,
        "constructed code must release both successful mappings"
    );
    assert_eq!(
        user_syscall
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0x01, 0xf8])
            .count(),
        3,
        "syscall entry must swap GS on entry and both exits"
    );
    assert!(
        user_syscall
            .windows(3)
            .any(|bytes| bytes == [0x48, 0x0f, 0x07]),
        "returning user syscalls must use SYSRETQ"
    );
    assert!(
        user_syscall
            .windows(4)
            .any(|bytes| bytes == 0x0060_0007_u32.to_le_bytes())
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]),
        "scalar identity calls must retain the provider probe and admit finalized executable continuations"
    );
    for (number, expected) in [
        (39_u32, 1_u8),
        (102, 0),
        (104, 0),
        (107, 0),
        (108, 0),
        (110, 0),
        (186, 1),
    ] {
        let mut invocation = [0_u8; 7];
        invocation[0] = 0xb8;
        invocation[1..5].copy_from_slice(&number.to_le_bytes());
        invocation[5..7].copy_from_slice(&[0x0f, 0x05]);
        let invocation_position = entry_payload
            .bytes
            .windows(7)
            .position(|bytes| bytes == invocation)
            .expect("constructed image must invoke every scalar identity syscall");
        if expected == 0 {
            assert_eq!(
                &entry_payload.bytes[invocation_position + 7..invocation_position + 10],
                &[0x48, 0x85, 0xc0]
            );
        } else {
            assert_eq!(
                &entry_payload.bytes[invocation_position + 7..invocation_position + 11],
                &[0x48, 0x83, 0xf8, 0x01]
            );
        }
        if number != 186 {
            assert!(
                user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, number as u8])
            );
        } else {
            assert!(
                user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0x3d, 0xba, 0, 0, 0])
            );
        }
    }
    assert!(
        user_syscall.windows(4).any(|bytes| bytes
            == u32::try_from(entry_payload.uname_return_address)
                .unwrap()
                .to_le_bytes())
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == INITIAL_USER_UNAME_CANDIDATE.to_le_bytes()),
        "constructed uname must admit only its sealed return site and exact writable stack candidate"
    );
    assert!(
        user_syscall.windows(4).any(|bytes| bytes
            == u32::try_from(entry_payload.nanosleep_return_address)
                .unwrap()
                .to_le_bytes())
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == INITIAL_USER_NANOSLEEP_CANDIDATE.to_le_bytes()),
        "constructed nanosleep must admit only its sealed return site and exact readable stack candidate"
    );
    assert!(
        user_syscall
            .windows(4)
            .any(|bytes| bytes == 390_u32.to_le_bytes()),
        "copy_to_user must retain the exact new_utsname extent"
    );
    assert!(
        user_syscall.windows(8).any(|bytes| bytes == b"7.2.9-to"),
        "uname bytes must come from the checked source-owned value"
    );
    let user_probe = initial_user_probe().unwrap();
    assert!(
        user_probe
            .windows(8)
            .any(|bytes| bytes == entry_payload.continuation_address.to_le_bytes()),
        "the provider probe must transfer to the sealed constructed-image syscall continuation"
    );
    assert!(
        user_probe
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x23, 0x00, 0x00, 0x00, 0x0f, 0x05])
            .count()
            >= 2,
        "the generated user probe must invoke both nanosleep qualification paths"
    );
    assert!(
        user_syscall
            .windows(4)
            .any(|bytes| bytes == [0x49, 0x83, 0xfa, 0x10]),
        "copy_from_user must retain the exact native timespec extent"
    );
    assert!(
        user_syscall
            .windows(4)
            .any(|bytes| bytes == [0x43, 0x8a, 0x04, 0x10]),
        "copy_from_user must load through the generated user candidate site"
    );
    assert!(
        user_syscall
            .windows(4)
            .any(|bytes| bytes == [0x42, 0x88, 0x04, 0x14]),
        "copy_from_user must stage bytes in private kernel storage"
    );
    assert!(
        user_syscall
            .windows(9)
            .any(|bytes| { bytes == [0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02] }),
        "copy_from_user must publish only its private read-direction recovery state"
    );
    let copy_fault = symbol_bytes(&file, X86_SYSTEMS_USER_COPY_FAULT_SYMBOL);
    assert!(
        copy_fault
            .windows(3)
            .any(|bytes| bytes == [0x0f, 0x20, 0xd0])
    );
    assert!(copy_fault.windows(2).any(|bytes| bytes == [0x48, 0xcf]));
    assert!(
        copy_fault
            .windows(5)
            .any(|bytes| bytes == [0x48, 0x83, 0x3c, 0x24, 0x02]),
        "copy_to_user recovery must require a supervisor write fault"
    );
    assert!(
        copy_fault
            .windows(5)
            .any(|bytes| bytes == [0x48, 0x83, 0x3c, 0x24, 0x00]),
        "copy_from_user recovery must require a supervisor read fault"
    );
    assert!(
        copy_fault
            .windows(4)
            .any(|bytes| bytes == 0x0060_2000_u32.to_le_bytes()),
        "copy fault recovery must validate the exact guard-page fault address"
    );

    let send = symbol_bytes(&file, X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL);
    assert!(send.windows(2).any(|bytes| bytes == [0x0f, 0x32]));
    assert!(send.windows(2).any(|bytes| bytes == [0x0f, 0x30]));
    assert!(
        send.windows(8)
            .any(|bytes| bytes == 0xfee0_0300_u64.to_le_bytes())
    );
    assert!(
        send.windows(4)
            .any(|bytes| bytes == 0x0004_40f1_u32.to_le_bytes())
    );
    let wait = symbol_bytes(&file, X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL);
    assert!(wait.windows(3).any(|bytes| bytes == [0xfb, 0xf4, 0xfa]));
    let complete = symbol_bytes(&file, X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL);
    assert!(
        complete
            .windows(8)
            .any(|bytes| bytes == 0xfee0_00b0_u64.to_le_bytes())
    );
    let clock = symbol_bytes(&file, X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL);
    assert!(
        clock
            .windows(8)
            .any(|bytes| bytes == 0x0000_0000_fed0_0000_u64.to_le_bytes())
    );
    assert!(
        clock
            .windows(6)
            .any(|bytes| bytes == [0x81, 0xfa, 0x80, 0x96, 0x98, 0x00])
    );
    assert!(
        clock
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x8b, 0x80, 0xf0, 0, 0, 0])
    );
    assert!(
        clock
            .windows(8)
            .any(|bytes| bytes == 0xc000_0000_0000_0000_u64.to_le_bytes())
    );
    assert!(
        clock
            .windows(8)
            .any(|bytes| bytes == 0x4000_0000_0000_0000_u64.to_le_bytes())
    );
    let deadline_after = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_AFTER_SYMBOL);
    assert!(
        deadline_after
            .windows(4)
            .any(|bytes| bytes == 100_000_u32.to_le_bytes()),
        "deadline construction must add exactly one millisecond of validated HPET ticks"
    );
    let deadline_arm = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_ARM_SYMBOL);
    for address in [
        0x0000_0000_fed0_0000_u64,
        0x0000_0000_fec0_0000_u64,
        0x0000_0000_fee0_0300_u64,
    ] {
        assert!(
            deadline_arm
                .windows(8)
                .any(|bytes| bytes == address.to_le_bytes())
        );
    }
    assert!(
        deadline_arm
            .windows(4)
            .any(|bytes| bytes == 0x0000_0404_u32.to_le_bytes()),
        "deadline arm must select one-shot timer 0 delivery on Q35 route 2"
    );
    assert!(
        deadline_arm
            .windows(4)
            .any(|bytes| bytes == 0x0004_40f2_u32.to_le_bytes()),
        "an already-expired deadline must become immediately deliverable"
    );
    assert!(
        deadline_arm
            .windows(6)
            .any(|bytes| bytes == [0xb0, 0xff, 0xe6, 0x21, 0xe6, 0xa1]),
        "deadline wait must mask unrelated legacy PIC delivery"
    );
    let deadline_wait = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_WAIT_SYMBOL);
    assert!(
        deadline_wait
            .windows(3)
            .any(|bytes| bytes == [0xfb, 0xf4, 0xfa])
    );
    let deadline_complete = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL);
    assert!(
        deadline_complete
            .windows(8)
            .any(|bytes| bytes == 0x0000_0000_fed0_0000_u64.to_le_bytes())
    );
    assert!(
        deadline_complete
            .windows(8)
            .any(|bytes| bytes == 0x0000_0000_fee0_00b0_u64.to_le_bytes())
    );
    let context_create = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_CREATE_SYMBOL);
    assert!(
        context_create
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x40, 0x00, 0x00]),
        "context creation must validate the exact 16 KiB stack extent"
    );
    assert!(
        context_create
            .windows(4)
            .any(|bytes| bytes == [0xc6, 0x47, 0x18, 0x01]),
        "context creation must publish exactly one suspended worker"
    );
    let context_transfer = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL);
    assert!(
        context_transfer
            .windows(10)
            .any(|bytes| { bytes == [0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57] }),
        "context transfer must save the x86-64 callee-saved continuation"
    );
    assert!(
        context_transfer
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x8b, 0x67, 0x08]),
        "context transfer must select the provider-private worker stack pointer"
    );
    assert!(
        context_transfer
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x89, 0x67, 0x08]),
        "cooperative handoff must suspend the running worker stack pointer"
    );
    assert!(
        context_transfer
            .windows(3)
            .any(|bytes| bytes == [0x48, 0x8b, 0x27]),
        "cooperative handoff must select the suspended dispatcher stack pointer"
    );
    for state in [0x01, 0x02] {
        assert!(
            context_transfer
                .windows(4)
                .any(|bytes| bytes == [0xc6, 0x47, 0x18, state]),
            "context transfer must publish both suspended and active states"
        );
    }
    let context_retire = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL);
    assert!(
        context_retire
            .windows(4)
            .any(|bytes| bytes == [0xc6, 0x47, 0x18, 0x03])
    );
    assert!(
        context_retire
            .windows(3)
            .any(|bytes| bytes == [0x48, 0x8b, 0x27]),
        "retirement must resume the exact saved caller stack pointer"
    );
    let context_reclaim = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL);
    assert!(
        context_reclaim
            .windows(4)
            .any(|bytes| bytes == [0xc6, 0x47, 0x18, 0x04]),
        "reclaim must terminally consume the completed context state"
    );

    let activator = file
        .symbol_by_name(X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL)
        .unwrap();
    let activator_start = usize::try_from(activator.address()).unwrap();
    let activator_end = activator_start + usize::try_from(activator.size()).unwrap();
    let activator = &data[activator_start..activator_end];
    assert!(
        activator
            .windows(3)
            .any(|bytes| bytes == [0x0f, 0x22, 0xdf]),
        "translation activation must retain the qualified CR3 transition"
    );
    assert!(
        activator
            .windows(5)
            .any(|bytes| bytes == [0x0d, 0x00, 0x08, 0x00, 0x00]),
        "translation activation must enable NX before admitting non-executable leaves"
    );

    let edit_begin = file
        .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL)
        .unwrap();
    let start = usize::try_from(edit_begin.address()).unwrap();
    let end = start + usize::try_from(edit_begin.size()).unwrap();
    let edit_begin = &data[start..end];
    assert!(
        edit_begin
            .windows(7)
            .any(|bytes| bytes == [0x49, 0x81, 0xc2, 0x00, 0x20, 0x00, 0x00]),
        "map-edit begin must reserve two complete metadata pages"
    );

    let edit_map = file
        .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL)
        .unwrap();
    let start = usize::try_from(edit_map.address()).unwrap();
    let end = start + usize::try_from(edit_map.size()).unwrap();
    let edit_map = &data[start..end];
    assert!(
        edit_map
            .windows(10)
            .any(|bytes| bytes == [0x48, 0xba, 0x03, 0, 0, 0, 0, 0, 0, 0x80]),
        "staged leaf must be present, writable, and non-executable"
    );

    let edit_unmap = file
        .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL)
        .unwrap();
    let start = usize::try_from(edit_unmap.address()).unwrap();
    let end = start + usize::try_from(edit_unmap.size()).unwrap();
    let edit_unmap = &data[start..end];
    assert!(
        edit_unmap
            .windows(8)
            .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0x08, 0x10, 0, 0, 0]),
        "unmap staging must validate the published parent entry"
    );

    let edit_commit = file
        .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL)
        .unwrap();
    let start = usize::try_from(edit_commit.address()).unwrap();
    let end = start + usize::try_from(edit_commit.size()).unwrap();
    let edit_commit = &data[start..end];
    assert!(
        edit_commit
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x89, 0x87, 0x08, 0x10, 0, 0]),
        "map commit must publish PDPT[1] only after staging"
    );
    assert!(
        edit_commit
            .windows(3)
            .any(|bytes| bytes == [0x0f, 0x01, 0x3e]),
        "unmap commit must invalidate the private mapping before returning frames"
    );
}
