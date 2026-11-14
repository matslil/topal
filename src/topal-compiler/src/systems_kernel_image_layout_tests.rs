use super::*;
use SystemsKernelImageRole::{Code, Immutable, Mutable};
use std::fs;
use std::path::Path;

fn span(start: u64, end: u64, role: SystemsKernelImageRole) -> SystemsKernelImageSpan {
    SystemsKernelImageSpan { start, end, role }
}

#[test]
fn minimum_pages_preserve_every_small_role_and_hole_pattern() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001: independent per-page oracle includes
    // holes, repeated roles, unordered input and all three access classes.
    for pattern in 1..256 {
        let mut digits = pattern;
        let mut cells = Vec::new();
        for _ in 0..4 {
            cells.push(match digits % 4 {
                0 => None,
                1 => Some(Code),
                2 => Some(Immutable),
                _ => Some(Mutable),
            });
            digits /= 4;
        }
        let mut regions = cells
            .iter()
            .enumerate()
            .filter_map(|(i, role)| {
                role.map(|role| span(i as u64 * PAGE + 1, (i as u64 + 1) * PAGE - 1, role))
            })
            .collect::<Vec<_>>();
        regions.reverse();
        let spans = partition(regions).unwrap();
        for (i, expected) in cells.iter().enumerate() {
            let actual = spans
                .iter()
                .filter(|r| r.start <= i as u64 * PAGE && (i as u64 + 1) * PAGE <= r.end)
                .map(|r| r.role)
                .collect::<Vec<_>>();
            assert_eq!(
                actual,
                expected.iter().copied().collect::<Vec<_>>(),
                "{pattern}/{i}"
            );
        }
        for pair in spans.windows(2) {
            assert!(pair[0].end <= pair[1].start);
            assert!(pair[0].end != pair[1].start || pair[0].role != pair[1].role);
        }
    }
}

#[test]
fn page_conflicts_refuse_instead_of_combining_permissions() {
    for (left, right) in [(Code, Mutable), (Mutable, Immutable), (Immutable, Code)] {
        assert!(partition(vec![span(0, PAGE - 1, left), span(PAGE - 1, PAGE, right)]).is_err());
        assert!(partition(vec![span(0, 2 * PAGE, left), span(PAGE, 3 * PAGE, right)]).is_err());
        assert_eq!(
            partition(vec![span(0, PAGE, left), span(PAGE, 2 * PAGE, right)])
                .unwrap()
                .len(),
            2
        );
    }
}

#[test]
fn compatible_overlap_is_merged_without_filling_holes() {
    assert_eq!(
        partition(vec![
            span(1, 2 * PAGE, Code),
            span(PAGE, 3 * PAGE - 1, Code),
            span(4 * PAGE, 5 * PAGE, Code)
        ])
        .unwrap(),
        vec![span(0, 3 * PAGE, Code), span(4 * PAGE, 5 * PAGE, Code)]
    );
}

#[test]
fn empty_overflow_and_out_of_domain_images_refuse() {
    assert!(partition(vec![]).is_err());
    for region in [
        span(0, 0, Code),
        span(PAGE, 1, Code),
        span(LIMIT, LIMIT + 1, Code),
        span(0, u64::MAX, Mutable),
    ] {
        assert!(partition(vec![region]).is_err());
    }
    assert_eq!(
        partition(vec![span(LIMIT - 1, LIMIT, Immutable)]).unwrap(),
        vec![span(LIMIT - PAGE, LIMIT, Immutable)]
    );
}

