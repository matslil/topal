//! Finite sequence ranges, evaluated costs and checked Optional selection.
use super::*;
fn input(
    helpers: &str,
    body: &str,
) -> (
    CompilerSystemsSourceLibraryRequirement,
    SystemsSourceLibraryInput,
) {
    let requirement = CompilerSystemsSourceLibraryRequirement {
        source: "policy/arrays.t".into(),
        export: "sleep-deadline".into(),
        interface: CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
        state_limit_bytes: None,
    };
    let input = SystemsSourceLibraryInput {
        source: requirement.source.clone(),
        source_name: requirement.source.clone(),
        modules: Vec::new(),
        text: format!(
            "use language (version is v0.1)\n{helpers}\npub sleep-deadline is fn ((seconds : Nat, nanoseconds : Nat, ticks : Nat, wrapped : Boolean)) -> Int\n  s : Int is seconds\n  n : Int is nanoseconds\n{body}\n"
        ),
    };
    (requirement, input)
}
fn checked(helpers: &str, body: &str) -> CheckedNumericLibrary {
    let (requirement, input) = input(helpers, body);
    check_numeric_library(&requirement, &input).unwrap_or_else(|error| {
        panic!(
            "{error}\nsource: {}\nfrontend: {:?}",
            input.text,
            topal_language::analyze_for_compiler(&input.text)
        )
    })
}
fn evaluate(unit: &CheckedNumericLibrary) -> Evaluation {
    let mut bounds = Bounds {
        functions: unit
            .program
            .functions
            .iter()
            .map(|f| (f.symbol.clone(), f))
            .collect(),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        maximum_bits: 0,
        regions: false,
    };
    let number = |n| Value::Number(Range(BigInt::from(n), BigInt::from(n)));
    bounds
        .function(
            &unit.entry_symbol,
            vec![number(3), number(5), number(7), Value::Boolean],
        )
        .unwrap()
}
const GET: &str = "choose-entry is fn (parts : Array (2, Int)) -> Int\n  array-at? (parts, 1)\n    Some value then value\n    None then 0\n";
const ARRAY: &str = "  entries : List Int is Entry (s, Entry (n, Empty))\n  parts : Array (2, Int) is entries collect Array\n";
#[test]
fn arrays_preserve_each_entry_and_charge_complete_construction_and_reads() {
    // TOPAL-SYSTEMS-NUMERIC-ARRAYS-001
    let unit = checked(GET, &format!("{ARRAY}  choose-entry parts"));
    let actual = evaluate(&unit);
    assert_eq!(
        actual.value.number().unwrap(),
        &Range(BigInt::from(5), BigInt::from(5))
    );
    assert_eq!(actual.bytes, 64); // two nodes, one header, one Optional
    let direct = checked(
        GET,
        "  entries : List Int is Entry (s, Entry (n, Empty))\n  choose-entry (entries collect Array)",
    );
    assert_eq!(evaluate(&direct).bytes, actual.bytes);
    let costly = checked(
        GET,
        &format!(
            "{}  choose-entry parts",
            ARRAY.replace("Entry (s,", "Entry (s * s,")
        ),
    );
    assert!(evaluate(&costly).bytes > actual.bytes); // unselected entry still evaluates
    assert_eq!(evaluate(&costly).value, actual.value);
}
#[test]
fn empty_and_out_of_bounds_arrays_return_none_and_charge_its_wrapper() {
    // TOPAL-SYSTEMS-NUMERIC-ARRAYS-001
    let helper = "read-empty is fn (parts : Array (0, Int)) -> Int\n  array-at? (parts, 0)\n    Some value then value\n    None then 17\n";
    let unit = checked(
        helper,
        "  entries : List Int is Empty\n  parts : Array (0, Int) is entries collect Array\n  read-empty parts",
    );
    assert_eq!(evaluate(&unit).value.number().unwrap().0, BigInt::from(17));
    assert_eq!(evaluate(&unit).bytes, 32);
    let outside = checked(
        &GET.replace("parts, 1", "parts, 2"),
        &format!("{ARRAY}  choose-entry parts"),
    );
    assert_eq!(
        evaluate(&outside).value.number().unwrap().0,
        BigInt::from(0)
    );
    assert_eq!(evaluate(&outside).bytes, 64);
    let count = checked("", &format!("{ARRAY}  entry-count parts"));
    assert_eq!(evaluate(&count).value.number().unwrap().0, BigInt::from(2));
    assert_eq!(evaluate(&count).bytes, 72);
}
#[test]
fn optional_and_array_joins_retain_all_possible_payload_ranges() {
    // TOPAL-SYSTEMS-NUMERIC-ARRAYS-001
    let helper = "make-optional is fn ((parts : Array (2, Int), wrapped : Boolean)) -> Optional Int\n  wrapped\n    true then array-at? (parts, 0)\n    false then None Int\nselect-value is fn ((parts : Array (2, Int), wrapped : Boolean)) -> Int\n  selected is make-optional (parts, wrapped)\n  selected\n    Some value then value\n    None then 19\n";
    let unit = checked(helper, &format!("{ARRAY}  select-value (parts, wrapped)"));
    let actual = evaluate(&unit);
    assert_eq!(
        actual.value.number().unwrap(),
        &Range(BigInt::from(3), BigInt::from(19))
    );
    assert_eq!(actual.bytes, 64);
    let helper = "make-choice is fn ((a : Array (2, Int), b : Array (2, Int), wrapped : Boolean)) -> Array (2, Int)\n  wrapped\n    true then a\n    false then b\nselect-array is fn ((a : Array (2, Int), b : Array (2, Int), wrapped : Boolean)) -> Int\n  selected is make-choice (a, b, wrapped)\n  array-at? (selected, 0)\n    Some value then value\n    None then 0\n";
    let unit = checked(
        helper,
        &format!(
            "{ARRAY}  other : List Int is Entry (n, Entry (s, Empty))\n  select-array (parts, other collect Array, wrapped)"
        ),
    );
    assert_eq!(
        evaluate(&unit).value.number().unwrap(),
        &Range(BigInt::from(3), BigInt::from(5))
    );
}
#[test]
fn unqualified_elements_extents_and_operations_are_rejected() {
    // TOPAL-SYSTEMS-NUMERIC-ARRAYS-001
    for body in [
        "  entries : List Int is Entry (s, Empty)\n  parts : Array (2, Int) is entries collect Array\n  0",
        "  entries : List Boolean is Entry (true, Empty)\n  parts is entries collect Array\n  0",
        "  entries : List Int is Entry (s, Empty)\n  parts is entries collect Set\n  0",
        "  entries : List Int is Entry (s, Empty)\n  values is entries reverse\n  0",
    ] {
        let (requirement, input) = input("", body);
        assert!(
            check_numeric_library(&requirement, &input).is_err(),
            "accepted {body}"
        );
    }
    let mut entries = "  entries0 : List Int is Empty\n".to_owned();
    for i in 1..258 {
        entries.push_str(&format!(
            "  entries{i} : List Int is Entry (s, entries{})\n",
            i - 1
        ));
    }
    let (requirement, input) = input("", &format!("{entries}  0"));
    let error = match check_numeric_library(&requirement, &input) {
        Ok(_) => panic!("oversized sequence admitted"),
        Err(e) => e.to_string(),
    };
    assert!(error.contains("sequence length"), "{error}");
}

