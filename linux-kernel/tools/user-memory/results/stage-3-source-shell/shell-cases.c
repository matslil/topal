/* Ordinary shell transfers for TOPAL-SYSTEMS-USER-EXTENT-001. */
typedef unsigned long word;
extern long topal_test_syscall4(word, word, word, word, word);
static const word child_action[4] = {0x4a6580,0x4000000,0x4a6580,0xfffffffe7fffffff};
static const word quit_action[4] = {1,0x4000000,0x4a6580,0xfffffffe7fffffff};

static long transfer(word n, word a, word b, word c, word d, word pointer) {
    if (n == 13) return b ? topal_test_syscall4(n,a,pointer,c,d) : topal_test_syscall4(n,a,b,pointer,d);
    if (n == 63 || n == 79 || n == 80) return topal_test_syscall4(n,pointer,b,c,d);
    return topal_test_syscall4(n,a,pointer,c,d);
}

long topal_user_extent_shell_cases(word n, word a, word b, word c, word d) {
    long original = topal_test_syscall4(12,0,0,0,0);
    if (original != 0x619000) return -301;
    if (topal_test_syscall4(12,0x63a000,0,0,0) != 0x63a000) return -302;
    long result;
    if (n == 16) {
        result = topal_test_syscall4(n,a,b,~(word)0,d);
        if (result != -25) return -303;
    } else {
        int input = n == 1 || n == 80 || (n == 13 && b);
        word size = n == 63 ? 390 : (n == 13 ? 32 : (n == 79 || n == 80 ? 2 : c));
        if (!size || size > 390) return -304;
        word source = n == 13 ? (b ? b : c) : (n == 63 || n == 79 || n == 80 ? a : b);
        const word bad[] = {0x63a000,~(word)0,0x800000,size == 1 ? 0x63b000 : 0x639fff};
        volatile unsigned char *edge = (volatile unsigned char *)0x639fff;
        edge[0] = 0xda;
        for (unsigned i = 0; i < 4; ++i) {
            if (transfer(n,a,b,c,d,bad[i]) != -14) return -305;
            if (edge[0] != 0xda) return -306;
        }
        if (!input && transfer(n,a,b,c,d,0x400000) != -14) return -307;
        word address = n == 79 ? 0x639ffe : (n == 0 ? 0x639fff : (n == 80 ? 0x638fff : 0x638ff8));
        volatile unsigned char *area = (volatile unsigned char *)(address-1);
        volatile unsigned char *out = (volatile unsigned char *)address;
        area[0] = 0xa7;
        if (address+size < 0x63a000) out[size] = 0xb6;
        for (word i = 0; i < size; ++i) out[i] = input ? ((const unsigned char *)source)[i] : 0xa5;
        word pointer = address;
        if (n == 13 && b) {
            const word *action = a == 17 ? child_action : quit_action;
            for (unsigned i = 0; i < 4; ++i) if (((const word *)source)[i] != action[i]) return -308;
            pointer = (word)action; /* Admitted read-only action data. */
        }
        result = transfer(n,a,b,c,d,pointer);
        long expected = n == 1 || n == 0 || n == 79 ? (long)size : 0;
        if (result != expected) return -309;
        if (area[0] != 0xa7) return -310;
        if (address+size < 0x63a000 && out[size] != 0xb6) return -311;
        if (input) {
            for (word i = 0; i < size; ++i) if (out[i] != ((const unsigned char *)source)[i]) return -312;
        } else {
            for (word i = 0; i < size; ++i) ((volatile unsigned char *)source)[i] = out[i];
        }
    }
    if (topal_test_syscall4(12,(word)original,0,0,0) != original) return -313;
    return result;
}
