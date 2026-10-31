# Step 4 bounded initial credential auxiliary-vector facts

## Outcome

The provider-sealed Linux-compatible process-entry block now contains the
ordered `AT_UID`/`AT_EUID`/`AT_GID`/`AT_EGID` quartet after `AT_EXECFN` and
before `AT_NULL`. All four facts carry the zero-valued identity of the sealed
PID-one root profile. Constructed code validates the complete fact group, then
consumes the 176-byte block back to the former stack top before continuing.

After exact kernel return, source publishes the prior process-entry witnesses,
the new credential witness, and then the existing image and teardown witnesses.
This systems-feature refinement changes no core-language rule and exposes no
credential or auxiliary-vector encoding to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK` after executable-name validation
and before the data witness. They reject a missing, duplicate, renamed, early,
or late witness and reject teardown before all observations.

Provider and provider-object revisions `/40` plan 67 typed operations. The
private x86-64 lowering writes every word of one aligned 176-byte block and
enters at `0x601f50`. After the existing facts it stores tags 11 through 14,
each paired with zero, followed by `AT_NULL` and the shared `topal-probe`
string. The provider installs and verifies every qword of a 496-byte sealed
payload. It uses explicit 32-bit displacements for stack offsets at or above
128 and a 32-bit add to consume exactly 176 bytes back to `0x602000`; this
avoids sign extension of x86-64 eight-bit displacements. Every mismatch fails
closed.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/41` records 62
placements and a 188-transition trace. Root revision
`topal.systems-root-object.x86_64/41` has 1,470 typed relocations, including
1,284 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 40 ordered markers. They observed the credential witness
between executable-name and data witnesses, retained all prior compatibility
outcomes, remained in the live fatal halt, and left serial output quiescent.
Gate schema `/43` records:

- linked kernel: `b5d1237c17874bebb53195108f5dcf6699d8e73ffe2e38a7fe4b7a70576582ff`;
- boot image: `5de979281f0a71e7092673ae98713bd2dca776a4c1effa3c8e664072e92f25f9`;
- artifact provenance: `e477fc0f110d0b722876100793d8769343f3397f6336fb0db9cc0db53e742fdb`;
- boot provenance: `cb4b8cc654e2937ca919ead6c3f5b2ae0c2a9e16ed75d432d18e416df2d6305c`;
- serial: `56a59a029ac961c48d2eecebb59816b3b65a47552bf4c4e3d2213df4ab439b68`;
- evidence record: `870ccf6d27842215c1be816edd52b377da8aed83ad334a9a92c2ef33d90fb537`.

## Remaining boundary

Credentials remain provider-sealed. Source-selected credentials, transitions,
namespaces, capabilities, secure-execution state, ELF program headers, random
state, platform strings, vDSO/vvar, dynamic linking, general ELF loading,
demand paging, and a general process model remain excluded.
