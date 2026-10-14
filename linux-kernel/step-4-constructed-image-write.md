# Step 4 constructed-image standard-output write

## Outcome

The supplied initial ELF now produces its first application-owned output using
Linux x86-64 `write(2)`. Its constructed continuation reserves 32 bytes in the
existing RW/NX user stack, initializes the exact 26-byte
`TOPAL_KERNEL_USER_WRITE_OK` payload, invokes `write(1, candidate, 26)`, checks
the exact byte-count result, releases the stack region, and continues through
the existing identity and terminal syscalls.

The kernel dispatcher accepts only the checked descriptor, candidate, extent,
return site, payload, and lifecycle state. It fault-containably copies all 26
bytes to private kernel staging before any output, validates the staged value,
then publishes it through the Q35 serial provider. No core-language rule
changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-WRITE-001` forbids arbitrary descriptors, candidates,
lengths, partial output, pointer retention, wrong return sites, and output after
terminal disposition. The systems source carries the sealed descriptor and
payload facts in its syscall disposition; syscall numbers, registers, stack
address, page-fault machinery, and UART polling remain provider-private.

Artifact schema `/53`, provider `/50`, provider object `/50`, artifact root
`/51`, boot adapter `/13`, and gate `/55` identify the changed executable
contract. The boot-resource schema remains `/1`. The artifact trace retains
197 semantic events, 63 placements, and 1,794 relocation applications; the
generated root retains 1,608 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 50 ordered observations. The user-produced write marker occurs
after the boot resource and before every source-owned post-return user-copy and
syscall witness, proving it was emitted during live ring-3 execution. The
evidence records:

- linked kernel: `0bb13b4b482e9f4b50f85b8d4f00dfeb4e205b13a34715ddc9346c601b591384`;
- boot image: `f583dde430eeccf07083275d042e025a44fbf17274e3bbcac888b25a9d4a0428`;
- boot resource: `84ff08640f2b9eedcac44e4946ef30e2b8bc48e7147e15d335b0fbf5c83d484d`;
- generated default image: `e61b0c8090ccdd969557edd2378fc30ed5a6db38691c859a47cddec73e47aaae`;
- supplied image: `9babfbfe0a945b3f434d568c2f19994fd8cf0cc518916ad481462b746f34b818`;
- artifact provenance: `ee5b4a9ab886254464196d055046cd1cfaeb249fcbfb524965908f197681e542`;
- boot provenance: `3351e3900b256fddf9bd57ae687226d595c68bf65308944440c0e4ca0503cbac`;
- serial: `28e8191f0266e469c069681d94c394af0b0968ffb76ac50d74425b629b3b39dc`;
- evidence record: `0b933549008f6e4ba7309e546282eb41bb70f774155a445860a05a5df9be275e`.

## Risk and review

Risk is high because this adds observable application output, consumes a user
pointer, and drives a physical device. Review covered exact dispatch facts,
complete pre-output staging, page-fault recovery, staged-byte validation,
result and return-site validation, stack balance, terminal ordering, provider-
private x86 and UART details, negative source checks, structural instruction
checks, full workspace validation, and two independent physical runs.

## Remaining boundary

This is one closed standard-output transaction, not a reusable `write`
implementation. There is no descriptor table, VFS object, console input,
partial-write behavior, path lookup, filesystem, dynamic loader, heap, signals,
threads, process creation, or ordinary shell. Those remain subsequent
userspace-gate work.
