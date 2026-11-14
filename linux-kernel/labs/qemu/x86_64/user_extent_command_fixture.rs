//! Ordinary short reads and retries on the first-image command stream.
//! TOPAL-SYSTEMS-USER-EXTENT-001.

use super::shell_outputs::sentinel;
use super::startup_outputs::{BASE, POP, PUSH, brk, compare, immediate, restore_arguments};

const COMMANDS: [&[u8]; 5] = [
    b"echo topal\n",
    b"cd /\n",
    b"echo /*; test -d /bin && echo /bin/*; test -f /bin/busybox && echo busybox-file; \
        test -x /bin/busybox && echo busybox-executable; test -s /bin/busybox && echo busybox-nonempty; \
        exec 3</bin/busybox; echo busybox-open; IFS= read -r -n 4 -u 3 _ && echo busybox-read; \
        exec 3<&-; echo busybox-close\n",
    b"exec 3</bin/busybox;pwd\n",
    b"exec /bin/busybox true\n",
];
const HOLE: u32 = 0x63a000;
const PREFIX: u32 = 0x638fff;
const ORIGINAL_BUFFER: u32 = 0x5f8f60;

#[derive(serde::Serialize)]
pub(crate) struct Observation {
    page: u64,
    length: u32,
    capacity: u32,
}

fn read(code: &mut Vec<u8>, assertions: &mut Vec<usize>, pointer: u32, count: u32, result: i32) {
    restore_arguments(code);
    immediate(code, 6, u64::from(pointer));
    immediate(code, 2, u64::from(count));
    code.extend_from_slice(&[0x31, 0xc0, 0x0f, 0x05]);
    compare(code, assertions, result);
}

fn check_and_copy(
    code: &mut Vec<u8>,
    assertions: &mut Vec<usize>,
    pointer: u32,
    offset: u8,
    bytes: &[u8],
) {
    for (index, expected) in bytes.iter().enumerate() {
        code.extend_from_slice(&[0x80, 0x3c, 0x25]);
        code.extend_from_slice(&(pointer + index as u32).to_le_bytes());
        code.extend_from_slice(&[*expected, 0x75, 0xfe]);
        assertions.push(code.len() - 2);
    }
    immediate(code, 6, u64::from(pointer));
    code.extend_from_slice(&[0x49, 0x8d, 0x7c, 0x24, offset]); // lea rdi,[r12+offset]
    immediate(code, 1, bytes.len() as u64);
    code.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
}

