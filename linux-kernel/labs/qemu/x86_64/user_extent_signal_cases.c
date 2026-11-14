/* Source-issued signal registrations; no signal delivery is requested.
 * TOPAL-SYSTEMS-USER-EXTENT-001; STATIC-PROCESS-SIGNAL-{ACTION,MASK}-001.
 */
typedef unsigned long word;
extern long topal_test_syscall4(word, word, word, word, word);
static const word action[4] = {0x570000,~(word)0,0x570000,~(word)0};
static const word ignored[4] = {1,0x4000000,0x570000,~(word)0};
static const word all = ~(word)0;
static const word one = 1;

static void copy(word address, const unsigned char *bytes, unsigned size) {
    for (unsigned i = 0; i < size; ++i) ((volatile unsigned char *)address)[i] = bytes[i];
}
static int equal(word address, const unsigned char *bytes, unsigned size) {
    for (unsigned i = 0; i < size; ++i)
        if (((volatile unsigned char *)address)[i] != bytes[i]) return 0;
    return 1;
}
static int call(word n, word a, word b, word c, word d, long expected) {
    return topal_test_syscall4(n,a,b,c,d) != expected;
}

long topal_user_extent_signal_cases(void) {
    if (topal_test_syscall4(12,0,0,0,0) != 0x619000) return 501;
    if (topal_test_syscall4(12,0x63a000,0,0,0) != 0x63a000) return 502;
    unsigned char saved[48], original[32];
    volatile unsigned char *directory = (volatile unsigned char *)0x5ffff7;
    for (unsigned i = 0; i < sizeof(saved); ++i) saved[i] = directory[i];
    if (call(13,10,0,0x638ff8,8,0)) return 503;
    for (unsigned i = 0; i < 32; ++i) original[i] = ((volatile unsigned char *)0x638ff8)[i];
    if (call(13,10,~(word)0,~(word)0,0,-22)) return 504;
    const word bad_action[] = {0x63a000,~(word)0,0x800000,0x639fff};
    for (unsigned i = 0; i < 4; ++i)
        if (call(13,10,bad_action[i],0,8,-14)) return 505;
    if (call(13,0,~(word)0,~(word)0,8,-14)) return 506;
    if (call(13,0,0,~(word)0,8,-22)) return 507;
    if (call(13,9,(word)action,~(word)0,8,-22)) return 508;
    if (call(13,19,(word)action,~(word)0,8,-22)) return 509;
    const word bad_output[] = {0x63a000,~(word)0,0x400000,0x800000,0x639fff};
    *(volatile unsigned char *)0x639fff = 0xda;
    for (unsigned i = 0; i < 5; ++i) {
        if (call(13,10,0,bad_output[i],8,-14)) return 510;
        if (*(volatile unsigned char *)0x639fff != 0xda) return 511;
    }
    if (call(13,10,(word)action,0,8,0)) return 512;
    if (call(13,10,0,0x5ffff8,8,0)) return 513;
    const word normalized[4] = {0x570000,0xdc000807,0x570000,~(word)0x40100};
    if (!equal(0x5ffff8,(const unsigned char *)normalized,32)) return 514;
    /* A valid update survives failure to copy its old value to read-only memory. */
    if (call(13,10,(word)ignored,0x400000,8,-14)) return 515;
    if (call(13,10,0,0x639fe0,8,0)) return 516;
    const word normalized_ignore[4] = {1,0x4000000,0x570000,~(word)0x40100};
    if (!equal(0x639fe0,(const unsigned char *)normalized_ignore,32)) return 517;
    const word zero[4] = {0,0,0,0};
    copy(0x638ff8,(const unsigned char *)zero,32);
    if (call(13,10,0x638ff8,0x638ff8,8,0)) return 518;
    if (!equal(0x638ff8,(const unsigned char *)normalized_ignore,32)) return 519;
    if (call(13,10,0,0x639fe0,8,0)) return 520;
    if (!equal(0x639fe0,(const unsigned char *)zero,32)) return 521;
    copy(0x637ff8,original,32);
    if (call(13,10,0x637ff8,0,8,0)) return 522;
    if (call(13,10,0,0x638ff8,8,0)) return 523;
    if (!equal(0x638ff8,original,32)) return 524;

    if (call(14,99,0,0x638ff8,8,0)) return 525;
    unsigned char blocked[8];
    for (unsigned i = 0; i < 8; ++i) blocked[i] = ((volatile unsigned char *)0x638ff8)[i];
    if (call(14,99,~(word)0,~(word)0,0,-22)) return 526;
    const word bad_mask[] = {0x63a000,~(word)0,0x800000,0x639ffc};
    for (unsigned i = 0; i < 4; ++i)
        if (call(14,2,bad_mask[i],0,8,-14)) return 527;
    if (call(14,99,~(word)0,~(word)0,8,-14)) return 528;
    if (call(14,99,(word)&all,~(word)0,8,-22)) return 529;
    *(volatile unsigned char *)0x639fff = 0xda;
    for (unsigned i = 0; i < 5; ++i) {
        word pointer = i == 4 ? 0x639ffc : bad_output[i];
        if (call(14,99,0,pointer,8,-14)) return 530;
        if (*(volatile unsigned char *)0x639fff != 0xda) return 531;
    }
    if (call(14,2,(word)&all,0,8,0)) return 532;
    if (call(14,99,0,0x5ffffc,8,0)) return 533;
    const word allowed = ~(word)0x40100;
    if (!equal(0x5ffffc,(const unsigned char *)&allowed,8)) return 534;
    if (call(14,2,(word)&one,0x400000,8,-14)) return 535;
    if (call(14,99,0,0x639ff8,8,0)) return 536;
    if (!equal(0x639ff8,(const unsigned char *)&one,8)) return 537;
    copy(0x638ffc,(const unsigned char *)&one,8);
    if (call(14,1,0x638ffc,0x638ffc,8,0)) return 538;
    if (!equal(0x638ffc,(const unsigned char *)&one,8)) return 539;
    if (call(14,0,(word)&one,0,8,0)) return 540;
    if (call(14,1,(word)&one,0,8,0)) return 541;
    if (call(14,99,0,0x639ff8,8,0)) return 542;
    if (!equal(0x639ff8,(const unsigned char *)zero,8)) return 543;
    copy(0x637ffc,blocked,8);
    if (call(14,2,0x637ffc,0,8,0)) return 544;
    if (call(14,99,0,0x638ff8,8,0)) return 545;
    if (!equal(0x638ff8,blocked,8)) return 546;
    for (unsigned i = 0; i < sizeof(saved); ++i) directory[i] = saved[i];
    if (topal_test_syscall4(12,0x619000,0,0,0) != 0x619000) return 547;
    return 0;
}
