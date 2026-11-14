# Step 4 increment: bounded BusyBox `true` image replacement

## Outcome

The qualified shell now receives `exec /bin/busybox true\n` as its fifth
publication, after the complete reopen and `pwd` transaction. Both the clean
fixture and the pinned external BusyBox issue syscall 59 with the exact
`/bin/busybox` pathname, two arguments, four inherited environment entries,
and null vector terminators. The five input results are 11, 5, 294, 24, and 23.

Successful replacement does not return to the shell. The provider restores the
writable ELF load from a separate kernel-private pristine snapshot, verifies
the copy, zeros and verifies BSS and all 34 anonymous heap pages, revokes the
old heap mappings, constructs a fresh stack and auxiliary vector, and extracts
new `AT_RANDOM` bytes. It resets caught dispositions, TLS, thread registrations,
SIMD and general registers before entering the ELF entry point. PID one, root
credentials, root cwd, standard descriptions, and the reopened non-CLOEXEC
descriptor three survive replacement. The `true` application runs startup and
exits through `exit_group(0)`; terminal handling revokes its inherited description
and restores the enclosing affine owners.

Syscall admission saves the complete user register set and switches to the
retained kernel stack before any provider work. Exec checks the unused fresh
user-stack prefix remains zero after random generation and before entry, so
neither a provider return address nor cryptographic scratch can enter that
prefix. Structural qualification enforces the register-save/stack-switch order.

The native shell resets SIGQUIT before exec, retaining its native restorer,
flags and mask while selecting the default handler. The root `cd` explains
the inherited `OLDPWD=/` entry. The external argument/environment corpus is
`[/bin/busybox, true]` and `[SHLVL=1, OLDPWD=/,
PATH=/sbin:/usr/sbin:/bin:/usr/bin, PWD=/]`.

## Contract and evidence

Decision `TK-DEC-238` and rule
`TOPAL-SYSTEMS-STATIC-ELF-SHELL-BUSYBOX-TRUE-EXECVE-001` govern this transition.
Artifact `/217`, root object `/206`, provider `/207`, provider object `/199`,
boot adapter `/117`, and gate `/217` bind the implementation. The ELF remains
2,024,544 bytes; the independently constructed clean continuation is 7,091
bytes. The clean entry trampoline preserves the sealed entry address while
placing the longer startup body in a separate executable interval.

Physical evidence is recorded in
`labs/qemu/x86_64/results/topal-toolchain-gate.json` and
`labs/qemu/x86_64/results/topal-pinned-static-busybox-gate.json`. The evidence
binds artifact identities, the exec profile, exact environment, descriptor
inheritance, fresh random extraction, complete ordered markers, and the final
running, quiescent halt. The external executable remains pinned by SHA-256
`7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd`.

## Review and remaining boundary

Risk is high: image replacement crosses memory, ABI, register, descriptor and
entropy lifetimes. Review checks complete argument extents before consumption,
active hierarchy and heap permissions, separate snapshot/heap storage sizes,
mutable-byte restoration, old-heap erasure and invalidation, fresh stack and
random state, preservation of kernel caller registers and ignored dispositions,
ordinary replacement startup, nonreturning entry, and complete rollback.
Structural checks and both physical profiles cover the transition together.

The full Cargo workspace suite, compiler structural suite, and five Python
lab contract tests pass. Validation uses the repository's bounded wrapper
for memory-intensive commands.

This is one same-artifact static replacement, not a completed distribution
boot. General executable selection and argument vectors, dynamic loading,
multiple processes, writable storage, networking, Docker/Podman, and VM-host
qualification remain required by the original plan. No core-language change
or Topal source assembly was introduced.
