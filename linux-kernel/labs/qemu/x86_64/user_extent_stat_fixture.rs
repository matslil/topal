//! Complete native fstat records through ordinary user-issued requests.
//! TOPAL-SYSTEMS-USER-EXTENT-001; no debugger-supplied guest state.
use super::shell_outputs::sentinel;
use super::startup_outputs::{BASE, POP, PUSH, brk, immediate, request};

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, u64, [u64; 4]) {
    let mut sites = vec![];
    for (descriptor, expected_count) in [(1, 1), (3, 3)] {
        let pattern = [
            0xbf, descriptor, 0, 0, 0, 0x48, 0x89, 0xe6, 0xb8, 5, 0, 0, 0, 0x0f, 0x05,
        ];
        let found = image
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(at, bytes)| (bytes == pattern).then_some(at + pattern.len() - 7))
            .collect::<Vec<_>>();
        assert_eq!(found.len(), expected_count, "fstat descriptor {descriptor}");
        sites.extend(found);
    }
    sites.sort_unstable();
    assert_eq!(sites.len(), 4);
    let mut all_assertions = vec![];
    let mut first_completed = 0;
    let mut pages = [0; 4];
    for (index, site) in sites.into_iter().enumerate() {
        let cave = 0x17c000 + index * 0x1000;
        pages[index] = (BASE + cave) as u64;
        let mut code = vec![0x48, 0x81, 0xec, 0x40, 1, 0, 0];
        code.extend_from_slice(PUSH);
        code.extend_from_slice(&[0x49, 0x89, 0xf4]); // original RSI in R12
        let mut assertions = vec![];
        brk(&mut code, &mut assertions, 0, 0x619000);
        code.extend_from_slice(&[0x48, 0x89, 0xc3]);
        brk(&mut code, &mut assertions, 0x63a000, 0x63a000);
        sentinel(&mut code, &mut assertions, 0x639ff8, false);
        for pointer in [0x63a000, 0x401000, 0x639ff8] {
            request(&mut code, &mut assertions, 5, 6, pointer, -14);
        }
        sentinel(&mut code, &mut assertions, 0x639ff8, true);
        for at in [0x638ff7, 0x639088] {
            sentinel(&mut code, &mut assertions, at, false);
        }
        request(&mut code, &mut assertions, 5, 6, 0x638ff8, 0);
        for at in [0x638ff7, 0x639088] {
            sentinel(&mut code, &mut assertions, at, true);
        }
        // Check all eighteen ABI words, including padding/reserved words,
        // against independently declared console/file/directory observations.
        immediate(&mut code, 7, 0x638ff8);
        code.extend_from_slice(&[0x48, 0x8d, 0x35]);
        let expected_displacement = code.len();
        code.extend_from_slice(&[0; 4]);
        immediate(&mut code, 1, 144);
        code.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
        assertions.push(code.len() - 2);
        immediate(&mut code, 6, 0x638ff8);
        code.extend_from_slice(&[0x4c, 0x89, 0xe7]);
        immediate(&mut code, 1, 144);
        code.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
        code.extend_from_slice(&[
            0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
        ]);
        assertions.push(code.len() - 2);
        if index == 0 {
            first_completed = (BASE + cave + code.len()) as u64;
        }
        code.extend_from_slice(POP);
        code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0, 0x31, 0xc0, 0xc3]);
        let displacement = i32::try_from(code.len() - expected_displacement - 4).unwrap();
        code[expected_displacement..expected_displacement + 4]
            .copy_from_slice(&displacement.to_le_bytes());
        let mut words = [0_u64; 18];
        words[1] = (index + 1) as u64;
        words[2] = match index {
            2 => 3,
            3 => 2,
            _ => 1,
        };
        words[3] = match index {
            0 => 0x2180,
            1 => 0x8124,
            _ => 0x41ed,
        };
        if index == 0 {
            words[5] = 0x4400;
        }
        words[7] = 4096;
        for word in words {
            code.extend_from_slice(&word.to_le_bytes());
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
    assert_ne!(first_completed, 0);
    (all_assertions, first_completed, pages)
}
