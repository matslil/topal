/* Ordinary owned file/directory requests: TOPAL-SYSTEMS-USER-EXTENT-001,
 * VOLATILE-FILE-TRANSFER-001 and POSITIONAL-FILE-TRANSFER-001. */
typedef unsigned long word;
extern long topal_test_syscall4(word, word, word, word, word);
static const unsigned char initial[16] = {
    0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1a,0x1b,0x1c,0x1d,0x1e,0x1f
};
static const unsigned char changed[8] = {0xa0,0xa1,0xa2,0xa3,0xa4,0xa5,0xa6,0xa7};
static int call(word n,word a,word b,word c,word d,long expected) {
    return topal_test_syscall4(n,a,b,c,d)!=expected;
}
static int equal(word pointer,const unsigned char *bytes,unsigned count) {
    for (unsigned i=0;i<count;++i)
        if (((volatile unsigned char *)pointer)[i]!=bytes[i]) return 0;
    return 1;
}

long topal_user_extent_storage_cases(void) {
    if (topal_test_syscall4(12,0,0,0,0)!=0x619000) return 701;
    if (topal_test_syscall4(12,0x63a000,0,0,0)!=0x63a000) return 702;
    unsigned char saved[32];
    for (unsigned i=0;i<32;++i) saved[i]=((volatile unsigned char *)0x5ffff0)[i];
    long fd=topal_test_syscall4(257,(word)-100,(word)"/tmp",0x410002,0600);
    if (fd<3 || fd>=16) return 703;
    const word bad_input[] = {0x63a000,~(word)0,0x800000,0x639ff8};
    for (unsigned i=0;i<4;++i)
        if (call(1,fd,bad_input[i],16,0,-14)) return 704;
    if (call(1,fd,(word)initial,16,0,16)) return 705;
    if (call(8,fd,0,0,0,0)) return 706;
    const word bad_output[] = {0x63a000,~(word)0,0x400000,0x800000,0x639ff8};
    for (unsigned i=0;i<5;++i)
        if (call(0,fd,bad_output[i],16,0,-14)) return 707;
    if (call(0,fd,0x5ffff8,16,0,16)) return 708;
    if (!equal(0x5ffff8,initial,16)) return 709;
    if (call(8,fd,0,1,0,16)) return 710;
    for (unsigned i=0;i<5;++i)
        if (call(17,fd,bad_output[i],16,0,-14)) return 711;
    if (call(17,fd,0x638ff8,16,0,16)) return 712;
    if (!equal(0x638ff8,initial,16)) return 713;
    if (call(8,fd,0,1,0,16)) return 714;
    for (unsigned i=0;i<4;++i) {
        word pointer=i==3 ? 0x639ffc : bad_input[i];
        if (call(18,fd,pointer,8,8,-14)) return 715;
    }
    for (unsigned i=0;i<8;++i) ((volatile unsigned char *)0x5ffffc)[i]=changed[i];
    if (call(18,fd,0x5ffffc,8,8,8)) return 716;
    if (call(8,fd,0,1,0,16)) return 717;
    if (call(17,fd,0x638ff8,16,0,16)) return 718;
    if (!equal(0x638ff8,initial,8) || !equal(0x639000,changed,8)) return 719;
    if (call(17,fd,0x63a000,16,16,0)) return 720;
    if (call(0,fd,0x63a000,16,0,0)) return 721;
    if (call(17,fd,~(word)0,1,~(word)0,-22)) return 722;
    if (call(3,fd,0,0,0,0)) return 723;

    fd=topal_test_syscall4(257,(word)-100,(word)"/bin/busybox",0,0);
    if (fd<3 || fd>=16) return 724;
    for (unsigned i=0;i<5;++i)
        if (call(17,fd,bad_output[i],16,0,-14)) return 725;
    if (call(17,fd,0x638ff8,16,0,16)) return 726;
    const unsigned char elf[7] = {0x7f,'E','L','F',2,1,1};
    if (!equal(0x638ff8,elf,7)) return 727;
    if (call(8,fd,0,1,0,0)) return 728;
    if (call(3,fd,0,0,0,0)) return 729;

    fd=topal_test_syscall4(257,(word)-100,(word)"/tmp",0x10000,0);
    if (fd<3 || fd>=16) return 730;
    for (unsigned i=0;i<5;++i) {
        word pointer=i==4 ? 0x639ff0 : bad_output[i];
        if (call(217,fd,pointer,48,0,-14)) return 731;
    }
    if (call(217,fd,0x638ff8,48,0,48)) return 732;
    volatile unsigned char *records=(volatile unsigned char *)0x638ff8;
    if (records[16]!=24 || records[17]!=0 || records[18]!=4 || records[19]!='.' || records[20]!=0) return 733;
    if (records[40]!=24 || records[41]!=0 || records[42]!=4 || records[43]!='.' || records[44]!='.' || records[45]!=0) return 734;
    if (call(3,fd,0,0,0,0)) return 735;
    for (unsigned i=0;i<32;++i) ((volatile unsigned char *)0x5ffff0)[i]=saved[i];
    if (topal_test_syscall4(12,0x619000,0,0,0)!=0x619000) return 736;
    return 0;
}
