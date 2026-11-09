use super::*;

pub(super) fn work(label: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "frame-reconstruction-{}-{nonce}-{label}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    eprintln!("frame reconstruction fixture: {}", path.display());
    path
}

#[test]
fn frame_reconstruction_uses_complete_fresh_source_and_native_roles() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: metadata is an observation,
    // never a replacement for the actual source/provider/caller reconstruction.
    let root = crate::systems_frame_target::root_tests::root();
    let input = crate::systems_frame_target::root_tests::input();
    let tools = LlvmTools::discover(None).unwrap();
    let work = work("fresh");
    let first = rebuild(&root, &(&input).into(), &tools, &work.join("first")).unwrap();
    let second = rebuild(&root, &(&input).into(), &tools, &work.join("second")).unwrap();
    assert_eq!(first.record, second.record);
    assert_eq!(first.objects, second.objects);
    assert!(first.program.frame_allocator_library.is_some());
    assert!(crate::plan_x86_64_systems_provider(&first.program).is_err());
    assert!(crate::generate_x86_64_systems_provider_object(&first.program).is_err());
    assert!(crate::systems_artifact::generate_root_object(&first.program).is_err());
    assert_eq!(first.record.objects.len(), 10);
    assert!(first.record.linked.is_empty());
    assert!(first.record.bounds.combined_with_syscall_caller <= 4096);
    assert_eq!(
        first.record.bounds.retired_scratch
            + first.record.bounds.saved_registers
            + first.record.bounds.return_and_alignment,
        first.record.bounds.caller_reserve
    );
    fs::write(
        work.join("first.binding.json"),
        serde_json::to_vec_pretty(&first.record).unwrap(),
    )
    .unwrap();
    let mut changed = input.clone();
    changed
        .text
        .push_str("\n# different immutable source snapshot\n");
    let third = rebuild(
        &root,
        &(&changed).into(),
        &tools,
        &work.join("changed-source"),
    )
    .unwrap();
    assert_ne!(first.record, third.record);
    assert_ne!(
        first.record.source_obligation_sha256,
        third.record.source_obligation_sha256
    );
    let mut wrong: Source = (&input).into();
    wrong.source = "../memory/frame-metadata.t".into();
    assert!(rebuild(&root, &wrong, &tools, &work.join("bad-path")).is_err());
    assert!(!work.join("bad-path").exists());
    wrong = (&input).into();
    wrong.text = wrong.text.replace("-> Nat\n", "-> Boolean\n");
    assert!(rebuild(&root, &wrong, &tools, &work.join("bad-interface")).is_err());
    assert!(!work.join("bad-interface").exists());
}
