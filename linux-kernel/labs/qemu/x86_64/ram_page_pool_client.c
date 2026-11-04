/* Independent source-component oracle. This is not a live kernel caller. */
#define main unused_weighted_frame_main
#include "region_frame_client.c"
#undef main

volatile int ram_debug_sample;
volatile uint64_t ram_expected_result;
volatile uint64_t ram_debug_samples;
unsigned char ram_expected_map[8];
static struct Control ram_control;
static unsigned ram_calls;
static uint64_t ram_maximum_peak;

#ifndef RAM_REVERSE_ORDER
#define RAM_REVERSE_ORDER 0
#endif

static int invoke(unsigned mask, uint64_t operation, uint64_t slot,
                  uint64_t extent, int sample) {
    _Alignas(8) unsigned char bytes[32];
    memset(bytes, 0xa5, sizeof(bytes));
    for (unsigned i = 0; i < 8; ++i)
        bytes[8 + i] = ram_expected_map[i] = (unsigned char)((mask >> i) & 1);
    ram_expected_result = 0;
    if (extent == 8 && operation == 0 && slot == 0) {
        for (unsigned ordinal = 0; ordinal < 8; ++ordinal) {
            unsigned i = RAM_REVERSE_ORDER ? 7 - ordinal : ordinal;
            if (!ram_expected_map[i]) {
                ram_expected_map[i] = 1;
                ram_expected_result = 1;
                break;
            }
        }
    } else if (extent == 8 && operation == 1 && slot < 8 && ram_expected_map[slot]) {
        ram_expected_map[slot] = 0;
        ram_expected_result = 1;
    }
    prepare(&ram_control, bytes);
    ram_control.count = extent;
    uint64_t generation = ram_control.generation + 1;
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    unsigned char *mapping = mmap(NULL, 3 * page, PROT_NONE,
                                  MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) return 20;
    uint64_t *facts = (uint64_t *)(mapping + 2 * page - 16);
    facts[0] = operation;
    facts[1] = slot;
    if (mprotect(mapping + page, page, PROT_READ)) return 21;
    ram_debug_sample = sample;
    if (sample) ++ram_debug_samples;
    uint64_t result = topal_kernel_region_frame_apply_v1(&ram_control, owner,
                                                       generation, facts, 2);
    ram_debug_sample = 0;
    if (facts[0] != operation || facts[1] != slot) return 22;
    if (mprotect(mapping + page, page, PROT_READ | PROT_WRITE)) return 23;
    memset(mapping + page, 0, page);
    const volatile unsigned char *cleared = mapping + page;
    for (size_t i = 0; i < page; ++i) if (cleared[i]) return 24;
    if (munmap(mapping, 3 * page)) return 25;
    if (peak > ram_maximum_peak) ram_maximum_peak = peak;
    if (result != ram_expected_result || finish(&ram_control, generation)) return 26;
    for (unsigned i = 0; i < sizeof(bytes); ++i) {
        unsigned char expected = i >= 8 && i < 16 ? ram_expected_map[i - 8] : 0xa5;
        if (bytes[i] != expected) {
            fprintf(stderr, "page map mismatch: mask=%u op=%llu slot=%llu byte=%u\n",
                    mask, (unsigned long long)operation, (unsigned long long)slot, i);
            return 27;
        }
    }
    ++ram_calls;
    return 0;
}

int main(void) {
    for (unsigned mask = 0; mask < 256; ++mask) {
        int sample = mask == 0 || mask == 255 || mask == 0x55 || mask == 0xaa;
        int result = invoke(mask, 0, 0, 8, sample);
        if (result) return result;
        for (unsigned slot = 0; slot < 8; ++slot) {
            result = invoke(mask, 1, slot, 8, sample);
            if (result) return result;
        }
    }
    const uint64_t invalid[][2] = {
        {2, 0}, {UINT64_C(1) << 32, 0}, {UINT64_C(1) << 63, 0}, {UINT64_MAX, 0},
        {(UINT64_C(1) << 32) | 1, 0}, {(UINT64_C(1) << 63) | 1, 0},
        {0, 1}, {0, 8}, {0, UINT64_C(1) << 32}, {0, UINT64_MAX},
        {1, 8}, {1, UINT64_C(1) << 32}, {1, UINT64_MAX},
        {1, (UINT64_C(1) << 32) | 2}, {1, (UINT64_C(1) << 63) | 2},
    };
    for (unsigned i = 0; i < sizeof(invalid) / sizeof(invalid[0]); ++i) {
        int result = invoke(0x55, invalid[i][0], invalid[i][1], 8, 1);
        if (result) return result;
    }
    const uint64_t extents[] = {1, 7, 9, 16};
    for (unsigned i = 0; i < sizeof(extents) / sizeof(extents[0]); ++i) {
        int result = invoke(0, 0, 0, extents[i], 1);
        if (result) return result;
    }
    int result = refusals();
    if (result) return result;
    printf("RAM_PAGE_POOL_NATIVE_PASS calls=%u samples=%llu refusals=16 peak=%llu\n",
           ram_calls, (unsigned long long)ram_debug_samples, (unsigned long long)ram_maximum_peak);
    return 0;
}
