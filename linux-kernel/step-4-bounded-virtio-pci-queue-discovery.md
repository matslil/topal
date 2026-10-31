# Step 4 bounded virtio-PCI queue discovery

## Outcome

The Topal kernel now discovers the pinned Q35 virtio-rng queue after confirmed
feature negotiation. Checked source observes one queue, selects queue zero,
proves its maximum size is eight, proves it remains disabled, observes
notification identity zero, and emits
`TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED`.

The five operations continue the generic PCI region-register transaction
sequence with identities 17 through 21. They carry the selected function,
configuration-access capability, BAR-relative common-configuration region,
exact two-byte width, direction, and value without exposing an assigned BAR
address. Queue-field interpretation remains source/library policy.

## Validation boundary

Rules `TOPAL-SYSTEMS-PCI-REGION-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-DISCOVERY-001` validate exact width, alignment,
bounds, value fit, the sealed request set, and complete ordering. Negative
tests reject an overflowing two-byte value and omitted, duplicated, or
reordered queue operations.

The x86-64 provider uses the discovered Virtio PCI Configuration Access
capability at `0x84`. Structural tests prove exact byte, word, and dword data
forms while retaining the same four configuration address selections. The
checked root contains ten region reads and eleven writes, 111 fatal edges, an
exact 251-event semantic trace, 2,056 relocations, and 1,772 UART calls.

Artifact schema `/77`, provider and provider-object schemas `/73`, artifact-root
schema `/75`, boot-adapter schema `/37`, and gate schema `/79` identify the
contract. Focused compiler, language, semantics, provider-byte, and gate-fixture
suites, formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. The five real configuration-gateway accesses observed the expected
QEMU queue facts. The queue remains disabled and no DMA memory is allocated.

Two independent pinned-QEMU runs were byte-identical and reached all 55
ordered observations. Hashes are:

- linked kernel: `1d34b27d8030ecff6308ec8a7ddc8f2ba8bd1ea3e9236ef27f0f8cec2db3084a`;
- boot image: `1cdf740463f9bc33947063f8d7a23cd7110fd18b514d45ad4aa698793f82dfed`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `f232cd85859d2692282ce164a3d549852a6d427dfc80ade9e7b0e7023368728e`;
- boot provenance: `d24fc9c3c09f5005300cbefa48d305ceba0e827fc83ddf34b89ebc5ec7c08f54`;
- serial output: `a85a970a9bf901b6e4dc7a99be83f999f3ce0535a3c77a5fe20ad32044b254bc`;
- evidence: `13233c32c6bf95eaa9ce9631913f65b26e005ee765d1e79f9e8879dad2c52981`.

## Risk and remaining boundary

Risk is medium because an incorrect register width or queue fact would corrupt
later queue sizing. Review covered Virtio 1.4 initialization order, exact
two-byte semantics, queue-zero selection, disabled-state validation,
transport-relative notification identity, cross-architecture realization,
generated provider bytes, negative sequences, and duplicate physical
execution.

PCI bus mastering, DMA allocation and ownership, split-ring construction,
queue enablement, `DRIVER_OK`, notification, completion, entropy delivery,
`AT_RANDOM`, and `getrandom` remain unavailable.
