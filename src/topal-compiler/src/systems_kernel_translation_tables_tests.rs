use super::*;
use SystemsKernelImageRole::{Code, Immutable, Mutable};

#[path = "systems_kernel_translation_guest_tests.rs"]
mod guest;

fn layout(spans: Vec<SystemsKernelImageSpan>) -> SystemsKernelImageLayout {
    SystemsKernelImageLayout {
        schema: "topal.kernel-image-layout.x86_64/1".into(),
        kernel_sha256: "fixture".into(),
        minimum_granule: PAGE,
        large_granule: LARGE,
        address_limit: LIMIT,
        spans,
        subdivided_regions: vec![],
        image_leaf_storage_bytes: 0,
    }
}
fn part(start: u64, end: u64, role: SystemsKernelImageRole) -> SystemsKernelImageSpan {
    SystemsKernelImageSpan { start, end, role }
}
fn word(bytes: &[u8], slot: usize, index: usize) -> u64 {
    let at = slot * PAGE as usize + index * 8;
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

// Independent four-level byte walk, including effective ancestor rights. This
// interprets saved bytes only; it does not execute guest code or create owners.
fn walk(bytes: &[u8], base: u64, address: u64) -> Option<(u64, bool, bool, bool)> {
    let mut table = base;
    let (mut write, mut user, mut execute) = (true, true, true);
    for shift in [39, 30, 21, 12] {
        let offset = usize::try_from(table.checked_sub(base)?).ok()?;
        let at = offset.checked_add(((address >> shift) as usize & 511) * 8)?;
        let entry = u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?);
        if entry & 1 == 0 {
            return None;
        }
        write &= entry & 2 != 0;
        user &= entry & 4 != 0;
        execute &= entry & NX == 0;
        if shift == 12 || entry & 128 != 0 {
            let mask = (1_u64 << shift) - 1;
            return Some((
                (entry & 0x000f_ffff_ffff_f000 & !mask) | (address & mask),
                write,
                user,
                execute,
            ));
        }
        table = entry & 0x000f_ffff_ffff_f000;
    }
    None
}

#[test]
fn both_bank_templates_preserve_effective_image_rights_and_exact_storage() {
    let image = layout(vec![
        part(0x200000, 0x201000, Immutable),
        part(0x800000, 0x2ae4000, Mutable),
        part(0x2ae4000, 0x2aec000, Immutable),
        part(0x2aec000, 0x2c22000, Code),
        part(0x2c22000, 0x2c23000, Immutable),
    ]);
    let tables = TranslationTables::new(&image).unwrap();
    assert_eq!(tables.pages(), 8);
    assert_eq!(tables.bytes(), 8 * PAGE);
    for kind in [
        BankKind::Bootstrap,
        BankKind::Process {
            low_user_table: 0x4000000,
        },
    ] {
        let base = 0x5000000;
        let bytes = tables.encode(base, kind).unwrap();
        assert_eq!(bytes.len(), tables.bytes() as usize);
        for span in &image.spans {
            for page in (span.start..span.end).step_by(PAGE as usize) {
                assert_eq!(
                    walk(&bytes, base, page),
                    Some((page, span.role == Mutable, false, span.role == Code))
                );
            }
        }
        assert_eq!(
            walk(&bytes, base, 0x180000),
            Some((0x180000, true, false, false))
        );
        assert_eq!(
            walk(&bytes, base, 0xfec00100),
            Some((0xfec00100, true, false, false))
        );
        assert_eq!(
            walk(&bytes, base, 0xfee00000),
            Some((0xfee00000, true, false, false))
        );
        assert!(walk(&bytes, base, 0xc0000000).is_none());
        assert!(
            bytes[USER_HIGH * PAGE as usize..(USER_HIGH + 1) * PAGE as usize]
                .iter()
                .all(|b| *b == 0)
        );
    }
}

#[test]
fn mutable_only_boundaries_do_not_allocate_unneeded_subdivisions() {
    let tables = TranslationTables::new(&layout(vec![part(0x801000, 0x803000, Mutable)])).unwrap();
    assert_eq!(tables.pages(), FIXED.len() as u64);
    let bytes = tables.encode(0x5000000, BankKind::Bootstrap).unwrap();
    assert!(word(&bytes, LOW_PD, 4) & 128 != 0);
    assert_eq!(
        walk(&bytes, 0x5000000, 0x801000),
        Some((0x801000, true, false, false))
    );
}

#[test]
fn invalid_geometry_and_aliasing_storage_refuse() {
    for spans in [
        vec![],
        vec![part(0x200000, 0x200001, Code)],
        vec![part(0x400000, 0x401000, Code)],
        vec![part(0x200000, LIMIT + PAGE, Mutable)],
        vec![
            part(0x800000, 0x802000, Code),
            part(0x801000, 0x803000, Mutable),
        ],
    ] {
        assert!(TranslationTables::new(&layout(spans)).is_err());
    }
    let tables = TranslationTables::new(&layout(vec![
        part(0x200000, 0x201000, Immutable),
        part(0x1000000, 0x1001000, Code),
    ]))
    .unwrap();
    for base in [0, 0x1000001, 0x1000000, LIMIT - PAGE, u64::MAX] {
        assert!(tables.encode(base, BankKind::Bootstrap).is_err());
    }
    for low in [0, 0x1000000, 0x5000000, 0x5001000, LIMIT] {
        assert!(
            tables
                .encode(
                    0x5000000,
                    BankKind::Process {
                        low_user_table: low
                    }
                )
                .is_err()
        );
    }
}

