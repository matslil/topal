# Bounded DMA device preparation

This record resolves `TK-DEC-097` as the first ownership and visibility
transition for constructed DMA objects. It prepares the entropy payload and
then the split-virtqueue control buffer for device access without exposing a
cache operation, mapping address, or architecture-specific barrier.

## Decision

A generic DMA preparation operation consumes one exclusively `CpuOwned`
buffer and produces the same opaque buffer identity in `DeviceOwned` state.
The transition names the expected buffer, device, translation domain,
direction, visibility policy, and role. The provider must complete all mapping,
pinning, cache clean/invalidate, and ordering work required to make prior CPU
writes visible and the permitted device accesses safe before it returns.

The initial virtio-rng policy prepares payload buffer two first and control
buffer one second. This ordering ensures the descriptor's referenced target is
device-accessible before the queue metadata which exposes that reference. The
payload permits device writes to its first 32 bytes. The bidirectional control
page permits device reads of the descriptor and available ring and device
writes of the used ring. Source performs no further access until a later typed
completion transition returns CPU ownership.

## Architecture pressure test

On coherent identity-mapped x86-64, the provider may realize preparation as a
compiler/device ordering boundary after proving that both static pages are
reachable by the pinned PCI function. A noncoherent provider performs the
required cache maintenance. An IOMMU provider installs and orders translations;
a bounce-buffer provider copies CPU-produced bytes and prepares its device-side
storage. All preserve the same source-visible ownership transition.

The operation deliberately expresses ownership and visibility, not an x86
fence, cache instruction, or IOMMU command. Direction and role constrain the
device's future access and remain part of the affine buffer state.

## Deliberate bounds

This increment admits exactly two preparation transitions after complete
split-ring construction: payload identity two, then control identity one. It
rejects alternate identities, order, metadata, omission, duplication, CPU
access after transfer, or preparation before construction completes.

It does not publish queue addresses, enable the queue, set `DRIVER_OK`, notify
the device, observe completion, return CPU ownership, consume entropy, or
define recovery.
