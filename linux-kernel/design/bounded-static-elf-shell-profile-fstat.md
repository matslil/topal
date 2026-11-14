# Bounded static-process shell profile `fstat`

This record resolves `TK-DEC-216` as the metadata observation for the live
bounded `/etc/profile` description. It publishes one complete immutable empty
regular-file value so the shell can advance to profile input without claiming
general descriptor metadata or a filesystem.

## Decision

The bounded static-process entry may admit exactly one `fstat` of the live
profile descriptor after its successful read-only open and before profile
input, close, prompt, or sealed-command work. The operation shall require the
live process, sealed executable continuation, active checked address-space
hierarchy, retained executable pathname, live child and ignored-quit
dispositions, exact shell-input predecessor phase, every prior interactive
observation, and the process-owned read-only description for the immutable
empty profile object.

The provider shall derive metadata from that retained object identity, build
the complete result in private staging, validate the complete destination
before user mutation, publish the complete value, erase and verify all staging,
retain no user pointer, and return zero. One private completion fact shall
publish only after the complete copy and staging erasure and shall reject a
duplicate, skipped-predecessor, reordered, wrong-descriptor, or substituted-
destination operation. Later profile input, close, and terminal rollback shall
require the fact. Process teardown may clear the fact and revoke the still-open
description; a later explicit close shall consume the description first. The
pinned external echo-command profile shall omit this optional observation.

The portable metadata meaning is one stable launch-owned regular-file
identity, link count one, root ownership, read-only permissions `0444`, zero
length, no allocated blocks or timestamps, and preferred I/O size 4096. The
object is immutable and empty for the lifetime of the process.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 5 with `fd=3`
and one exact clean-fixture continuation-stack candidate. It publishes the
complete native 144-byte `struct stat`: `st_dev=0`, `st_ino=2`, `st_nlink=1`,
`st_mode=S_IFREG|0444` (`0x8124`), UID/GID zero, zero padding, `st_rdev=0`,
`st_size=0`, `st_blksize=4096`, zero blocks, zero timestamp components, and
three zero unused words. Fresh provider-private staging and one completion byte
participate in initial-zero, terminal-zero, and qualification-restoration
proofs.

## Architecture pressure test

Object-derived metadata, immutable regular-file meaning, complete private
staging, destination validation, nonretention, and observation lifecycle are
portable Linux-personality policy. X86-64 selects syscall 5 and its native
144-byte layout. AArch64 and RISC-V may select different descriptor-relative
syscalls, calling conventions, UAPI layouts, padding, and private staging. No
native selector, layout, pointer, or instruction becomes Topal source
vocabulary.

## Alternatives rejected

Reusing the console metadata was rejected because the profile is a distinct
regular file, not a character device. Returning fabricated host metadata was
rejected because host identity and timestamps are neither sealed nor
reproducible. Constructing the value directly in user memory was rejected
because a late validation failure could expose a partial result. Generalizing
the operation to arbitrary descriptors or filesystem nodes was rejected as
unnecessary scope.

## Deliberate bounds

This increment admits one exact metadata observation for the already-open
immutable empty profile and preserves the pinned external echo-command profile.
It does not add profile input, descriptor close, other files, arbitrary
metadata queries, a prompt, refillable input, general filesystems, processes,
containers, or virtualization.
