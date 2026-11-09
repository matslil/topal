/* TK-KERNEL-FRAME-POLICY-001: ordinary source and complete metadata effects.
 * This process component grants no physical-frame or kernel caller authority. */
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
typedef struct { uint64_t index,count,identity,provenance; } Extent;
enum { MAX_CELLS=3000, OWNER_BYTES=128+24*MAX_CELLS };
static Guard owner,history,temporary,stack,output,input;
static unsigned char expected[OWNER_BYTES];
static uint64_t cases,allocations,releases,exhaustions,protocol_failures;
extern uint64_t topal_tail_owner_invoke_v1(void *,uint64_t,const uint64_t *,History *,void *,void *,Output *);
extern uintptr_t topal_x86_tail_boundary_active_v1;
__attribute__((noreturn)) void topal_tail_boundary_fatal_v1(void) { abort(); }

static Guard guard(size_t size) {
    size_t page=(size_t)sysconf(_SC_PAGESIZE),area=(size+page-1)/page*page;
    unsigned char *p=mmap(NULL,area+2*page,PROT_NONE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    assert(p!=MAP_FAILED && !mprotect(p+page,area,PROT_READ|PROT_WRITE));
    return (Guard){p,p+page+area-size,area+2*page,size};
}
static int zero(const void *v,size_t bytes) {
    const unsigned char *p=v;
    while(bytes--) if(*p++) return 0;
    return 1;
}
static uint64_t word(const unsigned char *p,size_t offset) {
    uint64_t value=0;
    for(size_t i=0;i<8;i++) value|=(uint64_t)p[offset+i]<<(8*i);
    return value;
}
static void put(unsigned char *p,size_t offset,uint64_t value) {
    for(size_t i=0;i<8;i++) p[offset+i]=(unsigned char)(value>>(8*i));
}
static size_t cell(uint64_t index) { assert(index<MAX_CELLS); return 128+(size_t)index*24; }

/* The oracle checks every possible start and its entire candidate interval.
 * It does not implement the source's incremental run/phase state machine. */
static uint64_t oracle_allocate(uint64_t n,uint64_t count,uint64_t alignment,uint64_t identity) {
    for(uint64_t i=0;i<n;i++) {
        uint64_t physical=word(expected,cell(i)+16),provenance=word(expected,cell(i)+8);
        if(!provenance || physical%alignment || count>n-i) continue;
        int valid=1;
        for(uint64_t j=0;j<count;j++) {
            size_t base=cell(i+j);
            if(word(expected,base) || word(expected,base+8)!=provenance ||
               word(expected,base+16)!=physical+j) { valid=0; break; }
        }
        if(!valid) continue;
        for(uint64_t j=0;j<count;j++) put(expected,cell(i+j),identity);
        return i+1;
    }
    return 0;
}
static uint64_t oracle_release(uint64_t n,Extent e) {
    if(!e.count || e.index>=n || e.count>n-e.index || !e.provenance) return UINT64_MAX;
    uint64_t physical=word(expected,cell(e.index)+16);
    for(uint64_t j=0;j<e.count;j++) {
        size_t base=cell(e.index+j);
        if(word(expected,base)!=e.identity || word(expected,base+8)!=e.provenance ||
           word(expected,base+16)!=physical+j) return UINT64_MAX;
    }
    for(uint64_t j=0;j<e.count;j++) put(expected,cell(e.index+j),0);
    return e.index+1;
}
static void prepare(uint64_t n,uint64_t count,uint64_t alignment,uint64_t identity,
                    uint64_t phase,uint64_t start,uint64_t provenance) {
    assert(n<=MAX_CELLS && alignment);
    for(size_t i=0;i<96;i+=8) put(owner.p,i,0);
    put(owner.p,0,n);put(owner.p,8,count);put(owner.p,16,alignment-1);
    put(owner.p,24,identity);put(owner.p,32,provenance);put(owner.p,40,phase);
    put(owner.p,48,start);put(owner.p,56,start);
    memcpy(expected,owner.p,owner.length);
}
static void invoke(uint64_t budget,uint64_t result) {
    assert(!mprotect(input.p,input.length,PROT_READ|PROT_WRITE));
    const uint64_t facts[]={budget,0,0};
    memcpy(input.p,facts,sizeof(facts));
    assert(!mprotect(input.p,input.length,PROT_READ));
    History before=*(History *)history.p;
    assert(topal_tail_owner_invoke_v1(owner.p,owner.length,(const uint64_t *)input.p,
           (History *)history.p,temporary.p,stack.p,(Output *)output.p)==0);
    assert(!memcmp(input.p,facts,sizeof(facts)));
    History *h=(History *)history.p;
    assert(h->generation==before.generation+1 && !h->active && !h->poisoned);
    assert(h->frame>=before.frame && h->loan>=before.loan && h->frame>=2 && h->loan>=2);
    Output *o=(Output *)output.p;
    uint64_t sign=word(o->value,0),length=word(o->value,8);
    uint64_t limbs=result ? (result>UINT32_MAX ? 2 : 1) : 0;
    if(o->kind!=2 || sign || length!=limbs || word(o->value,16)!=result)
        fprintf(stderr,"case=%llu wanted=%llu actual=%llu kind=%llu sign=%llu length=%llu phase=%llu cursor=%llu processed=%llu result=%llu\n",
                (unsigned long long)cases,(unsigned long long)result,
                (unsigned long long)word(o->value,16),(unsigned long long)o->kind,
                (unsigned long long)sign,(unsigned long long)length,
                (unsigned long long)word(owner.p,40),(unsigned long long)word(owner.p,48),
                (unsigned long long)word(owner.p,64),(unsigned long long)word(owner.p,88));
    assert(o->kind==2 && !sign && length==limbs);
    assert(word(o->value,16)==result && zero(o->value+24,24));
    /* Only the explicit ephemeral control prefix may differ from the oracle. */
    memcpy(expected+40,owner.p+40,56);
    assert(!memcmp(owner.p,expected,owner.length));
    if(result!=UINT64_MAX) {
        assert(word(owner.p,40)==4 && word(owner.p,88)==result);
    } else protocol_failures++;
    assert(zero(temporary.p,65528) && zero(stack.p,4096) && !topal_x86_tail_boundary_active_v1);
    for(size_t i=65528;i<65536;i++) assert(temporary.p[i]==0xd7);
    memset(output.p,0,56);cases++;
}
static Extent allocate(uint64_t n,uint64_t count,uint64_t alignment,uint64_t identity) {
    prepare(n,count,alignment,identity,0,0,0);
    uint64_t found=oracle_allocate(n,count,alignment,identity);
    invoke(2*n+3,found);
    if(!found) { exhaustions++; return (Extent){0,0,0,0}; }
    allocations++;
    return (Extent){found-1,count,identity,word(owner.p,cell(found-1)+8)};
}
static uint64_t release(uint64_t n,Extent e) {
    prepare(n,e.count,1,e.identity,2,e.index,e.provenance);
    uint64_t result=oracle_release(n,e);
    invoke(2*n+3,result);
    if(result!=UINT64_MAX) releases++;
    return result;
}
static void initialize(uint64_t n,int fragmented) {
    memset(owner.p,0x6d,owner.length);
    for(size_t i=0;i<128;i++) owner.p[i]=0;
    for(uint64_t i=0;i<n;i++) {
        size_t base=cell(i);
        put(owner.p,base,0);
        put(owner.p,base+8,fragmented ? (i/16)%2+1 : 1);
        put(owner.p,base+16,4096+i+(fragmented && i>=24 ? 17 : 0));
    }
    if(fragmented) {
        put(owner.p,cell(7),UINT64_MAX);put(owner.p,cell(7)+8,0);
        put(owner.p,cell(31),UINT64_MAX);put(owner.p,cell(31)+8,0);
        put(owner.p,cell(11),1000);
    }
}
int main(void) {
    owner=guard(OWNER_BYTES);history=guard(40);temporary=guard(65536);
    stack=guard(4096);output=guard(56);input=guard((size_t)sysconf(_SC_PAGESIZE));
    memset(temporary.p+65528,0xd7,8);
    initialize(64,1);
    Extent a=allocate(64,3,4,7),b=allocate(64,5,8,8);
    assert(a.count && b.count && a.index!=b.index);
    assert(allocate(64,65,1,9).count==0);
    assert(release(64,a)==a.index+1);
    Extent reuse=allocate(64,3,4,9);
    assert(reuse.index==a.index);
    assert(release(64,b)==b.index+1 && release(64,reuse)==reuse.index+1);
    /* Repeated independent live extents, fragmentation, exhaustion and reuse. */
    Extent live[128];size_t used=0;uint64_t random=17,next_id=10;
    for(uint64_t i=0;i<96;i++) {
        random=random*UINT64_C(6364136223846793005)+1;
        if(i%3==0 && used) {
            size_t victim=(size_t)(random%used);
            assert(release(64,live[victim])==live[victim].index+1);
            live[victim]=live[--used];
        } else {
            uint64_t count=(random>>8)%6+1,alignment=UINT64_C(1)<<((random>>16)%5);
            Extent e=allocate(64,count,alignment,next_id);
            if(e.count) { assert(used<128); live[used++]=e; next_id++; }
        }
    }
    while(used) { Extent e=live[--used];assert(release(64,e)==e.index+1); }
    /* Separate malformed release scenarios: no cell may be cleared. */
    initialize(16,0);a=allocate(16,3,1,400);
    Extent wrong=a;wrong.identity++;assert(release(16,wrong)==UINT64_MAX);
    initialize(16,0);a=allocate(16,3,1,401);
    wrong=a;wrong.provenance++;assert(release(16,wrong)==UINT64_MAX);
    initialize(16,0);a=allocate(16,3,1,406);
    wrong=a;wrong.provenance=0;assert(release(16,wrong)==UINT64_MAX);
    initialize(16,0);a=allocate(16,3,1,402);
    put(owner.p,cell(a.index+2),403);assert(release(16,a)==UINT64_MAX);
    initialize(16,0);a=allocate(16,3,1,404);
    put(owner.p,cell(a.index+2)+16,99999);assert(release(16,a)==UINT64_MAX);
    initialize(16,0);a=allocate(16,3,1,405);
    assert(release(16,a)==a.index+1);assert(release(16,a)==UINT64_MAX);
    /* Selection/commit and release beyond the legacy 65536-byte region size. */
    initialize(MAX_CELLS,0);
    for(uint64_t i=0;i<MAX_CELLS-4;i++) put(owner.p,cell(i),900);
    a=allocate(MAX_CELLS,4,1,901);assert(a.index==MAX_CELLS-4);
    assert(release(MAX_CELLS,a)==a.index+1);
    /* A first-cell result retires without burning the full table countdown.
     * The bound comes from the actual compiled source, including all helpers. */
    initialize(MAX_CELLS,0);
    uint64_t before=((History *)history.p)->frame;
    a=allocate(MAX_CELLS,1,1,902);assert(a.index==0);
    assert(((History *)history.p)->frame-before<=5*FRAME_STEP_BOUND+2);
    before=((History *)history.p)->frame;
    assert(release(MAX_CELLS,a)==1);
    assert(((History *)history.p)->frame-before<=5*FRAME_STEP_BOUND+2);
    /* Empty topology and invalid request refuse without allocation-cell writes. */
    initialize(0,0);assert(allocate(0,1,1,500).count==0);
    initialize(16,0);prepare(16,0,1,501,0,0,0);invoke(35,UINT64_MAX);
    initialize(16,0);prepare(16,1,1,0,0,0,0);invoke(35,UINT64_MAX);
    initialize(16,0);prepare(16,1,1,UINT64_MAX,0,0,0);invoke(35,UINT64_MAX);
    initialize(16,0);prepare(16,1,1,502,9,0,0);invoke(35,UINT64_MAX);
    /* Countdown failure after the first persistent commit must retain it.
     * A real owning caller must fail fatally; this is the final invocation. */
    initialize(16,0);prepare(16,2,1,503,0,0,0);
    put(expected,cell(0),503);invoke(3,UINT64_MAX);
    assert(word(owner.p,40)==1 && word(owner.p,cell(0))==503 && !word(owner.p,cell(1)));
    printf("FRAME_POLICY_PASS cases=%llu allocated=%llu released=%llu exhausted=%llu protocol=%llu\n",
           (unsigned long long)cases,(unsigned long long)allocations,(unsigned long long)releases,
           (unsigned long long)exhaustions,(unsigned long long)protocol_failures);
    return 0;
}