fn adapt_original_readback(image: &mut [u8], site: usize, case: usize, command: &[u8]) {
    let mut at = site + 9;
    let mut result_check = if command.len() < 128 {
        vec![0x48, 0x83, 0xf8, command.len() as u8]
    } else {
        let mut check = vec![0x48, 0x3d];
        check.extend_from_slice(&(command.len() as u32).to_le_bytes());
        check
    };
    result_check.extend_from_slice(&[0x75, 0xfe]);
    assert_eq!(image[at..at + result_check.len()], result_check);
    at += result_check.len();
    if case == 4 {
        // The application adds its own terminator after the serial result.
        assert_eq!(
            image[at..at + 8],
            [0xc6, 0x04, 0x25, 0x77, 0x8f, 0x5f, 0, 0]
        );
        at += 8;
    }
    for (index, bytes) in command.chunks(8).enumerate() {
        let address = ORIGINAL_BUFFER + (index * 8) as u32;
        let mut expected = [0_u8; 20];
        expected[..2].copy_from_slice(&[0x48, 0xb8]);
        expected[2..2 + bytes.len()].copy_from_slice(bytes);
        expected[10..14].copy_from_slice(&[0x48, 0x39, 0x04, 0x25]);
        expected[14..18].copy_from_slice(&address.to_le_bytes());
        expected[18..].copy_from_slice(&[0x75, 0xfe]);
        assert_eq!(image[at..at + expected.len()], expected);
        // The new fixture checks this partial word byte by byte in the
        // returned range. Keep every original complete-word assertion.
        if bytes.len() < 8 {
            image[at..at + expected.len()].fill(0x90);
        }
        at += expected.len();
    }
    if matches!(case, 1 | 3 | 4) {
        let beyond = ORIGINAL_BUFFER + command.len().next_multiple_of(8) as u32;
        let mut expected = vec![0x48, 0x83, 0x3c, 0x25];
        expected.extend_from_slice(&beyond.to_le_bytes());
        expected.extend_from_slice(&[0, 0x75, 0xfe]);
        assert_eq!(image[at..at + expected.len()], expected);
        image[at..at + expected.len()].fill(0x90);
    }
}

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, Vec<Observation>) {
    let mut sites = Vec::new();
    for capacity in [1023_u32, 23] {
        let mut signature = vec![0xba];
        signature.extend_from_slice(&capacity.to_le_bytes());
        signature.extend_from_slice(&[0x31, 0xc0, 0x0f, 0x05]);
        sites.extend(
            image[0x0a7600..0x0a9000]
                .windows(signature.len())
                .enumerate()
                .filter_map(|(at, bytes)| {
                    (bytes == signature).then_some((0x0a7600 + at, capacity))
                }),
        );
    }
    assert_eq!(sites.len(), COMMANDS.len());
    assert!(
        sites
            .iter()
            .map(|&(_, count)| count)
            .eq([1023, 1023, 1023, 1023, 23])
    );

    let mut all_assertions = Vec::new();
    let mut metadata = Vec::new();
    for (case, ((site, capacity), command)) in sites.into_iter().zip(COMMANDS).enumerate() {
        adapt_original_readback(image, site, case, command);
        let cave = 0x120000 + case * 4096;
        let page = (BASE + cave) as u64;
        let length = command.len() as u32;
        let suffix = length - 2;
        let mut code = vec![0xba];
        code.extend_from_slice(&capacity.to_le_bytes());
        code.extend_from_slice(&[0x48, 0x81, 0xec, 0x40, 1, 0, 0]);
        code.extend_from_slice(PUSH);
        code.extend_from_slice(&[0x49, 0x89, 0xf4]); // original destination in r12
        let mut assertions = Vec::new();
        brk(&mut code, &mut assertions, 0, 0x619000);
        code.extend_from_slice(&[0x48, 0x89, 0xc3]);
        brk(&mut code, &mut assertions, u64::from(HOLE), HOLE as i32);

        read(&mut code, &mut assertions, HOLE, 0, 0);
        read(&mut code, &mut assertions, HOLE, capacity, -14);
        sentinel(&mut code, &mut assertions, HOLE - 1, false);
        read(&mut code, &mut assertions, HOLE - 1, length, -14);
        sentinel(&mut code, &mut assertions, HOLE - 1, true);

        for at in [PREFIX - 1, PREFIX + 2] {
            sentinel(&mut code, &mut assertions, at, false);
        }
        read(&mut code, &mut assertions, PREFIX, 2, 2);
        for at in [PREFIX - 1, PREFIX + 2] {
            sentinel(&mut code, &mut assertions, at, true);
        }
        check_and_copy(&mut code, &mut assertions, PREFIX, 0, &command[..2]);
        read(&mut code, &mut assertions, HOLE, capacity, -14);

        let tail = HOLE - suffix;
        sentinel(&mut code, &mut assertions, tail - 1, false);
        read(&mut code, &mut assertions, tail, capacity, suffix as i32);
        sentinel(&mut code, &mut assertions, tail - 1, true);
        check_and_copy(&mut code, &mut assertions, tail, 2, &command[2..]);

        code.extend_from_slice(&[
            0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
        ]);
        assertions.push(code.len() - 2);
        code.extend_from_slice(POP);
        code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
        immediate(&mut code, 0, u64::from(length));
        code.push(0xc3);

        assert!(code.len() < 4096);
        assert!(image[cave..cave + code.len()].iter().all(|&byte| byte == 0));
        image[cave..cave + code.len()].copy_from_slice(&code);
        image[site..site + 9].fill(0x90);
        image[site] = 0xe8;
        let delta = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
        image[site + 1..site + 5].copy_from_slice(&delta.to_le_bytes());
        all_assertions.extend(assertions.into_iter().map(|at| page + at as u64));
        metadata.push(Observation {
            page,
            length,
            capacity,
        });
    }
    (all_assertions, metadata)
}
