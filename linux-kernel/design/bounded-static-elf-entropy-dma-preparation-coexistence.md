# Bounded static ELF entropy DMA-preparation coexistence

This record resolves `TK-DEC-166` as a reversible qualification transaction
for payload-then-control DMA preparation while the static process remains
behind its entropy-acquisition owner. It is a systems-profile refinement only
and adds no core-language element.

## Decision

After `TK-DEC-165`, `static image construction qualify entropy dma preparation`
shall consume and return the exact acquisition owner. It shall accept only
device identity `initial-virtio-rng-device`, translation-domain identity
`initial-dma-domain`, queue identity `initial-virtio-rng-queue-zero`, and policy
`bounded-static-elf-entropy-dma-preparation-coexistence`.

The x86-64 provider shall completely revalidate the retained process and replay
the exact prerequisite sequence through split-ring construction. It shall then
perform the existing payload-first, control-second DMA preparation, proving the
referenced payload becomes device-accessible before the queue metadata which
exposes it.

Because no queue address is published and the device cannot yet access either
buffer, the transaction shall cancel preparation in reverse order: control
first, then payload. It shall then erase and read back every ring field, reverse
bus-master authority, and restore verified device and configuration-gateway
state. No DMA identity shall escape and the ordinary later sequence shall
remain unconsumed. Success returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK`.

## Architecture pressure test

The contract names ownership and visibility, not cache, mapping, or barrier
instructions. Coherent identity-mapped x86-64 may use ordered metadata-only
transitions. AArch64 and RISC-V providers may clean or invalidate caches;
IOMMU and bounce-buffer providers may install and then remove translations or
copies. Every provider must make cancellation complete before returning CPU
ownership and preserve opaque identities and retained process state.

## Alternatives rejected

Leaving device ownership live was rejected because it would affect the later
ordinary lifecycle. Treating preparation as an irreversible submission was
rejected because no queue address has been published and no device access is
possible. Publishing queue addresses merely to justify completion was rejected
because it would collapse the next transport boundary. Exposing cache or IOMMU
operations was rejected because those are provider representation.

## Deliberate bounds

This increment admits exactly payload-then-control preparation and
control-then-payload cancellation inside one sealed transaction. It rejects
another owner, device, domain, queue, policy, preparation order, cancellation
order, escaped identity, final ownership or byte state, process mutation, or
recoverable disposition. It does not publish queue addresses, enable or notify
the queue, permit device access, consume entropy, publish `AT_RANDOM`, enter the
static process, or add a syscall.
