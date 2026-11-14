#[test]
fn kernel_image_tables_model_rejects_backing_substitution_and_interleaving() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-TABLES-001; invalid internal model states.
    let mut state = BootstrapAuthorityState { memory_described: true, allocator_created: true, ..Default::default() };
    let prepare = SystemsOperation::PrepareKernelImageTables { failure_message: "failed".into() };
    state.observe(2, &prepare).unwrap();
    assert!(state.observe(3, &SystemsOperation::DiscardKernelImageTables).is_err());
    assert!(state.observe(3, &SystemsOperation::ConsoleWrite { text: "TOPAL_KERNEL_IMAGE_BACKING_RESERVED".into() }).is_err());
    state.observe(3, &SystemsOperation::ConsoleWrite { text: "TOPAL_KERNEL_IMAGE_TABLES_PREPARED".into() }).unwrap();
    assert!(state.observe(4, &SystemsOperation::DiscardKernelImageBacking).is_err());
    assert!(state.observe(4, &prepare).is_err());
    state.observe(4, &SystemsOperation::DiscardKernelImageTables).unwrap();
    assert!(state.observe(5, &prepare).is_err());
    state.observe(5, &SystemsOperation::ConsoleWrite { text: "TOPAL_KERNEL_IMAGE_TABLES_DISCARDED".into() }).unwrap();
    assert!(state.observe(6, &SystemsOperation::DiscardKernelImageTables).is_err());
    let mut live = BootstrapAuthorityState { memory_described: true, allocator_created: true, ..Default::default() };
    live.observe(2,&prepare).unwrap();
    assert_eq!(live.complete().unwrap_err().code,"E-SYSTEMS-FRAMES-LIVE");
}

#[test]
fn kernel_image_backing_model_requires_single_affine_discard_without_other_work() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-PREPARE-001; model-only invalid states.
    let reserve = SystemsOperation::ReserveKernelImageBacking { failure_message: "failed".into() };
    let mut state = BootstrapAuthorityState { memory_described: true, allocator_created: true, ..Default::default() };
    assert!(state.observe(2, &SystemsOperation::DiscardKernelImageBacking).is_err());
    state.observe(2, &reserve).unwrap();
    assert!(state.observe(3, &reserve).is_err());
    assert!(state.observe(3, &SystemsOperation::AllocatePhysicalFrames { request: INITIAL_SINGLE_FRAME_REQUEST, failure_message: "failed".into() }).is_err());
    assert!(state.observe(3, &SystemsOperation::DiscardKernelImageBacking).is_err());
    state.observe(3, &SystemsOperation::ConsoleWrite { text: "TOPAL_KERNEL_IMAGE_BACKING_RESERVED".into() }).unwrap();
    state.observe(3, &SystemsOperation::DiscardKernelImageBacking).unwrap();
    assert!(state.observe(4, &reserve).is_err());
    state.observe(4, &SystemsOperation::ConsoleWrite { text: "TOPAL_KERNEL_IMAGE_BACKING_DISCARDED".into() }).unwrap();
    assert!(state.observe(4, &reserve).is_err());
    assert!(state.observe(4, &SystemsOperation::DiscardKernelImageBacking).is_err());
    let mut state = BootstrapAuthorityState { memory_described: true, allocator_created: true, ..Default::default() };
    state.observe(2, &reserve).unwrap();
    assert_eq!(state.complete().unwrap_err().code, "E-SYSTEMS-FRAMES-LIVE");
}
#[test]
fn retained_tables_need_exact_observations_and_whole_terminal_retirement() {
    let mut state=BootstrapAuthorityState::default();
    state.allocator_created=true;
    state.observe(1,&SystemsOperation::PrepareKernelImageTables{failure_message:"failed".into()}).unwrap();
    assert!(state.observe(2,&SystemsOperation::RetainKernelImageTables).is_err());
    state.observe(2,&SystemsOperation::ConsoleWrite{text:"TOPAL_KERNEL_IMAGE_TABLES_PREPARED".into()}).unwrap();
    state.observe(3,&SystemsOperation::RetainKernelImageTables).unwrap();
    assert!(state.observe(4,&SystemsOperation::ConsoleWrite{text:"ordinary".into()}).is_err());
    state.observe(4,&SystemsOperation::ConsoleWrite{text:"TOPAL_KERNEL_IMAGE_TABLES_RETAINED".into()}).unwrap();
    assert!(state.observe(5,&SystemsOperation::DiscardKernelImageTables).is_err());
    assert!(state.observe(5,&SystemsOperation::RetainKernelImageTables).is_err());
    assert!(state.observe(5,&SystemsOperation::DiscardRetainedKernelImageTables).is_err());
    state.translation=BootstrapTranslationState::RemovedActive;
    state.observe(5,&SystemsOperation::DiscardRetainedKernelImageTables).unwrap();
    assert!(state.observe(6,&SystemsOperation::ConsoleWrite{text:"ordinary".into()}).is_err());
    state.observe(6,&SystemsOperation::ConsoleWrite{text:"TOPAL_KERNEL_IMAGE_RETAINED_TABLES_DISCARDED".into()}).unwrap();
    assert!(state.observe(7,&SystemsOperation::ConsoleWrite{text:"ordinary".into()}).is_err());
}

#[test]
fn retained_tables_cannot_escape_completion_or_repeat_preparation() {
    let mut state=BootstrapAuthorityState::default();
    state.allocator_created=true;
    state.image_tables_retained=true;
    state.image_retention_observed=true;
    assert!(state.observe(0,&SystemsOperation::PrepareKernelImageTables{failure_message:"failed".into()}).is_err());
    assert!(state.complete().is_err());
}
