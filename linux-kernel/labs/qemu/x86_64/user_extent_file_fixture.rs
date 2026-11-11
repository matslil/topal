//! Ordinary pathname, directory record and EOF requests.
//! TOPAL-SYSTEMS-USER-EXTENT-001.
use super::shell_outputs::sentinel;
use super::startup_outputs::{BASE, POP, PUSH, brk, immediate, request};

#[derive(Clone, Copy)]
enum Case {
    Path(u8, i32),
    Records(&'static [u64], u32),
    Eof(u32),
}

const ROOT: &[u64] = &[
    3,
    1,
    0x2e04_0018,
    3,
    2,
    0x002e_2e04_0018,
    4,
    3,
    0x6e69_6204_0018,
];
const BIN: &[u64] = &[
    4,
    1,
    0x2e04_0018,
    3,
    2,
    0x002e_2e04_0018,
    5,
    3,
    0x6279_7375_6208_0020,
    0x786f,
];

fn calls(image: &[u8]) -> Vec<(usize, Case)> {
    let mut sites = vec![];
    for (number, cases) in [
        (
            257_u32,
            &[
                Case::Path(9, -2),
                Case::Path(13, 3),
                Case::Path(2, 3),
                Case::Path(6, 3),
                Case::Path(13, 3),
                Case::Path(13, 3),
            ][..],
        ),
        (
            217,
            &[
                Case::Records(ROOT, 0x638ff8),
                Case::Eof(217),
                Case::Records(BIN, 0x639f80),
                Case::Eof(217),
            ][..],
        ),
    ] {
        let mut pattern = vec![0xb8];
        pattern.extend_from_slice(&number.to_le_bytes());
        pattern.extend_from_slice(&[0x0f, 0x05]);
        let found = image[0x0a7000..0x0a9000]
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(at, bytes)| (bytes == pattern).then_some(0x0a7000 + at))
            .collect::<Vec<_>>();
        assert_eq!(found.len(), cases.len(), "native call {number}");
        sites.extend(found.into_iter().zip(cases.iter().copied()));
    }
    let profile_eof = [0xba, 128, 0, 0, 0, 0x31, 0xc0, 0x0f, 0x05, 0x48, 0x85, 0xc0];
    let found = image[0x0a7000..0x0a9000]
        .windows(profile_eof.len())
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == profile_eof).then_some(0x0a7000 + at))
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 1, "profile EOF read");
    sites.push((found[0], Case::Eof(0)));
    sites.sort_unstable_by_key(|&(offset, _)| offset);
    sites
}

fn copy(code: &mut Vec<u8>, heap: u32, count: u8, from_original: bool) {
    if from_original {
        code.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        immediate(code, 7, u64::from(heap));
    } else {
        immediate(code, 6, u64::from(heap));
        code.extend_from_slice(&[0x4c, 0x89, 0xe7]);
    }
    immediate(code, 1, u64::from(count));
    code.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
}

