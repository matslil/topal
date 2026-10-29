# Bounded static-process shell profile open

This record resolves `TK-DEC-215` as the first successful pathname-backed file
open in the bounded static shell. It introduces one immutable empty
`/etc/profile` object so later profile metadata, input, and close operations can
advance the shell toward its prompt without claiming a general filesystem.

## Decision

The bounded static-process entry may admit exactly one read-only open of the
sealed system profile after the post-banner effective-identity fact and before
profile metadata, input, close, prompt, or sealed-command work. The operation
shall require the live process, sealed executable continuation, active checked
address-space hierarchy, retained executable pathname, live child and ignored-
quit dispositions, exact shell-input predecessor phase, every prior
interactive observation, the unused next descriptor slot, and an immutable
empty profile-file object owned by the launch profile.

The provider shall validate the complete pathname before allocating exactly
the lowest free descriptor, bind that descriptor to a process-owned read-only
description for the immutable object, return the descriptor number, and retain
neither the user pathname pointer nor lookup scratch. One private lifecycle
fact shall publish only after complete validation and allocation and shall
reject a duplicate, skipped-predecessor, reordered, differently flagged, or
substituted-path operation. Later profile operations and terminal rollback
shall require the fact. Process teardown may revoke the still-open bounded
description; a later explicit close shall consume it first. The pinned external
echo-command profile shall omit this optional observation.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 257,
`openat(AT_FDCWD, 0x4a6938, O_RDONLY, 0)`, immutable pathname
`"/etc/profile\0"`, and result three. The pathname lies in the checked sealed
executable role. A fresh provider-private byte records the live profile
description after the post-banner effective-identity fact.

## Architecture pressure test

The sealed path role, immutable file identity, read-only access, lowest-free
descriptor allocation, description lifetime, and no-pointer-retention effect
are portable Linux-personality policy. X86-64, AArch64, and RISC-V may select
different syscall and flag encodings, calling conventions, pathname locations,
and private descriptor layouts. No selector, integer descriptor encoding,
pointer, or instruction becomes Topal source vocabulary.

## Alternatives rejected

Returning `-ENOENT` was rejected because it would bypass the observed profile
open and cannot exercise file metadata or input. Reusing a host profile was
rejected because host contents are unsealed and non-reproducible. Returning a
descriptor without retained object state was rejected because later operations
could not validate lifetime. Adding arbitrary lookup, writable access, or a
general filesystem was rejected as unnecessary scope.

## Deliberate bounds

This increment admits one exact clean-fixture open of one immutable empty
profile file and preserves the external echo-command profile. It does not add
other paths, directories, creation, mutation, profile metadata or input,
explicit close, a prompt, refillable input, general filesystems, processes,
containers, or virtualization.
