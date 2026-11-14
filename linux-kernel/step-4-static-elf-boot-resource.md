# Step 4 static ELF boot-resource transport

## Outcome

The qualified x86-64 boot adapter now requires the provenance-bound
`static-user.elf`, verifies its digest, and publishes its exact 2,024,544 bytes
as the second member of the deterministic initramfs. The existing 12 KiB
`init` remains first and remains the only selected executable; the archive now
has exact order `init`, `static-user.elf`, `TRAILER!!!` and total size
2,037,200 bytes.

Boot provenance binds the complete archive plus both member names, sizes, and
payload digests. The x86 provider validates the enlarged Linux initrd handoff,
both CPIO records, padding, trailer, and the complete 680-byte static ELF
header/program-header metadata. It reserves and excludes the complete archive
from frame allocation while skipping all remaining caller-owned static payload
bytes. No BusyBox bytes are embedded in provider code.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-BOOT-001` adds no core-language element. The
compiler-side resource test proves exact payload offsets 116 and 12532, exact
member bytes, exact terminal placement, deterministic generation, and empty
member rejection. The boot adapter rejects an absent static output and digest
mismatch before publication. Provider construction compares the exact archive
metadata, static ELF metadata, total size, overflow-free handoff extent, and
initial identity-map bound; the existing boot-memory reservation logic then
excludes the whole 2,037,200-byte resource.

Artifact `/114`, provider and provider-object `/105`, boot-resource `/2`, boot
adapter `/69`, and gate `/116` identify this executable contract. Root object
`/103`, the 149 fatal edges, 332-event semantic trace, 3,360 typed relocations,
2,966 UART calls, and 90 ordered observations remain unchanged.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs accepted the exact
two-member resource, reached `TOPAL_KERNEL_BOOT_RESOURCE_OK`, and then reached
all remaining 90 ordered observations with identical hashes. The unchanged
static-image digest proves that the clean-room input passed exactly through
artifact and boot publication. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `808ed742ce2a3ee7e796d96432b218e345f0cc8e412c67edca711313fc057652`;
- boot image: `836493116c6b0c91dcc72ae65319ac9f3b1f2fbfd7dc6217ff687bdd7f633fb2`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `381dac060ec4dc5e2374dc2f1f58e02671cba04255a88c1bb90e2d9f35ebc8d8`;
- boot provenance: `ce2073a346236fa2fc5693e7d8986bde9b0e517fec4ea2110427a21873593a1b`;
- serial output: `bde7294a4b0ba8dbc89ae3bcd350c8c5a279c951c4e6f4312e7c6d0391609bf6`;
- evidence: `38af27c00b0d10176ac60cd1d921273eeccbf35657761f11c8eb8f88edef418b`.

## Risk and remaining boundary

Risk is high because a wrong archive bound could expose input memory to the
allocator, while overvalidation could smuggle caller payload into provider
code and undervalidation could detach later copy facts from the selected file.
Review covered exact layout arithmetic, both provenance chains, whole-resource
reservation, metadata-only runtime comparison, negative tamper paths, and
duplicate physical execution.

The static member remains provider-private and no byte has entered the zeroed
503-frame construction. The next increment must add a typed consuming copy
transition that reads the four exact validated file intervals, verifies the
constructed result, and retains rollback authority before final permissions.
