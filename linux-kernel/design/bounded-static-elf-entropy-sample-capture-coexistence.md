# Bounded static ELF entropy-sample capture coexistence

This record resolves `TK-DEC-170` as one captured, then completely erased,
raw entropy sample while the static process remains behind its entropy-
acquisition owner. It is a systems-profile refinement only and adds no core-
language element.

## Decision

After `TK-DEC-169`, `static image construction qualify entropy sample capture`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device`, translation-domain identity
`initial-dma-domain`, queue identity `initial-virtio-rng-queue-zero`, transport
identity `initial-virtio-rng-pci-transport`, sample identity
`initial-virtio-rng-sample`, and policy
`bounded-static-elf-entropy-sample-capture-coexistence`.

The x86-64 provider shall completely revalidate the retained process and replay
the exact ordinary device sequence through completed payload ownership return.
It shall perform the exact opaque 32-byte, zero-credit sample capture defined by
`TK-DEC-104`, while preserving CPU ownership of the payload. Neither the source
nor the retained process may observe a byte, address, comparison, or branch
derived from the sample.

Before the transaction returns, the provider shall reset and verify all
transport state, overwrite and read back the complete DMA payload and ring, and
overwrite and read back all 32 bytes of the provider-private raw-sample storage.
It shall then reverse bus-master authority and restore verified configuration-
gateway state. No sample, payload byte, or DMA identity shall escape and the
ordinary later sequence shall remain structurally unconsumed. Success returns
the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK`.

## Architecture pressure test

The contract is an opaque exact-byte transfer followed by verified semantic
destruction, not an x86 load/store spelling. Noncoherent providers rely on the
completed DMA transition before capture; IOMMU or bounce-buffer providers
resolve their private CPU view. Every provider must erase all live copies it
created for the raw-sample identity and prove they are no longer observable
before returning the acquisition owner.

## Alternatives rejected

Exposing sample bytes to the checked source was rejected because it would leak
secret material and allow entropy policy to depend on target representation.
Crediting or assessing the sample during capture was rejected because exact
transfer and trust policy are separate boundaries. Returning a live sample
beside the unchanged acquisition owner was rejected because it would create an
untracked affine resource. Trusting later overwrite without readback was
rejected because it would weaken the confidentiality proof.

## Deliberate bounds

This increment admits exactly one 32-byte raw sample with identity one and zero
credited bits, exact capture after payload completion, complete transport,
payload, ring, and sample erasure, and same-owner return. It rejects another
owner, device, domain, queue, transport, sample, range, credit, policy,
incomplete copy, early or partial erase, escaped byte or identity, nonzero final
state, process mutation, or recoverable disposition. It does not assess or
credit entropy, initialize a pool or generator, publish `AT_RANDOM`, enter the
static process, or add a syscall.
