# Bounded static ELF entropy-transport discovery

This record resolves `TK-DEC-160` as the complete read-only virtio transport
discovery performed while the static process remains behind its acquisition
owner. It is a systems-profile refinement only and adds no core-language
element.

## Decision

After `TK-DEC-159`, `static image construction qualify entropy transport` shall
consume and return the exact acquisition owner. It shall accept only transport
identity `initial-virtio-rng-pci-transport` and policy
`bounded-static-elf-entropy-transport-discovery`.

The x86-64 provider shall completely revalidate the retained process, then
perform the remaining 23 reads of the existing exact 24-dword Q35 virtio-rng
configuration-discovery sequence. Together with the immediately preceding
identity read, these observations prove the sealed capability list, BAR roles,
offsets, lengths, and notification multiplier without a configuration write or
BAR access. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK`.

## Architecture pressure test

The portable contract names an opaque qualified transport, not PCI capability
offsets or an ECAM mechanism. X86-64 binds it to the pinned Q35 capability
sequence; AArch64 and RISC-V may use different qualified discovery mechanisms
while preserving read-only observation, complete process ownership, and no
exposed representation.

## Alternatives rejected

A synthetic transport-ready token was rejected because it would not exercise
the real capability chain. Combining discovery with feature negotiation or BAR
access was rejected because those operations write device state and have
different rollback obligations. Releasing the process before discovery or
exposing the capability fields to source was rejected because either would
weaken the coexistence and abstraction boundaries.

## Deliberate bounds

This increment admits one complete read-only discovery sequence and same-owner
return. It rejects another owner, transport, policy, order, value, omission,
duplication, write, BAR access, process mutation, output use, or recoverable
disposition. It does not negotiate, allocate DMA, drive a queue, consume
entropy, publish `AT_RANDOM`, enter the static process, or add a syscall.
