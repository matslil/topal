// TOPAL-SYSTEMS-STATIC-ELF-PROVIDER-BACKING-001: model counterexamples
// exercise invalid internal authority states without native state injection.
fn reserve_hierarchy() -> SystemsOperation {
    SystemsOperation::ReserveStaticImageHierarchy {
        failure_message: "hierarchy reservation failed".into(),
    }
}

#[test]
fn provider_selected_hierarchy_model_preserves_attached_owner_until_whole_release() {
    let mut state = BootstrapAuthorityState {
        allocator_created: true,
        memory_ownership: BootstrapMemoryOwnership::StaticElfConstructionSealed,
        ..Default::default()
    };
    for operation in [
        reserve_hierarchy(),
        SystemsOperation::AttachStaticImageAddressSpace,
        SystemsOperation::ActivateStaticImageAddressSpace,
        SystemsOperation::RestoreStaticImageAddressSpace,
        SystemsOperation::AuthorizeStaticImageEntry,
        SystemsOperation::RevokeStaticImageEntry,
        SystemsOperation::DetachStaticImageAddressSpace,
        SystemsOperation::ReclaimStaticImageConstructionHierarchy,
        SystemsOperation::ReleasePhysicalFrames,
    ] {
        assert!(state.observe(2, &operation).unwrap());
    }
    assert!(state.memory_ownership == BootstrapMemoryOwnership::StaticElfConstructionSealed);
    assert!(state.observe(2, &SystemsOperation::ReclaimStaticImageConstructionHierarchy).is_err());
    assert!(state.observe(2, &SystemsOperation::ReleasePhysicalFrames).is_err());
}

#[test]
fn provider_selected_hierarchy_model_refuses_unsealed_duplicate_and_unavailable_contexts() {
    for ownership in [
        BootstrapMemoryOwnership::None,
        BootstrapMemoryOwnership::StaticElfConstructionLive,
        BootstrapMemoryOwnership::StaticElfConstructionZeroed,
        BootstrapMemoryOwnership::StaticElfConstructionPopulated,
        BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy,
        BootstrapMemoryOwnership::StaticElfConstructionAttached,
    ] {
        let mut state = BootstrapAuthorityState {
            allocator_created: true, memory_ownership: ownership, ..Default::default()
        };
        assert_eq!(state.observe(2, &reserve_hierarchy()).unwrap_err().code, "E-SYSTEMS-FRAMES");
        assert!(state.memory_ownership == ownership);
    }
    for translation in [
        BootstrapTranslationState::Update,
        BootstrapTranslationState::InactiveSpace,
        BootstrapTranslationState::MapEdit,
        BootstrapTranslationState::MapStaged,
        BootstrapTranslationState::MappedActive,
        BootstrapTranslationState::UnmapEdit,
        BootstrapTranslationState::UnmapStaged,
    ] {
        let mut state = BootstrapAuthorityState {
            allocator_created: true,
            memory_ownership: BootstrapMemoryOwnership::StaticElfConstructionSealed,
            translation, ..Default::default()
        };
        assert!(state.observe(2, &reserve_hierarchy()).is_err());
        assert!(state.memory_ownership == BootstrapMemoryOwnership::StaticElfConstructionSealed);
    }
    let mut state = BootstrapAuthorityState {
        memory_ownership: BootstrapMemoryOwnership::StaticElfConstructionSealed,
        ..Default::default()
    };
    assert!(state.observe(2, &reserve_hierarchy()).is_err());
}
