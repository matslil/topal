/* Source-built user-mode cases for TOPAL-SYSTEMS-USER-EXTENT-001.
 * The normal fixture linker supplies this System V to Linux ABI adapter.
 * This program only passes ordinary syscall arguments and accesses its own
 * admitted image, stack and heap; it never edits kernel state or page tables.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word number, word first, word second, word third);

static int split_uname(word address) {
    volatile unsigned char *out = (volatile unsigned char *)address;
    unsigned char saved[392];
    for (unsigned i = 0; i < sizeof(saved); ++i) saved[i] = out[(long)i - 1];
    out[-1] = 0xa7;
    out[390] = 0xb6;
    long result = topal_test_syscall3(63, address, 0, 0);
    int failed = result != 0 || out[-1] != 0xa7 || out[390] != 0xb6;
    failed |= out[0] != 'L' || out[1] != 'i' || out[2] != 'n';
    failed |= out[3] != 'u' || out[4] != 'x' || out[5] != 0;
    for (unsigned i = 0; i < sizeof(saved); ++i) out[(long)i - 1] = saved[i];
    return failed;
}

#if TOPAL_PROBE_CONSOLE
static int copied_console(word address) {
    static const unsigned char marker[] = "TOPAL_USER_EXTENT_WRITE\n";
    volatile unsigned char *input = (volatile unsigned char *)address;
    unsigned char saved[sizeof(marker)+1];
    for (unsigned i = 0; i < sizeof(saved); ++i) saved[i] = input[(long)i-1];
    for (unsigned i = 0; i < sizeof(marker)-1; ++i) input[i] = marker[i];
    input[-1] = 0xa7;
    input[sizeof(marker)-1] = 0xb6;
    long result = topal_test_syscall3(1,1,address,sizeof(marker)-1);
    int failed = result != (long)(sizeof(marker)-1) || input[-1] != 0xa7 || input[sizeof(marker)-1] != 0xb6;
    for (unsigned i = 0; i < sizeof(marker)-1; ++i) failed |= input[i] != marker[i];
    for (unsigned i = 0; i < sizeof(saved); ++i) input[(long)i-1] = saved[i];
    return failed;
}
static int copied_cwd(void) {
    volatile unsigned char *out = (volatile unsigned char *)0x638fff;
    unsigned char saved[8];
    for (unsigned i = 0; i < sizeof(saved); ++i) saved[i] = out[(long)i-1];
    for (unsigned i = 0; i < 5; ++i) out[i] = 0xcc;
    out[-1] = 0xa7;
    out[5] = 0xb6;
    long result = topal_test_syscall3(79,0x638fff,5,0);
    int failed = out[0] != '/' || out[-1] != 0xa7 || out[5] != 0xb6;
    if (result == 2) {
        failed |= out[1] != 0;
        for (unsigned i = 2; i < 5; ++i) failed |= out[i] != 0xcc;
    } else if (result == 5) {
        int bin = out[1] == 'b' && out[2] == 'i' && out[3] == 'n';
        int tmp = out[1] == 't' && out[2] == 'm' && out[3] == 'p';
        failed |= !(bin || tmp) || out[4] != 0;
    } else failed = 1;
    for (unsigned i = 0; i < sizeof(saved); ++i) out[(long)i-1] = saved[i];
    return failed;
}
#endif

long topal_user_extent_source_cases(void) {
    long original = topal_test_syscall3(12, 0, 0, 0);
    if (original < 0x5f7000 || original > 0x619000) return 1;
    if (topal_test_syscall3(12, 0x63a000, 0, 0) != 0x63a000) return 2;

    /* Actual output across a directory boundary, then a page boundary. */
    if (split_uname(0x5ffff8)) return 3;
    if (split_uname(0x638ff8)) return 4;

    /* A refused full copy must preserve even its valid one-byte prefix. */
    volatile unsigned char *edge = (volatile unsigned char *)0x639fff;
    edge[-1] = 0xc9;
    edge[0] = 0xda;
    if (topal_test_syscall3(63, 0x639fff, 0, 0) != -14) return 5;
    if (edge[-1] != 0xc9 || edge[0] != 0xda) return 6;
    if (topal_test_syscall3(63, 0x63a000, 0, 0) != -14) return 7;
    if (topal_test_syscall3(63, ~(word)0, 0, 0) != -14) return 8;

    /* Application-supplied read-only or supervisor output candidates. */
    volatile const unsigned char *header = (volatile const unsigned char *)0x400000;
    unsigned char before = header[0];
    if (topal_test_syscall3(63, 0x400000, 0, 0) != -14) return 9;
    if (header[0] != before) return 10;
    if (topal_test_syscall3(63, 0x800000, 0, 0) != -14) return 11;

    /* Empty-transfer and independent descriptor/count error precedence. */
    if (topal_test_syscall3(1, 1, ~(word)0, 0) != 0) return 12;
    if (topal_test_syscall3(1, 0, ~(word)0, 0) != -9) return 13;
    if (topal_test_syscall3(1, 1, ~(word)0, 257) != -22) return 14;
    if (topal_test_syscall3(1, 1, ~(word)0, 1) != -14) return 15;

#if TOPAL_PROBE_CONSOLE
    if (topal_test_syscall3(1,1,0x639fff,2) != -14 || edge[-1] != 0xc9 || edge[0] != 0xda) return 17;
    if (copied_console(0x5ffff8)) return 18;
    if (copied_console(0x638ff8)) return 19;
    if (copied_cwd()) return 20;
#endif

    /* Preserve the surrounding witness's original heap and existing bytes. */
    if (topal_test_syscall3(12, (word)original, 0, 0) != original) return 16;
    return 0;
}
