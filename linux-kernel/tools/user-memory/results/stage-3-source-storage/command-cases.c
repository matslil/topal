/* Source-built short/refused command reads; TOPAL-SYSTEMS-USER-EXTENT-001. */
typedef unsigned long word;
extern long topal_test_syscall4(word, word, word, word, word);

long topal_user_extent_command_cases(word n, word a, word b, word count, word d) {
    if (n || a || !count || count > 1023) return -401;
    long original = topal_test_syscall4(12,0,0,0,0);
    if (original != 0x619000) return -402;
    if (topal_test_syscall4(12,0x63a000,0,0,0) != 0x63a000) return -403;
    if (topal_test_syscall4(0,0,~(word)0,0,d) != 0) return -404;
    const word bad[] = {0x63a000,~(word)0,0x400000,0x800000,0x639fff};
    volatile unsigned char *edge = (volatile unsigned char *)0x639fff;
    edge[0] = 0xda;
    for (unsigned i = 0; i < 5; ++i) {
        if (topal_test_syscall4(0,0,bad[i],count,d) != -14) return -405;
        if (edge[0] != 0xda) return -406;
    }
    volatile unsigned char *out = (volatile unsigned char *)0x638ff8;
    if (topal_test_syscall4(0,0,(word)out,1,d) != 1) return -407;
    if (topal_test_syscall4(0,0,~(word)0,0,d) != 0) return -408;
    if (topal_test_syscall4(0,0,0x639fff,count,d) != -14) return -409;
    long rest = topal_test_syscall4(0,0,(word)out+1,count,d);
    if (rest <= 0 || rest > 511 || (word)rest+1 > count) return -410;
    for (word i = 0; i <= (word)rest; ++i) ((volatile unsigned char *)b)[i] = out[i];
    if (topal_test_syscall4(12,(word)original,0,0,0) != original) return -411;
    return rest+1;
}
