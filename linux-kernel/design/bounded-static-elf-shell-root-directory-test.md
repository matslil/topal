# Bounded static-process shell root-directory test

Status: approved systems-feature design (`TK-DEC-223`).

## Purpose

The bounded shell can change to and print sealed root, but it has not observed
root as a filesystem object. This increment inserts exact `test -d /\n` after
`cd /\n` and before `pwd\n`. It proves one application-issued successful
pathname metadata lookup and directory-type decision without granting general
traversal, mutation, or process creation.

## Decision

The bounded static-process entry may admit exactly five ordered command
publications through one checked process buffer: `echo topal\n`, `cd /\n`,
`test -d /\n`, `pwd\n`, and `exit\n`. Each refill shall replace a consumed
publication, zero and validate its sealed tail, retain no user pointer, and
advance one exclusive private input-owner state. The clean profile shall
publish exact `/ # ` before every command; the pinned external BusyBox profile
shall require those optional prompts absent.

After the root-directory generation has been replaced by `cd /`, both
applications shall issue one metadata request for exact root. The provider
shall validate the live process, executable continuation, active hierarchy,
two-page stack, dispositions, exact predecessor, current input owner, lookup
selector and flags, complete readable `/\0` pathname, and complete writable
metadata candidate before publishing one complete native value for the sealed
immutable root-directory object. That value shall identify a directory owned
by sealed root credentials with read, write, and search permission for root
and read plus search permission for group and other. Success shall return zero
and publish one private root-directory-observation fact. No pathname,
destination, or native metadata bytes shall be retained.

The application shall consume the metadata only to decide that root is a
directory; `test` shall produce no output. The following qualified `pwd` shall
require the observation fact and preserve its existing profile-specific
behavior. Terminal handling shall require the fifth publication live, erase
the complete command buffer, consume its owner, and clear the directory
generation, metadata observation, and all command-time facts.

## X86-64 binding

The x86-64 realization binds the lookup to native syscall 262 with
`AT_FDCWD=-100`, flags zero, exact `/\0`, and one complete checked writable
144-byte native `stat` candidate. The published value shall have directory mode
`040755`, root UID and GID, no device-special identity, a 4096-byte preferred
I/O block, and a provider-sealed coherent value for every other native field.
The result is zero. Input results are 11, 5, 10, 4, and 5. Provider-private
phases and facts distinguish the added command, metadata publication, silent
test completion, later `pwd`, final input, and teardown. The affine stack
remains `[0x007f_e000, 0x0080_0000)` with both leaves RW/NX.

## Architecture pressure test

Exact-root identity, complete pathname and metadata transfers, immutable
directory metadata, silent Boolean command result, observation ordering, and
terminal erasure are portable Linux-personality policy. X86-64, AArch64, and
RISC-V may select different syscall encodings, native metadata layouts,
calling conventions, mapping layouts, and transfer implementations. No native
selector, structure layout, address, register, or instruction becomes Topal
source vocabulary.

## Alternatives rejected

Treating `test -d /` as true without an application syscall was rejected
because it fabricates command behavior. Returning only a mode word was rejected
because Linux publishes a complete native metadata value. Reusing console or
profile metadata was rejected because root is a distinct object. Host-root
inheritance, arbitrary paths, symlink traversal, mutable metadata, and a
general VFS were rejected as broader later work.

## Deliberate bounds

This increment proves one successful metadata lookup of the already sealed
root object and one silent directory test. It does not add relative lookup,
other pathnames, directory enumeration, file contents, mutable filesystem
objects, process creation, dynamic applications, containers, or
virtualization.
