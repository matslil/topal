use super::*;
use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object, ObjectSegment};
use std::{fs, path::Path, process::Command};

#[test]
fn bootstrap_frame_target_constructs_the_real_map_and_zeroes_owned_extents() {
    execute(false);
}

#[test]
fn frame_gateway_retires_actual_bootstrap_caller_storage_and_preserves_lifetimes() {
    execute(true);
}

#[test]
fn frame_gateway_refuses_unproved_and_overflowing_native_stack_budgets() {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: includes the machine gateway,
    // alignment, return word and saved registers within the original reserve.
    let generate = crate::systems_provider_object::tail_boundary::frame_gateway::component_object;
    for invalid in [0, 7, 4017, 4024, 4096, u64::MAX] {
        assert!(generate(invalid).is_err());
    }
    assert!(generate(4016).is_ok());
}

fn execute(gateway: bool) {
    // TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001: unpublished CPL0 component only.
    let source = include_str!("../../../linux-kernel/kernel/memory/frame-metadata.t");
    let body = topal_language::compiler::analyze_kernel_region_tail_nat_source(
        source,
        &[],
        "frame-policy",
        3,
    )
    .unwrap();
    let entry = &body
        .functions
        .iter()
        .find(|f| f.source_name == "frame-policy")
        .unwrap()
        .symbol;
    let owner =
        crate::emit_systems_tail_nat_owner_component(&body, "memory/frame-metadata.t", entry)
            .unwrap();
    assert_eq!(
        owner.runtime.source_step.storage.source_frames_per_step,
        "53"
    );
    assert_eq!(
        owner
            .runtime
            .source_step
            .storage
            .source_region_loans_per_step,
        "38"
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let prefix = if gateway {
        "frame-gateway"
    } else {
        "frame-target"
    };
    let work = std::env::temp_dir().join(format!("{prefix}-{}-{nonce}", std::process::id()));
    fs::create_dir(&work).unwrap();
    eprintln!("unpublished native frame target: {}", work.display());
    fs::write(work.join("source.t"), source).unwrap();
    fs::write(
        work.join("component.json"),
        serde_json::to_vec_pretty(&owner).unwrap(),
    )
    .unwrap();
    let mut probe = format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{}",
        crate::X86_SYSTEMS_DATA_LAYOUT,
        include_str!("../../../linux-kernel/labs/qemu/x86_64/frame_target_probe.ll")
    );
    let target = if gateway {
        gateway_llvm(&owner).unwrap()
    } else {
        llvm(&owner).unwrap()
    };
    if gateway {
        for (old, new) in [
            (
                "topal_x86_frame_allocator_create_v1",
                "probe.gateway_create",
            ),
            (
                "topal_frame_allocator_allocate_v1",
                "probe.gateway_allocate",
            ),
            ("topal_frame_allocator_release_v1", "probe.gateway_release"),
            ("topal_frame_allocator_pin_v1", "probe.gateway_pin"),
            ("topal_frame_allocator_resolve_v1", "probe.gateway_resolve"),
        ] {
            probe = probe.replace(old, new);
        }
        probe = probe
            .lines()
            .filter(|line| !line.starts_with("declare ") || !line.contains("@probe.gateway_"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        probe.push_str(include_str!(
            "../../../linux-kernel/labs/qemu/x86_64/frame_gateway_probe_wrappers.ll"
        ));
        let dispatch = format!(
            "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{}",
            crate::X86_SYSTEMS_DATA_LAYOUT,
            include_str!("runtime/kernel_frame_dispatch.ll")
        );
        fs::write(work.join("dispatch.ll"), dispatch).unwrap();
    }
    for (role, text) in [
        ("source", owner.runtime.source_step.llvm.as_str()),
        ("runtime", &owner.runtime.llvm),
        ("owner", &owner.llvm),
        (
            "controller",
            &crate::systems_frame_controller::llvm().unwrap(),
        ),
        ("target", &target),
        ("probe", &probe),
    ] {
        fs::write(work.join(format!("{role}.ll")), text).unwrap();
    }
    fs::write(work.join("boundary.o"), &owner.runtime.boundary).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let mut roles = vec![
        "source",
        "runtime",
        "owner",
        "controller",
        "target",
        "probe",
    ];
    if gateway {
        roles.push("dispatch");
    }
    for role in roles {
        run(
            &work,
            &format!("{role}-verify"),
            Command::new(tools.tool_path("opt"))
                .args(["-passes=verify", "-disable-output"])
                .arg(work.join(format!("{role}.ll"))),
        );
        run(
            &work,
            &format!("{role}-llc"),
            Command::new(tools.tool_path("llc"))
                .args([
                    "-filetype=obj",
                    "-O2",
                    "-mcpu=x86-64",
                    "-mattr=-mmx,-sse,-sse2,-avx,-avx2",
                    "--frame-pointer=all",
                    "-stack-size-section",
                    "-function-sections",
                    "--disable-tail-calls",
                ])
                .arg(work.join(format!("{role}.ll")))
                .arg("-o")
                .arg(work.join(format!("{role}.o"))),
        );
        run(
            &work,
            &format!("{role}-disassembly"),
            Command::new(tools.tool_path("llvm-objdump"))
                .args(["-d", "--no-show-raw-insn"])
                .arg(work.join(format!("{role}.o"))),
        );
        if !matches!(role, "target" | "probe") {
            crate::systems_library::inspect_instructions(
                &fs::read_to_string(work.join(format!("{role}-disassembly.stdout.log"))).unwrap(),
            )
            .unwrap();
        }
    }
    let target_disassembly =
        fs::read_to_string(work.join("target-disassembly.stdout.log")).unwrap();
    let mut function = "";
    let mut privileged = Vec::new();
    let mut filtered = String::new();
    for line in target_disassembly.lines() {
        if line.ends_with(">:") {
            if let Some((_, symbol)) = line.split_once(" <") {
                function = symbol.trim_end_matches(">:");
            }
        }
        let mnemonic = line
            .split_once(':')
            .and_then(|(_, body)| body.split_whitespace().next());
        if matches!(mnemonic, Some("cli" | "hlt")) {
            assert_eq!(function, "topal_x86_frame_target_halt_v1");
            privileged.push(mnemonic.unwrap());
        } else {
            filtered.push_str(line);
            filtered.push('\n');
        }
    }
    assert_eq!(
        privileged,
        if gateway { vec![] } else { vec!["cli", "hlt"] }
    );
    crate::systems_library::inspect_instructions(&filtered).unwrap();
    let inner = crate::systems_tail_runtime::inspection::inspect(
        &fs::read(work.join("source.o")).unwrap(),
        &fs::read(work.join("runtime.o")).unwrap(),
    )
    .unwrap();
    let outer = crate::systems_tail_owner::inspection::inspect(
        &fs::read(work.join("owner.o")).unwrap(),
        &inner,
    )
    .unwrap();
    let frame = if gateway {
        inspection::inspect_gateway(
            &fs::read(work.join("controller.o")).unwrap(),
            &fs::read(work.join("target.o")).unwrap(),
            &fs::read(work.join("dispatch.o")).unwrap(),
            &outer,
        )
        .unwrap()
    } else {
        inspection::inspect(
            &fs::read(work.join("controller.o")).unwrap(),
            &fs::read(work.join("target.o")).unwrap(),
            &outer,
        )
        .unwrap()
    };
    if gateway {
        let bound = frame["roots"]["topal_frame_kernel_dispatch_v1"]
            .as_u64()
            .unwrap();
        fs::write(
            work.join("gateway.o"),
            crate::systems_provider_object::tail_boundary::frame_gateway::component_object(bound)
                .unwrap(),
        )
        .unwrap();
        let gateway_bytes = fs::read(work.join("gateway.o")).unwrap();
        let gateway_file = object::File::parse(gateway_bytes.as_slice()).unwrap();
        use object::ObjectSymbol;
        let imports = gateway_file
            .symbols()
            .filter(|s| s.is_undefined())
            .filter_map(|s| s.name().ok())
            .filter(|s| !s.is_empty())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            imports,
            std::collections::BTreeSet::from([
                "topal_frame_kernel_dispatch_v1",
                "topal_x86_frame_context_v1",
                "topal_x86_frame_history_v1",
                "topal_x86_frame_private_v1",
                "topal_x86_frame_stack_v1",
                "topal_x86_frame_facts_v1",
                "topal_x86_frame_output_v1",
                "topal_frame_allocator_active_v1",
                "topal_x86_tail_boundary_active_v1",
                "topal_x86_frame_details_v1"
            ])
        );
        let retired = crate::systems_provider_object::tail_boundary::frame_gateway::RETIRED;
        fs::write(work.join("gateway-binding.json"),serde_json::to_vec_pretty(&serde_json::json!({"qualification":"unpublished bootstrap owner/caller component; production Root and canonical source/native/load publication remain unqualified","object_sha256":crate::artifact::sha256(&gateway_bytes),"imports":imports,"complete_body_stack_bound":bound,"combined_stack_bound":bound+72,"declared_stack_reserve":4096,"retired_scratch_bytes":retired,"saved_register_bytes":48,"recovery_bytes":48,"caller_result_frame_bytes":32})).unwrap()).unwrap();
        run(
            &work,
            "gateway-disassembly",
            Command::new(tools.tool_path("llvm-objdump"))
                .args(["-d", "--no-show-raw-insn"])
                .arg(work.join("gateway.o")),
        );
    }
    for (name, value) in [
        ("inner-inspection", inner),
        ("owner-inspection", outer),
        ("frame-inspection", frame),
    ] {
        fs::write(
            work.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
    // First link obtains the complete allocated end. Relink binds that exact
    // private symbol; verify the symbol's addition did not change load geometry.
    let mut reserved = 0x4000000_u64;
    for label in ["layout", "link"] {
        let mut command = Command::new(tools.tool_path("rust-lld"));
        command.args([
            "-flavor",
            "gnu",
            "-m",
            "elf_x86_64",
            "-static",
            "--no-dynamic-linker",
            "--build-id=none",
            "--entry",
            crate::X86_SYSTEMS_KERNEL_ENTRY,
            "--section-start=.bss=0x800000",
            "-z",
            "max-page-size=0x1000",
        ]);
        command.arg(format!(
            "--defsym=topal_x86_kernel_reservation_end_v1={reserved}"
        ));
        for role in [
            "probe",
            "source",
            "runtime",
            "owner",
            "controller",
            "target",
            "boundary",
        ] {
            command.arg(work.join(format!("{role}.o")));
        }
        if gateway {
            for role in ["dispatch", "gateway"] {
                command.arg(work.join(format!("{role}.o")));
            }
        }
        command.arg("-o").arg(work.join("kernel.elf"));
        run(&work, label, &mut command);
        let kernel = fs::read(work.join("kernel.elf")).unwrap();
        let elf = object::File::parse(kernel.as_slice()).unwrap();
        let actual = elf
            .segments()
            .map(|s| s.address() + s.size())
            .max()
            .unwrap()
            .max(0x1000000);
        if label == "link" {
            assert_eq!(actual, reserved);
        } else {
            reserved = actual;
        }
    }
    let kernel = fs::read(work.join("kernel.elf")).unwrap();
    fs::write(
        work.join("bzImage"),
        crate::systems_boot_image::unpublished_component_fixture(&kernel).unwrap(),
    )
    .unwrap();
    // The boot resource is owned handoff storage. No userspace is executed.
    let resource = crate::systems_boot_resource::generate_initial_boot_resource(
        &crate::systems_provider_object::initial_user_elf_file().unwrap(),
        &crate::qualified_x86_64_static_user_image_fixture(),
    )
    .unwrap();
    fs::write(work.join("initramfs.cpio"), resource).unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../linux-kernel/labs/qemu/x86_64/frame_target_inspect.py");
    let mut modes = vec![
        "real-map",
        "sparse-map",
        "unaligned-map",
        "duplicate-create",
        "interrupts",
        "bad-stack",
        "zero-reservation",
        "unsupported-map",
        "overflow-map",
        "empty-map",
        "reserved-map",
    ];
    if gateway {
        modes.extend([
            "direction-flag",
            "busy-gateway",
            "source-abort",
            "late-cleanup",
            "bad-result",
        ]);
    }
    for mode in modes {
        let mut command = Command::new("python3");
        command.arg(&script).arg(&work).arg("--mode").arg(mode);
        if gateway {
            command.arg("--gateway");
        }
        run(&work, &format!("guest-{mode}"), &mut command);
        let observation: serde_json::Value =
            serde_json::from_slice(&fs::read(work.join(mode).join("observations.json")).unwrap())
                .unwrap();
        assert_eq!(observation["passed"], true);
        if observation["created"] == true {
            use topal_semantics::{
                BootMemoryClass as Class, BootMemoryProvenance as Provenance,
                BootMemoryReservation as Reservation, BootMemorySourceRange as Range,
            };
            let claims = observation["e820"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Range {
                    start: v[0].as_u64().unwrap(),
                    length: v[1].as_u64().unwrap(),
                    provenance: Provenance {
                        class: if v[2] == 1 {
                            Class::Allocatable
                        } else {
                            Class::Reserved
                        },
                        source_kind: if v[2] == 1 {
                            "e820-ram".into()
                        } else {
                            format!("e820-{}", v[2])
                        },
                        provider: "x86".into(),
                    },
                })
                .collect::<Vec<_>>();
            let reservations = observation["reservations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Reservation {
                    start: v[0].as_u64().unwrap(),
                    length: v[1].as_u64().unwrap() - v[0].as_u64().unwrap(),
                    owner: "native-owned".into(),
                    reclaimable: false,
                })
                .collect::<Vec<_>>();
            let model =
                topal_semantics::normalize_boot_memory(&claims, &reservations, 4096).unwrap();
            let expected = model
                .ranges
                .iter()
                .filter(|r| r.class == Class::Allocatable)
                .flat_map(|r| r.start / 4096..r.end / 4096)
                .collect::<Vec<_>>();
            let actual = observation["frames"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                actual, expected,
                "{mode}: actual native map disagrees with reference normalization"
            );
        }
    }
}
