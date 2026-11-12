/* TOPAL-SYSTEMS-PROCESS-MM-001: successful maximum copy and 0/1-page handoff.
 * The compile choice changes this ordinary userspace program only.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
extern long topal_test_syscall4(word, word, word, word, word);

static void finish(word syscall,word status) { (void)topal_test_syscall3(syscall,status,0,0); }

static unsigned char pattern(word offset) { return (unsigned char)(offset^0x5a); }

static int matches(word count) {
    volatile unsigned char *bytes=(volatile unsigned char *)0x5f7000;
    for (word i=0;i<count;++i) if (bytes[i]!=pattern(i)) return 0;
    return 1;
}

static long geometry_body(void) {
    const word heap=0x5f7000,page=4096,maximum=0x7fe000;
    const word large=maximum-13,smaller=heap+518*page-7;
    volatile unsigned char *bytes=(volatile unsigned char *)heap;
    if (topal_test_syscall3(39,0,0,0)!=1 || topal_test_syscall3(12,0,0,0)!=(long)heap) return 3301;
    if (topal_test_syscall3(12,large,0,0)!=(long)large) return 3302;
    for (word i=0;i<519*page;++i) bytes[i]=pattern(i);
    long pid=topal_test_syscall4(TOPAL_GEOMETRY_SYSCALL,17,0,0,0);
    if (pid==0) {
        if (topal_test_syscall3(39,0,0,0)!=2 || topal_test_syscall3(110,0,0,0)!=1 ||
            topal_test_syscall3(12,0,0,0)!=(long)large || !matches(519*page)) { finish(60,91); return 3391; }
        if (topal_test_syscall3(12,maximum,0,0)!=(long)maximum ||
            topal_test_syscall3(12,large,0,0)!=(long)large || !matches(519*page)) { finish(60,92); return 3392; }
        for (word i=0;i<519*page;++i) bytes[i]=(unsigned char)(pattern(i)^0xff);
        if (topal_test_syscall3(12,heap,0,0)!=(long)heap || topal_test_syscall3(24,0,0,0)!=0) { finish(60,93); return 3393; }
        if (topal_test_syscall3(39,0,0,0)!=2 || topal_test_syscall3(12,0,0,0)!=(long)heap) { finish(60,94); return 3394; }
        if (topal_test_syscall3(12,heap+1,0,0)!=(long)(heap+1)) { finish(60,95); return 3395; }
        for (word i=0;i<page;++i) if (bytes[i]!=0) { finish(60,96); return 3396; }
        bytes[0]=0xc1;bytes[page-1]=0xc7;
        if (topal_test_syscall3(24,0,0,0)!=0 || topal_test_syscall3(39,0,0,0)!=2 ||
            topal_test_syscall3(12,0,0,0)!=(long)(heap+1) || bytes[0]!=0xc1 || bytes[page-1]!=0xc7) { finish(60,97); return 3397; }
        finish(60,23);return 3398;
    }
    if (pid!=2 || topal_test_syscall3(12,0,0,0)!=(long)large || !matches(519*page)) return 3303;
    if (topal_test_syscall3(57,0,0,0)!=-11) return 3304;
    if (topal_test_syscall3(12,smaller,0,0)!=(long)smaller || !matches(518*page)) return 3305;
    if (topal_test_syscall3(24,0,0,0)!=0 || topal_test_syscall3(12,0,0,0)!=(long)smaller || !matches(518*page)) return 3306;
    if (topal_test_syscall3(24,0,0,0)!=0 || topal_test_syscall3(12,0,0,0)!=(long)smaller || !matches(518*page)) return 3307;
    if (topal_test_syscall4(61,(word)pid,0x570000,0,0)!=-14 ||
        topal_test_syscall4(61,(word)pid,maximum-4,0,0)!=-14) return 3308;
    int status=0;
    if (topal_test_syscall4(61,(word)pid,(word)&status,0,0)!=pid || status!=(23<<8)) return 3309;
    if (topal_test_syscall4(61,(word)pid,0,0,0)!=-10 || !matches(518*page)) return 3310;
    finish(231,0);return 3311;
}

static int equal(const char *a,const char *b) {
    while (*a && *a==*b) { ++a; ++b; }
    return *a==*b;
}

void topal_process_exec_entry(const word *stack) {
    if (stack[0]!=2 || !equal((const char *)stack[1],"geometry-mm")) return;
    long error=geometry_body();finish(60,(word)error);
}

long topal_process_memory_geometry_cases(void) {
    const char *argv[]={"geometry-mm","independent",(const char *)0};
    (void)topal_test_syscall3(59,(word)"/bin/busybox",(word)argv,0);
    return 3312;
}
