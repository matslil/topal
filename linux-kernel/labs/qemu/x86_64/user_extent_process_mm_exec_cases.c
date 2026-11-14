/* TOPAL-SYSTEMS-PROCESS-MM-001: ordinary child exec with a retained parent.
 * Only userspace reads argv to select its entry routine. Kernel behavior is
 * independent of these strings and this program's expected observations.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
extern long topal_test_syscall4(word, word, word, word, word);

/* TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-{ACTION,MASK}-001. No delivery is
 * requested: these are ordinary registration/query calls across fork/exec.
 * SIGCHLD is left alone so the test does not change Linux wait semantics.
 */
static void signal_handler(void) {}

static void signal_action(word action[4], word signal, word phase) {
    action[0] = signal == 3 ? 1 : (phase == 2 ? 0 : (word)signal_handler);
    action[1] = phase == 2 ? 0 : 0x04000000;
    action[2] = phase == 2 ? 0 : (word)signal_handler;
    action[3] = phase == 2 ? 0 : (1UL << (phase == 0 ? 11 : 14));
}

static long signal_install(word phase, word old_quit[4], word old_user[4], word *old_mask) {
    word action[4], mask = 1UL << (phase == 0 ? 11 : 14);
    signal_action(action, 3, phase);
    if (topal_test_syscall4(13,3,(word)action,(word)old_quit,8) != 0) return 1;
    signal_action(action, 10, phase);
    if (topal_test_syscall4(13,10,(word)action,(word)old_user,8) != 0) return 2;
    if (topal_test_syscall4(14,2,(word)&mask,(word)old_mask,8) != 0) return 3;
    return 0;
}

static long signal_check(word phase) {
    word actual[4], expected[4], mask = 0;
    for (word signal = 3; signal <= 10; signal += 7) {
        signal_action(expected, signal, phase);
        if (topal_test_syscall4(13,signal,0,(word)actual,8) != 0) return 1;
        for (word i = 0; i < 4; ++i) if (actual[i] != expected[i]) return 2;
    }
    if (topal_test_syscall4(14,2,0,(word)&mask,8) != 0) return 3;
    return mask == (1UL << (phase == 0 ? 11 : 14)) ? 0 : 4;
}

static long failed_execs(word fd, word child_break, const char **argv) {
    const word bad = ~(word)0;
    const word names[] = {(word)"/no-such-exec",bad,(word)"/bin/busybox",(word)"/bin/busybox"};
    for (word i = 0; i < 4; ++i) {
        long result = topal_test_syscall3(59,names[i],i == 2 ? bad : (word)argv,i == 3 ? bad : 0);
        if (result != (i == 0 ? -2 : -14)) return 1;
        word tls = 0;
        if (signal_check(1) != 0 || topal_test_syscall3(12,0,0,0) != (long)child_break ||
            topal_test_syscall3(158,0x1003,(word)&tls,0) != 0 || tls != 0x5f7500 ||
            topal_test_syscall3(72,fd,1,0) != 1) return 2;
        if (*(volatile unsigned char *)(0x5f7000+34*4096+128) != 0xc1 ||
            *(volatile unsigned char *)(0x5f7000+66*4096+64) != 0xc2 ||
            *(volatile unsigned char *)(0x5f7000+101*4096-1) != 0xc7) return 3;
    }
    return 0;
}

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
    const char *descriptor = (const char *)initial_stack[2];
    word fd = 0;
    if (!*descriptor) { (void)end_child(64); return; }
    while (*descriptor) {
        if (*descriptor < '0' || *descriptor > '9' || fd > 214748364) { (void)end_child(64); return; }
        fd = fd * 10 + (word)(*descriptor++ - '0');
    }
    if (topal_test_syscall3(72,fd,1,0) != -9) { (void)end_child(65); return; }
    if (signal_check(2) != 0) { (void)end_child(62); return; }
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
    if (signal_check(2) != 0) { (void)end_child(63); return; }
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
    word old_quit[4], old_user[4], old_mask;
    if (signal_install(0,old_quit,old_user,&old_mask) != 0 || signal_check(0) != 0) return 2016;
    long descriptor = topal_test_syscall4(257,(word)-100,(word)"/bin/busybox",0,0);
    if (descriptor < 0 || topal_test_syscall3(72,(word)descriptor,1,0) != 0) return 2019;
    long pid = topal_test_syscall3(57,0,0,0);
    if (pid < 0) return 2003;
    if (pid == 0) {
        if (signal_check(0) != 0) return end_child(45);
        if (signal_install(1,0,0,0) != 0 || signal_check(1) != 0) return end_child(46);
        if (*first != 0xa1 || *far != 0xa2) return end_child(41);
        *first = 0xc1; *far = 0xc2;
        if (topal_test_syscall3(12,child_break,0,0) != (long)child_break) return end_child(42);
        *(volatile unsigned char *)(heap + 101 * page - 1) = 0xc7;
        long self = topal_test_syscall3(39,0,0,0);
        if (topal_test_syscall3(218,~(word)0,0,0) != self ||
            topal_test_syscall3(273,~(word)0,24,0) != 0 ||
            topal_test_syscall3(158,0x1002,0x5f7500,0) != 0) return end_child(44);
        if (topal_test_syscall3(72,(word)descriptor,2,1) != 0) return end_child(47);
        char descriptor_name[24];
        word digit = sizeof(descriptor_name)-1, number = (word)descriptor;
        descriptor_name[digit] = 0;
        do { descriptor_name[--digit] = (char)('0'+number%10); number /= 10; } while (number);
        const char *argv[] = {"child-mm", descriptor_name+digit, (const char *)0};
        if (failed_execs((word)descriptor,child_break,argv) != 0) return end_child(48);
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
    if (signal_check(0) != 0) return 2017;
    if (topal_test_syscall3(72,(word)descriptor,1,0) != 0 ||
        topal_test_syscall3(3,(word)descriptor,0,0) != 0) return 2020;
    if (topal_test_syscall4(13,3,(word)old_quit,0,8) != 0 ||
        topal_test_syscall4(13,10,(word)old_user,0,8) != 0 ||
        topal_test_syscall4(14,2,(word)&old_mask,0,8) != 0) return 2018;
    if (topal_test_syscall3(12,(word)original,0,0) != original) return 2014;
    return 0;
}
