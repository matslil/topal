# Step 4 initial standard-descriptor close

## Outcome

The Topal kernel now retains a bounded live/closed state for the initial Linux
descriptors zero through two. Linux x86-64 `close` atomically clears a live
entry and returns zero; negative, out-of-range, and already closed descriptors
return `-EBADF` without changing state.

The existing `read` and `write` paths consult that same state before user-
memory or external-I/O effects. The ring-3 probe closes standard input after
its final successful read, observes `-EBADF` from a later read without another
input byte, and observes `-EBADF` from a repeated close. Terminal teardown
requires only standard output and error to remain live, clears them, and
verifies the complete descriptor state is empty.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-CLOSE-001` refines the approved systems profile; it
adds no core-language element. Language, semantic, provider-planning,
artifact, and structural tests cover the three-entry bound, Linux error nine,
x86-64 syscall three, availability checks before effects, exact success and
failure results, shared state, ordered ring-3 calls, and terminal clearing.
The architecture pressure test retains the same portable meaning while
identifying native syscall 57 for later AArch64 and RISC-V providers.

The checked root contains 144 fatal edges, an exact 313-event semantic trace,
2,987 relocations, and 2,603 UART calls. The sealed provider plan contains 84
operations and retains the 160-byte entropy-storage layout. Artifact schema
`/101`, provider and provider-object schemas `/93`, artifact-root schema `/98`,
boot-adapter schema `/61`, and gate schema `/103` identify this contract.

## Physical evidence

The gate retains the pinned `pc-q35-10.2` machine, `qemu64-v1` CPU, SeaBIOS,
ISA serial console, and one `virtio-rng-pci-non-transitional` function at
`00:05.0`. `TOPAL_KERNEL_USER_CLOSE_OK` is reachable only after `close(0)` has
returned zero, a later `read(0, ..., 1)` has returned `-EBADF` without consuming
input, and a repeated `close(0)` has returned `-EBADF`.

Two independent pinned-QEMU runs reached all 79 ordered observations. Hashes
for the recorded run are:

- linked kernel: `ecf64fff621e34af1730920078641192a2dbbea485427fd4058c3a45a3647771`;
- boot image: `53ab4a2c16ace54b2ff4a314865e7648da5240d4bd90e6f9a79a778baac6b9bf`;
- boot resource: `6ef42698e82b94d05e4c7ad4e267c7208bfef3dfe2adfe89c98a1326a74e81ee`;
- generated image: `039c17c5a1eb68fbccc9db93b8145acfbf7f8ec14cc15b4261bef939bcfa12de`;
- supplied image: `8824dd365beb4b0d5c479bdb56839090b05706f887e342da6df06b5f2fa4ba85`;
- artifact provenance: `6f3bb7437bdc1e97674779f1c9cbe66caea9277dfcaabdeec14c9a1cb7ce705e`;
- boot provenance: `8f9c6b6d2ea334caf0925313c819bee6f125ef8b178df67c3670c13172beb404`;
- serial output: `2b533ef6f81fe844b954ed153a4ce8b53847ede445794add189371681ca38a01`;
- evidence: `2db8084289c63bfc1fb9fbc8f23030e0026bbe5dff51302a64289de90459e981`.

## Risk and remaining boundary

Risk is high because a new retained descriptor state now gates existing input
and output paths and must be cleared during process teardown. Review covered
effect ordering, exact error polarity, repeated-close behavior, shared-state
lookup, terminal-state invariants, architecture separation, structural
inspection, and duplicate physical execution.

The profile remains one process and one thread with exactly three inherited
standard descriptors. It does not yet provide descriptor allocation, `open`,
`openat`, `dup`, `fcntl`, open-file descriptions, VFS objects, filesystem
paths, pipes, sockets, polling, close-on-exec, inheritance, signals,
concurrent close, cancellation, device release, or deferred writeback errors.
