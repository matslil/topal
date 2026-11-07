/* Independent encoding/refusal oracle. Component caller, never a kernel input. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>

#if !defined(FIELD_WIDTH) || !defined(FIELD_BIG_ENDIAN) || !defined(FIELD_SOURCE_LIMIT)
#error qualification must supply the encoding and complete allocation bound
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
static _Alignas(16) unsigned char arena[65536];
static uint64_t cursor, peak, calls;
static int active;
static const uint64_t owner = UINT64_C(0x1122334455667788);
/* Exported C variables let the debugger inspect the actual loan and cleanup. */
static struct Control control;
static unsigned char *debug_payload;
static uint64_t debug_words[3];
static volatile uint64_t erased_calls;

__attribute__((noreturn, noinline)) void topal_kernel_region_frame_abort_v1(void) { _exit(74); }
__attribute__((noinline)) void *topal_kernel_region_frame_allocate_v1(uint64_t size) {
    if (!active || size > UINT64_MAX - 7) topal_kernel_region_frame_abort_v1();
    size = (size + 7) & ~UINT64_C(7);
    if (!size || cursor > FIELD_SOURCE_LIMIT || size > FIELD_SOURCE_LIMIT - cursor)
        topal_kernel_region_frame_abort_v1();
    void *result = arena + cursor;
    cursor += size;
    if (cursor > peak) peak = cursor;
    return result;
}

__attribute__((noinline)) static void erased_checkpoint(void) { ++erased_calls; }

static uint64_t invoke(unsigned char *base, uint64_t count, uint64_t rights,
                       uint64_t offset, uint64_t value, uint64_t command) {
    uint64_t generation = control.generation + 1;
    control = (struct Control){.code = topal_kernel_region_frame_apply_v1, .owner = owner,
        .generation = generation - 1, .base = base, .count = count, .writable = rights};
    memset(arena, 0, sizeof(arena));
    cursor = peak = 32; active = 1;
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    unsigned char *mapping = mmap(NULL, page * 3, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) exit(2);
    uint64_t *words = (uint64_t *)(mapping + 2 * page - 24);
    words[0] = offset; words[1] = value; words[2] = command;
    debug_words[0] = offset; debug_words[1] = value; debug_words[2] = command;
    debug_payload = base;
    if (mprotect(mapping + page, page, PROT_READ)) exit(2);
    uint64_t result = topal_kernel_region_frame_apply_v1(&control, owner, generation, words, 3);
    active = 0;
    if (control.code || control.owner || control.active || control.base || control.count
        || control.writable || control.issued || control.generation != generation
        || peak > FIELD_SOURCE_LIMIT || result > 1) exit(3);
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
    return result;
}

static void encode(unsigned char *bytes, uint64_t value) {
    /* Independent division oracle, rather than the implementation's shifts. */
    for (unsigned i = 0; i < FIELD_WIDTH; ++i) {
        unsigned place = FIELD_BIG_ENDIAN ? FIELD_WIDTH - 1 - i : i;
        bytes[place] = (unsigned char)(value % 256);
        value /= 256;
    }
}

static void valid_cases(unsigned char *page, size_t size) {
    const uint64_t maximum = UINT64_MAX >> ((8 - FIELD_WIDTH) * 8);
    for (unsigned edge = 0; edge < 2; ++edge) {
        unsigned char *base = edge ? page + size - 23 : page;
        for (unsigned offset = 0; offset <= 23; ++offset) {
            for (unsigned sample = 0; sample < FIELD_WIDTH * 8 + 4; ++sample) {
                uint64_t value = sample == 0 ? 0 : sample == 1 ? maximum
                    : sample == 2 ? UINT64_C(0x814e6302a597c8fb) & maximum
                    : sample == 3 ? UINT64_C(0x0123456789abcdef) & maximum
                    : UINT64_C(1) << (sample - 4);
                unsigned char expected[size];
                memset(page, 0xa5, size); memset(expected, 0xa5, size);
                int fits = offset <= 23 - FIELD_WIDTH;
                if (fits) encode(expected + (base - page) + offset, value);
                if (invoke(base, 23, 1, offset, value, 0) != (uint64_t)fits
                    || memcmp(page, expected, size)) exit(5);
                if (invoke(base, 23, 1, offset, value, 1) != (uint64_t)fits
                    || memcmp(page, expected, size)) exit(6);
                if (mprotect(page, size, PROT_READ)) exit(2);
                if (invoke(base, 23, 0, offset, value, 1) != (uint64_t)fits
                    || memcmp(page, expected, size)) exit(6);
                if (mprotect(page, size, PROT_READ | PROT_WRITE)) exit(2);
            }
        }
    }
}

static void inaccessible_refusals(unsigned char *inaccessible) {
    /* Any payload read or write here faults. The view/control remain valid. */
    for (uint64_t command = 0; command <= 6; ++command) {
        uint64_t offset = command < 2 ? 9 - FIELD_WIDTH : 0;
        if (invoke(inaccessible, 8, 1, offset, 0, command)) exit(7);
    }
    if (invoke(inaccessible, 8, 0, 0, 0, 0)) exit(7);
    for (uint64_t offset = 0; offset <= 8; ++offset) {
        if (invoke(inaccessible, 0, 1, offset, 0, 0)
            || invoke(inaccessible, 0, 1, offset, 0, 1)) exit(7);
    }
    if (FIELD_WIDTH < 8) {
        uint64_t too_large = UINT64_C(1) << (FIELD_WIDTH * 8 % 64);
        if (invoke(inaccessible, 8, 1, 0, too_large, 0)) exit(7);
    }
    if (invoke(inaccessible, 8, 1, UINT64_MAX, 0, 0)
        || invoke(inaccessible, 8, 1, UINT64_MAX, 0, 1)) exit(7);
}

int main(int argc, char **argv) {
    size_t size = (size_t)sysconf(_SC_PAGESIZE);
    unsigned char *mapping = mmap(NULL, size * 3, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + size, size, PROT_READ | PROT_WRITE)) return 2;
    if (argc == 2 && !strcmp(argv[1], "debug")) {
        unsigned char *base = mapping + 2 * size - FIELD_WIDTH;
        if (invoke(base, FIELD_WIDTH, 1, 0, UINT64_C(0x814e6302a597c8fb), 0) != 1) return 8;
        if (invoke(base, FIELD_WIDTH, 1, 0, UINT64_C(0x814e6302a597c8fb), 1) != 1) return 8;
        inaccessible_refusals(mapping);
    } else {
        valid_cases(mapping + size, size);
        inaccessible_refusals(mapping);
    }
    if (munmap(mapping, size * 3)) return 2;
    printf("UNSIGNED_NATIVE_PASS width=%d big=%d calls=%llu\n", FIELD_WIDTH, FIELD_BIG_ENDIAN,
           (unsigned long long)calls);
    return 0;
}
