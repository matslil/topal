# Step 4 increment: bounded static-process system-profile EOF

## Outcome

This increment adds the clean shell's sole content observation for its open
`/etc/profile`. The clean fixture issues native `read(3, candidate, 128)`,
requires end-of-file, proves all candidate bytes unchanged, erases the complete
candidate, and proves erasure before continuing.

The x86-64 provider accepts the read only at the exact unchanged shell-input
phase with every prior interactive observation complete, profile metadata
published, and the process-owned immutable empty description live at position
zero. It validates the complete stack candidate before observing the object,
returns zero without touching user memory or changing position, and retains no
pointer. A fresh private fact records completion; sealed input and terminal
rollback require it, and rollback clears it together with the still-open
description. The pinned external BusyBox echo profile requires the complete
optional state to remain absent.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-PROFILE-READ-001` and decision
`TK-DEC-217` bind the transition. Structural tests inspect native syscall 0,
descriptor three, the exact 128-byte checked stack candidate, live description
and zero position, no-output result zero, complete unchanged/erasure fixture
proofs, completion-fact lifecycle, successor gating, terminal clearing, and
full context restoration.

Artifact `/196`, provider `/186`, provider object `/178`, root object `/185`,
boot adapter `/96`, and gate `/198` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and
carries the same exact 2,048-byte continuation shape and 98-byte EOF sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`857288b3395d659787db6442bbda00fa0fab1aa1414873dc01b59bbf22a3c8dc`
and
`ca2b76c2855b2e23ae7fbaa243f95e33981776f9e2dc9e00cf92f9119f3da18f`.
They bind boot image
`677e2c63ed54c66e3a481b9ae95f9e6b00e04989f0095422f111e7d0014145af`,
distinguish the clean empty-profile EOF witness from the absent external
observation, complete all ordered markers, remain running afterward, and
remain serially quiescent. Both profiles passed twice. Their serial transcript
hashes remain
`4153ef3304cb77ddb4d131285a976bd5277367449ecdc73bb70b43e7aa155295`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler and language suites,
semantic suite, full Cargo workspace suite, formatting check, and diff check
all pass through the bounded validation workflow where applicable.

## Risk and remaining boundary

Risk is high because syscall zero now dispatches between a live file
description and sealed command input. Review combines exact descriptor, phase,
object/position, candidate, count, no-output, publication, successor, and
erasure checks; physical evidence; full-workspace validation; and repeated
clean plus external QEMU runs.

This does not provide profile contents, explicit close, arbitrary descriptor
reads, a prompt, refillable input, general commands, filesystems, processes,
containers, or VM-host support.
