# External ELF payload authority contract

This record resolves `TK-DEC-061` as a systems-profile refinement. It removes
the provider's second embedded copy of the initial executable payload from
runtime admission and verification. The immutable boot resource becomes the
authority for admitted payload bytes, while sealed archive and ELF metadata
continue to bound their interpretation. It changes no core-language rule.

## Decision

Runtime admission must independently validate the exact bounded archive
envelope, member identity, ELF header, program-header table, load extent,
entry, and permission policy. Bytes inside the validated executable load
payload are then accepted from the immutable boot resource without comparison
to provider-immediate executable bytes.

The installation operation must copy every admitted payload qword from the
resource into the writable construction mapping. After write authority is
revoked, verification must compare every mapping qword back to the same live
resource extent. Only then may execute authority be published. A source or
destination change, short copy, mismatched qword, lost resource authority, or
metadata mismatch fails closed.

```text
validate sealed archive + ELF metadata around external payload
  -> copy complete external load extent into RW/NX construction mapping
  -> revoke write -> compare complete RO/NX mapping to source extent
  -> publish RO/X -> enter -> exact return
  -> external-copy witness while mappings remain resident
```

After `TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK` and before the existing data
witness, source publishes `TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK`. Portable
source does not observe payload bytes, addresses, offsets, qword width, archive
fields, ELF fields, registers, or generated instructions.

## Initial bounded x86-64 qualification

The initial provider admits only the existing deterministic `newc` envelope,
one `init` member, the fixed 848-byte ELF extent, the exact 256-byte ELF
header/program-table prefix, and the terminal archive record. Its 592-byte
entry payload is not embedded in resource-admission or mapping-verification
code. Generated x86-64 code performs 106 resource loads and destination stores
during installation, then 106 resource loads and destination comparisons after
write revocation.

Build provenance still binds the generated payload digest. That evidence proves
reproducibility; it is not a runtime equality oracle. The initial board trusts
the boot chain to supply the published immutable resource, as it already trusts
the kernel image. Authenticated boot and hostile-bootloader policy remain a
separate security profile.

## Cross-architecture pressure

The portable contract distinguishes structural executable metadata from the
opaque byte payload it bounds. It requires same-resource provenance across
admission, copy, verification, and entry, but prescribes neither qwords nor an
ELF representation. Byte-granular, vector, DMA-assisted, or capability-aware
providers may implement the copy differently while proving the same complete
extent, immutability, destination isolation, W^X, and entry correspondence.

This increment does not admit arbitrary metadata, multiple load segments,
BSS, relocations, interpreters, dynamic linking, PIE, ASLR, compression,
filesystem lookup, demand paging, writable code, resource replacement, or
authenticated boot.
