/* Same private native entry emitters, guarded host component only. No authority. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#define CHECK(x) do { if (!(x)) {fprintf(stderr,"namespace-entries:%d:%s\n",__LINE__,#x);exit(1);} } while(0)
extern uint64_t topal_namespace_metadata_check_v1(const void *);
extern uint64_t topal_namespace_entry_lookup_v1(const void *, const void *, uint64_t, uint64_t *);
extern uint64_t topal_namespace_entry_free_v1(const void *);
extern void topal_namespace_entry_retire_v1(void *,uint64_t);
extern void topal_namespace_erase_di_v1(void *);
enum { OWNER=311296, BASE=0x43000, PAGE=4096 };
static uint8_t *owner,*metadata;
static uint64_t checks,lookups;
static void word(unsigned offset,uint64_t value) {memcpy(metadata+offset,&value,8);}
static void protect(int access) {CHECK(mprotect(metadata,PAGE,access)==0);}
static void seed(void) {
    protect(PROT_READ|PROT_WRITE);memset(metadata,0,PAGE);
    word(0,11);word(32,041777);
    for(unsigned i=0;i<4;i++) {
        word(256+128*i,1);word(264+128*i,7+i);word(272+128*i,27+i);
        word(280+128*i,0100640+i);word(288+128*i,100+i);
        word(296+128*i,200+i);word(304+128*i,300+i);word(320+128*i,1);
    }
}
static void name(unsigned slot,unsigned object,const void *bytes,unsigned length) {
    uint64_t n;memcpy(&n,metadata+312+128*object,8);
    word(312+128*object,n+1);word(320+128*object,0);
    word(64+16*slot,7+object);word(72+16*slot,length);
    memset(metadata+1536+slot*256,0,256);memcpy(metadata+1536+slot*256,bytes,length);
}
static void valid(int expected) {
    protect(PROT_READ);CHECK(topal_namespace_metadata_check_v1(metadata)==(uint64_t)expected);checks++;
}
static void lookup(const void *bytes,unsigned length,uint64_t id,uint64_t wanted_slot) {
    uint64_t slot=UINT64_MAX;
    protect(PROT_READ);CHECK(topal_namespace_entry_lookup_v1(owner,bytes,length,&slot)==id);
    CHECK(slot==wanted_slot);lookups++;
}
int main(void) {
    /* The readback must revisit the exact retired bytes, not the following
       area: put512 bytes at a page end with an inaccessible next page. */
    uint8_t *retirement=mmap(NULL,2*PAGE,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    CHECK(retirement!=MAP_FAILED && mprotect(retirement+PAGE,PAGE,PROT_NONE)==0);
    memset(retirement,0xa5,PAGE);
    topal_namespace_erase_di_v1(retirement+PAGE-512);
    for(unsigned i=0;i<PAGE;i++)CHECK(retirement[i]==(i<PAGE-512?0xa5:0));
    CHECK(munmap(retirement,2*PAGE)==0);
    uint8_t *map=mmap(NULL,OWNER+2*PAGE,PROT_NONE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    CHECK(map!=MAP_FAILED);owner=map+PAGE;metadata=owner+BASE;
    uint8_t *input=mmap(NULL,2*PAGE,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    CHECK(input!=MAP_FAILED && mprotect(input+PAGE,PAGE,PROT_NONE)==0);
#ifdef NS_ENTRIES_DEBUG
    seed();name(1,0,"first",5);valid(1);lookup("first",5,7,1);
    CHECK(topal_namespace_entry_free_v1(owner)==0);
    protect(PROT_READ|PROT_WRITE);word(0,6);valid(0);
    CHECK(munmap(map,OWNER+2*PAGE)==0 && munmap(input,2*PAGE)==0);
    puts("NAMESPACE_ENTRIES_DEBUG_PASS");return 0;
#endif
    for(unsigned rotation=0;rotation<4;rotation++)for(unsigned length=1;length<=255;length++) {
        seed();
        for(unsigned slot=0;slot<4;slot++) {
            uint8_t bytes[256];for(unsigned i=0;i<length;i++)bytes[i]=128+i%128;
            bytes[0]='A'+slot;name(slot,(slot+rotation)%4,bytes,length);
        }
        valid(1);CHECK(topal_namespace_entry_free_v1(owner)==4);
        for(unsigned slot=0;slot<4;slot++) {
            uint8_t *bytes=input+PAGE-length;
            for(unsigned i=0;i<length;i++)bytes[i]=128+i%128;
            bytes[0]='A'+slot;
            lookup(bytes,length,7+(slot+rotation)%4,slot);
        }
        lookup((void *)(uintptr_t)1,0,0,4);
        lookup((void *)(uintptr_t)1,256,0,4);
    }
    seed();valid(1);CHECK(topal_namespace_entry_free_v1(owner)==0);
    for(unsigned slot=0;slot<4;slot++) {
        protect(PROT_READ|PROT_WRITE);
        uint8_t bytes[7]={'a','l','i','a','s','-',(uint8_t)('A'+slot)};
        name(slot,0,bytes,7);
    }
    valid(1);CHECK(topal_namespace_entry_free_v1(owner)==4);
    for(unsigned slot=0;slot<4;slot++) {
        uint8_t bytes[7]={'a','l','i','a','s','-',(uint8_t)('A'+slot)};
        lookup(bytes,7,7,slot);
    }
    seed();name(0,2,"third",5);name(1,0,"first",5);name(3,2,"alias",5);
    valid(1);CHECK(topal_namespace_entry_free_v1(owner)==2);
    lookup("third",5,9,0);lookup("first",5,7,1);lookup("alias",5,9,3);lookup("nope",4,0,4);
    protect(PROT_READ|PROT_WRITE);
    /* Private admission/directory staging is not a persistent name/inode map. */
    memset(metadata+3072,0xa5,512);valid(1);
    protect(PROT_READ|PROT_WRITE);memset(metadata+3072,0,512);
    uint8_t saved[PAGE];memcpy(saved,metadata,PAGE);
    const struct {unsigned offset;uint64_t value;} bad[]={
        {0,6},{8,1},{32,0100644},{128,1},{768,1},{1280,1},{2560,1},{3584,1},
        {64,999},{72,0},{72,256},{80,10},{88,6},{1536,0},{1541,1},
        {256,2},{264,11},{272,UINT64_MAX},{280,040777},{312,0},{320,1},{328,1},
        {440,4},{448,2},{1024,999},{1040,1},{24,1},
    };
    for(unsigned i=0;i<sizeof(bad)/sizeof(bad[0]);i++) {
        protect(PROT_READ|PROT_WRITE);memcpy(metadata,saved,PAGE);word(bad[i].offset,bad[i].value);valid(0);
    }
    protect(PROT_READ|PROT_WRITE);memcpy(metadata,saved,PAGE);
    memcpy(metadata+2304,metadata+1536,256);valid(0); /* distinct entries duplicate raw name */
    protect(PROT_READ|PROT_WRITE);memcpy(metadata,saved,PAGE);word(0,UINT64_MAX);word(264,UINT64_MAX-1);word(80,UINT64_MAX-1);valid(1);
    lookup("first",5,UINT64_MAX-1,1);
    protect(PROT_READ|PROT_WRITE);memcpy(metadata,saved,PAGE);
    for(unsigned slot=0;slot<4;slot++) {
        protect(PROT_READ|PROT_WRITE);memcpy(metadata,saved,PAGE);
        word(64+slot*16,0);word(72+slot*16,0);memset(metadata+1536+slot*256,0,256);
        if(slot==0||slot==3)word(568,1); /* id9 links two -> one */
        if(slot==1)word(312,0);
        valid(1);CHECK(topal_namespace_entry_free_v1(owner)==(slot<2?slot:2));
    }
    for(unsigned object=0;object<4;object++) {
        protect(PROT_READ|PROT_WRITE);memcpy(metadata,saved,PAGE);
        uint8_t expected[PAGE];memcpy(expected,saved,PAGE);
        for(unsigned slot=0;slot<4;slot++) {
            uint64_t id;memcpy(&id,expected+64+slot*16,8);
            if(id==7+object) {
                memset(expected+64+slot*16,0,16);
                memset(expected+1536+slot*256,0,256);
            }
        }
        memset(expected+312+object*128,0,8);
        topal_namespace_entry_retire_v1(owner,object);
        CHECK(memcmp(metadata,expected,PAGE)==0);valid(1);
    }
    CHECK(munmap(map,OWNER+2*PAGE)==0 && munmap(input,2*PAGE)==0);
    printf("NAMESPACE_ENTRIES_COMPONENT_PASS checks=%llu lookups=%llu\n",(unsigned long long)checks,(unsigned long long)lookups);
    return 0;
}
