//! Real uname/getcwd requests preserve the original full ABI continuation.
//! TOPAL-SYSTEMS-USER-EXTENT-001. The debugger only observes the requests.
use super::startup_outputs::{BASE, POP, PUSH, brk, immediate, request};

pub(super) fn sentinel(code: &mut Vec<u8>, assertions: &mut Vec<usize>, pointer: u32, check: bool) {
    code.extend_from_slice(if check {
        &[0x80, 0x3c, 0x25]
    } else {
        &[0xc6, 0x04, 0x25]
    });
    code.extend_from_slice(&pointer.to_le_bytes());
    code.push(0xa5);
    if check {
        code.extend_from_slice(&[0x75, 0xfe]);
        assertions.push(code.len() - 2);
    }
}

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, u64) {
    let mut requests = vec![];
    for (number, pattern, expected_sites) in [
        (63, vec![0x48, 0x89, 0xe7, 0xb8, 63, 0, 0, 0, 0x0f, 0x05], 1),
        (
            79,
            vec![
                0xbf, 0x60, 0x94, 0x5f, 0, 0xbe, 0, 0x10, 0, 0, 0xb8, 79, 0, 0, 0, 0x0f, 0x05,
            ],
            3,
        ),
    ] {
        let sites = image
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(at, bytes)| (bytes == pattern).then_some(at + pattern.len() - 7))
            .collect::<Vec<_>>();
        assert_eq!(sites.len(), expected_sites, "shell syscall {number}");
        requests.extend(sites.into_iter().map(|site| (number, site)));
    }
    let mut all_assertions = vec![];
    let mut initial_cwd_completed = 0;
    for (index, (number, site)) in requests.into_iter().enumerate() {
        let cave = 0x186000 + index * 0x1000;
        let (pointer, count, result, partial) = if number == 63 {
            (0x638ff8_u64, 390, 0, 0x639ff8)
        } else {
            (0x638fff, 2, 2, 0x639fff)
        };
        let mut code = vec![0x48, 0x81, 0xec, 0x40, 1, 0, 0];
        code.extend_from_slice(PUSH);
        code.extend_from_slice(&[0x49, 0x89, 0xfc]); // retain original RDI in R12
        let mut assertions = vec![];
        brk(&mut code, &mut assertions, 0, 0x619000);
        code.extend_from_slice(&[0x48, 0x89, 0xc3]); // actual previous break
        brk(&mut code, &mut assertions, 0x63a000, 0x63a000);
        sentinel(&mut code, &mut assertions, partial, false);
        for destination in [0x63a000, 0x401000, u64::from(partial)] {
            request(&mut code, &mut assertions, number, 7, destination, -14);
        }
        sentinel(&mut code, &mut assertions, partial, true);
        for at in [pointer - 1, pointer + count] {
            sentinel(&mut code, &mut assertions, at as u32, false);
        }
        request(&mut code, &mut assertions, number, 7, pointer, result);
        for at in [pointer - 1, pointer + count] {
            sentinel(&mut code, &mut assertions, at as u32, true);
        }
        // Compare every output byte against independently declared fixture
        // fields, including every field's complete zero padding/terminator.
        immediate(&mut code, 7, pointer);
        code.extend_from_slice(&[0x48, 0x8d, 0x35]);
        let expected_displacement = code.len();
        code.extend_from_slice(&[0; 4]);
        immediate(&mut code, 1, count);
        code.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
        assertions.push(code.len() - 2);
        immediate(&mut code, 6, pointer);
        code.extend_from_slice(&[0x4c, 0x89, 0xe7]);
        immediate(&mut code, 1, count);
        code.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
        code.extend_from_slice(&[
            0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
        ]);
        assertions.push(code.len() - 2);
        if index == 1 {
            initial_cwd_completed = (BASE + cave + code.len()) as u64;
        }
        code.extend_from_slice(POP);
        code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
        immediate(&mut code, 0, result as u64);
        code.push(0xc3);
        let displacement = i32::try_from(code.len() - expected_displacement - 4).unwrap();
        code[expected_displacement..expected_displacement + 4]
            .copy_from_slice(&displacement.to_le_bytes());
        if number == 63 {
            let mut expected = [0; 390];
            for (field, value) in [
                "Linux",
                "topal",
                "7.2.9-topal",
                "#1 Topal",
                "x86_64",
                "(none)",
            ]
            .into_iter()
            .enumerate()
            {
                expected[field * 65..field * 65 + value.len()].copy_from_slice(value.as_bytes());
            }
            code.extend_from_slice(&expected);
        } else {
            code.extend_from_slice(b"/\0");
        }
        assert!(code.len() < 0x1000);
        assert!(image[cave..cave + code.len()].iter().all(|&b| b == 0));
        image[cave..cave + code.len()].copy_from_slice(&code);
        image[site..site + 7].fill(0x90);
        image[site] = 0xe8;
        let displacement = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
        image[site + 1..site + 5].copy_from_slice(&displacement.to_le_bytes());
        all_assertions.extend(assertions.into_iter().map(|at| (BASE + cave + at) as u64));
    }
    assert_ne!(initial_cwd_completed, 0);
    (all_assertions, initial_cwd_completed)
}
