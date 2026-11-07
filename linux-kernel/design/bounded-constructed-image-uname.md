# Bounded constructed-image `uname` output

This record resolves `TK-DEC-051` as a systems-profile refinement. It requires
the provider-sealed continuation in the finalized code-role mapping to perform
one pointer-bearing Linux-compatible `uname` call using the existing writable
user stack, validate the returned identity, and then continue through the
already-qualified constructed `getpid` and terminal exit. It changes no
core-language rule and does not expose a general pointer, syscall, buffer, or
executable-image interface.

## Decision

The existing opaque entry-payload installation and verification operations
retain their architecture-neutral meaning. For this bounded profile, after the
entry segment observes immutable image data and the provider-owned
compatibility probe returns to the sealed constructed continuation, that
continuation must:

1. reserve a bounded output region in its already-owned writable user stack;
2. invoke the target's Linux-compatible `uname` boundary with that exact
   output candidate;
3. require complete success and validate the returned `sysname` as `Linux`;
4. release the temporary stack region without retaining a pointer;
5. perform the existing constructed `getpid` and PID-one validation; and
6. terminate through `exit_group(0)`.

After the exact kernel continuation resumes, source publishes the entry and
data witnesses, `TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK`, the constructed-syscall
witness, and only then the existing residency witness. Both image-role owners
remain live through all observations and are subsequently revoked data first
and code second.

```text
constructed entry -> observe immutable data -> compatibility probe
  -> sealed constructed continuation -> reserve bounded user-stack output
  -> uname -> complete copy_to_user -> validate "Linux" -> release output
  -> getpid -> require PID one -> exit_group(0) -> exact kernel return
  -> entry -> data -> constructed-copy -> constructed-syscall -> residency
```

Source names neither the syscall selector, user candidate, buffer size,
layout, returned bytes, registers, instructions, nor virtual addresses. The
provider binds those details and the exact allowed return sites. Structural
evidence must prove exact payload installation and verification, bounded stack
reservation and restoration, the sealed `uname` call site, returned-byte
validation, the subsequent identity call, terminal disposition, and
fail-closed branches.

## Isolation and failure contract

The temporary output lies only in the existing RW/NX user-stack mapping. The
immutable data mapping remains RO/NX and the code mapping remains RO/X. The
constructed continuation cannot select another candidate, length, syscall, or
layout and cannot retain a candidate after stack restoration.

A partial or faulted transfer, nonzero `uname` result, wrong returned identity,
unbalanced stack reservation, wrong return site or lifecycle state, early or
missing constructed-copy witness, return to the provider probe after the
constructed calls, mutation after verification, or teardown before all
witnesses is invalid. Fatal-only completion remains in force after permission
publication begins.

## Cross-architecture pressure

The portable contract is a bounded Linux identity output into already-owned
writable user memory, followed by provider-sealed validation and disposal. It
does not prescribe a system-call instruction, register convention, stack
direction, alignment, structure padding, or byte-load encoding.

An x86-64 provider may reserve the selected `new_utsname` extent on `rsp`, pass
it in `rdi`, use `syscall`, and validate bytes before restoring `rsp`. AArch64
would use its Linux `x0`/`x8` and `svc` conventions; RISC-V would use `a0`/`a7`
and `ecall`. Capability architectures must derive a bounded writable stack
capability and must not widen or retain it. In every case source observes only
successful exact kernel return.

This increment does not admit arbitrary output candidates, buffer sizes,
layouts, syscall selection or arguments, source-visible syscall results,
general call gates, arbitrary program payloads, relocations, ELF loading,
arbitrary segments or aliases, demand paging, mapping ownership transfer, or a
general process model.
