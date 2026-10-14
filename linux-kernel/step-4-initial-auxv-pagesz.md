# Step 4 bounded initial `AT_PAGESZ` auxiliary vector

## Outcome

The provider-sealed Linux-compatible process-entry block now contains one
`AT_PAGESZ` entry before its terminating `AT_NULL` pair. The constructed entry
validates the exact provider-owned tag and 4096-byte x86-64 base-page value,
then consumes the complete 80-byte block back to the former stack top before
observing immutable image data and continuing through the existing Linux
compatibility qualifications.

After exact kernel return, source publishes the entry and initial-stack
witnesses, the new auxiliary-vector witness, and then the existing data,
transfer, constructed-syscall, and residency witnesses before data-first
teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither auxiliary-vector tags or values, addresses, pointer widths,
stack direction, alignment, word layout, nor general loader policy to Topal
source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_AUXV_OK` after initial-stack validation and before the
data witness. They reject a missing, duplicate, renamed, early, or late witness
and reject teardown before all post-return observations.

Provider and provider-object revisions `/37` plan 64 typed operations. The
private x86-64 lowering writes every word of one aligned 80-byte initial block
into the RW/NX stack page and enters at `0x601fb0`. Its semantic contents are
`argc=1`, the sealed `argv[0]` candidate, null `argv` and environment
terminators, `AT_PAGESZ=4096`, `AT_NULL`, and the null-terminated
`topal-probe` string. The provider installs and verifies every qword of a
336-byte sealed code payload. The entry validates those contents, consumes
exactly 80 bytes back to `0x602000`, and only then observes immutable data and
transfers to the compatibility probe. Every mismatch fails closed.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/38` records 62
placements and a 185-transition trace. Root revision
`topal.systems-root-object.x86_64/38` has 1,352 typed relocations, including
1,166 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 37 ordered markers. They observed the auxiliary-vector witness
between the stack and data witnesses, retained all existing Linux compatibility
and transfer outcomes, remained in the live fatal halt, and left serial output
quiescent. Gate schema `/40` records:

- linked kernel: `7c50a2770afc2bfb22895f48ab5fc6887750b9b6c066f9dd8bc95934ef2ca29f`;
- boot image: `caaeabea11db0ff285bc9dd62091c918555a1f059e2048e3532fd499c919dfe9`;
- artifact provenance: `fbe5d4e127ee63d7a63ff540cf133eb9644337769c2cdccba6318e46d6ada82d`;
- boot provenance: `e80906c77d1064d6ba3bea43d21e24b24c454106ee6c6f90037757c4e2e51139`;
- serial: `04e727163fcf9486128d6c916922b4b9561b31d5c9cfa94da6e4ff8f67834ead`;
- evidence record: `412a761b16e45353484ea56a8df274efeda4bcb7b5a5fdea461bab6c52199ae5`.

## Remaining boundary

The auxiliary vector remains provider-sealed and intentionally contains only
the base-page-size fact and its terminator. Source-selected entries,
executable or credential metadata, random state, platform strings, TLS,
vDSO/vvar, dynamic linking, arbitrary payloads, relocations, ELF loading,
arbitrary segments and aliases, demand paging, mapping ownership transfer, and
a general process model remain excluded.
