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
