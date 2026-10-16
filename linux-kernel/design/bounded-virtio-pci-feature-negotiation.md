# Bounded virtio-PCI feature negotiation

This record resolves `TK-DEC-092` as the first checked access to a discovered
PCI device region. It advances the pinned virtio-rng transport through feature
negotiation and confirmed `FEATURES_OK` without exposing an assigned BAR
address or making virtio policy a provider operation.

## Decision

The bootstrap platform authority may derive one transaction-scoped PCI region
session for a checked register read or write. A request identifies the PCI
function, the selected PCI configuration-access capability, BAR index,
BAR-relative region and register offset, legal access width, direction, and
expected or written value. The source owns this request and its register
protocol. The provider owns serialization, access-mechanism choice, ordering,
privilege, and target fault behavior.

The initial x86-64 provider realizes the requests through the discovered
Virtio PCI Configuration Access capability at configuration offset `0x84`. It
programs the capability's BAR, offset, and length fields and accesses its data
field with the target register's exact width. This is a private provider
representation of generic PCI region access, not a source-visible x86 port or
physical address. A later provider may instead resolve and map BAR 4 opaquely.

Checked source performs the Virtio 1.4 initialization sequence against the
common configuration region: reset status and observe zero; publish
`ACKNOWLEDGE` and `DRIVER`; read offered feature words `0x30000000` and
`0x00000101`; select only mandatory `VIRTIO_F_VERSION_1`; then publish and
confirm `FEATURES_OK`. Every status transition is read back. The exact sequence
is sealed for the pinned QEMU 10.2 device, while feature interpretation and
selection remain source/library policy.

## Architecture pressure test

The semantic region is identified by PCI coordinates, BAR index, relative
extent, register protocol, and width. It contains no mechanism-1 port, assigned
physical address, page-table attribute, register name, or instruction. An
AArch64 or RISC-V provider can use ECAM for the same configuration gateway or
an opaque device mapping with qualified Device memory attributes and barriers.
The observable access order and exact-width rule remain unchanged.

## Deliberate bounds

This increment admits only common-configuration offsets used for feature and
status negotiation, widths one and four, and the exact pinned sequence. It does
not generally expose PCI configuration writes, assigned BAR values, arbitrary
MMIO, or source-selected physical addresses. It does not select a queue, set
`DRIVER_OK`, configure interrupts, enable bus mastering, allocate DMA, publish
descriptors, notify the device, consume entropy, or claim recovery from a
failed device. Queue discovery, setup, and population must precede
`DRIVER_OK` under the Virtio initialization sequence.

The external transport reference is OASIS Virtio 1.4 Committee Specification
01, sections 2.1 and 4.1.3 through 4.1.5:
<https://docs.oasis-open.org/virtio/virtio/v1.4/cs01/virtio-v1.4-cs01.html>.
