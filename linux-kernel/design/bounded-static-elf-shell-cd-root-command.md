# Bounded static-process shell root `cd` command

Status: approved systems-feature design (`TK-DEC-222`).

## Purpose

The bounded shell can observe its launch directory but cannot change process
filesystem state. This increment inserts exact `cd /\n` before `pwd\n`, proving
application-issued pathname input, an idempotent current-directory transition,
a fourth affine command publication, and observation of the resulting state
without granting arbitrary lookup or a general VFS.

## Decision

The bounded static-process entry may admit exactly four ordered command
publications through one checked process buffer: `echo topal\n`, `cd /\n`,
`pwd\n`, and `exit\n`. Each refill shall replace a consumed prior publication,
zero and validate its sealed tail, retain no user pointer, and advance one
exclusive private input-owner state. The clean profile shall publish exact
`/ # ` prompts before every command; the pinned external BusyBox profile shall
require those optional prompt observations absent.

After the `cd /` publication, both applications shall issue one qualified
current-directory change with a complete readable candidate containing exact
root plus terminator. The provider shall validate the live process, executable
continuation, active hierarchy, two-page stack, child disposition, exact
profile predecessor, input owner, complete candidate extent, and exact bytes
before replacing the sealed current-directory state with a new sealed-root
generation. The result shall be zero. No user pointer or pathname bytes shall
be retained as provider state.

The following `pwd` command shall observe the new generation. The clean profile
shall make a fresh sealed-root query. Pinned BusyBox shall revalidate stdout
metadata and terminal absence before formatting from the new sealed-root fact.
Both shall retain the exact application-owned `/\n` result and profile-specific
prompt behavior approved by `TK-DEC-221`. Terminal handling shall require the
fourth publication live, erase the complete command buffer, consume its owner,
and clear every command-time and current-directory-generation fact.

## X86-64 binding

The x86-64 realization binds the change to native syscall 80 with an exact
two-byte `/\0` candidate wholly inside a checked readable process mapping and
result zero. The four input results are 11, 5, 4, and 5. The clean native `pwd`
binding remains exact. Because the additional BusyBox built-in may move its
formatting buffer, the pinned external output binding supersedes the prior
single-address constraint: the complete two-byte candidate shall lie inside
the already sealed readable heap mapping `[0x005f_7000, 0x0061_9000)`, contain
exact `/\n`, and retain count and result two. Provider-private phases and facts
distinguish `cd` input, sealed-root replacement, optional third prompt, `pwd`
input and observation, output, optional fourth prompt, final input, and
teardown. The fixed affine stack remains `[0x007f_e000, 0x0080_0000)` with both
adjacent leaves RW/NX.

## Architecture pressure test

Exact-root pathname meaning, checked readable transfer, current-directory
state replacement, affine command ownership, observation ordering, and
terminal erasure are portable Linux-personality policy. X86-64, AArch64, and
RISC-V may select different syscall encodings, user-copy mechanisms, mapping
layouts, and provider-private state. No syscall number, register, address,
page-table entry, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Provider-owned directory mutation was rejected because it bypasses the
application syscall. Accepting the command without observing a change request
was rejected because it fabricates behavior. A fixed pointer was rejected in
favor of a complete checked candidate because the two qualified applications
own different storage. Host current-directory inheritance, relative paths,
lookup of non-root components, `chroot`, mounts, and a general VFS were
rejected as broader later work.

## Deliberate bounds

This increment proves one idempotent exact-root `cd` followed by `pwd` in one
process. It does not add arbitrary pathnames, directory traversal, file lookup,
mutable filesystem objects, stack growth, dynamic applications, multiple
processes, containers, or virtualization.
