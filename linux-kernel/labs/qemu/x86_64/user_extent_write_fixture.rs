//! Ordinary direct writes from a grown heap. TOPAL-SYSTEMS-USER-EXTENT-001.
use super::shell_outputs::sentinel;
use super::startup_outputs::{BASE, POP, PUSH, brk, immediate, request};

pub(super) const LENGTHS: [u8; 20] = [
    1, 4, 40, 1, 109, 4, 6, 4, 4, 5, 13, 13, 19, 17, 13, 13, 14, 4, 2, 4,
];

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, [u64; LENGTHS.len()]) {
    let sites = image[0x0a6100..0x0a9000]
        .windows(17)
        .enumerate()
        .filter_map(|(offset, bytes)| {
            let count = bytes[1];
            (bytes
                == [
                    0xba, count, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0x0f, 0x05, 0x48, 0x83, 0xf8, count,
                    0x75,
                ])
            .then_some((0x0a6100 + offset + 5, count))
        })
        .collect::<Vec<_>>();
    assert!(sites.iter().map(|&(_, length)| length).eq(LENGTHS));
    let mut pages = [0; LENGTHS.len()];
    let mut all_assertions = vec![];
    for (index, (site, count)) in sites.into_iter().enumerate() {
        let cave = 0x150000 + index * 0x1000;
        pages[index] = (BASE + cave) as u64;
        let start = 0x638fff_u32;
        let end = start + u32::from(count);
        let mut code = vec![0x48, 0x81, 0xec, 0x40, 1, 0, 0];
        code.extend_from_slice(PUSH);
        code.extend_from_slice(&[0x49, 0x89, 0xf4]); // original input in R12
        let mut assertions = vec![];
        brk(&mut code, &mut assertions, 0, 0x619000);
        code.extend_from_slice(&[0x48, 0x89, 0xc3]); // old break in RBX
        brk(&mut code, &mut assertions, 0x63a000, 0x63a000);

        // Copy the fixture's actual data to the last byte of a heap page.
        code.extend_from_slice(&[0x4c, 0x89, 0xe6]); // mov RSI,R12
        immediate(&mut code, 7, u64::from(start));
        immediate(&mut code, 1, u64::from(count));
        code.extend_from_slice(&[0xfc, 0xf3, 0xa4]); // cld; rep movsb
        for at in [start - 1, end] {
            sentinel(&mut code, &mut assertions, at, false);
        }

        request(&mut code, &mut assertions, 1, 6, 0x63a000, -14);
        if count > 1 {
            sentinel(&mut code, &mut assertions, 0x639fff, false);
            request(&mut code, &mut assertions, 1, 6, 0x639fff, -14);
            sentinel(&mut code, &mut assertions, 0x639fff, true);
        }
        request(
            &mut code,
            &mut assertions,
            1,
            6,
            u64::from(start),
            i32::from(count),
        );
        for at in [start - 1, end] {
            sentinel(&mut code, &mut assertions, at, true);
        }
        code.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        immediate(&mut code, 7, u64::from(start));
        immediate(&mut code, 1, u64::from(count));
        code.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]); // compare input bytes
        assertions.push(code.len() - 2);

        code.extend_from_slice(&[
            0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
        ]);
        assertions.push(code.len() - 2);
        code.extend_from_slice(POP);
        code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
        immediate(&mut code, 0, u64::from(count));
        code.push(0xc3);

        assert!(code.len() < 0x1000);
        assert!(image[cave..cave + code.len()].iter().all(|&byte| byte == 0));
        image[cave..cave + code.len()].copy_from_slice(&code);
        image[site..site + 7].fill(0x90);
        image[site] = 0xe8;
        let displacement = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
        image[site + 1..site + 5].copy_from_slice(&displacement.to_le_bytes());
        all_assertions.extend(assertions.into_iter().map(|at| (BASE + cave + at) as u64));
    }
    (all_assertions, pages)
}
