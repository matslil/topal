# Step 4 bounded static ELF entropy queue-address coexistence

## Outcome

The qualified x86-64 path now performs one reversible queue-address
publication transaction while the complete static ELF process remains behind
its opaque acquisition owner. It revalidates the retained process; replays the
complete device, ring, and DMA-preparation sequence; then publishes the
descriptor, driver, and device-area references in the required order.

Because the queue remains disabled, the transaction clears the device-area,
driver, and descriptor address registers in reverse order and reads every
complete logical 64-bit register back as zero. It then cancels control and
payload preparation, clears and reads back the ring, reverses bus mastering,
and restores device and configuration-gateway state. No resolved reference
escapes, the same affine owner returns, and
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK` precedes nested rollback.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-ENTROPY-QUEUE-ADDRESS-COEXISTENCE-001` adds no
core-language element. Negative tests reject substituted owner, transport,
policy, and marker; the shared exact parser also rejects substituted device,
domain, or queue. Structural tests prove complete process validation, exact
prerequisite replay, ordered opaque-reference publication, reverse transport
zeroing and readback before DMA cancellation, complete ring erasure, device
rollback, and preservation of the later ordinary sequence.

Artifact `/149`, provider and provider-object `/139`, root object `/138`, gate
`/151`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 384 operations; the provider plan contains 142; the
root contains 471 fatal edges, a 445-event semantic trace, 6,597 typed
relocations, 5,500 UART calls, 104 required text symbols, 48 PCI-configuration
reads, eleven masked PCI-configuration updates, ten DMA-buffer creation calls,
four opaque DMA-reference bindings, 31 DMA integer stores, six DMA
preparations, six DMA completions, nineteen DMA integer loads, six PCI-region
DMA-reference publications, 66 control references, eleven payload references,
91 ordinary PCI-region reads, one bounded eventual read, 103 PCI-region
writes, 115 opaque-storage derivations, and 145 observations.

## Physical evidence

Two complete pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions produced
identical hashes and all 145 observations in order:

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `8b147a0192741943ac7100e6ab36d2a3205dd266bb2a9d63445ad79d39fd484c`;
- boot image: `f102bef69fda5e8fd1d32809ff5f3289acc67f32f0b8ff651909b31bb5424adc`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `2924cfe2f675286886a25010234b5704d6b1dc74fe25d05bbcb9c687ede5c839`;
- boot provenance: `c8ae3a718d946bb72bd0224ba2bfa2ceb5003b739b8ce22bec12f8431c4f5dd2`;
- serial: `745e3ca2c947b8575c281f3596e87070111d0052f981f02ace474c9f765776d5`;
- evidence: `0953d885a70af28e5179ca03bbe27fd58732245a506a27bd890521043718931a`.

## Risk and remaining boundary

Risk is high because the increment temporarily publishes real device-visible
references while retaining an unrelated complete process. Review covered
owner return, retained-state validation, reference opacity, publication and
clearing order, complete logical-register zero readback, DMA cancellation,
ring erasure, bus-master and device rollback, later-sequence accounting,
negative cases, and duplicate complete physical execution.

This increment leaves the queue disabled and returns no DMA owner. It does not
set `DRIVER_OK`, notify the queue, permit device access, consume entropy for the
static process, publish `AT_RANDOM`, initialize TLS, construct a user context,
enter ring 3, execute BusyBox, or add a syscall.
