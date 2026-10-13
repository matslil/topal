# Runtime-derived executable-extent contract

This record resolves `TK-DEC-063` as a systems-profile refinement. It removes
the generated executable byte count from the provider's copy and verification
instruction shape. It changes no core-language rule.

## Decision

The loader must derive the code segment's file and memory extents from the
validated external ELF program header at runtime. It must validate their
relationship and bounds before mapping population, retain that resource as the
only payload authority, and use bounded loops for both installation and
post-protection comparison.

For the initial profile, code file size must equal memory size, include the
complete resident ELF header and program-header table, be qword-complete, and
fit in one base page. The executable entry and segment placement remain sealed.
The size fields are no longer compared byte-for-byte with provider-generated
immediates.

```text
validate ELF identity and code filesz == memsz
  -> require table-end <= filesz <= one base page and filesz mod 8 == 0
  -> copy filesz bytes from the external resource with a bounded loop
  -> revoke write authority
  -> compare filesz bytes to the same resource with a bounded loop
  -> publish executable authority
```

After the external-copy witness and before the data/BSS witness, source
publishes `TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK`. Portable source does not
observe the byte count, loop, ELF offsets, addresses, or instruction family.

## Failure and authority

Zero, truncated, unequal, misaligned, overflowing, overlapping, or over-page
extents fail before executable mapping population. Copy and verification must
terminate exactly at the validated bound. A mismatch after write revocation
fails closed and executable permission is never published.

The immutable external resource remains authoritative. The generated sample
image fixes qualification input, but the provider's runtime copy and compare
control flow must not be unrolled from that sample's build-time length.

## Cross-architecture pressure

The common contract is a validated bounded segment extent and complete
population/verification. It does not prescribe ELF class, machine-word width,
loop instructions, page-table format, cache maintenance, or translation
completion. X86-64 may use qword loops; AArch64 and RISC-V providers must obey
their own data and instruction publication requirements.

This increment does not make placement or entry source-selectable, generalize
the data extent, admit writable data, add more segments, or provide
relocations, interpreters, dynamic linking, PIE, ASLR, demand paging, path
lookup, or a general process loader.
