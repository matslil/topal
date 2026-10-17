# Step 4 initial standard-descriptor duplication

## Outcome

The Topal kernel now maps each of the three initial descriptor slots to one
fixed open-description identity and access mode. Descriptor zero initially
names readable console input, one names writable console output, and two names
a distinct writable console error description. Closing a slot removes only
that mapping; another alias remains usable and retains its access mode.

Linux x86-64 `dup` selects the lowest closed slot, returns `-EBADF` for an
invalid source, and returns `-EMFILE` without state change when the bounded
table is full. `dup2` validates its source and destination before atomically
replacing the destination, including correct equal-descriptor behavior. The
ring-3 probe exercises full-table rejection, lowest-free selection, alias
survival after source-slot close, write-only read rejection, invalid-source
nonreplacement, error-description replacement, and exact cleanup.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-DUP-001` refines the approved systems profile and
`TK-KERNEL-OBJECT-001`; it adds no core-language element. Language, semantic,
provider-planning, artifact, and structural tests cover table count three,
description identities and modes, Linux errors `EBADF=9` and `EMFILE=24`,
x86-64 syscalls 32 and 33, lookup and bounds before replacement, exact ring-3
calls and results, five closes, two `dup` calls, three `dup2` calls, and terminal
clearing. The architecture pressure test retains the same portable operation
while identifying `dup3` syscall 24 as the later AArch64/RISC-V adapter basis.

The checked root contains 144 fatal edges, an exact 314-event semantic trace,
3,011 relocations, and 2,627 UART calls. The sealed provider plan contains 84
operations and retains the 160-byte entropy-storage layout. Artifact schema
`/102`, provider and provider-object schemas `/94`, artifact-root schema `/98`,
boot-adapter schema `/62`, and gate schema `/104` identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_DUP_OK` is reachable only after the complete
bounded duplication sequence returns exact Linux results and terminal dispatch
finds only the restored output and original error mappings before clearing the
table.

Two independent pinned-QEMU runs reached all 80 ordered observations. Hashes
for the recorded run are:

- linked kernel: `ebb4c10da59889b3b9c87e42f172fd208f81609b7f2d99ea8cecf1b11bf50639`;
- boot image: `77236cbd4c1d19756ee1e899dd054782f3abfcbab869403e31cff200c0adfa66`;
- boot resource: `142c375317f07f17d055a66763f83d706b1b149ee940ba63bceeb3fe5500ab23`;
- generated image: `2e32051555438b7e5a01b26324ea9b3f77f8cbdfee62b8110b78463bfb9ef65a`;
- supplied image: `97b2b94994e277e04457570691b78a6e94e701fd936c89999ced2b6b81b8e95a`;
- artifact provenance: `1dffa011af33cad8a7531d5e24d3b2ce5c9be0a37cf7f4b94ba2fea481bf59f6`;
- boot provenance: `0c11cba0d6f82d8d09e288cd13a89c9cbd0d2896d9c37f12ddf32a3d12034c16`;
- serial output: `4d291b223d643daad1f5216dbcb1b301f0078feb7e2fdc04ec9bc52e33c0b841`;
- evidence: `beb37251b57429b239064c9910450cda3863228d6d7dbdaba8b93e3b0db71fd5`.

## Risk and remaining boundary

Risk is high because descriptor integers now alias access-mode-bearing object
identities and `dup2` must not destroy a destination before source validation.
Review covered exact Linux error precedence, lowest-free selection, alias
lifetime, access-mode preservation, validation-before-replacement machine-code
ordering, terminal identities, architecture separation, structural inspection,
and duplicate physical execution.

The profile remains one process and one thread with three fixed console
descriptions and exactly three slots. It does not provide `dup3`, `fcntl`,
close-on-exec, larger descriptor tables, dynamically opened descriptions,
paths or VFS objects, general reference counts, pipes, sockets, polling,
fork/exec inheritance, concurrent lookup/close, signals, cancellation, or
device-specific final release.
