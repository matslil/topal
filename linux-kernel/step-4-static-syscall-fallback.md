# Step 4 increment: recoverable unknown static-process syscalls

A syscall number with no implemented arm now returns Linux ENOSYS instead of
halting the static-process adapter. The process can continue using implemented
services before and after exec. Dispatch normalizes the x86-64 selector to its
low 32 bits before selecting any shared or predecessor arm; upper bits cannot
hide an implemented native operation. Unselected ABI-tag selectors remain
unsupported.

Decision `TK-DEC-250` and rule `TOPAL-SYSTEMS-STATIC-PROCESS-ENOSYS-001`
extend the already approved initial-process fallback policy. Artifact `/226`,
root `/215`, provider `/216`, provider object `/208`, boot `/126` and gate `/226`
identify this increment. Core-language meaning is unchanged.

## Dispatch and preservation

Every implemented arm precedes the fallback. An unselected number returns exact
negative errno 38 through the existing SYSRET continuation, restoring non-result
registers and clearing saved entry registers. The fallback inspects no unused
arguments or user bytes and changes no startup phase, descriptor/position, cwd,
signal, image or execution-owner state. No new syscall implementation is
advertised. A selected arm's unqualified argument profiles remain outside its
qualification.

The source review uses pinned Linux `kernel/sys_ni.c`,
`arch/x86/entry/syscall_64.c` and `arch/x86/entry/entry_64.S`. The entry treats
the selector as a native integer; dispatch uses an unsigned range check and
routes unimplemented calls to ENOSYS. This project implements the observable
policy independently through the closed native adapter, without source assembly
or a provider-owned unsupported-call catalogue.

## Executed qualification

Independent Rust/Python user-image encoders invoke selectors `-1`, `0x7fffffff`,
`0x2000`, `0x100002000` and `0x40000027`. Every call supplies invalid unused
pointer candidates and expects ENOSYS. All six argument registers and the
existing non-result-register witness survive, and a user-memory canary remains
unchanged. Each probe is followed by `getpid` with upper selector bits set,
which must select the implemented native operation and return one. Later full
uname, file, signal, cwd, failed-exec and terminal checks still execute.

These witnesses run before first exec and after each of the maximum fixture's
three replacements. Empty/null-environment variants and the external application
corpus remain included. The matrix has three clean variants, twenty-four
external cases and five RTC cases using linked kernel SHA-256
`9856f7642c7b8c7c1a3177b518a3f3c2c59ba31039a718aff33e2c33dd2f17a6`.
The completed-deadline notification regression runs on that same kernel; its
historical predecessor rejection remains separate. Full bounded Cargo workspace
tests, fourteen lab contract tests, formatting, whitespace, local links and the
238-rule normative inventory pass. Retained evidence binds actual input/output,
status, replacements, full rollback, live QEMU and final serial quiescence.

## Risk review and remaining work

Risk is high because selector normalization and fallback placement can misroute
calls or strand a live context. Self-review checks all dispatch precedence,
unknown versus implemented native widths, ABI-tag separation, exact negative
result, unchanged phase/resources, register restoration/erasure and terminal
lifetime. Independent encoders and unmodified standard applications supplement
native source review. Human PR review and merge remain required.

The [design](design/static-syscall-fallback.md) preserves the portable distinction
between an unimplemented request and a successful service. AArch64 and RISC-V
need their own selector and return adapters. General executable selection,
dynamic ELF, fork/wait, writable storage, networking, distribution boot,
Docker/Podman and VM hosting remain required initial-plan work. Recoverable
failure does not establish implementation of those interfaces.
