//! Ordinary user programs redirect startup outputs to a grown, split-page heap.
//! TOPAL-SYSTEMS-USER-EXTENT-001. No debugger state edits supply these requests.
pub(super) const BASE: usize = 0x400000;
pub(super) const PUSH: &[u8] = &[
    0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57, 0x57, 0x56, 0x52, 0x41, 0x52, 0x41,
    0x50, 0x41, 0x51,
];
pub(super) const POP: &[u8] = &[
    0x41, 0x59, 0x41, 0x58, 0x41, 0x5a, 0x5a, 0x5e, 0x5f, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41,
    0x5c, 0x5d, 0x5b,
];

pub(super) fn immediate(code: &mut Vec<u8>, register: u8, value: u64) {
    code.extend_from_slice(&[0x48 | (register >> 3), 0xb8 | (register & 7)]);
    code.extend_from_slice(&value.to_le_bytes());
}
pub(super) fn compare(code: &mut Vec<u8>, assertions: &mut Vec<usize>, expected: i32) {
    code.extend_from_slice(&[0x48, 0x3d]);
    code.extend_from_slice(&expected.to_le_bytes());
    code.extend_from_slice(&[0x75, 0xfe]);
    assertions.push(code.len() - 2);
}
pub(super) fn brk(code: &mut Vec<u8>, assertions: &mut Vec<usize>, request: u64, expected: i32) {
    immediate(code, 7, request);
    code.extend_from_slice(&[0xb8, 12, 0, 0, 0, 0x0f, 0x05]);
    compare(code, assertions, expected);
}
pub(super) fn restore_arguments(code: &mut Vec<u8>) {
    // Restore all six original syscall arguments from the user-owned CALL
    // frame after the intervening brk calls, then redirect only the output.
    for (register, offset) in [(7, 40), (6, 32), (2, 24), (10, 16), (8, 8), (9, 0)] {
        code.extend_from_slice(&[
            0x48 | ((register >> 3) << 2),
            0x8b,
            0x44 | ((register & 7) << 3),
            0x24,
            offset,
        ]);
    }
}

pub(super) fn request(
    code: &mut Vec<u8>,
    assertions: &mut Vec<usize>,
    number: u32,
    output_register: u8,
    pointer: u64,
    expected: i32,
) {
    restore_arguments(code);
    immediate(code, output_register, pointer);
    code.push(0xb8);
    code.extend_from_slice(&number.to_le_bytes());
    code.extend_from_slice(&[0x0f, 0x05]);
    compare(code, assertions, expected);
}

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, u64) {
    let mut all_assertions = vec![];
    let mut completed = 0;
    for (index, number, output_register, pattern) in [
        (
            0,
            302_u32,
            10,
            vec![0x49, 0x89, 0xe2, 0xb8, 0x2e, 1, 0, 0, 0x0f, 0x05],
        ),
        (
            1,
            318,
            7,
            vec![
                0xbf, 0x68, 0x5e, 0x5f, 0, 0xbe, 8, 0, 0, 0, 0xba, 1, 0, 0, 0, 0xb8, 0x3e, 1, 0, 0,
                0x0f, 0x05,
            ],
        ),
        (
            2,
            318,
            7,
            vec![
                0xbf, 0x70, 0x5e, 0x5f, 0, 0xbe, 8, 0, 0, 0, 0xba, 1, 0, 0, 0, 0xb8, 0x3e, 1, 0, 0,
                0x0f, 0x05,
            ],
        ),
        (
            3,
            157,
            6,
            vec![
                0xbf, 16, 0, 0, 0, 0x48, 0x89, 0xe6, 0xb8, 0x9d, 0, 0, 0, 0x0f, 0x05,
            ],
        ),
    ] {
        let sites = image
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(at, bytes)| (bytes == pattern).then_some(at))
            .collect::<Vec<_>>();
        assert_eq!(sites.len(), 1, "startup syscall {number} / {index}");
        let site = sites[0] + pattern.len() - 7;
        let cave = 0x182000 + index * 0x1000;
        // The historical first image has sealed auxiliary fields below its
        // native continuation RSP. Keep this user CALL's saves below that
        // block, just as the predecessor's stack-limit/name scratch does.
        let mut code = vec![0x48, 0x81, 0xec, 0x40, 1, 0, 0];
        code.extend_from_slice(PUSH);
        let mut assertions = vec![];
        // R12 keeps the original destination; RBX keeps the actual old break.
        code.extend_from_slice(&[
            0x49 | ((output_register >> 3) << 2),
            0x89,
            0xc4 | ((output_register & 7) << 3),
        ]);
        let old_break = if number == 157 { 0x619000 } else { 0x5f7d80 };
        brk(&mut code, &mut assertions, 0, old_break);
        code.extend_from_slice(&[0x48, 0x89, 0xc3]);
        brk(&mut code, &mut assertions, 0x63a000, 0x63a000);
        // Ordinary invalid output arguments must leave the startup operation
        // available for the subsequent valid request, without partial output.
        for pointer in [0x63a000, 0x401000] {
            request(
                &mut code,
                &mut assertions,
                number,
                output_register,
                pointer,
                -14,
            );
        }
        let (pointer, count, result) = if number == 318 {
            (0x638ffc_u64, 8, 8)
        } else {
            (0x638ff8, 16, 0)
        };
        request(
            &mut code,
            &mut assertions,
            number,
            output_register,
            pointer,
            result,
        );
        let expected = match number {
            302 => Some([0x800000_u64, u64::MAX]),
            157 => Some([u64::from_le_bytes(*b"busybox\0"), 0]),
            _ => None,
        };
        for offset in (0..count).step_by(8) {
            code.extend_from_slice(&[0x48, 0xa1]);
            code.extend_from_slice(&(pointer + offset as u64).to_le_bytes());
            if let Some(words) = expected {
                immediate(&mut code, 13, words[offset / 8]);
                code.extend_from_slice(&[0x4c, 0x39, 0xe8, 0x75, 0xfe]);
                assertions.push(code.len() - 2);
            }
            code.extend_from_slice(if offset == 0 {
                &[0x49, 0x89, 0x04, 0x24]
            } else {
                &[0x49, 0x89, 0x44, 0x24, 8]
            });
        }
        code.extend_from_slice(&[
            0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
        ]);
        assertions.push(code.len() - 2);
        if number == 157 {
            completed = (BASE + cave + code.len()) as u64;
        }
        code.extend_from_slice(POP);
        code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
        immediate(&mut code, 0, result as u64);
        code.push(0xc3);
        assert!(code.len() < 0x1000);
        assert!(image[cave..cave + code.len()].iter().all(|&b| b == 0));
        image[cave..cave + code.len()].copy_from_slice(&code);
        image[site..site + 7].fill(0x90);
        image[site] = 0xe8;
        let displacement = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
        image[site + 1..site + 5].copy_from_slice(&displacement.to_le_bytes());
        all_assertions.extend(assertions.into_iter().map(|at| (BASE + cave + at) as u64));
    }
    assert_ne!(completed, 0);
    (all_assertions, completed)
}
