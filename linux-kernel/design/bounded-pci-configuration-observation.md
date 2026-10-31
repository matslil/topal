# Bounded PCI configuration observation

This record resolves `TK-DEC-090` as the first implemented device-access
refinement. It identifies the pinned Q35 virtio-rng function without moving PCI
enumeration or virtio policy into the compiler or x86 provider.

## Decision

The bootstrap platform authority may perform one checked PCI configuration
transaction. Source names the segment, bus, device, function, aligned register
offset, and expected little-endian 32-bit value. The operation derives a
transaction-scoped configuration session, performs exactly one observable
read, consumes the session, and returns the bootstrap authority only when the
value matches. Access failure or mismatch enters the fatal disposition.

The first program reads offset zero of `0000:00:05.0` and expects device ID
`0x1044` with vendor ID `0x1af4`. That address and identity come from the pinned
Step 3 Q35 manifest. They are kernel/library inputs: the semantic operation is
not named for virtio-rng and the provider does not enumerate or match devices.

The x86-64 provider realizes the transaction through PCI configuration
mechanism 1 (`0xcf8`/`0xcfc`) while only the bootstrap processor is running. It
constructs the enable/BDF/register selector internally and keeps the paired
address-write/data-read indivisible with respect to admitted configuration
access. Source cannot name those ports or mint configuration authority.

On AArch64 or RISC-V the same meaning can lower to a qualified ECAM window or a
firmware-mediated configuration service. Segment and BDF remain semantic PCI
coordinates; provider-specific physical addresses, port instructions, memory
attributes, barriers, and fault recovery remain hidden.

## Deliberate bounds

This increment does not enumerate arbitrary functions, return a value to
ordinary PCI policy, walk PCI capabilities, assign or map BARs, enable bus
mastering, negotiate virtio features, configure interrupts, allocate DMA, or
consume entropy. Those remain later library/device-session refinements. The
pinned QEMU gate adds only the selected non-transitional virtio-rng function;
the discovery proof does not touch its BARs or backend data path.
