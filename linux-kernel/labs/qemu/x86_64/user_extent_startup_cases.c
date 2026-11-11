/* Ordinary startup syscalls for TOPAL-SYSTEMS-USER-EXTENT-001.
 * A backend call adapter supplies the original request and preserves its
 * non-result registers. No guest kernel or translation state is edited.
 */
typedef unsigned long word;
extern long topal_test_syscall4(word, word, word, word, word);
extern long topal_user_extent_file_cases(word, word, word, word, word);

static long output_call(word n, word a, word b, word c, word d, word out) {
    if (n == 302) return topal_test_syscall4(n, a, b, c, out);
    if (n == 157) return topal_test_syscall4(n, a, out, c, d);
    return topal_test_syscall4(n, out, b, c, d);
}

long topal_user_extent_startup_cases(word n, word a, word b, word c, word d) {
    if (n == 5 || n == 257 || n == 217 || n == 0 || (n == 262 && b != 0x4a6900))
        return topal_user_extent_file_cases(n, a, b, c, d);
    if (n != 302 && n != 318 && n != 157 && n != 262) return -101;
    long original = topal_test_syscall4(12, 0, 0, 0, 0);
    if (original < 0x5f7000 || original > 0x619000) return -102;
    if (topal_test_syscall4(12, 0x63a000, 0, 0, 0) != 0x63a000) return -103;

    if (n == 262) {
        volatile unsigned char *path = (volatile unsigned char *)0x638ff8;
        const unsigned char *source = (const unsigned char *)b;
        for (unsigned i = 0; i < 18; ++i) path[i] = source[i];
        const word bad[] = {0x63a000, ~(word)0, 0x800000, 0x639fff};
        *(volatile unsigned char *)0x639fff = '/';
        for (unsigned i = 0; i < 4; ++i)
            if (topal_test_syscall4(n, a, bad[i], c, d) != -14) return -104;
        /* A missing object does not access its unused metadata destination. */
        if (topal_test_syscall4(n, a, (word)path, ~(word)0, d) != -2) return -105;
        if (topal_test_syscall4(12, (word)original, 0, 0, 0) != original) return -106;
        return -2;
    }

    word size = n == 318 ? b : 16;
    if (n == 318) {
        if (topal_test_syscall4(n, ~(word)0, 0, c, 0) != 0) return -107;
        if (topal_test_syscall4(n, ~(word)0, 8, 8, 0) != -22) return -108;
        if (topal_test_syscall4(n, ~(word)0, 17, c, 0) != -22) return -109;
    }
    const word bad[] = {0x63a000, ~(word)0, 0x400000, 0x800000, 0x639fff};
    volatile unsigned char *edge = (volatile unsigned char *)0x639fff;
    edge[-1] = 0xc9;
    edge[0] = 0xda;
    for (unsigned i = 0; i < 5; ++i) {
        if (output_call(n, a, b, c, d, bad[i]) != -14) return -110;
        if (edge[-1] != 0xc9 || edge[0] != 0xda) return -111;
    }
    /* Cross a directory for prlimit64; a page beyond page 34 for the others. */
    word address = n == 302 ? 0x5ffff8 : 0x638ff8;
    volatile unsigned char *out = (volatile unsigned char *)address;
    out[-1] = 0xa7;
    out[size] = 0xb6;
    long result = output_call(n, a, b, c, d, address);
    if (result != (n == 318 ? (long)size : 0)) return -112;
    if (out[-1] != 0xa7 || out[size] != 0xb6) return -113;
    if (n == 302 && (((volatile word *)out)[0] != 0x800000 ||
                     ((volatile word *)out)[1] != ~(word)0)) return -114;
    if (n == 157) {
        const unsigned char name[16] = {'b','u','s','y','b','o','x'};
        for (unsigned i = 0; i < 16; ++i) if (out[i] != name[i]) return -115;
    }
    word destination = n == 302 ? d : (n == 157 ? b : a);
    for (word i = 0; i < size; ++i) ((volatile unsigned char *)destination)[i] = out[i];
    if (topal_test_syscall4(12, (word)original, 0, 0, 0) != original) return -116;
    return result;
}
