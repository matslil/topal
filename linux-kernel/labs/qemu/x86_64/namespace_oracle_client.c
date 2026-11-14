/* Native comparison/cleanup helpers plus actual source; not a live kernel grant. */
#include "namespace_component_host.h"
extern uint64_t topal_x86_systems_namespace_prepare_v1(const unsigned char *, uint64_t, uint64_t);
extern uint64_t topal_x86_systems_namespace_check_v1(unsigned char *, uint64_t);
extern unsigned char topal_x86_systems_namespace_expected[4096];

static unsigned char before[8192], expected[8192], stored[8192], saved_oracle[4096];
static uint64_t transactions, refusals, corruptions, overlaps, result_refusals;
static uint64_t word(const unsigned char *p, size_t offset) {
    uint64_t n = 0;
    for (unsigned i = 0; i < 8; ++i) n |= (uint64_t)p[offset + i] << (8 * i);
    return n;
}
static void put(unsigned char *p, size_t offset, uint64_t n) {
    for (unsigned i = 0; i < 8; ++i) p[offset + i] = (unsigned char)(n >> (8 * i));
}
static size_t inode(unsigned slot) { return 256 + 128 * slot; }
static size_t entry(unsigned slot) { return 64 + 16 * slot; }
static size_t name(unsigned slot) { return 1536 + 256 * slot; }
static void publish(unsigned char *p, unsigned slot, uint64_t id, const unsigned char *text, size_t n) {
    put(p, entry(slot), id); put(p, entry(slot) + 8, n);
    memset(p + name(slot), 0, 256); memcpy(p + name(slot), text, n);
}
static void seed(unsigned char *payload, size_t displacement, unsigned op,
                 unsigned src, unsigned dst, unsigned old, unsigned next, unsigned length) {
    memset(payload, 0xa5, 8192);
    unsigned char *p = payload + displacement;
    memset(p, 0, 4096);
    put(p, 0, 11); put(p, 32, 0041777);
    put(p, 40, 22); put(p, 48, 33); put(p, 56, 44);
    for (unsigned i = 0; i < 4; ++i) {
        put(p, inode(i), 1); put(p, inode(i) + 8, 7 + i);
        put(p, inode(i) + 16, 100 + i); put(p, inode(i) + 24, 0100600 + i);
        put(p, inode(i) + 32, 123 + i); put(p, inode(i) + 40, 234 + i);
        put(p, inode(i) + 48, 345 + i); put(p, inode(i) + 64, 1);
    }
    publish(p, old, 7 + src, (const unsigned char *)"old", 3);
    put(p, inode(src) + 56, 1); put(p, inode(src) + 64, 0);
    for (unsigned i = 0; i < length; ++i) p[3200 + i] = (unsigned char)(128 + i % 128);
    if (op >= 2) {
        publish(p, next, 7 + dst, p + 3200, length);
        put(p, inode(dst) + 56, 1); put(p, inode(dst) + 64, 0);
    }
    const uint64_t fields[] = {src, old, op < 2 ? 4 : dst, op == 1 ? old : next,
        length, 7 + src, op < 2 ? 0 : 7 + dst, 1};
    for (unsigned i = 0; i < 8; ++i) put(p, 3072 + 8 * i, fields[i]);
}
/* Independent desired persistent delta. Never used to update the owner in the
 * actual-source cases, and never read from the native expected-state buffer. */
