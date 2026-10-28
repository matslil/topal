# Bounded static-process shell console `fstat`

This record resolves `TK-DEC-206` as the first interactive-shell prerequisite
after the sealed echo-command profile. It reuses the approved immutable console
object and native metadata contract without claiming that the console is a
terminal or that arbitrary descriptor metadata is available.

## Decision

The bounded static-process entry may admit one ordered `fstat` observation for
standard output after shell signal setup and before sealed command input. The
operation shall require the live process, sealed executable continuation,
active checked address-space hierarchy, retained initial stack, live child and
ignored-quit dispositions, exact shell-input predecessor phase, descriptor one,
and one exact complete writable stack candidate. It shall be observed at most
once and shall not change the startup phase.

The result is the already approved immutable console metadata from
`TOPAL-SYSTEMS-USER-FSTAT-001`: stable identity one, character-device kind,
root ownership, permissions `0600`, link count one, ttyS0 special-device
identity, zero size, blocks, timestamps, and unused fields, and preferred I/O
size 4096. The provider shall construct the complete value in private staging,
validate the complete destination before user mutation, copy the complete
value, clear and verify all staging, retain no user pointer, and return zero.
The later sealed-input transition and successful terminal handling shall both
require the header-profile-specific observation state. Terminal handling shall
clear the observation before complete process rollback.

The clean fixture shall issue and validate the observation before its existing
sealed echo-command read. The pinned external BusyBox echo-command profile
shall remain unchanged and shall prove that the optional observation is absent.
This separates the new ABI mechanism from a future switch to interactive shell
arguments, whose additional `/dev/tty`, terminal-probe, warning, metadata,
banner, profile, prompt, and refillable-input dependencies remain unclaimed.

## X86-64 binding

The x86-64 realization binds the operation to native syscall 5 with
`fd=1` and the clean fixture's exact continuation-stack candidate. It publishes
the complete native 144-byte `struct stat`: `st_dev=0`, `st_ino=1`,
`st_nlink=1`, `st_mode=S_IFCHR|0600` (`0x2180`), UID/GID zero, zero padding,
`st_rdev=0x4400`, `st_size=0`, `st_blksize=4096`, zero blocks, zero timestamp
components, and three zero unused words. Private staging and the one-byte
observation occupy an exact extension of the existing process record and are
included in its initial-zero, terminal-zero, and qualification-restoration
proofs.

## Architecture pressure test

Console identity, metadata meaning, lookup-before-copy ordering, complete
private staging, destination validation, nonretention, and observation
lifecycle are portable systems policy. X86-64 selects syscall 5 and its native
144-byte layout. AArch64 and RISC-V select their qualified descriptor-relative
syscall and UAPI layout adapters; neither must reproduce x86-64 padding,
syscall numbering, register assignment, or staging representation. No native
layout or instruction is added to Topal source.

## Alternatives rejected

Calling the earlier general descriptor implementation directly was rejected
because this process-entry profile owns a disjoint address space, context, and
rollback boundary. Constructing the value in user memory was rejected because
a late failure could expose a partial result. Using the user stack as privileged
staging was rejected because the candidate must not be mutated before complete
validation. Activating interactive BusyBox arguments in the same increment was
rejected because it would combine several independently testable syscall and
presentation dependencies. Reporting terminal attributes was rejected because
metadata and `TCGETS` are distinct interfaces.

## Deliberate bounds

This increment admits one exact standard-output metadata observation in the
clean shell witness and preserves the pinned external echo-command profile. It
does not add arbitrary descriptors or candidates, path lookup, `newfstatat`, a
filesystem, terminal attributes, `/dev/tty`, a banner or prompt, mutable input,
general shell commands, processes, containers, or virtualization.
