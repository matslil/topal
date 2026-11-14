# Bounded constructed-image syscall

This record resolves `TK-DEC-050` as a systems-profile refinement. It requires
the provider-sealed entry payload in the finalized code-role mapping to perform
one Linux-compatible system call after observing finalized immutable data and
before transferring to the existing qualification probe. It changes no
core-language rule and does not expose a general instruction, ABI-call, or
executable-image interface.

## Decision

The existing entry-payload installation and verification operations retain
their architecture-neutral meaning: they install and verify the exact opaque
payload selected by the provider. For this bounded profile, that payload must:

1. begin execution in the finalized code-role mapping;
2. observe the provider-sealed sentinel in the finalized immutable-data
   mapping;
3. transfer to the unchanged provider-owned qualification probe;
4. accept a return from that probe only at a provider-sealed continuation in
   the same constructed code mapping;
5. invoke the target's Linux-compatible `getpid` system-call boundary from
   that constructed continuation and require the qualified initial-process
   result, PID one; and
6. terminate through the existing `exit_group(0)` boundary without attempting
   to return to the provider-owned probe mapping.

After the exact kernel return, source publishes the entry witness, the data
witness, `TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK`, and only then the existing
residency witness. Both role owners remain live through all four observations
and are subsequently revoked data first and code second.

```text
finalize code payload RO/X + sentinel data RO/NX
  -> image-ready witness -> enter at code mapping
  -> read data sentinel -> transfer to qualified probe
  -> sealed return to constructed continuation
  -> invoke Linux getpid -> require PID one -> exit_group(0)
  -> exact kernel return
  -> entry witness -> data witness -> image-syscall witness
  -> residency witness -> data-first teardown
```

Source names neither the Linux syscall selector nor registers, instructions,
addresses, return conventions, or the sentinel value. The provider binds all
of those details and any cache or instruction-stream synchronization.
Structural evidence must prove exact installation and verification, a
read-only data access, the two sealed cross-mapping transfer sites,
system-call entry from the constructed continuation, return validation,
terminal disposition, and a fail-closed path.

## Isolation and failure contract

The data mapping remains RO/NX throughout execution. The payload may use the
sealed syscall boundary only for this zero-argument identity observation. It
may not retain a pointer, mutate either image role, accept an arbitrary syscall
number or argument, expose a target result directly to source, or bypass the
existing typed kernel entry.

A wrong sentinel, unexpected PID, skipped system call or comparison, a probe
return outside the sealed continuation, return to the probe after the
constructed syscall, early or missing image-syscall witness, mutation after
verification, or teardown before the witness is invalid.
Fatal-only completion remains in force after permission publication begins.

## Cross-architecture pressure

The portable contract is one provider-selected Linux identity observation from
constructed code after a read-only image-data observation. It does not
prescribe an instruction family, register assignment, immediate, trap number,
return register, or address materialization.

An x86-64 provider may use sealed indirect transfers between the two code
placements, place the selector in `eax`, use `syscall`, validate `rax`, and
finish with `exit_group`. AArch64 would use its Linux `x8` selector and `svc`
convention; RISC-V would use `a7` and `ecall`.
Capability architectures must preserve sealed execute and read authority
across their kernel transition without deriving general integer pointers. In
all cases the provider supplies ABI evidence while source observes only
successful exact return.

This increment does not admit arbitrary syscall selection or arguments,
source-visible syscall results, general call gates, arbitrary program payloads,
relocations, ELF loading, arbitrary segments or aliases, demand paging,
mapping ownership transfer, or a general process model.
