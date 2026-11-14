# Step 4 bounded virtio completion signal

## Outcome

The Topal kernel now observes and acknowledges the first virtio-rng transport
signal after submitting queue zero. Checked source performs one bounded
one-byte read-eventually-equals-one operation through configuration-access
capability `0x84` to the region described by ISR capability `0x50`, then emits
`TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL`.

The operation is generic PCI-region policy rather than a virtio provider
intrinsic. Source owns the target region, width, expected bit, bounded-operation
selection, and ordering. The systems operation fixes a 1,000,000-attempt limit;
the x86-64 provider owns the serialized configuration-gateway retries and
fails closed on exhaustion.

## Validation boundary

Rule `TOPAL-SYSTEMS-VIRTIO-PCI-COMPLETION-SIGNAL-001` requires the exact prior
queue notification and separates the configuration-access capability from the
ISR region descriptor. Negative tests reject malformed, missing, duplicated,
substituted, reordered, premature, or unbounded observation. A matching ISR
read acknowledges only the transport signal: it does not validate the used
ring, completed length, payload, or CPU ownership.

The checked root contains 130 fatal edges, an exact 278-event semantic trace,
2,365 relocations, and 2,032 UART calls. The sealed provider plan contains 75
operations, including the new bounded eventual-read lowering. Artifact schema
`/85`, provider and provider-object schemas `/81`, artifact-root schema `/83`,
boot-adapter schema `/45`, and gate schema `/87` identify the contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The corrected queue notification uses configuration-access
capability `0x84`; QEMU then completes the real DMA request, exposes ISR bit
zero, and clears it through the successful side-effectful read.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 63 ordered observations. Hashes are:

- linked kernel: `df10c281b870278d78667c755f93a604869350f6048b948db3419674927e13d7`;
- boot image: `168decf5a94721db00b665b32af58bfe19288c9705fe287bb4540588d7de614e`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `1712338e173f1c98e72dada9f04f3e94e44b8995af49c61ae1de5f8a13bef6e1`;
- boot provenance: `11f82e1a4c74431d43669ef1d2db8f2c38992cf735156bfe9297a0a0698b0621`;
- serial output: `931f3d2545fe4b381a9978306ab0009194776e9fe01e4fbc7df2033eef29123c`;
- evidence: `8c0ef6a1a0b3734b86e25b2d5c50744c6e14e77715ddebaa31e4699829fbf113`.

One additional launch exited before serial output, matching the lab's known
intermittent QEMU startup failure; no stale process remained, and both counted
qualification runs completed successfully with identical evidence.

## Risk and remaining boundary

Risk is high because the increment observes and acknowledges side-effectful
device state after real DMA submission. Review covered separation of access
and target capabilities, exact region and width, finite retry and exhaustion,
provider instruction encodings, source ordering, acknowledgement semantics,
and duplicate physical execution.

Used-ring validation, completed-length checking, DMA ownership return,
noncoherent synchronization, entropy consumption, `AT_RANDOM`, `getrandom`,
asynchronous interrupt routing, reset, and recovery remain unavailable.
