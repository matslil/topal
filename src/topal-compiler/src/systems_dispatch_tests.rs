//! Independent source selection and actual native nominal adaptation.
use super::*;

fn source() -> (
    CompilerSystemsSourceLibraryRequirement,
    SystemsSourceLibraryInput,
) {
    let requirement = CompilerSystemsSourceLibraryRequirement {
        source: "arch/x86_64/policy/static-linux-dispatch.t".into(),
        export: "select-operation".into(),
        interface: CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
        state_limit_bytes: None,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    let input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    (requirement, input)
}

#[test]
fn dispatch_requires_complete_nominal_vocabulary_and_full_declared_selector() {
    // TOPAL-SYSTEMS-NUMERIC-ENUMS-001; TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001
    let (requirement, input) = source();
    let checked = check_numeric_library(&requirement, &input).unwrap();
    assert_eq!(checked.allocation.maximum_bytes, 56);
    assert_eq!(checked.allocation.bridge_bytes, 24);
    assert_eq!(checked.allocation.source_evaluation_bytes, 0);
    assert_eq!(checked.allocation.maximum_operand_bits, 64);
    assert_eq!(checked.source_tags.len(), 56);
    assert_ne!(checked.source_tags, (0..56).collect::<Vec<_>>());
    for (before, after) in [
        ("StaticLinuxOperation", "DifferentOperation"),
        ("RestartableSequences", "DifferentSelection"),
        (
            "pub select-operation is fn (number : Nat)",
            "pub select-operation is fn (number : Int)",
        ),
    ] {
        let mut changed = input.clone();
        changed.text = changed.text.replace(before, after);
        assert_ne!(changed.text, input.text);
        assert!(
            check_numeric_library(&requirement, &changed).is_err(),
            "{after}"
        );
    }
}

#[test]
fn nominal_decisions_bound_reachable_actions_and_reject_unqualified_dead_forms() {
    // TOPAL-SYSTEMS-NUMERIC-ENUMS-001
    let (requirement, mut input) = source();
    let declaration = input
        .text
        .lines()
        .find(|line| line.starts_with("pub StaticLinuxOperation is"))
        .unwrap()
        .to_owned();
    input.modules.clear();
    let prefix = format!(
        "use language (version is v0.1)\n{declaration}\nSelectionKind is Enum (CheapCase, CostlyCase)\n"
    );
    input.text = format!(
        "{prefix}pub select-operation is fn (number : Nat) -> StaticLinuxOperation\n  ignored is number + 1\n  Write\n"
    );
    let flat = check_numeric_library(&requirement, &input)
        .unwrap()
        .allocation;
    let choose = "select-cheap is fn (number : Nat) -> StaticLinuxOperation\n  ignored is number + 1\n  Write\nselect-costly is fn (number : Nat) -> StaticLinuxOperation\n  first is number + 1\n  second is first + 1\n  Open\nchoose is fn ((number : Nat, operation : SelectionKind)) -> StaticLinuxOperation\n  operation\n    CheapCase then select-cheap number\n    otherwise select-costly number\n";
    let constant = format!(
        "{prefix}{choose}pub select-operation is fn (number : Nat) -> StaticLinuxOperation\n  choose (number, CheapCase)\n"
    );
    input.text = constant.clone();
    let selected = check_numeric_library(&requirement, &input)
        .unwrap()
        .allocation;
    assert_eq!(selected.maximum_bytes, flat.maximum_bytes);
    input.text = format!(
        "{prefix}{choose}select-kind is fn (number : Nat) -> SelectionKind\n  number = 0\n    true then CheapCase\n    false then CostlyCase\npub select-operation is fn (number : Nat) -> StaticLinuxOperation\n  selection is select-kind number\n  choose (number, selection)\n"
    );
    let joined = check_numeric_library(&requirement, &input)
        .unwrap()
        .allocation;
    assert!(joined.maximum_bytes > selected.maximum_bytes);
    input.text = constant.replace(
        "second is first + 1",
        "second is \"unqualified dead String\"",
    );
    assert!(check_numeric_library(&requirement, &input).is_err());
    input.text = format!(
        "{prefix}pub select-operation is fn (number : Nat) -> StaticLinuxOperation\n  same is Read = Read\n  same\n    true then Read\n    false then Unsupported\n"
    );
    assert_eq!(
        check_numeric_library(&requirement, &input)
            .unwrap()
            .allocation
            .maximum_bytes,
        56
    );
}

#[test]
fn extended_dispatch_rejects_cross_version_vocabulary_and_preserves_the_frozen_profile() {
    // TOPAL-SYSTEMS-LINUX-DISPATCH-V2-001
    let (legacy, old_input) = source();
    let mut extended = legacy.clone();
    extended.interface = CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV2;
    extended.source = "arch/x86_64/policy/static-linux-dispatch-v2.t".into();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    let input = crate::systems_library::load_source_library(&extended, &root).unwrap();
    assert_eq!(
        check_numeric_library(&extended, &input)
            .unwrap()
            .source_tags
            .len(),
        58
    );
    assert_eq!(
        check_numeric_library(&legacy, &old_input)
            .unwrap()
            .source_tags
            .len(),
        56
    );
    assert!(check_numeric_library(&legacy, &input).is_err());
    assert!(check_numeric_library(&extended, &old_input).is_err());
    for name in ["TruncateFile", "FileCreationMask"] {
        let mut bad = input.clone();
        bad.text = bad.text.replace(name, "UnqualifiedOperation");
        assert!(check_numeric_library(&extended, &bad).is_err());
    }
}

#[test]
fn native_dispatch_matches_independent_selector_set_across_holes_and_wide_inputs() {
    for version in 1..=6 {
        native_dispatch_profile(version);
    }
}

fn native_dispatch_profile(version: u8) {
    // TOPAL-SYSTEMS-NUMERIC-ENUMS-001; TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001
    let (mut requirement, mut input) = source();
    if version >= 2 {
        requirement.interface = CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV2;
        requirement.source = "arch/x86_64/policy/static-linux-dispatch-v2.t".into();
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    }
    if version == 3 {
        requirement.interface = CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV3;
        requirement.source = "arch/x86_64/policy/static-linux-dispatch-v3.t".into();
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    }
    if version == 4 {
        requirement.interface = CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV4;
        requirement.source = "arch/x86_64/policy/static-linux-dispatch-v4.t".into();
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    }
    if version == 5 {
        requirement.interface = CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV5;
        requirement.source = "arch/x86_64/policy/static-linux-dispatch-v5.t".into();
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    }
    if version == 6 {
        requirement.interface = CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6;
        requirement.source = "arch/x86_64/policy/static-linux-dispatch-v6.t".into();
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
        input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    }
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory = std::env::temp_dir().join(format!(
        "topal-dispatch-object-{}-{version}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let library = build_numeric_library(&requirement, &input, &tools, &directory).unwrap();
    assert!(library.record.maximum_stack_bytes <= 4096);
    assert_eq!(library.record.native_invocation_save_bytes, 64);
    assert_eq!(library.record.allocation.maximum_bytes, 56);
    assert!(
        library
            .record
            .functions
            .iter()
            .all(|f| !f.symbol.starts_with("topal.runtime.") && f.symbol != "memset")
    );
    let driver = directory.join("driver.c");
    let driver_source = r#"
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
static unsigned char arena[65536] __attribute__((aligned(16)));
static uint64_t cursor,highwater;
void *topal_kernel_numeric_allocate_v1(uint64_t size) {
  if(size>65529)exit(91);size=(size+7)&~UINT64_C(7);
  if(size>65536-cursor)exit(91);
  void *result=arena+cursor;cursor+=size;if(cursor>highwater)highwater=cursor;return result;
}
__attribute__((noreturn)) void topal_kernel_numeric_abort_v1(void) {exit(92);}
extern uint64_t topal_kernel_static_linux_dispatch_policy_v1(uint64_t);
/* Independently enumerated existing native ABI selectors, including bounded/error arms. */
static const uint64_t known[]={0,1,2,3,5,7,8,10,12,13,14,16,22,24,32,33,35,39,
  56,57,59,60,61,63,72,79,80,81,97,102,104,105,106,107,108,110,157,158,186,201,
  217,218,228,229,230,231,257,262,267,273,292,293,302,318,334};
static int check(uint64_t selector) {
  uint64_t expected=UINT64_MAX;
  for(unsigned i=0;i<sizeof(known)/sizeof(known[0]);i++)if(selector==known[i])expected=selector;
  memset(arena,0,sizeof(arena));cursor=32;
  uint64_t actual=topal_kernel_static_linux_dispatch_policy_v1(selector);
  return actual!=expected || cursor>56;
}
int main(void) {
  for(uint64_t selector=0;selector<65536;selector++)if(check(selector))return 1;
  for(unsigned i=0;i<sizeof(known)/sizeof(known[0]);i++) {
    if(check(known[i]|(UINT64_C(1)<<32)) || check(known[i]|(UINT64_C(1)<<63)))return 2;
  }
  const uint64_t wide[]={UINT64_MAX,UINT64_MAX-1,UINT64_C(1)<<32,UINT64_C(1)<<63,UINT64_C(0xffffffff)};
  for(unsigned i=0;i<sizeof(wide)/sizeof(wide[0]);i++)if(check(wide[i]))return 3;
  printf("%llu\n",(unsigned long long)highwater);return 0;
}
"#;
    let driver_source = if version >= 2 {
        driver_source
            .replace(
                "static_linux_dispatch_policy_v1",
                "static_linux_dispatch_policy_v2",
            )
            .replace("63,72,79", "63,72,77,79")
            .replace("80,81,97", "80,81,95,97")
    } else {
        driver_source.to_owned()
    };
    let driver_source = if version >= 3 {
        driver_source
            .replace(
                "static_linux_dispatch_policy_v2",
                "static_linux_dispatch_policy_v3",
            )
            .replace("80,81,95,97", "80,81,87,95,97")
            .replace("257,262,267", "257,262,263,267")
    } else {
        driver_source
    };
    let driver_source = if version >= 4 {
        driver_source
            .replace(
                "static_linux_dispatch_policy_v3",
                "static_linux_dispatch_policy_v4",
            )
            .replace("13,14,16,22", "13,14,16,17,18,22")
    } else {
        driver_source
    };
    let driver_source = if version >= 5 {
        driver_source
            .replace(
                "static_linux_dispatch_policy_v4",
                "static_linux_dispatch_policy_v5",
            )
            .replace("80,81,87,95", "80,81,87,90,91,95")
            .replace("263,267,273", "263,267,268,273")
            .replace("318,334", "318,334,452")
    } else {
        driver_source
    };
    let driver_source = if version == 6 {
        driver_source
            .replace(
                "static_linux_dispatch_policy_v5",
                "static_linux_dispatch_policy_v6",
            )
            .replace("80,81,87", "80,81,82,86,87")
            .replace("262,263,267", "262,263,264,265,267")
            .replace("302,318,334", "302,316,318,334")
    } else {
        driver_source
    };
    std::fs::write(&driver, driver_source).unwrap();
    let executable = directory.join("dispatch-policy");
    let compiled = std::process::Command::new("cc")
        .arg("-no-pie")
        .arg(&driver)
        .arg(directory.join("dispatch-library.o"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    if version == 6 {
        std::fs::write(directory.join("compile.stdout.log"), &compiled.stdout).unwrap();
        std::fs::write(directory.join("compile.stderr.log"), &compiled.stderr).unwrap();
    }
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let output = std::process::Command::new(&executable).output().unwrap();
    if version == 6 {
        std::fs::write(directory.join("native.stdout.log"), &output.stdout).unwrap();
        std::fs::write(directory.join("native.stderr.log"), &output.stderr).unwrap();
    }
    assert!(
        output.status.success(),
        "status={:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!(
        "dispatch stack={} bound={} highwater={}",
        library.record.maximum_stack_bytes,
        library.record.allocation.maximum_bytes,
        String::from_utf8_lossy(&output.stdout).trim()
    );
    let mut reordered = input.clone();
    let declaration = input
        .text
        .lines()
        .find(|line| line.starts_with("pub StaticLinuxOperation is Enum ("))
        .unwrap();
    let alternatives = declaration
        .strip_prefix("pub StaticLinuxOperation is Enum (")
        .unwrap()
        .strip_suffix(')')
        .unwrap()
        .rsplit(", ")
        .collect::<Vec<_>>();
    reordered.text = reordered.text.replace(
        declaration,
        &format!(
            "pub StaticLinuxOperation is Enum ({})",
            alternatives.join(", ")
        ),
    );
    let second = build_numeric_library(&requirement, &reordered, &tools, &directory).unwrap();
    assert_ne!(second.record.source_tags, library.record.source_tags);
    assert_eq!(second.record.allocation, library.record.allocation);
    for (original, reversed) in library
        .record
        .source_tags
        .iter()
        .zip(&second.record.source_tags)
    {
        let last_tag = match version {
            6 => 70,
            5 => 65,
            _ => 55 + 2 * u32::from(version - 1),
        };
        assert_eq!(original + reversed, last_tag);
    }
    let compiled = std::process::Command::new("cc")
        .arg("-no-pie")
        .arg(&driver)
        .arg(directory.join("dispatch-library.o"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    if version == 6 {
        std::fs::write(
            directory.join("reordered-compile.stdout.log"),
            &compiled.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join("reordered-compile.stderr.log"),
            &compiled.stderr,
        )
        .unwrap();
    }
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let output = std::process::Command::new(&executable).output().unwrap();
    if version == 6 {
        std::fs::write(
            directory.join("reordered-native.stdout.log"),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join("reordered-native.stderr.log"),
            &output.stderr,
        )
        .unwrap();
    }
    assert!(
        output.status.success(),
        "reordered status={:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!(
        "reordered dispatch stack={} bound={} highwater={}",
        second.record.maximum_stack_bytes,
        second.record.allocation.maximum_bytes,
        String::from_utf8_lossy(&output.stdout).trim()
    );
    if version == 6 {
        std::fs::write(
            directory.join("normal.record.json"),
            serde_json::to_vec_pretty(&library.record).unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("reordered.record.json"),
            serde_json::to_vec_pretty(&second.record).unwrap(),
        )
        .unwrap();
    } else {
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn all_dispatch_versions_admit_only_their_exact_nominal_vocabulary() {
    // TOPAL-SYSTEMS-LINUX-DISPATCH-V3-001
    let (legacy, _) = source();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    let versions = [
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV1,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV2,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV3,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV4,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV5,
        CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6,
    ];
    for (version, interface) in versions.iter().enumerate() {
        let mut q = legacy.clone();
        q.interface = *interface;
        q.source = if version == 0 {
            "arch/x86_64/policy/static-linux-dispatch.t".into()
        } else {
            format!(
                "arch/x86_64/policy/static-linux-dispatch-v{}.t",
                version + 1
            )
        };
        let input = crate::systems_library::load_source_library(&q, &root).unwrap();
        let checked = check_numeric_library(&q, &input).unwrap();
        assert_eq!(
            checked.source_tags.len(),
            match version {
                4 => 66,
                5 => 71,
                _ => 56 + 2 * version,
            }
        );
        for other in versions {
            if other == *interface {
                continue;
            }
            let mut wrong = q.clone();
            wrong.interface = other;
            assert!(check_numeric_library(&wrong, &input).is_err());
        }
    }
}