static void oracle(unsigned char *p, unsigned op, uint64_t seconds) {
    unsigned src = (unsigned)word(p, 3072), old = (unsigned)word(p, 3080);
    unsigned dst = (unsigned)word(p, 3088), next = (unsigned)word(p, 3096);
    uint64_t id = word(p, 3112), victim = word(p, 3120);
    if (op == 0) {
        publish(p, next, id, p + 3200, (size_t)word(p, 3104));
        put(p, inode(src) + 56, word(p, inode(src) + 56) + 1);
        put(p, inode(src) + 64, 0);
    } else if (op == 1) {
        publish(p, old, id, p + 3200, (size_t)word(p, 3104));
    } else if (op == 2) {
        put(p, entry(next), id); memset(p + entry(old), 0, 16); memset(p + name(old), 0, 256);
        put(p, inode(dst) + 56, word(p, inode(dst) + 56) - 1); put(p, inode(dst) + 48, seconds);
    } else {
        put(p, entry(old), victim); put(p, entry(next), id); put(p, inode(dst) + 48, seconds);
    }
    put(p, inode(src) + 48, seconds); put(p, 48, seconds); put(p, 56, seconds);
}
static void erased(void) {
    const volatile unsigned char *p = topal_x86_systems_namespace_expected;
    for (unsigned i = 0; i < 4096; ++i) if (p[i]) fail("native comparison snapshot was not erased");
}
static void prepare(unsigned char *payload, size_t displacement, unsigned op, uint64_t seconds) {
    unsigned char *p = payload + displacement;
    memcpy(before, payload, 8192); memcpy(expected, payload, 8192);
    oracle(expected + displacement, op, seconds);
    if (mprotect(payload, 8192, PROT_READ)) exit(2);
    if (topal_x86_systems_namespace_prepare_v1(p, op, seconds) != 1) {
        fprintf(stderr, "prepare op=%u src=%llu dst=%llu old=%llu next=%llu len=%llu\n", op,
            (unsigned long long)word(p, 3072), (unsigned long long)word(p, 3088),
            (unsigned long long)word(p, 3080), (unsigned long long)word(p, 3096),
            (unsigned long long)word(p, 3104));
        fail("native comparison preparation refused an admitted state");
    }
    if (memcmp(payload, before, 8192) || memcmp(topal_x86_systems_namespace_expected,
            expected + displacement, 4096)) fail("native snapshot is not the independent complete delta");
    memcpy(saved_oracle, topal_x86_systems_namespace_expected, 4096);
    if (topal_x86_systems_namespace_prepare_v1(p, op, seconds) != 0
        || memcmp(saved_oracle, topal_x86_systems_namespace_expected, 4096)
        || memcmp(payload, before, 8192)) fail("overlap destroyed a live comparison snapshot");
    ++overlaps;
    if (mprotect(payload, 8192, PROT_READ | PROT_WRITE)) exit(2);
}
static void accepted(unsigned char *payload, size_t displacement, unsigned op, uint64_t seconds) {
    unsigned char *p = payload + displacement;
    prepare(payload, displacement, op, seconds);
#ifdef NS_FALSE_RESULT
    const uint64_t source_result = 0, completion = 0;
#elif defined(NS_WRONG_ASSOCIATION)
    const uint64_t source_result = 1, completion = 0;
#else
    const uint64_t source_result = 1, completion = 1;
#endif
    invoke(p, 4096, 1, op, seconds, source_result);
#ifdef NS_WRONG_ASSOCIATION
    put(expected + displacement, 64, 999);
#endif
    if (memcmp(payload, expected, 8192)) fail("actual source delta differs from independent expected bytes");
    if (topal_x86_systems_namespace_check_v1(p, source_result) != completion)
        fail("native completion accepted a bad result/delta or rejected the actual correct source");
    memset(expected + displacement + 3072, 0, 512);
    if (memcmp(payload, expected, 8192)) fail("completion rolled back persistent writes or missed staging cleanup");
    erased(); ++transactions;
}
static void refused(unsigned char *payload, uint64_t op) {
    erased(); memcpy(before, payload, 8192);
    if (mprotect(payload, 8192, PROT_READ)) exit(2);
    if (topal_x86_systems_namespace_prepare_v1(payload, op, 999) != 0
        || memcmp(before, payload, 8192)) fail("refused native preparation changed the owner");
    erased(); ++refusals;
    if (mprotect(payload, 8192, PROT_READ | PROT_WRITE)) exit(2);
}

