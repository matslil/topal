use super::*;

#[test]
fn retained_tables_require_scoped_source_and_close_native_lifetime_component() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-RETAIN-001.
    let checked = crate::systems_frame_policy_binding::check(
        &with_retained_image_tables(&crate::systems_frame_target::root_tests::root()),
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
    let plan = Plan::for_retained_tables(&layout).unwrap();
    let (_, owners, binding) =
        crate::systems_artifact::generate_image_backing_root(&checked, &plan).unwrap();
    owners.validate().unwrap();
    binding.validate(&plan).unwrap();
    assert!(
        binding
            .banks
            .iter()
            .all(|b| b.slot != owners.bootstrap.slot)
    );
    assert!(
        binding
            .banks
            .iter()
            .all(|b| owners.roles.iter().all(|o| o.slot != b.slot))
    );
    assert!(
        crate::systems_artifact::generate_image_backing_root(
            &checked,
            &Plan::for_tables(&layout).unwrap()
        )
        .is_err()
    );
    let ordinary = crate::systems_frame_policy_binding::check(
        &with_image_tables(&crate::systems_frame_target::root_tests::root()),
        &crate::systems_frame_target::root_tests::input(),
    )
    .unwrap();
    assert!(crate::systems_artifact::generate_image_backing_root(&ordinary, &plan).is_err());
    let tools = crate::LlvmTools::discover(None).unwrap();
    let work = crate::systems_tail_publication::Scratch::new().unwrap();
    let text = tables_native::llvm(&plan, &binding).unwrap();
    crate::systems_frame_publication::compile("retained-image-tables", &text, &tools, &work.0)
        .unwrap();
    let record = inspect_selected(
        &std::fs::read(work.0.join("retained-image-tables.o")).unwrap(),
        true,
    )
    .unwrap();
    assert_eq!(record["acyclic"], true);
    assert!(record["combined_boot_stack_bytes"].as_u64().unwrap() <= 8192);
    for name in [TABLE_RETAIN, RETAINED_DISCARD, RETAINED_RELEASED] {
        assert!(record["frames"].get(name).is_some());
    }
}

#[test]
#[ignore = "requires the bounded authorized QEMU VM and read-only GDB observations"]
fn retained_tables_cross_ordinary_exec_and_restore_before_whole_retirement() {
    use std::{fs, path::Path, process::Command};
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = repo.join(format!(
        "linux-kernel/.cache/x86_64/retained-tables-user-{}-{nonce}",
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
        let plan = Plan::for_retained_tables(&actual).unwrap();
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
                "kernel_sha256":layout.kernel_sha256,"layout":layout,"bindings":bindings,"retained":true,
                "matched_final_partition_and_template":true,"protected_context":false,"production_qualified":false,
            })).unwrap()).unwrap();
            selected = Some(candidate);
            break;
        }
        actual = layout;
    }
    let guest = selected.expect("retained table template did not converge; no candidate loaded");
    eprintln!("final retained image tables candidate: {}", guest.display());
    crate::systems_tail_owner::run_tail_native_fixture(
        &guest,
        "image-tables-native",
        Command::new("python3")
            .arg(repo.join("linux-kernel/labs/qemu/x86_64/retained_image_tables_inspect.py"))
            .arg(&guest),
    );
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(guest.join("image-tables-observations.json")).unwrap())
            .unwrap();
    assert_eq!(record["passed"], true);
    assert!(record["retention_events"].as_u64().unwrap() >= 4);
    assert_eq!(record["successful_execs"], 3);
    assert_eq!(record["protected_context"], false);
}
