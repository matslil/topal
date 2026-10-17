# Bounded multipage initial executable

This record resolves `TK-DEC-124` as the first user-image mapping whose
executable extent spans more than one base page. It removes the immediate
one-page qualification ceiling without claiming a general ELF loader or
changing the core language.

## Decision

The initial x86-64 executable role shall consume one affine, physically
contiguous two-frame extent and publish both frames as one virtually contiguous
read-only executable segment. Construction shall copy and verify the complete
external segment while both pages are writable and non-executable, revoke write
authority for both leaf mappings, verify the complete copied extent, and only
then grant execute authority to both leaves. No intermediate page may be
writable and executable.

The qualified ELF segment begins at `0x603000`, has equal file and memory sizes
greater than 4096 and at most 8192 bytes, and retains eight-byte size alignment.
Its first page contains the ELF headers and deterministic padding; its entry and
successful terminal `exit_group` path execute from the second page. The existing
data role remains a distinct one-page mapping at `0x605000`. Mapping, protection,
verification, execute promotion, invalidation, unmapping, and frame return must
cover the exact two-page executable extent as one affine object.

Source shall request a two-frame `probe-code` backing extent but shall not name
page-table entries, physical addresses, instruction encodings, or x86
invalidation mechanics. Qualification shall emit one ordered
`TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK` witness only after the user context
has returned from the second-page terminal path and before image teardown.

## Architecture pressure test

The portable systems meaning is an executable mapping backed by a bounded
multi-frame extent, with complete construction, verification, W^X transition,
entry containment, execution, revocation, and affine return. It does not assume
x86 leaf-table encoding or instruction-cache coherence.

The x86-64 provider publishes two adjacent base-page leaves and invalidates both
addresses after each permission or presence transition. AArch64 and RISC-V
providers would additionally perform their architecture-defined translation and
instruction-coherence sequences. Capability architectures must derive the two
executable capabilities from the original extent authority and return that same
authority on unmap rather than reconstructing it from integers.

## Alternatives rejected

Compressing each new ABI probe into the remaining two bytes was rejected because
it hides a real loader constraint and cannot lead to ordinary applications.
Making the existing data page executable was rejected because it violates role
separation and would create writable-executable memory. Mapping two unrelated
single frames and treating them as one segment was rejected because partial
publication or teardown would weaken the affine extent boundary. Jumping over an
unmapped boundary was rejected because successful return would not physically
prove second-page instruction fetch.

## Deliberate bounds

This increment admits exactly two contiguous executable base pages for the
sealed initial image and retains one distinct data page. It does not admit an
arbitrary segment count or size, sparse segments, demand paging, relocations,
PIE, `PT_INTERP`, dynamic linking, TLS, a filesystem-backed executable, or the
multi-megabyte static BusyBox image. Those remain later dependency-ordered
loader work.
