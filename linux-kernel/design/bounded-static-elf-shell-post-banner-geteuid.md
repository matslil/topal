# Bounded static-process shell post-banner effective identity

This record resolves `TK-DEC-214` as the effective-user observation that
follows the bounded shell's post-banner current-directory query. It reuses the
sealed initial credential group while preserving the second query as a
separately ordered operation.

## Decision

The bounded static-process entry may admit exactly one effective-user query
after the post-banner current-directory fact and before profile, prompt, or
sealed-input work. The operation shall require the live process, sealed
executable continuation, active checked address-space hierarchy, retained
initial stack, live child and ignored-quit dispositions, exact shell-input
predecessor phase, every prior interactive observation, and the complete
coherent real/effective user/group credential facts. It shall return the sealed
effective user identity, retain no argument or mutable credential state, and
leave process phase unchanged.

One provider-private fact shall publish only after complete validation and
shall admit the query at most once. Later startup work, sealed input, and
terminal rollback shall require that fact. The clean fixture shall issue the
query and validate its exact result. The pinned external echo-command profile
shall omit this optional interactive observation.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 107 and result
zero. It revalidates the retained initial-stack leaf and the complete sealed
credential group before returning. The private completion fact occupies a
fresh byte after the erased banner-staging extent; the existing complete
interactive-observation qword remains unchanged.

## Architecture pressure test

The effective-user role, coherence with the sealed credential group, pointer-
free result, no-retention effect, and observation lifecycle are portable Linux-
personality policy. X86-64, AArch64, and RISC-V may select different syscall
encodings, calling conventions, stack validators, and private state layouts.
No selector, register, pointer, auxiliary-vector layout, or instruction becomes
Topal source vocabulary.

## Alternatives rejected

Reusing the earlier real-user result without a call was rejected because real
and effective identities are distinct interfaces and the call order is
observable. Returning a host identity or adding mutable credentials was
rejected because it would violate the sealed process model. Returning zero
without revalidating the complete credential group was rejected because it
would weaken coherence. Adding a general credential service was rejected as
unnecessary scope.

## Deliberate bounds

This increment admits one exact clean-fixture post-banner effective-user
observation and preserves the external echo-command profile. It does not add
credential mutation, namespaces, profile lookup, a prompt, mutable input,
filesystems, processes, containers, or virtualization.
