/* Independent source-component caller; never linked into a kernel. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>

#if !defined(INODE_EXPORT) || !defined(INODE_ALLOCATE) || !defined(INODE_ABORT) || !defined(INODE_SOURCE_LIMIT)
#error qualification must bind the actual instance symbols and complete bound
#endif
struct Control;
typedef uint64_t (*entry)(struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
extern uint64_t INODE_EXPORT(struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
struct Control {
    entry code;
    uint64_t owner, generation, active;
    unsigned char *base;
    uint64_t count, writable, issued;
};
_Static_assert(sizeof(struct Control) == 64, "qualified control extent");
static _Alignas(16) unsigned char arena[65536];
static struct Control control;
static uint64_t cursor, peak, maximum_peak, calls;
static int active, rejecting;
static const uint64_t owner = UINT64_C(0x1122334455667788);
static struct Control reject_before;
static unsigned char *reject_payload;
static unsigned char reject_bytes[32];
/* Deliberately visible for independent debugger observations. */
static unsigned char *debug_payload;
static uint64_t debug_words[2];
static volatile uint64_t debug_expected;
static volatile uint64_t erased_calls;

__attribute__((noreturn, noinline)) void INODE_ABORT(void) {
    if (rejecting) {
        if (cursor != 32 || memcmp(&control, &reject_before, sizeof(control))) _exit(91);
        for (size_t i = 0; i < sizeof(arena); ++i) if (arena[i]) _exit(92);
        if (memcmp(reject_payload, reject_bytes, 32)) _exit(93);
    }
    _exit(74);
}

__attribute__((noinline)) void *INODE_ALLOCATE(uint64_t size) {
    if (!active || size > UINT64_MAX - 7) INODE_ABORT();
    size = (size + 7) & ~UINT64_C(7);
    if (!size || cursor > INODE_SOURCE_LIMIT || size > INODE_SOURCE_LIMIT - cursor) INODE_ABORT();
    void *result = arena + cursor;
    cursor += size;
    if (cursor > peak) peak = cursor;
    if (peak > maximum_peak) maximum_peak = peak;
    return result;
}

__attribute__((noinline)) static void erased_checkpoint(void) { ++erased_calls; }

static void encode(unsigned char *bytes, uint64_t value) {
    /* Independent division oracle; no call to the compiler's field encoder. */
    for (unsigned i = 0; i < 8; ++i) {
        bytes[i] = (unsigned char)(value % 256);
        value /= 256;
    }
}

static void initialize(unsigned char *base, uint64_t mode) {
    encode(base, mode);
    encode(base + 8, UINT64_C(0x0102030405060708));
    encode(base + 16, UINT64_C(0x1112131415161718));
    encode(base + 24, UINT64_C(0x2122232425262728));
}

static void invoke(unsigned char *base, uint64_t count, uint64_t rights,
                   uint64_t permissions, uint64_t seconds, uint64_t expected) {
    uint64_t generation = control.generation + 1;
    control = (struct Control){.code = INODE_EXPORT, .owner = owner,
        .generation = generation - 1, .base = base, .count = count, .writable = rights};
    memset(arena, 0, sizeof(arena));
    cursor = peak = 32; active = 1;
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    unsigned char *mapping = mmap(NULL, page * 3, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) exit(2);
    uint64_t *words = (uint64_t *)(mapping + 2 * page - 16);
    words[0] = permissions; words[1] = seconds;
    debug_payload = base; debug_words[0] = permissions; debug_words[1] = seconds;
    debug_expected = expected;
    if (mprotect(mapping + page, page, PROT_READ)) exit(2);
    uint64_t result = INODE_EXPORT(&control, owner, generation, words, 2);
    active = 0;
    if (control.code || control.owner || control.active || control.base || control.count
        || control.writable || control.issued || control.generation != generation
        || peak > INODE_SOURCE_LIMIT || result != expected) {
        fprintf(stderr, "inode result mismatch: count=%llu rights=%llu permissions=%llu result=%llu expected=%llu\n",
                (unsigned long long)count, (unsigned long long)rights, (unsigned long long)permissions,
                (unsigned long long)result, (unsigned long long)expected);
        exit(3);
    }
    if (mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) exit(2);
    memset(mapping + page, 0, page);
    memset(arena, 0, sizeof(arena));
    const volatile unsigned char *readback = arena;
    for (size_t i = 0; i < sizeof(arena); ++i) if (readback[i]) exit(4);
    readback = mapping + page;
    for (size_t i = 0; i < page; ++i) if (readback[i]) exit(4);
    erased_checkpoint();
    if (munmap(mapping, page * 3)) exit(2);
    ++calls;
}

static void valid_cases(unsigned char *page, size_t size) {
    const uint64_t kinds[] = {0100000, 0040000, 0010000};
    for (unsigned edge = 0; edge < 3; ++edge) {
        unsigned char *base = edge == 0 ? page : edge == 1 ? page + 1 : page + size - 32;
        for (unsigned kind = 0; kind < 3; ++kind) {
            for (uint64_t permissions = 0; permissions <= 07777; ++permissions) {
                unsigned char expected[size];
                memset(page, 0xa5, size);
                initialize(base, kinds[kind] | (07777 - permissions));
                memcpy(expected, page, size);
                uint64_t seconds = UINT64_MAX - permissions;
                encode(expected + (base - page), kinds[kind] | permissions);
                encode(expected + (base - page) + 24, seconds);
                invoke(base, 32, 1, permissions, seconds, 1);
                if (memcmp(page, expected, size)) {
                    fprintf(stderr, "incorrect source inode bytes: edge=%u kind=%u permissions=%llu\n",
                            edge, kind, (unsigned long long)permissions);
                    exit(5);
                }
            }
        }
    }
}

