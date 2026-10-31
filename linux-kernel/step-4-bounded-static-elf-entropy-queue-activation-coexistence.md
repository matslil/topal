# Step 4 bounded static ELF entropy queue-activation coexistence

## Outcome

The qualified x86-64 path now performs one reversible empty-queue activation
while the complete static ELF process remains behind its opaque acquisition
owner. It revalidates the retained process, constructs the exact split ring,
withdraws and verifies the available index while CPU-owned, prepares both DMA
buffers, publishes all three queue references, enables queue zero, and verifies
`DRIVER_OK` without making work available or notifying the device.

The transaction resets device status and proves status, queue enable, and all
three logical 64-bit queue-address registers are zero before returning either
buffer to CPU ownership. It then clears and reads back the ring, reverses bus
mastering, and restores configuration-gateway state. No resolved reference
escapes, the same affine owner returns, and
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK` precedes nested rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-ACTIVATION-COEXISTENCE-001` adds
no core-language element. Negative tests reject substituted owner, policy, and
marker; the shared exact parser also rejects substituted device, domain, queue,
or transport. Structural tests prove complete process validation, CPU-owned
availability withdrawal, exact preparation and publication, enable/readback,
`DRIVER_OK`/readback, absence of notification, reset and complete transport
zero readback before DMA cancellation, ring erasure, and later-sequence
preservation.

Artifact `/150`, provider and provider-object `/140`, root object `/139`, gate
`/152`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 386 operations; the provider plan contains 143; the
root contains 534 fatal edges, a 447-event semantic trace, 6,799 typed
relocations, 5,551 UART calls, 105 required text symbols, 48 PCI-configuration
reads, thirteen masked PCI-configuration updates, twelve DMA-buffer creation
calls, five opaque DMA-reference bindings, 41 DMA integer stores, eight DMA
preparations, eight DMA completions, 25 DMA integer loads, nine PCI-region
DMA-reference publications, 88 control references, fourteen payload
references, 111 ordinary PCI-region reads, one bounded eventual read, 117
PCI-region writes, 136 opaque-storage derivations, and 146 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 146 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `d028d21aa25688e98d933466edfb7364cddc5077cac32909d22a6c1f54a07e5c`;
- boot image: `b7893ef05664bc6d2162b8b9509b21b2615026b589243355e0227f932bd46648`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `f591d8efc00abe8ec64548a5c2b014515b1fac41b0110cacaa2b1750dbd8e30c`;
- boot provenance: `02c78113eab349362140619e9b784ae07b209c1d63566d10f21ebcffad58561e`;
- serial: `e935c87fc79fdc501d08b4e6361d4427ebdd5e35fdb9b84e76dd8fb987fc37c7`;
- evidence: `6bb62a1d72e3798e7f455da9507467208a31b2b2d249409047e96dc0899be230`.

## Risk and remaining boundary

Risk is high because the increment temporarily activates a real queue while
retaining an unrelated complete process. Review covered the polling-device
race, zero-availability proof before device ownership, exact activation,
absence of notification, reset semantics, complete transport readback,
reference opacity, DMA cancellation, ring erasure, bus-master rollback,
negative cases, and duplicate complete physical execution.

This increment submits no work and returns no DMA owner. It does not notify the
queue, observe completion, consume entropy for the static process, publish
`AT_RANDOM`, initialize TLS, construct a user context, enter ring 3, execute
BusyBox, or add a syscall.
