# Runtime-derived data/BSS-extent contract

This record resolves `TK-DEC-064` as a systems-profile refinement. It replaces
the fixed initialized-data and BSS qword operations with bounds derived from
the validated external ELF data program header. It changes no core-language
rule.

## Decision

The loader must derive the data segment's file and memory extents from the
external ELF at runtime. It must validate `0 < p_filesz <= p_memsz`, require
both extents to be qword-complete, bound both by the segment's one-page
capacity, copy the complete initialized extent, and explicitly zero the
complete memory-only remainder.

After RO/NX publication, verification must compare all initialized qwords to
the same immutable resource and prove every BSS qword is zero. Constructed
user code must also observe the complete BSS extent before transfer.

```text
validate 0 < data filesz <= memsz <= one base page
  -> copy filesz bytes from the external resource with a bounded loop
  -> zero memsz - filesz bytes with a bounded loop
  -> revoke write authority
  -> compare filesz bytes and verify memsz - filesz zero bytes
  -> enter and observe the complete BSS extent
```

After the runtime executable-extent witness and before the existing BSS
witness, source publishes `TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK`. Portable
source does not observe segment sizes, loop structure, addresses, file
offsets, or instructions.

## Resource capacity and failure

The initial immutable ELF reserves one complete file page for the data
segment at its sealed offset. Bytes outside `p_filesz` are inert file padding,
not implicit process memory initialization. Only the validated file extent is
copied; only the validated memory-minus-file extent is zeroed.

Zero, inverted, non-qword, overflowing, or over-page extents fail before data
mapping population. Copy, zeroing, provider verification, and constructed-user
observation must terminate exactly at their respective validated bounds. A
mismatch or nonzero BSS byte fails closed.

## Cross-architecture pressure

The common contract is complete bounded initialized-data construction and
explicit zero initialization beyond the file extent. It does not prescribe ELF
encoding, machine-word width, loop instructions, page-table format, or cache
maintenance. Providers must use their target's data-publication and
translation-completion rules.

This increment retains fixed placement, immutable RO/NX data, one data page,
and the sealed sample entry. It does not admit writable data, additional
segments, relocations, interpreters, dynamic linking, TLS, PIE, ASLR, demand
paging, path lookup, or a general process loader.
