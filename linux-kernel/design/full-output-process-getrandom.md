# Full-output process `getrandom`

This record resolves `TK-DEC-114` as the removal of the artificial 16-byte
per-call ceiling from the reusable initial-process random service. It remains a
systems-profile refinement: portable source names a qualified provider-output
extent, while each target provider owns its Linux ABI, user-range validation,
copy, fault-recovery, and cryptographic realization.

## Decision

The `topal.systems.entropy.user-random.bind-reusable/1` request shall admit a
maximum delivery equal to its complete 32-byte provider-output extent. A
successful call may request 1 through 32 bytes. It consumes exactly one output
object: the reserved output for the first success or one freshly extracted
output for each later success. The service shall not retain an unselected
suffix between calls.

Unknown flags, the incompatible `GRND_RANDOM | GRND_INSECURE` combination, and
requests larger than 32 bytes return `-EINVAL` without changing the service.
Zero length returns zero without change. An invalid complete writable range
returns `-EFAULT` without extraction or output consumption. Complete argument
and range validation therefore still precedes every generator transition.

After validation, the provider copies and verifies exactly the requested
prefix, wipes and verifies all 32 output bytes, advances the delivery count,
and retains only the successor generator. Selecting the complete output does
not weaken the wipe obligation. The existing process-exit destruction of an
unused first output, live generator key, output storage, and private service
state remains unchanged.

The ring-3 qualification shall retain the four non-consuming probes and two
existing 16-byte successes, then issue one 32-byte request. It shall read all
four returned qwords without comparing secret-derived values with constants.
The third success requires a fresh extraction and demonstrates full-output
copy and erasure after the reusable path is already active.

## Architecture pressure test

The portable contract names output extent, request extent, validation order,
affine consumption, successor-generator ownership, and erasure. It does not
name 64-bit registers, an instruction, syscall number 318, GS-relative state,
or an x86 page-fault mechanism. An AArch64 or RISC-V provider can bind the same
service to its native Linux syscall table and exception return path while
enforcing the same one-output state transition.

The number 32 follows the qualified ChaCha20 extraction policy, not the x86-64
ABI. A future generator with a different provider-output extent would state
that extent in its sealed request. Multi-output requests remain a separate
portable service decision because they add a loop, partial-progress behavior,
and possible interruption boundaries.

## Alternatives rejected

Keeping 16 bytes was rejected because it needlessly returns `-EINVAL` for a
request that fits the already live output object. Retaining bytes 16 through 31
after a shorter request was rejected because it changes the established
complete-output wipe and teardown boundary. Silently splitting requests larger
than 32 bytes was rejected for this increment because that would choose
partial-completion and interruption policy without qualifying it. Returning a
source-visible byte array or output address remains outside the systems
abstraction.

## Deliberate bounds

This increment supports one process, one thread, repeated sequential requests,
and 1 through 32 bytes from one output per successful call. It does not provide
multi-output requests, partial completion, pre-readiness blocking, signal
interruption, concurrent callers, fork or exec inheritance, `/dev/random`,
`/dev/urandom`, vDSO delivery, reseeding, per-processor generators, batching,
queue recycling, asynchronous interrupt delivery, reset, or recovery.
