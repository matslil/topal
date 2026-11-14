/* Source-built file requests for TOPAL-SYSTEMS-USER-EXTENT-001.
 * Only ordinary syscall arguments and application-owned storage are used.
 */
typedef unsigned long word;
extern long topal_test_syscall4(word, word, word, word, word);

static long output_call(word n, word a, word b, word c, word d, word address) {
    if (n == 262) return topal_test_syscall4(n, a, b, address, d);
    return topal_test_syscall4(n, a, address, c, d);
}

static long output_cases(word n, word a, word b, word c, word d) {
    const word bad[] = {0x63a000, ~(word)0, 0x400000, 0x800000, 0x639fff};
    volatile unsigned char *edge = (volatile unsigned char *)0x639fff;
    edge[-1] = 0xc9;
    edge[0] = 0xda;
    for (unsigned i = 0; i < 5; ++i) {
        long result = output_call(n, a, b, c, d, bad[i]);
        if (result == 0 && n == 217 && i == 0) return 0; /* EOF has no output. */
        if (result == -2 && n == 262 && i == 0) return -2; /* Absent lookup. */
        if (result != -14) return -201;
        if (edge[-1] != 0xc9 || edge[0] != 0xda) return -202;
    }
    word address = n == 217 ? 0x639fb0 : (n == 5 && a == 1 ? 0x5ffff8 : 0x638ff8);
    volatile unsigned char *area = (volatile unsigned char *)(address - 1);
    volatile unsigned char *out = (volatile unsigned char *)address;
    unsigned char saved[146];
    word capacity = n == 217 ? 80 : 145;
    for (word i = 0; i <= capacity; ++i) saved[i] = area[i];
    /* Both possible directory records fit in 80 bytes. Guard the full stat
     * extent as well; the observer separately verifies the selected length. */
    area[0] = 0xa7;
    for (word i = 0; i < capacity; ++i) out[i] = 0xb6;
    long result = output_call(n, a, b, c, d, address);
    if ((n == 217 && result != 72 && result != 80) || (n != 217 && result != 0)) return -203;
    word size = n == 217 ? (word)result : 144;
    if (area[0] != 0xa7) return -204;
    for (word i = size; i < capacity; ++i) if (out[i] != 0xb6) return -214;
    word destination = n == 262 ? c : b;
    for (word i = 0; i < size; ++i) ((volatile unsigned char *)destination)[i] = out[i];
    for (word i = 0; i <= capacity; ++i) area[i] = saved[i];
    return result;
}

long topal_user_extent_file_cases(word n, word a, word b, word c, word d) {
    if (n != 5 && n != 262 && n != 257 && n != 217 && n != 0) return -205;
    long original = topal_test_syscall4(12, 0, 0, 0, 0);
    if (original != 0x619000) return -206;
    if (topal_test_syscall4(12, 0x63a000, 0, 0, 0) != 0x63a000) return -207;
    long result;
    if (n == 0) {
        if (topal_test_syscall4(n, a, ~(word)0, c, d) != -14) return -208;
        if (topal_test_syscall4(n, a, ((word)1 << 47)-4096-127, c, d) != -14) return -209;
        /* Numerically valid, unmapped EOF output needs no mapping admission. */
        result = topal_test_syscall4(n, a, 0x63a000, c, d);
        if (result != 0) return -210;
    } else if (n == 257 || n == 262) {
        const unsigned char *source = (const unsigned char *)b;
        word size = 0;
        while (size < 64 && source[size]) ++size;
        if (size == 64) return -211;
        ++size;
        volatile unsigned char *path = (volatile unsigned char *)0x638ff8;
        for (word i = 0; i < size; ++i) path[i] = source[i];
        const word bad[] = {0x63a000, ~(word)0, 0x800000, 0x639fff};
        *(volatile unsigned char *)0x639fff = '/';
        for (unsigned i = 0; i < 4; ++i)
            if (topal_test_syscall4(n, a, bad[i], c, d) != -14) return -212;
        /* Keep pathname and stat output disjoint while both are live. */
        path = (volatile unsigned char *)0x637ff8;
        for (word i = 0; i < size; ++i) path[i] = source[i];
        if (n == 257) result = topal_test_syscall4(n, a, (word)path, c, d);
        else result = output_cases(n, a, (word)path, c, d);
    } else result = output_cases(n, a, b, c, d);
    if (topal_test_syscall4(12, (word)original, 0, 0, 0) != original) return -213;
    return result;
}
