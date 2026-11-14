/* Actual native final-kind/lookup helpers, guarded bytes; no kernel authority. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#define CHECK(x) do {if(!(x)){fprintf(stderr,"namespace-final:%d:%s\n",__LINE__,#x);exit(1);}}while(0)
extern uint64_t topal_namespace_final_kind_v1(const void *,uint64_t,uint64_t);
extern uint64_t topal_namespace_final_lookup_v1(const void *,uint64_t,const void *,uint64_t,uint64_t *);
enum { PAGE=4096, OWNER=311296, BASE=0x43000 };
static uint8_t *owner,*metadata;
static uint64_t kinds,lookups;
static void word(unsigned offset,uint64_t value){memcpy(metadata+offset,&value,8);}
static void kind(const void *bytes,unsigned length,unsigned final,unsigned expected) {
    CHECK(topal_namespace_final_kind_v1(bytes,length,final)==expected);kinds++;
}
static void lookup(unsigned parent,const void *bytes,unsigned length,uint64_t identity,uint64_t expected_slot) {
    uint64_t slot=UINT64_MAX;uint8_t before[PAGE];memcpy(before,metadata,PAGE);
    CHECK(topal_namespace_final_lookup_v1(owner,parent,bytes,length,&slot)==identity);
    CHECK(slot==expected_slot && memcmp(before,metadata,PAGE)==0);lookups++;
}
int main(void) {
    uint8_t *map=mmap(NULL,OWNER+2*PAGE,PROT_NONE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    CHECK(map!=MAP_FAILED);owner=map+PAGE;metadata=owner+BASE;
    CHECK(mprotect(metadata,PAGE,PROT_READ|PROT_WRITE)==0);
    word(0,8);word(32,041777);word(512,1);word(520,7);word(536,0100644);word(568,1);
    word(64,7);word(72,4);memcpy(metadata+1536,"live",4);
    CHECK(mprotect(metadata,PAGE,PROT_READ)==0);
    uint8_t *input=mmap(NULL,2*PAGE,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    CHECK(input!=MAP_FAILED && mprotect(input+PAGE,PAGE,PROT_NONE)==0);
    kind(".",1,1,1);kind("..",2,1,1);kind("a",1,1,0);kind(".a",2,1,0);kind("a.",2,1,0);
    kind((void *)(uintptr_t)1,0,0,1);
    for(unsigned length=3;length<=4095;length++)kind((void *)(uintptr_t)1,length,1,0);
    for(unsigned parent=3;parent<=6;parent++)if(parent!=5) {
        lookup(parent,"bin",3,parent==3?4:0,4);
        lookup(parent,"tmp",3,parent==3?6:0,4);
        lookup(parent,"busybox",7,parent==4?5:0,4);
        lookup(parent,"live",4,parent==6?7:0,parent==6?0:4);
        lookup(parent,"missing",7,0,4);
    }
    for(unsigned length=1;length<=4095;length++) {
        uint8_t *bytes=input+PAGE-length;memset(bytes,0xfe,length);
        for(unsigned parent=3;parent<=6;parent++)if(parent!=5)
            lookup(parent,bytes,length,0,length>255?(uint64_t)(-36LL):4);
    }
    CHECK(munmap(input,2*PAGE)==0 && munmap(map,OWNER+2*PAGE)==0);
    printf("NAMESPACE_FINAL_COMPONENT_PASS kinds=%llu lookups=%llu\n",(unsigned long long)kinds,(unsigned long long)lookups);
    return 0;
}
