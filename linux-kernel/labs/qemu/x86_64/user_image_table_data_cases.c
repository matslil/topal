/* Ordinary user-mode test of inactive table BYTE preparation.
 * This program never activates a translation root or accesses physical memory.
 * Numeric physical addresses are inputs to a pure buffer-writing function.
 * The linker must supply the emitted preparation object and syscall adapter.
 * These constants describe a deliberately selected eight-page test topology;
 * they are not authority for the running kernel's actual image or frame owners.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
extern _Bool topal_x86_image_table_prepare_data_v1(void *, word, word, word, word);

#define PAGE 4096UL
#define BANK_BYTES (8UL * PAGE)
#define NX (1UL << 63)
#define LIMIT (1UL << 30)
#define REGION 0x640000UL

static word rights(word address) {
    if (address >= 0x2aec000 && address < 0x2c22000) return 0x61;
    if ((address >= 0x200000 && address < 0x201000) ||
        (address >= 0x2ae4000 && address < 0x2aec000) ||
        (address >= 0x2c22000 && address < 0x2c23000)) return NX | 0x61;
    return NX | 0x63;
}

/* Independent expected-word oracle. It computes leaf rights from the selected
 * test ranges rather than embedding the producer's encoded byte vector.
 */
static word expected(word slot, word index, word physical, word kind) {
    if (slot == 0) return index == 0 ? (physical + PAGE) | 7 : 0;
    if (slot == 1) {
        if (index == 0) return (physical + 2 * PAGE) | 7;
        return index == 3 ? (physical + 3 * PAGE) | 3 : 0;
    }
    if (slot == 2) {
        if (kind && index == 2) return 0x4000000 | 7;
        if (kind && index == 3) return (physical + 4 * PAGE) | 7;
        if (index == 1) return (physical + 5 * PAGE) | 3;
        if (index == 21) return (physical + 6 * PAGE) | 3;
        if (index == 22) return (physical + 7 * PAGE) | 3;
        return (index * 0x200000) | rights(index * 0x200000) | 0x80;
    }
    if (slot == 3) {
        if (index == 502) return NX | 0xfec0009b;
        return index == 503 ? NX | 0xfee0009b : 0;
    }
    if (slot == 4) return 0;
    word block = slot == 5 ? 0x200000 : (slot == 6 ? 0x2a00000 : 0x2c00000);
    word address = block + index * PAGE;
    return address | rights(address);
}

static void fill(volatile unsigned char *bytes) {
    for (word i = 0; i < BANK_BYTES + 2 * PAGE; ++i) bytes[i] = 0xa5;
}

static int sentinel(const volatile unsigned char *bytes, word first, word count) {
    for (word i = first; i < first + count; ++i)
        if (bytes[i] != 0xa5) return 0;
    return 1;
}

static long check_valid(volatile unsigned char *region, word physical, word kind, word low) {
    fill(region);
    volatile word *bank = (volatile word *)(region + PAGE);
    if (!topal_x86_image_table_prepare_data_v1((void *)bank, BANK_BYTES, physical, kind, low))
        return 1;
    for (word slot = 0; slot < 8; ++slot)
        for (word index = 0; index < 512; ++index)
            if (bank[slot * 512 + index] != expected(slot, index, physical, kind)) return 2;
    if (!sentinel(region, 0, PAGE) || !sentinel(region, BANK_BYTES + PAGE, PAGE)) return 3;
    return 0;
}

struct invalid_request { word offset, bytes, physical, kind, low; };

