/* Ordinary serial descriptor requests; TOPAL-SYSTEMS-USER-EXTENT-001 and
 * TOPAL-SYSTEMS-STATIC-PROCESS-FD-TABLE-001. */
typedef unsigned long word;
extern long topal_test_syscall3(word,word,word,word);
static const unsigned char ready_a[] = "TOPAL_SOURCE_SERIAL_A\n";
static const unsigned char ready_b[] = "TOPAL_SOURCE_SERIAL_B\n";
static const unsigned char expected_a[] = "abcdefg";
static const unsigned char expected_b[] = "12345678";

static int receive(word pointer,word count) {
    word done=0;
    while (done<count) {
        long result=topal_test_syscall3(0,0,pointer+done,count-done);
        if (result<=0 || (word)result>count-done) return 1;
        done+=(word)result;
    }
    return 0;
}
static int equal(word pointer,const unsigned char *bytes,unsigned count) {
    for (unsigned i=0;i<count;++i)
        if (((volatile unsigned char *)pointer)[i]!=bytes[i]) return 0;
    return 1;
}

long topal_user_extent_serial_cases(void) {
    if (topal_test_syscall3(12,0,0,0)!=0x619000) return 801;
    if (topal_test_syscall3(12,0x63a000,0,0)!=0x63a000) return 802;
    unsigned char saved[32];
    for (unsigned i=0;i<32;++i) saved[i]=((volatile unsigned char *)0x5ffff0)[i];
    if (topal_test_syscall3(1,1,(word)ready_a,sizeof(ready_a)-1)!=(long)sizeof(ready_a)-1) return 803;
    if (topal_test_syscall3(0,0,0x639fff,1)!=1 || *(volatile unsigned char *)0x639fff!='U') return 804;
    const word bad[] = {0x63a000,~(word)0,0x400000,0x800000,0x639ffc};
    for (unsigned i=0;i<5;++i)
        if (topal_test_syscall3(0,0,bad[i],8)!=-14) return 805;
    if (topal_test_syscall3(0,1,~(word)0,8)!=-9) return 806;
    if (topal_test_syscall3(0,99,~(word)0,8)!=-9) return 807;
    if (topal_test_syscall3(0,0,0x63a000,0)!=0) return 808;
    if (receive(0x5ffffc,7) || !equal(0x5ffffc,expected_a,7)) return 809;
    for (unsigned i=0;i<256;++i) ((volatile unsigned char *)0x639f00)[i]=0xda;
    *(volatile unsigned char *)0x639eff=0xc5;
    if (topal_test_syscall3(1,1,(word)ready_b,sizeof(ready_b)-1)!=(long)sizeof(ready_b)-1) return 810;
    /* The full numeric request is larger than its admitted 256-byte quantum. */
    long first=topal_test_syscall3(0,0,0x639f00,512);
    if (first<=0 || first>8 || receive(0x639f00+(word)first,8-(word)first)) return 811;
    if (!equal(0x639f00,expected_b,8) || *(volatile unsigned char *)0x639eff!=0xc5) return 812;
    for (unsigned i=8;i<256;++i)
        if (((volatile unsigned char *)0x639f00)[i]!=0xda) return 813;
    for (unsigned i=0;i<32;++i) ((volatile unsigned char *)0x5ffff0)[i]=saved[i];
    if (topal_test_syscall3(12,0x619000,0,0)!=0x619000) return 814;
    return 0;
}
