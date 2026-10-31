# Bounded static-process shell `pwd` command

Status: approved systems-feature design (`TK-DEC-221`).

## Purpose

The refillable shell proves two command publications but only one
output-producing built-in. This increment inserts exact `pwd\n` between
`echo topal\n` and `exit\n`, proving a second parsed command, a current-directory
observation at command execution, application-owned output, and a third affine
input publication without granting a general command stream.

## Decision

The bounded static-process entry may admit exactly three ordered command
publications through one checked process buffer: `echo topal\n`, `pwd\n`, and
`exit\n`. Each refill shall replace the consumed prior publication, zero and
validate its sealed tail, retain no user pointer, and advance an exclusive
private owner state. The clean profile shall publish exact `/ # ` prompts
before every command; the pinned external BusyBox profile shall require those
optional prompt observations absent.

After the `pwd` publication, the clean profile shall issue one qualified
current-directory request and receive exact sealed root plus terminator. The
pinned external BusyBox profile shall instead revalidate descriptor one with
one complete console-metadata query, observe one exact terminal-query miss
without candidate mutation, and use its already sealed root-directory fact.
Each application shall then publish exact `/\n` to standard output. The
provider shall validate the complete profile-specific requests, checked
candidates, sealed-root authority, readable output candidate, exact bytes,
results, ordering, and private observation lifecycle before admitting the
final prompt or exit refill.

The clean fixture may prove its prior command bytes unchanged before each
replacement. The external application may parse or mutate process-owned input
and current-directory output; its later transitions therefore rely on affine
owner and observation states rather than treating prior user bytes as provider
state. Terminal handling shall require the third publication live, erase the
complete command buffer, consume its owner, and clear every command-time fact.

The process stack shall be one affine, two-base-page extent. Both pages shall
be reserved, zeroed, published writable and non-executable, verified, and
released together. The Linux initial-stack value remains at the top of the
upper page; the lower page is bounded working stack, not an independently
owned mapping. This is the minimum static extent needed by the pinned shell to
parse the third publication and does not grant stack growth or demand paging.

## X86-64 binding

The x86-64 clean realization binds command-time current-directory observation
to native syscall 79, a complete checked anonymous output candidate at
`0x005f_9460`, a 4096-byte bound, exact `/\0`, and result two. The pinned
external realization binds its command-time revalidation to native syscall 5
on descriptor one with the exact 144-byte stack candidate at `0x007f_f370`,
then syscall 16 with `TCGETS` on descriptor one and the exact 36-byte stack
candidate at `0x007f_f2d0`, returning `-ENOTTY` without changing that candidate.
It may then format from its previously sealed root fact. The following output
binds to native syscall 1, descriptor one, exact `/\n`, count and result two;
the clean candidate is `0x005f_9460` and the pinned external candidate is
`0x005f_9570`. The three input results are 11, 4, and 5. Provider-private
phases and facts distinguish `pwd` input, profile-specific root authority and
revalidation, output, optional third prompt, final input, and teardown.
The stack extent occupies `[0x007f_e000, 0x0080_0000)`, with the initial ABI
value in its upper page and two adjacent high-table RW/NX leaves backed by one
two-frame affine allocation.

## Architecture pressure test

Command identity, affine refill ownership, sealed current-directory meaning,
complete metadata observation, terminal absence, checked user transfer, exact
output, and observation ordering are portable Linux-personality policy. A
digest-bound application may reuse a provider-sealed root fact only after its
own qualified command-time interface sequence revalidates the applicable
output channel. X86-64, AArch64, and RISC-V may select different syscall
encodings, metadata layouts, terminal probes, user-copy mechanisms, console
transports, and private layouts. No selector, register, address, or instruction
becomes Topal source vocabulary.

## Alternatives rejected

Provider-owned `/\n` output was rejected because it bypasses the application
boundary. Unconditional reuse of startup current-directory bytes was rejected
because it loses observable ordering and authority; the pinned external
profile may use a digest-bound sealed root fact only after exact console
metadata and terminal-absence revalidation. Requiring every application to
issue the clean fixture's syscall sequence was rejected because it would cease
to describe the observed pinned application ABI. Arbitrary command bytes, host
current-directory state, a general VFS, and an unbounded shell loop were
rejected as broader later work.

## Deliberate bounds

This increment proves one exact `pwd` built-in in the sealed root environment
and one fixed two-page initial stack. It does not add directory enumeration,
pathname lookup, mutable directories, arbitrary commands, stack growth,
demand paging, dynamic applications, multiple processes, containers, or
virtualization.
