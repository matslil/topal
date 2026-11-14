fn audit_provider_user_execution(file: &object::File<'_>) -> InitialUserEntryPayload {
    let user_address_prepare = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL);
    let user_prepare = symbol_bytes(&file, X86_SYSTEMS_USER_PREPARE_SYMBOL);
    for msr in [
        0xc000_0080_u32,
        0xc000_0081,
        0xc000_0082,
        0xc000_0084,
        0xc000_0102,
    ] {
        assert!(
            user_prepare
                .windows(4)
                .any(|bytes| bytes == msr.to_le_bytes()),
            "user preparation must configure MSR {msr:#x}"
        );
    }
    assert!(
        user_prepare
            .windows(7)
            .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x17]),
        "user preparation must require the bounded anonymous reservation"
    );
    assert!(
        user_address_prepare
            .windows(4)
            .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x38]),
        "address-space preparation must invalidate the sealed probe page after publication"
    );
    assert!(
        user_address_prepare
            .windows(4)
            .any(|bytes| bytes == [0x4c, 0x89, 0x57, 0x68]),
        "address-space preparation must retain the allocator-owned backing identity"
    );
    let code_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET,
    )
    .unwrap();
    let code_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET,
    )
    .unwrap();
    assert!(
        user_address_prepare.windows(7).any(|bytes| {
            bytes[0..3] == [0x49, 0x8b, 0x80] && bytes[3..7] == code_file_size_offset.to_le_bytes()
        }) && user_address_prepare.windows(7).any(|bytes| {
            bytes[0..3] == [0x49, 0x3b, 0x80]
                && bytes[3..7] == code_memory_size_offset.to_le_bytes()
        }) && user_address_prepare
            .windows(6)
            .any(|bytes| bytes == [0x48, 0x3d, 0x08, 0x11, 0, 0])
            && user_address_prepare
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x3d, 0, 0x20, 0, 0])
            && user_address_prepare
                .windows(2)
                .any(|bytes| bytes == [0xa8, 0x07]),
        "address-space preparation must validate a bounded equal qword-complete runtime code extent"
    );
    let data_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
    )
    .unwrap();
    let data_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET,
    )
    .unwrap();
    assert!(
        user_address_prepare.windows(7).any(|bytes| {
            bytes[0..3] == [0x49, 0x8b, 0x80] && bytes[3..7] == data_file_size_offset.to_le_bytes()
        }) && user_address_prepare.windows(7).any(|bytes| {
            bytes[0..3] == [0x49, 0x8b, 0x90]
                && bytes[3..7] == data_memory_size_offset.to_le_bytes()
        }) && user_address_prepare
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x83, 0xf8, 0x08])
            && user_address_prepare
                .windows(3)
                .any(|bytes| bytes == [0x48, 0x39, 0xd0])
            && user_address_prepare
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfa, 0, 0x10, 0, 0]),
        "address-space preparation must validate bounded ordered qword-complete data and BSS extents"
    );
    let entry_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_ENTRY_ELF_OFFSET,
    )
    .unwrap();
    assert!(
        user_address_prepare.windows(7).any(|bytes| {
            bytes[0..3] == [0x4d, 0x8b, 0x98] && bytes[3..7] == entry_offset.to_le_bytes()
        }) && user_address_prepare
            .windows(7)
            .any(|bytes| bytes == [0x4c, 0x89, 0x9f, 0x88, 0, 0, 0])
            && user_address_prepare
                .windows(7)
                .any(|bytes| bytes == [0x4d, 0x89, 0x9a, 0xe8, 0x1e, 0, 0]),
        "address-space preparation must validate and retain one ELF entry for AT_ENTRY"
    );
    let initial_stack = initial_user_stack_bytes();
    assert_eq!(
        u64::from_le_bytes(initial_stack[0..8].try_into().unwrap()),
        1
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[8..16].try_into().unwrap()),
        0x0060_1fc8
    );
    assert_eq!(initial_stack[16..32], [0; 16]);
    assert_eq!(
        u64::from_le_bytes(initial_stack[32..40].try_into().unwrap()),
        6
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[40..48].try_into().unwrap()),
        4096
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[48..56].try_into().unwrap()),
        9
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[56..64].try_into().unwrap()),
        0
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[64..72].try_into().unwrap()),
        31
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[72..80].try_into().unwrap()),
        0x0060_1fc8
    );
    for (offset, tag) in [(80, 11_u64), (96, 12), (112, 13), (128, 14)] {
        assert_eq!(
            u64::from_le_bytes(initial_stack[offset..offset + 8].try_into().unwrap()),
            tag
        );
        assert_eq!(initial_stack[offset + 8..offset + 16], [0; 8]);
    }
    for (offset, tag, value) in [
        (144, 3_u64, INITIAL_USER_PROGRAM_HEADER_ADDRESS),
        (160, 4, 56),
        (176, 5, 3),
    ] {
        assert_eq!(
            u64::from_le_bytes(initial_stack[offset..offset + 8].try_into().unwrap()),
            tag
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[offset + 8..offset + 16].try_into().unwrap()),
            value
        );
    }
    assert_eq!(
        u64::from_le_bytes(initial_stack[192..200].try_into().unwrap()),
        23
    );
    assert_eq!(initial_stack[200..208], [0; 8]);
    assert_eq!(
        u64::from_le_bytes(initial_stack[208..216].try_into().unwrap()),
        17
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[216..224].try_into().unwrap()),
        100
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[224..232].try_into().unwrap()),
        15
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[232..240].try_into().unwrap()),
        0x0060_1fc0
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[240..248].try_into().unwrap()),
        25
    );
    assert_eq!(
        u64::from_le_bytes(initial_stack[248..256].try_into().unwrap()),
        0x0060_1fe0
    );
    assert_eq!(initial_stack[256..272], [0; 16]);
    assert_eq!(&initial_stack[272..279], b"x86_64\0");
    assert_eq!(&initial_stack[280..291], b"topal-probe");
    assert_eq!(initial_stack[291], 0);
    assert_eq!(initial_stack[304..320], [0; 16]);
    let auxv_random_delivery = symbol_bytes(
        &file,
        X86_SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM_SYMBOL,
    );
    assert!(
        auxv_random_delivery
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x89, 0x81, 0xe0, 0x1f, 0, 0])
            && auxv_random_delivery
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x81, 0xe8, 0x1f, 0, 0])
            && auxv_random_delivery
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0xd0, 0, 0, 0, 1]),
        "initial AT_RANDOM delivery must copy both selected qwords and publish entry readiness"
    );
    let user_enter = symbol_bytes(&file, X86_SYSTEMS_USER_ENTER_SYMBOL);
    assert!(
        user_enter
            .windows(7)
            .any(|bytes| bytes == [0x80, 0xbf, 0xd0, 0, 0, 0, 1]),
        "initial user entry must reject an address space without AT_RANDOM delivery"
    );
    assert!(
        user_enter
            .windows(8)
            .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xd8, 0, 0, 0, 0]),
        "initial user entry must reject an address space without a live getrandom reservation"
    );
    let getrandom_reservation = symbol_bytes(
        &file,
        X86_SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM_SYMBOL,
    );
    assert!(
        getrandom_reservation
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x89, 0xb7, 0xd8, 0, 0, 0])
            && getrandom_reservation
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0xd0, 0, 0, 0, 1]),
        "initial getrandom reservation must retain only a private affine output reference"
    );
    let getrandom_binding =
        symbol_bytes(&file, X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL);
    assert!(
        getrandom_binding
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x89, 0x97, 0xd8, 0, 0, 0])
            && getrandom_binding
                .windows(4)
                .any(|bytes| bytes == [0xc6, 0x42, 0x10, 0x01]),
        "reusable getrandom binding must replace the reservation with sealed service state"
    );
    let user_syscall = symbol_bytes(&file, X86_SYSTEMS_USER_SYSCALL_SYMBOL);
    assert!(
        user_syscall
            .windows(5)
            .any(|bytes| bytes == [0x3d, 0x3e, 0x01, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x48, 0x8b, 0x0c, 0x25, 0xd8, 0, 0, 0])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x48, 0xff, 0x41, 0x18]),
        "native syscall 318 must validate and count each reusable service delivery"
    );
    let entry_payload = initial_user_entry_payload().unwrap();
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| **bytes == [0xb8, 0x3e, 0x01, 0, 0, 0x0f, 0x05])
            .count(),
        9,
        "ring-3 must preserve the prior eight getrandom calls and add one maximum large request"
    );
    assert!(
        entry_payload
            .bytes
            .windows(9)
            .any(|bytes| bytes == [0x4c, 0x8b, 0x14, 0x24, 0x4c, 0x8b, 0x5c, 0x24, 8]),
        "ring-3 must read both returned qwords without comparing them with constants"
    );
    assert!(
        entry_payload
            .bytes
            .windows(10)
            .any(|bytes| bytes == [0x4c, 0x8b, 0x64, 0x24, 0x10, 0x4c, 0x8b, 0x6c, 0x24, 0x18]),
        "ring-3 must read the complete four-qword provider output without constraining it"
    );
    assert!(
        entry_payload
            .bytes
            .windows(8)
            .any(|bytes| bytes == [0x4c, 0x8b, 0x9c, 0x24, 0xf8, 0x00, 0x00, 0x00]),
        "ring-3 must read the last qword of the maximum 256-byte small request"
    );
    assert!(
        entry_payload
            .bytes
            .windows(8)
            .any(|bytes| bytes == [0x4c, 0x8b, 0x1c, 0x25, 0xf8, 0x11, 0x60, 0x00]),
        "ring-3 must read the last qword of the maximum 512-byte large request"
    );
    assert!(
        user_syscall
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x02, 0x00, 0x00])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xfa, 0x20])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0xba, 0x20, 0x00, 0x00, 0x00]),
        "getrandom must cap complete requests at 512 bytes and split them into 32-byte outputs"
    );
    assert!(
        initial_stack
            .chunks_exact(8)
            .all(|chunk| user_address_prepare.windows(8).any(|bytes| bytes == chunk)),
        "address-space preparation must initialize every word of the sealed initial stack"
    );
    assert!(
        user_address_prepare
            .windows(7)
            .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x01]),
        "address-space state must not overlap the retained backing identity"
    );
    assert!(
        user_address_prepare
            .windows(8)
            .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes())
    );
    let user_mapping = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL);
    assert!(
        user_mapping
            .windows(8)
            .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes()),
        "mapping verification must install an RW/NX user leaf"
    );
    assert!(
        user_mapping
            .windows(4)
            .any(|bytes| bytes == [0x41, 0xc6, 0x02, 0x6d])
            && user_mapping
                .windows(4)
                .any(|bytes| bytes == [0x41, 0x80, 0x3a, 0x6d]),
        "mapping verification must perform its fixed byte store and load"
    );
    assert!(
        user_mapping
            .windows(4)
            .filter(|bytes| *bytes == [0x41, 0x0f, 0x01, 0x3a])
            .count()
            >= 2,
        "mapping verification must invalidate after publication and revocation"
    );
    let user_map = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL);
    assert!(
        user_map
            .windows(8)
            .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes())
            && user_map
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x89, 0x57, 0x78]),
        "user map must publish RW/NX authority and retain its exact frame identity"
    );
    assert!(
        user_map
            .windows(5)
            .any(|bytes| bytes == [0xb8, 0x04, 0, 0, 0])
            && user_map
                .windows(5)
                .any(|bytes| bytes == [0xb8, 0x11, 0, 0, 0])
            && user_map
                .windows(6)
                .any(|bytes| bytes == [0x88, 0x87, 0x70, 0, 0, 0])
            && user_map
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x97, 0x80, 0, 0, 0])
            && user_map
                .windows(8)
                .any(|bytes| bytes == 0x0060_5000_u64.to_le_bytes())
            && user_map
                .windows(8)
                .any(|bytes| bytes == 0x0060_3000_u64.to_le_bytes())
            && user_map
                .windows(4)
                .filter(|bytes| *bytes == [0x41, 0x0f, 0x01, 0x3a])
                .count()
                >= 2
            && !user_map
                .windows(8)
                .any(|bytes| bytes == 0x0060_2000_u64.to_le_bytes())
    );
    let user_store = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL);
    let data_payload_offset = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET
        + u32::try_from(INITIAL_USER_DATA_FILE_OFFSET).unwrap();
    assert!(
        user_store
            .windows(3)
            .any(|bytes| bytes == [0x49, 0x8b, 0x00])
            && user_store
                .windows(3)
                .any(|bytes| bytes == [0x49, 0x89, 0x02])
            && user_store
                .windows(7)
                .any(|bytes| bytes == [0x49, 0xc7, 0x02, 0, 0, 0, 0])
            && user_store
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xe9, 0x08])
            && user_store
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xeb, 0x08])
            && user_store
                .windows(4)
                .any(|bytes| bytes == data_payload_offset.to_le_bytes())
            && user_store
                .windows(6)
                .any(|bytes| bytes == [0x41, 0xbb, 0x00, 0x10, 0x00, 0x00])
            && user_store
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x87, 0x90, 0, 0, 0])
            && user_store
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x87, 0x98, 0, 0, 0])
            && user_store
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x12])
            && user_store
                .windows(8)
                .any(|bytes| bytes == INITIAL_USER_DATA_ADDRESS.to_le_bytes()),
        "user store must copy runtime data, derive both break states, and zero the complete BSS and future-heap page tail"
    );
    let user_install = symbol_bytes(
        &file,
        X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL,
    );
    let entry_payload = initial_user_entry_payload().unwrap();
    let code_image = initial_user_code_image().unwrap();
    let elf_file = initial_user_elf_file().unwrap();
    assert_eq!(code_image.len(), 4096 + 256 + entry_payload.bytes.len());
    assert!(
        entry_payload
            .bytes
            .windows(9)
            .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xc0, 0, 0, 0, 23])
            && entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xd0, 0, 0, 0, 17])
            && entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xd8, 0, 0, 0, 100])
            && entry_payload
                .bytes
                .windows(12)
                .any(|bytes| bytes == [0x48, 0x81, 0xbc, 0x24, 0xe8, 0, 0, 0, 0xc0, 0x1f, 0x60, 0])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xc4, 0x50, 0x01, 0, 0]),
        "constructed user code must validate AT_SECURE, AT_CLKTCK, AT_PLATFORM, and the enlarged initial block"
    );
    assert!(
        entry_payload
            .bytes
            .windows(8)
            .any(|bytes| bytes == (INITIAL_USER_DATA_ADDRESS + 8).to_le_bytes())
            && entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x41, 0xc6, 0x02, 0xa5])
            && entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x41, 0x80, 0x3a, 0xa5]),
        "constructed user code must write and read back one BSS byte"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x0c, 0, 0, 0, 0x0f, 0x05])
            .count(),
        4,
        "constructed user code must query, grow, shrink, and reject one program break"
    );
    assert!(
        entry_payload
            .bytes
            .windows(8)
            .any(|bytes| bytes == [0xc6, 0x04, 0x25, 0x10, 0x50, 0x60, 0x00, 0xa5])
            && entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == [0x80, 0x3c, 0x25, 0x10, 0x50, 0x60, 0x00, 0xa5]),
        "constructed user code must write and read the first newly admitted heap byte"
    );
    assert_eq!(
        elf_file.len(),
        INITIAL_USER_DATA_FILE_OFFSET + INITIAL_USER_DATA_CAPACITY
    );
    assert_eq!(&code_image[0..4], b"\x7fELF");
    assert_eq!(
        u16::from_le_bytes(code_image[18..20].try_into().unwrap()),
        62
    );
    assert_eq!(
        u64::from_le_bytes(code_image[24..32].try_into().unwrap()),
        INITIAL_USER_ENTRY_ADDRESS
    );
    assert_eq!(
        u64::from_le_bytes(code_image[32..40].try_into().unwrap()),
        64
    );
    assert_eq!(
        u16::from_le_bytes(code_image[54..56].try_into().unwrap()),
        56
    );
    assert_eq!(
        u16::from_le_bytes(code_image[56..58].try_into().unwrap()),
        3
    );
    assert_eq!(
        u64::from_le_bytes(code_image[80..88].try_into().unwrap()),
        INITIAL_USER_CODE_ADDRESS
    );
    assert_eq!(
        u64::from_le_bytes(code_image[96..104].try_into().unwrap()),
        u64::try_from(code_image.len()).unwrap()
    );
    assert_eq!(
        u32::from_le_bytes(code_image[120..124].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(code_image[124..128].try_into().unwrap()),
        6
    );
    assert_eq!(
        u64::from_le_bytes(code_image[128..136].try_into().unwrap()),
        u64::try_from(INITIAL_USER_DATA_FILE_OFFSET).unwrap()
    );
    assert_eq!(
        u64::from_le_bytes(code_image[136..144].try_into().unwrap()),
        INITIAL_USER_DATA_ADDRESS
    );
    assert_eq!(
        u64::from_le_bytes(code_image[152..160].try_into().unwrap()),
        8
    );
    assert_eq!(
        u64::from_le_bytes(code_image[160..168].try_into().unwrap()),
        16
    );
    assert_eq!(
        u32::from_le_bytes(code_image[176..180].try_into().unwrap()),
        0x6474_e551
    );
    assert_eq!(
        u32::from_le_bytes(code_image[180..184].try_into().unwrap()),
        6
    );
    assert_eq!(elf_file[INITIAL_USER_DATA_FILE_OFFSET], 0x6d);
    assert!(
        elf_file[INITIAL_USER_DATA_FILE_OFFSET + 1..]
            .iter()
            .all(|byte| *byte == 0)
    );
    assert!(code_image[256..4096 + 256].iter().all(|byte| *byte == 0x90));
    assert_eq!(&code_image[4096 + 256..], entry_payload.bytes.as_slice());
    assert!(
        user_install
            .windows(8)
            .any(|bytes| bytes == INITIAL_USER_CODE_ADDRESS.to_le_bytes())
            && user_install
                .windows(3)
                .filter(|bytes| *bytes == [0x49, 0x8b, 0x00])
                .count()
                == 1
            && user_install
                .windows(3)
                .filter(|bytes| *bytes == [0x49, 0x89, 0x02])
                .count()
                == 1
            && user_install
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xc0, 0x08])
            && user_install
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xc2, 0x08])
            && user_install
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xe9, 0x08])
            && user_install.windows(7).any(|bytes| {
                bytes
                    == [
                        0x49,
                        0x81,
                        0xc0,
                        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as u8,
                        0,
                        0,
                        0,
                    ]
            })
            && user_install
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x05]),
        "entry installation must copy the runtime-derived bounded extent from the external ELF member"
    );
    assert!(
        entry_payload
            .bytes
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x20, 0x06, 0x0f])
            && entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x81, 0x7c, 0x24, 0x28, 0x00, 0x10, 0x00, 0x00])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x30, 0x09, 0x0f])
            && entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x8b, 0x43, 0x18])
            && entry_payload
                .bytes
                .windows(5)
                .any(|bytes| bytes == [0x48, 0x39, 0x44, 0x24, 0x38])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x40, 0x1f, 0x0f])
            && entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x81, 0x7c, 0x24, 0x48, 0xc8, 0x1f, 0x60, 0x00])
            && entry_payload
                .bytes
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x50, 0x0b])
            && entry_payload
                .bytes
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x60, 0x0c])
            && entry_payload
                .bytes
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x70, 0x0d])
            && entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0x80, 0, 0, 0, 0x0e])
            && entry_payload
                .bytes
                .windows(12)
                .any(|bytes| bytes == [0x48, 0x81, 0xbc, 0x24, 0x98, 0, 0, 0, 0x40, 0x30, 0x60, 0])
            && entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xa0, 0, 0, 0, 4])
            && entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xa8, 0, 0, 0, 56])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xc4, 0x50, 0x01, 0, 0]),
        "constructed entry must validate process facts and resident program-header metadata before consuming the complete bounded stack"
    );
    assert!(
        entry_payload
            .bytes
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x81, 0xec, 0x00, 0x02, 0x00, 0x00])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xc4, 0x00, 0x02, 0x00, 0x00])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0x3f, 0, 0, 0, 0x0f, 0x05])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0x23, 0, 0, 0, 0x0f, 0x05])
            && entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xec, 0x10])
            && entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xc4, 0x10])
            && entry_payload
                .bytes
                .windows(6)
                .any(|bytes| bytes == [0x31, 0xc0, 0x48, 0x89, 0x04, 0x24])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0x27, 0, 0, 0, 0x0f, 0x05])
            && entry_payload.bytes.windows(4).any(|bytes| bytes == b"Linu")
            && entry_payload
                .bytes
                .windows(2)
                .any(|bytes| bytes == [0x31, 0xff]),
        "constructed transfer payload must qualify uname output, zero-duration nanosleep input, general bounded stdout write and stdin read before getpid and status-zero exit"
    );
    let user_syscall = symbol_bytes(&file, X86_SYSTEMS_USER_SYSCALL_SYMBOL);
    assert!(
        entry_payload
            .bytes
            .windows(7)
            .any(|bytes| bytes == [0xb8, 0x01, 0, 0, 0, 0x0f, 0x05])
    );
    assert!(
        entry_payload
            .bytes
            .windows(8)
            .any(|bytes| bytes == b"TOPAL_KE")
    );
    assert!(
        entry_payload
            .bytes
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x83, 0xec, 0x20])
    );
    assert!(
        entry_payload
            .bytes
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x83, 0xc4, 0x20])
    );
    assert!(
        !user_syscall.windows(8).any(|bytes| bytes == b"TOPAL_KE")
            && !user_syscall.windows(8).any(|bytes| bytes == b"RNEL_USE")
            && !user_syscall.windows(8).any(|bytes| bytes == b"R_WRITE_"),
        "the provider must not use the application payload as a write authorization oracle"
    );
    assert!(
        user_syscall
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x01, 0x00, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x00, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x20, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x40, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x60, 0x60, 0x00]),
        "bounded write must reject zero and oversized lengths and admit only complete readable mapping ranges"
    );
    assert!(
        user_syscall
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x81, 0xec, 0x08, 0x01, 0x00, 0x00])
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x89, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00])
            && user_syscall
                .windows(8)
                .filter(|bytes| { *bytes == [0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00] })
                .count()
                == 4
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x8b, 0x84, 0x24, 0x00, 0x01, 0x00, 0x00]),
        "bounded write must finish dynamic private staging before output and return the requested length"
    );
    for errno in [0xf7_u8, 0xea, 0xf2] {
        assert!(
            user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0xc7, 0xc0, errno, 0xff, 0xff, 0xff]),
            "bounded write must retain distinct EBADF, EINVAL, and EFAULT results"
        );
    }
    let dup2_replace = user_syscall
        .windows(8)
        .rposition(|bytes| bytes == [0x65, 0x44, 0x88, 0x86, 0xe0, 0, 0, 0])
        .expect("dup2 must perform one dynamic destination replacement");
    let dup2_source_lookup = user_syscall[..dup2_replace]
        .windows(8)
        .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
        .expect("dup2 must look up the source description before replacement");
    let dup2_destination_bound = user_syscall[..dup2_replace]
        .windows(4)
        .rposition(|bytes| bytes == [0x48, 0x83, 0xfe, 0x02])
        .expect("dup2 must validate its destination before replacement");
    assert!(
        dup2_source_lookup < dup2_destination_bound && dup2_destination_bound < dup2_replace,
        "dup2 must validate the source and destination before atomic replacement"
    );
    assert!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x03, 0, 0, 0, 0x0f, 0x05])
            .count()
            == 5
            && entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x20, 0, 0, 0, 0x0f, 0x05])
                .count()
                == 2
            && entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x21, 0, 0, 0, 0x0f, 0x05])
                .count()
                == 3
            && entry_payload
                .bytes
                .windows(4)
                .filter(|bytes| *bytes == [0x31, 0xc0, 0x0f, 0x05])
                .count()
                >= 2
            && user_address_prepare
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xe0, 0, 0, 0, 1, 2, 3, 0])
            && [0x03_u8, 0x20, 0x21].into_iter().all(|number| {
                user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, number])
            })
            && user_syscall
                .windows(8)
                .filter(|bytes| *bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
                .count()
                >= 5
            && [1_u8, 2, 3].into_iter().all(|identity| {
                user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0x80, 0xf8, identity])
            })
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x65, 0xc6, 0x87, 0xe0, 0, 0, 0, 0])
            && [0xe0_u8, 0xe1, 0xe2].into_iter().all(|offset| {
                user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x44, 0x88, 0x04, 0x25, offset, 0, 0, 0])
            })
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x65, 0x44, 0x88, 0x86, 0xe0, 0, 0, 0])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xe8, 0xff, 0xff, 0xff])
            && [(0xe0_u8, 0_u8), (0xe1, 2), (0xe2, 3)]
                .into_iter()
                .all(|(offset, identity)| {
                    user_syscall
                        .windows(9)
                        .any(|bytes| bytes == [0x65, 0x80, 0x3c, 0x25, offset, 0, 0, 0, identity])
                })
            && user_syscall.windows(13).any(|bytes| {
                bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xe0, 0, 0, 0, 0, 0, 0, 0]
            }),
        "duplication must map fixed description identities, preserve access modes and aliases, reject full or invalid requests, and verify complete terminal clearing"
    );
    let fcntl_first_command = user_syscall
        .windows(4)
        .rposition(|bytes| bytes == [0x48, 0x83, 0xfe, 0x01])
        .expect("fcntl must decode F_GETFD");
    let fcntl_lookup = user_syscall[..fcntl_first_command]
        .windows(8)
        .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
        .expect("fcntl must look up the mapped description");
    assert!(
        fcntl_lookup < fcntl_first_command
            && entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x48, 0, 0, 0, 0x0f, 0x05])
                .count()
                == 9
            && user_address_prepare
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xe8, 0, 0, 0, 0, 0, 0, 0])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x48])
            && [1_u8, 2, 3].into_iter().all(|command| {
                user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xfe, command])
            })
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x65, 0x0f, 0xb6, 0x87, 0xe8, 0, 0, 0])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x80, 0xe2, 0x01])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x65, 0x88, 0x97, 0xe8, 0, 0, 0])
            && [0xe8_u8, 0xe9, 0xea].into_iter().all(|offset| {
                user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0xc6, 0x04, 0x25, offset, 0, 0, 0, 0])
            })
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x65, 0xc6, 0x86, 0xe8, 0, 0, 0, 0])
            && user_syscall.windows(13).any(|bytes| {
                bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xe8, 0, 0, 0, 0, 0, 0, 0]
            }),
        "fcntl must separate slot flags from descriptions, decode only after lookup, clear duplication destinations, and erase terminal state"
    );
    let fstat_first_candidate_bound = user_syscall
        .windows(7)
        .rposition(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x10, 0x60, 0x00])
        .expect("fstat must bound the writable stack range");
    let fstat_lookup = user_syscall[..fstat_first_candidate_bound]
        .windows(8)
        .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
        .expect("fstat must look up the mapped description");
    assert!(
        fstat_lookup < fstat_first_candidate_bound
            && entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x05, 0, 0, 0, 0x0f, 0x05])
                .count()
                == 3
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x05])
            && user_syscall
                .windows(10)
                .any(|bytes| bytes == [0x49, 0x89, 0xf0, 0x49, 0x81, 0xc0, 0x90, 0, 0, 0])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xec, 0x90, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x48, 0xc7, 0x44, 0x24, 0x18, 0x80, 0x21, 0, 0])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xfa, 0x90, 0, 0, 0])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x43, 0x88, 0x04, 0x11])
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0, 0]),
        "fstat must resolve a live description before pointer access, stage the complete native layout, and fault-contain one full writable-range copy"
    );
    let lseek_whence = user_syscall
        .windows(4)
        .rposition(|bytes| bytes == [0x48, 0x83, 0xfa, 0x04])
        .expect("lseek must validate the maximum seek mode");
    let lseek_lookup = user_syscall[..lseek_whence]
        .windows(8)
        .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
        .expect("lseek must look up the mapped description");
    assert!(
        lseek_lookup < lseek_whence
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x08])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xe3, 0xff, 0xff, 0xff])
            && entry_payload
                .bytes
                .windows(5)
                .filter(|bytes| *bytes == [0x6a, 0x08, 0x58, 0x0f, 0x05])
                .count()
                == 3
            && entry_payload
                .bytes
                .windows(5)
                .any(|bytes| bytes == [0xff, 0xc3, 0x83, 0xfb, 0x02])
            && entry_payload
                .bytes
                .windows(2)
                .any(|bytes| bytes == [0xb2, 0x05]),
        "lseek must resolve descriptors before modes, return ESPIPE for both live console directions, reject unsupported live modes, and retain closed-slot EBADF precedence"
    );
    let ioctl_lookup = user_syscall
        .windows(8)
        .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
        .expect("ioctl must look up the mapped description");
    let ioctl_request = user_syscall[ioctl_lookup..]
        .windows(6)
        .position(|bytes| bytes == [0x81, 0xfe, 0x01, 0x54, 0, 0])
        .map(|position| position + ioctl_lookup)
        .expect("ioctl must decode TCGETS");
    let ioctl_candidate = user_syscall[ioctl_request..]
        .windows(7)
        .position(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x10, 0x60, 0x00])
        .map(|position| position + ioctl_request)
        .expect("ioctl must bound the writable candidate");
    assert!(
        ioctl_lookup < ioctl_request
            && ioctl_request < ioctl_candidate
            && entry_payload
                .bytes
                .windows(5)
                .filter(|bytes| *bytes == [0xbe, 0x01, 0x54, 0, 0])
                .count()
                == 3
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x10])
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == [0x48, 0xc7, 0x44, 0x24, 0x08, 0xbf, 0x08, 0])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0xc6, 0x44, 0x24, 0x17, 0x01])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 0x24])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]),
        "TCGETS must enforce descriptor/request/pointer precedence, stage the exact raw termios, copy 36 bytes under recovery, and return ENOTTY for unknown live-console requests"
    );
    let winsize_request = user_syscall[ioctl_request..]
        .windows(6)
        .position(|bytes| bytes == [0x81, 0xfe, 0x13, 0x54, 0, 0])
        .map(|position| position + ioctl_request)
        .expect("ioctl must decode TIOCGWINSZ");
    let winsize_candidate = user_syscall[winsize_request..]
        .windows(7)
        .position(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x10, 0x60, 0x00])
        .map(|position| position + winsize_request)
        .expect("TIOCGWINSZ must bound the writable candidate");
    assert!(
        ioctl_lookup < ioctl_request
            && ioctl_request < winsize_request
            && winsize_request < winsize_candidate
            && entry_payload
                .bytes
                .windows(5)
                .filter(|bytes| *bytes == [0xbe, 0x13, 0x54, 0, 0])
                .count()
                == 1
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x83, 0xec, 0x08, 0x31, 0xc0, 0x48, 0x89, 0x04])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 0x08])
            && entry_payload
                .bytes
                .windows(5)
                .any(|bytes| bytes == [0xba, 0x80, 0x51, 0x60, 0])
            && entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == [0x83, 0xc2, 0x08, 0xff, 0xc3, 0x83, 0xfb, 0x02])
            && entry_payload
                .bytes
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xc6, 0x12]),
        "TIOCGWINSZ must follow live-descriptor and request decoding, publish the exact zero geometry under recovery, and prove input/output equality plus closed-slot precedence"
    );
    assert!(
        user_syscall
            .windows(7)
            .any(|bytes| bytes == [0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20])
    );
    assert!(
        user_syscall
            .windows(5)
            .any(|bytes| bytes == [0x42, 0x8a, 0x04, 0x14, 0xee])
    );
    assert!(
        entry_payload
            .bytes
            .windows(4)
            .any(|bytes| bytes == [0x31, 0xc0, 0x0f, 0x05])
            && entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xf8, 0x04])
            && entry_payload.bytes.windows(4).any(|bytes| bytes == b"read"),
        "constructed code must request and validate the four-byte qualification input"
    );
    assert!(
        user_syscall
            .windows(7)
            .filter(|bytes| *bytes == [0x48, 0x81, 0xfa, 0x00, 0x01, 0x00, 0x00])
            .count()
            == 2
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x10, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x20, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x60, 0x60, 0x00]),
        "bounded read must reject zero and oversized lengths and admit only complete writable mapping ranges"
    );
    let user_publish = user_syscall
        .windows(4)
        .rposition(|bytes| bytes == [0x43, 0x88, 0x04, 0x10])
        .expect("bounded read must copy staged bytes to the user destination");
    let receive_stage = user_syscall
        .windows(4)
        .enumerate()
        .filter_map(|(position, bytes)| {
            (position < user_publish && bytes == [0x42, 0x88, 0x04, 0x14]).then_some(position)
        })
        .next_back()
        .expect("bounded read must stage received bytes privately");
    assert!(
        receive_stage < user_publish
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x01])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x66, 0xba, 0xf8, 0x03, 0xec])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x42, 0x8a, 0x04, 0x14])
            && user_syscall
                .windows(8)
                .filter(|bytes| { *bytes == [0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00] })
                .count()
                == 4,
        "bounded read must receive a dynamic exact count into private staging before the dynamic user copy"
    );
    for errno in [0xf7_u8, 0xea, 0xf2] {
        assert!(
            user_syscall
                .windows(7)
                .filter(|bytes| { *bytes == [0x48, 0xc7, 0xc0, errno, 0xff, 0xff, 0xff] })
                .count()
                >= 2,
            "bounded read and write must each retain EBADF, EINVAL, and EFAULT results"
        );
    }
    assert!(
        user_syscall
            .windows(3)
            .any(|bytes| bytes == [0x83, 0xf8, 0x0c])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x48, 0x8b, 0x04, 0x25, 0x90, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0x98, 0, 0, 0])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xff, 0x00, 0x60, 0x60, 0x00])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x3c, 0x25, 0x90, 0, 0, 0]),
        "brk must use private current and initial state and publish only an in-page request"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x9e, 0, 0, 0, 0x0f, 0x05])
            .count(),
        4,
        "constructed user code must set, get, clear, and re-read its FS base"
    );
    assert!(
        entry_payload
            .bytes
            .windows(8)
            .any(|bytes| bytes == [0x48, 0x81, 0x3c, 0x24, 0, 0x50, 0x60, 0])
            && entry_payload
                .bytes
                .windows(5)
                .any(|bytes| bytes == [0x48, 0x83, 0x3c, 0x24, 0]),
        "constructed user code must validate both the selected and cleared FS values"
    );
    assert!(
        user_syscall
            .windows(5)
            .any(|bytes| bytes == [0x3d, 0x9e, 0, 0, 0])
            && user_syscall
                .windows(6)
                .any(|bytes| bytes == [0x81, 0xff, 0x02, 0x10, 0, 0])
            && user_syscall
                .windows(6)
                .any(|bytes| bytes == [0x81, 0xff, 0x03, 0x10, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x34, 0x25, 0xa0, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0xa0, 0, 0, 0])
            && user_syscall.windows(2).any(|bytes| bytes == [0x0f, 0x30])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x4d, 0x89, 0x01]),
        "arch_prctl must retain and install FS state and publish GET_FS through fault-contained output"
    );
    assert!(
        entry_payload
            .bytes
            .windows(7)
            .any(|bytes| bytes == [0xb8, 0xda, 0, 0, 0, 0x0f, 0x05])
            && entry_payload
                .bytes
                .windows(11)
                .any(|bytes| bytes == [0xc7, 0x04, 0x25, 0x08, 0x50, 0x60, 0, 1, 0, 0, 0])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x3d, 0xda, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x3c, 0x25, 0xa8, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0xa8, 0, 0, 0])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x41, 0xc7, 0x00, 0, 0, 0, 0])
            && user_syscall
                .windows(13)
                .any(|bytes| bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xa8, 0, 0, 0, 0, 0, 0, 0]),
        "set_tid_address must register one checked word, return TID one, and clear the word and state before terminal exit"
    );
    assert!(
        entry_payload
            .bytes
            .windows(7)
            .any(|bytes| bytes == [0xb8, 0x11, 0x01, 0, 0, 0x0f, 0x05])
            && entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x89, 0x24, 0x24])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x3d, 0x11, 0x01, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x3c, 0x25, 0xb0, 0, 0, 0])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0xb0, 0, 0, 0])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xfa, 0x18])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x4c, 0x39, 0x04, 0x24])
            && user_syscall
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x10, 0])
            && user_syscall
                .windows(13)
                .any(|bytes| bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xb0, 0, 0, 0, 0, 0, 0, 0]),
        "set_robust_list must register one complete native head and terminally validate its empty state before clearing"
    );
    assert!(
        entry_payload
            .bytes
            .windows(7)
            .any(|bytes| bytes == [0xb8, 0x61, 0, 0, 0, 0x0f, 0x05])
            && entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0x2e, 0x01, 0, 0, 0x0f, 0x05])
            && entry_payload
                .bytes
                .windows(8)
                .filter(|bytes| *bytes == [0x48, 0x81, 0x3c, 0x24, 0, 0, 0x80, 0])
                .count()
                == 2
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x61])
            && user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x3d, 0x2e, 0x01, 0, 0])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xfa, 0x10])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x43, 0x88, 0x04, 0x11])
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == 0x0080_0000_u64.to_le_bytes())
            && user_syscall
                .windows(8)
                .any(|bytes| bytes == u64::MAX.to_le_bytes()),
        "getrlimit and query-only prlimit64 must share complete fault-contained publication of the initial stack limit"
    );
    assert_eq!(
        entry_payload
            .bytes
            .windows(7)
            .filter(|bytes| *bytes == [0xb8, 0x0e, 0, 0, 0, 0x0f, 0x05])
            .count(),
        4,
        "constructed user code must replace, query, unblock, and re-query the signal mask"
    );
    assert!(
        entry_payload
            .bytes
            .windows(4)
            .any(|bytes| bytes == 0x0004_0300_u32.to_le_bytes())
            && entry_payload
                .bytes
                .windows(9)
                .filter(|bytes| {
                    *bytes == [0x48, 0x81, 0x7c, 0x24, 0x08, 0x00, 0x02, 0x00, 0x00]
                })
                .count()
                == 2
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x0e])
            && user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xf8, 0x08])
            && user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xe0, 0xff, 0xfe, 0xfb, 0xff])
            && [0_u8, 1, 2].into_iter().all(|how| {
                user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xff, how])
            })
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x4d, 0x09, 0xc8])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x49, 0xf7, 0xd0])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x4d, 0x21, 0xc8])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x4c, 0x8b, 0x06])
            && user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x4c, 0x89, 0x0a])
            && user_syscall
                .windows(9)
                .any(|bytes| bytes == [0x65, 0x4c, 0x89, 0x04, 0x25, 0xb8, 0, 0, 0])
            && user_syscall
                .windows(13)
                .any(|bytes| bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xb8, 0, 0, 0, 0, 0, 0, 0]),
        "rt_sigprocmask must stage native input, remove unblockable bits, update private state, publish the prior word, and clear state at exit"
    );
    let user_protect = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL);
    assert!(
        user_protect
            .windows(4)
            .any(|bytes| bytes == [0x49, 0x83, 0xe0, 0xfd])
            && user_protect
                .windows(3)
                .any(|bytes| bytes == [0x0f, 0xae, 0xf0])
            && user_protect
                .windows(4)
                .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x3a])
            && user_protect
                .windows(5)
                .any(|bytes| bytes == [0xb8, 0x0c, 0, 0, 0])
            && user_protect
                .windows(5)
                .any(|bytes| bytes == [0xb8, 0x13, 0, 0, 0]),
        "user protection must revoke the writable bit, order publication, invalidate the target, and retain generation"
    );
    let user_load = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL);
    assert!(
        user_load
            .windows(3)
            .any(|bytes| bytes == [0x49, 0x8b, 0x00])
            && user_load
                .windows(3)
                .any(|bytes| bytes == [0x49, 0x39, 0x02])
            && user_load
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0x3a, 0x00])
            && user_load
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xe9, 0x08])
            && user_load
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xeb, 0x08])
            && user_load
                .windows(4)
                .any(|bytes| bytes == data_payload_offset.to_le_bytes())
            && user_load
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x13])
            && user_load
                .windows(5)
                .any(|bytes| bytes == [0xb8, 0x14, 0, 0, 0])
            && user_load
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x80, 0, 0, 0, 0, 0, 0, 0]),
        "user load must compare complete runtime initialized-data and BSS extents, revoking on mismatch"
    );
    let user_verify = symbol_bytes(
        &file,
        X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL,
    );
    assert!(
        user_verify
            .windows(8)
            .any(|bytes| bytes == INITIAL_USER_CODE_ADDRESS.to_le_bytes())
            && user_verify
                .windows(3)
                .filter(|bytes| *bytes == [0x49, 0x8b, 0x00])
                .count()
                == 1
            && user_verify
                .windows(3)
                .filter(|bytes| *bytes == [0x49, 0x39, 0x02])
                .count()
                == 1
            && user_verify
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xc0, 0x08])
            && user_verify
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xc2, 0x08])
            && user_verify
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xe9, 0x08])
            && user_verify
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x06]),
        "entry verification must compare the complete runtime-derived extent to the external ELF source"
    );
    let user_execute = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL);
    assert!(
        user_execute
            .windows(5)
            .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xe0, 0x3f])
            && user_execute
                .windows(5)
                .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xf0, 0x3f])
            && user_execute
                .windows(4)
                .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x3a])
            && user_execute
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x0e]),
        "user execute promotion must accept only generation one, require NX, clear it, and invalidate the target"
    );
    let user_data = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL);
    assert!(
        user_data
            .windows(7)
            .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x14])
            && user_data
                .windows(4)
                .any(|bytes| bytes == [0x41, 0xf6, 0xc0, 0x02])
            && user_data
                .windows(5)
                .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xe0, 0x3f])
            && user_data
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x15]),
        "immutable-data sealing must accept only generation two and preserve its RO/NX permissions"
    );
    let writable_data = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL);
    assert!(
        writable_data
            .windows(7)
            .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x14])
            && writable_data
                .windows(4)
                .any(|bytes| bytes == [0x41, 0xf6, 0xc0, 0x02])
            && writable_data
                .windows(5)
                .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xe0, 0x3f])
            && writable_data
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xc8, 0x02])
            && writable_data
                .windows(3)
                .any(|bytes| bytes == [0x0f, 0xae, 0xf0])
            && writable_data
                .windows(4)
                .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x3a])
            && writable_data
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x16]),
        "writable-data promotion must accept verified generation two, restore write while retaining NX, and invalidate"
    );
    let reserve_anonymous = symbol_bytes(
        &file,
        X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
    );
    let release_anonymous = symbol_bytes(
        &file,
        X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
    );
    assert!(
        reserve_anonymous
            .windows(7)
            .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x16])
            && reserve_anonymous
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xc0, 0, 0, 0, 0])
            && reserve_anonymous
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xc1, 0x30, 0x20, 0, 0])
            && reserve_anonymous
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x17]),
        "anonymous reservation must consume one frame while retaining a non-present provider-selected PTE"
    );
    assert!(
        release_anonymous
            .windows(7)
            .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x17])
            && release_anonymous
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xc1, 0x30, 0x20, 0, 0])
            && release_anonymous
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xc0, 0, 0, 0, 0, 0, 0, 0]),
        "anonymous release must require completed execution, an absent PTE, and return the exact frame"
    );
    let user_unmap = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL);
    assert!(
        user_unmap
            .windows(8)
            .any(|bytes| bytes == [0x48, 0xc7, 0x41, 0x08, 0, 0, 0, 0])
            && user_unmap
                .windows(4)
                .filter(|bytes| *bytes == [0x41, 0x0f, 0x01, 0x3a])
                .count()
                >= 2
            && user_unmap
                .windows(8)
                .any(|bytes| bytes == [0x48, 0xc7, 0x47, 0x78, 0, 0, 0, 0])
            && user_unmap
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x80, 0, 0, 0, 0, 0, 0, 0])
            && user_unmap
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x90, 0, 0, 0, 0, 0, 0, 0])
            && user_unmap
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x98, 0, 0, 0, 0, 0, 0, 0])
            && user_unmap
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xa0, 0, 0, 0, 0, 0, 0, 0])
            && user_unmap
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xa8, 0, 0, 0, 0])
            && user_unmap
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xa8, 0, 0, 0, 0, 0, 0, 0])
            && user_unmap
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xb0, 0, 0, 0, 0])
            && user_unmap
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xb0, 0, 0, 0, 0, 0, 0, 0])
            && user_unmap.windows(2).any(|bytes| bytes == [0x0f, 0x30])
            && user_unmap
                .windows(8)
                .any(|bytes| bytes == 0x0060_5000_u64.to_le_bytes())
            && user_unmap
                .windows(8)
                .any(|bytes| bytes == 0x0060_3000_u64.to_le_bytes())
            && !user_unmap
                .windows(8)
                .any(|bytes| bytes == 0x0060_2000_u64.to_le_bytes()),
        "user unmap must revoke, invalidate, clear break, FS, and mapping state, and return the frame"
    );
    let user_complete = symbol_bytes(&file, X86_SYSTEMS_USER_COMPLETE_SYMBOL);
    assert!(
        user_complete
            .windows(11)
            .any(|bytes| bytes == [0x48, 0xc7, 0x80, 0x18, 0x20, 0x00, 0x00, 0, 0, 0, 0])
    );
    assert_eq!(
        user_complete
            .windows(3)
            .filter(|bytes| *bytes == [0x0f, 0x01, 0x3a])
            .count(),
        3,
        "user completion must invalidate code, stack, and guard locations"
    );
    let user_reclaim = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL);
    assert!(
        user_reclaim
            .windows(4)
            .any(|bytes| bytes == [0x48, 0x8b, 0x47, 0x68]),
        "address-space reclaim must return the retained backing identity"
    );
    let user_enter = symbol_bytes(&file, X86_SYSTEMS_USER_ENTER_SYMBOL);
    assert!(user_enter.windows(2).any(|bytes| bytes == [0x48, 0xcf]));
    assert_eq!(&user_enter[user_enter.len() - 3..], &[0x31, 0xc0, 0xc3]);
    assert!(
        user_enter
            .windows(8)
            .any(|bytes| bytes == INITIAL_USER_STACK_ADDRESS.to_le_bytes())
    );
    assert!(
        user_enter
            .windows(7)
            .any(|bytes| bytes == [0x48, 0x8b, 0x87, 0x88, 0, 0, 0])
            && !user_enter
                .windows(8)
                .any(|bytes| bytes == INITIAL_USER_ENTRY_ADDRESS.to_le_bytes()),
        "initial user instruction fetch must use the retained runtime ELF entry"
    );
    entry_payload
}
