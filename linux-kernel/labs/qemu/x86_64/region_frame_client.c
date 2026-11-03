/* Independent caller of a source object. Never linked into a kernel. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>

#ifndef FRAME_WORD_COUNT
#error qualification must supply the declared scalar count
#endif
#ifndef FRAME_SOURCE_LIMIT_BYTES
#error qualification must supply the conservative source allocation bound
#endif

struct Control;
typedef uint64_t (*entry)(struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
extern uint64_t topal_kernel_region_frame_apply_v1(struct Control *, uint64_t, uint64_t,
                                                  const uint64_t *, uint64_t);
struct Control {
    entry code;
    uint64_t owner, generation, active;
    unsigned char *base;
    uint64_t count, writable, issued;
};
_Static_assert(sizeof(struct Control) == 64, "qualified control extent");
_Static_assert(offsetof(struct Control, base) == 32, "qualified owner position");

static _Alignas(16) unsigned char arena[65536];
static uint64_t cursor, peak;
static int active, rejecting;
static struct Control *reject_control;
static struct Control reject_before;
static unsigned char *reject_bytes;
static const uint64_t owner = UINT64_C(0x1122334455667788);

__attribute__((noreturn, noinline)) void topal_kernel_region_frame_abort_v1(void) {
    if (rejecting) {
        if (cursor != 32 || memcmp(reject_control, &reject_before, sizeof(reject_before)))
            _exit(91);
        for (size_t i = 0; i < sizeof(arena); ++i) if (arena[i]) _exit(92);
        for (size_t i = 0; i < 32; ++i) if (reject_bytes[i] != 0xa5) _exit(93);
    }
    _exit(74);
}

__attribute__((noinline)) void *topal_kernel_region_frame_allocate_v1(uint64_t size) {
    if (!active || size > UINT64_MAX - 7) topal_kernel_region_frame_abort_v1();
    size = (size + 7) & ~UINT64_C(7);
    if (!size || cursor > FRAME_SOURCE_LIMIT_BYTES || size > FRAME_SOURCE_LIMIT_BYTES - cursor)
        topal_kernel_region_frame_abort_v1();
    void *result = arena + cursor;
    cursor += size;
    if (cursor > peak) peak = cursor;
    return result;
}

static void prepare(struct Control *control, unsigned char *bytes) {
    uint64_t generation = control->generation;
    *control = (struct Control){.code = topal_kernel_region_frame_apply_v1, .owner = owner,
        .generation = generation, .base = bytes + 8, .count = 16, .writable = 1};
    memset(arena, 0, sizeof(arena));
    cursor = peak = 32;
    active = 1;
}

static int finish(const struct Control *control, uint64_t expected) {
    active = 0;
    if (control->code || control->owner || control->active || control->base || control->count
        || control->writable || control->issued || control->generation != expected) return 1;
    if (peak > FRAME_SOURCE_LIMIT_BYTES) return 1;
    memset(arena, 0, sizeof(arena));
    const volatile unsigned char *readback = arena;
    for (size_t i = 0; i < sizeof(arena); ++i) if (readback[i]) return 1;
    return 0;
}

static uint64_t word_value(unsigned sample, unsigned index) {
    static const uint64_t boundaries[] = {0, 1, 255, 256, UINT32_MAX,
        UINT64_C(1) << 32, INT64_MAX, UINT64_C(1) << 63, UINT64_MAX};
    if (sample < sizeof(boundaries) / sizeof(boundaries[0]))
        return boundaries[(sample + index) % (sizeof(boundaries) / sizeof(boundaries[0]))];
    uint64_t x = UINT64_C(0x9e3779b97f4a7c15) * (sample + 1) + index;
    x ^= x >> 30; x *= UINT64_C(0xbf58476d1ce4e5b9);
    x ^= x >> 27; x *= UINT64_C(0x94d049bb133111eb);
    return x ^ (x >> 31);
}

static int exercise(void) {
    struct Control control = {0};
    _Alignas(8) unsigned char bytes[32];
    for (unsigned sample = 0; sample < 48; ++sample) {
        memset(bytes, 0xa5, sizeof(bytes));
        prepare(&control, bytes);
        uint64_t generation = control.generation + 1;
        size_t page = (size_t)sysconf(_SC_PAGESIZE);
        unsigned char *mapping = NULL;
        uint64_t *words = NULL;
        __uint128_t sum = 0;
        if (FRAME_WORD_COUNT) {
            mapping = mmap(NULL, page * 3, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
            if (mapping == MAP_FAILED || mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) return 2;
            words = (uint64_t *)(mapping + page * 2 - FRAME_WORD_COUNT * 8);
            const unsigned frame_words = FRAME_WORD_COUNT;
            for (unsigned i = 0; i < frame_words; ++i) {
                words[i] = word_value(sample, i);
                sum += (__uint128_t)(i + 1) * words[i];
            }
            if (mprotect(mapping + page, page, PROT_READ)) return 2;
        }
        uint64_t result = topal_kernel_region_frame_apply_v1(&control, owner, generation,
                                                            words, FRAME_WORD_COUNT);
        /* The caller ends the immutable loan and erases its complete staging
           before resuming owner access. The following page stays inaccessible. */
        if (FRAME_WORD_COUNT) {
            if (mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) return 2;
            memset(mapping + page, 0, page);
            const volatile unsigned char *readback = mapping + page;
            for (size_t i = 0; i < page; ++i) if (readback[i]) return 3;
            if (munmap(mapping, page * 3)) return 2;
        }
        if (result != 1 || finish(&control, generation)) return 4;
        unsigned char expected = (unsigned char)(sum % 251);
        for (size_t i = 0; i < sizeof(bytes); ++i) {
            unsigned char wanted = FRAME_WORD_COUNT && i == 8 ? expected : 0xa5;
            if (bytes[i] != wanted) {
                fprintf(stderr, "frame value mismatch: words=%d sample=%u byte=%zu actual=%u expected=%u\n",
                        FRAME_WORD_COUNT, sample, i, bytes[i], wanted);
                return 5;
            }
        }
    }
    return 0;
}