#[derive(serde::Serialize)]
pub(crate) struct Observation {
    page: u64,
    number: u32,
    count: u8,
    result: i32,
}

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, Vec<Observation>) {
    let mut all_assertions = vec![];
    let mut metadata = vec![];
    for (index, (site, case)) in calls(image).into_iter().enumerate() {
        let cave = 0x140000 + index * 0x1000;
        let page = (BASE + cave) as u64;
        let mut code = vec![];
        if matches!(case, Case::Eof(0)) {
            code.extend_from_slice(&[0xba, 128, 0, 0, 0]);
        }
        code.extend_from_slice(&[0x48, 0x81, 0xec, 0x40, 1, 0, 0]);
        code.extend_from_slice(PUSH);
        code.extend_from_slice(&[0x49, 0x89, 0xf4]);
        let mut assertions = vec![];
        if !matches!(case, Case::Eof(_)) {
            brk(&mut code, &mut assertions, 0, 0x619000);
            code.extend_from_slice(&[0x48, 0x89, 0xc3]);
            brk(&mut code, &mut assertions, 0x63a000, 0x63a000);
        }
        let (number, count, result) = match case {
            Case::Path(count, result) => {
                let start = 0x638fff;
                copy(&mut code, start, count, true);
                request(&mut code, &mut assertions, 257, 6, 0x63a000, -14);
                sentinel(&mut code, &mut assertions, 0x639fff, false);
                request(&mut code, &mut assertions, 257, 6, 0x639fff, -14);
                sentinel(&mut code, &mut assertions, 0x639fff, true);
                request(&mut code, &mut assertions, 257, 6, u64::from(start), result);
                code.extend_from_slice(&[0x4c, 0x89, 0xe6]);
                immediate(&mut code, 7, u64::from(start));
                immediate(&mut code, 1, u64::from(count));
                code.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
                assertions.push(code.len() - 2);
                (257, count, result)
            }
            Case::Records(expected, start) => {
                let count = u8::try_from(8 * expected.len()).unwrap();
                for at in [0x63a000, 0x401000] {
                    request(&mut code, &mut assertions, 217, 6, at, -14);
                }
                for at in 0x639ff8..0x63a000 {
                    sentinel(&mut code, &mut assertions, at, false);
                }
                request(&mut code, &mut assertions, 217, 6, 0x639ff8, -14);
                for at in 0x639ff8..0x63a000 {
                    sentinel(&mut code, &mut assertions, at, true);
                }
                for at in [start - 1, start + u32::from(count)] {
                    sentinel(&mut code, &mut assertions, at, false);
                }
                request(
                    &mut code,
                    &mut assertions,
                    217,
                    6,
                    u64::from(start),
                    i32::from(count),
                );
                for at in [start - 1, start + u32::from(count)] {
                    sentinel(&mut code, &mut assertions, at, true);
                }
                for (index, value) in expected.iter().enumerate() {
                    immediate(&mut code, 0, *value);
                    code.extend_from_slice(&[0x48, 0x39, 0x04, 0x25]);
                    code.extend_from_slice(&(start + 8 * index as u32).to_le_bytes());
                    code.extend_from_slice(&[0x75, 0xfe]);
                    assertions.push(code.len() - 2);
                }
                copy(&mut code, start, count, false);
                (217, count, i32::from(count))
            }
            Case::Eof(number) => {
                // Keep a byte witness from the original caller destination.
                code.extend_from_slice(&[0x4d, 0x8b, 0x2c, 0x24]); // mov r13,[r12]
                if number == 0 {
                    // This requested range ends one byte beyond USER_PTR_MAX.
                    request(
                        &mut code,
                        &mut assertions,
                        number,
                        6,
                        (1_u64 << 47) - 4096 - 127,
                        -14,
                    );
                }
                request(&mut code, &mut assertions, number, 6, 0x63a000, 0);
                code.extend_from_slice(&[0x4d, 0x39, 0x2c, 0x24, 0x75, 0xfe]);
                assertions.push(code.len() - 2);
                (number, 0, 0)
            }
        };
        if !matches!(case, Case::Eof(_)) {
            code.extend_from_slice(&[
                0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
            ]);
            assertions.push(code.len() - 2);
        }
        code.extend_from_slice(POP);
        code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
        immediate(&mut code, 0, result as i64 as u64);
        code.push(0xc3);
        let replaced = if matches!(case, Case::Eof(0)) { 9 } else { 7 };
        assert!(code.len() < 0x1000);
        assert!(image[cave..cave + code.len()].iter().all(|&byte| byte == 0));
        image[cave..cave + code.len()].copy_from_slice(&code);
        image[site..site + replaced].fill(0x90);
        image[site] = 0xe8;
        let displacement = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
        image[site + 1..site + 5].copy_from_slice(&displacement.to_le_bytes());
        all_assertions.extend(assertions.into_iter().map(|at| page + at as u64));
        metadata.push(Observation {
            page,
            number,
            count,
            result,
        });
    }
    (all_assertions, metadata)
}
