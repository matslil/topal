/* TOPAL-SYSTEMS-TAIL-ADMISSION-001. Actual provider initializes context.
 * Guarded owned test facts do not establish privileged kernel authority. */
#define _GNU_SOURCE
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <setjmp.h>
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
static int zero(const void *v,size_t bytes) { const unsigned char *p=v; while(bytes--) if(*p++) return 0; return 1; }
extern uint64_t topal_tail_owner_invoke_v1(void *,uint64_t,const uint64_t *,History *,void *,void *,Output *);
extern uintptr_t topal_x86_tail_boundary_active_v1;
static Guard owner,history,temporary,stack,output,input;
static uint64_t admitted,refused;
static jmp_buf fatal;
static int fatal_seen;
__attribute__((noreturn)) void topal_tail_boundary_fatal_v1(void) {
    History *h=(History *)history.p;
    assert(h->active==0 && h->poisoned==1 && h->generation==1 && h->frame==4 && h->loan==4);
    assert(zero(temporary.p,65528) && zero(stack.p,4096) && zero(output.p,56));
    assert(!topal_x86_tail_boundary_active_v1);
    for(size_t i=0;i<owner.length;i++) assert(owner.p[i] == (i==5 ? 0 : 0x7b));
    fatal_seen=1; longjmp(fatal,1);
}
static uint64_t call(uint64_t remaining,uint64_t offset,uint64_t status) {
    assert(!mprotect(input.p,input.length,PROT_READ|PROT_WRITE));
    uint64_t words[]={remaining,offset,status};
    memcpy(input.p,words,sizeof(words));
    assert(!mprotect(input.p,input.length,PROT_READ));
    uint64_t result=topal_tail_owner_invoke_v1(owner.p,owner.length,(const uint64_t *)input.p,
      (History *)history.p,temporary.p,stack.p,(Output *)output.p);
    assert(!memcmp(input.p,words,sizeof(words)));
    return result;
}
#ifndef OWNER_FATAL_FIXTURE
static void accepted(uint64_t n,uint64_t offset,uint64_t status) {
    memset(owner.p,0x7b,owner.length);
    memset(output.p,0,56);
    History before=*(History *)history.p;
    assert(call(n,offset,status)==0);
    History *h=(History *)history.p;
    uint64_t first_frame=before.frame ? before.frame : 2;
    uint64_t first_loan=before.loan ? before.loan : 2;
    uint64_t available=offset<owner.length ? owner.length-offset : 0;
    uint64_t stores=status==1 ? (n<available ? n : available) : 0;
    uint64_t store_calls=status==1 ? (n<=available ? n : available+1) : 0;
    assert(h->frame==first_frame+2*n+store_calls && h->loan==first_loan+2*n+store_calls);
    assert(h->generation==before.generation+1 && !h->active && !h->poisoned);
    Output *o=(Output *)output.p; uint64_t yes; memcpy(&yes,o->value,8);
    assert(o->kind==3 && yes==(status==1 && n<=available) && zero(o->value+8,40));
    assert(zero(temporary.p,65528) && zero(stack.p,4096) && !topal_x86_tail_boundary_active_v1);
    for(size_t i=0;i<owner.length;i++) {
        unsigned char expected=status==1 && i>=offset && i-offset<stores ? 0 : 0x7b;
        assert(owner.p[i]==expected);
    }
    for(size_t i=65528;i<65536;i++) assert(temporary.p[i]==0xd7);
    ++admitted;
}
static void denied(uint64_t n) {
    unsigned char t[65528],s[4096],o[56],bytes[70032]; History h=*(History *)history.p;
    memcpy(t,temporary.p,sizeof(t)); memcpy(s,stack.p,sizeof(s)); memcpy(o,output.p,56);
    memcpy(bytes,owner.p,owner.length);
    uintptr_t cell=topal_x86_tail_boundary_active_v1;
    assert(call(n,0,1)==2);
    assert(!memcmp(t,temporary.p,sizeof(t)) && !memcmp(s,stack.p,sizeof(s)));
    assert(!memcmp(o,output.p,56) && !memcmp(&h,history.p,sizeof(h)));
    assert(!memcmp(bytes,owner.p,owner.length) && cell==topal_x86_tail_boundary_active_v1);
    for(size_t i=65528;i<65536;i++) assert(temporary.p[i]==0xd7);
    ++refused;
}
static void geometry_denials(void) {
    void *original[]={owner.p,input.p,history.p,temporary.p,stack.p,output.p};
    unsigned char before[65528]; memcpy(before,temporary.p,sizeof(before));
    assert(!mprotect(input.p,input.length,PROT_READ));
    for(unsigned a=0;a<6;a++) for(unsigned b=0;b<7;b++) {
        if(a==b) continue;
        void *args[6]; memcpy(args,original,sizeof(args));
        args[a]=b==6 ? (void *)&topal_x86_tail_boundary_active_v1 : original[b];
        assert(topal_tail_owner_invoke_v1(args[0],owner.length,args[1],args[2],args[3],args[4],args[5])==2);
        assert(!memcmp(before,temporary.p,sizeof(before)) && zero(history.p,40));
        assert(zero(output.p,56) && zero(stack.p,4096) && !topal_x86_tail_boundary_active_v1);
        assert(zero(owner.p,owner.length) && zero(input.p,input.length));
        for(size_t i=65528;i<65536;i++) assert(temporary.p[i]==0xd7);
        ++refused;
    }
    for(unsigned a=0;a<6;a++) {
        void *args[6]; memcpy(args,original,sizeof(args)); args[a]=NULL;
        assert(topal_tail_owner_invoke_v1(args[0],owner.length,args[1],args[2],args[3],args[4],args[5])==2);
        memcpy(args,original,sizeof(args)); args[a]=(void *)(UINTPTR_MAX-7);
        assert(topal_tail_owner_invoke_v1(args[0],owner.length,args[1],args[2],args[3],args[4],args[5])==2);
        assert(!memcmp(before,temporary.p,sizeof(before)) && zero(history.p,40));
        assert(zero(output.p,56) && zero(stack.p,4096) && !topal_x86_tail_boundary_active_v1);
        assert(zero(owner.p,owner.length) && zero(input.p,input.length));
        for(size_t i=65528;i<65536;i++) assert(temporary.p[i]==0xd7);
        refused+=2;
    }
    assert(topal_tail_owner_invoke_v1(owner.p,0,(const uint64_t *)input.p,(History *)history.p,temporary.p,stack.p,(Output *)output.p)==2);
    assert(!memcmp(before,temporary.p,sizeof(before)) && zero(history.p,40));
    assert(zero(output.p,56) && zero(stack.p,4096) && zero(owner.p,owner.length) && zero(input.p,input.length));
    ++refused;
}
#endif
int main(void) {
    owner=guard(70032); history=guard(40); temporary=guard(65536); stack=guard(4096);
    output=guard(56); input=guard((size_t)sysconf(_SC_PAGESIZE));
    memset(temporary.p+65528,0xd7,8);
#ifdef OWNER_FATAL_FIXTURE
    memset(owner.p,0x7b,owner.length);
    if(!setjmp(fatal)) { call(4,5,1); assert(0); }
    assert(fatal_seen);
#else
    geometry_denials();
    for(uint64_t n=0;n<10;n++) for(uint64_t state=0;state<3;state++)
        for(unsigned position=0;position<3;position++)
            accepted(n,position==0 ? 5 : position==1 ? owner.length-2 : UINT64_MAX,state);
    accepted(70000,11,1);
    memset(output.p,0,56);
    for(uint64_t frame=0;frame<17;frame++) for(uint64_t loan=0;loan<17;loan++)
        for(uint64_t measure=0;measure<9;measure++) {
            *(History *)history.p=(History){UINT64_MAX-frame,UINT64_MAX-loan,9,0,0};
            if(3*(measure+1)<=frame && 3*(measure+1)<=loan) accepted(measure,0,1);
            else denied(measure);
            memset(output.p,0,56);
        }
    *(History *)history.p=(History){2,2,UINT64_MAX,0,0}; denied(0);
    *(History *)history.p=(History){0,2,0,0,0}; denied(0);
    *(History *)history.p=(History){2,2,0,0,0}; denied(0);
    *(History *)history.p=(History){1,2,1,0,0}; denied(0);
    *(History *)history.p=(History){2,1,1,0,0}; denied(0);
    *(History *)history.p=(History){2,2,1,1,0}; denied(0);
    *(History *)history.p=(History){2,2,1,0,1}; denied(0);
    memset(history.p,0,40); denied(UINT64_MAX);
    temporary.p[64]=1; denied(0); temporary.p[64]=0;
    stack.p[0]=1; denied(0); stack.p[0]=0;
    output.p[0]=1; denied(0); output.p[0]=0;
    topal_x86_tail_boundary_active_v1=UINTPTR_MAX; denied(0);
    topal_x86_tail_boundary_active_v1=0;
#endif
    printf("TAIL_OWNER_PASS admitted=%llu refused=%llu fatal=%d\n",
      (unsigned long long)admitted,(unsigned long long)refused,fatal_seen);
    return 0;
}
