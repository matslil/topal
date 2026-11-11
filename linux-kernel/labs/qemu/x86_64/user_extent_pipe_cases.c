/* Ordinary process-owned FIFO requests: TOPAL-SYSTEMS-USER-EXTENT-001,
 * TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-{001,TRANSFER-001}. No signal delivery.
 */
typedef unsigned long word;
extern long topal_test_syscall3(word, word, word, word);
static const unsigned char payload[] = "owned-pipe-data!";

static int call(word n, word a, word b, word c, long expected) {
    return topal_test_syscall3(n,a,b,c) != expected;
}
static int equal(word address, const unsigned char *bytes, unsigned size) {
    for (unsigned i=0; i<size; ++i)
        if (((volatile unsigned char *)address)[i] != bytes[i]) return 0;
    return 1;
}

long topal_user_extent_pipe_cases(void) {
    if (topal_test_syscall3(12,0,0,0) != 0x619000) return 601;
    if (topal_test_syscall3(12,0x63a000,0,0) != 0x63a000) return 602;
    unsigned char saved[32];
    for (unsigned i=0; i<32; ++i) saved[i]=((volatile unsigned char *)0x5ffff0)[i];
    const word bad_pair[] = {0x63a000,~(word)0,0x400000,0x800000,0x639ffc};
    for (unsigned i=0; i<4; ++i) ((volatile unsigned char *)0x639ffc)[i]=0xda;
    for (unsigned i=0; i<5; ++i) {
        if (call(293,bad_pair[i],0x800,0,-14)) return 603;
        if (*(volatile unsigned *)0x639ffc != 0xdadadada) return 604;
    }
    if (call(293,0x638ffc,0x800,0,0)) return 605;
    unsigned reader=((volatile unsigned *)0x638ffc)[0];
    unsigned writer=((volatile unsigned *)0x638ffc)[1];
    if (reader<3 || reader>=16 || writer<3 || writer>=16 || reader==writer) return 606;
    if (call(16,reader,0x541b,0x5ffffe,0)) return 607;
    if (*(volatile unsigned *)0x5ffffe != 0) return 608;
    const word bad_input[] = {0x63a000,~(word)0,0x800000,0x639ff8};
    for (unsigned i=0; i<4; ++i)
        if (call(1,writer,bad_input[i],16,-14)) return 609;
    if (call(16,reader,0x541b,0x5ffffe,0)) return 610;
    if (*(volatile unsigned *)0x5ffffe != 0) return 611;
    /* Read-only application data is a valid write input. */
    if (call(1,writer,(word)payload,16,16)) return 612;
    const word bad_output[] = {0x63a000,~(word)0,0x400000,0x800000,0x639ffe};
    for (unsigned i=0; i<5; ++i)
        if (call(16,reader,0x541b,bad_output[i],-14)) return 613;
    if (call(16,reader,0x541b,0x5ffffe,0)) return 614;
    if (*(volatile unsigned *)0x5ffffe != 16) return 615;
    for (unsigned i=0; i<5; ++i) {
        word target=i==4 ? 0x639ff8 : bad_output[i];
        if (call(0,reader,target,16,-14)) return 616;
    }
    if (call(16,reader,0x541b,0x5ffffe,0)) return 617;
    if (*(volatile unsigned *)0x5ffffe != 16) return 618;
    if (call(0,reader,0x638ff8,16,16)) return 619;
    if (!equal(0x638ff8,payload,16)) return 620;
    /* Empty nonblocking input does not touch an unused unmapped destination. */
    if (call(0,reader,0x63a000,16,-11)) return 621;
    for (unsigned i=0; i<8; ++i) ((volatile unsigned char *)0x5ffffc)[i]=payload[i];
    if (call(1,writer,0x5ffffc,8,8)) return 622;
    /* Only the eight available bytes need mappings, despite a larger request. */
    if (call(0,reader,0x639ff8,16,8)) return 623;
    if (!equal(0x639ff8,payload,8)) return 624;
    if (call(16,reader,0x541b,0x5ffffe,0)) return 625;
    if (*(volatile unsigned *)0x5ffffe != 0) return 626;
    if (call(3,writer,0,0,0)) return 627;
    if (call(0,reader,0x63a000,16,0)) return 628;
    if (call(3,reader,0,0,0)) return 629;
    for (unsigned i=0; i<32; ++i) ((volatile unsigned char *)0x5ffff0)[i]=saved[i];
    if (topal_test_syscall3(12,0x619000,0,0) != 0x619000) return 630;
    return 0;
}
