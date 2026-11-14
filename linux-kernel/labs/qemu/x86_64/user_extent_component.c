/* Independent guarded host client for TOPAL-SYSTEMS-USER-EXTENT-001.
 * The callbacks own real guarded host table storage, but do not establish a
 * kernel syscall lease, CR3 ownership or canonical publication authority. */
#define _GNU_SOURCE
#include <assert.h>
#include <inttypes.h>
#include <setjmp.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>

#define PAGE UINT64_C(4096)
#define USER_END (UINT64_C(1) << 47)
#define NX (UINT64_C(1) << 63)
#define MASK UINT64_C(0x000ffffffffff000)
#define MAX_TABLES 16
#define MAX_BACKING 600

extern uint64_t topal_x86_owned_user_extent_v1(uint64_t, uint64_t, uint64_t);
struct table { uint64_t physical, level, *words; void *allocation; bool pinned; };
struct backing { uint64_t virtual, physical; bool writable, live, pinned; };
static struct table tables[MAX_TABLES];
static struct backing backing[MAX_BACKING];
static unsigned nt, nb;
static uint64_t epoch, current_epoch, root, reads, begins, ends, calls, permission_cases;
static bool active, mode_supported, context_valid, change_epoch_at_backing;
static jmp_buf recovery;

_Noreturn void topal_x86_user_extent_fatal_v1(void) { longjmp(recovery, 1); }

