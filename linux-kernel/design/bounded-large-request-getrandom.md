# Bounded large-request process `getrandom`

This record resolves `TK-DEC-116` as the first reusable `getrandom` request
above Linux's initialized small-request threshold. It extends the explicit
service bound without implying an unbounded provider loop or fabricating
signal-interruption behavior that the current process profile cannot produce.

## Decision

Before initial user entry, portable source shall consume the live small-request
service through
`topal.systems.entropy.user-random.expand-large-requests/1`. The request names
the same process, Linux interface, user-random service, generator algorithm,
and 32-byte output extent; a maximum of 16 outputs and 512 delivered bytes;
and complete-without-interruption policy. It returns one affine process-lifetime
service and performs no extraction during refinement.

Zero length returns zero without state change. Unknown flags, the incompatible
`GRND_RANDOM | GRND_INSECURE` combination, and lengths above 512 return
`-EINVAL` without state change. An invalid complete writable range returns
`-EFAULT` before extraction. Current valid flags do not alter delivery after
readiness.

For a valid request, the provider partitions the exact length into consecutive
chunks of at most 32 bytes. Every chunk uses a fresh qualified ChaCha20
transition unless the reserved first output is still live. The persistent key
is replaced before each fresh output is published. The provider copies and
verifies each chunk at its exact user offset, then wipes and verifies the
complete 32-byte output before beginning the next chunk. No suffix is retained.

The qualified process has one thread, no deliverable signals, and no concurrent
caller. After whole-range validation, extraction and checked copying are total
or fail terminally on an internal invariant violation. A successful request
therefore returns its exact full length and increments the interface delivery
count once. This is a selected signal-free execution profile, not a claim that
Linux guarantees complete large requests under interruption.

The ring-3 qualification shall preserve all prior probes and successes, then
issue one 512-byte request in a known writable user range. It shall verify
return value 512 and read the first and last qwords without constraining
secret-derived values. Structural evidence shall prove the 16-output bound,
per-output wipe, one delivery increment, and no extraction before complete-range
validation.

## Architecture pressure test

Service identity, maximum output count, output extent, full-range validation,
ordered chunk delivery, per-output key replacement and erasure, signal-free
completion, and interface-level accounting are portable. The 512-byte bound
is a profile constant, not a portable page-size operation. x86-64 selects
syscall 318, its entry registers, GS-relative private state, and page-fault
recovery. AArch64 and RISC-V select their own Linux syscall and exception
mechanisms. Portable source observes no loop register, pointer arithmetic,
cipher state, instruction, page-table layout, or partial output representation.

## Alternatives rejected

An unbounded request was rejected because it would hide resource and progress
bounds. Adding a source-visible allocation was rejected because the provider
already owns bounded user-transfer staging and no output needs to coexist.
Inventing a partial return after an arbitrary output count was rejected because
the current profile has no signal or scheduling event that could justify it.
Treating 4,096 as the architecture page size was rejected because the same
service abstraction must remain meaningful on targets with other page sizes.

## Deliberate bounds

This increment supports one process, one thread, initialized sequential calls,
complete signal-free requests through 512 bytes, and at most 16 outputs per
call. It does not provide larger requests, partial progress, pre-readiness
blocking, signal interruption, concurrent callers, fork or exec inheritance,
`/dev/random`, `/dev/urandom`, vDSO delivery, reseeding, per-processor
generators, batching, queue recycling, asynchronous interrupt delivery, reset,
or recovery.
