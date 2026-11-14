# Inactive kernel-image backing preparation

`TK-DEC-287` is a systems-only increment adopted under the standing chat
authorization. It implements the allocation/preparation phase of
[kernel-image protection](kernel-image-protection.md), without selecting an
active protected context or changing core language semantics.

Source requests the current image's backing through the boot allocator context,
with provider-selected page policy. A single affine preparation resource owns
all complete extents. The initial operation zeroes and verifies the backing;
the only initial consumer verifies inactivity, erases and verifies every byte,
unpins and releases whole extents. Source supplies no numeric geometry or
physical access. No other boot work may interleave with the resource.

The x86 component reserves distinct bootstrap and future-process banks using
the existing ordinary Topal allocator protocol. The bank count follows the
actual image and draft provider geometry, not a source integer. Existing
whole-owner issuer/generation/count/alignment/pin/provenance/exclusion checks
remain mandatory. Additional kernel table storage does not become USER-table
admissible. The exact legacy five-page paths remain unchanged.

Preparation runs before translation construction, other CPUs, interrupts,
device work or userspace. Native checks exclude the actual active root and
the final linker-bound image reservation. All pages remain inaccessible to
source. Failure is fatal with resources retained or poisoned; discard cannot
silently resume after a partial effect. No root is loaded from prepared backing.

Sizing feedback is resolved by rebuilding and freshly reconstructing the
complete candidate image until its required whole storage matches the emitted
allocation/helper binding. Provisional images remain non-executable. The final
publisher/boot binding carries actual image identity externally, avoiding a
self-hash inside the image. A copied manifest or rehashed source descriptor
does not supply authority. The first native consumer remains a private,
explicitly qualified test producer; public protected activation stays gated.

Alternatives rejected: a source `8` exposes target geometry without ownership;
globally widening five-page checks weakens existing contracts; allocating a
separate unbound scratch buffer does not establish image-backed whole owners;
activating a partially integrated bank omits mode, writer and lifetime proofs.

Later stages may consume the preparation resource for complete inactive table
construction. All bootstrap/process/alias/restoration producers, effective
controls, publication and lifetime/fatal obligations remain required before
protected activation. Complete Linux compatibility remains an independent goal.
