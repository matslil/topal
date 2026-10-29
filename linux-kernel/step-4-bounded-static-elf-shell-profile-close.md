# Step 4 increment: bounded static-process system-profile close

## Outcome

This increment consumes the clean shell's open `/etc/profile` description
after its empty-file observation. The clean fixture issues native `close(3)`,
requires result zero, and only then proceeds to sealed command input.

The x86-64 provider accepts the close only at the exact unchanged shell-input
phase with every prior interactive observation complete, the profile metadata
and EOF facts published, and the process-owned description live. It validates
the active stack and child disposition, consumes the description, proves it is
no longer live, publishes a distinct closed-state history fact, and returns
zero without retaining a native argument. Sealed input and terminal rollback
require the consumed-description state; rollback clears the closed-state fact
with the retained profile history. The pinned external BusyBox echo profile
requires the complete optional state to remain absent.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-PROFILE-CLOSE-001` and decision
`TK-DEC-218` bind the transition. Structural tests inspect native syscall 3,
descriptor three, the exact predecessor state, description consumption and
proof, closed-state publication, successor gating, terminal clearing, and full
context restoration.

Artifact `/197`, provider `/187`, provider object `/179`, root object `/186`,
boot adapter `/97`, and gate `/199` identify the qualified contract. The
Python physical mirror retains the exact 2,024,544-byte fixture extent and
carries the same exact 2,065-byte continuation shape and 17-byte close
sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`604e00c4968b22d750865bc7044e83aab6b6fd969665e28687f4507b5745c1f7`
and
`3f80580c566e1f4efcf8a865f4718281a35fc6fb932f376943fd62fd77616b57`.
They bind boot image
`84248707ce63b59d05d9048c8f97aa007718b55f966b31c9729901c886d9da49`,
distinguish the clean consumed-description witness from the absent external
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

Risk is high because descriptor consumption changes affine process-owned
state. Review combines exact phase, descriptor, predecessor, consumption,
publication, successor, and erasure checks; physical evidence;
full-workspace validation; and repeated clean plus external QEMU runs.

This does not provide a prompt, refillable input, general commands,
filesystems, processes, containers, or VM-host support.
