# Step 4 bounded constructed-image data observation

## Outcome

Execution beginning in the finalized code-role mapping now observes the
finalized immutable-data mapping before it can transfer to the existing
qualified syscall and fault-recovery probe. The x86-64 provider installs a
sealed 32-byte payload while the code mapping is RW/NX, source revokes writable
authority, the provider verifies every payload byte, and source promotes the
mapping to RO/X. At execution time the payload reads the sentinel byte from the
separate RO/NX data mapping, traps on mismatch, and transfers only on success.

After the exact kernel return, source publishes
`TOPAL_KERNEL_USER_IMAGE_ENTRY_OK`,
`TOPAL_KERNEL_USER_IMAGE_DATA_OK`, and then the existing residency witness.
Both role owners remain live until the later data-first teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither instructions, registers, addresses, nor general relocation
or loader policy to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require the data witness
after the entry witness and before the residency witness. They reject a
missing, duplicate, or early data witness and reject teardown before all three
post-return observations.

Provider and provider-object revisions `/32` still plan 63 typed operations.
The private x86-64 payload materializes the sealed data placement, compares its
byte with the provider-selected sentinel, branches to `UD2` on mismatch, and
otherwise jumps to the unchanged provider-owned probe. Structural tests inspect
all four eight-byte payload chunks, the read/compare encoding, the conditional
failure branch, and the trap; installation and verification must agree exactly.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/33` records 62
placements and a 180-transition trace. Root revision
`topal.systems-root-object.x86_64/33` has 1,185 typed relocations, including
999 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical and
reached all 32 ordered markers. They observed the new data witness between the
entry and residency witnesses, retained the unchanged Linux `-EFAULT`
outcomes, remained in the live fatal halt, and left serial output quiescent.
Gate schema `/35` records:

- linked kernel: `b4bb424435029ff9ad7b87797394ec65b3f5ff7812b1b81929290905f2490b95`;
- boot image: `db8563fa58a1e5d66264620fe75baa50f774a48b77ee66eb8a35864167668621`;
- artifact provenance: `c5b14e4295d44a83c53861d69080356b7223bc3acd6ae342a9db318efef1a125`;
- boot provenance: `a9780652d0326791fd38a70ad67a95355a0342f9cdbae85d109224ae9ec3d630`;
- serial: `39011bacaccea0cd5fc52808cdfae9a712bd0d527435d5e766fa479ea63812c7`;
- evidence record: `118908fe17fb10205a56a6f74436c16a4e30b3b7713ae129c4acadbd716e3052`.

## Remaining boundary

The constructed mapping still contains only the provider-sealed bounded
payload. Arbitrary program payloads, source-selected entry or data references,
mapping ownership transfer, relocations, ELF parsing, arbitrary segments and
aliases, demand paging, mutable image data, and a general process model remain
excluded.
