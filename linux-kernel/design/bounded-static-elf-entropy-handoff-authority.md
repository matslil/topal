# Bounded static ELF entropy-handoff authority

This record resolves `TK-DEC-157` as the ownership boundary between the
retained static initial process and the independent entropy pipeline. It is a
systems-profile refinement only and adds no core-language element.

## Decision

`static image construction authorize entropy handoff` shall consume the exact
retained process returned by `TK-DEC-156`, require process identity
`bounded-static-elf-initial-process` and policy
`bounded-static-elf-initial-random-delivery`, and produce one opaque affine
`EntropyPendingStaticElfInitialProcess`. The new owner preserves every retained
resource and byte while granting only the future authority to accept one
qualified entropy output for the already-authorized random slot.

The provider shall completely revalidate the retained process before transfer.
Source shall publish
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK` only after success. It
observes no stack address, random-slot address, output byte, resource tuple, or
provider representation.

For qualification in this increment, `static image construction revoke entropy
handoff` shall immediately consume the exact entropy-pending owner under policy
`bounded-static-elf-entropy-handoff-qualification-rollback`, prove every
resource and byte unchanged, return the exact retained-process owner, and
publish `TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK`. Existing retained-
process restoration follows that witness.

For qualified x86-64 realization, compiler-controlled root state retains the
payload, metadata, hierarchy, leaf, entry, and stack-backing identities. Both
transitions completely validate the inactive hierarchy, image, deterministic
process table, future random-object zeros, strings, guard, and otherwise-zero
stack bytes. They perform no allocation, release, store, fence, invalidation,
activation, output extraction, or delivery.

## Architecture pressure test

The portable contract transfers authority to accept one conditioned output,
not an address or native auxiliary-vector encoding. X86-64, AArch64, and RISC-V
providers may use different stack layouts and translation mechanisms while
preserving the same affine process/output boundary. Native tag encoding,
addresses, page tables, and secret-copy instructions remain provider-private.

## Alternatives rejected

Letting the entropy pipeline reach into loader state was rejected because it
would couple independent subsystems and expose mutation authority. Passing a
stack address or writable buffer to entropy code was rejected because it would
leak representation and permit partial publication. Combining handoff, entropy
acquisition, extraction, and `AT_RANDOM` publication was rejected because it
would collapse independent failure and review boundaries.

## Deliberate bounds

This increment admits one exact authorization and immediate qualification
revocation for one retained process. It rejects another owner, process identity,
policy, duplicate or partial transfer, changed resource or byte, early restore,
output use, or recoverable disposition.

It does not span the entropy pipeline yet, consume or expose entropy, publish
`AT_RANDOM`, initialize TLS, construct a user context, enter ring 3, execute
BusyBox, or add a syscall.
