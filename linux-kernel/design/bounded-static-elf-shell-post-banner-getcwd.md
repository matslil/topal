# Bounded static-process shell post-banner current directory

This record resolves `TK-DEC-213` as the current-directory observation that
follows the bounded shell's post-banner terminal miss. It reuses the sealed
root launch fact without treating the earlier startup query as a substitute
for this separately observable operation.

## Decision

The bounded static-process entry may admit exactly one current-directory query
after the post-banner terminal fact and before later identity, profile, prompt,
or sealed-input work. The operation shall require the live process, sealed
executable continuation, active checked address-space hierarchy, retained
anonymous heap, live child and ignored-quit dispositions, exact shell-input
predecessor phase, and every prior interactive observation. It shall publish
exactly the sealed root pathname with its terminator into one complete checked
writable candidate, return its exact length, retain no pointer, and leave
process phase unchanged.

One provider-private fact shall publish only after complete validation and
shall admit the query at most once. Later startup work, sealed input, and
terminal rollback shall require that fact. The clean fixture shall prefill the
complete published extent, issue the query, validate the result and exact root
value, and erase and prove erasure of the output. The pinned external echo-
command profile shall omit this optional interactive observation.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 79,
`getcwd(0x5f9460, 4096)`, publication of exactly `"/\0"`, and result two. The
candidate lies in the retained checked anonymous heap. The private completion
fact occupies the final byte of the existing interactive-observation qword.
The clean observation ends with all eight bytes populated; the external echo
state remains the all-zero qword.

## Architecture pressure test

The sealed-directory value, complete-publication effect, result length,
no-retention effect, and observation lifecycle are portable Linux-personality
policy. X86-64, AArch64, and RISC-V may select different syscall encodings,
calling conventions, candidate locations, and address-space validators. No
selector, register, pointer, page-table detail, or instruction becomes Topal
source vocabulary.

## Alternatives rejected

Reusing the earlier root result without a call was rejected because it would
lose observable ordering. Deriving a host directory or adding mutable current-
directory state was rejected because the launch remains sealed at root.
Returning success without writing the terminator or retaining the candidate
was rejected because either violates the Linux interface or weakens isolation.
Adding traversal or a general filesystem was rejected as unnecessary scope.

## Deliberate bounds

This increment admits one exact clean-fixture post-banner current-directory
publication and preserves the external echo-command profile. It does not add
directory mutation or traversal, host path inheritance, identity queries,
profile lookup, a prompt, mutable input, filesystems, processes, containers, or
virtualization.