static long image_table_data_checks(void) {
    long original = topal_test_syscall3(12, 0, 0, 0);
    if (original < 0x5f7000 || original > 0x619000) return 4001;
    if (topal_test_syscall3(12, REGION + BANK_BYTES + 2 * PAGE, 0, 0) !=
        (long)(REGION + BANK_BYTES + 2 * PAGE)) return 4002;
    volatile unsigned char *region = (volatile unsigned char *)REGION;
    long failed = 0;
    /* Bootstrap ignores the unused low-table parameter, including UINT64_MAX.
     * The second base is an exact closed upper bound for the complete bank.
     */
    for (word kind = 0; kind <= 1; ++kind)
        for (word edge = 0; edge <= 1; ++edge) {
            word physical = edge ? LIMIT - BANK_BYTES : 0x5000000;
            long result = check_valid(region, physical, kind, kind ? 0x4000000 : ~(word)0);
            if (result && !failed) failed = 4010 + (long)(kind * 8 + edge * 4) + result;
        }
    static const struct invalid_request requests[] = {
        {8, BANK_BYTES, 0x5000000, 0, 0},
        {0, 0, 0x5000000, 0, 0},
        {0, BANK_BYTES - PAGE, 0x5000000, 0, 0},
        {0, BANK_BYTES + PAGE, 0x5000000, 0, 0},
        {0, ~(word)0, 0x5000000, 0, 0},
        {0, BANK_BYTES, 0, 0, 0},
        {0, BANK_BYTES, 0x1000000 - PAGE, 0, 0},
        {0, BANK_BYTES, 0x5000001, 0, 0},
        {0, BANK_BYTES, 0x1000000, 0, 0},
        {0, BANK_BYTES, 0x2c22000, 0, 0},
        {0, BANK_BYTES, LIMIT - BANK_BYTES + PAGE, 0, 0},
        {0, BANK_BYTES, ~(word)0, 0, 0},
        {0, BANK_BYTES, 0x5000000, 2, 0},
        {0, BANK_BYTES, 0x5000000, ~(word)0, 0},
        {0, BANK_BYTES, 0x5000000, 1, 0},
        {0, BANK_BYTES, 0x5000000, 1, 0x4000001},
        {0, BANK_BYTES, 0x5000000, 1, 0x1000000},
        {0, BANK_BYTES, 0x5000000, 1, 0x2c22000},
        {0, BANK_BYTES, 0x5000000, 1, 0x5000000},
        {0, BANK_BYTES, 0x5000000, 1, 0x5007000},
        {0, BANK_BYTES, 0x5000000, 1, LIMIT},
        {0, BANK_BYTES, 0x5000000, 1, ~(word)0},
    };
    for (word i = 0; i < sizeof(requests) / sizeof(requests[0]); ++i) {
        const struct invalid_request *request = &requests[i];
        fill(region);
        if (topal_x86_image_table_prepare_data_v1((void *)(REGION + PAGE + request->offset),
                request->bytes, request->physical, request->kind, request->low) ||
            !sentinel(region, 0, BANK_BYTES + 2 * PAGE)) {
            if (!failed) failed = 4100 + (long)i;
        }
    }
    fill(region);
    if (topal_x86_image_table_prepare_data_v1((void *)0, BANK_BYTES, 0x5000000, 0, 0) ||
        !sentinel(region, 0, BANK_BYTES + 2 * PAGE)) {
        if (!failed) failed = 4200;
    }
    if (topal_test_syscall3(12, (word)original, 0, 0) != original && !failed) failed = 4201;
    if (!failed) {
        static const char marker[] = "TOPAL_IMAGE_TABLE_DATA_OK\n";
        if (topal_test_syscall3(1, 1, (word)marker, sizeof(marker)-1) != (long)(sizeof(marker)-1))
            failed = 4202;
    }
    return failed;
}

long topal_image_table_preparation_cases(void) {
    long result = image_table_data_checks();
    if (result) {
        static const char digits[] = "0123456789abcdef";
        char message[] = "TOPAL_IMAGE_TABLE_DATA_FAIL:0000\n";
        const word begin = sizeof(message) - 6;
        for (word i = 0; i < 4; ++i)
            message[begin + i] = digits[((word)result >> (12 - i * 4)) & 15];
        (void)topal_test_syscall3(1, 1, (word)message, sizeof(message) - 1);
    }
    return result;
}