#[test]
fn complete_fresh_image_plan_binds_sources_bytes_and_subdivision_storage() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001 / OWNERS-001: actual compiler inputs
    // and static link, no guest, executable mmap or debugger state injection.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = std::env::temp_dir().join(format!(
        "kernel-image-layout-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&work).unwrap();
    eprintln!("kernel image layout fixture: {}", work.display());
    let root = crate::systems_frame_target::root_tests::root();
    let fixture = work.join("static-elf-abi-fixture.elf");
    fs::write(
        &fixture,
        crate::systems_frame_target::memory_tests::fixture(),
    )
    .unwrap();
    let published = crate::publish_systems_source(
        &root,
        &crate::CompileOptions {
            source_name: "arch/x86_64/toolchain-gate.t".into(),
            output: work.join("artifact"),
            emit: crate::Emit::Executable,
            llvm_tools: None,
            library_root: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
            target: crate::TargetSelection {
                target: Some("x86_64-unknown-none".into()),
                board: Some("topal-qemu-pc-q35-10.2".into()),
                ..Default::default()
            },
            optimization: Default::default(),
            standard_library: None,
            initial_user_image: None,
            static_user_image: Some(fixture),
        },
    )
    .unwrap();
    let kernel = fs::read(&published.kernel).unwrap();
    let binding = published.record.frame_root.as_ref().unwrap();
    let tools = LlvmTools::discover(None).unwrap();
    let plan = plan_x86_64_systems_frame_kernel_image(&kernel, binding, &tools).unwrap();
    assert_eq!(plan.kernel_sha256, crate::artifact::sha256(&kernel));
    assert_eq!(plan.minimum_granule, PAGE);
    assert_eq!(plan.large_granule, BLOCK);
    assert_eq!(plan.address_limit, LIMIT);
    assert_eq!(
        plan.image_leaf_storage_bytes,
        plan.subdivided_regions.len() as u64 * PAGE
    );
    assert!(plan.spans.contains(&span(0x200000, 0x201000, Immutable)));
    assert!(plan.subdivided_regions.contains(&0x200000));
    assert!(plan.spans.iter().any(|s| s.role == Code));
    assert!(plan.spans.iter().any(|s| s.role == Mutable));
    // Check every provider block against a separate page-cell description.
    for block in (0..LIMIT).step_by(BLOCK as usize) {
        let cells = (block..block + BLOCK)
            .step_by(PAGE as usize)
            .map(|page| {
                plan.spans
                    .iter()
                    .find(|s| s.start <= page && page < s.end)
                    .map(|s| s.role)
            })
            .collect::<Vec<_>>();
        let any = cells.iter().any(Option::is_some);
        let uniform = cells[0].is_some() && cells.iter().all(|r| *r == cells[0]);
        assert_eq!(plan.subdivided_regions.contains(&block), any && !uniform);
    }
    fs::write(
        work.join("image-layout.json"),
        serde_json::to_vec_pretty(&plan).unwrap(),
    )
    .unwrap();
    assert_eq!(fs::read(&published.kernel).unwrap(), kernel);

    let mut changed = binding.clone();
    changed.llvm_version.push_str("-stale");
    assert!(plan_x86_64_systems_frame_kernel_image(&kernel, &changed, &tools).is_err());
    changed = binding.clone();
    changed.other_sources.clear();
    assert!(plan_x86_64_systems_frame_kernel_image(&kernel, &changed, &tools).is_err());
    changed = binding.clone();
    changed.linked[0].address += PAGE;
    assert!(plan_x86_64_systems_frame_kernel_image(&kernel, &changed, &tools).is_err());
    let mut changed_kernel = kernel.clone();
    let file = object::File::parse(kernel.as_slice()).unwrap();
    let section = file
        .sections()
        .find(|s| s.kind() == object::SectionKind::Text && s.size() > 0)
        .unwrap();
    let offset = section.file_range().unwrap().0 as usize;
    changed_kernel[offset] ^= 1;
    assert!(plan_x86_64_systems_frame_kernel_image(&changed_kernel, binding, &tools).is_err());

    // An otherwise matching ELF cannot acquire a new linker-owned GOT slot
    // merely by declaring it. This is static malformed-artifact validation;
    // the altered bytes are never loaded or executed.
    let got = file.section_by_name(".got").unwrap();
    assert_eq!(got.size(), 0);
    let table = u64::from_le_bytes(kernel[40..48].try_into().unwrap()) as usize;
    let stride = u16::from_le_bytes(kernel[58..60].try_into().unwrap()) as usize;
    let at = table + got.index().0 * stride + 32;
    let mut changed_kernel = kernel.clone();
    changed_kernel[at..at + 8].copy_from_slice(&8_u64.to_le_bytes());
    let error = plan_x86_64_systems_frame_kernel_image(&changed_kernel, binding, &tools)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unowned allocated kernel section .got"),
        "{error}"
    );
}
