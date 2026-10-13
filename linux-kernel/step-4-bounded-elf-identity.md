# Step 4 bounded ELF identity and program-header facts

## Outcome

The finalized code-role mapping now begins with a provider-sealed ELF64 header
and two-entry program-header table. Its executable entry is the same
`0x604100` instruction selected by user entry. The Linux-compatible initial
stack contains matching `AT_PHDR=0x604040`, `AT_PHENT=56`, and `AT_PHNUM=2`
facts before `AT_NULL`; constructed code validates them and consumes the
complete 224-byte block back to the former stack top.

After exact kernel return, source publishes the prior process-entry witnesses,
the new ELF identity witness, and then the existing image and teardown
witnesses. This systems-feature refinement changes no core-language rule and
does not expose ELF bytes, addresses, or loader policy to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_ELF_OK` after credential validation and before the
data witness. They reject a missing, duplicate, renamed, early, or late witness
and reject teardown before all observations.

Provider and provider-object revisions `/41` plan 68 typed operations. The
x86-64 provider constructs an 848-byte ELF64 little-endian `ET_EXEC` image at
`0x604000`: a 256-byte header/table prefix followed by the 592-byte sealed
payload. The header names `EM_X86_64`, entry `0x604100`, program-header offset
64, entry size 56, and count two. Its table contains one `PF_R|PF_X` `PT_LOAD`
covering the exact resident image and one `PF_R|PF_W` `PT_GNU_STACK` with
16-byte alignment. Installation and verification cover every image qword
before RO/X promotion. Entry code validates the matching initial-stack facts,
`AT_NULL`, and the shared string, then restores `0x602000`. Every mismatch
fails closed.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/42` records 62
placements and a 189-transition trace. Root revision
`topal.systems-root-object.x86_64/42` has 1,500 typed relocations, including
1,314 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 41 ordered markers. They fetched the relocated ELF entry,
observed the ELF witness between credential and data witnesses, retained all
prior compatibility outcomes, remained in the live fatal halt, and left
serial output quiescent. Gate schema `/44` records:

- linked kernel: `020a470c1ec8487f9fa6582020008b313a3e6da3a331c480a3a922fdc91591c4`;
- boot image: `10540a6dfba9f799bbcce11184245a04c4762e1694492e000a52bfd8d0834f0b`;
- artifact provenance: `9c757d1d1fd653ce3a51c10af10ee252e2df21e0ab005cc6e75ac1b776f58db9`;
- boot provenance: `2efa24748e9e3277253c4328cc9fe1e362d237ad6e0483c912a173711cd16c6c`;
- serial: `bd9e7fbb69a1c3802e15d8c4ee4005781e8f042a949e20f554acb7cd737938d1`;
- evidence record: `98f7e545bfa35ee3d32c6a7c541d42d9b2b87d07cc76125250f8b63cef1f9606`.

## Remaining boundary

The ELF image remains provider-sealed. External parsing, relocations,
interpreters, dynamic linking, arbitrary segments, BSS construction, PIE,
ASLR, source-selected entry points, filesystem lookup, demand paging, and a
general loader or process model remain excluded.
