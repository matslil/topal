//! TOPAL-SYSTEMS-FILE-OPEN-PLAN-001: independent native truth table and versions.
use super::*;
use topal_language::compiler::CompilerSystemsSourceLibraryInterface as I;
fn requirement() -> CompilerSystemsSourceLibraryRequirement {
    CompilerSystemsSourceLibraryRequirement {
        source: "policy/file-open-access.t".into(),
        export: "file-open-decision".into(),
        interface: I::FileOpenDecisionV1,
        stack_limit_bytes: 1024,
        runtime_limit_bytes: None,
        state_limit_bytes: None,
    }
}
fn input() -> SystemsSourceLibraryInput {
    load_source_library(
        &requirement(),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel"),
    )
    .unwrap()
}
#[test]
fn file_open_native_domain_adapts_every_source_tag_and_rejects_wide_masks() {
    let tools = crate::LlvmTools::discover(None).unwrap();
    let root = std::env::temp_dir().join(format!("topal-file-open-native-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for reversed in [false, true] {
        let directory = root.join(if reversed { "reversed" } else { "normal" });
        std::fs::create_dir_all(&directory).unwrap();
        let mut source = input();
        if reversed {
            source.text = source.text.replace(
                "OpenDescription, NotDirectory, DirectoryWrite, ReadOnlyView, Missing, AlreadyExists, CreateDescription, TruncateDescription",
                "TruncateDescription, CreateDescription, AlreadyExists, Missing, ReadOnlyView, DirectoryWrite, NotDirectory, OpenDescription");
        }
        let library = build_source_library(&requirement(), &source, &tools, &directory).unwrap();
        assert_eq!(
            library.record.source_tags,
            if reversed {
                (0..8).rev().collect::<Vec<_>>()
            } else {
                (0..8).collect()
            }
        );
        assert!(library.record.maximum_stack_bytes <= 1024);
        assert!(library.record.maximum_stack_bytes > 32);
        let driver = directory.join("driver.c");
        std::fs::write(
            &driver,
            r#"
#include <stdint.h>
extern uint64_t topal_kernel_file_open_policy_v1(uint64_t);
static uint64_t expected(uint64_t mask) {
  int w=mask&1,d=mask&2,r=mask&4,m=mask&8,e=mask&16,c=mask&32,x=mask&64,t=mask&128;
  if(!e) { if(!c) return 4; if(!m) return 3; if(r) return 1; return 6; }
  if(c&&x) return 5;
  if(r&&!d) return 1;
  if(d&&(w||c||t)) return 2;
  if(!d&&!m&&(w||t)) return 3;
  if(!d&&t) return 7;
  return 0;
}
int main(void) {
  unsigned seen=0;
  for(uint64_t i=0;i<256;i++) {
    uint64_t value=topal_kernel_file_open_policy_v1(i);
    if(value!=expected(i)) return 1;
    seen |= 1U<<value;
  }
  if(seen!=255) return 2;
  for(uint64_t i=256;i<65536;i++) if(topal_kernel_file_open_policy_v1(i)!=UINT64_MAX) return 3;
  return topal_kernel_file_open_policy_v1(UINT64_MAX)!=UINT64_MAX ||
    topal_kernel_file_open_policy_v1(UINT64_C(1)<<63)!=UINT64_MAX;
}
"#,
        )
        .unwrap();
        let executable = directory.join("policy");
        let output = std::process::Command::new("cc")
            .arg("-no-pie")
            .arg(&driver)
            .arg(directory.join("kernel-library.o"))
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            std::process::Command::new(executable)
                .status()
                .unwrap()
                .success()
        );
        let mut wrong = library.record.clone();
        wrong.interface = "boot-open-decision-v1".into();
        assert!(inspect_linked_library(&library.object, &wrong).is_err());
        wrong = library.record.clone();
        wrong.native_export = EXPORT_SYMBOL.into();
        assert!(inspect_linked_library(&library.object, &wrong).is_err());
        wrong = library.record.clone();
        wrong.source_tags.push(0);
        assert!(inspect_linked_library(&library.object, &wrong).is_err());
        wrong = library.record.clone();
        wrong.source_tags[7] = wrong.source_tags[0];
        assert!(inspect_linked_library(&library.object, &wrong).is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn file_open_rejects_frozen_shape_and_malformed_nominal_contracts() {
    let mut q = requirement();
    let source = input();
    q.interface = I::BootOpenDecisionV1;
    assert!(check_source_library(&q, &source).is_err());
    for (old, new) in [
        ("FileOpenDecision", "WrongDecision"),
        ("AlreadyExists", "Other"),
        ("write : Boolean", "write : Nat"),
    ] {
        let mut bad = source.clone();
        bad.text = bad.text.replace(old, new);
        assert!(check_source_library(&requirement(), &bad).is_err(), "{old}");
    }
}
