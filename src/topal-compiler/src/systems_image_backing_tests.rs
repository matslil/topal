use super::*;

fn checked() -> crate::systems_frame_policy_binding::Checked {
    crate::systems_frame_policy_binding::check(
        &with_image_preparation(&crate::systems_frame_target::root_tests::root()),
        &crate::systems_frame_target::root_tests::input(),
    )
    .unwrap()
}

fn plan() -> Plan {
    Plan::from_layout(&crate::SystemsKernelImageLayout {
        schema: "topal.kernel-image-layout.x86_64/1".into(),
        kernel_sha256: "synthetic compile-only geometry".into(),
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
    })
    .unwrap()
}

#[test]
fn kernel_image_backing_root_requires_bound_plan_and_retains_exact_preparation_owners() {
    let checked = checked();
    assert!(crate::systems_artifact::generate_frame_user_extent_root(&checked).is_err());
    let plan = plan();
    let (_, owners, bindings) =
        crate::systems_artifact::generate_image_backing_root(&checked, &plan).unwrap();
    bindings.validate(&plan).unwrap();
    assert_eq!(bindings.root_object_sha256, owners.root_object_sha256);
    assert_eq!(bindings.token_bytes, owners.token_bytes);
    assert_eq!(
        bindings
            .banks
            .map(|bank| (bank.count, bank.alignment, bank.pins)),
        [(6, 1, 1), (6, 1, 1)]
    );
    for bank in &bindings.banks {
        assert_ne!(bank.slot, owners.bootstrap.slot);
        assert!(owners.roles.iter().all(|owner| owner.slot != bank.slot));
    }
}

#[test]
fn kernel_image_backing_native_component_closes_and_checks_whole_owners() {
    let checked = checked();
    let plan = plan();
    let (_, _, bindings) =
        crate::systems_artifact::generate_image_backing_root(&checked, &plan).unwrap();
    let text = llvm(&plan, &bindings).unwrap();
    let work = crate::systems_tail_publication::Scratch::new().unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    crate::systems_frame_publication::compile("image-backing", &text, &tools, &work.0).unwrap();
    let record = inspect(&std::fs::read(work.0.join("image-backing.o")).unwrap()).unwrap();
    assert_eq!(record["acyclic"], true);
    assert!(record["combined_boot_stack_bytes"].as_u64().unwrap() <= 8192);
    for case in 0..5 {
        let mut changed = bindings.clone();
        match case {
            0 => changed.banks.swap(0, 1),
            1 => changed.banks[0].count += 1,
            2 => changed.banks[1].slot = changed.banks[0].slot,
            3 => changed.banks[0].pins = 0,
            4 => changed.banks[0].alignment = 2,
            _ => unreachable!(),
        }
        assert!(llvm(&plan, &changed).is_err());
    }
    // Compile-only counterexample; the object is never loaded or executed.
    let imported = text.replace("@topal_x86_frame_gateway_v1", "@unexpected_gateway");
    assert!(
        crate::systems_frame_publication::compile("image-backing", &imported, &tools, &work.0)
            .is_err()
    );
}

#[test]
#[ignore = "requires the authorized bounded QEMU VM and read-only GDB observations"]
fn kernel_image_backing_allocates_and_discards_actual_image_sized_banks() {
    use std::{fs, path::Path, process::Command};
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = repository.join(format!(
        "linux-kernel/.cache/x86_64/image-backing-user-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(work.parent().unwrap()).unwrap();
    fs::create_dir(&work).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let image = crate::systems_user_extent::source_fixture::build(&work, &tools);
    let (seed, mut layout) =
        crate::systems_frame_target::root_tests::prepare_image_backing_fixture(&image, &work, None);
    let mut attempts =
        vec![serde_json::json!({"build": seed, "loaded_during_sizing": false, "seed_only": true})];
    let mut selected = None;
    for _ in 0..4 {
        let plan = Plan::from_layout(&layout).unwrap();
        let (candidate, actual) =
            crate::systems_frame_target::root_tests::prepare_image_backing_fixture(
                &image,
                &work,
                Some(&plan),
            );
        let matched = plan.matches(&actual).unwrap();
        attempts.push(serde_json::json!({"build": candidate, "emitted_pages": plan.pages(), "matched_final_image": matched, "loaded_during_sizing": false}));
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
            fs::write(candidate.join("image-backing-plan-binding.json"), serde_json::to_vec_pretty(&serde_json::json!({
                "kernel_sha256": actual.kernel_sha256, "layout": actual, "bindings": bindings,
                "preparation_sizing_matched": true, "protected_context": false, "production_qualified": false,
                "scope": "Final fresh private source/object/link/image sizing binding for inactive zeroed backing only; no active protected root",
            })).unwrap()).unwrap();
            selected = Some(candidate);
            break;
        }
        layout = actual;
    }
    let guest = selected.expect("image-sized backing did not converge; no candidate was loaded");
    eprintln!(
        "final image-backing preparation candidate: {}",
        guest.display()
    );
    crate::systems_tail_owner::run_tail_native_fixture(
        &guest,
        "image-backing-native",
        Command::new("python3")
            .arg(repository.join("linux-kernel/labs/qemu/x86_64/image_backing_inspect.py"))
            .arg(&guest),
    );
    let observed: serde_json::Value =
        serde_json::from_slice(&fs::read(guest.join("image-backing-observations.json")).unwrap())
            .unwrap();
    assert_eq!(observed["passed"], true);
    assert_eq!(observed["native_execution"], true);
    assert_eq!(observed["protected_context"], false);
}