int main(void) {
    if (sysconf(_SC_PAGESIZE) != 4096) fail("qualification requires 4096-byte pages");
    uintptr_t snapshot = (uintptr_t)topal_x86_systems_namespace_expected;
    if (snapshot % 4096 || mprotect((void *)(snapshot - 4096), 4096, PROT_NONE)
        || mprotect((void *)(snapshot + 4096), 4096, PROT_NONE)) exit(2);
    unsigned char *mapping = mmap(NULL, 16384, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + 4096, 8192, PROT_READ | PROT_WRITE)) exit(2);
    unsigned char *payload = mapping + 4096;
    for (unsigned placement = 0; placement < 3; ++placement) {
        size_t displacement = placement == 0 ? 0 : placement == 1 ? 1 : 4096;
        for (unsigned length = 1; length <= 255; ++length) for (unsigned op = 0; op < 4; ++op) {
            unsigned src = length % 4, dst = (src + 1) % 4, old = (length + 2) % 4, next = (old + 1) % 4;
            seed(payload, displacement, op, src, dst, old, next, length);
            accepted(payload, displacement, op, length & 1 ? UINT64_MAX : 0);
#ifdef NS_ORACLE_DEBUG
            if (op == 3) {
                if (munmap(mapping, 16384)) exit(2);
                printf("NAMESPACE_NATIVE_ORACLE_DEBUG_CLIENT_PASS calls=%llu\n", (unsigned long long)calls);
                return 0;
            }
#endif
        }
    }
    for (unsigned src = 0; src < 4; ++src) for (unsigned dst = 0; dst < 4; ++dst) if (src != dst)
        for (unsigned old = 0; old < 4; ++old) for (unsigned next = 0; next < 4; ++next) if (old != next)
            for (unsigned op = 0; op < 4; ++op) {
                seed(payload, 0, op, src, dst, old, next, 7); accepted(payload, 0, op, 987654321);
            }
    for (unsigned op = 0; op < 4; ++op) {
        seed(payload, 0, op, 0, 1, 0, 1, 3);
        for (unsigned slot = op < 2 ? 1 : 2; slot < 4; ++slot) memset(payload + inode(slot), 0, 128);
        accepted(payload, 0, op, 89);
        for (unsigned kind = 0; kind < 5; ++kind) {
            const uint64_t results[] = {0, 2, UINT64_MAX, UINT64_C(0x100000001), 1};
            seed(payload, 0, op, 0, 1, 0, 1, 3); prepare(payload, 0, op, 90);
            /* Non-Boolean results refuse even after a correct effect; true
             * without any effect also refuses. Neither path restores bytes. */
            if (kind != 4) memcpy(payload, expected, 8192);
            memcpy(stored, payload, 8192); memset(stored + 3072, 0, 512);
            if (topal_x86_systems_namespace_check_v1(payload, results[kind]) != 0
                || memcmp(payload, stored, 8192)) fail("invalid/empty source completion resumed or rolled back");
            erased(); ++result_refusals;
        }
        for (unsigned offset = 0; offset < 4096; ++offset) {
            seed(payload, 0, op, 0, 1, 0, 1, 3); prepare(payload, 0, op, 90);
            /* Isolated native CHECK counterexample: supply the independent
             * desired effect, then corrupt EACH possible byte, including input. */
            memcpy(payload, expected, 8192); payload[offset] ^= 1;
            memcpy(stored, payload, 8192); memset(stored + 3072, 0, 512);
            if (topal_x86_systems_namespace_check_v1(payload, 1) != 0
                || memcmp(payload, stored, 8192)) fail("whole-page mismatch resumed or rewrote persistent state");
            erased(); ++corruptions;
        }
        for (unsigned field = 0; field < 8; ++field) {
            const uint64_t bad[] = {4, 5, op < 2 ? 3 : 4, 4, 256, 6, op < 2 ? 8 : 7, 5};
            seed(payload, 0, op, 0, 1, 0, 1, 3); put(payload, 3072 + 8 * field, bad[field]); refused(payload, op);
        }
        for (unsigned offset = 3136; offset < 3200; ++offset) {
            seed(payload, 0, op, 0, 1, 0, 1, 3); payload[offset] = 1; refused(payload, op);
        }
        for (unsigned offset = 3456; offset < 3584; ++offset) {
            seed(payload, 0, op, 0, 1, 0, 1, 3); payload[offset] = 1; refused(payload, op);
        }
    }
    /* Full owner layout, name uniqueness and page association refusals. */
    const unsigned reserved[] = {8, 23, 128, 255, 328, 383, 768, 1023, 1280, 1535, 2560, 3071, 3584, 4095};
    for (unsigned i = 0; i < sizeof(reserved) / sizeof(reserved[0]); ++i) {
        seed(payload, 0, 0, 0, 1, 0, 1, 3); payload[reserved[i]] = 1; refused(payload, 0);
    }
    const unsigned malformed[] = {0, 32, 256, 264, 272, 280, 312, 320, 72, 1024, 1032, 1040, 24};
    const uint64_t values[] = {6, 0100600, 2, 11, UINT64_MAX, 0040700, 2, 1, 0, 7, 1, 1, 1};
    for (unsigned i = 0; i < sizeof(malformed) / sizeof(malformed[0]); ++i) {
        seed(payload, 0, 0, 0, 1, 0, 1, 3); put(payload, malformed[i], values[i]); refused(payload, 0);
    }
    for (unsigned kind = 0; kind < 4; ++kind) {
        seed(payload, 0, 0, 0, 1, 0, 1, 3);
        payload[24] = payload[25] = 1;
        put(payload, 1024, 7); put(payload, 1056, 7); put(payload, 1064, 1);
        if (kind == 0) put(payload, 1064, 0);
        if (kind == 1) payload[25] = 0;
        if (kind == 2) put(payload, 1056, 11);
        if (kind == 3) put(payload, 1064, UINT64_MAX);
        refused(payload, 0);
    }
    seed(payload, 0, 0, 0, 1, 0, 1, 3);
    publish(payload, 2, 7, (const unsigned char *)"old", 3); put(payload, inode(0) + 56, 2);
    put(payload, 3128, 2); refused(payload, 0);
    /* Existing destination, invalid raw input and malformed full-width words. */
    for (unsigned kind = 0; kind < 7; ++kind) {
        seed(payload, 0, 0, 0, 1, 0, 1, 3);
        if (kind == 0) memcpy(payload + 3200, "old", 3);
        if (kind == 1) payload[3201] = '/';
        if (kind == 2) payload[3201] = 0;
        if (kind == 3) payload[3203] = 1;
        if (kind == 4) { memset(payload + 3200, 0, 256); payload[3200] = '.'; put(payload, 3104, 1); }
        if (kind == 5) { memset(payload + 3200, 0, 256); memcpy(payload + 3200, "..", 2); put(payload, 3104, 2); }
        if (kind == 6) put(payload, 3072, UINT64_C(0x100000000));
        refused(payload, 0);
    }
    for (unsigned op = 0; op < 4; ++op) {
        seed(payload, 0, op, 0, 1, 0, 1, 3);
        put(payload, 0, UINT64_MAX); put(payload, inode(0) + 8, UINT64_MAX - 1);
        put(payload, entry(0), UINT64_MAX - 1); put(payload, 3112, UINT64_MAX - 1);
        if (op >= 2) { put(payload, inode(1) + 8, UINT64_MAX - 2); put(payload, entry(1), UINT64_MAX - 2); put(payload, 3120, UINT64_MAX - 2); }
        accepted(payload, 0, op, UINT64_MAX);
    }
    /* Anonymous first link, full directory rename, and resident associations. */
    seed(payload, 0, 2, 0, 1, 0, 1, 3);
    publish(payload, 2, 8, (const unsigned char *)"victim-alias", 12);
    publish(payload, 3, 7, (const unsigned char *)"source-alias", 12);
    put(payload, inode(0) + 56, 2); put(payload, inode(1) + 56, 2); put(payload, 3128, 2);
    accepted(payload, 0, 2, 97);
    seed(payload, 0, 3, 0, 1, 0, 1, 3);
    publish(payload, 2, 7, (const unsigned char *)"source-alias", 12);
    put(payload, inode(0) + 56, 2); put(payload, 3128, 2);
    accepted(payload, 0, 3, 98);
    seed(payload, 0, 0, 0, 1, 0, 1, 3); put(payload, 3080, 4);
    accepted(payload, 0, 0, 98);
    seed(payload, 0, 0, 0, 1, 0, 1, 3);
    publish(payload, 2, 7, (const unsigned char *)"alias", 5);
    publish(payload, 3, 7, (const unsigned char *)"third", 5);
    put(payload, inode(0) + 56, 3); put(payload, 3128, 3);
    accepted(payload, 0, 0, 98);
    seed(payload, 0, 0, 0, 1, 0, 1, 3);
    memset(payload + entry(0), 0, 16); memset(payload + name(0), 0, 256);
    put(payload, inode(0) + 56, 0); put(payload, inode(0) + 64, 1); put(payload, 3080, 4); put(payload, 3128, 0);
    accepted(payload, 0, 0, 99);
    seed(payload, 0, 1, 0, 1, 0, 1, 3);
    for (unsigned i = 1; i < 4; ++i) { unsigned char label = 'a' + i; publish(payload, i, 7 + i, &label, 1); put(payload, inode(i) + 56, 1); put(payload, inode(i) + 64, 0); }
    for (unsigned i = 0; i < 8; ++i) { payload[24 + i] = 1; put(payload, 1024 + 32 * i, 7 + i % 4); put(payload, 1032 + 32 * i, i / 4); }
    accepted(payload, 0, 1, 100);
    if (topal_x86_systems_namespace_prepare_v1(mapping, 4, 0) != 0
        || topal_x86_systems_namespace_prepare_v1(mapping, UINT64_MAX, 0) != 0)
        fail("invalid operation dereferenced inaccessible owner");
    erased();
    if (munmap(mapping, 16384)) exit(2);
    printf("NAMESPACE_NATIVE_ORACLE_PASS transactions=%llu refusals=%llu corruptions=%llu overlaps=%llu source-calls=%llu result-refusals=%llu\n",
        (unsigned long long)transactions, (unsigned long long)refusals, (unsigned long long)corruptions,
        (unsigned long long)overlaps, (unsigned long long)calls, (unsigned long long)result_refusals);
    return 0;
}