static int refusals(void) {
    for (unsigned test = 0; test < 16; ++test) {
        struct Control control = {0};
        _Alignas(8) unsigned char bytes[32];
        _Alignas(8) uint64_t words[16] = {0};
        memset(bytes, 0xa5, sizeof(bytes));
        prepare(&control, bytes);
        struct Control *argument = &control;
        const uint64_t *frame = FRAME_WORD_COUNT ? words : NULL;
        uint64_t count = FRAME_WORD_COUNT, identity = owner, generation = 1;
        switch (test) {
        case 0: count = FRAME_WORD_COUNT + 1; break;
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
        case 11: frame = FRAME_WORD_COUNT ? NULL : words; break;
        case 12: frame = FRAME_WORD_COUNT ? (const uint64_t *)((const unsigned char *)words + 1) : words; break;
        case 13: frame = FRAME_WORD_COUNT ? (const uint64_t *)(uintptr_t)(UINT64_MAX - 7) : words; break;
        case 14: frame = (const uint64_t *)&control; break;
        case 15: frame = (const uint64_t *)control.base; break;
        }
        reject_control = &control; reject_before = control; reject_bytes = bytes;
        pid_t child = fork();
        if (child < 0) return 6;
        if (!child) {
            rejecting = 1;
            topal_kernel_region_frame_apply_v1(argument, identity, generation, frame, count);
            _exit(90);
        }
        int status = 0;
        if (waitpid(child, &status, 0) != child || !WIFEXITED(status) || WEXITSTATUS(status) != 74) {
            fprintf(stderr, "frame refusal %u failed: %d\n", test, status);
            return 7;
        }
    }
    return 0;
}

int main(void) {
    int result = exercise();
    if (!result) result = refusals();
    if (result) { fprintf(stderr, "frame client failed: %d\n", result); return result; }
    printf("FRAME_NATIVE_PASS words=%d calls=48 refusals=16\n", FRAME_WORD_COUNT);
    return 0;
}
