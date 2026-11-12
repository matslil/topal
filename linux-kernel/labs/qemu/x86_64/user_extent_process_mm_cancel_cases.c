/* TOPAL-SYSTEMS-PROCESS-MM-001: cancel a real blocked child and its resources.
 * These compile-time choices affect this userspace program only. Kernel
 * dispatch never reads its strings, expected status or selected case.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
extern long topal_test_syscall4(word, word, word, word, word);
#ifndef TOPAL_CANCEL_WRITER
#define TOPAL_CANCEL_WRITER 0
#endif

static long end(word syscall, word status) {
    (void)topal_test_syscall3(syscall, status, 0, 0);
    return 4000 + (long)status;
}

static long cancel_body(void) {
    const word heap=0x5f7000, page=4096;
    volatile unsigned char *bytes=(volatile unsigned char *)heap;
    if (topal_test_syscall3(39,0,0,0)!=1) return 3101;
    if (topal_test_syscall3(12,0,0,0)!=(long)(TOPAL_CANCEL_EARLY?heap:0x619000)) return 3102;
    if (!TOPAL_CANCEL_EARLY) {
        if (topal_test_syscall3(12,heap+67*page-7,0,0)!=(long)(heap+67*page-7)) return 3103;
        bytes[34*page+128]=0xa1; bytes[66*page+64]=0xa2;
    }
    int pipe[2]={-1,-1};
    if (topal_test_syscall3(22,(word)pipe,0,0)!=0) return 3104;
    long alias=topal_test_syscall3(32,(word)pipe[TOPAL_CANCEL_WRITER?0:1],0,0);
    if (alias<3) return 3105;
    long pid=topal_test_syscall3(57,0,0,0);
    if (pid<0) return 3106;
    if (pid==0) {
        if (topal_test_syscall3(3,(word)pipe[TOPAL_CANCEL_WRITER?0:1],0,0)!=0 ||
            topal_test_syscall3(3,(word)alias,0,0)!=0) return end(60,71);
        if (!TOPAL_CANCEL_EARLY &&
            (bytes[34*page+128]!=0xa1 || bytes[66*page+64]!=0xa2)) return end(60,72);
        if (topal_test_syscall3(12,heap+101*page-11,0,0)!=(long)(heap+101*page-11)) return end(60,73);
        bytes[34*page+128]=0xc1; bytes[66*page+64]=0xc2; bytes[101*page-1]=0xc7;
        long file=topal_test_syscall4(257,(word)-100,(word)"/tmp",0x410002,0600);
        if (file<3) return end(60,74);
        for (word i=0;i<256;++i) bytes[35*page+i]=(unsigned char)(i^0x5a);
        if (topal_test_syscall3(1,(word)file,heap+35*page,256)!=256) return end(60,75);
        if (TOPAL_CANCEL_WRITER) {
            /* Fill the whole ring with nontrivial data using ordinary writes.
             * The parent retains both reader aliases; the next write parks. */
            for (word i=0;i<16;++i)
                if (topal_test_syscall3(1,(word)pipe[1],heap+35*page,256)!=256) return end(60,77);
            (void)topal_test_syscall3(1,(word)pipe[1],heap+35*page,1);
        } else {
            /* Parent retains both writer aliases, so this read must park. */
            (void)topal_test_syscall3(0,(word)pipe[0],heap+34*page+128,1);
        }
        return end(60,76);
    }
    if (pid<2 || topal_test_syscall3(12,0,0,0)!=(long)(TOPAL_CANCEL_EARLY?heap:heap+67*page-7)) return 3107;
    if (!TOPAL_CANCEL_EARLY &&
        (bytes[34*page+128]!=0xa1 || bytes[66*page+64]!=0xa2)) return 3108;
    if (topal_test_syscall3(3,(word)pipe[TOPAL_CANCEL_WRITER?1:0],0,0)!=0) return 3109;
    if (topal_test_syscall3(12,heap+35*page,0,0)!=(long)(heap+35*page)) return 3110;
    bytes[34*page+128]=0xb1;
    return end(TOPAL_CANCEL_EXIT_SYSCALL,TOPAL_CANCEL_STATUS);
}

static int equal(const char *a,const char *b) {
    while (*a && *a==*b) { ++a; ++b; }
    return *a==*b;
}

void topal_process_exec_entry(const word *stack) {
    if (stack[0]!=2 || !equal((const char *)stack[1],"parent-cancel")) return;
    long error=cancel_body();
    (void)end(60,(word)error);
}

long topal_process_memory_cancel_cases(void) {
    if (!TOPAL_CANCEL_EARLY) return cancel_body();
    const char *argv[]={"parent-cancel","independent",(const char *)0};
    (void)topal_test_syscall3(59,(word)"/bin/busybox",(word)argv,0);
    return 3111;
}
