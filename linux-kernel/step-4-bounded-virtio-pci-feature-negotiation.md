# Step 4 bounded virtio-PCI feature negotiation

## Outcome

The Topal kernel now negotiates the pinned Q35 virtio-rng transport through a
generic checked PCI region-register protocol. Source resets and reads back
device status, observes device feature words `0x30000000` and `0x00000101`,
selects only mandatory `VIRTIO_F_VERSION_1`, and confirms `FEATURES_OK` before
emitting `TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED`.

The 16 operations carry PCI coordinates, the selected configuration-access
capability, BAR index, BAR-relative extent, exact register width, direction,
and value. They never expose the firmware-assigned BAR address. Virtio feature
and status interpretation remains source/library policy; the provider only
realizes generic region accesses.

## Validation boundary

Rules `TOPAL-SYSTEMS-PCI-REGION-001` and
`TOPAL-SYSTEMS-VIRTIO-PCI-NEGOTIATION-001` validate bounds, overflow, exact
widths, value fit, the sealed request set, and the complete order. Negative
tests reject omitted, duplicated, reordered, substituted, misaligned, and
out-of-region requests.

The x86-64 provider uses the discovered Virtio PCI Configuration Access
capability at `0x84`. For each transaction it programs BAR 4, target offset,
and length using dword configuration accesses, then uses an exact byte or
dword access for the data field. Structural tests prove four address selections,
the byte and dword data forms, six read calls, ten write calls, 106 fatal
edges including one per checked call, and an exact 245-event semantic trace.

Artifact schema `/76`, provider and provider-object schemas `/72`, artifact-root
schema `/74`, boot-adapter schema `/36`, and gate schema `/78` identify the
contract. Focused compiler, language, semantics, and gate-fixture suites,
formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. It performs real configuration-gateway writes and reads but does not
configure a queue or consume entropy. No disk or network device is attached.

Two independent pinned-QEMU runs were byte-identical and reached all 54
ordered observations. Hashes are:

- linked kernel: `d7f070576d69793001f10ebb72c701b5de38c6a74faac9430659d773d3a11878`;
- boot image: `db24695f9f1ba070e2600b770ff627aa1be73718e847b780e96ca47c31469f9a`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `69178c5e5509b9bc0efe1155960ba317e447eda433dd6c56adce7c1842133b00`;
- boot provenance: `5aa9a83ad44971a7c67fcf4ad8bb672fa3c0f4e17069c093d11c43af2e31c712`;
- serial output: `61f232aa72d22487ab505638b79f8f497d92f120495d169e6a9bbd3304aad86e`;
- evidence: `f6bcb4c85af9533a1e96b03f4408074daddaa1e1c4ddc6e50c224ccb64838766`.

## Risk and remaining boundary

Risk is medium because the change mutates privileged device state and an
incorrect width or order could make the device reject initialization. Review
covered Virtio 1.4 initialization obligations, exact-width accesses, feature
subset selection, status readback, configuration serialization, target-neutral
region identity, provider byte structure, negative sequences, and duplicate
physical execution.

Queue discovery and selection, `DRIVER_OK`, virtqueue memory, bus mastering,
notification, interrupts, DMA ownership, entropy delivery, `AT_RANDOM`, and
`getrandom` remain unavailable.
