/* Ordinary process/MM operations. TOPAL-SYSTEMS-PROCESS-MM-001.
 * The kernel sees normal syscalls and user buffers, not expected test results.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
extern long topal_test_syscall4(word, word, word, word, word);

static long child_end(word status) {
    (void)topal_test_syscall3(60, status, 0, 0);
    return 2000 + (long)status; /* An unexpected exit return is a failure. */
}

long topal_process_memory_cases(void) {
    const word heap = 0x5f7000, page = 4096;
    const word parent_break = heap + 67 * page - 7;
    const word child_break = heap + 101 * page - 11;
    volatile unsigned char *first = (volatile unsigned char *)(heap + 34 * page + 128);
    volatile unsigned char *far = (volatile unsigned char *)(heap + 66 * page + 64);
    volatile unsigned char *last = (volatile unsigned char *)(heap + 101 * page - 1);
    long original = topal_test_syscall3(12, 0, 0, 0);
    if (original != 0x619000) return 1001;
    if (topal_test_syscall3(39, 0, 0, 0) != 1) return 1002;
    if (topal_test_syscall4(56, 0x111, 0, 0, 0) != -38) return 1018;
    if (topal_test_syscall3(12, parent_break, 0, 0) != (long)parent_break) return 1003;
    *first = 0xa1;
    *far = 0xa2;
    long pid = topal_test_syscall3(57, 0, 0, 0);
    if (pid < 0) return 1004;
    if (pid == 0) {
        if (topal_test_syscall3(39, 0, 0, 0) < 2 ||
            topal_test_syscall3(110, 0, 0, 0) != 1) return child_end(31);
        if (*first != 0xa1 || *far != 0xa2) return child_end(32);
        *first = 0xc1;
        *far = 0xc2;
        if (topal_test_syscall3(12, child_break, 0, 0) != (long)child_break) return child_end(33);
        if (*last != 0) return child_end(34);
        *last = 0xc7;
        if (topal_test_syscall3(12, child_break - 1, 0, 0) != (long)(child_break - 1)) return child_end(38);
        if (topal_test_syscall3(12, child_break, 0, 0) != (long)child_break) return child_end(39);
        if (topal_test_syscall3(24, 0, 0, 0) != 0) return child_end(35);
        if (topal_test_syscall3(12, 0, 0, 0) != (long)child_break) return child_end(36);
        if (*first != 0xc1 || *far != 0xc2 || *last != 0xc7) return child_end(37);
        return child_end(17);
    }
    if (pid < 2 || topal_test_syscall3(12, 0, 0, 0) != (long)parent_break) return 1005;
    if (*first != 0xa1 || *far != 0xa2) return 1006;
    if (topal_test_syscall3(57, 0, 0, 0) != -11) return 1007;
    if (topal_test_syscall3(12, heap + 35 * page, 0, 0) != (long)(heap + 35 * page)) return 1008;
    if (*first != 0xa1) return 1009;
    *first = 0xb1;
    if (topal_test_syscall3(24, 0, 0, 0) != 0) return 1010;
    if (topal_test_syscall3(12, 0, 0, 0) != (long)(heap + 35 * page)) return 1011;
    if (*first != 0xb1) return 1012;
    if (topal_test_syscall4(61, (word)pid, 0x570000, 0, 0) != -14) return 1019;
    if (topal_test_syscall4(61, (word)pid, heap + 67 * page, 0, 0) != -14) return 1013;
    int status = 0;
    if (topal_test_syscall4(61, (word)pid, (word)&status, 0, 0) != pid) return 1014;
    if (status != (17 << 8)) return 1015;
    if (topal_test_syscall4(61, (word)pid, 0, 0, 0) != -10) return 1016;
    if (topal_test_syscall3(12, (word)original, 0, 0) != original) return 1017;
    return 0;
}
