# Bounded static ELF entropy-device coexistence

This record resolves `TK-DEC-159` as the first real entropy-side operation
performed while the static initial process remains behind its acquisition
owner. It is a systems-profile refinement only and adds no core-language
element.

## Decision

`static image construction qualify entropy device` shall consume and return
the exact `EntropyAcquiringStaticElfInitialProcess` produced by `TK-DEC-158`.
It shall accept only device identity `initial-virtio-rng-device` and policy
`bounded-static-elf-entropy-device-coexistence`.

The qualified x86-64 transition shall first completely revalidate every
retained process resource and byte. It shall then perform one read-only PCI
configuration observation of pinned function `0000:00:05.0`, register zero,
requiring the existing device/vendor dword `0x1044_1af4`. Only after both proofs
succeed shall it return the same acquisition owner and permit source to publish
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK`.

The observation reuses the existing generic checked PCI-configuration provider
boundary. It shall not release, mutate, activate, alias, or expose the process;
write configuration; touch a BAR; allocate DMA; negotiate virtio; capture a
sample; initialize or extract a generator; or consume the future output
authority.

## Architecture pressure test

The portable contract names an opaque entropy-device identity and a read-only
coexistence proof, not an x86 BDF, port, ECAM address, or instruction. X86-64
qualification binds that identity to the pinned Q35 function. AArch64 and
RISC-V providers may bind it through qualified ECAM or firmware discovery while
preserving complete process ownership, read-only device access, and no exposed
representation.

## Alternatives rejected

A synthetic device-ready flag was rejected because it would not exercise real
hardware access while the process owner is live. Repeating the whole device and
DMA pipeline was rejected because discovery, negotiation, allocation, and data
transfer have distinct failure and review boundaries. Returning process
resources before the observation was rejected because it would not prove
coexistence. Exposing the PCI coordinate to ordinary source was rejected
because the sealed device identity is the portable contract.

## Deliberate bounds

This increment admits one exact read-only device observation and same-owner
return. It rejects another owner, device, policy, value, ordering, duplicate
observation, process mutation, configuration write, BAR or DMA access, output
use, or recoverable disposition.

It does not negotiate or drive the device, span the entropy pipeline, consume
or expose entropy, publish `AT_RANDOM`, initialize TLS, construct a user
context, enter ring 3, execute BusyBox, or add a syscall.
