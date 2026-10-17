# Reusable process `getrandom` service

This record resolves `TK-DEC-113` as the transition from the first qualified
one-shot delivery to a process-lifetime random service. It keeps generator
ownership, native ABI handling, user-range validation, extraction, delivery,
and teardown explicit without exposing an instruction or secret byte to
source.

## Decision

After the initial user-random reservation from `TK-DEC-112`, the qualification
root shall consume the live generation-three generator and that reservation
through one
`topal.systems.entropy.user-random.bind-reusable/1` transition. The transition
returns one affine service capability bound to process identity one, Linux
interface identity one, and user-random service identity one. Initial user
entry requires that service, and process return consumes it only after the
provider has destroyed its remaining secret state.

The service retains the already extracted output identity two as its first
available block and takes exclusive ownership of the generator. A successful
nonempty request may return 1 through 16 bytes. The first success uses the
reserved output. Before every later success, the service performs the same
ChaCha20 extraction and fast-key-erasure transition qualified by
`TK-DEC-110`: the persistent key is replaced before output publication, only
the requested prefix is copied and verified, all unselected output bytes are
discarded, and the complete 32-byte output is wiped and verified. No remainder
is retained between calls.

Zero length returns zero without extracting or changing service state. An
unknown flag bit, the `GRND_RANDOM | GRND_INSECURE` combination, or a request
larger than 16 returns `-EINVAL` without extraction. An invalid complete
writable range returns `-EFAULT` without extraction. Complete argument and
range validation therefore precedes every later generator transition. Current
valid Linux flags do not change delivery because the bound generator is ready.

The provider may retain private references to the key, current output, and
service state. It shall not publish them or confuse the address-space state
with its retained page-table backing identity. On process exit it wipes and
verifies a still-unused initial output, the live generator key, and the service
state before clearing the process binding. The x86-64 provider continues to
map the service to native syscall 318; other providers choose their own Linux
ABI mechanism.

The ring-3 qualification shall preserve the existing zero, invalid-flag, and
guard-crossing probes, then complete two consecutive 16-byte requests. It
reads all four returned qwords without comparing secret-derived values with a
constant. The second success proves that the provider performed an on-demand
extraction after consuming the initial output.

## Architecture pressure test

Exclusive generator ownership, validation-before-extraction, fast key erasure,
per-call output wiping, process binding, and teardown destruction are portable.
x86-64 selects syscall 318, `syscall` entry registers, GS-relative private
process state, and its page-fault recovery mechanism. AArch64 and RISC-V select
their own Linux syscall tables, exception return paths, and private service
storage. None exposes a register, MSR, system register, output address, or
instruction to portable source.

The process service deliberately owns one sequential generator in this phase.
That maps directly to the single-thread qualification while leaving a later
implementation free to add a lock, per-CPU generators, or batched output under
the same observable call contract. Such concurrency policy is not hidden in
this transition.

## Alternatives rejected

Pre-extracting a fixed list of outputs was rejected because it merely raises a
one-shot bound and extends secret lifetime before demand. Retaining the unused
half of a delivered block was rejected because it complicates fault and
teardown ownership and weakens the existing complete-output wipe boundary.
Calling a source-visible ChaCha20 primitive from the syscall path was rejected
because it would expose cryptographic representation and target call mechanics.
Returning raw virtio-rng bytes was rejected because it bypasses conditioning
and key erasure. Treating `-ENOSYS` after the first success as reusable service
support was rejected as false compatibility.

## Deliberate bounds

This increment admits one process, one thread, one sequential generator, the
existing maximum of 16 bytes per call, current Linux flag validation, repeated
successful calls while the process lives, and fail-closed teardown. It does
not provide requests larger than 16 bytes, partial completion, blocking before
readiness, signal interruption, concurrent callers, fork or exec inheritance,
`/dev/random`, `/dev/urandom`, vDSO `getrandom`, reseeding, per-CPU generators,
batching, queue recycling, asynchronous interrupt routing, reset, or recovery.