static struct table *table_at(uint64_t physical) {
    for (unsigned i = 0; i < nt; ++i) if (tables[i].physical == physical) return &tables[i];
    return NULL;
}
static struct table *new_table(uint64_t level) {
    assert(nt < MAX_TABLES);
    struct table *t = &tables[nt];
    t->physical = UINT64_C(0x100000) + nt++ * PAGE;
    t->level = level; t->pinned = true;
    t->allocation = mmap(NULL, 3 * PAGE, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    assert(t->allocation != MAP_FAILED);
    t->words = (uint64_t *)((unsigned char *)t->allocation + PAGE);
    assert(mprotect(t->words, PAGE, PROT_READ | PROT_WRITE) == 0);
    memset(t->words, 0, PAGE);
    return t;
}
static void reset(void) {
    for (unsigned i = 0; i < nt; ++i) assert(munmap(tables[i].allocation, 3 * PAGE) == 0);
    memset(tables, 0, sizeof(tables)); memset(backing, 0, sizeof(backing)); nt = nb = 0;
    active = false; mode_supported = context_valid = true; change_epoch_at_backing = false;
    root = new_table(4)->physical;
}
static void map_page(uint64_t va, uint64_t pa) {
    assert(va % PAGE == 0 && pa % PAGE == 0 && nb < MAX_BACKING);
    struct table *t = table_at(root);
    for (unsigned level = 4; level > 1; --level) {
        unsigned index = (va / PAGE >> (9 * (level - 1))) % 512;
        if (!t->words[index]) t->words[index] = new_table(level - 1)->physical | 7;
        t = table_at(t->words[index] & MASK); assert(t);
    }
    t->words[va / PAGE % 512] = pa | NX | 7;
    backing[nb++] = (struct backing){va, pa, true, true, true};
}
static uint64_t *path_word(uint64_t va, unsigned wanted) {
    struct table *t = table_at(root);
    for (unsigned level = 4; level; --level) {
        uint64_t *word = &t->words[(va / PAGE >> (9 * (level - 1))) % 512];
        if (level == wanted) return word;
        t = table_at(*word & MASK); assert(t);
    }
    assert(false); return NULL;
}
static void live(uint64_t identity, uint64_t actual_root) {
    if (!active || identity != current_epoch || actual_root != root) topal_x86_user_extent_fatal_v1();
}
uint64_t topal_x86_user_extent_begin_v1(void) {
    ++begins;
    if (!context_valid || active) topal_x86_user_extent_fatal_v1();
    if (!mode_supported) return 0;
    active = true; current_epoch = ++epoch; assert(current_epoch); return current_epoch;
}
uint64_t topal_x86_user_extent_root_v1(uint64_t identity) {
    live(identity, root); return root;
}
void *topal_x86_user_extent_table_v1(uint64_t identity, uint64_t actual_root,
                                    uint64_t physical, uint64_t level) {
    live(identity, actual_root);
    struct table *t = table_at(physical);
    if (!t || !t->pinned || t->level != level) return NULL;
    ++reads; return t->words;
}
uint64_t topal_x86_user_extent_backing_v1(uint64_t identity, uint64_t actual_root,
                                        uint64_t va, uint64_t pa, uint64_t write) {
    live(identity, actual_root);
    if (change_epoch_at_backing) ++current_epoch;
    for (unsigned i = 0; i < nb; ++i) if (backing[i].virtual == va) {
        struct backing *b = &backing[i];
        if (!b->live || !b->pinned || pa != b->physical || pa >= (UINT64_C(1) << 30) || table_at(pa)) return 2;
        return write && !b->writable ? 1 : 0;
    }
    return 2;
}
void topal_x86_user_extent_end_v1(uint64_t identity, uint64_t actual_root) {
    live(identity, actual_root); ++ends; active = false;
}
static uint64_t bytes_hash(void) {
    uint64_t result = UINT64_C(1469598103934665603);
    for (unsigned i = 0; i < nt; ++i) for (unsigned j = 0; j < 512; ++j)
        result = (result ^ tables[i].words[j]) * UINT64_C(1099511628211);
    for (unsigned i = 0; i < nb; ++i) {
        result = (result ^ backing[i].virtual ^ backing[i].physical) * UINT64_C(1099511628211);
        result ^= (uint64_t)backing[i].writable | (uint64_t)backing[i].live << 1 | (uint64_t)backing[i].pinned << 2;
    }
    return result;
}
static void check(uint64_t start, uint64_t length, uint64_t write, uint64_t expected) {
    uint64_t before = bytes_hash(), old_begins = begins, old_ends = ends;
    if (setjmp(recovery)) { fprintf(stderr, "unexpected invariant failure\n"); assert(false); }
    uint64_t result = topal_x86_owned_user_extent_v1(start, length, write); ++calls;
    assert(result == expected && !active && before == bytes_hash());
    if (expected == 1 || length == 0) assert(begins == old_begins && ends == old_ends);
    else if (mode_supported) assert(begins == old_begins + 1 && ends == old_ends + 1);
}
static void fatal(uint64_t start, uint64_t length, uint64_t write) {
    uint64_t before = bytes_hash();
    if (!setjmp(recovery)) {
        (void)topal_x86_owned_user_extent_v1(start, length, write); assert(false);
    }
    assert(before == bytes_hash()); ++calls;
    /* A host longjmp is only the client's observation of noreturn failure.
     * It does not claim guest fatal cleanup or successful lease retirement. */
    active = false;
}
int main(void) {
    reset(); map_page(0x700000, 0x20000000);
    uint64_t *path[4], original[4];
    for (unsigned i = 0; i < 4; ++i) { path[i] = path_word(0x700000, 4 - i); original[i] = *path[i]; }
    for (unsigned mask = 0; mask < 4096; ++mask) {
        unsigned flags[4], m = mask;
        for (unsigned i = 0; i < 4; ++i) { flags[i] = m % 8; m /= 8; *path[i] = (original[i] & ~UINT64_C(7)) | flags[i]; }
        for (unsigned write = 0; write < 2; ++write) {
            uint64_t expected = 0;
            for (unsigned i = 0; i < 4; ++i) {
                if (!(flags[i] & 1)) { expected = 2; break; }
                if (!(flags[i] & 4) || (write && !(flags[i] & 2))) { expected = 3; break; }
            }
            check(0x700000, PAGE, write, expected); ++permission_cases;
        }
    }
    for (unsigned i = 0; i < 4; ++i) *path[i] = original[i] | NX | 0x60;
    check(0x700000, PAGE, 1, 0);
    for (unsigned level = 1; level <= 4; ++level) {
        uint64_t *p = path_word(0x700000, level), old = *p;
        for (unsigned bit = 7; bit <= 62; ++bit) if (bit == 7 || bit == 9 || bit == 52 || bit == 59 || bit == 62) {
            *p = old | (UINT64_C(1) << bit); check(0x700000, PAGE, 0, 4);
        }
        *p = old;
    }
    for (unsigned power = 21; power <= 39; power += 9) {
        reset(); uint64_t at = UINT64_C(1) << power;
        map_page(at - PAGE, 0x20000000); map_page(at, 0x30000000);
        check(at - 17, 34, 1, 0);
    }
    reset(); for (unsigned i = 0; i < 400; ++i) map_page(0x5f7000 + i * PAGE, 0x20000000 + i * PAGE);
    check(0x5f7000, 400 * PAGE, 1, 0);
    *path_word(0x650000, 1) = 0; check(0x5f7000, 400 * PAGE, 1, 2);
    reset(); map_page(USER_END - PAGE, 0x20000000); check(USER_END - 1, 1, 0, 0);
    uint64_t bad[][2] = {{UINT64_MAX, 1}, {UINT64_MAX - 2, 4}, {USER_END, 1},
                        {USER_END - 1, 2}, {UINT64_C(0xffff800000000000), PAGE}, {1, UINT64_MAX}};
    for (unsigned i = 0; i < sizeof(bad)/sizeof(bad[0]); ++i) check(bad[i][0], bad[i][1], 0, 1);
    context_valid = false;
    uint64_t empty[] = {0, USER_END, UINT64_MAX, UINT64_C(0xffff800000000000)};
    uint64_t old_reads = reads;
    for (unsigned i = 0; i < 4; ++i) check(empty[i], 0, 0, 0);
    assert(reads == old_reads); fatal(USER_END - 1, 1, 0); context_valid = true;
    mode_supported = false; check(USER_END - 1, 1, 0, 4); mode_supported = true;
    for (unsigned fault = 0; fault < 9; ++fault) {
        reset(); map_page(0x700000, 0x20000000);
        if (fault == 0) tables[1].pinned = false;
        if (fault == 1) tables[1].level = 1;
        if (fault == 2) *path_word(0x700000, 4) = UINT64_C(0x40000000) | 7;
        if (fault == 3) backing[0].live = false;
        if (fault == 4) backing[0].pinned = false;
        if (fault == 5) backing[0].physical += PAGE;
        if (fault == 6) { backing[0].physical = root; *path_word(0x700000, 1) = root | NX | 7; }
        if (fault == 7) change_epoch_at_backing = true;
        if (fault == 8) root += 1;
        uint64_t prior = reads; fatal(0x700000, PAGE, 0);
        if (fault <= 2) assert(reads == prior + 1);
        if (fault == 8) assert(reads == prior);
    }
    reset(); map_page(0x700000, 0x20000000);
    backing[0].writable = false; check(0x700000, 1, 0, 0); check(0x700000, 1, 1, 3);
    fatal(0x700000, 1, 2);
    printf("{\"passed\":true,\"calls\":%" PRIu64 ",\"permission_cases\":%" PRIu64 ",\"table_observations\":%" PRIu64 "}\n", calls, permission_cases, reads);
    reset(); for (unsigned i = 0; i < nt; ++i) assert(munmap(tables[i].allocation, 3 * PAGE) == 0);
    return 0;
}
