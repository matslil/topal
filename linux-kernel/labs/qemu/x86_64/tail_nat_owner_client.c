/* TOPAL-SYSTEMS-TAIL-NAT-OWNER-001. Guarded native component, no frame grant. */
#define _GNU_SOURCE
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
typedef struct { uint64_t frame,loan,generation,active,poisoned; } History;
typedef struct { unsigned char value[48]; uint64_t kind; } Output;
typedef struct { unsigned char *mapping,*p; size_t bytes,length; } Guard;
static Guard guard(size_t size) {
    size_t page=(size_t)sysconf(_SC_PAGESIZE),area=(size+page-1)/page*page;
    unsigned char *p=mmap(NULL,area+2*page,PROT_NONE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    assert(p!=MAP_FAILED && !mprotect(p+page,area,PROT_READ|PROT_WRITE));
    return (Guard){p,p+page+area-size,area+2*page,size};
}
static int zero(const void *v,size_t bytes) {
    const unsigned char *p=v; while(bytes--) if(*p++) return 0; return 1;
}
extern uint64_t topal_tail_owner_invoke_v1(void *,uint64_t,const uint64_t *,History *,void *,void *,Output *);
extern uintptr_t topal_x86_tail_boundary_active_v1;
__attribute__((noreturn)) void topal_tail_boundary_fatal_v1(void) { abort(); }
static Guard owner,history,temporary,stack,output,input;
static unsigned char snapshot[70032];
static uint64_t cases;
static void call(uint64_t remaining,uint64_t offset,const uint32_t expected[8]) {
    memcpy(snapshot,owner.p,owner.length);
    assert(!mprotect(input.p,input.length,PROT_READ|PROT_WRITE));
    const uint64_t facts[]={remaining,offset,0};
    memcpy(input.p,facts,sizeof(facts));
    assert(!mprotect(input.p,input.length,PROT_READ));
    History before=*(History *)history.p;
    assert(topal_tail_owner_invoke_v1(owner.p,owner.length,(const uint64_t *)input.p,
        (History *)history.p,temporary.p,stack.p,(Output *)output.p)==0);
    assert(!memcmp(input.p,facts,sizeof(facts)) && !memcmp(snapshot,owner.p,owner.length));
    History *h=(History *)history.p;
    assert(h->generation==before.generation+1 && !h->active && !h->poisoned);
    assert(h->frame>=2 && h->loan>=2 && h->frame>=before.frame && h->loan>=before.loan);
    Output *o=(Output *)output.p;
    uint64_t sign,length;memcpy(&sign,o->value,8);memcpy(&length,o->value+8,8);
    size_t used=8;while(used && !expected[used-1]) used--;
    assert(o->kind==2 && !sign && length==used);
    assert(!memcmp(o->value+16,expected,used*4));
    assert(zero(o->value+16+used*4,32-used*4));
    assert(zero(temporary.p,65528) && zero(stack.p,4096) && !topal_x86_tail_boundary_active_v1);
    for(size_t i=65528;i<65536;i++) assert(temporary.p[i]==0xd7);
    memset(output.p,0,56);cases++;
}
#if NAT_QUERY_MODE != 2
static void word(uint64_t n,uint64_t offset,uint64_t low,uint32_t carry) {
    const uint32_t expected[8]={(uint32_t)low,(uint32_t)(low>>32),carry,0,0,0,0,0};
    call(n,offset,expected);
}
#endif
int main(void) {
    owner=guard(70032);history=guard(40);temporary=guard(65536);stack=guard(4096);
    output=guard(56);input=guard((size_t)sysconf(_SC_PAGESIZE));
    memset(temporary.p+65528,0xd7,8);
#if NAT_QUERY_MODE == 0
    const uint64_t positions[]={0,1,65535,65536,70031,UINT64_MAX};
    for(size_t i=0;i<sizeof(positions)/sizeof(positions[0]);i++) {
        memset(owner.p,0x7b,owner.length);
        if(positions[i]!=UINT64_MAX) owner.p[positions[i]]=0;
        word(owner.length,0,positions[i]==UINT64_MAX ? 0 : owner.length-positions[i],0);
    }
    word(0,0,0,0);
    word(1,owner.length,UINT64_MAX,0);
#elif NAT_QUERY_MODE == 1
    memset(owner.p,0x7b,owner.length);
    const uint64_t offsets[]={0,UINT32_MAX,UINT64_MAX-1,UINT64_MAX};
    for(size_t i=0;i<sizeof(offsets)/sizeof(offsets[0]);i++) for(uint64_t n=0;n<5;n++) {
        uint64_t low=offsets[i]+n;
        word(n,offsets[i],low,low<offsets[i]);
    }
#else
    memset(owner.p,0x7b,owner.length);
    const uint32_t expected[8]={UINT32_MAX,UINT32_MAX,UINT32_MAX,UINT32_MAX,
        UINT32_MAX,UINT32_MAX,UINT32_MAX,UINT32_MAX};
    for(uint64_t n=1;n<=5;n++) call(n,UINT64_MAX,expected);
#endif
    printf("TAIL_NAT_OWNER_PASS mode=%d cases=%llu\n",NAT_QUERY_MODE,(unsigned long long)cases);
    return 0;
}
