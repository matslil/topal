# Bounded virtio-PCI capability discovery

This record resolves `TK-DEC-091` as the source-owned discovery step between
the first PCI identity observation and virtio transport negotiation. It uses
the existing portable configuration transaction instead of introducing a
virtio-specific provider operation.

## Decision

The bootstrap source performs one ordered, sealed walk of the pinned Q35
virtio-rng function's conventional PCI capability list. The walk begins at the
capability pointer, follows every link through the MSI-X capability, and then
validates the five virtio vendor capabilities exposed by the pinned device:
PCI configuration access, notification, device configuration, ISR status, and
common configuration. For each virtio capability it validates the header,
BAR selector, offset, and length; for notification it also validates the
notification multiplier. It deliberately does not read the selected BAR's
assigned address into portable source policy.

The source expresses the walk as a fixed sequence of checked dword reads. The
semantic model admits exactly that sequence and assigns a fresh transaction
identity to every read. The x86-64 provider continues to implement only one
ordered configuration read through mechanism 1. Capability links, virtio type
recognition, preferred-instance policy, region validation, and BAR association
remain in checked source/library policy.

This layout follows the Virtio 1.4 PCI transport: a non-transitional driver
scans PCI capabilities, vendor capability ID `0x09` describes transport
regions, and configuration types 1 through 5 identify common, notification,
ISR, device-specific, and PCI-configuration-access structures. The bounded
profile accepts larger future capability lengths only after a later refinement;
this increment seals the exact QEMU 10.2 layout so unexpected topology or
layout changes fail closed.

## Architecture pressure test

The semantic requests contain PCI segment/BDF coordinates and configuration
offsets, not x86 ports. AArch64 and RISC-V providers can lower the identical
transactions through a qualified ECAM mapping. Device Tree or ACPI may supply
host-controller discovery and ECAM windows without changing capability parsing
or virtio selection policy. Target-specific serialization, fault containment,
endianness conversion, and addressed-I/O details remain provider obligations.

## Deliberate bounds

This increment does not map a BAR, enable memory or bus-master access, read or
write a virtio common register, negotiate features, configure queues or
interrupts, allocate DMA, or consume entropy. It proves only that the pinned
function exposes the transport regions needed by those later steps. General
PCI enumeration, arbitrary capability lists, multiple supported instances,
64-bit capability extensions, opaque BAR resolution, hotplug, and
malformed-list recovery remain deferred.

The external transport reference is OASIS Virtio 1.4 Committee Specification
01, sections 4.1.2 through 4.1.5:
<https://docs.oasis-open.org/virtio/virtio/v1.4/cs01/virtio-v1.4-cs01.html>.
