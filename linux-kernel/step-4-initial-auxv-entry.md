# Step 4 bounded initial `AT_ENTRY` auxiliary-vector fact

## Outcome

The provider-sealed Linux-compatible process-entry block now contains one
`AT_ENTRY` fact after `AT_PAGESZ` and before `AT_NULL`. The constructed entry
validates that the provider-owned value is exactly the first instruction at
which it began, then consumes the complete 96-byte block back to the former
stack top before continuing through the existing image and Linux compatibility
qualifications.

After exact kernel return, source publishes the entry, initial-stack, and
page-size witnesses, the new entry-fact witness, and then the existing data,
transfer, constructed-syscall, and residency witnesses before data-first
teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither auxiliary-vector tags or values, entry addresses, pointer
widths, stack layout, nor general ELF-loader policy to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK` after page-size validation and before
the data witness. They reject a missing, duplicate, renamed, early, or late
witness and reject teardown before all post-return observations.

Provider and provider-object revisions `/38` plan 65 typed operations. The
private x86-64 lowering writes every word of one aligned 96-byte initial block
into the RW/NX stack page and enters at `0x601fa0`. Its semantic contents are
`argc=1`, the sealed `argv[0]` candidate, null `argv` and environment
terminators, `AT_PAGESZ=4096`, `AT_ENTRY=0x604000`, `AT_NULL`, and the
null-terminated `topal-probe` string. The provider selects `0x604000` as the
initial instruction and installs and verifies every qword of a 368-byte sealed
code payload there. The entry validates all contents, consumes exactly 96
bytes back to `0x602000`, and only then continues. Every mismatch fails closed.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/39` records 62
placements and a 186-transition trace. Root revision
`topal.systems-root-object.x86_64/39` has 1,389 typed relocations, including
1,203 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 38 ordered markers. They observed the entry-fact witness between
the page-size and data witnesses, retained all existing Linux compatibility
and transfer outcomes, remained in the live fatal halt, and left serial output
quiescent. Gate schema `/41` records:

- linked kernel: `6ff85f746ff61e870853df3ed68c63ec6cc273bb17070265cb36191253c799d0`;
- boot image: `cfc689c6a87b3300dc720c971bd7d009f099b9a40ad300190737cbdad5356b3f`;
- artifact provenance: `98132f98ac33c486937577c8626e92595525b41e1d44d89050df2cbf51f8cb1a`;
- boot provenance: `99e810bb5d92df5df3a6f4efafd90b774c319e37944e59da0ead626d197a5553`;
- serial: `5b4ef56ed320f83bbf825866d70ea726369b91dd492370861958af0bcabec96a`;
- evidence record: `9f778aef473e7e04adecae1f02caf8d4384e7a74d932a9ad346123c6f46ebc7f`.

## Remaining boundary

The auxiliary vector remains provider-sealed and intentionally contains only
the base-page-size and executable-entry facts plus its terminator.
Source-selected entries, ELF program headers, credentials, random state,
platform strings, TLS, vDSO/vvar, dynamic linking, arbitrary payloads,
relocations, general ELF loading, arbitrary segments and aliases, demand
paging, mapping ownership transfer, and a general process model remain
excluded.
