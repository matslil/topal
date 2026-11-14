use super::*;

#[test]
fn startup_storage_component_closes_and_preserves_the_normal_profile() {
    // TOPAL-SYSTEMS-BOOT-STARTUP-STORAGE-001.
    use crate::systems_image_backing as backing;
    let checked = crate::systems_frame_policy_binding::check(
        &backing::with_retained_image_tables(&crate::systems_frame_target::root_tests::root()),
        &crate::systems_frame_target::root_tests::input(),
    )
    .unwrap();
    let layout = crate::SystemsKernelImageLayout {
        schema: "topal.kernel-image-layout.x86_64/1".into(),
        kernel_sha256: "compile-only".into(),
        minimum_granule: 4096,
        large_granule: 2097152,
        address_limit: 1073741824,
        spans: vec![crate::SystemsKernelImageSpan {
            start: 0x200000,
            end: 0x201000,
            role: crate::SystemsKernelImageRole::Immutable,
        }],
        subdivided_regions: vec![],
        image_leaf_storage_bytes: 0,
    };
    let plan = backing::Plan::for_startup_storage(&layout).unwrap();
    let (_, _, binding) =
        crate::systems_artifact::generate_image_backing_root(&checked, &plan).unwrap();
    let text = backing::llvm(&plan, &binding).unwrap();
    assert!(!text.contains("{{"));
    assert_eq!(text.matches("call void @s.startup.claim(").count(), 1);
    assert_eq!(text.matches("call void @s.startup.verify(").count(), 1);
    let normal = backing::llvm(
        &backing::Plan::for_retained_tables(&layout).unwrap(),
        &binding,
    )
    .unwrap();
    assert!(!normal.contains("@s.startup"));
    let tools = crate::LlvmTools::discover(None).unwrap();
    let work = crate::systems_tail_publication::Scratch::new().unwrap();
    crate::systems_frame_publication::compile("startup-storage", &text, &tools, &work.0).unwrap();
    let record = backing::inspect_selected(
        &std::fs::read(work.0.join("startup-storage.o")).unwrap(),
        true,
    )
    .unwrap();
    assert!(record["combined_boot_stack_bytes"].as_u64().unwrap() <= 8192);
    assert!(record["frames"].get("s.startup.claim").is_some());
    // Exercise the publication section validator before paying for a VM build.
    crate::systems_tail_linked::prepare(&tools, &work.0.join("startup-storage.o"), "image-tables")
        .unwrap();
}

#[test]
fn startup_storage_pattern_covers_one_whole_distinct_setup_page() {
    let bytes = page();
    assert_eq!(bytes.len(), 4096);
    assert_eq!(&bytes[..8], b"TOPAL-ST");
    for (i, word) in bytes.chunks_exact(8).enumerate() {
        assert_eq!(
            u64::from_le_bytes(word.try_into().unwrap()),
            SEED.wrapping_add((i as u64).wrapping_mul(STRIDE))
        );
    }
    assert_eq!(contract()["processor_startup"], false);
}

#[test]
#[ignore = "requires the bounded authorized VM and read-only GDB observations"]
fn startup_storage_is_loaded_claimed_probed_restored_and_retained_in_the_vm() {
    use crate::systems_image_backing::Plan;
    use std::{fs, path::Path, process::Command};
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = repo.join(format!(
        "linux-kernel/.cache/x86_64/startup-storage-user-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&work).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let image = crate::systems_user_extent::source_fixture::build(&work, &tools);
    let (seed, mut actual) =
        crate::systems_frame_target::root_tests::prepare_image_backing_fixture(&image, &work, None);
    let mut attempts =
        vec![serde_json::json!({"build":seed,"seed_only":true,"loaded_during_sizing":false})];
    let mut selected = None;
    for _ in 0..6 {
        let plan = Plan::for_startup_storage(&actual).unwrap();
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
                "retained":true,"startup_storage":true,"matched_final_partition_and_template":true,
                "protected_context":false,"production_qualified":false,
            })).unwrap()).unwrap();
            selected = Some(candidate);
            break;
        }
        actual = layout;
    }
    let guest =
        selected.expect("startup-storage final image did not converge; no candidate loaded");
    eprintln!("final startup storage candidate: {}", guest.display());
    crate::systems_tail_owner::run_tail_native_fixture(
        &guest,
        "image-tables-native",
        Command::new("python3")
            .arg(repo.join("linux-kernel/labs/qemu/x86_64/boot_startup_storage_inspect.py"))
            .arg(&guest),
    );
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(guest.join("image-tables-observations.json")).unwrap())
            .unwrap();
    assert_eq!(record["passed"], true);
    assert_eq!(record["boot_startup_storage"]["final_owner_retained"], true);
    assert_eq!(record["boot_startup_storage"]["processor_startup"], false);
    assert_eq!(record["successful_execs"], 3);
}
