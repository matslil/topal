# Step 4 bounded virtio queue notification

## Outcome

The Topal kernel now submits its published virtio-rng request by notifying queue
zero in the region described by the discovered notification capability. Checked
source performs one two-byte write of value zero through configuration-access
capability `0x84` to the region described by capability `0x70`, BAR 4, region
offset `0x3000`, then emits `TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED`.

The operation reuses generic PCI-region access. Source owns queue selection,
notification-offset arithmetic, width, value, and order; the x86-64 provider
owns only its existing configuration-gateway realization.

## Validation boundary

Rule `TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-NOTIFY-001` requires verified queue
activation before the exact queue-zero notification. Negative tests reject a
malformed, missing, duplicated, substituted, reordered, or premature write,
including substitution of the notification descriptor for the distinct access
gateway. The model preserves device ownership and grants no completion
authority.

The checked root contains 129 fatal edges, an exact 276-event semantic trace,
2,326 relocations, and 1,995 UART calls. The sealed provider plan remains 74
operations because notification reuses the existing generic access primitive.
Artifact schema `/84`, provider and provider-object schemas `/80`, artifact-root
schema `/82`, boot-adapter schema `/44`, and gate schema `/86` identify the
contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. Notification follows final activation at the qualification boundary,
so QEMU executes the real device submission write after all independent
interrupt, scheduler, and userspace observations.

Two successful independent pinned-QEMU runs were byte-identical and reached
all 62 ordered observations. Hashes are:

- linked kernel: `a85811c6f2c7df98b81dd83425fa97be68b33d66d3624e4411ce9dbcb693d69c`;
- boot image: `d559537cfe22c23e90fac5ead20c54ad16e22e67b0226d5501b0fd2cc144a943`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `471aaeb5b83a824ce07425b192beeff0438f7c43a8db40dd65b1c698c2e3d758`;
- boot provenance: `e4c6320f92d9890ec9c1cdb1561d1f222a5b7fdaa7cef1d32cfb029e0613b7a6`;
- serial output: `a8d97363601d0abb7e2d8dbb7b2ebdd1dacbb7627c0a6f870e5e26dc7f41d4de`;
- evidence: `4c86637f6573560f8582649c339bdf958aa72000fe76b90cdff1206c112da704`.

## Risk and remaining boundary

Risk is high because the write makes a DMA request observable by a real
device. Review covered prior device ownership and activation, exact capability
and BAR-relative region selection, queue/multiplier arithmetic, access width
and value, absence of synthetic readback, provider encodings, and duplicate
physical execution.

Completion observation, virtio interrupt handling, used-ring validation, CPU
ownership return, entropy delivery, `AT_RANDOM`, and `getrandom` remain
unavailable.
