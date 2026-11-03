/* Independent same-signature source instances and cross-code refusal oracle. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>

struct Control;
typedef uint64_t (*entry)(struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
struct Control {
    entry code;
    uint64_t owner, generation, active;
    unsigned char *base;
    uint64_t count, writable, issued;
};
_Static_assert(sizeof(struct Control) == 64, "qualified control extent");
extern uint64_t topal_kernel_region_frame_left_owner_apply_v1(
    struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
extern uint64_t topal_kernel_region_frame_right_owner_apply_v1(
    struct Control *, uint64_t, uint64_t, const uint64_t *, uint64_t);
extern int left_main(void), right_main(void);
extern int left_arena_zero(void), right_arena_zero(void);
extern uint64_t left_cursor_value(void), right_cursor_value(void);

static struct Control *control;
static struct Control before;
static unsigned char *owner_bytes;
static int cross_probe;

/* Called in the child by either instance's abort hook, before real _exit. */
void region_instance_probe_exit(void) {
    if (!cross_probe) return;
    if (memcmp(control, &before, sizeof(before)) || left_cursor_value() != 32
        || right_cursor_value() != 32 || !left_arena_zero() || !right_arena_zero()) _exit(91);
    for (size_t i = 0; i < 32; ++i) if (owner_bytes[i] != 0xa5) _exit(92);
}

static int cross_calls(void) {
    unsigned char *shared = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                                 MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    if (shared == MAP_FAILED) return 1;
    control = (struct Control *)shared;
    owner_bytes = shared + 128;
    static const uint64_t frame[3] = {UINT64_MAX, UINT64_C(1) << 63, 1};
    entry entries[2] = {topal_kernel_region_frame_left_owner_apply_v1,
                        topal_kernel_region_frame_right_owner_apply_v1};
    for (unsigned i = 0; i < 2; ++i) {
        memset(owner_bytes, 0xa5, 32);
        *control = (struct Control){.code = entries[i], .owner = 1234,
            .base = owner_bytes + 8, .count = 16, .writable = 1};
        before = *control;
        cross_probe = 1;
        pid_t child = fork();
        if (child < 0) return 2;
        if (!child) {
            entries[1 - i](control, 1234, 1, frame, 3);
            _exit(90);
        }
        int status = 0;
        if (waitpid(child, &status, 0) != child || !WIFEXITED(status)
            || WEXITSTATUS(status) != 74 || memcmp(control, &before, sizeof(before))) return 3;
        for (size_t j = 0; j < 32; ++j) if (owner_bytes[j] != 0xa5) return 4;
    }
    cross_probe = 0;
    return munmap(shared, 4096) ? 5 : 0;
}

int main(void) {
    int result = left_main();
    if (!result) result = right_main();
    if (!result) result = cross_calls();
    if (result) { fprintf(stderr, "instance client failed: %d\n", result); return result; }
    puts("INSTANCE_NATIVE_PASS contexts=2 calls=96 refusals=32 cross=2");
    return 0;
}
