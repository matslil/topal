/* Independent native source-object caller. Never linked into a kernel. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>

struct Control;
extern uint64_t topal_kernel_region_action_apply_v1(struct Control *, uint64_t, uint64_t, uint64_t, uint64_t);
struct Control {
    uint64_t (*code)(struct Control *, uint64_t, uint64_t, uint64_t, uint64_t);
    uint64_t owner, generation, active;
    unsigned char *base;
    uint64_t count, writable, issued;
};
_Static_assert(sizeof(struct Control) == 64, "qualified private control extent");
_Static_assert(offsetof(struct Control, generation) == 16, "generation position");
_Static_assert(offsetof(struct Control, base) == 32, "owned bytes position");
_Static_assert(offsetof(struct Control, issued) == 56, "view budget position");

static _Alignas(16) unsigned char arena[65536];
static uint64_t cursor, peak;
static int allocator_active;
static const uint64_t owner_identity = UINT64_C(0x1122334455667788);
#ifndef REGION_SOURCE_LIMIT_BYTES
#error qualification must supply the conservative source allocation bound
#endif

__attribute__((noreturn, noinline)) void topal_kernel_region_action_abort_v1(void) {
    _exit(74);
}

/* Normal allocation is a leaf over explicitly lent static temporary storage.
   Host termination on a bad contract is a test oracle, not a kernel fatal hook. */
__attribute__((noinline)) void *topal_kernel_region_action_allocate_v1(uint64_t size) {
    if (!allocator_active || size > UINT64_MAX - 7)
        topal_kernel_region_action_abort_v1();
    size = (size + 7) & ~UINT64_C(7);
    if (!size || cursor > REGION_SOURCE_LIMIT_BYTES || size > REGION_SOURCE_LIMIT_BYTES - cursor)
        topal_kernel_region_action_abort_v1();
    void *result = arena + cursor;
    cursor += size;
    if (cursor > peak) peak = cursor;
    return result;
}

static void prepare(struct Control *control, unsigned char *bytes,
                    uint64_t count, uint64_t writable) {
    uint64_t generation = control->generation;
    *control = (struct Control){
        .code = topal_kernel_region_action_apply_v1, .owner = owner_identity,
        .generation = generation, .base = bytes, .count = count,
        .writable = writable,
    };
    memset(arena, 0, sizeof(arena));
    cursor = 32; /* Includes the declared temporary-runtime header reserve. */
    peak = cursor;
    allocator_active = 1;
}

static int finish(const struct Control *control, uint64_t generation) {
    allocator_active = 0;
    if (control->code || control->owner || control->active || control->base
        || control->count || control->writable || control->issued
        || control->generation != generation) {
        fprintf(stderr, "incomplete native invocation revocation\n");
        return 1;
    }
    memset(arena, 0, sizeof(arena));
    const volatile unsigned char *readback = arena;
    for (size_t i = 0; i < sizeof(arena); ++i)
        if (readback[i]) return 1;
    return 0;
}

static int dispositions(struct Control *control, unsigned char *bytes) {
    for (int test = 0; test < 12; ++test) {
        control->generation = 0;
        prepare(control, bytes, 16, 1);
        uint64_t expected_owner = owner_identity, generation = 1;
        struct Control *argument = control;
        switch (test) {
        case 0: control->code = NULL; break;
        case 1: control->owner = 0; break;
        case 2: expected_owner ^= 1; break;
        case 3: generation = 2; break;
        case 4: control->generation = UINT64_MAX; generation = 0; break;
        case 5: control->active = 1; break;
        case 6: control->issued = 1; break;
        case 7: control->count = 65537; break;
        case 8: control->writable = 2; break;
        case 9: argument = NULL; break;
        case 10: control->base = (unsigned char *)control; control->count = 64; break;
        case 11: control->base = (unsigned char *)(uintptr_t)(UINT64_MAX - 3); control->count = 8; break;
        }
        struct Control before = *control;
        unsigned char saved[16];
        memcpy(saved, bytes, sizeof(saved));
        pid_t child = fork();
        if (child < 0) return 1;
        if (!child) {
            topal_kernel_region_action_apply_v1(argument, expected_owner, generation, 1, 1);
            _exit(99);
        }
        int status = 0;
        if (waitpid(child, &status, 0) != child || !WIFEXITED(status)
            || WEXITSTATUS(status) != 74 || memcmp(&before, control, sizeof(before))
            || memcmp(saved, bytes, sizeof(saved))) {
            fprintf(stderr, "wrong pre-entry disposition/publication at case %d\n", test);
            return 1;
        }
    }
    return 0;
}

