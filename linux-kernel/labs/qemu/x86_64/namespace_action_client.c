/* Independent full-page source-action oracle; never linked into the kernel. */
#include "namespace_component_host.h"

enum { PAGE_BYTES = 4096, INPUT = 3072, NEW_NAME = 3200 };
static unsigned char expected[8192];
static unsigned char original[8192];
static uint64_t get_word(const unsigned char *base, size_t offset) {
    uint64_t value = 0;
    for (unsigned i = 0; i < 8; ++i) value |= (uint64_t)base[offset + i] << (8 * i);
    return value;
}
static void put_word(unsigned char *base, size_t offset, uint64_t value) {
    for (unsigned i = 0; i < 8; ++i) base[offset + i] = (unsigned char)(value >> (8 * i));
}
static size_t object(unsigned slot) { return 256 + 128 * slot; }
static size_t entry(unsigned slot) { return 64 + 16 * slot; }
static size_t name(unsigned slot) { return 1536 + 256 * slot; }
static void publish(unsigned char *base, unsigned slot, uint64_t inode,
                    const unsigned char *text, size_t length) {
    put_word(base, entry(slot), inode);
    put_word(base, entry(slot) + 8, length);
    memset(base + name(slot), 0, 256);
    memcpy(base + name(slot), text, length);
}
static void seed(unsigned char *payload, size_t displacement, unsigned operation,
                 unsigned source, unsigned target, unsigned old, unsigned next, unsigned length) {
    memset(payload, 0xa5, 8192);
    unsigned char *base = payload + displacement;
    memset(base + 64, 0, 64);
    memset(base + 256, 0, 512);
    memset(base + 1536, 0, 1024);
    memset(base + INPUT, 0, 512);
    for (unsigned i = 0; i < 4; ++i) {
        size_t offset = object(i);
        put_word(base, offset, 1);
        put_word(base, offset + 8, 7 + i);
        put_word(base, offset + 16, 89 + 13 * i);
        put_word(base, offset + 24, 0100600 + i);
        put_word(base, offset + 32, 100 + i);
        put_word(base, offset + 40, 200 + i);
        put_word(base, offset + 48, 300 + i);
        put_word(base, offset + 64, 1);
    }
    publish(base, old, 7 + source, (const unsigned char *)"old", 3);
    put_word(base, object(source) + 56, 1);
    put_word(base, object(source) + 64, 0);
    for (unsigned i = 0; i < length; ++i) base[NEW_NAME + i] = (unsigned char)(128 + i % 128);
    if (operation >= 2) {
        publish(base, next, 7 + target, base + NEW_NAME, length);
        put_word(base, object(target) + 56, 1);
        put_word(base, object(target) + 64, 0);
    }
    const uint64_t fields[] = {source, old, operation <= 1 ? 4 : target,
        operation == 1 ? old : next, length, 7 + source, operation <= 1 ? 0 : 7 + target, 1};
    for (unsigned i = 0; i < 8; ++i) put_word(base, INPUT + i * 8, fields[i]);
}
static void oracle(unsigned char *base, unsigned operation, uint64_t seconds) {
    unsigned source = (unsigned)get_word(base, INPUT);
    unsigned old = (unsigned)get_word(base, INPUT + 8);
    unsigned target = (unsigned)get_word(base, INPUT + 16);
    unsigned next = (unsigned)get_word(base, INPUT + 24);
    uint64_t length = get_word(base, INPUT + 32);
    uint64_t identity = get_word(base, INPUT + 40);
    uint64_t victim = get_word(base, INPUT + 48);
    if (operation == 0) {
        publish(base, next, identity, base + NEW_NAME, (size_t)length);
        put_word(base, object(source) + 56, get_word(base, object(source) + 56) + 1);
        put_word(base, object(source) + 64, 0);
    } else if (operation == 1) {
        publish(base, old, identity, base + NEW_NAME, (size_t)length);
    } else if (operation == 2) {
        put_word(base, entry(next), identity);
        memset(base + entry(old), 0, 16);
        memset(base + name(old), 0, 256);
        put_word(base, object(target) + 56, get_word(base, object(target) + 56) - 1);
        put_word(base, object(target) + 48, seconds);
    } else {
        put_word(base, entry(old), victim);
        put_word(base, entry(next), identity);
        put_word(base, object(target) + 48, seconds);
    }
    put_word(base, object(source) + 48, seconds);
    put_word(base, 48, seconds);
    put_word(base, 56, seconds);
}
static void accepted(unsigned char *payload, size_t displacement, unsigned operation, uint64_t seconds) {
    memcpy(expected, payload, sizeof(expected));
    oracle(expected + displacement, operation, seconds);
#ifdef NS_WRONG_ASSOCIATION
    put_word(expected + displacement, 64, 999);
#endif
#ifdef NS_FALSE_RESULT
    invoke(payload + displacement, PAGE_BYTES, 1, operation, seconds, 0);
#else
    invoke(payload + displacement, PAGE_BYTES, 1, operation, seconds, 1);
#endif
    if (memcmp(payload, expected, sizeof(expected))) fail("namespace action missed complete expected page delta");
}
static void refused(unsigned char *payload, size_t displacement, unsigned operation, uint64_t seconds) {
    memcpy(original, payload, sizeof(original));
    if (mprotect(payload, 8192, PROT_READ)) exit(2);
    invoke(payload + displacement, PAGE_BYTES, 1, operation, seconds, 0);
    if (memcmp(payload, original, sizeof(original))) fail("refused namespace action changed owner bytes");
    if (mprotect(payload, 8192, PROT_READ | PROT_WRITE)) exit(2);
}

