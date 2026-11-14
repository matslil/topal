/* Ordinary pre-import missing-path lookup. This is a separate prefix test,
 * not a complete execution of the surrounding ABI witness.
 * TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001.
 */
#define topal_user_extent_command_cases topal_original_command_cases
#include "command-original.c"
#undef topal_user_extent_command_cases

const unsigned char topal_prelude_stat_path[9] = "/missing";

__attribute__((noinline)) long topal_prelude_stat_completed(long result) {
    volatile long captured = result;
    return captured;
}

long topal_user_extent_command_cases(word n, word a, word b, word count, word d) {
    long result = topal_original_command_cases(n,a,b,count,d);
    const unsigned char *text = (const unsigned char *)b;
    const unsigned char prefix[8] = {'e','c','h','o',' ','/','*',';'};
    if (result < 8) return result;
    for (unsigned i=0; i<sizeof(prefix); ++i) if (text[i]!=prefix[i]) return result;
    /* The missing lookup must not use its metadata destination. */
    long missing = topal_test_syscall4(262,(word)-100,(word)topal_prelude_stat_path,~(word)0,0);
    long captured = topal_prelude_stat_completed(missing);
    return captured == -2 ? result : -412;
}