int main(void) {
    const size_t mapping_bytes = 65536 + 128;
    unsigned char *mapping = mmap(NULL, mapping_bytes, PROT_READ | PROT_WRITE,
                                 MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    struct Control *control = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                                  MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || control == MAP_FAILED) return 1;
    unsigned char *bytes = mapping + 64;
    static const uint64_t extents[] = {0, 1, 7, 8, 9, 16, 65536};
    static const uint64_t inputs[][2] = {
        {0, 0}, {0, 1}, {1, 2}, {UINT64_C(0x0102030405060700), 8},
        {INT64_MAX, 0}, {INT64_MAX - 1, 1}, {INT64_MAX, 1},
        {UINT64_MAX, 0}, {UINT64_MAX, 1}, {UINT64_MAX, UINT64_MAX},
        {UINT32_MAX, 1}, {UINT64_C(1) << 63, 0},
    };
#ifndef ACTION_WORD_LIMIT
#define ACTION_WORD_LIMIT INT64_MAX
#endif
    uint64_t largest_peak = 0;
    unsigned calls = 0;
    for (uint64_t writable = 0; writable <= 1; ++writable) {
        for (size_t run = 0; run < sizeof(extents) / sizeof(extents[0]); ++run) {
            uint64_t count = extents[run];
            for (size_t sample = 0; sample < sizeof(inputs) / sizeof(inputs[0]); ++sample) {
                uint64_t base = inputs[sample][0], transferred = inputs[sample][1];
                memset(mapping, 0xa7, mapping_bytes);
                unsigned char expected[65536 + 128];
                memcpy(expected, mapping, sizeof(expected));
                __uint128_t sum = (__uint128_t)base + transferred;
                uint64_t accepted = writable && count >= 8 && sum <= ACTION_WORD_LIMIT;
                if (accepted) {
                    uint64_t word = (uint64_t)sum;
                    for (unsigned byte = 0; byte < 8; ++byte)
                        expected[64 + byte] = (unsigned char)(word >> (byte * 8));
                }
                prepare(control, bytes, count, writable);
                int permissions = !count ? PROT_NONE : writable ? PROT_READ | PROT_WRITE : PROT_READ;
                if (mprotect(mapping, mapping_bytes, permissions)) return 1;
                uint64_t generation = control->generation + 1;
                uint64_t result = topal_kernel_region_action_apply_v1(control, owner_identity,
                                                                     generation, base, transferred);
                uint64_t used = peak;
                if (finish(control, generation)) return 1;
                if (mprotect(mapping, mapping_bytes, PROT_READ | PROT_WRITE)) return 1;
                if (result != accepted || memcmp(mapping, expected, sizeof(expected))) {
                    fprintf(stderr, "incorrect action result/bytes: extent=%llu write=%llu sample=%zu expected=%llu observed=%llu\n",
                            (unsigned long long)count, (unsigned long long)writable, sample,
                            (unsigned long long)accepted, (unsigned long long)result);
                    return 1;
                }
                if (used > largest_peak) largest_peak = used;
                ++calls;
            }
        }
    }
    if (dispositions(control, bytes)) return 1;
    printf("REGION_ACTION_OK calls=%u rejected=12 peak=%llu\n", calls, (unsigned long long)largest_peak);
    if (munmap(mapping, mapping_bytes) || munmap(control, 4096)) return 1;
    return 0;
}
