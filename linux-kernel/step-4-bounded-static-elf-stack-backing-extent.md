# Step 4 bounded static ELF stack-backing extent

## Outcome

The qualified x86-64 path now reserves one opaque physical frame beneath the
live static ELF stack-slot authority, publishes
`TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK`, and releases the complete
extent before slot and entry revocation. Checked source receives neither a
physical address nor a virtual stack address.

The checked root preserves the four-frame hierarchy identity, asks the existing
bounded allocator for one unit-aligned frame at or above the first frame after
that hierarchy, retains the result only while its affine extent is live, then
clears the backing identity and restores the hierarchy identity on whole
release. Existing E820 and boot-reservation validation proves the complete
candidate allocatable and disjoint from payload, leaf metadata, hierarchy, and
earlier reservations.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-STACK-BACKING-001` adds no core-language element
and no provider operation. Source-negative tests reject substituted owners,
alternative shapes, wrong witnesses, and release of any other extent. The
semantic model admits only slot-authorized to slot-authorized-with-backing to
slot-authorized ownership before slot revocation.

Structural root tests prove the exact post-hierarchy allocation floor,
separate private hierarchy retention, backing-identity clearing, hierarchy
restoration, eleven typed allocator calls, and whole release. Artifact `/122`
and root object `/111` identify the new checked-root contract; provider and
provider-object remain `/112`, boot adapter `/69`, boot-resource `/2`, and gate
is `/124`. The root contains 161 fatal edges, a 359-event semantic trace, 3,932
typed relocations, 3,514 UART calls, and 102 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions reserved and
released the stack-backing frame, reached the new witness in order, completed
all 102 observations, remained in the terminal fatal halt, and produced
identical artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `8ef034a6e0e67516f73ced36d73116ce298215f21878bbeb150a4d4427b2cdbd`;
- boot image: `95f7ab7c8902ea7b1dbee67679bfe5caa021f85b4e073282a6c4bd9eb0591436`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `f329188d1bf67ecfae3a69e1d3caeb61d674e8d0696c694a3c0a8353a91d4c33`;
- boot provenance: `11086bde0405594211613621cf9e7410ce8f5121f1de10e05d13551987765cea`;
- serial output: `0c889329b81486770cf61dce86224fdaf78132ebfe92fdbc2b5e0b7191cae1e7`;
- evidence: `344449613fad9e7762b7c6b849cf74c9c0e620644c69970b84f27b65be231ff7`.

## Risk and remaining boundary

Risk is high because overlapping allocation, a lost hierarchy identity, or
release after slot revocation would corrupt construction ownership and later
mapping safety. Review covered allocator floors, complete extent validation,
resource disjointness, private register lifetimes, affine state transitions,
failure edges, negative source tests, structural root inspection, and duplicate
physical execution.

This increment releases the frame without reading or modifying its bytes. It
does not zero backing, install a stack leaf, initialize `argc`/`argv`/`envp` or
auxiliary-vector contents, initialize TLS, construct a user context, enter ring
3, or execute BusyBox.
