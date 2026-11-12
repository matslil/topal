use super::*;
use crate::systems_tail_owner::run_tail_native_fixture as run;
use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

fn predecessor() -> String {
    include_str!("runtime/kernel_frame_target.ll").replace(
        "nounwind noinline {",
        "nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {",
    )
}

#[test]
fn preserves_the_bytewise_predecessor_and_all_private_bindings() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: same complete checks, owned target only.
    let old = predecessor();
    let new = adapt(old.clone()).unwrap();
    assert!(old.contains(ORIGINAL));
    assert!(!new.contains(ORIGINAL));
    assert_eq!(new, old.replace(ORIGINAL, WIDE.trim_end()));
    assert!(!WIDE.contains("store "));
    assert!(!WIDE.contains("call "));
}

#[test]
fn rejects_changed_predicate_or_unreviewed_private_geometry() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: do not silently replace future checks.
    let old = predecessor();
    assert!(adapt(old.replace("%zero = icmp eq i8 %v,0", "%zero = icmp ne i8 %v,0")).is_err());
    assert!(adapt(old.replace("private_v1,i64 65528", "private_v1,i64 65536")).is_err());
    assert!(adapt(old.replace("global [65536 x i8]", "global [65528 x i8]")).is_err());
    assert!(adapt(old + "\n  call i1 @t.clean(ptr null,i64 0)\n").is_err());
}

#[test]
fn native_component_preserves_every_private_byte_and_tail() {
    // TOPAL-SYSTEMS-USER-EXTENT-001: ordinary host component over its own
    // buffers, not a booted-kernel ownership or lifetime qualification.
    let work = std::env::temp_dir().join(format!(
        "user-extent-private-clean-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&work).unwrap();
    eprintln!("private zero-check component: {}", work.display());
    let rename = |text: &str, name: &str| {
        text.replace(
            "define internal i1 @t.clean",
            &format!("define dso_local i1 @{name}"),
        )
    };
    let ir = format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{}\n{}\n",
        crate::X86_SYSTEMS_DATA_LAYOUT,
        rename(ORIGINAL, "topal_test_clean_old"),
        rename(WIDE, "topal_test_clean_wide")
    );
    fs::write(work.join("clean.ll"), ir).unwrap();
    let tools = crate::LlvmTools::discover(None).unwrap();
    run(
        &work,
        "verify",
        Command::new(tools.tool_path("opt"))
            .args(["-passes=verify", "-disable-output"])
            .arg(work.join("clean.ll")),
    );
    run(
        &work,
        "compile",
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
            .arg(work.join("clean.ll"))
            .arg("-o")
            .arg(work.join("clean.o")),
    );
    let bytes = fs::read(work.join("clean.o")).unwrap();
    let object = object::File::parse(bytes.as_slice()).unwrap();
    assert!(!object.symbols().any(|symbol| symbol.is_undefined()));
    let frames = crate::systems_library::stack_sizes(&object).unwrap();
    assert_eq!(frames.len(), 2);
    assert!(frames.values().all(|bytes| *bytes <= 32));
    fs::write(
        work.join("stack-sizes.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
    let mut functions = serde_json::Map::new();
    for name in ["topal_test_clean_old", "topal_test_clean_wide"] {
        let symbol = object.symbol_by_name(name).unwrap();
        let section = object
            .section_by_index(symbol.section_index().unwrap())
            .unwrap();
        assert_eq!(section.relocations().count(), 0);
        let start = (symbol.address() - section.address()) as usize;
        let body = &section.data().unwrap()[start..start + symbol.size() as usize];
        functions.insert(
            name.into(),
            serde_json::json!({
            "bytes":body.len(),"sha256":format!("{:x}",Sha256::digest(body))}),
        );
    }
    fs::write(
        work.join("functions.json"),
        serde_json::to_vec_pretty(&functions).unwrap(),
    )
    .unwrap();
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../linux-kernel/labs/qemu/x86_64/private_clean_cases.c"),
        work.join("cases.c"),
    )
    .unwrap();
    run(
        &work,
        "link",
        Command::new("cc")
            .args(["-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-no-pie"])
            .arg(work.join("cases.c"))
            .arg(work.join("clean.o"))
            .arg("-o")
            .arg(work.join("cases")),
    );
    run(&work, "probe", &mut Command::new(work.join("cases")));
    let output = fs::read_to_string(work.join("probe.stdout.log")).unwrap();
    let probes = output
        .trim()
        .strip_prefix("PRIVATE_CLEAN_COMPONENT_PASS probes=")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    assert!(probes > 200000);
}
