# Step 4 increment: bounded static-process shell banner

## Outcome

This increment adds the single interactive banner write that follows the clean
shell's root-mailbox metadata miss. The clean fixture stages the exact 109-byte
BusyBox version and help banner, writes it to standard output, requires result
109, and erases and verifies its complete 112-byte stack extent before sealed
command input.

The x86-64 provider requires every predecessor interactive observation, exact
unchanged shell-input phase, live dispositions, checked hierarchy, standard
output, exact length, and a complete readable initial-stack candidate. It
copies all bytes to a fixed private 112-byte extent, validates the entire value
and zero tail before output, publishes only from private staging, then erases
and verifies every staging word. One private byte records completion, while the
pinned external BusyBox echo profile continues to require the full observation
qword to remain zero.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-BANNER-001` and decision `TK-DEC-211` bind
the transition. Structural tests inspect native syscall 1, descriptor one,
exact length and payload words, complete private copy, private-only output,
result, full staging erasure, predecessor and successor observations, and full
context restoration.

Artifact `/190`, provider `/180`, provider object `/172`, root object `/179`,
boot adapter `/90`, and gate `/192` identify the qualified contract. The Python
physical mirror retains the exact 2,024,544-byte fixture extent and the same
1,574-byte continuation; it differs from the Rust fixture only at the existing
gate sentinel byte.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`6722a90d4b1d6ab92d89433a8d40bcd680170d0e8149e9250361f90ac4605f5a`
and
`32213c448aedf998ee6a060ddd61408f402bdff5747b477c93ebe45768b0da8f`.
They bind boot image
`3283b483170fe1f470ce72afd9ca14b67cafd702cc3964344d0864346166933b`,
distinguish the clean exact 109-byte banner witness from the absent external
observation, complete all ordered markers, remain running afterward, and remain
serially quiescent. Both profiles passed twice. Their serial transcript hashes
are
`4153ef3304cb77ddb4d131285a976bd5277367449ecdc73bb70b43e7aa155295`
and
`4a0821fec50f6be05a7ae646e6fbcbc6400f936ee62c8a190cd66ee91dfa9097`.

The five Python gate contract tests, focused compiler, language, and semantic
suites, full Cargo workspace suite, formatting check, and diff check all pass
through the bounded validation runner.

## Risk and remaining boundary

Risk is high because this adds long application-controlled output, private
staging loops, and ordered process state. Review combines exact state,
descriptor, payload, length, candidate, complete-copy-before-output, result,
and erasure checks; physical transcript evidence; mirror parity; full-workspace
validation; and repeated clean plus external QEMU runs.

This does not provide general output formatting, banner variants, the following
terminal probe, current-directory and effective-identity queries, profile
lookup, a prompt, refillable input, general commands, filesystems, process
creation, containers, or VM-host support.
