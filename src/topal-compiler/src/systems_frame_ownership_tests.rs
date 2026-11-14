//! TOPAL-SYSTEMS-KERNEL-IMAGE-OWNERS-001 / PLAN-001.
use super::*;

fn owners() -> crate::systems_user_extent::Owners {
    let checked = crate::systems_frame_policy_binding::check(
        &crate::systems_frame_target::root_tests::root(),
        &crate::systems_frame_target::root_tests::input(),
    )
    .unwrap();
    let (bound, owners) =
        crate::systems_artifact::generate_frame_user_extent_root(&checked).unwrap();
    assert_eq!(
        bound,
        crate::systems_artifact::generate_frame_root_checked(&checked).unwrap()
    );
    owners
}

#[test]
fn whole_root_allocation_purpose_requires_the_consuming_construction_transition() {
    for (count, purpose) in [
        (1, Purpose::StaticImageLeaf),
        (503, Purpose::StaticImagePayload),
    ] {
        let mut allocation = Allocation {
            count,
            alignment: 1,
            purpose: Purpose::SourceExtent,
        };
        assert_eq!(allocation.purpose, Purpose::SourceExtent);
        allocation.refine_construction(purpose).unwrap();
        assert_eq!(allocation.purpose, purpose);
        assert!(allocation.refine_construction(purpose).is_err());
    }
    for mut allocation in [
        Allocation {
            count: 503,
            alignment: 2,
            purpose: Purpose::SourceExtent,
        },
        Allocation {
            count: 5,
            alignment: 1,
            purpose: Purpose::SourceExtent,
        },
        Allocation {
            count: 503,
            alignment: 1,
            purpose: Purpose::BootstrapHierarchy,
        },
    ] {
        let before = allocation;
        assert!(
            allocation
                .refine_construction(Purpose::StaticImagePayload)
                .is_err()
        );
        assert_eq!(allocation, before);
    }
    let mut allocation = Allocation {
        count: 5,
        alignment: 1,
        purpose: Purpose::SourceExtent,
    };
    assert!(
        allocation
            .refine_construction(Purpose::BootstrapHierarchy)
            .is_err()
    );
}

#[test]
fn whole_root_owners_keep_distinct_bootstrap_and_process_purposes() {
    let owners = owners();
    owners.validate().unwrap();
    assert_eq!(owners.bootstrap.purpose, Purpose::BootstrapHierarchy);
    assert_eq!(owners.bootstrap.count, 5);
    assert_eq!(owners.bootstrap.alignment, 1);
    assert_eq!(owners.bootstrap.pins, 1);
    assert_eq!(
        owners.roles.map(|owner| owner.purpose),
        [
            Purpose::StaticImageHierarchy,
            Purpose::StaticImageLeaf,
            Purpose::StaticImagePayload,
            Purpose::StaticImageStack,
        ]
    );
    assert!(
        owners
            .roles
            .iter()
            .all(|owner| owner.slot != owners.bootstrap.slot)
    );
    assert!(owners.roles.iter().all(|owner| owner.alignment == 1));
}

#[test]
fn whole_root_owners_refuse_equal_sized_cross_purpose_substitution_and_aliases() {
    let owners = owners();
    for case in 0..8 {
        let mut changed = owners.clone();
        match case {
            0 => std::mem::swap(&mut changed.bootstrap, &mut changed.roles[0]),
            1 => changed.bootstrap.slot = changed.roles[0].slot,
            2 => changed.bootstrap.count = 8,
            3 => changed.bootstrap.alignment = 2,
            4 => changed.bootstrap.pins = 0,
            5 => changed.bootstrap.slot = u64::MAX,
            6 => changed.roles[0].purpose = Purpose::BootstrapHierarchy,
            7 => changed.roles[1].purpose = Purpose::SourceExtent,
            _ => unreachable!(),
        }
        assert!(changed.validate().is_err(), "counterexample {case}");
    }
}

#[test]
fn whole_root_owner_requirements_follow_image_geometry_without_granting_authority() {
    let owners = owners();
    for (role, required) in [
        (crate::SystemsKernelImageRole::Mutable, 5),
        (crate::SystemsKernelImageRole::Immutable, 6),
    ] {
        let layout = crate::SystemsKernelImageLayout {
            schema: "topal.kernel-image-layout.x86_64/1".into(),
            kernel_sha256: "synthetic geometry-only test".into(),
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
        };
        let record =
            crate::systems_kernel_image_layout::backing_requirements(&layout, &owners).unwrap();
        assert_eq!(record["banks"].as_array().unwrap().len(), 2);
        for bank in record["banks"].as_array().unwrap() {
            assert_eq!(bank["current_owner"]["count"], 5);
            assert_eq!(bank["required_frame_count"], required);
            assert_eq!(bank["whole_extent_size_matches"], required == 5);
        }
        assert_eq!(record["process_low_leaf"]["purpose"], "static-image-leaf");
        for flag in [
            "runtime_ownership_qualified",
            "protected_context",
            "production_qualified",
        ] {
            assert_eq!(record[flag], false);
        }
        let mut wrong = owners.clone();
        wrong.bootstrap.count = required + 1;
        assert!(crate::systems_kernel_image_layout::backing_requirements(&layout, &wrong).is_err());
    }
}
