use super::*;

#[test]
fn image_tables_boot_observation_sites_refuse_writes_and_unqualified_reads() {
    let assembly = "0 <k.image.active_root>:\n 0: movq %cr3, %rax\n 3: retq\n\
0 <k.boot.cr0>:\n 0: movq %cr0, %rax\n 3: retq\n\
0 <k.boot.cr4>:\n 0: movq %cr4, %rax\n 3: retq\n\
0 <k.boot.cs>:\n 0: movw %cs, %ax\n 3: retq\n\
0 <k.boot.cpuid>:\n 0: cpuid\n 2: retq\n\
0 <k.boot.msr>:\n 0: rdmsr\n 2: retq\n\
0 <k.boot.descriptors>:\n 0: sgdtq (%rax)\n 3: sidtq (%rax)\n 6: retq\n";
    inspect_instructions_selected(assembly, true).unwrap();
    inspect_instructions_selected(&(assembly.to_owned() + " 10: nopw %cs:(%rax,%rax)\n"), true)
        .unwrap();
    assert!(inspect_instructions_selected(assembly, false).is_err());
    for (old, new) in [
        ("movq %cr0, %rax", "movq %rax, %cr0"),
        ("rdmsr", "wrmsr"),
        ("k.boot.msr", "unqualified.reader"),
        ("sgdtq", "lgdtq"),
        ("0: rdmsr", "0: rdmsr\n 2: rdmsr"),
    ] {
        assert!(inspect_instructions_selected(&assembly.replace(old, new), true).is_err());
    }
}

fn layout(role: crate::SystemsKernelImageRole) -> crate::SystemsKernelImageLayout {
    crate::SystemsKernelImageLayout {
        schema: "topal.kernel-image-layout.x86_64/1".into(),
        kernel_sha256: "compile-only geometry".into(),
        minimum_granule: 4096,
        large_granule: 2097152,
        address_limit: 1073741824,
        spans: vec![crate::SystemsKernelImageSpan {
            start: 0x200000,
            end: 0x201000,
            role,
        }],
        subdivided_regions: vec![],
        image_leaf_storage_bytes: 0,
    }
}

fn checked() -> crate::systems_frame_policy_binding::Checked {
    crate::systems_frame_policy_binding::check(
        &with_image_tables(&crate::systems_frame_target::root_tests::root()),
        &crate::systems_frame_target::root_tests::input(),
    )
    .unwrap()
}

#[test]
fn image_tables_require_exact_partition_and_a_distinct_whole_low_leaf() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-TABLES-001.
    let selected = layout(crate::SystemsKernelImageRole::Immutable);
    let changed = layout(crate::SystemsKernelImageRole::Code);
    let plan = Plan::for_tables(&selected).unwrap();
    assert!(plan.matches(&selected).unwrap());
    assert!(!plan.matches(&changed).unwrap());
    assert!(
        Plan::from_layout(&selected)
            .unwrap()
            .matches(&changed)
            .unwrap()
    );
    let checked = checked();
    assert!(crate::systems_artifact::generate_frame_user_extent_root(&checked).is_err());
    assert!(
        crate::systems_artifact::generate_image_backing_root(
            &checked,
            &Plan::from_layout(&selected).unwrap()
        )
        .is_err()
    );
    let (_, owners, binding) =
        crate::systems_artifact::generate_image_backing_root(&checked, &plan).unwrap();
    binding.validate(&plan).unwrap();
    let low = binding.low_user_leaf.unwrap();
    assert_eq!(
        (low.count, low.alignment, low.pins, low.purpose),
        (
            1,
            1,
            1,
            crate::systems_frame_ownership::Purpose::PreparedImageLowUserLeaf
        )
    );
    assert!(binding.banks.iter().all(|b| b.slot != low.slot));
    assert!(owners.roles.iter().all(|b| b.slot != low.slot));
    let bytes = plan
        .table_plan()
        .unwrap()
        .encode(0x3000000, Some(0x3100000))
        .unwrap();
    assert_eq!(bytes.len() as u64, plan.pages() * 4096);
    assert_eq!(
        u64::from_le_bytes(bytes[(1026 * 8)..(1027 * 8)].try_into().unwrap()),
        0x3100007
    );
}