int main(void) {
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    if (page != PAGE_BYTES) fail("qualification requires 4096-byte host pages");
    unsigned char *mapping = mmap(NULL, 4 * page, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + page, 2 * page, PROT_READ | PROT_WRITE)) exit(2);
    unsigned char *payload = mapping + page;
    for (unsigned placement = 0; placement < 3; ++placement) {
        size_t displacement = placement == 0 ? 0 : placement == 1 ? 1 : page;
        for (unsigned length = 1; length <= 255; ++length) {
            for (unsigned operation = 0; operation < 4; ++operation) {
                unsigned source = length % 4, target = (source + 1) % 4;
                unsigned old = (length + 2) % 4, next = (old + 1) % 4;
                seed(payload, displacement, operation, source, target, old, next, length);
                accepted(payload, displacement, operation, length & 1 ? UINT64_MAX : 0);
            }
#ifdef NS_DEBUG
            if (munmap(mapping, 4 * page)) exit(2);
            printf("NAMESPACE_ACTION_DEBUG_CLIENT_PASS calls=%llu\n", (unsigned long long)calls);
            return 0;
#endif
        }
    }
    for (unsigned source = 0; source < 4; ++source) {
        for (unsigned target = 0; target < 4; ++target) if (source != target) {
            for (unsigned old = 0; old < 4; ++old) {
                for (unsigned next = 0; next < 4; ++next) if (old != next) {
                    for (unsigned operation = 0; operation < 4; ++operation) {
                        seed(payload, 0, operation, source, target, old, next, 7);
                        accepted(payload, 0, operation, 987654321);
                    }
                }
            }
        }
    }
    /* All malformed facts must fail before any store, even on physically read-only backing. */
    const uint64_t wrong[] = {4, 5, 3, 4, 256, 6, 9, 5};
    for (unsigned operation = 0; operation < 4; ++operation) {
        for (unsigned field = 0; field < 8; ++field) {
            seed(payload, 0, operation, 0, 1, 0, 1, 3);
            put_word(payload, INPUT + 8 * field, wrong[field]);
            refused(payload, 0, operation, 70);
        }
        for (unsigned index = 3136; index < 3200; ++index) {
            seed(payload, 0, operation, 0, 1, 0, 1, 3);
            payload[index] = 1;
            refused(payload, 0, operation, 70);
        }
        for (unsigned index = 3456; index < 3584; ++index) {
            seed(payload, 0, operation, 0, 1, 0, 1, 3);
            payload[index] = 1;
            refused(payload, 0, operation, 70);
        }
        for (unsigned kind = 0; kind < 8; ++kind) {
            seed(payload, 0, operation, 0, 1, 0, 1, 3);
            if (kind == 0) put_word(payload, object(0), 0);
            if (kind == 1) put_word(payload, object(0) + 8, 99);
            if (kind == 2) put_word(payload, object(0) + 24, 0040700);
            if (kind == 3) put_word(payload, object(0) + 56, 2);
            if (kind == 4) put_word(payload, object(0) + 64, 1);
            if (kind == 5) put_word(payload, entry(0), 8);
            if (kind == 6) payload[NEW_NAME + 1] = '/';
            if (kind == 7) payload[NEW_NAME + 3] = 1;
            refused(payload, 0, operation, 70);
        }
    }
    for (unsigned operation = 2; operation < 4; ++operation) {
        for (unsigned kind = 0; kind < 8; ++kind) {
            seed(payload, 0, operation, 0, 1, 0, 1, 3);
            if (kind == 0) put_word(payload, object(1), 0);
            if (kind == 1) put_word(payload, object(1) + 8, 99);
            if (kind == 2) put_word(payload, object(1) + 24, 0040700);
            if (kind == 3) put_word(payload, object(1) + 56, 2);
            if (kind == 4) put_word(payload, object(1) + 64, 1);
            if (kind == 5) put_word(payload, entry(1), 99);
            if (kind == 6) put_word(payload, entry(1) + 8, 0);
            if (kind == 7) put_word(payload, INPUT + 16, 0);
            refused(payload, 0, operation, 70);
        }
    }
    /* Replacement retains other victim links; exchange preserves unequal counts. */
    seed(payload, 0, 2, 0, 1, 0, 1, 3);
    publish(payload, 2, 8, (const unsigned char *)"victim-alias", 12);
    publish(payload, 3, 7, (const unsigned char *)"source-alias", 12);
    put_word(payload, object(0) + 56, 2);
    put_word(payload, object(1) + 56, 2);
    put_word(payload, INPUT + 56, 2);
    accepted(payload, 0, 2, 75);
    seed(payload, 0, 3, 0, 1, 0, 1, 3);
    publish(payload, 2, 7, (const unsigned char *)"source-alias", 12);
    put_word(payload, object(0) + 56, 2);
    put_word(payload, INPUT + 56, 2);
    accepted(payload, 0, 3, 76);
    /* Missing-target rename reuses its entry when every slot is occupied. */
    seed(payload, 0, 1, 0, 1, 0, 1, 3);
    for (unsigned i = 1; i < 4; ++i) {
        unsigned char label = (unsigned char)('a' + i);
        publish(payload, i, 7 + i, &label, 1);
        put_word(payload, object(i) + 56, 1);
        put_word(payload, object(i) + 64, 0);
    }
    accepted(payload, 0, 1, 77);
    seed(payload, 0, 0, 0, 1, 0, 1, 3);
    put_word(payload, INPUT + 8, 4);
    accepted(payload, 0, 0, 78);
    /* Identities retain all 64 bits independently of table slots and clock values. */
    for (unsigned operation = 0; operation < 4; ++operation) {
        seed(payload, 0, operation, 0, 1, 0, 1, 3);
        put_word(payload, object(0) + 8, UINT64_MAX);
        put_word(payload, entry(0), UINT64_MAX);
        put_word(payload, INPUT + 40, UINT64_MAX);
        if (operation >= 2) {
            put_word(payload, object(1) + 8, UINT64_MAX - 1);
            put_word(payload, entry(1), UINT64_MAX - 1);
            put_word(payload, INPUT + 48, UINT64_MAX - 1);
        }
        accepted(payload, 0, operation, UINT64_MAX);
    }
    /* Initial anonymous publication consumes eligibility; ineligible unlinked files fail. */
    for (unsigned eligible = 0; eligible < 2; ++eligible) {
        seed(payload, 0, 0, 0, 1, 0, 1, 3);
        memset(payload + entry(0), 0, 16);
        memset(payload + name(0), 0, 256);
        put_word(payload, object(0) + 56, 0);
        put_word(payload, object(0) + 64, eligible);
        put_word(payload, INPUT + 8, 4);
        put_word(payload, INPUT + 56, 0);
        if (eligible) accepted(payload, 0, 0, 80);
        else refused(payload, 0, 0, 80);
    }
    /* Existing aliases change capacity and observed link counts independently of inode slots. */
    seed(payload, 0, 0, 0, 1, 0, 1, 3);
    publish(payload, 2, 7, (const unsigned char *)"alias", 5);
    publish(payload, 3, 7, (const unsigned char *)"third", 5);
    put_word(payload, object(0) + 56, 3);
    put_word(payload, INPUT + 56, 3);
    accepted(payload, 0, 0, 90);
    for (uint64_t extent = 0; extent < PAGE_BYTES; ++extent) invoke(mapping, extent, 1, 0, 0, 0);
    invoke(mapping, PAGE_BYTES, 1, 4, 0, 0);
    invoke(mapping, PAGE_BYTES, 1, UINT64_MAX, 0, 0);
    /* Read-only authority cannot mutate; backing protection also detects accidental stores. */
    seed(payload, 0, 0, 0, 1, 0, 1, 3);
    memcpy(original, payload, sizeof(original));
    if (mprotect(payload, 8192, PROT_READ)) exit(2);
    invoke(payload, PAGE_BYTES, 0, 0, 0, 0);
    if (memcmp(payload, original, sizeof(original))) fail("read-only namespace action changed owner");
    if (munmap(mapping, 4 * page)) exit(2);
#if defined(NS_FALSE_RESULT)
    const char *marker = "NAMESPACE_FALSE_AFTER_STORE_COMPONENT_PASS";
#elif defined(NS_WRONG_ASSOCIATION)
    const char *marker = "NAMESPACE_WRONG_ASSOCIATION_COMPONENT_PASS";
#else
    const char *marker = "NAMESPACE_ACTION_NATIVE_PASS";
#endif
    printf("%s calls=%llu maximum_peak=%llu bound=%llu\n", marker,
        (unsigned long long)calls, (unsigned long long)maximum_peak, (unsigned long long)NS_LIMIT);
    return 0;
}
