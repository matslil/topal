# Step 4 increment: bounded static-process shell child disposition

## Outcome

This increment adds one process-owned child-status disposition after the
bounded shell `getpid` transition. The clean fixture constructs its complete
native request in the established disjoint stack-scratch prefix, invokes the
provider, requires success, clears the user request, and exits. The provider
copies the complete value, validates executable handler and trampoline
addresses plus the exact policy and mask, retains no user pointer, and clears
the private disposition before complete process rollback.

The pinned external BusyBox `echo` profile still terminates after its newline.
The provider admits the same disposition transition after `getpid` for the
future external `sh` profile, but this increment does not change the sealed
external arguments or claim shell execution.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-SHELL-SIGCHLD-001` and decision `TK-DEC-201`
bind the portable lifecycle. X86-64 qualification uses `rt_sigaction` syscall
13, signal 17, null old-action output, signal-set size eight, `SA_RESTORER`,
and the approved child mask. Negative source tests reject another startup or
policy. Structural tests inspect the clean request, syscall dispatch,
continuation and stack checks, copied private fields, phase 18, terminal
validation, and clearing.

Artifact `/180`, provider `/170`, provider object `/162`, root object `/169`,
boot adapter `/80`, and gate `/182` identify the qualified contract.

## Physical evidence

The clean and external evidence files are
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`, with SHA-256
`1a1bd51260d7aa145bfceae4b1d65348176bcf956274c340742b422f7cd72d94`
and
`8299c36db12e87e357077bf866ee33d5d1364b1842c97481a68cc69c96b1afed`.
They distinguish the clean installed-and-cleared witness from the external
echo terminal boundary, complete every ordered marker, contain exactly one
newline, remain running afterward, and remain serially quiescent. Both bind
boot image
`b1f62d677b67bfb6589084daa9415ac018c8e18589e745e0631e8147b9d1381b`
and serial transcript
`5f31cf4fd830d8002d4833d5bd6ea32f6f016fa909e0e1f1429f483b5437d84b`.

## Risk and remaining boundary

Risk is high because this introduces retained native process state and a
pointer-bearing syscall on the real-shell path. Review combines complete-copy
and executable-range checks, exact private-state lifecycle, focused language
and compiler tests, full-workspace validation, and clean plus external QEMU
runs.

This adds no signal generation, pending queue, delivery, signal frame,
`rt_sigreturn`, handler execution, restart execution, old-action publication,
arbitrary disposition, child creation, `getppid`, filesystem query, terminal
discovery, shell input, Docker, Podman, or VM-host support.
