# Step 4 bounded constructed-image syscall

## Outcome

The finalized constructed code mapping now crosses the Linux userspace ABI
directly. Its sealed entry segment observes the immutable-data sentinel and
transfers to the unchanged provider-owned compatibility probe. After that
probe completes its existing `getpid`, `uname`, user-copy, `nanosleep`, and
fault-recovery checks, it transfers to a sealed continuation in the constructed
mapping. The continuation issues `getpid`, requires PID one, and terminates
through `exit_group(0)`. It never attempts to return to the provider probe
after the constructed syscall.

After the exact kernel continuation resumes, source publishes the entry, data,
constructed-syscall, and residency witnesses before data-first teardown.

This is a systems-feature refinement only. It changes no core-language rule
and exposes neither instructions, registers, syscall selectors, addresses,
nor a general executable-image or system-call interface to Topal source.

## Model and x86-64 lowering

The checked source and architecture-independent model require
`TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK` after the data witness and before the
residency witness. They reject a missing, duplicate, renamed, or late witness
and reject teardown before all post-return observations.

Provider and provider-object revisions `/33` still plan 63 typed operations.
The private x86-64 lowering installs and verifies a 56-byte payload containing
an entry segment and a terminal continuation. Exact call-site checks admit the
provider probe's original `getpid` return only in its original lifecycle state
and admit the constructed continuation's `getpid` return only after the probe
has completed. The continuation validates PID one and performs
`exit_group(0)`; all mismatches fail closed. Structural tests inspect every
eight-byte payload chunk, both sealed transfer sites, both system-call return
sites, return validation, and the trap path.

Artifact revision `topal.systems-artifact.x86_64-qemu-pc-q35/34` records 62
placements and a 181-transition trace. Root revision
`topal.systems-root-object.x86_64/34` has 1,219 typed relocations, including
1,033 UART writes. The boot adapter remains `/6`.

## Physical evidence

Two pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical and
reached all 33 ordered markers. They observed the constructed syscall witness
between the data and residency witnesses, retained the existing Linux
compatibility and `-EFAULT` outcomes, remained in the live fatal halt, and left
serial output quiescent. Gate schema `/36` records:

- linked kernel: `bfc2a27f1326fbe6e60571ae8103f448893150b3da72a95008d1cf7208b0a541`;
- boot image: `b2fef7fdb7c2d2f951a57a5e1d1d9137e47aa6bd3926aea6283cd89de59f13d1`;
- artifact provenance: `d7df76ac63acaecb8939d984fc5de546017655e49b0e6dd943c7acfc0f758d87`;
- boot provenance: `6b5a177c8687407e68d830fce83fe22d1f2b7515e52c838f177033f42788f212`;
- serial: `b273b209524139d890dbff8bace03cd986a27f01f358108beb4030ab9108cd41`;
- evidence record: `5de2db8becabe5187ebe13ba3248ec9fff804a0e5466efd965d5d01b854236c7`.

## Remaining boundary

The constructed mapping still contains only a provider-sealed bounded payload.
Arbitrary syscall selection or arguments, source-visible syscall results,
general call gates, arbitrary program payloads, source-selected entry or data
references, relocations, ELF loading, arbitrary segments and aliases, demand
paging, mapping ownership transfer, and a general process model remain
excluded.
