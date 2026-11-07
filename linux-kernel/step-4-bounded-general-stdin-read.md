# Step 4 bounded general standard-input read

## Outcome

The initial-process personality now accepts caller-selected
`read(0, candidate, length)` requests for one through 256 bytes into a complete
provider-known writable user extent. The handler no longer requires one exact
return address, stack candidate, length, or input value. It rejects malformed
requests before input consumption, receives the exact bounded count into
private kernel staging, copies the complete staged prefix to user memory under
fault recovery, and returns the requested length.

Invalid descriptors, lengths, and candidates return Linux-compatible
`-EBADF`, `-EINVAL`, and `-EFAULT`, respectively. The harness now injects the
four-byte sequence `read`, and the supplied ELF validates the result and all
four bytes after return. The checked disposition and provider contain no copy
of that expected input. No core-language rule changes.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-READ-001` records the bounded descriptor, live-state,
executable-return, length, overflow, writable-mapping, exact-fill staging,
copy-to-user, and result requirements. Structural tests prove the 256-byte
cap, both known writable mapping ranges, dynamic receive and copy loops,
staging-before-publication order, argument-derived return, and distinct error
results.

Artifact schema `/56`, provider `/53`, provider object `/53`, artifact root
`/54`, boot adapter `/16`, and gate `/58` identify the changed contract. The
boot-resource schema remains `/1`. The artifact trace contains 198 semantic
events, 63 placements, and 1,819 relocation applications; the generated root
contains 1,633 source console-byte writes.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. Evidence binds the four input bytes
and their SHA-256 identity. The read witness follows complete user validation
and terminal return and precedes the other constructed-image witnesses. The
evidence records:

- linked kernel: `7318f2187527375465a7f9bbacac0c731472051e5330d5fb46b0ba1e3e12ec1b`;
- boot image: `9d5a424e50febff78e32339f9564911d2edf28bd1a4a9de957ed93d9c6dfffa4`;
- boot resource: `e2c14b9978d1af986e94ef9bf32a86f197b31537082658415e02495f8d9435d6`;
- generated default image: `4e62b58683fd63e1f6f9e7af8631761417dabdc84d31f457528fb08fdd0d5b19`;
- supplied image: `f31eba543c05e91c6fd6430152099ff4e473016b77dd5d539350c81accb25558`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `30aa83ab05cb93fa7c0f5184cf6252efa750fc20a62d9991632cd1cd896a0f17`;
- boot provenance: `da05934f4ef8627dde562720bf2ff7a2b172ff809669bfc1cf11b80461c860cd`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `9376eaec11e2f988eda41175958b48a25cbcd3cb46df952232648514a0290ce0`.

## Risk and review

Risk is high because a fixed external-input oracle becomes a reusable
pointer-bearing syscall path. Review covered arithmetic wrap, writable
half-open range containment, descriptor and size errors, rejection before
input consumption, bounded exact-fill polling, complete staging before user
publication, copy-fault recovery, dynamic loop termination, result and stack
restoration, pointer disposal, structural x86 checks, full workspace
validation, and two independent physical runs.

## Remaining boundary

This is still a serial-console-backed exact-fill initial-process adapter, not
general Linux file input. It has no descriptor table, end-of-file, short
reads, nonblocking mode, signals, restart behavior, terminal line discipline,
readiness polling, files, pipes, sockets, vectored I/O, asynchronous I/O, or
requests above 256 bytes. Those remain subsequent userspace-gate work.
