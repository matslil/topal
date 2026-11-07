# Step 4 bounded PCI bus-master authorization

## Outcome

The Topal kernel now authorizes the pinned Q35 virtio-rng function to initiate
DMA after feature negotiation and queue discovery. Checked source requests one
masked PCI command-register update for segment zero, bus zero, device five,
function zero: command bit 2 changes from clear to set while every other bit is
preserved. Successful readback emits `TOPAL_KERNEL_PCI_BUS_MASTER_OK`.

The operation is provider-neutral. It carries PCI function identity, aligned
configuration offset, mask, expected masked pre-state, and desired masked
post-state. The x86-64 provider realizes that contract through PCI configuration
mechanism one without exposing ports or instruction forms to Topal source.

## Validation boundary

Rule `TOPAL-SYSTEMS-PCI-CONFIG-UPDATE-001` requires an exact two-byte
read/verify/write/readback transaction. The word-width access deliberately
avoids writing the adjacent PCI status register and its write-one-to-clear
bits. Source validation admits only the sealed initial request and rejects a
missing, duplicated, substituted, malformed, or out-of-order authorization.

The generated provider contains one selector write, two word reads, and one
word write for the update. The checked root contains one update call, 112 fatal
edges, an exact 253-event semantic trace, 2,088 relocations, and 1,802 UART
calls. The sealed provider plan contains 69 operations.

Artifact schema `/78`, provider and provider-object schemas `/74`, artifact-root
schema `/76`, boot-adapter schema `/38`, and gate schema `/80` identify the
contract. Focused compiler, language, semantics, provider-byte, and gate-fixture
suites, formatting, and the complete workspace suite pass.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. QEMU accepted the exact command-register transition and its readback
before the kernel emitted the new marker.

Two independent pinned-QEMU runs were byte-identical and reached all 56 ordered
observations. Hashes are:

- linked kernel: `54cb491d22a389cb771032a02172a658b70af48eb9b7722946c8b14c535d258b`;
- boot image: `af513fe7862d7b352d4aa1b5a7c15d39c0c9550244d37fa1901719d75b8343ef`;
- boot resource: `4e40627cbf586de35544acdf166cd1e84476deb603773b856119285f8e1813a5`;
- generated image: `a5378022c142d7040e45b9ef8036b05338b8d28efd2c92ca5b547cb4a9fafc01`;
- supplied image: `085b9584104febbc97035fe96f1732d6b10fc266a45778f3d11f5220be50904e`;
- artifact provenance: `98789e97ff8b02256aea88a36490342b31198032494c052f45cedb45017550b5`;
- boot provenance: `f2be17270e53ecb976b506a30550f8a4bd01437c1f53667e5554bb58cd14848a`;
- serial output: `b94d083e747cefcaeb01cd7d865f14f708ca9ea99a6655af1b71dd6bd4ae162f`;
- evidence: `a8fbfaa57d8e764585aa77ec387a919fcae93d4d3c2ab5892727a76b0d35729e`.

## Risk and remaining boundary

Risk is medium because premature or over-broad bus-master enablement could
permit unintended DMA. Review covered exact device identity, command-word
width, preservation of unrelated command bits, status-register isolation,
expected pre-state, verified post-state, provider encoding, negative semantic
sequences, and duplicate physical execution.

No DMA memory is yet allocated or assigned to the device. DMA ownership,
split-ring construction, queue enablement, `DRIVER_OK`, notification,
completion, entropy delivery, `AT_RANDOM`, and `getrandom` remain unavailable.
