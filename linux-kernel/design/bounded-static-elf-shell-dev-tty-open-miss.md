# Bounded static-process shell `/dev/tty` open miss

This record resolves `TK-DEC-207` as the pathname probe immediately following
the bounded shell's console metadata observation. It reports the truthful
absence of a controlling-terminal pathname without introducing a VFS or
changing the separately modeled inherited console descriptions.

## Decision

The bounded static-process entry may admit one ordered attempt to open the
immutable pathname `/dev/tty` after the standard-output console `fstat` and
before sealed command input. The operation shall require the live process,
sealed executable continuation, active checked address-space hierarchy,
retained initial stack, live child and ignored-quit dispositions, exact shell-
input predecessor phase, completed console-metadata observation, `AT_FDCWD`,
read/write open intent, zero mode, and the exact complete pathname in an
immutable executable mapping.

The provider shall validate every argument and all nine pathname bytes before
returning `-ENOENT`. It shall not access user output, allocate a descriptor,
retain a pathname pointer or lookup object, advance the startup phase, or
change any process resource. A private at-most-once observation shall order the
later sealed input and terminal rollback. The clean fixture shall issue and
validate the miss after its complete console-stat check. The pinned external
echo-command profile shall omit both optional interactive prerequisites.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 257 with
`dirfd=-100`, the clean fixture's exact immutable `/dev/tty\0` candidate,
`flags=O_RDWR` (`2`), and `mode=0`. The returned native value is exactly
`-ENOENT` (`-2`). Its observation shares the already qualified private
observation qword with console `fstat`; the clean state becomes two ordered
one-byte facts and the external echo state remains the all-zero qword. Existing
initial-zero and complete qualification-restoration proofs cover the storage.

## Architecture pressure test

The immutable path identity, relative-directory meaning, read/write intent,
absence result, no-allocation effect, ordering, and observation lifecycle are
portable Linux-personality policy. X86-64, AArch64, and RISC-V may select
different syscall encodings, register assignments, pointer representations,
and provider lookup adapters. No native selector, number, pointer, or
instruction becomes Topal source vocabulary.

## Alternatives rejected

Fabricating a successful controlling-terminal open was rejected because no
pathname-backed terminal object exists. Treating the inherited serial console
description as `/dev/tty` was rejected because descriptor inheritance and
pathname resolution are distinct interfaces. Adding a general `openat` or VFS
service was rejected because this prerequisite needs one immutable absence
fact only. Returning `-ENOSYS` was rejected because BusyBox distinguishes an
unavailable syscall from an absent optional device. Activating interactive
arguments here was rejected because terminal probes and warning output remain
separate testable dependencies.

## Deliberate bounds

This increment admits one exact clean-fixture `/dev/tty` absence observation
and preserves the external echo-command profile. It does not add other paths,
directories, files, devices, descriptor allocation, permissions, mounts, a
VFS, terminal attributes, an interactive banner or prompt, mutable input,
processes, containers, or virtualization.
