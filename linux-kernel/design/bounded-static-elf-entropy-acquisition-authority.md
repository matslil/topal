# Bounded static ELF entropy-acquisition authority

This record resolves `TK-DEC-158` as the request boundary between one
entropy-pending static initial process and the independent entropy pipeline. It
is a systems-profile refinement only and adds no core-language element.

## Decision

`static image construction authorize entropy acquisition` shall consume the
exact `EntropyPendingStaticElfInitialProcess` produced by `TK-DEC-157`. It shall
accept only output identity one, resulting generator generation two, and policy
`bounded-static-elf-initial-entropy-acquisition`, and shall produce one opaque
affine `EntropyAcquiringStaticElfInitialProcess`.

The new owner preserves every retained resource and byte while granting only
the future authority to accept the existing qualified first entropy output. It
does not assert that acquisition has occurred. The provider shall completely
revalidate the retained process before transfer. Source shall publish
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK` only after success
and shall observe no process resource, output byte, address, device
representation, or allocator state.

For qualification in this increment, `static image construction revoke entropy
acquisition` shall immediately consume the exact acquisition owner under policy
`bounded-static-elf-entropy-acquisition-qualification-rollback`, prove every
resource and byte unchanged, return the exact entropy-pending owner, and publish
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK`. Existing handoff
revocation and retained-process restoration then follow.

For qualified x86-64 realization, compiler-controlled root state shall retain
the private payload, metadata, hierarchy, leaf, entry, and stack-backing
identities. Both transitions shall completely validate the inactive hierarchy,
image, deterministic process table, future random-object zeros, strings, guard,
and otherwise-zero stack bytes. They shall perform no device access,
allocation, release, store, fence, invalidation, activation, extraction, output
copy, or delivery.

## Architecture pressure test

The portable contract names a qualified output identity and successor
generation, not DMA addresses, page-table shapes, interrupt controllers,
cipher instructions, or auxiliary-vector encodings. X86-64, AArch64, and
RISC-V providers can acquire entropy differently while preserving the same
exclusive process owner and exact future-output authority.

## Alternatives rejected

Letting the entropy pipeline select an implicit destination was rejected
because it would hide which process and output contract are authorized. Passing
a stack address or writable buffer was rejected because it would expose
provider representation and premature mutation authority. Treating
authorization as proof of acquisition was rejected because request, device,
cryptographic, and output-liveness failures are distinct. Combining request,
acquisition, and `AT_RANDOM` publication was rejected because it would collapse
independent failure and review boundaries.

## Deliberate bounds

This increment admits one exact acquisition authorization and immediate
qualification rollback. It rejects another owner, output identity, generation,
policy, duplicate or partial transfer, mutation, acquisition, output use, early
handoff revocation, or recoverable disposition.

It does not span the entropy pipeline, acquire, consume, or expose entropy,
publish `AT_RANDOM`, initialize TLS, construct a user context, enter ring 3,
execute BusyBox, or add a syscall.
