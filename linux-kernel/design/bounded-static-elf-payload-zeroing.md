# Bounded static ELF payload zeroing

This record resolves `TK-DEC-130` as the first content-construction transition
for the pinned static ELF. It is a systems-profile refinement only: portable
source requests complete deterministic initialization of an opaque static-image
construction, while the provider owns target access and verification mechanics.

## Decision

The live inactive `StaticImageConstruction` shall be consumed into one refined
owner whose complete 503-frame payload is known to contain zero in every byte.
The operation is fallible and shall:

- require the exact construction produced by the approved initial-leaf
  publication contract;
- retain the same 503 payload frames and disjoint metadata frame without
  exposing either base;
- initialize the complete `503 * 4096` byte payload, including every future
  segment gap, page padding byte, and BSS byte, to zero;
- independently verify the complete zeroed extent before success; and
- preserve all 503 inactive RW/NX leaves and the final nine absent entries.

The checked source emits exactly one
`TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK` witness while the refined owner is
live. It then uses the existing complete-revocation protocol, releases returned
metadata, reclaims and releases the indivisible payload, and only then
continues. Failure is fatal and no partially initialized owner may escape.

## Provider boundary

Source does not name a byte address, physical address, store width, loop,
mapping level, cache instruction, or register. The x86-64 provider may use the
qualified bootstrap identity reachability of the retained below-one-GiB payload
to perform the complete initialization. Before writing, it shall revalidate the
two resource intervals and verify the exact live initial-leaf table. It shall
write and then read back the complete fixed extent, reject any mismatch, and
return success only with the leaf table unchanged.

The object remains inactive throughout. No user context can observe the
temporary RW/NX construction permissions, and zeroing grants no ordinary
source byte access. Verification is provider-owned evidence refining the opaque
construction, not a source-visible array or mapping.

## Affine lifecycle

Zeroing consumes the uninitialized construction identity and produces a new
`ZeroedStaticImageConstruction` identity. The original identity cannot be
reused. Revocation accepts only the zeroed refinement for this profile. Wrong
owner, duplicate zeroing, revocation before zeroing, release before revocation,
and disposition with either construction identity live are invalid. Rollback
on provider failure remains the non-returning fatal path; recoverable partial
initialization is outside this bounded gate.

## Architecture pressure test

Complete zero-before-copy initialization and independent verification are
portable policy. AArch64 may require cache maintenance before later executable
publication; RISC-V may require different coherence operations; capability
systems may write only through a sealed temporary construction capability.
None of those mechanics changes the source operation or exposes addresses.

## Alternatives rejected

Relying on allocator-zeroed memory was rejected because the allocator contract
does not promise contents and stale bytes would leak into padding or BSS.
Zeroing only declared BSS was rejected because segment padding and gaps must be
deterministic before copies. Combining zeroing with four file copies was
rejected because it would hide the required zero-before-copy boundary and make
failure evidence less precise. Source-visible byte loops were rejected because
they expose physical access and scale target mechanics into portable source.
Skipping readback was rejected because the physical qualification would then
prove only control flow, not the complete initialization result.

## Deliberate bounds

This increment zeroes and revokes the exact inactive payload but does not copy
ELF file bytes, import or redistribute BusyBox, select a runtime image, apply
final permissions, initialize TLS or stack state, attach an address space,
publish an entry, or enter userspace. General extents, sparse initialization,
recoverable partial failure, and background or concurrent population remain
outside scope.
