# Bounded static-process shell profile close

This record resolves `TK-DEC-218` as the explicit lifetime transition for the
bounded `/etc/profile` description. It consumes descriptor three after the
immutable empty object's EOF observation so later prompt startup proceeds with
the lowest nonstandard descriptor free and no teardown-only substitute for the
application close.

## Decision

The bounded static-process entry may admit exactly one close of the live
profile descriptor after its complete EOF observation and before prompt or
sealed-command work. The operation shall require the live process, sealed
executable continuation, active checked address-space hierarchy, retained
executable pathname and stack, live child and ignored-quit dispositions, exact
shell-input predecessor phase, every prior interactive observation, and the
process-owned read-only description for the immutable empty profile at
position zero.

Success shall consume the description, free exactly descriptor three, return
zero, retain no argument or object state, and publish one private closed-state
fact. The EOF observation shall remain as immutable history. A duplicate,
skipped-predecessor, reordered, wrong-descriptor, already-closed, or aliased
close shall fail closed. Later startup work, sealed input, and terminal
rollback shall require the description absent and the closed-state fact live.
Terminal handling shall clear that fact with the predecessor observations.
The pinned external echo-command profile shall omit this optional transition.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 3 with `fd=3`
and result zero. A fresh provider-private byte records the closed state after
the EOF completion fact; the prior live-description byte is cleared and proved
zero before return.

## Architecture pressure test

Description consumption, lowest-slot release, no-alias lifetime, result,
immutable history, and closed-state lifecycle are portable Linux-personality
policy. X86-64, AArch64, and RISC-V may select different syscall encodings,
calling conventions, descriptor representations, and private state layouts.
No selector, register, integer representation, or instruction becomes Topal
source vocabulary.

## Alternatives rejected

Relying on process teardown was rejected because the application-visible close
would be skipped. Clearing the description without a distinct closed fact was
rejected because later work could not distinguish explicit close from missing
predecessor state. Retaining an inactive description was rejected because it
would permit accidental use after close. General descriptor tables, aliases,
and arbitrary close were rejected as unnecessary scope.

## Deliberate bounds

This increment admits one exact close of the bounded profile description and
preserves the pinned external echo-command profile. It does not add other
descriptors, reopen, profile contents, a prompt, refillable input, filesystems,
processes, containers, or virtualization.
