# Bounded external ELF load contract

This record resolves `TK-DEC-060` as a systems-profile refinement. It changes
the source of bytes for the existing sealed executable mapping from provider
immediates to the validated `init` member of the retained boot resource. It
changes no core-language rule and does not admit a general archive or ELF
loader.

## Decision

The sealed `initial-probe` layout must discover exactly one provider-qualified
executable member inside the retained immutable boot resource. Before mapping
publication, the provider must validate the bounded archive identity, member
identity and extent, ELF class and target identity, program-header bounds,
entry correspondence, load extent, and permissions. It must populate the
existing writable construction mapping by reading the admitted external member,
then follow the existing RO/X finalization and verification lifecycle.

```text
validated opaque boot resource
  -> discover exact executable member and bounded ELF identity
  -> copy admitted load extent into allocator-backed construction mapping
  -> revoke write -> verify contents -> grant execute
  -> enter at validated ELF entry -> exact return and external-load witness
```

Portable source continues to request only the semantic `initial-probe` layout.
It cannot choose a path, offset, address, ELF machine, segment, permission,
entry, relocation, interpreter, or archive algorithm. After the existing ELF
identity witness and before the data witness, source publishes one exact
`TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK` marker while both finalized image roles
remain resident.

## Initial bounded x86-64 qualification

The first provider accepts only boot-resource schema
`topal.boot-resource.x86_64-initial-newc/1`: one deterministic uncompressed
`newc` archive with one regular executable `init` member and a terminal record.
It accepts only the exact 848-byte ELF64 little-endian `EM_X86_64` image already
qualified by `TK-DEC-058`, with two bounded program headers, entry
`0x604100`, one `PF_R|PF_X` load extent covering the complete file, and one
non-executable GNU-stack policy.

The provider must load mapping contents from the resource range, not from
immediate payload bytes in the installation operation. Complete byte identity
validation during boot-resource admission and post-copy mapping verification
remain defense in depth. Structural evidence must identify the resource loads,
mapping stores, exact extent, provider-private handoff, and absence of another
input or writable-executable phase.

An absent or duplicate member, malformed hexadecimal field, nonregular or
nonexecutable mode, unexpected owner or timestamp, overflow, missing padding
or trailer, wrong ELF identity, out-of-bounds table or segment, permission
mismatch, entry mismatch, short copy, post-copy mismatch, or changed resource
must fail closed before executable publication.

## Cross-architecture pressure

The portable contract is discovery of one qualified executable object from an
opaque immutable boot resource and population of one allocator-backed
executable mapping from its validated load extent. It does not prescribe CPIO,
ELF64, little endian, machine number, address width, page-table representation,
or instruction encoding.

AArch64 and RISC-V providers may use different ELF machine identities and
entry conventions while preserving identical bounds, immutable-input,
permission, and mapping-publication obligations. Capability architectures must
derive the destination and entry capabilities from the validated resource and
mapping authorities rather than reconstructing them from numeric fields.

This increment does not admit multiple files, directories, arbitrary names,
compressed archives, multiple load segments, BSS, relocations, `PT_INTERP`,
dynamic linking, PIE, ASLR, demand paging, filesystem lookup, credentials,
signals, threads, or a general process loader.
