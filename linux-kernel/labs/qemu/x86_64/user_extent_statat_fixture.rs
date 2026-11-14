//! Ordinary pathname/output requests for selected native newfstatat paths.
//! TOPAL-SYSTEMS-USER-EXTENT-001; all requests originate in the user payload.
use super::shell_outputs::sentinel;
use super::startup_outputs::{BASE, POP, PUSH, brk, compare, immediate, restore_arguments};

fn query(code: &mut Vec<u8>, assertions: &mut Vec<usize>, path: u64, output: u64, result: i32) {
    restore_arguments(code);
    immediate(code, 6, path);
    immediate(code, 2, output);
    code.extend_from_slice(&[0xb8, 6, 1, 0, 0, 0x0f, 0x05]);
    compare(code, assertions, result);
}

pub(super) fn install(image: &mut [u8]) -> (Vec<u64>, u64, [u64; 6]) {
    let signature = [
        0x48, 0x89, 0xe2, 0x45, 0x31, 0xd2, 0xb8, 6, 1, 0, 0, 0x0f, 0x05,
    ];
    let sites = image
        .windows(signature.len())
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == signature).then_some(at + 6))
        .collect::<Vec<_>>();
    assert_eq!(sites.len(), 6);
    let paths: [&[u8]; 6] = [
        b"/etc/busybox.conf\0",
        b"/var/spool/mail/root\0",
        b"/bin\0",
        b"/bin/busybox\0",
        b"/bin/busybox\0",
        b"/bin/busybox\0",
    ];
    let mut pages = [0; 6];
    let mut all_assertions = vec![];
    let mut first_completed = 0;
    for (index, site) in sites.into_iter().enumerate() {
        let cave = 0x170000 + index * 0x1000;
        pages[index] = (BASE + cave) as u64;
        let negative = index < 2;
        let result = if negative { -2 } else { 0 };
        let path = paths[index];
        let mut code = vec![0x48, 0x81, 0xec, 0x40, 1, 0, 0];
        code.extend_from_slice(PUSH);
        code.extend_from_slice(&[0x49, 0x89, 0xd4]); // original output in R12
        let mut assertions = vec![];
        brk(&mut code, &mut assertions, 0, 0x619000);
        code.extend_from_slice(&[0x48, 0x89, 0xc3]);
        brk(&mut code, &mut assertions, 0x63a000, 0x63a000);
        // Seed the original result to expose any accidental missing-query write.
        if negative {
            code.extend_from_slice(&[0x4c, 0x89, 0xe7]);
            immediate(&mut code, 0, 0xa5);
            immediate(&mut code, 1, 144);
            code.extend_from_slice(&[0xfc, 0xf3, 0xaa]);
        }
        // Place the complete pathname beyond page 34 across a page boundary.
        for (offset, byte) in path.iter().enumerate() {
            code.extend_from_slice(&[0xc6, 0x04, 0x25]);
            code.extend_from_slice(&(0x638ffc_u32 + offset as u32).to_le_bytes());
            code.push(*byte);
        }
        code.extend_from_slice(&[0x66, 0xc7, 0x04, 0x25, 0xfe, 0x9f, 0x63, 0]);
        code.extend_from_slice(&path[..2]);
        // Invalid path wins over an independently invalid metadata destination.
        query(&mut code, &mut assertions, 0x63a000, 0x63a000, -14);
        query(&mut code, &mut assertions, 0x639ffe, 0x63a000, -14);
        if negative {
            query(&mut code, &mut assertions, 0x638ffc, 0x63a000, -2);
            code.extend_from_slice(&[0x4c, 0x89, 0xe7]);
            immediate(&mut code, 0, 0xa5);
            immediate(&mut code, 1, 144);
            code.extend_from_slice(&[0xfc, 0xf3, 0xae, 0x75, 0xfe]);
            assertions.push(code.len() - 2);
            check_path(&mut code, &mut assertions, path);
        } else {
            sentinel(&mut code, &mut assertions, 0x639ff8, false);
            for output in [0x63a000, 0x401000, 0x639ff8] {
                query(&mut code, &mut assertions, 0x638ffc, output, -14);
            }
            sentinel(&mut code, &mut assertions, 0x639ff8, true);
            for at in [0x637ff7, 0x638088] {
                sentinel(&mut code, &mut assertions, at, false);
            }
            query(&mut code, &mut assertions, 0x638ffc, 0x637ff8, 0);
            for at in [0x637ff7, 0x638088] {
                sentinel(&mut code, &mut assertions, at, true);
            }
            immediate(&mut code, 7, 0x637ff8);
            code.extend_from_slice(&[0x48, 0x8d, 0x35]);
            let expected_displacement = code.len();
            code.extend_from_slice(&[0; 4]);
            immediate(&mut code, 1, 144);
            code.extend_from_slice(&[0xfc, 0xf3, 0xa6, 0x75, 0xfe]);
            assertions.push(code.len() - 2);
            immediate(&mut code, 6, 0x637ff8);
            code.extend_from_slice(&[0x4c, 0x89, 0xe7]);
            immediate(&mut code, 1, 144);
            code.extend_from_slice(&[0xfc, 0xf3, 0xa4]);
            check_path(&mut code, &mut assertions, path);
            // Append expected bytes after the return, resolving this RIP-relative
            // reference only once the complete executable block is known.
            finish(&mut code, &mut assertions, result);
            let delta = i32::try_from(code.len() - expected_displacement - 4).unwrap();
            code[expected_displacement..expected_displacement + 4]
                .copy_from_slice(&delta.to_le_bytes());
            let mut words = [0_u64; 18];
            words[1] = if index == 2 { 4 } else { 5 };
            words[2] = if index == 2 { 2 } else { 1 };
            words[3] = if index == 2 { 0x41ed } else { 0x816d };
            words[7] = 4096;
            if index != 2 {
                words[6] = 2_024_544;
                words[8] = 3955;
            }
            for word in words {
                code.extend_from_slice(&word.to_le_bytes());
            }
        }
        if negative {
            code.extend_from_slice(&[
                0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
            ]);
            assertions.push(code.len() - 2);
            if index == 0 {
                first_completed = (BASE + cave + code.len()) as u64;
            }
            code.extend_from_slice(POP);
            code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
            immediate(&mut code, 0, (-2_i64) as u64);
            code.push(0xc3);
        }
        assert!(code.len() < 0x1000);
        assert!(image[cave..cave + code.len()].iter().all(|&b| b == 0));
        image[cave..cave + code.len()].copy_from_slice(&code);
        image[site..site + 7].fill(0x90);
        image[site] = 0xe8;
        let delta = i32::try_from(cave as isize - (site + 5) as isize).unwrap();
        image[site + 1..site + 5].copy_from_slice(&delta.to_le_bytes());
        all_assertions.extend(assertions.into_iter().map(|at| (BASE + cave + at) as u64));
    }
    assert_ne!(first_completed, 0);
    (all_assertions, first_completed, pages)
}

fn finish(code: &mut Vec<u8>, assertions: &mut Vec<usize>, result: i32) {
    code.extend_from_slice(&[
        0x48, 0x89, 0xdf, 0xb8, 12, 0, 0, 0, 0x0f, 0x05, 0x48, 0x39, 0xd8, 0x75, 0xfe,
    ]);
    assertions.push(code.len() - 2);
    code.extend_from_slice(POP);
    code.extend_from_slice(&[0x48, 0x81, 0xc4, 0x40, 1, 0, 0]);
    immediate(code, 0, result as i64 as u64);
    code.push(0xc3);
}

fn check_path(code: &mut Vec<u8>, assertions: &mut Vec<usize>, path: &[u8]) {
    for (offset, byte) in path.iter().enumerate() {
        code.extend_from_slice(&[0x80, 0x3c, 0x25]);
        code.extend_from_slice(&(0x638ffc_u32 + offset as u32).to_le_bytes());
        code.extend_from_slice(&[*byte, 0x75, 0xfe]);
        assertions.push(code.len() - 2);
    }
}
