/* Native ownership/controller component. The callbacks below own actual
 * guarded process domains, not a privileged kernel context or physical map. */
#define _GNU_SOURCE
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
#include <sys/mman.h>
#include <unistd.h>

typedef struct { unsigned char *mapping,*p;size_t bytes,length; } Guard;
typedef struct { uint64_t issuer,id,slot; } Token;
typedef struct { uint64_t issuer,id,start,count,alignment,provenance,pins,live; } Record;
typedef struct { uint64_t frame,loan,generation,active,poison; } History;
typedef struct {
    Guard context,table,snapshot,records,history,temporary,stack,facts,output,pool,tokens,details;
    uint64_t n,issuer;
} Owner;
enum { N=64, PAGE=4096, TOKENS=128 };
static Owner owners[2];
static jmp_buf fatal_return;
static uint64_t calls,zeros,fatals;
static int fail_zero;
static Token *current_token;
extern bool topal_frame_allocator_allocate_v1(void *,uint64_t,uint64_t,Token *);
extern bool topal_frame_allocator_release_v1(void *,Token *);
extern bool topal_frame_allocator_pin_v1(void *,Token *,bool);
extern bool topal_frame_allocator_resolve_v1(void *,Token *,void *);
extern uintptr_t topal_frame_allocator_active_v1,topal_x86_tail_boundary_active_v1;

