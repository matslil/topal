#[path = "../../../linux-kernel/tools/user-memory/image_backing_source.rs"]
mod image_backing_source;

#[test]
fn kernel_image_tables_source_preserves_its_distinct_affine_resource() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-TABLES-001.
    let text = image_backing_source::with_image_tables(SOURCE);
    let program = analyze_systems_for_compiler(&text, &CompilerSystemsTargetSelection::initial_x86_64_qemu()).unwrap();
    let transitions = model_systems_transitions(&program).unwrap();
    assert_eq!(transitions.iter().filter(|t| matches!(t, CompilerSystemsTransition::PrepareKernelImageTables)).count(), 1);
    assert_eq!(transitions.iter().filter(|t| matches!(t, CompilerSystemsTransition::DiscardKernelImageTables)).count(), 1);
    assert!(!transitions.iter().any(|t| matches!(t, CompilerSystemsTransition::ReserveKernelImageBacking)));
    for (old, new) in [
        ("discard tables image-backing", "discard backing image-backing"),
        ("prepare tables", "reserve tables"),
        ("TOPAL_KERNEL_IMAGE_TABLES_PREPARED", "TOPAL_KERNEL_IMAGE_BACKING_RESERVED"),
        ("page-policy is provider-selected", "page-policy is 8"),
        ("discard tables image-backing", "discard tables unrelated"),
    ] {
        assert!(text.contains(old));
        assert!(analyze_systems_for_compiler(&text.replacen(old,new,1), &CompilerSystemsTargetSelection::initial_x86_64_qemu()).is_err());
    }
}

#[test]
fn kernel_image_backing_source_preserves_reservation_discard_and_continuation() {
    // TOPAL-SYSTEMS-KERNEL-IMAGE-PREPARE-001.
    let text = image_backing_source::with_image_preparation(SOURCE);
    let program = analyze_systems_for_compiler(&text, &CompilerSystemsTargetSelection::initial_x86_64_qemu()).unwrap();
    let transitions = model_systems_transitions(&program).unwrap();
    assert_eq!(transitions.iter().filter(|t| matches!(t, CompilerSystemsTransition::ReserveKernelImageBacking)).count(), 1);
    assert_eq!(transitions.iter().filter(|t| matches!(t, CompilerSystemsTransition::DiscardKernelImageBacking)).count(), 1);
    assert!(transitions.iter().any(|t| matches!(t, CompilerSystemsTransition::BeginTranslationUpdate { .. })));
    assert!(!program.bootstrap.handler.operations.iter().any(|op| matches!(op,
        CompilerSystemsOperation::AllocatePhysicalFrames { request, .. } if request.frame_count == 8)));
}

#[test]
fn kernel_image_backing_source_rejects_substitution_geometry_and_interleaving() {
    let text = image_backing_source::with_image_preparation(SOURCE);
    for (old, new) in [
        ("image is current-kernel", "image is user-elf"),
        ("page-policy is provider-selected", "page-policy is 8"),
        ("image is current-kernel,", "image is current-kernel, frame-count is 8,"),
        ("image is current-kernel,", "image is current-kernel, image is current-kernel,"),
        ("discard backing image-backing", "discard backing other-backing"),
        ("memory kernel image discard backing image-backing", "other kernel image discard backing image-backing"),
        ("memory kernel image discard backing image-backing", "memory frames release image-backing"),
        ("memory kernel image discard backing image-backing", "memory console write \"interleave\"\n                          memory kernel image discard backing image-backing"),
        ("TOPAL_KERNEL_IMAGE_BACKING_RESERVED", "TOPAL_KERNEL_IMAGE_BACKING_BAD"),
        ("TOPAL_KERNEL_IMAGE_BACKING_DISCARDED", "TOPAL_KERNEL_IMAGE_BACKING_BAD"),
    ] {
        assert!(text.contains(old));
        assert!(analyze_systems_for_compiler(&text.replacen(old, new, 1), &CompilerSystemsTargetSelection::initial_x86_64_qemu()).is_err(), "{old}");
    }
}