#[test]
fn actual_native_arrays_execute_empty_absent_present_and_joined_optional_paths() {
    // TOPAL-SYSTEMS-NUMERIC-ARRAYS-001
    let helpers = "absent-empty is fn (parts : Array (0, Int)) -> Int\n  array-at? (parts, 0)\n    Some value then 400\n    None then 17\nabsent-full is fn (parts : Array (2, Int)) -> Int\n  array-at? (parts, 2)\n    Some value then 500\n    None then 19\npick-entry is fn ((parts : Array (2, Int), wrapped : Boolean)) -> Optional Int\n  wrapped\n    true then array-at? (parts, 1)\n    false then None Int\nunwrap-value is fn (candidate : Optional Int) -> Int\n  candidate\n    Some value then value\n    None then 23\n";
    let body = format!(
        "{ARRAY}  empty-list : List Int is Empty\n  empty-array : Array (0, Int) is empty-list collect Array\n  missing-empty is absent-empty empty-array\n  missing-full is absent-full parts\n  selected is unwrap-value (pick-entry (parts, wrapped))\n  count is entry-count parts\n  s + selected + missing-empty + missing-full + count"
    );
    let (requirement, source) = input(helpers, &body);
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory =
        std::env::temp_dir().join(format!("topal-numeric-arrays-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let library = build_numeric_library(&requirement, &source, &tools, &directory).unwrap();
    assert!(library.record.maximum_stack_bytes <= 4096);
    let driver = directory.join("driver.c");
    std::fs::write(
        &driver,
        r#"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
static unsigned char arena[65536] __attribute__((aligned(16)));
static uint64_t cursor;
void *topal_kernel_numeric_allocate_v1(uint64_t size) {
  if (size>65529) exit(91);
  size=(size+7)&~UINT64_C(7);
  if(size>65536-cursor) exit(91);
  void *result=arena+cursor;cursor+=size;return result;
}
__attribute__((noreturn)) void topal_kernel_numeric_abort_v1(void) {exit(92);}
extern uint64_t topal_kernel_sleep_deadline_policy_v1(uint64_t,uint64_t,uint64_t,uint64_t);
int main(void) {
  for(uint64_t s=0;s<32;s++) for(uint64_t n=0;n<32;n++) for(uint64_t w=0;w<2;w++) {
    memset(arena,0,sizeof(arena));cursor=32;
    uint64_t actual=topal_kernel_sleep_deadline_policy_v1(s,n,0,w);
    if(actual!=s+(w?n:23)+17+19+2) return 1;
  }
  return 0;
}
"#,
    )
    .unwrap();
    let executable = directory.join("arrays");
    let compiled = std::process::Command::new("cc")
        .arg("-no-pie")
        .arg(&driver)
        .arg(directory.join("time-library.o"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(
        std::process::Command::new(&executable)
            .status()
            .unwrap()
            .success()
    );
    std::fs::remove_dir_all(directory).unwrap();
}
