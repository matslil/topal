#[test]
fn frame_memory_uses_actual_syscall_requests_and_whole_exec_exit_release() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: the complete actual boot Root
    // and userspace ABI fixture, with runtime backing instead of a fixed heap.
    let work = super::root_tests::execute(true);
    inspect_faults(&work);
}

pub(crate) fn inspect_faults(work: &std::path::Path) {
    for mode in [
        "return-site",
        "gs-context",
        "address-space",
        "entry-identity",
        "abi-number",
        "captured-stack",
        "request",
        "busy",
        "direction-flag",
        "partial-source",
        "published-unpin",
        "no-fit",
    ] {
        crate::systems_tail_owner::run_tail_native_fixture(
            work,
            &format!("memory-{mode}"),
            std::process::Command::new("python3")
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../linux-kernel/labs/qemu/x86_64/frame_memory_fault_inspect.py"),
                )
                .arg(&work)
                .arg(mode),
        );
    }
}

pub(crate) fn fixture() -> Vec<u8> {
    // Keep all predecessor continuation addresses and ELF geometry. Only this
    // unpublished fixture calls the extra user-mode requests from unused RX
    // bytes after its ordinary startup/getuid sequence has completed.
    let mut image = crate::qualified_x86_64_static_user_image_fixture();
    let original = [
        0xb8, 0x66, 0, 0, 0, 0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe,
    ];
    let sites = image
        .windows(original.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == original).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(sites.len(), 1);
    let site = sites[0];
    let cave = 0x180000_usize;
    let mut code = original.to_vec();
    // SYSCALL must mask DF in the kernel while SYSRET preserves the user's DF.
    code.push(0xfd); // std
    request(&mut code, 0, 0x619000);
    code.extend_from_slice(&[
        0x9c, 0x41, 0x5a, 0x49, 0xf7, 0xc2, 0, 4, 0, 0, 0x74, 0xfe, 0xfc,
    ]);
    byte(&mut code, 0x5f7000, 0xa7, false);
    request(&mut code, 0x63a000, 0x63a000); // 67 pages
    byte(&mut code, 0x5f7000, 0xa7, true);
    byte(&mut code, 0x639fff, 0, true);
    byte(&mut code, 0x639fff, 0xb6, false);
    request(&mut code, 0, 0x63a000);
    request(&mut code, 0x639ff1, 0x639ff1); // same extent, different byte break
    request(&mut code, 0x5f6fff, 0x639ff1); // below the image/heap boundary
    request(&mut code, u64::MAX, 0x639ff1); // unsigned overflow-sized request
    request(&mut code, 0x5fa000, 0x5fa000); // three retained pages
    byte(&mut code, 0x5f7000, 0xa7, true);
    request(&mut code, 0x63a000, 0x63a000);
    byte(&mut code, 0x639fff, 0, true); // dropped page cannot retain its byte
    request(&mut code, 0x7fe000, 0x7fe000); // all 519 current leaf positions
    byte(&mut code, 0x5f7000, 0xa7, true);
    byte(&mut code, 0x7fdfff, 0, true);
    byte(&mut code, 0x7fdfff, 0xc9, false);
    request(&mut code, 0x5fa000, 0x5fa000);
    request(&mut code, 0x7fe000, 0x7fe000);
    byte(&mut code, 0x5f7000, 0xa7, true);
    byte(&mut code, 0x7fdfff, 0, true);
    request(&mut code, 0, 0x7fe000);
    request(&mut code, 0x5f7000, 0x5f7000); // revoke and release the whole heap
    request(&mut code, 0x619000, 0x619000); // predecessor remaining ABI checks
    byte(&mut code, 0x5f7000, 0, true);
    code.push(0xc3);
    assert!(image[cave..cave + code.len()].iter().all(|&b| b == 0));
    image[cave..cave + code.len()].copy_from_slice(&code);
    image[site..site + original.len()].fill(0x90);
    image[site] = 0xe8;
    let displacement = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
    image[site + 1..site + 5].copy_from_slice(&displacement.to_le_bytes());
    image
}

fn request(code: &mut Vec<u8>, value: u64, expected: u32) {
    code.extend_from_slice(&[0x48, 0xbf]); // movabs request,%rdi
    code.extend_from_slice(&value.to_le_bytes());
    code.extend_from_slice(&[0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x3d]);
    code.extend_from_slice(&expected.to_le_bytes());
    code.extend_from_slice(&[0x75, 0xfe]);
}

fn byte(code: &mut Vec<u8>, address: u32, value: u8, compare: bool) {
    code.extend_from_slice(if compare {
        &[0x80, 0x3c, 0x25]
    } else {
        &[0xc6, 0x04, 0x25]
    });
    code.extend_from_slice(&address.to_le_bytes());
    code.push(value);
    if compare {
        code.extend_from_slice(&[0x75, 0xfe]);
    }
}
