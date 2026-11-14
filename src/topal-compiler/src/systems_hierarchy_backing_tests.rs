//! TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001,
//! TOPAL-SYSTEMS-STATIC-ELF-PROVIDER-BACKING-001.
use topal_language::compiler::{CompilerSystemsOperation as O, model_systems_transitions};

pub(super) fn selected_root() -> String {
    with_provider_backing(&super::root())
}

fn with_provider_backing(root: &str) -> String {
    let start = root.find("frame-count is 5,").unwrap();
    let last = "alignment-frames is 1";
    let end = start + root[start..].find(last).unwrap() + last.len();
    root.replacen(&root[start..end], "backing is provider-selected", 1)
}

#[test]
fn provider_selected_hierarchy_keeps_exact_native_owners_and_legacy_root_bytes() {
    let input = super::input();
    let legacy = crate::systems_frame_policy_binding::check(&super::root(), &input).unwrap();
    let selected = crate::systems_frame_policy_binding::check(&selected_root(), &input).unwrap();
    let (old_bytes, old_owners) =
        crate::systems_artifact::generate_frame_user_extent_root(&legacy).unwrap();
    let (new_bytes, new_owners) =
        crate::systems_artifact::generate_frame_user_extent_root(&selected).unwrap();
    assert_eq!(new_bytes, old_bytes);
    assert_eq!(new_owners, old_owners);
    assert_eq!(new_owners.roles[0].count, 5);
    assert_eq!(new_owners.roles[0].pins, 1);
    // Frame-backed native publication retains its complete binding gate.
    assert!(crate::plan_x86_64_systems_provider(&selected.program).is_err());
    let target = topal_language::compiler::CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let pre_frame = topal_language::compiler::analyze_systems_for_compiler(
        &with_provider_backing(super::ROOT),
        &target,
    )
    .unwrap();
    let plan = crate::plan_x86_64_systems_provider(&pre_frame).unwrap();
    let reservation = plan
        .operations
        .iter()
        .filter(|op| {
            op.semantic_identity == "topal.systems.static-image.construction.reserve-hierarchy/1"
        })
        .collect::<Vec<_>>();
    assert_eq!(reservation.len(), 1);
    assert_eq!(
        reservation[0].lowering,
        crate::X86SystemsLowering::ReserveStaticImageHierarchy
    );
    assert_eq!(
        reservation[0].lowering.provider_identity(),
        "topal.provider.x86_64.static-image.reserve-bootstrap-equivalent-hierarchy/1"
    );
    let trace = crate::systems_artifact::semantic_trace(&selected.program).unwrap();
    assert!(trace.iter().any(|line|
        line == "topal.systems.static-image.construction.reserve-hierarchy/1:backing=provider-selected"));
    assert!(
        !trace
            .iter()
            .any(|line| line.contains(":frames=5:alignment=1"))
    );
    assert!(
        crate::systems_artifact::semantic_trace(&legacy.program)
            .unwrap()
            .iter()
            .any(|line| line.contains(":frames=5:alignment=1"))
    );
}

#[test]
fn provider_selected_hierarchy_rejects_ir_lifetime_violations_before_native_lowering() {
    let checked =
        crate::systems_frame_policy_binding::check(&selected_root(), &super::input()).unwrap();
    let reserve = checked
        .program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|op| matches!(op, O::ReserveStaticImageHierarchy { .. }))
        .unwrap();
    let seal = checked
        .program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|op| matches!(op, O::SealStaticImageConstructionPermissions))
        .unwrap();
    let reclaim = checked
        .program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|op| matches!(op, O::ReclaimStaticImageConstructionHierarchy))
        .unwrap();
    for case in 0..4 {
        let mut changed =
            crate::systems_frame_policy_binding::check(&selected_root(), &super::input()).unwrap();
        let ops = &mut changed.program.bootstrap.handler.operations;
        match case {
            0 => ops.insert(reserve + 1, ops[reserve].clone()),
            1 => ops.swap(seal, reserve),
            2 => {
                ops.remove(reclaim);
            }
            3 => ops.insert(reserve + 1, O::RevokeStaticImageConstructionMetadata),
            _ => unreachable!(),
        }
        let mut effects = ops
            .iter()
            .map(|op| op.semantic_identity().to_owned())
            .collect::<Vec<_>>();
        effects.push(
            changed
                .program
                .bootstrap
                .handler
                .disposition
                .semantic_identity(topal_language::compiler::CompilerSystemsContextKind::Bootstrap)
                .into(),
        );
        effects.sort();
        effects.dedup();
        changed.program.bootstrap.handler.effects = effects;
        assert!(
            model_systems_transitions(&changed.program).is_err(),
            "case {case}"
        );
        assert!(
            crate::systems_artifact::generate_frame_user_extent_root(&changed).is_err(),
            "case {case}"
        );
    }
}
