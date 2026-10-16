# Bounded virtio completion-signal observation

This record resolves `TK-DEC-101` as the first observation after queue
notification. It observes and acknowledges the pinned virtio-rng queue signal
in the region described by the discovered ISR-status capability, using the
separate configuration-access capability, without conflating the interrupt
signal with used-ring validation or DMA ownership return.

## Decision

After the exact queue-zero notification, the initial driver performs one
source-owned bounded one-byte read-eventually-equals-one operation through
configuration-access capability `0x84` to the region described by ISR
capability `0x50`: BAR 4, region offset `0x1000`, region length `0x1000`, and
register offset zero. The provider performs no more than 1,000,000 serialized
reads, succeeds on the first matching value, and fails closed if the bound is
exhausted. Bit zero denotes a used-buffer notification. The matching ISR read
acknowledges the transport signal as specified by virtio.

The generic PCI-region interface gains a bounded eventual-read variant for
asynchronous device state. Source policy owns capability selection, exact
region, width, expected value, bounded-operation selection, and placement after
notification. The operation identity fixes the 1,000,000-attempt limit. The
provider owns only the qualified access mechanism and bounded retry, and does
not infer interrupt, queue, completion, or acknowledgement policy.

The observation proves only that the transport reported used-buffer activity.
It does not prove the used-ring index, entry identity, completed length, or
payload contents and therefore grants no CPU access to either DMA buffer.

## Architecture pressure test

Virtio PCI ISR semantics are shared across x86-64, AArch64, and RISC-V PCI
hosts. Configuration-gateway, ECAM, and mapped-MMIO providers can perform the
same bounded exact-width observation without exposing assigned addresses.
AArch64 and RISC-V providers may realize retries with ordered volatile MMIO
loads rather than x86 port I/O. Platforms using virtio-mmio require a
separately qualified interrupt-status mapping but retain the separation between
signal acknowledgement, ring validation, and ownership return.

## Deliberate bounds

This increment admits exactly one bounded queue-interrupt ISR observation after
the sealed notification. It rejects another capability, BAR, region, offset,
width, value, attempt bound, access or target capability, omission,
duplication, reordering, premature observation, or more than 1,000,000 provider
reads. The finite retry bound is a qualification limit, not a scheduling or
timing guarantee.

It does not configure an interrupt route, accept asynchronous entry, validate
the used ring, return DMA ownership, consume entropy, populate `AT_RANDOM`,
implement `getrandom`, or define reset and recovery.
