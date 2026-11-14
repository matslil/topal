/* TOPAL-SYSTEMS-PROCESS-MM-001: real shortage, retry and 0/1/519-page heaps.
 * Only userspace chooses its entry routine. The same kernel is used for
 * ordinary larger-RAM cohorts; no expected result selects kernel behavior.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
extern long topal_test_syscall4(word, word, word, word, word);

static void finish(word syscall, word status) {
    (void)topal_test_syscall3(syscall,status,0,0);
}

static unsigned char pattern(word offset) { return (unsigned char)(offset^0x5a); }

static int contents(word count) {
    volatile unsigned char *bytes=(volatile unsigned char *)0x5f7000;
    for (word i=0;i<count;++i) if (bytes[i]!=pattern(i)) return 0;
    return 1;
}

static long allocation_body(void) {
    const word heap=0x5f7000, page=4096, maximum=0x7fe000;
    volatile unsigned char *bytes=(volatile unsigned char *)heap;
    if (topal_test_syscall3(39,0,0,0)!=1 || topal_test_syscall3(12,0,0,0)!=(long)heap) return 3201;
    if (topal_test_syscall3(12,maximum-13,0,0)!=(long)(maximum-13)) return 3202;
    for (word i=0;i<519*page;++i) bytes[i]=pattern(i);
    if (topal_test_syscall3(12,maximum,0,0)!=(long)maximum || !contents(519*page)) return 3203;
    /* This guest's actual physical capacity cannot hold a second 519-page
     * extent alongside the parent and retained mapping owners. */
    if (topal_test_syscall3(57,0,0,0)!=-12) return 3204;
    if (topal_test_syscall3(12,0,0,0)!=(long)maximum || !contents(519*page)) return 3205;
    if (topal_test_syscall3(12,heap+1,0,0)!=(long)(heap+1) || !contents(page)) return 3206;
    long pid=topal_test_syscall3(57,0,0,0);
    if (pid==0) {
        if (!contents(page) || topal_test_syscall3(12,0,0,0)!=(long)(heap+1)) { finish(60,81); return 3281; }
        long limit=topal_test_syscall3(12,maximum,0,0);
        /* A one-page owner can split the remaining free span. A refused raw
         * break keeps its old value; retry the adjacent smaller extent. */
        if (limit==(long)(heap+1)) limit=topal_test_syscall3(12,maximum-page,0,0);
        if (limit!=(long)maximum && limit!=(long)(maximum-page)) { finish(60,82); return 3282; }
        if (!contents(page)) { finish(60,83); return 3283; }
        for (word i=page;i<(word)limit-heap;++i)
            if (bytes[i]!=0) { finish(60,84); return 3284; }
        bytes[0]=0xc1;bytes[(word)limit-heap-1]=0xc7;
        finish(60,17);return 3285;
    }
    if (pid!=2 || !contents(page)) return 3207;
    if (topal_test_syscall4(61,(word)pid,0x570000,0,0)!=-14) return 3208;
    int status=0;
    if (topal_test_syscall4(61,(word)pid,(word)&status,0,0)!=pid || status!=(17<<8)) return 3209;
    if (topal_test_syscall4(61,(word)pid,0,0,0)!=-10) return 3210;
    if (topal_test_syscall3(12,0,0,0)!=(long)(heap+1) || !contents(page)) return 3211;
    if (topal_test_syscall3(12,heap,0,0)!=(long)heap) return 3212;
    pid=topal_test_syscall3(57,0,0,0);
    if (pid==0) {
        if (topal_test_syscall3(12,0,0,0)!=(long)heap) { finish(60,85); return 3286; }
        finish(60,19);return 3287;
    }
    if (pid!=3) return 3213;
    if (topal_test_syscall4(61,(word)pid,(word)&status,0,0)!=pid || status!=(19<<8)) return 3214;
    if (topal_test_syscall3(12,0,0,0)!=(long)heap) return 3215;
    finish(231,0);return 3216;
}

static int equal(const char *a,const char *b) {
    while (*a && *a==*b) { ++a; ++b; }
    return *a==*b;
}

void topal_process_exec_entry(const word *stack) {
    if (stack[0]!=2 || !equal((const char *)stack[1],"allocation-mm")) return;
    long error=allocation_body();
    finish(60,(word)error);
}

long topal_process_memory_allocation_cases(void) {
    const char *argv[]={"allocation-mm","independent",(const char *)0};
    (void)topal_test_syscall3(59,(word)"/bin/busybox",(word)argv,0);
    return 3217;
}
