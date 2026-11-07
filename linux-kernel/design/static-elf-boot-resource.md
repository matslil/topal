# Static ELF boot-resource transport

This record resolves `TK-DEC-132` as the transport boundary between the
digest-bound `static-user.elf` artifact and later inactive-image population.
It is a systems boot-adapter and provider refinement only and changes no core
Topal language rule.

## Decision

The qualified x86-64 boot adapter shall require the canonical
`static-user.elf` output, verify its SHA-256 digest against artifact
provenance, and append those exact bytes to the deterministic uncompressed
`newc` boot resource as a second regular executable member named
`static-user.elf`. The existing `init` member remains first and retains its
current 12 KiB executable meaning. The terminal record follows the complete
static member.

```text
provenance-bound initial-user.elf + static-user.elf
  -> verify both canonical digests
  -> deterministic newc { init, static-user.elf, TRAILER!!! }
  -> bind complete archive and both payloads in boot provenance
  -> immutable Linux initrd handoff and reservation
  -> provider validates exact two-member layout and static ELF metadata
```

The boot adapter shall bind the static member name, byte count, and digest in
boot provenance in addition to the complete archive digest. Missing,
duplicated, changed, reordered, renamed, or unbound input fails before boot
publication. A systems artifact which omitted the optional static output is
valid as an artifact but cannot satisfy this qualified boot profile.

## Runtime boundary

The x86-64 provider shall update its retained boot-resource contract to the
exact two-member archive. It shall validate the complete handoff bounds, exact
archive length, both CPIO headers and padding, the terminal record, and the
complete approved ELF header and eleven-program-header metadata of the static
member. The member extent shall remain inside the immutable reserved resource
and the whole enlarged resource shall remain excluded from frame allocation.

The provider shall not compare caller-owned static payload bytes with an
embedded binary. Exact-byte authority is established by the artifact digest,
adapter verification, complete archive digest, and immutable handoff; runtime
metadata validation establishes that later fixed copy facts still match the
transported file. The existing `TOPAL_KERNEL_BOOT_RESOURCE_OK` observation may
occur only after the extended resource has passed these checks.

This increment retains the static member privately. Portable source cannot
name a file, offset, address, archive field, payload byte, or parsing rule, and
no construction operation may yet read from the member.

## Architecture pressure test

Exact-byte provenance, immutable transport, whole-resource reservation, and a
distinct semantic payload identity are portable policy. AArch64 may use Device
Tree initrd bounds and another archive layout; UEFI may retain a loaded-file
object; capability systems may retain bounded read authority instead of an
integer base. None requires CPIO, Linux boot-parameter offsets, or ELF machine
values in portable source.

## Alternatives rejected

Replacing `init` was rejected because it would make the not-yet-runnable image
the active application. Publishing a second host file beside the initrd was
rejected because QEMU would not provide one coherent immutable handoff.
Concatenating raw bytes without a member boundary was rejected because later
copy authority would lack an independently validated extent. Embedding or
comparing the complete external binary in provider code was rejected because
it would duplicate and redistribute caller-owned GPL content. Trusting only
the CPIO filename was rejected because layout and ELF metadata must still
agree with the approved construction plan.

## Deliberate bounds

This increment verifies, packages, reserves, and validates one static member.
It does not expose the resource to source, copy any static payload byte into
the zeroed construction, verify copied intervals, apply final permissions,
initialize TLS or stack state, attach an address space, publish an entry, or
execute the static image. General archives, lookup, compression, multiple
static images, PIE, relocations, interpreters, and dynamic linking remain out
of scope.
