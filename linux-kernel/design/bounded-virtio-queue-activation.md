# Bounded virtio queue activation

This record resolves `TK-DEC-099` as the transition from a fully published
split queue to a live virtio driver. It enables queue zero, verifies the queue
state, sets `DRIVER_OK`, and verifies the final device status through the
existing generic PCI-region register protocol.

## Decision

The initial virtio-rng driver performs exactly four source-owned transactions
after all three queue addresses are published:

1. write value one to the two-byte `queue_enable` register at common offset 28;
2. read back value one from that register;
3. write status value 15 (`ACKNOWLEDGE | DRIVER | FEATURES_OK | DRIVER_OK`) to
   the one-byte `device_status` register at common offset 20; and
4. read back status value 15.

Activation uses the existing generic checked PCI-region reads and writes. The
provider remains unaware of queue or status meaning and owns only serialized
access through the selected configuration gateway or opaque mapping.

## Architecture pressure test

Queue enablement and virtio status bits are transport policy shared across
x86-64, AArch64, and RISC-V systems using modern virtio PCI. An x86 provider
may use configuration mechanism one, while an ECAM or mapped-MMIO provider uses
its qualified access path. Exact widths, offsets, values, order, and readback
requirements are unchanged.

The design introduces no target register, port, fence, or assembly element.
Platforms using virtio-mmio would implement a separately qualified transport
mapping while retaining the higher-level ready-state invariant.

## Deliberate bounds

This increment admits only the four transactions above and only after complete
queue-address publication. It rejects alternate queues, registers, widths,
values, order, omission, duplication, premature activation, or missing
readback.

It does not notify the queue, observe completion, return CPU ownership, consume
entropy, handle device reset/failure, or define recovery.
