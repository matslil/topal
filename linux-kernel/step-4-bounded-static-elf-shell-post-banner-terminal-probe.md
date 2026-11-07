# Step 4 increment: bounded static-process post-banner terminal probe

## Outcome

This increment adds the clean shell's standard-input terminal probe immediately
after the exact BusyBox banner. The clean fixture places a sentinel byte at the
candidate address, issues `ioctl(0, TCGETS, candidate)`, requires `-ENOTTY`,
requires the candidate to remain unchanged, and erases and verifies its byte
before sealed command input.

The x86-64 provider requires every predecessor interactive observation, exact
unchanged shell-input phase, live dispositions, checked hierarchy, descriptor
zero, request `TCGETS`, and a writable initial-stack candidate. It returns
`-ENOTTY` without touching the candidate and publishes one private completion
byte. The pinned external BusyBox echo profile continues to require the full
observation qword to remain zero.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-POST-BANNER-TERMINAL-PROBE-001` and
decision `TK-DEC-212` bind the transition. Structural tests inspect native
syscall 16, descriptor zero, request `TCGETS`, exact candidate, `-ENOTTY`,
candidate preservation and erasure, predecessor and successor observations,
and full context restoration.

Artifact `/191`, provider `/181`, provider object `/173`, root object `/180`,
boot adapter `/91`, and gate `/193` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and carries the
same 1,617-byte continuation as the Rust fixture; it differs only at the
existing gate sentinel byte.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`89cf3f6ef52f2c2831ba4ebed488b43d93a73f858d19946c62d03a5a560e8862`
and
`12dde322506e552a94dd4bbfef3792e1c628da979730473bc7a25300b39c48b2`.
They bind boot image
`761c3bd98b4a5e7e4f3db82edef6e634482bd357f53233b9ad16e208afaa860a`,
distinguish the clean post-banner stdin `-ENOTTY` witness from the absent
external observation, complete all ordered markers, remain running afterward,
and remain serially quiescent. Both profiles passed twice. Their serial
transcript hashes are
`4153ef3304cb77ddb4d131285a976bd5277367449ecdc73bb70b43e7aa155295`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, full Cargo workspace suite, formatting
check, and diff check all pass through the bounded validation runner.

## Risk and remaining boundary

Risk is high because this extends ordered application-controlled syscall state
and writable-candidate handling. Review combines exact state, descriptor,
request, candidate, negative result, non-mutation, and erasure checks; physical
transcript evidence; mirror parity; full-workspace validation; and repeated
clean plus external QEMU runs.

This does not provide terminal success, current-directory and effective-
identity queries, profile lookup, a prompt, refillable input, general commands,
filesystems, process creation, containers, or VM-host support.
