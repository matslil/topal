//! Source admission over actual copied byte-domain assumptions.
use super::*;

fn source() -> (
    CompilerSystemsSourceLibraryRequirement,
    SystemsSourceLibraryInput,
) {
    let requirement = CompilerSystemsSourceLibraryRequirement {
        source: "policy/static-elf64-header.t".into(),
        export: "header-admitted?".into(),
        interface: CompilerSystemsSourceLibraryInterface::StaticElf64HeaderV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../linux-kernel/kernel");
    let input = crate::systems_library::load_source_library(&requirement, &root).unwrap();
    (requirement, input)
}

#[test]
fn header_source_has_exact_array_facts_boolean_result_and_complete_copy_cost() {
    // TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001
    let (requirement, input) = source();
    let checked = check_numeric_library(&requirement, &input).unwrap();
    assert_eq!(checked.policy, NumericPolicy::Elf64Header);
    assert_eq!(checked.allocation.bridge_bytes, 2624);
    assert_eq!(checked.allocation.maximum_operand_bits, 32);
    assert_eq!(checked.allocation.maximum_bytes, 9648);
    assert_eq!(checked.program.dependencies.len(), 1);
    let entry = checked
        .program
        .functions
        .iter()
        .find(|f| f.symbol == checked.entry_symbol)
        .unwrap();
    assert_eq!(entry.result_type, CompilerType::Boolean);
    assert_eq!(
        entry
            .parameters
            .iter()
            .map(|p| p.value_type.clone())
            .collect::<Vec<_>>(),
        checked.policy.parameters()
    );
}

#[test]
fn native_header_snapshot_preserves_bytes_and_matches_independent_wire_decoding() {
    // TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001
    let (requirement, input) = source();
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory =
        std::env::temp_dir().join(format!("topal-header-object-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let library = build_numeric_library(&requirement, &input, &tools, &directory).unwrap();
    assert_eq!(
        library.record.native_export,
        NumericPolicy::Elf64Header.native_export()
    );
    assert!(library.record.maximum_stack_bytes <= 4096);
    assert_eq!(library.record.allocation.maximum_bytes, 9648);
    assert!(
        library
            .record
            .functions
            .iter()
            .all(|f| !f.symbol.starts_with("topal.runtime.") && f.symbol != "memset")
    );
    let driver = directory.join("driver.c");
    std::fs::write(&driver,r#"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
static unsigned char arena[65536] __attribute__((aligned(16)));
static uint64_t cursor=32,highwater=32;
void *topal_kernel_numeric_allocate_v1(uint64_t size) {
  if(size>65529) exit(91);
  size=(size+7)&~UINT64_C(7);
  if(size>65536-cursor) exit(91);
  void *result=arena+cursor;cursor+=size;
  if(cursor>highwater)highwater=cursor;
  return result;
}
__attribute__((noreturn)) void topal_kernel_numeric_abort_v1(void) {exit(92);}
extern uint64_t topal_kernel_static_elf64_header_policy_v1(const unsigned char*,uint64_t,uint64_t,uint64_t);
static unsigned word(const unsigned char *b,unsigned offset) {return (unsigned)b[offset]|((unsigned)b[offset+1]<<8);}
static int expected(const unsigned char *b,uint64_t machine,uint64_t maximum) {
  return !memcmp(b,"\177ELF\2\1\1",7) && word(b,16)==2 && word(b,18)==machine
    && b[20]==1 && b[21]==0 && b[22]==0 && b[23]==0 && word(b,52)==64 && word(b,54)==56
    && word(b,56)>=1 && word(b,56)<=maximum;
}
static int check(const unsigned char *b,uint64_t machine,uint64_t maximum) {
  struct {unsigned char before[17],bytes[64],after[17];} input __attribute__((aligned(16)));
  memset(input.before,0xa5,17);memcpy(input.bytes,b,64);memset(input.after,0x5a,17);
  if(((uintptr_t)input.bytes&15)!=1)return 6;
  memset(arena,0,sizeof(arena));cursor=32;
  uint64_t actual=topal_kernel_static_elf64_header_policy_v1(input.bytes,64,machine,maximum);
  if(actual!=(uint64_t)expected(b,machine,maximum) || cursor>9648 || memcmp(input.bytes,b,64))return 1;
  for(unsigned i=0;i<17;i++)if(input.before[i]!=0xa5 || input.after[i]!=0x5a)return 2;
  return 0;
}
int main(int argc,char **argv) {
  if(argc>1) {
    const unsigned char *bad=(const unsigned char *)(uintptr_t)1;
    switch(argv[1][0]) {
      case 'n':topal_kernel_static_elf64_header_policy_v1(0,64,62,16);break;
      case 'l':topal_kernel_static_elf64_header_policy_v1(bad,63,62,16);break;
      case 'L':topal_kernel_static_elf64_header_policy_v1(bad,65,62,16);break;
      case 'm':topal_kernel_static_elf64_header_policy_v1(bad,64,0,16);break;
      case 'M':topal_kernel_static_elf64_header_policy_v1(bad,64,65536,16);break;
      case 'p':topal_kernel_static_elf64_header_policy_v1(bad,64,62,0);break;
      case 'P':topal_kernel_static_elf64_header_policy_v1(bad,64,62,65536);break;
      case 'w':topal_kernel_static_elf64_header_policy_v1((const unsigned char *)(UINTPTR_MAX-31),64,62,16);break;
    }
    return 3;
  }
  unsigned char base[64]={0x7f,'E','L','F',2,1,1,[16]=2,[18]=62,[20]=1,[52]=64,[54]=56,[56]=4};
  for(unsigned position=0;position<64;position++)for(unsigned value=0;value<256;value++) {
    unsigned char changed[64];memcpy(changed,base,64);changed[position]=(unsigned char)value;
    if(check(changed,62,16))return 4;
  }
  for(unsigned machine=1;machine<=65535;machine+=32767) {
    base[18]=(unsigned char)machine;base[19]=(unsigned char)(machine>>8);
    base[56]=255;base[57]=255;
    if(check(base,machine,65535) || check(base,machine,65534))return 5;
  }
  printf("%llu\n",(unsigned long long)highwater);
  return 0;
}
"#).unwrap();
    let executable = directory.join("header-policy");
    let compiled = std::process::Command::new("cc")
        .arg("-no-pie")
        .arg(&driver)
        .arg(directory.join("image-library.o"))
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
        "status={:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!(
        "header stack={} bound={} highwater={}",
        library.record.maximum_stack_bytes,
        library.record.allocation.maximum_bytes,
        String::from_utf8_lossy(&output.stdout).trim()
    );
    for argument in ["n", "l", "L", "m", "M", "p", "P", "w"] {
        assert_eq!(
            std::process::Command::new(&executable)
                .arg(argument)
                .status()
                .unwrap()
                .code(),
            Some(92),
            "guard={argument}"
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}
