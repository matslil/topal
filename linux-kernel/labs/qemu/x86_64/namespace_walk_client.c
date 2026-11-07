/* Actual copied-byte walker; CWD/absolute component evidence, no kernel grant. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#ifndef NS_WALK_ENTRIES
#define NS_WALK_ENTRIES 1
#endif
#define CHECK(x) do { if (!(x)) {fprintf(stderr,"namespace-walk:%d:%s\n",__LINE__,#x);exit(1);} } while(0)
extern int64_t topal_namespace_walk_component_v1(void *, uint64_t, uint64_t, uint64_t);
enum { PAGE=4096, OWNER=311296, BASE=0x43000, FACTS=1600, MODE=1632, FINAL=1640 };
static uint8_t *owner,*metadata;
static uint64_t cases;
static void word(uint8_t *base,unsigned offset,uint64_t value) {memcpy(base+offset,&value,8);}
static uint64_t get(unsigned offset) {uint64_t n;memcpy(&n,owner+offset,8);return n;}

static void check(const char *path,unsigned cwd,unsigned mode,uint64_t flags,int64_t wanted,
                  int component,unsigned length,unsigned parent,unsigned trailing,unsigned final) {
    size_t size=strlen(path)+1;CHECK(size<=PAGE);
    CHECK(mprotect(owner+PAGE,PAGE,PROT_READ|PROT_WRITE)==0);
    memset(owner+PAGE,0,PAGE);memcpy(owner+PAGE,path,size);
    word(owner,40,cwd);memset(owner+FACTS,0,56);word(owner,MODE,mode);
    CHECK(mprotect(owner+PAGE,PAGE,PROT_READ)==0);
    uint8_t prefix[PAGE],copied[PAGE],before[PAGE];
    memcpy(prefix,owner,PAGE);memcpy(copied,owner+PAGE,PAGE);memcpy(before,metadata,PAGE);
    /* Absolute paths must ignore even this invalid native descriptor. Relative
       paths exercise AT_FDCWD with nonzero upper bits. Real FD lookup traps. */
    uint64_t dirfd=path[0]=='/'?UINT64_C(0xfeedbeef12345678):UINT64_C(0xfeedbeefffffff9c);
    int64_t got=topal_namespace_walk_component_v1(owner,dirfd,mode,flags);
    if(got!=wanted)fprintf(stderr,"path=%s mode=%u got=%lld wanted=%lld\n",path,mode,(long long)got,(long long)wanted);
    CHECK(got==wanted);
    CHECK(memcmp(metadata,before,PAGE)==0 && memcmp(owner+PAGE,copied,PAGE)==0);
    if(component>=0) {
        CHECK(get(FACTS)==(unsigned)component && get(FACTS+8)==length);
        CHECK(get(FACTS+16)==parent && get(FACTS+24)==trailing && get(FINAL)==final);
    }
    /* Only the five explicit walker fact words may change. */
    memcpy(prefix+FACTS,owner+FACTS,32);memcpy(prefix+FINAL,owner+FINAL,8);
    CHECK(memcmp(owner,prefix,PAGE)==0);cases++;
}

int main(void) {
    uint8_t *map=mmap(NULL,OWNER+2*PAGE,PROT_NONE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
    CHECK(map!=MAP_FAILED);owner=map+PAGE;metadata=owner+BASE;
    CHECK(mprotect(owner,2*PAGE,PROT_READ|PROT_WRITE)==0);
    CHECK(mprotect(metadata,PAGE,PROT_READ|PROT_WRITE)==0);
    word(metadata,0,8);word(metadata,32,041777);
    word(metadata,512,1);word(metadata,520,7);word(metadata,536,0100644);word(metadata,568,1);
    if(NS_WALK_ENTRIES) {
        word(metadata,64,7);word(metadata,72,4);memcpy(metadata+1536,"live",4);
    } else {
        word(metadata,584,4);memcpy(metadata+1536+2*256,"live",4);
    }
    CHECK(mprotect(metadata,PAGE,PROT_READ)==0);
    for(unsigned mode=0;mode<3;mode++) {
        check("/tmp/live",3,mode,0,7,5,4,6,0,1);
        check("/tmp/live/",3,mode,0,-20,-1,0,0,0,0);
        check("/tmp/live/x",3,mode,0,-20,-1,0,0,0,0);
        check("/bin/busybox",3,mode,0,mode==2?-30:5,mode==2?-1:5,7,4,0,1);
        check("/missing/x",3,mode,0,-2,-1,0,0,0,0);
        check("/tmp/./live",3,mode,0,7,7,4,6,0,1);
        check("",3,mode,0,-2,-1,0,0,0,0);
    }
    check("/tmp/missing",3,1,0,0,5,7,6,0,1);
    check("/tmp/.",3,2,0,-21,-1,0,0,0,0);
    check("/tmp/",3,2,0,-30,-1,0,0,0,0);
    check("/tmp/..",3,0,0,3,5,2,6,0,1);
    check("/tmp",3,0,0,6,1,3,3,0,1);
    if(NS_WALK_ENTRIES) {
        check("/tmp/live",3,3,0,6,5,4,6,0,1);
        check("/tmp/missing/",3,3,0,6,5,7,6,1,1);
        check("/tmp/live///",3,3,0,6,5,4,6,1,1);
        check("/bin/busybox",3,3,0,4,5,7,4,0,1);
        check("/bin/missing",3,3,0,4,5,7,4,0,1);
        check("/tmp/.",3,3,0,6,5,1,6,0,1);
        check("/tmp/..",3,3,0,6,5,2,6,0,1);
        check("/tmp/../bin/busybox",3,3,0,4,12,7,4,0,1);
        check("/",3,3,0,3,0,0,0,0,0);
        check("////",6,3,0,3,0,0,0,0,0);
        check(".",6,3,0,6,0,1,6,0,1);
        check("../",6,3,0,6,0,2,6,1,1);
        check("relative",3,3,0,3,0,8,3,0,1);
        check("relative",4,3,0,4,0,8,4,0,1);
        check("relative",6,3,0,6,0,8,6,0,1);
        check("/tmp/live/x",3,3,0,-20,-1,0,0,0,0);
        check("/bin/busybox/x",3,3,0,-20,-1,0,0,0,0);
        check("/missing/x",3,3,0,-2,-1,0,0,0,0);
        check("/bin/missing/x",3,3,0,-2,-1,0,0,0,0);
        check("",3,3,0,-2,-1,0,0,0,0);
    }
    char path[PAGE];
    for(unsigned length=1;length<=4090;length++) {
        memcpy(path,"/tmp/",5);memset(path+5,0xfe,length);path[5+length]=0;
        for(unsigned mode=0;mode<3;mode++) {
            int64_t wanted=length>255?-36:(mode==1?0:-2);
            check(path,3,mode,0,wanted,mode==1&&length<=255?5:-1,length,6,0,1);
        }
        if(NS_WALK_ENTRIES)check(path,3,3,0,6,5,length,6,0,1);
    }
    if(NS_WALK_ENTRIES)for(unsigned length=1;length<=4088;length++) {
        memcpy(path,"/tmp/",5);memset(path+5,0xfe,length);memcpy(path+5+length,"/x",3);
        check(path,3,3,0,length>255?-36:-2,-1,0,0,0,0);
    }
    CHECK(munmap(map,OWNER+2*PAGE)==0);
    printf("NAMESPACE_PARENT_WALK_PASS layout=%s checks=%llu\n",NS_WALK_ENTRIES?"entries":"legacy",(unsigned long long)cases);
    return 0;
}
