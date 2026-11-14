/* Ordinary poll/read requests; TOPAL-SYSTEMS-USER-EXTENT-001.
 * Uses only this process's image/heap and its live read-only descriptor.
 */
typedef unsigned long word;
extern long topal_test_syscall4(word,word,word,word,word);
static const word record = 0xffff000100000003;

static int poll_at(word pointer,long expected) {
    return topal_test_syscall4(7,pointer,1,~(word)0,0) != expected;
}

long topal_user_extent_poll_cases(word n,word a,word b,word c,word d) {
    if (n!=0 || a!=3 || c!=1) return -601;
    if (topal_test_syscall4(12,0,0,0,0)!=0x619000) return -602;
    if (topal_test_syscall4(12,0x63a000,0,0,0)!=0x63a000) return -603;
    unsigned char saved[10];
    volatile unsigned char *directory=(volatile unsigned char *)0x5ffffb;
    for (unsigned i=0;i<10;++i) saved[i]=directory[i];
    volatile unsigned char *edge=(volatile unsigned char *)0x639ff9;
    for (unsigned i=0;i<7;++i) edge[i]=0xda;
    const word bad[]={0x63a000,~(word)0,0x800000,0x639ff9};
    for (unsigned i=0;i<4;++i) {
        if (poll_at(bad[i],-14)) return -604;
        for (unsigned j=0;j<7;++j) if (edge[j]!=0xda) return -605;
    }
    /* Readable record, but its output field is in read-only code storage. */
    if (poll_at((word)&record,-14) || record!=0xffff000100000003) return -606;
    const word addresses[]={0x5ffffc,0x638ffc,0x639ff8,0x638ff9};
    word address=addresses[b&3];
    volatile unsigned char *out=(volatile unsigned char *)address;
    for (unsigned i=0;i<8;++i) out[i]=((const unsigned char *)&record)[i];
    if (poll_at(address,1)) return -607;
    if (out[6]!=1 || out[7]!=0) return -608;
    /* Refusal and a repeated poll must preserve pending readiness/position. */
    if (poll_at((word)&record,-14)) return -609;
    out[6]=0xa5; out[7]=0x5a;
    if (poll_at(address,1)) return -610;
    if (out[6]!=1 || out[7]!=0) return -611;
    const word bad_read[]={0x63a000,~(word)0,0x400000,0x800000};
    for (unsigned i=0;i<4;++i)
        if (topal_test_syscall4(0,3,bad_read[i],1,d)!=-14) return -612;
    *(volatile unsigned char *)0x639fff=0xa5;
    if (topal_test_syscall4(0,3,0x639fff,1,d)!=1) return -613;
    ((volatile unsigned char *)b)[0]=*(volatile unsigned char *)0x639fff;
    for (unsigned i=0;i<10;++i) directory[i]=saved[i];
    if (topal_test_syscall4(12,0x619000,0,0,0)!=0x619000) return -614;
    return 1;
}
