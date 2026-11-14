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
    fixture_selected(false).image
}

pub(crate) struct CallerFixture {
    pub image: Vec<u8>,
    pub assertions: Vec<u64>,
    pub startup_completed: u64,
    pub shell_completed: u64,
    pub stat_completed: u64,
    pub stat_pages: [u64; 4],
    pub statat_completed: u64,
    pub statat_pages: [u64; 6],
    pub write_pages: [u64; write_inputs::LENGTHS.len()],
    pub file_cases: Vec<file_transfers::Observation>,
    pub shell_io_cases: Vec<shell_io::Observation>,
    pub command_cases: Vec<command_input::Observation>,
}

pub(crate) const WRITE_LENGTHS: [u8; write_inputs::LENGTHS.len()] = write_inputs::LENGTHS;

pub(crate) fn caller_fixture() -> CallerFixture {
    fixture_selected(true)
}

fn fixture_selected(user_buffers: bool) -> CallerFixture {
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
    let mut fixture = CallerFixture {
        image,
        assertions: vec![],
        startup_completed: 0,
        shell_completed: 0,
        stat_completed: 0,
        stat_pages: [0; 4],
        statat_completed: 0,
        statat_pages: [0; 6],
        write_pages: [0; write_inputs::LENGTHS.len()],
        file_cases: vec![],
        shell_io_cases: vec![],
        command_cases: vec![],
    };
    if user_buffers {
        fixture.assertions = install_ordinary_buffer_requests(&mut fixture.image);
        let (assertions, completed) = startup_outputs::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.startup_completed = completed;
        let (assertions, completed) = shell_outputs::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.shell_completed = completed;
        let (assertions, completed, pages) = stat_outputs::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.stat_completed = completed;
        fixture.stat_pages = pages;
        let (assertions, completed, pages) = statat_paths::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.statat_completed = completed;
        fixture.statat_pages = pages;
        let (assertions, pages) = write_inputs::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.write_pages = pages;
        let (assertions, cases) = file_transfers::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.file_cases = cases;
        let (assertions, cases) = shell_io::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.shell_io_cases = cases;
        let (assertions, cases) = command_input::install(&mut fixture.image);
        fixture.assertions.extend(assertions);
        fixture.command_cases = cases;
    }
    fixture
}

fn install_ordinary_buffer_requests(image: &mut [u8]) -> Vec<u64> {
    // The ordinary descriptor/time domains are live at the existing time-test
    // entry, rather than during the earlier sealed startup/getuid sequence.
    let entry = crate::systems_artifact::static_time_fixture_entry();
    let at = entry as usize - 0x400000;
    let prologue = [0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57];
    assert_eq!(&image[at..at + prologue.len()], &prologue);
    let cave = 0x181000_usize;
    let mut code = prologue.to_vec();
    let mut assertions = vec![];
    request(&mut code, 0, 0x619000);
    assertions.push(code.len() - 2);
    code.extend_from_slice(&[0x48, 0x89, 0xc3]); // retain the original byte break
    for (rex, address) in [(0x4c, 0x5ffff8_u32), (0x4c, 0x600000)] {
        code.extend_from_slice(&[
            rex,
            0x8b,
            if address == 0x5ffff8 { 0x24 } else { 0x2c },
            0x25,
        ]);
        code.extend_from_slice(&address.to_le_bytes()); // R12/R13 retain old user bytes
    }
    request(&mut code, 0x63a000, 0x63a000);
    assertions.push(code.len() - 2);
    ordinary_buffer_requests(&mut code, &mut assertions);
    for (move_rax, address) in [
        ([0x4c, 0x89, 0xe0], 0x5ffff8_u64),
        ([0x4c, 0x89, 0xe8], 0x600000),
    ] {
        code.extend_from_slice(&move_rax);
        code.extend_from_slice(&[0x48, 0xa3]);
        code.extend_from_slice(&address.to_le_bytes());
    }
    code.extend_from_slice(&[
        0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
    ]);
    assertions.push(code.len() - 2);
    code.extend_from_slice(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    code.extend_from_slice(&prologue);
    code.push(0xe9);
    let after = (0x400000 + cave + code.len() + 4) as i64;
    let displacement = i32::try_from(i64::from(entry) + prologue.len() as i64 - after).unwrap();
    code.extend_from_slice(&displacement.to_le_bytes());
    assert!(image[cave..cave + code.len()].iter().all(|&b| b == 0));
    image[cave..cave + code.len()].copy_from_slice(&code);
    image[at..at + prologue.len()].fill(0x90);
    image[at] = 0xe9;
    let displacement = i32::try_from(cave as i64 - (at + 5) as i64).unwrap();
    image[at + 1..at + 5].copy_from_slice(&displacement.to_le_bytes());
    assertions
        .into_iter()
        .map(|offset| 0x400000 + cave as u64 + offset as u64)
        .collect()
}

fn ordinary_buffer_requests(code: &mut Vec<u8>, assertions: &mut Vec<usize>) {
    fn syscall(
        code: &mut Vec<u8>,
        assertions: &mut Vec<usize>,
        number: u32,
        first: u64,
        second: u64,
    ) {
        for (op, value) in [(0xbf, first), (0xbe, second)] {
            code.extend_from_slice(&[0x48, op]);
            code.extend_from_slice(&value.to_le_bytes());
        }
        code.push(0xb8);
        code.extend_from_slice(&number.to_le_bytes());
        code.extend_from_slice(&[0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe]);
        assertions.push(code.len() - 2);
    }
    // Two ordinary output layouts: a leaf-table crossing and a split-page
    // timespec beyond the predecessor's 34-page heap.
    for address in [0x5ffff8_u32, 0x638ff8] {
        syscall(code, assertions, 229, 1, u64::from(address));
        for (at, want) in [(address, 0), (address + 8, 10)] {
            code.extend_from_slice(&[0x48, 0x83, 0x3c, 0x25]);
            code.extend_from_slice(&at.to_le_bytes());
            code.extend_from_slice(&[want, 0x75, 0xfe]);
            assertions.push(code.len() - 2);
        }
    }
    // A complete readable timespec in the same large-heap split-page range.
    for address in [0x638ff8_u64, 0x639000] {
        code.extend_from_slice(&[0x48, 0x31, 0xc0, 0x48, 0xa3]);
        code.extend_from_slice(&address.to_le_bytes());
    }
    syscall(code, assertions, 35, 0x638ff8, 0);
    syscall(code, assertions, 228, 1, 0x638ff8);
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
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_command_fixture.rs"]
mod command_input;
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_file_fixture.rs"]
mod file_transfers;
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_shell_io_fixture.rs"]
mod shell_io;
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_shell_fixture.rs"]
mod shell_outputs;
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_startup_fixture.rs"]
mod startup_outputs;
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_stat_fixture.rs"]
mod stat_outputs;
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_statat_fixture.rs"]
mod statat_paths;
#[path = "../../../linux-kernel/labs/qemu/x86_64/user_extent_write_fixture.rs"]
mod write_inputs;
