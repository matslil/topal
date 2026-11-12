/* TOPAL-SYSTEMS-USER-EXTENT-001. Ordinary native component test over its own
 * buffers. The linked functions are compiled from the exact old/new LLVM
 * predicate bodies. This does not qualify kernel ownership or a booted VM.
 */
#include <stdio.h>

typedef unsigned long word;
extern _Bool topal_test_clean_old(const unsigned char *, word);
extern _Bool topal_test_clean_wide(const unsigned char *, word);
static unsigned char storage[65568];
static word probes;

static int same(unsigned char *p, word n, int expected) {
    ++probes;
    return topal_test_clean_old(p,n) == expected && topal_test_clean_wide(p,n) == expected;
}

static int guarded(word n, word offset, int exhaustive) {
    unsigned char *p = storage+8+offset;
    for (word i=0; i<sizeof(storage); ++i) storage[i]=0xa5;
    for (word i=0; i<n; ++i) p[i]=0;
    if (!same(p,n,1)) return 1;
    for (word i=0; i<n; ++i) {
        if (!exhaustive && i>8 && i+8<n && i!=n/2 && i!=4095 && i!=4096) continue;
        unsigned bits = n<=65 ? 8 : 1;
        for (unsigned bit=0; bit<bits; ++bit) {
            p[i]=(unsigned char)(1u<<bit);
            if (!same(p,n,0) || p[i]!=(unsigned char)(1u<<bit)) return 2;
            p[i]=0;
        }
    }
    /* Every byte outside the tested window remains nonzero, detecting reads
     * beyond its bounds; the complete buffer must also remain unchanged. */
    for (word i=0; i<sizeof(storage); ++i) {
        unsigned char expected = i>=8+offset && i<8+offset+n ? 0 : 0xa5;
        if (storage[i]!=expected) return 3;
    }
    if (!same(p,n,1)) return 4;
    return 0;
}

int main(void) {
    if (!same((unsigned char *)0,0,1)) return 1;
    for (word n=0; n<=65; ++n)
        for (word offset=0; offset<8; ++offset)
            if (guarded(n,offset,1)) return 2;
    const word sizes[] = {24,56,4096,65528};
    for (word index=0; index<sizeof(sizes)/sizeof(sizes[0]); ++index)
        for (word offset=0; offset<8; ++offset)
            if (guarded(sizes[index],offset,offset==0)) return 3;
    printf("PRIVATE_CLEAN_COMPONENT_PASS probes=%lu\n",probes);
    return 0;
}
