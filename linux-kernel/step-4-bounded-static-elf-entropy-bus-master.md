# Step 4 bounded static ELF entropy bus-master qualification

## Outcome

The qualified x86-64 path now performs reversible PCI bus-master authorization
while the complete static ELF process remains behind its opaque acquisition
owner. It completely revalidates the retained process, replays the exact
16-access negotiation and five-access queue discovery, performs the existing
masked command-word enable transition, and then performs its exact inverse.
Both transitions preserve all unmasked command bits and verify their result by
readback. Device status and the private configuration gateway are also restored
and verified. The transition returns the same affine owner and publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK` before nested rollback.

The checked source names only the acquisition owner, portable device identity,
and policy. It cannot observe PCI coordinates, command bits, configuration
mechanisms, addresses, process resources, allocator state, or entropy bytes.
The qualification allocates no DMA memory and constructs, enables, or notifies
no queue.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-BUS-MASTER-001` adds no core-language
element. Negative tests reject substituted owner, device, policy, and marker.
The affine semantic model requires the exact live acquisition owner and returns
it unchanged. Structural tests prove complete process validation, one new typed
provider dependency, exact prerequisite replay, two checked masked-word
updates, status reset, configuration-gateway rollback after every region
access, and preservation of the later ordinary sequence.

Artifact `/145`, provider and provider-object `/135`, root object `/134`, gate
`/147`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 376 operations; the provider plan contains 138; the
root contains 291 fatal edges, a 437-event semantic trace, 5,987 typed
relocations, 5,309 UART calls, 100 required text symbols, 48 PCI-configuration
read calls, three masked PCI-configuration updates, 41 ordinary PCI-region
reads, one bounded eventual PCI-region read, 49 PCI-region writes, and 141
observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
141 observations in order and produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `15e798376c8b1f311bc29994f442d049781ce1174fa969fe0ee3bca720fd2ab5`;
- boot image: `114292d18c8bb9026f3ea7487205e3e171657e603d78e03afb91e49617eac1da`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `87362cc907c242fa38f15c42f89b43c343219765f39ece925d552d0613b34ecf`;
- boot provenance: `766b274af1ecd20a9242e251a6d15d00c4ba39e409e55ec742c02e6f9f011ee3`;
- serial: `ef53425c5b9377db61e9889089496647e8a69e0fd2044a02c7327ccc3d41d034`;
- evidence: `d16238605a1d66ca65c0e0b0043822430d50c69b4f274880cf8b0b90fffe00af`.

## Risk and remaining boundary

Risk is high because the increment toggles real PCI device-initiation
authority while retaining an unrelated complete process. Review covered exact
affine owner return, complete retained-state validation, all prerequisite
accesses, masked enable and inverse transitions, preservation of unmasked bits,
device and gateway rollback, later-sequence accounting, structural call counts,
negative cases, and duplicate physical execution.

This increment only qualifies reversible bus-master authority. It does not
allocate or bind DMA, construct or enable a queue, notify the device, retain the
owner across DMA or extraction, consume entropy for the static process, publish
`AT_RANDOM`, initialize TLS, construct a user context, enter ring 3, execute
BusyBox, or add a syscall.
