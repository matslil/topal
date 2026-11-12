/* TOPAL-SYSTEMS-PROCESS-MM-001: ordinary child exec with a retained parent.
 * Only userspace reads argv to select its entry routine. Kernel behavior is
 * independent of these strings and this program's expected observations.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
extern long topal_test_syscall4(word, word, word, word, word);

static long end_child(word status) {
    (void)topal_test_syscall3(60, status, 0, 0);
    return 3000 + (long)status;
}

static int equal(const char *a, const char *b) {
    while (*a && *a == *b) { ++a; ++b; }
    return *a == *b;
}

/* C startup called from a normal backend Linux-entry-to-System-V adapter.
 * The initial parent and its ordinary later generations use the surrounding
 * ABI witness. A successfully exec'd child runs this independent C program.
 */
void topal_process_exec_entry(const word *initial_stack) {
    if (initial_stack[0] != 2 || !equal((const char *)initial_stack[1], "child-mm")) return;
    const word heap = 0x5f7000, page = 4096, limit = heap + 101 * page - 11;
    long pid = topal_test_syscall3(39, 0, 0, 0);
    if (pid < 2 || topal_test_syscall3(110, 0, 0, 0) != 1) { (void)end_child(51); return; }
    if (topal_test_syscall3(12, 0, 0, 0) != (long)heap) { (void)end_child(52); return; }
    word tls = ~(word)0;
    if (topal_test_syscall3(158,0x1003,(word)&tls,0) != 0 || tls != 0) { (void)end_child(59); return; }
    word registration = 0;
    if (topal_test_syscall3(218, (word)&registration, 0, 0) != pid) { (void)end_child(53); return; }
    word robust[3] = {0,0,0};
    if (topal_test_syscall3(273, (word)robust, 24, 0) != 0) { (void)end_child(54); return; }
    if (topal_test_syscall3(12, limit, 0, 0) != (long)limit) { (void)end_child(55); return; }
    volatile unsigned char *bytes = (volatile unsigned char *)heap;
    for (word i = 0; i < 101 * page; ++i)
        if (bytes[i] != 0) { (void)end_child(56); return; }
    bytes[34 * page + 128] = 0xe1;
    bytes[66 * page + 64] = 0xe2;
    bytes[101 * page - 1] = 0xe7;
    if (topal_test_syscall3(158,0x1002,heap+2*page,0) != 0) { (void)end_child(60); return; }
    if (topal_test_syscall3(24, 0, 0, 0) != 0) { (void)end_child(57); return; }
    if (topal_test_syscall3(39, 0, 0, 0) != pid ||
        topal_test_syscall3(12, 0, 0, 0) != (long)limit ||
        bytes[34 * page + 128] != 0xe1 || bytes[66 * page + 64] != 0xe2 ||
        bytes[101 * page - 1] != 0xe7) { (void)end_child(58); return; }
    if (topal_test_syscall3(158,0x1003,(word)&tls,0) != 0 || tls != heap+2*page) { (void)end_child(61); return; }
    (void)end_child(17);
}

long topal_process_memory_exec_cases(void) {
    const word heap = 0x5f7000, page = 4096;
    const word parent_break = heap + 67 * page - 7, child_break = heap + 101 * page - 11;
    volatile unsigned char *first = (volatile unsigned char *)(heap + 34 * page + 128);
    volatile unsigned char *far = (volatile unsigned char *)(heap + 66 * page + 64);
    long original = topal_test_syscall3(12, 0, 0, 0);
    if (original != 0x619000 || topal_test_syscall3(39,0,0,0) != 1) return 2001;
    if (topal_test_syscall3(12,parent_break,0,0) != (long)parent_break) return 2002;
    *first = 0xa1; *far = 0xa2;
    long pid = topal_test_syscall3(57,0,0,0);
    if (pid < 0) return 2003;
    if (pid == 0) {
        if (*first != 0xa1 || *far != 0xa2) return end_child(41);
        *first = 0xc1; *far = 0xc2;
        if (topal_test_syscall3(12,child_break,0,0) != (long)child_break) return end_child(42);
        *(volatile unsigned char *)(heap + 101 * page - 1) = 0xc7;
        long self = topal_test_syscall3(39,0,0,0);
        if (topal_test_syscall3(218,~(word)0,0,0) != self ||
            topal_test_syscall3(273,~(word)0,24,0) != 0 ||
            topal_test_syscall3(158,0x1002,0x5f7500,0) != 0) return end_child(44);
        const char *argv[] = {"child-mm", "independent", (const char *)0};
        (void)topal_test_syscall3(59,(word)"/bin/busybox",(word)argv,0);
        return end_child(43);
    }
    if (pid < 2 || topal_test_syscall3(12,0,0,0) != (long)parent_break) return 2004;
    if (*first != 0xa1 || *far != 0xa2) return 2005;
    if (topal_test_syscall3(57,0,0,0) != -11) return 2006;
    if (topal_test_syscall3(12,heap+35*page,0,0) != (long)(heap+35*page)) return 2007;
    *first = 0xb1;
    if (topal_test_syscall3(24,0,0,0) != 0) return 2008;
    if (topal_test_syscall3(12,0,0,0) != (long)(heap+35*page) || *first != 0xb1) return 2009;
    word tls=0;
    if (topal_test_syscall3(158,0x1003,(word)&tls,0) != 0 || tls != 0x5f7400) return 2015;
    if (topal_test_syscall4(61,(word)pid,0x570000,0,0) != -14) return 2010;
    if (topal_test_syscall4(61,(word)pid,heap+67*page,0,0) != -14) return 2011;
    int status=0;
    if (topal_test_syscall4(61,(word)pid,(word)&status,0,0) != pid || status != 17<<8) return 2012;
    if (topal_test_syscall4(61,(word)pid,0,0,0) != -10) return 2013;
    if (topal_test_syscall3(12,(word)original,0,0) != original) return 2014;
    return 0;
}