static void data_refusals(unsigned char *page, size_t size, unsigned char *inaccessible) {
    for (uint64_t count = 0; count < 32; ++count) invoke(inaccessible, count, 1, 0, 0, 0);
    invoke(inaccessible, 32, 1, 4096, 0, 0);
    invoke(inaccessible, 32, 1, UINT64_MAX, 0, 0);
    const uint64_t modes[] = {0, 07777, 0020600, 0060600, 0120600, 0140600, UINT64_MAX};
    unsigned char before[size];
    for (size_t i = 0; i < sizeof(modes) / sizeof(modes[0]); ++i) {
        memset(page, 0xa5, size); initialize(page, modes[i]); memcpy(before, page, size);
        if (mprotect(page, size, PROT_READ)) exit(2);
        invoke(page, 32, 1, 0777, 100, 0);
        if (memcmp(page, before, size)) exit(6);
        if (mprotect(page, size, PROT_READ | PROT_WRITE)) exit(2);
    }
    initialize(page, 0100600); memcpy(before, page, size);
    if (mprotect(page, size, PROT_READ)) exit(2);
    invoke(page, 32, 0, 0777, 100, 0);
    if (memcmp(page, before, size)) exit(6);
    if (mprotect(page, size, PROT_READ | PROT_WRITE)) exit(2);
}

static void abi_refusals(void) {
    for (unsigned test = 0; test < 16; ++test) {
        _Alignas(8) unsigned char bytes[32];
        _Alignas(8) uint64_t words[2] = {0755, 100};
        initialize(bytes, 0100644);
        control = (struct Control){.code = INODE_EXPORT, .owner = owner,
            .base = bytes, .count = 32, .writable = 1};
        memset(arena, 0, sizeof(arena)); cursor = peak = 32; active = 1;
        struct Control *argument = &control;
        const uint64_t *frame = words;
        uint64_t count = 2, identity = owner, generation = 1;
        switch (test) {
        case 0: count = 1; break;
        case 1: count = UINT64_MAX; break;
        case 2: control.code = NULL; break;
        case 3: identity ^= 1; break;
        case 4: generation = 2; break;
        case 5: control.generation = UINT64_MAX; generation = 0; break;
        case 6: control.active = 1; break;
        case 7: control.issued = 1; break;
        case 8: argument = NULL; break;
        case 9: control.count = 65537; break;
        case 10: control.writable = 2; break;
        case 11: frame = NULL; break;
        case 12: frame = (const uint64_t *)((const unsigned char *)words + 1); break;
        case 13: frame = (const uint64_t *)(uintptr_t)(UINT64_MAX - 7); break;
        case 14: frame = (const uint64_t *)&control; break;
        case 15: frame = (const uint64_t *)control.base; break;
        }
        reject_before = control; reject_payload = bytes; memcpy(reject_bytes, bytes, 32);
        pid_t child = fork();
        if (child < 0) exit(2);
        if (!child) { rejecting = 1; INODE_EXPORT(argument, identity, generation, frame, count); _exit(90); }
        int status = 0;
        if (waitpid(child, &status, 0) != child || !WIFEXITED(status) || WEXITSTATUS(status) != 74) {
            fprintf(stderr, "inode ABI refusal %u failed: %d\n", test, status); exit(7);
        }
    }
}

static void debug_cases(unsigned char *page, size_t size, unsigned char *inaccessible) {
    unsigned char *base = page + size - 32;
    const uint64_t kinds[] = {0100000, 0040000, 0010000};
    const uint64_t permissions[] = {0755, 02750, 0};
    for (unsigned i = 0; i < 3; ++i) {
        initialize(base, kinds[i] | 0600);
        invoke(base, 32, 1, permissions[i], UINT64_MAX - i, 1);
    }
    initialize(base, 0100644);
    if (mprotect(page, size, PROT_READ)) exit(2);
    invoke(base, 32, 0, 0755, 100, 0);
    if (mprotect(page, size, PROT_READ | PROT_WRITE)) exit(2);
    initialize(base, 0140600); invoke(base, 32, 1, 0755, 100, 0);
    invoke(inaccessible, 31, 1, 0755, 100, 0);
    invoke(inaccessible, 32, 1, 4096, 100, 0);
    invoke(inaccessible, 32, 1, UINT64_MAX, 100, 0);
    initialize(base, 0100755); encode(base + 24, 100);
    invoke(base, 32, 1, 0755, 100, 1);
}

int main(int argc, char **argv) {
    size_t size = (size_t)sysconf(_SC_PAGESIZE);
    unsigned char *mapping = mmap(NULL, size * 3, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + size, size, PROT_READ | PROT_WRITE)) return 2;
    if (argc == 2 && !strcmp(argv[1], "debug")) debug_cases(mapping + size, size, mapping);
    else if (argc == 2 && !strcmp(argv[1], "wide")) invoke(mapping, 32, 1, 0, 0, 0);
    else if (argc == 2 && !strcmp(argv[1], "false-after-mode")) {
        unsigned char *base = mapping + size;
        unsigned char expected[32];
        initialize(base, 0100644); memcpy(expected, base, 32); encode(expected, 0100755);
        invoke(base, 32, 1, 0755, 100, 0);
        if (memcmp(base, expected, 32)) return 8;
    }
    else { valid_cases(mapping + size, size); data_refusals(mapping + size, size, mapping); abi_refusals(); }
    if (munmap(mapping, size * 3)) return 2;
    printf("INODE_NATIVE_PASS calls=%llu peak=%llu bound=%llu\n", (unsigned long long)calls,
           (unsigned long long)maximum_peak, (unsigned long long)INODE_SOURCE_LIMIT);
    return 0;
}
