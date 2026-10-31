# Step 4 bounded Linux-compatible initial user stack

## Outcome

The prepared user context now begins with a sealed Linux-compatible process
entry block in its existing RW/NX stack mapping. The constructed entry validates
`argc=1`, the exact `argv[0]` candidate and `topal-probe` string, null `argv`
and environment terminators, and a terminating `AT_NULL` entry. It then
consumes the 64-byte block, restores the former stack top, observes immutable
image data, and continues through the existing compatibility and constructed
syscall qualifications.

After exact kernel return, source publishes the entry, initial-stack, data,
transfer, constructed-syscall, and residency witnesses before data-first
teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither addresses, pointer widths, stack direction, alignment,
word layout, string bytes, nor general loader-stack policy to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_STACK_OK` after the entry witness and before the data
witness. They reject a missing, duplicate, renamed, or late witness and reject
teardown before all post-return observations.

Provider and provider-object revisions `/36` plan 63 typed operations. The
private x86-64 lowering writes every word of one aligned 64-byte initial block
into the stack page, enters at `0x601fc0`, and installs and verifies every qword
of a 304-byte sealed code payload. The entry checks the semantic block and
string contents, consumes exactly 64 bytes back to the former stack top, and
only then observes immutable data and transfers to the compatibility probe.
Every mismatch fails closed. Structural tests inspect initialization, entry
stack selection, exact payload symmetry, semantic validation, bounded
consumption, subsequent stack balance, and fail-closed disposition.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/37` records 62
placements and a 184-transition trace. Root revision
`topal.systems-root-object.x86_64/37` has 1,321 typed relocations, including
1,135 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 36 ordered markers. They observed the initial-stack witness
between entry and data witnesses, retained all existing Linux compatibility
and transfer outcomes, remained in the live fatal halt, and left serial output
quiescent. Gate schema `/39` records:

- linked kernel: `123b97a131547ee967a6e0814c0116d61c8a9154ee3801075c41e63a18ff48c2`;
- boot image: `f9f4b9419db924b0faa824e594e6bf7226f4d84e9180a833550fb825a2b36265`;
- artifact provenance: `0b20148b14bfed62b17eac5454577d35b06651c2578b7c70e185743a45ad3a9a`;
- boot provenance: `65f3664a0ff9d504ee7dd5f7d7e2350c50ccc6b47deb43eb72b4b6b6227e0f70`;
- serial: `6ca6083055b505fcca59813781eadc1f3f6971b6d015b10cc2368f5d566b55d0`;
- evidence record: `367c51549ec2a97825f993914ef5c0309607b457b40802dfa09eeefc8bb07fdf`.

## Remaining boundary

The initial block remains provider-sealed and intentionally minimal. Source-
selected arguments or environments, general auxiliary-vector entries, random
state, executable metadata, TLS, vDSO/vvar, dynamic linking, arbitrary
payloads, relocations, ELF loading, arbitrary segments and aliases, demand
paging, mapping ownership transfer, and a general process model remain
excluded.
