# Bounded static ELF payload copy

This record resolves `TK-DEC-133` as the second content-construction transition
for the approved static ELF. It is a systems-profile refinement only: checked
source selects a sealed copy plan and consumes an opaque zeroed construction,
while the provider owns boot-resource access, offsets, addresses, loops, and
verification mechanics.

## Decision

`static image construction copy payload` shall consume the exact live
`ZeroedStaticImageConstruction` and refine it into one affine
`PopulatedStaticImageConstruction`. The sealed request shall name only
`retained-static-user-elf` and `bounded-static-elf-four-loads`; source shall
not spell a file offset, virtual or physical address, byte count, member name,
archive field, or target access mechanism.

The operation shall:

- revalidate the exact 503-frame payload, disjoint metadata frame, 503 live
  initial RW/NX leaves, and nine absent entries;
- revalidate the retained two-member boot-resource handoff and approved static
  ELF metadata before reading payload bytes;
- copy exactly the four approved file intervals into their corresponding
  memory starts inside the already-zeroed payload;
- compare every copied byte with the retained immutable source;
- verify that every gap, page-padding byte, and BSS byte outside those four
  destinations remains zero; and
- verify that all inactive leaves remain unchanged before returning success.

The checked source shall publish exactly one
`TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK` witness while the populated owner is
live. It shall then use the existing complete revocation protocol, release
returned metadata, reclaim the exact payload, and release that indivisible
extent. Failure is fatal and no partially populated owner may escape.

## Exact bounded plan

The x86-64 provider shall copy these source/destination/length triples, where
source offsets are relative to `static-user.elf` and destination offsets are
relative to virtual base `0x400000` and payload frame ordinal zero:

| Role | Source offset | Destination offset | Bytes |
| --- | ---: | ---: | ---: |
| headers | `0x0` | `0x0` | `0x6e0` |
| executable | `0x1000` | `0x1000` | `0x18b285` |
| immutable data | `0x18d000` | `0x18d000` | `0x57040` |
| writable data | `0x1e4660` | `0x1e5660` | `0x9728` |

The complete destination envelope remains `[0, 0x1f7000)`. Copying must not
change construction permissions, publish an active mapping, or make entry
authority available.

## Affine lifecycle

Copying consumes the zeroed identity; neither it nor the original
uninitialized identity can be reused. Revocation in this profile shall accept
only the populated refinement. Wrong owner, resource, plan, order, interval,
source identity, copied content, preserved-zero byte, or leaf state; duplicate
copy; revocation before copy; and disposition with any construction identity
live are invalid. The fatal path remains the only rollback for a provider
failure during population.

## Architecture pressure test

Zero-before-copy, immutable-source authority, exact interval selection,
complete post-copy verification, and affine refinement are portable policy.
AArch64 may require data-cache clean and instruction-cache invalidation before
later executable publication; RISC-V may require `fence.i`; capability systems
may copy through bounded capabilities. Those operations remain provider-owned
and do not alter the source transition.

## Alternatives rejected

Source-visible byte loops were rejected because they expose target addresses
and scale millions of operations into portable semantics. Copying the complete
file contiguously was rejected because file and virtual offsets differ for the
writable role and gaps must remain zero. Copying before whole-payload zeroing
was rejected because BSS, padding, and gaps would retain allocator residue.
Comparing only a sample or segment digest was rejected because later execution
depends on exact constructed bytes and the provider already owns bounded read
access. Applying final permissions in the same operation was rejected because
content verification and W^X publication require distinct rollback states.

## Deliberate bounds

This increment populates, verifies, revokes, and returns the inactive payload.
It does not retain the populated owner for final permissions, apply RO/X,
RO/NX, RW/NX, or RELRO roles, initialize TLS or a process stack, attach an
address space, publish an entry, or execute the image. General copy plans,
sparse files, demand paging, relocations, interpreters, and dynamic linking
remain out of scope.
