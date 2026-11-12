/* Ordinary stat queries: TOPAL-SYSTEMS-USER-EXTENT-001.
 * The kernel and application image are built from source before VM execution.
 */
typedef unsigned long word;
extern long topal_test_syscall4(word,word,word,word,word);
static const unsigned char payload[] = "stat";
static int call(word n,word a,word b,word c,word d,long result) {
    return topal_test_syscall4(n,a,b,c,d) != result;
}
static int record(word pointer,word device,word type,word links,word size) {
    const volatile word *value=(const volatile word *)pointer;
    return value[0]==device && (value[3]&0170000)==type && value[2]==links &&
           value[6]==size && value[7]==4096;
}

long topal_user_extent_stat_cases(void) {
    const word out=0x638fc0,bad=0x639fc0;
    if (topal_test_syscall4(12,0,0,0,0)!=0x619000) return 801;
    if (topal_test_syscall4(12,0x63a000,0,0,0)!=0x63a000) return 802;
    if (call(5,1,out,0,0,0) || !record(out,1,0020000,1,0)) return 803;
    if (call(5,0,out,0,0,0) || !record(out,1,0020000,1,0)) return 835;
    if (call(5,1,bad,0,0,-14)) return 804;
    if (call(5,1,0x400000,0,0,-14)) return 805;
    if (call(5,16,0x63a000,0,0,-9)) return 806;
    if (call(262,(word)-100,(word)"/bin/busybox",out,0,0) ||
        !record(out,1,0100000,1,2024544)) return 807;
    if (call(262,(word)-100,(word)"/missing",0x63a000,0,-2)) return 808;
    if (call(262,(word)-100,(word)"/tmp",bad,0,-14)) return 809;
    if (call(262,(word)-100,(word)"/",out,0,0) || !record(out,1,0040000,4,0)) return 810;
    if (call(262,(word)-100,(word)"/bin",out,0,0) || !record(out,1,0040000,2,0)) return 811;
    if (call(262,(word)-100,(word)"/tmp",out,0,0) || !record(out,2,0040000,2,0)) return 812;
    long fd=topal_test_syscall4(257,(word)-100,(word)"/bin/busybox",0,0);
    if (fd<3 || fd>=16) return 813;
    if (call(5,fd,out,0,0,0) || !record(out,1,0100000,1,2024544)) return 814;
    if (call(3,fd,0,0,0,0)) return 815;
    fd=topal_test_syscall4(257,(word)-100,(word)"/tmp",0x10000,0);
    if (fd<3 || fd>=16) return 836;
    if (call(5,fd,out,0,0,0) || !record(out,2,0040000,2,0)) return 837;
    if (call(3,fd,0,0,0,0)) return 838;
    if (call(293,0x638ff8,0x800,0,0,0)) return 816;
    unsigned reader=((volatile unsigned *)0x638ff8)[0];
    unsigned writer=((volatile unsigned *)0x638ff8)[1];
    if (reader<3 || reader>=16 || writer<3 || writer>=16 || reader==writer) return 817;
    if (call(5,reader,out,0,0,0) || !record(out,2,0010000,1,0)) return 818;
    if (call(5,writer,out,0,0,0) || !record(out,2,0010000,1,0)) return 819;
    if (call(3,writer,0,0,0,0) || call(3,reader,0,0,0,0)) return 820;
    fd=topal_test_syscall4(257,(word)-100,(word)"/tmp",0x410002,0600);
    if (fd<3 || fd>=16) return 821;
    if (call(5,fd,bad,0,0,-14)) return 822;
    if (call(5,fd,out,0,0,0) || !record(out,2,0100000,0,0)) return 823;
    if (call(262,fd,0,out,0x1000,0) || !record(out,2,0100000,0,0)) return 824;
    if (call(3,fd,0,0,0,0)) return 825;
    fd=topal_test_syscall4(257,(word)-100,(word)"/tmp/stat",0x42,0600);
    if (fd<3 || fd>=16) return 826;
    if (call(1,fd,(word)payload,4,0,4)) return 827;
    if (call(5,fd,out,0,0,0) || !record(out,2,0100000,1,4)) return 828;
    if (call(262,(word)-100,(word)"/tmp/stat",out,0,0) || !record(out,2,0100000,1,4)) return 829;
    if (call(262,(word)-100,(word)"/tmp/stat",bad,0,-14)) return 830;
    if (call(263,(word)-100,(word)"/tmp/stat",0,0,0)) return 831;
    if (call(5,fd,out,0,0,0) || !record(out,2,0100000,0,4)) return 832;
    if (call(3,fd,0,0,0,0)) return 833;
    if (topal_test_syscall4(12,0x619000,0,0,0)!=0x619000) return 834;
    return 0;
}
