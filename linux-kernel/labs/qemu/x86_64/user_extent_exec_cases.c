/* Ordinary execve argument transfers: TOPAL-SYSTEMS-USER-EXTENT-001,
 * TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-{ARGUMENTS,ENVIRONMENT}-001.
 * Every exec attempt here is rejected before image replacement. The enclosing
 * application subsequently executes its existing successful replacement.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
static const unsigned char path[] = "/bin/busybox";
static const unsigned char argument[] = "true";
static const unsigned char environment[] = "A=one";
static const word readonly_argv[] = {(word)path, (word)argument, 0};

static void put(word address, word value) {
    volatile unsigned char *out = (volatile unsigned char *)address;
    for (unsigned i = 0; i < 8; ++i) out[i] = (unsigned char)(value >> (8*i));
}
static void copy(word address, const unsigned char *bytes, unsigned size) {
    for (unsigned i = 0; i < size; ++i) ((volatile unsigned char *)address)[i] = bytes[i];
}
static int rejected(word name, word argv, word envp, long error) {
    return topal_test_syscall3(59,name,argv,envp) != error;
}

long topal_user_extent_exec_cases(void) {
    long original = topal_test_syscall3(12,0,0,0);
    if (original != 0x619000) return 401;
    if (topal_test_syscall3(12,0x63a000,0,0) != 0x63a000) return 402;
    unsigned char saved[32];
    volatile unsigned char *directory = (volatile unsigned char *)0x5ffffb;
    for (unsigned i = 0; i < sizeof(saved); ++i) saved[i] = directory[i];
    const word bad[] = {0,0x63a000,~(word)0,0x800000};
    for (unsigned i = 0; i < 4; ++i)
        if (rejected(bad[i],~(word)0,~(word)0,-14)) return 403;
    *(volatile unsigned char *)0x639fff = '/';
    if (rejected(0x639fff,~(word)0,~(word)0,-14)) return 404;
    if (rejected((word)"/dev/tty",~(word)0,~(word)0,-2)) return 405;
    for (unsigned i = 0; i < 1025; ++i) ((volatile unsigned char *)0x635000)[i] = 'x';
    if (rejected(0x635000,~(word)0,~(word)0,-36)) return 406;

    copy(0x638ff8,path,sizeof(path)); /* Split-page pathname beyond page 34. */
    const word bad_vector[] = {0,0x63a000,~(word)0-3,0x800000,0x639ffc};
    for (unsigned i = 0; i < 5; ++i)
        if (rejected(0x638ff8,bad_vector[i],~(word)0,-14)) return 407;
    put(0x5ffffc,(word)path);
    put(0x600004,(word)argument);
    put(0x60000c,0);
    if (rejected(0x638ff8,0x5ffffc,~(word)0,-14)) return 408;
    copy(0x636fff,argument,sizeof(argument));
    put(0x637ffc,0x638ff8);
    put(0x638004,0x636fff);
    put(0x63800c,0);
    if (rejected(0x638ff8,0x637ffc,~(word)0,-14)) return 409;
    if (rejected((word)path,(word)readonly_argv,~(word)0,-14)) return 410;
    put(0x638004,0x639fff);
    if (rejected(0x638ff8,0x637ffc,~(word)0,-14)) return 411;
    put(0x639ff0,(word)path);
    put(0x639ff8,(word)argument);
    if (rejected((word)path,0x639ff0,~(word)0,-14)) return 412;
    put(0x637ffc,0);
    if (rejected((word)path,0x637ffc,~(word)0,-22)) return 413;
    for (unsigned i = 0; i < 8; ++i) put(0x637ffc+8*i,(word)"");
    put(0x63803c,~(word)0);
    if (rejected((word)path,0x637ffc,~(word)0,-7)) return 414;
    put(0x637ffc,(word)path);
    put(0x638004,0x635000);
    put(0x63800c,0);
    if (rejected((word)path,0x637ffc,~(word)0,-7)) return 415;

    /* A complete argv must precede environment-vector and nested-byte faults. */
    for (unsigned i = 1; i < 4; ++i)
        if (rejected((word)path,0x5ffffc,bad[i],-14)) return 416;
    if (rejected((word)path,0x5ffffc,0x639ffc,-14)) return 417;
    put(0x636ffc,(word)environment);
    put(0x637004,0x639fff);
    put(0x63700c,0);
    *(volatile unsigned char *)0x639fff = 'X';
    if (rejected((word)path,0x5ffffc,0x636ffc,-14)) return 418;
    put(0x639ff0,(word)environment);
    put(0x639ff8,(word)environment);
    if (rejected((word)path,0x5ffffc,0x639ff0,-14)) return 419;
    for (unsigned i = 0; i < 16; ++i) put(0x636ffc+8*i,(word)environment);
    put(0x63707c,~(word)0); /* Count overflow must not dereference entry 17. */
    if (rejected((word)path,0x5ffffc,0x636ffc,-7)) return 420;
    put(0x636ffc,0x635000);
    put(0x637004,0);
    ((volatile unsigned char *)0x635000)[1024] = 0;
    if (rejected((word)path,0x5ffffc,0x636ffc,-7)) return 421;
    for (unsigned i = 0; i < 512; ++i) {
        ((volatile unsigned char *)0x635000)[i] = 'a';
        ((volatile unsigned char *)0x635300)[i] = 'b';
    }
    *(volatile unsigned char *)0x635200 = 0;
    *(volatile unsigned char *)0x635500 = 0;
    put(0x637004,0x635300);
    put(0x63700c,0);
    if (rejected((word)path,0x5ffffc,0x636ffc,-7)) return 422;
    for (unsigned i = 0; i < sizeof(saved); ++i) directory[i] = saved[i];
    if (topal_test_syscall3(12,(word)original,0,0) != original) return 423;
    return 0;
}
