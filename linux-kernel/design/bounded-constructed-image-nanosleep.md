# Bounded constructed-image `nanosleep` input

This record resolves `TK-DEC-052` as a systems-profile refinement. It requires
the provider-sealed continuation in the finalized code-role mapping to perform
one input-bearing Linux-compatible zero-duration `nanosleep` call using the
existing writable user stack, prove complete input transfer, and then continue
through the already-qualified constructed `getpid` and terminal exit. It
changes no core-language rule and does not expose a general pointer, syscall,
buffer, layout, or executable-image interface.

## Decision

The existing opaque entry-payload installation and verification operations
retain their architecture-neutral meaning. For this bounded profile, after the
constructed continuation completes its `uname` output qualification and
releases that temporary region, it must:

1. reserve a bounded input region in its already-owned writable user stack;
2. initialize one provider-sealed zero-duration native timespec value;
3. invoke the target's Linux-compatible `nanosleep` boundary with that exact
   input candidate and no remainder candidate;
4. require complete `copy_from_user`, valid zero-duration input, and successful
   return;
5. release the temporary stack region without retaining a pointer;
6. perform the existing constructed `getpid` and PID-one validation; and
7. terminate through `exit_group(0)`.

After the exact kernel continuation resumes, source publishes the entry, data,
constructed-copy-to, `TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK`, the constructed
syscall, and residency witnesses in that order. Both image-role owners remain
live through all observations and are subsequently revoked data first and code
second.

```text
constructed continuation -> uname output -> release output
  -> reserve and initialize bounded user-stack input
  -> nanosleep -> complete copy_from_user -> validate zero duration
  -> successful return -> release input -> getpid -> exit_group(0)
  -> exact kernel return -> copy-to -> copy-from -> syscall -> residency
```

Source names neither the syscall selector, user candidate, timespec layout,
values, registers, instructions, nor virtual addresses. The provider binds
those details and the exact allowed return sites. Structural evidence must
prove exact payload installation and verification, bounded stack reservation,
initialization and restoration, the sealed call site, successful return, the
subsequent identity call, terminal disposition, and fail-closed branches.

## Isolation and failure contract

The temporary input lies only in the existing RW/NX user-stack mapping. The
immutable data mapping remains RO/NX and the code mapping remains RO/X. The
constructed continuation cannot select another candidate, duration, remainder
candidate, syscall, or layout and cannot retain a candidate after stack
restoration.

A partial or faulted transfer, invalid or nonzero duration, nonzero syscall
result, unbalanced stack reservation, wrong return site or lifecycle state,
early or missing constructed-copy-from witness, return to the provider probe
after the constructed calls, mutation after verification, or teardown before
all witnesses is invalid. Fatal-only completion remains in force after
permission publication begins.

## Cross-architecture pressure

The portable contract is a bounded Linux zero-duration input from already-
owned writable user memory, followed by provider-sealed validation and
disposal. It does not prescribe a system-call instruction, register convention,
stack direction or alignment, native timespec word size, padding, or store
encoding.

An x86-64 provider may stage two zero 64-bit words on `rsp`, pass the candidate
in `rdi`, pass a null remainder candidate in `rsi`, and use `syscall`. AArch64
would use its Linux `x0`/`x1`/`x8` and `svc` conventions; RISC-V would use
`a0`/`a1`/`a7` and `ecall`. Capability architectures must derive a bounded
readable stack capability and must not widen or retain it. In every case source
observes only successful exact kernel return.

This increment does not admit arbitrary input candidates, durations, remainder
candidates, buffer sizes, layouts, syscall selection or arguments, source-
visible syscall results, general call gates, arbitrary program payloads,
relocations, ELF loading, arbitrary segments or aliases, demand paging,
mapping ownership transfer, or a general process model.
