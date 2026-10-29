# Bounded static-process shell profile EOF read

This record resolves `TK-DEC-217` as the sole content observation for the live
bounded `/etc/profile` description. It returns end-of-file from the immutable
empty object so the shell can advance to descriptor close without claiming a
general file stream or mutable filesystem contents.

## Decision

The bounded static-process entry may admit exactly one read from the live
profile descriptor after its complete metadata observation and before profile
close, prompt, or sealed-command work. The operation shall require the live
process, sealed executable continuation, active checked address-space
hierarchy, retained executable pathname and stack, live child and ignored-quit
dispositions, exact shell-input predecessor phase, every prior interactive
observation, and the process-owned read-only description for the immutable
empty profile object.

The provider shall validate the complete writable candidate and requested
bound before observing the retained object. Because the object's sealed length
and current description position are both zero, it shall return zero without
reading or modifying any candidate byte, changing the description position,
or retaining a user pointer. One private completion fact shall publish only
after complete validation and shall reject a duplicate, skipped-predecessor,
reordered, wrong-description, differently bounded, or substituted-candidate
operation. Later profile close and terminal rollback shall require the fact.
Process teardown may clear it and revoke the still-open description; a later
explicit close shall consume the description first. The pinned external echo-
command profile shall omit this optional observation.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 0 with `fd=3`,
the clean fixture's exact continuation-stack candidate, count 128, and result
zero. The candidate lies completely in the checked retained stack leaf. The
clean fixture shall prefill all 128 bytes, issue the request, prove the complete
candidate unchanged, erase it, and prove erasure. A fresh provider-private byte
records completion after the profile-metadata fact.

## Architecture pressure test

Description identity, immutable empty-object EOF, complete candidate
validation, no-output semantics, nonretention, and completion lifecycle are
portable Linux-personality policy. X86-64, AArch64, and RISC-V may select
different syscall encodings, calling conventions, candidate locations, and
private state layouts. No selector, register, pointer, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

Skipping the read was rejected because it would bypass an observed application
boundary. Returning host profile bytes was rejected because they are unsealed
and non-reproducible. Returning EOF without validating the candidate was
rejected because invalid user ranges remain observable even for empty files.
Adding arbitrary file contents, offsets, buffering, or descriptor reads was
rejected as unnecessary scope.

## Deliberate bounds

This increment admits one exact EOF read from the already-open immutable empty
profile and preserves the pinned external echo-command profile. It does not add
profile contents, explicit close, other files, general reads, a prompt,
refillable input, filesystems, processes, containers, or virtualization.
