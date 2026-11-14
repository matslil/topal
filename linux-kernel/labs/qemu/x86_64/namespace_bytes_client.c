/* Independent byte-helper oracle; never linked into the kernel. */
#include "namespace_component_host.h"

int main(void) {
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    if (page != 4096) fail("qualification requires 4096-byte host pages");
    size_t bytes = 2 * page;
    unsigned char *mapping = mmap(NULL, bytes + 2 * page, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED || mprotect(mapping + page, bytes, PROT_READ | PROT_WRITE)) exit(2);
    unsigned char *payload = mapping + page;
    unsigned char expected[8192];
    for (unsigned placement = 0; placement < 3; ++placement) {
        unsigned char *base = payload + (placement == 0 ? 0 : placement == 1 ? 1 : page);
        for (uint64_t length = 1; length <= 255; ++length) {
            memset(payload, 0xa5, bytes);
            memset(base + 3200, 0, 256);
            for (uint64_t i = 0; i < length; ++i) base[3200 + i] = (unsigned char)(128 + i % 128);
            memcpy(expected, payload, bytes);
            invoke(base, 4096, 1, 0, length, 1);
            if (memcmp(payload, expected, bytes)) fail("validation changed owner bytes");
            memcpy(expected + (base - payload) + 1536, base + 3200, 256);
            invoke(base, 4096, 1, 1, length, 1);
            if (memcmp(payload, expected, bytes)) fail("copy missed exact name-only delta");
            invoke(base, 4096, 1, 3, length, 0);
            if (memcmp(payload, expected, bytes)) fail("nonzero check changed bytes");
            memset(expected + (base - payload) + 1536, 0, 256);
            invoke(base, 4096, 1, 2, length, 1);
            if (memcmp(payload, expected, bytes)) fail("clear missed exact name-only delta");
            invoke(base, 4096, 1, 3, length, 1);
            if (memcmp(payload, expected, bytes)) fail("zero check changed bytes");
        }
    }
    unsigned char *base = payload;
    for (unsigned index = 0; index < 255; ++index) {
        for (unsigned kind = 0; kind < 2; ++kind) {
            memset(payload, 0xa5, bytes); memset(base + 3200, 'x', 255); base[3455] = 0;
            base[3200 + index] = kind ? '/' : 0;
            memcpy(expected, payload, bytes);
            if (mprotect(payload, bytes, PROT_READ)) exit(2);
            invoke(base, 4096, 1, 0, 255, 0);
            if (memcmp(payload, expected, bytes)) fail("invalid input changed read-only owner");
            if (mprotect(payload, bytes, PROT_READ | PROT_WRITE)) exit(2);
        }
    }
    for (unsigned kind = 0; kind < 4; ++kind) {
        memset(payload, 0xa5, bytes); memset(base + 3200, 0, 256);
        uint64_t length = kind == 0 ? 0 : kind == 1 ? 256 : kind == 2 ? 1 : 2;
        if (kind >= 2) memset(base + 3200, '.', (size_t)length);
        memcpy(expected, payload, bytes);
        invoke(base, 4096, 1, 0, length, 0);
        if (memcmp(payload, expected, bytes)) fail("bad component changed owner");
    }
    memset(base + 3200, 0, 256); base[3200] = 'x'; base[3455] = 'y';
    memcpy(expected, payload, bytes); invoke(base, 4096, 1, 0, 1, 0);
    if (memcmp(payload, expected, bytes)) fail("bad padding changed owner");
    if (mprotect(payload, bytes, PROT_READ)) exit(2);
    invoke(base, 4096, 0, 1, 1, 0); invoke(base, 4096, 0, 2, 1, 0);
    if (memcmp(payload, expected, bytes)) fail("read-only invocation changed owner");
    if (mprotect(payload, bytes, PROT_READ | PROT_WRITE)) exit(2);
    for (uint64_t extent = 0; extent < 4096; ++extent) invoke(mapping, extent, 1, 0, 1, 0);
    invoke(mapping, 4096, 1, 0, UINT64_MAX, 0);
    invoke(mapping, 4096, 1, UINT64_MAX, 1, 0);
    if (munmap(mapping, bytes + 2 * page)) exit(2);
    printf("NAMESPACE_BYTES_NATIVE_PASS calls=%llu maximum_peak=%llu bound=%llu\n",
        (unsigned long long)calls, (unsigned long long)maximum_peak, (unsigned long long)NS_LIMIT);
    return 0;
}
