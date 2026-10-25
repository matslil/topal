# Step 4 bounded static ELF entropy split-ring coexistence

## Outcome

The qualified x86-64 path now performs one reversible split-ring construction
while the complete static ELF process remains behind its opaque acquisition
owner. It revalidates the retained process; replays negotiation, queue
discovery, bus mastering, and both DMA-buffer qualifications; binds the opaque
payload reference; and performs the exact four existing ring stores with the
available index last.

The transaction then clears every field in reverse publication order, reads
all five fields back as zero at their exact widths, reverses bus mastering, and
restores device and gateway state. The 64-bit reference is erased without ever
becoming a source-visible address. The same affine owner returns and
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK` precedes nested rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-SPLIT-RING-COEXISTENCE-001` adds no
core-language element. Negative tests reject substituted owner, device, domain,
queue, policy, and marker. Structural tests prove complete process validation,
exact prerequisite replay, opaque binding, available-index-last construction,
reverse-order erasure, exact-width zero readback, complete device rollback, and
preservation of the later ordinary sequence.

Artifact `/147`, provider and provider-object `/137`, root object `/136`, gate
`/149`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 380 operations; the provider plan contains 140; the
root contains 362 fatal edges, a 441-event semantic trace, 6,236 typed
relocations, 5,400 UART calls, 102 required text symbols, 48 PCI-configuration
reads, seven masked PCI-configuration updates, six DMA-buffer creation calls,
two opaque DMA-reference bindings, thirteen DMA integer stores, nine DMA
integer loads, 63 ordinary PCI-region reads, one bounded eventual read, 73
PCI-region writes, and 143 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 143 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `db8fa5e1ca83b58eedea33333402ff0e16389968561b4520feef66c9279a98af`;
- boot image: `4d3220603541ceaba2e8ddb4f38f8f6d30f542613cd1e9d661fac506c6567699`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `17b80ae23a4de4c046fbbd0b151bd83b6b0a1fda517096bf275988a7fb639c19`;
- boot provenance: `f0ea89b704570c41c15792c23ae05f7366121f293be138ea32c1fb0985fabd85`;
- serial: `050b8188bc03702a898514cf0c177107c745b64f0f3bc18c7f8a992dc053948d`;
- evidence: `8150d75fa5ce9c13c739c73bf74003764a0972f06639235236b20d6233606f00`.

## Risk and remaining boundary

Risk is high because the increment temporarily creates a real device-visible
reference while retaining an unrelated complete process. Review covered owner
return, retained-state validation, reference opacity, ordering, complete byte
rollback with physical readback, bus-master and device rollback, later-sequence
accounting, negative cases, and duplicate complete physical execution.

This increment returns no DMA owner. It does not prepare or publish DMA, enable
or notify the queue, grant device ownership, consume entropy for the static
process, publish `AT_RANDOM`, initialize TLS, construct a user context, enter
ring 3, execute BusyBox, or add a syscall.
