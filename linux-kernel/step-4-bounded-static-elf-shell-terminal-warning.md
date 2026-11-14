# Step 4 increment: bounded static-process shell terminal warning

## Outcome

This increment adds the exact three-write warning that follows the clean shell's
standard-terminal probe failures. The clean fixture writes `"sh: "`, then
`"can't access tty; job control turned off"`, then a newline to standard error.
Each write uses a checked initial-stack candidate, requires the exact byte count,
and erases and verifies the complete candidate before the next segment or sealed
command input.

The x86-64 provider requires the completed `/dev/tty` and ordered terminal-probe
observations, exact unchanged shell-input predecessor phase, live dispositions,
checked hierarchy, standard error, and the exact next segment. One private byte
counts zero through three; each successful segment publishes its count, while
the pinned external BusyBox echo profile continues to require the complete
observation qword to remain zero.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-TERMINAL-WARNING-001` and decision
`TK-DEC-209` bind the transition. Structural tests inspect all three native
write sequences, exact segment bytes and lengths, checked result handling,
candidate erasure, ordered private-state publication, complete-count
requirements, and full context restoration.

Artifact `/188`, provider `/178`, provider object `/170`, root object `/177`,
boot adapter `/88`, and gate `/190` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and records the
same 216-byte warning sequence.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`3a03389e73bb6bfd6f30ad628f4ff719454be5e2a449a7985ad75814c2ea702e`
and
`056b80fcdb9abb03fa3b9339c62d771945073918930b2cbd3edadb8857341adf`.
They bind boot image
`9d741dda5ea2e6ffe15fd84a84241c21191d058e023aed6929e68cacff0375f9`,
distinguish the clean exact three-segment warning witness from the absent
external observation, complete all ordered markers, remain running afterward,
and remain serially quiescent. Both profiles passed twice. Their serial
transcript hashes are
`6a97d67d4c2d57c123c3c0efacff0897ce8f1b0af8f058277a2d95ce842989a7`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler and language suites, full
Cargo workspace suite, formatting check, and diff check all pass through the
bounded validation runner.

## Risk and remaining boundary

Risk is high because this extends a shared native write selector with ordered
multi-segment output and extends private process state. Review combines exact
state, descriptor, payload, and extent checks; complete erasure and result
checks; physical transcript evidence; full-workspace validation; mirror parity;
and repeated clean plus external QEMU runs.

This does not provide terminal attributes, successful terminal operations,
arbitrary `ioctl`, the shell banner, `MAILPATH` lookup, profile lookup, a prompt,
refillable input, general commands, filesystem-backed execution, process
creation, containers, or VM-host support.
