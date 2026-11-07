# Bounded small-request process `getrandom`

This record resolves `TK-DEC-115` as the first reusable `getrandom` request
that spans several qualified CRNG outputs. It introduces a service refinement
rather than hiding a provider loop inside the one-output contract.

## Decision

Before initial user entry, portable source shall consume the live full-output
service through
`topal.systems.entropy.user-random.expand-small-requests/1`. The request names
the same process, Linux interface, user-random service, generator algorithm,
and 32-byte output extent; a maximum of eight outputs and 256 delivered bytes;
and complete-request policy. It returns one affine process-lifetime service and
does not extract output during refinement.

Zero length returns zero without state change. Unknown flags, the incompatible
`GRND_RANDOM | GRND_INSECURE` combination, and lengths above 256 return
`-EINVAL` without state change. An invalid complete writable range returns
`-EFAULT` before extraction. Current valid flags do not alter delivery after
readiness.

For a valid request, the provider partitions the exact length into consecutive
chunks of at most 32 bytes. Each chunk uses the reserved first output when it is
still live, otherwise it performs a fresh qualified ChaCha20 transition. The
persistent key is replaced before each fresh output is published. The provider
copies and verifies the chunk at its exact user offset, then wipes and verifies
the complete 32-byte output before beginning the next chunk. No suffix is
retained.

The qualified single-thread, signal-free profile makes extraction and checked
copy total after complete-range validation. A successful small request therefore
returns its exact full length and increments the interface delivery count once,
not once per output. Any violated internal extraction, copy-verification, wipe,
or accounting invariant fails terminally; it shall not return a fabricated
partial success. Process-exit destruction remains unchanged.

The ring-3 qualification shall preserve all prior probes and successes, then
issue one 256-byte request. It shall verify return value 256 and read the first
and last qwords without constraining secret-derived values. Structural evidence
shall prove eight output iterations, per-output wipe, one delivery increment,
and no extraction before complete-range validation.

## Architecture pressure test

Service identity, maximum output count, output extent, full-range validation,
ordered chunk delivery, per-output key replacement and erasure, complete
success, and interface-level accounting are portable. x86-64 selects syscall
318, its entry registers, GS-relative private state, and page-fault recovery.
AArch64 and RISC-V select their own Linux syscall and exception mechanisms.
Portable source observes no loop register, pointer arithmetic, cipher state,
instruction, or partial output representation.

The eight-output limit describes the selected Linux small-request profile, not
an architecture word size. A later large-request refinement may add partial
progress and signal interruption without weakening this initialized small-
request contract.

## Alternatives rejected

Changing the existing one-output binding to imply an unbounded loop was
rejected because it would erase an observable service-policy boundary.
Pre-extracting eight outputs was rejected because it extends secret lifetime
and advances state for calls that may never occur. Counting every chunk as an
interface delivery was rejected because Linux observes one syscall result.
Returning a short result in the current signal-free profile was rejected as
false small-request behavior. A source-visible byte buffer or ChaCha loop
remains outside the systems abstraction.

## Deliberate bounds

This increment supports one process, one thread, initialized sequential calls,
complete requests through 256 bytes, and at most eight outputs per call. It does
not provide larger requests, partial progress, pre-readiness blocking, signal
interruption, concurrent callers, fork or exec inheritance, `/dev/random`,
`/dev/urandom`, vDSO delivery, reseeding, per-processor generators, batching,
queue recycling, asynchronous interrupt delivery, reset, or recovery.
