# Bounded static ELF load plan

This record resolves `TK-DEC-125` as the metadata boundary between a supplied
static executable and later frame allocation, mapping, copying, permission,
TLS, and entry operations. It introduces a systems-level load plan without
changing the core language or claiming execution of the pinned BusyBox yet.

## Decision

The x86-64 compiler shall derive one immutable load plan from a completely
validated ELF64 file before the kernel artifact may consume any segment fact.
The bounded static envelope shall require:

- little-endian ELF64, current version, `ET_EXEC`, and `EM_X86_64`;
- a complete 64-byte header and complete 56-byte program-header table;
- at most 16 program headers and exactly four ordered `PT_LOAD` entries;
- no `PT_INTERP`, dynamic loader, or writable-executable segment;
- `p_filesz <= p_memsz`, power-of-two alignment at least one base page, and
  congruent file/virtual page offsets for every load segment;
- overflow-free file and virtual intervals wholly inside their envelopes;
- disjoint page-rounded load intervals with exact final roles R, R/X, R, R/W;
- an entry contained in the executable segment;
- a page-rounded combined span of at most 512 base pages wholly inside one
  aligned 2 MiB leaf-table window;
- one non-executable `PT_GNU_STACK` declaration;
- at most one `PT_TLS` whose file and memory ranges lie inside the writable
  load segment; and
- at most one `PT_GNU_RELRO` whose page-rounded range lies inside the writable
  load segment.

The plan shall retain only checked integers: file and memory intervals, page
intervals, permission roles, entry, total page span, optional TLS initialization
and memory extents, optional RELRO page interval, and stack executability. It
shall not retain borrowed input pointers or expose x86 page-table encodings.

The currently executable two-page qualification image remains governed by its
stricter sealed validator. This increment adds the reusable plan derivation and
tests the exact pinned BusyBox-shaped envelope; it does not silently make that
image selectable by the runtime path.

## Architecture pressure test

ELF file parsing and permission-separated load planning are portable kernel
policy. The 512-page/one-leaf-table ceiling is an x86-64 qualification bound,
not a portable systems abstraction. AArch64 and RISC-V may realize the same
validated segment plan with different table geometry and instruction-coherence
steps. Capability architectures must derive bounded executable and data
capabilities from mapped extents rather than reconstruct authority from ELF
integers.

## Alternatives rejected

Accepting the whole file as one executable blob was rejected because it loses
read-only data, writable BSS, RELRO, TLS, and W^X. Parsing program headers in
the ring-0 provider was rejected because file-format policy would become target
mechanism. Publishing unchecked header integers to Topal source was rejected
because malformed input could influence allocation before validation. Requiring
the exact BusyBox digest in the loader was rejected because the digest belongs
to qualification provenance, not general segment semantics.

## Deliberate bounds

This increment derives a plan only. It does not allocate the 503-page image,
populate leaves, copy segments, zero BSS, apply RELRO, initialize TLS, construct
BusyBox arguments/environment, enter the application, or add missing syscalls.
PIE, relocations, `PT_DYNAMIC`, shared objects, demand paging, overlapping load
pages, executable stacks, and images spanning more than one leaf-table window
remain outside the accepted envelope.
