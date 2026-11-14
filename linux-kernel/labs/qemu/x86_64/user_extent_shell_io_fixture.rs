//! Ordinary chdir, signal, terminal query and header read requests.
//! TOPAL-SYSTEMS-USER-EXTENT-001.
use super::shell_outputs::sentinel;
use super::startup_outputs::{BASE, POP, PUSH, brk, immediate, request};

#[derive(Clone, Copy)]
enum Transfer {
    Input(u8),
    Output(&'static [u8]),
    Unused,
}

#[derive(Clone, Copy)]
struct Case {
    number: u32,
    argument: u8,
    result: i32,
    transfer: Transfer,
    replaced: usize,
}

#[derive(serde::Serialize)]
pub(crate) struct Observation {
    page: u64,
    number: u32,
    argument: u8,
    result: i32,
    direction: Option<u8>,
    count: u8,
}

fn sites(image: &[u8]) -> Vec<(usize, Case)> {
    const DEFAULT_ACTION: &[u8] = &[0; 32];
    let input_action = Case {
        number: 13,
        argument: 6,
        result: 0,
        transfer: Transfer::Input(32),
        replaced: 7,
    };
    let default_action = Case {
        argument: 2,
        transfer: Transfer::Output(DEFAULT_ACTION),
        ..input_action
    };
    let tcgets = Case {
        number: 16,
        argument: 2,
        result: -25,
        transfer: Transfer::Unused,
        replaced: 7,
    };
    let chdir = Case {
        number: 80,
        argument: 7,
        result: 0,
        transfer: Transfer::Input(2),
        replaced: 7,
    };
    let header = |byte| Case {
        number: 0,
        argument: 6,
        result: 1,
        transfer: Transfer::Output(byte),
        replaced: 9,
    };
    let mut result = vec![];
    for (pattern, cases) in [
        (
            &[0xb8, 13, 0, 0, 0, 0x0f, 0x05][..],
            &[
                input_action,
                default_action,
                default_action,
                input_action,
                default_action,
            ][..],
        ),
        (&[0xb8, 16, 0, 0, 0, 0x0f, 0x05][..], &[tcgets; 5][..]),
        (&[0xb8, 80, 0, 0, 0, 0x0f, 0x05][..], &[chdir][..]),
        (
            &[0xba, 1, 0, 0, 0, 0x31, 0xc0, 0x0f, 0x05][..],
            &[header(b"\x7f"), header(b"E"), header(b"L"), header(b"F")][..],
        ),
    ] {
        let offsets = image[0x0a6100..0x0a9000]
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == pattern).then_some(0x0a6100 + offset))
            .collect::<Vec<_>>();
        assert_eq!(offsets.len(), cases.len());
        result.extend(offsets.into_iter().zip(cases.iter().copied()));
    }
    result.sort_unstable_by_key(|&(offset, _)| offset);
    result
}

