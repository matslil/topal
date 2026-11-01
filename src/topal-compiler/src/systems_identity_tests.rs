//! Named result extraction and the actual private two-word native return ABI.
use super::*;

fn source() -> (
    CompilerSystemsSourceLibraryRequirement,
    SystemsSourceLibraryInput,
) {
    let requirement = CompilerSystemsSourceLibraryRequirement {
        source: "policy/process-identity.t".into(),
        export: "process-identity".into(),
        interface: CompilerSystemsSourceLibraryInterface::ProcessIdentityTransitionV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    let input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    (requirement, input)
}

#[test]
fn identity_requires_exact_declared_input_and_named_flat_numeric_output() {
    // TOPAL-SYSTEMS-NUMERIC-RESULT-RECORD-001; TOPAL-SYSTEMS-PROCESS-IDENTITY-LIBRARY-001
    let (requirement, input) = source();
    let checked = check_numeric_library(&requirement, &input).unwrap();
    assert_eq!(checked.result_field_order, ["allocated", "next"]);
    assert!(checked.source_tags.is_empty());
    assert_eq!(checked.allocation.bridge_bytes, 24);
    for (from, to) in [
        ("state : Record (next : Nat)", "state : Record (next : Int)"),
        ("allocated : Nat, next : Nat", "allocated : Int, next : Nat"),
        ("allocated : Nat, next : Nat", "next : Nat"),
        ("pub process-identity", "process-identity"),
    ] {
        let mut changed = input.clone();
        changed.text = input.text.replace(from, to);
        assert_ne!(input.text, changed.text);
        assert!(
            check_numeric_library(&requirement, &changed).is_err(),
            "{to}"
        );
    }
}

#[test]
fn native_identity_return_matches_independent_transition_and_field_order() {
    // TOPAL-SYSTEMS-NUMERIC-RESULT-RECORD-001; TOPAL-SYSTEMS-PROCESS-IDENTITY-LIBRARY-001
    let (requirement, input) = source();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory =
        std::env::temp_dir().join(format!("topal-identity-object-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let driver = directory.join("driver.c");
    std::fs::write(&driver, r#"
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
static unsigned char arena[65536] __attribute__((aligned(16)));
static uint64_t cursor=32,highwater=32;
static int input_guard;
void *topal_kernel_numeric_allocate_v1(uint64_t size) {
  if(size>65529)exit(91);size=(size+7)&~UINT64_C(7);
  if(size>65536-cursor)exit(91);
  void *result=arena+cursor;cursor+=size;if(cursor>highwater)highwater=cursor;return result;
}
__attribute__((noreturn)) void topal_kernel_numeric_abort_v1(void) {exit(input_guard && cursor!=32?93:92);}
typedef struct {uint64_t allocated,next;} transition;
extern transition topal_kernel_process_identity_transition_v1(uint64_t);
static int check(uint64_t current) {
  uint64_t allocated=current?current:2;
  uint64_t following=allocated==INT32_MAX?2:allocated+1;
  memset(arena,0,sizeof(arena));cursor=32;
  transition actual=topal_kernel_process_identity_transition_v1(current);
  return actual.allocated!=allocated || actual.next!=following || cursor>65536;
}
int main(int argc,char **argv) {
  if(argc>1) {
    input_guard=1;
    if(argv[1][0]=='a') topal_kernel_process_identity_transition_v1(1);
    if(argv[1][0]=='b') topal_kernel_process_identity_transition_v1(UINT64_C(2147483648));
    if(argv[1][0]=='c') topal_kernel_process_identity_transition_v1(UINT64_MAX);
    return 2;
  }
  for(uint64_t current=0;current<65536;current++)if(current!=1 && check(current))return 3;
  const uint64_t boundary[]={INT32_MAX-2,INT32_MAX-1,INT32_MAX};
  for(unsigned i=0;i<3;i++)if(check(boundary[i]))return 4;
  uint64_t seed=UINT64_C(0x1234567890abcdef);
  for(unsigned i=0;i<1000;i++) {
    seed=seed*UINT64_C(6364136223846793005)+1;
    if(check(2+(seed%(INT32_MAX-1))))return 5;
  }
  printf("%llu\n",(unsigned long long)highwater);return 0;
}
"#).unwrap();
    let mut reordered = input.clone();
    reordered.text = reordered
        .text
        .replace(
            "Record (allocated : Nat, next : Nat)",
            "Record (next : Nat, allocated : Nat)",
        )
        .replace(
            "(allocated is allocated, next is following)",
            "(next is following, allocated is allocated)",
        );
    for (index, source) in [input.clone(), reordered].iter().enumerate() {
        let library = build_numeric_library(&requirement, source, &tools, &directory).unwrap();
        assert_eq!(library.record.result_field_order, ["allocated", "next"]);
        assert!(library.record.result_order_suffix);
        assert!(library.record.maximum_stack_bytes <= 4096);
        let executable = directory.join("identity-policy");
        let compiled = std::process::Command::new("cc")
            .arg("-no-pie")
            .arg(&driver)
            .arg(directory.join("process-identity-library.o"))
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let output = std::process::Command::new(&executable).output().unwrap();
        assert!(
            output.status.success(),
            "{:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        let highwater = String::from_utf8(output.stdout)
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
        assert!(highwater <= library.record.allocation.maximum_bytes);
        eprintln!(
            "identity order={index} stack={} bound={} highwater={highwater}",
            library.record.maximum_stack_bytes, library.record.allocation.maximum_bytes
        );
        for argument in ["a", "b", "c"] {
            let output = std::process::Command::new(&executable)
                .arg(argument)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(92));
        }
    }
    // Ordinary source can have the right type but violate the selected result
    // protocol. The bridge must reject, including a multiword value that would
    // appear valid if it were truncated to one native limb.
    for (index, expression) in ["0", "2147483648", "18446744073709551618"]
        .iter()
        .enumerate()
    {
        let mut changed = input.clone();
        changed.text = input
            .text
            .replace("next is following", &format!("next is {expression}"));
        build_numeric_library(&requirement, &changed, &tools, &directory).unwrap();
        let executable = directory.join(format!("invalid-{index}"));
        let compiled = std::process::Command::new("cc")
            .arg("-no-pie")
            .arg(&driver)
            .arg(directory.join("process-identity-library.o"))
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        assert_eq!(
            std::process::Command::new(executable)
                .status()
                .unwrap()
                .code(),
            Some(92)
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}
