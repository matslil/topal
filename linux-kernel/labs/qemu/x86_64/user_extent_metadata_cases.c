/* Ordinary named-file, mode and namespace requests.
 * TOPAL-SYSTEMS-USER-EXTENT-001; INODE-MODE-STATE-001;
 * NAMESPACE-TRANSACTION-001. No guest-private state is supplied by this program.
 */
typedef unsigned long word;
extern long topal_test_syscall4(word,word,word,word,word);
static const unsigned char payload[4] = {0x31,0x62,0x93,0xc4};
static int call(word n,word a,word b,word c,word d,long expected) {
    return topal_test_syscall4(n,a,b,c,d)!=expected;
}
static int record(word fd,word out,word identity,word mode,word links,word size) {
    if (call(5,fd,out,0,0,0)) return 0;
    volatile word *value=(volatile word *)out;
    return value[0]==2 && value[1]==identity && value[2]==links &&
           value[3]==mode && value[6]==size && value[7]==4096;
}

long topal_user_extent_metadata_cases(void) {
    const word out=0x638fc0;
    const word a=(word)"/tmp/extent-a",b=(word)"/tmp/extent-b",c=(word)"/tmp/extent-c";
    if (topal_test_syscall4(12,0,0,0,0)!=0x619000) return 901;
    if (topal_test_syscall4(12,0x63a000,0,0,0)!=0x63a000) return 902;
    if (call(90,0x63a000,0600,0,0,-14)) return 903;
    long fd=topal_test_syscall4(257,(word)-100,a,0x42,0600);
    if (fd<3 || fd>=16) return 904;
    if (call(5,fd,out,0,0,0)) return 905;
    word identity=((volatile word *)out)[1];
    if (!identity || !record(fd,out,identity,0100600,1,0)) return 906;
    if (call(18,fd,(word)payload,4,0,4)) return 907;
    if (call(8,fd,0,1,0,0)) return 908;
    if (!record(fd,out,identity,0100600,1,4)) return 909;
    if (call(77,fd,2,0,0,0)) return 933;
    if (!record(fd,out,identity,0100600,1,2)) return 934;
    if (call(91,16,0640,0,0,-9)) return 910;
    if (call(91,fd,0640,0,0,0)) return 911;
    if (!record(fd,out,identity,0100640,1,2)) return 912;
    if (call(86,a,b,0,0,0)) return 913;
    if (!record(fd,out,identity,0100640,2,2)) return 914;
    if (call(82,b,c,0,0,0)) return 915;
    if (call(87,a,0,0,0,0)) return 916;
    if (!record(fd,out,identity,0100640,1,2)) return 917;
    long second=topal_test_syscall4(257,(word)-100,c,0x202,0);
    if (second<3 || second>=16 || second==fd) return 918;
    if (!record(fd,out,identity,0100640,1,0)) return 919;
    if (call(18,second,(word)payload,4,0,4)) return 920;
    if (call(8,second,0,1,0,0)) return 921;
    if (!record(fd,out,identity,0100640,1,4)) return 922;
    if (call(91,second,02755,0,0,0)) return 923;
    if (call(86,c,c,0,0,-17)) return 924;
    if (call(82,c,0x63a000,0,0,-14)) return 925;
    if (call(87,c,0,0,0,0)) return 926;
    if (!record(fd,out,identity,0102755,0,4)) return 927;
    if (call(3,second,0,0,0,0)) return 928;
    if (!record(fd,out,identity,0102755,0,4)) return 929;
    if (call(3,fd,0,0,0,0)) return 930;
    if (call(87,c,0,0,0,-2)) return 931;
    if (topal_test_syscall4(12,0x619000,0,0,0)!=0x619000) return 932;
    return 0;
}
