/* Independent whole-value protocol oracle; never linked into the kernel. */
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
typedef struct { uint64_t allocated,next; } transition;
extern transition topal_kernel_structured_process_identity_policy_v1(uint64_t,void *,uint64_t,uint64_t);
struct history { void *creations; _Bool wrapped; uint32_t order[2]; };
struct state { struct history history; void *next; uint32_t order[2]; };
struct number { uint64_t sign,length; uint32_t limbs[2]; };
static unsigned char cell[4096] __attribute__((aligned(4096)));
static unsigned char temporary[65536] __attribute__((aligned(16)));
static uint64_t cursor=32,highwater=32,checks,rejections;
static const uint64_t owner=UINT64_C(0x12345678);
static jmp_buf failure;
static int guarded,variant;
void *topal_kernel_numeric_allocate_v1(uint64_t count) {
    if(count>65529)exit(91);count=(count+7)&~UINT64_C(7);
    if(count>65536-cursor)exit(91);
    void *result=temporary+cursor;cursor+=count;
    if(cursor>highwater)highwater=cursor;return result;
}
__attribute__((noreturn)) void topal_kernel_numeric_abort_v1(void) {
    if(guarded)longjmp(failure,1);exit(92);
}
static uint64_t code(void) {return (uint64_t)(uintptr_t)&topal_kernel_structured_process_identity_policy_v1;}
static uint64_t get(unsigned offset) {uint64_t result;memcpy(&result,cell+offset,8);return result;}
static void put(unsigned offset,uint64_t value) {memcpy(cell+offset,&value,8);}
static unsigned bank(uint64_t index) {return 64+(unsigned)index*2016;}
static uint64_t numeric(const struct number *n) {return n->limbs[0]|((uint64_t)n->limbs[1]<<32);}
static void number(struct number *n,uint64_t value) {
    n->sign=0;n->length=value>UINT32_MAX?2:1;n->limbs[0]=(uint32_t)value;n->limbs[1]=(uint32_t)(value>>32);
}
static void initial(void) {memset(cell,0,sizeof(cell));put(0,owner);put(8,code());}
static void representation(unsigned char *bytes,unsigned offset,uint64_t next,uint64_t creations,int wrapped,int orders) {
    memset(bytes,0,2016);
    struct state *s=(void *)bytes;
    s->history.creations=cell+offset+sizeof(*s);
    s->history.wrapped=wrapped;
    s->history.order[0]=(orders&4)?1:0;s->history.order[1]=(orders&4)?0:1;
    s->next=cell+offset+sizeof(*s)+sizeof(struct number);
    s->order[0]=(orders&2)?0:1;s->order[1]=(orders&2)?1:0;
    number((void *)(bytes+sizeof(*s)),creations);
    number((void *)(bytes+sizeof(*s)+sizeof(struct number)),next);
}
static int graph(unsigned offset,uint64_t next,uint64_t creations,int wrapped,int orders) {
    unsigned char expected[2016];representation(expected,offset,next,creations,wrapped,orders);
    return memcmp(cell+offset,expected,sizeof(expected))==0;
}
/* These are controlled valid-domain histories, not billions of executed calls. */
static void seed(uint64_t next,uint64_t creations,int wrapped) {
    initial();put(16,1);put(32,1);representation(cell+64,64,next,creations,wrapped,2);
}
static int check(uint64_t input,uint64_t creations,int wrapped) {
    unsigned char before[4096];memcpy(before,cell,sizeof(cell));
    uint64_t old=get(24),epoch=get(32),allocated=input?input:2;
    uint64_t next=allocated==INT32_MAX?2:allocated+1;
    int next_wrapped=wrapped || allocated==INT32_MAX;
    memset(temporary,0,sizeof(temporary));cursor=32;
    transition actual=topal_kernel_structured_process_identity_policy_v1(input,cell,owner,code());
    if(actual.allocated!=allocated || actual.next!=next)return 1;
    if(memcmp(before,cell,40) || memcmp(before+bank(old),cell+bank(old),2016))return 2;
    if(get(40)!=1 || get(48)!=(old^1) || get(56)!=next)return 3;
    if(!graph(bank(old^1),next,creations+1,next_wrapped,variant))return 4;
    memset(temporary,0,sizeof(temporary));cursor=32;
    if(!graph(bank(old^1),next,creations+1,next_wrapped,variant))return 5;
    /* Independent owner publication after last temporary use. */
    put(16,1);put(24,old^1);put(32,epoch+1);put(40,0);put(48,0);put(56,0);
    memset(cell+bank(old),0,2016);
    for(unsigned i=0;i<2016;i++)if(cell[bank(old)+i])return 6;
    if(!graph(bank(old^1),next,creations+1,next_wrapped,variant))return 7;
    checks++;return 0;
}
static int rejected(uint64_t input,void *store,uint64_t expected_owner,uint64_t expected_code,int preallocation) {
    unsigned char before[4096];memcpy(before,cell,sizeof(cell));
    memset(temporary,0,sizeof(temporary));cursor=32;guarded=1;
    if(setjmp(failure)==0) {
        (void)topal_kernel_structured_process_identity_policy_v1(input,store,expected_owner,expected_code);
        guarded=0;return 1;
    }
    guarded=0;rejections++;
    return memcmp(before,cell,sizeof(cell)) || (preallocation && cursor!=32);
}
static int malformed(void) {
    for(unsigned i=0;i<3;i++) {
        const uint64_t invalid[]={1,UINT64_C(2147483648),UINT64_MAX};
        initial();if(rejected(invalid[i],cell,owner,code(),1))return 20+i;
    }
    initial();if(rejected(0,0,owner,code(),1))return 23;
    initial();if(rejected(0,cell+1,owner,code(),1))return 24;
    initial();if(rejected(0,(void *)(uintptr_t)(UINT64_MAX-4095),owner,code(),1))return 25;
    initial();if(rejected(0,cell,0,code(),1))return 26;
    initial();if(rejected(0,cell,owner,0,1))return 27;
    initial();if(rejected(0,cell,owner^1,code(),1))return 28;
    initial();if(rejected(0,cell,owner,code()^1,1))return 29;
    const unsigned offsets[]={0,8,16,24,32,40,48,56,64,2080};
    const uint64_t values[]={owner^1,0,2,2,UINT64_MAX,1,1,2,1,1};
    for(unsigned i=0;i<10;i++) {
        initial();put(offsets[i],values[i]);
        if(rejected(0,cell,owner,code(),1))return 30+i;
    }
    initial();put(32,1);if(rejected(0,cell,owner,code(),1))return 40;
    initial();put(24,1);if(rejected(0,cell,owner,code(),1))return 41;
    initial();if(rejected(2,cell,owner,code(),1))return 42;
    const unsigned graph_offsets[]={
        0,24,40,48,56,48,64,72,80,72,32,36,12,16,8,20,88,2016
    };
    const uint64_t graph_values[]={
        0,0,1,0,0,3,1,0,UINT64_C(0x100000002),2,UINT32_MAX,0,2,0,2,1,1,1
    };
    for(unsigned i=0;i<sizeof(graph_offsets)/sizeof(graph_offsets[0]);i++) {
        seed(2,2,0);
        if(graph_offsets[i]==20)cell[64+20]=1;
        else put(64+graph_offsets[i],graph_values[i]);
        if(rejected(2,cell,owner,code(),1))return 50+i;
    }
    /* Invalid one-/two-limb normalization, count exhaustion and cached mismatch. */
    seed(2,UINT64_C(1)<<32,0);put(64+48,1);if(rejected(2,cell,owner,code(),1))return 70;
    seed(2,2,0);put(64+48,2);if(rejected(2,cell,owner,code(),1))return 71;
    seed(2,UINT64_MAX,0);if(rejected(2,cell,owner,code(),1))return 72;
    seed(2,2,0);put(32,0);if(rejected(2,cell,owner,code(),1))return 73;
    seed(2,2,0);if(rejected(3,cell,owner,code(),1))return 74;
    seed(2,2,0);if(rejected(0,cell,owner,code(),1))return 75;
    initial();memset(temporary,0,sizeof(temporary));cursor=32;
    (void)topal_kernel_structured_process_identity_policy_v1(0,cell,owner,code());
    if(rejected(0,cell,owner,code(),1))return 76;
    return 0;
}
int main(int argc,char **argv) {
    if(argc>1 && !strcmp(argv[1],"invalid")) {seed(2,2,0);return rejected(2,cell,owner,code(),0);}
    variant=argc>1?atoi(argv[1]):0;
    if(variant<0 || variant>7 || sizeof(struct state)!=40 || sizeof(struct number)!=24)return 90;
    initial();uint64_t current=0,creations=0;int wrapped=0;
    for(unsigned i=0;i<65536;i++) {
        int status=check(current,creations,wrapped);if(status)return status;
        struct state *s=(void *)(cell+bank(get(24)));
        current=numeric(s->next);creations=numeric(s->history.creations);wrapped=s->history.wrapped;
    }
    const uint64_t counts[]={1,UINT32_MAX-1,UINT32_MAX,UINT64_C(1)<<32,UINT64_MAX-1};
    const uint64_t cursors[]={2,3,INT32_MAX-1,INT32_MAX};
    for(unsigned i=0;i<5;i++)for(unsigned j=0;j<4;j++)for(unsigned flag=0;flag<2;flag++) {
        seed(cursors[j],counts[i],flag);
        int status=check(cursors[j],counts[i],flag);if(status)return status;
        if(cursors[j]==INT32_MAX && counts[i]<UINT64_MAX-1) {
            status=check(2,counts[i]+1,1);if(status)return status;
        }
    }
    uint64_t random=UINT64_C(0x1234567890abcdef);
    for(unsigned i=0;i<1000;i++) {
        random=random*UINT64_C(6364136223846793005)+1;
        uint64_t next=2+random%(INT32_MAX-1);
        uint64_t count=1+random%(UINT64_MAX-1);int flag=random&1;
        seed(next,count,flag);int status=check(next,count,flag);if(status)return status;
    }
    int status=malformed();if(status)return status;
    printf("%llu %llu %llu\n",(unsigned long long)highwater,(unsigned long long)checks,(unsigned long long)rejections);
    return 0;
}