fn verify_bytes(code: &mut Vec<u8>, assertions: &mut Vec<usize>, at: u32, bytes: &[u8]) {
    for (offset, byte) in bytes.iter().enumerate() {
        code.extend_from_slice(&[0x80, 0x3c, 0x25]);
        code.extend_from_slice(&(at + offset as u32).to_le_bytes());
        code.extend_from_slice(&[*byte, 0x75, 0xfe]);
        assertions.push(code.len() - 2);
    }
}

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, Vec<Observation>) {
    let mut all_assertions = vec![];
    let mut metadata = vec![];
    for (index, (site, case)) in sites(image).into_iter().enumerate() {
        let cave = 0x130000 + index * 0x1000;
        let page = (BASE + cave) as u64;
        let mut code = vec![];
        if case.replaced == 9 {
            code.extend_from_slice(&[0xba, 1, 0, 0, 0]);
        }
        code.extend_from_slice(&[0x48, 0x81, 0xec, 0x40, 1, 0, 0]);
        code.extend_from_slice(PUSH);
        // R12 retains the original syscall buffer. These arguments use low registers.
        code.extend_from_slice(&[0x49, 0x89, 0xc4 | (case.argument << 3)]);
        let mut assertions = vec![];
        if !matches!(case.transfer, Transfer::Unused) {
            brk(&mut code, &mut assertions, 0, 0x619000);
            code.extend_from_slice(&[0x48, 0x89, 0xc3]);
            brk(&mut code, &mut assertions, 0x63a000, 0x63a000);
        }
        let (direction, count) = match case.transfer {
            Transfer::Input(count) => {
                let start = 0x638fff;
                code.extend_from_slice(&[0x4c, 0x89, 0xe6]);
                immediate(&mut code, 7, start);
                immediate(&mut code, 1, u64::from(count));
                code.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
                request(
                    &mut code,
                    &mut assertions,
                    case.number,
                    case.argument,
                    0x63a000,
                    -14,
                );
                sentinel(&mut code, &mut assertions, 0x639fff, false);
                request(
                    &mut code,
                    &mut assertions,
                    case.number,
                    case.argument,
                    0x639fff,
                    -14,
                );
                sentinel(&mut code, &mut assertions, 0x639fff, true);
                request(
                    &mut code,
                    &mut assertions,
                    case.number,
                    case.argument,
                    start,
                    case.result,
                );
                code.extend_from_slice(&[0x4c, 0x89, 0xe6]);
                immediate(&mut code, 7, start);
                immediate(&mut code, 1, u64::from(count));
                code.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
                assertions.push(code.len() - 2);
                (Some(0), count)
            }
            Transfer::Output(bytes) => {
                let count = u8::try_from(bytes.len()).unwrap();
                let start = 0x638fff_u32;
                let end = start + u32::from(count);
                for pointer in [0x63a000, 0x401000] {
                    request(
                        &mut code,
                        &mut assertions,
                        case.number,
                        case.argument,
                        pointer,
                        -14,
                    );
                }
                if count > 1 {
                    sentinel(&mut code, &mut assertions, 0x639fff, false);
                    request(
                        &mut code,
                        &mut assertions,
                        case.number,
                        case.argument,
                        0x639fff,
                        -14,
                    );
                    sentinel(&mut code, &mut assertions, 0x639fff, true);
                }
                for pointer in [start - 1, end] {
                    sentinel(&mut code, &mut assertions, pointer, false);
                }
                request(
                    &mut code,
                    &mut assertions,
                    case.number,
                    case.argument,
                    u64::from(start),
                    case.result,
                );
                for pointer in [start - 1, end] {
                    sentinel(&mut code, &mut assertions, pointer, true);
                }
                verify_bytes(&mut code, &mut assertions, start, bytes);
                immediate(&mut code, 6, u64::from(start));
                code.extend_from_slice(&[0x4c, 0x89, 0xe7]);
                immediate(&mut code, 1, u64::from(count));
                code.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
                (Some(1), count)
            }
            Transfer::Unused => {
                code.extend_from_slice(&[0x4d, 0x8b, 0x2c, 0x24]);
                request(
                    &mut code,
                    &mut assertions,
                    case.number,
                    case.argument,
                    0x63a000,
                    case.result,
                );
                code.extend_from_slice(&[0x4d, 0x39, 0x2c, 0x24, 0x75, 0xfe]);
                assertions.push(code.len() - 2);
                (None, 0)
            }
        };
        if direction.is_some() {
            code.extend_from_slice(&[
                0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
            ]);
            assertions.push(code.len() - 2);
        }
        code.extend_from_slice(POP);
        code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
        immediate(&mut code, 0, case.result as i64 as u64);
        code.push(0xc3);
        assert!(code.len() < 0x1000);
        assert!(image[cave..cave + code.len()].iter().all(|&byte| byte == 0));
        image[cave..cave + code.len()].copy_from_slice(&code);
        image[site..site + case.replaced].fill(0x90);
        image[site] = 0xe8;
        let delta = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
        image[site + 1..site + 5].copy_from_slice(&delta.to_le_bytes());
        all_assertions.extend(assertions.into_iter().map(|at| page + at as u64));
        metadata.push(Observation {
            page,
            number: case.number,
            argument: case.argument,
            result: case.result,
            direction,
            count,
        });
    }
    (all_assertions, metadata)
}
