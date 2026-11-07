# Step 4 bounded static ELF entropy-queue discovery

## Outcome

The qualified x86-64 path now performs reversible virtio-rng queue discovery
while the complete static ELF process remains behind its opaque acquisition
owner. It completely revalidates the retained process, replays the exact
16-access feature negotiation, executes the existing exact five-access queue-
discovery sequence, and proves queue zero exists with size eight, disabled
state, and notification identity zero. It then writes and reads back device
status zero. Every access also restores and verifies the private PCI
configuration gateway scratch fields. The transition returns the same affine
owner and publishes `TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK` before the
existing nested rollback.

The checked source names only the acquisition owner, portable transport and
queue identities, and policy. It cannot observe a PCI coordinate, capability
field, BAR, register, access width, feature word, queue size, notification
offset, address, port, process resource, allocator state, or entropy byte. The
qualification performs no bus mastering, DMA allocation, queue construction
or enablement, notification, process mutation, or entropy use.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-DISCOVERY-001` adds no core-
language element. Negative tests reject substituted owner, transport, queue,
policy, and marker. The affine semantic model requires the exact live
acquisition owner and returns it unchanged. Structural tests prove complete
process validation, one new typed provider dependency, 16 exact negotiation
accesses, five exact queue observations, status reset, configuration-gateway
rollback after every access, and preservation of the later ordinary sequence.

Artifact `/144`, provider and provider-object `/134`, root object `/133`, gate
`/146`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 374 operations; the provider plan contains 137; the
root contains 265 fatal edges, a 435-event semantic trace, 5,890 typed
relocations, 5,264 UART calls, 99 required text symbols, 48 PCI-configuration
read calls, 30 ordinary PCI-region reads, one bounded eventual PCI-region read,
37 PCI-region writes, and 140 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
140 observations in order and produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `3951f19653c02ad235480eea1ab650f0ca2adcdd8bd237e8c92bdc7ab2fd09bf`;
- boot image: `e0647941c5fd3181215a457c893bc27caf13cf02a57b4cd9200d3760c5beffff`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `4035273b8b7334ed545ce6de07fd673a481e2d64d2c1dccfb19ff1ca61901fc9`;
- boot provenance: `35967a2ad6330f230a83f9a08f78884968512e8634a146de3ba1baf272316056`;
- serial: `a75b77641a77c9995799394d2c483ba21b6b468398e5b572cb119972771795d9`;
- evidence: `91d1a19d07fdc987e13f404f12e60ee9997fcaaffb24c2c1768530761b11542c`.

## Risk and remaining boundary

Risk is high because the increment repeats real device and PCI configuration
writes while retaining an unrelated complete process. Review covered exact
affine owner return, complete retained-state validation, every negotiation and
queue access, queue facts, device reset, gateway scratch rollback, preserved
later-sequence accounting, structural call counts, negative cases, exact
nested rollback, and duplicate successful physical execution.

This increment only discovers a queue. It does not authorize bus mastering,
allocate or bind DMA, construct or enable a queue, notify the device, retain
the owner across DMA or extraction, consume entropy for the static process,
publish `AT_RANDOM`, initialize TLS, construct a user context, enter ring 3,
execute BusyBox, or add a syscall.
