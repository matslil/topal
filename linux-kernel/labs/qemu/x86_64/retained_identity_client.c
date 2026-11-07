/* Independent native protocol client; never linked into a kernel artifact. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>

typedef struct { uint64_t allocated, next; } transition;
extern transition topal_kernel_retained_process_identity_policy_v1(
    uint64_t, void *, uint64_t, uint64_t);
static unsigned char temporary[65536] __attribute__((aligned(16)));
static unsigned char cell[4096] __attribute__((aligned(4096)));
static uint64_t cursor=32,highwater=32;
static int guarded;
static jmp_buf failure;
static const uint64_t owner=UINT64_C(0x12345678);

void *topal_kernel_numeric_allocate_v1(uint64_t bytes) {
    if(bytes>65529)exit(91);bytes=(bytes+7)&~UINT64_C(7);
    if(bytes>65536-cursor)exit(91);
    void *value=temporary+cursor;cursor+=bytes;
    if(cursor>highwater)highwater=cursor;
    return value;
}
__attribute__((noreturn)) void topal_kernel_numeric_abort_v1(void) {
    if(guarded)longjmp(failure,1);
    exit(92);
}
static uint64_t code(void) {
    return (uint64_t)(uintptr_t)&topal_kernel_retained_process_identity_policy_v1;
}
static uint64_t get(unsigned offset) {
    uint64_t value;memcpy(&value,cell+offset,8);return value;
}
static void put(unsigned offset,uint64_t value) {memcpy(cell+offset,&value,8);}
static unsigned bank(uint64_t index) {return 64+(unsigned)index*2016;}
static int zero(unsigned offset,unsigned count) {
    for(unsigned i=0;i<count;i++)if(cell[offset+i])return 0;
    return 1;
}
static void initial(void) {
    memset(cell,0,sizeof(cell));put(0,owner);put(8,code());
}
/* Seed a legal published graph for boundary cases; no billions-of-lifetimes claim. */
static void seed(uint64_t next) {
    initial();put(16,1);put(32,1);
    put(64,(uint64_t)(uintptr_t)(cell+80));put(88,1);put(96,next);
}
static int graph(unsigned offset,uint64_t value) {
    return get(offset)==(uint64_t)(uintptr_t)(cell+offset+16)
        && get(offset+8)==0 && get(offset+16)==0
        && get(offset+24)==1 && get(offset+32)==value
        && zero(offset+40,2016-40);
}
static int check(uint64_t input) {
    unsigned char before[4096];memcpy(before,cell,sizeof(cell));
    uint64_t old=get(24),generation=get(32),allocated=input?input:2;
    uint64_t next=allocated==INT32_MAX?2:allocated+1;
    memset(temporary,0,sizeof(temporary));cursor=32;
    transition actual=topal_kernel_retained_process_identity_policy_v1(input,cell,owner,code());
    if(actual.allocated!=allocated || actual.next!=next || cursor>312)return 1;
    if(memcmp(before,cell,40) || memcmp(before+bank(old),cell+bank(old),2016))return 2;
    if(get(40)!=1 || get(48)!=(old^1) || get(56)!=next || !graph(bank(old^1),next))return 3;
    /* Reclaim invocation storage first; the prepared ordinary graph must survive. */
    memset(temporary,0,sizeof(temporary));cursor=32;
    if(!graph(bank(old^1),next))return 4;
    /* Independent owner model of the qualified publish/retire protocol. */
    put(16,1);put(24,old^1);put(32,generation+1);put(40,0);put(48,0);put(56,0);
    memset(cell+bank(old),0,2016);
    if(!zero(bank(old),2016) || !graph(bank(old^1),next))return 5;
    return 0;
}
static int rejected(uint64_t input,void *store,uint64_t expected_owner,uint64_t expected_code,int preallocation) {
    unsigned char before[4096];memcpy(before,cell,sizeof(cell));
    memset(temporary,0,sizeof(temporary));cursor=32;guarded=1;
    if(setjmp(failure)==0) {
        (void)topal_kernel_retained_process_identity_policy_v1(input,store,expected_owner,expected_code);
        guarded=0;return 1;
    }
    guarded=0;
    return memcmp(before,cell,sizeof(cell)) || (preallocation && cursor!=32);
}
static int malformed(void) {
    const uint64_t bad[]={1,UINT64_C(2147483648),UINT64_MAX};
    for(unsigned i=0;i<3;i++) {initial();if(rejected(bad[i],cell,owner,code(),1))return 10+i;}
    initial();if(rejected(0,0,owner,code(),1))return 13;
    initial();if(rejected(0,(void *)(uintptr_t)(UINT64_MAX-4095),owner,code(),1))return 14;
    initial();if(rejected(0,cell+1,owner,code(),1))return 15;
    initial();if(rejected(0,cell,0,code(),1))return 16;
    initial();if(rejected(0,cell,owner,0,1))return 17;
    const unsigned offsets[]={0,8,16,24,32,40,48,56,64,2080};
    const uint64_t values[]={owner^1,0,2,2,UINT64_MAX,1,1,2,1,1};
    for(unsigned i=0;i<10;i++) {
        initial();put(offsets[i],values[i]);
        if(rejected(0,cell,owner,code(),1))return 20+i;
    }
    initial();put(32,1);if(rejected(0,cell,owner,code(),1))return 30;
    initial();put(24,1);if(rejected(0,cell,owner,code(),1))return 31;
    initial();if(rejected(2,cell,owner,code(),1))return 32;
    const unsigned graph_offsets[]={64,72,80,88,96,104,2080};
    const uint64_t graph_values[]={0,1,1,2,UINT64_C(0x100000002),1,1};
    for(unsigned i=0;i<7;i++) {
        seed(2);put(graph_offsets[i],graph_values[i]);
        if(rejected(2,cell,owner,code(),1))return 40+i;
    }
    seed(2);put(32,0);if(rejected(2,cell,owner,code(),1))return 47;
    seed(2);if(rejected(3,cell,owner,code(),1))return 48;
    seed(2);if(rejected(0,cell,owner,code(),1))return 49;
    initial();if(rejected(0,cell,owner^1,code(),1))return 50;
    initial();if(rejected(0,cell,owner,code()^1,1))return 51;
    /* Preparing twice is stale use even when all numeric payloads are legal. */
    initial();memset(temporary,0,sizeof(temporary));cursor=32;
    (void)topal_kernel_retained_process_identity_policy_v1(0,cell,owner,code());
    if(rejected(0,cell,owner,code(),1))return 52;
    return 0;
}
int main(int argc,char **argv) {
    (void)argv;
    if(argc>1) {seed(2);return rejected(2,cell,owner,code(),0);}
    initial();uint64_t current=0;
    for(unsigned i=0;i<65536;i++) {
        int status=check(current);if(status)return status;
        current=get(bank(get(24))+32);
    }
    const uint64_t boundary[]={2,3,INT32_MAX-2,INT32_MAX-1,INT32_MAX};
    for(unsigned i=0;i<5;i++) {seed(boundary[i]);int status=check(boundary[i]);if(status)return status;}
    uint64_t random=UINT64_C(0x1234567890abcdef);
    for(unsigned i=0;i<1000;i++) {
        random=random*UINT64_C(6364136223846793005)+1;
        uint64_t value=2+random%(INT32_MAX-1);seed(value);
        int status=check(value);if(status)return status;
    }
    int status=malformed();if(status)return status;
    printf("%llu\n",(unsigned long long)highwater);return 0;
}
