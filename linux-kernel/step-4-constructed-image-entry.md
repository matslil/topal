# Step 4 bounded constructed-image entry increment

## Outcome

The initial user instruction fetch now begins in the finalized code-role
mapping. While that mapping is RW/NX, the x86-64 provider installs a sealed
12-byte transfer payload. Source then revokes writable authority, verifies the
exact payload, and promotes the mapping to RO/X. The payload transfers to the
unchanged provider-owned syscall and fault-recovery probe, whose exact kernel
return publishes `TOPAL_KERNEL_USER_IMAGE_ENTRY_OK` before the existing
residency witness and ordered teardown.

This is a systems-feature refinement only. It changes no core-language rule
and does not expose assembly, instruction bytes, virtual addresses, or a
general executable-image format to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model distinguish entry
payload installation and verification from the second-generation fixed-byte
data qualification. They reject role substitution, verification before
read-only protection, executable publication before exact verification, a
missing or early entry witness, and teardown before both post-return
witnesses.

Provider and provider-object revisions `/31` plan 63 typed operations. The
provider writes and checks the private x86-64 `movabs`/indirect-jump payload at
the code placement and starts `iretq` at that placement; the destination probe
and fault guard remain unchanged. Artifact revision
`topal.systems-artifact.x86_64-qemu-pc-q35/32` records 62 placements and a
179-transition trace. Root revision `topal.systems-root-object.x86_64/32` has
1,154 typed relocations, including 968 UART writes. The boot adapter remains
`/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical and
reached all 31 ordered markers,
including the new entry witness, retained the unchanged Linux `-EFAULT`
outcomes, remained in the live fatal halt, and left serial output quiescent.
Gate schema `/34` records:

- linked kernel: `56a7058da93c7bb28831472574e0506982264c78e4704aaf87e256e62c3db9cd`;
- boot image: `58abc93272a8d926a5194ca8666b7a3de621dceac4ad8cc2ac09bcafffd16635`;
- artifact provenance: `411f35036163916dc663537eef96c2c96e180b4e07f54c94951dedced6352110`;
- boot provenance: `d9978a6087b11d1bfe5e52ba2718bcc83f11a83532a05a1cb9c65e0546c43d4a`;
- serial: `4145672d651654594e16ca9cd30063654d193df57f523c51ebf6e835d7ae9a70`;
- evidence record: `ce03122e43e8e6d3d9f91598faf0d117620c3bd6a2806e88fb5c07a41ada47db`.

## Remaining boundary

The constructed mapping contains only the provider-sealed transfer payload.
Arbitrary program payloads, source-selected entry, mapping ownership transfer,
relocations, ELF parsing, arbitrary segment collections and aliases, demand
paging, and a general process model remain excluded.
