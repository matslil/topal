/* TOPAL-SYSTEMS-TAIL-BOUNDS-001. Actual mapped owner extent observation;
 * this native process fixture establishes no privileged allocator authority. */
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
    size_t page=(size_t)sysconf(_SC_PAGESIZE), area=(size+page-1)/page*page;
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
int main(void) {
    const size_t sizes[]={1,65535,65536,65537,70032,131073,262144};
    Guard history=guard(40),temporary=guard(65536),stack=guard(4096),output=guard(56);
    Guard input=guard((size_t)sysconf(_SC_PAGESIZE));
    memset(temporary.p+65528,0xd7,8);
    uint64_t cases=0;
    for(size_t i=0;i<sizeof(sizes)/sizeof(sizes[0]);i++) {
        Guard owner=guard(sizes[i]);
        memset(owner.p,0x7b,owner.length);
        for(uint64_t steps=0;steps<4;steps++) for(uint64_t wrong=0;wrong<2;wrong++) {
            assert(!mprotect(input.p,input.length,PROT_READ|PROT_WRITE));
            const uint64_t facts[]={steps,owner.length+wrong,1};
            memcpy(input.p,facts,sizeof(facts));
            assert(!mprotect(input.p,input.length,PROT_READ));
            History before=*(History *)history.p;
            assert(topal_tail_owner_invoke_v1(owner.p,owner.length,(const uint64_t *)input.p,
                (History *)history.p,temporary.p,stack.p,(Output *)output.p)==0);
            assert(!memcmp(input.p,facts,sizeof(facts)));
            History *h=(History *)history.p;
            assert(h->generation==before.generation+1 && !h->active && !h->poisoned);
            assert(h->frame>before.frame && h->loan>before.loan);
            Output *o=(Output *)output.p;uint64_t yes;memcpy(&yes,o->value,8);
            assert(o->kind==3 && yes==!wrong && zero(o->value+8,40));
            assert(zero(temporary.p,65528) && zero(stack.p,4096) && !topal_x86_tail_boundary_active_v1);
            for(size_t n=0;n<owner.length;n++) assert(owner.p[n]==0x7b);
            for(size_t n=65528;n<65536;n++) assert(temporary.p[n]==0xd7);
            memset(output.p,0,56);
            cases++;
        }
        assert(!munmap(owner.mapping,owner.bytes));
    }
    printf("TAIL_COUNT_PASS cases=%llu sizes=7 max_extent=262144\n",(unsigned long long)cases);
    return 0;
}
