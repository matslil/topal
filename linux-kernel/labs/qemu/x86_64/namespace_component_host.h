#ifndef TOPAL_NAMESPACE_COMPONENT_HOST_H
#define TOPAL_NAMESPACE_COMPONENT_HOST_H
/* Independent component oracle; never linked into the kernel. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#if !defined(NS_EXPORT) || !defined(NS_ALLOCATE) || !defined(NS_ABORT) || !defined(NS_LIMIT)
#error qualify the actual source instance and complete allocation bound
#endif
struct Control;
typedef uint64_t (*Entry)(struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
extern uint64_t NS_EXPORT(struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
struct Control {
    Entry code;
    uint64_t owner, generation, active;
    unsigned char *base;
    uint64_t count, writable, issued;
};
_Static_assert(sizeof(struct Control) == 64, "qualified view control");
static _Alignas(16) unsigned char arena[65536];
static struct Control control;
static uint64_t cursor, peak, maximum_peak, calls;
static int active;
static const uint64_t owner = UINT64_C(0x2132435465768798);

__attribute__((noreturn)) void NS_ABORT(void) { _exit(74); }
void *NS_ALLOCATE(uint64_t size) {
    if (!active || size > UINT64_MAX - 7) NS_ABORT();
    size = (size + 7) & ~UINT64_C(7);
    if (!size || cursor > NS_LIMIT || size > NS_LIMIT - cursor) NS_ABORT();
    void *result = arena + cursor;
    cursor += size;
    if (cursor > peak) peak = cursor;
    if (peak > maximum_peak) maximum_peak = peak;
    return result;
}
static void fail(const char *message) { fprintf(stderr, "%s\n", message); exit(3); }
static void invoke(unsigned char *base, uint64_t count, uint64_t writable,
                   uint64_t operation, uint64_t length, uint64_t expected) {
    uint64_t generation = control.generation + 1;
    control = (struct Control){.code = NS_EXPORT, .owner = owner,
        .generation = generation - 1, .base = base, .count = count, .writable = writable};
    memset(arena, 0, sizeof(arena));
    cursor = peak = 32; active = 1;
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    unsigned char *mapping = mmap(NULL, page * 3, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) exit(2);
    uint64_t *words = (uint64_t *)(mapping + 2 * page - 16);
    words[0] = operation; words[1] = length;
    if (mprotect(mapping + page, page, PROT_READ)) exit(2);
    uint64_t result = NS_EXPORT(&control, owner, generation, words, 2);
    active = 0;
    if (result != expected || control.code || control.owner || control.active || control.base
        || control.count || control.writable || control.issued || control.generation != generation
        || peak > NS_LIMIT || words[0] != operation || words[1] != length) {
        fprintf(stderr, "namespace invocation=%llu operation=%llu fact=%llu result=%llu expected=%llu generation=%llu/%llu peak=%llu bound=%llu\n",
            (unsigned long long)calls, (unsigned long long)operation, (unsigned long long)length,
            (unsigned long long)result, (unsigned long long)expected,
            (unsigned long long)control.generation, (unsigned long long)generation,
            (unsigned long long)peak, (unsigned long long)NS_LIMIT);
        fail("namespace byte result/control/frame mismatch");
    }
    if (mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) exit(2);
    memset(mapping + page, 0, page);
    memset(arena, 0, sizeof(arena));
    const volatile unsigned char *readback = arena;
    for (size_t i = 0; i < sizeof(arena); ++i) if (readback[i]) fail("namespace arena not erased");
    readback = mapping + page;
    for (size_t i = 0; i < page; ++i) if (readback[i]) fail("namespace frame backing not erased");
    if (munmap(mapping, page * 3)) exit(2);
    ++calls;
}

#endif
