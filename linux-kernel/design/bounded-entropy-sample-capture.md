# Bounded entropy-sample capture

This record resolves `TK-DEC-104` as the first transfer of completed device
bytes out of DMA transport storage. It captures the exact 32-byte virtio-rng
payload into an opaque raw-sample resource without exposing values or claiming
entropy credit.

## Decision

After `TK-DEC-103`, one generic entropy-sample capture operation borrows
CPU-owned payload buffer identity two, copies its range `[0, 32)` into fresh
opaque sample identity one, and returns the payload buffer still CPU-owned. The
request retains device identity one, translation-domain identity one,
device-to-memory direction, provider-managed visibility, the `EntropyPayload`
role, and an explicit credited-entropy value of zero bits.

The source can move and later consume the sample identity but cannot inspect,
compare, branch on, serialize, log, or convert its bytes. The provider resolves
the opaque source and destination storage and proves an exact complete copy; it
does not condition the bytes or select trust policy. The sample remains raw and
uncredited until a later kernel entropy protocol explicitly consumes it.

Payload-buffer ownership is preserved so a later queue-recycling design can
reuse the DMA allocation. Capture does not prepare the buffer for the device,
modify queue metadata, or grant device access.

## Architecture pressure test

The operation is defined as an opaque exact-byte transfer after DMA completion,
not as an x86 load/store sequence. An x86-64 provider may use coherent ordinary
loads and stores. A noncoherent AArch64 or RISC-V provider relies on the prior
completion transition for visibility, then copies through its qualified CPU
mapping. IOMMU and bounce-buffer providers resolve their private CPU views
without returning an address to source.

The destination is kernel-owned opaque sample storage, never device storage.
No cache-line size, physical address, IOVA, mapping token, or instruction is
source-visible.

## Alternatives rejected

Reading 32 source-visible byte values was rejected because it would needlessly
expose sensitive material and encourage policy in the bootstrap source.
Crediting all 256 bits during capture was rejected because source trust,
conditioning, health tests, and pool policy are distinct from an exact byte
transfer. Consuming the DMA buffer was rejected because the allocation is
needed by later queue recycling.

## Deliberate bounds

This increment admits one fresh sample identity, payload buffer identity two,
offset zero, byte count 32, the exact existing device/domain/direction/
visibility/role facts, and zero credited entropy bits. It rejects another
identity, range, buffer, metadata, quality, omission, duplication, reordering,
or capture before payload CPU ownership.

It does not condition or mix the sample, perform health tests, credit entropy,
create a CRNG, populate `AT_RANDOM`, implement `getrandom`, recycle or renotify
the queue, configure asynchronous interrupt routing, reset the device, or
define failure recovery.
