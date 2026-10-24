# Step 4 bounded static ELF entropy-transport negotiation

## Outcome

The qualified x86-64 path now performs one reversible write-capable virtio-rng
transport negotiation while the complete static ELF process remains behind its
opaque acquisition owner. It completely revalidates the retained process,
executes the exact existing 16-access feature-negotiation sequence, selects
only `VIRTIO_F_VERSION_1`, proves retained `FEATURES_OK`, then writes and reads
back device status zero. The generic PCI-region provider additionally restores
and verifies the discovered zero BAR, offset, and length fields of its private
configuration-access gateway after every access. The transition returns the
same affine owner and publishes
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK` before the existing nested
rollback.

The checked source names only the acquisition owner, portable transport
identity, and policy. It cannot observe a PCI coordinate, capability field,
BAR, register, access width, feature word, address, port, process resource,
allocator state, or entropy byte. The qualification performs no queue access,
bus mastering, DMA, notification, process mutation, or entropy use.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-TRANSPORT-NEGOTIATION-001` adds no core-
language element. Negative tests reject substituted owner, transport, policy,
and marker. The affine semantic model requires the exact live acquisition owner
and returns it unchanged. Structural tests prove complete process validation,
one new typed provider dependency, seven exact reads and eleven exact writes
added through the generic PCI-region boundary, and three restore/readback pairs
inside every configuration-gateway access. The failed first physical attempt
also established why gateway scratch rollback is part of the final contract:
without it, the later ordinary capability rediscovery stops after memory
description.

Artifact `/143`, provider and provider-object `/133`, root object `/132`, gate
`/145`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 372 operations; the provider plan contains 136; the
root contains 241 fatal edges, a 433-event semantic trace, 5,802 typed
relocations, 5,224 UART calls, 98 required text symbols, 48 PCI-configuration
read calls, 19 ordinary PCI-region reads, one bounded eventual PCI-region read,
25 PCI-region writes, and 139 observations.

## Physical evidence

After correcting the gateway rollback exposed by the first physical attempt,
two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
139 observations in order and produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `4e7a33fa3cd1ad96ccf5e420718fe5fee73ac89e8db331f167cab72be85e1f7f`;
- boot image: `bcf8acd209fa020bace0002cef93d89108cd3cbf39f207741943fc8ba048b22a`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `256c96bf36ecb05b37e462352db20b5edc937a1c4b408240e9c11f811159ee32`;
- boot provenance: `6343ae7628a19a066d2a9ad05f5879a56688d19cf7b35a9dd0c3c89519c4b47d`;
- serial: `39a60f5ad176c565724923163f38620f3e91d74855214399188c7b3bf2110eae`;
- evidence: `ac77d55df4ee97b27f3504df1152cf04d0a526173a0f336e5b396ae79d79ae03`.

## Risk and remaining boundary

Risk is high because the increment performs real device and PCI configuration
writes while retaining an unrelated complete process. Review covered exact
affine owner return, complete retained-state validation, all negotiation
accesses, mandatory feature selection, device reset, gateway scratch rollback,
preserved later sequence accounting, structural call counts, negative cases,
exact nested rollback, the initial physical failure, and duplicate successful
physical execution after correction.

This increment performs only reversible feature negotiation. It does not
discover or configure a queue, authorize bus mastering, allocate DMA, retain
the owner across DMA or extraction, consume entropy for the static process,
publish `AT_RANDOM`, initialize TLS, construct a user context, enter ring 3,
execute BusyBox, or add a syscall.