static Guard guard(size_t size) {
    size_t page=(size_t)sysconf(_SC_PAGESIZE),area=(size+page-1)/page*page;
    unsigned char *p=mmap(NULL,area+2*page,PROT_NONE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    assert(p!=MAP_FAILED && !mprotect(p+page,area,PROT_READ|PROT_WRITE));
    return (Guard){p,p+page+area-size,area+2*page,size};
}
static int zero(const void *v,size_t n) {
    const unsigned char *p=v;while(n--) if(*p++) return 0;return 1;
}
static uint64_t *words(Guard g) { return (uint64_t *)g.p; }
static Token *token(Owner *o,size_t i) { assert(i<TOKENS);return (Token *)o->tokens.p+i; }
static Record *records(Owner *o) { return (Record *)o->records.p; }
static uint64_t *cell(Owner *o,uint64_t index) { assert(index<o->n);return words(o->table)+16+3*index; }
static Owner *lookup(void *ctx) {
    for(size_t i=0;i<2;i++) if(ctx==owners[i].context.p) return &owners[i];
    return NULL;
}
bool topal_frame_kernel_admit_v1(void *ctx,uint64_t operation,uint64_t count,uint64_t alignment) {
    Owner *o=lookup(ctx);if(!o) return false;
    uint64_t *c=words(o->context);History *h=(History *)o->history.p;
    if(c[0]!=1 || c[1]!=o->issuer || !c[2] || c[3]!=o->n || c[12]!=PAGE) return false;
    void *pointers[]={o->table.p,o->snapshot.p,o->records.p,o->history.p,o->temporary.p,
                     o->stack.p,o->facts.p,o->output.p};
    for(size_t i=0;i<8;i++) if(c[4+i]!=(uintptr_t)pointers[i]) return false;
    if(words(o->table)[0]!=o->n || h->active || h->poison ||
       topal_frame_allocator_active_v1 || topal_x86_tail_boundary_active_v1) return false;
    if(!zero(o->temporary.p,65528) || !zero(o->stack.p,PAGE) ||
       !zero(o->facts.p,24) || !zero(o->output.p,56)) return false;
    if(operation && (!count || !alignment || (alignment&(alignment-1)))) return false;
    if(operation) {
        __uint128_t work=2*o->n+4;
        if(h->generation==UINT64_MAX || (__uint128_t)h->frame+work*FRAME_STEP_BOUND+2>UINT64_MAX ||
           (__uint128_t)h->loan+work*LOAN_STEP_BOUND+2>UINT64_MAX) return false;
    }
    return true;
}
bool topal_frame_kernel_token_domain_v1(void *ctx,void *ptr,bool write) {
    (void)write;Owner *o=lookup(ctx);if(!o) return false;
    uintptr_t p=(uintptr_t)ptr,start=(uintptr_t)o->tokens.p;
    return p>=start && p-start<=o->tokens.length-sizeof(Token) && (p-start)%sizeof(Token)==0;
}
bool topal_frame_kernel_facts_domain_v1(void *ctx,void *ptr,uint64_t bytes) {
    Owner *o=lookup(ctx);return o && ptr==o->details.p && bytes==32;
}
bool topal_frame_kernel_zero_v1(void *ctx,uint64_t physical,uint64_t count) {
    Owner *o=lookup(ctx);assert(o);
    uint64_t *c=words(o->context),start=c[13],id=c[15];
    assert(c[0]==2 && c[14]==count && id==c[2] && count && count<=o->n-start);
    assert(current_token && zero(current_token,sizeof(*current_token)));
    for(uint64_t i=0;i<count;i++) {
        uint64_t *p=cell(o,start+i);
        assert(p[0]==id && p[1] && p[2]==physical+i);
        assert(physical+i<N+8);
        memset(o->pool.p+(physical+i)*PAGE,0,PAGE);
        if(fail_zero) return false; /* actual partial physical write, no grant */
    }
    for(uint64_t i=0;i<o->n;i++) assert(!records(o)[i].live || records(o)[i].id!=id);
    zeros++;return true;
}
__attribute__((noreturn)) void topal_frame_kernel_fatal_v1(void *ctx) {
    Owner *o=lookup(ctx);assert(o);
    uint64_t *c=words(o->context);History *h=(History *)o->history.p;
    assert(c[0]==3 && h->poison && !h->active);
    assert(!topal_x86_tail_boundary_active_v1);
    assert(zero(o->temporary.p,65528) && zero(o->stack.p,PAGE));
    /* This models only the target callback's owned-domain cleanup. It makes no
     * claim about physical kernel caller-stack retirement or interrupt state. */
    memset(o->facts.p,0,24);memset(o->output.p,0,56);
    topal_frame_allocator_active_v1=0;fatals++;
    longjmp(fatal_return,1);
}
static void initialize(Owner *o,uint64_t issuer) {
    o->n=N;o->issuer=issuer;
    o->context=guard(128);o->table=guard(128+N*24);o->snapshot=guard(128+N*24);
    o->records=guard(N*sizeof(Record));o->history=guard(sizeof(History));
    o->temporary=guard(65536);o->stack=guard(PAGE);o->facts=guard(24);o->output=guard(56);
    o->pool=guard((N+8)*PAGE);o->tokens=guard(TOKENS*sizeof(Token));
    o->details=guard(32);
    memset(o->pool.p,0xa5,o->pool.length);memset(o->temporary.p+65528,0xd7,8);
    uint64_t *c=words(o->context);c[0]=1;c[1]=issuer;c[2]=1;c[3]=N;c[12]=PAGE;
    void *pointers[]={o->table.p,o->snapshot.p,o->records.p,o->history.p,o->temporary.p,
                     o->stack.p,o->facts.p,o->output.p};
    for(size_t i=0;i<8;i++) c[4+i]=(uintptr_t)pointers[i];
    words(o->table)[0]=N;
    for(uint64_t i=0;i<N;i++) {
        uint64_t *p=cell(o,i);p[0]=0;p[1]=(i/16)%2+1;p[2]=i+(i>=24 ? 8 : 0);
    }
    cell(o,7)[0]=UINT64_MAX;cell(o,7)[1]=0;
    cell(o,31)[0]=UINT64_MAX;cell(o,31)[1]=0;
}
static void retired(Owner *o) {
    assert(words(o->context)[0]==1 && zero(o->context.p+13*8,24));
    assert(zero(o->facts.p,24) && zero(o->output.p,56));
    assert(zero(o->temporary.p,65528) && zero(o->stack.p,PAGE));
    for(size_t i=65528;i<65536;i++) assert(o->temporary.p[i]==0xd7);
    assert(!topal_frame_allocator_active_v1 && !topal_x86_tail_boundary_active_v1);
}
static __attribute__((unused)) bool allocate(Owner *o,uint64_t count,uint64_t alignment,Token *t) {
    uint64_t before=words(o->context)[2];current_token=t;
    bool result=topal_frame_allocator_allocate_v1(o->context.p,count,alignment,t);calls++;
    retired(o);
    if(result) {
        assert(t->issuer==o->issuer && t->id==before && t->slot<o->n);
        Record *r=records(o)+t->slot;
        assert(r->live==1 && r->issuer==t->issuer && r->id==t->id && r->count==count && r->alignment==alignment && !r->pins);
        assert(r->count<=o->n-r->start);
        uint64_t physical=cell(o,r->start)[2];
        assert(topal_frame_allocator_resolve_v1(o->context.p,t,o->details.p));
        uint64_t expected[]={physical,count,alignment,r->provenance};
        assert(!memcmp(o->details.p,expected,sizeof(expected)));
        assert(!topal_frame_allocator_resolve_v1(o->context.p,t,o->records.p));
        assert(!topal_frame_allocator_resolve_v1(o->context.p,t,o->table.p));
        assert(physical%alignment==0);
        for(uint64_t i=0;i<count;i++) {
            uint64_t *p=cell(o,r->start+i);
            assert(p[0]==r->id && p[1]==r->provenance && p[2]==physical+i);
            assert(zero(o->pool.p+p[2]*PAGE,PAGE));
        }
        assert(words(o->context)[2]==before+1);
    } else assert(words(o->context)[2]==before);
    return result;
}
static __attribute__((unused)) void release(Owner *o,Token *t) {
    Token old=*t;Record prior=records(o)[old.slot];
    assert(topal_frame_allocator_release_v1(o->context.p,t));calls++;retired(o);
    assert(zero(t,sizeof(*t)) && zero(records(o)+old.slot,sizeof(Record)));
    for(uint64_t i=0;i<prior.count;i++) assert(!cell(o,prior.start+i)[0]);
}
static __attribute__((unused)) void refused_release(Owner *o,Token *t) {
    unsigned char table[128+N*24],registry[N*sizeof(Record)],context[128],value[24];
    History history=*(History *)o->history.p;
    memcpy(table,o->table.p,sizeof(table));memcpy(registry,o->records.p,sizeof(registry));
    memcpy(context,o->context.p,sizeof(context));memcpy(value,t,sizeof(value));
    assert(!topal_frame_allocator_release_v1(o->context.p,t));calls++;
    assert(!memcmp(table,o->table.p,sizeof(table)) && !memcmp(registry,o->records.p,sizeof(registry)));
    assert(!memcmp(context,o->context.p,sizeof(context)) && !memcmp(value,t,sizeof(value)));
    assert(!memcmp(&history,o->history.p,sizeof(history)));retired(o);
}
int main(void) {
    initialize(&owners[0],11);initialize(&owners[1],22);
    Owner *o=&owners[0];
#if CONTROLLER_MODE == 1
    current_token=token(o,0);
    if(!setjmp(fatal_return)) {
        (void)topal_frame_allocator_allocate_v1(o->context.p,2,1,current_token);
        abort();
    }
    assert(words(o->context)[0]==3 && words(o->context)[2]==1 && !zeros);
    assert(zero(token(o,0),sizeof(Token)) && !cell(o,0)[0]);
#elif CONTROLLER_MODE == 2
    current_token=token(o,0);
    if(!setjmp(fatal_return)) {
        (void)topal_frame_allocator_allocate_v1(o->context.p,2,1,current_token);
        abort();
    }
    assert(words(o->context)[0]==3 && words(o->context)[2]==1 && !zeros);
    assert(zero(token(o,0),sizeof(Token)) && cell(o,0)[0]==2 && cell(o,1)[0]==2);
#else
    Owner *other=&owners[1];
    assert(!allocate(o,0,1,token(o,0)) && !allocate(o,1,0,token(o,0)));
    assert(!allocate(o,1,3,token(o,0)));
    assert(allocate(o,3,4,token(o,0)) && allocate(o,5,8,token(o,1)));
    *token(o,2)=*token(o,0); /* retained stale-byte fixture, not source duplication */
    assert(topal_frame_allocator_pin_v1(o->context.p,token(o,0),true));
    refused_release(o,token(o,0));
    assert(topal_frame_allocator_pin_v1(o->context.p,token(o,0),false));
    assert(!topal_frame_allocator_pin_v1(o->context.p,token(o,0),false));
    Record *pinned=records(o)+token(o,0)->slot;
    pinned->pins=UINT64_MAX;
    assert(!topal_frame_allocator_pin_v1(o->context.p,token(o,0),true) && pinned->pins==UINT64_MAX);
    pinned->pins=0;
    uint64_t old=token(o,0)->id,start=records(o)[token(o,0)->slot].start;
    release(o,token(o,0));
    memset(o->details.p,0x7b,32);
    assert(!topal_frame_allocator_resolve_v1(o->context.p,token(o,2),o->details.p));
    for(size_t i=0;i<32;i++) assert(o->details.p[i]==0x7b);
    refused_release(o,token(o,2));
    memset(o->pool.p+cell(o,start)[2]*PAGE,0x7b,3*PAGE);
    assert(allocate(o,3,4,token(o,0)) && token(o,0)->id!=old);
    assert(records(o)[token(o,0)->slot].start==start);
    refused_release(o,token(o,2));
    assert(allocate(other,1,1,token(other,0)));
    *token(other,1)=*token(o,0);refused_release(other,token(other,1));
    assert(!topal_frame_allocator_release_v1(other->context.p,token(o,0)));
    assert(!topal_frame_allocator_allocate_v1(NULL,1,1,token(o,3)));
    assert(!topal_frame_allocator_allocate_v1(o->context.p,1,1,NULL));
    uint64_t issuer=words(o->context)[2];words(o->context)[2]=UINT64_MAX;
    assert(!allocate(o,1,1,token(o,3)));words(o->context)[2]=issuer;
    for(size_t i=3;i<60;i++) (void)allocate(o,(i%5)+1,UINT64_C(1)<<(i%4),token(o,i));
    unsigned char prior[N*24];memcpy(prior,o->table.p+128,sizeof(prior));
    assert(!allocate(o,N+1,1,token(o,100)) && !memcmp(prior,o->table.p+128,sizeof(prior)));
    for(size_t i=0;i<60;i++) if(i!=2 && token(o,i)->id) release(o,token(o,i));
    release(other,token(other,0));
    assert(cell(o,7)[0]==UINT64_MAX && cell(o,31)[0]==UINT64_MAX);
    /* Zero failure retains source cell writes and pending issuance, no token. */
    fail_zero=1;current_token=token(o,100);
    uint64_t next=words(o->context)[2];
    if(!setjmp(fatal_return)) {
        (void)topal_frame_allocator_allocate_v1(o->context.p,2,1,current_token);
        abort();
    }
    assert(words(o->context)[0]==3 && words(o->context)[2]==next);
    assert(words(o->context)[14]==2 && words(o->context)[15]==next);
    assert(cell(o,0)[0]==next && cell(o,1)[0]==next);
    assert(zero(token(o,100),sizeof(Token)) && zero(o->pool.p,PAGE));
    unsigned char context[128];memcpy(context,o->context.p,128);
    assert(!topal_frame_allocator_allocate_v1(o->context.p,1,1,token(o,101)));
    assert(!memcmp(context,o->context.p,128));
#endif
    printf("FRAME_CONTROLLER_PASS mode=%d calls=%llu zeroed=%llu fatal=%llu\n",CONTROLLER_MODE,
        (unsigned long long)calls,(unsigned long long)zeros,(unsigned long long)fatals);
    return 0;
}
