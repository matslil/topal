/* TOPAL-SYSTEMS-PROCESS-MM-001: terminate with a runnable or zombie child.
 * Compile-time choices affect ordinary userspace only. */
typedef unsigned long word;
extern long topal_test_syscall3(word,word,word,word);
extern long topal_test_syscall4(word,word,word,word,word);

static long end(word syscall,word status) {
    (void)topal_test_syscall3(syscall,status,0,0);
    return 4000+(long)status;
}

static long relationship_body(void) {
    const word heap=0x5f7000,page=4096;
    volatile unsigned char *bytes=(volatile unsigned char *)heap;
    if (topal_test_syscall3(39,0,0,0)!=1 ||
        topal_test_syscall3(12,0,0,0)!=(long)(TOPAL_RELATIONSHIP_EARLY?heap:0x619000)) return 3401;
    if (!TOPAL_RELATIONSHIP_EARLY) {
        if (topal_test_syscall3(12,heap+67*page-7,0,0)!=(long)(heap+67*page-7)) return 3402;
        bytes[34*page+128]=0xa1;bytes[66*page+64]=0xa2;
    }
    int pipe[2]={-1,-1};
    if (topal_test_syscall3(22,(word)pipe,0,0)!=0) return 3403;
    long alias=topal_test_syscall3(32,(word)pipe[1],0,0);
    if (alias<3) return 3404;
    long pid=topal_test_syscall3(57,0,0,0);
    if (pid<0) return 3405;
    if (pid==0) {
        if (topal_test_syscall3(39,0,0,0)!=2 ||
            topal_test_syscall3(3,(word)pipe[1],0,0)!=0 || topal_test_syscall3(3,(word)alias,0,0)!=0) return end(60,81);
        if (!TOPAL_RELATIONSHIP_EARLY && (bytes[34*page+128]!=0xa1 || bytes[66*page+64]!=0xa2)) return end(60,82);
        if (topal_test_syscall3(12,heap+101*page-11,0,0)!=(long)(heap+101*page-11)) return end(60,83);
        bytes[34*page+128]=0xc1;bytes[66*page+64]=0xc2;bytes[101*page-1]=0xc7;
        long file=topal_test_syscall4(257,(word)-100,(word)"/tmp",0x410002,0600);
        if (file<3) return end(60,84);
        for (word i=0;i<256;++i) bytes[35*page+i]=(unsigned char)(i^0x5a);
        if (topal_test_syscall3(1,(word)file,heap+35*page,256)!=256) return end(60,85);
        if (TOPAL_RELATIONSHIP_ZOMBIE) return end(60,0x217);
        (void)topal_test_syscall3(24,0,0,0);
        return end(60,86); /* A runnable child must be cancelled, not resumed. */
    }
    if (pid!=2 || topal_test_syscall3(12,0,0,0)!=(long)(TOPAL_RELATIONSHIP_EARLY?heap:heap+67*page-7)) return 3406;
    if (!TOPAL_RELATIONSHIP_EARLY && (bytes[34*page+128]!=0xa1 || bytes[66*page+64]!=0xa2)) return 3407;
    if (topal_test_syscall3(57,0,0,0)!=-11) return 3408;
    /* WNOHANG does not publish an output for a runnable child. A zombie's
     * invalid status buffer must leave its status and relationship intact. */
    long wait=topal_test_syscall4(61,(word)pid,0x570000,TOPAL_RELATIONSHIP_ZOMBIE?0:1,0);
    if (wait!=(TOPAL_RELATIONSHIP_ZOMBIE?-14:0)) return 3409;
    if (topal_test_syscall3(3,(word)pipe[0],0,0)!=0 ||
        topal_test_syscall3(12,heap+35*page,0,0)!=(long)(heap+35*page)) return 3410;
    bytes[34*page+128]=0xb1;
    const word status=TOPAL_RELATIONSHIP_EARLY?(TOPAL_RELATIONSHIP_GROUP?0x301:0x2c5):(TOPAL_RELATIONSHIP_GROUP?0x200:0x13a);
    return end(TOPAL_RELATIONSHIP_GROUP?231:60,status);
}

static int equal(const char *a,const char *b) {
    while (*a && *a==*b) { ++a;++b; }
    return *a==*b;
}

void topal_process_exec_entry(const word *stack) {
    if (stack[0]!=2 || !equal((const char *)stack[1],"parent-relationship")) return;
    (void)end(60,(word)relationship_body());
}

long topal_process_memory_relationship_cases(void) {
    if (!TOPAL_RELATIONSHIP_EARLY) return relationship_body();
    const char *argv[]={"parent-relationship","independent",(const char *)0};
    (void)topal_test_syscall3(59,(word)"/bin/busybox",(word)argv,0);
    return 3411;
}