#[test]
fn image_tables_native_component_closes_and_refuses_incomplete_low_leaf_bindings() {
    let plan = Plan::for_tables(&layout(crate::SystemsKernelImageRole::Immutable)).unwrap();
    let (_, _, binding) =
        crate::systems_artifact::generate_image_backing_root(&checked(), &plan).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let work = crate::systems_tail_publication::Scratch::new().unwrap();
    let text = llvm(&plan, &binding).unwrap();
    crate::systems_frame_publication::compile("image-tables", &text, &tools, &work.0).unwrap();
    let inspection =
        inspect_selected(&std::fs::read(work.0.join("image-tables.o")).unwrap(), true).unwrap();
    assert_eq!(inspection["acyclic"], true);
    assert!(inspection["combined_boot_stack_bytes"].as_u64().unwrap() <= 8192);
    for case in 0..5 {
        let mut changed = binding.clone();
        match case {
            0 => changed.low_user_leaf = None,
            1 => changed.low_user_leaf.as_mut().unwrap().count = 2,
            2 => changed.low_user_leaf.as_mut().unwrap().pins = 0,
            3 => changed.low_user_leaf.as_mut().unwrap().slot = changed.banks[0].slot,
            4 => changed.low_user_leaf.as_mut().unwrap().purpose = changed.banks[0].purpose,
            _ => unreachable!(),
        }
        assert!(llvm(&plan, &changed).is_err());
    }
}

#[test]
#[ignore = "requires the bounded authorized QEMU VM and read-only GDB observations"]
fn image_tables_construct_and_discard_both_final_image_banks_with_separate_low_leaf() {
    use std::{fs, path::Path, process::Command};
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = repo.join(format!(
        "linux-kernel/.cache/x86_64/image-tables-user-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(work.parent().unwrap()).unwrap();
    fs::create_dir(&work).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let image = crate::systems_user_extent::source_fixture::build(&work, &tools);
    let (seed, mut actual) =
        crate::systems_frame_target::root_tests::prepare_image_backing_fixture(&image, &work, None);
    let mut attempts =
        vec![serde_json::json!({"build":seed,"seed_only":true,"loaded_during_sizing":false})];
    let mut selected = None;
    for _ in 0..6 {
        let plan = Plan::for_tables(&actual).unwrap();
        let (candidate, layout) =
            crate::systems_frame_target::root_tests::prepare_image_backing_fixture(
                &image,
                &work,
                Some(&plan),
            );
        let matched = plan.matches(&layout).unwrap();
        attempts.push(serde_json::json!({"build":candidate,"matched_final_partition_and_template":matched,"pages":plan.pages(),"loaded_during_sizing":false}));
        fs::write(
            work.join("sizing-attempts.json"),
            serde_json::to_vec_pretty(&attempts).unwrap(),
        )
        .unwrap();
        if matched {
            let bindings: serde_json::Value = serde_json::from_slice(
                &fs::read(candidate.join("image-backing-bindings.json")).unwrap(),
            )
            .unwrap();
            fs::write(candidate.join("image-tables-plan-binding.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "kernel_sha256":layout.kernel_sha256,"layout":layout,"bindings":bindings,
                "matched_final_partition_and_template":true,"protected_context":false,"production_qualified":false,
            })).unwrap()).unwrap();
            selected = Some(candidate);
            break;
        }
        actual = layout;
    }
    let guest = selected.expect("final image/template did not converge; no candidate loaded");
    eprintln!("final inactive image tables candidate: {}", guest.display());
    crate::systems_tail_owner::run_tail_native_fixture(
        &guest,
        "image-tables-native",
        Command::new("python3")
            .arg(repo.join("linux-kernel/labs/qemu/x86_64/image_tables_inspect.py"))
            .arg(&guest),
    );
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(guest.join("image-tables-observations.json")).unwrap())
            .unwrap();
    assert_eq!(result["passed"], true);
    assert_eq!(result["native_execution"], true);
    assert_eq!(result["protected_context"], false);
}
