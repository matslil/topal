# Bounded static ELF entropy bus-master qualification

This record resolves `TK-DEC-163` as reversible PCI bus-master qualification
while the static process remains behind its entropy-acquisition owner. It is a
systems-profile refinement only and adds no core-language element.

## Decision

After `TK-DEC-162`, `static image construction qualify entropy bus master`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device` and policy
`bounded-static-elf-entropy-bus-master`.

The x86-64 provider shall completely revalidate the retained process, replay
the exact 16-access negotiation and five-access queue discovery, and perform
the existing exact masked command-word transition that enables bus mastering.
It shall then perform the inverse masked transition, proving the bit returns
to zero while every unmasked command bit remains preserved, and write and read
back device status zero. Every region access shall preserve the verified
configuration-gateway scratch rollback, and the later ordinary sequence shall
remain unconsumed. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK`.

## Architecture pressure test

The portable contract names an opaque device identity and authorization fact,
not PCI coordinates, command bits, configuration ports, ECAM addresses, or an
IOMMU mechanism. X86-64 binds it to the pinned Q35 PCI function; AArch64 and
RISC-V providers may use ECAM or firmware mediation while preserving exact
masked transition, unmasked state, complete rollback, and process ownership.
Non-PCI entropy devices may qualify equivalent device-initiation authority
without changing portable source.

## Alternatives rejected

Leaving bus mastering enabled was rejected because it would affect the later
ordinary lifecycle. Treating queue discovery as implicit DMA authority was
rejected because device initiation and memory ownership are separate. Combining
authorization with DMA allocation was rejected because allocation introduces
fresh resource, translation-domain, direction, visibility, and cleanup
obligations.

## Deliberate bounds

This increment admits one exact enable transition, its exact inverse, and
complete device/gateway rollback after the exact prerequisite replay. It
rejects any other owner, device, policy, coordinate, mask, value, order,
omission, duplication, final state, process mutation, or recoverable
disposition. It does not allocate or bind DMA, construct or enable a queue,
notify the device, consume entropy, publish `AT_RANDOM`, enter the static
process, or add a syscall.