#[test]
fn larger_whole_owner_does_not_admit_kernel_tables_to_user_walker() {
    let tables =
        TranslationTables::new(&layout(vec![part(0x200000, 0x201000, Immutable)])).unwrap();
    let base = 0x5000000;
    assert_eq!(tables.pages(), 6);
    for slot in 0..tables.pages() {
        for level in 1..=4 {
            let expected = match level {
                4 => slot == 0,
                3 => slot == 1,
                2 => slot == 2 || slot == 3,
                1 => slot == 4,
                _ => false,
            };
            assert_eq!(
                tables.user_table(base, 0x4000000, base + slot * PAGE, level),
                expected
            );
        }
    }
    assert!(tables.user_table(base, 0x4000000, 0x4000000, 1));
    assert!(!tables.user_table(base, 0x4000000, 0x4000000, 2));
    for level in 0..=5 {
        assert!(!tables.user_table(u64::MAX, 0x4000000, u64::MAX, level));
        assert!(!tables.user_table(base, base, base, level));
    }
}

#[test]
fn all_small_image_patterns_match_independent_page_rights() {
    for pattern in 0..256 {
        let mut digits = pattern;
        let mut cells = Vec::new();
        let mut spans = Vec::new();
        for index in 0..4 {
            let role = match digits % 4 {
                0 => None,
                1 => Some(Code),
                2 => Some(Immutable),
                _ => Some(Mutable),
            };
            digits /= 4;
            cells.push(role.unwrap_or(Mutable));
            if let Some(role) = role {
                spans.push(part(
                    0x200000 + index * PAGE,
                    0x200000 + (index + 1) * PAGE,
                    role,
                ));
            }
        }
        if spans.is_empty() {
            assert!(TranslationTables::new(&layout(spans)).is_err());
            continue;
        }
        let plan = TranslationTables::new(&layout(spans)).unwrap();
        assert_eq!(
            plan.pages(),
            5 + u64::from(cells.iter().any(|r| *r != Mutable))
        );
        for kind in [
            BankKind::Bootstrap,
            BankKind::Process {
                low_user_table: 0x4000000,
            },
        ] {
            let bytes = plan.encode(0x5000000, kind).unwrap();
            for (index, role) in cells.iter().enumerate() {
                let address = 0x200000 + index as u64 * PAGE + 5;
                assert_eq!(
                    walk(&bytes, 0x5000000, address),
                    Some((address, *role == Mutable, false, *role == Code)),
                    "pattern {pattern}, page {index}"
                );
            }
        }
    }
}

#[test]
fn native_buffer_preparation_is_closed_and_bounded() {
    // Compiler/static inspection only. No generated host code or VM executes.
    // Caller ownership, actual CPU controls, publication and lifetime remain
    // external preconditions before any kernel may use this data preparation.
    use object::{Object, ObjectSection, ObjectSymbol};
    let plan = TranslationTables::new(&layout(vec![
        part(0x200000, 0x201000, Immutable),
        part(0x800000, 0x802000, Code),
    ]))
    .unwrap();
    let work = crate::systems_tail_publication::Scratch::new().unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let llvm = plan.llvm();
    crate::systems_frame_publication::compile("image-table-prepare", &llvm, &tools, &work.0)
        .unwrap();
    let raw = std::fs::read(work.0.join("image-table-prepare.o")).unwrap();
    let file = object::File::parse(raw.as_slice()).unwrap();
    assert!(!file.symbols().any(|s| s.is_undefined()));
    let symbol = file
        .symbol_by_name("topal_x86_image_table_prepare_data_v1")
        .unwrap();
    assert_eq!(symbol.kind(), object::SymbolKind::Text);
    assert!(symbol.size() > 100);
    let stacks = crate::systems_library::stack_sizes(&file).unwrap();
    assert!(stacks["topal_x86_image_table_prepare_data_v1"] <= 128);
    for section in file.sections() {
        if let object::SectionFlags::Elf { sh_flags, .. } = section.flags() {
            if sh_flags.0 & 2 != 0 && section.size() > 0 {
                assert!([2, 6].contains(&(sh_flags.0 & 7)));
            }
        }
    }
    crate::systems_library::inspect_instructions(
        &std::fs::read_to_string(work.0.join("image-table-prepare-disassembly.stdout.log"))
            .unwrap(),
    )
    .unwrap();
    for (name, kind) in [
        ("bootstrap", BankKind::Bootstrap),
        (
            "process",
            BankKind::Process {
                low_user_table: 0x4000000,
            },
        ),
    ] {
        std::fs::write(
            work.0.join(format!("{name}.tables")),
            plan.encode(0x5000000, kind).unwrap(),
        )
        .unwrap();
    }
}
