# Step 4 increment: bounded static-process system-profile metadata

## Outcome

This increment adds the clean shell's metadata observation immediately after
its successful read-only `/etc/profile` open. The clean fixture issues native
`fstat(3, ...)`, requires a completely initialized immutable empty regular-file
value, erases the user candidate, and proceeds only after proving erasure.

The x86-64 provider accepts the observation only at the exact unchanged shell-
input phase with every prior interactive observation complete and the process-
owned profile description live. It derives the result from that retained
object, constructs all 144 bytes in fresh private staging, validates the exact
stack destination before mutation, publishes and verifies the complete value,
then erases and verifies all staging. A fresh private fact records completion;
sealed input and terminal rollback require it, and rollback clears it together
with the open description. The pinned external BusyBox echo profile requires
the complete optional state to remain absent.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-PROFILE-FSTAT-001` and decision
`TK-DEC-216` bind the transition. Structural tests inspect native syscall 5,
descriptor three, exact candidate, complete regular-file fields, private
staging construction/copy/verification/erasure, result zero, completion-fact
publication, successor gating, terminal clearing, and full context
restoration.

Artifact `/195`, provider `/185`, provider object `/177`, root object `/184`,
boot adapter `/95`, and gate `/197` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and
carries the same 1,950-byte continuation shape and exact 227-byte metadata
sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`49e5abd2ae9b9f5722160dd3e08acf9b5b3972cfe81ec29a1f47de0fdb7981ce`
and
`1acdb4f18825fc57a2d81ddee0d96150af2ba1374f11f09b49a1b058a2dc61cc`.
They bind boot image
`394a0bffe40ea998ffb104b2dba2e88d9ca66207b1e7dac87244aa68f1d54c10`,
distinguish the clean complete-profile-metadata witness from the absent
external observation, complete all ordered markers, remain running afterward,
and remain serially quiescent. Both profiles passed twice. Their serial
transcript hashes are
`4153ef3304cb77ddb4d131285a976bd5277367449ecdc73bb70b43e7aa155295`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler and language suites,
semantic suite, full Cargo workspace suite, formatting check, and diff check
all pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because this publishes a complete native UAPI structure from a
new live descriptor. Review combines exact phase, object identity, complete
destination and field checks, validation-before-copy, private staging erasure,
fact lifecycle, physical transcript evidence, full-workspace validation, and
repeated clean plus external QEMU runs.

This does not provide profile input, descriptor close, arbitrary descriptor
metadata, a prompt, refillable input, general commands, filesystems, processes,
containers, or VM-host support.
