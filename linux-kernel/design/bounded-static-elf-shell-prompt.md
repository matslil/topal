# Bounded static-process shell prompt

This record resolves `TK-DEC-219` as the first standard prompt publication for
the bounded static BusyBox shell. It follows explicit system-profile close and
precedes command input, preserving the application write boundary rather than
substituting provider-owned output.

## Decision

The bounded static-process entry may admit exactly one four-byte root prompt
write after the system-profile description has been consumed and before any
command input. The operation shall require the live process, sealed executable
continuation, active checked address-space hierarchy, retained executable
pathname and stack, live child and ignored-quit dispositions, exact shell-input
predecessor phase, every prior interactive observation, absent profile
description, live profile-closed history, and a fresh prompt-completion fact.

The application shall supply the exact `/ # ` segment through a completely
checked readable user candidate and request standard error. The provider shall
copy the complete segment into private staging, validate it before publication,
emit it in source order, erase and prove the staging empty, return four, retain
no pointer, and publish one private prompt-completion fact. A duplicate,
skipped-predecessor, reordered, wrong-description, wrong-length, aliased,
partially readable, or different-byte write shall fail closed. Later input and
terminal rollback shall require the prompt fact; rollback shall clear it with
the predecessor observations. The pinned external echo-command profile shall
omit this optional transition.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 1 with `fd=2`,
`count=4`, exact readable bytes `/ # `, and result four. Provider-private bytes
stage the segment and a fresh provider-private byte records prompt completion.

## Architecture pressure test

Prompt identity, application-supplied bytes, checked-copy isolation, ordered
publication, exact result, staging erasure, and completion lifecycle are
portable Linux-personality policy. X86-64, AArch64, and RISC-V may select
different syscall encodings, calling conventions, checked-copy mechanisms,
console transports, and private layouts. No selector, register, integer
representation, UART port, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Provider-owned prompt substitution was rejected because it would bypass the
application-visible write. Folding the prompt into the banner was rejected
because it loses a distinct syscall and ordering boundary. Publishing directly
from unchecked user memory was rejected because it weakens isolation. Adding
cursor-position queries or general terminal editing was rejected as separate
behavior. A refillable stream was rejected as the next lifecycle increment.

## Deliberate bounds

This increment admits one exact root prompt and preserves the pinned external
echo-command profile. It does not add terminal cursor queries, prompt editing,
refillable input, arbitrary output, filesystems, processes, containers, or
virtualization.
