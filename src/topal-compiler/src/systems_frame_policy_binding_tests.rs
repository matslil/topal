use super::*;
use std::{fs, path::Path};

const ROOT: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/pre-frame-kernel-root.t");
const DECLARATION: &str = "  frame-allocator-library is lang systems source-library (\n    source is \"memory/frame-metadata.t\",\n    export is frame-policy,\n    interface is physical-frame-policy-tail-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";

fn root() -> String {
    let selected = ROOT.replacen(
        "lang systems artifact (\n",
        &format!("lang systems artifact (\n{DECLARATION}"),
        1,
    );
    assert_eq!(
        selected,
        include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t")
    );
    selected
}
fn input() -> SystemsSourceLibraryInput {
    SystemsSourceLibraryInput {
        source: "memory/frame-metadata.t".into(),
        source_name: "memory/frame-metadata.t".into(),
        text: include_str!("../../../linux-kernel/kernel/memory/frame-metadata.t").into(),
        modules: vec![],
    }
}

fn save(checked: &Checked, label: &str) {
    let parent = std::env::temp_dir().join(format!("frame-source-{}-{label}", std::process::id()));
    fs::create_dir(&parent).unwrap();
    fs::write(
        parent.join("source-obligation.json"),
        serde_json::to_vec_pretty(&checked.source).unwrap(),
    )
    .unwrap();
    fs::write(
        parent.join("component.json"),
        serde_json::to_vec_pretty(&checked.component).unwrap(),
    )
    .unwrap();
    fs::write(parent.join("controller.ll"), &checked.controller).unwrap();
    fs::write(
        parent.join("bootstrap-target.ll"),
        &checked.bootstrap_target,
    )
    .unwrap();
    fs::write(
        parent.join("native-refusal.txt"),
        checked.native_binding_required().to_string(),
    )
    .unwrap();
    eprintln!("frame source fixture: {}", parent.display());
}

#[test]
fn frame_policy_binding_retains_exact_root_source_and_component_without_publication() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001.
    let root = root();
    let checked = check(&root, &input()).unwrap();
    save(&checked, "original");
    assert_eq!(checked.source.root_source, root);
    assert_eq!(
        checked.source.root_sha256,
        crate::artifact::sha256(root.as_bytes())
    );
    assert_eq!(checked.source.library.text, input().text);
    assert_eq!(
        checked.source.schema,
        "topal.frame-policy-source-obligation/3"
    );
    assert_eq!(
        checked.component.schema,
        "topal.tail-nat-owner-controller.x86_64/1"
    );
    assert_eq!(
        checked
            .component
            .runtime
            .source_step
            .storage
            .source_plan
            .functions
            .len(),
        33
    );
    assert_eq!(
        checked.source.component_sha256,
        crate::artifact::sha256(&serde_json::to_vec(&checked.component).unwrap())
    );
    let again = check(&checked.source.root_source, &input()).unwrap();
    assert_eq!(
        serde_json::to_vec(&checked.source).unwrap(),
        serde_json::to_vec(&again.source).unwrap()
    );
    assert!(
        checked
            .native_binding_required()
            .to_string()
            .contains("ownership/zeroing/caller")
    );
    assert!(
        crate::plan_x86_64_systems_provider(&checked.program)
            .unwrap_err()
            .to_string()
            .contains("E-SYSTEMS-FRAME-POLICY-BINDING")
    );
    assert!(crate::generate_x86_64_systems_provider_object(&checked.program).is_err());
    let tools = crate::LlvmTools::discover(None).unwrap();
    let parent = std::env::temp_dir().join(format!("frame-gate-{}", std::process::id()));
    fs::create_dir(&parent).unwrap();
    let destination = parent.join("unqualified");
    let result = crate::publish_x86_64_systems_artifact(&checked.program, &tools, &destination);
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("E-SYSTEMS-FRAME-POLICY-BINDING")
    );
    assert!(!destination.exists());
    let loaded = crate::systems_library::load_source_library(
        checked.program.frame_allocator_library.as_ref().unwrap(),
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
    )
    .unwrap();
    assert_eq!(
        check(&root, &loaded).unwrap().source.library.text,
        input().text
    );
    let options = crate::CompileOptions {
        source_name: "frame-root.t".into(),
        output: parent.join("pipeline-unqualified"),
        emit: crate::Emit::Executable,
        llvm_tools: Some(parent.join("unavailable-tools")),
        library_root: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
        target: crate::TargetSelection {
            target: Some("x86_64-unknown-none".into()),
            board: Some("topal-qemu-pc-q35-10.2".into()),
            ..Default::default()
        },
        optimization: Default::default(),
        standard_library: None,
        initial_user_image: None,
        static_user_image: None,
    };
    let error = crate::publish_systems_source(&root, &options)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("LLVM") || error.contains("unavailable-tools"),
        "{error}"
    );
    assert!(!options.output.exists());
}

#[test]
fn frame_policy_binding_refuses_unbound_source_and_wrong_result_or_progress() {
    assert!(check(ROOT, &input()).is_err());
    let root = root();
    for fault in 0..8 {
        let mut source = input();
        match fault {
            0 => source.source = "memory/other.t".into(),
            1 => source.source_name.push('\0'),
            2 => source.text = source.text.replace("pub frame-policy", "frame-policy"),
            3 => {
                source.text = source.text.replace(
                    "-> Nat : Decreases remaining",
                    "-> Boolean : Decreases remaining",
                )
            }
            4 => {
                source.text = source
                    .text
                    .replace(": Decreases remaining", ": Decreases status")
            }
            5 => {
                source.text = source.text.replace(
                    "otherwise frame-policy (region, remaining - 1",
                    "otherwise frame-policy (region, remaining",
                )
            }
            6 => {
                source.text = source.text.replace(
                    "status : Nat)) -> Nat : Decreases",
                    "status : Boolean)) -> Nat : Decreases",
                )
            }
            _ => source.modules.push(crate::SystemsSourceModuleInput {
                identity: vec!["extra".into()],
                source: "extra.t".into(),
                text: "use language (version is v0.1, features is (systems))\npub extra is fn () -> Nat\n  1\n".into(),
            }),
        }
        assert!(check(&root, &source).is_err(), "fault {fault}");
    }
}

#[test]
fn frame_policy_binding_rechecks_changed_valid_root_and_library_independently() {
    let original = check(&root(), &input()).unwrap();
    let changed_root = root().replacen(
        "TOPAL_KERNEL_FRAME_ALLOCATED",
        "TOPAL_KERNEL_FRAME_ALLOCATED_VARIANT",
        1,
    );
    let changed = check(&changed_root, &input()).unwrap();
    save(&changed, "root-change");
    assert_ne!(original.source.root_sha256, changed.source.root_sha256);
    assert_eq!(
        original.source.component_sha256,
        changed.source.component_sha256
    );
    let mut source = input();
    source.text = source.text.replace("count > 0", "count > 1");
    assert_ne!(source.text, input().text);
    let changed = check(&root(), &source).unwrap();
    save(&changed, "policy-change");
    assert_eq!(original.source.root_sha256, changed.source.root_sha256);
    assert_ne!(
        original.source.component_sha256,
        changed.source.component_sha256
    );
    assert_ne!(original.source.library.text, changed.source.library.text);
    assert!(
        changed
            .native_binding_required()
            .to_string()
            .contains("E-SYSTEMS-FRAME-POLICY-BINDING")
    );
}
