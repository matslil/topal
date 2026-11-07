# Step 4 bounded constructed-image `nanosleep` input

## Outcome

The finalized constructed code mapping now exercises a pointer-bearing Linux
userspace input operation directly. After its qualified `uname` output path,
the sealed continuation reserves 16 bytes in the existing RW/NX user stack,
initializes one zero-duration native timespec, invokes `nanosleep` without a
remainder candidate, requires complete input transfer and successful return,
and restores the stack. It then performs the already-qualified constructed
`getpid`, requires PID one, and terminates through `exit_group(0)`.

After the exact kernel continuation resumes, source publishes the entry, data,
constructed-copy-to, constructed-copy-from, constructed-syscall, and residency
witnesses before data-first teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither instructions, registers, syscall selectors, addresses,
native layouts, input values, nor a general executable-image or system-call
interface to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK` after the constructed output witness and
before the constructed-syscall witness. They reject a missing, duplicate,
renamed, or late witness and reject teardown before all post-return
observations.

Provider and provider-object revisions `/35` plan 63 typed operations. The
private x86-64 lowering installs and verifies every qword of a 176-byte sealed
payload. Its continuation initializes two zero 64-bit words, admits
`nanosleep` only from its exact return site with candidate `0x601ff0` in the
existing stack mapping and a null remainder candidate, requires a complete
16-byte native-timespec transfer, validates both words, requires successful
return, and balances the stack before the existing constructed identity and
terminal calls. Every mismatch fails closed. Structural tests inspect the
exact payload, initialization, stack balance, syscall return sites, candidate,
lifecycle state, input extent and validation, and terminal disposition.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/36` records 62
placements and a 183-transition trace. Root revision
`topal.systems-root-object.x86_64/36` has 1,289 typed relocations, including
1,103 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two final consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were
byte-identical and reached all 35 ordered markers. They observed the
constructed-copy-from witness between the constructed output and syscall
witnesses, retained the existing Linux compatibility and `-EFAULT` outcomes,
remained in the live fatal halt, and left serial output quiescent. An earlier
retry exited during the pre-existing interrupt qualification before reaching
any constructed-image behavior; the successful evidence was therefore
repeated consecutively rather than treating that attempt as feature evidence.
Gate schema `/38` records:

- linked kernel: `badea44a88afb8641758d7f271cc98c44bf2cf68b17c4afb7e0a82b7ce8f617e`;
- boot image: `0399e693a1bfd9fca155e56df48ed1c1bcee250dadc4f66edaa2f779c39989f6`;
- artifact provenance: `d58fc91d58dba500cd017c147845b135e7f6e7b81f427702c40fc7f6a9df96dd`;
- boot provenance: `5bc28247eb9f872bb1c92adf7850d76ba88ef4f147e20acf7744207dfe7e3655`;
- serial: `b58900b09ef0f8172a3aeef31f2b2f1b04b072a3e7fb06965d0d257cf0fd25a8`;
- evidence record: `6c4ab8b56989ae56d5bbd8efa16c370bb1e5d5faa1e53f4529efdedf54c19648`.

## Remaining boundary

The constructed mapping still contains only a provider-sealed bounded payload.
Arbitrary syscall selection or arguments, source-visible syscall results,
general input candidates, values or layouts, nonzero or interruptible waits,
general call gates, arbitrary program payloads, source-selected entry or data
references, relocations, ELF loading, arbitrary segments and aliases, demand
paging, mapping ownership transfer, and a general process model remain
excluded.
