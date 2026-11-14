# Step 4 bounded constructed-image `uname` output

## Outcome

The finalized constructed code mapping now exercises a pointer-bearing Linux
userspace ABI operation directly. After the entry observes immutable data and
the existing compatibility probe returns, the sealed continuation reserves a
bounded buffer in the existing RW/NX user stack, invokes `uname`, validates a
complete return and the returned `Linux` identity, and restores the stack. It
then performs the already-qualified constructed `getpid`, requires PID one,
and terminates through `exit_group(0)`.

After the exact kernel continuation resumes, source publishes the entry, data,
constructed-copy, constructed-syscall, and residency witnesses before
data-first teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither instructions, registers, syscall selectors, addresses,
buffer layouts, returned bytes, nor a general executable-image or system-call
interface to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK` after the data witness and before the
constructed-syscall witness. They reject a missing, duplicate, renamed, or
late witness and reject teardown before all post-return observations.

Provider and provider-object revisions `/34` plan 63 typed operations. The
private x86-64 lowering installs and verifies every qword of a 128-byte sealed
payload. Its continuation reserves and restores 512 stack bytes, admits
`uname` only from its exact return site with candidate `0x601e00` in the
existing stack mapping, requires a complete 390-byte `new_utsname` transfer,
and validates `Linux` before the existing constructed identity and terminal
calls. Every mismatch fails closed. Structural tests inspect the exact
payload, stack balance, syscall return sites, candidate, returned bytes,
lifecycle state, and terminal disposition.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/35` records 62
placements and a 182-transition trace. Root revision
`topal.systems-root-object.x86_64/35` has 1,253 typed relocations, including
1,067 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical and
reached all 34 ordered markers. They observed the constructed-copy witness
between the data and constructed-syscall witnesses, retained the existing
Linux compatibility and `-EFAULT` outcomes, remained in the live fatal halt,
and left serial output quiescent. Gate schema `/37` records:

- linked kernel: `b625c56b745dbbe89df88725e054e390d9ea5ee5cf2cc7aa7e3dcab577e2e174`;
- boot image: `4391931612aab390c64bf1dea577b4f6bd0547d8ff02a444f321c3010cc93f9d`;
- artifact provenance: `945c1c56a71435ae1c56f84d496692ba09a4b83c1b7e53ce69f1729b80c9e225`;
- boot provenance: `ecf0f35ee47391281ed665423a0ea55814fb68044d9d4e37622d770986d61149`;
- serial: `ac5898035ee483f75aee1c00a6c8ac83ba8f7e18020e96f7de9cfa1f82f0d71e`;
- evidence record: `0145549da92dbb940c957844a0029c42c5d243fb4d5ba0db30d870f87598e79c`.

## Remaining boundary

The constructed mapping still contains only a provider-sealed bounded payload.
Arbitrary syscall selection or arguments, source-visible syscall results,
general output candidates or layouts, general call gates, arbitrary program
payloads, source-selected entry or data references, relocations, ELF loading,
arbitrary segments and aliases, demand paging, mapping ownership transfer, and
a general process model remain excluded.
