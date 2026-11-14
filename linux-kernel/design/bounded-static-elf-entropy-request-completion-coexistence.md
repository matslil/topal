# Bounded static ELF entropy request-completion coexistence

This record resolves `TK-DEC-169` as one completed and erased device request
while the static process remains behind its entropy-acquisition owner. It is a
systems-profile refinement only and adds no core-language element.

## Decision

After `TK-DEC-168`, `static image construction qualify entropy request completion`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device`, translation-domain identity
`initial-dma-domain`, queue identity `initial-virtio-rng-queue-zero`, transport
identity `initial-virtio-rng-pci-transport`, and policy
`bounded-static-elf-entropy-request-completion-coexistence`.

The x86-64 provider shall completely revalidate the retained process and replay
the exact ordinary device sequence through queue activation with available
descriptor zero. It shall notify queue zero, observe and acknowledge the exact
bounded completion signal, return control to CPU ownership, validate used-ring
flags zero, used index one, descriptor identity zero, and completed length 32,
then return the payload to CPU ownership.

The transaction shall not capture or credit an entropy sample. It shall reset
device status and prove status, queue enable, and all three logical 64-bit
queue-address registers are zero. It shall overwrite and read back all 32
payload bytes as zero, erase and read back every source- and device-written
ring field, reverse bus-master authority, and restore verified configuration-
gateway state. No payload byte or DMA identity shall escape and the ordinary
later sequence shall remain structurally unconsumed. Success returns the same
owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK`.

## Architecture pressure test

The contract separates completion, ownership return, and secret erasure from
interrupt, cache, IOMMU, or register mechanics. Noncoherent providers must
perform their normal control-then-payload synchronization before CPU erasure;
IOMMU and bounce-buffer providers must revoke or reconcile their device-side
views. The entropy source may advance irreversibly, but no architecture may
retain or expose the discarded bytes, and later correctness must not depend on
replaying the physical entropy stream.

## Alternatives rejected

Rolling back immediately after notification was rejected because the device
may still own or mutate both buffers. Treating interrupt acknowledgement as
completion was rejected because it omits used-ring and ownership validation.
Capturing the sample in this transaction was rejected because it would collapse
the next entropy-policy boundary. Leaving random bytes or used-ring state in
reusable storage was rejected because it would weaken confidentiality and
later-state proofs.

## Deliberate bounds

This increment admits exactly one submitted 32-byte request, exact completion
and ownership return, no entropy capture or credit, complete transport reset,
complete payload and ring erasure, and same-owner return. It rejects another
owner, device, domain, queue, transport, policy, result identity or length,
missing acknowledgement, early CPU access, partial erase, escaped byte or
identity, nonzero final state, process mutation, or recoverable disposition.
It does not retain entropy, initialize a pool or generator, publish `AT_RANDOM`,
enter the static process, or add a syscall.
