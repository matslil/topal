use super::*;

#[test]
fn compiled_numeric_policy_uses_owned_arena_and_general_native_inputs() {
    // TOPAL-SYSTEMS-NUMERIC-RUNTIME-001, TOPAL-SYSTEMS-SLEEP-DEADLINE-LIBRARY-001
    let requirement = CompilerSystemsSourceLibraryRequirement {
        source: "policy/relative-sleep-deadline.t".into(),
        export: "sleep-deadline".into(),
        interface: CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
    };
    let source = SystemsSourceLibraryInput {
        source: requirement.source.clone(),
        source_name: requirement.source.clone(),
        text: include_str!("../../../linux-kernel/kernel/policy/relative-sleep-deadline.t").into(),
    };
    let tools = crate::LlvmTools::discover(None).unwrap();
    let directory =
        std::env::temp_dir().join(format!("topal-numeric-object-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let library = build_numeric_library(&requirement, &source, &tools, &directory).unwrap();
    assert!(library.record.maximum_stack_bytes <= 4096);
    assert_eq!(library.record.allocation.maximum_bytes, 41544);
    let driver = directory.join("driver.c");
    std::fs::write(
        &driver,
        r#"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>
static unsigned char arena[65536] __attribute__((aligned(16)));
static uint64_t cursor=32, highwater=32;
void *topal_kernel_numeric_allocate_v1(uint64_t size) {
  if(size>65536-7) _exit(91);
  size=(size+7)&~UINT64_C(7);
  if(size>65536-cursor) _exit(91);
  void *p=arena+cursor; cursor+=size;
  if(cursor>highwater) highwater=cursor;
  return p;
}
__attribute__((noreturn)) void topal_kernel_numeric_abort_v1(void) {_exit(92);}
extern uint64_t topal_kernel_sleep_deadline_policy_v1(uint64_t,uint64_t,uint64_t,uint64_t);
static int check(uint64_t s,uint64_t n,uint64_t t,uint64_t w) {
  const __uint128_t maximum=INT64_MAX;
  __uint128_t d=s>=UINT64_C(9223372036)?maximum:(__uint128_t)s*1000000000+n;
  __uint128_t a=w?maximum:(__uint128_t)t*10+d;
  if(a>maximum) a=maximum;
  uint64_t expected=(uint64_t)((a+9)/10);
  memset(arena,0,sizeof(arena));cursor=32;
  uint64_t actual=topal_kernel_sleep_deadline_policy_v1(s,n,t,w);
  if(actual!=expected || cursor>41544) return 1;
  return 0;
}
int main(int argc,char **argv) {
  if(argc>1) {
    switch(argv[1][0]) {
      case 's':topal_kernel_sleep_deadline_policy_v1(UINT64_MAX,0,0,0);break;
      case 'n':topal_kernel_sleep_deadline_policy_v1(0,1000000000,0,0);break;
      case 'w':topal_kernel_sleep_deadline_policy_v1(0,0,0,2);break;
    }
    return 2;
  }
  const uint64_t seconds[]={0,1,9223372035,9223372036,INT64_MAX};
  const uint64_t nanos[]={0,1,9,10,11,999999999};
  const uint64_t ticks[]={0,1,300000000,922337203685477580,UINT64_MAX};
  for(unsigned i=0;i<5;i++) for(unsigned j=0;j<6;j++)
    for(unsigned k=0;k<5;k++) for(unsigned w=0;w<2;w++)
      if(check(seconds[i],nanos[j],ticks[k],w)) return 3;
  uint64_t seed=UINT64_C(0xabcdef1234567890);
  for(unsigned i=0;i<1000;i++) {
    seed=seed*UINT64_C(6364136223846793005)+1;
    uint64_t s=seed%UINT64_C(9223372037);
    seed=seed*UINT64_C(6364136223846793005)+1;
    uint64_t t=seed;
    seed=seed*UINT64_C(6364136223846793005)+1;
    if(check(s,seed%1000000000,t,i&1)) return 4;
  }
  printf("%llu\n",(unsigned long long)highwater);
  return 0;
}
"#,
    )
    .unwrap();
    let executable = directory.join("numeric-policy");
    let compile = std::process::Command::new("cc")
        .arg("-no-pie")
        .arg(&driver)
        .arg(directory.join("time-library.o"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let output = std::process::Command::new(&executable).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let highwater = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse::<u64>()
        .unwrap();
    assert!(highwater > 32 && highwater <= 41544);
    eprintln!(
        "numeric native maximum stack={} bytes; measured highwater={highwater}; conservative allocation bound={}",
        library.record.maximum_stack_bytes, library.record.allocation.maximum_bytes
    );
    for argument in ["s", "n", "w"] {
        assert_eq!(
            std::process::Command::new(&executable)
                .arg(argument)
                .status()
                .unwrap()
                .code(),
            Some(92)
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}
