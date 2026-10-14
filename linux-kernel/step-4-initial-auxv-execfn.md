# Step 4 bounded initial `AT_EXECFN` auxiliary-vector fact

## Outcome

The provider-sealed Linux-compatible process-entry block now contains one
`AT_EXECFN` fact after `AT_ENTRY` and before `AT_NULL`. Both `argv[0]` and the
new pointer-bearing fact contain the same provider-owned address of the single
bounded executable-name string. The constructed entry validates exact pointer
identity and string contents, then consumes the complete 112-byte block back
to the former stack top before continuing through the existing qualifications.

After exact kernel return, source publishes the existing process-entry
witnesses, the new executable-name fact witness, and then the data, transfer,
constructed-syscall, and residency witnesses before data-first teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither auxiliary-vector tags, user pointers, string bytes, stack
layout, nor general loader policy to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK` after `AT_ENTRY` validation and before
the data witness. They reject a missing, duplicate, renamed, early, or late
witness and reject teardown before all post-return observations.

Provider and provider-object revisions `/39` plan 66 typed operations. The
private x86-64 lowering writes every word of one aligned 112-byte initial block
into the RW/NX stack page and enters at `0x601f90`. Its semantic contents are
`argc=1`, `argv[0]=0x601ff0`, null `argv` and environment terminators,
`AT_PAGESZ=4096`, `AT_ENTRY=0x604000`, `AT_EXECFN=0x601ff0`, `AT_NULL`, and the
single null-terminated `topal-probe` string. The provider installs and verifies
every qword of a 400-byte sealed code payload. The entry validates both pointer
facts against the one in-block string, consumes exactly 112 bytes back to
`0x602000`, and only then continues. Every mismatch fails closed.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/40` records 62
placements and a 187-transition trace. Root revision
`topal.systems-root-object.x86_64/40` has 1,427 typed relocations, including
1,241 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 39 ordered markers. They observed the executable-name witness
between the entry-fact and data witnesses, retained all existing Linux
compatibility and transfer outcomes, remained in the live fatal halt, and left
serial output quiescent. Gate schema `/42` records:

- linked kernel: `b72e22e607c5b397e63e9cd548d789108d255293e440154af71d61bdb17775eb`;
- boot image: `66df6092fbcb602acd7bc476b333850aafc3baaa4c7ca5a468adfa4d79268752`;
- artifact provenance: `d47d8d5da6f60b1001d782d47d90c4ecd6c931ea49aad5d82b250953a7ceb3c4`;
- boot provenance: `4bd96abe11c5f5c054f5529006b90bc199c32e6d8a00c3e90932e74fe90d997b`;
- serial: `b4a1f11f6076f7aaf09e821b914e41e708c76094d7efc918f9381293d7655f7a`;
- evidence record: `87de5fe2135b7af7c3441d6cd31fc7158ad457d78323aff70f0ce6dff819ee97`.

## Remaining boundary

The auxiliary vector and executable name remain provider-sealed. Source-
selected arguments, names, or entries, ELF program headers, credentials,
random state, platform strings, TLS, vDSO/vvar, dynamic linking, arbitrary
payloads, relocations, general ELF loading, arbitrary segments and aliases,
demand paging, mapping ownership transfer, and a general process model remain
excluded.
