# Reusable static-process signal registration

Status: approved systems-feature direction under the standing authorization;
implemented and qualified (`TK-DEC-241`), as recorded in the
[Step 4 report](../step-4-reusable-static-file-descriptions.md).

## Purpose

The file-consuming `dd` application installs a `SIGUSR1` action during setup.
The existing process adapter recognizes a few shell-specific action requests.
This refinement gives the live process shared action-registration, query and
blocked-mask services. Its state belongs to the process independently of file
descriptions and may occupy disjoint storage in the same private process page.
It is part of the current file-consumer qualification dependency, with its own
formal rules and error/exec tests. It does not establish asynchronous delivery,
pending-signal queues, signal frames, restart or `rt_sigreturn` support.

## State and operations

There is one action per native signal and a blocked-signal set. The adapter
copies a complete input action before validating its signal-specific request,
normalizes native flags and unmaskable signals according to the pinned ABI,
captures the old action, commits a valid update, and then publishes any requested
old value. Input and output may overlap. A bad input cannot commit; a bad output
can report `EFAULT` after a valid update has committed. The implementation must
preserve that distinction rather than treating every error as a rollback.

For the initial x86-64 LP64 binding, signals are numbered one through 64, native
actions contain four eight-byte fields, and the native signal-set size is eight.
Installing an action for `SIGKILL` or `SIGSTOP` is invalid; querying their default
actions is allowed. Unknown action flags are cleared using the baseline UAPI
mask. The native mask excludes `SIGKILL` and `SIGSTOP`. Handler and restorer
fields are opaque ABI values, not Topal references or privileged callable
capabilities. Registration does not dereference them or compare them with an
application's handler address. Any future delivery must validate the user
address-space transition and construct a qualified signal frame separately.

Blocked-mask operations support block, unblock and set. The input is copied
and filtered before an update is committed. The selector is validated only
when a new set is supplied; a pure query ignores it. The old value is captured
before an update, and an output-copy error does not undo a committed update.
Qualification is for complete admitted transfers; partial user-copy and signal
delivery semantics remain explicit later conformance work.

## Lifetime and portability

Construction imports the already qualified prelude's action facts and consumes
their former private representation. Subsequent requests use the shared state
without a shell phase, applet name, fixed return site or chosen handler oracle.
Failed exec preserves actions and the blocked set. Successful exec preserves
only ignored handler dispositions, resets other handlers to default, and clears
every action's flags, restorer and action mask. The blocked set survives exec.
Every process exit status wipes and verifies the complete private state before
outer reclamation. No action may remain available through a destroyed image.

The portable model is a process-owned action map and blocked set. Signal numbers,
unmaskable-signal identities, field widths, flags, restorer conventions and copy
adapters stay behind the architecture's Linux ABI provider. No source assembly,
core-language aliasing, reference conversion or ordinary Topal signal effect is
introduced.

Risk is high because signal state crosses image lifetime and user-copy commit
boundaries. Review must check overlaps, input/error precedence, flag/mask
normalization, state after output failure, ignored-action exec behavior,
blocked-mask persistence, no authority from raw handler values and total erasure.
Clean tests must exercise these properties before the external `dd` case can
qualify the shared service. Per-handler exceptions and synthesized successes
would not establish registration semantics and are rejected.

The primary reference is pinned Linux 7.2.9 `kernel/signal.c`:
`rt_sigaction`, `do_sigaction`, `rt_sigprocmask` and `flush_signal_handlers`,
with `include/linux/signal_types.h` and the x86 UAPI signal definitions.
In particular, `flush_signal_handlers` clears metadata even for ignored actions;
the ignored *handler* alone survives exec. See [the baseline](../baseline.md).
