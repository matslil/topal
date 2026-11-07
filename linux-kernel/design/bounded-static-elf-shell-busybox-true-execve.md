# Bounded static-process shell BusyBox `true` image replacement

Status: approved systems-feature design (`TK-DEC-238`).

## Problem

The qualified static BusyBox shell can inspect and reopen its sealed executable,
but it cannot launch an independent application. Standard userspace, dynamic
loading, containers, and VM-host tooling all require successful process-image
replacement. Substituting kernel-owned output or treating a shell builtin as a
new application would not cross that boundary.

## Decision

The bounded static-process profile shall retain the complete
`TOPAL-SYSTEMS-STATIC-ELF-SHELL-BIN-BUSYBOX-REOPEN-001` transaction through
the reopened description, `pwd` output, and following profile-specific prompt.
It shall replace the terminal `exit\n` publication with exact
`exec /bin/busybox true\n`. The application shall then issue successful
`execve` for the artifact-bound `/bin/busybox` object with exact argument vector
`["/bin/busybox", "true"]` and its complete inherited environment containing
exact `SHLVL=1`, `OLDPWD=/`, `PATH=/sbin:/usr/sbin:/bin:/usr/bin`, and
`PWD=/` entries. The prior root `cd` command accounts for `OLDPWD=/`.

Successful replacement shall consume the old user-image generation without
returning to its syscall continuation. It shall preserve PID one, credentials,
the sealed root current directory, standard descriptions, and the reopened
non-close-on-exec descriptor-three mapping. It shall reset caught signal
dispositions, thread-pointer state, clear-child-TID and robust-list
registrations, random-delivery and command-owner state, the old stack, heap,
user mappings, and every writable image byte that is not preserved by Linux
`execve` semantics.

The provider shall instantiate a fresh generation of the same digest-bound ELF
artifact: validate its already sealed load plan, restore every loadable file
byte, zero every BSS and anonymous byte, restore initial mapping permissions,
construct a fresh Linux x86-64 initial stack for the exact arguments,
environment, auxiliary vector, credentials, and newly delivered `AT_RANDOM`,
and enter the artifact entry point. The new `true` application shall execute
its ordinary qualified startup sequence and terminate with `exit_group(0)`.
Terminal handling shall revoke the inherited reopened description exactly once
and return every user-image, stack, heap, random, and process resource.

## Portable abstraction

The source-level operation is an affine process-image replacement. It consumes
one live process-image owner, a checked executable object, checked argument and
environment values, and replacement resources; success produces no ordinary
return and transfers the unchanged process identity into a fresh image owner.
Failure would return the original owner unchanged, but this first increment
qualifies only success. ELF parsing, native stack layout, page tables, register
files, syscall numbers, and entry instructions remain provider details.

The abstraction is suitable for x86-64, AArch64, and RISC-V: each target may
use different syscall calling conventions, initial-stack alignment, auxiliary
facts, translation maintenance, and exception return mechanics while
preserving the same atomic owner transition and Linux-visible image state.

## Ordering and failure containment

All pathname, vector, string, extent, artifact-identity, mapping, and prior-state
checks shall complete before old-image authority is consumed. No user pointer
may be retained. Once destructive replacement begins, provider failure is
terminal; it shall never return a partially replaced image to userspace.
Close-on-exec processing precedes new entry, while the qualified reopened
descriptor three remains live because it lacks that flag.

## Alternatives rejected

- Executing shell builtin `true` would not exercise image replacement.
- Fabricating a successful return would violate the nonreturning `execve` ABI.
- Reusing mutated writable pages, heap, stack, TLS, random bytes, or caught
  signal state would leak the old image into the new one.
- Starting a second PID would introduce process creation and waiting rather
  than the required same-process replacement.
- Exposing native registers, page-table operations, or assembly in Topal source
  would select the wrong abstraction level and impede other architectures.

## Deliberate bounds

This increment proves one successful replacement by the same immutable static
artifact, one exact argument/environment corpus, inherited descriptor state,
fresh startup, and terminal cleanup. It does not add arbitrary executables,
dynamic interpreters, scripts, general argv/environment limits, process
creation, waiting, signals during replacement, containers, or virtualization.
